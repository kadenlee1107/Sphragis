// Bat_OS — BSD Socket Bridge Layer
// Routes BSD-socket API calls into our native TCP stack (src/net/tcp.rs)
// and a stub UDP layer. This layer is intended to be called from the Linux
// syscall dispatcher once wiring is done. No heap; no std; fixed-size static
// table of 128 socket slots.
//
// IMPORTANT: Chromium (BoringSSL in-binary) does TLS in userspace. This layer
// intentionally provides RAW TCP only.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// Constants: address families, types, protocols, errno
// ============================================================================

pub const AF_UNIX:  i32 = 1;
pub const AF_INET:  i32 = 2;
pub const AF_INET6: i32 = 10;

pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM:  i32 = 2;

// Linux-style flag bits OR'd into sock_type
pub const SOCK_NONBLOCK: i32 = 0o4000;
pub const SOCK_CLOEXEC:  i32 = 0o2000000;
pub const SOCK_TYPE_MASK: i32 = 0xFF;

pub const IPPROTO_IP:  i32 = 0;
pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;

// Sockopt levels
pub const SOL_SOCKET: i32 = 1;
// IPPROTO_IP reused (0), IPPROTO_TCP reused (6)

// SOL_SOCKET optnames (Linux)
pub const SO_REUSEADDR: i32 = 2;
pub const SO_TYPE:      i32 = 3;
pub const SO_ERROR:     i32 = 4;
pub const SO_BROADCAST: i32 = 6;
pub const SO_SNDBUF:    i32 = 7;
pub const SO_RCVBUF:    i32 = 8;
pub const SO_KEEPALIVE: i32 = 9;
pub const SO_LINGER:    i32 = 13;
pub const SO_REUSEPORT: i32 = 15;
pub const SO_RCVTIMEO:  i32 = 20;
pub const SO_SNDTIMEO:  i32 = 21;

// IPPROTO_TCP optnames
pub const TCP_NODELAY:  i32 = 1;

// IPPROTO_IP optnames
pub const IP_TOS:       i32 = 1;
pub const IP_TTL:       i32 = 2;

// shutdown(how)
pub const SHUT_RD:   i32 = 0;
pub const SHUT_WR:   i32 = 1;
pub const SHUT_RDWR: i32 = 2;

// errno (negated for return values per Linux syscall ABI)
pub const EPERM:        i64 = -1;
pub const EBADF:        i64 = -9;
pub const EAGAIN:       i64 = -11;
pub const EWOULDBLOCK:  i64 = -11;
pub const ENOMEM:       i64 = -12;
pub const EFAULT:       i64 = -14;
pub const EINVAL:       i64 = -22;
pub const EMFILE:       i64 = -24;
pub const ENOTSOCK:     i64 = -88;
pub const EMSGSIZE:     i64 = -90;
pub const EPROTONOSUPPORT: i64 = -93;
pub const EAFNOSUPPORT: i64 = -97;
pub const EADDRINUSE:   i64 = -98;
pub const EADDRNOTAVAIL:i64 = -99;
pub const ENETUNREACH:  i64 = -101;
pub const ECONNABORTED: i64 = -103;
pub const ECONNRESET:   i64 = -104;
pub const ENOBUFS:      i64 = -105;
pub const EISCONN:      i64 = -106;
pub const ENOTCONN:     i64 = -107;
pub const ETIMEDOUT:    i64 = -110;
pub const ECONNREFUSED: i64 = -111;
pub const EINPROGRESS:  i64 = -115;
pub const EOPNOTSUPP:   i64 = -95;

// ============================================================================
// C-ABI structs
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SockaddrIn {
    pub sin_family: u16,
    pub sin_port:   u16, // big-endian
    pub sin_addr:   u32, // big-endian
    pub sin_zero:   [u8; 8],
}

impl SockaddrIn {
    pub const fn zeroed() -> Self {
        SockaddrIn { sin_family: 0, sin_port: 0, sin_addr: 0, sin_zero: [0; 8] }
    }
    pub const SIZE: u32 = 16;
}

/// iovec (scatter/gather I/O)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Iovec {
    pub iov_base: *mut u8,
    pub iov_len:  usize,
}

/// msghdr (sendmsg/recvmsg)
#[repr(C)]
pub struct Msghdr {
    pub msg_name:       *mut u8,   // SockaddrIn* or null
    pub msg_namelen:    u32,
    pub msg_iov:        *mut Iovec,
    pub msg_iovlen:     usize,
    pub msg_control:    *mut u8,
    pub msg_controllen: usize,
    pub msg_flags:      i32,
}

// ============================================================================
// Byte-order helpers (inline, per constraints)
// ============================================================================

#[inline(always)] pub const fn htons(x: u16) -> u16 { x.to_be() }
#[inline(always)] pub const fn ntohs(x: u16) -> u16 { u16::from_be(x) }
#[inline(always)] pub const fn htonl(x: u32) -> u32 { x.to_be() }
#[inline(always)] pub const fn ntohl(x: u32) -> u32 { u32::from_be(x) }

