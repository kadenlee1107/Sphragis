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
use x25519_dalek::StaticSecret;

/// Maximum concurrent peers. Small fixed array — Sphragis is single-
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

    /// Initiator-side handshake-in-progress storage (per peer).
    /// `peer_init_active[i] = 1` means we've called start_handshake
    /// for peer `i` and are waiting for the responder's Response.
    /// The other fields below are valid only when `peer_init_active[i] == 1`.
    peer_init_active: [u32; MAX_PEERS],
    /// Our chosen sender_index for the in-progress handshake.
    /// Becomes the `receiver_index` field in the Response we
    /// expect back. Also used as a "session id" the wg_dispatch
    /// session table keys on.
    peer_init_our_sender_index: [u32; MAX_PEERS],
    /// Per-peer UDP endpoint (the IPv4 address + port the peer's
    /// WG socket listens on). 0/0 means "not set" — outbound
    /// connect-out is refused. Stored in cave-private so a
    /// compromised non-sys-wg caller can't learn or rewrite the
    /// endpoint without going through the IPC mailbox.
    peer_endpoint_ip:   [u32; MAX_PEERS],
    peer_endpoint_port: [u16; MAX_PEERS],
    _peer_endpoint_pad: [u16; MAX_PEERS],  // 16-byte align next field
    /// Per-peer initiator state, mirroring `wireguard::InitiatorState`
    /// but split into the byte-arrays that are #[repr(C)]-friendly.
    /// `eph_sk_seed` is the X25519 ephemeral private-key seed; we
    /// reconstruct `StaticSecret::from(seed)` on demand to drive
    /// `initiator_finish_handshake` (mirrors the same trick the
    /// static keypair uses).
    peer_init_eph_sk_seed: [[u8; 32]; MAX_PEERS],
    peer_init_eph_pk: [[u8; 32]; MAX_PEERS],
    /// Running chaining-key + handshake-hash from the Noise IK
    /// half-handshake we drove with `initiator_send_init`.
    peer_init_c: [[u8; 32]; MAX_PEERS],
    peer_init_h: [[u8; 32]; MAX_PEERS],
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

impl From<u8> for PeerId {
    fn from(v: u8) -> Self { PeerId(v) }
}

impl PeerId {
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
                s.peer_init_active[i] = 0;
                s.peer_init_our_sender_index[i] = 0;
                s.peer_init_eph_sk_seed[i] = [0u8; 32];
                s.peer_init_eph_pk[i] = [0u8; 32];
                s.peer_init_c[i] = [0u8; 32];
                s.peer_init_h[i] = [0u8; 32];
                s.peer_endpoint_ip[i] = 0;
                s.peer_endpoint_port[i] = 0;
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
        s.peer_init_active[i] = 0;
        s.peer_init_our_sender_index[i] = 0;
        s.peer_init_eph_sk_seed[i] = [0u8; 32];
        s.peer_init_eph_pk[i] = [0u8; 32];
        s.peer_init_c[i] = [0u8; 32];
        s.peer_init_h[i] = [0u8; 32];
        s.peer_endpoint_ip[i] = 0;
        s.peer_endpoint_port[i] = 0;
        Ok(())
    })
}

/// Set the UDP endpoint (`ip:port`) for a registered peer. Used
/// before `start_handshake_as_initiator` so the eventual
/// outbound transmission knows where to go. Both arguments are
/// in host byte order; `udp::send` handles the to_be wire
/// conversion. Pass `(0, 0)` to clear.
pub fn set_peer_endpoint(peer_id: PeerId, ip: u32, port: u16) -> Result<(), SysWgError> {
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<(), SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        s.peer_endpoint_ip[i] = ip;
        s.peer_endpoint_port[i] = port;
        Ok(())
    })
}

