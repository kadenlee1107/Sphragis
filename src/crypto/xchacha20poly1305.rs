//! XChaCha20-Poly1305 — extended-nonce AEAD.
//!
//! Why we want it alongside the regular ChaCha20-Poly1305 we already
//! use in `batcave/secure_channel.rs` and `fs/batfs.rs`:
//!
//! ChaCha20-Poly1305 uses a 96-bit (12-byte) nonce. That's safe under
//! a counter discipline, but a random nonce only allows ~2^32 messages
//! per key before birthday-paradox collision risk becomes non-trivial.
//!
//! XChaCha20-Poly1305 uses a 192-bit (24-byte) nonce, derived via
//! HChaCha20 subkey. A random nonce is safe for ~2^80 messages per
//! key — effectively unlimited. Use this whenever a counter-nonce
//! isn't natural (e.g. distributed encryption where senders can't
//! coordinate a counter).
//!
//! Reference: draft-irtf-cfrg-xchacha-03.

#![allow(dead_code)]

use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305 as XcpInner, XNonce,
};

pub const KEY_LEN: usize = 32;       // 256-bit key
pub const NONCE_LEN: usize = 24;     // 192-bit nonce
pub const TAG_LEN: usize = 16;       // 128-bit Poly1305 tag

#[derive(Debug)]
pub enum XCpError {
    EncryptFailed,
    DecryptFailed,
    BadKeyLen,
    BadNonceLen,
}

/// Stateless encrypt. Returns ciphertext || tag (Poly1305 tag appended).
pub fn encrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN],
               aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, XCpError> {
    let cipher = XcpInner::new(key.into());
    let n = XNonce::from_slice(nonce);
    let payload = chacha20poly1305::aead::Payload { msg: plaintext, aad };
    cipher.encrypt(n, payload).map_err(|_| XCpError::EncryptFailed)
}

/// Stateless decrypt. Input is ciphertext || tag.
pub fn decrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN],
               aad: &[u8], ciphertext_tag: &[u8]) -> Result<Vec<u8>, XCpError> {
    let cipher = XcpInner::new(key.into());
    let n = XNonce::from_slice(nonce);
    let payload = chacha20poly1305::aead::Payload { msg: ciphertext_tag, aad };
    cipher.decrypt(n, payload).map_err(|_| XCpError::DecryptFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [0x42u8; KEY_LEN];
        let nonce = [0x37u8; NONCE_LEN];
        let aad = b"associated-data";
        let pt = b"hello sphragis xchacha";
        let ct = encrypt(&key, &nonce, aad, pt).expect("encrypt");
        let rt = decrypt(&key, &nonce, aad, &ct).expect("decrypt");
        assert_eq!(rt.as_slice(), pt);
    }

    #[test]
    fn tampered_ciphertext_rejected() {
        let key = [0x99u8; KEY_LEN];
        let nonce = [0x11u8; NONCE_LEN];
        let pt = b"sensitive";
        let mut ct = encrypt(&key, &nonce, b"", pt).expect("encrypt");
        ct[0] ^= 1;
        let r = decrypt(&key, &nonce, b"", &ct);
        assert!(r.is_err(), "tampered ciphertext must be rejected");
    }
}
