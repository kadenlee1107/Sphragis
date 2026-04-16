// Bat_OS — Per-Cave Resource Quotas
// =============================================================================
//
// ROOT-6 (PENTEST_SUMMARY.md): every global resource table — sockets, PCBs,
// eventfds, timerfds, threads, VFS nodes, epoll instances, mmap pages — is
// shared across caves with no per-cave cap. A single malicious cave can
// exhaust any of them and deny every other cave.
//
// This module implements a simple per-cave "ledger" that allocating syscalls
// charge against and deallocating syscalls refund. On overflow the syscall
// returns the appropriate Linux errno (-ENOMEM for memory, -EMFILE for fds,
// -EAGAIN for clone) so well-behaved libcs see familiar failures.
//
// NOTES
// -----
// * Defaults are generous (1 GiB mem, 32 sockets, 16 threads, 64 fds, 16
//   eventfds / timerfds / epolls).  The existing test binaries — `hello`,
//   `v8_exec`, busybox, the Chromium blob — all fit comfortably.
// * `cave_id == usize::MAX` means "no cave active" (kernel / early boot).
//   In that case charge/refund are no-ops: kernel-internal allocation is
//   not rate-limited.
// * Refund is saturating — we never wrap below zero.  If the accounting
//   ever drifts (e.g. an allocation path was added without hooking the
//   free path) we want the underflow to be benign, not to wrap to a huge
//   value that then blocks the cave forever.
// * All state is atomic — the module is call-safe from any context
//   including interrupt handlers.

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::batcave::cave::MAX_CAVES;

// Linux errnos used as negative return values.
const ENOMEM: i64 = -12;
const EMFILE: i64 = -24;
const EAGAIN: i64 = -11;

/// Resources tracked per cave.
#[derive(Clone, Copy)]
pub enum Resource {
    Mem,        // bytes (page-granular in practice)
    Sockets,
    Threads,
    Fds,
    Epolls,
    Eventfds,
    Timerfds,
}

/// One cave's resource ledger.
pub struct CaveQuota {
    pub mem_bytes:     AtomicUsize,
    pub sockets:       AtomicUsize,
    pub threads:       AtomicUsize,
    pub fds:           AtomicUsize,
    pub epolls:        AtomicUsize,
    pub eventfds:      AtomicUsize,
    pub timerfds:      AtomicUsize,

    // Caps (plain usize — set at init, never changes at runtime).
    pub mem_limit:     usize,
    pub sockets_limit: usize,
    pub threads_limit: usize,
    pub fds_limit:     usize,
    pub epolls_limit:  usize,
    pub eventfds_limit: usize,
    pub timerfds_limit: usize,
}

impl CaveQuota {
    const fn new() -> Self {
        Self {
            mem_bytes: AtomicUsize::new(0),
            sockets:   AtomicUsize::new(0),
            threads:   AtomicUsize::new(0),
            fds:       AtomicUsize::new(0),
            epolls:    AtomicUsize::new(0),
            eventfds:  AtomicUsize::new(0),
            timerfds:  AtomicUsize::new(0),

            mem_limit:      DEFAULT_MEM,
            sockets_limit:  DEFAULT_SOCKETS,
            threads_limit:  DEFAULT_THREADS,
            fds_limit:      DEFAULT_FDS,
            epolls_limit:   DEFAULT_EPOLLS,
            eventfds_limit: DEFAULT_EVENTFDS,
            timerfds_limit: DEFAULT_TIMERFDS,
        }
    }
}

// ─── Default limits ──────────────────────────────────────────────────────
// Chosen to be comfortably above anything the existing test binaries
// consume (hello, v8_exec, busybox, Chromium blob startup) while still
// catching "allocate until death" attacks.
pub const DEFAULT_MEM:      usize = 1 << 30;   // 1 GiB
pub const DEFAULT_SOCKETS:  usize = 32;
pub const DEFAULT_THREADS:  usize = 16;
pub const DEFAULT_FDS:      usize = 64;
pub const DEFAULT_EPOLLS:   usize = 16;
pub const DEFAULT_EVENTFDS: usize = 16;
pub const DEFAULT_TIMERFDS: usize = 16;