// ============================================================================
// Socket state machine
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SockKind {
    Tcp,
    Udp,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SockStatus {
    Unbound,      // just created
    Bound,        // bind() done
    Listening,    // listen() done
    Connecting,   // connect() in progress (non-blocking)
    Connected,    // established
    Closed,       // shutdown or peer closed
    Error,        // terminal failure
}

/// Per-socket state; stored in the static SOCKET_TABLE.
#[derive(Clone, Copy)]
pub struct SocketState {
    pub in_use:     bool,
    pub kind:       SockKind,
    pub status:     SockStatus,
    pub nonblock:   bool,
    pub cloexec:    bool,

    // Local endpoint (host byte order)
    pub local_addr: u32,
    pub local_port: u16,

    // Peer endpoint (host byte order)
    pub peer_addr:  u32,
    pub peer_port:  u16,

    // Options Chromium frequently sets. Stored but largely advisory — our TCP
    // stack doesn't honor most yet (see gaps list in task report).
    pub opt_reuseaddr:  bool,
    pub opt_reuseport:  bool,
    pub opt_keepalive:  bool,
    pub opt_tcp_nodelay:bool,
    pub opt_broadcast:  bool,
    pub opt_sndbuf:     i32,
    pub opt_rcvbuf:     i32,
    pub opt_ip_tos:     i32,
    pub opt_ip_ttl:     i32,
    pub opt_linger_on:  bool,
    pub opt_linger_sec: i32,

    // Pending asynchronous error surface (for SO_ERROR).
    pub so_error:   i32,

    // listen() backlog (stubbed — we can't actually accept inbound yet).
    pub backlog:    i32,

    // True once connect() has driven the global TCP stack into an
    // established state (kept for API/ABI compatibility; not used in the
    // multi-PCB stack — each socket has its own `pcb_id` below).
    pub owns_global_tcp: bool,

    /// TCP-only: slot in the per-connection PCB table (src/net/tcp.rs).
    /// `-1` means no PCB allocated yet (e.g. unbound UDP socket).
    pub pcb_id: i32,
}

impl SocketState {
    const fn empty() -> Self {
        SocketState {
            in_use: false,
            kind: SockKind::Unknown,
            status: SockStatus::Unbound,
            nonblock: false,
            cloexec: false,
            local_addr: 0,
            local_port: 0,
            peer_addr: 0,
            peer_port: 0,
            opt_reuseaddr: false,
            opt_reuseport: false,
            opt_keepalive: false,
            opt_tcp_nodelay: false,
            opt_broadcast: false,
            opt_sndbuf: 65536,
            opt_rcvbuf: 65536,
            opt_ip_tos: 0,
            opt_ip_ttl: 64,
            opt_linger_on: false,
            opt_linger_sec: 0,
            so_error: 0,
            backlog: 0,
            owns_global_tcp: false,
            pcb_id: -1,
        }
    }
}

// ============================================================================
// Static socket table (128 slots, no heap)
// ============================================================================

pub const MAX_SOCKETS: usize = 128;

/// Socket slot array. `SOCKET_FD_BASE + slot_index` is the fd number we hand
/// back to userspace. We deliberately use a high base so it never collides
/// with the regular VFS fd table in `fd.rs` (which uses 0..MAX_FDS=64).
pub const SOCKET_FD_BASE: i32 = 1024;

static mut SOCKET_TABLE: [SocketState; MAX_SOCKETS] =
    [SocketState::empty(); MAX_SOCKETS];

/// V6-XLAYER-005/006: clear every socket slot on cave switch so the
/// new tenant doesn't inherit socket bindings or live TCP PCB ids.
///
/// V8-ROOT-1: IRQ-masked for the duration. lock()/unlock() alone wasn't
/// enough — an IRQ between lock and unlock could schedule another thread
/// that observed the half-wiped table.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    lock();
    unsafe {
        for i in 0..MAX_SOCKETS {
            SOCKET_TABLE[i] = SocketState::empty();
        }
    }
    unlock();
}

// Serializes allocation/free. Not a real lock — we're single-core right now
// and syscall entries are non-reentrant, but the flag catches logic bugs.
static SOCK_LOCK: AtomicBool = AtomicBool::new(false);

fn lock() {
    while SOCK_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}
fn unlock() { SOCK_LOCK.store(false, Ordering::Release); }

/// True if the given fd is in the socket-fd range.
pub fn is_socket_fd(fd: i32) -> bool {
    fd >= SOCKET_FD_BASE && (fd as usize) < (SOCKET_FD_BASE as usize + MAX_SOCKETS)
}

fn slot_of(fd: i32) -> Option<usize> {
    if !is_socket_fd(fd) { return None; }
    Some((fd - SOCKET_FD_BASE) as usize)
}

fn alloc_slot() -> Option<usize> {
    lock();
    let mut out = None;
    unsafe {
        for i in 0..MAX_SOCKETS {
            if !SOCKET_TABLE[i].in_use {
                SOCKET_TABLE[i] = SocketState::empty();
                SOCKET_TABLE[i].in_use = true;
                out = Some(i);
                break;
            }
        }
    }
    unlock();
    out
}

fn free_slot(i: usize) {
    lock();
    unsafe {
        if i < MAX_SOCKETS {
            SOCKET_TABLE[i] = SocketState::empty();
        }
    }
    unlock();
}

/// Borrow a socket slot by fd. Returns EBADF/ENOTSOCK if not ours.
fn with_slot<F, R>(fd: i32, f: F) -> Result<R, i64>
where F: FnOnce(&mut SocketState) -> Result<R, i64>,
{
    let idx = slot_of(fd).ok_or(ENOTSOCK)?;
    unsafe {
        if !SOCKET_TABLE[idx].in_use { return Err(EBADF); }
        f(&mut SOCKET_TABLE[idx])
    }
}

/// Public read-only view of a socket — used by future epoll wiring.
pub fn peek(fd: i32) -> Option<SocketState> {
    let idx = slot_of(fd)?;
    unsafe {
        if !SOCKET_TABLE[idx].in_use { return None; }
        Some(SOCKET_TABLE[idx])
    }
}

// ============================================================================
// Pointer-safety helpers
// ============================================================================

fn read_sockaddr(addr: *const SockaddrIn, len: u32) -> Result<(u32, u16, u16), i64> {
    if addr.is_null() { return Err(EFAULT); }
    if (len as usize) < core::mem::size_of::<SockaddrIn>() { return Err(EINVAL); }
    // NEW-SYS-040: validate the pointer lives in userspace. Previously this
    // was a read oracle — an attacker passing `&kernel_state` could recover
    // the bytes via whatever the syscall did with (ip, port, family).
    if !is_user(addr as usize, core::mem::size_of::<SockaddrIn>()) {
        return Err(EFAULT);
    }
    let sa = unsafe { core::ptr::read_unaligned(addr) };
    Ok((ntohl(sa.sin_addr), ntohs(sa.sin_port), sa.sin_family))
}

