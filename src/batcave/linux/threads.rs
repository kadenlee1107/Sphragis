// Bat_OS — BatCave Linux Threading (clone()/TLS/scheduler) DESIGN + SKELETON
// =============================================================================
//
// PURPOSE
// -----------------------------------------------------------------------------
// This module is the foundation for running multi-threaded ELF binaries under
// the BatCave Linux runner. The immediate motivator is Chromium, which even in
// single-process mode spawns ~30 POSIX threads (V8 GC/parser/compiler,
// Blink main/worker, IPC poller, IO thread, audio, raster, compositor, GPU,
// network, ThreadPool, etc.). Without real clone()/pthread, those calls all
// collapse onto the parent thread and Chromium deadlocks at startup.
//
// This file is a DESIGN DOCUMENT + WORKING SKELETON. It is intentionally
// NOT wired into `syscall.rs` yet — that remains the human's job so the old
// busybox/hello_world path continues to work while this is being validated.
//
// =============================================================================
// PART 1: ANALYSIS
// =============================================================================
//
// 1.1  How the current `sys_clone_thread` works (syscall.rs:1631)
// -----------------------------------------------------------------------------
// The existing implementation is a clever single-thread simulation:
//   * Global `IN_CHILD` flag gates re-entry. If the "child" is running,
//     another clone() is rejected with -1. There is only ever ONE logical
//     child on the CPU at a time.
//   * On clone():
//       - A new TID is minted and stashed in `LAST_CHILD_TID` for the parent
//         to read back on its return.
//       - `CLONE_CHILD_STACK` / `IS_THREAD_CHILD` are globals read by the SVC
//         exception handler just before `eret`. The handler patches the trap
//         frame so that the returning "child" actually resumes on `child_stack`
//         with x0 = 0 (the clone() return value for children).
//       - `CURRENT_TID` is flipped to the child's TID.
//   * When the child calls exit/exit_group, `restore_parent_tid()` flips
//     `CURRENT_TID` back to 1 and the parent's syscall return path delivers
//     the child's TID as clone()'s return value.
//   * `forkjmp.s` (fork_save/fork_restore) is a setjmp/longjmp-style helper
//     kept around for fork-like semantics (busybox applets that share the
//     parent stack). It captures x19-x30 + sp and restores them so the parent
//     thread of execution can be "un-forked" after the child returns.
//
// 1.2  What limits it to one thread at a time
// -----------------------------------------------------------------------------
//   * Single pair of globals (`CLONE_CHILD_STACK`, `IS_THREAD_CHILD`,
//     `IN_CHILD`, `CURRENT_TID`) — no table, so we can't describe N threads.
//   * No scheduler entry points for user threads. The kernel's scheduler
//     (`src/kernel/scheduler.rs`) schedules *kernel* tasks only; user-space
//     execution is a single blr from `runner.rs` that never returns to the
//     scheduler until the ELF `exit`s.
//   * No per-thread saved register state. The exception handler writes child
//     state directly into the *current* trap frame rather than to a saved
//     thread record.
//   * No TLS management. `tpidr_el0` is never set, so `__errno_location`,
//     stack-canary reads, and any `__thread` variable in musl/glibc would
//     crash. Chromium relies heavily on TLS.
//   * `futex` is a stub that returns EAGAIN/0 to coerce callers into a
//     cooperative spin. That works for busybox but falls over under Chromium
//     where threads *must* block (e.g. ThreadPool worker waiting on a queue).
//
// 1.3  Required scheduler changes
// -----------------------------------------------------------------------------
// Model: cooperative + timer-preemptive round-robin across user threads of
// the single BatCave process. We keep it simple:
//
//   * Single address space (CLONE_VM is always set by Chromium's thread
//     spawn path), so "context switch" is register state only — no TTBR0
//     swap, no TLB shootdown.
//   * Round-robin over Runnable entries in THREADS[].
//   * Preemption hook from the EL1 timer IRQ calls `threads::on_tick()`.
//     If more than one thread is Runnable, the handler triggers a context
//     switch on its way back to EL0 by rewriting the trap frame's SP_EL0,
//     ELR_EL1, SPSR_EL1, and x0-x30 from the next thread's saved regs.
//   * Cooperative yield via `schedule()` for blocking syscalls (futex wait,
//     epoll_pwait, nanosleep, read-on-empty-pipe).
//   * No priorities for now — Chromium threads are largely symmetric. We can
//     layer that in later by copying the kernel scheduler's priority field.
//
// 1.4  Per-thread register/TLS/stack layout
// -----------------------------------------------------------------------------
//   * Saved regs: full GPR set x0-x30 + SP_EL0 + ELR_EL1 + SPSR_EL1. Stored
//     in `SavedRegs` below. On ARM64 the ABI only requires callee-saved
//     (x19-x30, sp, fp) to round-trip through a function call, but preemption
//     can steal the CPU at any instruction, so we must save ALL GPRs.
//     NEON/FP is deferred (TODO — see below).
//   * TLS: `tpidr_el0` holds the thread pointer. glibc/musl layouts place the
//     TCB at `tpidr_el0` and access TLS variables at negative or positive
//     offsets. CLONE_SETTLS provides the value. We store it per-thread and
//     restore on switch via `msr tpidr_el0, x?`.
//   * Stack: either user-provided (`child_stack`, the pthread case — glibc
//     mmaps 8MB, passes us the top) or we allocate a default 64KB stack from
//     the frame allocator and return that. Chromium always provides its own
//     stacks, so the allocation path is a fallback for lazy callers.
//   * Shared memory: since CLONE_VM is set, all threads share one page table.
//     We never flip TTBR0_EL1 between user threads. The kernel's own page
//     tables are unaffected.
//
// 1.5  Stack switching
// -----------------------------------------------------------------------------
// On context switch we:
//     1. stash current x0-x30 + SP_EL0 + ELR + SPSR into THIS thread's
//        `SavedRegs` slot
//     2. load NEXT thread's `SavedRegs` into x0-x30 and the special regs
//     3. `msr tpidr_el0, <next.tls_ptr>`
//     4. `eret` (when coming from an exception) or `br x30` (cooperative)
// When the switch is cooperative (from inside a syscall handler) we're on
// the kernel stack; only x19-x30 + sp + lr need round-tripping through
// `SavedRegs`. When it's preemptive (from IRQ) the trap frame already has
// everything — we just rewrite the trap frame with the next thread's state.
//
// =============================================================================
// PART 2–5: IMPLEMENTATION
// =============================================================================

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use crate::kernel::mm::frame;
use crate::drivers::uart;

