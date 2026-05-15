#![allow(dead_code)]
// Sphragis — TCP Layer (Multi-PCB)
// Supports up to 64 concurrent TCP connections via a static TCB (PCB) table.
// Designed to unblock Chromium's subresource fetch parallelism.
//
// Design summary
// * Replace the single `CONN_STATE` / `REMOTE_IP` / ... globals with a
// fixed-size [TcpPcb; MAX_PCBS] table. No heap.
// * PCB fields are plain non-atomic where they are only mutated under the
// single-threaded packet-dispatch & connect path. Shared-visibility flags
// (state, in_use, error, nonblocking, ring head/tail) use atomics so that
// epoll/poll callers and the RX dispatch path can coordinate.
// * Non-blocking connect: `connect_start()` + `connect_poll()`. The legacy
// synchronous `connect()` wraps these using PCB 0 so the existing
// netsurf_test keeps working.
// * Per-PCB rx/tx rings (8 KiB each) — enough for one in-flight TCP segment
// with window room; Chromium's HTTP/1.1 pipelines fit.
// * epoll integration: when rx_buf gains data, the PCB transitions to
// ESTABLISHED, or tx_buf drains, we call `epoll::mark_ready(fd, ...)`.
// * Half-close: `shutdown_write` sends a FIN but keeps RX open;
// `shutdown_read` drops further incoming data.
// * State machine: full 11-state table with the transitions noted in the
// task. TIME_WAIT 2MSL timeout is a best-effort counter, not wall-clock.

use super::ip::{self, IpPacket};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ISN randomization (ATTACK-NET-009)
//
// The previous scheme was `1000 + pcb_id * 997`, which is deterministic and
// allows blind RST / data injection as soon as the attacker observes a single
// handshake. We now derive the ISN from
// hash(cntpct_el0 ^ remote_ip ^ remote_port ^ pcb_id ^ boot_cookie)
// where boot_cookie is a one-shot 64-bit value seeded from the timer the
// first time it is read. This is still not RFC 6528 (needs a real CSPRNG
// and per-4-tuple hash key) but it defeats the "read one ISN, predict all
// future ISNs" primitive.
static BOOT_COOKIE: AtomicU64 = AtomicU64::new(0);

// RFC 5961 challenge-ACK rate limit. System-wide cap of 100 per second to
// prevent an attacker from using us as a reflection source.
static CHAL_ACK_WINDOW_START: AtomicU64 = AtomicU64::new(0);
static CHAL_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
const CHAL_ACK_LIMIT_PER_SEC: u32 = 100;

/// Returns true iff we should emit a challenge ACK right now. Uses the
/// ARM generic timer for wall-clock timing and resets the window on
/// every second boundary.
fn try_challenge_ack() -> bool {
    let now: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let window_start = CHAL_ACK_WINDOW_START.load(Ordering::Relaxed);
    if now.saturating_sub(window_start) > freq {
        // New 1-second window.
        CHAL_ACK_WINDOW_START.store(now, Ordering::Relaxed);
        CHAL_ACK_COUNT.store(1, Ordering::Relaxed);
        return true;
    }
    let n = CHAL_ACK_COUNT.fetch_add(1, Ordering::Relaxed);
    n < CHAL_ACK_LIMIT_PER_SEC
}

/// Is `seq` within the inclusive-start, exclusive-end window
/// [rcv_nxt, rcv_nxt + wnd)? Handles u32 wraparound.
fn seq_in_window(seq: u32, rcv_nxt: u32, wnd: u32) -> bool {
    if wnd == 0 { return seq == rcv_nxt; }
    let offset = seq.wrapping_sub(rcv_nxt);
    offset < wnd
}

fn boot_cookie() -> u64 {
    let existing = BOOT_COOKIE.load(Ordering::Relaxed);
    if existing != 0 { return existing; }
    let t1: u64;
    let t2: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) t1);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) t2);
    }
    let mixed = t1 ^ t2.rotate_left(17) ^ freq.rotate_left(31) ^ 0xA5A5_5A5A_C3C3_3C3C;
    // Make sure we never return 0 (else we'd re-seed every call).
    let v = if mixed == 0 { 0xDEAD_BEEF_CAFE_BABE } else { mixed };
    // Race is benign — whichever writer wins, the result is still unique
    // per boot and unpredictable to an off-path attacker.
    BOOT_COOKIE.store(v, Ordering::Relaxed);
    v
}

fn compute_isn(pcb_id: usize, remote_ip: u32, remote_port: u16) -> u32 {
    let ticks: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) ticks); }
    let cookie = boot_cookie();
    let mut h: u64 = cookie
        ^ ticks
        ^ (remote_ip as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (remote_port as u64).wrapping_mul(0xBF58476D1CE4E5B9)
        ^ (pcb_id as u64).wrapping_mul(0x94D049BB133111EB);
    // Splittable-random-style mixing (SplitMix64 finalizer).
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D049BB133111EB);
    h ^= h >> 31;
    h as u32
}

// Constants

const TCP_HDR_SIZE: usize = 20;
pub const TCP_FIN: u8 = 0x01;
pub const TCP_SYN: u8 = 0x02;
pub const TCP_RST: u8 = 0x04;
pub const TCP_PSH: u8 = 0x08;
pub const TCP_ACK: u8 = 0x10;

// TCP states — matches RFC 793 naming, encoded in AtomicU32.
pub const STATE_CLOSED:      u32 = 0;
pub const STATE_LISTEN:      u32 = 1;
pub const STATE_SYN_SENT:    u32 = 2;
pub const STATE_SYN_RECEIVED:u32 = 3;
pub const STATE_ESTABLISHED: u32 = 4;
pub const STATE_FIN_WAIT_1:  u32 = 5;
pub const STATE_FIN_WAIT_2:  u32 = 6;
pub const STATE_CLOSE_WAIT:  u32 = 7;
pub const STATE_CLOSING:     u32 = 8;
pub const STATE_LAST_ACK:    u32 = 9;
pub const STATE_TIME_WAIT:   u32 = 10;

// epoll event bits (copy here to avoid pulling the whole module on no_std path).
const EPOLLIN:  u32 = 0x001;
const EPOLLOUT: u32 = 0x004;
const EPOLLERR: u32 = 0x008;
const EPOLLHUP: u32 = 0x010;

// errno (positive; socket bridge negates).
pub const E_OK:          i32 = 0;
pub const E_INPROGRESS:  i32 = 115;
pub const E_AGAIN:       i32 = 11;
pub const E_CONNRESET:   i32 = 104;
pub const E_CONNREFUSED: i32 = 111;
pub const E_TIMEDOUT:    i32 = 110;
pub const E_NOTCONN:     i32 = 107;
pub const E_PIPE:        i32 = 32;
pub const E_NETUNREACH:  i32 = 101;

pub const MAX_PCBS: usize = 64;
pub const RX_BUF_SIZE: usize = 8192;
pub const TX_BUF_SIZE: usize = 8192;

// Legacy synchronous API uses slot 0.
const LEGACY_PCB: usize = 0;

/// Ephemeral local-port allocator. Starts at 49152 and wraps around 65535.
// /
/// previously a single `AtomicU32::fetch_add(1)`. On Apple
/// Silicon (and on QEMU/HVF here) the kernel runs with the MMU
/// disabled, so the backing memory is treated as Device-nGnRnE and
/// LDXR/STXR have unpredictable behavior — STXR always fails, so
/// `fetch_add` busy-spins forever inside its compare-exchange-weak
/// retry loop. Same family of bug as the heap allocator's switch
/// from `spin::Mutex` to manual IRQ masking. Single-CPU bring-up
/// means we don't need the atomic; mask IRQs and read/write directly.
static mut NEXT_LOCAL_PORT: u16 = 49152;

#[inline(always)]
unsafe fn irq_save() -> u64 {
    let prev: u64;
    unsafe {
        core::arch::asm!(
            "mrs {p}, daif",
            "msr daifset, #0x2",
            p = out(reg) prev,
            options(nostack, preserves_flags),
        );
    }
    prev
}

#[inline(always)]
unsafe fn irq_restore(prev: u64) {
    unsafe {
        core::arch::asm!("msr daif, {p}", p = in(reg) prev,
            options(nostack, preserves_flags));
    }
}

fn alloc_local_port() -> u16 {
    unsafe {
        let saved = irq_save();
        let p = core::ptr::addr_of_mut!(NEXT_LOCAL_PORT);
        let port = core::ptr::read_volatile(p);
        let next = if port == 65535 || port < 49152 { 49152 } else { port + 1 };
        core::ptr::write_volatile(p, next);
        irq_restore(saved);
        port
    }
}

// PCB (TCB)

/// Per-connection TCP control block.
pub struct TcpPcb {
    pub state: AtomicU32,

    // Endpoint tuple. `remote_ip` is big-endian; `local_port`/`remote_port`
    // are host byte order on the wire they are swapped into network order
    // inside `send_tcp_pcb`.
    pub local_port:  u16,
    pub remote_ip:   u32,
    pub remote_port: u16,

    // Send sequence variables (RFC 793).
    pub snd_nxt: u32,
    pub snd_una: u32,
    pub snd_wnd: u16,
    // Receive sequence variables.
    pub rcv_nxt: u32,
    pub rcv_wnd: u16,

    pub rx_buf:  [u8; RX_BUF_SIZE],
    pub rx_head: AtomicUsize,  // write index (producer = RX dispatch)
    pub rx_tail: AtomicUsize,  // read  index (consumer = userspace)

    pub tx_buf:  [u8; TX_BUF_SIZE],
    pub tx_head: AtomicUsize,
    pub tx_tail: AtomicUsize,

    pub is_nonblocking: AtomicBool,
    pub error:  AtomicI32,     // most recent async errno
    pub in_use: AtomicBool,

    /// Associated userspace fd (from sockets.rs). -1 when not bound to a
    /// socket (e.g. the legacy direct-API user).
    pub fd: AtomicI32,

    /// RX side shut by shutdown_read(): drop incoming data payload.
    pub rx_shut: AtomicBool,
    /// TX side shut by shutdown_write(): FIN already queued, reject sends.
    pub tx_shut: AtomicBool,

    /// cntpct_el0 tick count when we entered TIME_WAIT. Zero otherwise.
    /// `poll_once` scans all PCBs and transitions TIME_WAIT -> CLOSED
    /// once 30 s (2×MSL shortened) have elapsed.
    pub time_wait_entered: AtomicU64,

    /// for PCBs allocated by the SYN-on-LISTEN handler,
    /// this is the parent listener's slot index in `LISTENERS[]`.
    /// `u16::MAX` means "client-side connection, not from a listener."
    /// Used by step 3's SYN_RECEIVED→ESTABLISHED transition to know
    /// which listener's accept queue to push onto.
    pub parent_listener_idx: AtomicU32,
}

impl TcpPcb {
    const fn empty() -> Self {
        TcpPcb {
            state: AtomicU32::new(STATE_CLOSED),
            local_port: 0,
            remote_ip: 0,
            remote_port: 0,
            snd_nxt: 0,
            snd_una: 0,
            snd_wnd: 8192,
            rcv_nxt: 0,
            rcv_wnd: RX_BUF_SIZE as u16,
            rx_buf: [0u8; RX_BUF_SIZE],
            rx_head: AtomicUsize::new(0),
            rx_tail: AtomicUsize::new(0),
            tx_buf: [0u8; TX_BUF_SIZE],
            tx_head: AtomicUsize::new(0),
            tx_tail: AtomicUsize::new(0),
            is_nonblocking: AtomicBool::new(false),
            error:  AtomicI32::new(0),
            in_use: AtomicBool::new(false),
            fd:     AtomicI32::new(-1),
            rx_shut: AtomicBool::new(false),
            tx_shut: AtomicBool::new(false),
            time_wait_entered: AtomicU64::new(0),
            parent_listener_idx: AtomicU32::new(u32::MAX),
        }
    }