fn write_sockaddr(addr: *mut SockaddrIn, len: *mut u32, ip_host: u32, port_host: u16) -> Result<(), i64> {
    if addr.is_null() || len.is_null() { return Err(EFAULT); }
    // NEW-SYS-041: gate both writes. The SockaddrIn struct is 16 bytes;
    // the length word is 4 bytes.
    if !is_user(addr as usize, core::mem::size_of::<SockaddrIn>()) {
        return Err(EFAULT);
    }
    if !is_user(len as usize, 4) {
        return Err(EFAULT);
    }
    let available = unsafe { core::ptr::read_unaligned(len) };
    if (available as usize) < core::mem::size_of::<SockaddrIn>() {
        // Linux truncates; we mimic by writing what fits, but flag EINVAL for now.
        return Err(EINVAL);
    }
    let sa = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port:   htons(port_host),
        sin_addr:   htonl(ip_host),
        sin_zero:   [0; 8],
    };
    unsafe {
        core::ptr::write_unaligned(addr, sa);
        core::ptr::write_unaligned(len, SockaddrIn::SIZE);
    }
    Ok(())
}

// ============================================================================
// Public API — BSD socket functions
// ============================================================================

/// socket(2)
pub fn socket(domain: i32, sock_type: i32, protocol: i32) -> i64 {
    // Only IPv4 for now. IPv6 and UNIX return EAFNOSUPPORT; Chromium falls back.
    if domain != AF_INET {
        return EAFNOSUPPORT;
    }

    let flags = sock_type & !SOCK_TYPE_MASK;
    let base_type = sock_type & SOCK_TYPE_MASK;

    let kind = match base_type {
        SOCK_STREAM => {
            if protocol != 0 && protocol != IPPROTO_TCP { return EPROTONOSUPPORT; }
            SockKind::Tcp
        }
        SOCK_DGRAM => {
            if protocol != 0 && protocol != IPPROTO_UDP { return EPROTONOSUPPORT; }
            SockKind::Udp
        }
        _ => return EPROTONOSUPPORT,
    };

    let idx = match alloc_slot() {
        Some(i) => i,
        None => return EMFILE,
    };

    unsafe {
        let s = &mut SOCKET_TABLE[idx];
        s.kind = kind;
        s.status = SockStatus::Unbound;
        s.nonblock = (flags & SOCK_NONBLOCK) != 0;
        s.cloexec  = (flags & SOCK_CLOEXEC)  != 0;
    }

    (SOCKET_FD_BASE + idx as i32) as i64
}

/// bind(2) — record local address/port. Our TCP stack picks ephemeral ports
/// itself on connect(), so binding is advisory for outbound sockets.
pub fn bind(fd: i32, addr: *const SockaddrIn, len: u32) -> i64 {
    let (ip, port, family) = match read_sockaddr(addr, len) {
        Ok(v) => v, Err(e) => return e,
    };
    if family != AF_INET as u16 { return EAFNOSUPPORT; }

    match with_slot(fd, |s| {
        if s.status != SockStatus::Unbound && s.status != SockStatus::Bound {
            return Err(EINVAL);
        }
        s.local_addr = ip;
        s.local_port = port;
        s.status = SockStatus::Bound;
        Ok(0i64)
    }) {
        Ok(r) => r, Err(e) => e,
    }
}

/// listen(2)
///
/// STUMP #148: real server-side listen wired through to
/// `tcp::listen_register`. The Slot bookkeeping (status=Listening,
/// backlog) stays — those drive sockopt/getsockname behavior and
/// epoll readability checks — but now we ALSO register a `Listener`
/// in the TCP stack so the SYN-on-LISTEN path in
/// `tcp::handle_incoming` knows which port to accept on.
///
/// Errors:
///   EOPNOTSUPP — non-TCP socket
///   EINVAL     — socket not in Unbound/Bound state, or no port
///                bound yet (listen requires bind first)
///   EADDRINUSE — another listener already owns this port
///   EMFILE     — kernel listener table full (MAX_LISTENERS=16)
pub fn listen(fd: i32, backlog: i32) -> i64 {
    // First validate + lock in the Slot side. Capture the port so we
    // can call into the TCP stack OUTSIDE the with_slot lock (the
    // listen_register call takes its own alloc_lock and we don't want
    // nested lock acquisition).
    let port = match with_slot(fd, |s| {
        if s.kind != SockKind::Tcp { return Err(EOPNOTSUPP); }
        if s.status != SockStatus::Bound && s.status != SockStatus::Unbound {
            return Err(EINVAL);
        }
        // listen() requires a bound port. If the caller never bind()ed
        // a port, this is EINVAL — Linux returns the same.
        if s.local_port == 0 { return Err(EINVAL); }
        s.backlog = backlog.max(1);
        // Don't flip status to Listening yet — only after the TCP
        // listener registration succeeds. Otherwise a follow-up
        // accept() on a "Listening" Slot with no kernel listener
        // would EAGAIN forever.
        Ok(s.local_port)
    }) {
        Ok(port) => port,
        Err(e) => return e,
    };

    // Register with the TCP stack. cave_id 0 is a placeholder until
    // STUMP #148 step 6 wires per-cave inbound policy.
    let cave_id = 0u16;
    match crate::net::tcp::listen_register(port, backlog, cave_id, fd) {
        Ok(_idx) => {
            // TCP side accepted — flip Slot status to Listening.
            let _ = with_slot(fd, |s| {
                s.status = SockStatus::Listening;
                Ok(0i64)
            });
            0
        }
        Err("EADDRINUSE") => EADDRINUSE,
        Err("EMFILE")     => EMFILE,
        Err(_)            => EINVAL,
    }
}