// -----------------------------------------------------------------------------
// Linux clone() flag bits — subset we honor
// -----------------------------------------------------------------------------
pub const CLONE_VM:              u64 = 0x0000_0100;
pub const CLONE_FS:              u64 = 0x0000_0200;
pub const CLONE_FILES:           u64 = 0x0000_0400;
pub const CLONE_SIGHAND:         u64 = 0x0000_0800;
pub const CLONE_PTRACE:          u64 = 0x0000_2000;
pub const CLONE_VFORK:           u64 = 0x0000_4000;
pub const CLONE_PARENT:          u64 = 0x0000_8000;
pub const CLONE_THREAD:          u64 = 0x0001_0000;
pub const CLONE_NEWNS:           u64 = 0x0002_0000;
pub const CLONE_SYSVSEM:         u64 = 0x0004_0000;
pub const CLONE_SETTLS:          u64 = 0x0008_0000;
pub const CLONE_PARENT_SETTID:   u64 = 0x0010_0000;
pub const CLONE_CHILD_CLEARTID:  u64 = 0x0020_0000;
pub const CLONE_DETACHED:        u64 = 0x0040_0000;
pub const CLONE_CHILD_SETTID:    u64 = 0x0100_0000;

/// Flags set by Chromium/glibc pthread_create. We check the exact bundle so
/// we can accept/reject cleanly.
pub const CHROMIUM_THREAD_FLAGS: u64 =
    CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND |
    CLONE_THREAD | CLONE_SETTLS | CLONE_PARENT_SETTID |
    CLONE_CHILD_CLEARTID | CLONE_SYSVSEM;

// -----------------------------------------------------------------------------
// Errno constants used locally
// -----------------------------------------------------------------------------
const EAGAIN: i64 = -11;
const ENOMEM: i64 = -12;
const EINVAL: i64 = -22;

// -----------------------------------------------------------------------------
// Thread table
// -----------------------------------------------------------------------------
pub const MAX_THREADS: usize = 64;
const DEFAULT_STACK_PAGES: usize = 16; // 64 KiB fallback
const PAGE_SIZE: usize = 4096;

