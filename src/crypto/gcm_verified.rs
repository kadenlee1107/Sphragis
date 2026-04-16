// Bat_OS — AES-128-GCM with proper tag verification.
//
// Built on RustCrypto's audited `aes` (constant-time AES block) and
// `ghash` (constant-time GF(2^128)) primitives. This replaces the
// pentest-flagged `Aes128::gcm_crypt` which was pure XOR stream with
// no authentication — an MITM could flip any plaintext bit.
//
// This module is the minimal, correct AES-128-GCM implementation the
// TLS 1.3 record layer needs. Single-shot encrypt / decrypt, no
// streaming, fits inside a TLS record (< 16384 bytes plaintext per
// RFC 8446).

#![allow(dead_code)]

use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128 as Aes128Block;
use ghash::universal_hash::UniversalHash;
use ghash::GHash;

/// AES-128-GCM context. Holds the AES block key and the derived
/// H (= E_K(0^128)) used by GHASH.
pub struct Aes128Gcm {
    aes: Aes128Block,
    h: [u8; 16],
}

impl Aes128Gcm {
    /// Build a new GCM context from a 16-byte key.
    pub fn new(key: &[u8; 16]) -> Self {
        let aes = Aes128Block::new(key.into());
        // H = E_K(0^128). The GHASH subkey.
        let mut h_block = [0u8; 16];
        aes.encrypt_block((&mut h_block).into());
        Aes128Gcm { aes, h: h_block }
    }

    /// Counter-mode encrypt / decrypt (they're symmetric). Nonce is 12
    /// bytes; J0 = nonce || 0x00000001 for the initial counter block
    /// (first one AFTER J0 encrypts the tag). Subsequent blocks
    /// increment the low 32 bits.
    fn ctr_crypt(&self, nonce: &[u8; 12], buf: &mut [u8]) {
        let mut counter: u32 = 2; // start at 2; counter 1 is used for the tag
        let mut block = [0u8; 16];
        let n = buf.len();
        let mut off = 0;
        while off < n {
            block[..12].copy_from_slice(nonce);
            block[12..16].copy_from_slice(&counter.to_be_bytes());
            let mut enc = block;
            self.aes.encrypt_block((&mut enc).into());
            let take = core::cmp::min(16, n - off);
            for i in 0..take {
                buf[off + i] ^= enc[i];
            }
            off += take;
            counter = counter.wrapping_add(1);
        }
    }

    /// Compute the GHASH authentication over (AAD || ciphertext ||
    /// len(AAD)_64 || len(C)_64). Returns the 16-byte GHASH output
    /// (not yet XORed with E_K(J0)).
    fn ghash(&self, aad: &[u8], ct: &[u8]) -> [u8; 16] {
        let mut g = GHash::new(&self.h.into());
        // RustCrypto's GHash processes 16-byte blocks. Pad partial
        // blocks with zeros, which is what the spec requires.
        let mut buf = [0u8; 16];
        let mut feed = |g: &mut GHash, data: &[u8]| {
            let mut i = 0;
            while i + 16 <= data.len() {
                g.update(core::slice::from_ref(
                    (&data[i..i + 16]).try_into().unwrap()));
                i += 16;
            }
            if i < data.len() {
                buf.fill(0);
                buf[..data.len() - i].copy_from_slice(&data[i..]);
                g.update(core::slice::from_ref((&buf).into()));
            }
        };
        feed(&mut g, aad);
        feed(&mut g, ct);
        // Length block: 64-bit AAD bit length || 64-bit C bit length
        let mut lens = [0u8; 16];
        lens[..8].copy_from_slice(&((aad.len() as u64) * 8).to_be_bytes());
        lens[8..16].copy_from_slice(&((ct.len() as u64) * 8).to_be_bytes());
        g.update(core::slice::from_ref((&lens).into()));
        let out = g.finalize();
        out.into()
    }

    /// Decrypt a GCM ciphertext, verifying the 16-byte tag.
    ///
    /// Input:
    ///   nonce  — 12 bytes
    ///   aad    — additional authenticated data (may be empty)
    ///   ct_and_tag — the wire bytes: ciphertext concatenated with the
    ///                16-byte tag at the end
    ///
    /// Returns Ok(plaintext_len) if the tag verifies (in which case
    /// the first `ct_and_tag.len() - 16` bytes of `ct_and_tag` have
    /// been overwritten in-place with plaintext), or Err("tag
    /// mismatch") if authentication fails.
    ///
    /// Uses constant-time comparison on the tag — no timing oracle.
    pub fn decrypt_inplace(
        &self,
        nonce: &[u8; 12],
        aad: &[u8],
        ct_and_tag: &mut [u8],
    ) -> Result<usize, &'static str> {
        if ct_and_tag.len() < 16 {
            return Err("ct_and_tag too short for tag");
        }
        let ct_len = ct_and_tag.len() - 16;
        let (ct, tag_slice) = ct_and_tag.split_at_mut(ct_len);

        // Compute expected tag over the CIPHERTEXT.
        let mut expected = self.ghash(aad, ct);

        // XOR with E_K(J0) where J0 = nonce || 0x00000001
        let mut j0 = [0u8; 16];
        j0[..12].copy_from_slice(nonce);
        j0[15] = 1;
        let mut ek_j0 = j0;
        self.aes.encrypt_block((&mut ek_j0).into());
        for i in 0..16 { expected[i] ^= ek_j0[i]; }

        // Constant-time compare.
        let mut diff: u8 = 0;
        for i in 0..16 { diff |= expected[i] ^ tag_slice[i]; }
        if diff != 0 { return Err("GCM tag mismatch"); }

        // Tag OK — safe to decrypt in place.
        self.ctr_crypt(nonce, ct);
        Ok(ct_len)
    }

    /// Encrypt + append tag. `buf` contains plaintext on entry and
    /// ciphertext||tag on exit. Returns the total written length
    /// (plaintext.len() + 16), which the caller is expected to have
    /// allocated for.
    pub fn encrypt_inplace(
        &self,
        nonce: &[u8; 12],
        aad: &[u8],
        buf: &mut [u8],
        plaintext_len: usize,
    ) -> usize {
        assert!(buf.len() >= plaintext_len + 16,
            "encrypt_inplace: buffer too small for ciphertext + tag");
        // Encrypt plaintext in place.
        self.ctr_crypt(nonce, &mut buf[..plaintext_len]);
        // Compute tag over the ciphertext.
        let mut tag = self.ghash(aad, &buf[..plaintext_len]);
        // XOR with E_K(J0)
        let mut j0 = [0u8; 16];
        j0[..12].copy_from_slice(nonce);
        j0[15] = 1;
        let mut ek_j0 = j0;
        self.aes.encrypt_block((&mut ek_j0).into());
        for i in 0..16 { tag[i] ^= ek_j0[i]; }
        // Append tag.
        buf[plaintext_len..plaintext_len + 16].copy_from_slice(&tag);
        plaintext_len + 16
    }
}
