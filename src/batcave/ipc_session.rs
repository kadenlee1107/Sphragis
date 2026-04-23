//! DESIGN_CRYPTO.md #10 + #13 — Mutually-authenticated IPC sessions.
//!
//! Before this module, `batcave::batpipe` routed raw bytes between
//! caves with no authentication — any cave that found the pipe could
//! impersonate a sender. This module lifts IPC to a Noise-style
//! mutual-auth session:
//!
//!   1. Each cave owns a long-term **Ed25519 identity keypair**.
//!      Created at `cave::create`, destroyed at `cave::destroy`.
//!   2. On session setup between caves A and B:
//!        * A picks ephemeral x25519_A, sends x25519_pub_A + A's
//!          Ed25519 identity_pub + Ed25519-sign("IPC-A→B" || ...).
//!        * B verifies A's signature against A's known identity_pub.
//!        * B replies with x25519_pub_B + B's identity_pub +
//!          Ed25519-sign("IPC-B→A" || ...).
//!        * A verifies B's signature.
//!        * Both compute shared_secret = X25519(eph_sk, peer_eph_pub),
//!          then derive per-session session_key via SHA-256 KDF.
//!   3. Subsequent IPC frames are encrypted under the session key
//!      with ChaCha20-Poly1305.
//!
//! This gets us what Noise XX gives with less protocol-theory
//! overhead — forward secrecy (ephemeral x25519) + mutual auth
//! (long-term Ed25519 identities) + AEAD for frames.
//!
//! **Scope of THIS commit:** the SessionKey derivation + the
//! Ed25519 identity-binding. Wiring into `batpipe` (replacing the
//! raw-byte path with an encrypted framed channel) is a follow-up
//! — the primitive lives here and is tested via a shell-side
//! self-test that runs two simulated caves through the handshake
//! and asserts they agree on the session key.
//!
//! Future PQ upgrade: swap X25519 → hybrid X25519+ML-KEM-768 (from
//! crypto::pq_hybrid) when we move to PQ-everywhere. The handshake
//! shape stays identical; only the key-exchange size grows.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use ed25519_compact::{KeyPair, PublicKey, SecretKey, Seed, Signature};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};

use crate::crypto::{rng, sha256, pq_hybrid::BatRng};

/// Ed25519 public + secret sizes (reference).
pub const ID_PUB_LEN: usize = 32;
pub const ID_SIG_LEN: usize = 64;

/// X25519 public size.
pub const EPH_PUB_LEN: usize = 32;

/// Derived session key size.
pub const SESSION_KEY_LEN: usize = 32;

/// One side's IPC identity. Long-lived for the cave's lifetime.
pub struct IpcIdentity {
    /// Ed25519 secret (seed+pub concatenated, per ed25519-compact layout).
    pub sk: [u8; 64],
    pub pk: [u8; ID_PUB_LEN],
}

impl IpcIdentity {
    pub fn generate() -> Self {
        let mut seed_bytes = [0u8; 32];
        rng::fill_bytes(&mut seed_bytes);
        let kp = KeyPair::from_seed(Seed::new(seed_bytes));
        Self { sk: *kp.sk, pk: *kp.pk }
    }
}

/// First message of the handshake (A → B).
/// Layout: eph_pub (32) | id_pub (32) | sig (64) = 128 B.
pub fn build_offer(
    sender_id: &IpcIdentity,
    eph_pub: &[u8; EPH_PUB_LEN],
    session_label: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let sk = SecretKey::from_slice(&sender_id.sk)
        .map_err(|_| "ipc-session: bad secret")?;
    // Sign eph_pub || label. Binds the ephemeral to this identity
    // for this specific session label.
    let mut msg = [0u8; 128];
    msg[..EPH_PUB_LEN].copy_from_slice(eph_pub);
    let n = session_label.len().min(96);
    msg[EPH_PUB_LEN..EPH_PUB_LEN + n].copy_from_slice(&session_label[..n]);
    let sig = sk.sign(&msg[..EPH_PUB_LEN + n], None);

    let mut out = Vec::with_capacity(EPH_PUB_LEN + ID_PUB_LEN + ID_SIG_LEN);
    out.extend_from_slice(eph_pub);
    out.extend_from_slice(&sender_id.pk);
    out.extend_from_slice(sig.as_slice());
    Ok(out)
}