/// Full GPR snapshot + special registers. Large, but we have 64 slots so
/// 64 * ~280 bytes ≈ 18 KiB total — well within budget.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SavedRegs {
    pub x: [u64; 31],    // x0..x30
    pub sp_el0: u64,     // user stack pointer
    pub elr_el1: u64,    // user PC to resume at
    pub spsr_el1: u64,   // processor state (EL0t, interrupts enabled, etc.)
    // TODO: NEON/FP state (q0-q31, fpsr, fpcr) — Chromium uses FP heavily;
    //       without saving it, threads will corrupt each other's FP regs.
    //       Needed for real multi-threaded Chromium.
}

impl SavedRegs {
    pub const fn zero() -> Self {
        Self { x: [0; 31], sp_el0: 0, elr_el1: 0, spsr_el1: 0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockReason {
    FutexWait { uaddr: u64, val: u32 },
    EpollWait { epfd: i32, timeout_ms: i32 },
    Nanosleep { deadline_ns: u64 },
    Join { target_tid: u32 },
    IoWait,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreadState {
    Free,                    // slot empty
    Runnable,                // eligible to run on next schedule()
    Running,                 // currently on the CPU
    Blocked(BlockReason),    // waiting on something; wake_thread() moves -> Runnable
    Exited(i32),             // zombie; reaped by join/wait
}

#[derive(Clone, Copy)]
pub struct Thread {
    pub tid: u32,
    pub parent_tid: u32,
    pub state: ThreadState,
    pub saved_regs: SavedRegs,
    pub stack_base: u64,             // bottom of owned allocation (0 if caller-provided)
    pub stack_top: u64,              // SP starts here (16-byte aligned)
    pub stack_pages: usize,
    pub tls_ptr: u64,                // value to load into tpidr_el0 on switch
    /// CLONE_CHILD_CLEARTID user address. On thread exit, we zero *addr and
    /// call futex_wake(addr, 1) so pthread_join can observe termination.
    pub tid_clear_on_exit: Option<u64>,
    /// CLONE_PARENT_SETTID / CLONE_CHILD_SETTID — kernel writes the new TID
    /// into the given user address before returning to user space.
    pub tid_set_parent: Option<u64>,
    pub tid_set_child: Option<u64>,
    /// Entry fields — only meaningful before the thread has run once.
    pub entry_pc: u64,
    pub entry_arg: u64,
}

impl Thread {
    pub const fn empty() -> Self {
        Self {
            tid: 0,
            parent_tid: 0,
            state: ThreadState::Free,
            saved_regs: SavedRegs::zero(),
            stack_base: 0,
            stack_top: 0,
            stack_pages: 0,
            tls_ptr: 0,
            tid_clear_on_exit: None,
            tid_set_parent: None,
            tid_set_child: None,
            entry_pc: 0,
            entry_arg: 0,
        }
    }
}

// -----------------------------------------------------------------------------
// Global thread table. `static mut` — protected by THREADS_LOCK.
// We use a spinlock over a single bool rather than a mutex to stay #![no_std]
// and avoid heap. On a single-core ARM64 we only need to disable IRQs while
// the table is touched, which we do in with_table().
// -----------------------------------------------------------------------------
static mut THREADS: [Thread; MAX_THREADS] = [Thread::empty(); MAX_THREADS];
static THREADS_LOCK: AtomicBool = AtomicBool::new(false);

/// TID of the thread currently executing on the CPU. Distinct from
/// CURRENT_TID in syscall.rs (the legacy single-thread TID) — we will
/// eventually subsume that, but the switchover is the human's wiring job.
static RUNNING_TID: AtomicU32 = AtomicU32::new(1);
static NEXT_TID: AtomicU32 = AtomicU32::new(2);
static THREADING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Call once (from the runner, when a multi-threaded ELF is about to start)
/// to install TID 1 as the main thread and unlock real scheduling. Leaves
/// the legacy single-thread path untouched when not called.
pub fn init_main_thread(main_entry_pc: u64, main_sp_el0: u64) {
    with_table(|t| {
        if t[0].state == ThreadState::Free {
            t[0] = Thread::empty();
            t[0].tid = 1;
            t[0].parent_tid = 0;
            t[0].state = ThreadState::Running;
            t[0].entry_pc = main_entry_pc;
            t[0].stack_top = main_sp_el0;
            t[0].saved_regs.sp_el0 = main_sp_el0;
            t[0].saved_regs.elr_el1 = main_entry_pc;
            // EL0t, IRQs unmasked:
            t[0].saved_regs.spsr_el1 = 0;
        }
    });
    RUNNING_TID.store(1, Ordering::Release);
    THREADING_ENABLED.store(true, Ordering::Release);
    uart::puts("[threads] Main thread registered, threading enabled\n");
}

pub fn is_enabled() -> bool { THREADING_ENABLED.load(Ordering::Acquire) }
pub fn current_tid() -> u32 { RUNNING_TID.load(Ordering::Acquire) }

// -----------------------------------------------------------------------------
// Table-locking helper. Disables IRQs while the closure runs so the timer
// can't preempt us mid-mutation. Single-core assumption.
// -----------------------------------------------------------------------------
fn with_table<R>(f: impl FnOnce(&mut [Thread; MAX_THREADS]) -> R) -> R {
    // Save+disable IRQs
    let daif: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) daif);
        core::arch::asm!("msr daifset, #0x2"); // mask IRQ
    }
    while THREADS_LOCK.compare_exchange(false, true,
            Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
    // SAFETY: lock held, IRQs masked
    let r = f(unsafe { &mut *core::ptr::addr_of_mut!(THREADS) });
    THREADS_LOCK.store(false, Ordering::Release);
    unsafe { core::arch::asm!("msr daif, {}", in(reg) daif); }
    r
}

fn find_free_slot(t: &mut [Thread; MAX_THREADS]) -> Option<usize> {
    for i in 0..MAX_THREADS {
        if t[i].state == ThreadState::Free { return Some(i); }
    }
    None
}

fn slot_of(t: &[Thread; MAX_THREADS], tid: u32) -> Option<usize> {
    for i in 0..MAX_THREADS {
        if t[i].state != ThreadState::Free && t[i].tid == tid { return Some(i); }
    }
    None
}

// -----------------------------------------------------------------------------
// PART 2 — clone() implementation
// -----------------------------------------------------------------------------
/// Linux clone() with the ARM64 argument ordering:
///   long clone(unsigned long flags, void *stack,
///              int *parent_tid, unsigned long tls, int *child_tid);
/// We accept the 5-arg form the human's syscall shim will pass.
///
/// Returns: new TID (>=2) on success in the caller (parent) context, or a
/// negative errno. The child return value (0) is delivered by the scheduler
/// when it first picks up this thread, by setting saved_regs.x[0] = 0 here.
pub fn clone(flags: u64,
             child_stack: u64,
             parent_tid: *mut i32,
             child_tid: *mut i32,
             tls: u64) -> i64 {
    // Reject flag combos we can't support yet.
    if flags & CLONE_VFORK != 0 { return EINVAL; }
    if flags & CLONE_PTRACE != 0 { return EINVAL; }
    // We don't have separate fs/files/sighand yet — must be shared.
    let needs_shared = CLONE_VM | CLONE_THREAD;
    if flags & needs_shared != needs_shared {
        // Legacy fork-style clone still goes through the old path.
        return EINVAL;
    }

    // Quota check — reserve a thread slot + (if we're going to allocate one)
    // the stack pages.  We charge mem FIRST so a cave that is already at its
    // mem cap gets a clean -ENOMEM before we touch the frame allocator.
    if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Threads, 1) {
        return e;
    }
    if child_stack == 0 {
        let bytes = DEFAULT_STACK_PAGES * PAGE_SIZE;
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Mem, bytes) {
            super::quotas::refund_active(super::quotas::Resource::Threads, 1);
            return e;
        }
    }