    #[inline]
    fn rx_available(&self) -> usize {
        let head = self.rx_head.load(Ordering::Acquire);
        let tail = self.rx_tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    #[inline]
    fn rx_free(&self) -> usize {
        RX_BUF_SIZE - self.rx_available()
    }
}

static mut TCP_PCBS: [TcpPcb; MAX_PCBS] = [const { TcpPcb::empty() }; MAX_PCBS];

// ─── Server-side TCP — Listener table ─────────────────────
//
// A `Listener` represents a `(local_port, backlog)` pair waiting to
// accept inbound connections. It owns:
// the local port the operator bind()ed to + listen()ed on
// the listening fd (so `epoll::mark_ready` can wake `epoll_wait`)
// the cave that owns it (per-cave isolation when caves close)
// an accept queue: a fixed-size ring of `pcb_id`s for connections
// that completed the 3-way handshake but haven't been `accept()`ed
// yet
// `backlog` capping the accept queue depth
//
// The listener does NOT hold sequence/window state — it's purely an
// accept-rendezvous. SYN_RECEIVED PCBs (allocated by the SYN-on-LISTEN
// handler in `handle_incoming`) carry their own state and reference
// their parent listener via `parent_listener_idx` for the third-ACK
// completion path.
//
// Mixing listeners and connection PCBs in one table would force every
// state-machine branch in `handle_incoming` to handle "LISTEN means
// data fields are garbage" — separate tables keep both clean.

pub const MAX_LISTENERS: usize = 16;
pub const ACCEPT_QUEUE_DEPTH: usize = 16;

pub struct Listener {
    pub in_use: AtomicBool,
    /// Local port we accept on (host byte order).
    pub local_port: u16,
    /// `backlog` from `listen(2)` — caps the accept queue.
    pub backlog: u8,
    /// Cave that owns this listener; cleared on cave-switch.
    pub cave_id: u16,
    /// Listening fd for `epoll_wait` readiness signaling. -1 if no fd.
    pub fd: AtomicI32,
    /// Accept queue: ring of pcb_ids for completed-handshake conns
    /// awaiting `accept()`. Producer: SYN_RECEIVED→ESTABLISHED path.
    /// Consumer: `sys_accept` / `accept_pop`.
    pub accept_q: [u16; ACCEPT_QUEUE_DEPTH],
    pub accept_head: AtomicUsize, // producer index (write)
    pub accept_tail: AtomicUsize, // consumer index (read)
    /// Closing-generation counter. Bumped by `listener_close` so a
    /// concurrent accept() that already popped a pcb_id can detect
    /// the listener went away mid-flight and refund.
    pub close_gen: AtomicU32,
}

impl Listener {
    const fn empty() -> Self {
        Self {
            in_use: AtomicBool::new(false),
            local_port: 0,
            backlog: 0,
            cave_id: 0,
            fd: AtomicI32::new(-1),
            accept_q: [0u16; ACCEPT_QUEUE_DEPTH],
            accept_head: AtomicUsize::new(0),
            accept_tail: AtomicUsize::new(0),
            close_gen: AtomicU32::new(0),
        }
    }

    #[inline]
    fn accept_count(&self) -> usize {
        let h = self.accept_head.load(Ordering::Acquire);
        let t = self.accept_tail.load(Ordering::Acquire);
        h.wrapping_sub(t)
    }
}

static mut LISTENERS: [Listener; MAX_LISTENERS] =
    [const { Listener::empty() }; MAX_LISTENERS];

#[inline]
fn listener(idx: usize) -> &'static Listener {
    unsafe {
        let p = core::ptr::addr_of!(LISTENERS) as *const Listener;
        &*p.add(idx)
    }
}

#[inline]
fn listener_mut(idx: usize) -> &'static mut Listener {
    unsafe {
        let p = core::ptr::addr_of_mut!(LISTENERS) as *mut Listener;
        &mut *p.add(idx)
    }
}

/// Look up a listener by port. Returns the slot index if any active
/// listener owns `local_port`, else None.
pub fn listener_lookup_by_port(local_port: u16) -> Option<usize> {
    for i in 0..MAX_LISTENERS {
        let l = listener(i);
        if l.in_use.load(Ordering::Acquire) && l.local_port == local_port {
            return Some(i);
        }
    }
    None
}

/// Register a new listener for `local_port`. Returns the slot index
/// or one of:
/// `Err("EADDRINUSE")` if any active listener or non-TIME_WAIT
/// PCB already owns this port
/// `Err("EMFILE")` if all MAX_LISTENERS slots are full
// /
/// `backlog` is clamped to `ACCEPT_QUEUE_DEPTH` (Linux's `somaxconn`
/// equivalent for us).
// /
/// Caller (sockets.rs::listen) is responsible for `bind()`ing the
/// fd before calling — we don't validate the cave owns the fd.
pub fn listen_register(
    local_port: u16,
    backlog: i32,
    cave_id: u16,
    fd: i32,
) -> Result<usize, &'static str> {
    if local_port == 0 { return Err("EINVAL: port 0"); }

    alloc_lock();

    // Port collision against existing listeners.
    for i in 0..MAX_LISTENERS {
        let l = listener(i);
        if l.in_use.load(Ordering::Acquire) && l.local_port == local_port {
            alloc_unlock();
            return Err("EADDRINUSE");
        }
    }
    // Port collision against active client connection PCBs that
    // happen to use the same local_port (rare — outbound clients
    // get random ephemeral ports — but possible with explicit bind).
    // TIME_WAIT is excluded so SO_REUSEADDR can land in a future step.
    unsafe {
        for i in 0..MAX_PCBS {
            let p = &TCP_PCBS[i];
            if !p.in_use.load(Ordering::Acquire) { continue; }
            if p.local_port != local_port { continue; }
            let st = p.state.load(Ordering::Acquire);
            if st != STATE_CLOSED && st != STATE_TIME_WAIT {
                alloc_unlock();
                return Err("EADDRINUSE");
            }
        }
    }

    // Find a free slot.
    let mut slot = None;
    for i in 0..MAX_LISTENERS {
        let l = listener(i);
        if !l.in_use.load(Ordering::Acquire) {
            slot = Some(i);
            break;
        }
    }
    let slot = match slot {
        Some(s) => s,
        None => { alloc_unlock(); return Err("EMFILE"); }
    };

    // Populate the slot.
    let l = listener_mut(slot);
    l.local_port = local_port;
    l.backlog = backlog.clamp(1, ACCEPT_QUEUE_DEPTH as i32) as u8;
    l.cave_id = cave_id;
    l.fd.store(fd, Ordering::Release);
    l.accept_head.store(0, Ordering::Release);
    l.accept_tail.store(0, Ordering::Release);
    l.close_gen.fetch_add(1, Ordering::AcqRel);
    l.in_use.store(true, Ordering::Release);

    alloc_unlock();

    // install a per-listener firewall rule so inbound
    // TCP to this dst_port is explicitly allowed. Today this is
    // redundant with the boot-time wildcard inbound TCP rule, but it
    // hardens the path for a future tightening pass that drops the
    // wildcard. Rule survives until listen_close revokes it.
    crate::net::firewall::allow_inbound_tcp_dst_port(local_port);

    Ok(slot)
}

/// Tear down a listener. Drains the accept queue (each pending PCB
/// gets RST+free) and clears the slot. Idempotent — silent no-op if
/// no listener exists for `local_port`.
pub fn listen_close(local_port: u16) {
    // Find the slot under the alloc_lock. We collect everything we
    // need to RST inside the lock, then release the lock, then send
    // the RSTs. send_tcp_pcb does ip::send → arp resolve → enqueue
    // which can be slow; we don't want to hold alloc_lock for that.
    alloc_lock();
    let slot = (0..MAX_LISTENERS).find(|&i| {
        let l = listener(i);
        l.in_use.load(Ordering::Acquire) && l.local_port == local_port
    });
    let pending: [usize; ACCEPT_QUEUE_DEPTH];
    let pending_count: usize;
    if let Some(i) = slot {
        let l = listener_mut(i);
        // Bump generation FIRST so any in-flight accept() sees the
        // listener went away and refunds before we wipe state.
        l.close_gen.fetch_add(1, Ordering::AcqRel);
        // Snapshot the accept queue so we can RST outside the lock.
        let mut buf = [0usize; ACCEPT_QUEUE_DEPTH];
        let mut n = 0usize;
        loop {
            let h = l.accept_head.load(Ordering::Acquire);
            let t = l.accept_tail.load(Ordering::Acquire);
            if h == t { break; }
            let pcb_id = l.accept_q[t % ACCEPT_QUEUE_DEPTH] as usize;
            l.accept_tail.store(t.wrapping_add(1), Ordering::Release);
            if n < buf.len() {
                buf[n] = pcb_id;
                n += 1;
            }
        }
        pending = buf;
        pending_count = n;
        // Wipe the slot.
        l.local_port = 0;
        l.backlog = 0;
        l.cave_id = 0;
        l.fd.store(-1, Ordering::Release);
        l.in_use.store(false, Ordering::Release);
    } else {
        pending = [0usize; ACCEPT_QUEUE_DEPTH];
        pending_count = 0;
    }
    alloc_unlock();

    // step 7: send RST + free for each pending PCB so
    // the peer sees ECONNRESET instead of silent drop. Pre-fix did
    // just `pcb_free` which leaks the connection from the peer's
    // perspective until their own retransmit timer fires.
    for i in 0..pending_count {
        let pcb_id = pending[i];
        if pcb_id < MAX_PCBS {
            send_tcp_pcb(pcb_id, TCP_RST, &[]);
            pcb_free(pcb_id);
        }
    }

    // revoke the per-listener firewall rule.
    // Idempotent — silent if no matching rule (e.g. the listener was
    // already closed once and we're being called a second time, or
    // the rule was somehow externally removed).
    crate::net::firewall::revoke_inbound_tcp_dst_port(local_port);

    // Also abort any SYN_RECEIVED PCBs whose parent_listener_idx
    // was the listener we just closed. Without this they'd sit in
    // the table for the zombie-drain timeout (~30s) wasting slots
    // even though their parent is gone. We send RST so the peer
    // (which thinks it's mid-handshake) gives up cleanly.
    //
    // We didn't store the listener_idx of the closing slot; instead
    // we walk all PCBs and check whose parent_listener points at a
    // listener slot that's NOT in_use. Cheap (64 PCBs).
    unsafe {
        for i in 0..MAX_PCBS {
            let p = &TCP_PCBS[i];
            if !p.in_use.load(Ordering::Acquire) { continue; }
            let st = p.state.load(Ordering::Acquire);
            if st != STATE_SYN_RECEIVED { continue; }
            let parent = p.parent_listener_idx.load(Ordering::Acquire);
            if parent == u32::MAX { continue; }
            let parent = parent as usize;
            if parent >= MAX_LISTENERS { continue; }
            // Parent slot still in use? then this SYN_RECV belongs
            // to a different (still-alive) listener — leave alone.
            if listener(parent).in_use.load(Ordering::Acquire) { continue; }
            // Parent slot was closed — kill the orphan.
            send_tcp_pcb(i, TCP_RST, &[]);
            pcb_free(i);
        }
    }
}

