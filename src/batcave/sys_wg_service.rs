//! sys-wg service — Arc 3 slices 1 & 2.
//!
//! Encapsulates WireGuard state (static keypair, per-peer transport
//! keys) inside the sys-wg cave's module privacy boundary. Public
//! callers operate the WG state machine only through this module's
//! API; neither the static secret nor any peer's session keys are
//! exposed.
//!
//! Each public entry point runs its actual work via `with_sys_wg_cave`,
//! which:
//!   1. records the caller's current `cave_id` + `TTBR0_EL1`,
//!   2. tags the running task with sys-wg's `cave_id` and loads
//!      sys-wg's L1 into TTBR0 (so the scheduler MMU hook keeps it
//!      there across yields *during* the call too),
//!   3. invokes the closure,
//!   4. restores the caller's `cave_id` + TTBR0 before returning.
//!
//! Today the kernel boots with MMU off in the serial-shell path, so
//! the TTBR0 swap is a register-write only — no hardware translation
//! change. When `setup_and_enable` becomes part of the kernel boot
//! sequence, the same code path will give real cross-cave memory
//! isolation: a caller cannot read sys-wg's static key from its own
//! L1, because the closure's execution happens with sys-wg's L1
//! installed.
//!
//! Phase plan:
//!   * Slice 1: single-session in-process API.
//!     `init()` lazy-creates the WgKeypair; `debug_local_round_trip`
//!     runs an Arc-3 selftest that proves the keys never escape.
//!   * Slice 2 (this commit): peer-table-keyed API.
//!     `register_peer(static_pk) -> PeerId`,
//!     `complete_handshake_as_responder(peer_id, init_msg) -> wire`,
//!     `wrap(peer_id, pt)`, `unwrap(peer_id, counter, ct)`. The
//!     responder-side TransportKeys live inside sys-wg's peer table
//!     and never escape to the caller.
//!   * Slice 3 (after the kernel-boot MMU enable lands): the actual
//!     IPC mailbox + service task so the boundary is hardware-enforced,
//!     not just module-private.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use crate::batcave::{cave, sys_caves};
use crate::kernel::process;
use crate::net::wireguard::{
    self, InitiatorState, ResponderState, TransportKeys, WgKeypair, WgError,
};

/// Identity keypair for sys-wg. Allocated on first `init()` call and
/// kept inside this module for the life of the boot — *no* getter
/// exposes the secret half. The pubkey is reachable via
/// `service_pubkey()`.
static mut KEYPAIR: Option<WgKeypair> = None;

/// `service_pubkey()` is what callers pin against. Returned by value
/// (a 32-byte X25519 public key) so the caller never holds a borrow
/// into our state.
pub fn service_pubkey() -> Option<[u8; wireguard::KEY_LEN]> {
    ensure_init();
    unsafe {
        let kp = (*core::ptr::addr_of!(KEYPAIR)).as_ref()?;
        Some(kp.static_pk)
    }
}

/// Idempotent. Allocates the sys-wg keypair on first call.
pub fn init() {
    ensure_init();
}

fn ensure_init() {
    unsafe {
        let slot = &mut *core::ptr::addr_of_mut!(KEYPAIR);
        if slot.is_none() {
            *slot = Some(WgKeypair::generate());
        }
    }
}

/// Run `f` "inside" the sys-wg cave. The caller's `cave_id` + TTBR0
/// are saved, swapped to sys-wg's, and restored before returning.
///
/// When the kernel runs with MMU off (boot-time serial-shell path),
/// the TTBR0 writes have no translation effect — the swap is purely
/// architectural / forward-compatible. When `setup_and_enable` runs
/// at kernel boot (open follow-up), this routine becomes the real
/// trampoline-into-cave: kernel code executing inside the closure
/// can only reach sys-wg-owned memory because sys-wg's L1 is the
/// active L1.
///
/// Hard requirement: sys-wg must have been brought up at boot (see
/// `sys_caves::init`). If not, we fall through to running `f` in the
/// caller's context and the security-claim is degraded to "module-
/// private state only." That's still correct — just not MMU-enforced.
fn with_sys_wg_cave<R>(f: impl FnOnce() -> R) -> R {
    let task_id = process::current_id();
    let saved_cave = process::get(task_id).cave_id;

    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => {
            // sys-wg never came up at boot. Run f in the caller's
            // context; the module-privacy boundary is still upheld.
            return f();
        }
    };

    let saved_ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) saved_ttbr0); }

    process::set_cave(task_id, sys_wg_id);
    if let Some(target_l1) = cave::get_cave_l1_phys(sys_wg_id) {
        crate::batcave::linux::mmu::switch_to_cave(target_l1);
    }

    let out = f();

    process::set_cave(task_id, saved_cave);
    if saved_ttbr0 != 0 {
        crate::batcave::linux::mmu::switch_to_cave(saved_ttbr0 as usize);
    }
    out
}

