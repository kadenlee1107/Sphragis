//! Sphragis — POSIX-style anonymous pipes.
//!
//! `create()` returns `(read_fd, write_fd)` on the calling task's fd
//! table. Bytes written to the write end are read from the read end
//! in FIFO order through an in-kernel ring buffer.
//!
//! Why not just use kernel/ipc.rs? `ipc.rs` is message-passing with a
//! capability check per channel — appropriate for cave-to-cave
//! request/response, not for the byte-stream shape POSIX programs
//! expect. Pipes inherit the creator's privilege and carry no
//! per-channel cap; ownership is the fd itself.
//!
//! Semantics:
//! - read(empty) blocks until a writer writes or all writers close.
//! - write(full) blocks until a reader reads or all readers close.
//! - read returns 0 (EOF) once the buffer is empty AND no writers
//!   remain.
//! - write returns Err(EPIPE) once no readers remain.
//! - close() decrements the corresponding end's refcount; when both
//!   reach zero the pipe is reclaimed.
//!
//! Audit: create + close are logged. Read/write are NOT (per-byte
//! traffic would drown the ring). A reviewer can reconstruct fd
//! ownership from the create/close pairs.

#![allow(dead_code)]

use crate::drivers::uart;
use crate::kernel::process::{self, FdEntry, FdKind, PipeEnd, TaskId, TaskState};
use crate::security::audit::{self, Category};

pub const MAX_PIPES: usize = 32;
pub const PIPE_BUF_SIZE: usize = 4096;

struct Pipe {
    active: bool,
    buf: [u8; PIPE_BUF_SIZE],
    head: usize,           // next byte to read
    tail: usize,           // next byte to write
    len: usize,            // bytes currently in buffer
    readers: u8,           // open read-end refcount
    writers: u8,           // open write-end refcount
    waiting_reader: Option<TaskId>,
    waiting_writer: Option<TaskId>,
}

impl Pipe {
    const fn empty() -> Self {
        Self {
            active: false,
            buf: [0u8; PIPE_BUF_SIZE],
            head: 0,
            tail: 0,
            len: 0,
            readers: 0,
            writers: 0,
            waiting_reader: None,
            waiting_writer: None,
        }
    }
}

static mut PIPES: [Pipe; MAX_PIPES] = {
    const EMPTY: Pipe = Pipe::empty();
    [EMPTY; MAX_PIPES]
};

pub fn init() {
    uart::puts("  [pipe] anonymous pipe table initialized\n");
}

fn pipe_mut(id: u16) -> Option<&'static mut Pipe> {
    let i = id as usize;
    if i >= MAX_PIPES {
        return None;
    }
    unsafe { Some(&mut (*core::ptr::addr_of_mut!(PIPES))[i]) }
}

