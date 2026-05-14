// Sphragis — KEEP_GOING / skip-and-log infrastructure.
//
// Goal: instead of bailing out at the first non-zero exit code, the
// first unknown syscall, or the first cave-fatal fault, *log* the
// event into a structured ring buffer and CONTINUE running. At
// cave-teardown the kernel dumps a summary so one smoke run produces
// a complete failure tree — every distinct issue we'd otherwise hit
// one-at-a-time is collected in a single trace.
//
// Wire-up policy: every skip site checks `is_enabled()`. When the
// flag is off (production / signed-initrd path) the original
// fail-fast behaviour is preserved.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::drivers::uart;

/// Global enable. Flipped by `main` based on the
/// `SPHRAGIS_KEEP_GOING` env var (compile-time today; runtime later).
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Hard cap so we don't run forever if a skip turns into an
/// infinite loop. Bumped only when we know what we're trading off.
const MAX_SKIPS: usize = 256;

/// Per-event slot.
#[derive(Clone, Copy)]
pub struct SkipEvent {
    pub seq: u32,
    pub kind: SkipKind,
    pub tid: u32,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub elr: u64,
    pub far: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkipKind {
    /// `sys_exit` / `sys_exit_group` with a non-zero code.
    /// a0 = exit code.
    Exit,
    /// Unknown syscall number — would have returned ENOSYS.
    /// a0 = syscall number, a1..a2 = first two args.
    UnknownSyscall,
    /// User-mode data abort skipped (PC advanced, dest reg zeroed).
    /// elr = faulting PC, far = FAR_EL1.
    UserDataAbort,
    /// User-mode instruction abort skipped.
    UserInstAbort,
    /// All threads in the cave are blocked (FutexWait/EpollWait/Join/etc).
    /// Scheduler force-woke one to keep the run going.
    /// a0 = woken tid, a1 = uaddr/epfd/etc, a2 = block-reason discriminant.
    FutexDeadlock,
}

impl SkipKind {
    pub fn name(self) -> &'static str {
        match self {
            SkipKind::Exit => "EXIT",
            SkipKind::UnknownSyscall => "UNKNOWN_SYSCALL",
            SkipKind::UserDataAbort => "USER_DATA_ABORT",
            SkipKind::UserInstAbort => "USER_INST_ABORT",
            SkipKind::FutexDeadlock => "FUTEX_DEADLOCK",
        }
    }
}

const RING_SIZE: usize = MAX_SKIPS;

static mut RING: [SkipEvent; RING_SIZE] = [SkipEvent {
    seq: 0,
    kind: SkipKind::Exit,
    tid: 0,
    a0: 0,
    a1: 0,
    a2: 0,
    elr: 0,
    far: 0,
}; RING_SIZE];
static SEQ: AtomicU32 = AtomicU32::new(0);
static COUNT: AtomicU32 = AtomicU32::new(0);

/// Per-kind tally so the summary doesn't have to walk the ring twice.
static N_EXIT: AtomicU32 = AtomicU32::new(0);
static N_UNK_SC: AtomicU32 = AtomicU32::new(0);
static N_DABT: AtomicU32 = AtomicU32::new(0);
static N_IABT: AtomicU32 = AtomicU32::new(0);
static N_DEADLOCK: AtomicU32 = AtomicU32::new(0);

pub fn enable() {
    ENABLED.store(true, Ordering::Release);
    uart::puts("[skip] SPHRAGIS_KEEP_GOING enabled — ");
    uart::puts("kernel will skip-and-log instead of cave-terminate.\n");
}

#[inline]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// Reset the ring + counters. Called when the shell starts a fresh
/// `chromium` invocation so each run produces an independent map.
pub fn reset() {
    SEQ.store(0, Ordering::Release);
    COUNT.store(0, Ordering::Release);
    N_EXIT.store(0, Ordering::Release);
    N_UNK_SC.store(0, Ordering::Release);
    N_DABT.store(0, Ordering::Release);
    N_IABT.store(0, Ordering::Release);
    N_DEADLOCK.store(0, Ordering::Release);
}

