//! Sphragis — CNSA 2.0 post-quantum primitives.
//!
//! This module exposes **ML-KEM-1024** (FIPS 203) and **ML-DSA-87**
//! (FIPS 204) as standalone CNSA-2.0-compliant KEM and signature
//! services for non-TLS contexts: BatFS at-rest key wrap, audit-ring
//! seal, attestation quote signing, cave-mediated IPC sealing.
//!
//! ## Why a separate module from `pq_hybrid.rs` / `pq_hybrid_sig.rs`?
//!
//! Those files implement TLS PQ-hybrid per
//! `draft-ietf-tls-ecdhe-mlkem-04` (codepoint `X25519MLKEM768`,
//! 0x11EC). That codepoint mandates **ML-KEM-768** + X25519, and the
//! companion signature uses **ML-DSA-65**. Both are NIST security
//! category 3.
//!
//! **CNSA 2.0** (NSA, May 2025) mandates the **category 5** variants
//! (ML-KEM-1024, ML-DSA-87) for new National Security System
//! acquisitions starting 2027-01-01, exclusive use by 2033. There is
//! no IETF-standardized TLS hybrid codepoint for ML-KEM-1024 yet, so
//! the existing TLS path stays on 768/65 for interop reality. Every
//! Sphragis surface that ISN'T constrained by TLS-peer interop —
//! BatFS, audit, IPC, attestation — routes through this module
//! instead, automatically clearing the CNSA 2.0 bar.
//!
//! See `ANTI_FEATURES.md` §ANTI-005 (no weak crypto in gov build) and
//! the Sphragis gov-OS requirements spec (REQ-CRY-001, CRY-002).
//!
//! ## Wire sizes (per FIPS 203 / FIPS 204)
//!
//! | Item                    | Bytes |
//! |-------------------------|-------|
//! | ML-KEM-1024 encap key   | 1568  |
//! | ML-KEM-1024 decap key   | 3168  |
//! | ML-KEM-1024 ciphertext  | 1568  |
//! | ML-KEM-1024 shared sec  | 32    |
//! | ML-DSA-87 public key    | 2592  |
//! | ML-DSA-87 signature     | 4627  |
//!
//! ## Format
//!
//! Wire layouts are raw FIPS 203 / FIPS 204 byte encodings. There is
//! no hybrid X25519 layer here (CNSA 2.0 is PQ-only by 2033; the
//! hybrid story belongs in `pq_hybrid.rs` for TLS interop). Callers
//! who want hybrid-with-X25519 today should keep using `pq_hybrid.rs`
//! and migrate when ready.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use ml_kem::{KemCore, MlKem1024, EncodedSizeUser, Encoded};
use ml_kem::kem::{Encapsulate, Decapsulate};
use ml_dsa::{KeyGen, MlDsa87, SigningKey, VerifyingKey, Signature as MlDsaSignature, B32};
use ml_dsa::{EncodedSignature, EncodedVerifyingKey};

use crate::crypto::pq_hybrid::KernelRng;
use crate::crypto::rng;

// ── ML-KEM-1024 (FIPS 203) wire sizes ────────────────────────────

/// ML-KEM-1024 encapsulation (public) key length per FIPS 203.
pub const MLKEM1024_EK_LEN: usize = 1568;
/// ML-KEM-1024 decapsulation (private) key length per FIPS 203.
pub const MLKEM1024_DK_LEN: usize = 3168;
/// ML-KEM-1024 ciphertext length per FIPS 203.
pub const MLKEM1024_CT_LEN: usize = 1568;
/// ML-KEM-1024 shared-secret length per FIPS 203.
pub const MLKEM1024_SS_LEN: usize = 32;

// ── ML-DSA-87 (FIPS 204) wire sizes ──────────────────────────────

/// ML-DSA-87 public (verifying) key length per FIPS 204.
pub const MLDSA87_PK_LEN: usize = 2592;
/// ML-DSA-87 signature length per FIPS 204.
pub const MLDSA87_SIG_LEN: usize = 4627;

// ── ML-KEM-1024 KEM ──────────────────────────────────────────────