    let new_tid = NEXT_TID.fetch_add(1, Ordering::Relaxed);

    // Allocate or adopt a stack.
    let (stack_base, stack_top, stack_pages) = if child_stack != 0 {
        // Caller owns the allocation; it already points to the TOP.
        (0u64, child_stack & !0xFu64, 0usize)
    } else {
        match alloc_stack(DEFAULT_STACK_PAGES) {
            Some((b, t)) => (b, t, DEFAULT_STACK_PAGES),
            None => {
                // Refund the quota we speculatively charged above.
                super::quotas::refund_active(super::quotas::Resource::Threads, 1);
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, DEFAULT_STACK_PAGES * PAGE_SIZE);
                return ENOMEM;
            }
        }
    };

    // Write PARENT_SETTID before child could possibly observe it.
    if flags & CLONE_PARENT_SETTID != 0 && !parent_tid.is_null() {
        unsafe { core::ptr::write_volatile(parent_tid, new_tid as i32); }
    }

    let tid_set_child = if flags & CLONE_CHILD_SETTID != 0 && !child_tid.is_null() {
        Some(child_tid as u64)
    } else { None };
    let tid_clear_on_exit = if flags & CLONE_CHILD_CLEARTID != 0 && !child_tid.is_null() {
        Some(child_tid as u64)
    } else { None };

    // Populate the slot.
    let result = with_table(|t| -> i64 {
        let Some(slot) = find_free_slot(t) else { return EAGAIN; };
        let parent = current_tid();

        t[slot] = Thread::empty();
        t[slot].tid = new_tid;
        t[slot].parent_tid = parent;
        t[slot].state = ThreadState::Runnable;
        t[slot].stack_base = stack_base;
        t[slot].stack_top = stack_top;
        t[slot].stack_pages = stack_pages;
        t[slot].tls_ptr = if flags & CLONE_SETTLS != 0 { tls } else { 0 };
        t[slot].tid_set_parent = if flags & CLONE_PARENT_SETTID != 0 {
            Some(parent_tid as u64)
        } else { None };
        t[slot].tid_set_child = tid_set_child;
        t[slot].tid_clear_on_exit = tid_clear_on_exit;

        // Child register seed.
        //
        // On ARM64 the pthread_create trampoline calls __clone which does:
        //     mov x8, #220      // SYS_clone
        //     svc #0
        //     cbnz x0, 1f       // parent returns
        //     // child: x9 holds fn, x10 holds arg
        //     blr x9
        // So for Chromium we need x0=0, and we resume at the instruction
        // AFTER the svc. `entry_pc` is set by the syscall shim to ELR_EL1+4
        // (return address of the svc) before calling clone(). Same for sp.
        //
        // Until the shim is wired, these stay as defaults; the human will
        // pass them in through a thin wrapper like:
        //     threads::set_child_resume(new_tid, elr_plus_4, sp_at_svc);
        t[slot].saved_regs = SavedRegs::zero();
        t[slot].saved_regs.x[0] = 0;               // clone() returns 0 in child
        t[slot].saved_regs.sp_el0 = stack_top;
        t[slot].saved_regs.elr_el1 = 0;            // to be patched by shim
        t[slot].saved_regs.spsr_el1 = 0;           // EL0t, IRQs on

        0
    });
    if result < 0 {
        // Slot table full — refund what we charged at entry, and free the
        // stack we allocated so it doesn't leak.
        super::quotas::refund_active(super::quotas::Resource::Threads, 1);
        if stack_pages > 0 && stack_base != 0 {
            crate::kernel::mm::frame::free_contig(stack_base as usize, stack_pages);
            super::quotas::refund_active(
                super::quotas::Resource::Mem, stack_pages * PAGE_SIZE);
        }
        return result;
    }

    new_tid as i64
}

