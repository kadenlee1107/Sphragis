//! sys-wg IPC mailbox — Arc 3 slice 3.
//!
//! Until this slice, every call into `sys_wg_service` went
//! through the synchronous `with_cave_active(sys_wg_id, ...)`
//! trampoline: the caller's task briefly assumed sys-wg's
//! cave_id + TTBR0 to do the work. Architecturally clean enough
//! for the cave-private boundary (MMU-enforced — see
//! DESIGN_CAVE_ISOLATION.md), but the work still ran on the
//! caller's task.
//!
//! Slice 3 moves sys-wg work onto a *dedicated kernel task* tagged
//! with `cave_id = sys_wg_id` for its whole lifetime. Clients
//! never assume sys-wg's identity; they post requests into a
//! mailbox, the service task picks them up, processes them in
//! sys-wg's cave context, and writes responses back. The
//! security shape is now Qubes-like: even a compromised caller
//! can only emit IPC bytes; the cave-private state is reachable
//! only from inside the service task.
//!
//! Scope (this slice):
//!   - One opcode: `OP_PUBKEY`. Establishes the pattern; future
//!     slices add OP_HANDSHAKE / OP_WRAP / OP_UNWRAP / etc.
//!   - Request-scoped service task: each `request_pubkey()` call
//!     spawns a fresh kernel task that runs one cycle of
//!     "read request -> dispatch -> write response -> terminate."
//!     A long-running service-task with proper block/wake is a
//!     future arc gated on richer scheduler primitives.
//!   - Single-threaded contract — one outstanding request at a
//!     time. The global mailbox protects against accidental
//!     interleaving via `IPC_BUSY`.
//!
//! Mailbox memory: lives in regular kernel `.bss` (not
//! cave-private). The bytes flowing through it — opcodes, public
//! keys, ciphertexts — are NOT sensitive. The cave-private state
//! the service task touches (static seed, peer transport keys)
//! stays inside sys-wg's cave-private page as before.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use crate::batcave::{sys_caves, sys_wg_service};
use crate::kernel::{process, scheduler};
use crate::net::wireguard;

/// Maximum request payload bytes (request_data buffer). Picked
/// to fit the largest expected opcode argument: an OP_UNWRAP
/// with a full-size WG transport ciphertext (~1500 IP MTU + 16
/// AEAD tag) plus the 16-byte opcode header. 2 KiB headroom.
pub const REQ_DATA_MAX: usize = 2048;

/// Maximum response payload bytes. Picked to fit a Response
/// wire message (92 B) or a typical transport plaintext
/// (~1500 B IP payload). 1600 covers both.
pub const RSP_DATA_MAX: usize = 1600;

// ── Opcodes ─────────────────────────────────────────────────────
pub const OP_NONE:   u32 = 0;
pub const OP_PUBKEY: u32 = 1;
// Reserved for future slices (declared so opcode space is stable):
pub const OP_HANDSHAKE: u32 = 2;
pub const OP_WRAP:      u32 = 3;
pub const OP_UNWRAP:    u32 = 4;

// ── Response status ─────────────────────────────────────────────
pub const STATUS_PENDING: i32 =  0;
pub const STATUS_OK:      i32 =  1;
pub const STATUS_ERR_OP:  i32 = -1;
pub const STATUS_ERR_SVC: i32 = -2;
pub const STATUS_ERR_LEN: i32 = -3;

// ── Mailbox ─────────────────────────────────────────────────────
// `IPC_BUSY` guards against re-entrant requests: a client takes it
// before posting; the service task clears it after writing the
// response (but BEFORE terminating, so the next client sees `false`
// when it tries to acquire).
static IPC_BUSY: AtomicBool = AtomicBool::new(false);

static REQ_OP: AtomicU32 = AtomicU32::new(OP_NONE);
static REQ_LEN: AtomicU32 = AtomicU32::new(0);
static mut REQ_DATA: [u8; REQ_DATA_MAX] = [0u8; REQ_DATA_MAX];

static RSP_STATUS: AtomicI32 = AtomicI32::new(STATUS_PENDING);
static RSP_LEN: AtomicU32 = AtomicU32::new(0);
static mut RSP_DATA: [u8; RSP_DATA_MAX] = [0u8; RSP_DATA_MAX];

