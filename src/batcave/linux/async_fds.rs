// Bat_OS — eventfd + timerfd for BatCave Linux
//
// Used by the Chromium port for cross-thread wakeups (eventfd) and
// animation / scheduler timers (timerfd). Both are file-descriptor-based
// primitives that become readable when a condition is met, so they plug
// into the same epoll/poll machinery as sockets.
//
// Design overview:
//   * Static arrays — no heap. 64 eventfds, 64 timerfds.
//   * Each slot holds an atomic u64 counter plus flags / clock bookkeeping.
//   * Reads are non-blocking-or-busy-wait. Since BatCave is single-threaded
//     with cooperative yielding, "block" here means: return EAGAIN if
//     NONBLOCK, otherwise the caller is expected to yield and retry
//     (wired up by the epoll layer in syscall.rs — not done here).
//   * Monotonic time is read from cntpct_el0 / cntfrq_el0, matching the
//     sys_clock_gettime implementation in syscall.rs.
//
// This module is self-contained — syscall.rs integration is deliberately
// left out. The integration side will wrap these in Fd-table entries.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU64, Ordering};

// ─── Public flag constants (match Linux ABI) ───────────────────────────────

pub const EFD_CLOEXEC: i32   = 0o2000000;
pub const EFD_NONBLOCK: i32  = 0o4000;
pub const EFD_SEMAPHORE: i32 = 0o1;

pub const TFD_CLOEXEC: i32   = 0o2000000;
pub const TFD_NONBLOCK: i32  = 0o4000;

pub const CLOCK_REALTIME: i32  = 0;
pub const CLOCK_MONOTONIC: i32 = 1;

pub const TFD_TIMER_ABSTIME: i32 = 1 << 0;

// errno values (negated on return)
const EAGAIN: i64 = -11;
const EBADF: i64  = -9;
const EFAULT: i64 = -14;
const EINVAL: i64 = -22;
const EMFILE: i64 = -24;

// ─── Time types ────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec:  i64,
    pub tv_nsec: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Itimerspec {
    pub it_interval: Timespec,
    pub it_value:    Timespec,
}

impl Timespec {
    #[inline]
    fn to_ns(&self) -> i128 {
        (self.tv_sec as i128) * 1_000_000_000 + (self.tv_nsec as i128)
    }
    #[inline]
    fn from_ns(ns: i128) -> Self {
        let ns = if ns < 0 { 0 } else { ns };
        Timespec {
            tv_sec:  (ns / 1_000_000_000) as i64,
            tv_nsec: (ns % 1_000_000_000) as i64,
        }
    }
    #[inline]
    fn is_zero(&self) -> bool { self.tv_sec == 0 && self.tv_nsec == 0 }
}

// ─── Monotonic clock (ARM64 generic timer) ─────────────────────────────────

/// Read cntpct_el0 / cntfrq_el0 and return nanoseconds since boot.
/// Matches sys_clock_gettime in syscall.rs.
fn now_ns() -> i128 {
    let count: u64;
    let freq:  u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { return 0; }
    let secs  = count / freq;
    let nsecs = ((count % freq) * 1_000_000_000) / freq;
    (secs as i128) * 1_000_000_000 + (nsecs as i128)
}

// ─── Eventfd ───────────────────────────────────────────────────────────────

pub const MAX_EVENTFDS: usize = 64;
pub const MAX_TIMERFDS: usize = 64;

pub struct EventfdState {
    pub in_use:  AtomicBool,
    pub counter: AtomicU64,
    pub flags:   AtomicI32,
    // Each state is externally addressed by its slot index; the owning fd
    // number lives in the fd-table entry that points here.
}

impl EventfdState {
    const fn new() -> Self {
        EventfdState {
            in_use:  AtomicBool::new(false),
            counter: AtomicU64::new(0),
            flags:   AtomicI32::new(0),
        }
    }
}

static EVENTFDS: [EventfdState; MAX_EVENTFDS] = {
    const S: EventfdState = EventfdState::new();
    [S; MAX_EVENTFDS]
};