/// Push a completed-handshake PCB onto the listener's accept queue.
/// Called from the SYN_RECEIVED→ESTABLISHED transition in
/// `handle_incoming`. Returns `Ok(())` if queued, `Err(())` if the
/// queue is full (caller should RST and free the PCB).
// /
/// On success, also signals epoll readiness on the listening fd so
/// a parked `epoll_wait` returns with EPOLLIN.
pub fn listener_accept_push(listener_idx: usize, pcb_id: usize) -> Result<(), ()> {
    if listener_idx >= MAX_LISTENERS { return Err(()); }
    let l = listener_mut(listener_idx);
    if !l.in_use.load(Ordering::Acquire) { return Err(()); }
    let h = l.accept_head.load(Ordering::Acquire);
    let t = l.accept_tail.load(Ordering::Acquire);
    if h.wrapping_sub(t) >= (l.backlog as usize) { return Err(()); }
    l.accept_q[h % ACCEPT_QUEUE_DEPTH] = pcb_id as u16;
    l.accept_head.store(h.wrapping_add(1), Ordering::Release);
    // Wake epoll_wait on the listening fd.
    let fd = l.fd.load(Ordering::Acquire);
    if fd >= 0 {
        crate::caves::linux::epoll::mark_ready(fd, EPOLLIN);
    }
    Ok(())
}

/// Pop a completed-handshake PCB from the listener's accept queue.
/// Called from `sys_accept[4]`. Returns `Some(pcb_id)` or None when
/// the queue is empty (caller returns EAGAIN or blocks).
pub fn listener_accept_pop(listener_idx: usize) -> Option<usize> {
    if listener_idx >= MAX_LISTENERS { return None; }
    let l = listener_mut(listener_idx);
    if !l.in_use.load(Ordering::Acquire) { return None; }
    let h = l.accept_head.load(Ordering::Acquire);
    let t = l.accept_tail.load(Ordering::Acquire);
    if h == t { return None; }
    let pcb_id = l.accept_q[t % ACCEPT_QUEUE_DEPTH] as usize;
    l.accept_tail.store(t.wrapping_add(1), Ordering::Release);
    Some(pcb_id)
}

// ─── step 5: SYN cookies ──────────────────────────────────
//
// RFC 4987 SYN cookies — DoS mitigation against SYN floods. Without
// them, an attacker spraying SYNs from spoofed source IPs exhausts
// MAX_PCBS=64 (each SYN allocates a SYN_RECEIVED PCB that never gets
// the third ACK because the spoof source can't see our SYN+ACK).
// Real users then get "no free PCBs" silently dropped.
//
// With cookies, we don't allocate ANY state for an inbound SYN when
// the table is full. Instead we encode all the state we'll need
// (MSS class + time window + MAC) into the ISN we send back. When
// the third ACK arrives with `ack = cookie_isn + 1`, we re-derive
// the MAC and verify it — if valid, the connection is "real" (the
// peer must have received our SYN+ACK to know our ISN), so we
// allocate the PCB at ESTABLISHED-time.
//
// Cookie ISN encoding (32 bits):
// bits 31..27 (5) : MSS class (we only use 0..3 today)
// bits 26..24 (3) : time window number (3 bits → 8 windows × 60s
// = 8 minutes total cookie validity, but we
// only accept current + previous = ~120s)
// bits 23..0 (24) : MAC = HMAC-SHA-256(secret, src_ip || src_port
// || dst_port || mss_class || twin)[..3]
//
// The 24-bit MAC has a 1/16M forge probability per attempt. An
// off-path attacker can't see our SYN+ACK so they have to guess
// blind; 16M attempts at line rate is hours. The threat model
// for SYN cookies is DoS, not authentication — anyone observing
// our SYN+ACK could replay. Acceptable per the RFC.
//
// Cookies engage ONLY when pcb_alloc returns None (table full).
// Normal operation = no cookies = full TCP options (window scaling,
// SACK, etc.) preserved. The MSS-class downgrade only matters under
// flood, which is the exact case we're trying to keep alive.

const COOKIE_WINDOW_SECONDS: u64 = 60;
const COOKIE_MSS_CLASSES: [u16; 4] = [536, 1220, 1380, 1460];

fn cookie_secret() -> [u8; 32] {
    // Per-boot secret keyed off boot_cookie. Same primitive used for
    // ISN derivation; OK to share because the cookie input space
    // (src_ip || src_port || dst_port || mss || twin) is disjoint
    // from the ISN input space (pcb_id || remote_ip || remote_port).
    let bc = boot_cookie();
    let mut input = [0u8; 16];
    input[..8].copy_from_slice(&bc.to_le_bytes());
    input[8..].copy_from_slice(b"syn-cookie\0\0\0\0\0\0");
    crate::crypto::sha256::hash(&input)
}

fn cookie_current_twin() -> u8 {
    // Time window number from cntpct_el0 / freq / 60. Take low 3 bits.
    let now: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { return 0; }
    ((now / freq / COOKIE_WINDOW_SECONDS) & 0x07) as u8
}

fn cookie_mss_class_for(mss: u16) -> u8 {
    // Pick the largest class <= mss. Falls through to class 0 (536).
    let mut cls = 0u8;
    for (i, &m) in COOKIE_MSS_CLASSES.iter().enumerate() {
        if mss >= m { cls = i as u8; }
    }
    cls
}

/// Compute the 24-bit MAC over the 4-tuple + mss class + time window.
/// Returns the cookie ISN with all bit fields packed.
fn cookie_compute(
    secret: &[u8; 32],
    src_ip: u32,
    src_port: u16,
    dst_port: u16,
    mss_class: u8,
    twin: u8,
) -> u32 {
    let mut input = [0u8; 10];
    input[0..4].copy_from_slice(&src_ip.to_be_bytes());
    input[4..6].copy_from_slice(&src_port.to_be_bytes());
    input[6..8].copy_from_slice(&dst_port.to_be_bytes());
    input[8] = mss_class;
    input[9] = twin;
    let mac = crate::crypto::sha256::hmac(secret, &input);
    let mac24 = ((mac[0] as u32) << 16) | ((mac[1] as u32) << 8) | (mac[2] as u32);
    ((mss_class as u32 & 0x1F) << 27)
        | ((twin as u32 & 0x07) << 24)
        | (mac24 & 0x00FF_FFFF)
}

/// Validate a cookie carried by a third-ACK's `ack-1` value.
/// Tries the current and previous time windows. On success returns
/// `Some(mss_class)`; on failure returns None.
fn cookie_validate(
    secret: &[u8; 32],
    src_ip: u32,
    src_port: u16,
    dst_port: u16,
    cookie_isn: u32,
) -> Option<u8> {
    let mss_class = ((cookie_isn >> 27) & 0x1F) as u8;
    let twin_in   = ((cookie_isn >> 24) & 0x07) as u8;
    let mac_in    = cookie_isn & 0x00FF_FFFF;

    if (mss_class as usize) >= COOKIE_MSS_CLASSES.len() { return None; }

    // Accept current and previous time window (modular wrap-around
    // because we only have 3 bits — drop windows that aren't within
    // 1 of "now" mod 8).
    let now = cookie_current_twin();
    let prev = (now.wrapping_sub(1)) & 0x07;
    if twin_in != now && twin_in != prev {
        return None;
    }

    // Re-compute and constant-time compare the MAC.
    let mut input = [0u8; 10];
    input[0..4].copy_from_slice(&src_ip.to_be_bytes());
    input[4..6].copy_from_slice(&src_port.to_be_bytes());
    input[6..8].copy_from_slice(&dst_port.to_be_bytes());
    input[8] = mss_class;
    input[9] = twin_in;
    let mac = crate::crypto::sha256::hmac(secret, &input);
    let expected = ((mac[0] as u32) << 16) | ((mac[1] as u32) << 8) | (mac[2] as u32);
    let mut diff = (mac_in ^ expected) as u8;
    diff |= ((mac_in ^ expected) >> 8)  as u8;
    diff |= ((mac_in ^ expected) >> 16) as u8;
    if diff == 0 { Some(mss_class) } else { None }
}

/// Send a TCP segment without an associated PCB. Used by the SYN-cookie
/// path where we deliberately don't allocate state for a SYN_RECEIVED
/// half-open. Caller supplies the full 4-tuple, sequence numbers, and
/// flags; we build the segment, compute the checksum, and ship it via
/// `ip::send`.
// /
/// `wnd` is the receive window we advertise. With cookies engaged the
/// PCB doesn't exist yet so we use a fixed reasonable default.
fn send_tcp_raw(
    src_ip: u32, // our IP (network order); pass 0 to use ip::our_ip()
    dst_ip: u32,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u8,
    wnd: u16,
) {
    let mut tcp = [0u8; TCP_HDR_SIZE];
    tcp[0..2].copy_from_slice(&src_port.to_be_bytes());
    tcp[2..4].copy_from_slice(&dst_port.to_be_bytes());
    tcp[4..8].copy_from_slice(&seq.to_be_bytes());
    tcp[8..12].copy_from_slice(&ack.to_be_bytes());
    tcp[12] = 0x50; // data offset 5 words, no options
    tcp[13] = flags;
    tcp[14..16].copy_from_slice(&wnd.to_be_bytes());

    let our_ip_be = if src_ip == 0 { ip::our_ip() } else { src_ip };
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&our_ip_be.to_be_bytes());
    pseudo[4..8].copy_from_slice(&dst_ip.to_be_bytes());
    pseudo[9] = 6;
    pseudo[10..12].copy_from_slice(&(TCP_HDR_SIZE as u16).to_be_bytes());
    let cksum = tcp_checksum(&pseudo, &tcp);
    tcp[16..18].copy_from_slice(&cksum.to_be_bytes());

    let _ = ip::send(dst_ip, 6, &tcp);
}

/// enumerate active listeners for the `tcp-list` shell
/// command. Calls `f(local_port, backlog, fd, accept_count)` for each
/// active listener slot. Cheap — walks MAX_LISTENERS=16 once.
pub fn for_each_listener<F: FnMut(u16, u8, i32, usize)>(mut f: F) {
    for i in 0..MAX_LISTENERS {
        let l = listener(i);
        if !l.in_use.load(Ordering::Acquire) { continue; }
        let h = l.accept_head.load(Ordering::Acquire);
        let t = l.accept_tail.load(Ordering::Acquire);
        f(l.local_port, l.backlog, l.fd.load(Ordering::Acquire),
          h.wrapping_sub(t));
    }
}

/// enumerate active connection PCBs for the `tcp-list`
/// shell command. Calls `f(state, local_port, remote_ip_host, remote_port,
/// fd)` for each non-CLOSED PCB. Cheap — walks MAX_PCBS=64 once.
pub fn for_each_pcb<F: FnMut(u32, u16, u32, u16, i32)>(mut f: F) {
    unsafe {
        for i in 0..MAX_PCBS {
            let p = &TCP_PCBS[i];
            if !p.in_use.load(Ordering::Acquire) { continue; }
            let st = p.state.load(Ordering::Acquire);
            if st == STATE_CLOSED { continue; }
            f(st, p.local_port, p.remote_ip, p.remote_port,
              p.fd.load(Ordering::Acquire));
        }
    }
}

/// human-readable name for a TCP state code. Used by
/// `tcp-list` shell output.
pub fn state_name(state: u32) -> &'static str {
    match state {
        STATE_CLOSED        => "CLOSED",
        STATE_LISTEN        => "LISTEN",
        STATE_SYN_SENT      => "SYN_SENT",
        STATE_SYN_RECEIVED  => "SYN_RECV",
        STATE_ESTABLISHED   => "ESTAB",
        STATE_FIN_WAIT_1    => "FINW1",
        STATE_FIN_WAIT_2    => "FINW2",
        STATE_CLOSE_WAIT    => "CLOSEW",
        STATE_CLOSING       => "CLOSING",
        STATE_LAST_ACK      => "LAST_ACK",
        STATE_TIME_WAIT     => "TWAIT",
        _ => "?",
    }
}

