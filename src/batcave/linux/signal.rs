// Bat_OS — POSIX signal delivery for BatCave Linux processes.
//
// Chromium / V8 relies on in-process signal handlers for a number of
// load-bearing features:
//   * WebAssembly bounds-check trap handlers (`udf #1` sentinel bytes
//     in the V8 pointer-compression cage, caught as SIGILL, rerouted
//     to the WASM trap-handler chain).
//   * OOB-access trap handlers for the V8 sandbox.
//   * Stack-overflow guard pages (SIGSEGV → stack-expansion / bailout).
//   * Crash reporters for fatal signals.
//
// Without actually running the user-registered handlers, every
// V8-internal fault becomes a hard kernel crash. This module plumbs
// the full path: fault classification → signal mapping → signal-frame
// construction on the user stack → eret into the user's handler →
// rt_sigreturn restore.
//
// Scope for the first pass:
//   * SIGILL, SIGSEGV, SIGBUS, SIGFPE, SIGTRAP, SIGABRT.
//   * SA_SIGINFO path (three-arg handler: int, siginfo_t*, ucontext_t*).
//   * Basic ucontext with GPRs, SP, PC, PSTATE, and fault address.
//   * Single-threaded signal delivery — the signal targets whichever
//     thread caused the fault. Cross-thread tgkill wake-ups come
//     later.
//
// Not yet:
//   * sigaltstack (SA_ONSTACK)
//   * FP/NEON/SVE register save in ucontext
//   * Real-time signal queuing
//   * Signal masks enforcement during handler execution

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::drivers::uart;

// ─────────────────────────────────────────────────────────────────────────
// Constants — AArch64 Linux syscall API
// ─────────────────────────────────────────────────────────────────────────

pub const SIGHUP:  u32 = 1;
pub const SIGINT:  u32 = 2;
pub const SIGILL:  u32 = 4;
pub const SIGTRAP: u32 = 5;
pub const SIGABRT: u32 = 6;
pub const SIGBUS:  u32 = 7;
pub const SIGFPE:  u32 = 8;
pub const SIGKILL: u32 = 9;
pub const SIGSEGV: u32 = 11;
pub const SIGTERM: u32 = 15;

pub const MAX_SIG: usize = 64;

/// sigaction flags we care about.
pub const SA_NOCLDSTOP: u64 = 0x0000_0001;
pub const SA_NOCLDWAIT: u64 = 0x0000_0002;
pub const SA_SIGINFO:   u64 = 0x0000_0004;
pub const SA_RESTORER:  u64 = 0x0400_0000;
pub const SA_ONSTACK:   u64 = 0x0800_0000;
pub const SA_RESTART:   u64 = 0x1000_0000;
pub const SA_NODEFER:   u64 = 0x4000_0000;
pub const SA_RESETHAND: u64 = 0x8000_0000;

/// siginfo si_code values used by V8's trap handler path.
pub const SI_USER:      i32 = 0;
pub const ILL_ILLOPC:   i32 = 1;   // illegal opcode (UDF #X)
pub const ILL_ILLOPN:   i32 = 2;   // illegal operand
pub const ILL_ILLADR:   i32 = 3;   // illegal addressing mode
pub const ILL_ILLTRP:   i32 = 4;   // illegal trap
pub const SEGV_MAPERR:  i32 = 1;   // address not mapped to object
pub const SEGV_ACCERR:  i32 = 2;   // invalid permissions
pub const BUS_ADRALN:   i32 = 1;   // misaligned address
pub const BUS_ADRERR:   i32 = 2;   // nonexistent physical address

/// SIG_DFL and SIG_IGN — special handler pointers.
pub const SIG_DFL: u64 = 0;
pub const SIG_IGN: u64 = 1;

// ─────────────────────────────────────────────────────────────────────────
// Per-process signal-handler table
// ─────────────────────────────────────────────────────────────────────────

/// A single sigaction entry. Matches the first 32 bytes of Linux's
/// struct sigaction layout (the portion user-space cares about): handler,
/// flags, mask, restorer. Restorer is ignored — we always eret via our
/// own rt_sigreturn trampoline address and let the vsyscall layer forward
/// the call.
#[derive(Clone, Copy)]
pub struct Sigaction {
    pub handler:  u64,
    pub flags:    u64,
    pub restorer: u64,
    pub mask:     u64,
}

impl Sigaction {
    pub const fn default() -> Self {
        Self { handler: SIG_DFL, flags: 0, restorer: 0, mask: 0 }
    }
}

static mut HANDLERS: [Sigaction; MAX_SIG] = [Sigaction::default(); MAX_SIG];
static HANDLERS_LOCK: AtomicBool = AtomicBool::new(false);

fn lock_handlers() { while HANDLERS_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() { core::hint::spin_loop(); } }
fn unlock_handlers() { HANDLERS_LOCK.store(false, Ordering::Release); }

/// Install a new sigaction and optionally return the previous one.
/// Called from `sys_rt_sigaction`.
pub fn set_action(signo: u32, new: Option<Sigaction>, old_out: Option<&mut Sigaction>) -> i64 {
    if signo == 0 || signo as usize >= MAX_SIG { return -22; } // EINVAL
    if signo == SIGKILL { return -22; }
    let idx = signo as usize;
    lock_handlers();
    unsafe {
        if let Some(old) = old_out {
            *old = HANDLERS[idx];
        }
        if let Some(n) = new {
            HANDLERS[idx] = n;
        }
    }
    unlock_handlers();
    0
}

