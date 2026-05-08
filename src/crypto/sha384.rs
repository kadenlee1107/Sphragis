// Bat_OS — SHA-384 / HMAC-SHA-384 / HKDF-SHA-384
//
// parallel module to `sha256.rs` for the TLS 1.3
// `TLS_AES_256_GCM_SHA384` cipher suite. Same API surface, but:
// hash output: 48 bytes (vs 32 for SHA-256)
// HMAC block: 128 bytes (vs 64 — SHA-384 inherits SHA-512's
// block size)
// HKDF-Extract / Expand-Label: produce 48-byte secrets
//
// Backed by RustCrypto's audited `sha2::Sha384`. We hand-roll HMAC
// (standard ipad/opad construction) + HKDF (RFC 5869) on top so we
// don't pull the `hmac` / `hkdf` crates as additional dependencies.
//
// All `&[u8; 48]` types match SHA-384's 384-bit output. TLS 1.3
// callers in `net/tls.rs` branch on the negotiated cipher suite and
// call this module's functions when the suite is *_SHA384.

#![allow(dead_code)]

use sha2::{Sha384, Digest};

/// SHA-384 over a single byte slice. 48-byte output.
pub fn hash(data: &[u8]) -> [u8; 48] {
    let mut h = Sha384::new();
    h.update(data);
    let out = h.finalize();
    let mut r = [0u8; 48];
    r.copy_from_slice(&out);
    r
}

/// HMAC-SHA384(key, message) → 48 bytes.
// /
/// Block size for SHA-384 is 128 bytes (same as SHA-512), so ipad/opad
/// are 128-byte buffers. This is the only structural difference from
/// HMAC-SHA-256 (which uses 64-byte ipad/opad).
pub fn hmac(key: &[u8], message: &[u8]) -> [u8; 48] {
    const BLOCK: usize = 128;
    let mut padded_key = [0u8; BLOCK];
    if key.len() > BLOCK {
        let h = hash(key);
        padded_key[..48].copy_from_slice(&h);
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= padded_key[i];
        opad[i] ^= padded_key[i];
    }

    let mut inner = Sha384::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha384::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    let out = outer.finalize();
    let mut r = [0u8; 48];
    r.copy_from_slice(&out);
    r
}

/// HKDF-Extract(salt, ikm) → PRK (48 bytes).
/// RFC 5869: PRK = HMAC-Hash(salt, ikm). Empty salt expands to a
/// 48-byte zero IKM per RFC 8446's HKDF usage with the hash-output-
/// sized zero string.
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 48] {
    let s = if salt.is_empty() { &[0u8; 48] as &[u8] } else { salt };
    hmac(s, ikm)
}

/// HKDF-Expand(PRK, info, L) → OKM (≤48 bytes).
/// Only supports length ≤ 48 — one HMAC-block of output, all that
/// TLS 1.3 key derivation needs per call.
pub fn hkdf_expand(prk: &[u8; 48], info: &[u8], length: usize) -> [u8; 48] {
    // T(1) = HMAC-Hash(PRK, info || 0x01)
    let mut input = [0u8; 256];
    let ilen = info.len().min(254);
    input[..ilen].copy_from_slice(&info[..ilen]);
    input[ilen] = 0x01;
    let result = hmac(prk, &input[..ilen + 1]);

    if length <= 48 {
        return result;
    }
    // For length > 48 we'd need T(2). Not needed for TLS 1.3.
    result
}

/// HKDF-Expand-Label per RFC 8446 §7.1.
/// `HkdfLabel = length(2) || "tls13 " || label || context`.
pub fn hkdf_expand_label(secret: &[u8; 48], label: &[u8], context: &[u8], length: usize) -> [u8; 48] {
    let mut info = [0u8; 128];
    let mut pos = 0;

    // length (2 bytes, big-endian)
    info[pos] = (length >> 8) as u8; pos += 1;
    info[pos] = length as u8; pos += 1;

    // label with "tls13 " prefix
    let prefix = b"tls13 ";
    let label_len = prefix.len() + label.len();
    info[pos] = label_len as u8; pos += 1;
    info[pos..pos + prefix.len()].copy_from_slice(prefix); pos += prefix.len();
    let ll = label.len().min(64);
    info[pos..pos + ll].copy_from_slice(&label[..ll]); pos += ll;

    // context
    let cl = context.len().min(48);
    info[pos] = cl as u8; pos += 1;
    if cl > 0 {
        info[pos..pos + cl].copy_from_slice(&context[..cl]); pos += cl;
    }

    hkdf_expand(secret, &info[..pos], length)
}
