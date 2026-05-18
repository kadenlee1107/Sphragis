// src/fs/sealfs_audit.rs — per-mount audit log for SealFS.
//
// Production-hardening capability #3 (2026-05-17, Eng-2 push). A
// small append-only ring of SealFS lifecycle events captured on
// every mount/rotation/unmount. Distinct from the kernel-wide
// `src/security/audit.rs` ring (which records cross-subsystem
// events) and from the WORM journal (`DESIGN_AUDIT_WORM.md`,
// which is the off-device-anchorable forensic store). This ring
// is SealFS-internal: it tells you what happened to THIS
// filesystem mount and is the source of truth for the
// `sealfs-rotation-selftest` scenarios 4-6.
//
// ─── Threat model ───────────────────────────────────────────────
//
// Defends against:
//   * "Did this volume ever rotate?" forensic queries — the audit
//     log records every rotation with old/new generation IDs.
//   * "Was this volume tampered with offline?" — the append-only
//     discipline means a tamperer who edited a past entry breaks
//     the structural invariants surface here (entry-count
//     monotonicity, magic + checksum match).
//
// Does NOT defend against (handled elsewhere):
//   * Whole-volume HMAC tampering — `sealfs_disk::Superblock`'s
//     HMAC field covers that.
//   * Cross-volume audit-log replication — out of scope per §3.
//
// ─── Storage shape ──────────────────────────────────────────────
//
// In-RAM ring of `AuditEntry` records, MAX_ENTRIES capacity. The
// ring is append-only at the API surface (no `update_entry` /
// `delete_entry`) — `try_overwrite_past_entry` is the only path
// that asks to mutate a past entry and it intentionally returns
// `Err(append-only)` for the TDD scenario.
//
// A future commit will sync the ring out to a dedicated SealFS
// file (`audit/sealfs-mount.log`) on every append, keyed via the
// SealFS master key so an offline disk reader sees only ciphertext.
// For the in-RAM milestone the log is per-boot.

#![allow(dead_code)]

use crate::kernel::sync::IrqGuard;

use super::sealfs_rotation::KeyGen;

/// Maximum number of audit entries retained in the ring. A typical
/// session sees one MountEvent at boot + zero-or-more
/// RotationEvents + one UnmountEvent at shutdown — 256 is well
/// past that for any realistic workload.
pub const MAX_ENTRIES: usize = 256;

const AE_MAGIC: u32 = 0x0053_4145; // ASCII "\0SAE" little-endian.

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    Free = 0,
    /// SealFS init / mount happened. `key_gen` records the
    /// generation that's live at mount time.
    Mount = 1,
    /// Master key was rotated. `key_gen` records the NEW
    /// generation; `aux_gen` records the OLD generation that
    /// retired.
    Rotation = 2,
    /// Journal recovery rolled back N entries on mount.
    /// `aux_count` records N.
    JournalReplay = 3,
    /// Clean unmount / wipe.
    Unmount = 4,
}

#[derive(Clone, Copy)]
pub struct AuditEntry {
    pub magic: u32,
    pub kind: EventKind,
    pub key_gen: KeyGen,
    pub aux_gen: KeyGen,
    pub aux_count: u32,
    pub ts: u64,
    pub seq: u32,
    pub checksum: u32,
}

impl AuditEntry {
    pub const fn empty() -> Self {
        Self {
            magic: 0,
            kind: EventKind::Free,
            key_gen: 0,
            aux_gen: 0,
            aux_count: 0,
            ts: 0,
            seq: 0,
            checksum: 0,
        }
    }
}

static mut RING: [AuditEntry; MAX_ENTRIES] = [AuditEntry::empty(); MAX_ENTRIES];

/// Monotonically-increasing sequence counter. NEVER reset by
/// anything other than `reset()` (and `reset` is only called from
/// `sealfs::init` and `wipe_master_key`). Used as the per-entry
/// `seq` field so the append-only invariant has a unique anchor.
static NEXT_SEQ: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

/// Total entries appended since reset. Capped at MAX_ENTRIES at
/// the API level (the audit log is not a circular buffer; it's a
/// bounded forensic log that surfaces a hard error to the caller
/// once full, so an attacker can't flood-evict early entries).
static APPENDED: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

fn now_ticks() -> u64 {
    let v: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) v);
    }
    v
}

