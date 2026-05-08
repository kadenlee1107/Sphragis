// Bat_OS — AES-128-GCM AND AES-256-GCM with proper tag verification.
//
// Built on RustCrypto's audited `aes` (constant-time AES block) and
// `ghash` (constant-time GF(2^128)) primitives. This replaces the
// pentest-flagged `Aes128::gcm_crypt` which was pure XOR stream with
// no authentication — an MITM could flip any plaintext bit.
//
// AES-256-GCM (`Aes256Gcm`) added in to support the TLS
// 1.3 `TLS_AES_256_GCM_SHA384` cipher suite. Same GCM construction;
// only the AES key size + block primitive differ.
//
// This module is the minimal, correct GCM implementation the TLS 1.3
// record layer needs. Single-shot encrypt / decrypt, no streaming,
// fits inside a TLS record (< 16384 bytes plaintext per RFC 8446).

#![allow(dead_code)]

use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128 as Aes128Block;
use aes::Aes256 as Aes256Block;
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
                    (&data[i..i + 16]).try_into()
                        .expect("gcm: 16-byte slice → [u8; 16] is infallible")));
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
    // /
    /// Input:
    /// nonce — 12 bytes
    /// aad — additional authenticated data (may be empty)
    /// ct_and_tag — the wire bytes: ciphertext concatenated with the
    /// 16-byte tag at the end
    // /
    /// Returns Ok(plaintext_len) if the tag verifies (in which case
    /// the first `ct_and_tag.len() - 16` bytes of `ct_and_tag` have
    /// been overwritten in-place with plaintext), or Err("tag
    /// mismatch") if authentication fails.
    // /
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

/// AES-256-GCM context. Same construction as `Aes128Gcm` (GCM is
/// agnostic to AES key size — it just uses E_K as a black box), but
/// holds an `Aes256Block` instead of `Aes128Block`. Used by the TLS
/// 1.3 `TLS_AES_256_GCM_SHA384` cipher suite.
// /
/// The method bodies are intentionally identical to `Aes128Gcm`'s —
/// `encrypt_block` is provided by the `BlockEncrypt` trait that both
/// AES variants implement. We don't share a generic impl because the
/// few-method duplication is cheaper than the trait-bounds + lifetimes
/// dance, and keeps the call sites concrete (no monomorphization
/// surprises in `no_std`).
pub struct Aes256Gcm {
    aes: Aes256Block,
    h: [u8; 16],
}

impl Aes256Gcm {
    /// Build a new GCM context from a 32-byte key.
    pub fn new(key: &[u8; 32]) -> Self {
        let aes = Aes256Block::new(key.into());
        // H = E_K(0^128). The GHASH subkey.
        let mut h_block = [0u8; 16];
        aes.encrypt_block((&mut h_block).into());
        Aes256Gcm { aes, h: h_block }
    }

