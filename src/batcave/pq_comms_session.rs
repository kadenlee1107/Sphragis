//! Post-quantum hybrid comms handshake.
//!
//! Future-mode for `apps::comms`. The classical-only handshake we
//! ship today uses X25519 + Ed25519 (matches the Python test server
//! exactly). This module implements the PQ-hybrid version that
//! Bat_OS would actually want to run in production: every primitive
//! is doubled with its NIST PQC counterpart, so an adversary has to
//! break BOTH the classical curve AND the PQ lattice scheme to
//! recover a session or forge a server identity.
//!
//! Primitives:
//!   * Key agreement: X25519 + ML-KEM-768 (KEM, not DH — server has
//!     long-term keypair, client encapsulates).
//!   * Authentication: Ed25519 + ML-DSA-65 (hybrid signatures).
//!   * Transport: ChaCha20-Poly1305 (same as classical path), with
//!     keys derived via SHA-256 KDF over (shared || labels).
//!
//! Why this isn't wired into the live `apps::comms` yet:
//!   * The Python test server can't speak ML-KEM/ML-DSA without
//!     pulling in liboqs (heavy native dep) or alpha-quality pure-
//!     Python libs. We don't ship something that *claims* to be
//!     PQ-secure but where one side is classical-only — the whole
//!     point of hybrid is that BOTH halves enforce. So the wire
//!     deployment waits until we have a real PQ peer (a second
//!     Bat_OS instance is the obvious candidate).
//!   * The handshake logic itself we can validate today, via
//!     `selftest_round_trip` below. It runs the full client+server
//!     halves in-process and asserts both sides agree on the
//!     session key and can round-trip an AEAD-encrypted message.
//!     `pq-comms-selftest` shell command exercises it.
//!
//! Wire protocol (documented for the eventual deployment):
//!
//!   SETUP (out-of-band):
//!     - Server publishes its HybridSigPub (1984 B = Ed25519 32 +
//!       ML-DSA-65 1952) and HybridKemPub (1216 B = ML-KEM-768
//!       1184 + X25519 32). Client pins both.
//!     - Server pins each client's HybridSigPub via allowlist.
//!
//!   CONNECT (TCP):
//!     Client → Server:
//!       client_sig_pub (1984)
//!         || ciphertext_blob (1120)  [ML-KEM ct 1088 + X25519 eph 32]
//!         || hybrid_sig over (ciphertext_blob || LABEL || "c2s")
//!            signed by client_sig_sk  (3373 B = Ed25519 64 + ML-DSA 3309)
//!       Total: 6477 B
//!
//!     Server → Client:
//!       hybrid_sig over (ciphertext_blob || LABEL || "s2c") signed by
//!       server_sig_sk  (3373 B)
//!
//!   KEY DERIVATION (both sides):
//!     shared = ML-KEM-SS(32) || X25519-SS(32)  (per draft-ietf-tls-ecdhe-mlkem-04)
//!     c2s_key = SHA-256(b"BAT_OS-PQ-COMMS-c2s-v1" || shared)
//!     s2c_key = SHA-256(b"BAT_OS-PQ-COMMS-s2c-v1" || shared)
//!
//!   TRANSPORT: same as classical path —
//!     len(4 BE) || nonce(12) || ChaCha20-Poly1305 ct+tag(16)
//!     Counter nonce per direction (u64 BE + 4 zero bytes).

#![allow(dead_code)]

use crate::crypto::{chacha20poly1305 as cp, pq_hybrid, pq_hybrid_sig, sha256};

pub const LABEL: &[u8] = b"BAT_OS-PQ-COMMS-v1";

/// Per-direction key derivation. Mirrors the classical
/// `derive_directional_keys` but takes a 64-byte hybrid shared
/// secret (ML-KEM-SS || X25519-SS) as input.
pub fn derive_directional_keys(shared: &[u8; pq_hybrid::SHARED_LEN])
    -> ([u8; 32], [u8; 32])
{
    let mut c2s_in = [0u8; 22 + pq_hybrid::SHARED_LEN];
    c2s_in[..22].copy_from_slice(b"BAT_OS-PQ-COMMS-c2s-v1");
    c2s_in[22..].copy_from_slice(shared);
    let c2s = sha256::hash(&c2s_in);

    let mut s2c_in = [0u8; 22 + pq_hybrid::SHARED_LEN];
    s2c_in[..22].copy_from_slice(b"BAT_OS-PQ-COMMS-s2c-v1");
    s2c_in[22..].copy_from_slice(shared);
    let s2c = sha256::hash(&s2c_in);

    (c2s, s2c)
}

/// 12-byte nonce: u64 BE counter + 4 zero bytes. Identical to the
/// classical-path scheme so the transport layer doesn't fork.
pub fn nonce_from_ctr(ctr: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[..8].copy_from_slice(&ctr.to_be_bytes());
    n
}

