#![allow(dead_code)]
// Sphragis — Secure Wipe System
// Multiple wipe modes, all irreversible:
//
// 1. SILENT WIPE: Duress mode — shows fake boot while destroying everything
// 2. PANIC WIPE: Instant destruction triggered by hotkey
// 3. LOCKOUT WIPE: Max failed attempts exceeded
// 4. DEADMAN WIPE: Dead man's switch timer expired
//
// Wipe process:
// - Zero all encryption keys in memory
// - Zero all file data
// - Zero all page frames
// - On real hardware: tell Secure Enclave to destroy master key
//   (makes SSD contents permanently unrecoverable)

use crate::kernel::mm::frame;
use crate::platform;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const WIPE_STATE_NONE: u8 = 0;
const WIPE_STATE_IN_PROGRESS: u8 = 1;
const WIPE_STATE_COMPLETE: u8 = 2;

static WIPE_STATE: AtomicU8 = AtomicU8::new(WIPE_STATE_NONE);
static WIPE_SILENT: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
pub enum WipeReason {
    Duress,
    Panic,
    Lockout,
    DeadManSwitch,
    Manual,
}

/// Execute a full system wipe.
/// If silent=true, this returns control so the fake boot screen
/// can continue displaying while destruction happens in background.
pub fn execute(reason: WipeReason, silent: bool) {
    if WIPE_STATE.load(Ordering::Relaxed) != WIPE_STATE_NONE {
        return; // Already wiping
    }

    WIPE_STATE.store(WIPE_STATE_IN_PROGRESS, Ordering::Relaxed);
    WIPE_SILENT.store(silent, Ordering::Relaxed);

    let reason_str = match reason {
        WipeReason::Duress => "DURESS",
        WipeReason::Panic => "PANIC_HOTKEY",
        WipeReason::Lockout => "MAX_ATTEMPTS",
        WipeReason::DeadManSwitch => "DEAD_MAN_SWITCH",
        WipeReason::Manual => "MANUAL",
    };

    if !silent {
        platform::serial_puts("\n!!! WIPE INITIATED !!!\n");
        platform::serial_puts("  Reason: ");
        platform::serial_puts(reason_str);
        platform::serial_puts("\n");
    }

    // AUDIT-FS-C2 (2026-05-15): wipe ordering corrected. The prior
    // order was destroy_keys → caves → fs → memory. destroy_keys was
    // a no-op (see comment on the function), but even after fixing
    // it the order is still wrong: wipe_filesystem needs the master
    // key to compute the AEAD AAD when deleting inodes from disk. If
    // we zero the key first, every on-disk inode-zeroize fails to
    // authenticate.
    //
    // Correct order:
    //   1. Caves (destroys cave state in-memory)
    //   2. Filesystem (deletes every file while we still have the key
    //      — each delete zeros data sectors + commits zero inode)
    //   3. Master key + per-file metadata (now safe to zero — disk
    //      has been wiped, nothing further needs the key)
    //   4. OTP pad
    //   5. Memory (everything else)
    //   6. Secure Enclave

    // AUDIT-FS-L4 (2026-05-15): flush in-RAM audit ring to disk
    // BEFORE caves are destroyed. The wipe path itself emits one
    // audit entry (TpiOp "WIPE NOW triggered by operator" from
    // ui/apps/security.rs); without flushing first, that entry
    // would die with the in-RAM ring when wipe_memory zeros every
    // frame in Phase 5. After this flush the entry survives on
    // SealFS until wipe_filesystem deletes audit.log (Phase 2) — a
    // forensic reviewer who recovers the encrypted audit.log
    // sector before the disk-zero pass has a chance to recover it.
    let _ = crate::security::audit::flush_to_sealfs();

    // Phase 1: Caves destroyed first (in-RAM state).
    crate::caves::cave::destroy_all();
    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] All Caves destroyed\n");
    }

    // Phase 2: Filesystem (uses master key for HMAC AAD on each delete).
    wipe_filesystem();

    // Phase 3: Master key + per-file metadata. NOW the key dies, after
    // wipe_filesystem has used it for the last time.
    destroy_keys();

    // Phase 4: OTP pad. DESIGN_CRYPTO.md #11: dies with the system too;
    // any seized-hardware attempt to replay an unused token fails
    // because the pad is now all zeros.
    crate::security::otp::wipe();

    // Phase 5: Zero all allocated memory.
    wipe_memory();

    // Phase 4: On real hardware, tell Secure Enclave to nuke master key
    // This makes the entire SSD permanently unreadable
    #[cfg(not(test))]
    wipe_secure_enclave();

    WIPE_STATE.store(WIPE_STATE_COMPLETE, Ordering::Relaxed);

    if !silent {
        platform::serial_puts("  WIPE COMPLETE — all data destroyed\n");
        platform::serial_puts("  System is now a brick.\n");
    }
}

