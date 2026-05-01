// Bat_OS — audit log (STUMP #103, Sprint 2.3).
//
// Append-only ring buffer for security-relevant events. Built so the
// renderer's hot paths (every fetch, every click, every script run)
// can call `record()` without touching disk. Operator dumps recent
// entries via the `audit` shell command, or flushes the whole buffer
// to BatFS as one encrypted blob with `audit-flush`.
//
// Format per entry:
//   timestamp (u64 ticks from cntpct_el0)
//   category  (Category enum, 1 byte)
//   message   (up to MSG_LEN bytes of operator-readable detail)
//
// Sensitive content (form bodies, passphrases, key material) MUST NOT
// be passed in. The `record()` callers below redact body contents and
// pass only counts + URLs / DOM indices / box numbers. Treat the log
// as "what the user did," not "what the user said."

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::drivers::uart;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Category {
    Fetch       = 1,  // GET / POST against a URL
    Script      = 2,  // JS engine started/finished
    Click       = 3,  // user-initiated click (real or simulated)
    Nav         = 4,  // explicit URL navigation (e.g. <a href> followed)
    FormSubmit  = 5,  // form POST with N inputs
    Mode        = 6,  // tls-mode / js-mode flipped
    Auth        = 7,  // login / logout / failed attempt
    Boot        = 8,  // kernel boot, cave switch
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Fetch      => "fetch",
            Category::Script     => "script",
            Category::Click      => "click",
            Category::Nav        => "nav",
            Category::FormSubmit => "form",
            Category::Mode       => "mode",
            Category::Auth       => "auth",
            Category::Boot       => "boot",
        }
    }
}

const MSG_LEN: usize = 192;
const RING_CAP: usize = 1024;

#[derive(Clone, Copy)]
pub struct Entry {
    pub ts:   u64,
    pub cat:  u8,           // Category as raw u8 so we can const-init.
    pub mlen: u8,
    pub msg:  [u8; MSG_LEN],
}

impl Entry {
    pub const fn empty() -> Self {
        Entry { ts: 0, cat: 0, mlen: 0, msg: [0; MSG_LEN] }
    }
}

static mut RING: [Entry; RING_CAP] = [Entry::empty(); RING_CAP];
/// Monotonically-increasing event counter. `RING[head % RING_CAP]` is
/// the next slot to write. We never decrement so `count - RING_CAP`
/// gives the index of the oldest still-resident entry.
static HEAD: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn now_ticks() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

/// Record an event. Truncates `msg` to MSG_LEN bytes. Cheap — single
/// store of an Entry into the ring + an atomic increment. Safe to call
/// from any kernel context.
pub fn record(cat: Category, msg: &[u8]) {
    let h = HEAD.fetch_add(1, Ordering::Relaxed);
    let slot = h % RING_CAP;
    let copy = msg.len().min(MSG_LEN);
    unsafe {
        let e = &mut RING[slot];
        e.ts = now_ticks();
        e.cat = cat as u8;
        e.mlen = copy as u8;
        e.msg[..copy].copy_from_slice(&msg[..copy]);
        if copy < MSG_LEN { e.msg[copy] = 0; }
    }
}

/// Dump the most-recent `n` entries to the UART (operator-visible).
/// Used by the `audit` shell command.
pub fn dump_tail(n: usize) {
    let total = HEAD.load(Ordering::Relaxed);
    if total == 0 { uart::puts("  audit: log is empty\n"); return; }
    let want = n.min(total).min(RING_CAP);
    let start = total - want;
    uart::puts("  audit: showing last ");
    crate::kernel::mm::print_num(want);
    uart::puts(" of ");
    crate::kernel::mm::print_num(total);
    uart::puts(" entries\n");
    for i in 0..want {
        let idx = (start + i) % RING_CAP;
        let e = unsafe { &RING[idx] };
        let cat = match e.cat {
            1 => "fetch",
            2 => "script",
            3 => "click",
            4 => "nav",
            5 => "form",
            6 => "mode",
            7 => "auth",
            8 => "boot",
            _ => "?",
        };
        uart::puts("  [");
        crate::kernel::mm::print_num((start + i) as usize);
        uart::puts("] ");
        uart::puts(cat);
        uart::puts(": ");
        let msg = unsafe { core::str::from_utf8_unchecked(&e.msg[..e.mlen as usize]) };
        uart::puts(msg);
        uart::puts("\n");
    }
}

/// Total events recorded since boot.
pub fn count() -> usize { HEAD.load(Ordering::Relaxed) }

/// Serialize the whole resident ring (oldest-first) into `out` as
/// newline-delimited records. Returns the number of bytes written.
/// Used by `audit-flush` to push the buffer into BatFS as one file.
pub fn serialize(out: &mut [u8]) -> usize {
    let total = HEAD.load(Ordering::Relaxed);
    if total == 0 { return 0; }
    let resident = total.min(RING_CAP);
    let start = total - resident;
    let mut pos = 0usize;
    for i in 0..resident {
        let idx = (start + i) % RING_CAP;
        let e = unsafe { &RING[idx] };
        let cat = match e.cat {
            1 => "fetch", 2 => "script", 3 => "click",
            4 => "nav",   5 => "form",   6 => "mode",
            7 => "auth",  8 => "boot",   _ => "?",
        };
        // ts cat msg\n — caller decodes ts.
        pos += write_u64(&mut out[pos..], e.ts);
        if pos < out.len() { out[pos] = b' '; pos += 1; }
        pos += copy_to(&mut out[pos..], cat.as_bytes());
        if pos < out.len() { out[pos] = b' '; pos += 1; }
        pos += copy_to(&mut out[pos..], &e.msg[..e.mlen as usize]);
        if pos < out.len() { out[pos] = b'\n'; pos += 1; }
        if pos >= out.len() { break; }
    }
    pos
}

fn copy_to(out: &mut [u8], src: &[u8]) -> usize {
    let n = src.len().min(out.len());
    out[..n].copy_from_slice(&src[..n]);
    n
}

fn write_u64(out: &mut [u8], mut v: u64) -> usize {
    if v == 0 {
        if !out.is_empty() { out[0] = b'0'; return 1; }
        return 0;
    }
    let mut buf = [0u8; 24];
    let mut i = 0;
    while v > 0 && i < buf.len() { buf[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    let len = i.min(out.len());
    for j in 0..len { out[j] = buf[i - 1 - j]; }
    len
}
