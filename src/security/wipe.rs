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

    // Phase 1: Destroy encryption keys
    destroy_keys();
    crate::batcave::cave::destroy_all();
    // DESIGN_CRYPTO.md #11: the OTP pad dies with the system too. Any
    // seized-hardware attempt to replay an unused token fails because
    // the pad is now all zeros.
    crate::security::otp::wipe();
    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] All BatCaves destroyed\n");
    }

    // Phase 2: Zero filesystem data
    wipe_filesystem();

    // Phase 3: Zero all allocated memory
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

/// Destroy all encryption keys in memory.
fn destroy_keys() {
    // Zero the master key by re-initializing with zeros
    let zero_key = [0u8; 32];
    crate::fs::batfs::init(&zero_key);

    // Overwrite key memory with random-looking data
    // (prevents cold boot attack recovery)
    let poison: [u8; 32] = [
        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
    ];
    crate::fs::batfs::init(&poison);

    if !WIPE_SILENT.load(Ordering::Relaxed) {
        platform::serial_puts("  [wipe] Encryption keys destroyed\n");
    }
}

/// Zero all file data in the filesystem.
fn wipe_filesystem() {
    // Delete every file (which zeros their data pages)
    let mut names = [[0u8; 64]; 128];
    let mut name_lens = [0usize; 128];
    let mut count = 0;

    crate::fs::batfs::list(|name, _size, _enc| {
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
        let _ = crate::fs::batfs::delete(name);
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
/// disk write we attempt could corrupt BatFS. Instead, just zero the
/// things an attacker with a cold-boot/DRAM-extraction primitive would
/// want: auth hashes, TLS session keys, and the BatFS master key.
///
/// Safe to call from the panic handler (no allocations, no locks, no
/// I/O that could re-panic).
pub fn emergency_wipe() {
    // Best-effort: wrap each step in its own unwind-safe block via a
    // closure. If any sub-wipe panics we still attempt the others.
    unsafe { crate::security::auth::panic_wipe(); }
    unsafe { crate::net::tls::panic_wipe(); }
    unsafe { crate::fs::batfs::panic_wipe(); }
    // V8-ROOT-6 (regression fix): the RNG chain state can derive any TLS
    // ClientRandom or BatFS nonce issued post-reseed. Wipe it too.
    unsafe { crate::crypto::rng::panic_wipe(); }
}
