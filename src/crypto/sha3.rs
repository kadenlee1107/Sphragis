//! SHA-3 family wrappers — FIPS 202 (Keccak).
//!
//! We get SHA-2 from `sha2::` (already used by sha256.rs / sha384.rs).
//! SHA-3 lives here for protocols that explicitly mandate Keccak
//! (e.g. some FIPS submodes, KMAC-class constructions) and as a
//! defensive alternative if a SHA-2 break ever lands.
//!
//! Exposes the FIPS 202 fixed-output digests (Sha3_256, Sha3_384,
//! Sha3_512) plus the SHAKE128 / SHAKE256 extendable-output functions.

#![allow(dead_code)]

use alloc::vec::Vec;
use sha3::{Digest, Sha3_256, Sha3_384, Sha3_512};
use sha3::digest::{ExtendableOutput, XofReader};

pub const SHA3_256_LEN: usize = 32;
pub const SHA3_384_LEN: usize = 48;
pub const SHA3_512_LEN: usize = 64;

/// One-shot SHA3-256. Output is 32 bytes.
pub fn sha3_256(data: &[u8]) -> [u8; SHA3_256_LEN] {
    let mut h = Sha3_256::new();
    h.update(data);
    let out = h.finalize();
    let mut buf = [0u8; SHA3_256_LEN];
    buf.copy_from_slice(&out);
    buf
}

/// One-shot SHA3-384. Output is 48 bytes.
pub fn sha3_384(data: &[u8]) -> [u8; SHA3_384_LEN] {
    let mut h = Sha3_384::new();
    h.update(data);
    let out = h.finalize();
    let mut buf = [0u8; SHA3_384_LEN];
    buf.copy_from_slice(&out);
    buf
}

/// One-shot SHA3-512. Output is 64 bytes.
pub fn sha3_512(data: &[u8]) -> [u8; SHA3_512_LEN] {
    let mut h = Sha3_512::new();
    h.update(data);
    let out = h.finalize();
    let mut buf = [0u8; SHA3_512_LEN];
    buf.copy_from_slice(&out);
    buf
}

/// SHAKE128 extendable-output function. Returns the requested number
/// of bytes. SHAKE128 has 128-bit security level — adequate for KEMs
/// and for protocol nonce derivation. Use SHAKE256 for 256-bit level.
pub fn shake128(data: &[u8], out_len: usize) -> Vec<u8> {
    use sha3::digest::Update;
    let mut h = sha3::Shake128::default();
    Update::update(&mut h, data);
    let mut reader = h.finalize_xof();
    let mut out = alloc::vec![0u8; out_len];
    reader.read(&mut out);
    out
}

/// SHAKE256 extendable-output function.
pub fn shake256(data: &[u8], out_len: usize) -> Vec<u8> {
    use sha3::digest::Update;
    let mut h = sha3::Shake256::default();
    Update::update(&mut h, data);
    let mut reader = h.finalize_xof();
    let mut out = alloc::vec![0u8; out_len];
    reader.read(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS 202 KAT: SHA3-256("") = a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a.
    #[test]
    fn fips202_kat_empty() {
        let h = sha3_256(b"");
        let expected: [u8; 32] = [
            0xa7,0xff,0xc6,0xf8,0xbf,0x1e,0xd7,0x66,
            0x51,0xc1,0x47,0x56,0xa0,0x61,0xd6,0x62,
            0xf5,0x80,0xff,0x4d,0xe4,0x3b,0x49,0xfa,
            0x82,0xd8,0x0a,0x4b,0x80,0xf8,0x43,0x4a,
        ];
        assert_eq!(h, expected);
    }
}