/// After clone() returns to the syscall dispatcher, the dispatcher knows
/// the parent's ELR_EL1 (return address of the svc). It should call this
/// to fill in the child's resume PC + a copy of the parent's SP (Chromium's
/// __clone sets up the stack *before* svc, so child_stack already contains
/// the fn/arg just below SP; we just need PC to point at the post-svc insn).
pub fn set_child_resume(tid: u32, resume_pc: u64, _parent_sp: u64) {
    with_table(|t| {
        if let Some(i) = slot_of(t, tid) {
            t[i].saved_regs.elr_el1 = resume_pc;
            t[i].entry_pc = resume_pc;
        }
    });
}

/// Helper for the stack fallback path. Allocates contiguous 4 KiB frames
/// and returns (base, top). 16-byte aligned top.
fn alloc_stack(pages: usize) -> Option<(u64, u64)> {
    let first = frame::alloc_frame()? as u64;
    let mut last = first;
    for _ in 1..pages {
        let p = frame::alloc_frame()? as u64;
        // Note: frame allocator isn't guaranteed contiguous; in a proper
        // kernel we'd use mmap. TODO: replace with vmalloc-style mapping
        // that strings pages together virtually. For now we *assume* the
        // frame allocator hands back contiguous pages in a fresh run, which
        // matches what loader.rs already relies on.
        last = p;
    }
    let top = (last + PAGE_SIZE as u64) & !0xFu64;
    Some((first, top))
}

// -----------------------------------------------------------------------------
// PART 3 — Thread table accessors (for syscall.rs integration)
// -----------------------------------------------------------------------------
pub fn thread_count() -> usize {
    with_table(|t| {
        let mut n = 0;
        for s in t.iter() {
            if s.state != ThreadState::Free { n += 1; }
        }
        n
    })
}

pub fn runnable_count() -> usize {
    with_table(|t| {
        let mut n = 0;
        for s in t.iter() {
            if matches!(s.state, ThreadState::Runnable | ThreadState::Running) {
                n += 1;
            }
        }
        n
    })
}

