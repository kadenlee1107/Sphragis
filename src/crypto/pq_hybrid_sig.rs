//! DESIGN_CRYPTO.md #6 — Post-quantum hybrid signatures.
//!
//! Companion to `pq_hybrid.rs` (key exchange). Same threat model,
//! different primitive: signatures. Classical Ed25519 breaks under
//! Shor on a CRQC. The fix is a HYBRID signature — sign with BOTH
//! Ed25519 and ML-DSA-65, concatenate, and require BOTH to verify
//! for acceptance.
//!
//! Security contract: a forgery requires breaking BOTH primitives.
//! Classical attacker who somehow defeats Ed25519 still fails against
//! ML-DSA; future CRQC that defeats ML-DSA still fails against
//! Ed25519 against *pre-CRQC signed* messages. Long-term trust roots
//! (kernel signing, initrd signing, release manifest signing) are the
//! canonical use case.
//!
//! Primitive choice: **Ed25519 + ML-DSA-65 (NIST FIPS 204)**
//!   * Ed25519 sig: 64 B, pub 32 B — tiny, fast, battle-tested
//!   * ML-DSA-65 sig: 3309 B, pub 1952 B — NIST Cat 3 (AES-192)
//!   * On M4 aarch64: Ed25519 sign/verify ~50 µs, ML-DSA-65 ~300 µs.
//!     Combined ~400 µs per verify. Trivial at trust-root scale.
//!
//! On-wire layout of a hybrid signature:
//!     ed25519_sig (64 B) || ml_dsa_sig (3309 B)   = 3373 B total
//!
//! On-wire layout of a hybrid public key:
//!     ed25519_pub (32 B) || ml_dsa_pub (1952 B)   = 1984 B total
//!
//! TLS-side wiring (hybrid in a CertificateVerify / cert chain
//! signature) lives as a follow-up. This module gives us the
//! primitive + a shell self-test so correctness is provable.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;

use ml_dsa::{KeyGen, MlDsa65, SigningKey, VerifyingKey, EncodedVerifyingKey,
    Signature as MlDsaSignature, B32};

use crate::crypto::rng;

/// Ed25519 signature size.
pub const ED25519_SIG_LEN: usize = 64;
/// Ed25519 public key size.
pub const ED25519_PUB_LEN: usize = 32;
/// ML-DSA-65 signature size.
pub const MLDSA_SIG_LEN: usize = 3309;
/// ML-DSA-65 verifying (public) key size.
pub const MLDSA_PUB_LEN: usize = 1952;

pub const HYBRID_SIG_LEN: usize = ED25519_SIG_LEN + MLDSA_SIG_LEN;
pub const HYBRID_PUB_LEN: usize = ED25519_PUB_LEN + MLDSA_PUB_LEN;

pub struct HybridSigKeyPair {
    pub ed25519_sk: [u8; 64],         // ed25519-compact seed+pub concat
    pub ed25519_pk: [u8; ED25519_PUB_LEN],
    pub mldsa_sk:   SigningKey<MlDsa65>,
}

impl HybridSigKeyPair {
    pub fn generate() -> Self {
        use ed25519_compact::{KeyPair, Seed};
        let mut seed_bytes = [0u8; 32];
        rng::fill_bytes(&mut seed_bytes);
        let ed_kp = KeyPair::from_seed(Seed::new(seed_bytes));
        let ed25519_sk = *ed_kp.sk;
        let ed25519_pk = *ed_kp.pk;

        // ML-DSA-65 keygen via `from_seed` (deterministic; we seed from
        // our RNDR-backed CSPRNG). Avoids threading a CryptoRng impl.
        let mut seed = B32::default();
        rng::fill_bytes(seed.as_mut_slice());
        let mldsa_sk: SigningKey<MlDsa65> = <MlDsa65 as KeyGen>::from_seed(&seed);

        Self { ed25519_sk, ed25519_pk, mldsa_sk }
    }