/// Service task entry. Runs in a kernel task tagged with
/// `cave_id = sys_wg_id` (set by the client before spawn — see
/// `dispatch_one_shot`). Reads the request, calls into the cave-
/// private API, writes the response, terminates via
/// `process::current_terminate`.
fn service_main() -> ! {
    let op = REQ_OP.load(Ordering::Acquire);
    match op {
        OP_PUBKEY    => handle_pubkey(),
        OP_HANDSHAKE => handle_handshake(),
        OP_WRAP      => handle_wrap(),
        OP_UNWRAP    => handle_unwrap(),
        _ => {
            RSP_STATUS.store(STATUS_ERR_OP, Ordering::Release);
        }
    }
    // Reset busy flag so the next client can acquire. Order
    // matters: response status MUST commit before busy clears,
    // otherwise a fast follow-up client might see the old response.
    core::sync::atomic::fence(Ordering::Release);
    IPC_BUSY.store(false, Ordering::Release);
    process::current_terminate();
}

fn handle_pubkey() {
    match sys_wg_service::service_pubkey() {
        Some(pk) => unsafe {
            let dst = core::ptr::addr_of_mut!(RSP_DATA) as *mut u8;
            for i in 0..wireguard::KEY_LEN {
                core::ptr::write_volatile(dst.add(i), pk[i]);
            }
            RSP_LEN.store(wireguard::KEY_LEN as u32, Ordering::Release);
            RSP_STATUS.store(STATUS_OK, Ordering::Release);
        },
        None => {
            RSP_STATUS.store(STATUS_ERR_SVC, Ordering::Release);
        }
    }
}

/// OP_HANDSHAKE request layout in `REQ_DATA` (108 bytes total):
///   byte   0     : peer_id (u8)
///   bytes  1..4  : reserved (zero)
///   bytes  4..36 : initiator_eph_pk ([u8; 32])
///   bytes 36..84 : enc_static ([u8; 48] = 32 plain + 16 tag)
///   bytes 84..112: enc_timestamp ([u8; 28] = 12 plain + 16 tag)
///
/// Response layout in `RSP_DATA` (76 bytes):
///   bytes  0..32 : responder_eph_pk
///   bytes 32..48 : enc_empty (AEAD tag — 0 plaintext + 16 tag)
///   bytes 48..60 : initiator_timestamp (echoed back from the
///                  decrypted InitMsg, for the client to sanity-
///                  check timestamp continuity)
const HS_REQ_LEN: usize = 112;
const HS_RSP_LEN: usize = 60;

fn handle_handshake() {
    let req_len = REQ_LEN.load(Ordering::Acquire) as usize;
    if req_len != HS_REQ_LEN {
        RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
        return;
    }
    let (peer_id_raw, eph_pk, enc_static, enc_ts) = unsafe {
        let src = core::ptr::addr_of!(REQ_DATA) as *const u8;
        let peer_id_raw = core::ptr::read_volatile(src);
        let mut eph_pk = [0u8; wireguard::KEY_LEN];
        for i in 0..wireguard::KEY_LEN { eph_pk[i] = core::ptr::read_volatile(src.add(4 + i)); }
        let mut enc_static = [0u8; wireguard::KEY_LEN + wireguard::TAG_LEN];
        for i in 0..enc_static.len() { enc_static[i] = core::ptr::read_volatile(src.add(36 + i)); }
        let mut enc_ts = [0u8; wireguard::TIMESTAMP_LEN + wireguard::TAG_LEN];
        for i in 0..enc_ts.len() { enc_ts[i] = core::ptr::read_volatile(src.add(84 + i)); }
        (peer_id_raw, eph_pk, enc_static, enc_ts)
    };
    let peer_id = sys_wg_service::PeerId::from(peer_id_raw);
    match sys_wg_service::complete_handshake_as_responder(
        peer_id, &eph_pk, &enc_static, &enc_ts,
    ) {
        Ok(wire) => {
            if wire.enc_empty.len() != wireguard::TAG_LEN {
                RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
                return;
            }
            unsafe {
                let dst = core::ptr::addr_of_mut!(RSP_DATA) as *mut u8;
                for i in 0..wireguard::KEY_LEN {
                    core::ptr::write_volatile(dst.add(i), wire.responder_eph_pk[i]);
                }
                for i in 0..wireguard::TAG_LEN {
                    core::ptr::write_volatile(dst.add(32 + i), wire.enc_empty[i]);
                }
                for i in 0..wireguard::TIMESTAMP_LEN {
                    core::ptr::write_volatile(dst.add(48 + i), wire.initiator_timestamp[i]);
                }
            }
            RSP_LEN.store(HS_RSP_LEN as u32, Ordering::Release);
            RSP_STATUS.store(STATUS_OK, Ordering::Release);
        }
        Err(_) => RSP_STATUS.store(STATUS_ERR_SVC, Ordering::Release),
    }
}

