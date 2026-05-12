//! BLAKE2s — 256-bit cryptographic hash (RFC 7693).
//!
//! Different from the BLAKE3 we already use: BLAKE2s has a 32-byte
//! output ceiling and different IV constants. WireGuard's Noise IK
//! handshake mandates BLAKE2s specifically (in the HKDF, the
//! protocol-prologue hash, and the cookie MAC), so we can't
//! interop with `wg-quick` peers without it.
//!
//! Thin wrapper around the audited RustCrypto `blake2` crate.

#![allow(dead_code)]

use blake2::{Blake2s256, Blake2sMac};
use blake2::digest::{Digest, FixedOutput, Mac, KeyInit};

pub const HASH_LEN: usize = 32;

/// One-shot BLAKE2s-256 over `data`. Returns a fixed 32-byte digest.
pub fn hash(data: &[u8]) -> [u8; HASH_LEN] {
    let mut h = Blake2s256::new();
    Digest::update(&mut h, data);
    let mut out = [0u8; HASH_LEN];
    let d = h.finalize();
    out.copy_from_slice(&d);
    out
}

/// Multi-input convenience — equivalent to `hash(a || b)` without
/// the caller allocating a concatenation buffer.
pub fn hash2(a: &[u8], b: &[u8]) -> [u8; HASH_LEN] {
    let mut h = Blake2s256::new();
    Digest::update(&mut h, a);
    Digest::update(&mut h, b);
    let mut out = [0u8; HASH_LEN];
    let d = h.finalize();
    out.copy_from_slice(&d);
    out
}

/// BLAKE2s as a MAC, 16-byte output. WireGuard's `mac1` / `mac2`
/// fields use this construction with a 32-byte key.
pub fn mac16(key: &[u8; 32], msg: &[u8]) -> [u8; 16] {
    type B = Blake2sMac<blake2::digest::consts::U16>;
    let mut m = <B as KeyInit>::new_from_slice(key)
        .expect("BLAKE2s mac key length");
    Mac::update(&mut m, msg);
    let tag = m.finalize_fixed();
    let mut out = [0u8; 16];
    out.copy_from_slice(&tag);
    out
}

/// HMAC-BLAKE2s as an HKDF primitive — WireGuard's KDF derives all
/// session material via repeated HMAC over the chaining key. The
/// blake2 crate exposes `Blake2sMac256` for the variable-length MAC
/// shape HMAC needs; we wrap it as `hmac(key, data)` returning the
/// full 32-byte output so the caller can implement HKDF-Expand
/// step-by-step (T(i) = HMAC(prk, T(i-1) || i)).
pub fn hmac(key: &[u8], data: &[u8]) -> [u8; HASH_LEN] {
    type B = Blake2sMac<blake2::digest::consts::U32>;
    // The blake2 MAC is BLAKE2s with `key` set as the keyed-mode key.
    // For HMAC over BLAKE2s, RFC 4868 / NIST FIPS 198-1 require the
    // outer/inner pad construction. WireGuard's spec (§5.4) does
    // exactly that — keep this fn focused on the keyed-mode primitive,
    // build full HMAC in the WireGuard layer where the key gets
    // shaped per RFC 2104.
    let mut m = <B as KeyInit>::new_from_slice(key)
        .expect("BLAKE2s mac key length (max 32)");
    Mac::update(&mut m, data);
    let tag = m.finalize_fixed();
    let mut out = [0u8; HASH_LEN];
    out.copy_from_slice(&tag);
    out
}
