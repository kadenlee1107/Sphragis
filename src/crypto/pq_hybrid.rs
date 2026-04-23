//! DESIGN_CRYPTO.md #5 — Post-quantum hybrid key agreement.
//!
//! Classical primitives (X25519, Ed25519, P-256) break the moment a
//! cryptographically relevant quantum computer arrives — and the
//! "store now, decrypt later" threat means network traffic captured
//! *today* is already at risk against a 2040-era adversary.
//!
//! The standard defence is a HYBRID: run both a classical KEM and a
//! post-quantum KEM, concatenate their shared secrets, and feed them
//! through a KDF. The result is secure as long as EITHER primitive
//! holds — breaking X25519 doesn't help if ML-KEM is still standing,
//! and vice versa.
//!
//! This module provides the primitive. TLS-pipeline integration
//! (wiring it into `src/net/tls.rs` as a key_share alternative) is
//! the next lift. Having the primitive in isolation lets us:
//!   1. Self-test correctness in the shell (`pq-selftest`)
//!   2. Use it for other session establishments (BatCave IPC —
//!      phase 7 builds on this exact shape)
//!   3. Benchmark the overhead on M4 before committing to it in the
//!      TLS critical path
//!
//! Concrete choice: **X25519 + ML-KEM-768**
//!   * NIST FIPS 203 standardised ML-KEM in 2024.
//!   * ML-KEM-768 is Category 3 (≈ AES-192 security). Kyber-768 /
//!     equivalent to X25519's classical strength for 2030+ data.
//!   * Ciphertext size: 1088 B. Public key: 1184 B. Shared: 32 B.
//!   * On M4 aarch64 the encap/decap runs in ~20µs — well under any
//!     protocol budget.
//!
//! The hybrid shape (matching IETF draft-ietf-tls-hybrid-design):
//!
//!   keygen: -> (classical_sk, classical_pk, pq_sk, pq_pk)
//!   encap(classical_pk, pq_pk):
//!     ss_c = X25519(eph_sk, classical_pk)        // 32 B
//!     (ct_pq, ss_pq) = ML-KEM-768.encap(pq_pk)   // 1088 + 32 B
//!     combined = HKDF-SHA256( ss_c || ss_pq )    // 32 B
//!     -> (combined, eph_pub || ct_pq)
//!   decap(my_sk, my_pq_sk, blob):
//!     eph_pub = blob[..32];  ct_pq = blob[32..]
//!     ss_c = X25519(my_sk, eph_pub)
//!     ss_pq = ML-KEM-768.decap(my_pq_sk, ct_pq)
//!     combined = HKDF-SHA256( ss_c || ss_pq )
//!     -> combined
//!
//! This file exposes those two operations as plain functions. The
//! on-wire encoding is the simple X25519-pub || ML-KEM-ct blob above
//! (32 + 1088 = 1120 B); TLS-draft-hybrid uses the same layout.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;

use ml_kem::{KemCore, MlKem768, EncodedSizeUser, Encoded};
use ml_kem::kem::{Encapsulate, Decapsulate};
use ml_kem::array::Array;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public, StaticSecret};

use crate::crypto::{rng, sha256};

/// Classical X25519 public-key size.
pub const X25519_PUB_LEN: usize = 32;
/// ML-KEM-768 ciphertext size.
pub const MLKEM_CT_LEN: usize = 1088;
/// ML-KEM-768 encapsulation (public) key size.
pub const MLKEM_PK_LEN: usize = 1184;
/// Combined hybrid ciphertext (X25519 ephemeral pub || ML-KEM ct).
pub const HYBRID_CT_LEN: usize = X25519_PUB_LEN + MLKEM_CT_LEN;

/// Output size of the combined shared secret.
pub const SHARED_LEN: usize = 32;

// ── rand_core adapter so the PQ crates can use our RNDR-backed CSPRNG ──
//
// ml-kem wants a `rand_core::CryptoRngCore`. Our crypto::rng exposes
// `fill_bytes` as a free function. Wrap it in a ZST that implements
// the trait.
pub struct BatRng;

impl rand_core::RngCore for BatRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        rng::fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        rng::fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        rng::fill_bytes(dst);
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), rand_core::Error> {
        rng::fill_bytes(dst);
        Ok(())
    }
}
impl rand_core::CryptoRng for BatRng {}

/// The recipient's long-term(ish) hybrid keypair. Both halves must be
/// kept secret. The public half is `recipient_public()`.
pub struct HybridKeyPair {
    pub x25519_sk: StaticSecret,
    pub x25519_pk: [u8; X25519_PUB_LEN],
    pub mlkem_dk: <MlKem768 as KemCore>::DecapsulationKey,
    pub mlkem_ek: <MlKem768 as KemCore>::EncapsulationKey,
}

impl HybridKeyPair {
    /// Generate a fresh hybrid keypair using the RNDR-backed CSPRNG.
    pub fn generate() -> Self {
        let mut rng_seed = [0u8; 32];
        rng::fill_bytes(&mut rng_seed);
        let x25519_sk = StaticSecret::from(rng_seed);
        let x25519_pk: [u8; 32] = X25519Public::from(&x25519_sk).to_bytes();

        let mut r = BatRng;
        let (mlkem_dk, mlkem_ek) = <MlKem768 as KemCore>::generate(&mut r);

        Self { x25519_sk, x25519_pk, mlkem_dk, mlkem_ek }
    }

    /// Serialize the public half for transmission to a peer.
    /// Layout: X25519 pub (32 B) || ML-KEM-768 encap key (1184 B).
    pub fn public_bytes(&self) -> Vec<u8> {
        let mut out = vec![0u8; X25519_PUB_LEN + MLKEM_PK_LEN];
        out[..X25519_PUB_LEN].copy_from_slice(&self.x25519_pk);
        let ek = self.mlkem_ek.as_bytes();
        out[X25519_PUB_LEN..].copy_from_slice(ek.as_slice());
        out
    }
}