/// Allocate an eventfd slot. Returns the slot index, not an fd number.
/// The caller (syscall integration) is responsible for wiring the slot
/// into the process fd table.
pub fn alloc_eventfd_slot(initval: u32, flags: i32) -> Result<usize, i64> {
    // Validate flags — unknown bits => EINVAL, like Linux.
    let known = EFD_CLOEXEC | EFD_NONBLOCK | EFD_SEMAPHORE;
    if flags & !known != 0 { return Err(EINVAL); }

    for i in 0..MAX_EVENTFDS {
        if EVENTFDS[i]
            .in_use
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            EVENTFDS[i].counter.store(initval as u64, Ordering::Release);
            EVENTFDS[i].flags.store(flags, Ordering::Release);
            return Ok(i);
        }
    }
    Err(EMFILE)
}

/// Free an eventfd slot (called when the fd is closed).
pub fn free_eventfd_slot(slot: usize) -> Result<(), i64> {
    if slot >= MAX_EVENTFDS { return Err(EBADF); }
    EVENTFDS[slot].counter.store(0, Ordering::Release);
    EVENTFDS[slot].flags.store(0, Ordering::Release);
    EVENTFDS[slot].in_use.store(false, Ordering::Release);
    Ok(())
}

fn eventfd_slot(slot: usize) -> Result<&'static EventfdState, i64> {
    if slot >= MAX_EVENTFDS { return Err(EBADF); }
    if !EVENTFDS[slot].in_use.load(Ordering::Acquire) { return Err(EBADF); }
    Ok(&EVENTFDS[slot])
}

/// Public eventfd2 — required API signature.
/// Returns the slot index (cast to i64) on success, or -errno.
/// Final fd-number allocation happens in the syscall layer.
pub fn eventfd2(initval: u32, flags: i32) -> i64 {
    // Eventfd lives in two layers:
    //   1. The slot in EVENTFDS holds the counter + flags.
    //   2. The fd-table entry (FdKind::Eventfd) carries the user-visible
    //      fd number that points back at the slot. Without (2), Chromium's
    //      epoll_ctl(EPOLL_CTL_ADD, fd=<slot>) would fail with EBADF
    //      because fd::get(slot) returns None.
    let slot = match alloc_eventfd_slot(initval, flags) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if slot > u16::MAX as usize {
        let _ = free_eventfd_slot(slot);
        return EMFILE;
    }
    // Translate EFD_CLOEXEC / EFD_NONBLOCK into the fd-table flags.
    // We don't currently key off these inside the fd table, but
    // recording them keeps fcntl(F_GETFL/F_SETFL) coherent.
    let mut fd_flags: u32 = 0;
    if flags & EFD_NONBLOCK != 0 { fd_flags |= 0o4000; } // O_NONBLOCK
    if flags & EFD_CLOEXEC  != 0 { fd_flags |= 0o2000000; } // O_CLOEXEC
    match crate::batcave::linux::fd::alloc_fd_eventfd(slot as u16, fd_flags) {
        Ok(fd) => fd as i64,
        Err(e) => {
            let _ = free_eventfd_slot(slot);
            e
        }
    }
}

