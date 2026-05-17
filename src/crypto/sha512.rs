// Sphragis — SHA-512 / HMAC-SHA-512 / HKDF-SHA-512 (FIPS 180-4)
//
// Parallel module to `sha384.rs` for contexts that require the full
// 512-bit hash output (vs SHA-384's truncated 384-bit). Both share
// the same compression function and 128-byte HMAC block; SHA-512
// just emits all 64 output bytes.
//
// CNSA 2.0 accepts SHA-384 (preferred) OR SHA-512 for hashing. This
// module exists so:
//   * Attestation quote signing can use SHA-512 where the wider hash
//     is required by an external verifier (some Caliptra / SEP /
//     TPM endorsement chains prefer SHA-512 over SHA-384).
//   * BatFS Merkle-tree intermediate nodes can use SHA-512 where the
//     full 64-byte hash gives a wider security margin per node.
//   * The boot-time KAT covers both SHA-384 and SHA-512 so a
//     regression in either is caught at first boot.
//
// Backed by RustCrypto's audited `sha2::Sha512`. HMAC + HKDF
// hand-rolled in the same shape as sha384.rs so we don't pull in
// the `hmac` / `hkdf` crates as additional dependencies. See
// REQ-CRY-005 in docs/superpowers/specs/2026-05-16-sphragis-gov-os-requirements.md.

#![allow(dead_code)]

use sha2::{Sha512, Digest};

/// SHA-512 over a single byte slice. 64-byte output.
pub fn hash(data: &[u8]) -> [u8; 64] {
    let mut h = Sha512::new();
    h.update(data);
    let out = h.finalize();
    let mut r = [0u8; 64];
    r.copy_from_slice(&out);
    r
}

/// HMAC-SHA512(key, message) → 64 bytes.
///
/// Block size for SHA-512 is 128 bytes (same as SHA-384). ipad/opad
/// are 128-byte buffers.
pub fn hmac(key: &[u8], message: &[u8]) -> [u8; 64] {
    const BLOCK: usize = 128;
    let mut padded_key = [0u8; BLOCK];
    if key.len() > BLOCK {
        let h = hash(key);
        padded_key[..64].copy_from_slice(&h);
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= padded_key[i];
        opad[i] ^= padded_key[i];
    }

    let mut inner = Sha512::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha512::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    let out = outer.finalize();
    let mut r = [0u8; 64];
    r.copy_from_slice(&out);
    r
}

/// HKDF-Extract(salt, ikm) → PRK (64 bytes).
/// RFC 5869: PRK = HMAC-Hash(salt, ikm). Empty salt expands to a
/// 64-byte zero IKM per RFC 8446's HKDF usage with the hash-output-
/// sized zero string.
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 64] {
    let s = if salt.is_empty() { &[0u8; 64] as &[u8] } else { salt };
    hmac(s, ikm)
}

/// HKDF-Expand(PRK, info, L) → OKM (≤64 bytes).
///
/// Single-block expansion only. Matches the policy in sha256.rs /
/// sha384.rs from audit CRYPTO-F8 (silent T(1) fallthrough): fail
/// closed on length > 64 rather than silently truncating.
pub fn hkdf_expand(prk: &[u8; 64], info: &[u8], length: usize) -> [u8; 64] {
    if length > 64 {
        panic!("hkdf_expand(sha512): length > 64 — multi-block expansion not implemented");
    }
    let mut input = [0u8; 256];
    let ilen = info.len().min(254);
    input[..ilen].copy_from_slice(&info[..ilen]);
    input[ilen] = 0x01;
    hmac(prk, &input[..ilen + 1])
}

/// HKDF-Expand-Label per RFC 8446 §7.1, with SHA-512 backing.
///
/// Matches the audit CRYPTO-F9 fail-closed policy from sha256/384:
/// panic instead of silent truncation.
pub fn hkdf_expand_label(secret: &[u8; 64], label: &[u8], context: &[u8], length: usize) -> [u8; 64] {
    let prefix = b"tls13 ";
    let label_total = prefix.len() + label.len();
    let info_len = 2 + 1 + label_total + 1 + context.len();
    if label_total > 255 {
        panic!("hkdf_expand_label(sha512): label too long");
    }
    if context.len() > 255 {
        panic!("hkdf_expand_label(sha512): context too long");
    }
    if info_len > 128 {
        panic!("hkdf_expand_label(sha512): HkdfLabel overruns info buffer");
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

/// Boot-time Known-Answer Test. FIPS 180-4 §F.3 vector for "abc":
/// SHA-512("abc") = ddaf35a1 93617aba cc417349 ae204131 12e6fa4e
///                  89a97ea2 0a9eeee6 4b55d39a 2192992a 274fc1a8
///                  36ba3c23 a3feebbd 454d4423 643ce80e 2a9ac94f
///                  a54ca49f
/// Returns `Err` if the computed hash diverges from the spec value;
/// the kernel `unwrap()`s the result so any divergence halts boot
/// (the established fail-closed self-test pattern, audit CRYPTO-F7).
pub fn kat() -> Result<(), &'static str> {
    let h = hash(b"abc");
    let expected: [u8; 64] = [
        0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba,
        0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31,
        0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2,
        0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a,
        0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8,
        0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd,
        0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
        0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
    ];
    let mut diff: u8 = 0;
    for i in 0..64 { diff |= h[i] ^ expected[i]; }
    if diff != 0 {
        return Err("KAT-FAIL: SHA-512 \"abc\" mismatch vs FIPS 180-4 §F.3");
    }

    // HMAC-SHA-512 round-trip: same key/message twice produces the
    // same MAC. Independent shape-check (not a NIST vector, just a
    // codegen sanity).
    let m1 = hmac(b"sphragis-kat-key", b"hello sha512");
    let m2 = hmac(b"sphragis-kat-key", b"hello sha512");
    let mut diff2: u8 = 0;
    for i in 0..64 { diff2 |= m1[i] ^ m2[i]; }
    if diff2 != 0 {
        return Err("KAT-FAIL: HMAC-SHA-512 non-deterministic");
    }

    Ok(())
}
