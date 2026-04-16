// Bat_OS — Linux epoll(7) implementation for BatCave processes
//
// Why this exists:
//   Chromium's entire event loop (Mojo IPC, the network service, timers,
//   IPC::ChannelMojo, etc.) is built on top of epoll. Blink, V8's
//   inspector, and the network stack all rely on epoll_create1 /
//   epoll_ctl / epoll_pwait. Without a working epoll, the browser
//   process can't pump messages and Chromium deadlocks on startup.
//
// Design summary:
//   * An epoll "instance" is a kernel object allocated out of a static
//     table (`INSTANCES`). It is exposed to userspace via the existing
//     fd table (`super::fd`) by allocating an fd whose `node_idx` is a
//     sentinel value in the range [EPOLL_NODE_BASE, EPOLL_NODE_BASE +
//     MAX_INSTANCES). The syscall dispatcher detects this with
//     `is_epoll_fd()` and routes the call here.
//   * Each instance holds a fixed-capacity "interest list" of
//     (fd, events, data, flags) tuples, plus a parallel "ready" bitmap
//     so delivery order is deterministic and cheap.
//   * Other fds (sockets, pipes) signal readiness by calling
//     `mark_ready(fd, events)`. That walks every instance and ORs the
//     bits into whichever interest entries watch that fd. This is O(N*M)
//     but N=64, M=256 so the worst case is ~16K comparisons — fine for
//     the cooperative single-threaded runner.
//   * `epoll_pwait` with timeout=0 is a non-blocking poll. timeout=-1
//     spins forever with `yield` (really: a WFE hint). A positive
//     timeout is treated as "spin up to N iterations" — the BatCave
//     runner has no preemptive timer yet, so we approximate.
//
// Non-goals / known limitations (see bottom of file):
//   * No heap — everything is static. 64 instances × 256 interests each.
//   * Edge-triggered (EPOLLET) and EPOLLONESHOT are stored but the
//     semantics are only partially enforced — see TODOs.
//   * `sigmask` is ignored (we have no signal delivery yet).
//   * `mark_ready` must be called from whichever subsystem owns the fd;
//     there is no automatic poll integration yet.
//
// Thread-safety:
//   Bat_OS is single-threaded per BatCave today, but we still use
//   `AtomicBool` for the `in_use` slot flag so that when we grow to
//   SMP the allocator is race-free. Interest mutation is serialized
//   by being only called from syscall context.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// ─────────────────────── Public Linux ABI ───────────────────────

/// Event descriptor exchanged with userspace. Matches the Linux
/// `struct epoll_event` ABI: packed, little-endian, 12 bytes on 64-bit.
/// Note that glibc uses `__attribute__((__packed__))` on x86_64 so the
/// struct is 12 bytes there; on arm64 musl it's also 12 bytes. We match.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    /// Bitmask of EPOLLIN / EPOLLOUT / EPOLLERR / ... plus flags
    /// EPOLLET and EPOLLONESHOT in the high bits.
    pub events: u32,
    /// Opaque cookie returned to the caller on wakeup. Chromium stuffs
    /// a pointer to its `MessagePumpEpoll::EpollHandler` object here.
    pub data: u64,
}

// Operations for epoll_ctl(2).
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

// Event bits. Values chosen to match Linux exactly — Chromium's headers
// hardcode these and we have no choice.
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLPRI: u32 = 0x002;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
pub const EPOLLRDHUP: u32 = 0x2000;
pub const EPOLLONESHOT: u32 = 0x40000000;
pub const EPOLLET: u32 = 0x80000000;

/// Flag to epoll_create1(2) meaning "set FD_CLOEXEC on the returned fd".
/// We record it but there is no exec path yet that clears it.
pub const EPOLL_CLOEXEC: u32 = 0o2000000;

// Linux errnos we return. Kept local to avoid a dependency on syscall.rs
// constants (epoll may be called from tests or bench code).
const EBADF: i64 = -9;
const ENOMEM: i64 = -12;
const EFAULT: i64 = -14;
const EINVAL: i64 = -22;
const EEXIST: i64 = -17;
const ENOENT: i64 = -2;
const EMFILE: i64 = -24;

// ─────────────────────── Internal Tables ───────────────────────

/// Max number of concurrent epoll instances across all BatCaves.
/// Chromium's browser process typically has 2–3 (main pump, IO thread
/// pump, watcher). 64 leaves plenty of headroom for renderers.
const MAX_INSTANCES: usize = 64;