/// Core eventfd read (slot-indexed). Returns bytes read (8) or -errno.
///
/// Semantics:
///   * counter == 0 => EAGAIN if NONBLOCK, else the caller is expected
///     to yield-and-retry (we don't have a wait queue here).
///   * EFD_SEMAPHORE: decrement by 1, return 1.
///   * otherwise:     return current counter, clear to 0.
pub fn eventfd_read_slot(slot: usize, out_value: *mut u64) -> i64 {
    let s = match eventfd_slot(slot) { Ok(s) => s, Err(e) => return e };
    if out_value.is_null() { return EFAULT; }
    // V8-ROOT-8 / V8-PTR-003: gate out_value (8-byte kernel write otherwise).
    if !crate::batcave::linux::uaccess::is_user_range(out_value as usize, 8) {
        return EFAULT;
    }

    let flags = s.flags.load(Ordering::Acquire);
    let semaphore = (flags & EFD_SEMAPHORE) != 0;
    let nonblock  = (flags & EFD_NONBLOCK)  != 0;

    loop {
        let cur = s.counter.load(Ordering::Acquire);
        if cur == 0 {
            if nonblock { return EAGAIN; }
            // No wait queue — single-threaded cooperative. Signal EAGAIN
            // so the caller (epoll / scheduler) can reschedule. The real
            // "block" comes from epoll_wait; direct blocking reads without
            // NONBLOCK simply busy-spin one iteration and return EAGAIN.
            return EAGAIN;
        }

        let (new_val, returned) = if semaphore { (cur - 1, 1u64) } else { (0u64, cur) };
        if s.counter
            .compare_exchange(cur, new_val, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            unsafe { core::ptr::write_volatile(out_value, returned); }
            return 8;
        }
        // else: contention — retry
    }
}

/// Core eventfd write. Adds `value` to counter. Returns 8 or -errno.
/// Linux rejects u64::MAX and overflow. If counter would exceed u64::MAX - 1,
/// return EAGAIN (matches Linux semantics for overflow without blocking).
pub fn eventfd_write_slot(slot: usize, value: u64) -> i64 {
    let s = match eventfd_slot(slot) { Ok(s) => s, Err(e) => return e };
    if value == u64::MAX { return EINVAL; }

    loop {
        let cur = s.counter.load(Ordering::Acquire);
        let new_val = match cur.checked_add(value) {
            Some(v) if v < u64::MAX => v,
            _ => return EAGAIN, // would overflow; Linux returns EAGAIN
        };
        if s
            .counter
            .compare_exchange(cur, new_val, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return 8;
        }
    }
}

/// Public eventfd_read — writes counter value to `*value`, returns 0 or -errno.
/// NOTE: the caller passes an fd number, which must be translated to a slot
/// index by the syscall layer. The signature here follows the task spec.
pub fn eventfd_read(fd: i32, value: *mut u64) -> i64 {
    if fd < 0 { return EBADF; }
    let r = eventfd_read_slot(fd as usize, value);
    if r < 0 { r } else { 0 }
}

/// Public eventfd_write — adds `value` to the counter. Returns 0 or -errno.
pub fn eventfd_write(fd: i32, value: u64) -> i64 {
    if fd < 0 { return EBADF; }
    let r = eventfd_write_slot(fd as usize, value);
    if r < 0 { r } else { 0 }
}

/// Poll helper — is this eventfd slot readable right now?
pub fn eventfd_is_readable(slot: usize) -> bool {
    match eventfd_slot(slot) {
        Ok(s)  => s.counter.load(Ordering::Acquire) > 0,
        Err(_) => false,
    }
}

/// Poll helper — is this eventfd slot writable right now?
/// True as long as counter < u64::MAX - 1.
pub fn eventfd_is_writable(slot: usize) -> bool {
    match eventfd_slot(slot) {
        Ok(s)  => s.counter.load(Ordering::Acquire) < u64::MAX - 1,
        Err(_) => false,
    }
}

// ─── Timerfd ───────────────────────────────────────────────────────────────

pub struct TimerfdState {
    pub in_use:      AtomicBool,
    pub clock_id:    AtomicI32,
    pub flags:       AtomicI32,
    pub expires_at:  AtomicI64,   // monotonic-ns deadline; 0 = disarmed
    pub interval_ns: AtomicI64,   // 0 = one-shot
    pub counter:     AtomicU64,   // number of pending expirations
}

impl TimerfdState {
    const fn new() -> Self {
        TimerfdState {
            in_use:      AtomicBool::new(false),
            clock_id:    AtomicI32::new(0),
            flags:       AtomicI32::new(0),
            expires_at:  AtomicI64::new(0),
            interval_ns: AtomicI64::new(0),
            counter:     AtomicU64::new(0),
        }
    }
}