/// Allocate a fresh pipe and install both ends on the calling task's
/// fd table. Returns `(read_fd, write_fd)`.
///
/// Errors:
/// - "no free pipe"   — pipe table exhausted
/// - "fd table full"  — caller has no slots; pipe is rolled back
pub fn create() -> Result<(u16, u16), &'static str> {
    let caller_id = process::current_id();

    // Gap-audit item 030 first slice — charge the active cave's
    // quota before grabbing a slot. Pipe ring buffer is one page
    // (4 KiB). Refunded on any rollback path below + on release_end
    // when both ends close.
    crate::caves::cave::active_charge_pages(1)?;

    // Find a free pipe slot.
    let mut slot = None;
    unsafe {
        for i in 0..MAX_PIPES {
            if !(*core::ptr::addr_of!(PIPES))[i].active {
                slot = Some(i);
                break;
            }
        }
    }
    let slot = match slot {
        Some(s) => s,
        None => {
            crate::caves::cave::active_release_pages(1);
            return Err("no free pipe");
        }
    };

    // Mark active before installing fds so a concurrent caller can't
    // claim the same slot. (Single-threaded today, but cheap.)
    let p = pipe_mut(slot as u16).unwrap();
    p.active = true;
    p.head = 0;
    p.tail = 0;
    p.len = 0;
    p.readers = 1;
    p.writers = 1;
    p.waiting_reader = None;
    p.waiting_writer = None;

    let task = process::get(caller_id);
    let read_fd = match task.fd_alloc(FdEntry {
        kind: FdKind::Pipe { id: slot as u16, end: PipeEnd::Read },
    }) {
        Some(fd) => fd,
        None => {
            // Roll back.
            p.active = false;
            p.readers = 0;
            p.writers = 0;
            crate::caves::cave::active_release_pages(1);
            return Err("fd table full");
        }
    };
    let write_fd = match task.fd_alloc(FdEntry {
        kind: FdKind::Pipe { id: slot as u16, end: PipeEnd::Write },
    }) {
        Some(fd) => fd,
        None => {
            task.fd_take(read_fd);
            p.active = false;
            p.readers = 0;
            p.writers = 0;
            crate::caves::cave::active_release_pages(1);
            return Err("fd table full");
        }
    };

    let mut buf = [0u8; 48];
    let n = format_create_msg(&mut buf, caller_id.0, slot as u16, read_fd, write_fd);
    audit::record(Category::Pipe, &buf[..n]);

    Ok((read_fd, write_fd))
}

/// Drop a held end. The fd must already have been removed from the
/// task's fd table by the caller (typically `sys_close` does both).
/// We do the refcount work + wake any task blocked on the other end
/// + reclaim the pipe when both ends reach zero.
pub fn release_end(id: u16, end: PipeEnd) {
    let Some(p) = pipe_mut(id) else { return; };
    if !p.active { return; }

    match end {
        PipeEnd::Read => {
            if p.readers > 0 { p.readers -= 1; }
            // No more readers → wake any blocked writer so they see EPIPE.
            if p.readers == 0 {
                if let Some(tid) = p.waiting_writer.take() {
                    process::get(tid).state = TaskState::Ready;
                }
            }
        }
        PipeEnd::Write => {
            if p.writers > 0 { p.writers -= 1; }
            // No more writers → wake any blocked reader so they see EOF.
            if p.writers == 0 {
                if let Some(tid) = p.waiting_reader.take() {
                    process::get(tid).state = TaskState::Ready;
                }
            }
        }
    }

    let mut buf = [0u8; 48];
    let n = format_close_msg(
        &mut buf,
        process::current_id().0,
        id,
        end,
        p.readers,
        p.writers,
    );
    audit::record(Category::Pipe, &buf[..n]);

    if p.readers == 0 && p.writers == 0 {
        p.active = false;
        p.len = 0;
        p.head = 0;
        p.tail = 0;
        // Refund the page charge from create-time. Same drift caveat
        // as shm — release happens against the *active* cave at
        // close-time, which may differ from the creator. Bounded by
        // the original charge so the books eventually balance.
        crate::caves::cave::active_release_pages(1);
    }
}

/// Read up to `out.len()` bytes from the pipe. Blocks if the buffer
/// is empty and at least one writer remains. Returns 0 on EOF (no
/// writers + empty buffer). Partial reads are valid.
pub fn read(id: u16, out: &mut [u8]) -> Result<usize, &'static str> {
    if out.is_empty() {
        return Ok(0);
    }
    let caller_id = process::current_id();

    loop {
        let Some(p) = pipe_mut(id) else {
            return Err("bad pipe id");
        };
        if !p.active {
            return Err("pipe closed");
        }

        if p.len > 0 {
            let take = if out.len() < p.len { out.len() } else { p.len };
            for i in 0..take {
                out[i] = p.buf[p.head];
                p.head = (p.head + 1) % PIPE_BUF_SIZE;
            }
            p.len -= take;
            // Wake a blocked writer; the buffer just got space.
            if let Some(tid) = p.waiting_writer.take() {
                process::get(tid).state = TaskState::Ready;
            }
            return Ok(take);
        }

        // Buffer empty.
        if p.writers == 0 {
            // EOF: empty + no writers.
            return Ok(0);
        }

        // Block until a writer writes or all writers close.
        p.waiting_reader = Some(caller_id);
        process::get(caller_id).state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();
        // Loop and retry.
    }
}

