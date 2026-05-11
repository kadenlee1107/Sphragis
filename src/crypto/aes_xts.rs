//! AES-XTS — block-level disk/sector encryption.
//!
//! XTS (XEX-based tweaked codebook with ciphertext stealing) is the
//! standard for full-disk encryption (FIPS-approved per NIST SP
//! 800-38E). It encrypts each sector independently using a
//! sector-derived tweak, so random-access reads/writes don't have to
//! re-encrypt the whole file.
//!
//! Why we need it on top of the AES-GCM and ChaCha20-Poly1305 AEADs
//! BatFS already uses:
//!
//! - GCM/Poly1305 are NOT appropriate for random-access blocks. A
//!   block-level decrypt would have to re-MAC the whole file's chunk
//!   range, which is wasteful and exposes timing patterns.
//! - XTS gives us length-preserving encryption per sector — needed
//!   when the eventual NVMe driver lands and we want native FDE-class
//!   semantics under BatFS's file layer.
//!
//! Limitations of XTS (documented for the future): it does NOT
//! authenticate, only confidentiality. Pair with a separate integrity
//! mechanism (Merkle tree per extent, or dm-verity-class signed root)
//! for full assurance. For BatFS today, file-level Poly1305 sits on
//! top — XTS would slot in beneath as the block cipher when we move
//! off the GCM-per-file design.

#![allow(dead_code)]

use aes::cipher::{generic_array::GenericArray, KeyInit};
use aes::{Aes128, Aes256};
use xts_mode::Xts128;

pub const SECTOR_SIZE_DEFAULT: usize = 4096;

#[derive(Debug)]
pub enum XtsError {
    BadKeyLen,
    BufferNotMultipleOfBlock,
}

/// AES-256-XTS encryption of one full sector (or aligned multi-block
/// chunk). `key1` and `key2` MUST be independent — concatenated they
/// form the XTS key. `tweak` is a 16-byte (128-bit) value that is
/// typically the sector index serialized big-endian, but any
/// uniquely-derived value works.
pub fn aes256_xts_encrypt_in_place(
    key1: &[u8; 32], key2: &[u8; 32], tweak: &[u8; 16], buf: &mut [u8],
) -> Result<(), XtsError> {
    if buf.len() % 16 != 0 {
        return Err(XtsError::BufferNotMultipleOfBlock);
    }
    let cipher_1 = Aes256::new(GenericArray::from_slice(key1));
    let cipher_2 = Aes256::new(GenericArray::from_slice(key2));
    let xts = Xts128::<Aes256>::new(cipher_1, cipher_2);
    xts.encrypt_sector(buf, *tweak);
    Ok(())
}

/// AES-256-XTS decryption of one full sector / aligned chunk.
pub fn aes256_xts_decrypt_in_place(
    key1: &[u8; 32], key2: &[u8; 32], tweak: &[u8; 16], buf: &mut [u8],
) -> Result<(), XtsError> {
    if buf.len() % 16 != 0 {
        return Err(XtsError::BufferNotMultipleOfBlock);
    }
    let cipher_1 = Aes256::new(GenericArray::from_slice(key1));
    let cipher_2 = Aes256::new(GenericArray::from_slice(key2));
    let xts = Xts128::<Aes256>::new(cipher_1, cipher_2);
    xts.decrypt_sector(buf, *tweak);
    Ok(())
}

/// AES-128-XTS (uses 16-byte keys). Smaller key, smaller perf cost
/// on aarch64. Same security level as AES-128 in any other mode.
pub fn aes128_xts_encrypt_in_place(
    key1: &[u8; 16], key2: &[u8; 16], tweak: &[u8; 16], buf: &mut [u8],
) -> Result<(), XtsError> {
    if buf.len() % 16 != 0 {
        return Err(XtsError::BufferNotMultipleOfBlock);
    }
    let cipher_1 = Aes128::new(GenericArray::from_slice(key1));
    let cipher_2 = Aes128::new(GenericArray::from_slice(key2));
    let xts = Xts128::<Aes128>::new(cipher_1, cipher_2);
    xts.encrypt_sector(buf, *tweak);
    Ok(())
}

pub fn aes128_xts_decrypt_in_place(
    key1: &[u8; 16], key2: &[u8; 16], tweak: &[u8; 16], buf: &mut [u8],
) -> Result<(), XtsError> {
    if buf.len() % 16 != 0 {
        return Err(XtsError::BufferNotMultipleOfBlock);
    }
    let cipher_1 = Aes128::new(GenericArray::from_slice(key1));
    let cipher_2 = Aes128::new(GenericArray::from_slice(key2));
    let xts = Xts128::<Aes128>::new(cipher_1, cipher_2);
    xts.decrypt_sector(buf, *tweak);
    Ok(())
}

/// Helper: build a 16-byte tweak from a u64 sector index, big-endian
/// in the low 8 bytes, zero-padded.
pub fn tweak_from_sector(sector: u64) -> [u8; 16] {
    let mut t = [0u8; 16];
    t[8..].copy_from_slice(&sector.to_be_bytes());
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes256_xts_roundtrip_one_sector() {
        let key1 = [0xaau8; 32];
        let key2 = [0xbbu8; 32];
        let tweak = tweak_from_sector(42);
        let mut buf = [0u8; SECTOR_SIZE_DEFAULT];
        for (i, b) in buf.iter_mut().enumerate() { *b = (i & 0xff) as u8; }
        let mut ct = buf;
        aes256_xts_encrypt_in_place(&key1, &key2, &tweak, &mut ct).unwrap();
        assert_ne!(buf[..], ct[..], "encryption must change the buffer");
        aes256_xts_decrypt_in_place(&key1, &key2, &tweak, &mut ct).unwrap();
        assert_eq!(buf[..], ct[..], "round-trip must restore plaintext");
    }

    #[test]
    fn different_tweak_yields_different_ciphertext() {
        let key1 = [0x10u8; 32];
        let key2 = [0x20u8; 32];
        let mut a = [0xccu8; 64];
        let mut b = [0xccu8; 64];
        aes256_xts_encrypt_in_place(&key1, &key2, &tweak_from_sector(1), &mut a).unwrap();
        aes256_xts_encrypt_in_place(&key1, &key2, &tweak_from_sector(2), &mut b).unwrap();
        assert_ne!(a[..], b[..], "different sector index must produce different ciphertext");
    }
}