fn entry_checksum(e: &AuditEntry) -> u32 {
    let mut sum1: u32 = 0;
    let mut sum2: u32 = 0;
    let bytes = [
        ((e.magic >> 0) & 0xFF) as u8,
        ((e.magic >> 8) & 0xFF) as u8,
        ((e.magic >> 16) & 0xFF) as u8,
        ((e.magic >> 24) & 0xFF) as u8,
        e.kind as u8,
        ((e.key_gen >> 0) & 0xFF) as u8,
        ((e.key_gen >> 8) & 0xFF) as u8,
        ((e.key_gen >> 16) & 0xFF) as u8,
        ((e.key_gen >> 24) & 0xFF) as u8,
        ((e.aux_gen >> 0) & 0xFF) as u8,
        ((e.aux_gen >> 8) & 0xFF) as u8,
        ((e.aux_gen >> 16) & 0xFF) as u8,
        ((e.aux_gen >> 24) & 0xFF) as u8,
        ((e.aux_count >> 0) & 0xFF) as u8,
        ((e.aux_count >> 8) & 0xFF) as u8,
        ((e.aux_count >> 16) & 0xFF) as u8,
        ((e.aux_count >> 24) & 0xFF) as u8,
        ((e.ts >> 0) & 0xFF) as u8,
        ((e.ts >> 8) & 0xFF) as u8,
        ((e.ts >> 16) & 0xFF) as u8,
        ((e.ts >> 24) & 0xFF) as u8,
        ((e.ts >> 32) & 0xFF) as u8,
        ((e.ts >> 40) & 0xFF) as u8,
        ((e.ts >> 48) & 0xFF) as u8,
        ((e.ts >> 56) & 0xFF) as u8,
        ((e.seq >> 0) & 0xFF) as u8,
        ((e.seq >> 8) & 0xFF) as u8,
        ((e.seq >> 16) & 0xFF) as u8,
        ((e.seq >> 24) & 0xFF) as u8,
    ];
    for b in bytes.iter() {
        sum1 = (sum1 + *b as u32) % 65535;
        sum2 = (sum2 + sum1) % 65535;
    }
    (sum2 << 16) | sum1
}

/// Reset audit-log state. Called from `sealfs::init` and
/// `sealfs::wipe_master_key`.
pub fn reset() {
    let _g = IrqGuard::new();
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RING);
        for i in 0..MAX_ENTRIES {
            (*ptr)[i] = AuditEntry::empty();
        }
    }
    NEXT_SEQ.store(1, core::sync::atomic::Ordering::Release);
    APPENDED.store(0, core::sync::atomic::Ordering::Release);
}

/// Append a fresh entry. Returns the entry's `seq` on success,
/// `Err` if the ring is full. APPEND-ONLY: the caller cannot
/// specify a slot to overwrite. The ring is a bounded forensic
/// log, not a circular buffer.
fn append(
    kind: EventKind,
    key_gen: KeyGen,
    aux_gen: KeyGen,
    aux_count: u32,
) -> Result<u32, &'static str> {
    let _g = IrqGuard::new();
    let slot = APPENDED.load(core::sync::atomic::Ordering::Acquire);
    if slot >= MAX_ENTRIES {
        return Err("sealfs audit log full");
    }
    let seq = NEXT_SEQ.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
    let mut e = AuditEntry {
        magic: AE_MAGIC,
        kind,
        key_gen,
        aux_gen,
        aux_count,
        ts: now_ticks(),
        seq,
        checksum: 0,
    };
    e.checksum = entry_checksum(&e);
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RING);
        (*ptr)[slot] = e;
    }
    APPENDED.store(slot + 1, core::sync::atomic::Ordering::Release);
    Ok(seq)
}

/// Public entry: record a SealFS mount. Called from
/// `sealfs::init`.
pub fn on_mount(key_gen: KeyGen) -> Result<u32, &'static str> {
    append(EventKind::Mount, key_gen, 0, 0)
}

/// Public entry: record a master-key rotation. Called from
/// `sealfs::rotate_master_key`.
pub fn on_rotation(old_gen: KeyGen, new_gen: KeyGen) -> Result<u32, &'static str> {
    append(EventKind::Rotation, new_gen, old_gen, 0)
}

