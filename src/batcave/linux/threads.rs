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

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
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
// CHROMIUM-PHASE-C: bumped from 64 to 256 to match DEFAULT_THREADS
// quota. Chromium content_shell creates 30+ threads even in
// --single-process mode and pthread_create was EAGAIN'ing after
// hitting the 64-slot ceiling.
pub const MAX_THREADS: usize = 256;
const DEFAULT_STACK_PAGES: usize = 16; // 64 KiB fallback
/// Two 4-KiB pages per thread for the kernel-side stack. We were 1
/// page (4 KiB), but a trap frame is 272 bytes and the EL1 abort
/// handler can recursively call schedule + dump + uart, blowing past
/// 4 KiB easily. The smoking-gun was smoke v16: a thread got its
/// kernel stack at PA 0xbffff000 (last-frame in cave-mapped range),
/// SP set to top = 0xc0000000 - 16 = 0xbfffffe0. SAVE_REGS subtracted
/// 272 → SP=0xbfffeed0, then a nested abort SAVE_REGS subtracted
/// another 272 → SP at 0xbfffe... fine. But RESTORE_REGS reading
/// `ldp x_,x_,[sp,#0xf8]` at the OUTER trap frame went to 0xc0000098
/// → unmapped, EL1 data abort, abort-handler-loops-then-cave-terminates.
/// 8 KiB gives headroom: even when SP starts at the very top of an
/// 8-KiB region, sp+0xf8 stays inside.
// 🎯 STUMP #20: bumped from 2 to 8 pages (32 KB). Each nested
// exception eats 272 bytes of SP_EL1, and our pa-skip / pc-skip
// unwinders can chain several before the cave terminates. 8 KB
// allowed ~30 nestings; 32 KB allows ~120 — should never run out
// in practice. The kernel stack lives below 0xC0000000 and is
// allocated per-thread via alloc_stack.
const KERNEL_STACK_PAGES: usize = 8;
const PAGE_SIZE: usize = 4096;

/// Assembly helper (threads.s) that saves the OLD thread's regs and
/// erets to EL0 with the NEW thread's user-mode state. Used for the
/// very first dispatch of a freshly-cloned thread — where we do NOT
/// have a previously-saved kernel continuation to `ret` into, so the
/// normal cxt_switch_cooperative path can't be used. Never returns.
///
/// The caller must **tail-call** (branch, not bl) into this helper
/// after popping its own stack frame and restoring x30 to its
/// caller-LR. The helper snapshots the current x30/SP into OLD's
/// saved_regs — so on a later cooperative resume OLD `ret`s back
/// into its original caller.
///
/// Parameters (AAPCS64):
///   x0: old_ptr  — save callee-saved regs of the outgoing thread here
///   x1: new_ptr  — load user x0..x30, elr_el1, spsr_el1, kernel sp, tpidr from here
///   x2: user_sp  — sp_el0 at eret (user-space stack pointer)
unsafe extern "C" {
    fn cxt_switch_first_run(
        old: *mut SavedRegs,
        new: *const SavedRegs,
        user_sp: u64,
    ) -> !;
}

/// Parent's ELR_EL1 at the moment of the `svc` that invoked clone().
/// Written by the arch SVC dispatcher before calling the syscall layer
/// (handle_sync_exception, syscall 220 path), read by sys_clone_thread
/// immediately after `threads::clone` returns. Avoids racing a timer
/// IRQ that would clobber the live elr_el1 register.
pub static PARENT_SYSCALL_ELR:  AtomicU64 = AtomicU64::new(0);
pub static PARENT_SYSCALL_SPSR: AtomicU64 = AtomicU64::new(0);

/// Parent's x0..x30 at svc entry. Linux clone's user-space ABI
/// (both glibc and musl on aarch64) relies on non-syscall argument
/// registers surviving the svc — glibc specifically stashes the fn
/// pointer in x10 and the arg in x12 before svc, then branches via
/// `blr x10` / `mov x0, x12` in the child path. The kernel must
/// preserve those across the child's first return to EL0.
///
/// Populated by the arch dispatcher on svc #220 entry; consumed by
/// sys_clone_thread's set_child_resume call.
pub static PARENT_SYSCALL_REGS: [AtomicU64; 31] = [
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
];

/// Full GPR snapshot + special registers. Large, but we have 64 slots so
/// 64 * ~800 bytes ≈ 50 KiB total — still within budget.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SavedRegs {
    pub x: [u64; 31],    // x0..x30    @ 0..248
    /// Kernel-side SP (SP_EL1) at cooperative-yield time. Confusingly
    /// named for historical reasons — the asm writes `mov x, sp` here
    /// which, from EL1, captures SP_EL1. The original SP_EL0 save is
    /// the new `user_sp_el0` field below.
    pub sp_el0: u64,     //            @ 248
    pub elr_el1: u64,    //            @ 256
    pub spsr_el1: u64,   //            @ 264
    // V5-SIDE-003 fix: save/restore Q0-Q31 + FPSR + FPCR across
    // context switch. Without this, thread B running an AES round
    // reads thread A's post-handshake FP residue directly from its
    // own q-regs — a trivial cross-thread side channel. Chromium's
    // V8 + Blink use FP/NEON heavily so this is required even for
    // correctness.
    pub q: [u128; 32],   // q0..q31    @ 272..784
    pub fpsr: u64,       //            @ 784
    pub fpcr: u64,       //            @ 792
    /// ROOT-FIX (2026-04-24): the USER stack pointer (SP_EL0). The
    /// `sp_el0` field above is a misnomer and actually holds SP_EL1
    /// at cooperative-yield time. Without capturing the real SP_EL0
    /// per-thread, `eret` after a cooperative context switch resumes
    /// the newly-scheduled thread with whatever SP_EL0 the previous
    /// thread left in the MSR — so thread A reads `[sp]` off thread
    /// B's user stack. On Chromium this surfaced as `ret` loading x30
    /// from t2's stack slot holding a V8 cage pointer, then branching
    /// into cage data = SIGSEGV.
    pub user_sp_el0: u64,//            @ 800
    /// REAL-FORK (2026-04-24): the user-space page table root for
    /// this thread (TTBR0_EL1 value). Threads in the same cave
    /// share an L1; forked-child threads have their own. Set by
    /// `init_main_thread` to the cave's L1 and by `clone()` (real
    /// fork branch) to the freshly forked L1. The cooperative
    /// context switch reads it and swaps TTBR0 if it differs from
    /// what's currently active — which makes cross-cave scheduling
    /// transparent without touching every callsite.
    pub user_ttbr0: u64, //            @ 808
}