/// accept4(2) — STUMP #148 step 4: real server-side accept.
///
/// Validates the listening fd, looks up the matching kernel
/// `Listener`, and pops a completed-handshake PCB off its accept
/// queue. Returns a fresh socket fd whose Slot is bound to that PCB
/// in the Connected state. The peer's address is written to
/// `addr`/`*len` if non-null.
///
/// Blocking semantics: if the queue is empty AND the listening
/// socket is blocking, spin-poll the network stack for up to 30s
/// waiting for a SYN_RECV → ESTABLISHED transition to enqueue
/// something. Non-blocking sockets get EAGAIN immediately.
///
/// Errors:
///   EOPNOTSUPP — non-TCP fd
///   EINVAL     — fd not in Listening state, or no kernel listener
///                bound to it (shouldn't happen if listen() succeeded)
///   EAGAIN     — non-blocking and queue empty
///   EMFILE     — couldn't allocate a new socket Slot
pub fn accept4(fd: i32, addr: *mut SockaddrIn, len: *mut u32, flags: i32) -> i64 {
    // Validate listening + capture nonblock.
    let nonblock_listener = match with_slot(fd, |s| {
        if s.kind != SockKind::Tcp { return Err(EOPNOTSUPP); }
        if s.status != SockStatus::Listening { return Err(EINVAL); }
        Ok(s.nonblock)
    }) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // Find the kernel Listener for this fd.
    let listener_idx = match crate::net::tcp::listener_lookup_by_fd(fd) {
        Some(i) => i,
        None => return EINVAL, // sockets::listen succeeded but no kernel listener? shouldn't happen
    };

    // Pop a ready PCB. If none and we're blocking, spin-poll the
    // network stack until one shows up or 30s elapses.
    let pcb_id = if let Some(id) = crate::net::tcp::listener_accept_pop(listener_idx) {
        id
    } else if nonblock_listener {
        return EAGAIN;
    } else {
        // Blocking accept: spin with a 30s deadline.
        let start: u64;
        let freq: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
            core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        }
        let deadline = start + freq * 30;
        let mut popped: Option<usize> = None;
        loop {
            crate::net::poll_once();
            if let Some(id) = crate::net::tcp::listener_accept_pop(listener_idx) {
                popped = Some(id);
                break;
            }
            let now: u64;
            unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
            if now > deadline { break; }
            core::hint::spin_loop();
        }
        match popped {
            Some(id) => id,
            None => return EAGAIN, // 30s timeout — Linux uses ETIMEDOUT but EAGAIN is also valid here
        }
    };

    // Allocate a new socket Slot for the accepted connection.
    let new_slot = match alloc_slot() {
        Some(i) => i,
        None => {
            // Couldn't allocate a Slot — RST the PCB so the peer's
            // server doesn't think the conn is established.
            crate::net::tcp::close_pcb(pcb_id);
            return EMFILE;
        }
    };

    let new_fd = SOCKET_FD_BASE + new_slot as i32;
    let new_nonblock = (flags & SOCK_NONBLOCK) != 0;
    let new_cloexec  = (flags & SOCK_CLOEXEC)  != 0;

    // Populate the new Slot.
    let _ = with_slot(new_fd, |s| {
        s.kind = SockKind::Tcp;
        s.status = SockStatus::Connected;
        s.nonblock = new_nonblock;
        s.cloexec = new_cloexec;
        s.pcb_id = pcb_id as i32;
        // local_port matches the listener's port (this PCB inherits
        // the bound port from the SYN-on-LISTEN allocation).
        let (rip, rport) = crate::net::tcp::pcb_remote(pcb_id);
        s.local_port = 0; // server doesn't track its own port per-conn;
                          // use the listener's port if needed via getsockname
        s.peer_addr = rip;
        s.peer_port = rport;
        Ok(0i64)
    });

    // Tie the PCB to the new fd so future read/write/close from
    // this fd reaches this PCB.
    crate::net::tcp::pcb_bind_fd(pcb_id, new_fd);
    crate::net::tcp::pcb_set_nonblocking(pcb_id, new_nonblock);

    // Fill the caller's sockaddr_in if they passed one. POSIX:
    // addr may be NULL meaning "don't care," in which case we
    // skip the write entirely.
    if !addr.is_null() && !len.is_null() {
        let (rip, rport) = crate::net::tcp::pcb_remote(pcb_id);
        let _ = write_sockaddr(addr, len, rip, rport);
    }

    new_fd as i64
}