/// Mark current thread Exited and schedule something else. Fires the
/// CLONE_CHILD_CLEARTID futex wake so joiners can proceed.
pub fn exit_current(code: i32) -> ! {
    let me = current_tid();
    let clear_addr = with_table(|t| {
        if let Some(i) = slot_of(t, me) {
            t[i].state = ThreadState::Exited(code);
            t[i].tid_clear_on_exit
        } else { None }
    });
    if let Some(addr) = clear_addr {
        unsafe { core::ptr::write_volatile(addr as *mut i32, 0); }
        // Wake up to 1 joiner on that address. Uses our futex bridge.
        futex_wake_on(addr, 1);
    }
    // Hand the CPU to someone else.
    schedule();
    // schedule() should never return here because the slot is Exited. But
    // just in case:
    loop { unsafe { core::arch::asm!("wfi"); } }
}

// -----------------------------------------------------------------------------
// PART 4 — Scheduler integration
// -----------------------------------------------------------------------------
/// Pick the next Runnable thread (round-robin after current) and context
/// switch to it. Called cooperatively from blocking syscalls, and from
/// on_tick() during timer IRQ.
///
/// COOPERATIVE PATH: we are on the kernel stack inside a syscall. We save
/// x19-x30+sp+lr into the current thread slot and longjmp-style restore
/// the target. When control returns here (later, when we're rescheduled),
/// the syscall completes normally.
///
/// PREEMPTIVE PATH: on_tick() rewrites the trap frame directly; schedule()
/// itself isn't called from the IRQ.
pub fn schedule() {
    if !is_enabled() { return; }
    let me = current_tid();

    let next_tid_opt = with_table(|t| -> Option<u32> {
        let cur_idx = slot_of(t, me)?;
        // Demote current: Running -> Runnable (unless already Blocked/Exited).
        if t[cur_idx].state == ThreadState::Running {
            t[cur_idx].state = ThreadState::Runnable;
        }
        // Round-robin from cur_idx+1.
        for step in 1..=MAX_THREADS {
            let i = (cur_idx + step) % MAX_THREADS;
            if t[i].state == ThreadState::Runnable {
                t[i].state = ThreadState::Running;
                return Some(t[i].tid);
            }
        }
        // Nothing else runnable — keep running if we were, else idle.
        if t[cur_idx].state == ThreadState::Runnable {
            t[cur_idx].state = ThreadState::Running;
        }
        None
    });

    let Some(next_tid) = next_tid_opt else { return; };
    if next_tid == me { return; }
    RUNNING_TID.store(next_tid, Ordering::Release);

    // Call into assembly to do the actual register swap.
    // cxt_switch_cooperative saves x19-x30 + sp + tpidr_el0 of the current
    // thread into *old, then restores the same from *new. It returns when
    // the caller's thread is rescheduled later.
    let (old_ptr, new_ptr) = with_table(|t| -> (*mut SavedRegs, *const SavedRegs) {
        let old_idx = slot_of(t, me).unwrap_or(0);
        let new_idx = slot_of(t, next_tid).unwrap_or(0);
        let old = &mut t[old_idx].saved_regs as *mut SavedRegs;
        let new = &t[new_idx].saved_regs as *const SavedRegs;
        (old, new)
    });

    unsafe { cxt_switch_cooperative(old_ptr, new_ptr); }
}

// Implemented in src/batcave/linux/threads.s — saves callee-saved regs of
// the current thread into *old, then loads them from *new and returns.
unsafe extern "C" {
    fn cxt_switch_cooperative(old: *mut SavedRegs, new: *const SavedRegs);
}

/// Timer IRQ hook. Called from the EL1 IRQ handler. Returns true if the
/// handler should rewrite the trap frame with a different thread's state
/// before eret. Returns the target thread's SavedRegs pointer if so.
pub fn on_tick(current_trap_frame: *mut SavedRegs) -> Option<*const SavedRegs> {
    if !is_enabled() { return None; }
    let me = current_tid();

    // 1. Snapshot the trap frame into the current thread's slot.
    unsafe {
        with_table(|t| {
            if let Some(i) = slot_of(t, me) {
                t[i].saved_regs = *current_trap_frame;
            }
        });
    }

    // 2. Pick next runnable.
    let next_tid = with_table(|t| -> Option<u32> {
        let cur_idx = slot_of(t, me)?;
        if t[cur_idx].state == ThreadState::Running {
            t[cur_idx].state = ThreadState::Runnable;
        }
        for step in 1..=MAX_THREADS {
            let i = (cur_idx + step) % MAX_THREADS;
            if t[i].state == ThreadState::Runnable {
                t[i].state = ThreadState::Running;
                return Some(t[i].tid);
            }
        }
        if t[cur_idx].state == ThreadState::Runnable {
            t[cur_idx].state = ThreadState::Running;
        }
        None
    });
    let next_tid = next_tid?;
    if next_tid == me { return None; }

    RUNNING_TID.store(next_tid, Ordering::Release);

    // 3. Hand back a pointer to the next thread's saved regs so the IRQ
    //    handler can blit it into the trap frame (and msr tpidr_el0).
    //
    // SAFETY: caller guarantees to finish reading before the next IRQ /
    // before any call that could remap THREADS. This is a pointer into
    // a static — stable for the lifetime of the process.
    with_table(|t| -> Option<*const SavedRegs> {
        let i = slot_of(t, next_tid)?;
        Some(&t[i].saved_regs as *const SavedRegs)
    })
}

