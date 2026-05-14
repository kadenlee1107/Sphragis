// Sphragis — Syscall history ring buffer for crash forensics.
//
// When the kernel's UNHANDLED dump fires (Chromium ret's to a cage
// pointer, glibc corrupts a saved LR, …) the register state at
// fault-time only tells us WHERE we crashed, not HOW we got here.
// x29 / x30 on the corrupted frame are already the bad values; the
// "stack LR candidates" scan reconstructs a few BL sites but misses
// the register state that was live when each call was made.
//
// This module captures a chronological ring of the last N syscalls
// with their register snapshots, keyed by thread. When the
// UNHANDLED path calls `dump()`, we print the ring so the operator
// can see (for each recent syscall):
//
//   tid    syscall    x0        x1        x29        x30      sp       elr
//   t1     98 futex   0x1bcff4  0x81      0x1bcff5   0x1c152  ...      ...
//   t2     131 tgkill 0x0       0x2       ...        ...      ...      ...
//   t1     (fault)    —         —         0x18001c   0x18001c ...      0x18001c
//
// Reading down the ring shows:
//   * What syscalls t1 was making just before the crash.
//   * Whether x29/x30 were already cage-pointers before the fault
//     (points at memory corruption somewhere earlier).
//   * Any syscall that passed a cage pointer as an argument (points
//     at a specific mis-wired glibc / Chromium call).
//
// Ring size: 64 entries × ~80 bytes = 5 KB kernel static. Cheap for
// forensics; turn off via `set_enabled(false)` to disable in
// production.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::drivers::uart;

/// Single captured syscall-entry event. We keep the most useful 8
/// registers — enough to reconstruct the last few calls.
#[derive(Clone, Copy)]
pub struct Entry {
    pub tid:         u32,
    pub _pad:        u32,
    pub seq:         u64,    // monotonically increasing sequence number
    pub syscall_num: u64,
    pub x0:          u64,
    pub x1:          u64,
    pub x2:          u64,
    pub x29:         u64,
    pub x30:         u64,
    pub sp_el0:      u64,
    pub elr:         u64,    // next user instruction after svc
}

impl Entry {
    const fn empty() -> Self {
        Self {
            tid: 0, _pad: 0, seq: 0, syscall_num: 0,
            x0: 0, x1: 0, x2: 0,
            x29: 0, x30: 0, sp_el0: 0, elr: 0,
        }
    }
}

const RING_SIZE: usize = 64;

static mut RING: [Entry; RING_SIZE] = [Entry::empty(); RING_SIZE];
static HEAD: AtomicU32 = AtomicU32::new(0);
static SEQ:  AtomicU64 = AtomicU64::new(0);
/// Record a syscall-entry snapshot. Called from the SVC dispatcher
/// in `arch/mod.rs` before the syscall runs so `x0..x5` reflect
/// the *arguments* (not the return value) and `x30` reflects the
/// caller's LR (what the post-svc `ret` will jump to).
pub fn record(tid: u32, syscall_num: u64, regs: &[u64; 31], elr: u64) {
    let seq = SEQ.fetch_add(1, Ordering::AcqRel);
    let idx = (HEAD.fetch_add(1, Ordering::AcqRel) as usize) % RING_SIZE;
    let sp_el0: u64;
    unsafe {
        core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0);
    }
    let e = Entry {
        tid,
        _pad: 0,
        seq,
        syscall_num,
        x0:  regs[0],
        x1:  regs[1],
        x2:  regs[2],
        x29: regs[29],
        x30: regs[30],
        sp_el0,
        elr,
    };
    unsafe {
        let ring = &mut *core::ptr::addr_of_mut!(RING);
        ring[idx] = e;
    }
}

/// Print the ring in chronological order (oldest → newest). Called
/// from the UNHANDLED-exception path in `arch/mod.rs` right before
/// the cave teardown / wedge.
pub fn dump() {
    uart::puts("  syscall history (last ");
    crate::kernel::mm::print_num(RING_SIZE);
    uart::puts(", oldest first):\n");

    // Walk the ring. HEAD points to the *next* slot to write, so the
    // oldest unseen entry is at HEAD (modulo size). Skip empty slots.
    let head = HEAD.load(Ordering::Acquire) as usize;
    for i in 0..RING_SIZE {
        let slot = (head + i) % RING_SIZE;
        let e = unsafe {
            let ring = &*core::ptr::addr_of!(RING);
            ring[slot]
        };
        if e.seq == 0 && e.tid == 0 && e.syscall_num == 0 { continue; }
        print_entry(&e);
    }
    uart::puts("  (end history)\n");
}

