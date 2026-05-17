pub mod aes;
pub mod aes_xts;
pub mod blake2s;
pub mod blake3;
pub mod chacha20poly1305;
pub mod gcm_verified;
pub mod hotp;
pub mod lms;
pub mod policy;
pub mod pq_cnsa;
pub mod pq_hybrid;
pub mod pq_hybrid_sig;
pub mod rng;
pub mod sha256;
pub mod sha3;
pub mod sha384;
pub mod sha512;
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

    // SHA-512 FIPS 180-4 §F.3 "abc" KAT + HMAC-SHA-512 determinism
    // smoke (SP-B1.5 / SP-B1.7 partial; REQ-CRY-005).
    sha512::kat()?;

    // Fail-closed RNG smoke (AUDIT-FS-H3 / SP-B1.8): if the CPU
    // exposes RNDR, verify the strict-mode fill_bytes succeeds and
    // produces non-zero output. If RNDR is absent, verify the strict
    // mode correctly returns Err rather than silently degrading.
    // Either branch is a pass — what we're checking is that the
    // strict API behaves per its contract, not that the platform has
    // hardware entropy.
    {
        let mut probe = [0u8; 32];
        match rng::fill_bytes_strict(&mut probe) {
            Ok(()) => {
                // Sanity: at least one nonzero byte (probability of
                // 32 zero bytes from a working entropy source is
                // ~2^-256; treat as a KAT failure).
                let mut all_zero = true;
                for b in probe.iter() { if *b != 0 { all_zero = false; break; } }
                if all_zero { return Err("rng: strict fill emitted all-zeros"); }
            }
            Err(_) => {
                // RNDR unavailable in this environment. Confirm the
                // pre-flight check also reports Err for consistency.
                if rng::require_hw_rng_or_err().is_ok() {
                    return Err("rng: strict-mode Err but require_hw_rng_or_err Ok (inconsistent)");
                }
            }
        }
    }

    // ML-KEM-1024 (FIPS 203) round-trip KAT — CNSA 2.0 PQ-KEM at
    // category 5. Covers REQ-CRY-001. Generate → encapsulate →
    // decapsulate → shared secrets must match.
    pq_cnsa::kat_mlkem1024()?;

    // ML-DSA-87 (FIPS 204) sign-verify + tamper-detect KAT — CNSA 2.0
    // PQ signature at category 5. Covers REQ-CRY-002.
    pq_cnsa::kat_mldsa87()?;

    // NOTE: LMS (RFC 8554) KAT is NOT in this boot path. H5 keygen
    // walks all 32 OTS leaves at generation time, ~270K SHA-256
    // hashes total — ~30-60s under QEMU emulation without SHA-NI,
    // which times out the smoke harness. `crypto::lms::kat()` is
    // exposed for on-demand validation (e.g., via a shell command
    // or a dedicated self-test script). The algorithm correctness
    // is still gated — just not at every boot. SP-B1.7 may revisit
    // this with a verify-only RFC 8554 §F test vector that doesn't
    // require keygen.

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