/// OP_WRAP request layout in `REQ_DATA`:
///   byte  0    : peer_id (u8)
///   bytes 1..4 : reserved (zero)
///   bytes 4..8 : plaintext_len (u32 LE)
///   bytes 8..  : plaintext_len bytes of plaintext
///
/// Response: ciphertext bytes (with 16-byte AEAD tag) in
/// `RSP_DATA`; `RSP_LEN` set to ct.len().
fn handle_wrap() {
    let req_len = REQ_LEN.load(Ordering::Acquire) as usize;
    if req_len < 8 {
        RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
        return;
    }
    let (peer_id_raw, pt_len) = unsafe {
        let src = core::ptr::addr_of!(REQ_DATA) as *const u8;
        let peer_id_raw = core::ptr::read_volatile(src);
        let mut len_bytes = [0u8; 4];
        for i in 0..4 { len_bytes[i] = core::ptr::read_volatile(src.add(4 + i)); }
        (peer_id_raw, u32::from_le_bytes(len_bytes) as usize)
    };
    if 8usize.saturating_add(pt_len) > req_len || 8usize.saturating_add(pt_len) > REQ_DATA_MAX {
        RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
        return;
    }
    let plaintext: &[u8] = unsafe {
        let p = (core::ptr::addr_of!(REQ_DATA) as *const u8).add(8);
        core::slice::from_raw_parts(p, pt_len)
    };
    let peer_id = sys_wg_service::PeerId::from(peer_id_raw);
    match sys_wg_service::wrap(peer_id, plaintext) {
        Ok(ct) => {
            if ct.len() > RSP_DATA_MAX {
                RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
                return;
            }
            unsafe {
                let dst = core::ptr::addr_of_mut!(RSP_DATA) as *mut u8;
                for i in 0..ct.len() {
                    core::ptr::write_volatile(dst.add(i), ct[i]);
                }
            }
            RSP_LEN.store(ct.len() as u32, Ordering::Release);
            RSP_STATUS.store(STATUS_OK, Ordering::Release);
        }
        Err(_) => RSP_STATUS.store(STATUS_ERR_SVC, Ordering::Release),
    }
}

/// OP_UNWRAP request layout in `REQ_DATA`:
///   byte   0     : peer_id (u8)
///   bytes  1..4  : reserved (zero)
///   bytes  4..12 : counter (u64 LE)
///   bytes 12..16 : ct_len (u32 LE)
///   bytes 16..   : ct_len bytes of ciphertext+tag
///
/// Response: plaintext bytes in `RSP_DATA`; `RSP_LEN` set.
fn handle_unwrap() {
    let req_len = REQ_LEN.load(Ordering::Acquire) as usize;
    if req_len < 16 {
        RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
        return;
    }
    let (peer_id_raw, counter, ct_len) = unsafe {
        let src = core::ptr::addr_of!(REQ_DATA) as *const u8;
        let peer_id_raw = core::ptr::read_volatile(src);
        let mut counter_bytes = [0u8; 8];
        for i in 0..8 { counter_bytes[i] = core::ptr::read_volatile(src.add(4 + i)); }
        let mut ct_len_bytes = [0u8; 4];
        for i in 0..4 { ct_len_bytes[i] = core::ptr::read_volatile(src.add(12 + i)); }
        (peer_id_raw,
         u64::from_le_bytes(counter_bytes),
         u32::from_le_bytes(ct_len_bytes) as usize)
    };
    if 16usize.saturating_add(ct_len) > req_len || 16usize.saturating_add(ct_len) > REQ_DATA_MAX {
        RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
        return;
    }
    let ct: &[u8] = unsafe {
        let p = (core::ptr::addr_of!(REQ_DATA) as *const u8).add(16);
        core::slice::from_raw_parts(p, ct_len)
    };
    let peer_id = sys_wg_service::PeerId::from(peer_id_raw);
    match sys_wg_service::unwrap(peer_id, counter, ct) {
        Ok(pt) => {
            if pt.len() > RSP_DATA_MAX {
                RSP_STATUS.store(STATUS_ERR_LEN, Ordering::Release);
                return;
            }
            unsafe {
                let dst = core::ptr::addr_of_mut!(RSP_DATA) as *mut u8;
                for i in 0..pt.len() {
                    core::ptr::write_volatile(dst.add(i), pt[i]);
                }
            }
            RSP_LEN.store(pt.len() as u32, Ordering::Release);
            RSP_STATUS.store(STATUS_OK, Ordering::Release);
        }
        Err(_) => RSP_STATUS.store(STATUS_ERR_SVC, Ordering::Release),
    }
}