/// Look up the current sigaction for `signo`. Returns None for an
/// invalid signo.
pub fn get_action(signo: u32) -> Option<Sigaction> {
    if signo == 0 || signo as usize >= MAX_SIG { return None; }
    lock_handlers();
    let a = unsafe { HANDLERS[signo as usize] };
    unlock_handlers();
    Some(a)
}

/// Reset every handler to SIG_DFL. Called from `reset_cave_statics`.
pub fn reset() {
    lock_handlers();
    unsafe {
        let tbl = &mut *core::ptr::addr_of_mut!(HANDLERS);
        for h in tbl.iter_mut() {
            *h = Sigaction::default();
        }
    }
    unlock_handlers();
    // Also drop any pending signals and clear the block-mask; the
    // dying cave's `kill()` / `tgkill()` state must not carry over.
    PENDING.store(0, Ordering::Release);
    MASK.store(0, Ordering::Release);
}

// ─────────────────────────────────────────────────────────────────────────
// Async signal delivery — `kill(2)` / `tgkill(2)` pending-bit path
// ─────────────────────────────────────────────────────────────────────────
//
// Synchronous signals (SIGSEGV from a data abort, SIGILL from a UDF)
// are delivered inline in the fault handler — we know they need to
// fire before user code resumes because the instruction that caused
// the fault can't retire.
//
// Async signals (tgkill, timer expiry, RT signals for Chromium's
// thread-cancel path on signo=33) don't have that immediacy. When a
// sender calls `sys_tgkill(signo)` we OR a bit into PENDING here. On
// the way back to user mode (syscall return, preemption yield),
// `try_deliver_pending` pulls the lowest unblocked bit and redirects
// the trap frame into the handler just like a synchronous fault.
//
// Today PENDING is process-wide (not per-thread). That's good enough
// for Chromium's self-kill pattern — the calling thread always does
// `tgkill(getpid(), gettid(), signo)`, and the syscall-return poll
// happens on the same thread. Per-thread queuing is a V2 item.

static PENDING: AtomicU64 = AtomicU64::new(0);
static MASK:    AtomicU64 = AtomicU64::new(0);

/// OR a single signal into the pending mask. Called from `sys_tgkill`
/// / `sys_kill`. No error checking — caller is expected to validate
/// signo already.
pub fn mark_pending(signo: u32) {
    if signo == 0 || (signo as usize) >= MAX_SIG { return; }
    PENDING.fetch_or(1u64 << signo, Ordering::AcqRel);
}

/// Install a new signal-block mask. Returns the previous mask so
/// `sys_rt_sigprocmask` can report it back to user space.
pub fn set_mask(new_mask: u64) -> u64 {
    MASK.swap(new_mask, Ordering::AcqRel)
}

/// Read the current signal-block mask.
pub fn get_mask() -> u64 { MASK.load(Ordering::Acquire) }

/// Read-modify-write helpers matching the Linux
/// `SIG_BLOCK` / `SIG_UNBLOCK` / `SIG_SETMASK` `how` codes.
pub fn mask_block(add: u64) -> u64 {
    MASK.fetch_or(add, Ordering::AcqRel)
}
pub fn mask_unblock(remove: u64) -> u64 {
    MASK.fetch_and(!remove, Ordering::AcqRel)
}

/// Signals whose POSIX default action is *not* terminate: these are
/// safely dropped on the floor when delivered with SIG_DFL and no
/// handler installed. All other signals trigger `terminate_cave_fatal`
/// on SIG_DFL delivery.
///
/// Default "ignore": SIGCHLD (17), SIGURG (23), SIGWINCH (28),
/// SIGCONT (18). SIGSTOP / SIGTSTP / SIGTTIN / SIGTTOU (19/20/21/22)
/// default to "stop the process" which we treat as ignore for the
/// first pass since we have no job-control infrastructure.
const DEFAULT_IGNORE_MASK: u64 =
      (1u64 << 17)  // SIGCHLD
    | (1u64 << 18)  // SIGCONT
    | (1u64 << 19)  // SIGSTOP  (we don't stop; treat as ignore)
    | (1u64 << 20)  // SIGTSTP
    | (1u64 << 21)  // SIGTTIN
    | (1u64 << 22)  // SIGTTOU
    | (1u64 << 23)  // SIGURG
    | (1u64 << 28); // SIGWINCH

/// Pick the lowest-numbered pending-and-unblocked signal, clear its
/// bit, and return it. Returns `None` if nothing is deliverable.
fn take_pending_unblocked() -> Option<u32> {
    loop {
        let pend = PENDING.load(Ordering::Acquire);
        let mask = MASK.load(Ordering::Acquire);
        let deliverable = pend & !mask;
        if deliverable == 0 { return None; }
        let signo = deliverable.trailing_zeros();
        let bit = 1u64 << signo;
        // CAS-clear so concurrent senders don't lose a set-bit race.
        if PENDING
            .compare_exchange(pend, pend & !bit, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return Some(signo);
        }
        // Lost the race — retry. The only way we can race is if
        // another thread modified PENDING between our load and CAS;
        // either another sender ORed in a new bit (harmless — we'll
        // see it on a later poll) or we raced with ourselves on an
        // SMP system (which we don't support yet, but this is
        // future-proofing).
    }
}