/// Write all of `data` into the pipe. Blocks if the buffer is full
/// and at least one reader remains. Returns `Err("EPIPE")` if no
/// readers remain. Partial writes ARE possible if the pipe transitions
/// to no-readers mid-write; in that case Ok(n) is returned with the
/// number of bytes that landed before the transition.
pub fn write(id: u16, data: &[u8]) -> Result<usize, &'static str> {
    if data.is_empty() {
        return Ok(0);
    }
    let caller_id = process::current_id();
    let mut written = 0;

    while written < data.len() {
        let Some(p) = pipe_mut(id) else {
            return Err("bad pipe id");
        };
        if !p.active {
            return Err("pipe closed");
        }
        if p.readers == 0 {
            // No one to read it. If we already wrote some bytes,
            // return short — POSIX would deliver SIGPIPE, we don't
            // have signals.
            if written > 0 {
                return Ok(written);
            }
            return Err("EPIPE");
        }

        let free = PIPE_BUF_SIZE - p.len;
        if free == 0 {
            // Block until a reader frees space (or closes).
            p.waiting_writer = Some(caller_id);
            process::get(caller_id).state = TaskState::Blocked;
            crate::kernel::scheduler::yield_now();
            continue;
        }

        let remaining = data.len() - written;
        let put = if remaining < free { remaining } else { free };
        for i in 0..put {
            p.buf[p.tail] = data[written + i];
            p.tail = (p.tail + 1) % PIPE_BUF_SIZE;
        }
        p.len += put;
        written += put;

        // Wake a blocked reader; bytes are available.
        if let Some(tid) = p.waiting_reader.take() {
            process::get(tid).state = TaskState::Ready;
        }
    }

    Ok(written)
}

// ---------- formatting helpers (no_std, no alloc) ----------

fn u16_dec(buf: &mut [u8], at: usize, v: u16) -> usize {
    if v == 0 {
        buf[at] = b'0';
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
        buf[at + j] = tmp[i - 1 - j];
    }
    at + i
}

fn write_str(buf: &mut [u8], at: usize, s: &[u8]) -> usize {
    let n = s.len().min(buf.len().saturating_sub(at));
    buf[at..at + n].copy_from_slice(&s[..n]);
    at + n
}

fn format_create_msg(buf: &mut [u8], pid: u16, id: u16, rfd: u16, wfd: u16) -> usize {
    let mut at = 0;
    at = write_str(buf, at, b"create pid=");
    at = u16_dec(buf, at, pid);
    at = write_str(buf, at, b" id=");
    at = u16_dec(buf, at, id);
    at = write_str(buf, at, b" r=");
    at = u16_dec(buf, at, rfd);
    at = write_str(buf, at, b" w=");
    at = u16_dec(buf, at, wfd);
    at
}

fn format_close_msg(
    buf: &mut [u8],
    pid: u16,
    id: u16,
    end: PipeEnd,
    readers: u8,
    writers: u8,
) -> usize {
    let mut at = 0;
    at = write_str(buf, at, b"close pid=");
    at = u16_dec(buf, at, pid);
    at = write_str(buf, at, b" id=");
    at = u16_dec(buf, at, id);
    at = write_str(buf, at, match end {
        PipeEnd::Read => b" end=r",
        PipeEnd::Write => b" end=w",
    });
    at = write_str(buf, at, b" rc=");
    at = u16_dec(buf, at, readers as u16);
    at = write_str(buf, at, b" wc=");
    at = u16_dec(buf, at, writers as u16);
    at
}