/// Verify an incoming offer. Returns the peer's ephemeral pub on success.
pub fn verify_offer(
    offer: &[u8],
    expected_sender_id_pub: Option<&[u8; ID_PUB_LEN]>,
    session_label: &[u8],
) -> Result<[u8; EPH_PUB_LEN], &'static str> {
    if offer.len() != EPH_PUB_LEN + ID_PUB_LEN + ID_SIG_LEN {
        return Err("ipc-session: bad offer length");
    }
    let eph_bytes = &offer[..EPH_PUB_LEN];
    let id_bytes  = &offer[EPH_PUB_LEN..EPH_PUB_LEN + ID_PUB_LEN];
    let sig_bytes = &offer[EPH_PUB_LEN + ID_PUB_LEN..];

    // If caller supplied an expected identity (they already know
    // the peer's cave), pin to it.
    if let Some(expected) = expected_sender_id_pub {
        if expected[..] != id_bytes[..] {
            return Err("ipc-session: peer identity mismatch");
        }
    }

    let pk = PublicKey::from_slice(id_bytes)
        .map_err(|_| "ipc-session: bad peer id pub")?;
    let sig = Signature::from_slice(sig_bytes)
        .map_err(|_| "ipc-session: bad signature")?;

    let mut msg = [0u8; 128];
    msg[..EPH_PUB_LEN].copy_from_slice(eph_bytes);
    let n = session_label.len().min(96);
    msg[EPH_PUB_LEN..EPH_PUB_LEN + n].copy_from_slice(&session_label[..n]);
    pk.verify(&msg[..EPH_PUB_LEN + n], &sig)
        .map_err(|_| "ipc-session: peer sig verify failed")?;

    let mut out = [0u8; EPH_PUB_LEN];
    out.copy_from_slice(eph_bytes);
    Ok(out)
}

/// Derive the per-session symmetric key from both sides' ephemerals
/// and identity pubs. Same derivation must run on both sides so
/// they land on the same 32-byte key.
pub fn derive_session_key(
    my_eph_sk: EphemeralSecret,
    peer_eph_pub: &[u8; EPH_PUB_LEN],
    a_id_pub: &[u8; ID_PUB_LEN],
    b_id_pub: &[u8; ID_PUB_LEN],
    label: &[u8],
) -> [u8; SESSION_KEY_LEN] {
    let peer_pk = X25519Public::from(*peer_eph_pub);
    let shared = my_eph_sk.diffie_hellman(&peer_pk);

    // Mix everything into SHA-256. Order is canonical: lexicographic
    // order of identity pubs, so A and B derive identically.
    let (id_lo, id_hi) = if a_id_pub <= b_id_pub {
        (a_id_pub, b_id_pub)
    } else {
        (b_id_pub, a_id_pub)
    };
    let mut input = Vec::with_capacity(32 + 32 + 32 + 64);
    input.extend_from_slice(shared.as_bytes());
    input.extend_from_slice(id_lo);
    input.extend_from_slice(id_hi);
    input.extend_from_slice(b"BATOS-IPC-SESSION-V1");
    let n = label.len().min(40);
    input.extend_from_slice(&label[..n]);
    let h = sha256::hash(&input);
    let mut out = [0u8; SESSION_KEY_LEN];
    out.copy_from_slice(&h);
    out
}

/// Simulate a full A↔B handshake in one function. Used by the
/// `ipc-selftest` shell command to prove correctness.
///
/// Returns (session_key_alice_prefix, session_key_bob_prefix,
/// match) — the prefixes should be identical (and match == true) on
/// success.
pub fn selftest_round_trip() -> Result<([u8; 8], [u8; 8], bool), &'static str> {
    let label = b"batos-selftest-session";

    // Create two cave identities
    let alice = IpcIdentity::generate();
    let bob   = IpcIdentity::generate();

    // Each side picks its own ephemeral X25519
    let mut rng_a = BatRng;
    let alice_eph_sk = EphemeralSecret::random_from_rng(&mut rng_a);
    let alice_eph_pk: [u8; 32] = X25519Public::from(&alice_eph_sk).to_bytes();
    let mut rng_b = BatRng;
    let bob_eph_sk = EphemeralSecret::random_from_rng(&mut rng_b);
    let bob_eph_pk: [u8; 32] = X25519Public::from(&bob_eph_sk).to_bytes();

    // Alice sends her offer
    let alice_offer = build_offer(&alice, &alice_eph_pk, label)?;
    // Bob verifies and extracts Alice's eph
    let alice_eph_back = verify_offer(&alice_offer, None, label)?;
    assert_eq_arr(&alice_eph_back, &alice_eph_pk)?;

    // Bob sends his offer
    let bob_offer = build_offer(&bob, &bob_eph_pk, label)?;
    // Alice verifies Bob's offer
    let bob_eph_back = verify_offer(&bob_offer, None, label)?;
    assert_eq_arr(&bob_eph_back, &bob_eph_pk)?;

    // Both derive the session key
    let key_alice = derive_session_key(
        alice_eph_sk, &bob_eph_pk,
        &alice.pk, &bob.pk, label);
    let key_bob = derive_session_key(
        bob_eph_sk, &alice_eph_pk,
        &alice.pk, &bob.pk, label);

    let mut a_prefix = [0u8; 8];
    let mut b_prefix = [0u8; 8];
    a_prefix.copy_from_slice(&key_alice[..8]);
    b_prefix.copy_from_slice(&key_bob[..8]);
    Ok((a_prefix, b_prefix, key_alice == key_bob))
}

fn assert_eq_arr(a: &[u8; 32], b: &[u8; 32]) -> Result<(), &'static str> {
    let mut diff: u8 = 0;
    for i in 0..32 { diff |= a[i] ^ b[i]; }
    if diff != 0 { Err("ipc-session: ephemerals mismatch") } else { Ok(()) }
}