static TIMERFDS: [TimerfdState; MAX_TIMERFDS] = {
    const T: TimerfdState = TimerfdState::new();
    [T; MAX_TIMERFDS]
};

/// Allocate a timerfd slot.
pub fn alloc_timerfd_slot(clockid: i32, flags: i32) -> Result<usize, i64> {
    if clockid != CLOCK_REALTIME && clockid != CLOCK_MONOTONIC {
        return Err(EINVAL);
    }
    let known = TFD_CLOEXEC | TFD_NONBLOCK;
    if flags & !known != 0 { return Err(EINVAL); }

    for i in 0..MAX_TIMERFDS {
        if TIMERFDS[i]
            .in_use
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            TIMERFDS[i].clock_id.store(clockid, Ordering::Release);
            TIMERFDS[i].flags.store(flags, Ordering::Release);
            TIMERFDS[i].expires_at.store(0, Ordering::Release);
            TIMERFDS[i].interval_ns.store(0, Ordering::Release);
            TIMERFDS[i].counter.store(0, Ordering::Release);
            return Ok(i);
        }
    }
    Err(EMFILE)
}

/// Free a timerfd slot (called from close()).
pub fn free_timerfd_slot(slot: usize) -> Result<(), i64> {
    if slot >= MAX_TIMERFDS { return Err(EBADF); }
    TIMERFDS[slot].expires_at.store(0, Ordering::Release);
    TIMERFDS[slot].interval_ns.store(0, Ordering::Release);
    TIMERFDS[slot].counter.store(0, Ordering::Release);
    TIMERFDS[slot].flags.store(0, Ordering::Release);
    TIMERFDS[slot].in_use.store(false, Ordering::Release);
    Ok(())
}

fn timerfd_slot(slot: usize) -> Result<&'static TimerfdState, i64> {
    if slot >= MAX_TIMERFDS { return Err(EBADF); }
    if !TIMERFDS[slot].in_use.load(Ordering::Acquire) { return Err(EBADF); }
    Ok(&TIMERFDS[slot])
}

/// Public timerfd_create — returns a real fd whose `FdKind::Timerfd`
/// tag points at the slot. Same two-layer structure as eventfd2 (see
/// the long comment in `eventfd2` above).
pub fn timerfd_create(clockid: i32, flags: i32) -> i64 {
    let slot = match alloc_timerfd_slot(clockid, flags) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if slot > u16::MAX as usize {
        let _ = free_timerfd_slot(slot);
        return EMFILE;
    }
    let mut fd_flags: u32 = 0;
    if flags & TFD_NONBLOCK != 0 { fd_flags |= 0o4000; } // O_NONBLOCK
    if flags & TFD_CLOEXEC  != 0 { fd_flags |= 0o2000000; } // O_CLOEXEC
    match crate::batcave::linux::fd::alloc_fd_timerfd(slot as u16, fd_flags) {
        Ok(fd) => fd as i64,
        Err(e) => {
            let _ = free_timerfd_slot(slot);
            e
        }
    }
}

/// Sweep the timer: if expires_at has passed, increment counter by the number
/// of elapsed intervals, and advance expires_at by a multiple of interval_ns.
/// Called lazily on every read / poll / gettime — no real timer IRQ yet.
fn sweep(s: &TimerfdState) {
    let expires = s.expires_at.load(Ordering::Acquire);
    if expires == 0 { return; } // disarmed

    let now = now_ns();
    if (now as i128) < (expires as i128) { return; }

    let interval = s.interval_ns.load(Ordering::Acquire);
    if interval <= 0 {
        // one-shot — fire exactly once and disarm.
        s.counter.fetch_add(1, Ordering::AcqRel);
        s.expires_at.store(0, Ordering::Release);
        return;
    }

    // periodic — count how many intervals elapsed, advance deadline.
    let elapsed  = (now as i128) - (expires as i128);
    let extra    = (elapsed / (interval as i128)) as u64 + 1;
    let new_exp  = (expires as i128) + (extra as i128) * (interval as i128);
    s.counter.fetch_add(extra, Ordering::AcqRel);
    s.expires_at.store(new_exp as i64, Ordering::Release);
}