/// Called on the way back to user mode (from the SVC exit path, or
/// any other safe yield point). If a pending-and-unblocked signal is
/// deliverable:
///   * SIG_IGN  → clear and continue (no delivery).
///   * SIG_DFL  → if fatal-by-default, tear the cave down; else drop.
///   * Handler  → build an rt_sigframe and redirect the trap frame.
/// Returns `true` if the trap frame was mutated (caller should NOT
/// overwrite x0 / ELR afterward); `false` if nothing happened or the
/// signal was dropped silently.
pub fn try_deliver_pending(frame: &mut TrapFrame) -> bool {
    // Loop so we drop all currently-deliverable SIG_IGN signals in a
    // single pass — otherwise a stream of ignored signals would each
    // cost one syscall-return round trip.
    loop {
        let signo = match take_pending_unblocked() {
            Some(s) => s,
            None => return false,
        };
        let action = match get_action(signo) {
            Some(a) => a,
            None    => continue,
        };
        if action.handler == SIG_IGN {
            // Drop silently; check if another signal is also pending.
            continue;
        }
        if action.handler == SIG_DFL {
            if (DEFAULT_IGNORE_MASK & (1u64 << signo)) != 0 {
                // Default action is ignore — drop silently.
                continue;
            }
            // Default action is terminate. Tear the cave down.
            terminate_cave_fatal(signo, 0);
            // never returns
        }
        // Real user handler: redirect via the synchronous path. The
        // frame's ELR already points at the next user instruction
        // (post-svc or preemption-resume point), so the saved
        // ucontext will cause rt_sigreturn to resume there.
        return try_deliver_synchronous(frame, signo, SI_TKILL, 0);
    }
}

/// siginfo si_code for a signal sent via tkill/tgkill.
pub const SI_TKILL: i32 = -6;

// ─────────────────────────────────────────────────────────────────────────
// Trap-frame / signal-frame layouts
// ─────────────────────────────────────────────────────────────────────────

/// Matches `src/kernel/arch/mod.rs::TrapFrame`.
#[repr(C)]
pub struct TrapFrame {
    pub x:    [u64; 31],
    pub elr:  u64,
    pub spsr: u64,
}

/// AArch64 Linux kernel uapi `struct sigcontext`. Contains the saved GPRs,
/// SP, PC, and PSTATE plus a 4 KiB reserved area for FP/SVE state (we
/// don't populate that yet — Chromium's fault handlers don't read it).
#[repr(C, align(16))]
pub struct Sigcontext {
    pub fault_address: u64,
    pub regs:          [u64; 31],
    pub sp:            u64,
    pub pc:            u64,
    pub pstate:        u64,
    // 4 KiB reserved — FPSIMD + SVE state go here in Linux. We zero it.
    pub reserved:      [u8; 4096],
}

/// The Linux AArch64 `struct ucontext_t` layout. The `uc_mcontext`
/// offset (176 = 0xb0) is load-bearing — glibc hand-codes it.
#[repr(C, align(16))]
pub struct Ucontext {
    pub uc_flags:   u64,             // 0x00
    pub uc_link:    u64,             // 0x08
    // stack_t: ss_sp (u64) + ss_flags (i32) + ss_size (u64) = 24 bytes,
    // but Linux pads to 24 with ss_flags at bits 8:11 plus padding.
    pub ss_sp:      u64,             // 0x10
    pub ss_flags:   i32,             // 0x18
    pub ss_pad:     i32,             // 0x1c
    pub ss_size:    u64,             // 0x20
    pub uc_sigmask: [u64; 16],       // 0x28..0xa8 (128 bytes = sigset_t)
    pub _pad_to_mc: [u8; 8],         // 0xa8..0xb0
    pub uc_mcontext: Sigcontext,     // 0xb0
}

/// The combined rt_sigframe the kernel places on the user stack before
/// dispatching to an SA_SIGINFO handler.
#[repr(C, align(16))]
pub struct RtSigframe {
    pub info: Siginfo,
    pub uc:   Ucontext,
}

/// Minimal siginfo_t — 128 bytes on AArch64 Linux.
#[repr(C, align(8))]
pub struct Siginfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code:  i32,
    pub _pad0:    i32,
    pub si_addr:  u64,
    pub _pad:     [u8; 128 - 24],
}

// ─────────────────────────────────────────────────────────────────────────
// Fault classification & signal delivery
// ─────────────────────────────────────────────────────────────────────────

