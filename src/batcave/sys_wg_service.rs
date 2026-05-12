//! sys-wg service — Arc 3 slices 1, 2, 3.
//!
//! Encapsulates WireGuard state (static keypair, per-peer transport
//! keys) inside the sys-wg cave's MMU boundary. As of slice 3, the
//! state lives in a cave-private 4 KiB page (`cave_private::ensure_page`)
//! mapped only in sys-wg's L1 — code paths that touch the keypair or
//! peer table go through `cave::with_cave_active(sys_wg_id, ...)`
//! and any code path that doesn't would walker-fault on the dereference.
//!
//! ### Privacy boundary today
//!
//! Three layers of defence, in order of strength:
//!
//!   1. **Module privacy.** No `pub` getter returns the secret bytes.
//!      Compile-time enforcement.
//!   2. **VA-level MMU enforcement.** The cave-private VA where the
//!      state lives is unmapped in `PRIMARY_L1` and in every other
//!      cave's L1 (proved by the `cave-private-selftest`'s
//!      `pte_lookup` walks). Any access from outside
//!      `with_cave_active(sys_wg_id, ...)` faults at the MMU walker.
//!   3. *(future)* **PA-level isolation.** Today the cave-private
//!      page is allocated from the kernel-pool, which is still
//!      identity-mapped via `L1[1..=4]` in PRIMARY_L1 — so an
//!      attacker who already knows the PA can reach the bytes via
//!      kernel identity. Closing this requires a frame-allocator
//!      carve-out that reserves a separate PA range, mapped only
//!      via per-cave L1s. Tracked as a future arc.
//!
//! ### State layout
//!
//! The cave-private page hosts a single `PrivateState` struct (see
//! below), placed at offset 0. We store the X25519 seed bytes rather
//! than a constructed `StaticSecret` so:
//!   - the layout is `#[repr(C)]` + plain bytes, no Drop concerns;
//!   - reinitialising on boot is a memset/write, not a destructor run;
//!   - x25519-dalek's `StaticSecret` is rebuilt on demand via
//!     `WgKeypair::from_seed`.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::batcave::{cave, cave_private, sys_caves};
use crate::net::wireguard::{self, TransportKeys, WgKeypair, WgError};

/// Maximum concurrent peers. Small fixed array — Bat_OS is single-
/// machine, single-operator; even a handful is plenty.
pub const MAX_PEERS: usize = 8;

/// Layout of the sys-wg cave-private page. Plain bytes only:
/// `#[repr(C)]` with explicit-width fields so the layout is
/// deterministic and `core::ptr::write` semantics are obvious.
/// The x25519 secret is stored as 32 seed bytes; `WgKeypair::from_seed`
/// reconstructs the `StaticSecret`/`X25519Public` pair on demand.
#[repr(C)]
struct PrivateState {
    /// 0 = page contents are still zero-initialised (frame allocator
    /// hands out zeroed pages). 1 = `init()` ran and the keypair
    /// fields are valid.
    initialized: u32,
    _pad0: [u8; 4],
    static_sk_seed: [u8; 32],
    static_pk: [u8; 32],
    /// Per-peer arrays kept as parallel slot-indexed arrays (SoA)
    /// rather than `[PeerSlot; MAX_PEERS]` to keep the layout
    /// trivially `#[repr(C)]`-friendly without nested struct
    /// padding surprises.
    peer_in_use: [u32; MAX_PEERS],
    peer_has_session: [u32; MAX_PEERS],
    peer_static_pk: [[u8; 32]; MAX_PEERS],
    peer_send_key: [[u8; 32]; MAX_PEERS],
    peer_recv_key: [[u8; 32]; MAX_PEERS],
    peer_send_counter: [u64; MAX_PEERS],
    peer_recv_counter: [u64; MAX_PEERS],
    /// Replay-window bitmap per peer (whitepaper §5.4.6). Bit `i`
    /// indicates "counter (peer_recv_counter - i) has been
    /// accepted." Paired with `peer_recv_counter` (which acts as
    /// the window top). Width fixed at `REPLAY_WINDOW_WIDTH = 64`.
    peer_recv_window_bits: [u64; MAX_PEERS],
}

/// VA where `PrivateState` lives, set on successful `init()`. 0
/// before init; reading 0 means "no protected state — degraded mode."
static STATE_VA: AtomicUsize = AtomicUsize::new(0);

