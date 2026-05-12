//! ChaCha20-Poly1305 — 96-bit nonce AEAD.
//!
//! Thin wrapper around the RustCrypto `chacha20poly1305` crate. We
//! already have `xchacha20poly1305.rs` for the extended-nonce variant;
//! this exposes the standard 12-byte-nonce version that interops with
//! every off-the-shelf TLS/QUIC/Noise stack — including Python's
//! `cryptography.hazmat.primitives.ciphers.aead.ChaCha20Poly1305`,
//! which is what the test server uses.
//!
//! Nonce safety: 96-bit nonces are not safe under random sampling
//! (birthday risk after ~2^32 messages per key). Callers MUST use a
//! deterministic counter discipline. The comms session does this
//! by allocating separate keys per direction and using a u64 frame
//! counter padded with 4 zero bytes for the nonce.

#![allow(dead_code)]

use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305 as CpInner, Nonce,
};

pub const KEY_LEN:   usize = 32;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN:   usize = 16;

#[derive(Debug)]
pub enum CpError {
    EncryptFailed,
    DecryptFailed,
}

/// Stateless encrypt. Returns ciphertext || tag.
pub fn encrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN],
               aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CpError> {
    let cipher = CpInner::new(key.into());
    let n = Nonce::from_slice(nonce);
    let payload = chacha20poly1305::aead::Payload { msg: plaintext, aad };
    cipher.encrypt(n, payload).map_err(|_| CpError::EncryptFailed)
}

/// Stateless decrypt. Input is ciphertext || tag.
pub fn decrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN],
               aad: &[u8], ciphertext_tag: &[u8]) -> Result<Vec<u8>, CpError> {
    let cipher = CpInner::new(key.into());
    let n = Nonce::from_slice(nonce);
    let payload = chacha20poly1305::aead::Payload { msg: ciphertext_tag, aad };
    cipher.decrypt(n, payload).map_err(|_| CpError::DecryptFailed)
}
