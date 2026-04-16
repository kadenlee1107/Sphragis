#![allow(dead_code)]
// Bat_OS — TCP Layer (Multi-PCB)
// Supports up to 64 concurrent TCP connections via a static TCB (PCB) table.
// Designed to unblock Chromium's subresource fetch parallelism.
//
// Design summary
// --------------
//   * Replace the single `CONN_STATE` / `REMOTE_IP` / ... globals with a
//     fixed-size [TcpPcb; MAX_PCBS] table. No heap.
//   * PCB fields are plain non-atomic where they are only mutated under the
//     single-threaded packet-dispatch & connect path. Shared-visibility flags
//     (state, in_use, error, nonblocking, ring head/tail) use atomics so that
//     epoll/poll callers and the RX dispatch path can coordinate.
//   * Non-blocking connect: `connect_start()` + `connect_poll()`. The legacy
//     synchronous `connect()` wraps these using PCB 0 so the existing
//     netsurf_test keeps working.
//   * Per-PCB rx/tx rings (8 KiB each) — enough for one in-flight TCP segment
//     with window room; Chromium's HTTP/1.1 pipelines fit.
//   * epoll integration: when rx_buf gains data, the PCB transitions to
//     ESTABLISHED, or tx_buf drains, we call `epoll::mark_ready(fd, ...)`.
//   * Half-close: `shutdown_write` sends a FIN but keeps RX open;
//     `shutdown_read` drops further incoming data.
//   * State machine: full 11-state table with the transitions noted in the
//     task. TIME_WAIT 2MSL timeout is a best-effort counter, not wall-clock.

use super::ip::{self, IpPacket};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// ISN randomization (ATTACK-NET-009)
// ---------------------------------------------------------------------------
//
// The previous scheme was `1000 + pcb_id * 997`, which is deterministic and
// allows blind RST / data injection as soon as the attacker observes a single
// handshake. We now derive the ISN from
//     hash(cntpct_el0 ^ remote_ip ^ remote_port ^ pcb_id ^ boot_cookie)
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

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

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

pub const MAX_PCBS: usize = 64;
pub const RX_BUF_SIZE: usize = 8192;
pub const TX_BUF_SIZE: usize = 8192;

// Legacy synchronous API uses slot 0.
const LEGACY_PCB: usize = 0;

/// Ephemeral local-port allocator. Starts at 49152 and wraps around 65535.
static NEXT_LOCAL_PORT: AtomicU32 = AtomicU32::new(49152);

fn alloc_local_port() -> u16 {
    loop {
        let p = NEXT_LOCAL_PORT.fetch_add(1, Ordering::Relaxed);
        let port = (p & 0xFFFF) as u16;
        if port < 49152 {
            // Reset into ephemeral range.
            NEXT_LOCAL_PORT.store(49152, Ordering::Relaxed);
            continue;
        }
        return port;
    }
}

// ---------------------------------------------------------------------------
// PCB (TCB)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// PCB allocation / free
// ---------------------------------------------------------------------------

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
pub fn pcb_free(id: usize) {
    if id >= MAX_PCBS { return; }
    alloc_lock();
    let p = pcb_mut(id);
    p.state.store(STATE_CLOSED, Ordering::Release);
    p.in_use.store(false, Ordering::Release);
    p.fd.store(-1, Ordering::Release);
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

// ---------------------------------------------------------------------------
// epoll notify helper
// ---------------------------------------------------------------------------

fn notify_epoll(id: usize, events: u32) {
    let fd = pcb(id).fd.load(Ordering::Acquire);
    if fd >= 0 {
        crate::batcave::linux::epoll::mark_ready(fd, events);
    }
}

// ---------------------------------------------------------------------------
// Drain zombie PCBs: any non-ESTABLISHED, non-LISTEN PCB that has been in
// its current transient state longer than the per-state limit transitions
// to CLOSED and releases its slot.
//
// NEW-DOS-008: previously this only covered TIME_WAIT. A remote peer
// that dropped our SYN silently (or sent FIN and then vanished) would
// pin the PCB in SYN_SENT / FIN_WAIT_* until reboot — 64 such hangs
// drained the whole TCP table.
// ---------------------------------------------------------------------------
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
                crate::batcave::linux::quotas::refund_active(
                    crate::batcave::linux::quotas::Resource::Sockets, 1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Incoming packet dispatch
// ---------------------------------------------------------------------------

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
    if !crate::net::firewall::allow_inbound_tcp(pkt.src, src_port) {
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
        None => return, // unmatched segment — ignore (no RST to keep it simple)
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
                notify_epoll(id, EPOLLOUT);
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

// ---------------------------------------------------------------------------
// Segment transmit
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Unit tests (run on the host, not the kernel target)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Non-blocking connect
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Per-PCB I/O
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Half-close
// ---------------------------------------------------------------------------

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
    pcb(id).state.store(STATE_CLOSED, Ordering::Release);
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

/// Legacy blocking connect (netsurf_test path). Uses PCB 0.
pub fn connect(dst_ip: u32, dst_port: u16) -> Result<(), &'static str> {
    ensure_legacy_pcb();
    // Force blocking.
    pcb(LEGACY_PCB).is_nonblocking.store(false, Ordering::Relaxed);
    let rc = connect_start(LEGACY_PCB, dst_ip, dst_port);
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
        match connect_poll(LEGACY_PCB) {
            ConnectStatus::Established => return Ok(()),
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

    pcb(LEGACY_PCB).state.store(STATE_CLOSED, Ordering::Release);
    Err("connection timed out")
}

/// Legacy synchronous send (PCB 0).
pub fn send_data(data: &[u8]) -> Result<(), &'static str> {
    let mut off = 0;
    while off < data.len() {
        match send_data_pcb(LEGACY_PCB, &data[off..]) {
            Ok(0) => return Err("send: no progress"),
            Ok(n) => off += n,
            Err(_) => return Err("send failed"),
        }
    }
    Ok(())
}

/// Legacy synchronous recv (PCB 0) — blocks up to 5 seconds.
pub fn recv_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let timeout = freq * 5;
    loop {
        super::poll_once();
        if data_ready(LEGACY_PCB) {
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
            return match recv_data_pcb(LEGACY_PCB, buf) {
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

/// Legacy zero-arg data_ready() — PCB 0.
pub fn data_ready_legacy() -> bool {
    data_ready(LEGACY_PCB)
}

/// Legacy close (PCB 0).
pub fn close() {
    close_pcb(LEGACY_PCB);
}
