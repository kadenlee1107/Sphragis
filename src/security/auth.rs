#![allow(dead_code)]
// Bat_OS — Authentication Gate
// Runs BEFORE anything else. No passphrase + YubiKey = no access.
// Three paths: SUCCESS, FAIL (count), DURESS (fake boot + wipe).
//
// Security model:
// - Passphrase: knowledge factor (in your head)
// - YubiKey: possession factor (can be destroyed)
// - Duress code: silent wipe under coercion
// - Max attempts: exceed = permanent destruction

use crate::crypto::sha256;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

const MAX_ATTEMPTS: u8 = 5;
const MAX_PASSPHRASE_LEN: usize = 128;

// Stored as SHA-256 hash — the actual passphrase never exists on disk
// In production, this hash lives in the Secure Enclave
static mut PASSPHRASE_HASH: [u8; 32] = [0; 32];
static mut DURESS_HASH: [u8; 32] = [0; 32];

static ATTEMPT_COUNT: AtomicU8 = AtomicU8::new(0);
static AUTHENTICATED: AtomicBool = AtomicBool::new(false);
static LOCKED_OUT: AtomicBool = AtomicBool::new(false);

#[derive(PartialEq)]
pub enum AuthResult {
    Success,
    Failed,
    Duress,
    LockedOut,
}

/// Initialize the auth system with passphrase and duress code.
/// In production, these hashes come from Secure Enclave setup.
/// For dev/QEMU, we set them here.
pub fn init(passphrase: &str, duress_code: &str) {
    let pass_hash = kdf(passphrase.as_bytes());
    let duress_hash = kdf(duress_code.as_bytes());
    unsafe {
        PASSPHRASE_HASH = pass_hash;
        DURESS_HASH = duress_hash;
    }
    // V8-ROOT-4: Release-store so the hash writes above are visible to any
    // CPU that subsequently calls authenticate() (which now Acquire-loads).
    ATTEMPT_COUNT.store(0, Ordering::Release);
    AUTHENTICATED.store(false, Ordering::Release);
    LOCKED_OUT.store(false, Ordering::Release);
}

/// Iterated SHA-256 KDF (ATTACK-CRYPTO-005 partial). The prior
/// implementation was a single `sha256::hash(passphrase)` — unsalted,
/// uniterated, 10-char alphanumeric passphrases fall to a single GPU
/// in minutes. This adds:
///   - salt (prevents rainbow-table reuse across installs)
///   - 4096 iterations (each guess costs ~4096 SHA-256 ops)
///
/// Not Argon2 — that's the Phase B target. This is the "stop making
/// it trivially dictionary-attackable" fix.
fn kdf(passphrase: &[u8]) -> [u8; 32] {
    const SALT: [u8; 16] = *b"bat_os-auth-v1\0\0";
    let n = passphrase.len().min(64);
    let mut buf = [0u8; 128];
    buf[..n].copy_from_slice(&passphrase[..n]);
    buf[64..64 + 16].copy_from_slice(&SALT);
    let mut h = sha256::hash(&buf);
    for round in 0u64..4096 {
        let mut round_buf = [0u8; 128];
        round_buf[..32].copy_from_slice(&h);
        round_buf[32..32 + n].copy_from_slice(&passphrase[..n]);
        round_buf[96..96 + 16].copy_from_slice(&SALT);
        round_buf[112..120].copy_from_slice(&round.to_le_bytes());
        h = sha256::hash(&round_buf);
    }
    h
}

/// Attempt authentication with a passphrase.
/// Returns the result — caller decides what to do.
pub fn authenticate(input: &str) -> AuthResult {
    // V8-ROOT-4: Acquire-load LOCKED_OUT so that the Release-store in the
    // lockout branch (line ~117) is observed by other CPUs that hit this
    // path concurrently. Without Acquire, a CPU could see LOCKED_OUT=true
    // but stale memory writes that preceded the store.
    if LOCKED_OUT.load(Ordering::Acquire) {
        return AuthResult::LockedOut;
    }

    let input_hash = kdf(input.as_bytes());

    // Check duress code FIRST — if they're being coerced
    let is_duress = unsafe {
        let duress = core::ptr::read_volatile(core::ptr::addr_of!(DURESS_HASH));
        constant_time_eq(&input_hash, &duress)
    };
    if is_duress {
        return AuthResult::Duress;
    }

    // Check real passphrase
    let is_correct = unsafe {
        let pass = core::ptr::read_volatile(core::ptr::addr_of!(PASSPHRASE_HASH));
        constant_time_eq(&input_hash, &pass)
    };

    if is_correct {
        ATTEMPT_COUNT.store(0, Ordering::Release);
        AUTHENTICATED.store(true, Ordering::Release);
        return AuthResult::Success;
    }

    // V8-ROOT-4: Failed attempt — use atomic fetch_add. Previously this was
    // load+1+store, which is a non-atomic RMW: two concurrent failed
    // attempts could both load 0 and both store 1, costing one count of
    // brute-force protection per race window.
    let prev = ATTEMPT_COUNT.fetch_add(1, Ordering::AcqRel);
    let attempts = prev.saturating_add(1);

    if attempts >= MAX_ATTEMPTS {
        LOCKED_OUT.store(true, Ordering::Release);
        return AuthResult::LockedOut;
    }

    AuthResult::Failed
}

pub fn is_authenticated() -> bool {
    AUTHENTICATED.load(Ordering::Acquire)
}

pub fn attempts_remaining() -> u8 {
    let used = ATTEMPT_COUNT.load(Ordering::Acquire);
    if used >= MAX_ATTEMPTS { 0 } else { MAX_ATTEMPTS - used }
}

pub fn is_locked_out() -> bool {
    LOCKED_OUT.load(Ordering::Acquire)
}

/// Lock the session (require re-authentication).
pub fn lock() {
    // V8-ROOT-4: Release so a follow-up is_authenticated() call from another
    // CPU sees the lock immediately (paired with Acquire-load).
    AUTHENTICATED.store(false, Ordering::Release);
}

/// V8-ROOT-6: panic-handler-only secret wipe. Uses volatile writes so the
/// compiler cannot DCE them, and takes no locks so it can run in any
/// kernel state. Best-effort only — if we panic after a partial write the
/// first N bytes may already be zero.
///
/// # Safety
/// May only be called from the panic handler or from wipe::emergency_wipe.
/// Writing to these statics without the init path is otherwise a data race.
pub unsafe fn panic_wipe() {
    let pass_ptr = core::ptr::addr_of_mut!(PASSPHRASE_HASH) as *mut u8;
    let duress_ptr = core::ptr::addr_of_mut!(DURESS_HASH) as *mut u8;
    for i in 0..32 {
        core::ptr::write_volatile(pass_ptr.add(i), 0);
        core::ptr::write_volatile(duress_ptr.add(i), 0);
    }
}

/// Constant-time comparison to prevent timing attacks.
/// An attacker measuring response time cannot determine
/// which bytes of the hash matched.
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff: u8 = 0;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}