/// Read the UDP endpoint for a registered peer. Returns
/// `Some((ip, port))` if set, `None` if the peer is unknown or
/// no endpoint has been configured.
pub fn get_peer_endpoint(peer_id: PeerId) -> Option<(u32, u16)> {
    let sys_wg_id = sys_caves::sys_wg_id()? as u16;
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = state_ref()?;
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 { return None; }
        let ip = s.peer_endpoint_ip[i];
        let port = s.peer_endpoint_port[i];
        if ip == 0 && port == 0 { return None; }
        Some((ip, port))
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

/// True if the given peer slot is live (registered but possibly
/// without a session yet). Used by the WG dispatcher to walk all
/// registered peers when an InitMsg arrives.
pub fn peer_slot_in_use(peer_id: PeerId) -> bool {
    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => return false,
    };
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = match state_ref() { Some(s) => s, None => return false };
        let i = peer_id.0 as usize;
        i < MAX_PEERS && s.peer_in_use[i] != 0
    })
}

/// Return the pinned static pubkey for a peer slot, or None if the
/// slot is empty. Public-key bytes are not sensitive — sys-wg
/// exposes them so the dispatch layer can build outgoing
/// ResponseMsg wire bytes (mac1 keyed on the initiator's pubkey).
pub fn peer_static_pk(peer_id: PeerId) -> Option<[u8; wireguard::KEY_LEN]> {
    let sys_wg_id = sys_caves::sys_wg_id()? as u16;
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = state_ref()?;
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 { return None; }
        Some(s.peer_static_pk[i])
    })
}

/// Look up a peer slot by pinned static pubkey. Returns the
/// `PeerId` if a slot is using that pubkey, else None.
pub fn find_peer_by_pk(static_pk: &[u8; wireguard::KEY_LEN]) -> Option<PeerId> {
    let sys_wg_id = sys_caves::sys_wg_id()? as u16;
    cave::with_cave_active(sys_wg_id, || unsafe {
        let s = state_ref()?;
        for i in 0..MAX_PEERS {
            if s.peer_in_use[i] != 0 && s.peer_static_pk[i] == *static_pk {
                return Some(PeerId(i as u8));
            }
        }
        None
    })
}

/// Convenience: tear down whatever peer slot pins `static_pk`.
/// No-op if no slot matches. Useful for selftests that want a
/// clean slate without remembering the PeerId.
pub fn close_peer_by_static_pk(static_pk: &[u8; wireguard::KEY_LEN]) -> Result<(), SysWgError> {
    match find_peer_by_pk(static_pk) {
        Some(id) => close_peer(id),
        None => Ok(()),
    }
}