/// Public timerfd_settime.
///
/// * `flags` may include TFD_TIMER_ABSTIME — if set, new_value.it_value is
///   an absolute deadline on the configured clock. Otherwise relative.
/// * Writes the previous setting to `old_value` if non-null.
/// * If new_value.it_value == {0,0}, disarms the timer.
pub fn timerfd_settime(
    fd: i32,
    flags: i32,
    new_value: *const Itimerspec,
    old_value: *mut Itimerspec,
) -> i64 {
    if fd < 0 { return EBADF; }
    // Translate fd → slot via the per-cave FD table (the user-visible
    // fd is a real file descriptor whose FdKind tag carries the slot
    // index). Falls back to interpreting `fd` as a raw slot index for
    // backward-compat with pre-bridge call sites that still hand us
    // slot numbers — Chromium goes through the bridge, so this fallback
    // path is only for in-kernel callers.
    let slot_idx = match crate::batcave::linux::fd::timerfd_slot(fd as u32) {
        Some(s) => s,
        None => fd as usize,
    };
    let s = match timerfd_slot(slot_idx) { Ok(s) => s, Err(e) => return e };
    if new_value.is_null() { return EFAULT; }

    // V8-ROOT-8 / V8-PTR-003: gate both pointers before any deref. Itimerspec
    // is 32 bytes. Without this, a cave passing `new_value=kptr` got a
    // 32-byte kernel read; `old_value=kptr` got a 32-byte kernel write.
    let itspec_size = core::mem::size_of::<Itimerspec>();
    if !crate::batcave::linux::uaccess::is_user_range(new_value as usize, itspec_size) {
        return EFAULT;
    }
    if !old_value.is_null()
        && !crate::batcave::linux::uaccess::is_user_range(old_value as usize, itspec_size)
    {
        return EFAULT;
    }

    // Sweep first so the "old_value" we report is consistent.
    sweep(s);

    // Capture the old setting for caller.
    let old_expires  = s.expires_at.load(Ordering::Acquire);
    let old_interval = s.interval_ns.load(Ordering::Acquire);

    let nv: Itimerspec = unsafe { core::ptr::read_volatile(new_value) };

    // Validate tv_nsec range.
    if nv.it_value.tv_nsec < 0    || nv.it_value.tv_nsec    >= 1_000_000_000
    || nv.it_interval.tv_nsec < 0 || nv.it_interval.tv_nsec >= 1_000_000_000
    || nv.it_value.tv_sec < 0     || nv.it_interval.tv_sec  < 0
    {
        return EINVAL;
    }

    let interval_ns = nv.it_interval.to_ns() as i64;

    let new_expires: i64 = if nv.it_value.is_zero() {
        0 // disarm
    } else if (flags & TFD_TIMER_ABSTIME) != 0 {
        nv.it_value.to_ns() as i64
    } else {
        // relative — add to current monotonic time.
        // (For CLOCK_REALTIME we also use monotonic here since we don't have
        //  a wall-clock source yet; this is a known simplification.)
        (now_ns() + nv.it_value.to_ns()) as i64
    };

    s.interval_ns.store(interval_ns, Ordering::Release);
    s.expires_at.store(new_expires, Ordering::Release);
    // Arming a timer clears any prior pending expirations, per Linux.
    s.counter.store(0, Ordering::Release);

    if !old_value.is_null() {
        let now = now_ns();
        let remaining = if old_expires == 0 {
            Timespec::default()
        } else {
            Timespec::from_ns((old_expires as i128) - now)
        };
        let old = Itimerspec {
            it_interval: Timespec::from_ns(old_interval as i128),
            it_value:    remaining,
        };
        unsafe { core::ptr::write_volatile(old_value, old); }
    }

    0
}