/// Max interests per instance. Chromium's browser process on Linux
/// watches on the order of 50–150 fds at peak. 256 is comfortable
/// without blowing up static memory (256 * 64 * 32B ≈ 512KB).
const MAX_INTERESTS: usize = 256;

/// Sentinel `node_idx` values written into the fd table to mark an fd
/// as an epoll fd. fd.rs uses u16 for node_idx, so we claim the top
/// slice of the u16 space. The VFS never allocates this range.
const EPOLL_NODE_BASE: u16 = 0xFF00;

/// One interest entry = one fd being watched by one epoll instance.
#[derive(Clone, Copy)]
struct Interest {
    /// True if this slot is populated.
    used: bool,
    /// The fd being watched (the "target" — NOT the epoll fd).
    fd: i32,
    /// Event mask the user asked to watch, including EPOLLET/ONESHOT.
    events: u32,
    /// Opaque userspace cookie (struct epoll_event.data.u64).
    data: u64,
    /// Currently-pending ready events, filtered by `events`. Reset
    /// whenever the interest is reported via epoll_pwait for a
    /// level-triggered watch; retained for edge-triggered until
    /// explicitly cleared by a subsequent mark_ready with new bits.
    ready: u32,
    /// Suppress further deliveries once this interest has fired once.
    /// Set when EPOLLONESHOT is in `events` and we've delivered.
    oneshot_fired: bool,
}

impl Interest {
    const fn empty() -> Self {
        Interest {
            used: false,
            fd: -1,
            events: 0,
            data: 0,
            ready: 0,
            oneshot_fired: false,
        }
    }
}

/// One epoll instance. Owns a fixed array of interests.
struct EpollInstance {
    /// True if the slot is in use. Atomic so alloc is SMP-safe later.
    in_use: AtomicBool,
    /// Flags passed to epoll_create1 (currently only EPOLL_CLOEXEC).
    flags: AtomicU32,
    /// Interest list. Indexed by slot, not by fd.
    interests: [Interest; MAX_INTERESTS],
}

impl EpollInstance {
    const fn empty() -> Self {
        EpollInstance {
            in_use: AtomicBool::new(false),
            flags: AtomicU32::new(0),
            interests: [Interest::empty(); MAX_INTERESTS],
        }
    }
}

// Static table of instances. `static mut` plus atomic guards is the
// pattern used throughout Bat_OS (see vfs.rs, fd.rs) — no_std, no heap.
static mut INSTANCES: [EpollInstance; MAX_INSTANCES] = {
    const E: EpollInstance = EpollInstance::empty();
    [E; MAX_INSTANCES]
};

// ─────────────────────── Instance Helpers ───────────────────────

/// Allocate a fresh instance slot. Returns the slot index, not an fd.
fn alloc_instance(flags: u32) -> Option<usize> {
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for (i, inst) in table.iter_mut().enumerate() {
            // compare_exchange: only claim slots currently flagged free.
            if inst
                .in_use
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                // Fresh slot — wipe any stale interests from a previous tenant.
                for slot in inst.interests.iter_mut() {
                    *slot = Interest::empty();
                }
                inst.flags.store(flags, Ordering::Release);
                return Some(i);
            }
        }
    }
    None
}

/// Release an instance slot. Called from epoll_close().
fn free_instance(slot: usize) {
    if slot >= MAX_INSTANCES {
        return;
    }
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for i in 0..MAX_INTERESTS {
            table[slot].interests[i] = Interest::empty();
        }
        table[slot].flags.store(0, Ordering::Release);
        table[slot].in_use.store(false, Ordering::Release);
    }
}

/// Borrow an instance mutably by slot index, returning None if the slot
/// is free. Callers must already hold "the lock" (we're single-threaded).
fn get_instance_mut(slot: usize) -> Option<&'static mut EpollInstance> {
    if slot >= MAX_INSTANCES {
        return None;
    }
    unsafe {
        let inst = &mut (*core::ptr::addr_of_mut!(INSTANCES))[slot];
        if inst.in_use.load(Ordering::Acquire) {
            Some(inst)
        } else {
            None
        }
    }
}

// ─────────────────────── fd ↔ instance glue ───────────────────────

/// Return true if the given fd is an epoll fd (i.e. was allocated via
/// `epoll_create1`). The syscall dispatcher uses this to decide whether
/// to send read/write/close through the VFS or through here.
pub fn is_epoll_fd(fd: i32) -> bool {
    instance_slot_for_fd(fd).is_some()
}

