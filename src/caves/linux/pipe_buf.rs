// Sphragis — Minimal pipe buffer pool for socketpair() and pipe2().
//
// Chromium's Mojo IPC in --single-process still uses socketpair(2) as
// the wake-up channel between its main thread and various worker
// threads. Our earlier stub — two VFS Socket nodes with no backing —
// meant writes vanished and reads never saw data, so ppoll on those
// fds blocked forever and Chromium's event loop never made progress.
//
// This module provides a fixed pool of 8 bidirectional pipe pairs,
// each with two 4 KB circular buffers (A→B and B→A). Allocated by
// `socketpair()`, associated with a pair of fds via
// `fd::FdEntry.pipe_slot`, serviced by read/write/poll hooks in
// `syscall.rs`. Buffers live in a single `static mut` so there's no
// heap involvement — matches the rest of the kernel.
//
// Limitations (deliberate for first cut):
// - No blocking. read() on empty returns EAGAIN. Chromium's event
//   loop handles that (retries on next wake).
// - No partial-write retry. If the outbound buffer is full, write()
//   returns EAGAIN. Caller should poll before trying again.
// - Fixed 8 pairs × 4 KB × 2 = 64 KB kernel BSS. Cheap.
// - No cleanup on close(). The buffers sit until the pool wraps
//   around; fine for a short-lived shell run.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const MAX_PAIRS: usize = 16;
pub const BUF_SIZE: usize = 4096;
pub const MAX_FDS_QUEUED: usize = 32;

#[repr(C)]
pub struct PairBuf {
    /// data written by end A, read by end B
    a_to_b: [u8; BUF_SIZE],
    a_to_b_len: usize,
    /// data written by end B, read by end A
    b_to_a: [u8; BUF_SIZE],
    b_to_a_len: usize,
    /// File descriptors passed via SCM_RIGHTS, A→B and B→A. Single-process
    /// only — sender and receiver share the same per-cave fd table, so the
    /// fd numbers are valid on both sides; we just need to NOT drop them
    /// the way iov data alone would. Chromium's IPC channel handshake
    /// relies on this — the receiver pulls a Mojo socket fd out of the
    /// first inbound message and binds its IPC endpoint to it.
    a_to_b_fds: [u32; MAX_FDS_QUEUED],
    a_to_b_fds_len: usize,
    b_to_a_fds: [u32; MAX_FDS_QUEUED],
    b_to_a_fds_len: usize,
    active: bool,
}

impl PairBuf {
    const fn empty() -> Self {
        Self {
            a_to_b: [0; BUF_SIZE],
            a_to_b_len: 0,
            b_to_a: [0; BUF_SIZE],
            b_to_a_len: 0,
            a_to_b_fds: [0; MAX_FDS_QUEUED],
            a_to_b_fds_len: 0,
            b_to_a_fds: [0; MAX_FDS_QUEUED],
            b_to_a_fds_len: 0,
            active: false,
        }
    }
}

static mut PAIRS: [PairBuf; MAX_PAIRS] = {
    const EMPTY: PairBuf = PairBuf::empty();
    [EMPTY; MAX_PAIRS]
};

/// Simple spinlock guarding PAIRS mutation.  Reads also hold it so
/// that read-side sees the most-recent write without barriers.
static LOCK: AtomicBool = AtomicBool::new(false);

fn lock() {
    while LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

fn unlock() {
    LOCK.store(false, Ordering::Release);
}

/// Allocate a new pair. Returns the slot index (0..MAX_PAIRS) or
/// None if the pool is exhausted. The two fds that share this slot
/// should be marked with `side = 0` and `side = 1` respectively via
/// `fd::attach_pipe`.
pub fn alloc_pair() -> Option<usize> {
    lock();
    let result = unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        let mut chosen: Option<usize> = None;
        for i in 0..MAX_PAIRS {
            if !table[i].active {
                table[i] = PairBuf::empty();
                table[i].active = true;
                chosen = Some(i);
                break;
            }
        }
        chosen
    };
    unlock();
    result
}

/// Count of pair writes since boot — diagnostic only.
pub static WRITES: AtomicUsize = AtomicUsize::new(0);
pub static READS:  AtomicUsize = AtomicUsize::new(0);

/// Write `data` to the outbound buffer of `side` in pair `slot`.
/// Returns number of bytes queued, or Err(-errno). If the outbound
/// buffer doesn't have enough room, queues as much as fits.
pub fn write(slot: usize, side: u8, data: &[u8]) -> Result<usize, i64> {
    if slot >= MAX_PAIRS { return Err(-9); /* EBADF */ }
    lock();
    let result = unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        let p = &mut table[slot];
        if !p.active { unlock(); return Err(-9); /* EBADF */ }
        let (buf, len_field): (&mut [u8; BUF_SIZE], &mut usize) = if side == 0 {
            (&mut p.a_to_b, &mut p.a_to_b_len)
        } else {
            (&mut p.b_to_a, &mut p.b_to_a_len)
        };
        let free = BUF_SIZE - *len_field;
        if free == 0 {
            unlock();
            return Err(-11); // EAGAIN
        }
        let n = data.len().min(free);
        buf[*len_field..*len_field + n].copy_from_slice(&data[..n]);
        *len_field += n;
        WRITES.fetch_add(1, Ordering::Relaxed);
        Ok(n)
    };
    unlock();
    result
}