    pub fn public_bytes(&self) -> Vec<u8> {
        let mut out = vec![0u8; HYBRID_PUB_LEN];
        out[..ED25519_PUB_LEN].copy_from_slice(&self.ed25519_pk);
        // verifying_key() lives on ExpandedSigningKey; get there via
        // signing_key() which hands out the expanded form.
        let vk = self.mldsa_sk.signing_key().verifying_key();
        let vk_bytes = vk.encode();
        out[ED25519_PUB_LEN..].copy_from_slice(vk_bytes.as_slice());
        out
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, &'static str> {
        use ed25519_compact::SecretKey;
        let sk = SecretKey::from_slice(&self.ed25519_sk)
            .map_err(|_| "bad Ed25519 secret key")?;
        let ed_sig = sk.sign(msg, None);

        let mldsa_sig: MlDsaSignature<MlDsa65> = self.mldsa_sk.signing_key()
            .sign_deterministic(msg, b"BATOS-HYBRID-SIG")
            .map_err(|_| "ML-DSA sign failed")?;
        let mldsa_enc = mldsa_sig.encode();

        let mut out = vec![0u8; HYBRID_SIG_LEN];
        out[..ED25519_SIG_LEN].copy_from_slice(ed_sig.as_slice());
        out[ED25519_SIG_LEN..].copy_from_slice(mldsa_enc.as_slice());
        Ok(out)
    }
}

/// Verify a hybrid signature. BOTH primitives must verify for
/// acceptance — if EITHER fails, we reject.
pub fn verify(pub_bytes: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use ed25519_compact::{PublicKey, Signature};
    use ml_dsa::{VerifyingKey, EncodedVerifyingKey};

    if pub_bytes.len() != HYBRID_PUB_LEN { return Err("hybrid-sig: bad pub len"); }
    if sig.len() != HYBRID_SIG_LEN { return Err("hybrid-sig: bad sig len"); }

    // ── Classical half: Ed25519 ──
    let ed_pk = PublicKey::from_slice(&pub_bytes[..ED25519_PUB_LEN])
        .map_err(|_| "hybrid-sig: bad Ed25519 pub")?;
    let ed_sig = Signature::from_slice(&sig[..ED25519_SIG_LEN])
        .map_err(|_| "hybrid-sig: bad Ed25519 sig")?;
    ed_pk.verify(msg, &ed_sig).map_err(|_| "hybrid-sig: Ed25519 verify failed")?;

    // ── PQ half: ML-DSA-65 ──
    use hybrid_array::Array;
    use ml_dsa::EncodedSignature;

    let vk_bytes: &[u8] = &pub_bytes[ED25519_PUB_LEN..];
    let vk_arr: EncodedVerifyingKey<MlDsa65> = Array::try_from(vk_bytes)
        .map_err(|_| "hybrid-sig: ML-DSA pub length mismatch")?;
    let vk: VerifyingKey<MlDsa65> = VerifyingKey::decode(&vk_arr);

    let dsa_sig_bytes: &[u8] = &sig[ED25519_SIG_LEN..];
    let dsa_sig_arr: EncodedSignature<MlDsa65> = Array::try_from(dsa_sig_bytes)
        .map_err(|_| "hybrid-sig: ML-DSA sig length mismatch")?;
    let dsa_sig = MlDsaSignature::<MlDsa65>::decode(&dsa_sig_arr)
        .ok_or("hybrid-sig: ML-DSA sig decode failed")?;

    if !vk.verify_with_context(msg, b"BATOS-HYBRID-SIG", &dsa_sig) {
        return Err("hybrid-sig: ML-DSA verify failed");
    }
    Ok(())
}

/// Exposed as `pq-sig-selftest` shell command.
///
/// 1. Gen keypair.
/// 2. Sign a test message.
/// 3. Verify — expect Ok.
/// 4. Flip a byte in the signature — expect Err.
/// 5. Report sizes + prefix.
pub fn selftest() -> Result<(usize, usize, [u8; 8]), &'static str> {
    let kp = HybridSigKeyPair::generate();
    let pub_bytes = kp.public_bytes();
    let msg = b"test message from BatOS pq-sig-selftest";
    let mut sig = kp.sign(msg)?;
    verify(&pub_bytes, msg, &sig)?;
    // Tamper: flip a bit in the Ed25519 half
    sig[10] ^= 0x01;
    if verify(&pub_bytes, msg, &sig).is_ok() {
        return Err("hybrid-sig: tampered sig verified (BUG)");
    }
    sig[10] ^= 0x01; // restore
    // Tamper: flip a bit in the ML-DSA half (catch the "only checked
    // one primitive" bug)
    sig[ED25519_SIG_LEN + 500] ^= 0x01;
    if verify(&pub_bytes, msg, &sig).is_ok() {
        return Err("hybrid-sig: tampered ML-DSA verified (BUG)");
    }
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&sig[..8]);
    Ok((HYBRID_PUB_LEN, HYBRID_SIG_LEN, prefix))
}