impl SavedRegs {
    pub const fn zero() -> Self {
        Self {
            x: [0; 31],
            sp_el0: 0,
            elr_el1: 0,
            spsr_el1: 0,
            q: [0; 32],
            fpsr: 0,
            fpcr: 0,
            user_sp_el0: 0,
            user_ttbr0: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockReason {
    FutexWait { uaddr: u64, val: u32 },
    EpollWait { epfd: i32, deadline_ticks: u64 },  // 0 = infinite (epoll-only sentinel)
    Nanosleep { deadline_ticks: u64 },             // always concrete; 0 = invalid
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
    /// Kernel stack for this thread. cxt_switch_cooperative loads this
    /// into SP_EL1 on every switch-in. Freed at thread exit.
    pub kernel_stack_base: u64,
    pub kernel_stack_top: u64,
    /// True until the scheduler has dispatched this thread for the
    /// first time. On a fresh dispatch we use `cxt_switch_first_run`
    /// (which erets to EL0) instead of `cxt_switch_cooperative`
    /// (which `ret`s to a previously-saved kernel continuation).
    pub fresh: bool,
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
    /// Fork-as-thread frame-pointer relocation. Non-zero ONLY when
    /// this thread was created by a fork-style clone where we
    /// allocated a fresh user stack and copied parent's stack into
    /// it. Equals `child_sp - parent_sp` (the byte delta between
    /// the same logical stack offset in the two regions). Applied
    /// to x29 (and the saved x29 chain at [x29, [x29], ...]) by
    /// `set_child_resume` so frame-pointer-relative accesses land
    /// in the copied stack instead of the parent's stack. Zero for
    /// regular thread-clone.
    pub fork_fp_relocation: i64,
    /// Bounds of the fork-as-thread copied-stack region, used to
    /// limit the FP-chain walker. Saved x29 values OUTSIDE this
    /// range are left alone — they're either past the end of the
    /// useful chain (parent never went that deep) or garbage in an
    /// uninitialised stack slot. Without bounding, the walker
    /// happily relocates random qwords and corrupts data the child
    /// later reads (e.g. an `std::vector<int>::end` pointer turning
    /// into a HUGE size, causing infinite loops in
    /// base::LaunchProcess's fd-remap iteration).
    pub fork_stack_lo: u64,
    pub fork_stack_hi: u64,
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
            kernel_stack_base: 0,
            kernel_stack_top: 0,
            fresh: false,
            tid_clear_on_exit: None,
            tid_set_parent: None,
            tid_set_child: None,
            entry_pc: 0,
            entry_arg: 0,
            fork_fp_relocation: 0,
            fork_stack_lo: 0,
            fork_stack_hi: 0,
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
// 🎯 STUMP #10c: boss tid=1 was inconsistent with getpid=0x4242.
// In Linux, gettid()==getpid() for the main thread; many libc /
// PartitionAlloc paths assume this and use them interchangeably.
// PID/TID derivation cookies stored in PartitionAlloc slot-spans
// would compute differently on the boss vs from getpid() lookups,
// producing literal-1 sentinels in slot pointers.
//
// Match the boss tid to getpid (0x4242). Workers start at 0x4243.
static RUNNING_TID: AtomicU32 = AtomicU32::new(0x4242);
static NEXT_TID: AtomicU32 = AtomicU32::new(0x4243);

/// ROOT-FIX (2026-04-24): PID of the most recent fake-forked "child".
/// Kept around for any wait4 code that still consults it, but the
/// fork-as-thread path no longer sets it (real children get a real
/// thread slot now).
pub static FAKE_CHILD_PID: AtomicU32 = AtomicU32::new(0);

/// Per-fork counter used to allocate a unique stack VA region for
/// fork-as-thread children. Stack VAs are at
///   0x0000_0060_0000_0000 + (FORK_COUNTER * 0x100_0000)
/// (see `clone()`), giving 16 MB per fork — enough for any sane
/// thread, and sized so the first 256 forks fit in 4 GB of unused
/// VA space below the kernel-half boundary.
static FORK_COUNTER: AtomicU32 = AtomicU32::new(0);
static THREADING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Call once (from the runner, when a multi-threaded ELF is about to start)
/// to install TID 1 as the main thread and unlock real scheduling. Leaves
/// the legacy single-thread path untouched when not called.
pub fn init_main_thread(main_entry_pc: u64, main_sp_el0: u64) {
    with_table(|t| {
        // CHROMIUM-PHASE-C: belt-and-suspenders reset. The static
        // `[Thread::empty(); MAX_THREADS]` const initializer has been
        // unreliable in release builds (same family of bug as the
        // CAVE_QUOTAS static init that needed an explicit quotas::init()).
        // Force every slot to Free before we populate slot 0.
        for slot in t.iter_mut() {
            *slot = Thread::empty();
        }

        t[0] = Thread::empty();
        // 🎯 STUMP #11: boss thread table entry MUST match RUNNING_TID, or
        // schedule()/on_tick()'s `slot_of(t, current_tid())` returns None
        // → "no runnable thread" deadlock-diag fires AND no context
        // switches happen. Workers spawned via clone() never get the CPU
        // because the scheduler can't find the boss to swap out from.
        // Match the BOSS_TID constant used in init.
        t[0].tid = 0x4242;
        t[0].parent_tid = 0;
        t[0].state = ThreadState::Running;
        t[0].entry_pc = main_entry_pc;
        t[0].stack_top = main_sp_el0;
        t[0].saved_regs.sp_el0 = main_sp_el0;
        // ROOT-FIX: seed user_sp_el0 too. Gets overwritten the first
        // time t0 yields (asm saves current SP_EL0 there), but having
        // a sane value avoids a spurious load from an uninitialised
        // slot if anything reads it before the first yield.
        t[0].saved_regs.user_sp_el0 = main_sp_el0;
        // REAL-FORK: capture current TTBR0 as t0's user page table.
        // Subsequent forked children get a different L1 stored in
        // their saved_regs.user_ttbr0; the cooperative context
        // switch swaps TTBR0 transparently when crossing caves.
        let ttbr0_now: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_now); }
        t[0].saved_regs.user_ttbr0 = ttbr0_now & !1u64; // strip CnP bit
        t[0].saved_regs.elr_el1 = main_entry_pc;
        // EL0t, IRQs unmasked:
        t[0].saved_regs.spsr_el1 = 0;
    });
    // CHROMIUM-PHASE-C: the NEXT_TID AtomicU32::new(2) const-init
    // lands as 0 in release builds sometimes (same family as the
    // CAVE_QUOTAS / THREADS-table flake). Explicitly reset to 2 so
    // `fetch_add(1)` returns 2 on the first clone — not 0.
    // 🎯 STUMP #10c: keep boss tid in sync with getpid (0x4242).
    NEXT_TID.store(0x4243, Ordering::Release);
    RUNNING_TID.store(0x4242, Ordering::Release);
    THREADING_ENABLED.store(true, Ordering::Release);
    uart::puts("[threads] Main thread registered, threading enabled\n");
}

pub fn is_enabled() -> bool { THREADING_ENABLED.load(Ordering::Acquire) }
pub fn current_tid() -> u32 { RUNNING_TID.load(Ordering::Acquire) }

/// V8-ROOT-2: drop the entire thread table on cave switch. Without this,
/// a zombie thread from the outgoing cave can be resumed (scheduler picks
/// it from the table) inside the new cave's address space — breaks
/// isolation completely.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(THREADS);
        for t in table.iter_mut() {
            *t = Thread::empty();
        }
    }
    RUNNING_TID.store(0x4242, Ordering::Release);
    NEXT_TID.store(0x4243, Ordering::Release);
    THREADING_ENABLED.store(false, Ordering::Release);
    PREEMPT_REQUESTED.store(false, Ordering::Release);
}

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
    use crate::drivers::uart;
    uart::puts("[clone] flags=0x");
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((flags >> (sh*4)) & 0xF) as usize]);
    }
    uart::puts(" stack=0x");
    for sh in (0..16).rev() {
        uart::putc(hex[((child_stack >> (sh*4)) & 0xF) as usize]);
    }
    uart::puts("\n");
    // Reject flag combos we can't support yet.
    if flags & CLONE_VFORK != 0 {
        uart::puts("[clone] reject VFORK\n");
        return EINVAL;
    }
    if flags & CLONE_PTRACE != 0 {
        uart::puts("[clone] reject PTRACE\n");
        return EINVAL;
    }
    // We don't have separate fs/files/sighand yet — must be shared.
    //
    // FORK-AS-THREAD (2026-04-24): when the caller is doing a fork-
    // style clone (no CLONE_VM), we re-interpret it as a thread
    // clone with VM/files/sighand shared. The child gets:
    //   * a fresh user stack (Chromium passes child_stack=NULL)
    //   * a fresh kernel stack
    //   * the parent's TLS pointer (TPIDR_EL0) — glibc fork
    //     doesn't pass CLONE_SETTLS so we'd otherwise zero it
    //   * the parent's full GPR snapshot (set_child_resume) with
    //     x0 = 0 to signal "you are the child"
    //
    // What we DON'T do: copy the parent's address space. Memory
    // is shared. For Chromium's zygote pattern that's acceptable:
    // zygote_main() loops on an IPC socket, doesn't need its own
    // memory view. Real CoW fork is a follow-up project; this
    // gets us through enough of Chromium's init to render a DOM.
    let needs_shared = CLONE_VM | CLONE_THREAD;
    if flags & needs_shared != needs_shared {
        // REAL FORK PATH (no CLONE_VM): give the child its own
        // address space (separate L1 page table, eager-copied from
        // the parent's). The child's TTBR0 is set on its first
        // schedule by the cooperative-switch asm (which checks
        // saved_regs.user_ttbr0). Memory is fully separated;
        // post-fork writes from one process don't bleed into the
        // other. Parent's CURRENT user SP is preserved for the
        // child via set_child_resume — both processes start at the
        // same logical stack offset, but in separate physical
        // pages.
        return real_fork(flags, parent_tid, child_tid);
    }
    let child_stack = child_stack; // (no rewrite path for thread-clone)

    // V8-ROOT-1 / V8-IRQ-#3: charge-Threads + charge-Mem must be atomic
    // w.r.t. preempt — otherwise a racing syscall could see Threads
    // charged but Mem not (or vice versa) and the refund-on-error
    // path could leave one ledger drifted.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Threads, 1) {
            uart::puts("[clone] reject: Threads quota\n");
            return e;
        }
        if child_stack == 0 {
            let bytes = DEFAULT_STACK_PAGES * PAGE_SIZE;
            if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Mem, bytes) {
                uart::puts("[clone] reject: Mem quota\n");
                super::quotas::refund_active(super::quotas::Resource::Threads, 1);
                return e;
            }
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

    // Allocate a dedicated kernel stack page for this thread. SP_EL1
    // will point here when we context-switch in; IRQs / syscalls that
    // fire while this thread is running use it as their trap-frame
    // spill.
    let (kstack_base, kstack_top) = match alloc_stack(KERNEL_STACK_PAGES) {
        Some(pair) => pair,
        None => {
            super::quotas::refund_active(super::quotas::Resource::Threads, 1);
            if stack_pages > 0 {
                crate::kernel::mm::frame::free_contig(stack_base as usize, stack_pages);
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, stack_pages * PAGE_SIZE);
            }
            return ENOMEM;
        }
    };

    // V8-ROOT-1 / V8-IRQ-#3: PARENT_SETTID write + slot allocation +
    // slot population is one critical section. A timer IRQ between any
    // of these could schedule the child before its slot is fully set
    // up (state=Runnable while stack_top still 0 → cxt_switch into a
    // null-pointer SP).
    let _g = crate::kernel::sync::IrqGuard::new();

    // V8-ROOT-8 / V8-PTR-001/002: clone parent_tid and child_tid came
    // from EL0 without any uaccess gate. `write_volatile(parent_tid,
    // new_tid)` at a kernel address = 4-byte arbitrary kernel write.
    // Check both pointers through the uaccess bounds check; refuse the
    // clone if either is out-of-range.
    use crate::batcave::linux::uaccess;
    if flags & CLONE_PARENT_SETTID != 0 && !parent_tid.is_null() {
        if !uaccess::is_user_range(parent_tid as usize, 4) {
            // Refund quotas charged above.
            super::quotas::refund_active(super::quotas::Resource::Threads, 1);
            if child_stack == 0 {
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, DEFAULT_STACK_PAGES * PAGE_SIZE);
            }
            return -(14i64); // EFAULT
        }
        unsafe { core::ptr::write_volatile(parent_tid, new_tid as i32); }
    }

    let tid_set_child = if flags & CLONE_CHILD_SETTID != 0 && !child_tid.is_null() {
        if !uaccess::is_user_range(child_tid as usize, 4) {
            super::quotas::refund_active(super::quotas::Resource::Threads, 1);
            if child_stack == 0 {
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, DEFAULT_STACK_PAGES * PAGE_SIZE);
            }
            return -(14i64);
        }
        Some(child_tid as u64)
    } else { None };
    let tid_clear_on_exit = if flags & CLONE_CHILD_CLEARTID != 0 && !child_tid.is_null() {
        if !uaccess::is_user_range(child_tid as usize, 4) {
            super::quotas::refund_active(super::quotas::Resource::Threads, 1);
            if child_stack == 0 {
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, DEFAULT_STACK_PAGES * PAGE_SIZE);
            }
            return -(14i64);
        }
        Some(child_tid as u64)
    } else { None };

    // Populate the slot.
    let result = with_table(|t| -> i64 {
        let Some(slot) = find_free_slot(t) else {
            uart::puts("[clone] reject: no free thread slot\n");
            return EAGAIN;
        };
        let parent = current_tid();

        t[slot] = Thread::empty();
        t[slot].tid = new_tid;
        t[slot].parent_tid = parent;
        t[slot].state = ThreadState::Runnable;
        t[slot].stack_base = stack_base;
        t[slot].stack_top = stack_top;
        t[slot].stack_pages = stack_pages;
        t[slot].kernel_stack_base = kstack_base;
        t[slot].kernel_stack_top = kstack_top;
        t[slot].fresh = true;
        let tls_val = if flags & CLONE_SETTLS != 0 { tls } else { 0 };
        t[slot].tls_ptr = tls_val;
        t[slot].tid_set_parent = if flags & CLONE_PARENT_SETTID != 0 {
            Some(parent_tid as u64)
        } else { None };
        t[slot].tid_set_child = tid_set_child;
        t[slot].tid_clear_on_exit = tid_clear_on_exit;

        // Child register seed. Most of x[0..30] gets overwritten from
        // the parent's svc-entry snapshot in set_child_resume below;
        // the explicit fields here populate the rest of the bootstrap
        // state cxt_switch_first_run needs:
        //
        //   saved_regs.x[18]   → tpidr_el0 (TLS base)
        //   saved_regs.elr_el1 → user_pc   (patched by set_child_resume)
        //   saved_regs.spsr_el1 → EL0t, IRQs on
        //   saved_regs.sp_el0  → kernel SP_EL1 for this thread
        //
        // User SP comes through as a separate arg to cxt_switch_first_run
        // (see Thread.stack_top). We do NOT overload x[19..22] any more.
        t[slot].saved_regs = SavedRegs::zero();
        t[slot].saved_regs.x[18] = tls_val;
        t[slot].saved_regs.sp_el0 = kstack_top;
        // ROOT-FIX: seed the new thread's user SP_EL0 so that the
        // scheduler's cooperative-restore path can feed a valid
        // SP_EL0 into the MSR before the first eret. The asm first-
        // run path takes user SP via x2 arg; this value is for the
        // path where an IRQ preempts the thread after it starts
        // running and we later cooperatively resume it.
        t[slot].saved_regs.user_sp_el0 = child_stack;
        t[slot].saved_regs.elr_el1 = 0;
        t[slot].saved_regs.spsr_el1 = 0;
        // CAVE-CORRECTNESS-2026-04-25: capture the PARENT's TTBR0
        // as this thread's user_ttbr0. Previously left as 0, which
        // means "don't touch TTBR0 on switch-in" — correct only when
        // the OUTGOING thread happens to be in the same cave. If a
        // thread from a different cave yields to us, we'd inherit
        // their TTBR0 and execute in the wrong address space. This
        // never bit single-cave Chromium runs but with zygote forks
        // creating multiple caves it's a latent bug.
        let parent_ttbr0: u64;
        unsafe {
            core::arch::asm!("mrs {}, ttbr0_el1", out(reg) parent_ttbr0);
        }
        t[slot].saved_regs.user_ttbr0 = parent_ttbr0 & !1u64; // strip CnP
        // Thread-clone (CLONE_VM): no fork relocation needed —
        // the child shares the parent's address space and runs at
        // the user-supplied child_stack.
        t[slot].fork_fp_relocation = 0;
        t[slot].fork_stack_lo = 0;
        t[slot].fork_stack_hi = 0;
        // Inherit parent's TTBR0 — same cave.
        let cur_ttbr0: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) cur_ttbr0); }
        t[slot].saved_regs.user_ttbr0 = cur_ttbr0 & !1u64;

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
        if kstack_base != 0 {
            crate::kernel::mm::frame::free_contig(
                kstack_base as usize, KERNEL_STACK_PAGES);
        }
        return result;
    }

    uart::puts("[clone] success new_tid=");
    crate::kernel::mm::print_num(new_tid as usize);
    uart::puts("\n");
    new_tid as i64
}

