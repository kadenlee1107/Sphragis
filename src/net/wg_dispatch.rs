//! WireGuard wire-level dispatcher.
//!
//! Single entry point: `dispatch_wire(bytes)` takes a raw WG wire
//! message (an Init / Response / Cookie / Transport blob exactly as
//! it would arrive over UDP), routes it through `sys_wg_service`
//! plus the `wireguard` wire codec, and returns the dispatcher's
//! result:
//!
//!   - `Reply(bytes)` — bytes to transmit back to the sender (e.g.
//!     a Response to an Init).
//!   - `InboundPacket(plaintext)` — a decrypted transport payload
//!     ready to forward up the stack.
//!   - `Nothing` — message was valid but produced no response
//!     (e.g. a Cookie we don't yet handle).
//!   - `Err(WgError)` — malformed bytes, mac1 failure, replay,
//!     pinned-key mismatch, etc. Caller drops the packet silently
//!     (per spec).
//!
//! Phase-2.5 limits:
//!   - Initiator role isn't implemented — we accept handshake
//!     Initiations and reply with Responses, but never send our
//!     own Init (that's a connect-out flow, future arc).
//!   - Cookie/MAC2 path is not implemented; messages route through
//!     even without cookies. WG accepts that unless the receiver is
//!     rate-limiting.
//!   - peer lookup is naive: on Init we iterate registered peers
//!     and try each until one accepts; on Transport we look up by
//!     `receiver_index` in a small fixed-size table this module
//!     maintains (the `sys-wg` cave doesn't see the index map).
//!
//! Threading: single-threaded contract today (cooperative scheduling,
//! single CPU). The `SESSIONS` table is plain `static mut` with
//! IrqGuard discipline.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use crate::batcave::sys_wg_service::{self, PeerId, SysWgError};
use crate::net::wireguard::{
    self, MSG_TYPE_INIT, MSG_TYPE_RESPONSE, MSG_TYPE_TRANSPORT, MSG_TYPE_COOKIE,
    WgError, KEY_LEN,
};

/// Default WireGuard UDP listen port — matches what `wg-quick`
/// uses unless explicitly configured. The kernel's UDP handler
/// routes packets with `dst_port == WG_LISTEN_PORT` through
/// `dispatch_wire`.
pub const WG_LISTEN_PORT: u16 = 51820;

pub enum WgDispatchResult {
    /// Send these bytes back to the sender over UDP.
    Reply(Vec<u8>),
    /// Decrypted transport plaintext — caller forwards up the stack.
    InboundPacket(Vec<u8>),
    /// Message was valid but produced no observable result.
    Nothing,
    /// Malformed / unauthenticated / replayed message. Caller drops.
    Err(WgError),
}

/// Track our `sender_index` for each established session. The
/// initiator embedded its index in the InitMsg; we picked our own
/// when sending the Response, and reused theirs as `receiver_index`
/// in future Transports we send back. Inbound Transports they send
/// will carry OUR `sender_index` as `receiver_index`, so we use it
/// to look up which peer this session belongs to.
const MAX_SESSIONS: usize = 8;

#[derive(Clone, Copy)]
struct SessionEntry {
    in_use: bool,
    our_sender_index: u32,
    their_sender_index: u32,
    peer_id: PeerId,
}

static mut SESSIONS: [SessionEntry; MAX_SESSIONS] = [SessionEntry {
    in_use: false,
    our_sender_index: 0,
    their_sender_index: 0,
    peer_id: unsafe { core::mem::transmute(0u8) },
}; MAX_SESSIONS];

/// Monotonic counter for sender-index allocation. Restarts at 1 on
/// boot — 0 is reserved as "unset."
static SENDER_INDEX_SEQ: core::sync::atomic::AtomicU32
    = core::sync::atomic::AtomicU32::new(1);

