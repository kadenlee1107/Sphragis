//! BLAKE3 wrappers — keyed hash, MAC, content addressing.
//!
//! BLAKE3 is significantly faster than SHA-256 on modern CPUs (3-10x
//! depending on input length) and supports keyed/MAC mode + arbitrary
//! output length out of the box. We use it for:
//!
//! - **Content addressing** in the RAG corpus + SBOM (one BLAKE3 hash
//!   per file, fixed 32 bytes).
//! - **High-throughput MAC** as an alternative to HMAC-SHA256 when
//!   FIPS-mode isn't required.
//! - **Key-derivation** via the `derive_key` context tag (not a
//!   FIPS-approved KDF; use HKDF-SHA256 from `crypto::sha256` for
//!   approved-mode work).
//!
//! BLAKE3 is NOT a FIPS-approved hash. Don't use it where FIPS
//! conformance is needed.

#![allow(dead_code)]

use alloc::vec::Vec;

pub const BLAKE3_OUT_LEN: usize = 32;

/// One-shot 32-byte digest.
pub fn hash(data: &[u8]) -> [u8; BLAKE3_OUT_LEN] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut out = [0u8; BLAKE3_OUT_LEN];
    out.copy_from_slice(digest.as_bytes());
    out
}

/// Variable-length output (XOF mode). Pulls `out_len` bytes from the
/// BLAKE3 stream.
pub fn xof(data: &[u8], out_len: usize) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let mut reader = hasher.finalize_xof();
    let mut out = alloc::vec![0u8; out_len];
    reader.fill(&mut out);
    out
}

/// Keyed BLAKE3 — used as a MAC. Key must be exactly 32 bytes.
pub fn mac(key: &[u8; 32], data: &[u8]) -> [u8; BLAKE3_OUT_LEN] {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(data);
    let mut out = [0u8; BLAKE3_OUT_LEN];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

/// Constant-time verification of a BLAKE3 MAC. Returns true iff
/// `mac == expected_mac`. Uses byte-XOR-OR accumulation — matches
/// our constant-cost abort discipline.
pub fn mac_verify(key: &[u8; 32], data: &[u8], expected: &[u8; BLAKE3_OUT_LEN]) -> bool {
    let actual = mac(key, data);
    let mut diff: u8 = 0;
    for i in 0..BLAKE3_OUT_LEN {
        diff |= actual[i] ^ expected[i];
    }
    diff == 0
}

/// Derive a 32-byte key from input key material using a domain-separation
/// context. NOT a FIPS-approved KDF — use HKDF for approved mode.
pub fn derive_key(context: &str, ikm: &[u8]) -> [u8; 32] {
    blake3::derive_key(context, ikm)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BLAKE3 KAT (from the official test vectors): hash(b"") =
    /// af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262.
    #[test]
    fn blake3_kat_empty() {
        let h = hash(b"");
        let expected: [u8; 32] = [
            0xaf,0x13,0x49,0xb9,0xf5,0xf9,0xa1,0xa6,
            0xa0,0x40,0x4d,0xea,0x36,0xdc,0xc9,0x49,
            0x9b,0xcb,0x25,0xc9,0xad,0xc1,0x12,0xb7,
            0xcc,0x9a,0x93,0xca,0xe4,0x1f,0x32,0x62,
        ];
        assert_eq!(h, expected);
    }
}
