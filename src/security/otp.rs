//! DESIGN_CRYPTO.md #11 + #12 — One-time-pad-backed emergency tokens.
//!
//! Scope
//! -----
//! This module is the legitimate use of OTP in Sphragis. OTP is
//! information-theoretically unbreakable only when the key is truly
//! random, at least as long as the plaintext, never reused, and kept
//! secret. For filesystems / streams those preconditions are
//! impossible to maintain at scale — but for small, pre-distributable,
//! single-use secrets (emergency-wipe triggers, deadman proof-of-life)
//! they're a perfect fit.
//!
//! The pad
//! -------
//! On boot (after `rng::probe_hw_rng`) we draw `PAD_TOKENS` × 32 bytes
//! of true-random material from the ARMv8.5 RNDR-backed CSPRNG. Each
//! 32-byte slot is one single-use token. The operator is expected to
//! print / write down the pad contents (QR codes, paper slips) at
//! first boot, keeping a physical copy offline.
//!
//! Later, the operator presents one token via any offline channel
//! (SMS, courier, satellite) to trigger:
//!   - DURESS: wipe everything NOW. Token matches a slot in the
//!     "duress" region of the pad → `security::wipe::execute` fires.
//!   - KEEPALIVE: "I'm still alive" — consumes a slot in the
//!     "deadman" region and refreshes `security::deadman::refresh()`.
//!
//! Single-use semantics are enforced by overwriting the consumed slot
//! with zeroes immediately, before returning. An attacker who captures
//! a used token learns nothing — the token's one-shot secret has
//! already been burned.
//!
//! Security properties
//! -------------------
//! * Info-theoretically unforgeable — an attacker without the pad
//!   cannot guess any future token faster than 2^256.
//! * Replay-proof — once a slot is consumed, it's zeroed. Presenting
//!   the same token twice fails.
//! * Forward-secret for the consumed slot — even seizure of the pad
//!   after consumption doesn't reveal which historical tokens were
//!   used legitimately (all zeroed rows look the same).
//!
//! What this is NOT
//! ----------------
//! Not a general authentication primitive. Use Argon2id+HMAC for that.
//! Not a filesystem cipher. Use ChaCha20-Poly1305 for that.
//! Not a TLS-style session — use TLS 1.3 (+ PQ hybrid) for that.
//!
//! References
//! ----------
//! - Shannon, C. (1949). Communication Theory of Secrecy Systems.
//! - DESIGN_CRYPTO.md rows #11 + #12 for context.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};

use crate::crypto::rng;

/// Tokens per region. The pad is organised:
///   slot 0..DURESS_TOKENS         → duress (wipe) channel
///   DURESS_TOKENS..DURESS_TOKENS+DEADMAN_TOKENS → deadman keepalive
pub const DURESS_TOKENS: usize = 8;
pub const DEADMAN_TOKENS: usize = 24;
pub const PAD_TOKENS: usize = DURESS_TOKENS + DEADMAN_TOKENS;
pub const TOKEN_LEN: usize = 32;

/// Pad sits in kernel BSS. Read/written through `core::ptr::*_volatile`
/// so zeroization is not optimised away by the compiler.
static mut PAD: [[u8; TOKEN_LEN]; PAD_TOKENS] = [[0u8; TOKEN_LEN]; PAD_TOKENS];

static READY: AtomicBool = AtomicBool::new(false);

/// Initialise the OTP pad with fresh true-random bytes. Call ONCE at
/// boot, after `rng::probe_hw_rng`. Idempotent — a second call is a
/// no-op (otherwise we'd invalidate already-distributed tokens).
pub fn init() {
    if READY.load(Ordering::Acquire) {
        return;
    }
    unsafe {
        let p = core::ptr::addr_of_mut!(PAD) as *mut u8;
        let mut buf = [0u8; TOKEN_LEN];
        for slot in 0..PAD_TOKENS {
            rng::fill_bytes(&mut buf);
            for i in 0..TOKEN_LEN {
                core::ptr::write_volatile(p.add(slot * TOKEN_LEN + i), buf[i]);
            }
        }
        // Wipe the stack-local buf so we don't leak one token via a
        // later stack-reuse.
        for i in 0..TOKEN_LEN {
            core::ptr::write_volatile(&mut buf[i], 0);
        }
    }
    READY.store(true, Ordering::Release);
    crate::drivers::uart::puts("  [otp] pad initialised: ");
    crate::kernel::mm::print_num(PAD_TOKENS);
    crate::drivers::uart::puts(" tokens (");
    crate::kernel::mm::print_num(DURESS_TOKENS);
    crate::drivers::uart::puts(" duress + ");
    crate::kernel::mm::print_num(DEADMAN_TOKENS);
    crate::drivers::uart::puts(" deadman)\n");
}

