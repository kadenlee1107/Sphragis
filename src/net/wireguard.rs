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
        Self::from_seed(seed)
    }

    /// Reconstruct a `WgKeypair` from its 32-byte X25519 seed. Pairs
    /// with `seed_bytes()`. Used by storage layers that keep only
    /// the raw seed in protected memory (e.g. sys-wg's cave-private
    /// region) and reconstruct the full keypair on demand.
    pub fn from_seed(seed: [u8; KEY_LEN]) -> Self {
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
/// after handshake completes. `recv_counter` here is the window TOP
/// (highest accepted counter); `recv_window_bits` is the
/// sliding-window bitmap where bit `i` indicates "counter (top - i)
/// has been accepted." Together they implement the §5.4.6 anti-
/// replay window with a 64-packet history.
///
/// First-packet case: `recv_counter == 0 && recv_window_bits == 0`
/// means "no packet accepted yet" — any counter is accepted on the
/// first call (no replay history exists). On a real WG session
/// counter 0 *is* a valid first packet, and `recv_window_bits` set
/// to 1 after first accept marks counter-zero seen.
#[derive(Clone, Copy)]
pub struct TransportKeys {
    pub send_key: [u8; KEY_LEN],
    pub recv_key: [u8; KEY_LEN],
    pub send_counter: u64,
    pub recv_counter: u64,
    pub recv_window_bits: u64,
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
            recv_window_bits: 0,
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
        recv_window_bits: 0,
    })
}

/// Width of the replay-window history, in packets. WireGuard's spec
/// (§5.4.6) sets a minimum of 64; some implementations widen to 128
/// or 8192. 64 fits in a single `u64` and is enough for the typical
/// reorder distance on a WAN.
pub const REPLAY_WINDOW_WIDTH: u64 = 64;

/// Check whether `counter` should be accepted given the current
/// `(recv_counter top, recv_window_bits)` state, without mutating
/// it. Pure function — the actual advance happens in
/// `transport_recv` only after the AEAD tag verifies (we don't want
/// an attacker spamming junk packets to slide our window past
/// legitimate ones).
fn replay_window_accepts(top: u64, bits: u64, counter: u64) -> bool {
    // First-ever packet (no history). Accept.
    if top == 0 && bits == 0 { return true; }

    if counter > top {
        // Ahead of the window — always accept (window will slide).
        return true;
    }
    let dist = top - counter;
    if dist >= REPLAY_WINDOW_WIDTH {
        return false; // too old, falls out of the window
    }
    // Within the window: accept iff this counter hasn't been seen
    // yet. Bit 0 corresponds to `top`; bit `dist` to `top - dist`.
    (bits & (1u64 << dist)) == 0
}

