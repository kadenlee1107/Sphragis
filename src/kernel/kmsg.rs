//! Kernel message ring — dmesg-equivalent for non-security events.
//!
//! The audit ring (`security::audit`) is for security-relevant
//! events: it's sealed, length-capped, and overflow-warning. That's
//! the wrong place for "VirtIO GPU initialized" or "set frequency
//! to 2.4 GHz" or "loaded ELF binary at 0x...".
//!
//! kmsg is the other ring — plaintext, in-RAM, capped, lossy. Same
//! shape as Linux kmsg: ring of UTF-8 lines tagged with severity and
//! a monotonic timestamp. Operators read it via the shell, dump it
//! to UART, or forward via SIEM.
//!
//! Not encrypted because the operator already trusts the kernel that
//! emits it. The audit ring covers the cases where the *integrity* of
//! the log matters; kmsg covers "did this code path run."

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

const LINE_LEN: usize = 240;
const RING_CAP: usize = 512;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Severity {
    Trace = 0,
    Debug = 1,
    Info  = 2,
    Warn  = 3,
    Error = 4,
}

impl Severity {
    pub fn tag(self) -> &'static str {
        match self {
            Severity::Trace => "TRACE",
            Severity::Debug => "DEBUG",
            Severity::Info  => "INFO",
            Severity::Warn  => "WARN",
            Severity::Error => "ERROR",
        }
    }
}

#[derive(Clone, Copy)]
pub struct KmsgLine {
    pub ts: u64,
    pub sev: u8,
    pub mlen: u8,
    pub msg: [u8; LINE_LEN],
}

impl KmsgLine {
    pub const fn empty() -> Self {
        Self { ts: 0, sev: 0, mlen: 0, msg: [0; LINE_LEN] }
    }
}

static mut RING: [KmsgLine; RING_CAP] = [KmsgLine::empty(); RING_CAP];
static HEAD: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn now_ticks() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) v) };
    v
}

/// Record one log line. Truncates `msg` to LINE_LEN. Lossy — oldest
/// entries silently roll off when the ring is full.
pub fn log(sev: Severity, msg: &[u8]) {
    let n = if msg.len() > LINE_LEN { LINE_LEN } else { msg.len() };
    let slot = HEAD.fetch_add(1, Ordering::Relaxed) % RING_CAP;
    unsafe {
        let line = &mut (*core::ptr::addr_of_mut!(RING))[slot];
        line.ts = now_ticks();
        line.sev = sev as u8;
        line.mlen = n as u8;
        for i in 0..n { line.msg[i] = msg[i]; }
    }
}

/// Convenience macros — `kmsg::info(b"...")`, `kmsg::warn(b"...")`.
pub fn trace(msg: &[u8]) { log(Severity::Trace, msg); }
pub fn debug(msg: &[u8]) { log(Severity::Debug, msg); }
pub fn info (msg: &[u8]) { log(Severity::Info,  msg); }
pub fn warn (msg: &[u8]) { log(Severity::Warn,  msg); }
pub fn error(msg: &[u8]) { log(Severity::Error, msg); }

/// Mirror a kernel-info line to UART AND the kmsg ring. Used by
/// the boot path so a post-boot `dmesg` shows the same lines the
/// operator watched over serial. Without this the ring stays
/// empty on a clean boot — defeating the point of having a ring.
pub fn boot(msg: &str) {
    crate::drivers::uart::puts(msg);
    info(msg.trim_end_matches('\n').as_bytes());
}

/// Number of lines ever recorded.
pub fn head() -> usize { HEAD.load(Ordering::Relaxed) }

/// Read the most recent `n` lines (capped at RING_CAP). Returns a
/// snapshot — caller decides what to do with each line.
pub fn recent<F: FnMut(&KmsgLine)>(n: usize, mut f: F) {
    let h = HEAD.load(Ordering::Relaxed);
    let cap = if h < RING_CAP { h } else { RING_CAP };
    let take = if n < cap { n } else { cap };
    let start = h.saturating_sub(take);
    for i in start..h {
        let slot = i % RING_CAP;
        unsafe {
            let line = &(*core::ptr::addr_of!(RING))[slot];
            f(line);
        }
    }
}

/// Drain everything to a callback. Used by the shell `dmesg` command
/// and by the future SIEM forwarder.
pub fn drain<F: FnMut(&KmsgLine)>(f: F) {
    recent(RING_CAP, f);
}