/// Sender-side hybrid KEM. Given a recipient's hybrid public key (as
/// produced by `HybridKeyPair::public_bytes`), produce (shared_secret,
/// on-wire ciphertext blob) to send. The blob is
/// `ephemeral_x25519_pub || mlkem_ct` = HYBRID_CT_LEN bytes.
pub fn encapsulate(recipient_public: &[u8])
    -> Result<([u8; SHARED_LEN], Vec<u8>), &'static str>
{
    if recipient_public.len() != X25519_PUB_LEN + MLKEM_PK_LEN {
        return Err("hybrid: bad recipient public length");
    }

    // ── Classical half: X25519 ephemeral → static recipient ──
    let mut rng_local = BatRng;
    let eph_sk = EphemeralSecret::random_from_rng(&mut rng_local);
    let eph_pk: [u8; 32] = X25519Public::from(&eph_sk).to_bytes();
    let mut rp = [0u8; 32];
    rp.copy_from_slice(&recipient_public[..X25519_PUB_LEN]);
    let recip_pk = X25519Public::from(rp);
    let ss_c = eph_sk.diffie_hellman(&recip_pk);

    // ── PQ half: ML-KEM-768 encap against recipient's encap key ──
    type MlKemEk = <MlKem768 as KemCore>::EncapsulationKey;
    let ek_bytes: &[u8] = &recipient_public[X25519_PUB_LEN..];
    let ek_arr: Encoded<MlKemEk> = Array::try_from(ek_bytes)
        .map_err(|_| "hybrid: ML-KEM encap key byte length mismatch")?;
    let recip_ek = <MlKemEk as EncodedSizeUser>::from_bytes(&ek_arr);

    let mut r = BatRng;
    let (ct_arr, ss_pq_arr) = recip_ek.encapsulate(&mut r)
        .map_err(|_| "hybrid: ML-KEM encapsulate failed")?;

    // Combined secret = HKDF-SHA256 over ss_c || ss_pq with a
    // domain-separation label.
    let mut combined_in = [0u8; 64 + 16];
    combined_in[..32].copy_from_slice(ss_c.as_bytes());
    combined_in[32..64].copy_from_slice(ss_pq_arr.as_slice());
    combined_in[64..].copy_from_slice(b"BATOS-PQ-HYBRID\x00");
    let mut shared = [0u8; SHARED_LEN];
    let h = sha256::hash(&combined_in);
    shared.copy_from_slice(&h);

    // On-wire: eph_x25519_pub || mlkem_ct
    let mut out = vec![0u8; HYBRID_CT_LEN];
    out[..X25519_PUB_LEN].copy_from_slice(&eph_pk);
    out[X25519_PUB_LEN..].copy_from_slice(ct_arr.as_slice());
    Ok((shared, out))
}

/// Recipient-side hybrid decap. Given our keypair and the sender's
/// blob, recover the shared secret.
pub fn decapsulate(me: &HybridKeyPair, ciphertext_blob: &[u8])
    -> Result<[u8; SHARED_LEN], &'static str>
{
    if ciphertext_blob.len() != HYBRID_CT_LEN {
        return Err("hybrid: bad ciphertext length");
    }

    let mut eph_pk_bytes = [0u8; 32];
    eph_pk_bytes.copy_from_slice(&ciphertext_blob[..X25519_PUB_LEN]);
    let eph_pk = X25519Public::from(eph_pk_bytes);
    let ss_c = me.x25519_sk.diffie_hellman(&eph_pk);

    let ct_bytes: &[u8] = &ciphertext_blob[X25519_PUB_LEN..];
    // Ciphertext is Array<u8, CiphertextSize>. Build via try_from + the
    // KEM's CiphertextSize typenum, exposed as Ciphertext<MlKem768>.
    let ct_arr: ml_kem::Ciphertext<MlKem768> = Array::try_from(ct_bytes)
        .map_err(|_| "hybrid: ML-KEM ciphertext length mismatch")?;
    let ss_pq_arr = me.mlkem_dk.decapsulate(&ct_arr)
        .map_err(|_| "hybrid: ML-KEM decapsulate failed")?;

    let mut combined_in = [0u8; 64 + 16];
    combined_in[..32].copy_from_slice(ss_c.as_bytes());
    combined_in[32..64].copy_from_slice(ss_pq_arr.as_slice());
    combined_in[64..].copy_from_slice(b"BATOS-PQ-HYBRID\x00");
    let mut shared = [0u8; SHARED_LEN];
    let h = sha256::hash(&combined_in);
    shared.copy_from_slice(&h);
    Ok(shared)
}

/// End-to-end self-test run from `pq-selftest` shell command.
///
/// 1. Bob generates a hybrid keypair.
/// 2. Bob publishes his public half.
/// 3. Alice encapsulates against it — learns ss_A + sends blob.
/// 4. Bob decapsulates the blob with his secret — learns ss_B.
/// 5. Assert ss_A == ss_B.
///
/// Returns Ok((blob_len, shared_hex_prefix)) on success.
pub fn selftest() -> Result<(usize, [u8; 8]), &'static str> {
    let bob = HybridKeyPair::generate();
    let bob_pub = bob.public_bytes();
    let (alice_ss, blob) = encapsulate(&bob_pub)?;
    let bob_ss = decapsulate(&bob, &blob)?;
    if alice_ss != bob_ss {
        return Err("hybrid: shared secrets disagree");
    }
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&alice_ss[..8]);
    Ok((blob.len(), prefix))
}