/// Client-side helper. Acquires the mailbox, sets up the request,
/// spawns a fresh service task tagged with sys-wg's cave_id, and
/// yields until a response arrives. Returns the response data on
/// success, `None` on any error.
///
/// Single-threaded contract — IPC_BUSY guards against concurrent
/// callers. A second caller racing in will spin-yield until the
/// first finishes; since Bat_OS is cooperative single-CPU, in
/// practice this can't happen unless the client yields explicitly
/// before completing.
fn dispatch_one_shot(op: u32, req: &[u8]) -> Option<&'static [u8]> {
    if req.len() > REQ_DATA_MAX { return None; }
    let sys_wg_id = sys_caves::sys_wg_id()? as u16;

    // Acquire the mailbox.
    while IPC_BUSY.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
        scheduler::yield_now();
    }

    // Populate request bytes.
    unsafe {
        let dst = core::ptr::addr_of_mut!(REQ_DATA) as *mut u8;
        for i in 0..req.len() {
            core::ptr::write_volatile(dst.add(i), req[i]);
        }
    }
    REQ_LEN.store(req.len() as u32, Ordering::Release);
    RSP_STATUS.store(STATUS_PENDING, Ordering::Release);
    RSP_LEN.store(0, Ordering::Release);
    // Publish op last so the service-side load+Acquire sees a
    // fully-populated request.
    REQ_OP.store(op, Ordering::Release);

    // Spawn the service task. Priority 5 (higher than the
    // calling shell, which is task 0 @ 255), so as soon as we
    // yield the service is picked.
    let svc_id = match process::create_kernel_task(
        "sys-wg-svc", service_main, /* priority */ 5,
    ) {
        Some(id) => id,
        None => {
            IPC_BUSY.store(false, Ordering::Release);
            return None;
        }
    };
    // Tag with sys-wg's cave id. From this point until the
    // task terminates, the scheduler MMU hook will swap TTBR0
    // to sys-wg's L1 every time this task is scheduled.
    process::set_cave(svc_id, sys_wg_id);

    // Wait for response. Bounded by a generous loop limit so a
    // regressed scheduler can't lock us up; 1024 yields is far
    // more than the single yield we expect.
    let mut tries = 0usize;
    while RSP_STATUS.load(Ordering::Acquire) == STATUS_PENDING && tries < 1024 {
        scheduler::yield_now();
        tries += 1;
    }
    if RSP_STATUS.load(Ordering::Acquire) != STATUS_OK {
        return None;
    }

    // Read response bytes. Returning `&'static [u8]` is sound:
    // RSP_DATA is a static; the caller copies bytes out before
    // releasing IPC_BUSY (the service task already released
    // before terminating, but we hold it again here for the
    // window between this return and the caller's copy).
    let len = RSP_LEN.load(Ordering::Acquire) as usize;
    let ptr = unsafe { core::ptr::addr_of!(RSP_DATA) as *const u8 };
    Some(unsafe { core::slice::from_raw_parts(ptr, len) })
}

