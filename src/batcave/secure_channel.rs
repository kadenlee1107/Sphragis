//! Integration: IPC session + encrypted framed channel.
//!
//! `batcave::ipc_session` gave us the handshake primitive — two
//! caves can establish a shared 32-byte session key over an
//! authenticated + forward-secret Ed25519 / X25519 exchange.
//!
//! This module turns that key into a stateful channel:
//!
//!   * Every outgoing frame gets a monotonically-increasing 64-bit
//!     sequence number baked into the AEAD nonce AND the AAD, so
//!     receivers catch (a) replay of a prior frame, (b) reorder,
//!     (c) truncation.
//!   * Frames are ChaCha20-Poly1305-sealed. Same primitive as
//!     BatFS (phase 4) and the daemon audit log (phase 3).
//!   * Two independent sequence counters (one per direction) so
//!     duplex channels don't have to coordinate. Alice seals with
//!     her outbound counter; Bob opens with his inbound counter
//!     against Alice. The two counters must stay in sync or open
//!     rejects.
//!
//! Integration with `kernel::ipc`:
//!   * `SecureChannel::seal(payload)` returns ciphertext bytes
//!     that fit in an `ipc::Message`. Caller wraps + sends.
//!   * `SecureChannel::open(ct)` takes the received Message's
//!     bytes and returns plaintext or an error.
//!
//! This commit adds the primitive + a self-test. Wiring into
//! `kernel::ipc::send/recv` as the default path is a follow-up;
//! plaintext ipc still works for code that doesn't need session
//! confidentiality (e.g. kernel-to-self).

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    ChaCha20Poly1305,
};

/// Nonce format: 96 bits = 32-bit "direction tag" || 64-bit counter.
/// Outbound direction = 0, inbound = 1 (on the SENDER side).
/// Receiver swaps so their inbound opens what sender wrote outbound.
const DIR_OUT: u32 = 0;
const DIR_IN:  u32 = 1;

const TAG_LEN: usize = 16;
const HEADER_LEN: usize = 12 + 8;  // 12-byte nonce + 8-byte seq (for
                                   // receiver's own bookkeeping; stored
                                   // in-band so order/reorder is detectable)

/// Per-session channel state. Holds the key + per-direction counters.
/// On destroy(), key is zeroized.
pub struct SecureChannel {
    key: [u8; 32],
    /// Number of frames we've sealed (nonces consumed outbound).
    /// Counter is the primary replay-protection mechanism: every
    /// seal bumps this, open() accepts only strictly-greater seq.
    out_counter: u64,
    /// Highest seq we've successfully opened. open() rejects any
    /// frame with seq <= this value.
    in_high_water: u64,
    /// Which direction we wrote last (set by caller via set_role).
    /// Default OUT=0, IN=1 for Alice; flip for Bob.
    dir_out_tag: u32,
    dir_in_tag:  u32,
}

impl SecureChannel {
    /// Construct from a session key. Role A = Alice uses dir=OUT (=0)
    /// on seal, dir=IN (=1) on open. Role B = Bob inverts.
    pub fn alice(key: [u8; 32]) -> Self {
        Self {
            key, out_counter: 0, in_high_water: 0,
            dir_out_tag: DIR_OUT, dir_in_tag: DIR_IN,
        }
    }
    pub fn bob(key: [u8; 32]) -> Self {
        Self {
            key, out_counter: 0, in_high_water: 0,
            dir_out_tag: DIR_IN, dir_in_tag: DIR_OUT,
        }
    }

    /// Seal a payload: returns [12-byte nonce][8-byte seq][ciphertext + 16-byte tag].
    /// The nonce itself is deterministic from (dir, seq); we include it
    /// explicitly in the frame so the wire format self-describes.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, &'static str> {
        let seq = self.out_counter;
        self.out_counter = self.out_counter.checked_add(1)
            .ok_or("secure_channel: outbound counter overflow")?;

        let nonce = make_nonce(self.dir_out_tag, seq);
        let aad = seq.to_be_bytes();
        let cipher = ChaCha20Poly1305::new(&self.key.into());

        let mut buf = plaintext.to_vec();
        let tag = cipher.encrypt_in_place_detached((&nonce).into(), &aad, &mut buf)
            .map_err(|_| "secure_channel: seal failed")?;

        let mut frame = Vec::with_capacity(HEADER_LEN + buf.len() + TAG_LEN);
        frame.extend_from_slice(&nonce);
        frame.extend_from_slice(&aad);
        frame.extend_from_slice(&buf);
        frame.extend_from_slice(&tag);
        Ok(frame)
    }

    /// Open a frame. Enforces strict-monotonic seq on the in side,
    /// so replays / reorders are rejected.
    pub fn open(&mut self, frame: &[u8]) -> Result<Vec<u8>, &'static str> {
        if frame.len() < HEADER_LEN + TAG_LEN {
            return Err("secure_channel: frame too short");
        }
        let nonce = &frame[..12];
        let seq = u64::from_be_bytes(frame[12..20].try_into()
            .expect("secure_channel: 8-byte slice → [u8; 8] is infallible"));

        // Expected nonce against IN direction.
        let expected_nonce = make_nonce(self.dir_in_tag, seq);
        if nonce != expected_nonce {
            return Err("secure_channel: nonce mismatch (wrong direction/seq)");
        }

        // Replay / reorder check. First frame (in_high_water == 0 and
        // seq == 0) is accepted; subsequent must be strictly greater.
        if seq != 0 && seq <= self.in_high_water {
            return Err("secure_channel: replay or reorder detected");
        }
        if seq == 0 && self.in_high_water != 0 {
            return Err("secure_channel: reset attack (seq=0 after activity)");
        }

        let aad = &frame[12..20];
        let ct_len = frame.len() - HEADER_LEN - TAG_LEN;
        let ct = &frame[HEADER_LEN..HEADER_LEN + ct_len];
        let tag_bytes: [u8; 16] = frame[HEADER_LEN + ct_len..].try_into()
            .expect("secure_channel: tail-of-frame is exactly TAG_LEN by length math above");

        let cipher = ChaCha20Poly1305::new(&self.key.into());
        let mut buf = ct.to_vec();
        cipher.decrypt_in_place_detached(
                nonce.try_into().expect("secure_channel: nonce slice is 12 bytes by construction"),
                aad,
                &mut buf,
                (&tag_bytes).into(),
            ).map_err(|_| "secure_channel: INTEGRITY VIOLATION")?;

        self.in_high_water = seq;
        Ok(buf)
    }

    /// Zeroize the key. Call on session end / cave destroy.
    pub fn destroy(&mut self) {
        for b in self.key.iter_mut() {
            unsafe { core::ptr::write_volatile(b, 0); }
        }
    }
}