/// Translate an fd into its backing instance slot, or None.
fn instance_slot_for_fd(fd: i32) -> Option<usize> {
    if fd < 0 {
        return None;
    }
    let entry = super::fd::get(fd as u32)?;
    let n = entry.node_idx;
    if n >= EPOLL_NODE_BASE && (n as usize) < EPOLL_NODE_BASE as usize + MAX_INSTANCES {
        Some((n - EPOLL_NODE_BASE) as usize)
    } else {
        None
    }
}

/// Called from sys_close when the fd is known to be an epoll fd.
/// Frees the backing instance, then the caller frees the fd entry.
pub fn epoll_close(fd: i32) -> i64 {
    match instance_slot_for_fd(fd) {
        Some(slot) => {
            free_instance(slot);
            0
        }
        None => EBADF,
    }
}

// ─────────────────────── Syscall entry points ───────────────────────

/// epoll_create1(2). Returns a new fd or -errno.
pub fn epoll_create1(flags: u32) -> i64 {
    // Only EPOLL_CLOEXEC is defined; anything else is EINVAL.
    if flags & !EPOLL_CLOEXEC != 0 {
        return EINVAL;
    }

    let slot = match alloc_instance(flags) {
        Some(s) => s,
        None => return ENOMEM, // out of instance slots
    };

    // Expose the instance to userspace via the fd table. We borrow the
    // existing fd allocator and park our slot index in `node_idx` plus
    // an offset so `is_epoll_fd` can recognize it later.
    let sentinel_node = EPOLL_NODE_BASE + slot as u16;
    match super::fd::alloc_fd(sentinel_node, flags) {
        Ok(fd) => fd as i64,
        Err(e) => {
            // Roll back the instance on fd-table exhaustion.
            free_instance(slot);
            e
        }
    }
}

/// epoll_ctl(2). Add/mod/del an interest in `fd` on the epoll instance
/// identified by `epfd`. `event` may be null for EPOLL_CTL_DEL.
///
/// SAFETY: `event` is a userspace pointer. We do a minimal alignment
/// check and dereference inside an unsafe block. In a post-MMU world
/// this should go through copy_from_user; for now the BatCave shares
/// the address space with the kernel so a direct read is defensible.
pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *const EpollEvent) -> i64 {
    let slot = match instance_slot_for_fd(epfd) {
        Some(s) => s,
        None => return EBADF,
    };
    // Can't watch yourself — Linux rejects this with EINVAL.
    if epfd == fd {
        return EINVAL;
    }
    // Target fd must exist (except for DEL where some callers tolerate
    // stale fds; Linux itself returns ENOENT here, so we do too).
    if super::fd::get(fd as u32).is_none() {
        return EBADF;
    }

    let inst = match get_instance_mut(slot) {
        Some(i) => i,
        None => return EBADF,
    };

    match op {
        EPOLL_CTL_ADD => {
            // Reject if fd already registered — Linux returns EEXIST.
            for entry in inst.interests.iter() {
                if entry.used && entry.fd == fd {
                    return EEXIST;
                }
            }
            if event.is_null() {
                return EFAULT;
            }
            // SAFETY: pointer provenance is the caller's problem; we
            // copy field-by-field via addr_of! (taking a reference to a
            // packed field is UB, hence the raw pointer dance).
            let (events, data) = unsafe {
                let p = event;
                (
                    core::ptr::read_unaligned(core::ptr::addr_of!((*p).events)),
                    core::ptr::read_unaligned(core::ptr::addr_of!((*p).data)),
                )
            };
            // Find a free interest slot.
            for entry in inst.interests.iter_mut() {
                if !entry.used {
                    *entry = Interest {
                        used: true,
                        fd,
                        events,
                        data,
                        ready: 0,
                        oneshot_fired: false,
                    };
                    return 0;
                }
            }
            ENOMEM // no free interest slot
        }
        EPOLL_CTL_MOD => {
            if event.is_null() {
                return EFAULT;
            }
            let (events, data) = unsafe {
                let p = event;
                (
                    core::ptr::read_unaligned(core::ptr::addr_of!((*p).events)),
                    core::ptr::read_unaligned(core::ptr::addr_of!((*p).data)),
                )
            };
            for entry in inst.interests.iter_mut() {
                if entry.used && entry.fd == fd {
                    entry.events = events;
                    entry.data = data;
                    // A MOD clears the oneshot latch per Linux semantics.
                    entry.oneshot_fired = false;
                    // Mask any stale readiness bits the caller no longer cares about.
                    entry.ready &= events | EPOLLERR | EPOLLHUP;
                    return 0;
                }
            }
            ENOENT
        }
        EPOLL_CTL_DEL => {
            for entry in inst.interests.iter_mut() {
                if entry.used && entry.fd == fd {
                    *entry = Interest::empty();
                    return 0;
                }
            }
            ENOENT
        }
        _ => EINVAL,
    }
}