/// ML-KEM-1024 keypair: caller keeps both halves; only the
/// encapsulation (public) half goes on the wire.
pub struct Kem1024Key {
    pub dk: <MlKem1024 as KemCore>::DecapsulationKey,
    pub ek: <MlKem1024 as KemCore>::EncapsulationKey,
}

impl Kem1024Key {
    /// Generate a fresh ML-KEM-1024 keypair using the RNDR-backed CSPRNG.
    pub fn generate() -> Self {
        let mut r = KernelRng;
        let (dk, ek) = <MlKem1024 as KemCore>::generate(&mut r);
        Self { dk, ek }
    }

    /// Serialize the encapsulation key for transmission (1568 B).
    pub fn ek_bytes(&self) -> Vec<u8> {
        self.ek.as_bytes().as_slice().to_vec()
    }

    /// Serialize the decapsulation key for storage (3168 B).
    pub fn dk_bytes(&self) -> Vec<u8> {
        self.dk.as_bytes().as_slice().to_vec()
    }
}

/// Sender-side ML-KEM-1024 encapsulation. Given a recipient's
/// 1568-byte encapsulation key, produce `(shared_secret, ciphertext)`
/// where the ciphertext is 1568 B for transmission and the shared
/// secret is 32 B for KDF input.
pub fn encapsulate_1024(recipient_ek: &[u8])
    -> Result<([u8; MLKEM1024_SS_LEN], Vec<u8>), &'static str>
{
    use ml_kem::array::Array as KemArray;
    if recipient_ek.len() != MLKEM1024_EK_LEN {
        return Err("pq_cnsa: bad ML-KEM-1024 encap key length");
    }
    type Ek = <MlKem1024 as KemCore>::EncapsulationKey;
    let ek_arr: Encoded<Ek> = KemArray::try_from(recipient_ek)
        .map_err(|_| "pq_cnsa: ML-KEM-1024 encap key byte mismatch")?;
    let ek = <Ek as EncodedSizeUser>::from_bytes(&ek_arr);

    let mut r = KernelRng;
    let (ct_arr, ss_arr) = ek.encapsulate(&mut r)
        .map_err(|_| "pq_cnsa: ML-KEM-1024 encapsulate failed")?;

    let mut ss = [0u8; MLKEM1024_SS_LEN];
    ss.copy_from_slice(ss_arr.as_slice());
    let ct = ct_arr.as_slice().to_vec();
    Ok((ss, ct))
}

/// Receiver-side ML-KEM-1024 decapsulation. Given the receiver's
/// keypair and a 1568-byte ciphertext, recover the 32-byte shared
/// secret.
pub fn decapsulate_1024(kp: &Kem1024Key, ciphertext: &[u8])
    -> Result<[u8; MLKEM1024_SS_LEN], &'static str>
{
    use ml_kem::array::Array as KemArray;
    if ciphertext.len() != MLKEM1024_CT_LEN {
        return Err("pq_cnsa: bad ML-KEM-1024 ciphertext length");
    }
    let ct_arr: ml_kem::Ciphertext<MlKem1024> = KemArray::try_from(ciphertext)
        .map_err(|_| "pq_cnsa: ML-KEM-1024 ciphertext byte mismatch")?;
    let ss_arr = kp.dk.decapsulate(&ct_arr)
        .map_err(|_| "pq_cnsa: ML-KEM-1024 decapsulate failed")?;
    let mut ss = [0u8; MLKEM1024_SS_LEN];
    ss.copy_from_slice(ss_arr.as_slice());
    Ok(ss)
}

// ── ML-DSA-87 signatures ─────────────────────────────────────────
//
// Note: ml-dsa uses `hybrid-array` 0.2 internally; ml-kem uses 0.4.
// They aren't interchangeable, so we keep import-aliases scoped per
// function and never mix encoded types across the two algorithms.

/// ML-DSA-87 signing key. The verifying-key half can be derived via
/// `verifying_bytes()`.
pub struct Dsa87Key {
    pub sk: SigningKey<MlDsa87>,
}

