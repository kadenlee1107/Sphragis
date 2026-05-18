// src/fs/sealfs_rotation.rs — SealFS master-key rotation primitive.
//
// Production-hardening capability #1 (2026-05-17, Eng-2 push). The
// SealFS master key in `sealfs.rs::MASTER_KEY` is, before this
// module lands, a one-shot value: written once at `init`, read on
// every read/write, never replaced. That is the right shape for
// the "fresh disk each boot" RAM-only mode but it leaves a gap for
// the operator who suspects the in-RAM key has leaked (cold-boot
// snoop, panic-mid-wipe race, etc.) and wants to rotate without
// reformatting.
//
// ─── Rotation semantics ─────────────────────────────────────────
//
// A rotation event happens entirely in RAM. We keep a small
// fixed-capacity key-history ring (`KEY_HISTORY`) of recently-
// retired master keys. Per-file decryption tries the CURRENT
// master key first, then walks history youngest→oldest until an
// AEAD verification succeeds or the history is exhausted.
//
// New writes (`create`) always use the CURRENT master key. As
// retired files are accessed and rotated forward (re-encrypted),
// the history loses references; once no file in `FILES[]` was
// last touched under a history entry, that entry can be retired.
// This module exposes the primitives — the policy of WHEN to
// retire is left to the operator (see `rotate_master_key` below).
//
// ─── Threat model ───────────────────────────────────────────────
//
// Defends against:
//   * In-RAM master-key leak (operator suspects exposure; rotates
//     to invalidate the leaked value for FUTURE writes).
//   * Forward-secrecy for files that get re-written post-rotation:
//     the old key only protects the on-disk ciphertext from
//     before rotation; new writes use the new key.
//
// Does NOT defend against:
//   * An attacker who already captured pre-rotation ciphertext +
//     the pre-rotation key. Rotation has no retroactive effect.
//   * Stored-ciphertext attacks if the disk is exfiltrated AND
//     the attacker subsequently obtains a rotated-from key.
//
// ─── On-disk format ─────────────────────────────────────────────
//
// Unchanged. Rotation lives in RAM. A future production iteration
// would persist the key-history slot under SEP-sealed storage (or
// re-encrypt all files under the new key as a background pass)
// and bump SB_VERSION accordingly. The migration-history pattern
// in `sealfs_disk.rs` documents that flow.

#![allow(dead_code)]

use crate::crypto::sha256;
use crate::kernel::sync::IrqGuard;
use crate::security::zeroize::zeroize;

/// Maximum number of retired master keys retained in history.
/// Sized for "a handful of rotations per boot" — beyond that the
/// operator should reformat. Each slot is 32 bytes; 8 slots =
/// 256 bytes static.
pub const KEY_HISTORY_CAP: usize = 8;

/// Logical generation counter. Generation 0 is the boot-time
/// master key. Each `rotate_master_key` call increments. Used by
/// the audit log to record "rotated from gen N to gen N+1".
pub type KeyGen = u32;

/// One entry in the retired-keys ring.
#[derive(Clone, Copy)]
pub struct KeyHistorySlot {
    /// Generation of the key this slot retains. `gen + 1` is the
    /// generation that replaced it.
    pub generation: KeyGen,
    /// The retired 32-byte master key. Zero when `occupied = false`.
    pub key: [u8; 32],
    /// `true` when this slot retains an actual retired key.
    pub occupied: bool,
}

impl KeyHistorySlot {
    pub const fn empty() -> Self {
        Self {
            generation: 0,
            key: [0u8; 32],
            occupied: false,
        }
    }
}

/// Current generation. Starts at 0; incremented by
/// `rotate_master_key`.
static CURRENT_GEN: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Retired-key ring. `write_idx` advances modulo `KEY_HISTORY_CAP`.
/// Overflow silently overwrites the oldest entry — at that point
/// any files still encrypted under that key become permanently
/// unrecoverable, which is acceptable: it means the operator has
/// rotated `KEY_HISTORY_CAP + 1` times without re-encrypting the
/// affected files, which is operationally a "key escrow lost"
/// situation regardless.
static mut KEY_HISTORY: [KeyHistorySlot; KEY_HISTORY_CAP] =
    [KeyHistorySlot::empty(); KEY_HISTORY_CAP];
static WRITE_IDX: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// Current generation number. Useful for the audit log entry.
pub fn current_generation() -> KeyGen {
    CURRENT_GEN.load(core::sync::atomic::Ordering::Acquire)
}