/// Record an event. Returns `true` if the caller is permitted to
/// continue (we're under cap), `false` if we've exceeded MAX_SKIPS
/// and the caller should fall back to its old fail-fast path.
pub fn record(kind: SkipKind, tid: u32, a0: u64, a1: u64, a2: u64,
              elr: u64, far: u64) -> bool {
    let n = COUNT.fetch_add(1, Ordering::AcqRel);
    if (n as usize) >= MAX_SKIPS {
        return false;
    }
    let seq = SEQ.fetch_add(1, Ordering::AcqRel);
    let ev = SkipEvent { seq, kind, tid, a0, a1, a2, elr, far };
    unsafe {
        let ring = &mut *core::ptr::addr_of_mut!(RING);
        ring[(n as usize) % RING_SIZE] = ev;
    }
    match kind {
        SkipKind::Exit            => { N_EXIT.fetch_add(1, Ordering::Relaxed); }
        SkipKind::UnknownSyscall  => { N_UNK_SC.fetch_add(1, Ordering::Relaxed); }
        SkipKind::UserDataAbort   => { N_DABT.fetch_add(1, Ordering::Relaxed); }
        SkipKind::UserInstAbort   => { N_IABT.fetch_add(1, Ordering::Relaxed); }
        SkipKind::FutexDeadlock   => { N_DEADLOCK.fetch_add(1, Ordering::Relaxed); }
    }
    // One-line trace per event so a `grep '^\[SKIP'` against the
    // serial log reproduces the full timeline.
    uart::puts("[SKIP ");
    print_num(seq as u64);
    uart::puts(" ");
    uart::puts(kind.name());
    uart::puts(" tid=");
    print_num(tid as u64);
    uart::puts(" a0=0x");
    print_hex64(a0);
    uart::puts(" a1=0x");
    print_hex64(a1);
    uart::puts(" a2=0x");
    print_hex64(a2);
    uart::puts(" elr=0x");
    print_hex64(elr);
    uart::puts(" far=0x");
    print_hex64(far);
    uart::puts("]\n");
    true
}

/// Print a per-call summary at end of run. Format is parser-friendly:
///
/// ```
/// [SKIP-SUMMARY total=N exit=A unk=B dabt=C iabt=D]
/// [SKIP-DETAIL kind=K count=N example_a0=X example_elr=Y]   (one per uniq kind+a0)
/// ```
pub fn dump_summary() {
    if !is_enabled() { return; }
    let total = COUNT.load(Ordering::Acquire);
    if total == 0 {
        uart::puts("[SKIP-SUMMARY total=0]\n");
        return;
    }
    uart::puts("[SKIP-SUMMARY total=");
    print_num(total as u64);
    uart::puts(" exit=");
    print_num(N_EXIT.load(Ordering::Relaxed) as u64);
    uart::puts(" unk_sc=");
    print_num(N_UNK_SC.load(Ordering::Relaxed) as u64);
    uart::puts(" dabt=");
    print_num(N_DABT.load(Ordering::Relaxed) as u64);
    uart::puts(" iabt=");
    print_num(N_IABT.load(Ordering::Relaxed) as u64);
    uart::puts(" deadlock=");
    print_num(N_DEADLOCK.load(Ordering::Relaxed) as u64);
    uart::puts("]\n");

    // De-dup by (kind, a0) — collapse repeated unknown-syscalls of
    // the same number into a single row, etc. Up to 64 distinct
    // signatures per run; if we exceed that the trailer says so.
    let n_in_ring = core::cmp::min(total as usize, RING_SIZE);
    const MAX_UNIQ: usize = 64;
    let mut keys: [(u8, u64); MAX_UNIQ] = [(0, 0); MAX_UNIQ];
    let mut counts: [u32; MAX_UNIQ] = [0; MAX_UNIQ];
    let mut elrs:   [u64; MAX_UNIQ] = [0; MAX_UNIQ];
    let mut far_examples: [u64; MAX_UNIQ] = [0; MAX_UNIQ];
    let mut n_uniq: usize = 0;
    let mut overflow = false;

    for i in 0..n_in_ring {
        let ev = unsafe {
            let ring = &*core::ptr::addr_of!(RING);
            ring[i]
        };
        let kind_byte = ev.kind as u8;
        let mut hit: Option<usize> = None;
        for j in 0..n_uniq {
            if keys[j].0 == kind_byte && keys[j].1 == ev.a0 {
                hit = Some(j); break;
            }
        }
        match hit {
            Some(j) => counts[j] = counts[j].saturating_add(1),
            None => {
                if n_uniq < MAX_UNIQ {
                    keys[n_uniq] = (kind_byte, ev.a0);
                    counts[n_uniq] = 1;
                    elrs[n_uniq] = ev.elr;
                    far_examples[n_uniq] = ev.far;
                    n_uniq += 1;
                } else {
                    overflow = true;
                }
            }
        }
    }

    for j in 0..n_uniq {
        uart::puts("[SKIP-DETAIL kind=");
        let name = match keys[j].0 {
            0 => "EXIT",
            1 => "UNKNOWN_SYSCALL",
            2 => "USER_DATA_ABORT",
            3 => "USER_INST_ABORT",
            4 => "FUTEX_DEADLOCK",
            _ => "OTHER",
        };
        uart::puts(name);
        uart::puts(" a0=0x");
        print_hex64(keys[j].1);
        uart::puts(" count=");
        print_num(counts[j] as u64);
        uart::puts(" elr=0x");
        print_hex64(elrs[j]);
        uart::puts(" far=0x");
        print_hex64(far_examples[j]);
        uart::puts("]\n");
    }
    if overflow {
        uart::puts("[SKIP-DETAIL truncated — more than 64 distinct (kind,a0) pairs]\n");
    }
}

#[inline]
fn print_num(n: u64) {
    crate::kernel::mm::print_num(n as usize);
}

#[inline]
fn print_hex64(v: u64) {
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((v >> (sh * 4)) & 0xF) as usize]);
    }
}
