//! Sphragis — AF_UNIX SOCK_STREAM (abstract namespace).
//!
//! Gap-audit item 025. Pairs with `kernel/pipe.rs`: both expose a
//! byte-stream over the fd table, but a Unix socket adds a named
//! endpoint + listen/accept handshake so two arbitrary tasks can
//! find each other without sharing a pipe at birth.
//!
//! Scope today:
//!   - SOCK_STREAM only (SOCK_DGRAM deferred to a follow-up).
//!   - Abstract namespace only: names are arbitrary strings up to
//!     64 bytes, compared byte-for-byte. No filesystem backing —
//!     filesystem-path sockets need BatFS to expose a SOCKET inode
//!     type, same as the FIFO deferral on pipes.
//!   - Single per-listener accept queue (depth 8).
//!   - Per-socket 4 KiB rx ring (the peer's writes land here).
//!
//! Wire-up with the existing pipe surface: SYS_READ/WRITE/CLOSE
//! already handle the Pipe FdKind; here we extend them to also
//! recognize the Socket FdKind in `role = Connected`.

#![allow(dead_code)]

use crate::drivers::uart;
use crate::kernel::process::{self, FdEntry, FdKind, SocketRole, TaskId, TaskState};
use crate::security::audit::{self, Category};

pub const MAX_SOCKETS: usize = 64;
pub const SOCK_RX_BUF: usize = 4096;
pub const SOCK_NAME_MAX: usize = 64;
pub const ACCEPT_BACKLOG: usize = 8;

struct Socket {
    active: bool,
    role: SocketRole,
    /// Abstract namespace key. Only populated for Listener sockets.
    name: [u8; SOCK_NAME_MAX],
    name_len: u8,
    /// Owning task — recorded at create() for audit attribution.
    owner: TaskId,
    /// AUDIT-BATCAVE-F6 / CAVE-M5 (2026-05-16): cave that called
    /// sys_socket / sys_bind / sys_accept. 0xFFFF = kernel context.
    /// Recorded for forensic provenance; FD-holder model still
    /// gates access today.
    owner_cave: u16,
    /// For role = Connected: index of the paired peer socket. None
    /// once peer closes; surfaces as EOF on the next read.
    peer: Option<u16>,
    /// Bytes the peer has written into us, waiting for read().
    rx_buf: [u8; SOCK_RX_BUF],
    rx_head: u16,
    rx_tail: u16,
    rx_len: u16,
    /// Listener-only: ring of socket ids pending accept().
    accept_q: [u16; ACCEPT_BACKLOG],
    accept_head: u8,
    accept_tail: u8,
    accept_len: u8,
    /// Task currently blocked inside accept() on this listener.
    waiting_acceptor: Option<TaskId>,
    /// Task currently blocked inside connect() waiting for the
    /// listener to accept their request.
    waiting_connector: Option<TaskId>,
    /// Task currently blocked inside read() on this socket.
    waiting_reader: Option<TaskId>,
    /// Task currently blocked inside write() when rx_buf full on
    /// the peer side.
    waiting_writer: Option<TaskId>,
}

impl Socket {
    const fn empty() -> Self {
        Self {
            active: false,
            role: SocketRole::Unbound,
            name: [0u8; SOCK_NAME_MAX],
            name_len: 0,
            owner: TaskId(0),
            owner_cave: 0xFFFF,
            peer: None,
            rx_buf: [0u8; SOCK_RX_BUF],
            rx_head: 0,
            rx_tail: 0,
            rx_len: 0,
            accept_q: [0u16; ACCEPT_BACKLOG],
            accept_head: 0,
            accept_tail: 0,
            accept_len: 0,
            waiting_acceptor: None,
            waiting_connector: None,
            waiting_reader: None,
            waiting_writer: None,
        }
    }

    fn name_str(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }
}

static mut SOCKETS: [Socket; MAX_SOCKETS] = {
    const EMPTY: Socket = Socket::empty();
    [EMPTY; MAX_SOCKETS]
};

pub fn init() {
    uart::puts("  [unix-sock] AF_UNIX SOCK_STREAM table initialized\n");
}

fn sock_mut(id: u16) -> Option<&'static mut Socket> {
    let i = id as usize;
    if i >= MAX_SOCKETS { return None; }
    unsafe { Some(&mut (*core::ptr::addr_of_mut!(SOCKETS))[i]) }
}

fn alloc_socket(role: SocketRole, owner: TaskId) -> Option<u16> {
    unsafe {
        for i in 0..MAX_SOCKETS {
            let s = &mut (*core::ptr::addr_of_mut!(SOCKETS))[i];
            if !s.active {
                *s = Socket::empty();
                s.active = true;
                s.role = role;
                s.owner = owner;
                // AUDIT-BATCAVE-F6: stamp active cave.
                s.owner_cave = {
                    let a = crate::caves::cave::get_active();
                    if a == usize::MAX { 0xFFFF } else { (a as u16) & 0x7FFF }
                };
                return Some(i as u16);
            }
        }
    }
    None
}

