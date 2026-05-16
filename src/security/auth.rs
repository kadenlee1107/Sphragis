#![allow(dead_code)]
// Sphragis — Authentication Gate
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

/// DESIGN_CRYPTO.md #1: memory-hard passphrase KDF via Argon2id.
///
/// Argon2id is the 2015 Password Hashing Competition winner and the
/// current NIST / OWASP recommendation. Unlike iterated SHA-256, its
/// cost function is MEMORY-hard: a GPU/ASIC attacker can't parallelize
/// millions of guesses because each one burns `memory` bytes of RAM
/// for `time` passes. Flat-out the biggest single crypto upgrade on
/// the Sphragis roadmap.
///
/// Parameters (m=16384, t=3, p=1) = ~16 MiB × 3 passes ≈ 50 ms on M4.
/// Sized to be barely noticeable at auth but painful to brute:
///   - At ~20 guesses/s per attacker machine, a 10-char alphanumeric
///     passphrase (~60 bits) takes centuries of CPU-years.
///   - Memory cost resists ASIC/GPU farms because memory is expensive
///     to add linearly.
///
/// We use a 16-byte salt baked into the binary. Production should
/// derive the salt from a per-device secret (TPM, Secure Enclave, or
/// first-boot randomness). For this commit we preserve the existing
/// semantics — same passphrase → same hash across reboots — using the
/// sphragis-auth-v2 domain-separation constant so anyone comparing to
/// the pre-upgrade hash can see it changed.
fn kdf(passphrase: &[u8]) -> [u8; 32] {
    use argon2::{Argon2, Algorithm, Version, Params};

    // 8 MiB × 3 passes × 1 lane. Fits comfortably in our 32 MB kernel
    // heap (see kernel/mm/heap.rs). Auth wait on M4 native is ~30 ms;
    // on QEMU-virt emulated ~150 ms — both imperceptible to a human
    // but crushing to GPU/ASIC offline attackers.
    const SALT: &[u8; 18] = b"sphragis-auth-v2\0\0";
    const MEM_KIB: u32 = 8_192;    // 8 MiB
    const TIME_COST: u32 = 3;
    const PARALLELISM: u32 = 1;
    const OUTLEN: usize = 32;

    let params = match Params::new(MEM_KIB, TIME_COST, PARALLELISM, Some(OUTLEN)) {
        Ok(p) => p,
        Err(_) => {
            // Parameters are constants known-good at build time; fall
            // back to defaults if the crate API rejects them for any
            // reason (version drift).
            Params::default()
        }
    };
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; 32];
    // Sphragis crate ships `alloc` — argon2 uses `Vec<u8>` scratch for
    // the memory blocks. On no_std + global_allocator that's fine.
    if argon.hash_password_into(passphrase, SALT, &mut out).is_err() {
        // Argon2 failed for some reason (e.g. passphrase length out of
        // range). Fall through to the old SHA-256 path so auth stays
        // functional. Logs via uart at boot would be ideal here.
        return fallback_sha256_kdf(passphrase);
    }
    out
}

/// Retained as a last-resort fallback if the Argon2 path errors out.
/// Same construction as the pre-upgrade code: N-round SHA-256 over the
/// passphrase + salt + round counter. Domain-separated salt so its
/// output is distinct from the Argon2id output for the same passphrase.
fn fallback_sha256_kdf(passphrase: &[u8]) -> [u8; 32] {
    const SALT: [u8; 18] = *b"sphragis-auth-v1\0\0";
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

    // AUDIT-CAVE-M6 (2026-05-15): exponential per-attempt backoff.
    // Prior code returned Failed immediately, and Argon2id's KDF
    // wall-time (~100 ms) was the only thing slowing brute force.
    // Five failed attempts can be issued back-to-back in <1s with
    // serial console + scripted retry. Add a busy-wait scaled to
    // attempts: 2^attempts × ~100 ms wall time (using monotonic
    // ticks → tick rate query → spin). NIST SP 800-63B §5.2.2
    // recommends rate-limiting or hardware-tied backoff.
    //
    // attempts | delay
    //   1      |  ~100 ms
    //   2      |  ~200 ms
    //   3      |  ~400 ms
    //   4      |  ~800 ms
    //   5      |  ~1600 ms (then LockedOut)
    //
    // Implementation: monotonic_secs is too coarse; use cntpct_el0
    // raw ticks against cntfrq_el0 (the timer frequency). This is
    // a real wall-clock delay on QEMU and on M4.
    {
        let freq: u64;
        unsafe { core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) freq); }
        let base_ticks = freq / 10; // 100 ms
        let scale: u64 = 1u64 << attempts.min(5);  // cap at 32×
        let target_ticks = base_ticks.saturating_mul(scale);
        let start: u64;
        unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) start); }
        loop {
            let now: u64;
            unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) now); }
            if now.wrapping_sub(start) >= target_ticks { break; }
            core::hint::spin_loop();
        }
    }

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
        unsafe {
            core::ptr::write_volatile(pass_ptr.add(i), 0);
            core::ptr::write_volatile(duress_ptr.add(i), 0);
        }
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