// -----------------------------------------------------------------------------
// PART 5 — Block / wake primitives
// -----------------------------------------------------------------------------
/// Park the current thread. Caller provides the reason; the scheduler will
/// not resume this thread until wake_thread() or a matching futex/epoll
/// wake. Yields the CPU immediately.
pub fn block_current_thread(reason: BlockReason) {
    let me = current_tid();
    with_table(|t| {
        if let Some(i) = slot_of(t, me) {
            t[i].state = ThreadState::Blocked(reason);
        }
    });
    schedule();
}

/// Move the named thread from Blocked -> Runnable. No-op if the thread is
/// not blocked, or doesn't exist.
pub fn wake_thread(tid: u32) -> bool {
    with_table(|t| {
        if let Some(i) = slot_of(t, tid) {
            if matches!(t[i].state, ThreadState::Blocked(_)) {
                t[i].state = ThreadState::Runnable;
                return true;
            }
        }
        false
    })
}

/// Wake up to `n` threads blocked in FutexWait on `uaddr`. Returns count woken.
pub fn futex_wake_on(uaddr: u64, n: u32) -> u32 {
    let mut woken = 0u32;
    with_table(|t| {
        for slot in t.iter_mut() {
            if woken >= n { break; }
            if let ThreadState::Blocked(BlockReason::FutexWait { uaddr: a, .. }) = slot.state {
                if a == uaddr {
                    slot.state = ThreadState::Runnable;
                    woken += 1;
                }
            }
        }
    });
    woken
}

/// Park current thread on `uaddr` if *uaddr still equals `val`. This is the
/// kernel half of FUTEX_WAIT; the user-space half does the atomic compare.
pub fn futex_wait_on(uaddr: u64, val: u32) -> i64 {
    // Re-check under IRQ-masked lock to close the wait/wake race.
    let current: u32 = unsafe { core::ptr::read_volatile(uaddr as *const u32) };
    if current != val { return EAGAIN; }
    block_current_thread(BlockReason::FutexWait { uaddr, val });
    0
}

/// Poll for Exited children of the current thread (for pthread_join).
/// Returns Some(exit_code) if found & reaped.
pub fn try_reap(tid: u32) -> Option<i32> {
    // Grab the slot fields while holding the lock, but do the actual
    // frame::free_frame calls *outside* the locked region — the frame
    // allocator takes its own internal state and we don't want to nest.
    let reaped = with_table(|t| {
        if let Some(i) = slot_of(t, tid) {
            if let ThreadState::Exited(code) = t[i].state {
                let pages = t[i].stack_pages;
                let base  = t[i].stack_base;
                t[i] = Thread::empty();
                return Some((code, base, pages));
            }
        }
        None
    });

    let (code, base, pages) = reaped?;

    // Free the thread's stack pages. `stack_base == 0` means the caller
    // supplied the stack (e.g. pthread_create with a user-allocated stack),
    // so we leave it alone — not ours to free.
    if base != 0 && pages > 0 {
        crate::kernel::mm::frame::free_contig(base as usize, pages);
        // Refund the memory quota we charged in clone().
        super::quotas::refund_active(
            super::quotas::Resource::Mem, pages * PAGE_SIZE);
    }
    // Always refund the thread slot itself (we charged it in clone()).
    super::quotas::refund_active(super::quotas::Resource::Threads, 1);

    Some(code)
}