/// AUDIT-CAVE-H5 (2026-05-16): the active cave is the AF_UNIX
/// namespace key. A cave's bind / connect can only see listeners
/// owned by the same cave — Linux's process-global abstract
/// namespace would let a malicious cave hijack another cave's
/// "init.sock" or similar well-known name. Sphragis defaults to
/// per-cave isolation; cross-cave IPC must go through an explicit
/// mechanism (audit-ring, batpipe, etc.), not a string-name race.
/// `0xFFFF` is the kernel-context cave; kernel-bound sockets are
/// only visible to other kernel-context callers.
fn caller_cave() -> u16 {
    let a = crate::caves::cave::get_active();
    if a == usize::MAX { 0xFFFF } else { (a as u16) & 0x7FFF }
}

fn find_listener_by_name(name: &[u8]) -> Option<u16> {
    if name.is_empty() || name.len() > SOCK_NAME_MAX {
        return None;
    }
    let cave = caller_cave();
    unsafe {
        for i in 0..MAX_SOCKETS {
            let s = &(*core::ptr::addr_of!(SOCKETS))[i];
            if s.active
                && s.role == SocketRole::Listener
                && s.owner_cave == cave
                && s.name_str() == name
            {
                return Some(i as u16);
            }
        }
    }
    None
}

// ---------------- creation / lifecycle ----------------

/// Allocate a new AF_UNIX socket in Unbound state. Returns the fd
/// installed on the caller's fd table.
pub fn create() -> Result<u16, &'static str> {
    let owner = process::current_id();
    let sid = alloc_socket(SocketRole::Unbound, owner).ok_or("no free socket")?;
    let task = process::get(owner);
    let fd = match task.fd_alloc(FdEntry {
        kind: FdKind::Socket { id: sid, role: SocketRole::Unbound },
    }) {
        Some(fd) => fd,
        None => {
            sock_mut(sid).unwrap().active = false;
            return Err("fd table full");
        }
    };
    Ok(fd)
}

/// Bind a name to an Unbound or Listener-pending socket. After bind
/// the socket can be promoted to a listener with `listen()`.
pub fn bind(sid: u16, name: &[u8]) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("empty name");
    }
    if name.len() > SOCK_NAME_MAX {
        return Err("name too long");
    }
    if find_listener_by_name(name).is_some() {
        return Err("name in use");
    }
    let s = sock_mut(sid).ok_or("bad sock id")?;
    if !s.active {
        return Err("sock closed");
    }
    if s.role != SocketRole::Unbound {
        return Err("already bound");
    }
    s.name[..name.len()].copy_from_slice(name);
    s.name_len = name.len() as u8;
    audit_evt(b"bind", sid, name);
    Ok(())
}

/// Promote a bound socket to a listener. Subsequent connect()s with
/// the matching name land in this socket's accept queue.
pub fn listen(sid: u16) -> Result<(), &'static str> {
    let s = sock_mut(sid).ok_or("bad sock id")?;
    if !s.active { return Err("sock closed"); }
    if s.name_len == 0 { return Err("not bound"); }
    if s.role == SocketRole::Connected { return Err("already connected"); }
    s.role = SocketRole::Listener;
    update_fd_role(sid, SocketRole::Listener);
    Ok(())
}

/// Connect to a named listener. The caller's socket becomes
/// Connected; a fresh peer socket is allocated and pushed onto the
/// listener's accept queue.
///
/// Today this returns immediately without blocking — if the backlog
/// is full it errors with EAGAIN-like "backlog full". A blocking
/// variant could wait on accept_q drain, but most callers want the
/// fail-fast shape so they can retry / time out at their layer.
pub fn connect(sid: u16, name: &[u8]) -> Result<(), &'static str> {
    let me = sock_mut(sid).ok_or("bad sock id")?;
    if !me.active { return Err("sock closed"); }
    if me.role != SocketRole::Unbound {
        return Err("already connected or listening");
    }
    let listener_id = find_listener_by_name(name).ok_or("no such name")?;

    // Allocate the peer socket the listener will hand back through
    // accept(). It's pre-connected to the caller.
    let owner = process::current_id();
    let peer_sid = alloc_socket(SocketRole::Connected, owner)
        .ok_or("no free socket for peer")?;

    // Wire the two halves together.
    sock_mut(sid).unwrap().peer = Some(peer_sid);
    sock_mut(sid).unwrap().role = SocketRole::Connected;
    sock_mut(peer_sid).unwrap().peer = Some(sid);

    // Push peer onto the listener's accept queue.
    let listener = sock_mut(listener_id).unwrap();
    if listener.accept_len as usize >= ACCEPT_BACKLOG {
        // Roll back.
        sock_mut(sid).unwrap().peer = None;
        sock_mut(sid).unwrap().role = SocketRole::Unbound;
        sock_mut(peer_sid).unwrap().active = false;
        return Err("backlog full");
    }
    let slot = listener.accept_tail as usize;
    listener.accept_q[slot] = peer_sid;
    listener.accept_tail = ((listener.accept_tail as usize + 1) % ACCEPT_BACKLOG) as u8;
    listener.accept_len += 1;

    // Wake any task blocked in accept() on this listener.
    if let Some(tid) = listener.waiting_acceptor.take() {
        process::get(tid).state = TaskState::Ready;
    }

    update_fd_role(sid, SocketRole::Connected);
    audit_evt(b"connect", sid, name);
    Ok(())
}