// ─── The ledger ──────────────────────────────────────────────────────────
static CAVE_QUOTAS: [CaveQuota; MAX_CAVES] = {
    const INIT: CaveQuota = CaveQuota::new();
    [INIT; MAX_CAVES]
};

/// Is this cave_id valid for ledger access?
#[inline]
fn valid(cave_id: usize) -> bool {
    cave_id < MAX_CAVES
}

/// Pick the (counter, limit, errno) triple for a resource on the given cave.
#[inline]
fn slot(cave_id: usize, r: Resource) -> (&'static AtomicUsize, usize, i64) {
    let q = &CAVE_QUOTAS[cave_id];
    match r {
        Resource::Mem       => (&q.mem_bytes, q.mem_limit,     ENOMEM),
        Resource::Sockets   => (&q.sockets,   q.sockets_limit, EMFILE),
        Resource::Threads   => (&q.threads,   q.threads_limit, EAGAIN),
        Resource::Fds       => (&q.fds,       q.fds_limit,     EMFILE),
        Resource::Epolls    => (&q.epolls,    q.epolls_limit,  EMFILE),
        Resource::Eventfds  => (&q.eventfds,  q.eventfds_limit, EMFILE),
        Resource::Timerfds  => (&q.timerfds,  q.timerfds_limit, EMFILE),
    }
}

/// Charge `amount` of `r` against `cave_id`.  Returns `Ok(())` if the
/// reservation fit inside the cap, or `Err(-errno)` if it would overflow.
/// The counter is rolled back on failure so repeated failing charges
/// don't drift the ledger.
///
/// When `cave_id` is out of range we treat it as "kernel context" and
/// allow the allocation unconditionally.  This keeps early boot, kthreads,
/// and interrupt handlers from being blocked by the ledger.
pub fn charge(cave_id: usize, r: Resource, amount: usize) -> Result<(), i64> {
    if !valid(cave_id) { return Ok(()); }
    let (ctr, limit, errno) = slot(cave_id, r);
    let prev = ctr.fetch_add(amount, Ordering::Relaxed);
    if prev.saturating_add(amount) > limit {
        // Roll back — we never want the counter to stick above the cap.
        ctr.fetch_sub(amount, Ordering::Relaxed);
        return Err(errno);
    }
    Ok(())
}

/// Refund `amount` of `r` to `cave_id`.  Saturating: we never wrap below
/// zero, even if the ledger has drifted (unhooked allocation path, reap
/// called twice, …).  Out-of-range cave_id is a silent no-op.
pub fn refund(cave_id: usize, r: Resource, amount: usize) {
    if !valid(cave_id) { return; }
    let (ctr, _, _) = slot(cave_id, r);
    loop {
        let cur = ctr.load(Ordering::Relaxed);
        let new = cur.saturating_sub(amount);
        if ctr.compare_exchange(cur, new,
                Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return;
        }
    }
}

/// Read the current usage (for diagnostics / /proc).
#[allow(dead_code)]
pub fn usage(cave_id: usize, r: Resource) -> (usize, usize) {
    if !valid(cave_id) { return (0, 0); }
    let (ctr, limit, _) = slot(cave_id, r);
    (ctr.load(Ordering::Relaxed), limit)
}

/// Reset all counters for a cave (called when the cave is destroyed /
/// re-allocated).  Limits stay untouched.
#[allow(dead_code)]
pub fn reset(cave_id: usize) {
    if !valid(cave_id) { return; }
    let q = &CAVE_QUOTAS[cave_id];
    q.mem_bytes.store(0, Ordering::Relaxed);
    q.sockets.store(0, Ordering::Relaxed);
    q.threads.store(0, Ordering::Relaxed);
    q.fds.store(0, Ordering::Relaxed);
    q.epolls.store(0, Ordering::Relaxed);
    q.eventfds.store(0, Ordering::Relaxed);
    q.timerfds.store(0, Ordering::Relaxed);
}

/// Convenience: charge the *active* cave (from cave::get_active()).  When
/// there is no active cave we're in kernel context — returns Ok.
pub fn charge_active(r: Resource, amount: usize) -> Result<(), i64> {
    charge(crate::batcave::cave::get_active(), r, amount)
}

/// Convenience: refund the *active* cave.
pub fn refund_active(r: Resource, amount: usize) {
    refund(crate::batcave::cave::get_active(), r, amount);
}