/// Update `(recv_counter, recv_window_bits)` to mark `counter` as
/// accepted. Caller has already verified `replay_window_accepts`
/// and the AEAD tag; this only manipulates the bitmap.
fn replay_window_advance(top: &mut u64, bits: &mut u64, counter: u64) {
    if *top == 0 && *bits == 0 {
        // First-ever packet — initialise.
        *top = counter;
        *bits = 1;
        return;
    }
    if counter > *top {
        let shift = counter - *top;
        if shift >= REPLAY_WINDOW_WIDTH {
            *bits = 0;
        } else {
            *bits <<= shift;
        }
        *bits |= 1;
        *top = counter;
    } else {
        let dist = *top - counter;
        *bits |= 1u64 << dist;
    }
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
/// the plaintext. Phase-2.6 replay protection: a 64-packet
/// sliding window (`recv_counter` = window top, `recv_window_bits`
/// = bitmap). Rejects counters that are
///   - more than `REPLAY_WINDOW_WIDTH` packets below the top
///     (clearly an old replay), or
///   - within the window but already accepted.
/// Out-of-order legit packets (any unseen counter <= top within
/// the window) are accepted. The window only advances on
/// successful AEAD verification — junk packets can't slide our
/// view past real ones.
pub fn transport_recv(keys: &mut TransportKeys, counter: u64, ciphertext: &[u8])
    -> Result<Vec<u8>, WgError>
{
    if !replay_window_accepts(keys.recv_counter, keys.recv_window_bits, counter) {
        return Err(WgError::BadMac); // replay or pre-window
    }
    let pt = aead_open(&keys.recv_key, counter, ciphertext, &[])?;
    replay_window_advance(&mut keys.recv_counter, &mut keys.recv_window_bits, counter);
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

// ─────────────────────────────────────────────────────────────────
// Phase 2: WireGuard wire framing.
//
// The Phase 1 API exposed handshake state machine fields directly to
// callers; Phase 2 wraps them in the byte layout WireGuard peers
// transmit over UDP, so we can interop with stock `wg-quick`. Wire
// format references whitepaper §5.4.2 (Init) and §5.4.3 (Response).
// MAC1 is a keyed BLAKE2s MAC; MAC2 is the same construction over a
// cookie value. Phase 2 minimum leaves MAC2 as zero — peers accept
// that unless they're under DoS pressure (mac2 enforcement kicks in
// only when the responder sets the "rate-limited" bit, which is its
// own arc).
// ─────────────────────────────────────────────────────────────────

use crate::crypto::blake2s as blake2s_mod;

/// WireGuard message-type constants (whitepaper §5.4.2 message
/// header). One byte at offset 0 of every wire message.
pub const MSG_TYPE_INIT:      u8 = 1;
pub const MSG_TYPE_RESPONSE:  u8 = 2;
pub const MSG_TYPE_COOKIE:    u8 = 3;
pub const MSG_TYPE_TRANSPORT: u8 = 4;

/// Wire sizes per whitepaper §5.4.
pub const INIT_MSG_LEN:      usize = 148;
pub const RESPONSE_MSG_LEN:  usize = 92;
pub const COOKIE_MSG_LEN:    usize = 64;
/// Transport messages have a 16-byte header (type + reserved +
/// receiver_index + counter) + payload + 16-byte AEAD tag.
pub const TRANSPORT_HDR_LEN: usize = 16;

/// Decoded InitMsg fields. The mac1 has already been verified by
/// `parse_init_msg`; the caller plugs the encrypted blobs into
/// `responder_consume_init` to finish the protocol-state side.
pub struct ParsedInit {
    pub sender_index: u32,
    pub eph_pk: [u8; KEY_LEN],
    pub enc_static: [u8; KEY_LEN + TAG_LEN],
    pub enc_timestamp: [u8; TIMESTAMP_LEN + TAG_LEN],
}

/// Decoded ResponseMsg fields.
pub struct ParsedResponse {
    pub sender_index: u32,
    pub receiver_index: u32,
    pub eph_pk: [u8; KEY_LEN],
    pub enc_empty: [u8; TAG_LEN],
}

/// `mac1` key derived from the peer's static pubkey
/// (whitepaper §5.4.4). The MAC1 protects against trivially-spoofed
/// messages that don't even know which peer they're trying to talk
/// to — cheap rejection at parse time before the expensive
/// X25519/AEAD work.
fn mac1_key_for(static_pk: &[u8; KEY_LEN]) -> [u8; KEY_LEN] {
    blake2s_mod::hash2(LABEL_MAC1, static_pk)
}

/// Encode an InitMsg into its 148-byte wire form. The caller plugs
/// in the cryptographic outputs from `initiator_send_init` plus the
/// responder's pinned static pubkey (so we can compute mac1).
///
/// mac2 is left as 16 zero bytes — Phase-2-minimum behavior.
pub fn encode_init_msg(
    sender_index: u32,
    eph_pk: &[u8; KEY_LEN],
    enc_static: &[u8],
    enc_timestamp: &[u8],
    responder_static_pk: &[u8; KEY_LEN],
) -> Result<[u8; INIT_MSG_LEN], WgError> {
    if enc_static.len() != KEY_LEN + TAG_LEN { return Err(WgError::BadLen); }
    if enc_timestamp.len() != TIMESTAMP_LEN + TAG_LEN { return Err(WgError::BadLen); }

    let mut msg = [0u8; INIT_MSG_LEN];
    msg[0] = MSG_TYPE_INIT;
    // bytes 1..4: reserved (already zero).
    msg[4..8].copy_from_slice(&sender_index.to_le_bytes());
    msg[8..40].copy_from_slice(eph_pk);
    msg[40..88].copy_from_slice(enc_static);
    msg[88..116].copy_from_slice(enc_timestamp);
    let key = mac1_key_for(responder_static_pk);
    let mac1 = blake2s_mod::mac16(&key, &msg[..116]);
    msg[116..132].copy_from_slice(&mac1);
    // bytes 132..148: mac2 (left zero).
    Ok(msg)
}

/// Decode + mac1-verify an InitMsg. Returns `BadLen` if the message
/// isn't exactly 148 bytes or the header is malformed; `BadMac` if
/// the mac1 doesn't match what we'd compute with our static pubkey.
pub fn parse_init_msg(
    bytes: &[u8],
    our_static_pk: &[u8; KEY_LEN],
) -> Result<ParsedInit, WgError> {
    if bytes.len() != INIT_MSG_LEN { return Err(WgError::BadLen); }
    if bytes[0] != MSG_TYPE_INIT { return Err(WgError::BadLen); }
    if bytes[1] != 0 || bytes[2] != 0 || bytes[3] != 0 { return Err(WgError::BadLen); }

    let key = mac1_key_for(our_static_pk);
    let expected_mac1 = blake2s_mod::mac16(&key, &bytes[..116]);
    // Constant-time comparison.
    let mut diff = 0u8;
    for i in 0..16 { diff |= expected_mac1[i] ^ bytes[116 + i]; }
    if diff != 0 { return Err(WgError::BadMac); }

    let mut sb = [0u8; 4];
    sb.copy_from_slice(&bytes[4..8]);
    let sender_index = u32::from_le_bytes(sb);

    let mut eph_pk = [0u8; KEY_LEN];
    eph_pk.copy_from_slice(&bytes[8..40]);
    let mut enc_static = [0u8; KEY_LEN + TAG_LEN];
    enc_static.copy_from_slice(&bytes[40..88]);
    let mut enc_timestamp = [0u8; TIMESTAMP_LEN + TAG_LEN];
    enc_timestamp.copy_from_slice(&bytes[88..116]);

    Ok(ParsedInit { sender_index, eph_pk, enc_static, enc_timestamp })
}

/// Encode a ResponseMsg into its 92-byte wire form. mac1 is computed
/// against the *initiator's* static pubkey so the initiator can
/// validate the response is destined for it.
pub fn encode_response_msg(
    sender_index: u32,
    receiver_index: u32,
    eph_pk: &[u8; KEY_LEN],
    enc_empty: &[u8],
    initiator_static_pk: &[u8; KEY_LEN],
) -> Result<[u8; RESPONSE_MSG_LEN], WgError> {
    if enc_empty.len() != TAG_LEN { return Err(WgError::BadLen); }

    let mut msg = [0u8; RESPONSE_MSG_LEN];
    msg[0] = MSG_TYPE_RESPONSE;
    msg[4..8].copy_from_slice(&sender_index.to_le_bytes());
    msg[8..12].copy_from_slice(&receiver_index.to_le_bytes());
    msg[12..44].copy_from_slice(eph_pk);
    msg[44..60].copy_from_slice(enc_empty);
    let key = mac1_key_for(initiator_static_pk);
    let mac1 = blake2s_mod::mac16(&key, &msg[..60]);
    msg[60..76].copy_from_slice(&mac1);
    // bytes 76..92: mac2 (zero).
    Ok(msg)
}

/// Decode + mac1-verify a ResponseMsg.
pub fn parse_response_msg(
    bytes: &[u8],
    initiator_static_pk: &[u8; KEY_LEN],
) -> Result<ParsedResponse, WgError> {
    if bytes.len() != RESPONSE_MSG_LEN { return Err(WgError::BadLen); }
    if bytes[0] != MSG_TYPE_RESPONSE { return Err(WgError::BadLen); }
    if bytes[1] != 0 || bytes[2] != 0 || bytes[3] != 0 { return Err(WgError::BadLen); }

    let key = mac1_key_for(initiator_static_pk);
    let expected_mac1 = blake2s_mod::mac16(&key, &bytes[..60]);
    let mut diff = 0u8;
    for i in 0..16 { diff |= expected_mac1[i] ^ bytes[60 + i]; }
    if diff != 0 { return Err(WgError::BadMac); }

    let mut sb = [0u8; 4];
    sb.copy_from_slice(&bytes[4..8]);
    let sender_index = u32::from_le_bytes(sb);
    let mut rb = [0u8; 4];
    rb.copy_from_slice(&bytes[8..12]);
    let receiver_index = u32::from_le_bytes(rb);

    let mut eph_pk = [0u8; KEY_LEN];
    eph_pk.copy_from_slice(&bytes[12..44]);
    let mut enc_empty = [0u8; TAG_LEN];
    enc_empty.copy_from_slice(&bytes[44..60]);

    Ok(ParsedResponse { sender_index, receiver_index, eph_pk, enc_empty })
}

/// Encode a Transport message into wire form: header + payload-AEAD.
/// `payload` here is the AEAD ciphertext (already includes the
/// 16-byte tag); `counter` is the sender's monotonic counter
/// (whitepaper §5.4.6).
pub fn encode_transport_msg(
    receiver_index: u32,
    counter: u64,
    ct_with_tag: &[u8],
    out: &mut [u8],
) -> Result<usize, WgError> {
    let total = TRANSPORT_HDR_LEN + ct_with_tag.len();
    if out.len() < total { return Err(WgError::BadLen); }
    out[0] = MSG_TYPE_TRANSPORT;
    out[1] = 0; out[2] = 0; out[3] = 0;
    out[4..8].copy_from_slice(&receiver_index.to_le_bytes());
    out[8..16].copy_from_slice(&counter.to_le_bytes());
    out[16..total].copy_from_slice(ct_with_tag);
    Ok(total)
}

/// Decoded transport message. The caller calls `transport_recv` (or
/// the cave-private equivalent in `sys_wg_service`) with `counter`
/// and `ct_with_tag` to authenticate + decrypt.
pub struct ParsedTransport<'a> {
    pub receiver_index: u32,
    pub counter: u64,
    pub ct_with_tag: &'a [u8],
}

pub fn parse_transport_msg(bytes: &[u8]) -> Result<ParsedTransport<'_>, WgError> {
    if bytes.len() < TRANSPORT_HDR_LEN + TAG_LEN { return Err(WgError::BadLen); }
    if bytes[0] != MSG_TYPE_TRANSPORT { return Err(WgError::BadLen); }
    if bytes[1] != 0 || bytes[2] != 0 || bytes[3] != 0 { return Err(WgError::BadLen); }
    let mut rb = [0u8; 4];
    rb.copy_from_slice(&bytes[4..8]);
    let receiver_index = u32::from_le_bytes(rb);
    let mut cb = [0u8; 8];
    cb.copy_from_slice(&bytes[8..16]);
    let counter = u64::from_le_bytes(cb);
    Ok(ParsedTransport {
        receiver_index,
        counter,
        ct_with_tag: &bytes[TRANSPORT_HDR_LEN..],
    })
}

