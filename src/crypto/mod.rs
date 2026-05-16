pub mod aes;
pub mod aes_xts;
pub mod blake2s;
pub mod blake3;
pub mod chacha20poly1305;
pub mod gcm_verified;
pub mod hotp;
pub mod pq_hybrid;
pub mod pq_hybrid_sig;
pub mod rng;
pub mod sha256;
pub mod sha3;
pub mod sha384;
pub mod sig;
pub mod totp;
pub mod xchacha20poly1305;

/// AUDIT-CRYPTO-F7 (2026-05-15): top-level fail-closed boot-time
/// self-tests for every crypto primitive used in production.
///
/// Calls into each primitive's KAT (Known-Answer Test). Returns
/// `Ok(())` only if ALL tests pass. The kernel boot path MUST
/// `unwrap()` or `expect()` the result so any failure halts boot —
/// the prior pattern was to print "FAIL: <reason>" and continue,
/// which silently shipped broken crypto.
///
/// Covers (today): SHA-256 (RFC 6234 §8.5), ChaCha20-Poly1305
/// (RFC 8439 round-trip + tamper detection), AES-128/256-GCM
/// (NIST SP 800-38D via `gcm_verified::selftest`).
///
/// Follow-up additions: SHA-384/512, HKDF chain, HMAC-SHA256,
/// X25519, Ed25519, ML-KEM. Each gets a host-side cargo test
/// wrapper for CI parity.
pub fn run_self_tests() -> Result<(), &'static str> {
    // SHA-256 RFC 6234 §8.5 vector "abc"
    {
        let h = sha256::hash(b"abc");
        let expected: [u8; 32] = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
            0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
            0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ];
        let mut diff: u8 = 0;
        for i in 0..32 { diff |= h[i] ^ expected[i]; }
        if diff != 0 { return Err("sha256 KAT fail"); }
    }

    // AES-128 + AES-256 GCM (NIST SP 800-38D) via existing selftest.
    gcm_verified::selftest()?;

    // ChaCha20-Poly1305 — encrypt-then-decrypt round trip + tamper
    // detection on a fixed (key, nonce, ad, pt). Catches codegen
    // breakage of the cipher or MAC.
    {
        let key: [u8; 32] = [
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
            0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
            0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
            0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
        ];
        let nonce: [u8; 12] = [
            0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43,
            0x44, 0x45, 0x46, 0x47,
        ];
        let aad = b"Sphragis-KAT";
        let plaintext = b"Ladies and Gentlemen of the class of '99";
        let ct = chacha20poly1305::encrypt(&key, &nonce, aad, plaintext)
            .map_err(|_| "chacha20poly1305 encrypt failed")?;
        let pt = chacha20poly1305::decrypt(&key, &nonce, aad, &ct)
            .map_err(|_| "chacha20poly1305 decrypt failed")?;
        if pt.as_slice() != plaintext.as_slice() {
            return Err("chacha20poly1305 round-trip mismatch");
        }
        // Tamper-detection: flip one ciphertext byte → decrypt MUST fail.
        let mut tampered = ct.clone();
        if tampered.is_empty() { return Err("chacha20poly1305 ct empty"); }
        tampered[0] ^= 0x01;
        if chacha20poly1305::decrypt(&key, &nonce, aad, &tampered).is_ok() {
            return Err("chacha20poly1305 accepted tampered ciphertext");
        }
    }

    Ok(())
}