/// Compile-time check that our Sigcontext/Ucontext offsets match what
/// glibc / V8 expect. If these asserts fail we'll know immediately from
/// the build — glibc's `ucontext_t.uc_mcontext` is at offset 0xb0, and
/// `sigcontext.pc` is at 0x108 (176 + 8 + 256 + 8).
const _: () = {
    assert!(core::mem::offset_of!(Ucontext, uc_mcontext) == 0xb0);
    assert!(core::mem::offset_of!(Sigcontext, regs)          == 0x08);
    assert!(core::mem::offset_of!(Sigcontext, sp)            == 0x100);
    assert!(core::mem::offset_of!(Sigcontext, pc)            == 0x108);
    assert!(core::mem::offset_of!(Sigcontext, pstate)        == 0x110);
};

/// Total stack space a full rt_sigframe consumes. We subtract this from
/// the user's SP (rounded down to 16-byte alignment) when dispatching.
pub const RT_SIGFRAME_SIZE: usize = core::mem::size_of::<RtSigframe>();

/// Round `value` down to the nearest 16-byte boundary (AArch64 AAPCS
/// requires 16-byte SP alignment at function-call boundaries).
#[inline(always)]
fn align_down_16(value: u64) -> u64 { value & !0xFu64 }

/// Count successful signal deliveries for telemetry/debugging.
pub static DELIVERIES: AtomicU64 = AtomicU64::new(0);