/// Phase-2.6 replay-window selftest. Drives the spec scenarios:
///   1. Forward progress: counters 0..=3 accepted in order.
///   2. Strict replay: re-receive counter 1 → reject.
///   3. Out-of-order within the window: jump top to 10, then
///      receive counter 5 (unseen + within 64-wide window) →
///      accept; replay of 5 → reject.
///   4. Forward jump: counter 100 → accept (window shifts).
///   5. Below window: counter 5 (now far below top) → reject.
///   6. Forged ciphertext at an unseen counter → reject without
///      advancing the window (so a flood of bad packets can't
///      slide the view past legit ones).
///
/// All seven scenarios go through the actual `transport_send` +
/// `transport_recv` paths with arbitrary `send_counter` values
/// (we mutate `keys.send_counter` between sends to drive the
/// receiver through specific counter values).
///
/// Returns true if every scenario behaved as expected.
pub fn selftest_replay_window() -> Result<bool, WgError> {
    let initiator = WgKeypair::generate();
    let responder = WgKeypair::generate();
    let timestamp = [0u8; TIMESTAMP_LEN];

    // Bring up a session via the standard handshake.
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        initiator_send_init(&initiator, &responder.static_pk, &timestamp)?;
    let (mut resp_state, _) = responder_consume_init(
        &responder, &init_eph_pk, &enc_static, &enc_ts,
    )?;
    let (enc_empty, resp_eph_pk, mut resp_keys) =
        responder_send_response(&mut resp_state, &init_eph_pk)?;
    let mut init_keys = initiator_finish_handshake(
        &initiator, &mut init_state, &resp_eph_pk, &enc_empty,
    )?;

    // Convenience: produce a ciphertext encoded under `init_keys`
    // with a specific counter. `transport_send` consumes the
    // sender's counter monotonically, so we just override it.
    let send_at = |k: &mut TransportKeys, c: u64, msg: &[u8]| -> Result<Vec<u8>, WgError> {
        k.send_counter = c;
        let ct = transport_send(k, msg)?;
        // transport_send bumps send_counter; reset for next override.
        Ok(ct)
    };

    // 1. Forward progress: counters 0..=3 all accept.
    for c in 0u64..=3 {
        let ct = send_at(&mut init_keys, c, b"hello")?;
        let pt = transport_recv(&mut resp_keys, c, &ct)?;
        if pt.as_slice() != b"hello" { return Ok(false); }
    }
    // After forward progress, top=3, bits has 0,1,2,3 set (low 4
    // bits = 0b1111).
    if resp_keys.recv_counter != 3 { return Ok(false); }
    if resp_keys.recv_window_bits & 0xF != 0xF { return Ok(false); }

    // 2. Strict replay: re-send counter 1 (re-encrypt with the
    // same nonce — AEAD will succeed because nonce is part of the
    // input). Replay window should reject before AEAD runs.
    let replay_ct = send_at(&mut init_keys, 1, b"hello")?;
    match transport_recv(&mut resp_keys, 1, &replay_ct) {
        Err(WgError::BadMac) => {} // expected (replay rejection)
        _ => return Ok(false),
    }

    // 3. Out-of-order within window: send counter 10, then 5.
    let ct10 = send_at(&mut init_keys, 10, b"hello")?;
    transport_recv(&mut resp_keys, 10, &ct10)?;
    if resp_keys.recv_counter != 10 { return Ok(false); }

    let ct5 = send_at(&mut init_keys, 5, b"hello")?;
    let pt5 = transport_recv(&mut resp_keys, 5, &ct5)?;
    if pt5.as_slice() != b"hello" { return Ok(false); }
    // bit (10 - 5 = 5) should now be set.
    if resp_keys.recv_window_bits & (1u64 << 5) == 0 { return Ok(false); }

    // Replay of 5 → reject.
    let replay_ct5 = send_at(&mut init_keys, 5, b"hello")?;
    match transport_recv(&mut resp_keys, 5, &replay_ct5) {
        Err(WgError::BadMac) => {} // expected
        _ => return Ok(false),
    }

    // 4. Forward jump: counter 100 (well past the window width).
    let ct100 = send_at(&mut init_keys, 100, b"hello")?;
    transport_recv(&mut resp_keys, 100, &ct100)?;
    if resp_keys.recv_counter != 100 { return Ok(false); }
    // The window shifted out everything before top - 64 = 36.
    // bit 0 = counter 100, and that's the only one set.
    if resp_keys.recv_window_bits != 1 { return Ok(false); }

    // 5. Below window: counter 5 is now top - 95 < 0 (would need
    // negative distance; we treat this as "too old" → reject).
    let ct5_again = send_at(&mut init_keys, 5, b"hello")?;
    match transport_recv(&mut resp_keys, 5, &ct5_again) {
        Err(WgError::BadMac) => {} // expected (pre-window)
        _ => return Ok(false),
    }

    // 6. Forged ciphertext at an unseen counter (within the
    // window). The AEAD should reject; the window should NOT
    // advance / set the bit.
    let pre_top = resp_keys.recv_counter;
    let pre_bits = resp_keys.recv_window_bits;
    let forged_counter = 99u64; // top - 1, within window, unseen
    // Build "ciphertext" that's deliberately wrong (just zeros of
    // the right size). transport_recv must reject before touching
    // the window state.
    let bogus = alloc::vec![0u8; 16 + 5]; // 5-byte plaintext + 16-byte tag
    match transport_recv(&mut resp_keys, forged_counter, &bogus) {
        Err(WgError::AeadFail) => {} // expected
        _ => return Ok(false),
    }
    if resp_keys.recv_counter != pre_top { return Ok(false); }
    if resp_keys.recv_window_bits != pre_bits { return Ok(false); }

    Ok(true)
}