    /// Counter-mode encrypt / decrypt (symmetric). See `Aes128Gcm`
    /// for the J0 / counter layout.
    fn ctr_crypt(&self, nonce: &[u8; 12], buf: &mut [u8]) {
        let mut counter: u32 = 2;
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

    /// GHASH over (AAD || ciphertext || len(AAD)_64 || len(C)_64).
    fn ghash(&self, aad: &[u8], ct: &[u8]) -> [u8; 16] {
        let mut g = GHash::new(&self.h.into());
        let mut buf = [0u8; 16];
        let mut feed = |g: &mut GHash, data: &[u8]| {
            let mut i = 0;
            while i + 16 <= data.len() {
                g.update(core::slice::from_ref(
                    (&data[i..i + 16]).try_into()
                        .expect("gcm: 16-byte slice → [u8; 16] is infallible")));
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
        let mut lens = [0u8; 16];
        lens[..8].copy_from_slice(&((aad.len() as u64) * 8).to_be_bytes());
        lens[8..16].copy_from_slice(&((ct.len() as u64) * 8).to_be_bytes());
        g.update(core::slice::from_ref((&lens).into()));
        let out = g.finalize();
        out.into()
    }

    /// Decrypt + verify tag (constant-time compare).
    /// See `Aes128Gcm::decrypt_inplace` for full semantics.
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

        let mut expected = self.ghash(aad, ct);

        let mut j0 = [0u8; 16];
        j0[..12].copy_from_slice(nonce);
        j0[15] = 1;
        let mut ek_j0 = j0;
        self.aes.encrypt_block((&mut ek_j0).into());
        for i in 0..16 { expected[i] ^= ek_j0[i]; }

        let mut diff: u8 = 0;
        for i in 0..16 { diff |= expected[i] ^ tag_slice[i]; }
        if diff != 0 { return Err("GCM tag mismatch"); }

        self.ctr_crypt(nonce, ct);
        Ok(ct_len)
    }

    /// Encrypt + append 16-byte tag. Returns plaintext_len + 16.
    pub fn encrypt_inplace(
        &self,
        nonce: &[u8; 12],
        aad: &[u8],
        buf: &mut [u8],
        plaintext_len: usize,
    ) -> usize {
        assert!(buf.len() >= plaintext_len + 16,
            "encrypt_inplace: buffer too small for ciphertext + tag");
        self.ctr_crypt(nonce, &mut buf[..plaintext_len]);
        let mut tag = self.ghash(aad, &buf[..plaintext_len]);
        let mut j0 = [0u8; 16];
        j0[..12].copy_from_slice(nonce);
        j0[15] = 1;
        let mut ek_j0 = j0;
        self.aes.encrypt_block((&mut ek_j0).into());
        for i in 0..16 { tag[i] ^= ek_j0[i]; }
        buf[plaintext_len..plaintext_len + 16].copy_from_slice(&tag);
        plaintext_len + 16
    }
}

/// Known-answer self-test for both AES-128-GCM and AES-256-GCM.
/// Vectors taken from NIST SP 800-38D Test Cases 2 (AES-128) and 14
/// (AES-256). Returns Ok(()) on full pass, or Err with the failing
/// case name. Called from the kernel self-test on the Apple boot
/// path so we can verify on real M4 silicon that both ciphers
/// round-trip and that the GHASH+CTR construction reproduces the
/// published tags.
pub fn selftest() -> Result<(), &'static str> {
    // AES-128-GCM, NIST Test Case 2 ----------------------------
    // K = 00..00 (16 zero bytes)
    // IV = 00..00 (12 zero bytes)
    // A = empty
    // P = 00..00 (16 zero bytes)
    // C = 0388dace60b6a392f328c2b971b2fe78
    // T = ab6e47d42cec13bdf53a67b21257bddf
    {
        let key = [0u8; 16];
        let nonce = [0u8; 12];
        let mut buf = [0u8; 32]; // 16 plaintext + 16 tag
        let g = Aes128Gcm::new(&key);
        let n = g.encrypt_inplace(&nonce, &[], &mut buf, 16);
        if n != 32 { return Err("aes128gcm: encrypt length"); }
        let expect_ct: [u8; 16] = [
            0x03, 0x88, 0xda, 0xce, 0x60, 0xb6, 0xa3, 0x92,
            0xf3, 0x28, 0xc2, 0xb9, 0x71, 0xb2, 0xfe, 0x78,
        ];
        let expect_tag: [u8; 16] = [
            0xab, 0x6e, 0x47, 0xd4, 0x2c, 0xec, 0x13, 0xbd,
            0xf5, 0x3a, 0x67, 0xb2, 0x12, 0x57, 0xbd, 0xdf,
        ];
        if buf[..16] != expect_ct { return Err("aes128gcm: ciphertext mismatch"); }
        if buf[16..32] != expect_tag { return Err("aes128gcm: tag mismatch"); }
        // Round-trip: decrypt back to zeros.
        let pt_len = g.decrypt_inplace(&nonce, &[], &mut buf)
            .map_err(|_| "aes128gcm: decrypt rejected valid tag")?;
        if pt_len != 16 { return Err("aes128gcm: decrypt length"); }
        if buf[..16] != [0u8; 16] { return Err("aes128gcm: roundtrip plaintext"); }
        // Tamper test: flip a ciphertext bit, expect tag mismatch.
        let mut tampered = [0u8; 32];
        tampered[..16].copy_from_slice(&expect_ct);
        tampered[16..32].copy_from_slice(&expect_tag);
        tampered[0] ^= 1;
        if g.decrypt_inplace(&nonce, &[], &mut tampered).is_ok() {
            return Err("aes128gcm: accepted tampered ciphertext");
        }
    }

    // AES-256-GCM, NIST Test Case 14 ---------------------------
    // K = 00..00 (32 zero bytes)
    // IV = 00..00 (12 zero bytes)
    // A = empty
    // P = 00..00 (16 zero bytes)
    // C = cea7403d4d606b6e074ec5d3baf39d18
    // T = d0d1c8a799996bf0265b98b5d48ab919
    {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let mut buf = [0u8; 32];
        let g = Aes256Gcm::new(&key);
        let n = g.encrypt_inplace(&nonce, &[], &mut buf, 16);
        if n != 32 { return Err("aes256gcm: encrypt length"); }
        let expect_ct: [u8; 16] = [
            0xce, 0xa7, 0x40, 0x3d, 0x4d, 0x60, 0x6b, 0x6e,
            0x07, 0x4e, 0xc5, 0xd3, 0xba, 0xf3, 0x9d, 0x18,
        ];
        let expect_tag: [u8; 16] = [
            0xd0, 0xd1, 0xc8, 0xa7, 0x99, 0x99, 0x6b, 0xf0,
            0x26, 0x5b, 0x98, 0xb5, 0xd4, 0x8a, 0xb9, 0x19,
        ];
        if buf[..16] != expect_ct { return Err("aes256gcm: ciphertext mismatch"); }
        if buf[16..32] != expect_tag { return Err("aes256gcm: tag mismatch"); }
        // Round-trip.
        let pt_len = g.decrypt_inplace(&nonce, &[], &mut buf)
            .map_err(|_| "aes256gcm: decrypt rejected valid tag")?;
        if pt_len != 16 { return Err("aes256gcm: decrypt length"); }
        if buf[..16] != [0u8; 16] { return Err("aes256gcm: roundtrip plaintext"); }
        // Tamper test on the tag byte.
        let mut tampered = [0u8; 32];
        tampered[..16].copy_from_slice(&expect_ct);
        tampered[16..32].copy_from_slice(&expect_tag);
        tampered[31] ^= 0x80;
        if g.decrypt_inplace(&nonce, &[], &mut tampered).is_ok() {
            return Err("aes256gcm: accepted tampered tag");
        }
    }

    Ok(())
}