/// Try to deliver `signo` (triggered by a synchronous fault) to the
/// current thread. Returns `true` if the handler was installed and the
/// trap frame has been rewritten so `eret` will enter it; `false` if
/// no user handler is registered and the caller must fall through to
/// the default (usually terminal) path.
///
/// `fault_addr` is whatever FAR_EL1 or equivalent the hardware gave us.
/// `si_code` is the POSIX `si_code` classifier for the signal type.
pub fn try_deliver_synchronous(
    frame: &mut TrapFrame,
    signo: u32,
    si_code: i32,
    fault_addr: u64,
) -> bool {
    let action = match get_action(signo) {
        Some(a) => a,
        None    => return false,
    };
    if action.handler == SIG_IGN {
        // User explicitly ignored the signal. For a synchronous fault
        // that's a recipe for an infinite retry loop, but POSIX says
        // the behaviour is implementation-defined. Skip the faulting
        // instruction to make forward progress.
        frame.elr = frame.elr.wrapping_add(4);
        return true;
    }
    if action.handler == SIG_DFL {
        // Default action for synchronous fatal signals is TERMINATE.
        // We don't yet have a per-cave termination path, so return
        // false and let the caller fall through to the (currently
        // kernel-wedging) UNHANDLED dump. Log which signal we gave
        // up on so it's visible in the serial trace.
        static DFL_COUNT: core::sync::atomic::AtomicU64 =
            core::sync::atomic::AtomicU64::new(0);
        let n = DFL_COUNT.fetch_add(1, Ordering::Relaxed);
        if n < 8 {
            uart::puts("[sig] SIG_DFL signo=");
            crate::kernel::mm::print_num(signo as usize);
            uart::puts(" — terminate (no user handler)\n");
        }
        return false;
    }

    // Pick the frame base: current user SP, rounded down past the
    // rt_sigframe and aligned to 16 bytes.
    let user_sp = unsafe {
        let sp: u64;
        core::arch::asm!("mrs {}, sp_el0", out(reg) sp);
        sp
    };
    // Leave a 128-byte red zone so the handler can use stack scratch
    // without trampling the frame itself.
    let frame_top = align_down_16(user_sp.saturating_sub(128u64));
    let frame_base = align_down_16(frame_top.saturating_sub(RT_SIGFRAME_SIZE as u64));

    if frame_base < 0x1000 {
        uart::puts("[sig] frame_base too low, refusing delivery\n");
        return false;
    }
    if !super::uaccess::is_user_range(frame_base as usize, RT_SIGFRAME_SIZE) {
        uart::puts("[sig] frame_base not in user range, refusing\n");
        return false;
    }

    unsafe {
        // Zero the frame first so no stale kernel state leaks via
        // padding bytes. (Also puts FP/SVE reserved area to zero,
        // which is a valid "no FP state present" marker.)
        let raw = frame_base as *mut u8;
        for i in 0..RT_SIGFRAME_SIZE {
            core::ptr::write_volatile(raw.add(i), 0);
        }

        let sf = &mut *(frame_base as *mut RtSigframe);
        // Populate siginfo_t.
        sf.info.si_signo = signo as i32;
        sf.info.si_errno = 0;
        sf.info.si_code  = si_code;
        sf.info.si_addr  = fault_addr;
        // Populate ucontext_t.uc_mcontext from the trap frame.
        sf.uc.uc_flags = 0;
        sf.uc.uc_link  = 0;
        sf.uc.ss_sp    = 0;
        sf.uc.ss_flags = 2; // SS_DISABLE — we don't have sigaltstack yet
        sf.uc.ss_size  = 0;
        sf.uc.uc_mcontext.fault_address = fault_addr;
        sf.uc.uc_mcontext.regs.copy_from_slice(&frame.x);
        sf.uc.uc_mcontext.sp     = user_sp;
        sf.uc.uc_mcontext.pc     = frame.elr;
        sf.uc.uc_mcontext.pstate = frame.spsr;
    }

    // Redirect the trap frame: eret lands in the handler.
    //   x0 = signo
    //   x1 = pointer to siginfo_t within the frame
    //   x2 = pointer to ucontext_t within the frame
    //   lr = restorer (our rt_sigreturn trampoline — planted in the frame)
    //   sp = frame_base
    //   pc = handler
    //
    // We don't have a vDSO to host the restorer, so we pre-load a
    // known user-space address with the return trampoline instead.
    // See `restorer_addr` below.
    frame.x[0] = signo as u64;
    let info_va = frame_base;
    let uc_va   = frame_base + core::mem::offset_of!(RtSigframe, uc) as u64;
    frame.x[1] = info_va;
    frame.x[2] = uc_va;
    frame.x[30] = restorer_addr();
    frame.elr  = action.handler;
    // Keep SPSR as-is (same PSTATE the thread had at fault time).

    // Adjust user SP so the handler sees the frame.
    unsafe {
        core::arch::asm!("msr sp_el0, {}", in(reg) frame_base);
    }

    let n = DELIVERIES.fetch_add(1, Ordering::Relaxed);
    if n < 16 || (n & 0xFF) == 0 {
        uart::puts("[sig] deliver signo=");
        crate::kernel::mm::print_num(signo as usize);
        uart::puts(" handler=0x");
        let hex = b"0123456789abcdef";
        for sh in (0..16).rev() {
            uart::putc(hex[((action.handler >> (sh*4)) & 0xF) as usize]);
        }
        uart::puts(" fault=0x");
        for sh in (0..16).rev() {
            uart::putc(hex[((fault_addr >> (sh*4)) & 0xF) as usize]);
        }
        uart::puts(" frame=0x");
        for sh in (0..16).rev() {
            uart::putc(hex[((frame_base >> (sh*4)) & 0xF) as usize]);
        }
        uart::puts(" #"); crate::kernel::mm::print_num(n as usize);
        uart::puts("\n");
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────
// rt_sigreturn trampoline
// ─────────────────────────────────────────────────────────────────────────
//
// Signal handlers return via the `restorer` address we put in x30. The
// restorer is a tiny piece of user-mode code that invokes the
// rt_sigreturn syscall (#139). We plant it in a fixed user page (the
// "signal trampoline page") that every cave has mapped RX on boot. If
// that page isn't available we fall back to a well-known libc address
// (glibc provides its own `__restore_rt` symbol via the sa_restorer
// field — when present we use that instead).
//
// For the first pass we mint our own trampoline and stash its address
// here so the per-thread-signal code has a stable LR to use.

static RESTORER_ADDR: AtomicU64 = AtomicU64::new(0);

pub fn set_restorer_addr(addr: u64) {
    RESTORER_ADDR.store(addr, Ordering::Release);
}

#[inline]
fn restorer_addr() -> u64 {
    RESTORER_ADDR.load(Ordering::Acquire)
}

/// Bytes of the restorer we inject: a `mov x8, #139 ; svc #0` sequence.
/// When the handler returns (via ret → x30 = this address), the user
/// falls into this code and triggers the rt_sigreturn syscall with no
/// further user-space scaffolding. The kernel then restores the
/// pre-signal trap frame from the ucontext saved on the user stack.
pub const RESTORER_BYTES: [u8; 8] = [
    0x68, 0x11, 0x80, 0xd2,   // mov x8, #139
    0x01, 0x00, 0x00, 0xd4,   // svc #0
];

/// Fixed user VA where we plant the rt_sigreturn trampoline for the
/// current cave. Chosen WELL above every library we load and away
/// from V8's heap cage / Ladybird's mimalloc arena. User code only
/// ever reaches this address via the x30 we pre-load at signal
/// delivery time, so it doesn't need to sit in a "natural" mmap
/// region.
///
/// History: 0x0080_0000 (8 MB) collided with Ladybird's
/// LibJS/mimalloc, which writes a zero byte to 0x800000 during init
/// (probably part of a fixed-base pointer-compression scheme that
/// uses low-VA region for cell metadata). Moved to 0x0FFF_F000
/// (just below the 256 MB cave_virt_base) — outside any pre-cave
/// heap arena, outside the cave's library-loading region, outside
/// the small-mmap region (0x70_xxxx_xxxx), and outside V8's typical
/// 0x40_xxxx_xxxx and 0x30_xxxx_xxxx reservation hints.
pub const RT_SIGRETURN_TRAMPOLINE_VA: u64 = 0x0FFF_F000;

/// Allocate a fresh 4 KB frame, copy the restorer bytes into it, map
/// it at `RT_SIGRETURN_TRAMPOLINE_VA` in the cave's L1 (passed in
/// `l1_phys`), and stash the VA in `RESTORER_ADDR`. Must be called
/// once per cave after its L1 is active.
pub fn install_trampoline(l1_phys: u64) -> Result<(), &'static str> {
    use crate::kernel::mm::frame;

    let page = frame::alloc_frame().ok_or("trampoline: OOM")?;
    unsafe {
        // Zero the page first.
        let p = page as *mut u8;
        for i in 0..4096 {
            core::ptr::write_volatile(p.add(i), 0);
        }
        // Copy the restorer bytes at offset 0.
        for (i, b) in RESTORER_BYTES.iter().enumerate() {
            core::ptr::write_volatile(p.add(i), *b);
        }
    }

    // Same page-table flags as a regular user page but *with* the exec
    // permission (UXN cleared). Our demand_page default is UXN-on, so
    // we have to program the entry by hand.
    const PAGE_VALID:      u64 = 0b11;
    const PAGE_AF:         u64 = 1 << 10;
    const PAGE_SH:         u64 = 3 << 8;
    const PAGE_AP_EL0_RO:  u64 = 0b11 << 6;  // EL0 R/O (no write allowed)
    const PAGE_PXN:        u64 = 1 << 53;
    // UXN = 0 → EL0 can execute
    let flags = PAGE_VALID | PAGE_AF | PAGE_SH | PAGE_AP_EL0_RO | PAGE_PXN;
    let frame_pa_masked = (page as u64) & 0x0000_FFFF_FFFF_F000;
    let entry = frame_pa_masked | flags;

    install_l3_mapping(l1_phys, RT_SIGRETURN_TRAMPOLINE_VA, entry)?;

    // STUMP #161 iter 11+12: Ladybird's glibc zero-fills a RANGE
    // starting at 0x0080_0000 during init (observed via [dp/low-va]
    // diagnostic — first attempt with one page caught 0x800000,
    // second attempt saw 0x801000, ...). It's a hot loop that writes
    // zeros to a hardcoded address.
    //
    // Pre-map 64 pages (256 KB) at 0x0080_0000 as RW zero-pages in
    // every cave. If glibc's range exceeds 256 KB we'll see new
    // [dp/low-va] events at 0x84_0000+ and can grow further; for now
    // 256 KB is enough headroom for any plausible TLS-init or RSEQ
    // setup scratch area without burning frames unnecessarily.
    {
        const SCRATCH_BASE: u64 = 0x0080_0000;
        const SCRATCH_PAGES: usize = 64;  // 256 KB
        const PAGE_AP_EL0_RW: u64 = 0b01 << 6;
        const PAGE_UXN:       u64 = 1 << 54;
        let scratch_flags = PAGE_VALID | PAGE_AF | PAGE_SH
            | PAGE_AP_EL0_RW | PAGE_PXN | PAGE_UXN;
        for i in 0..SCRATCH_PAGES {
            let pg = frame::alloc_frame().ok_or("scratch: OOM")?;
            unsafe {
                let p = pg as *mut u8;
                for j in 0..4096 {
                    core::ptr::write_volatile(p.add(j), 0);
                }
            }
            let pa = (pg as u64) & 0x0000_FFFF_FFFF_F000;
            let entry = pa | scratch_flags;
            install_l3_mapping(
                l1_phys,
                SCRATCH_BASE + (i as u64) * 4096,
                entry,
            )?;
        }
        uart::puts("[sig] glibc-scratch range 0x800000..0x840000 (256 KB RW)\n");
    }

    // TLB sledgehammer; cave just switched TTBR0 anyway so this is
    // mostly for our own peace of mind.
    unsafe {
        core::arch::asm!("dsb ishst");
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }
    RESTORER_ADDR.store(RT_SIGRETURN_TRAMPOLINE_VA, Ordering::Release);

    uart::puts("[sig] rt_sigreturn trampoline @ 0x");
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((RT_SIGRETURN_TRAMPOLINE_VA >> (sh*4)) & 0xF) as usize]);
    }
    uart::puts(" -> phys 0x");
    for sh in (0..16).rev() {
        uart::putc(hex[((page as u64 >> (sh*4)) & 0xF) as usize]);
    }
    uart::puts("\n");
    Ok(())
}