/// True iff `listener_idx` has at least one PCB ready to accept.
pub fn listener_has_pending(listener_idx: usize) -> bool {
    if listener_idx >= MAX_LISTENERS { return false; }
    listener(listener_idx).accept_count() > 0
}

/// Look up an active listener by its associated fd. Returns the
/// slot index, or None if no active listener owns this fd.
/// Used by sys_accept[4] to find the right listener from the
/// listening file descriptor.
pub fn listener_lookup_by_fd(fd: i32) -> Option<usize> {
    if fd < 0 { return None; }
    for i in 0..MAX_LISTENERS {
        let l = listener(i);
        if l.in_use.load(Ordering::Acquire) && l.fd.load(Ordering::Acquire) == fd {
            return Some(i);
        }
    }
    None
}

/// Read-only accessor for an established PCB's remote endpoint.
/// Returns `(remote_ip, remote_port)` both in HOST byte order
/// (despite the `TcpPcb::remote_ip` field comment saying big-endian
/// that comment is wrong; the field is set from
/// `IpPacket::parse`'s `from_be_bytes` which returns host-order,
/// and `send_tcp_pcb` calls `.to_be_bytes()` to put it back on the
/// wire). Returns `(0, 0)` if `id` is out of range or the PCB is
/// unused.
pub fn pcb_remote(id: usize) -> (u32, u16) {
    if id >= MAX_PCBS { return (0, 0); }
    let p = pcb(id);
    if !p.in_use.load(Ordering::Acquire) { return (0, 0); }
    (p.remote_ip, p.remote_port)
}

// ─── selftest ──────────────────────────────────────────────
//
// Kernel-internal unit-style test for the server-side TCP machinery.
// Wired as `tcp-selftest` shell command. Exercises the data-structure
// code paths (listener_register, accept_push/pop, listen_close, port
// collision, queue overflow) without needing real wire-level packet
// flow — that's the operator's job to verify with `nc` from the
// QEMU host once the in-kernel logic is locked down.
//
// Output is `Ok(report)` on full pass; `Err(reason)` on the first
// failed assertion. The report counts how many assertions passed
// so the operator can see partial progress.

pub struct TcpServerSelftestReport {
    pub assertions_passed: u32,
    pub final_listener_count: u32,
    pub final_pcb_count: u32,
}

pub fn selftest_server() -> Result<TcpServerSelftestReport, &'static str> {
    let mut passed: u32 = 0;
    macro_rules! assert_ok { ($cond:expr, $msg:literal) => {
        if !($cond) { return Err($msg); }
        passed += 1;
    }; }

    // Use a dedicated test port that won't clash with anything else.
    const TEST_PORT: u16 = 0xBA70; // 47728 — well above ephemeral range
    const TEST_FD:   i32 = 0x7E57; // 32343 — distinct sentinel for fd

    // Make sure we start clean (idempotent — silent no-op if absent).
    listen_close(TEST_PORT);

    // 1. listen_register on a fresh port → Ok with a slot.
    let slot = listen_register(TEST_PORT, 4, 0, TEST_FD)
        .map_err(|_| "listen_register rejected fresh port")?;
    assert_ok!(slot < MAX_LISTENERS, "listen_register slot out of range");
    assert_ok!(listener_lookup_by_port(TEST_PORT) == Some(slot),
               "listener_lookup_by_port mismatch");
    assert_ok!(listener_lookup_by_fd(TEST_FD) == Some(slot),
               "listener_lookup_by_fd mismatch");
    assert_ok!(!listener_has_pending(slot),
               "fresh listener should have empty accept queue");

    // 2. Re-registering the same port → EADDRINUSE.
    match listen_register(TEST_PORT, 4, 0, TEST_FD + 1) {
        Err("EADDRINUSE") => passed += 1,
        Ok(_)  => return Err("listen_register accepted duplicate port"),
        Err(_) => return Err("listen_register wrong errno on duplicate port"),
    }

    // 3. Allocate two fake PCBs and push them onto the accept queue.
    let pcb_a = pcb_alloc().ok_or("pcb_alloc failed (test pcb A)")?;
    let pcb_b = pcb_alloc().ok_or("pcb_alloc failed (test pcb B)")?;
    {
        let pa = pcb_mut(pcb_a);
        pa.local_port = TEST_PORT;
        pa.remote_ip = 0x01020304;
        pa.remote_port = 5001;
        pa.state.store(STATE_ESTABLISHED, Ordering::Release);
        pa.parent_listener_idx.store(slot as u32, Ordering::Release);
    }
    {
        let pb = pcb_mut(pcb_b);
        pb.local_port = TEST_PORT;
        pb.remote_ip = 0x01020305;
        pb.remote_port = 5002;
        pb.state.store(STATE_ESTABLISHED, Ordering::Release);
        pb.parent_listener_idx.store(slot as u32, Ordering::Release);
    }

    listener_accept_push(slot, pcb_a)
        .map_err(|_| "listener_accept_push A unexpectedly failed")?;
    passed += 1;
    listener_accept_push(slot, pcb_b)
        .map_err(|_| "listener_accept_push B unexpectedly failed")?;
    passed += 1;
    assert_ok!(listener_has_pending(slot),
               "listener_has_pending false after pushes");

    // 4. Pop both — order should be FIFO.
    let p1 = listener_accept_pop(slot).ok_or("accept_pop returned None for A")?;
    assert_ok!(p1 == pcb_a, "accept_pop FIFO order broken (1st)");
    let p2 = listener_accept_pop(slot).ok_or("accept_pop returned None for B")?;
    assert_ok!(p2 == pcb_b, "accept_pop FIFO order broken (2nd)");
    assert_ok!(listener_accept_pop(slot).is_none(),
               "accept_pop returned Some on empty queue");
    assert_ok!(!listener_has_pending(slot),
               "listener_has_pending true after draining");

    // Free the test PCBs since we don't actually own them.
    pcb_free(pcb_a);
    pcb_free(pcb_b);

    // 5. Backlog enforcement: pre-fill the queue to backlog capacity,
    // then verify push fails with Err(()).
    //
    // The previous version of this test held a `let l =
    // listener_mut(slot);` reference across the test body AND
    // called `listener_accept_push(slot, 99)` which internally
    // creates its OWN `&'static mut Listener` to the same memory.
    // That's aliased &mut → UB, and rustc's optimizer assumed
    // `l` was exclusive, caching the head value in a register
    // and never re-reading it after the push. Tests printed
    // head=0 after 4 fetch_adds because the increments never
    // made it back to memory before the inner mut ref read it.
    //
    // Fix: scope each `&mut` access tightly so no two refs are
    // live concurrently. The production code paths are immune
    // to this because each public fn gets its own fresh ref
    // and uses it in a short bounded sequence.
    let backlog_n = {
        let l = listener_mut(slot);
        let n = l.backlog as usize;
        l.accept_head.store(n, Ordering::Release);
        l.accept_tail.store(0, Ordering::Release);
        n
    }; // <-- &mut released here, no aliasing when push acquires its own
    let push_result = listener_accept_push(slot, 99);
    assert_ok!(push_result.is_err(),
               "listener_accept_push succeeded past backlog");
    // Drain (re-scoped &mut for the same UB-avoidance reason).
    {
        let l = listener_mut(slot);
        l.accept_head.store(0, Ordering::Release);
        l.accept_tail.store(0, Ordering::Release);
    }
    let _ = backlog_n; // suppress unused-binding lint if ever reorganized

    // 6. listen_close cleans up.
    listen_close(TEST_PORT);
    assert_ok!(listener_lookup_by_port(TEST_PORT).is_none(),
               "listen_close left lookup_by_port stale");
    assert_ok!(listener_lookup_by_fd(TEST_FD).is_none(),
               "listen_close left lookup_by_fd stale");

    // 7. Re-register after close — should succeed (slot reusable).
    let _slot2 = listen_register(TEST_PORT, 4, 0, TEST_FD)
        .map_err(|_| "re-register after close failed")?;
    listen_close(TEST_PORT);
    passed += 1;

    // 8. step 7: listen_close drains pending PCBs cleanly.
    // Re-register, push 2 PCBs onto the accept queue, then close.
    // All PCBs in the queue should be freed. We can't easily verify
    // the RST went on the wire from a self-test (no ip_send mock),
    // but we CAN verify the PCB slots got freed.
    let s3 = listen_register(TEST_PORT, 4, 0, TEST_FD)
        .map_err(|_| "step 8 re-register failed")?;
    let p_c = pcb_alloc().ok_or("step 8 pcb_alloc C failed")?;
    let p_d = pcb_alloc().ok_or("step 8 pcb_alloc D failed")?;
    {
        let pc = pcb_mut(p_c);
        pc.local_port = TEST_PORT;
        pc.remote_ip = 0x0A0B0C0D;
        pc.remote_port = 5005;
        pc.state.store(STATE_ESTABLISHED, Ordering::Release);
        pc.parent_listener_idx.store(s3 as u32, Ordering::Release);
    }
    {
        let pd = pcb_mut(p_d);
        pd.local_port = TEST_PORT;
        pd.remote_ip = 0x0A0B0C0E;
        pd.remote_port = 5006;
        pd.state.store(STATE_ESTABLISHED, Ordering::Release);
        pd.parent_listener_idx.store(s3 as u32, Ordering::Release);
    }
    listener_accept_push(s3, p_c)
        .map_err(|_| "step 8 push C failed")?;
    listener_accept_push(s3, p_d)
        .map_err(|_| "step 8 push D failed")?;
    passed += 1;
    // Close the listener — should RST + free both pending PCBs.
    listen_close(TEST_PORT);
    assert_ok!(!pcb(p_c).in_use.load(Ordering::Acquire),
               "listen_close left pending PCB C in use");
    assert_ok!(!pcb(p_d).in_use.load(Ordering::Acquire),
               "listen_close left pending PCB D in use");

    // 9. step 7: orphan SYN_RECEIVED PCBs are reaped.
    // Register a listener, allocate a PCB and put it in
    // SYN_RECEIVED with parent_listener_idx pointing at our slot,
    // then close the listener. The orphan PCB should be freed.
    let s4 = listen_register(TEST_PORT, 4, 0, TEST_FD)
        .map_err(|_| "step 9 re-register failed")?;
    let p_e = pcb_alloc().ok_or("step 9 pcb_alloc E failed")?;
    {
        let pe = pcb_mut(p_e);
        pe.local_port = TEST_PORT;
        pe.remote_ip = 0x0A0B0C0F;
        pe.remote_port = 5007;
        pe.state.store(STATE_SYN_RECEIVED, Ordering::Release);
        pe.parent_listener_idx.store(s4 as u32, Ordering::Release);
    }
    listen_close(TEST_PORT);
    assert_ok!(!pcb(p_e).in_use.load(Ordering::Acquire),
               "listen_close left orphan SYN_RECV PCB in use");

    // Count final state for the report.
    let mut listener_count: u32 = 0;
    for i in 0..MAX_LISTENERS {
        if listener(i).in_use.load(Ordering::Acquire) {
            listener_count += 1;
        }
    }
    let mut pcb_count: u32 = 0;
    unsafe {
        for i in 0..MAX_PCBS {
            if TCP_PCBS[i].in_use.load(Ordering::Acquire) {
                pcb_count += 1;
            }
        }
    }

    Ok(TcpServerSelftestReport {
        assertions_passed: passed,
        final_listener_count: listener_count,
        final_pcb_count: pcb_count,
    })
}