fn alloc_sender_index() -> u32 {
    SENDER_INDEX_SEQ.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

fn install_session(our_idx: u32, their_idx: u32, peer_id: PeerId) -> bool {
    let _g = crate::kernel::sync::IrqGuard::new();
    let sessions = unsafe { &mut *core::ptr::addr_of_mut!(SESSIONS) };
    for slot in sessions.iter_mut() {
        if !slot.in_use {
            *slot = SessionEntry {
                in_use: true,
                our_sender_index: our_idx,
                their_sender_index: their_idx,
                peer_id,
            };
            return true;
        }
    }
    false
}

fn session_by_our_index(our_idx: u32) -> Option<PeerId> {
    let sessions = unsafe { &*core::ptr::addr_of!(SESSIONS) };
    sessions.iter()
        .find(|s| s.in_use && s.our_sender_index == our_idx)
        .map(|s| s.peer_id)
}

/// Drop all session entries — for selftests to start from a known
/// state. NOT exposed publicly outside of selftests.
pub fn debug_clear_sessions() {
    let _g = crate::kernel::sync::IrqGuard::new();
    let sessions = unsafe { &mut *core::ptr::addr_of_mut!(SESSIONS) };
    for slot in sessions.iter_mut() { slot.in_use = false; }
    SENDER_INDEX_SEQ.store(1, core::sync::atomic::Ordering::Relaxed);
}

/// Single-entry dispatcher. Inspects the first byte (message type),
/// routes through the right wireguard.rs parser + sys-wg
/// responder/decrypt, returns a `WgDispatchResult`.
pub fn dispatch_wire(bytes: &[u8]) -> WgDispatchResult {
    if bytes.is_empty() {
        return WgDispatchResult::Err(WgError::BadLen);
    }
    match bytes[0] {
        MSG_TYPE_INIT => dispatch_init(bytes),
        MSG_TYPE_TRANSPORT => dispatch_transport(bytes),
        MSG_TYPE_RESPONSE => {
            // We don't initiate handshakes yet, so an inbound
            // Response we didn't ask for is a stray packet.
            WgDispatchResult::Err(WgError::BadLen)
        }
        MSG_TYPE_COOKIE => {
            // Cookie / DoS path not implemented; spec allows
            // dropping cookies as long as we don't rate-limit.
            WgDispatchResult::Nothing
        }
        _ => WgDispatchResult::Err(WgError::BadLen),
    }
}

fn dispatch_init(bytes: &[u8]) -> WgDispatchResult {
    let our_pk = match sys_wg_service::service_pubkey() {
        Some(pk) => pk,
        None => return WgDispatchResult::Err(WgError::KdfFail),
    };
    let parsed = match wireguard::parse_init_msg(bytes, &our_pk) {
        Ok(p) => p,
        Err(e) => return WgDispatchResult::Err(e),
    };

    // Try each registered peer until one accepts (pinned-key
    // check inside `complete_handshake_as_responder` will reject
    // peers whose pinned static_pk doesn't match what the
    // initiator embedded in `enc_static`).
    let peer_count = sys_wg_service::peer_count();
    if peer_count == 0 {
        return WgDispatchResult::Err(WgError::BadLen);
    }

    let mut session_for: Option<(PeerId, [u8; KEY_LEN], Vec<u8>)> = None;
    for slot in 0u8..(sys_wg_service::MAX_PEERS as u8) {
        let peer_id = PeerId::from(slot);
        if !sys_wg_service::peer_slot_in_use(peer_id) { continue; }
        match sys_wg_service::complete_handshake_as_responder(
            peer_id, &parsed.eph_pk, &parsed.enc_static, &parsed.enc_timestamp,
        ) {
            Ok(resp_wire) => {
                session_for = Some((peer_id, resp_wire.responder_eph_pk, resp_wire.enc_empty));
                break;
            }
            // Pinned mismatch — try next peer. Anything else is
            // a hard error (we leave the door open for legitimate
            // retries of the same init in case of state cleanup).
            Err(SysWgError::Wg(WgError::BadLen)) => continue,
            Err(SysWgError::Wg(other)) => return WgDispatchResult::Err(other),
            Err(_) => continue,
        }
    }

    let (peer_id, responder_eph_pk, enc_empty) = match session_for {
        Some(s) => s,
        None => return WgDispatchResult::Err(WgError::BadMac),
    };

    let our_idx = alloc_sender_index();
    if !install_session(our_idx, parsed.sender_index, peer_id) {
        // Session table full — for now we just drop and let the
        // initiator retry. A real implementation would evict
        // oldest.
        return WgDispatchResult::Err(WgError::KdfFail);
    }

    // We need the INITIATOR's static_pk for the response's mac1
    // computation. That's pinned at register_peer time, exposed
    // via sys_wg_service::peer_static_pk.
    let initiator_pk = match sys_wg_service::peer_static_pk(peer_id) {
        Some(pk) => pk,
        None => return WgDispatchResult::Err(WgError::KdfFail),
    };

    match wireguard::encode_response_msg(
        our_idx, parsed.sender_index,
        &responder_eph_pk, &enc_empty,
        &initiator_pk,
    ) {
        Ok(resp_bytes) => WgDispatchResult::Reply(resp_bytes.to_vec()),
        Err(e) => WgDispatchResult::Err(e),
    }
}

fn dispatch_transport(bytes: &[u8]) -> WgDispatchResult {
    let parsed = match wireguard::parse_transport_msg(bytes) {
        Ok(p) => p,
        Err(e) => return WgDispatchResult::Err(e),
    };
    // Look up which session this packet targets. `receiver_index`
    // is OUR sender_index for that session.
    let peer_id = match session_by_our_index(parsed.receiver_index) {
        Some(p) => p,
        None => return WgDispatchResult::Err(WgError::BadLen),
    };
    match sys_wg_service::unwrap(peer_id, parsed.counter, parsed.ct_with_tag) {
        Ok(pt) => WgDispatchResult::InboundPacket(pt),
        Err(SysWgError::Wg(e)) => WgDispatchResult::Err(e),
        Err(_) => WgDispatchResult::Err(WgError::BadMac),
    }
}

/// Synthetic end-to-end selftest. Walks through:
///   1. Register a peer (caller-side initiator keypair).
///   2. Build InitMsg wire bytes (initiator).
///   3. Drive `dispatch_wire(init)` → `Reply(response_wire)`.
///   4. Parse the response, finish handshake on initiator side.
///   5. Build TransportMsg wire bytes (initiator-side encrypt) for
///      a plaintext.
///   6. Drive `dispatch_wire(transport)` → `InboundPacket(plain)`.
///   7. Verify plaintext matches.
///
/// Returns (handshake_keys_consistent, transport_plaintext_ok).
pub fn selftest() -> Result<(bool, bool), WgError> {
    use wireguard::{
        WgKeypair, TIMESTAMP_LEN, INIT_MSG_LEN,
        TRANSPORT_HDR_LEN,
    };

    debug_clear_sessions();

    let initiator = WgKeypair::generate();

    // Register the initiator's static_pk so sys-wg knows about it.
    // Caller side: also generates an InitiatorState we'll need to
    // finish the handshake later.
    let _ = sys_wg_service::close_peer_by_static_pk(&initiator.static_pk);
    let peer_id = match sys_wg_service::register_peer(initiator.static_pk) {
        Ok(id) => id,
        Err(SysWgError::DuplicatePeer) => {
            // Already pinned from a prior selftest run — fine.
            sys_wg_service::find_peer_by_pk(&initiator.static_pk)
                .ok_or(WgError::KdfFail)?
        }
        Err(_) => return Err(WgError::KdfFail),
    };
    let _ = peer_id;

    let timestamp = [0u8; TIMESTAMP_LEN];
    let our_pk = sys_wg_service::service_pubkey().ok_or(WgError::KdfFail)?;
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        wireguard::initiator_send_init(&initiator, &our_pk, &timestamp)?;
    let initiator_sender_idx = 0x12345678u32;
    let init_wire = wireguard::encode_init_msg(
        initiator_sender_idx,
        &init_eph_pk, &enc_static, &enc_ts,
        &our_pk,
    )?;
    if init_wire.len() != INIT_MSG_LEN { return Err(WgError::BadLen); }

    let reply = match dispatch_wire(&init_wire) {
        WgDispatchResult::Reply(b) => b,
        WgDispatchResult::Err(e) => return Err(e),
        _ => return Err(WgError::KdfFail),
    };

    let parsed_resp = wireguard::parse_response_msg(&reply, &initiator.static_pk)?;
    if parsed_resp.receiver_index != initiator_sender_idx {
        return Err(WgError::BadLen);
    }

    let mut init_keys = wireguard::initiator_finish_handshake(
        &initiator, &mut init_state,
        &parsed_resp.eph_pk,
        &parsed_resp.enc_empty,
    )?;

    let plaintext = b"phase 2.5 end-to-end through dispatch_wire";
    let ct = wireguard::transport_send(&mut init_keys, plaintext)?;
    let mut t_wire = alloc::vec![0u8; TRANSPORT_HDR_LEN + ct.len()];
    wireguard::encode_transport_msg(
        // outgoing: receiver_index = the peer's sender_index (= our_idx
        // from sys-wg's POV, = parsed_resp.sender_index).
        parsed_resp.sender_index, 0, &ct, &mut t_wire,
    )?;

    let inbound = match dispatch_wire(&t_wire) {
        WgDispatchResult::InboundPacket(b) => b,
        WgDispatchResult::Err(e) => return Err(e),
        _ => return Err(WgError::KdfFail),
    };

    let keys_consistent = true; // proven implicitly by transport success
    let transport_ok = inbound.as_slice() == plaintext;

    Ok((keys_consistent, transport_ok))
}