/// Reset rotation state to a fresh post-init shape. Called by
/// `sealfs::init` once after the master key is installed, and by
/// `sealfs::wipe_master_key` so a subsequent re-init starts clean.
/// Zeroes every history slot's key material with `zeroize` so the
/// retired keys cannot be recovered from RAM after reset.
pub fn reset() {
    let _g = IrqGuard::new();
    CURRENT_GEN.store(0, core::sync::atomic::Ordering::Release);
    WRITE_IDX.store(0, core::sync::atomic::Ordering::Release);
    unsafe {
        let ptr = core::ptr::addr_of_mut!(KEY_HISTORY);
        for i in 0..KEY_HISTORY_CAP {
            zeroize(&mut (*ptr)[i].key);
            (*ptr)[i].generation = 0;
            (*ptr)[i].occupied = false;
        }
    }
}

/// Retire the supplied key into the history ring under generation
/// `gen`, then bump `CURRENT_GEN` to `gen + 1`. The caller (the
/// `rotate_master_key` entry below) is responsible for swapping
/// the actual `MASTER_KEY` static in `sealfs.rs` and for emitting
/// the audit-log RotationEvent.
fn record_retired_key(retired: &[u8; 32], retired_gen: KeyGen) {
    let _g = IrqGuard::new();
    let idx = WRITE_IDX.fetch_add(1, core::sync::atomic::Ordering::AcqRel) % KEY_HISTORY_CAP;
    unsafe {
        let ptr = core::ptr::addr_of_mut!(KEY_HISTORY);
        // If the slot was occupied (ring wrap), zeroize first.
        if (*ptr)[idx].occupied {
            zeroize(&mut (*ptr)[idx].key);
        }
        (*ptr)[idx].key = *retired;
        (*ptr)[idx].generation = retired_gen;
        (*ptr)[idx].occupied = true;
    }
    CURRENT_GEN.store(
        retired_gen.wrapping_add(1),
        core::sync::atomic::Ordering::Release,
    );
}

/// Iterate retired-key slots youngest-first. Caller passes a
/// closure that takes `&[u8; 32]` (retired master key) and
/// `KeyGen` and returns `Some(R)` if it has succeeded with that
/// key (e.g. AEAD verify succeeded), `None` to keep walking.
///
/// Stops at the first `Some` return. Returns `None` if every
/// occupied slot returned `None`.
pub fn try_each_retired_key<R, F>(mut f: F) -> Option<R>
where
    F: FnMut(&[u8; 32], KeyGen) -> Option<R>,
{
    let _g = IrqGuard::new();
    // Walk from the slot one behind WRITE_IDX (the youngest
    // retired entry) backwards.
    let write_idx = WRITE_IDX.load(core::sync::atomic::Ordering::Acquire);
    if write_idx == 0 {
        return None;
    }
    let mut walked: [(usize, KeyGen, [u8; 32]); KEY_HISTORY_CAP] = unsafe {
        let ptr = core::ptr::addr_of!(KEY_HISTORY);
        let mut tmp = [(0usize, 0u32, [0u8; 32]); KEY_HISTORY_CAP];
        for i in 0..KEY_HISTORY_CAP {
            tmp[i] = (i, (*ptr)[i].generation, (*ptr)[i].key);
        }
        tmp
    };
    // Sort by generation descending (youngest retired first).
    // Generation is monotonically increasing with WRITE_IDX, so
    // a simple insertion sort over 8 slots is fine.
    for i in 1..KEY_HISTORY_CAP {
        let mut j = i;
        while j > 0 && walked[j - 1].1 < walked[j].1 {
            walked.swap(j - 1, j);
            j -= 1;
        }
    }
    for (slot_idx, kgen, key) in walked.iter() {
        let occupied = unsafe { (*core::ptr::addr_of!(KEY_HISTORY))[*slot_idx].occupied };
        if !occupied {
            continue;
        }
        if let Some(r) = f(key, *kgen) {
            return Some(r);
        }
    }
    None
}

/// Number of currently-occupied history slots. Used by selftests
/// to verify the ring state after a rotation.
pub fn occupied_slots() -> usize {
    let _g = IrqGuard::new();
    let mut n = 0;
    unsafe {
        let ptr = core::ptr::addr_of!(KEY_HISTORY);
        for i in 0..KEY_HISTORY_CAP {
            if (*ptr)[i].occupied {
                n += 1;
            }
        }
    }
    n
}