/// Accept one queued connection on a listener. Blocks if the accept
/// queue is empty. Returns the fd of the new Connected socket
/// installed on the caller's fd table.
pub fn accept(sid: u16) -> Result<u16, &'static str> {
    let caller = process::current_id();
    loop {
        let s = sock_mut(sid).ok_or("bad sock id")?;
        if !s.active { return Err("sock closed"); }
        if s.role != SocketRole::Listener { return Err("not a listener"); }

        if s.accept_len > 0 {
            let head = s.accept_head as usize;
            let peer_sid = s.accept_q[head];
            s.accept_head = ((head + 1) % ACCEPT_BACKLOG) as u8;
            s.accept_len -= 1;

            let task = process::get(caller);
            let fd = task.fd_alloc(FdEntry {
                kind: FdKind::Socket { id: peer_sid, role: SocketRole::Connected },
            });
            let fd = match fd {
                Some(fd) => fd,
                None => {
                    // Roll back the dequeue — caller has no slot.
                    let s = sock_mut(sid).unwrap();
                    s.accept_head = ((s.accept_head as usize + ACCEPT_BACKLOG - 1) % ACCEPT_BACKLOG) as u8;
                    s.accept_len += 1;
                    return Err("fd table full");
                }
            };
            audit_evt(b"accept", peer_sid, s.name_str());
            return Ok(fd);
        }

        // Empty queue → block.
        s.waiting_acceptor = Some(caller);
        process::get(caller).state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();
    }
}

// ---------------- byte stream ----------------

/// Read from a Connected socket — pulls bytes out of *our* rx_buf,
/// which is where the peer's writes have landed.
///
/// Blocks if rx empty + peer still open. EOF (Ok(0)) if rx empty
/// + peer is gone.
pub fn read(sid: u16, out: &mut [u8]) -> Result<usize, &'static str> {
    if out.is_empty() { return Ok(0); }
    let caller = process::current_id();
    loop {
        let s = sock_mut(sid).ok_or("bad sock id")?;
        if !s.active { return Err("sock closed"); }
        if s.role != SocketRole::Connected { return Err("not connected"); }

        if s.rx_len > 0 {
            let take_u = (out.len()).min(s.rx_len as usize);
            for i in 0..take_u {
                out[i] = s.rx_buf[s.rx_head as usize];
                s.rx_head = ((s.rx_head as usize + 1) % SOCK_RX_BUF) as u16;
            }
            s.rx_len -= take_u as u16;
            // Wake a writer (on the peer side) blocked because OUR
            // rx_buf was full.
            if let Some(peer_id) = s.peer {
                if let Some(peer) = sock_mut(peer_id) {
                    if let Some(tid) = peer.waiting_writer.take() {
                        process::get(tid).state = TaskState::Ready;
                    }
                }
            }
            return Ok(take_u);
        }

        // Empty rx. EOF if peer is gone, else block.
        if s.peer.is_none() {
            return Ok(0);
        }
        s.waiting_reader = Some(caller);
        process::get(caller).state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();
    }
}

/// Write to a Connected socket — bytes land in the PEER's rx_buf.
///
/// Blocks if peer's rx full + peer still open. Returns Err("EPIPE")
/// if peer is gone.
pub fn write(sid: u16, data: &[u8]) -> Result<usize, &'static str> {
    if data.is_empty() { return Ok(0); }
    let caller = process::current_id();
    let mut written = 0;

    while written < data.len() {
        let me = sock_mut(sid).ok_or("bad sock id")?;
        if !me.active { return Err("sock closed"); }
        if me.role != SocketRole::Connected { return Err("not connected"); }
        let peer_id = match me.peer {
            Some(p) => p,
            None => {
                if written > 0 { return Ok(written); }
                return Err("EPIPE");
            }
        };

        let peer = match sock_mut(peer_id) {
            Some(p) if p.active => p,
            _ => {
                if written > 0 { return Ok(written); }
                return Err("EPIPE");
            }
        };

        let free = SOCK_RX_BUF - peer.rx_len as usize;
        if free == 0 {
            // Peer's buffer full → block until peer reads.
            me.waiting_writer = Some(caller);
            process::get(caller).state = TaskState::Blocked;
            crate::kernel::scheduler::yield_now();
            continue;
        }

        let remaining = data.len() - written;
        let put = remaining.min(free);
        for i in 0..put {
            peer.rx_buf[peer.rx_tail as usize] = data[written + i];
            peer.rx_tail = ((peer.rx_tail as usize + 1) % SOCK_RX_BUF) as u16;
        }
        peer.rx_len += put as u16;
        written += put;

        // Wake a blocked reader on the peer side.
        if let Some(tid) = peer.waiting_reader.take() {
            process::get(tid).state = TaskState::Ready;
        }
    }
    Ok(written)
}