/// epoll_pwait(2). Copy at most `maxevents` ready events to `events`
/// and return the count. `timeout` in milliseconds: 0 = non-blocking,
/// -1 = wait forever, >0 = wait that many milliseconds.
///
/// `sigmask` is accepted for ABI compatibility but ignored — Bat_OS
/// has no signal delivery yet.
pub fn epoll_pwait(
    epfd: i32,
    events: *mut EpollEvent,
    maxevents: i32,
    timeout: i32,
    _sigmask: *const u64,
) -> i64 {
    if maxevents <= 0 {
        return EINVAL;
    }
    if events.is_null() {
        return EFAULT;
    }

    let slot = match instance_slot_for_fd(epfd) {
        Some(s) => s,
        None => return EBADF,
    };

    // Cooperative spin loop. Each iteration: scan interests for ready
    // bits, copy them out, return. If nothing is ready, yield and
    // maybe loop (depending on timeout).
    //
    // With no timer subsystem we approximate `timeout_ms` by counting
    // spin iterations — roughly 10µs each on the current runner. A
    // proper implementation will block on a waitqueue once the
    // scheduler grows timers.
    const SPIN_PER_MS: i32 = 100;
    let mut remaining: i64 = match timeout {
        0 => 0,
        t if t < 0 => i64::MAX, // indefinite
        t => (t as i64) * SPIN_PER_MS as i64,
    };

    loop {
        let n = drain_ready(slot, events, maxevents as usize);
        if n > 0 {
            return n as i64;
        }
        if remaining <= 0 {
            return 0; // timed out with no events
        }
        // yield/wait hint — lets other cooperative tasks run and is a
        // no-op on bare silicon if we're alone.
        cooperative_yield();
        remaining -= 1;
    }
}

// ─────────────────────── Ready-list delivery ───────────────────────

/// Scan an instance's interest list, deliver up to `max` ready events
/// into the user's buffer, clear level-triggered ready bits as they
/// are consumed, and return how many events we wrote.
fn drain_ready(slot: usize, events_ptr: *mut EpollEvent, max: usize) -> usize {
    let inst = match get_instance_mut(slot) {
        Some(i) => i,
        None => return 0,
    };

    let mut written = 0usize;
    for entry in inst.interests.iter_mut() {
        if written >= max {
            break;
        }
        if !entry.used {
            continue;
        }
        if entry.oneshot_fired {
            continue;
        }
        // Intersect pending with watched. EPOLLERR/HUP are always
        // reported regardless of whether the user asked.
        let deliverable = entry.ready & (entry.events | EPOLLERR | EPOLLHUP);
        if deliverable == 0 {
            continue;
        }

        // Copy the event out. Use unaligned writes because
        // struct epoll_event is packed on Linux.
        unsafe {
            let out = events_ptr.add(written);
            core::ptr::write_unaligned(core::ptr::addr_of_mut!((*out).events), deliverable);
            core::ptr::write_unaligned(core::ptr::addr_of_mut!((*out).data), entry.data);
        }
        written += 1;

        // Level-triggered: clear the bits we just delivered so they
        // won't fire again until mark_ready is called anew.
        //
        // Edge-triggered (EPOLLET): leave `ready` alone — the caller is
        // expected to drain the underlying fd to EAGAIN. This is a
        // simplification; true ET semantics only re-fire on a fresh
        // transition. See TODO below.
        if entry.events & EPOLLET == 0 {
            entry.ready &= !deliverable;
        }

        // One-shot: suppress further deliveries until EPOLL_CTL_MOD.
        if entry.events & EPOLLONESHOT != 0 {
            entry.oneshot_fired = true;
        }
    }

    written
}

// ─────────────────────── Readiness notification ───────────────────────