/// Walk / create L1 → L2 → L3 for `user_va` under the cave's L1 and
/// install the provided raw L3 entry. Reimplemented locally because
/// the demand-page version has slightly different permission flags
/// and this version needs to force the trampoline page executable.
fn install_l3_mapping(l1_phys: u64, user_va: u64, l3_entry: u64)
    -> Result<(), &'static str>
{
    use crate::kernel::mm::frame;
    const TABLE_DESC: u64 = 0b11;

    let l1_idx = ((user_va >> 30) & 0x1FF) as usize;
    let l2_idx = ((user_va >> 21) & 0x1FF) as usize;
    let l3_idx = ((user_va >> 12) & 0x1FF) as usize;

    // L1 entry → L2 table.
    let l1_ent_ptr = (l1_phys + (l1_idx * 8) as u64) as *mut u64;
    let l1_ent = unsafe { core::ptr::read_volatile(l1_ent_ptr) };
    let l2_phys = if (l1_ent & TABLE_DESC) == TABLE_DESC {
        l1_ent & 0x0000_FFFF_FFFF_F000
    } else {
        let new_l2 = frame::alloc_frame().ok_or("L2 alloc")? as u64;
        // Zero the new L2 table.
        unsafe {
            let p = new_l2 as *mut u8;
            for i in 0..4096 { core::ptr::write_volatile(p.add(i), 0); }
        }
        unsafe {
            core::ptr::write_volatile(l1_ent_ptr, new_l2 | TABLE_DESC);
        }
        new_l2
    };

    // L2 entry → L3 table.
    let l2_ent_ptr = (l2_phys + (l2_idx * 8) as u64) as *mut u64;
    let l2_ent = unsafe { core::ptr::read_volatile(l2_ent_ptr) };
    let l3_phys = if (l2_ent & TABLE_DESC) == TABLE_DESC {
        l2_ent & 0x0000_FFFF_FFFF_F000
    } else {
        let new_l3 = frame::alloc_frame().ok_or("L3 alloc")? as u64;
        unsafe {
            let p = new_l3 as *mut u8;
            for i in 0..4096 { core::ptr::write_volatile(p.add(i), 0); }
        }
        unsafe {
            core::ptr::write_volatile(l2_ent_ptr, new_l3 | TABLE_DESC);
        }
        new_l3
    };

    // L3 entry — the leaf.
    let l3_ent_ptr = (l3_phys + (l3_idx * 8) as u64) as *mut u64;
    unsafe {
        core::ptr::write_volatile(l3_ent_ptr, l3_entry);
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────
// rt_sigreturn entry point
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Fatal-signal cave termination (SIG_DFL for synchronous faults)
// ─────────────────────────────────────────────────────────────────────────
//
// POSIX default action for SIGSEGV / SIGILL / SIGBUS / SIGFPE / SIGABRT
// / SIGTRAP is "terminate process abnormally" (typically with a core
// dump). When a user-mode synchronous fault lands in `try_deliver_
// synchronous` with no handler installed, we used to fall through to
// the arch UNHANDLED dump and `wfe` the whole kernel — which meant
// Chromium's first crash bricked the machine and made the test loop
// serial-only through a power cycle.
//
// This function is the soft-landing: it logs the signal, tears the
// cave's TTBR0 back to primary, restores the kernel stack the loader
// stashed before erets, and jumps into the shell's main loop. It
// mirrors exactly what the regular exit-syscall path does (see
// `arch/mod.rs` line ~1000), just without the user-level `exit()`
// getting a chance to run.
//
// NB: we intentionally don't free the cave's page table / ELF frames
// here. The host-cave owner (runner.rs) owns those lifetimes, and
// doing the free from inside an exception handler would race with
// whatever `desktop::resume()` does next. For today, the cave's
// residual frames stay allocated until reboot — which matches the
// status quo for a normal `exit_group` too.

/// Tear down the current cave after a fatal synchronous fault.
/// Called from the arch handler's UNHANDLED path when SPSR shows the
/// fault came from EL0 and no user handler is installed for the
/// signal. Never returns — resumes the shell.
pub fn terminate_cave_fatal(signo: u32, fault_addr: u64) -> ! {
    terminate_cave_fatal_with_lr(signo, fault_addr, 0)
}

/// Variant that also prints x30 / LR — invaluable when ELR=0x1 (jumped
/// to a bad function pointer) and we need the caller's resume PC.
pub fn terminate_cave_fatal_with_lr(signo: u32, fault_addr: u64, lr: u64) -> ! {
    uart::puts("[sig] fatal signo=");
    crate::kernel::mm::print_num(signo as usize);
    uart::puts(" fault=0x");
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        uart::putc(hex[((fault_addr >> (sh * 4)) & 0xF) as usize]);
    }
    // 🎯 STUMP #14: also print user PC (ELR_EL1) so we can addr2line
    // the actual instruction that faulted. fault_addr is just the
    // dereferenced address (FAR) — without ELR we can't tell which
    // user-code function blew up.
    let elr_now: u64;
    unsafe { core::arch::asm!("mrs {}, elr_el1", out(reg) elr_now); }
    uart::puts(" elr=0x");
    for sh in (0..16).rev() {
        uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
    }
    if lr != 0 {
        uart::puts(" lr=0x");
        for sh in (0..16).rev() {
            uart::putc(hex[((lr >> (sh * 4)) & 0xF) as usize]);
        }
    }
    uart::puts(" — terminating cave, returning to shell\n");

    // 🎯 STUMP #17: when ELR/LR are both garbage (e.g. =0x1, jumped via
    // bad function pointer), the only way to identify the culprit is
    // walking the user FP chain. Dump up to 16 saved (FP, LR) pairs
    // from the top of the user stack so we can see the genuine call
    // chain.
    //
    // 🎯 STUMP #16: each load is gated by is_user_range so we don't
    // recursively fault on an unmapped tail page (the cave's stack
    // mmap may end before sp+0x100). On reject, stop the dump.
    let sp_el0: u64;
    unsafe { core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0); }
    if sp_el0 >= 0x1000 && sp_el0 < 0x0000_0100_0000_0000 {
        uart::puts("  user-stack@sp:");
        for i in 0..16usize {
            let off = i * 16;
            let addr = sp_el0 + off as u64;
            // Bail if either u64 in this 16-byte slot would overflow
            // out of a known user range.
            if !crate::batcave::linux::uaccess::is_user_range(addr as usize, 16) {
                uart::puts("\n    [stops at unmapped page]");
                break;
            }
            // 🎯 STUMP #57: also check the page is actually MAPPED.
            // is_user_range only validates the address falls in our
            // user VA window — the page itself can still be uncommitted
            // (e.g. demand-paged but not yet faulted in). Under TCG a
            // raw `ldr` against an unmapped user page silently returns
            // garbage; under HVF the resulting translation fault has
            // ESR.ISV=0 and QEMU asserts hvf_handle_exception.
            if !crate::kernel::arch::page_is_mapped(addr)
                || !crate::kernel::arch::page_is_mapped(addr + 8)
            {
                uart::puts("\n    [stops at unmapped page]");
                break;
            }
            let fp_v: u64 = unsafe {
                let v: u64;
                core::arch::asm!("ldr {v}, [{a}]",
                    a = in(reg) addr, v = out(reg) v);
                v
            };
            let lr_v: u64 = unsafe {
                let v: u64;
                core::arch::asm!("ldr {v}, [{a}]",
                    a = in(reg) addr + 8, v = out(reg) v);
                v
            };
            uart::puts("\n    +0x");
            let h = b"0123456789abcdef";
            uart::putc(h[(off >> 8) & 0xF]);
            uart::putc(h[(off >> 4) & 0xF]);
            uart::putc(h[off & 0xF]);
            uart::puts(": fp=0x");
            for sh in (0..16).rev() {
                uart::putc(h[((fp_v >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" lr=0x");
            for sh in (0..16).rev() {
                uart::putc(h[((lr_v >> (sh * 4)) & 0xF) as usize]);
            }
        }
        uart::puts("\n");
    }

    // Clear the per-cave signal-handler table so a subsequent cave
    // doesn't inherit the dying Chromium's dispositions. (The full
    // `reset_cave_statics` runs at the next cave launch anyway, but
    // we wipe here too so anything that peeks between now and then
    // sees a clean slate.)
    reset();

    unsafe {
        // Mirror the exit-syscall path in `arch/mod.rs`:
        //   1. Switch TTBR0 back to primary so no stray memory access
        //      between here and `desktop::resume()` touches the dying
        //      cave's page tables.
        //   2. Restore the kernel SP the loader stashed in
        //      KERNEL_SP_SAVE before the eret into EL0. Without this
        //      we'd continue on the exception-entry kernel stack and
        //      eventually blow it.
        //   3. Jump to `desktop::resume()` — runs the shell prompt
        //      forever. `-> !` means we never come back here.
        crate::batcave::linux::mmu::switch_to_primary();
        let save_addr = crate::kernel::arch::kernel_sp_save_addr();
        core::arch::asm!(
            "ldr x0, [{addr}]",
            "mov sp, x0",
            addr = in(reg) save_addr,
            out("x0") _,
        );
    }
    // BAT_OS_KEEP_GOING: dump the skip-summary on cave-fatal teardown
    // too — otherwise a fatal fault that fires before the shell's
    // post-run dump_summary call would lose every event recorded
    // during this run.
    super::skip_log::dump_summary();

    // 🎯 STUMP #57b: in headless mode, `console`/`gpu` aren't
    // initialized, so calling desktop::resume() → console::prompt()
    // dereferences a NULL framebuffer pointer → re-enters this
    // handler → infinite loop ([abort] EL1 fault unrecoverable spam
    // forever). Land back in serial_shell() instead, which only
    // uses the UART.
    if crate::IS_HEADLESS.load(core::sync::atomic::Ordering::Acquire) {
        crate::serial_shell()
    } else {
        crate::ui::desktop::resume()
    }
}

/// Complete an rt_sigreturn: read the ucontext at the current user SP
/// and restore the trap frame. Called from `sys_rt_sigreturn`.
///
/// The sequence is:
///   1. User handler completes and `ret`s into the restorer (x30 =
///      restorer).
///   2. Restorer executes `svc #139` (mov x8,139 ; svc 0) with SP still
///      pointing at the rt_sigframe we planted.
///   3. Kernel enters this handler, reads the ucontext at SP, and
///      rewrites the trap frame so eret picks up the pre-signal
///      state.
///
/// Returns the value that should be put in x0 after restore. Per
/// Linux, rt_sigreturn doesn't actually return a value — the syscall
/// dispatcher should just rewrite the whole frame. We return
/// `regs[0]` so if the dispatcher stores x0 on top of our restore,
/// the user-visible x0 is still what the ucontext said.
pub fn complete_rt_sigreturn(frame: &mut TrapFrame) -> i64 {
    let sp = unsafe {
        let s: u64;
        core::arch::asm!("mrs {}, sp_el0", out(reg) s);
        s
    };
    // SP on entry should point at the start of the rt_sigframe we
    // installed. Reach across to uc_mcontext.
    let uc_va = sp + core::mem::offset_of!(RtSigframe, uc) as u64;
    if !super::uaccess::is_user_range(uc_va as usize,
        core::mem::size_of::<Ucontext>())
    {
        uart::puts("[sig] rt_sigreturn: ucontext not in user range\n");
        return -14; // EFAULT
    }
    unsafe {
        let uc = &*(uc_va as *const Ucontext);
        frame.x.copy_from_slice(&uc.uc_mcontext.regs);
        frame.elr  = uc.uc_mcontext.pc;
        frame.spsr = uc.uc_mcontext.pstate;
        core::arch::asm!("msr sp_el0, {}", in(reg) uc.uc_mcontext.sp);
    }
    // Return the restored x0 so the syscall dispatcher's f.x[0] =
    // result; line doesn't clobber our restore.
    frame.x[0] as i64
}