/// End-to-end initiator-role selftest. Caller plays the
/// RESPONDER side (using its own keypair), sys-wg plays the
/// INITIATOR side. Validates:
///   - sys-wg's start_handshake produces wire-valid InitMsg bytes.
///   - mac1 against the responder's static_pk verifies.
///   - Responder side can drive responder_consume_init +
///     responder_send_response with sys-wg's bytes.
///   - sys-wg's finish_handshake correctly consumes the
///     response and installs session keys.
///   - A wrap/unwrap round trip through sys-wg uses those keys.
///
/// Returns `(handshake_ok, transport_ok)`.
pub fn selftest_initiator() -> Option<(bool, bool)> {
    // We need a peer with sys-wg's pubkey = our actual pubkey
    // (sys-wg's static pubkey, which we PIN against an initiator
    // we generated). For the initiator path, peer.static_pk is
    // the RESPONDER's pubkey from sys-wg's POV, so we register a
    // peer keyed on a RESPONDER keypair the test owns.
    let responder = WgKeypair::generate();

    // Clean any prior registration of this pubkey, then register.
    let _ = close_peer_by_static_pk(&responder.static_pk);
    let peer_id = match register_peer(responder.static_pk) {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Pick our_sender_index (in the real flow, wg_dispatch picks
    // it; for this direct-API selftest we pick locally).
    let our_idx: u32 = 0xCAFEC0DE;

    let wire = start_handshake_as_initiator(peer_id, our_idx).ok()?;

    // Responder side: parse + verify mac1 against the responder
    // pubkey it owns.
    let parsed = wireguard::parse_init_msg(&wire.init_wire, &responder.static_pk).ok()?;
    if parsed.sender_index != our_idx { return None; }

    // Drive responder_consume_init + responder_send_response.
    let (mut resp_state, _ts) = wireguard::responder_consume_init(
        &responder, &parsed.eph_pk, &parsed.enc_static, &parsed.enc_timestamp,
    ).ok()?;
    let (enc_empty, responder_eph_pk, mut responder_tx_keys) =
        wireguard::responder_send_response(&mut resp_state, &parsed.eph_pk).ok()?;

    // sys-wg's finish_handshake consumes the response bytes.
    finish_handshake_as_initiator(peer_id, &responder_eph_pk, &enc_empty).ok()?;
    let handshake_ok = peer_has_session(peer_id);

    // Wrap/unwrap round trip using sys-wg's installed keys.
    let pt = b"initiator wins";
    let ct = wrap(peer_id, pt).ok()?;
    let recovered = wireguard::transport_recv(&mut responder_tx_keys, 0, &ct).ok()?;
    let transport_ok = recovered.as_slice() == pt;

    let _ = close_peer(peer_id);
    Some((handshake_ok, transport_ok))
}

/// Output of `start_handshake_as_initiator`: the InitMsg wire
/// bytes (148 B per WG whitepaper §5.4.2) the caller transmits
/// to the peer, plus the `our_sender_index` field we picked. The
/// caller registers `(our_sender_index, peer_id)` in the
/// `wg_dispatch` session table so an incoming Response can be
/// routed back to this peer.
pub struct InitiatorWire {
    pub init_wire: [u8; wireguard::INIT_MSG_LEN],
    pub our_sender_index: u32,
}

/// Build a new InitMsg targeting `peer_id`. Generates a fresh
/// ephemeral keypair (seed stored in cave-private), runs Noise
/// IK half-handshake, stashes `(eph_sk_seed, c, h)` in the
/// cave-private peer slot, and emits 148 bytes of wire-format
/// InitMsg the caller can hand to `udp::send` (or any other
/// transport).
///
/// Pre-conditions:
///   - `peer_id` must be a registered peer (its `static_pk`
///     is the responder we're targeting).
///   - No in-progress initiator handshake for this peer (we
///     reject if `peer_init_active != 0` — call `close_peer` to
///     reset first if you really want to abandon a pending one).
///
/// `our_sender_index` is provided by the caller (since
/// `wg_dispatch` owns the sender-index allocator); we just
/// record it for later validation in `finish_handshake_as_initiator`.
pub fn start_handshake_as_initiator(
    peer_id: PeerId,
    our_sender_index: u32,
) -> Result<InitiatorWire, SysWgError> {
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<InitiatorWire, SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        if s.initialized == 0 { return Err(SysWgError::UnknownPeer); }
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }
        if s.peer_init_active[i] != 0 {
            // A previous start_handshake is still waiting for
            // its Response. Caller should call close_peer +
            // register_peer to fully reset, or finish the
            // pending one. We refuse to silently overwrite.
            return Err(SysWgError::Wg(WgError::KdfFail));
        }

        let kp = wireguard::WgKeypair::from_seed(s.static_sk_seed);
        let responder_static_pk = s.peer_static_pk[i];

        // Generate eph seed.
        let mut eph_seed = [0u8; wireguard::KEY_LEN];
        crate::crypto::rng::fill_bytes(&mut eph_seed);

        let timestamp = [0u8; wireguard::TIMESTAMP_LEN];
        let (state, eph_pk, enc_static, enc_ts) =
            wireguard::initiator_send_init_with_seed(
                &kp, &responder_static_pk, &timestamp, eph_seed,
            )?;

        // Encode wire bytes (148 B).
        let init_wire = wireguard::encode_init_msg(
            our_sender_index, &eph_pk, &enc_static, &enc_ts,
            &responder_static_pk,
        )?;

        // Stash the chaining state in cave-private. We need
        // `eph_sk_seed`, `c`, `h` to drive
        // `initiator_finish_handshake` on Response arrival.
        s.peer_init_active[i] = 1;
        s.peer_init_our_sender_index[i] = our_sender_index;
        s.peer_init_eph_sk_seed[i] = eph_seed;
        s.peer_init_eph_pk[i] = eph_pk;
        s.peer_init_c[i] = state.c;
        s.peer_init_h[i] = state.h;

        Ok(InitiatorWire { init_wire, our_sender_index })
    })
}