fn print_entry(e: &Entry) {
    uart::puts("    #");
    crate::kernel::mm::print_num(e.seq as usize);
    uart::puts(" t");
    crate::kernel::mm::print_num(e.tid as usize);
    uart::puts(" sc=");
    crate::kernel::mm::print_num(e.syscall_num as usize);
    uart::puts(" (");
    uart::puts(crate::batcave::linux::syscall::syscall_name(e.syscall_num));
    uart::puts(")");
    uart::puts("\n      x0=0x");  hex64(e.x0);
    uart::puts(" x1=0x");  hex64(e.x1);
    uart::puts(" x2=0x");  hex64(e.x2);
    uart::puts("\n      x29=0x"); hex64(e.x29);
    uart::puts(" x30=0x"); hex64(e.x30);
    uart::puts("\n      sp=0x"); hex64(e.sp_el0);
    uart::puts(" elr=0x"); hex64(e.elr);
    uart::puts("\n");
}

fn hex64(v: u64) {
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((v >> (sh * 4)) & 0xF) as usize]);
    }
}

/// Compact per-tid dump — show the LAST syscall for each TID seen in
/// the ring, with its ELR (the user-mode PC the syscall returned to).
/// At a deadlock that ELR points into the user code each thread is now
/// executing — combine with `rust-objdump` to identify what function
/// is hogging the CPU or sitting in a CPU loop.
pub fn dump_per_tid_last() {
    uart::puts("  per-tid last syscall + return PC:\n");
    let head = HEAD.load(Ordering::Acquire) as usize;
    // Track which TIDs we've already printed so we only show the most-recent
    // entry per TID. Walk newest → oldest.
    let mut seen: [u32; 32] = [0u32; 32];
    let mut seen_count = 0usize;
    for i in 0..RING_SIZE {
        let slot = (head + RING_SIZE - 1 - i) % RING_SIZE;
        let e = unsafe {
            let ring = &*core::ptr::addr_of!(RING);
            ring[slot]
        };
        if e.seq == 0 && e.tid == 0 && e.syscall_num == 0 { continue; }
        // Already saw a more-recent entry for this TID?
        let mut already = false;
        for j in 0..seen_count {
            if seen[j] == e.tid { already = true; break; }
        }
        if already { continue; }
        if seen_count < seen.len() {
            seen[seen_count] = e.tid;
            seen_count += 1;
        }
        // Compact one-liner: "tid=N sc=K (name) elr=0x..."
        uart::puts("    t");
        crate::kernel::mm::print_num(e.tid as usize);
        uart::puts(" sc=");
        crate::kernel::mm::print_num(e.syscall_num as usize);
        uart::puts(" (");
        uart::puts(crate::batcave::linux::syscall::syscall_name(e.syscall_num));
        uart::puts(") elr=0x");
        hex64(e.elr);
        uart::puts(" lr=0x");
        hex64(e.x30);
        uart::puts("\n");
    }
}

/// Read the most-recently-recorded entry. Useful when a syscall
/// handler needs to peek at the caller's LR (`x30`) for diagnostic
/// purposes — the dispatcher already captured it into the ring at
/// SVC entry, so we just look it up rather than re-plumbing the
/// trap frame through every handler.
pub fn last_entry() -> Option<Entry> {
    let head = HEAD.load(Ordering::Acquire) as usize;
    if head == 0 { return None; }
    let slot = (head - 1) % RING_SIZE;
    let e = unsafe {
        let ring = &*core::ptr::addr_of!(RING);
        ring[slot]
    };
    if e.seq == 0 && e.tid == 0 && e.syscall_num == 0 { return None; }
    Some(e)
}

/// Reset the ring. Called from `reset_cave_statics` so the next
/// cave starts with an empty history.
pub fn reset() {
    HEAD.store(0, Ordering::Release);
    SEQ.store(0, Ordering::Release);
    unsafe {
        let ring = &mut *core::ptr::addr_of_mut!(RING);
        for e in ring.iter_mut() {
            *e = Entry::empty();
        }
    }
}