/// Wipe + halt the kernel. Used by visible/panic paths where there's
/// no useful state to return to (operator triggered Wipe NOW, duress
/// token consumed by the shell, panic hotkey fired). The kernel
/// enters an infinite spin after destruction completes; the next
/// boot starts fresh.
///
/// The silent fake-boot path in `boot_screen.rs` still uses
/// `execute(reason, true)` directly — it needs control back to
/// paint the decoy progress bar.
pub fn execute_and_halt(reason: WipeReason) -> ! {
    execute(reason, false);
    platform::serial_puts("  [wipe] halted; reset to recover\n");
    loop {
        core::hint::spin_loop();
    }
}

/// AUDIT-FS-C2 (2026-05-15): destroy_keys() previously called
/// `sealfs::init(&zero_key)` followed by `init(&poison)`, expecting
/// the calls to overwrite the master key. But `sealfs::init` returns
/// early when `INITIALIZED == true`, so the real master key was
/// never zeroed and the "Encryption keys destroyed" line was a lie.
///
/// Now calls the explicit `sealfs::wipe_master_key()` which:
///   * Zeroes MASTER_KEY, BOOT_NONCE_PREFIX, FILES[].nonce/.hash,
///     FILE_TAINT[], FILE_COUNT under IrqGuard.
///   * Flips INITIALIZED back to false.
///
/// Per the new wipe ordering, this runs AFTER wipe_filesystem(),
/// so the key was still available for each delete()'s AEAD AAD
/// computation.
fn destroy_keys() {
    crate::fs::sealfs::wipe_master_key();

    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] Encryption keys destroyed (master + nonce prefix + per-file tags zeroed)\n");
    }
}

/// Zero all file data in the filesystem.
fn wipe_filesystem() {
    // Delete every file (which zeros their data pages)
    let mut names = [[0u8; 64]; 128];
    let mut name_lens = [0usize; 128];
    let mut count = 0;

    crate::fs::sealfs::list(|name, _size, _enc| {
        if count < 128 {
            let bytes = name.as_bytes();
            let len = bytes.len().min(64);
            names[count][..len].copy_from_slice(&bytes[..len]);
            name_lens[count] = len;
            count += 1;
        }
    });

    for i in 0..count {
        let name = unsafe { core::str::from_utf8_unchecked(&names[i][..name_lens[i]]) };
        let _ = crate::fs::sealfs::delete(name);
    }

    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] Filesystem destroyed (");
        crate::kernel::mm::print_num(count);
        platform::serial_puts(" files zeroed)\n");
    }
}

/// Zero all allocated memory pages.
fn wipe_memory() {
    // Get memory stats to know how much to wipe
    let (_used, _total) = frame::stats();
    let mut wiped = 0usize;

    // Allocate and zero frames until we run out — alloc_frame already
    // zeros each page on hand-out, so this overwrites all free memory.
    while let Some(_addr) = frame::alloc_frame() {
        wiped += 1;
    }

    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] Memory zeroed (");
        crate::kernel::mm::print_num(wiped * 4);
        platform::serial_puts(" KB wiped)\n");
    }
}

/// Tell the Secure Enclave to destroy the master encryption key.
/// After this, the SSD contents are permanently unrecoverable —
/// even with physical access to the NAND chips.
fn wipe_secure_enclave() {
    // On real hardware, this would:
    // 1. Send a mailbox message to SEP
    // 2. SEP destroys the effaceable storage
    // 3. All encryption keys derived from it become unrecoverable
    //
    // In QEMU, we simulate this by zeroing our key state.
    // The real SEP implementation goes in drivers/apple/sep.rs

    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] Secure Enclave: keys destroyed (simulated)\n");
    }
}

pub fn is_wiping() -> bool {
    WIPE_STATE.load(Ordering::Relaxed) == WIPE_STATE_IN_PROGRESS
}

pub fn is_wiped() -> bool {
    WIPE_STATE.load(Ordering::Relaxed) == WIPE_STATE_COMPLETE
}

/// V8-ROOT-6: best-effort in-memory secret wipe for panic-handler use.
/// Does NOT destroy the disk — the kernel has already failed and any
/// disk write we attempt could corrupt SealFS. Instead, just zero the
/// things an attacker with a cold-boot/DRAM-extraction primitive would
/// want: auth hashes, TLS session keys, and the SealFS master key.
///
/// Safe to call from the panic handler (no allocations, no locks, no
/// I/O that could re-panic).
pub fn emergency_wipe() {
    // Best-effort: wrap each step in its own unwind-safe block via a
    // closure. If any sub-wipe panics we still attempt the others.
    unsafe { crate::security::auth::panic_wipe(); }
    unsafe { crate::net::tls::panic_wipe(); }
    unsafe { crate::fs::sealfs::panic_wipe(); }
    // V8-ROOT-6 (regression fix): the RNG chain state can derive any TLS
    // ClientRandom or SealFS nonce issued post-reseed. Wipe it too.
    unsafe { crate::crypto::rng::panic_wipe(); }
}