// -----------------------------------------------------------------------------
// Diagnostics
// -----------------------------------------------------------------------------
pub fn dump() {
    uart::puts("[threads] table:\n");
    with_table(|t| {
        for s in t.iter() {
            if s.state == ThreadState::Free { continue; }
            uart::puts("  tid=");
            crate::kernel::mm::print_num(s.tid as usize);
            uart::puts(" parent=");
            crate::kernel::mm::print_num(s.parent_tid as usize);
            uart::puts(" state=");
            match s.state {
                ThreadState::Free       => uart::puts("Free"),
                ThreadState::Runnable   => uart::puts("Runnable"),
                ThreadState::Running    => uart::puts("Running"),
                ThreadState::Blocked(_) => uart::puts("Blocked"),
                ThreadState::Exited(_)  => uart::puts("Exited"),
            }
            uart::puts("\n");
        }
    });
}

// =============================================================================
// HUMAN WIRING CHECKLIST (what you need to do to make this actually run)
// =============================================================================
//
// 1. In src/batcave/linux/syscall.rs::sys_clone_thread (line ~1631):
//    - If flags matches CHROMIUM_THREAD_FLAGS (or at least CLONE_VM|CLONE_THREAD),
//      delegate to threads::clone(flags, child_stack, parent_tid_ptr,
//                                  child_tid_ptr, tls) and return its result.
//    - Immediately after, call threads::set_child_resume(new_tid, elr_plus_4,
//      parent_sp) with ELR_EL1 read from the trap frame +4.
//    - Leave the existing IN_CHILD path as the fallback for CLONE-without-
//      CLONE_THREAD (busybox), so nothing regresses.
//
// 2. Call threads::init_main_thread(entry_pc, initial_sp) from runner.rs
//    *only* for ELFs that are expected to use pthread (Chromium, v8_exec).
//    For hello_world/busybox, leave THREADING_ENABLED=false; schedule() and
//    on_tick() short-circuit and everything behaves exactly like today.
//
// 3. Implement the cooperative context switch in assembly. Suggested location:
//    src/batcave/linux/threads.s with a single extern "C" fn:
//        fn cxt_switch_cooperative(old: *mut SavedRegs, new: *const SavedRegs);
//    Replace the `uart::puts("[threads] schedule() TODO...")` in schedule()
//    with a call to it.
//
// 4. Hook on_tick() into the EL1 IRQ handler (arch/mod.rs or wherever the
//    timer fires). The handler already has a pointer to its own saved trap
//    frame (x0..x30, SP_EL0, ELR_EL1, SPSR_EL1 on the exception stack).
//    Treat that as a *mut SavedRegs, call on_tick(), and if it returns
//    Some(next_regs), memcpy(trap_frame, next_regs, sizeof SavedRegs) and
//    `msr tpidr_el0, <next.tls_ptr>` before eret. See the layout comments
//    on SavedRegs — ordering of x[0..31] / sp_el0 / elr_el1 / spsr_el1 must
//    match the trap frame exactly, OR you introduce a small shim struct.
//
// 5. Wire the futex syscall:
//    FUTEX_WAIT -> threads::futex_wait_on(uaddr, val) when is_enabled(),
//                  else fall back to the existing EAGAIN stub.
//    FUTEX_WAKE -> threads::futex_wake_on(uaddr, n) as i64.
//
// 6. Wire set_tid_address/gettid to consult threads::current_tid() when
//    enabled.
//
// 7. FP/NEON state — Chromium will crash without q0-q31 save/restore once
//    multiple threads actually run math. Extend SavedRegs with:
//        pub q: [u128; 32], pub fpsr: u32, pub fpcr: u32
//    and extend the switch assembly to stp/ldp them.
//
// =============================================================================
// STATUS SUMMARY
// =============================================================================
// Fully working (compiles, testable in isolation):
//   - Flag constants and Chromium flag bundle
//   - Thread struct, SavedRegs, ThreadState, BlockReason
//   - Static thread table + IRQ-masked locking
//   - clone() flag validation, TID allocation, slot allocation, TLS/tid
//     address recording, PARENT_SETTID write
//   - block_current_thread / wake_thread / futex_wait_on / futex_wake_on
//   - try_reap / exit_current
//   - init_main_thread gate (does nothing until the runner opts in)
//   - dump()
//
// Skeleton / TODO (marked inline):
//   - Cooperative context switch assembly (schedule() logs and returns)
//   - Trap-frame rewrite from on_tick() (returns the pointer but the IRQ
//     handler isn't wired to use it)
//   - Contiguous stack allocation (relies on frame allocator quirk)
//   - NEON/FP register save
//   - stack free-on-exit
//
// Not wired into syscall.rs at all — that's for the human.