/// Close a socket end. Tears down the peer pointer (if any) so any
/// blocked reader on the peer wakes to EOF, and frees the slot once
/// both ends are gone.
pub fn close(sid: u16) {
    let s = sock_mut(sid);
    let s = match s { Some(s) if s.active => s, _ => return };

    let role = s.role;
    let peer_id = s.peer.take();
    let name_copy = {
        let mut n = [0u8; SOCK_NAME_MAX];
        let len = s.name_len as usize;
        if len > 0 { n[..len].copy_from_slice(&s.name[..len]); }
        (n, len)
    };

    // Wake anything still blocked on us before we go.
    if let Some(tid) = s.waiting_reader.take() {
        process::get(tid).state = TaskState::Ready;
    }
    if let Some(tid) = s.waiting_writer.take() {
        process::get(tid).state = TaskState::Ready;
    }
    if let Some(tid) = s.waiting_acceptor.take() {
        process::get(tid).state = TaskState::Ready;
    }
    if let Some(tid) = s.waiting_connector.take() {
        process::get(tid).state = TaskState::Ready;
    }
    s.active = false;

    // Tell the peer we left. They'll see EOF on next read or EPIPE
    // on next write.
    if let Some(pid) = peer_id {
        if let Some(p) = sock_mut(pid) {
            if p.active {
                p.peer = None;
                if let Some(tid) = p.waiting_reader.take() {
                    process::get(tid).state = TaskState::Ready;
                }
                if let Some(tid) = p.waiting_writer.take() {
                    process::get(tid).state = TaskState::Ready;
                }
            }
        }
    }

    audit_evt(
        match role {
            SocketRole::Listener  => b"close-listener",
            SocketRole::Connected => b"close-stream",
            SocketRole::Unbound   => b"close-unbound",
        },
        sid,
        &name_copy.0[..name_copy.1],
    );
}

/// Update the cached `role` byte in the caller's fd table so future
/// SYS_READ/WRITE branches see the right kind. Called from listen()
/// and connect() since those transition the socket's role.
fn update_fd_role(sid: u16, new_role: SocketRole) {
    let task = process::current();
    for fd in 0..task.fds.len() {
        if let Some(entry) = task.fds[fd] {
            if let FdKind::Socket { id, .. } = entry.kind {
                if id == sid {
                    task.fds[fd] = Some(FdEntry {
                        kind: FdKind::Socket { id, role: new_role },
                    });
                }
            }
        }
    }
}

// ---------------- audit helper ----------------

fn audit_evt(verb: &[u8], sid: u16, name: &[u8]) {
    let mut buf = [0u8; 96];
    let mut at = 0;
    at = push(&mut buf, at, verb);
    at = push(&mut buf, at, b" pid=");
    at = u16_dec(&mut buf, at, process::current_id().0);
    at = push(&mut buf, at, b" sid=");
    at = u16_dec(&mut buf, at, sid);
    if !name.is_empty() {
        at = push(&mut buf, at, b" name=");
        let take = name.len().min(buf.len() - at);
        // Sanitize: any non-printable becomes '?'.
        for i in 0..take {
            let b = name[i];
            buf[at + i] = if (0x20..=0x7e).contains(&b) { b } else { b'?' };
        }
        at += take;
    }
    audit::record(Category::Socket, &buf[..at]);
}

fn push(buf: &mut [u8], at: usize, s: &[u8]) -> usize {
    let n = s.len().min(buf.len().saturating_sub(at));
    buf[at..at + n].copy_from_slice(&s[..n]);
    at + n
}

fn u16_dec(buf: &mut [u8], at: usize, v: u16) -> usize {
    if v == 0 {
        if at < buf.len() { buf[at] = b'0'; }
        return at + 1;
    }
    let mut tmp = [0u8; 5];
    let mut i = 0;
    let mut n = v;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in 0..i {
        if at + j < buf.len() { buf[at + j] = tmp[i - 1 - j]; }
    }
    at + i
}