/// Diagnostic — read TTBR0_EL1 *from inside* the sys-wg cave context.
/// Used by the Arc-3 selftest to prove the with_sys_wg_cave trampoline
/// actually loads sys-wg's L1 around the closure body.
pub fn read_ttbr0_inside_sys_wg() -> u64 {
    with_sys_wg_cave(|| {
        let v: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) v); }
        v
    })
}

/// One-shot handshake-and-transport round trip with a hypothetical
/// peer whose keypair the caller passes in.
///
/// What the caller sees: a pair of `TransportKeys` derived from the
/// handshake. What the caller never sees: sys-wg's static secret —
/// the DH operations involving it run only inside the closure.
///
/// Slice-2 will move to a peer-id-keyed API (`begin_handshake(peer_pk)
/// -> session_id`, `wrap(session_id, ...)`) so multiple peers can
/// coexist. This slice's surface is what the selftest needs.
pub struct LocalRoundTrip {
    pub initiator_to_responder_keys: TransportKeys,
    pub responder_to_initiator_keys: TransportKeys,
    pub initiator_eph_pk: [u8; wireguard::KEY_LEN],
    pub responder_eph_pk: [u8; wireguard::KEY_LEN],
}

/// Drive a full WG handshake where sys-wg plays the responder role
/// and the caller-supplied `peer` plays the initiator. The peer's
/// `WgKeypair` is passed in (selftest-only; real callers do not have
/// access to sys-wg's secret). Returns transport keys for both sides
/// so the selftest can run a transport round trip; production callers
/// would only ever get the responder-side keys back.
pub fn debug_local_round_trip(peer: &WgKeypair)
    -> Result<LocalRoundTrip, WgError>
{
    ensure_init();
    let timestamp = [0u8; wireguard::TIMESTAMP_LEN];

    // Snapshot sys-wg's pubkey *outside* the closure (the caller would
    // get this via service_pubkey() in production).
    let sys_wg_pk = match service_pubkey() {
        Some(pk) => pk,
        None => return Err(WgError::KdfFail),
    };

    // The peer (initiator) builds InitMsg using sys-wg's pubkey. This
    // happens in the CALLER's context — peer keys belong to caller.
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        wireguard::initiator_send_init(peer, &sys_wg_pk, &timestamp)?;

    // Everything from here that touches sys-wg's keypair runs INSIDE
    // sys-wg. The closure returns just the bytes the caller is allowed
    // to see (response ciphertexts + transport keys for this peer).
    //
    // The trick that makes this an architectural boundary: the static
    // `KEYPAIR` is reachable only through this closure's body. A
    // future EL0 sys-wg task with MMU enforcement will literally fault
    // on any access to KEYPAIR from outside the cave; today the same
    // guarantee is upheld by module privacy + the with_sys_wg_cave
    // trampoline (no `pub` getter returns the SecretKey).
    let (enc_empty, resp_eph_pk, resp_tx_keys) =
        with_sys_wg_cave(|| -> Result<_, WgError> {
            let kp = unsafe { (*core::ptr::addr_of!(KEYPAIR)).as_ref().unwrap() };
            let (mut resp_state, ts_back) = wireguard::responder_consume_init(
                kp, &init_eph_pk, &enc_static, &enc_ts,
            )?;
            if ts_back != timestamp { return Err(WgError::BadLen); }
            let (enc_empty, resp_eph_pk, resp_keys) =
                wireguard::responder_send_response(&mut resp_state, &init_eph_pk)?;
            Ok((enc_empty, resp_eph_pk, resp_keys))
        })?;

    let init_tx_keys = wireguard::initiator_finish_handshake(
        peer, &mut init_state, &resp_eph_pk, &enc_empty,
    )?;

    Ok(LocalRoundTrip {
        initiator_to_responder_keys: init_tx_keys,
        responder_to_initiator_keys: resp_tx_keys,
        initiator_eph_pk: init_eph_pk,
        responder_eph_pk: resp_eph_pk,
    })
}

/// AEAD-wrap a plaintext for transport with the given keys, running
/// inside the sys-wg cave. Used by future per-peer `wrap` calls; for
/// the slice-1 selftest, callers pass in the keys the local round
/// trip handed back.
pub fn wrap_with_keys(keys: &mut TransportKeys, plaintext: &[u8])
    -> Result<Vec<u8>, WgError>
{
    with_sys_wg_cave(|| wireguard::transport_send(keys, plaintext))
}