/// V6-XLAYER-005/006: clear every PCB on cave switch so a new tenant
/// can't inherit (or hijack) the previous cave's TCP connections.
// /
/// V8-ROOT-1: IRQ-masked for duration. Same reasoning as sockets reset.
// /
/// also clear the LISTENERS table — a listening port owned
/// by cave A must not stay open into cave B. (We can't do per-cave
/// filtering here cheaply; the rare valid case where cave B has its
/// own listener is handled by re-registration after enter.)
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    alloc_lock();
    unsafe {
        for i in 0..MAX_PCBS {
            TCP_PCBS[i] = TcpPcb::empty();
        }
        for i in 0..MAX_LISTENERS {
            LISTENERS[i] = Listener::empty();
        }
    }
    alloc_unlock();
}

// We're single-core and the dispatch path is non-reentrant, but a tiny
// spin-flag catches reentrancy bugs in PCB allocation.
static ALLOC_LOCK: AtomicBool = AtomicBool::new(false);

fn alloc_lock()  { while ALLOC_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() { core::hint::spin_loop(); } }
fn alloc_unlock(){ ALLOC_LOCK.store(false, Ordering::Release); }

/// Safe accessor — used by both driver dispatch and user-level code.
#[inline]
fn pcb_mut(id: usize) -> &'static mut TcpPcb {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TCP_PCBS) as *mut TcpPcb;
        &mut *ptr.add(id)
    }
}

#[inline]
pub fn pcb(id: usize) -> &'static TcpPcb {
    unsafe {
        let ptr = core::ptr::addr_of!(TCP_PCBS) as *const TcpPcb;
        &*ptr.add(id)
    }
}

// PCB allocation / free

/// Allocate a free PCB slot. Returns the pcb_id or None if the table is full.
pub fn pcb_alloc() -> Option<usize> {
    alloc_lock();
    let mut out = None;
    for i in 0..MAX_PCBS {
        let p = pcb_mut(i);
        if !p.in_use.load(Ordering::Acquire) {
            // Reset fields.
            p.state.store(STATE_CLOSED, Ordering::Relaxed);
            p.local_port = 0;
            p.remote_ip = 0;
            p.remote_port = 0;
            // Seed with a provisional ISN (the real one is re-derived in
            // connect_start() once we know the 4-tuple). This keeps the PCB
            // useful even if the caller inspects snd_nxt before connecting.
            p.snd_nxt = compute_isn(i, 0, 0);
            p.snd_una = p.snd_nxt;
            p.snd_wnd = 8192;
            p.rcv_nxt = 0;
            p.rcv_wnd = RX_BUF_SIZE as u16;
            p.rx_head.store(0, Ordering::Relaxed);
            p.rx_tail.store(0, Ordering::Relaxed);
            p.tx_head.store(0, Ordering::Relaxed);
            p.tx_tail.store(0, Ordering::Relaxed);
            p.is_nonblocking.store(false, Ordering::Relaxed);
            p.error.store(0, Ordering::Relaxed);
            p.fd.store(-1, Ordering::Relaxed);
            p.rx_shut.store(false, Ordering::Relaxed);
            p.tx_shut.store(false, Ordering::Relaxed);
            p.in_use.store(true, Ordering::Release);
            out = Some(i);
            break;
        }
    }
    alloc_unlock();
    out
}

/// Free a PCB slot. Caller must have already closed or reset the connection.
// /
/// V11-state-sweep: previously only state/in_use/fd were updated, leaving
/// the 16 KiB rx/tx buffers full of the prior connection's plaintext
/// (HTTP bodies, mail, whatever) for the next `pcb_alloc` tenant to
/// observe. Also leaked the 4-tuple (local_port, remote_ip, remote_port)
/// a fingerprint of the prior peer. Now we scrub everything.
pub fn pcb_free(id: usize) {
    if id >= MAX_PCBS { return; }
    alloc_lock();
    let p = pcb_mut(id);
    p.state.store(STATE_CLOSED, Ordering::Release);
    p.in_use.store(false, Ordering::Release);
    p.fd.store(-1, Ordering::Release);
    // Scrub tuple + sequence state.
    p.local_port = 0;
    p.remote_ip = 0;
    p.remote_port = 0;
    p.snd_nxt = 0;
    p.snd_una = 0;
    p.snd_wnd = 8192;
    p.rcv_nxt = 0;
    p.rcv_wnd = RX_BUF_SIZE as u16;
    // Scrub the data buffers — this is the plaintext-leak fix.
    for b in p.rx_buf.iter_mut() { *b = 0; }
    for b in p.tx_buf.iter_mut() { *b = 0; }
    p.rx_head.store(0, Ordering::Release);
    p.rx_tail.store(0, Ordering::Release);
    p.tx_head.store(0, Ordering::Release);
    p.tx_tail.store(0, Ordering::Release);
    p.rx_shut.store(false, Ordering::Release);
    p.tx_shut.store(false, Ordering::Release);
    p.error.store(0, Ordering::Release);
    p.is_nonblocking.store(false, Ordering::Release);
    p.time_wait_entered.store(0, Ordering::Release);
    // clear parent-listener back-pointer.
    p.parent_listener_idx.store(u32::MAX, Ordering::Release);
    alloc_unlock();
}

/// Attach a userspace fd so we can call `epoll::mark_ready(fd, ...)`.
pub fn pcb_bind_fd(id: usize, fd: i32) {
    if id >= MAX_PCBS { return; }
    pcb(id).fd.store(fd, Ordering::Release);
}

pub fn pcb_set_nonblocking(id: usize, nb: bool) {
    if id >= MAX_PCBS { return; }
    pcb(id).is_nonblocking.store(nb, Ordering::Release);
}

pub fn pcb_state(id: usize) -> u32 {
    if id >= MAX_PCBS { return STATE_CLOSED; }
    pcb(id).state.load(Ordering::Acquire)
}

// epoll notify helper

fn notify_epoll(id: usize, events: u32) {
    let fd = pcb(id).fd.load(Ordering::Acquire);
    if fd >= 0 {
        crate::caves::linux::epoll::mark_ready(fd, events);
    }
}

// Drain zombie PCBs: any non-ESTABLISHED, non-LISTEN PCB that has been in
// its current transient state longer than the per-state limit transitions
// to CLOSED and releases its slot.
//
// NEW-DOS-008: previously this only covered TIME_WAIT. A remote peer
// that dropped our SYN silently (or sent FIN and then vanished) would
// pin the PCB in SYN_SENT / FIN_WAIT_* until reboot — 64 such hangs
// drained the whole TCP table.
const TIME_WAIT_NS_2MSL:  u64 = 30_000_000_000;  // 30 s (2×MSL shortened)
const SYN_SENT_NS:        u64 = 60_000_000_000;  // 60 s connect deadline
const FIN_WAIT_NS:        u64 = 60_000_000_000;  // 60 s for peer FIN/ACK
const LAST_ACK_NS:        u64 = 60_000_000_000;
const CLOSING_NS:         u64 =  60_000_000_000;

fn state_limit_ns(state: u32) -> u64 {
    match state {
        STATE_TIME_WAIT  => TIME_WAIT_NS_2MSL,
        STATE_SYN_SENT   => SYN_SENT_NS,
        STATE_FIN_WAIT_1 => FIN_WAIT_NS,
        STATE_FIN_WAIT_2 => FIN_WAIT_NS,
        STATE_LAST_ACK   => LAST_ACK_NS,
        STATE_CLOSING    => CLOSING_NS,
        _ => 0,
    }
}

fn drain_time_wait() {
    let now: u64; let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { return; }
    for i in 0..MAX_PCBS {
        let p = pcb(i);
        let state = p.state.load(Ordering::Relaxed);
        let limit_ns = state_limit_ns(state);
        if limit_ns == 0 { continue; }
        let deadline_ticks = limit_ns / (1_000_000_000 / freq.max(1));
        let entered = p.time_wait_entered.load(Ordering::Relaxed);
        if entered != 0 && now.saturating_sub(entered) > deadline_ticks {
            p.state.store(STATE_CLOSED, Ordering::Release);
            p.in_use.store(false, Ordering::Release);
            p.time_wait_entered.store(0, Ordering::Relaxed);
            // Refund the owning cave's socket/fd quota.
            let fd = p.fd.load(Ordering::Relaxed);
            if fd >= 0 {
                crate::caves::linux::quotas::refund_active(
                    crate::caves::linux::quotas::Resource::Sockets, 1);
            }
        }
    }
}

// Incoming packet dispatch

/// Find the PCB owning (remote_ip, remote_port, local_port).
fn pcb_lookup(remote_ip: u32, remote_port: u16, local_port: u16) -> Option<usize> {
    for i in 0..MAX_PCBS {
        let p = pcb(i);
        if !p.in_use.load(Ordering::Acquire) { continue; }
        if p.local_port != local_port { continue; }
        // Half-opened sockets don't have the remote set yet — match on
        // local_port only when state is SYN_SENT (we just sent SYN).
        let st = p.state.load(Ordering::Acquire);
        if st == STATE_SYN_SENT {
            if p.remote_ip == remote_ip && p.remote_port == remote_port {
                return Some(i);
            }
            continue;
        }
        if p.remote_ip == remote_ip && p.remote_port == remote_port {
            return Some(i);
        }
    }
    None
}