/// Consume a Response on the initiator side. Reconstructs the
/// `InitiatorState` saved at `start_handshake_as_initiator` time,
/// runs `initiator_finish_handshake`, installs the resulting
/// `TransportKeys` in the peer slot, clears the in-progress
/// flag.
///
/// `their_sender_index` is the responder's chosen sender_index
/// (the `sender_index` field of the Response message). We
/// record it implicitly via the wg_dispatch session table the
/// caller is expected to update — sys-wg doesn't need it for
/// the AEAD math, only for routing future outbound packets,
/// which the caller does.
pub fn finish_handshake_as_initiator(
    peer_id: PeerId,
    responder_eph_pk: &[u8; wireguard::KEY_LEN],
    enc_empty: &[u8],
) -> Result<(), SysWgError> {
    if enc_empty.len() != wireguard::TAG_LEN { return Err(SysWgError::Wg(WgError::BadLen)); }
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<(), SysWgError> {
        let s = unsafe { state_mut().ok_or(SysWgError::UnknownPeer)? };
        let i = peer_id.0 as usize;
        if i >= MAX_PEERS || s.peer_in_use[i] == 0 || s.peer_init_active[i] == 0 {
            return Err(SysWgError::UnknownPeer);
        }

        // Rebuild keypair + initiator state from cave-private.
        let kp = wireguard::WgKeypair::from_seed(s.static_sk_seed);
        let eph_sk = StaticSecret::from(s.peer_init_eph_sk_seed[i]);

        let mut state = wireguard::InitiatorState {
            eph_sk,
            eph_pk: s.peer_init_eph_pk[i],
            responder_static_pk: s.peer_static_pk[i],
            c: s.peer_init_c[i],
            h: s.peer_init_h[i],
        };
        let tx_keys = wireguard::initiator_finish_handshake(
            &kp, &mut state, responder_eph_pk, enc_empty,
        )?;

        // Install transport keys + clear init-in-progress state.
        s.peer_send_key[i] = tx_keys.send_key;
        s.peer_recv_key[i] = tx_keys.recv_key;
        s.peer_send_counter[i] = tx_keys.send_counter;
        s.peer_recv_counter[i] = tx_keys.recv_counter;
        s.peer_recv_window_bits[i] = tx_keys.recv_window_bits;
        s.peer_has_session[i] = 1;

        s.peer_init_active[i] = 0;
        s.peer_init_eph_sk_seed[i] = [0u8; 32];  // wipe seed
        s.peer_init_c[i] = [0u8; 32];
        s.peer_init_h[i] = [0u8; 32];

        Ok(())
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
/// Returns just the ciphertext (with tag). Use `wrap_full` if
/// you need the counter that was used for wire framing.
pub fn wrap(peer_id: PeerId, plaintext: &[u8]) -> Result<Vec<u8>, SysWgError> {
    wrap_full(peer_id, plaintext).map(|(ct, _)| ct)
}

/// AEAD-encrypt with counter exposed. Returns `(ciphertext, counter)`
/// where `counter` is the value sys-wg's send-counter held at the
/// instant of the AEAD seal (i.e., the value the caller must put
/// in the wire transport message's counter field). sys-wg's
/// internal counter is bumped to `counter + 1` afterwards.
pub fn wrap_full(peer_id: PeerId, plaintext: &[u8])
    -> Result<(Vec<u8>, u64), SysWgError>
{
    let sys_wg_id = sys_caves::sys_wg_id().ok_or(SysWgError::UnknownPeer)? as u16;
    cave::with_cave_active(sys_wg_id, || -> Result<(Vec<u8>, u64), SysWgError> {
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
        let counter_used = keys.send_counter;
        let ct = wireguard::transport_send(&mut keys, plaintext)?;
        s.peer_send_counter[i] = keys.send_counter;
        Ok((ct, counter_used))
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