/// AEAD-unwrap, mirror of `wrap_with_keys`.
pub fn unwrap_with_keys(keys: &mut TransportKeys, counter: u64, ct: &[u8])
    -> Result<Vec<u8>, WgError>
{
    with_sys_wg_cave(|| wireguard::transport_recv(keys, counter, ct))
}

// ─────────────────────────────────────────────────────────────────
// Slice 2: peer table + per-peer wrap/unwrap.
//
// Each peer slot pins the peer's static pubkey at `register_peer`
// time. When `complete_handshake_as_responder` finishes the Noise IK
// handshake driven from the peer's InitMsg, the resulting
// TransportKeys are stashed inside the slot. Subsequent
// `wrap(peer_id, pt)` / `unwrap(peer_id, counter, ct)` calls look up
// the slot, run the AEAD inside `with_sys_wg_cave`, and never expose
// the keys to the caller.
// ─────────────────────────────────────────────────────────────────

/// Maximum concurrent peers. Small fixed array — Bat_OS is single-
/// machine, single-operator; even a handful is plenty for slice 2.
/// Bumping this is one constant change + a longer scan in
/// `register_peer`.
pub const MAX_PEERS: usize = 8;

/// Opaque handle returned by `register_peer`. The numeric value is
/// the slot index; we wrap it so callers can't fabricate one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerId(u8);

impl PeerId {
    pub fn as_u8(self) -> u8 { self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysWgError {
    /// `register_peer` ran out of peer slots.
    NoSlot,
    /// `register_peer` was given a pubkey already pinned in another
    /// slot. Caller should reuse the existing PeerId rather than
    /// asking for a new one.
    DuplicatePeer,
    /// PeerId doesn't correspond to a live slot (slot is None, or
    /// out of range, or the slot was closed).
    UnknownPeer,
    /// Operation needs a completed handshake (transport keys
    /// installed) but the slot has none yet.
    NoSession,
    /// WireGuard primitive itself returned an error — AEAD failed,
    /// timestamp mismatched, etc.
    Wg(WgError),
}

impl From<WgError> for SysWgError {
    fn from(e: WgError) -> Self { Self::Wg(e) }
}

/// Per-peer state. `static_pk` is set at `register_peer`; `keys` is
/// set by `complete_handshake_as_responder` and cleared by
/// `close_peer`.
struct PeerState {
    static_pk: [u8; wireguard::KEY_LEN],
    /// Responder-side TransportKeys, populated after handshake.
    keys: Option<TransportKeys>,
}

static mut PEERS: [Option<PeerState>; MAX_PEERS] = [
    None, None, None, None, None, None, None, None,
];

/// Snapshot of the wire bytes the responder side has to send back
/// to the initiator after `complete_handshake_as_responder`.
/// Production callers pass these to whatever UDP-shaped transport
/// they're using (Phase-2 work) — slice 2 returns them to the
/// selftest so the caller can drive `initiator_finish_handshake`
/// locally and finalize its own side of the session.
pub struct ResponderWire {
    pub responder_eph_pk: [u8; wireguard::KEY_LEN],
    pub enc_empty: Vec<u8>,
    pub initiator_timestamp: [u8; wireguard::TIMESTAMP_LEN],
}

/// Pin a peer's static pubkey and allocate a session slot for it.
/// Idempotent on the pubkey: a second registration of the same key
/// returns `Err(DuplicatePeer)` so the caller doesn't accidentally
/// create two slots tracking the same peer.
///
/// The pubkey is the only piece of public state we store. Once the
/// handshake completes, the slot will also hold session
/// `TransportKeys` — those never leave the slot.
pub fn register_peer(peer_static_pk: [u8; wireguard::KEY_LEN])
    -> Result<PeerId, SysWgError>
{
    ensure_init();
    with_sys_wg_cave(|| -> Result<PeerId, SysWgError> {
        let peers = unsafe { &mut *core::ptr::addr_of_mut!(PEERS) };
        for slot in peers.iter() {
            if let Some(p) = slot.as_ref() {
                if p.static_pk == peer_static_pk {
                    return Err(SysWgError::DuplicatePeer);
                }
            }
        }
        for (i, slot) in peers.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(PeerState { static_pk: peer_static_pk, keys: None });
                return Ok(PeerId(i as u8));
            }
        }
        Err(SysWgError::NoSlot)
    })
}

/// Forget a peer. Clears its slot and any installed session keys.
/// Returns `UnknownPeer` if the slot was already empty.
pub fn close_peer(peer_id: PeerId) -> Result<(), SysWgError> {
    with_sys_wg_cave(|| -> Result<(), SysWgError> {
        let peers = unsafe { &mut *core::ptr::addr_of_mut!(PEERS) };
        let slot = peers.get_mut(peer_id.0 as usize).ok_or(SysWgError::UnknownPeer)?;
        if slot.is_none() { return Err(SysWgError::UnknownPeer); }
        *slot = None;
        Ok(())
    })
}

