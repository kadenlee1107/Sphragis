//! Net activity ring — fixed-size circular buffer of recent network
//! events. Painted by the Wave-4 NET cockpit's ACTIVITY panel.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

pub const RING_CAP: usize = 256;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ActivityKind {
    Dns             = 0,
    TcpOpen         = 1,
    TcpClose        = 2,
    FwDrop          = 3,
    TlsHs           = 4,
    CountersCleared = 5,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::Dns             => "dns",
            ActivityKind::TcpOpen         => "tcp_open",
            ActivityKind::TcpClose        => "tcp_close",
            ActivityKind::FwDrop          => "fw_drop",
            ActivityKind::TlsHs           => "tls_hs",
            ActivityKind::CountersCleared => "counters_cleared",
        }
    }
    pub fn from_u8(b: u8) -> ActivityKind {
        match b {
            1 => ActivityKind::TcpOpen,
            2 => ActivityKind::TcpClose,
            3 => ActivityKind::FwDrop,
            4 => ActivityKind::TlsHs,
            5 => ActivityKind::CountersCleared,
            _ => ActivityKind::Dns,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Entry {
    pub ts:          u64,    // monotonic seconds since boot
    pub kind:        u8,     // ActivityKind discriminant
    pub summary:     [u8; 96],
    pub summary_len: u8,
}

impl Entry {
    pub const fn empty() -> Self {
        Self { ts: 0, kind: 0, summary: [0; 96], summary_len: 0 }
    }
    pub fn summary_str(&self) -> &str {
        let n = (self.summary_len as usize).min(self.summary.len());
        unsafe { core::str::from_utf8_unchecked(&self.summary[..n]) }
    }
}

static mut RING: [Entry; RING_CAP] = [Entry::empty(); RING_CAP];
static HEAD:  AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);

/// Push a new activity entry. Called from net subsystem call sites.
pub fn push(kind: ActivityKind, summary: &str) {
    let mut e = Entry::empty();
    e.ts = crate::kernel::time::monotonic_secs();
    e.kind = kind as u8;
    let bytes = summary.as_bytes();
    let mut snap = bytes.len().min(e.summary.len());
    while snap > 0 && !summary.is_char_boundary(snap) { snap -= 1; }
    e.summary[..snap].copy_from_slice(&bytes[..snap]);
    e.summary_len = snap as u8;

    let head = HEAD.fetch_add(1, Ordering::Relaxed) % RING_CAP;
    unsafe { *core::ptr::addr_of_mut!(RING[head]) = e; }
    let prev = COUNT.load(Ordering::Relaxed);
    if prev < RING_CAP { COUNT.store(prev + 1, Ordering::Relaxed); }
}

/// Iterate newest-first. Calls `f` with each entry until f returns false.
pub fn iter_newest_first<F: FnMut(&Entry) -> bool>(mut f: F) {
    let count = COUNT.load(Ordering::Relaxed);
    let head  = HEAD.load(Ordering::Relaxed);
    for i in 0..count {
        let idx = (head + RING_CAP - 1 - i) % RING_CAP;
        let entry = unsafe { &*core::ptr::addr_of!(RING[idx]) };
        if !f(entry) { break; }
    }
}

pub fn count() -> usize {
    COUNT.load(Ordering::Relaxed)
}

pub fn clear() {
    COUNT.store(0, Ordering::Relaxed);
    HEAD.store(0, Ordering::Relaxed);
}
