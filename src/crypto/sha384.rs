// Sphragis — SHA-384 / HMAC-SHA-384 / HKDF-SHA-384
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
///
/// AUDIT-CRYPTO-F8 (2026-05-15): see sha256.rs::hkdf_expand. Same
/// silent-T(1)-fallthrough bug. Fail-closed: panic on length > 48.
pub fn hkdf_expand(prk: &[u8; 48], info: &[u8], length: usize) -> [u8; 48] {
    if length > 48 {
        panic!("hkdf_expand: length > 48 — multi-block expansion not implemented");
    }
    let mut input = [0u8; 256];
    let ilen = info.len().min(254);
    input[..ilen].copy_from_slice(&info[..ilen]);
    input[ilen] = 0x01;
    hmac(prk, &input[..ilen + 1])
}

/// HKDF-Expand-Label per RFC 8446 §7.1.
///
/// AUDIT-CRYPTO-F9 (2026-05-15): see sha256.rs::hkdf_expand_label.
/// Same silent-truncation bug — panic instead.
pub fn hkdf_expand_label(secret: &[u8; 48], label: &[u8], context: &[u8], length: usize) -> [u8; 48] {
    let prefix = b"tls13 ";
    let label_total = prefix.len() + label.len();
    let info_len = 2 + 1 + label_total + 1 + context.len();
    if label_total > 255 {
        panic!("hkdf_expand_label(sha384): label too long");
    }
    if context.len() > 255 {
        panic!("hkdf_expand_label(sha384): context too long");
    }
    if info_len > 128 {
        panic!("hkdf_expand_label(sha384): HkdfLabel overruns info buffer");
    }

    let mut info = [0u8; 128];
    let mut pos = 0;
    info[pos] = (length >> 8) as u8; pos += 1;
    info[pos] = length as u8; pos += 1;
    info[pos] = label_total as u8; pos += 1;
    info[pos..pos + prefix.len()].copy_from_slice(prefix); pos += prefix.len();
    info[pos..pos + label.len()].copy_from_slice(label); pos += label.len();
    info[pos] = context.len() as u8; pos += 1;
    info[pos..pos + context.len()].copy_from_slice(context); pos += context.len();

    hkdf_expand(secret, &info[..pos], length)
}

/// Boot-time KAT. FIPS 180-4 §F.4 vector for "abc":
/// SHA-384("abc") = cb00753f 45a35e8b b5a03d69 9ac65007 272c32ab 0eded163
///                  1a8b605a 43ff5bed 8086072b a1e7cc23 58baeca1 34c825a7
/// Plus HMAC-SHA-384 RFC 4231 test case 1: key=0x0b*20, data="Hi There".
/// Returns `Err` on divergence; kernel halts boot via the established
/// fail-closed self-test pattern (audit CRYPTO-F7).
pub fn kat() -> Result<(), &'static str> {
    let h = hash(b"abc");
    let expected: [u8; 48] = [
        0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b,
        0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6, 0x50, 0x07,
        0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63,
        0x1a, 0x8b, 0x60, 0x5a, 0x43, 0xff, 0x5b, 0xed,
        0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23,
        0x58, 0xba, 0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7,
    ];
    let mut diff: u8 = 0;
    for i in 0..48 { diff |= h[i] ^ expected[i]; }
    if diff != 0 {
        return Err("KAT-FAIL: SHA-384 \"abc\" mismatch vs FIPS 180-4 §F.4");
    }

    // RFC 4231 §4.2 test case 1 — HMAC-SHA-384 vector.
    let key = [0x0bu8; 20];
    let mac = hmac(&key, b"Hi There");
    let expected_mac: [u8; 48] = [
        0xaf, 0xd0, 0x39, 0x44, 0xd8, 0x48, 0x95, 0x62,
        0x6b, 0x08, 0x25, 0xf4, 0xab, 0x46, 0x90, 0x7f,
        0x15, 0xf9, 0xda, 0xdb, 0xe4, 0x10, 0x1e, 0xc6,
        0x82, 0xaa, 0x03, 0x4c, 0x7c, 0xeb, 0xc5, 0x9c,
        0xfa, 0xea, 0x9e, 0xa9, 0x07, 0x6e, 0xde, 0x7f,
        0x4a, 0xf1, 0x52, 0xe8, 0xb2, 0xfa, 0x9c, 0xb6,
    ];
    let mut mdiff: u8 = 0;
    for i in 0..48 { mdiff |= mac[i] ^ expected_mac[i]; }
    if mdiff != 0 {
        return Err("KAT-FAIL: HMAC-SHA-384 mismatch vs RFC 4231 §4.2 TC1");
    }
    Ok(())
}