/// Read the raw pointer (does NOT dereference). Returns null when
/// `init()` hasn't published a VA yet.
fn state_ptr() -> *mut PrivateState {
    STATE_VA.load(Ordering::Acquire) as *mut PrivateState
}

/// MUST be called from inside `with_cave_active(sys_wg_id, ...)`,
/// otherwise the dereference faults at the MMU walker. The compiler
/// can't enforce this — it's a runtime contract enforced by the
/// hardware.
unsafe fn state_mut() -> Option<&'static mut PrivateState> {
    let p = state_ptr();
    if p.is_null() { None } else { Some(unsafe { &mut *p }) }
}

unsafe fn state_ref() -> Option<&'static PrivateState> {
    let p = state_ptr();
    if p.is_null() { None } else { Some(unsafe { &*p }) }
}

/// `service_pubkey()` is what callers pin against. Returned by
/// value (a 32-byte X25519 public key) so the caller never holds
/// a borrow into our state.
pub fn service_pubkey() -> Option<[u8; wireguard::KEY_LEN]> {
    let sys_wg_id = sys_caves::sys_wg_id()? as u16;
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = state_ref()?;
        if s.initialized == 0 { None } else { Some(s.static_pk) }
    })
}

/// Idempotent. Allocates the sys-wg cave-private page and generates
/// the static keypair into it on first call.
pub fn init() {
    if STATE_VA.load(Ordering::Acquire) != 0 {
        return; // already initialised
    }

    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => return, // sys-wg never came up; degraded mode
    };

    let va = match cave_private::ensure_page(sys_wg_id) {
        Some(v) => v,
        None => return,
    };

    // Generate the keypair material *outside* the cave so the seed
    // never lands in kernel-ns memory beyond the brief stack lifetime.
    let mut seed = [0u8; 32];
    crate::crypto::rng::fill_bytes(&mut seed);
    let kp = WgKeypair::from_seed(seed);

    // Place the state inside the cave-private page. Must run with
    // the cave's L1 active so the VA translates.
    cave::with_cave_active(sys_wg_id, || unsafe {
        let p = va as *mut PrivateState;
        // The frame allocator zeroed this page (via the in-cave
        // path in cave_private::ensure_page), so we only need to
        // populate the fields we care about. Use field-by-field
        // writes (rather than ptr::write of the whole struct) so
        // the secret seed lives only in the cave-private page; a
        // whole-struct write would temporarily materialise the
        // value on the kernel-ns stack first.
        (*p).static_sk_seed = seed;
        (*p).static_pk = kp.static_pk;
        // Order matters: stores above must commit before we set
        // `initialized = 1`, because state_ref readers gate on it.
        core::arch::asm!("dsb sy");
        (*p).initialized = 1;
        core::arch::asm!("dsb sy");
    });

    // Wipe the local seed copy on the kernel-ns stack ASAP. Not
    // bulletproof (compiler may have spilled to other regs/slots
    // already) but cheap and reduces the window.
    for b in seed.iter_mut() { *b = 0; }

    STATE_VA.store(va, Ordering::Release);
}

/// Diagnostic — read TTBR0_EL1 *from inside* the sys-wg cave context.
/// Used by the Arc-3 selftest to prove the trampoline actually
/// loads sys-wg's L1 around the closure body.
pub fn read_ttbr0_inside_sys_wg() -> u64 {
    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => return 0,
    };
    cave::with_cave_active(sys_wg_id, || {
        let v: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) v); }
        v
    })
}

/// Internal helper — load the keypair (rebuilt from the cave-private
/// seed bytes) for use inside a closure. Must be called inside
/// `with_cave_active(sys_wg_id, ...)`.
unsafe fn with_keypair<R>(f: impl FnOnce(&WgKeypair) -> R) -> Option<R> {
    let s = unsafe { state_ref()? };
    if s.initialized == 0 { return None; }
    let kp = WgKeypair::from_seed(s.static_sk_seed);
    Some(f(&kp))
}

/// Snapshot for selftest use only — drives a full WG handshake
/// where the caller plays initiator and sys-wg plays responder.
pub struct LocalRoundTrip {
    pub initiator_to_responder_keys: TransportKeys,
    pub responder_to_initiator_keys: TransportKeys,
    pub initiator_eph_pk: [u8; wireguard::KEY_LEN],
    pub responder_eph_pk: [u8; wireguard::KEY_LEN],
}