/// connect(2)
///
/// Blocking semantics: drives our global TCP stack through a SYN handshake
/// (which internally polls with a timeout) and returns 0 on success.
///
/// Non-blocking semantics: we mark the socket Connecting and return
/// EINPROGRESS, but currently tcp::connect() is synchronous so we *do*
/// actually block for the handshake in both cases. This is a GAP.
///
/// TODO(tcp): non-blocking SYN — expose a tcp::connect_start() that returns
/// immediately and a tcp::connect_poll() that returns Pending/Ok/Err.
///
/// CRITICAL GAP: src/net/tcp.rs holds a single global connection. Opening a
/// second socket and connect()ing will clobber the first. This layer guards
/// against that by refusing a second concurrent TCP connect.
pub fn connect(fd: i32, addr: *const SockaddrIn, len: u32) -> i64 {
    let (ip, port, family) = match read_sockaddr(addr, len) {
        Ok(v) => v, Err(e) => return e,
    };
    if family != AF_INET as u16 { return EAFNOSUPPORT; }

    // Snapshot & validate.
    let (kind, status, nonblock) = match with_slot(fd, |s| {
        Ok((s.kind, s.status, s.nonblock))
    }) {
        Ok(v) => v, Err(e) => return e,
    };

    match kind {
        SockKind::Udp => {
            // UDP connect just records the peer.
            return match with_slot(fd, |s| {
                s.peer_addr = ip;
                s.peer_port = port;
                s.status = SockStatus::Connected;
                Ok(0i64)
            }) { Ok(r) => r, Err(e) => e };
        }
        SockKind::Tcp => {}
        _ => return EOPNOTSUPP,
    }

    if status == SockStatus::Connected { return EISCONN; }

    // Allocate a PCB for this socket (multi-PCB stack — up to 64 concurrent).
    let pcb_id = match crate::net::tcp::pcb_alloc() {
        Some(i) => i,
        None => return EMFILE, // out of PCB slots
    };
    crate::net::tcp::pcb_bind_fd(pcb_id, fd);
    crate::net::tcp::pcb_set_nonblocking(pcb_id, nonblock);

    let _ = with_slot(fd, |s| {
        s.peer_addr = ip;
        s.peer_port = port;
        s.status = SockStatus::Connecting;
        s.owns_global_tcp = true; // legacy field; harmless here
        s.pcb_id = pcb_id as i32;
        Ok(())
    });

    // Kick off the SYN.
    let rc = crate::net::tcp::connect_start(pcb_id, ip, port);

    if nonblock {
        // Report EINPROGRESS to userspace; epoll on this fd will deliver
        // EPOLLOUT when the handshake completes (or EPOLLERR on failure).
        if rc != 0 && rc != crate::net::tcp::E_INPROGRESS as i32 {
            let _ = with_slot(fd, |s| {
                s.status = SockStatus::Error;
                s.so_error = rc;
                Ok(())
            });
            return -(rc as i64);
        }
        return EINPROGRESS;
    }

    // Blocking: spin until established, failed, or 30s timeout.
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let deadline = start + freq * 30;
    loop {
        crate::net::poll_once();
        match crate::net::tcp::connect_poll(pcb_id) {
            crate::net::tcp::ConnectStatus::Established => {
                let _ = with_slot(fd, |s| {
                    s.status = SockStatus::Connected;
                    Ok(())
                });
                return 0;
            }
            crate::net::tcp::ConnectStatus::Failed(e) => {
                let _ = with_slot(fd, |s| {
                    s.status = SockStatus::Error;
                    s.so_error = e;
                    s.owns_global_tcp = false;
                    Ok(())
                });
                crate::net::tcp::pcb_free(pcb_id);
                return ECONNREFUSED;
            }
            crate::net::tcp::ConnectStatus::InProgress => {}
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now > deadline { break; }
        core::hint::spin_loop();
    }

    let _ = with_slot(fd, |s| {
        s.status = SockStatus::Error;
        s.so_error = -(ETIMEDOUT as i32);
        s.owns_global_tcp = false;
        Ok(())
    });
    crate::net::tcp::pcb_free(pcb_id);
    ETIMEDOUT
}

/// sendto(2) — for connected TCP, dest_addr is ignored.
pub fn sendto(fd: i32, buf: *const u8, len: usize, _flags: i32,
              dest_addr: *const SockaddrIn, addrlen: u32) -> i64 {
    if buf.is_null() && len > 0 { return EFAULT; }

    // V5-XLAYER-013 fix: refuse non-user `buf` before constructing a
    // slice over it. Previously `from_raw_parts(buf=0x40000000, len)`
    // would read kernel BSS and TLS-encrypt it over the wire, giving
    // a net-capped cave an exfil-any-kernel-state primitive.
    if len > 0 && !is_user(buf as usize, len) {
        return EFAULT;
    }

    let (kind, status, nonblock, peer_ip, peer_port, pcb_id) = match with_slot(fd, |s| {
        Ok((s.kind, s.status, s.nonblock, s.peer_addr, s.peer_port, s.pcb_id))
    }) { Ok(v) => v, Err(e) => return e };

    // SAFETY: trust boundary as above.
    let data = unsafe { core::slice::from_raw_parts(buf, len) };

    match kind {
        SockKind::Tcp => {
            if status != SockStatus::Connected { return ENOTCONN; }
            if pcb_id < 0 { return ENOTCONN; }
            let _ = nonblock;
            // Drain in chunks: send_data_pcb caps at one MSS per call.
            let mut off = 0;
            while off < data.len() {
                match crate::net::tcp::send_data_pcb(pcb_id as usize, &data[off..]) {
                    Ok(0) => break,
                    Ok(n) => off += n,
                    Err(_) => {
                        if off > 0 { return off as i64; }
                        return ECONNRESET;
                    }
                }
            }
            off as i64
        }
        SockKind::Udp => {
            // Use dest_addr if provided, else connected peer.
            let (dst_ip, dst_port) = if !dest_addr.is_null() && addrlen >= SockaddrIn::SIZE {
                match read_sockaddr(dest_addr, addrlen) {
                    Ok((ip, p, fam)) => {
                        if fam != AF_INET as u16 { return EAFNOSUPPORT; }
                        (ip, p)
                    }
                    Err(e) => return e,
                }
            } else {
                if peer_ip == 0 { return ENOTCONN; }
                (peer_ip, peer_port)
            };
            // Ephemeral source port: reuse local_port or fabricate.
            let src_port = {
                let lp = with_slot(fd, |s| Ok(s.local_port)).unwrap_or(0);
                if lp != 0 { lp } else { 49152 }
            };
            match crate::net::udp::send(dst_ip, src_port, dst_port, data) {
                Ok(()) => len as i64,
                Err(_) => ENETUNREACH,
            }
        }
        _ => EOPNOTSUPP,
    }
}

/// recvfrom(2)
pub fn recvfrom(fd: i32, buf: *mut u8, len: usize, _flags: i32,
                src_addr: *mut SockaddrIn, addrlen: *mut u32) -> i64 {
    if buf.is_null() && len > 0 { return EFAULT; }
    // V5-XLAYER-013 companion: recvfrom writes to `buf`. Without this
    // check, a cave could pass buf=0x40000000 and have the kernel TLS-
    // decrypt incoming bytes into kernel BSS — arbitrary write via
    // attacker-controlled TCP payload.
    if len > 0 && !is_user(buf as usize, len) { return EFAULT; }

    let (kind, status, nonblock, peer_ip, peer_port, pcb_id) = match with_slot(fd, |s| {
        Ok((s.kind, s.status, s.nonblock, s.peer_addr, s.peer_port, s.pcb_id))
    }) { Ok(v) => v, Err(e) => return e };

    match kind {
        SockKind::Tcp => {
            if status != SockStatus::Connected { return ENOTCONN; }
            if pcb_id < 0 { return ENOTCONN; }
            let id = pcb_id as usize;

            // Nonblocking: return EAGAIN if nothing buffered (and not EOF).
            if nonblock && !crate::net::tcp::data_ready(id) {
                let st = crate::net::tcp::pcb_state(id);
                if st == crate::net::tcp::STATE_CLOSE_WAIT
                   || st == crate::net::tcp::STATE_CLOSED
                {
                    return 0; // EOF
                }
                return EAGAIN;
            }

            // Blocking: spin with poll_once() until data arrives or EOF.
            if !nonblock {
                let start: u64;
                let freq: u64;
                unsafe {
                    core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
                    core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
                }
                let deadline = start + freq * 5;
                loop {
                    crate::net::poll_once();
                    if crate::net::tcp::data_ready(id) { break; }
                    let st = crate::net::tcp::pcb_state(id);
                    if st == crate::net::tcp::STATE_CLOSE_WAIT
                       || st == crate::net::tcp::STATE_CLOSED
                    {
                        return 0;
                    }
                    let now: u64;
                    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
                    if now > deadline { return 0; }
                    core::hint::spin_loop();
                }
            }

            // SAFETY: trust boundary.
            let out = unsafe { core::slice::from_raw_parts_mut(buf, len) };
            match crate::net::tcp::recv_data_pcb(id, out) {
                Ok(n) => {
                    if !src_addr.is_null() && !addrlen.is_null() {
                        let _ = write_sockaddr(src_addr, addrlen, peer_ip, peer_port);
                    }
                    n as i64
                }
                Err(_) => {
                    if nonblock { EAGAIN } else { 0 }
                }
            }
        }
        SockKind::Udp => {
            // TODO(udp): no per-socket receive queue. src/net/udp.rs dumps
            // inbound into a syscall-layer ring (UDP_RX_BUF). Wiring that
            // here requires plumbing we don't have yet.
            let _ = peer_ip; let _ = peer_port;
            EAGAIN
        }
        _ => EOPNOTSUPP,
    }
}

// ATTACK-SYS-014/015/016 fix: msghdr, iovec array, and each iov_base must
// all live in userspace. Previously these were dereferenced raw, which gave
// renderers a full kernel-read (sendmsg) and kernel-write (recvmsg) primitive
// via attacker-chosen iovec bases. We cap the iovec count at IOV_MAX to bound
// the kernel work per call.
const IOV_MAX: usize = 32;

fn is_user(p: usize, n: usize) -> bool {
    if n == 0 { return p != 0; }
    crate::batcave::linux::uaccess::is_user_range(p, n)
}

/// sendmsg(2) — iterates iovecs and calls sendto() per-vec.
pub fn sendmsg(fd: i32, msg: *const Msghdr, flags: i32) -> i64 {
    if msg.is_null() { return EFAULT; }
    if !is_user(msg as usize, core::mem::size_of::<Msghdr>()) { return EFAULT; }
    // Copy the msghdr out of user memory before using it, so concurrent
    // writes from another cave-thread can't race us (TOCTOU).
    let m: Msghdr = unsafe { core::ptr::read(msg) };
    if m.msg_iovlen > IOV_MAX { return EINVAL; }
    if m.msg_iov.is_null() && m.msg_iovlen > 0 { return EFAULT; }

    let iov_bytes = m.msg_iovlen.checked_mul(core::mem::size_of::<Iovec>())
        .unwrap_or(usize::MAX);
    if m.msg_iovlen > 0
        && !is_user(m.msg_iov as usize, iov_bytes)
    {
        return EFAULT;
    }

    let dest = if !m.msg_name.is_null() {
        if !is_user(m.msg_name as usize, m.msg_namelen as usize) {
            return EFAULT;
        }
        m.msg_name as *const SockaddrIn
    } else {
        core::ptr::null()
    };
    let dlen = m.msg_namelen;

    // V8-ROOT-8: snapshot the iovec array into a kernel stack buffer to
    // defeat TOCTOU. Without this, a sibling cave-thread can mutate the
    // user-space iov entries between our bounds check and the actual
    // sendto() — turning the gate into a no-op.
    let mut iov_snap: [Iovec; IOV_MAX] = [Iovec { iov_base: core::ptr::null_mut(), iov_len: 0 }; IOV_MAX];
    for i in 0..m.msg_iovlen {
        iov_snap[i] = unsafe { core::ptr::read(m.msg_iov.add(i)) };
    }
    let mut total: i64 = 0;
    for iv in iov_snap[..m.msg_iovlen].iter() {
        if iv.iov_len == 0 { continue; }
        if iv.iov_base.is_null() { return EFAULT; }
        if !is_user(iv.iov_base as usize, iv.iov_len) { return EFAULT; }
        let r = sendto(fd, iv.iov_base, iv.iov_len, flags, dest, dlen);
        if r < 0 {
            return if total > 0 { total } else { r };
        }
        total += r;
        if (r as usize) < iv.iov_len { break; } // short write
    }
    total
}

/// recvmsg(2) — fills iovecs sequentially from a single recvfrom() call.
/// (True gather-recv would need peek+partition; TCP is a byte stream so
/// sequential fill is fine.)
pub fn recvmsg(fd: i32, msg: *mut Msghdr, flags: i32) -> i64 {
    if msg.is_null() { return EFAULT; }
    if !is_user(msg as usize, core::mem::size_of::<Msghdr>()) { return EFAULT; }

    // V8-ROOT-8: snapshot the entire msghdr — we previously held a `&mut`
    // pointing into user memory across recvfrom() calls, which let a sibling
    // thread mutate iov_base / iov_len after our bounds check (TOCTOU →
    // arbitrary kernel write).
    let m: Msghdr = unsafe { core::ptr::read(msg) };
    if m.msg_iovlen > IOV_MAX { return EINVAL; }
    if m.msg_iov.is_null() && m.msg_iovlen > 0 { return EFAULT; }

    let iov_bytes = m.msg_iovlen.checked_mul(core::mem::size_of::<Iovec>())
        .unwrap_or(usize::MAX);
    if m.msg_iovlen > 0
        && !is_user(m.msg_iov as usize, iov_bytes)
    {
        return EFAULT;
    }

    let name = if !m.msg_name.is_null() {
        if !is_user(m.msg_name as usize, m.msg_namelen as usize) {
            return EFAULT;
        }
        m.msg_name as *mut SockaddrIn
    } else {
        core::ptr::null_mut()
    };
    // namelen lives in user space — recvfrom validates it again before write.
    let namelen_offset = core::mem::offset_of!(Msghdr, msg_namelen);
    let namelen_ptr: *mut u32 = unsafe {
        (msg as *mut u8).add(namelen_offset) as *mut u32
    };

    // Snapshot the iovec array into a kernel stack buffer.
    let mut iov_snap: [Iovec; IOV_MAX] = [Iovec { iov_base: core::ptr::null_mut(), iov_len: 0 }; IOV_MAX];
    for i in 0..m.msg_iovlen {
        iov_snap[i] = unsafe { core::ptr::read(m.msg_iov.add(i)) };
    }

    let mut total: i64 = 0;
    for iv in iov_snap[..m.msg_iovlen].iter() {
        if iv.iov_len == 0 { continue; }
        if iv.iov_base.is_null() { return EFAULT; }
        if !is_user(iv.iov_base as usize, iv.iov_len) { return EFAULT; }
        let r = recvfrom(fd, iv.iov_base, iv.iov_len, flags, name, namelen_ptr);
        if r < 0 {
            return if total > 0 { total } else { r };
        }
        if r == 0 { break; } // EOF
        total += r;
        if (r as usize) < iv.iov_len { break; }
    }
    // Write back the trailing fields with explicit bounds checks.
    let flags_offset = core::mem::offset_of!(Msghdr, msg_flags);
    let ctrllen_offset = core::mem::offset_of!(Msghdr, msg_controllen);
    unsafe {
        let flags_ptr = (msg as *mut u8).add(flags_offset) as *mut i32;
        let ctrllen_ptr = (msg as *mut u8).add(ctrllen_offset) as *mut usize;
        if is_user(flags_ptr as usize, 4) {
            core::ptr::write(flags_ptr, 0);
        }
        if is_user(ctrllen_ptr as usize, core::mem::size_of::<usize>()) {
            core::ptr::write(ctrllen_ptr, 0);
        }
    }
    total
}

// ============================================================================
// getsockopt / setsockopt
// ============================================================================

fn write_int_opt(optval: *mut u8, optlen: *mut u32, value: i32) -> i64 {
    if optval.is_null() || optlen.is_null() { return EFAULT; }
    // Gate optlen first (4-byte read+write).
    if !is_user(optlen as usize, 4) { return EFAULT; }
    let avail = unsafe { core::ptr::read_unaligned(optlen) };
    if (avail as usize) < 4 { return EINVAL; }
    if !is_user(optval as usize, 4) { return EFAULT; }
    unsafe {
        core::ptr::write_unaligned(optval as *mut i32, value);
        core::ptr::write_unaligned(optlen, 4);
    }
    0
}

fn read_int_opt(optval: *const u8, optlen: u32) -> Result<i32, i64> {
    if optval.is_null() { return Err(EFAULT); }
    if (optlen as usize) < 4 { return Err(EINVAL); }
    if !is_user(optval as usize, 4) { return Err(EFAULT); }
    Ok(unsafe { core::ptr::read_unaligned(optval as *const i32) })
}

pub fn getsockopt(fd: i32, level: i32, optname: i32,
                  optval: *mut u8, optlen: *mut u32) -> i64 {
    let s = match peek(fd) { Some(s) => s, None => return EBADF };

    let value: i32 = match (level, optname) {
        (SOL_SOCKET, SO_TYPE) => match s.kind {
            SockKind::Tcp => SOCK_STREAM,
            SockKind::Udp => SOCK_DGRAM,
            _ => 0,
        },
        (SOL_SOCKET, SO_ERROR) => {
            let e = s.so_error;
            // Consume the error (POSIX: getsockopt SO_ERROR clears it).
            let _ = with_slot(fd, |x| { x.so_error = 0; Ok(()) });
            e
        }
        (SOL_SOCKET, SO_REUSEADDR) => s.opt_reuseaddr as i32,
        (SOL_SOCKET, SO_REUSEPORT) => s.opt_reuseport as i32,
        (SOL_SOCKET, SO_KEEPALIVE) => s.opt_keepalive as i32,
        (SOL_SOCKET, SO_BROADCAST) => s.opt_broadcast as i32,
        (SOL_SOCKET, SO_SNDBUF)    => s.opt_sndbuf,
        (SOL_SOCKET, SO_RCVBUF)    => s.opt_rcvbuf,
        (l, TCP_NODELAY) if l == IPPROTO_TCP => s.opt_tcp_nodelay as i32,
        (l, IP_TOS) if l == IPPROTO_IP => s.opt_ip_tos,
        (l, IP_TTL) if l == IPPROTO_IP => s.opt_ip_ttl,
        _ => return EINVAL, // Chromium usually ignores return code; that's fine.
    };

    write_int_opt(optval, optlen, value)
}

pub fn setsockopt(fd: i32, level: i32, optname: i32,
                  optval: *const u8, optlen: u32) -> i64 {
    let v = match read_int_opt(optval, optlen) {
        Ok(v) => v, Err(e) => return e,
    };

    match with_slot(fd, |s| {
        match (level, optname) {
            (SOL_SOCKET, SO_REUSEADDR) => s.opt_reuseaddr = v != 0,
            (SOL_SOCKET, SO_REUSEPORT) => s.opt_reuseport = v != 0,
            (SOL_SOCKET, SO_KEEPALIVE) => s.opt_keepalive = v != 0,
            (SOL_SOCKET, SO_BROADCAST) => s.opt_broadcast = v != 0,
            (SOL_SOCKET, SO_SNDBUF)    => s.opt_sndbuf = v.max(1024),
            (SOL_SOCKET, SO_RCVBUF)    => s.opt_rcvbuf = v.max(1024),
            (SOL_SOCKET, SO_RCVTIMEO)  |
            (SOL_SOCKET, SO_SNDTIMEO)  => { /* stub: ignore */ }
            (SOL_SOCKET, SO_LINGER)    => {
                // struct linger = { on, sec }; we only read the first int.
                s.opt_linger_on = v != 0;
            }
            (l, TCP_NODELAY) if l == IPPROTO_TCP => s.opt_tcp_nodelay = v != 0,
            (l, IP_TOS) if l == IPPROTO_IP => s.opt_ip_tos = v,
            (l, IP_TTL) if l == IPPROTO_IP => s.opt_ip_ttl = v,
            _ => {
                // Unknown options: tolerate silently — Chromium probes many.
            }
        }
        Ok(0i64)
    }) { Ok(r) => r, Err(e) => e }
}

// ============================================================================
// getsockname / getpeername / shutdown / close
// ============================================================================

pub fn getsockname(fd: i32, addr: *mut SockaddrIn, len: *mut u32) -> i64 {
    let s = match peek(fd) { Some(s) => s, None => return EBADF };
    match write_sockaddr(addr, len, s.local_addr, s.local_port) {
        Ok(()) => 0, Err(e) => e,
    }
}

pub fn getpeername(fd: i32, addr: *mut SockaddrIn, len: *mut u32) -> i64 {
    let s = match peek(fd) { Some(s) => s, None => return EBADF };
    if s.status != SockStatus::Connected { return ENOTCONN; }
    match write_sockaddr(addr, len, s.peer_addr, s.peer_port) {
        Ok(()) => 0, Err(e) => e,
    }
}

/// shutdown(2) — SHUT_RD is advisory (we can't half-close our TCP stack),
/// SHUT_WR and SHUT_RDWR trigger a FIN via tcp::close() if this socket owns
/// the global connection.
///
/// TODO(tcp): real half-close semantics in src/net/tcp.rs.
pub fn shutdown(fd: i32, how: i32) -> i64 {
    match with_slot(fd, |s| {
        if s.kind != SockKind::Tcp { return Err(EOPNOTSUPP); }
        if s.status != SockStatus::Connected { return Err(ENOTCONN); }
        let id = s.pcb_id;
        match how {
            SHUT_RD => {
                if id >= 0 {
                    crate::net::tcp::shutdown_read(id as usize);
                }
            }
            SHUT_WR => {
                if id >= 0 {
                    crate::net::tcp::shutdown_write(id as usize);
                }
            }
            SHUT_RDWR => {
                if id >= 0 {
                    crate::net::tcp::shutdown_read(id as usize);
                    crate::net::tcp::shutdown_write(id as usize);
                }
                s.status = SockStatus::Closed;
            }
            _ => return Err(EINVAL),
        }
        Ok(0i64)
    }) { Ok(r) => r, Err(e) => e }
}

/// close(2) entry for socket fds — called by the syscall dispatcher once it
/// detects a socket-range fd. Releases the slot and tears down TCP if owned.
pub fn close(fd: i32) -> i64 {
    let idx = match slot_of(fd) { Some(i) => i, None => return EBADF };
    unsafe {
        if !SOCKET_TABLE[idx].in_use { return EBADF; }
        let s = &SOCKET_TABLE[idx];
        if s.kind == SockKind::Tcp && s.pcb_id >= 0 {
            let id = s.pcb_id as usize;
            if s.status == SockStatus::Connected {
                crate::net::tcp::close_pcb(id);
            }
            crate::net::tcp::pcb_free(id);
        }
    }
    free_slot(idx);
    0
}

// ============================================================================
// Helpers exposed for future epoll integration
// ============================================================================

/// Is there pending readable data? Used by epoll/poll.
pub fn readable(fd: i32) -> bool {
    let s = match peek(fd) { Some(s) => s, None => return false };
    match s.kind {
        SockKind::Tcp => {
            if s.status != SockStatus::Connected || s.pcb_id < 0 { return false; }
            crate::net::tcp::data_ready(s.pcb_id as usize)
        }
        _ => false,
    }
}

/// Is the socket writable? True when connected and PCB has TX room.
pub fn writable(fd: i32) -> bool {
    let s = match peek(fd) { Some(s) => s, None => return false };
    if s.status != SockStatus::Connected { return false; }
    if s.pcb_id < 0 { return false; }
    crate::net::tcp::can_write(s.pcb_id as usize)
}

/// Reset all socket slots — called from process teardown.
pub fn reset() {
    lock();
    unsafe {
        for i in 0..MAX_SOCKETS {
            SOCKET_TABLE[i] = SocketState::empty();
        }
    }
    unlock();
}