/// Called by `super::ip` when a TCP segment arrives.
pub fn handle_incoming(pkt: &IpPacket) {
    // Opportunistically drain expired TIME_WAIT PCBs so their slots
    // become reusable. Cheap (~64 loads on the hot path).
    drain_time_wait();

    if pkt.payload.len() < TCP_HDR_SIZE { return; }

    // ATTACK-NET-011: verify the TCP checksum before trusting any field.
    // An L2-adjacent attacker who can inject segments but not intercept the
    // real peer's traffic will typically get the checksum wrong; requiring
    // a valid checksum forces the attacker into the same position as a
    // real MITM (much harder).
    if !verify_tcp_checksum(pkt) { return; }

    let src_port = u16::from_be_bytes([pkt.payload[0], pkt.payload[1]]);
    let dst_port = u16::from_be_bytes([pkt.payload[2], pkt.payload[3]]);

    // NET2-019: per-port firewall check now that we've parsed the header.
    // The pre-parse `allow_inbound` check only matched on src_ip + protocol;
    // a port-gated rule (e.g. "allow TCP from 10.0.0.1 port 443 only") would
    // otherwise let in any TCP port.
    if !crate::net::firewall::allow_inbound_tcp(pkt.src, src_port, dst_port) {
        return;
    }
    let seq = u32::from_be_bytes([pkt.payload[4], pkt.payload[5], pkt.payload[6], pkt.payload[7]]);
    let ack = u32::from_be_bytes([pkt.payload[8], pkt.payload[9], pkt.payload[10], pkt.payload[11]]);
    let data_off = ((pkt.payload[12] >> 4) as usize) * 4;
    let flags = pkt.payload[13];
    let wnd = u16::from_be_bytes([pkt.payload[14], pkt.payload[15]]);

    if data_off < TCP_HDR_SIZE || data_off > pkt.payload.len() { return; }

    let id = match pcb_lookup(pkt.src, src_port, dst_port) {
        Some(i) => i,
        None => {
            // No PCB for this 4-tuple. Three sub-paths:
            // A) bare SYN + listener for dst_port → new connection
            // (steps 2 + 5 SYN-cookie fallback if PCB table full)
            // B) bare ACK + listener for dst_port + valid cookie →
            // reconstruct PCB at ESTABLISHED (step 5 second half)
            // C) anything else → silently drop
            let listener_idx = listener_lookup_by_port(dst_port);

            let is_bare_syn = (flags & TCP_SYN) != 0 && (flags & TCP_ACK) == 0;
            let is_bare_ack = (flags & TCP_SYN) == 0 && (flags & TCP_ACK) != 0;

            // Path B: bare ACK potentially carrying a SYN cookie.
            if is_bare_ack {
                let li = match listener_idx { Some(i) => i, None => return };
                // The peer's third ACK has ack = our_isn + 1, where
                // our_isn was the cookie we encoded.
                let cookie_isn = ack.wrapping_sub(1);
                let secret = cookie_secret();
                let mss_class = match cookie_validate(
                    &secret, pkt.src, src_port, dst_port, cookie_isn,
                ) {
                    Some(c) => c,
                    None => return, // not a cookie we'd issue — drop
                };

                // Cookie validated. Allocate a real PCB now (the SYN
                // flood that exhausted the table earlier may have
                // eased; if alloc still fails the connection is lost
                // but the peer can retry).
                let new_id = match pcb_alloc() { Some(i) => i, None => return };
                let np = pcb_mut(new_id);
                np.local_port  = dst_port;
                np.remote_ip   = pkt.src;
                np.remote_port = src_port;
                // We never tracked peer_seq + 1, so derive it from
                // the third ACK: peer's `seq` is the byte after the
                // SYN's seq, so peer_seq_original = received seq - 1
                // and rcv_nxt = peer_seq_original + 1 = received seq.
                np.rcv_nxt = seq;
                np.snd_nxt = ack;             // peer ack'd cookie+1
                np.snd_una = ack;
                np.rcv_wnd = RX_BUF_SIZE as u16;
                np.parent_listener_idx.store(li as u32, Ordering::Release);
                np.state.store(STATE_ESTABLISHED, Ordering::Release);
                let _ = mss_class; // future: clamp send-MSS

                // Push onto accept queue + wake epoll on listening fd.
                if listener_accept_push(li, new_id).is_err() {
                    // Queue full — RST + free.
                    send_tcp_pcb(new_id, TCP_RST, &[]);
                    pcb_free(new_id);
                }
                return;
            }

            // Path A: bare SYN to a listener.
            if is_bare_syn {
                let li = match listener_idx { Some(i) => i, None => return };

                // Try normal SYN_RECEIVED allocation first. If table
                // is full, fall through to the SYN-cookie path that
                // sends a SYN+ACK without keeping any state.
                if let Some(new_id) = pcb_alloc() {
                    let np = pcb_mut(new_id);
                    np.local_port  = dst_port;
                    np.remote_ip   = pkt.src;
                    np.remote_port = src_port;
                    np.rcv_nxt = seq.wrapping_add(1);
                    let isn = compute_isn(new_id, pkt.src, src_port);
                    np.snd_nxt = isn.wrapping_add(1);
                    np.snd_una = isn;
                    np.rcv_wnd = RX_BUF_SIZE as u16;
                    np.parent_listener_idx.store(li as u32, Ordering::Release);
                    np.state.store(STATE_SYN_RECEIVED, Ordering::Release);
                    unsafe {
                        let now: u64;
                        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
                        np.time_wait_entered.store(now, Ordering::Release);
                    }
                    send_tcp_pcb(new_id, TCP_SYN | TCP_ACK, &[]);
                } else {
                    // step 5: PCB table full. Don't drop —
                    // fall back to a SYN cookie. We allocate ZERO
                    // state; the cookie ISN we send back encodes
                    // everything we need to validate the third ACK.
                    let secret = cookie_secret();
                    let twin = cookie_current_twin();
                    // Default MSS class for an unknown peer (no MSS
                    // option support yet — when we parse TCP options
                    // we'll wire the real peer-advertised MSS here).
                    let mss_class = cookie_mss_class_for(1460);
                    let cookie_isn = cookie_compute(
                        &secret, pkt.src, src_port, dst_port,
                        mss_class, twin,
                    );
                    // SYN+ACK with seq=cookie_isn, ack=peer_seq+1.
                    send_tcp_raw(
                        0,             // our_ip — let send_tcp_raw fetch
                        pkt.src,
                        dst_port,
                        src_port,
                        cookie_isn,
                        seq.wrapping_add(1),
                        TCP_SYN | TCP_ACK,
                        RX_BUF_SIZE as u16,
                    );
                }
                return;
            }

            // Path C: drop (FIN/RST/etc. with no matching PCB).
            return;
        }
    };

    let p = pcb_mut(id);
    let state = p.state.load(Ordering::Acquire);
    p.snd_wnd = wnd;

    // RFC 5961 §3.2: only accept RST that matches rcv_nxt exactly.
    // In-window-but-not-exact RSTs get a challenge ACK (which forces
    // the real peer — if this is a spoof — to drop the injection or
    // reveal itself). Out-of-window RSTs are silently dropped.
    // This defeats blind RST injection against a guessed sequence.
    if flags & TCP_RST != 0 {
        if seq == p.rcv_nxt {
            p.error.store(E_CONNRESET, Ordering::Release);
            p.state.store(STATE_CLOSED, Ordering::Release);
            notify_epoll(id, EPOLLIN | EPOLLOUT | EPOLLERR | EPOLLHUP);
            return;
        }
        let wnd = p.rcv_wnd as u32;
        let in_window = seq_in_window(seq, p.rcv_nxt, wnd);
        if in_window && try_challenge_ack() {
            send_tcp_pcb(id, TCP_ACK, &[]);
        }
        return;
    }

    match state {
        STATE_SYN_SENT => {
            if flags & TCP_SYN != 0 && flags & TCP_ACK != 0 {
                // Validate ACK covers our SYN.
                if ack != p.snd_nxt {
                    // Unexpected ACK — drop.
                    return;
                }
                p.snd_una = ack;
                p.rcv_nxt = seq.wrapping_add(1);
                p.state.store(STATE_ESTABLISHED, Ordering::Release);
                send_tcp_pcb(id, TCP_ACK, &[]);
                notify_epoll(id, EPOLLOUT); // connect completed → writable
            } else if flags & TCP_SYN != 0 {
                // Simultaneous open — SYN only → move to SYN_RECEIVED.
                p.rcv_nxt = seq.wrapping_add(1);
                p.state.store(STATE_SYN_RECEIVED, Ordering::Release);
                send_tcp_pcb(id, TCP_SYN | TCP_ACK, &[]);
            }
        }

        STATE_SYN_RECEIVED => {
            if flags & TCP_ACK != 0 && ack == p.snd_nxt {
                p.snd_una = ack;
                p.state.store(STATE_ESTABLISHED, Ordering::Release);
                // Clear the SYN_RECEIVED zombie-drain timestamp; the
                // PCB is now a real connection and the existing
                // ESTABLISHED reaping rules apply (idle timeouts,
                // explicit close, etc.).
                p.time_wait_entered.store(0, Ordering::Release);
                notify_epoll(id, EPOLLOUT);

                // step 3: if this PCB came from a
                // SYN-on-LISTEN allocation, push it onto the parent
                // listener's accept queue so the next sys_accept
                // returns this fd and epoll_wait wakes on the
                // listening fd. parent_listener_idx == u32::MAX
                // means this was a client-side simultaneous-open
                // PCB, not a server-accept — leave it alone.
                let parent = p.parent_listener_idx.load(Ordering::Acquire);
                if parent != u32::MAX {
                    let parent = parent as usize;
                    match listener_accept_push(parent, id) {
                        Ok(()) => { /* listener_accept_push handles epoll */ }
                        Err(()) => {
                            // Accept queue full or listener gone.
                            // RST + free. The peer will see ECONNRESET.
                            send_tcp_pcb(id, TCP_RST, &[]);
                            pcb_free(id);
                        }
                    }
                }
            }
        }

        STATE_ESTABLISHED | STATE_FIN_WAIT_1 | STATE_FIN_WAIT_2 => {
            // RFC 5961 §4: a bare SYN arriving on an already-established
            // connection is a potential blind-SYN injection. Respond with
            // a challenge ACK carrying our current state; the real peer's
            // stack will see the ACK is for a sequence past the injection
            // and do the right thing, while an attacker is forced to
            // burn a round-trip to learn our sequence number.
            if flags & TCP_SYN != 0 {
                if try_challenge_ack() {
                    send_tcp_pcb(id, TCP_ACK, &[]);
                }
                return;
            }
            // ACK updates snd_una.
            if flags & TCP_ACK != 0 {
                // Only move forward; guard against old ACKs.
                if seq_leq(p.snd_una, ack) && seq_leq(ack, p.snd_nxt) {
                    p.snd_una = ack;
                    notify_epoll(id, EPOLLOUT); // space freed
                }
            }

            // Data payload.
            let payload_len = pkt.payload.len() - data_off;
            if payload_len > 0 && state != STATE_FIN_WAIT_2 {
                // FIN_WAIT_2 still accepts data per RFC but simplify:
                // accept in ESTABLISHED & FIN_WAIT_1 only.
                let rx_shut = p.rx_shut.load(Ordering::Acquire);
                if !rx_shut && seq == p.rcv_nxt {
                    // In-order segment. Copy into rx ring.
                    let free = p.rx_free();
                    let copy = payload_len.min(free);
                    if copy > 0 {
                        let head = p.rx_head.load(Ordering::Acquire);
                        for i in 0..copy {
                            let idx = (head + i) & (RX_BUF_SIZE - 1);
                            p.rx_buf[idx] = pkt.payload[data_off + i];
                        }
                        p.rx_head.store(head.wrapping_add(copy), Ordering::Release);
                        p.rcv_nxt = p.rcv_nxt.wrapping_add(copy as u32);
                    }
                    // ACK what we accepted (even if copy<payload — peer will retransmit).
                    send_tcp_pcb(id, TCP_ACK, &[]);
                    if copy > 0 { notify_epoll(id, EPOLLIN); }
                } else {
                    // Out-of-order: just ACK current rcv_nxt to prompt retransmit.
                    send_tcp_pcb(id, TCP_ACK, &[]);
                }
            }

            // FIN handling.
            if flags & TCP_FIN != 0 {
                // Consume the FIN's sequence space.
                p.rcv_nxt = p.rcv_nxt.wrapping_add(1);
                send_tcp_pcb(id, TCP_ACK, &[]);
                match state {
                    STATE_ESTABLISHED => {
                        p.state.store(STATE_CLOSE_WAIT, Ordering::Release);
                        notify_epoll(id, EPOLLIN | EPOLLRDHUP);
                    }
                    STATE_FIN_WAIT_1 => {
                        // Our FIN was acked simultaneously with theirs?
                        if flags & TCP_ACK != 0 && p.snd_una == p.snd_nxt {
                            { let now: u64; unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); } p.time_wait_entered.store(now, Ordering::Relaxed); p.state.store(STATE_TIME_WAIT, Ordering::Release); }
                        } else {
                            p.state.store(STATE_CLOSING, Ordering::Release);
                        }
                        notify_epoll(id, EPOLLIN | EPOLLHUP);
                    }
                    STATE_FIN_WAIT_2 => {
                        { let now: u64; unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); } p.time_wait_entered.store(now, Ordering::Relaxed); p.state.store(STATE_TIME_WAIT, Ordering::Release); }
                        notify_epoll(id, EPOLLIN | EPOLLHUP);
                    }
                    _ => {}
                }
            } else if state == STATE_FIN_WAIT_1
                && flags & TCP_ACK != 0
                && p.snd_una == p.snd_nxt
            {
                // Our FIN has been ACKed — move to FIN_WAIT_2.
                // NEW-DOS-008: restamp state-entered for the new state.
                let now: u64;
                unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
                p.time_wait_entered.store(now, Ordering::Relaxed);
                p.state.store(STATE_FIN_WAIT_2, Ordering::Release);
            }
        }

        STATE_CLOSE_WAIT => {
            // Peer already sent FIN; mostly just track their ACKs.
            if flags & TCP_ACK != 0 && seq_leq(p.snd_una, ack) && seq_leq(ack, p.snd_nxt) {
                p.snd_una = ack;
            }
        }

        STATE_LAST_ACK => {
            if flags & TCP_ACK != 0 && ack == p.snd_nxt {
                p.state.store(STATE_CLOSED, Ordering::Release);
                notify_epoll(id, EPOLLHUP);
            }
        }

        STATE_CLOSING => {
            if flags & TCP_ACK != 0 && ack == p.snd_nxt {
                { let now: u64; unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); } p.time_wait_entered.store(now, Ordering::Relaxed); p.state.store(STATE_TIME_WAIT, Ordering::Release); }
            }
        }

        STATE_TIME_WAIT | STATE_CLOSED | STATE_LISTEN => {
            // Late segments: drop quietly (real stack would RST here).
        }

        _ => {}
    }
}