/// In-process round trip exercising the full PQ hybrid handshake +
/// AEAD transport. Used by `pq-comms-selftest`. Returns a tuple of:
///   (server_session_key_prefix_c2s,
///    client_session_key_prefix_c2s,
///    keys_match_both_sides,
///    aead_round_trip_succeeded,
///    client_sig_pub_len, server_sig_pub_len)
///
/// Prefixes are the first 8 bytes of each side's c2s key — printing
/// them lets the operator visually confirm both sides agreed.
pub fn selftest_round_trip()
    -> Result<([u8; 8], [u8; 8], bool, bool, usize, usize), &'static str>
{
    // ── Out-of-band setup ────────────────────────────────────────
    // Server long-term hybrid keys (sig identity + kem capacity).
    let server_sig = pq_hybrid_sig::HybridSigKeyPair::generate();
    let server_kem = pq_hybrid::HybridKeyPair::generate();
    let server_sig_pub = server_sig.public_bytes();
    let server_kem_pub = server_kem.public_bytes();

    // Client long-term hybrid sig identity (no KEM keys — client
    // only encapsulates to server, doesn't receive encapsulations).
    let client_sig = pq_hybrid_sig::HybridSigKeyPair::generate();
    let client_sig_pub = client_sig.public_bytes();

    // ── Client → Server: encapsulate + sign ──────────────────────
    let (shared_client, ct_blob) = pq_hybrid::encapsulate(&server_kem_pub)?;

    let mut c2s_sig_msg = [0u8; 1120 + 18 + 3];
    c2s_sig_msg[..ct_blob.len()].copy_from_slice(&ct_blob);
    c2s_sig_msg[ct_blob.len()..ct_blob.len() + LABEL.len()].copy_from_slice(LABEL);
    c2s_sig_msg[ct_blob.len() + LABEL.len()..ct_blob.len() + LABEL.len() + 3]
        .copy_from_slice(b"c2s");
    let c2s_sig_len = ct_blob.len() + LABEL.len() + 3;
    let c2s_sig = client_sig.sign(&c2s_sig_msg[..c2s_sig_len])?;

    // ── Server-side verify + decapsulate ─────────────────────────
    pq_hybrid_sig::verify(&client_sig_pub, &c2s_sig_msg[..c2s_sig_len], &c2s_sig)?;

    let shared_server = pq_hybrid::decapsulate_from_bytes(
        &server_kem.x25519_sk_bytes(),
        &server_kem.mlkem_dk_bytes(),
        &ct_blob,
    )?;

    // Compare shared secrets (constant-time-ish — selftest, not
    // production hot path).
    let mut diff = 0u8;
    for i in 0..pq_hybrid::SHARED_LEN {
        diff |= shared_server[i] ^ shared_client[i];
    }
    let shared_match = diff == 0;
    if !shared_match {
        return Err("pq-comms: client/server shared secrets disagree");
    }

    // ── Server → Client: sign acknowledgment over ct||label||"s2c" ─
    let mut s2c_sig_msg = [0u8; 1120 + 18 + 3];
    s2c_sig_msg[..ct_blob.len()].copy_from_slice(&ct_blob);
    s2c_sig_msg[ct_blob.len()..ct_blob.len() + LABEL.len()].copy_from_slice(LABEL);
    s2c_sig_msg[ct_blob.len() + LABEL.len()..ct_blob.len() + LABEL.len() + 3]
        .copy_from_slice(b"s2c");
    let s2c_sig_len = ct_blob.len() + LABEL.len() + 3;
    let s2c_sig = server_sig.sign(&s2c_sig_msg[..s2c_sig_len])?;

    // Client verifies server sig against pinned server pub.
    pq_hybrid_sig::verify(&server_sig_pub, &s2c_sig_msg[..s2c_sig_len], &s2c_sig)?;

    // ── Both sides: derive AEAD keys ─────────────────────────────
    let (c2s_key, s2c_key) = derive_directional_keys(&shared_client);
    let (c2s_key_s, s2c_key_s) = derive_directional_keys(&shared_server);

    let mut keys_match = c2s_key == c2s_key_s && s2c_key == s2c_key_s;

    // ── AEAD round trip in both directions ───────────────────────
    let aead_ok = aead_round_trip(&c2s_key, &s2c_key)?;
    if !aead_ok {
        keys_match = false;
    }

    let mut c2s_prefix_c = [0u8; 8];
    let mut c2s_prefix_s = [0u8; 8];
    c2s_prefix_c.copy_from_slice(&c2s_key[..8]);
    c2s_prefix_s.copy_from_slice(&c2s_key_s[..8]);

    Ok((c2s_prefix_s, c2s_prefix_c, keys_match, aead_ok,
        client_sig_pub.len(), server_sig_pub.len()))
}

fn aead_round_trip(c2s_key: &[u8; 32], s2c_key: &[u8; 32])
    -> Result<bool, &'static str>
{
    // Client sends; server decrypts.
    let client_plaintext = b"hello from client (PQ)";
    let n1 = nonce_from_ctr(0);
    let c1 = cp::encrypt(c2s_key, &n1, &[], client_plaintext)
        .map_err(|_| "pq-comms: c2s encrypt failed")?;
    let p1 = cp::decrypt(c2s_key, &n1, &[], &c1)
        .map_err(|_| "pq-comms: c2s decrypt failed (tag verify)")?;
    if p1.as_slice() != client_plaintext {
        return Ok(false);
    }

    // Server replies; client decrypts.
    let server_plaintext = b"hello back from server (PQ)";
    let n2 = nonce_from_ctr(0);
    let c2 = cp::encrypt(s2c_key, &n2, &[], server_plaintext)
        .map_err(|_| "pq-comms: s2c encrypt failed")?;
    let p2 = cp::decrypt(s2c_key, &n2, &[], &c2)
        .map_err(|_| "pq-comms: s2c decrypt failed (tag verify)")?;
    if p2.as_slice() != server_plaintext {
        return Ok(false);
    }

    Ok(true)
}
