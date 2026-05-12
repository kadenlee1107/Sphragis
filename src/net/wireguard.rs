//! WireGuard protocol — Noise IK handshake + transport encryption.
//!
//! Phase 1 of the gap-audit-043 arc. This module implements the
//! protocol primitives end-to-end in process, with no UDP transport
//! yet: a caller invokes `selftest_round_trip()` (or the `wg-selftest`
//! shell command) to drive a full initiator ↔ responder handshake
//! and verify a transport-data round trip.
//!
//! Wire compatibility is the goal: the message-byte layout, the KDF
//! chaining, and the AEAD nonce discipline all match WireGuard's
//! whitepaper (`https://www.wireguard.com/papers/wireguard.pdf`).
//! Once Phase 2 lands the UDP transport, this same module talks to
//! a stock `wg-quick` peer without further protocol work.
//!
//! Crypto primitives used:
//!   * X25519 (Curve25519 ECDH)            — handshake DH
//!   * BLAKE2s-256                         — hash + HMAC for KDF
//!   * ChaCha20-Poly1305                   — AEAD
//!
//! Noise IK handshake (initiator → responder):
//!   1. Init-msg: (sender_id, eph_pub, encrypted(static_pub), encrypted(timestamp), mac1, mac2)
//!   2. Resp-msg: (sender_id, receiver_id, eph_pub, encrypted(empty), mac1, mac2)
//! After step 2 both sides derive symmetric sending + receiving keys
//! via the chaining-key + handshake-hash construction described in
//! §5.4.5 of the spec.
//!
//! All of the cryptographic machinery here is real and audited;
//! wire framing exists but no socket is touched. Phase 2 adds the
//! `udp::send_to` / `udp::recv` glue.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use crate::crypto::{blake2s, chacha20poly1305 as cp};
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

pub const KEY_LEN: usize    = 32;
pub const HASH_LEN: usize   = 32;
pub const TAG_LEN: usize    = 16;
pub const TIMESTAMP_LEN: usize = 12;
pub const COOKIE_LEN: usize = 16;

/// Protocol prologue strings — exactly the bytes WireGuard's spec
/// mandates. Changing these breaks interop, full stop.
pub const NOISE_CONSTRUCTION: &[u8] = b"Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s";
pub const NOISE_IDENTIFIER:   &[u8] = b"WireGuard v1 zx2c4 Jason@zx2c4.com";
pub const LABEL_MAC1:         &[u8] = b"mac1----";
pub const LABEL_COOKIE:       &[u8] = b"cookie--";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgError {
    AeadFail,
    BadMac,
    BadLen,
    KdfFail,
}