/// Client-side public API: request sys-wg's static public key via
/// IPC instead of via the synchronous `sys_wg_service::service_pubkey`.
/// Returns the 32-byte X25519 pubkey, or `None` if the IPC path
/// fails (mailbox unreachable, service task couldn't be spawned,
/// or the service reported an error).
pub fn request_pubkey() -> Option<[u8; wireguard::KEY_LEN]> {
    let bytes = dispatch_one_shot(OP_PUBKEY, &[])?;
    if bytes.len() != wireguard::KEY_LEN { return None; }
    let mut out = [0u8; wireguard::KEY_LEN];
    out.copy_from_slice(&bytes[..wireguard::KEY_LEN]);
    Some(out)
}

/// Parsed responder side of a handshake completed via the IPC
/// mailbox.
pub struct HandshakeResult {
    pub responder_eph_pk: [u8; wireguard::KEY_LEN],
    pub enc_empty: [u8; wireguard::TAG_LEN],
    pub initiator_timestamp: [u8; wireguard::TIMESTAMP_LEN],
}

/// IPC client for OP_HANDSHAKE. Submits an initiator's InitMsg
/// payload (eph_pk + enc_static + enc_timestamp) to sys-wg via
/// the mailbox; sys-wg validates the pinned peer pubkey, runs
/// the responder side, installs session keys in the cave-private
/// peer slot, and returns the bytes the caller needs to build a
/// ResponseMsg wire packet + finish its own initiator-side
/// derivation (`initiator_finish_handshake`).
pub fn request_handshake(
    peer_id: u8,
    initiator_eph_pk: &[u8; wireguard::KEY_LEN],
    enc_static: &[u8],
    enc_timestamp: &[u8],
) -> Option<HandshakeResult> {
    if enc_static.len() != wireguard::KEY_LEN + wireguard::TAG_LEN { return None; }
    if enc_timestamp.len() != wireguard::TIMESTAMP_LEN + wireguard::TAG_LEN { return None; }
    let mut req = [0u8; HS_REQ_LEN];
    req[0] = peer_id;
    req[4..36].copy_from_slice(initiator_eph_pk);
    req[36..84].copy_from_slice(enc_static);
    req[84..112].copy_from_slice(enc_timestamp);
    let bytes = dispatch_one_shot(OP_HANDSHAKE, &req)?;
    if bytes.len() != HS_RSP_LEN { return None; }
    let mut responder_eph_pk = [0u8; wireguard::KEY_LEN];
    responder_eph_pk.copy_from_slice(&bytes[..32]);
    let mut enc_empty = [0u8; wireguard::TAG_LEN];
    enc_empty.copy_from_slice(&bytes[32..48]);
    let mut initiator_timestamp = [0u8; wireguard::TIMESTAMP_LEN];
    initiator_timestamp.copy_from_slice(&bytes[48..60]);
    Some(HandshakeResult { responder_eph_pk, enc_empty, initiator_timestamp })
}

/// IPC client for OP_WRAP. Encrypts `plaintext` under the peer
/// slot's responder send_key (via the service task, never
/// touching the keys directly). Returns the ciphertext (with
/// 16-byte AEAD tag).
pub fn request_wrap(peer_id: u8, plaintext: &[u8]) -> Option<alloc::vec::Vec<u8>> {
    if 8 + plaintext.len() > REQ_DATA_MAX { return None; }
    let mut req = alloc::vec![0u8; 8 + plaintext.len()];
    req[0] = peer_id;
    req[4..8].copy_from_slice(&(plaintext.len() as u32).to_le_bytes());
    req[8..].copy_from_slice(plaintext);
    let bytes = dispatch_one_shot(OP_WRAP, &req)?;
    Some(bytes.to_vec())
}

/// IPC client for OP_UNWRAP. Decrypts `ct_with_tag` under the
/// peer slot's responder recv_key at the given counter (via the
/// service task). Returns the plaintext.
pub fn request_unwrap(peer_id: u8, counter: u64, ct_with_tag: &[u8])
    -> Option<alloc::vec::Vec<u8>>
{
    if 16 + ct_with_tag.len() > REQ_DATA_MAX { return None; }
    let mut req = alloc::vec![0u8; 16 + ct_with_tag.len()];
    req[0] = peer_id;
    req[4..12].copy_from_slice(&counter.to_le_bytes());
    req[12..16].copy_from_slice(&(ct_with_tag.len() as u32).to_le_bytes());
    req[16..].copy_from_slice(ct_with_tag);
    let bytes = dispatch_one_shot(OP_UNWRAP, &req)?;
    Some(bytes.to_vec())
}