/// Read from the inbound buffer for `side` in pair `slot`. Returns
/// bytes read or Err(-errno). If inbound buffer is empty, returns
/// EAGAIN (caller should poll before trying again).
pub fn read(slot: usize, side: u8, out: &mut [u8]) -> Result<usize, i64> {
    if slot >= MAX_PAIRS { return Err(-9); }
    lock();
    let result = unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        let p = &mut table[slot];
        if !p.active { unlock(); return Err(-9); }
        // Side A reads what B wrote → b_to_a.
        // Side B reads what A wrote → a_to_b.
        let (buf, len_field): (&mut [u8; BUF_SIZE], &mut usize) = if side == 0 {
            (&mut p.b_to_a, &mut p.b_to_a_len)
        } else {
            (&mut p.a_to_b, &mut p.a_to_b_len)
        };
        if *len_field == 0 {
            unlock();
            return Err(-11); // EAGAIN
        }
        let n = out.len().min(*len_field);
        out[..n].copy_from_slice(&buf[..n]);
        // Shift remaining bytes to the front.
        let remaining = *len_field - n;
        if remaining > 0 {
            buf.copy_within(n..*len_field, 0);
        }
        *len_field = remaining;
        READS.fetch_add(1, Ordering::Relaxed);
        Ok(n)
    };
    unlock();
    result
}

/// Returns true if `side` of pair `slot` has buffered inbound data
/// ready to read. Used by poll/ppoll for POLLIN reporting.
pub fn has_readable(slot: usize, side: u8) -> bool {
    if slot >= MAX_PAIRS { return false; }
    lock();
    let result = unsafe {
        let p = &(*core::ptr::addr_of!(PAIRS))[slot];
        if !p.active { false }
        else if side == 0 { p.b_to_a_len > 0 }
        else              { p.a_to_b_len > 0 }
    };
    unlock();
    result
}

/// Push file descriptors onto the pair's outbound fd queue. Used by
/// sendmsg(SCM_RIGHTS): the sender's iov data goes through write(),
/// the cmsg fd list goes through here. The receiving recvmsg() pops
/// them via pop_fds and re-encodes them in its own cmsg buffer.
///
/// `side` matches the side WRITING (i.e. the same side parameter that
/// `write()` would use); the fds queue onto the OPPOSITE side for
/// receiver to drain.
pub fn push_fds(slot: usize, side: u8, fds: &[u32]) -> Result<(), i64> {
    if slot >= MAX_PAIRS { return Err(-9); } // EBADF
    lock();
    let result = unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        let p = &mut table[slot];
        if !p.active { unlock(); return Err(-9); }
        let target_side = side ^ 1;
        let (queue, len) = if target_side == 1 {
            (&mut p.a_to_b_fds[..], &mut p.a_to_b_fds_len)
        } else {
            (&mut p.b_to_a_fds[..], &mut p.b_to_a_fds_len)
        };
        let space = MAX_FDS_QUEUED - *len;
        if fds.len() > space {
            unlock();
            return Err(-105); // ENOBUFS
        }
        for &fd in fds {
            queue[*len] = fd;
            *len += 1;
        }
        Ok(())
    };
    if result.is_ok() { unlock(); }
    result
}

/// Drain up to `out.len()` fds from the pair's inbound fd queue.
/// Returns the number of fds copied. `side` is the receiver side.
pub fn pop_fds(slot: usize, side: u8, out: &mut [u32]) -> usize {
    if slot >= MAX_PAIRS { return 0; }
    lock();
    let n = unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        let p = &mut table[slot];
        if !p.active { unlock(); return 0; }
        // Receiver side reads from its own queue (the queue the sender
        // pushed to, which is OUR side from the receiver's POV).
        let (queue, len) = if side == 0 {
            (&mut p.b_to_a_fds[..], &mut p.b_to_a_fds_len)
        } else {
            (&mut p.a_to_b_fds[..], &mut p.a_to_b_fds_len)
        };
        let take = (*len).min(out.len());
        for i in 0..take {
            out[i] = queue[i];
        }
        // Shift remaining fds down.
        for i in take..*len {
            queue[i - take] = queue[i];
        }
        *len -= take;
        take
    };
    unlock();
    n
}

/// Number of fds currently queued for the receiver on `side`.
pub fn pending_fds(slot: usize, side: u8) -> usize {
    if slot >= MAX_PAIRS { return 0; }
    lock();
    let n = unsafe {
        let p = &(*core::ptr::addr_of!(PAIRS))[slot];
        if !p.active { unlock(); return 0; }
        if side == 0 { p.b_to_a_fds_len } else { p.a_to_b_fds_len }
    };
    unlock();
    n
}

/// Deactivate a pair — called when BOTH fds have been closed. Leaves
/// the buffer contents in place so a concurrent read can still drain
/// (the next alloc_pair will wipe it).
#[allow(dead_code)]
pub fn release(slot: usize) {
    if slot >= MAX_PAIRS { return; }
    lock();
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PAIRS);
        table[slot].active = false;
    }
    unlock();
}