// RDHUP bit — we don't pull from epoll module to avoid tight coupling
// but must match its value (0x2000).
const EPOLLRDHUP: u32 = 0x2000;

/// sequence arithmetic (RFC 1323): `a <= b` in modulo-2^32 arithmetic.
#[inline]
fn seq_leq(a: u32, b: u32) -> bool {
    (b.wrapping_sub(a) as i32) >= 0
}

// Segment transmit

fn send_tcp_pcb(id: usize, flags: u8, payload: &[u8]) {
    let p = pcb_mut(id);
    let local_port  = p.local_port;
    let remote_port = p.remote_port;
    let remote_ip   = p.remote_ip;
    let seq = p.snd_nxt;
    let ack = p.rcv_nxt;

    let total = TCP_HDR_SIZE + payload.len();
    if total > 1400 { return; } // we don't fragment
    let mut tcp = [0u8; 1400];

    tcp[0..2].copy_from_slice(&local_port.to_be_bytes());
    tcp[2..4].copy_from_slice(&remote_port.to_be_bytes());
    tcp[4..8].copy_from_slice(&seq.to_be_bytes());
    tcp[8..12].copy_from_slice(&ack.to_be_bytes());
    tcp[12] = 0x50; // data offset 5 words
    tcp[13] = flags;
    tcp[14..16].copy_from_slice(&p.rcv_wnd.to_be_bytes());

    if !payload.is_empty() {
        tcp[TCP_HDR_SIZE..TCP_HDR_SIZE + payload.len()].copy_from_slice(payload);
    }

    // Pseudo-header checksum.
    let src_ip = ip::our_ip();
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&src_ip.to_be_bytes());
    pseudo[4..8].copy_from_slice(&remote_ip.to_be_bytes());
    pseudo[9] = 6;
    pseudo[10..12].copy_from_slice(&(total as u16).to_be_bytes());
    let cksum = tcp_checksum(&pseudo, &tcp[..total]);
    tcp[16..18].copy_from_slice(&cksum.to_be_bytes());

    // Advance snd_nxt by payload len (SYN/FIN also consume one seq).
    let mut consume = payload.len() as u32;
    if flags & TCP_SYN != 0 { consume = consume.wrapping_add(1); }
    if flags & TCP_FIN != 0 { consume = consume.wrapping_add(1); }
    if consume > 0 {
        p.snd_nxt = p.snd_nxt.wrapping_add(consume);
    }

    let _ = ip::send(remote_ip, 6, &tcp[..total]);
}