impl Dsa87Key {
    /// Generate a fresh ML-DSA-87 signing key seeded from the RNDR-backed CSPRNG.
    pub fn generate() -> Self {
        let mut seed_bytes = [0u8; 32];
        rng::fill_bytes(&mut seed_bytes);
        let seed = B32::from(seed_bytes);
        let sk: SigningKey<MlDsa87> = <MlDsa87 as KeyGen>::from_seed(&seed);
        Self { sk }
    }

    /// Serialize the verifying (public) key for distribution (2592 B).
    pub fn verifying_bytes(&self) -> Vec<u8> {
        let vk = self.sk.signing_key().verifying_key();
        vk.encode().as_slice().to_vec()
    }

    /// Produce a 4627-byte ML-DSA-87 signature over `msg`. Uses
    /// deterministic signing with the Sphragis domain-separator
    /// context (matches the pq_hybrid_sig pattern).
    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, &'static str> {
        let sig: MlDsaSignature<MlDsa87> = self.sk.signing_key()
            .sign_deterministic(msg, b"SPHRAGIS-CNSA-MLDSA87")
            .map_err(|_| "pq_cnsa: ML-DSA-87 sign failed")?;
        Ok(sig.encode().as_slice().to_vec())
    }
}

/// Verify an ML-DSA-87 signature. Returns `Ok(())` if `sig` is a
/// valid signature on `msg` under `verifying_key`. Uses the same
/// domain-separator context as `Dsa87Key::sign`.
pub fn verify_mldsa87(verifying_key: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use hybrid_array::Array;
    if verifying_key.len() != MLDSA87_PK_LEN {
        return Err("pq_cnsa: bad ML-DSA-87 public key length");
    }
    if sig.len() != MLDSA87_SIG_LEN {
        return Err("pq_cnsa: bad ML-DSA-87 signature length");
    }
    let vk_arr: EncodedVerifyingKey<MlDsa87> = Array::try_from(verifying_key)
        .map_err(|_| "pq_cnsa: ML-DSA-87 verifying-key byte mismatch")?;
    let vk: VerifyingKey<MlDsa87> = VerifyingKey::decode(&vk_arr);

    let sig_arr: EncodedSignature<MlDsa87> = Array::try_from(sig)
        .map_err(|_| "pq_cnsa: ML-DSA-87 signature byte mismatch")?;
    let dsa_sig = MlDsaSignature::<MlDsa87>::decode(&sig_arr)
        .ok_or("pq_cnsa: ML-DSA-87 signature decode failed")?;

    if !vk.verify_with_context(msg, b"SPHRAGIS-CNSA-MLDSA87", &dsa_sig) {
        return Err("pq_cnsa: ML-DSA-87 verify failed");
    }
    Ok(())
}

// ── Boot-time self-tests (closes REQ-CRY-006 partial for these algos) ──

/// Round-trip KAT: generate a key, encapsulate, decapsulate, assert
/// shared secrets match. Called from `crypto::run_self_tests()` at
/// boot; the kernel `unwrap()`s the result so any failure halts boot
/// (gov build: KAT failure must be fail-closed).
pub fn kat_mlkem1024() -> Result<(), &'static str> {
    let kp = Kem1024Key::generate();
    let ek = kp.ek_bytes();
    let (ss_send, ct) = encapsulate_1024(&ek)?;
    let ss_recv = decapsulate_1024(&kp, &ct)?;
    if ss_send != ss_recv {
        return Err("KAT-FAIL: ML-KEM-1024 shared secrets diverged");
    }
    Ok(())
}

/// Sign-verify KAT plus a tamper-detection check: the verifier must
/// reject a bit-flipped message.
pub fn kat_mldsa87() -> Result<(), &'static str> {
    let kp = Dsa87Key::generate();
    let vk = kp.verifying_bytes();
    let msg = b"sphragis ML-DSA-87 KAT vector 2026-05-16";
    let sig = kp.sign(msg)?;
    verify_mldsa87(&vk, msg, &sig)?;
    // Bit-flip the message and assert verify fails.
    let mut bad_msg = msg.to_vec();
    bad_msg[0] ^= 0x01;
    if verify_mldsa87(&vk, &bad_msg, &sig).is_ok() {
        return Err("KAT-FAIL: ML-DSA-87 accepted modified message");
    }
    Ok(())
}