/// True if the peer slot is live and has a completed handshake
/// (TransportKeys installed). Exposed for the selftest to assert
/// session state without exposing the keys themselves.
pub fn peer_has_session(peer_id: PeerId) -> bool {
    let peers = unsafe { &*core::ptr::addr_of!(PEERS) };
    match peers.get(peer_id.0 as usize).and_then(|s| s.as_ref()) {
        Some(p) => p.keys.is_some(),
        None => false,
    }
}

/// Number of currently-registered peers (live slots, with or without
/// completed handshakes).
pub fn peer_count() -> usize {
    let peers = unsafe { &*core::ptr::addr_of!(PEERS) };
    peers.iter().filter(|s| s.is_some()).count()
}

/// Consume an InitMsg from `peer_id`, derive the same `(c, h)` chain
/// the initiator produced, build the ResponseMsg wire bytes, and
/// stash the resulting responder-side `TransportKeys` inside the
/// peer slot. After this call, `wrap(peer_id, ...)` and
/// `unwrap(peer_id, ...)` work.
///
/// The peer-side WgKeypair is NOT passed in — we only need its
/// pubkey, which the peer included as `initiator_static_pk` inside
/// the encrypted InitMsg payload. The handshake validates that the
/// payload's static_pk matches what we pinned at `register_peer`
/// time; mismatches fail with `WgError::BadLen` (matches the
/// existing wireguard.rs error vocabulary).
pub fn complete_handshake_as_responder(
    peer_id: PeerId,
    initiator_eph_pk: &[u8; wireguard::KEY_LEN],
    enc_static: &[u8],
    enc_timestamp: &[u8],
) -> Result<ResponderWire, SysWgError> {
    ensure_init();
    with_sys_wg_cave(|| -> Result<ResponderWire, SysWgError> {
        let peers = unsafe { &mut *core::ptr::addr_of_mut!(PEERS) };
        let slot = peers.get_mut(peer_id.0 as usize).ok_or(SysWgError::UnknownPeer)?;
        let peer = slot.as_mut().ok_or(SysWgError::UnknownPeer)?;

        let kp = unsafe { (*core::ptr::addr_of!(KEYPAIR)).as_ref().unwrap() };

        let (mut resp_state, ts_back) = wireguard::responder_consume_init(
            kp, initiator_eph_pk, enc_static, enc_timestamp,
        )?;
        if resp_state.initiator_static_pk != peer.static_pk {
            // Pinned key mismatch — refuse the handshake.
            return Err(SysWgError::Wg(WgError::BadLen));
        }

        let (enc_empty, responder_eph_pk, transport_keys) =
            wireguard::responder_send_response(&mut resp_state, initiator_eph_pk)?;

        peer.keys = Some(transport_keys);

        Ok(ResponderWire {
            responder_eph_pk,
            enc_empty,
            initiator_timestamp: ts_back,
        })
    })
}

/// AEAD-encrypt `plaintext` under the peer slot's responder-side
/// `send_key`, then bump the slot's `send_counter`. Returns the
/// ciphertext for the caller to forward over its transport.
pub fn wrap(peer_id: PeerId, plaintext: &[u8]) -> Result<Vec<u8>, SysWgError> {
    with_sys_wg_cave(|| -> Result<Vec<u8>, SysWgError> {
        let peers = unsafe { &mut *core::ptr::addr_of_mut!(PEERS) };
        let peer = peers.get_mut(peer_id.0 as usize)
            .and_then(|s| s.as_mut())
            .ok_or(SysWgError::UnknownPeer)?;
        let keys = peer.keys.as_mut().ok_or(SysWgError::NoSession)?;
        let ct = wireguard::transport_send(keys, plaintext)?;
        Ok(ct)
    })
}

/// AEAD-decrypt `ciphertext` under the peer slot's responder-side
/// `recv_key` with the given counter (set by the peer). Updates the
/// slot's strict-monotonic recv counter on success.
pub fn unwrap(peer_id: PeerId, counter: u64, ciphertext: &[u8])
    -> Result<Vec<u8>, SysWgError>
{
    with_sys_wg_cave(|| -> Result<Vec<u8>, SysWgError> {
        let peers = unsafe { &mut *core::ptr::addr_of_mut!(PEERS) };
        let peer = peers.get_mut(peer_id.0 as usize)
            .and_then(|s| s.as_mut())
            .ok_or(SysWgError::UnknownPeer)?;
        let keys = peer.keys.as_mut().ok_or(SysWgError::NoSession)?;
        let pt = wireguard::transport_recv(keys, counter, ciphertext)?;
        Ok(pt)
    })
}