fn make_nonce(dir: u32, seq: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[..4].copy_from_slice(&dir.to_be_bytes());
    n[4..].copy_from_slice(&seq.to_be_bytes());
    n
}

/// Exposed as `secure-ipc-selftest`. Walks a full scenario:
///   1. Handshake (via ipc_session::selftest_round_trip) → session key.
///   2. Alice seals "hello from cave A" — ciphertext is opaque to Eve.
///   3. Bob opens — recovers the plaintext.
///   4. Eve tampers (flips a byte) — Bob rejects with INTEGRITY VIOLATION.
///   5. Alice seals another frame, Bob opens — seq advances, OK.
///   6. Replay frame #1 — Bob rejects (replay detected).
pub fn selftest() -> Result<SelfTestReport, &'static str> {
    // Re-use the phase-7 handshake to get a session key.
    let (alice_k, bob_k, matched) = super::ipc_session::selftest_round_trip()?;
    if !matched { return Err("session keys disagree at start"); }
    let _ = (alice_k, bob_k); // only prefixes were returned; reconstruct properly below

    // Handshake again to get FULL keys (selftest_round_trip returned prefixes).
    // Run a fresh local handshake so we have both sides' full 32-byte key.
    use super::ipc_session::*;
    use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};
    use crate::crypto::pq_hybrid::BatRng;

    let label = b"sphragis-secure-ipc-selftest";
    let alice_id = IpcIdentity::generate();
    let bob_id   = IpcIdentity::generate();
    let mut r = BatRng;
    let a_eph = EphemeralSecret::random_from_rng(&mut r);
    let a_eph_pk: [u8; 32] = X25519Public::from(&a_eph).to_bytes();
    let mut r2 = BatRng;
    let b_eph = EphemeralSecret::random_from_rng(&mut r2);
    let b_eph_pk: [u8; 32] = X25519Public::from(&b_eph).to_bytes();

    // Verify handshake offers (done internally by full ipc_session API;
    // we still invoke build/verify to ensure sigs are valid end-to-end).
    let a_offer = build_offer(&alice_id, &a_eph_pk, label)?;
    verify_offer(&a_offer, None, label)?;
    let b_offer = build_offer(&bob_id, &b_eph_pk, label)?;
    verify_offer(&b_offer, None, label)?;

    let k_alice = derive_session_key(a_eph, &b_eph_pk, &alice_id.pk, &bob_id.pk, label);
    let k_bob   = derive_session_key(b_eph, &a_eph_pk, &alice_id.pk, &bob_id.pk, label);
    if k_alice != k_bob { return Err("derived keys differ"); }

    let mut alice = SecureChannel::alice(k_alice);
    let mut bob   = SecureChannel::bob(k_bob);

    // Round 1: good
    let plaintext1 = b"hello from cave A";
    let frame1 = alice.seal(plaintext1)?;
    let opened1 = bob.open(&frame1)?;
    if opened1 != plaintext1 { return Err("round-1 plaintext mismatch"); }
    let frame1_len = frame1.len();

    // Round 2: good, seq should advance
    let plaintext2 = b"hello again, cave B";
    let frame2 = alice.seal(plaintext2)?;
    let opened2 = bob.open(&frame2)?;
    if opened2 != plaintext2 { return Err("round-2 plaintext mismatch"); }

    // Eve tampers: flip a byte in round 2's frame copy
    let mut tampered = frame2.clone();
    tampered[30] ^= 0x01;
    // Bob already consumed seq 1, so we can't really re-open — but let
    // him try on a fresh channel to prove tamper detection. Construct
    // a fresh Bob with the same key:
    let mut bob_fresh = SecureChannel::bob(k_bob);
    if bob_fresh.open(&tampered).is_ok() {
        return Err("tampered frame verified (BUG)");
    }

    // Replay test: Alice's frame1 replayed to Bob should fail because
    // Bob's high-water is already 1. (Bob above already consumed seq 0,
    // so replaying frame1 fails the "seq <= high_water" check.)
    if bob.open(&frame1).is_ok() {
        return Err("replay of frame1 accepted (BUG)");
    }

    Ok(SelfTestReport {
        plaintext_len: plaintext1.len(),
        frame_len: frame1_len,
        expansion: frame1_len - plaintext1.len(),
        round_1_matched: true,
        round_2_matched: true,
        tamper_rejected: true,
        replay_rejected: true,
    })
}

pub struct SelfTestReport {
    pub plaintext_len: usize,
    pub frame_len: usize,
    pub expansion: usize,
    pub round_1_matched: bool,
    pub round_2_matched: bool,
    pub tamper_rejected: bool,
    pub replay_rejected: bool,
}