/// Full one-shot round trip. The caller-side WgKeypair is owned by
/// the caller; sys-wg's keypair stays inside the cave-private page.
pub fn debug_local_round_trip(peer: &WgKeypair)
    -> Result<LocalRoundTrip, WgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(WgError::KdfFail)? as u16;
    let timestamp = [0u8; wireguard::TIMESTAMP_LEN];
    let sys_wg_pk = service_pubkey().ok_or(WgError::KdfFail)?;

    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        wireguard::initiator_send_init(peer, &sys_wg_pk, &timestamp)?;

    let (enc_empty, resp_eph_pk, resp_tx_keys) =
        cave::with_cave_active(sys_wg_id, || -> Result<_, WgError> {
            let result = unsafe {
                with_keypair(|kp| -> Result<_, WgError> {
                    let (mut resp_state, ts_back) = wireguard::responder_consume_init(
                        kp, &init_eph_pk, &enc_static, &enc_ts,
                    )?;
                    if ts_back != timestamp { return Err(WgError::BadLen); }
                    wireguard::responder_send_response(&mut resp_state, &init_eph_pk)
                })
            };
            result.ok_or(WgError::KdfFail)?
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

/// AEAD-wrap using a caller-supplied `TransportKeys`. Runs inside
/// the sys-wg cave even though no cave-private state is touched —
/// matches the contract that all sys-wg crypto runs in-cave.
pub fn wrap_with_keys(keys: &mut TransportKeys, plaintext: &[u8])
    -> Result<Vec<u8>, WgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(WgError::KdfFail)? as u16;
    cave::with_cave_active(sys_wg_id, || wireguard::transport_send(keys, plaintext))
}

/// AEAD-unwrap, mirror of `wrap_with_keys`.
pub fn unwrap_with_keys(keys: &mut TransportKeys, counter: u64, ct: &[u8])
    -> Result<Vec<u8>, WgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(WgError::KdfFail)? as u16;
    cave::with_cave_active(sys_wg_id, || wireguard::transport_recv(keys, counter, ct))
}

// ─────────────────────────────────────────────────────────────────
// Peer-table-keyed API. All state lives in the cave-private page.
// ─────────────────────────────────────────────────────────────────

/// Opaque handle returned by `register_peer`. Wraps the slot index
/// so callers can't fabricate one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerId(u8);

impl PeerId {
    pub fn as_u8(self) -> u8 { self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysWgError {
    NoSlot,
    DuplicatePeer,
    UnknownPeer,
    NoSession,
    Wg(WgError),
}

impl From<WgError> for SysWgError {
    fn from(e: WgError) -> Self { Self::Wg(e) }
}

pub struct ResponderWire {
    pub responder_eph_pk: [u8; wireguard::KEY_LEN],
    pub enc_empty: Vec<u8>,
    pub initiator_timestamp: [u8; wireguard::TIMESTAMP_LEN],
}

/// Pin a peer's static pubkey and allocate a session slot for it.
pub fn register_peer(peer_static_pk: [u8; wireguard::KEY_LEN])
    -> Result<PeerId, SysWgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::NoSlot)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<PeerId, SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::NoSlot)? };
        if s.initialized == 0 { return Err(SysWgError::NoSlot); }

        // Duplicate check.
        for i in 0..MAX_PEERS {
            if s.peer_in_use[i] != 0 && s.peer_static_pk[i] == peer_static_pk {
                return Err(SysWgError::DuplicatePeer);
            }
        }
        // First-free scan.
        for i in 0..MAX_PEERS {
            if s.peer_in_use[i] == 0 {
                s.peer_static_pk[i] = peer_static_pk;
                s.peer_in_use[i] = 1;
                s.peer_has_session[i] = 0;
                s.peer_send_counter[i] = 0;
                s.peer_recv_counter[i] = 0;
                s.peer_recv_window_bits[i] = 0;
                // Wipe any stale key bytes from a previous occupant.
                s.peer_send_key[i] = [0u8; 32];
                s.peer_recv_key[i] = [0u8; 32];
                return Ok(PeerId(i as u8));
            }
        }
        Err(SysWgError::NoSlot)
    })
}

/// Forget a peer. Wipes both pubkey and session-key bytes.
pub fn close_peer(peer_id: PeerId) -> Result<(), SysWgError> {
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<(), SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        s.peer_in_use[i] = 0;
        s.peer_has_session[i] = 0;
        s.peer_static_pk[i] = [0u8; 32];
        s.peer_send_key[i] = [0u8; 32];
        s.peer_recv_key[i] = [0u8; 32];
        s.peer_send_counter[i] = 0;
        s.peer_recv_counter[i] = 0;
        s.peer_recv_window_bits[i] = 0;
        Ok(())
    })
}