fn tcp_checksum(pseudo: &[u8], tcp: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < pseudo.len() {
        sum += u16::from_be_bytes([pseudo[i], pseudo[i+1]]) as u32;
        i += 2;
    }
    i = 0;
    while i + 1 < tcp.len() {
        sum += u16::from_be_bytes([tcp[i], tcp[i+1]]) as u32;
        i += 2;
    }
    if i < tcp.len() {
        sum += (tcp[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Validate the inbound pseudo-header + TCP-segment checksum. Returns true
/// when the segment's checksum is valid (RFC 793 one's-complement sum over
/// the pseudo-header and segment == 0xFFFF, i.e. !sum == 0).
fn verify_tcp_checksum(pkt: &IpPacket) -> bool {
    let tcp = pkt.payload;
    if tcp.len() < TCP_HDR_SIZE { return false; }

    // Pseudo-header: src_ip(4) | dst_ip(4) | 0 | proto(1) | tcp_len(2)
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&pkt.src.to_be_bytes());
    pseudo[4..8].copy_from_slice(&pkt.dst.to_be_bytes());
    pseudo[9] = 6;
    pseudo[10..12].copy_from_slice(&(tcp.len() as u16).to_be_bytes());

    // Sum pseudo-header and segment (segment includes the checksum field,
    // so a correct packet sums to 0xFFFF).
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < pseudo.len() {
        sum += u16::from_be_bytes([pseudo[i], pseudo[i + 1]]) as u32;
        i += 2;
    }
    i = 0;
    while i + 1 < tcp.len() {
        sum += u16::from_be_bytes([tcp[i], tcp[i + 1]]) as u32;
        i += 2;
    }
    if i < tcp.len() {
        sum += (tcp[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    (!(sum as u16)) == 0
}

// Unit tests (run on the host, not the kernel target)

#[cfg(test)]
mod checksum_tests {
    use super::*;

    fn build_segment(src: u32, dst: u32, data: &[u8]) -> ([u8; 1500], usize) {
        let mut buf = [0u8; 1500];
        let total = TCP_HDR_SIZE + data.len();
        buf[0..2].copy_from_slice(&1234u16.to_be_bytes());
        buf[2..4].copy_from_slice(&80u16.to_be_bytes());
        buf[4..8].copy_from_slice(&1u32.to_be_bytes());
        buf[8..12].copy_from_slice(&2u32.to_be_bytes());
        buf[12] = 0x50;
        buf[13] = TCP_ACK;
        buf[14..16].copy_from_slice(&8192u16.to_be_bytes());
        if !data.is_empty() {
            buf[TCP_HDR_SIZE..TCP_HDR_SIZE + data.len()].copy_from_slice(data);
        }
        let mut pseudo = [0u8; 12];
        pseudo[0..4].copy_from_slice(&src.to_be_bytes());
        pseudo[4..8].copy_from_slice(&dst.to_be_bytes());
        pseudo[9] = 6;
        pseudo[10..12].copy_from_slice(&(total as u16).to_be_bytes());
        let cksum = tcp_checksum(&pseudo, &buf[..total]);
        buf[16..18].copy_from_slice(&cksum.to_be_bytes());
        (buf, total)
    }

    #[test]
    fn accepts_valid_checksum() {
        let src = 0x0A000203;
        let dst = 0x0A00020F;
        let (buf, total) = build_segment(src, dst, b"hello");
        let pkt = IpPacket { src, dst, protocol: 6, payload: &buf[..total], ttl: 64 };
        assert!(verify_tcp_checksum(&pkt));
    }

    #[test]
    fn rejects_flipped_bit() {
        let src = 0x0A000203;
        let dst = 0x0A00020F;
        let (mut buf, total) = build_segment(src, dst, b"hello");
        buf[TCP_HDR_SIZE] ^= 0x01; // flip one bit in the payload
        let pkt = IpPacket { src, dst, protocol: 6, payload: &buf[..total], ttl: 64 };
        assert!(!verify_tcp_checksum(&pkt));
    }

    #[test]
    fn rejects_wrong_src_ip() {
        let src = 0x0A000203;
        let dst = 0x0A00020F;
        let (buf, total) = build_segment(src, dst, b"hello");
        // Pretend the packet arrived claiming a different src_ip — the
        // pseudo-header no longer matches, so verification must fail.
        let pkt = IpPacket { src: 0xC0A80001, dst, protocol: 6, payload: &buf[..total], ttl: 64 };
        assert!(!verify_tcp_checksum(&pkt));
    }
}

// Non-blocking connect

pub enum ConnectStatus {
    InProgress,
    Established,
    Failed(i32),
}

/// Kick off a SYN. Returns 0 (success queued) or E_INPROGRESS for nonblocking.
/// On transport-level failure returns a positive errno.
pub fn connect_start(id: usize, ip_be: u32, port: u16) -> i32 {
    if id >= MAX_PCBS { return E_NOTCONN; }
    let p = pcb_mut(id);
    if !p.in_use.load(Ordering::Acquire) { return E_NOTCONN; }

    // Global isolation gate. Refuse before allocating a local port +
    // sending the SYN; the caller gets a fast clear error instead of
    // the 30-second connect timeout that ip::send's silently-discarded
    // refusal would otherwise produce.
    if crate::net::is_isolated() {
        crate::drivers::uart::puts("[tcp] outbound refused: net isolated\n");
        return E_NETUNREACH;
    }

    let lport = alloc_local_port();
    p.local_port  = lport;
    p.remote_ip   = ip_be;
    p.remote_port = port;
    p.error.store(0, Ordering::Relaxed);
    p.rx_head.store(0, Ordering::Relaxed);
    p.rx_tail.store(0, Ordering::Relaxed);
    p.tx_head.store(0, Ordering::Relaxed);
    p.tx_tail.store(0, Ordering::Relaxed);
    p.rcv_nxt = 0;
    // ATTACK-NET-009: compute the ISN now that we know remote_ip and
    // remote_port. This binds the ISN to the 4-tuple + boot cookie so an
    // off-path observer of one connection's ISN cannot predict another's.
    p.snd_nxt = compute_isn(id, ip_be, port);
    p.snd_una = p.snd_nxt; // mark ISS
    // NEW-DOS-008: stamp state-entered so drain_time_wait can expire a
    // stuck SYN_SENT PCB after SYN_SENT_NS (60 s).
    {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        p.time_wait_entered.store(now, Ordering::Relaxed);
    }
    p.state.store(STATE_SYN_SENT, Ordering::Release);

    send_tcp_pcb(id, TCP_SYN, &[]);

    // gap-audit item 045: register the flow in conntrack so the
    // inbound side can recognise reply packets as belonging to a
    // Sphragis-initiated connection. Wildcard inbound TCP allow rule
    // still exists; conntrack here is defense-in-depth + foundation
    // for a future hardening pass that removes the wildcard.
    crate::net::conntrack::register_outbound(
        6, // TCP
        ip_be, port, lport,
        crate::net::conntrack::State::New,
    );

    if p.is_nonblocking.load(Ordering::Acquire) {
        E_INPROGRESS
    } else {
        0
    }
}

/// Polls an in-progress connection. Callers that want to block must drive
/// `super::poll_once()` themselves between calls.
pub fn connect_poll(id: usize) -> ConnectStatus {
    if id >= MAX_PCBS { return ConnectStatus::Failed(E_NOTCONN); }
    let p = pcb(id);
    match p.state.load(Ordering::Acquire) {
        STATE_ESTABLISHED => ConnectStatus::Established,
        STATE_SYN_SENT | STATE_SYN_RECEIVED => ConnectStatus::InProgress,
        _ => {
            let e = p.error.load(Ordering::Acquire);
            ConnectStatus::Failed(if e != 0 { e } else { E_CONNREFUSED })
        }
    }
}

// Per-PCB I/O

/// Send data on an established PCB. Returns bytes accepted or errno (positive).
pub fn send_data_pcb(id: usize, data: &[u8]) -> Result<usize, i32> {
    if id >= MAX_PCBS { return Err(E_NOTCONN); }
    let p = pcb_mut(id);
    if !p.in_use.load(Ordering::Acquire) { return Err(E_NOTCONN); }
    if p.tx_shut.load(Ordering::Acquire) { return Err(E_PIPE); }
    let st = p.state.load(Ordering::Acquire);
    if st != STATE_ESTABLISHED && st != STATE_CLOSE_WAIT {
        return Err(E_NOTCONN);
    }
    if data.is_empty() { return Ok(0); }

    // MSS ceiling — one segment per call keeps things simple.
    let chunk = data.len().min(1360);
    send_tcp_pcb(id, TCP_PSH | TCP_ACK, &data[..chunk]);
    Ok(chunk)
}

/// Receive from an established PCB's rx ring. Returns bytes read; 0 means EOF
/// (peer FIN received and ring drained); errno for error.
pub fn recv_data_pcb(id: usize, buf: &mut [u8]) -> Result<usize, i32> {
    if id >= MAX_PCBS { return Err(E_NOTCONN); }
    let p = pcb_mut(id);
    if !p.in_use.load(Ordering::Acquire) { return Err(E_NOTCONN); }

    let avail = p.rx_available();
    if avail == 0 {
        let st = p.state.load(Ordering::Acquire);
        if st == STATE_CLOSE_WAIT
            || st == STATE_CLOSED
            || st == STATE_LAST_ACK
            || st == STATE_TIME_WAIT
        {
            return Ok(0); // EOF
        }
        if p.is_nonblocking.load(Ordering::Acquire) {
            return Err(E_AGAIN);
        }
        return Err(E_AGAIN); // caller blocks externally
    }

    let tail = p.rx_tail.load(Ordering::Acquire);
    let n = avail.min(buf.len());
    for i in 0..n {
        buf[i] = p.rx_buf[(tail + i) & (RX_BUF_SIZE - 1)];
    }
    p.rx_tail.store(tail.wrapping_add(n), Ordering::Release);
    Ok(n)
}

/// Is there buffered data ready to read?
pub fn data_ready(id: usize) -> bool {
    if id >= MAX_PCBS { return false; }
    pcb(id).rx_available() > 0
}

/// Is there room in the TX path? Today we always have room (we flush
/// immediately), so this is true iff state permits sending.
pub fn can_write(id: usize) -> bool {
    if id >= MAX_PCBS { return false; }
    let p = pcb(id);
    if p.tx_shut.load(Ordering::Acquire) { return false; }
    let s = p.state.load(Ordering::Acquire);
    s == STATE_ESTABLISHED || s == STATE_CLOSE_WAIT
}

// Half-close

pub fn shutdown_write(id: usize) {
    if id >= MAX_PCBS { return; }
    let p = pcb_mut(id);
    if !p.in_use.load(Ordering::Acquire) { return; }
    if p.tx_shut.swap(true, Ordering::AcqRel) { return; } // already done

    let st = p.state.load(Ordering::Acquire);
    // NEW-DOS-008: stamp state-entered so FIN_WAIT_* / LAST_ACK get drained
    // if the peer never sends the expected ACK/FIN.
    let now: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
    match st {
        STATE_ESTABLISHED => {
            p.time_wait_entered.store(now, Ordering::Relaxed);
            p.state.store(STATE_FIN_WAIT_1, Ordering::Release);
            send_tcp_pcb(id, TCP_FIN | TCP_ACK, &[]);
        }
        STATE_CLOSE_WAIT => {
            p.time_wait_entered.store(now, Ordering::Relaxed);
            p.state.store(STATE_LAST_ACK, Ordering::Release);
            send_tcp_pcb(id, TCP_FIN | TCP_ACK, &[]);
        }
        _ => {}
    }
}

pub fn shutdown_read(id: usize) {
    if id >= MAX_PCBS { return; }
    let p = pcb(id);
    if !p.in_use.load(Ordering::Acquire) { return; }
    p.rx_shut.store(true, Ordering::Release);
}

/// Full teardown — FIN + wait briefly + mark CLOSED.
pub fn close_pcb(id: usize) {
    if id >= MAX_PCBS { return; }
    shutdown_write(id);
    // Brief drain — poll loop gives RX/ACKs a chance.
    for _ in 0..50_000 {
        super::poll_once();
        let s = pcb(id).state.load(Ordering::Acquire);
        if s == STATE_CLOSED || s == STATE_TIME_WAIT { break; }
        core::hint::spin_loop();
    }
    let p = pcb(id);
    let local_port = p.local_port;
    p.state.store(STATE_CLOSED, Ordering::Release);
    // QEMU-BUGFIX: release the PCB so `ensure_legacy_pcb()` will
    // re-initialize it on the next `connect()`. Without this, the
    // legacy PCB keeps stale tx/rx buffers, local port, and half-
    // drained state from the previous session — the second
    // `connect()` succeeds at SYN level but subsequent sends fail
    // with "send: no progress". Symptom: first docker-* command
    // works, second one returns "send failed".
    p.in_use.store(false, Ordering::Release);
    // gap-audit 045: the local ephemeral port is about to be
    // reclaimed; flush any conntrack flows that referenced it so
    // a future PCB on the same port can't inherit stale state.
    if local_port != 0 {
        let _ = crate::net::conntrack::release_local_port(local_port);
    }
}

// ===========================================================================
// Legacy synchronous API (PCB 0) — used by netsurf_test and other callers
// that predate the multi-PCB refactor.
// ===========================================================================

fn ensure_legacy_pcb() {
    let p = pcb(LEGACY_PCB);
    if !p.in_use.load(Ordering::Acquire) {
        // Manually claim slot 0 rather than calling pcb_alloc(), so we always
        // get the same slot across reconnects.
        alloc_lock();
        let p0 = pcb_mut(LEGACY_PCB);
        p0.state.store(STATE_CLOSED, Ordering::Relaxed);
        let isn = compute_isn(LEGACY_PCB, 0, 0);
        p0.snd_nxt = isn;
        p0.snd_una = isn;
        p0.rcv_nxt = 0;
        p0.rx_head.store(0, Ordering::Relaxed);
        p0.rx_tail.store(0, Ordering::Relaxed);
        p0.tx_head.store(0, Ordering::Relaxed);
        p0.tx_tail.store(0, Ordering::Relaxed);
        p0.is_nonblocking.store(false, Ordering::Relaxed);
        p0.rx_shut.store(false, Ordering::Relaxed);
        p0.tx_shut.store(false, Ordering::Relaxed);
        p0.error.store(0, Ordering::Relaxed);
        p0.fd.store(-1, Ordering::Relaxed);
        p0.in_use.store(true, Ordering::Release);
        alloc_unlock();
    }
}

/// Blocking connect on a specific PCB. Drives `connect_start +
/// connect_poll` to completion (30 s timeout). The HTTPS syscall
/// uses this on a freshly-allocated PCB; the legacy `connect`
/// wrapper calls this against `LEGACY_PCB` for backwards compat.
pub fn connect_blocking_pcb(pcb_id: usize, dst_ip: u32, dst_port: u16)
    -> Result<(), &'static str>
{
    // Force blocking on this PCB.
    pcb(pcb_id).is_nonblocking.store(false, Ordering::Relaxed);
    let rc = connect_start(pcb_id, dst_ip, dst_port);
    if rc == E_NETUNREACH {
        return Err("net isolated");
    }
    if rc != 0 && rc != E_INPROGRESS {
        return Err("connect_start failed");
    }

    crate::drivers::uart::puts("[tcp] sending SYN to ");
    crate::kernel::mm::print_num(((dst_ip >> 24) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num(((dst_ip >> 16) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num(((dst_ip >> 8) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num((dst_ip & 0xFF) as usize);
    crate::drivers::uart::puts("\n");

    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let timeout_ticks = freq * 30;
    let mut last_print = start;

    loop {
        super::poll_once();
        match connect_poll(pcb_id) {
            ConnectStatus::Established => {
                // gap-audit 045: SYN-ACK received -> flow is now
                // Established. Connection-tracker semantics: from
                // this point on, inbound segments on this 4-tuple
                // are recognised as legitimate response traffic.
                let p = pcb(pcb_id);
                crate::net::conntrack::mark_established(
                    6, p.remote_ip, p.remote_port, p.local_port,
                );
                return Ok(());
            }
            ConnectStatus::Failed(_)   => return Err("connect failed"),
            ConnectStatus::InProgress  => {}
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now - last_print > freq * 5 {
            crate::drivers::uart::puts("[tcp] waiting ");
            crate::kernel::mm::print_num(((now - start) / freq) as usize);
            crate::drivers::uart::puts("s\n");
            last_print = now;
        }
        if now - start > timeout_ticks { break; }
        core::hint::spin_loop();
    }

    pcb(pcb_id).state.store(STATE_CLOSED, Ordering::Release);
    Err("connection timed out")
}

/// Legacy blocking connect (netsurf_test path). Uses PCB 0. Thin
/// wrapper around `connect_blocking_pcb`.
pub fn connect(dst_ip: u32, dst_port: u16) -> Result<(), &'static str> {
    ensure_legacy_pcb();
    connect_blocking_pcb(LEGACY_PCB, dst_ip, dst_port)
}

/// Blocking send to a specific PCB. Loops over `send_data_pcb` until
/// `data` is fully written or the connection errors. The kernel TLS
/// stack uses this on its own PCB; the legacy `send_data` calls this
/// against `LEGACY_PCB` for backwards compat.
pub fn send_data_blocking_pcb(pcb: usize, data: &[u8]) -> Result<(), &'static str> {
    let mut off = 0;
    while off < data.len() {
        match send_data_pcb(pcb, &data[off..]) {
            Ok(0) => return Err("send: no progress"),
            Ok(n) => off += n,
            Err(_) => return Err("send failed"),
        }
    }
    Ok(())
}

/// Blocking recv on a specific PCB — blocks up to 5 seconds.
/// Same shape as the legacy `recv_data` but parameterised by PCB so
/// concurrent TLS sessions (https_open syscall) can each block on
/// their own connection.
pub fn recv_data_blocking_pcb(pcb: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let timeout = freq * 5;
    loop {
        super::poll_once();
        if data_ready(pcb) {
            // Coalesce briefly.
            let coalesce_start: u64;
            unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) coalesce_start); }
            let coalesce_timeout = freq / 10;
            loop {
                super::poll_once();
                let cn: u64;
                unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) cn); }
                if cn - coalesce_start > coalesce_timeout { break; }
                core::hint::spin_loop();
            }
            return match recv_data_pcb(pcb, buf) {
                Ok(n) => Ok(n),
                Err(_) => Err("recv failed"),
            };
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now - start > timeout { break; }
        core::hint::spin_loop();
    }
    Err("receive timeout")
}

/// Legacy synchronous send (PCB 0). Thin wrapper around
/// `send_data_blocking_pcb` for backwards compat with callers that
/// only know about the legacy single-PCB shape.
pub fn send_data(data: &[u8]) -> Result<(), &'static str> {
    let r = send_data_blocking_pcb(LEGACY_PCB, data);
    if r.is_ok() {
        // Gap-audit item 030 IO slice — attribute bytes to the
        // active cave for observability. No enforcement yet.
        crate::caves::cave::active_add_tx_bytes(data.len() as u64);
    }
    r
}

/// Legacy synchronous recv (PCB 0). Thin wrapper around
/// `recv_data_blocking_pcb`.
pub fn recv_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let r = recv_data_blocking_pcb(LEGACY_PCB, buf);
    if let Ok(n) = r {
        crate::caves::cave::active_add_rx_bytes(n as u64);
    }
    r
}

/// Legacy zero-arg data_ready() — PCB 0.
pub fn data_ready_legacy() -> bool {
    data_ready(LEGACY_PCB)
}

/// Legacy close (PCB 0).
pub fn close() {
    close_pcb(LEGACY_PCB);
}