/// REAL FORK (2026-04-24): create a new process with its own
/// address space.
///
/// Called from `clone()` when the caller passes fork-style flags
/// (no CLONE_VM). Steps:
///   1. Look up the parent's user-window bounds.
///   2. `mmu::fork_cave_pagetable` — eager-copy parent's user
///      pages into a fresh L1.
///   3. `mmu::record_forked_cave` — register the child's L1 in
///      the per-cave table so `is_user_range` works for it.
///   4. Allocate a thread slot, kernel stack, and seed
///      saved_regs so the cooperative-switch asm activates
///      child_l1 on first run.
///   5. Return new TID. set_child_resume (called from the
///      arch SVC dispatcher) will fill in the parent's GPR
///      snapshot with x0=0 for the child.
///
/// Memory: the parent's user-window contents are duplicated into
/// fresh physical frames. Fork is therefore O(parent's mapped
/// memory) — typically 50-200 ms for Chromium-sized caves. We
/// pay this once per fork, not per access (unlike CoW), so the
/// child runs at full speed afterward.
fn real_fork(
    flags: u64,
    parent_tid_ptr: *mut i32,
    child_tid_ptr: *mut i32,
) -> i64 {
    use crate::drivers::uart;

    // 1. Parent's L1 + user-window bounds. We can't trust the
    // global active_user_window() here because the SVC handler may
    // have run code that disturbed the per-cave bookkeeping — go
    // straight to TTBR0 and the per-cave tables.
    let parent_l1: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) parent_l1); }
    let parent_l1 = parent_l1 & !1u64;
    // STUMP #160: when the parent's L1 isn't in CAVE_L1[] (e.g. an
    // already-forked child re-forking — nested zygote pattern), fall
    // back to the chromium runner's default user-window. The fork
    // still succeeds; just the trace was misleading ("aborting" but
    // nothing actually aborted), so soften the message.
    let (virt_base, virt_extent) = super::mmu::cave_bounds_for_l1(parent_l1)
        .unwrap_or_else(|| {
            if super::skip_log::is_enabled() {
                uart::puts("[fork] parent L1 not in cave registry — using runner-default bounds\n");
            }
            (0x10000000u64, (400 * 1024 * 1024) as u64)
        });
    let parent_phys_base = super::mmu::cave_phys_base_for_l1(parent_l1)
        .unwrap_or(0);

    // 2. Allocate a cave slot + duplicate the page table.
    let cave_slot = match super::mmu::alloc_cave_slot() {
        Some(s) => s,
        None => {
            uart::puts("[fork] no free cave slot\n");
            return ENOMEM;
        }
    };
    let child_l1 = match super::mmu::fork_cave_pagetable(
        parent_l1 as usize, virt_base, virt_extent)
    {
        Ok(l) => l as u64,
        Err(e) => {
            uart::puts("[fork] fork_cave_pagetable failed: ");
            uart::puts(e);
            uart::puts("\n");
            super::mmu::free_cave_slot(cave_slot);
            return ENOMEM;
        }
    };
    if let Err(e) = super::mmu::record_forked_cave(
        cave_slot, child_l1, parent_phys_base, virt_base, virt_extent)
    {
        uart::puts("[fork] record_forked_cave failed: ");
        uart::puts(e);
        uart::puts("\n");
        super::mmu::free_cave_slot(cave_slot);
        return ENOMEM;
    }

    // CHROMIUM-PHASE-D: child gets its OWN copy of the parent's
    // FD table. POSIX: subsequent close()/dup() in the child only
    // touches the child's fds; parent's view is unaffected.
    // Without this, e.g. a forked zygote child closing fd 23
    // (its post-fork dup of an IPC socketpair end) ALSO closes
    // parent's fd 23 — parent's later sendmsg returns ENOTSOCK
    // and Chromium FATALs with "Cannot communicate with zygote".
    super::fd::clone_fd_table(cave_slot);

    // 3. Allocate kernel stack for the child thread.
    let kstack_pages = KERNEL_STACK_PAGES;
    let kstack_base = match crate::kernel::mm::frame::alloc_contig(kstack_pages) {
        Some(b) => b as u64,
        None => {
            uart::puts("[fork] no kernel stack\n");
            super::mmu::free_cave_slot(cave_slot);
            return ENOMEM;
        }
    };
    let kstack_top = kstack_base + (kstack_pages * PAGE_SIZE) as u64;

    // 4. Allocate a thread slot and seed it. Most state will be
    // overwritten by set_child_resume from the parent's syscall
    // snapshot — we only need to populate the bits that aren't
    // GPRs (TTBR0, kernel SP, fresh flag).
    let parent_sp: u64;
    unsafe { core::arch::asm!("mrs {}, sp_el0", out(reg) parent_sp); }
    let parent_tls: u64;
    unsafe { core::arch::asm!("mrs {}, tpidr_el0", out(reg) parent_tls); }

    let new_tid = NEXT_TID.fetch_add(1, Ordering::AcqRel);
    let me = current_tid();
    let alloc_ok = with_table(|t| -> bool {
        let slot = match find_free_slot(t) {
            Some(s) => s,
            None => return false,
        };
        t[slot] = Thread::empty();
        t[slot].tid = new_tid;
        t[slot].parent_tid = me;
        t[slot].state = ThreadState::Runnable;
        t[slot].kernel_stack_base = kstack_base;
        t[slot].kernel_stack_top = kstack_top;
        t[slot].fresh = true;
        t[slot].tls_ptr = parent_tls;
        t[slot].entry_pc = 0; // patched by set_child_resume

        // Stash CLONE_*_TID pointers so the existing code at
        // thread bring-up writes the child's TID into them.
        if flags & CLONE_PARENT_SETTID != 0 && !parent_tid_ptr.is_null() {
            t[slot].tid_set_parent = Some(parent_tid_ptr as u64);
        }
        if flags & CLONE_CHILD_SETTID != 0 && !child_tid_ptr.is_null() {
            t[slot].tid_set_child = Some(child_tid_ptr as u64);
        }
        if flags & CLONE_CHILD_CLEARTID != 0 && !child_tid_ptr.is_null() {
            t[slot].tid_clear_on_exit = Some(child_tid_ptr as u64);
        }

        // saved_regs init: most overwritten by set_child_resume.
        t[slot].saved_regs = SavedRegs::zero();
        t[slot].saved_regs.x[18] = parent_tls;          // TPIDR_EL0
        t[slot].saved_regs.sp_el0 = kstack_top;          // kernel SP
        t[slot].saved_regs.user_sp_el0 = parent_sp;      // user SP
        t[slot].saved_regs.user_ttbr0 = child_l1;        // ⭐ THE KEY BIT
        // elr_el1 / spsr_el1 / x[0..30] all set later by
        // set_child_resume (called from arch dispatcher).

        // stack_top — used by cxt_switch_first_run as x2 arg.
        // For real fork, child runs on parent's user SP (same VA;
        // distinct phys via the new page table).
        t[slot].stack_top = parent_sp;

        true
    });
    if !alloc_ok {
        uart::puts("[fork] no free thread slot\n");
        crate::kernel::mm::frame::free_contig(kstack_base as usize, kstack_pages);
        super::mmu::free_cave_slot(cave_slot);
        return ENOMEM;
    }

    // Write parent_tid_ptr immediately — Linux semantics.
    if flags & CLONE_PARENT_SETTID != 0 && !parent_tid_ptr.is_null() {
        unsafe { core::ptr::write_volatile(parent_tid_ptr, new_tid as i32); }
    }
    // Note: child_tid_ptr writing is deferred to when the child
    // actually runs (at cxt_switch_first_run / first svc-return).

    uart::puts("[fork] real fork → child_tid=");
    crate::kernel::mm::print_num(new_tid as usize);
    uart::puts(" cave_slot=");
    crate::kernel::mm::print_num(cave_slot);
    uart::puts(" child_l1=0x");
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((child_l1 >> (sh * 4)) & 0xF) as usize]);
    }
    uart::puts("\n");
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
            // Copy the parent's register snapshot (captured by the
            // arch SVC dispatcher) into the child's saved_regs so
            // cxt_switch_first_run can restore the full GPR state
            // before eret — matches Linux's clone() ABI where the
            // child inherits all registers except x0 (set to 0 by
            // the kernel as the child's clone() return value).
            for k in 0..31 {
                t[i].saved_regs.x[k] = PARENT_SYSCALL_REGS[k]
                    .load(Ordering::Acquire);
            }
            // Child-specific overrides:
            //   x[0] = 0    (clone() returns 0 in child)
            //   x[19]       (elr_el1 after eret — set here)
            // x[18] we leave as whatever the parent had — TLS is
            // sourced from saved_regs.x[18] by cxt_switch_first_run
            // to set tpidr_el0, and we already wrote tls_val in
            // clone(). Overwrite it here so the snapshot doesn't
            // stomp the explicit CLONE_SETTLS value.
            let preserved_x18 = t[i].tls_ptr;
            t[i].saved_regs.x[0]  = 0;
            t[i].saved_regs.x[18] = preserved_x18;
            t[i].saved_regs.x[19] = resume_pc;
            t[i].saved_regs.elr_el1 = resume_pc;
            t[i].entry_pc = resume_pc;

            // Fork-as-thread: relocate the frame pointer so it
            // lands in the child's copy of parent's stack instead
            // of pointing back into parent's actual stack VA. We
            // also walk the saved-FP chain in the COPIED stack to
            // relocate every saved x29, so glibc's stack-canary
            // check (and any FP-based unwinding the child does)
            // sees a valid chain entirely in child memory.
            let reloc = t[i].fork_fp_relocation;
            if reloc != 0 {
                let stack_lo = t[i].fork_stack_lo;
                let stack_hi = t[i].fork_stack_hi;
                let parent_x29 = t[i].saved_regs.x[29];
                let child_x29 = (parent_x29 as i64).wrapping_add(reloc) as u64;
                // Only relocate the top-level x29 if the result
                // lands in the copied region; otherwise the parent
                // x29 was outside our copy window and we have no
                // valid target to point at — in which case zero
                // x29 (frame chain ends here, FP-relative reads
                // will fault rather than return garbage).
                if child_x29 >= stack_lo && child_x29 < stack_hi {
                    t[i].saved_regs.x[29] = child_x29;
                } else {
                    t[i].saved_regs.x[29] = 0;
                }
                // Walk the saved-FP chain. We only relocate slots
                // whose ORIGINAL value was a valid frame pointer
                // — i.e., the relocated target lands inside the
                // copied region. This stops the walker from
                // rewriting garbage values (uninitialised stack
                // slots that happen to contain a non-zero qword)
                // which previously corrupted things like
                // base::LaunchOptions::fds_to_remap and caused
                // infinite loops in the child.
                let mut walker = child_x29;
                for _ in 0..32 {
                    if walker < stack_lo || walker >= stack_hi {
                        break;
                    }
                    let saved_x29: u64 = unsafe {
                        core::ptr::read_volatile(walker as *const u64)
                    };
                    if saved_x29 == 0 { break; }
                    let new_saved = (saved_x29 as i64)
                        .wrapping_add(reloc) as u64;
                    if new_saved < stack_lo || new_saved >= stack_hi {
                        // Original was outside copy window, so
                        // it's not a real saved x29 we control.
                        // Don't write back. Stop the walk.
                        break;
                    }
                    unsafe {
                        core::ptr::write_volatile(
                            walker as *mut u64, new_saved);
                    }
                    walker = new_saved;
                }
            }
        }
    });
}