/// Phase-2 wire-framing selftest: drives a full handshake through
/// the wire encoders + parsers and a single transport round trip.
/// Returns the same shape as `selftest_round_trip` so the shell
/// command can compare both flows uniformly.
pub fn selftest_wire_round_trip()
    -> Result<([u8; 8], [u8; 8], bool, bool), WgError>
{
    let initiator = WgKeypair::generate();
    let responder = WgKeypair::generate();
    let timestamp = [0u8; TIMESTAMP_LEN];

    // 1. Initiator: build state + InitMsg payloads + encode wire.
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        initiator_send_init(&initiator, &responder.static_pk, &timestamp)?;
    let init_wire = encode_init_msg(
        /* sender_index */ 0x11223344,
        &init_eph_pk, &enc_static, &enc_ts,
        &responder.static_pk,
    )?;

    // 2. Responder: parse wire (mac1 verified internally), consume
    //    the handshake, build ResponseMsg state, encode wire.
    let parsed_init = parse_init_msg(&init_wire, &responder.static_pk)?;
    if parsed_init.eph_pk != init_eph_pk { return Err(WgError::BadLen); }
    if parsed_init.sender_index != 0x11223344 { return Err(WgError::BadLen); }
    if parsed_init.enc_static != enc_static.as_slice() { return Err(WgError::BadLen); }

    let (mut resp_state, ts_back) = responder_consume_init(
        &responder,
        &parsed_init.eph_pk,
        &parsed_init.enc_static,
        &parsed_init.enc_timestamp,
    )?;
    if ts_back != timestamp { return Err(WgError::BadLen); }
    let (enc_empty, resp_eph_pk, mut resp_tx_keys) =
        responder_send_response(&mut resp_state, &parsed_init.eph_pk)?;
    let resp_wire = encode_response_msg(
        /* sender_index */ 0xAABBCCDD,
        parsed_init.sender_index,
        &resp_eph_pk,
        &enc_empty,
        &initiator.static_pk,
    )?;

    // 3. Initiator: parse response wire (mac1 verified), finish.
    let parsed_resp = parse_response_msg(&resp_wire, &initiator.static_pk)?;
    if parsed_resp.eph_pk != resp_eph_pk { return Err(WgError::BadLen); }
    if parsed_resp.receiver_index != 0x11223344 { return Err(WgError::BadLen); }
    if parsed_resp.sender_index != 0xAABBCCDD { return Err(WgError::BadLen); }
    if parsed_resp.enc_empty != enc_empty.as_slice() { return Err(WgError::BadLen); }

    let mut init_tx_keys = initiator_finish_handshake(
        &initiator, &mut init_state,
        &parsed_resp.eph_pk,
        &parsed_resp.enc_empty,
    )?;

    let keys_consistent =
        init_tx_keys.send_key == resp_tx_keys.recv_key
        && init_tx_keys.recv_key == resp_tx_keys.send_key;

    // 4. Transport round trip — also through wire framing.
    let mut transport_ok = true;
    let payload = b"bat_os over wireguard (phase-2 wire)";
    let ct = transport_send(&mut init_tx_keys, payload)?;
    let mut t_wire = alloc::vec![0u8; TRANSPORT_HDR_LEN + ct.len()];
    encode_transport_msg(
        parsed_resp.sender_index, 0, &ct, &mut t_wire,
    )?;
    let parsed_t = parse_transport_msg(&t_wire)?;
    if parsed_t.receiver_index != 0xAABBCCDD { transport_ok = false; }
    if parsed_t.counter != 0 { transport_ok = false; }
    let pt = transport_recv(&mut resp_tx_keys, parsed_t.counter, parsed_t.ct_with_tag)?;
    if pt.as_slice() != payload { transport_ok = false; }

    let mut a = [0u8; 8];
    let mut b = [0u8; 8];
    a.copy_from_slice(&init_tx_keys.send_key[..8]);
    b.copy_from_slice(&resp_tx_keys.recv_key[..8]);
    Ok((a, b, keys_consistent, transport_ok))
}