/// Signal that `fd` has become ready for `new_events`. Walks every
/// active epoll instance and ORs the relevant bits into any interest
/// that watches this fd. Call this from:
///   * the socket layer when data arrives / send buffer drains
///   * the pipe layer on write/read
///   * the timer subsystem when a timerfd fires
///   * close paths (to deliver EPOLLHUP)
///
/// Safe to call at any time; it never blocks.
pub fn mark_ready(fd: i32, new_events: u32) {
    if fd < 0 || new_events == 0 {
        return;
    }
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for inst in table.iter_mut() {
            if !inst.in_use.load(Ordering::Acquire) {
                continue;
            }
            for entry in inst.interests.iter_mut() {
                if entry.used && entry.fd == fd {
                    entry.ready |= new_events;
                }
            }
        }
    }
}

/// Symmetric helper for "this fd is no longer ready for X" — used when
/// a socket's recv buffer drains to empty or send buffer fills up.
/// The caller passes the bits to CLEAR, not the new state.
pub fn clear_ready(fd: i32, clear_events: u32) {
    if fd < 0 || clear_events == 0 {
        return;
    }
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for inst in table.iter_mut() {
            if !inst.in_use.load(Ordering::Acquire) {
                continue;
            }
            for entry in inst.interests.iter_mut() {
                if entry.used && entry.fd == fd {
                    entry.ready &= !clear_events;
                }
            }
        }
    }
}

/// Called from sys_close on ANY fd (not just epoll fds) so that we can
/// prune dangling interests pointing at the just-closed fd. Otherwise a
/// future fd reuse would see spurious EPOLLIN.
pub fn notify_fd_closed(fd: i32) {
    if fd < 0 {
        return;
    }
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for inst in table.iter_mut() {
            if !inst.in_use.load(Ordering::Acquire) {
                continue;
            }
            for entry in inst.interests.iter_mut() {
                if entry.used && entry.fd == fd {
                    *entry = Interest::empty();
                }
            }
        }
    }
}

// ─────────────────────── Low-level helpers ───────────────────────

/// Cooperative yield hint for spin-wait loops. On ARM64 `wfe` parks the
/// core until an event; on single-core bare metal with no other tasks
/// it acts as a cheap power-save. Replace with a real scheduler hook
/// once the BatCave runner has one.
#[inline(always)]
fn cooperative_yield() {
    unsafe {
        core::arch::asm!("yield", options(nomem, nostack, preserves_flags));
    }
}

// ─────────────────────── Debug / introspection ───────────────────────

/// Count how many interests an instance currently holds. Useful for
/// the BatCave status dump and for tests.
pub fn interest_count(epfd: i32) -> Option<usize> {
    let slot = instance_slot_for_fd(epfd)?;
    let inst = get_instance_mut(slot)?;
    Some(inst.interests.iter().filter(|e| e.used).count())
}

/// Count the number of currently-allocated epoll instances across all
/// caves. For telemetry / leak detection.
pub fn active_instance_count() -> usize {
    let mut n = 0;
    unsafe {
        let table = &*core::ptr::addr_of!(INSTANCES);
        for inst in table.iter() {
            if inst.in_use.load(Ordering::Acquire) {
                n += 1;
            }
        }
    }
    n
}

// ─────────────────────── Known limitations / TODOs ───────────────────────
//
// TODO(kaden): integrate `mark_ready` with:
//   - src/net/tcp.rs recv/send paths          → EPOLLIN / EPOLLOUT
//   - src/batcave/linux/vfs.rs pipe support   → EPOLLIN / EPOLLHUP
//   - a future timerfd implementation         → EPOLLIN
//
// TODO(kaden): `notify_fd_closed(fd)` must be called from sys_close in
// syscall.rs BEFORE fd::close() runs, otherwise we leak stale interests.
//
// TODO(kaden): proper timeout support. Today we spin-count; once the
// kernel grows a monotonic clock hook, read it at entry + compute a
// real deadline instead of the SPIN_PER_MS heuristic.
//
// TODO(kaden): true edge-triggered semantics. Right now EPOLLET only
// suppresses the post-delivery `ready` clear; a conformant impl also
// tracks "last reported level" and only re-fires on a 0→1 transition.
//
// TODO(kaden): EPOLLEXCLUSIVE, EPOLLWAKEUP — Chromium doesn't use them
// but glibc's header defines them.
//
// TODO(kaden): nested epoll (epoll fds watching other epoll fds). The
// browser's IPC layer sometimes does this. Would need a cycle check
// in EPOLL_CTL_ADD.
//
// TODO(kaden): signal mask handling in epoll_pwait once Bat_OS has
// signals. Until then pwait == wait.
