// Sphragis — Stdio Ring Buffer
// Async 256 KB ring between BatCave sys_write (fd 1/2) and the PL011 UART.
//
// Motivation: Chromium content_shell with --enable-logging=stderr emits
// thousands of log lines. The emulated PL011 UART at 0x09000000 is baud-rate
// limited (~115200 baud); writing each stderr byte synchronously would block
// the issuing task on UART back-pressure and stall the kernel.
//
// Design:
//   - Single 256 KB static buffer (no heap).
//   - AtomicUsize head/tail, monotonically increasing; index into RING
//     is `pos % RING_SIZE`. Wrap-safe as long as usize is 64-bit on aarch64.
//   - Lossy on overflow: drop the OLDEST bytes. Losing verbose logs is a
//     better failure mode than stalling a renderer.
//   - Producer: sys_write (any task). Consumer: drain_to_uart() called from
//     the timer tick at 100 Hz, pops up to DRAIN_CHUNK bytes per tick.
//   - force_flush() bypasses the ring and writes directly to the UART; use
//     from panic handlers where we cannot wait for the drain kthread.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::drivers::uart;

/// 256 KB ring.
pub const RING_SIZE: usize = 256 * 1024;

/// Soft cap on bytes drained per timer tick.  At 100 Hz this yields a sustained
/// drain of ~51 KB/s which comfortably outpaces a ~115200 baud UART (~11 KB/s)
/// while keeping per-tick jitter bounded.
pub const DRAIN_CHUNK: usize = 512;

// Raw backing storage.  `static mut` is fine: access is serialised via the
// atomic head/tail indices plus a small drain-side guard.
static mut RING: [u8; RING_SIZE] = [0; RING_SIZE];

/// Monotonic write cursor (bytes ever pushed).
static HEAD: AtomicUsize = AtomicUsize::new(0);
/// Monotonic read cursor (bytes ever popped).
static TAIL: AtomicUsize = AtomicUsize::new(0);

/// Becomes true once the ring is considered "live" and sys_write should route
/// through it.  Before this, sys_write must fall through to direct UART so
/// early boot diagnostics still appear even if the drain hook has not fired.
static READY: AtomicBool = AtomicBool::new(false);

/// Re-entrancy guard for the drain path.  Prevents the drain from running
/// recursively if a UART write path were ever to call back into sys_write
/// (it does not today, but this keeps us honest).
static DRAINING: AtomicBool = AtomicBool::new(false);

/// Mark the ring as live.  Call once, after the UART is up.
pub fn init() {
    READY.store(true, Ordering::Release);
}

/// Is the ring accepting writes?
pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

/// Capacity in bytes.
#[inline]
pub fn capacity() -> usize {
    RING_SIZE
}

/// Bytes currently buffered.
#[inline]
pub fn len() -> usize {
    let h = HEAD.load(Ordering::Acquire);
    let t = TAIL.load(Ordering::Acquire);
    h.saturating_sub(t)
}

#[inline]
pub fn is_empty() -> bool {
    len() == 0
}

/// Push one byte.  If the ring is full, advance TAIL to drop the oldest byte.
pub fn push(byte: u8) {
    let h = HEAD.load(Ordering::Relaxed);
    let t = TAIL.load(Ordering::Acquire);
    let used = h - t;
    if used >= RING_SIZE {
        // Overflow: drop oldest byte.
        TAIL.store(t + 1 + (used - RING_SIZE), Ordering::Release);
    }
    unsafe {
        let slot = h % RING_SIZE;
        let p = core::ptr::addr_of_mut!(RING) as *mut u8;
        core::ptr::write_volatile(p.add(slot), byte);
    }
    HEAD.store(h + 1, Ordering::Release);
}

/// Bulk enqueue.  Drops oldest bytes in bulk on overflow.
pub fn push_slice(s: &[u8]) {
    if s.is_empty() { return; }

    let n = s.len();
    let h = HEAD.load(Ordering::Relaxed);
    let t = TAIL.load(Ordering::Acquire);
    let used = h - t;

    // If this push would overflow, bump TAIL ahead to make room.
    let projected = used + n;
    if projected > RING_SIZE {
        let drop = projected - RING_SIZE;
        TAIL.store(t + drop, Ordering::Release);
    }

    // If the incoming slice is larger than the ring itself, only the last
    // RING_SIZE bytes can possibly survive; skip the prefix.
    let (src, mut write_pos) = if n > RING_SIZE {
        (&s[n - RING_SIZE..], h + (n - RING_SIZE))
    } else {
        (s, h)
    };

    unsafe {
        let base = core::ptr::addr_of_mut!(RING) as *mut u8;
        for &b in src {
            let slot = write_pos % RING_SIZE;
            core::ptr::write_volatile(base.add(slot), b);
            write_pos += 1;
        }
    }
    HEAD.store(h + n, Ordering::Release);
}

/// Drain up to `buf.len()` bytes into `buf`.  Returns number of bytes copied.
pub fn pop_chunk(buf: &mut [u8]) -> usize {
    if buf.is_empty() { return 0; }

    let h = HEAD.load(Ordering::Acquire);
    let t = TAIL.load(Ordering::Relaxed);
    let available = h.saturating_sub(t);
    if available == 0 { return 0; }

    let n = core::cmp::min(available, buf.len());
    unsafe {
        let base = core::ptr::addr_of!(RING) as *const u8;
        for i in 0..n {
            let slot = (t + i) % RING_SIZE;
            buf[i] = core::ptr::read_volatile(base.add(slot));
        }
    }
    TAIL.store(t + n, Ordering::Release);
    n
}

/// Pop up to DRAIN_CHUNK bytes and shove them at the UART.  Intended for the
/// timer-tick hook.  Re-entrant-safe: bails if another drain is already in
/// flight on this CPU.
pub fn drain_to_uart() {
    if DRAINING.swap(true, Ordering::Acquire) {
        return; // another drain is already running
    }

    // First drain tick also flips the ring live, since by the time the timer
    // IRQ is firing the UART is guaranteed to be up.
    if !READY.load(Ordering::Acquire) {
        READY.store(true, Ordering::Release);
    }

    let mut buf = [0u8; DRAIN_CHUNK];
    let n = pop_chunk(&mut buf);
    if n > 0 {
        for &b in &buf[..n] {
            uart::putc(b);
        }
    }

    DRAINING.store(false, Ordering::Release);
}

/// Panic-path helper: bypass the ring entirely and blast `s` straight to the
/// UART.  Does not touch head/tail, so buffered logs are preserved for any
/// subsequent drain.
pub fn force_flush(s: &[u8]) {
    for &b in s {
        uart::putc(b);
    }
}

/// Drop everything currently buffered without emitting it.
pub fn clear() {
    let h = HEAD.load(Ordering::Acquire);
    TAIL.store(h, Ordering::Release);
}

/// V8-ROOT-2: drop any un-drained bytes from the previous cave. Without
/// this, a print from the outgoing cave could still be in the ring when
/// the next cave boots, leaking that cave's last stdout content out via
/// the serial console of the new cave's boot.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    clear();
    DRAINING.store(false, Ordering::Release);
    // Zero the backing ring so snapshots (e.g. crash dumps from a later
    // cave) don't recover stale bytes from a previous cave.
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RING) as *mut u8;
        for i in 0..RING_SIZE {
            core::ptr::write_volatile(ptr.add(i), 0);
        }
    }
}