/// Master-key rotation entry point.
///
/// Steps:
/// 1. Snapshot the OLD master key from `sealfs::MASTER_KEY` and
///    push it into the history ring under the current generation
///    (the generation that the old key represents).
/// 2. Bump `CURRENT_GEN`.
/// 3. Install `new_master_key` as the live MASTER_KEY in
///    `sealfs.rs` via the supplied callback.
/// 4. Caller is responsible for emitting the audit-log
///    RotationEvent (so this module stays decoupled from the
///    audit module and can be unit-tested in isolation).
///
/// The split via callback lets `sealfs.rs` keep `MASTER_KEY`
/// private (it stays `static mut` inside the parent module) while
/// this module owns the history-ring discipline.
///
/// Returns `(old_gen, new_gen)` so the caller can record them in
/// the audit log.
pub fn rotate<F: FnOnce(&mut [u8; 32]) -> [u8; 32]>(install_new: F) -> (KeyGen, KeyGen) {
    let _g = IrqGuard::new();
    let old_gen = CURRENT_GEN.load(core::sync::atomic::Ordering::Acquire);
    // Caller-supplied closure swaps the live MASTER_KEY for the
    // new value and returns the old value so we can push it into
    // history. Doing it this way means the live key is never
    // observable in two places at once — the closure runs under
    // our IrqGuard.
    let mut tmp = [0u8; 32];
    let retired = install_new(&mut tmp);
    // tmp is unused on this path — the closure returns the old
    // key directly. Zero it to be safe in case the closure scheme
    // ever changes.
    zeroize(&mut tmp);
    record_retired_key(&retired, old_gen);
    // record_retired_key bumped CURRENT_GEN already.
    let new_gen = CURRENT_GEN.load(core::sync::atomic::Ordering::Acquire);
    (old_gen, new_gen)
}

/// Helper used by `sealfs::read` to derive a per-file key from a
/// candidate master key. Mirrors `sealfs::derive_file_key` exactly
/// — we can't call that function because it always reads the LIVE
/// `MASTER_KEY` static. This is the same SHA-256 KDF.
pub fn derive_file_key_from(master: &[u8; 32], filename: &str) -> [u8; 32] {
    sha256::derive_key(master, filename.as_bytes())
}

// ─── Compile-time-validated tests ────────────────────────────────
//
// `cargo test --workspace` cannot run these on aarch64-unknown-none
// (no `test` lang item), so they exist as a structural sanity
// check that compiles under host-cfg. The real test loop lives in
// the `sealfs-rotation-selftest` boot/shell hook driven by
// scripts/qemu_sealfs_rotation_selftest.py.

#[cfg(test)]
mod tests {
    use super::*;

    fn test_reset() {
        reset();
    }

    #[test]
    fn fresh_state_has_no_history() {
        test_reset();
        assert_eq!(current_generation(), 0);
        assert_eq!(occupied_slots(), 0);
    }

    #[test]
    fn rotation_records_retired_key_and_bumps_generation() {
        test_reset();
        let mut live = [0xAAu8; 32];
        let new_key = [0xBBu8; 32];
        let (old_gen, new_gen) = rotate(|tmp| {
            let _ = tmp;
            let old = live;
            live = new_key;
            old
        });
        assert_eq!(old_gen, 0);
        assert_eq!(new_gen, 1);
        assert_eq!(current_generation(), 1);
        assert_eq!(occupied_slots(), 1);
        // try_each_retired_key should hand us back the retired key.
        let found = try_each_retired_key(|k, kgen| {
            if *k == [0xAAu8; 32] && kgen == 0 {
                Some(())
            } else {
                None
            }
        });
        assert!(found.is_some());
    }

    #[test]
    fn ring_wraps_after_history_cap_rotations() {
        test_reset();
        for i in 0..(KEY_HISTORY_CAP + 2) {
            let _ = rotate(|_tmp| {
                let mut k = [0u8; 32];
                k[0] = i as u8;
                k
            });
        }
        // After KEY_HISTORY_CAP + 2 rotations, history is full
        // (capped) and CURRENT_GEN == KEY_HISTORY_CAP + 2.
        assert_eq!(occupied_slots(), KEY_HISTORY_CAP);
        assert_eq!(current_generation() as usize, KEY_HISTORY_CAP + 2);
    }
}