/// Selftest used by `sys-wg-ipc-selftest` to verify the IPC path
/// returns the same value as the direct API. Returns
/// `(direct_pk_prefix, ipc_pk_prefix, equal)` so the shell
/// command can render both for debugging.
pub fn selftest() -> Option<([u8; 8], [u8; 8], bool)> {
    let direct = sys_wg_service::service_pubkey()?;
    let via_ipc = request_pubkey()?;
    let equal = direct == via_ipc;
    let mut a = [0u8; 8];
    let mut b = [0u8; 8];
    a.copy_from_slice(&direct[..8]);
    b.copy_from_slice(&via_ipc[..8]);
    Some((a, b, equal))
}

/// IPC wrap/unwrap round-trip selftest.
///   1. Generate an initiator keypair caller-side.
///   2. Register it with sys-wg → `peer_id`.
///   3. Drive a Noise IK handshake directly (NOT through IPC —
///      that's OP_HANDSHAKE, future slice). Both sides end up
///      with mirror TransportKeys; caller holds its half on the
///      stack, sys-wg holds the responder half in the cave-
///      private peer slot.
///   4. `request_wrap(peer_id, "hello")` → ciphertext (sys-wg
///      encrypts with the responder's send_key, which equals
///      our recv_key).
///   5. Caller `transport_recv` the ciphertext → expect "hello".
///   6. Caller `transport_send` "world" → ct2.
///   7. `request_unwrap(peer_id, 0, &ct2)` → expect "world".
///
/// Returns `(wrap_ok, unwrap_ok)`.
pub fn selftest_wrap_unwrap() -> Option<(bool, bool)> {
    use crate::net::wireguard::{
        self, WgKeypair, TIMESTAMP_LEN, TransportKeys,
    };

    let our_pk = sys_wg_service::service_pubkey()?;
    let initiator = WgKeypair::generate();

    // Register (or reuse existing slot).
    let peer_id = match sys_wg_service::register_peer(initiator.static_pk) {
        Ok(id) => id,
        Err(sys_wg_service::SysWgError::DuplicatePeer) => {
            sys_wg_service::find_peer_by_pk(&initiator.static_pk)?
        }
        Err(_) => return None,
    };

    // Drive a handshake — through OP_HANDSHAKE so the entire
    // responder side runs in the service task. The caller never
    // touches `sys_wg_service` directly.
    let timestamp = [0u8; TIMESTAMP_LEN];
    let (mut init_state, init_eph_pk, enc_static, enc_ts) =
        wireguard::initiator_send_init(&initiator, &our_pk, &timestamp).ok()?;
    let hs = request_handshake(
        peer_id.as_u8(), &init_eph_pk, &enc_static, &enc_ts,
    )?;
    if hs.initiator_timestamp != timestamp { return None; }
    let mut caller_keys: TransportKeys = wireguard::initiator_finish_handshake(
        &initiator, &mut init_state,
        &hs.responder_eph_pk,
        &hs.enc_empty,
    ).ok()?;

    // OP_WRAP through IPC: sys-wg encrypts; caller decrypts.
    let plaintext = b"hello-via-ipc-wrap";
    let ct = request_wrap(peer_id.as_u8(), plaintext)?;
    let pt = wireguard::transport_recv(&mut caller_keys, 0, &ct).ok()?;
    let wrap_ok = pt.as_slice() == plaintext;

    // OP_UNWRAP through IPC: caller encrypts; sys-wg decrypts.
    let plaintext2 = b"world-via-ipc-unwrap";
    let ct2 = wireguard::transport_send(&mut caller_keys, plaintext2).ok()?;
    let pt2 = request_unwrap(peer_id.as_u8(), 0, &ct2)?;
    let unwrap_ok = pt2.as_slice() == plaintext2;

    Some((wrap_ok, unwrap_ok))
}