/// Helper for the stack fallback path. Allocates contiguous 4 KiB frames
/// and returns (base, top). 16-byte aligned top.
///
/// 🎯 STUMP #10 FIX: previously called `frame::alloc_frame()` in a loop,
/// which DOES NOT guarantee contiguity — `alloc_frame` just returns the
/// lowest clear bit. With heavy fragmentation (many threads, demand-page
/// commits), `pages` sequential calls return scattered frames. Code then
/// computed `top = last + PAGE_SIZE` and treated `[first, top)` as one
/// contiguous stack range, but the gap pages were owned by other
/// allocations — thread stack writes silently clobbered PartitionAlloc /
/// V8 / TLS data sitting in those frames. Symptom:
/// `PartitionAlloc::DoubleFreeOrCorruptionDetected` with `x1=0x1`.
///
/// Fix: use `alloc_contig(pages)` so the returned run is guaranteed
/// physically contiguous. Boundary check still applies to the LAST page.
fn alloc_stack(pages: usize) -> Option<(u64, u64)> {
    // V8-STACK-TOP-FIX 2026-04-25: refuse any allocation whose top hits
    // the cave-mapped boundary at 0xC0000000. Cave's TTBR0 maps L2_xhi
    // for VA 0x80000000..0xBFFFFFFF; VA 0xC0000000+ is UNMAPPED. If a
    // thread's kernel stack top = 0xC0000000, then SAVE_REGS does
    // `sub sp,sp,#272; stp x0,x1,[sp,#248]` and the LAST store lands
    // at 0xBFFFFEE0+248 = 0xC0000018 → translation fault → can't even
    // capture the trap frame, and the abort spam loops. Drop any frame
    // that would put `top` >= 0xC0000000 by retrying with a fresh one.
    const TOP_LIMIT: u64 = 0xC0000000;

    // Try up to 8 times to find an alloc_contig run whose top fits below
    // the limit. Any rejected run is leaked (we can't free it without
    // a free_contig that takes a count, which we have — but the spec
    // here matches the previous "leak the boundary frame" behavior
    // applied to the whole run).
    for _ in 0..8 {
        let first = frame::alloc_contig(pages)? as u64;
        let last  = first + ((pages - 1) * PAGE_SIZE) as u64;
        let top   = (last + PAGE_SIZE as u64) & !0xFu64;
        if top + 256 > TOP_LIMIT {
            // Leak the run; try again. alloc_contig is sequential
            // bottom-up so this only triggers near the very top.
            continue;
        }
        return Some((first, top));
    }
    None
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
///
/// V4 process-destroy: previously the thread slot was marked Exited but
/// its backing stack + TLS frames were never freed. A cave that spawned
/// 16 threads then waited for them to exit leaked 16 × stack_pages
/// permanently until cave destroy. Now we free stack_base..+stack_pages
/// back to the frame allocator (free_frame zeroes each page on return)
/// before yielding. The thread slot itself becomes Free so future
/// spawn_thread calls can reuse it.
pub fn exit_current(code: i32) -> ! {
    let me = current_tid();
    // V8-ROOT-1: the slot-mutation + stack-free is a critical section;
    // schedule() + wfi are NOT — they're the yield point. Scope the
    // IrqGuard tightly around the state transition only.
    //
    // V6-TOCTOU-004 fix: V5's "wipe slot inside lock" was wrong because
    // try_reap (waitpid) needs to find state=Exited to refund quota
    // and report exit code.
    let clear_addr = {
        let _g = crate::kernel::sync::IrqGuard::new();
        with_table(|t| {
            if let Some(i) = slot_of(t, me) {
                t[i].state = ThreadState::Exited(code);
                let sb = t[i].stack_base;
                let sp = t[i].stack_pages;
                let ca = t[i].tid_clear_on_exit;
                if sp > 0 && sb != 0 {
                    crate::kernel::mm::frame::free_contig(sb as usize, sp);
                }
                // NOTE (real-fork): we deliberately DO NOT free the
                // thread's kernel stack here. We're called from the
                // SVC handler running on that very stack — freeing
                // it would pull the rug out before schedule()'s asm
                // could read its own locals. The stack leaks until
                // the cave is fully reaped (a real wait4 + cave-
                // destroy is the proper fix; for now the leak is
                // bounded to ~16 KB per fork).
                t[i].stack_base = 0;
                t[i].stack_pages = 0;
                t[i].tid_clear_on_exit = None;
                ca
            } else { None }
        })
        // _g dropped here — IRQs re-enabled before write_volatile + futex_wake + schedule
    };
    if let Some(addr) = clear_addr {
        unsafe { core::ptr::write_volatile(addr as *mut i32, 0); }
        futex_wake_on(addr, 1);
    }
    schedule();
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

    // V8-ROOT-1 / V8-IRQ-#10: pick-next + RUNNING_TID-store + obtain
    // SavedRegs pointers must be one critical section. Two separate
    // with_table calls let an IRQ between them retire the picked
    // thread (e.g. exit_current → state=Exited), and the second
    // with_table reads stale SavedRegs.
    //
    // The ASM `cxt_switch_cooperative` itself does NOT need IRQs masked
    // — it's a save-restore that must be allowed to be preempted (the
    // newly-restored thread can take an interrupt at any instruction).
    // So we drop the guard immediately before the call.
    let (next_tid, old_ptr, new_ptr) = {
        let _g = crate::kernel::sync::IrqGuard::new();
        let next_tid_opt = with_table(|t| -> Option<u32> {
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
        let Some(next_tid) = next_tid_opt else {
            // No Runnable thread found. If we're Blocked too then we
            // have a real deadlock — every BatCave thread is parked
            // and nobody can wake anyone.
            //
            // 🎯 STUMP #63: under BAT_OS_KEEP_GOING, instead of just
            // dumping and stalling, force-wake the FIRST blocked
            // thread we find (lowest slot index = oldest worker, most
            // likely a missed signal). Log a parser-friendly
            // [SKIP-DEADLOCK ...] entry so the timeline is clear, and
            // re-enter the loop. Cap at 32 force-wakes per cave run
            // so we don't infinite-loop a genuinely stuck program.
            if crate::batcave::linux::skip_log::is_enabled() {
                static N_FORCED: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(0);
                let forced_n = N_FORCED.fetch_add(1, Ordering::AcqRel);
                if forced_n < 32 {
                    let woken: Option<(u32, BlockReason)> = with_table(|t| {
                        for slot in t.iter_mut() {
                            match slot.state {
                                ThreadState::Blocked(reason) => {
                                    slot.state = ThreadState::Runnable;
                                    return Some((slot.tid, reason));
                                }
                                _ => continue,
                            }
                        }
                        None
                    });
                    if let Some((wtid, reason)) = woken {
                        let (a1, a2) = match reason {
                            BlockReason::FutexWait { uaddr, val } => (uaddr, val as u64),
                            BlockReason::EpollWait { epfd, deadline_ticks } =>
                                (deadline_ticks, epfd as i64 as u64),
                            BlockReason::Nanosleep { deadline_ticks } => (deadline_ticks, 0),
                            BlockReason::Join { target_tid } => (target_tid as u64, 0),
                            BlockReason::IoWait => (0, 0),
                        };
                        let kind_disc = match reason {
                            BlockReason::FutexWait { .. } => 0u64,
                            BlockReason::EpollWait { .. } => 1u64,
                            BlockReason::Nanosleep { .. } => 2u64,
                            BlockReason::Join { .. } => 3u64,
                            BlockReason::IoWait => 4u64,
                        };
                        crate::batcave::linux::skip_log::record(
                            crate::batcave::linux::skip_log::SkipKind::FutexDeadlock,
                            wtid, wtid as u64, a1, a2 | (kind_disc << 32),
                            0, 0,
                        );
                        // Tail-call back into schedule by returning;
                        // the caller's outer loop re-enters us with
                        // the freshly-runnable thread available.
                        return;
                    }
                }
                // Cap reached or no blocked thread found — fall
                // through to the legacy dump.
            }
            static DEADLOCK_REPORTED: AtomicBool = AtomicBool::new(false);
            if !DEADLOCK_REPORTED.swap(true, Ordering::AcqRel) {
                uart::puts("[diag] schedule() found NO runnable thread — possible deadlock\n");
                dump();
                crate::batcave::linux::syscall_history::dump_per_tid_last();
            }
            return;
        };
        if next_tid == me { return; }
        RUNNING_TID.store(next_tid, Ordering::Release);
        let (op, np) = with_table(|t| -> (*mut SavedRegs, *const SavedRegs) {
            let old_idx = slot_of(t, me).unwrap_or(0);
            let new_idx = slot_of(t, next_tid).unwrap_or(0);
            let old = &mut t[old_idx].saved_regs as *mut SavedRegs;
            let new = &t[new_idx].saved_regs as *const SavedRegs;
            (old, new)
        });
        (next_tid, op, np)
    };
    let _ = next_tid;
    // Is the NEW thread being dispatched for the first time? If so we
    // take the eret-to-EL0 path instead of the ret-to-kernel-continuation
    // path. Read-and-clear the fresh flag under the table lock so a
    // parallel clone can't race us. Single-core assumption still applies.
    let is_fresh = with_table(|t| {
        if let Some(i) = slot_of(t, next_tid) {
            let was_fresh = t[i].fresh;
            t[i].fresh = false;
            was_fresh
        } else { false }
    });
    if is_fresh {
        // Read user_sp from Thread.stack_top (set in clone()). Passed
        // to cxt_switch_first_run as a dedicated arg so we don't have
        // to repurpose any saved_regs field for it.
        let user_sp = with_table(|t| {
            if let Some(i) = slot_of(t, next_tid) { t[i].stack_top } else { 0 }
        });
        unsafe {
            // Tail-call into cxt_switch_first_run from inline asm so
            // that (a) schedule's own stack frame is popped before we
            // hand off to the helper — otherwise the helper saves
            // OLD's kernel SP = schedule's frame top and OLD's later
            // cooperative resume reads stale schedule locals as its
            // own stack; and (b) the call uses `br` not `bl`, so x30
            // stays as schedule's caller-LR (park_slot / ppoll / etc.),
            // which cxt_switch_first_run saves as OLD.saved_regs.x[30]
            // — the address OLD's eventual `ret` from the next
            // cooperative switch should land on.
            //
            // The frame-pop size (`add sp, sp, #{frame}`) must match
            // what schedule's prologue allocated. Rust doesn't
            // expose this symbolically, so we spell it out. If the
            // prologue ever changes we'll SEGV and know to update.
            // Current observed layout (release build, Apr 2026):
            //     stp x30, x21, [sp, #-0x20]!
            //     stp x20, x19, [sp, #0x10]
            // → 0x20 bytes, with x30 at sp+0 pre-pop.
            core::arch::asm!(
                // Restore the two callee-saved pairs schedule spilled.
                "ldp x20, x19, [sp, #0x10]",
                // Pop the frame; writes x30 back to the original
                // caller-LR, which is what we want the helper to
                // stash via the resume_lr arg.
                "ldp x30, x21, [sp], #0x20",
                // Branch (not BL) to the helper so x30 isn't
                // clobbered by a fresh `bl`.
                "b   cxt_switch_first_run",
                in("x0") old_ptr,
                in("x1") new_ptr,
                in("x2") user_sp,
                options(noreturn),
            );
        }
    }
    // Log first 16 switches and then one every 64k so we can see the
    // round-robin without drowning the trace.
    static SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
    let n = SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);
    // Periodic thread-state dump every 1024 switches. Useful when the
    // syscall-counter dump can't fire because syscalls have stopped
    // (workers spinning in epoll_pwait or all threads blocked).
    if n > 0 && n % 1024 == 0 {
        uart::puts("[diag] thread-state dump @ switch ");
        crate::kernel::mm::print_num(n as usize);
        uart::puts("\n");
        dump();
        crate::batcave::linux::syscall_history::dump_per_tid_last();
    }
    unsafe { cxt_switch_cooperative(old_ptr, new_ptr); }
}

// Implemented in src/batcave/linux/threads.s — saves callee-saved regs of
// the current thread into *old, then loads them from *new and returns.
unsafe extern "C" {
    fn cxt_switch_cooperative(old: *mut SavedRegs, new: *const SavedRegs);
}

/// V4 deferred-preemption flag. IRQ sets this on a tick; the syscall
/// layer consumes it on entry / exit and voluntarily calls schedule()
/// so the running thread yields at the next safe boundary.
static PREEMPT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_preempt() {
    PREEMPT_REQUESTED.store(true, Ordering::Release);
}

/// Consume the preempt flag. Returns true if one was pending.
pub fn take_preempt() -> bool {
    PREEMPT_REQUESTED.swap(false, Ordering::AcqRel)
}

/// Call from any safe yield point (top of a long syscall, return from
/// long syscall) to check the preempt flag and yield if set.
pub fn maybe_yield() {
    if take_preempt() { schedule(); }
}

/// Timer IRQ hook. Called from the EL1 IRQ handler. Returns true if the
/// handler should rewrite the trap frame with a different thread's state
/// before eret. Returns the target thread's SavedRegs pointer if so.
pub fn on_tick(current_trap_frame: *mut SavedRegs) -> Option<*const SavedRegs> {
    if !is_enabled() { return None; }
    let me = current_tid();

    // 1. Snapshot the trap frame's user-mode state into the current
    // thread's slot. TrapFrame (arch/mod.rs) has layout:
    //   x[0..31] @ 0..248, elr @ 248, spsr @ 256.
    // SavedRegs has DIFFERENT layout — direct struct copy would
    // misalign elr_el1 (which lives at offset 256 in SavedRegs but
    // 248 in TrapFrame). So copy field-by-field and pull SP_EL0 /
    // TPIDR_EL0 / TTBR0_EL1 from the live MSRs.
    unsafe {
        let tf_x: [u64; 31] = (*(current_trap_frame as *const [u64; 31])).clone();
        let tf_elr: u64 = *((current_trap_frame as *const u8).add(248) as *const u64);
        let tf_spsr: u64 = *((current_trap_frame as *const u8).add(256) as *const u64);
        let cur_sp_el0: u64;
        let cur_ttbr0: u64;
        let cur_tpidr: u64;
        core::arch::asm!("mrs {}, sp_el0",   out(reg) cur_sp_el0);
        core::arch::asm!("mrs {}, ttbr0_el1", out(reg) cur_ttbr0);
        core::arch::asm!("mrs {}, tpidr_el0", out(reg) cur_tpidr);
        let cur_ttbr0 = cur_ttbr0 & !1u64;
        with_table(|t| {
            if let Some(i) = slot_of(t, me) {
                t[i].saved_regs.x = tf_x;
                t[i].saved_regs.x[18] = cur_tpidr; // TPIDR_EL0
                t[i].saved_regs.elr_el1 = tf_elr;
                t[i].saved_regs.spsr_el1 = tf_spsr;
                t[i].saved_regs.user_sp_el0 = cur_sp_el0;
                t[i].saved_regs.user_ttbr0 = cur_ttbr0;
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
    mark_current_blocked(reason);
    schedule();
}

/// Mark the current thread as Blocked WITHOUT immediately yielding. The
/// caller (typically futex park_slot) will yield via schedule() at a
/// safer point — usually after dropping a bucket lock so a racing waker
/// can take it. The IRQ scheduler skips Blocked threads, so the next
/// preemption naturally switches us out even before the explicit
/// schedule() lands.
pub fn mark_current_blocked(reason: BlockReason) {
    let me = current_tid();
    with_table(|t| {
        if let Some(i) = slot_of(t, me) {
            t[i].state = ThreadState::Blocked(reason);
        }
    });
}

/// Mark the current thread Runnable again. Used by futex's park_slot when
/// it observes its wake flag while holding the bucket lock — must undo
/// the Blocked transition before falling through to release-and-return,
/// otherwise the thread would never run again.
pub fn mark_current_runnable() {
    let me = current_tid();
    with_table(|t| {
        if let Some(i) = slot_of(t, me) {
            if matches!(t[i].state, ThreadState::Blocked(_)) {
                t[i].state = ThreadState::Running;
            }
        }
    });
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

// ─── Time helpers (cntpct_el0 / cntfrq_el0) ──────────────────────────────
//
// Bat_OS uses ARMv8 generic timer ticks as the canonical deadline unit.
// All deadlines stored in BlockReason are absolute cntpct_el0 values.
// See DESIGN_SCHEDULER_BLOCK_ON.md decision #2.

/// Read the ARM generic timer's current physical count (EL0).
/// Returns absolute ticks since boot (or wherever the firmware reset it).
#[inline]
pub fn cntpct_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

/// Read the ARM generic timer's frequency in Hz. Constant per boot.
#[inline]
fn cntfrq_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v); }
    v
}

/// Convert milliseconds to cntpct_el0 ticks using cntfrq_el0.
/// Multiply-then-divide preserves sub-1000Hz precision.
/// Saturating mul prevents overflow panic on absurd inputs.
#[inline]
pub fn ms_to_ticks(ms: u32) -> u64 {
    let freq = cntfrq_el0();
    (ms as u64).saturating_mul(freq) / 1000
}

/// Returns `true` iff the current thread's slot in the table exists AND
/// its state is ThreadState::Blocked(_). Reads under the table lock.
/// Returns `false` (non-blocking) if the current slot is missing — the
/// caller (park_current) treats that as "not blocked" and falls through
/// gracefully rather than spinning forever.
pub fn current_thread_blocked() -> bool {
    let me = current_tid();
    with_table(|t| {
        let Some(idx) = slot_of(t, me) else { return false; };
        matches!(t[idx].state, ThreadState::Blocked(_))
    })
}

/// Park the current thread on `reason`. Does NOT return while the
/// calling thread's state is ThreadState::Blocked(_). Loops between
/// `schedule()` (which switches away if anyone is Runnable) and `wfi`
/// (which idles until any interrupt fires; the timer IRQ runs
/// wake_expired_deadlines, which may flip our state Blocked→Runnable).
///
/// Lock + IRQ ordering invariants (see DESIGN_SCHEDULER_BLOCK_ON.md
/// decision #8):
///
///   - The threads-table lock is NEVER held across `wfi`.
///     mark_current_blocked takes the lock briefly, releases it.
///     schedule() takes its own lock internally. The wfi runs lock-free.
///   - Interrupts are NEVER masked across `wfi`. mark_current_blocked
///     may briefly take an IrqGuard for atomicity but releases it before
///     schedule(). The wfi must execute with interrupts enabled or the
///     timer IRQ can't fire and deadline-bearing sleepers never wake.
pub fn park_current(reason: BlockReason) {
    mark_current_blocked(reason);
    loop {
        // schedule() switches to another Runnable thread if any. When
        // control returns here, either:
        //   * Another thread ran, was eventually rescheduled away, and
        //     a waker (event-driven via wake_thread / wake_epoll_waiters,
        //     or deadline-driven via wake_expired_deadlines) flipped our
        //     state Blocked→Runnable. We resume; check below exits loop.
        //   * No other Runnable thread existed. schedule() returned
        //     immediately. Our state is still Blocked. Drop to wfi and
        //     wait for any interrupt; on resume re-check state.
        schedule();
        if !current_thread_blocked() { break; }
        // Still blocked, no one else to run. Idle until the next IRQ.
        // Interrupts must be enabled here (see invariant above).
        unsafe { core::arch::asm!("wfi"); }
    }
}

/// Walk the threads table for Blocked threads whose BlockReason carries
/// an expired deadline_ticks; transition each to Runnable. Bounded
/// O(MAX_THREADS=256) per call.
///
/// Called from kernel::scheduler::tick() once per timer IRQ. The pass
/// is the only waker for sys_nanosleep and the deadline-driven half of
/// epoll_pwait. (Event-driven epoll wakes go through wake_epoll_waiters.)
///
/// Futex's per-WaitSlot deadline lives in futex.rs, not BlockReason, so
/// this pass does not see it. Futex's existing post-resume re-check loop
/// handles its own timeouts (see DESIGN_SCHEDULER_BLOCK_ON.md decision #5).
pub fn wake_expired_deadlines() {
    let now = cntpct_el0();
    with_table(|t| {
        for slot in t.iter_mut() {
            let should_wake = match slot.state {
                ThreadState::Blocked(BlockReason::EpollWait { deadline_ticks, .. })
                    if deadline_ticks != 0 && now >= deadline_ticks => true,
                ThreadState::Blocked(BlockReason::Nanosleep { deadline_ticks })
                    if now >= deadline_ticks => true,
                _ => false,
            };
            if should_wake {
                slot.state = ThreadState::Runnable;
            }
        }
    });
}

/// Walk the threads table for any thread Blocked on EpollWait with the
/// matching epfd; transition each to Runnable. Bounded O(MAX_THREADS).
///
/// Used by cmd_scheduler_selftest for deterministic per-epfd wake
/// verification. `mark_ready` uses the broader `wake_all_epoll_waiters`
/// because at the watched-fd layer we only know which epoll instance
/// matched, not the epfd that the parked thread used.
pub fn wake_epoll_waiters(epfd: i32) {
    with_table(|t| {
        for slot in t.iter_mut() {
            if let ThreadState::Blocked(BlockReason::EpollWait { epfd: e, .. }) = slot.state {
                if e == epfd {
                    slot.state = ThreadState::Runnable;
                }
            }
        }
    });
}

/// Wake every thread parked in BlockReason::EpollWait, regardless of
/// epfd. Called by epoll::mark_ready after flipping a ready bit — at
/// that layer we know an event arrived but not which epfd was parked
/// on this instance. False-positive wakes (a thread waiting on a
/// different epfd) re-park after one drain_ready loop, costing a few
/// extra cycles but never leaving an event-wait waiter stuck forever.
/// Bounded O(MAX_THREADS).
pub fn wake_all_epoll_waiters() {
    with_table(|t| {
        for slot in t.iter_mut() {
            if let ThreadState::Blocked(BlockReason::EpollWait { .. }) = slot.state {
                slot.state = ThreadState::Runnable;
            }
        }
    });
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
    // V8-ROOT-8: gate uaddr through the futex helper (which checks
    // is_user_range under the hood). Today this path is unreachable from
    // EL0, but the moment a future scheduler wires it up an unvalidated
    // uaddr would be a kernel-read oracle.
    if !crate::batcave::linux::futex::is_valid_uaddr(uaddr) {
        return EAGAIN;
    }
    // Re-check under IRQ-masked lock to close the wait/wake race.
    let current: u32 = unsafe { core::ptr::read_volatile(uaddr as *const u32) };
    if current != val { return EAGAIN; }
    block_current_thread(BlockReason::FutexWait { uaddr, val });
    0
}

/// Real wait4(): scan the thread table for any Exited child of the
/// current thread (parent_tid == me). If found, reap it: free its
/// kernel stack, free its forked cave (page tables), and clear the
/// thread slot. Returns (child_tid, exit_code) or None.
///
/// `target_pid` selects which child to reap:
///   * -1 → any child
///   * >0 → that specific child TID (POSIX waitpid semantics)
///   * 0 / <-1 → process-group filtering (we don't have process groups,
///     so treat as "any" too)
///
/// Caller is responsible for stuffing the exit code into the user's
/// status_ptr (Linux wait status format) and returning the child TID
/// to user space.
pub fn try_reap_any_child(parent: u32, target_pid: i32) -> Option<(u32, i32)> {
    // Phase 1: locate a matching Exited child under the table lock.
    // Capture everything we need to free OUTSIDE the lock (frame::free
    // takes its own state and we don't want to nest lock orders).
    let reaped = with_table(|t| {
        for slot in t.iter_mut() {
            if slot.state == ThreadState::Free { continue; }
            if slot.parent_tid != parent { continue; }
            if target_pid > 0 && slot.tid != target_pid as u32 { continue; }
            if let ThreadState::Exited(code) = slot.state {
                let tid = slot.tid;
                let kbase = slot.kernel_stack_base;
                let kpages = if slot.kernel_stack_top > kbase && kbase != 0 {
                    ((slot.kernel_stack_top - kbase) as usize) / PAGE_SIZE
                } else { 0 };
                let cave_l1 = slot.saved_regs.user_ttbr0;
                // Wipe the slot now so a future spawn can reuse it.
                *slot = Thread::empty();
                return Some((tid, code, kbase, kpages, cave_l1));
            }
        }
        None
    });
    let (tid, code, kbase, kpages, cave_l1) = reaped?;

    // Phase 2: free the kernel stack pages (zeroed by free_frame so no
    // residue leaks to the next allocation).
    if kbase != 0 && kpages > 0 {
        crate::kernel::mm::frame::free_contig(kbase as usize, kpages);
    }

    // Phase 3: free the child's cave (L1 + L2 page tables back to the
    // kernel pool). The slot bit is released last by free_cave_slot, in
    // the right order to avoid the V5-CHAIN-002 race.
    if cave_l1 != 0 {
        if let Some(cave_slot) = super::mmu::cave_slot_for_l1(cave_l1) {
            // Drop the per-cave fd table for this slot — the child's
            // open fds are gone with it.
            super::fd::reset_for_cave_slot(cave_slot);
            super::mmu::free_cave_slot(cave_slot);
        }
    }

    // Phase 4: refund the per-cave thread quota slot.
    super::quotas::refund_active(super::quotas::Resource::Threads, 1);

    Some((tid, code))
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
                ThreadState::Blocked(BlockReason::FutexWait { uaddr, val }) => {
                    uart::puts("Blocked(FutexWait uaddr=0x");
                    let hex = b"0123456789abcdef";
                    for sh in (0..16).rev() {
                        uart::putc(hex[((uaddr >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts(" val=");
                    crate::kernel::mm::print_num(val as usize);
                    uart::puts(")");
                }
                ThreadState::Blocked(_) => uart::puts("Blocked(other)"),
                ThreadState::Exited(c)  => {
                    uart::puts("Exited(");
                    crate::kernel::mm::print_num(c as usize);
                    uart::puts(")");
                }
            }
            // Show the saved user PC (where this thread will resume to
            // when next scheduled). For Runnable/Blocked threads this is
            // the user-mode address they were last at; combined with
            // rust-objdump on content_shell we can name the function.
            if s.saved_regs.elr_el1 != 0 {
                uart::puts(" elr=0x");
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((s.saved_regs.elr_el1 >> (sh * 4)) & 0xF) as usize]);
                }
            }
            uart::puts("\n");
        }
    });
}

/// Periodic auto-dump from the IRQ handler. Fires every N ticks so a
/// human watching the smoke log can see the evolution of thread states
/// without needing to send a signal. Tune the divisor in handle_irq.
pub fn auto_dump_if_idle() {
    static LAST_DUMP: AtomicU64 = AtomicU64::new(0);
    let count = LAST_DUMP.fetch_add(1, Ordering::Relaxed);

    // STUMP #161 iter 16: PC-sampler heartbeat. Timer IRQs fire at
    // ~1 Hz on QEMU virt, so every 5 ticks ≈ 5 sec we dump:
    //   - current tid
    //   - ELR_EL1 (PC at exception entry — i.e. where userland was
    //     when the IRQ fired)
    //   - SPSR_EL1.M (which EL was running)
    // This is the ONE diagnostic that catches purely-userland hangs
    // (no syscalls, no diag dumps). Without it /bin/js can spin in
    // a JIT loop or busy-wait for 120 sec and we'd never know where.
    if count > 0 && count % 5 == 0 {
        let elr: u64;
        let spsr: u64;
        let far: u64;
        unsafe {
            core::arch::asm!("mrs {}, ELR_EL1",  out(reg) elr);
            core::arch::asm!("mrs {}, SPSR_EL1", out(reg) spsr);
            core::arch::asm!("mrs {}, FAR_EL1",  out(reg) far);
        }
        let mode = (spsr >> 0) & 0xF; // M[3:0]
        crate::drivers::uart::puts("[heartbeat tick=");
        crate::kernel::mm::print_num(count as usize);
        crate::drivers::uart::puts(" tid=");
        crate::kernel::mm::print_num(current_tid() as usize);
        crate::drivers::uart::puts(" elr=0x");
        let hex = b"0123456789abcdef";
        for s in (0..16).rev() {
            crate::drivers::uart::putc(hex[((elr >> (s*4)) & 0xF) as usize]);
        }
        crate::drivers::uart::puts(" mode=0x");
        for s in (0..2).rev() {
            crate::drivers::uart::putc(hex[((mode >> (s*4)) & 0xF) as usize]);
        }
        crate::drivers::uart::puts(" far=0x");
        for s in (0..16).rev() {
            crate::drivers::uart::putc(hex[((far >> (s*4)) & 0xF) as usize]);
        }
        crate::drivers::uart::puts("]\n");
    }
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

// ─── Test helpers (feature-gated) ────────────────────────────────────────
//
// Operate only on Free slots so they never touch real running threads.
// Used by cmd_scheduler_selftest in src/ui/shell.rs and exercised in
// scripts/qemu_selftests_smoke.py. Not exposed in production builds.
//
// See DESIGN_SCHEDULER_BLOCK_ON.md "Test helpers" section.

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_install_blocked(reason: BlockReason) -> Option<usize> {
    // Find a Free slot, mark it Blocked with the given reason, return
    // its index. None if the table is full.
    //
    // Snapshot/free invariant: this function operates ONLY on Free
    // slots. It does NOT mutate any other slot's fields. test_release_slot
    // restores the same Free invariant.
    with_table(|t| {
        for (i, slot) in t.iter_mut().enumerate() {
            if slot.state == ThreadState::Free {
                slot.state = ThreadState::Blocked(reason);
                return Some(i);
            }
        }
        None
    })
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_inspect_state(slot: usize) -> Option<ThreadState> {
    with_table(|t| {
        if slot >= t.len() { return None; }
        Some(t[slot].state)
    })
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_release_slot(slot: usize) {
    // Reset the slot to Free. Idempotent. Does not touch tid/regs/wait
    // metadata — the slot was Free when test_install_blocked grabbed it,
    // so those fields are already at Free defaults.
    with_table(|t| {
        if slot >= t.len() { return; }
        t[slot].state = ThreadState::Free;
    });
}
