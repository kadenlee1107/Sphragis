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
/// to fit our largest expected opcode argument: handshake
/// 148-byte InitMsg + a little headroom (future slices).
pub const REQ_DATA_MAX: usize = 192;

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
        OP_PUBKEY => handle_pubkey(),
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
