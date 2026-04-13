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
use crate::drivers::uart;
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
    let pass_hash = sha256::hash(passphrase.as_bytes());
    let duress_hash = sha256::hash(duress_code.as_bytes());
    unsafe {
        PASSPHRASE_HASH = pass_hash;
        DURESS_HASH = duress_hash;
    }
    ATTEMPT_COUNT.store(0, Ordering::Relaxed);
    AUTHENTICATED.store(false, Ordering::Relaxed);
    LOCKED_OUT.store(false, Ordering::Relaxed);
}

/// Attempt authentication with a passphrase.
/// Returns the result — caller decides what to do.
pub fn authenticate(input: &str) -> AuthResult {
    if LOCKED_OUT.load(Ordering::Relaxed) {
        return AuthResult::LockedOut;
    }

    let input_hash = sha256::hash(input.as_bytes());

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
        ATTEMPT_COUNT.store(0, Ordering::Relaxed);
        AUTHENTICATED.store(true, Ordering::Release);
        return AuthResult::Success;
    }

    // Failed attempt
    let attempts = ATTEMPT_COUNT.load(Ordering::Relaxed) + 1; ATTEMPT_COUNT.store(attempts, Ordering::Relaxed);

    if attempts >= MAX_ATTEMPTS {
        LOCKED_OUT.store(true, Ordering::Release);
        return AuthResult::LockedOut;
    }

    AuthResult::Failed
}

pub fn is_authenticated() -> bool {
    AUTHENTICATED.load(Ordering::Relaxed)
}

pub fn attempts_remaining() -> u8 {
    let used = ATTEMPT_COUNT.load(Ordering::Relaxed);
    if used >= MAX_ATTEMPTS { 0 } else { MAX_ATTEMPTS - used }
}

pub fn is_locked_out() -> bool {
    LOCKED_OUT.load(Ordering::Relaxed)
}

/// Lock the session (require re-authentication).
pub fn lock() {
    AUTHENTICATED.store(false, Ordering::Relaxed);
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