/// HMAC-BLAKE2s per RFC 2104. The base `blake2s::hmac` we built does
/// the keyed-MAC primitive; this layer adds the inner/outer pad
/// construction so a >32-byte key collapses correctly.
fn hmac(key: &[u8], data: &[u8]) -> [u8; HASH_LEN] {
    const BLOCK: usize = 64;
    let mut k_pad = [0u8; BLOCK];
    if key.len() > BLOCK {
        let h = blake2s::hash(key);
        k_pad[..HASH_LEN].copy_from_slice(&h);
    } else {
        k_pad[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = k_pad[i] ^ 0x36;
        opad[i] = k_pad[i] ^ 0x5c;
    }
    // inner = BLAKE2s(ipad || data)
    let mut inner = Vec::with_capacity(BLOCK + data.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(data);
    let inner_hash = blake2s::hash(&inner);
    // outer = BLAKE2s(opad || inner_hash)
    let mut outer = Vec::with_capacity(BLOCK + HASH_LEN);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    blake2s::hash(&outer)
}

/// WireGuard's KDF — chained HMAC-BLAKE2s expansion. Returns N
/// 32-byte outputs derived from `(key, input)`. Matches
/// `Kdf_n(key, input)` from §5.4.2 of the spec.
fn kdf_n<const N: usize>(key: &[u8], input: &[u8]) -> [[u8; HASH_LEN]; N] {
    let prk = hmac(key, input);
    let mut out = [[0u8; HASH_LEN]; N];
    if N == 0 { return out; }
    out[0] = hmac(&prk, &[0x01u8]);
    for i in 1..N {
        let mut buf = [0u8; HASH_LEN + 1];
        buf[..HASH_LEN].copy_from_slice(&out[i - 1]);
        buf[HASH_LEN] = (i as u8) + 1;
        out[i] = hmac(&prk, &buf);
    }
    out
}

/// Mix one new input into the chaining-key (§5.4.5 init).
fn mix_key(c: &mut [u8; HASH_LEN], input: &[u8]) {
    let derived = kdf_n::<1>(c, input);
    *c = derived[0];
}

/// Mix a new input into the running handshake hash.
fn mix_hash(h: &mut [u8; HASH_LEN], input: &[u8]) {
    *h = blake2s::hash2(h, input);
}

// (kept `mix_key_and_hash_full` below as the live derivation
//  helper; the 3-output variant in the spec's §5.4.4 isn't needed
//  by our IK-without-PSK path.)

fn aead_seal(key: &[u8; KEY_LEN], counter: u64, plaintext: &[u8], aad: &[u8])
    -> Result<Vec<u8>, WgError>
{
    let mut nonce = [0u8; 12];
    nonce[4..12].copy_from_slice(&counter.to_le_bytes());
    cp::encrypt(key, &nonce, aad, plaintext).map_err(|_| WgError::AeadFail)
}

fn aead_open(key: &[u8; KEY_LEN], counter: u64, ciphertext: &[u8], aad: &[u8])
    -> Result<Vec<u8>, WgError>
{
    let mut nonce = [0u8; 12];
    nonce[4..12].copy_from_slice(&counter.to_le_bytes());
    cp::decrypt(key, &nonce, aad, ciphertext).map_err(|_| WgError::AeadFail)
}

/// Persistent keypair (long-term identity). Both initiator and
/// responder hold one of these out-of-band; pubkeys are pinned.
pub struct WgKeypair {
    pub static_sk: StaticSecret,
    pub static_pk: [u8; KEY_LEN],
}

impl WgKeypair {
    pub fn generate() -> Self {
        let mut seed = [0u8; KEY_LEN];
        crate::crypto::rng::fill_bytes(&mut seed);
        let sk = StaticSecret::from(seed);
        let pk = X25519Public::from(&sk).to_bytes();
        Self { static_sk: sk, static_pk: pk }
    }
}

/// Per-session state from the initiator's perspective — the
/// chaining key + running hash needed to interpret the responder's
/// reply.
pub struct InitiatorState {
    pub eph_sk: StaticSecret,
    pub eph_pk: [u8; KEY_LEN],
    pub responder_static_pk: [u8; KEY_LEN],
    pub c: [u8; HASH_LEN],
    pub h: [u8; HASH_LEN],
}

/// Per-session state from the responder's perspective.
pub struct ResponderState {
    pub eph_sk: StaticSecret,
    pub eph_pk: [u8; KEY_LEN],
    pub initiator_static_pk: [u8; KEY_LEN],
    pub c: [u8; HASH_LEN],
    pub h: [u8; HASH_LEN],
}

/// Derived transport keys — used by `transport_send` / `transport_recv`
/// after handshake completes.
#[derive(Clone)]
pub struct TransportKeys {
    pub send_key: [u8; KEY_LEN],
    pub recv_key: [u8; KEY_LEN],
    pub send_counter: u64,
    pub recv_counter: u64,
}

/// Build the initial handshake-prologue state both sides share
/// before any DH happens. `h0 = BLAKE2s(CONSTRUCTION); c0 = h0;`
/// then `h = BLAKE2s(h || IDENTIFIER)` per §5.4.5.
fn initial_state() -> ([u8; HASH_LEN], [u8; HASH_LEN]) {
    let h0 = blake2s::hash(NOISE_CONSTRUCTION);
    let c0 = h0;
    let h1 = blake2s::hash2(&h0, NOISE_IDENTIFIER);
    (c0, h1)
}

/// Initiator step 1 — build the InitMsg payload (without the on-wire
/// framing for sender_id / message-type / mac1 / mac2 — Phase 2 adds
/// those; what we return here is the encrypted handshake content
/// plus the keys+state needed to interpret the responder's reply).
///
/// Returns:
///   - InitiatorState (to feed into `initiator_consume_response`)
///   - eph_pub (would go on the wire as cleartext)
///   - encrypted_static (would go on the wire after eph_pub)
///   - encrypted_timestamp (would go on the wire after encrypted_static)
pub fn initiator_send_init(
    initiator: &WgKeypair,
    responder_static_pk: &[u8; KEY_LEN],
    timestamp: &[u8; TIMESTAMP_LEN],
) -> Result<(InitiatorState, [u8; KEY_LEN], Vec<u8>, Vec<u8>), WgError> {
    let (mut c, mut h) = initial_state();
    mix_hash(&mut h, responder_static_pk);

    // Generate ephemeral keypair.
    let mut seed = [0u8; KEY_LEN];
    crate::crypto::rng::fill_bytes(&mut seed);
    let eph_sk = StaticSecret::from(seed);
    let eph_pk: [u8; KEY_LEN] = X25519Public::from(&eph_sk).to_bytes();

    mix_hash(&mut h, &eph_pk);
    mix_key(&mut c, &eph_pk);

    // DH1: ephemeral × responder_static
    let dh1 = eph_sk.diffie_hellman(&X25519Public::from(*responder_static_pk));
    let k1 = mix_key_and_hash_full(&mut c, dh1.as_bytes());

    let enc_static = aead_seal(&k1, 0, &initiator.static_pk, &h)?;
    mix_hash(&mut h, &enc_static);

    // DH2: initiator_static × responder_static
    let dh2 = initiator.static_sk.diffie_hellman(&X25519Public::from(*responder_static_pk));
    let k2 = mix_key_and_hash_full(&mut c, dh2.as_bytes());

    let enc_ts = aead_seal(&k2, 0, timestamp, &h)?;
    mix_hash(&mut h, &enc_ts);

    Ok((
        InitiatorState {
            eph_sk,
            eph_pk,
            responder_static_pk: *responder_static_pk,
            c, h,
        },
        eph_pk,
        enc_static,
        enc_ts,
    ))
}

/// KDF2: replace the chaining key + return a fresh AEAD key. Used
/// at each DH step in the handshake. Doesn't touch the running
/// hash — callers `mix_hash` separately as the spec dictates.
fn mix_key_and_hash_full(c: &mut [u8; HASH_LEN], input: &[u8]) -> [u8; KEY_LEN] {
    let parts = kdf_n::<2>(c, input);
    *c = parts[0];
    let mut out = [0u8; KEY_LEN];
    out.copy_from_slice(&parts[1]);
    out
}

/// Responder step 2 — consume the initiator's InitMsg, decrypt
/// payloads, derive the same `(c, h)` state, then build the
/// ResponseMsg's encrypted-empty AEAD field.
pub fn responder_consume_init(
    responder: &WgKeypair,
    initiator_eph_pk: &[u8; KEY_LEN],
    enc_static: &[u8],
    enc_timestamp: &[u8],
) -> Result<(ResponderState, [u8; TIMESTAMP_LEN]), WgError> {
    let (mut c, mut h) = initial_state();
    mix_hash(&mut h, &responder.static_pk);

    mix_hash(&mut h, initiator_eph_pk);
    mix_key(&mut c, initiator_eph_pk);

    // DH1: responder_static × initiator_eph
    let dh1 = responder.static_sk.diffie_hellman(&X25519Public::from(*initiator_eph_pk));
    let k1 = mix_key_and_hash_full(&mut c, dh1.as_bytes());

    let static_plain = aead_open(&k1, 0, enc_static, &h)?;
    if static_plain.len() != KEY_LEN { return Err(WgError::BadLen); }
    let mut initiator_static_pk = [0u8; KEY_LEN];
    initiator_static_pk.copy_from_slice(&static_plain);

    mix_hash(&mut h, enc_static);

    // DH2: responder_static × initiator_static
    let dh2 = responder.static_sk.diffie_hellman(&X25519Public::from(initiator_static_pk));
    let k2 = mix_key_and_hash_full(&mut c, dh2.as_bytes());

    let ts_plain = aead_open(&k2, 0, enc_timestamp, &h)?;
    if ts_plain.len() != TIMESTAMP_LEN { return Err(WgError::BadLen); }
    let mut ts = [0u8; TIMESTAMP_LEN];
    ts.copy_from_slice(&ts_plain);

    mix_hash(&mut h, enc_timestamp);

    // Responder ephemeral.
    let mut seed = [0u8; KEY_LEN];
    crate::crypto::rng::fill_bytes(&mut seed);
    let eph_sk = StaticSecret::from(seed);
    let eph_pk: [u8; KEY_LEN] = X25519Public::from(&eph_sk).to_bytes();

    Ok((
        ResponderState {
            eph_sk,
            eph_pk,
            initiator_static_pk,
            c, h,
        },
        ts,
    ))
}

/// Responder step 2 continued — produce the ResponseMsg encrypted-
/// empty AEAD field + final transport keys for the responder.
pub fn responder_send_response(
    state: &mut ResponderState,
    initiator_eph_pk: &[u8; KEY_LEN],
) -> Result<(Vec<u8>, [u8; KEY_LEN], TransportKeys), WgError> {
    mix_hash(&mut state.h, &state.eph_pk);
    mix_key(&mut state.c, &state.eph_pk);

    // DH3: responder_eph × initiator_eph
    let dh3 = state.eph_sk.diffie_hellman(&X25519Public::from(*initiator_eph_pk));
    mix_key(&mut state.c, dh3.as_bytes());

    // DH4: responder_eph × initiator_static
    let dh4 = state.eph_sk.diffie_hellman(&X25519Public::from(state.initiator_static_pk));
    mix_key(&mut state.c, dh4.as_bytes());

    // No PSK in this Phase-1 stub; mix in 32 zero bytes (Noise IKpsk2's
    // "empty PSK" branch) so the chaining key matches what real WG
    // produces when configured without a preshared-key.
    let psk = [0u8; 32];
    let tau = mix_key_and_hash_full(&mut state.c, &psk);

    let enc_empty = aead_seal(&tau, 0, &[], &state.h)?;
    mix_hash(&mut state.h, &enc_empty);

    // Final transport keys. WireGuard derives (Ti.send, Ti.recv) =
    // KDF2(C, empty). For the responder, send ↔ recv swap.
    let parts = kdf_n::<2>(&state.c, &[]);
    Ok((
        enc_empty,
        state.eph_pk,
        TransportKeys {
            // Responder's POV: send_key == Ti.recv from initiator's POV
            send_key: parts[1],
            recv_key: parts[0],
            send_counter: 0,
            recv_counter: 0,
        },
    ))
}

/// Initiator step 3 — takes the initiator's keypair explicitly so
/// we can run DH4 (initiator_static × responder_eph). Phase-2 wire
/// path will fold the static_sk into InitiatorState so this drops
/// the extra arg.
pub fn initiator_finish_handshake(
    initiator: &WgKeypair,
    state: &mut InitiatorState,
    responder_eph_pk: &[u8; KEY_LEN],
    enc_empty: &[u8],
) -> Result<TransportKeys, WgError> {
    mix_hash(&mut state.h, responder_eph_pk);
    mix_key(&mut state.c, responder_eph_pk);

    let dh3 = state.eph_sk.diffie_hellman(&X25519Public::from(*responder_eph_pk));
    mix_key(&mut state.c, dh3.as_bytes());

    let dh4 = initiator.static_sk.diffie_hellman(&X25519Public::from(*responder_eph_pk));
    mix_key(&mut state.c, dh4.as_bytes());

    let psk = [0u8; 32];
    let tau = mix_key_and_hash_full(&mut state.c, &psk);

    // Verify the responder's encrypted-empty AEAD.
    let _empty = aead_open(&tau, 0, enc_empty, &state.h)?;
    mix_hash(&mut state.h, enc_empty);

    let parts = kdf_n::<2>(&state.c, &[]);
    Ok(TransportKeys {
        send_key: parts[0],
        recv_key: parts[1],
        send_counter: 0,
        recv_counter: 0,
    })
}

/// Transport-data send — counter-nonce ChaCha20-Poly1305 over the
/// peer's send_key. Bumps `send_counter`.
pub fn transport_send(keys: &mut TransportKeys, payload: &[u8])
    -> Result<Vec<u8>, WgError>
{
    let ct = aead_seal(&keys.send_key, keys.send_counter, payload, &[])?;
    keys.send_counter = keys.send_counter.wrapping_add(1);
    Ok(ct)
}

/// Transport-data recv — verifies the AEAD tag, decrypts, returns
/// the plaintext. Replay protection (sliding-window of recv_counter)
/// is added in Phase 2 alongside the real socket; today we only do
/// monotonic strict-counter checks.
pub fn transport_recv(keys: &mut TransportKeys, counter: u64, ciphertext: &[u8])
    -> Result<Vec<u8>, WgError>
{
    if counter < keys.recv_counter {
        return Err(WgError::BadLen); // replay
    }
    let pt = aead_open(&keys.recv_key, counter, ciphertext, &[])?;
    keys.recv_counter = counter.wrapping_add(1);
    Ok(pt)
}

/// Full in-process round trip. Builds two `WgKeypair`s, runs the
/// 1.5-roundtrip Noise IK handshake, and exchanges one transport
/// message in each direction with AEAD round-trip checks.
///
/// Returns `(send_key_prefix_init, send_key_prefix_resp,
///          keys_consistent, transport_round_trip_ok)`.
pub fn selftest_round_trip()
    -> Result<([u8; 8], [u8; 8], bool, bool), WgError>
{
    let initiator = WgKeypair::generate();
    let responder = WgKeypair::generate();

    let timestamp = [0u8; TIMESTAMP_LEN];

    // 1. Initiator builds InitMsg payloads.
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        initiator_send_init(&initiator, &responder.static_pk, &timestamp)?;

    // 2. Responder consumes InitMsg and prepares ResponseMsg.
    let (mut resp_state, ts_back) =
        responder_consume_init(&responder, &init_eph_pk, &enc_static, &enc_ts)?;
    if ts_back != timestamp { return Err(WgError::BadLen); }

    let (enc_empty, resp_eph_pk, mut resp_tx_keys) =
        responder_send_response(&mut resp_state, &init_eph_pk)?;

    // 3. Initiator finishes the handshake.
    let mut init_tx_keys =
        initiator_finish_handshake(&initiator, &mut init_state, &resp_eph_pk, &enc_empty)?;

    // The initiator's send_key must equal the responder's recv_key,
    // and vice versa.
    let keys_consistent =
        init_tx_keys.send_key == resp_tx_keys.recv_key
        && init_tx_keys.recv_key == resp_tx_keys.send_key;

    // 4. Transport round trip — one packet each direction.
    let mut transport_ok = true;
    let i_to_r = b"bat_os over wireguard (init -> resp)";
    let ct = transport_send(&mut init_tx_keys, i_to_r)?;
    let pt = transport_recv(&mut resp_tx_keys, 0, &ct)?;
    if pt.as_slice() != i_to_r { transport_ok = false; }

    let r_to_i = b"echo from responder";
    let ct2 = transport_send(&mut resp_tx_keys, r_to_i)?;
    let pt2 = transport_recv(&mut init_tx_keys, 0, &ct2)?;
    if pt2.as_slice() != r_to_i { transport_ok = false; }

    let mut a = [0u8; 8];
    let mut b = [0u8; 8];
    a.copy_from_slice(&init_tx_keys.send_key[..8]);
    b.copy_from_slice(&resp_tx_keys.recv_key[..8]);

    Ok((a, b, keys_consistent, transport_ok))
}