pub fn peer_has_session(peer_id: PeerId) -> bool {
    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => return false,
    };
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = match state_ref() { Some(s) => s, None => return false };
        let i = peer_id.0 as usize;
        i < MAX_PEERS && s.peer_in_use[i] != 0 && s.peer_has_session[i] != 0
    })
}

pub fn peer_count() -> usize {
    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => return 0,
    };
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = match state_ref() { Some(s) => s, None => return 0 };
        let mut n = 0;
        for i in 0..MAX_PEERS {
            if s.peer_in_use[i] != 0 { n += 1; }
        }
        n
    })
}

/// Consume an InitMsg, derive `(c, h)`, install responder
/// TransportKeys in the peer slot, return the ResponseMsg wire bytes.
pub fn complete_handshake_as_responder(
    peer_id: PeerId,
    initiator_eph_pk: &[u8; wireguard::KEY_LEN],
    enc_static: &[u8],
    enc_timestamp: &[u8],
) -> Result<ResponderWire, SysWgError> {
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<ResponderWire, SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        if s.initialized == 0 { return Err(SysWgError::UnknownPeer); }

        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        let pinned_pk = s.peer_static_pk[i];

        // Rebuild keypair from the seed (lives only on this stack
        // frame, inside the cave).
        let kp = WgKeypair::from_seed(s.static_sk_seed);

        let (mut resp_state, ts_back) = wireguard::responder_consume_init(
            &kp, initiator_eph_pk, enc_static, enc_timestamp,
        )?;
        if resp_state.initiator_static_pk != pinned_pk {
            return Err(SysWgError::Wg(WgError::BadLen));
        }

        let (enc_empty, responder_eph_pk, transport_keys) =
            wireguard::responder_send_response(&mut resp_state, initiator_eph_pk)?;

        s.peer_send_key[i] = transport_keys.send_key;
        s.peer_recv_key[i] = transport_keys.recv_key;
        s.peer_send_counter[i] = transport_keys.send_counter;
        s.peer_recv_counter[i] = transport_keys.recv_counter;
        s.peer_recv_window_bits[i] = transport_keys.recv_window_bits;
        s.peer_has_session[i] = 1;

        Ok(ResponderWire {
            responder_eph_pk,
            enc_empty,
            initiator_timestamp: ts_back,
        })
    })
}

/// AEAD-encrypt under the peer slot's responder send_key.
pub fn wrap(peer_id: PeerId, plaintext: &[u8]) -> Result<Vec<u8>, SysWgError> {
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<Vec<u8>, SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        if s.peer_has_session[i] == 0 {
            return Err(SysWgError::NoSession);
        }
        // Reconstruct a TransportKeys on the cave stack, run the
        // AEAD, write the bumped counter back. The keys never leave
        // the cave-private page; we hold them on the stack only for
        // the duration of the call.
        let mut keys = TransportKeys {
            send_key: s.peer_send_key[i],
            recv_key: s.peer_recv_key[i],
            send_counter: s.peer_send_counter[i],
            recv_counter: s.peer_recv_counter[i],
            recv_window_bits: s.peer_recv_window_bits[i],
        };
        let ct = wireguard::transport_send(&mut keys, plaintext)?;
        s.peer_send_counter[i] = keys.send_counter;
        Ok(ct)
    })
}

/// AEAD-decrypt under the peer slot's responder recv_key.
pub fn unwrap(peer_id: PeerId, counter: u64, ciphertext: &[u8])
    -> Result<Vec<u8>, SysWgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<Vec<u8>, SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        if s.peer_has_session[i] == 0 {
            return Err(SysWgError::NoSession);
        }
        let mut keys = TransportKeys {
            send_key: s.peer_send_key[i],
            recv_key: s.peer_recv_key[i],
            send_counter: s.peer_send_counter[i],
            recv_counter: s.peer_recv_counter[i],
            recv_window_bits: s.peer_recv_window_bits[i],
        };
        let pt = wireguard::transport_recv(&mut keys, counter, ciphertext)?;
        s.peer_recv_counter[i] = keys.recv_counter;
        s.peer_recv_window_bits[i] = keys.recv_window_bits;
        Ok(pt)
    })
}