/// Public timerfd_gettime. Writes remaining time + interval.
pub fn timerfd_gettime(fd: i32, curr_value: *mut Itimerspec) -> i64 {
    if fd < 0 { return EBADF; }
    let slot_idx = match crate::batcave::linux::fd::timerfd_slot(fd as u32) {
        Some(s) => s,
        None => fd as usize,
    };
    let s = match timerfd_slot(slot_idx) { Ok(s) => s, Err(e) => return e };
    if curr_value.is_null() { return EFAULT; }
    // V8-ROOT-8 / V8-PTR-003: gate curr_value (32-byte write to attacker ptr).
    if !crate::batcave::linux::uaccess::is_user_range(
        curr_value as usize, core::mem::size_of::<Itimerspec>())
    {
        return EFAULT;
    }

    sweep(s);

    let expires  = s.expires_at.load(Ordering::Acquire);
    let interval = s.interval_ns.load(Ordering::Acquire);
    let now      = now_ns();

    let remaining = if expires == 0 {
        Timespec::default()
    } else {
        Timespec::from_ns((expires as i128) - now)
    };

    let out = Itimerspec {
        it_interval: Timespec::from_ns(interval as i128),
        it_value:    remaining,
    };
    unsafe { core::ptr::write_volatile(curr_value, out); }
    0
}

/// Core timerfd read: returns the number of expirations as a u64 and clears
/// the counter. Blocks (returns EAGAIN here) if counter is 0.
pub fn timerfd_read_slot(slot: usize, out_value: *mut u64) -> i64 {
    let s = match timerfd_slot(slot) { Ok(s) => s, Err(e) => return e };
    if out_value.is_null() { return EFAULT; }

    sweep(s);

    let flags    = s.flags.load(Ordering::Acquire);
    let nonblock = (flags & TFD_NONBLOCK) != 0;

    loop {
        let cur = s.counter.load(Ordering::Acquire);
        if cur == 0 {
            if nonblock { return EAGAIN; }
            return EAGAIN; // cooperative: caller yields via epoll
        }
        if s
            .counter
            .compare_exchange(cur, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            unsafe { core::ptr::write_volatile(out_value, cur); }
            return 8;
        }
    }
}

/// Poll helper for timerfd readability — also drives the lazy sweep so
/// epoll_wait pollers see expirations without needing a timer IRQ.
pub fn timerfd_is_readable(slot: usize) -> bool {
    match timerfd_slot(slot) {
        Ok(s) => { sweep(s); s.counter.load(Ordering::Acquire) > 0 }
        Err(_) => false,
    }
}

/// Next absolute expiry in monotonic-ns, or None if disarmed.
/// The scheduler can use this to compute the next epoll_wait timeout.
pub fn timerfd_next_expiry(slot: usize) -> Option<i64> {
    let s = timerfd_slot(slot).ok()?;
    let e = s.expires_at.load(Ordering::Acquire);
    if e == 0 { None } else { Some(e) }
}

/// V8-ROOT-2: drop every async-fd slot on cave switch. Without this, a new
/// cave's fd table reuse can inherit a previous cave's pending eventfd
/// count or timerfd expiration — event-smuggling across the isolation
/// boundary.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    for s in EVENTFDS.iter() {
        s.in_use.store(false, Ordering::Release);
        s.counter.store(0, Ordering::Relaxed);
        s.flags.store(0, Ordering::Relaxed);
    }
    for s in TIMERFDS.iter() {
        s.in_use.store(false, Ordering::Release);
        s.counter.store(0, Ordering::Relaxed);
        s.flags.store(0, Ordering::Relaxed);
        s.expires_at.store(0, Ordering::Relaxed);
        s.interval_ns.store(0, Ordering::Relaxed);
    }
}

// ─── Silence unused-field warnings for types the integrator will consume ───

#[allow(dead_code)]
fn _unused_refs() {
    let _ = AtomicU32::new(0); // placeholder to keep the import used
}