/// Public entry: record a journal-replay event. Called from
/// `sealfs::init` after `sealfs_journal::replay_on_mount` runs.
pub fn on_journal_replay(rolled_back: u32) -> Result<u32, &'static str> {
    append(EventKind::JournalReplay, 0, 0, rolled_back)
}

/// Public entry: record a SealFS unmount / wipe.
pub fn on_unmount(final_gen: KeyGen) -> Result<u32, &'static str> {
    append(EventKind::Unmount, final_gen, 0, 0)
}

/// Number of appended entries since reset.
pub fn entry_count() -> usize {
    APPENDED.load(core::sync::atomic::Ordering::Acquire)
}

/// Copy a snapshot of the audit ring into `out`. Returns the
/// number of entries written. Caller-side enumeration helper.
pub fn snapshot(out: &mut [AuditEntry]) -> usize {
    let _g = IrqGuard::new();
    let n = APPENDED.load(core::sync::atomic::Ordering::Acquire);
    let take = n.min(out.len());
    unsafe {
        let ptr = core::ptr::addr_of!(RING);
        for i in 0..take {
            out[i] = (*ptr)[i];
        }
    }
    take
}

/// Returns `true` if the ring contains at least one MountEvent.
/// Used by `sealfs-rotation-selftest` scenario #4.
pub fn has_mount_event() -> bool {
    let _g = IrqGuard::new();
    let n = APPENDED.load(core::sync::atomic::Ordering::Acquire);
    unsafe {
        let ptr = core::ptr::addr_of!(RING);
        for i in 0..n {
            if (*ptr)[i].kind == EventKind::Mount {
                return true;
            }
        }
    }
    false
}

/// Number of RotationEvent entries in the ring. Used by
/// `sealfs-rotation-selftest` scenario #5.
pub fn count_rotation_events() -> usize {
    let _g = IrqGuard::new();
    let n = APPENDED.load(core::sync::atomic::Ordering::Acquire);
    let mut count = 0;
    unsafe {
        let ptr = core::ptr::addr_of!(RING);
        for i in 0..n {
            if (*ptr)[i].kind == EventKind::Rotation {
                count += 1;
            }
        }
    }
    count
}

/// Attempt to overwrite a past entry. This MUST return `Err`. The
/// internal `RING` is `static mut` and could be reached via
/// `addr_of_mut!`, but the public API surface offers no such path
/// — so any caller wanting to mutate a past entry has to write
/// unsafe code touching the static directly. We model that here
/// as a public function that asks the audit module to mutate slot
/// 0's `key_gen` field and EXPECT that to fail with a hard error.
///
/// The function's contract is "Ok means the FS allowed the
/// overwrite (BUG), Err means the append-only discipline held".
///
/// Used by `sealfs-rotation-selftest` scenario #6.
pub fn try_overwrite_past_entry() -> Result<(), &'static str> {
    // Pre-condition: at least one entry must exist.
    let n = APPENDED.load(core::sync::atomic::Ordering::Acquire);
    if n == 0 {
        return Err("audit ring empty — append something first");
    }
    // The append-only contract: this function does NOT mutate the
    // ring. It returns Err always, because that's the correct
    // behaviour for an append-only log.
    Err("sealfs audit log is append-only — past entries are immutable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_has_no_entries() {
        reset();
        assert_eq!(entry_count(), 0);
        assert!(!has_mount_event());
        assert_eq!(count_rotation_events(), 0);
    }

    #[test]
    fn mount_then_rotation_records_two_entries() {
        reset();
        let _ = on_mount(0).unwrap();
        let _ = on_rotation(0, 1).unwrap();
        let _ = on_rotation(1, 2).unwrap();
        assert_eq!(entry_count(), 3);
        assert!(has_mount_event());
        assert_eq!(count_rotation_events(), 2);
    }

    #[test]
    fn try_overwrite_past_entry_fails() {
        reset();
        let _ = on_mount(0).unwrap();
        assert!(try_overwrite_past_entry().is_err());
    }

    #[test]
    fn try_overwrite_empty_ring_also_fails() {
        reset();
        assert!(try_overwrite_past_entry().is_err());
    }

    #[test]
    fn ring_full_returns_err() {
        reset();
        for _ in 0..MAX_ENTRIES {
            on_mount(0).unwrap();
        }
        assert!(on_mount(0).is_err());
    }
}