/// Export the pad for the operator to record offline. MUST be called
/// exactly once at provisioning time. Returns a reference that's valid
/// only for the lifetime of the call — caller should print immediately
/// (via UART or framebuffer), never store elsewhere.
///
/// Returns None if `init` hasn't run.
pub fn dump_for_provisioning(visitor: &mut dyn FnMut(usize, &str, &[u8; TOKEN_LEN])) -> bool {
    if !READY.load(Ordering::Acquire) {
        return false;
    }
    unsafe {
        let p = core::ptr::addr_of!(PAD) as *const u8;
        for slot in 0..PAD_TOKENS {
            let mut tok = [0u8; TOKEN_LEN];
            for i in 0..TOKEN_LEN {
                tok[i] = core::ptr::read_volatile(p.add(slot * TOKEN_LEN + i));
            }
            let region = if slot < DURESS_TOKENS { "duress" } else { "deadman" };
            visitor(slot, region, &tok);
        }
    }
    true
}

/// Attempt to consume a token. Walks the pad (constant-time), checks
/// every unused slot. Returns the region name the token matched, or
/// None if no match.
///
/// Constant-time discipline: we scan ALL slots even after a match, and
/// compare bytes without early-exit, to avoid timing-leak of which
/// slot matched.
pub fn consume(token: &[u8; TOKEN_LEN]) -> Option<&'static str> {
    if !READY.load(Ordering::Acquire) {
        return None;
    }
    let mut matched_slot: i32 = -1;
    unsafe {
        let p = core::ptr::addr_of_mut!(PAD) as *mut u8;
        for slot in 0..PAD_TOKENS {
            // Constant-time memcmp over 32 bytes.
            let mut diff: u8 = 0;
            // Skip already-consumed slots (all-zero); detect via OR
            // of slot bytes. Still scan every byte for const-time.
            let mut slot_empty: u8 = 0xFF;
            for i in 0..TOKEN_LEN {
                let b = core::ptr::read_volatile(p.add(slot * TOKEN_LEN + i));
                diff |= b ^ token[i];
                slot_empty &= if b == 0 { 0xFF } else { 0x00 };
            }
            let slot_ok = (diff == 0) & (slot_empty == 0x00);
            if slot_ok && matched_slot < 0 {
                matched_slot = slot as i32;
                // Don't break — keep scanning for const-time.
            }
        }

        // Zeroize the matched slot, if any, BEFORE returning so
        // re-presentation fails.
        if matched_slot >= 0 {
            for i in 0..TOKEN_LEN {
                core::ptr::write_volatile(p.add(matched_slot as usize * TOKEN_LEN + i), 0);
            }
        }
    }
    if matched_slot < 0 { return None; }
    let s = matched_slot as usize;
    if s < DURESS_TOKENS { Some("duress") } else { Some("deadman") }
}

/// Zeroize the entire pad. Called on wipe events so seized hardware
/// contains no usable OTP material.
pub fn wipe() {
    unsafe {
        let p = core::ptr::addr_of_mut!(PAD) as *mut u8;
        for i in 0..(PAD_TOKENS * TOKEN_LEN) {
            core::ptr::write_volatile(p.add(i), 0);
        }
    }
    READY.store(false, Ordering::Release);
    crate::drivers::uart::puts("  [otp] pad zeroized\n");
}

/// How many duress / deadman tokens remain usable.
pub fn remaining() -> (usize, usize) {
    if !READY.load(Ordering::Acquire) {
        return (0, 0);
    }
    let (mut d, mut k) = (0usize, 0usize);
    unsafe {
        let p = core::ptr::addr_of!(PAD) as *const u8;
        for slot in 0..PAD_TOKENS {
            let mut any: u8 = 0;
            for i in 0..TOKEN_LEN {
                any |= core::ptr::read_volatile(p.add(slot * TOKEN_LEN + i));
            }
            if any != 0 {
                if slot < DURESS_TOKENS { d += 1; } else { k += 1; }
            }
        }
    }
    (d, k)
}
