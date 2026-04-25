#![allow(dead_code)]
// Bat_OS — Architecture-specific kernel code (ARM64)

use crate::drivers::uart;

core::arch::global_asm!(include_str!("../../batcave/linux/forkjmp.s"));
core::arch::global_asm!(include_str!("../../batcave/linux/threads.s"));

unsafe extern "C" {
    fn fork_save(buf: *mut u64) -> u64;
    fn fork_restore(buf: *const u64, retval: u64) -> !;
}

// Saved busybox SP at clone time (for eret back to parent)
static mut SAVED_BUSYBOX_SP: u64 = 0;

/// Kernel SP save slot used by `batcave::linux::loader::execute_with_args`
/// before erets to EL0, and read back by the exit-syscall + brk paths below
/// so the shell can resume after a user ELF exits.
///
/// Lives in kernel BSS so it's guaranteed writable (unlike the previous
/// hardcoded `0x40000100`/`0x40001000` addresses which both sat inside the
/// Linux arm64 Image header region and were mapped R-X by the kernel MMU).
/// The addresses also didn't match between the store and restore sites;
/// that was the root cause of the QEMU `DATA ABORT DFSC=0x0e` at FAR
/// 0x40000100 for every BatCave-runner ELF (netsurf/freetype/png/v8/etc).
#[unsafe(no_mangle)]
pub static mut KERNEL_SP_SAVE: u64 = 0;

/// Return the address of `KERNEL_SP_SAVE` as a u64 so inline asm can use it.
#[inline(always)]
pub fn kernel_sp_save_addr() -> u64 {
    &raw const KERNEL_SP_SAVE as u64
}

// Saved exception frame in kernel BSS (safe from busybox)
static mut SAVED_FRAME: [u64; 35] = [0; 35]; // 35 * 8 = 280 bytes > 272

// Saved stack contents at clone time (child shares parent stack — must restore)
const STACK_SAVE_SIZE: usize = 4096;
static mut SAVED_STACK: [u8; STACK_SAVE_SIZE] = [0; STACK_SAVE_SIZE];

#[repr(C)]
pub struct TrapFrame {
    pub x: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
}

pub fn init_exceptions() {
    unsafe {
        core::arch::asm!(
            "adr x0, exception_vectors",
            "msr vbar_el1, x0",
            "isb",
            out("x0") _,
        );

        // V5-SIDE-005 fix: clear CNTKCTL_EL1 so EL0 cannot read the
        // physical/virtual timer registers directly. Without this,
        // `mrs xN, cntpct_el0` from user space returns a 40ns-resolution
        // clock — the universal enabler for every timing side-channel
        // attack (AES S-box cache timing, scalar-mult branch timing,
        // etc.). sys_clock_gettime still provides timing to EL0 via
        // the syscall boundary where we can add noise/resolution caps.
        //
        // Bits in CNTKCTL_EL1:
        //   EL0PCTEN (bit 0) = 1 enables EL0 access to cntpct_el0
        //   EL0VCTEN (bit 1) = 1 enables EL0 access to cntvct_el0
        //   EVNTEN   (bit 2)
        //   EVNTDIR  (bit 3)
        //   EVNTI    (bits 7:4)
        //   EL0VTEN  (bit 8)
        //   EL0PTEN  (bit 9)
        // Setting to 0 denies all EL0 timer register access.
        core::arch::asm!("msr cntkctl_el1, xzr");
        core::arch::asm!("isb");
    }
    uart::puts("  [arch] Exception vectors installed\n");
    uart::puts("  [arch] CNTKCTL_EL1 cleared — EL0 timer access denied\n");
}

/// Minimal GICv2 init for QEMU virt. The "virt" machine wires:
///   GIC Distributor (GICD)  @ 0x0800_0000
///   GIC CPU Interface (GICC)@ 0x0801_0000
/// Physical-timer IRQ is PPI #14 → INTID 30.
///
/// We need: enable the distributor, enable the CPU interface,
/// set PMR (priority mask) to accept all priorities, enable
/// INTID 30 in GICD's ISENABLER. Without this the timer fires
/// in CNTP_CTL but the CPU never sees an IRQ.
fn init_gicv2() {
    const GICD_BASE: usize = 0x0800_0000;
    const GICC_BASE: usize = 0x0801_0000;
    const GICD_CTLR: usize = GICD_BASE + 0x000;
    const GICD_ISENABLER0: usize = GICD_BASE + 0x100;
    const GICD_IPRIORITYR: usize = GICD_BASE + 0x400;
    const GICC_CTLR: usize = GICC_BASE + 0x000;
    const GICC_PMR:  usize = GICC_BASE + 0x004;

    unsafe {
        // Enable distributor (Group 0).
        core::ptr::write_volatile(GICD_CTLR as *mut u32, 1);
        // Lowest priority byte for INTID 30 (timer PPI). Priority
        // 0xa0 — middle of the range so other IRQs (if any) can
        // override.
        let prio_word_addr = (GICD_IPRIORITYR + (30 / 4) * 4) as *mut u32;
        let mut prio = core::ptr::read_volatile(prio_word_addr);
        let lane = (30 % 4) * 8;
        prio &= !(0xFFu32 << lane);
        prio |= 0xA0u32 << lane;
        core::ptr::write_volatile(prio_word_addr, prio);
        // Enable INTID 30 (PPI #14 = physical timer) — bit 30 of
        // GICD_ISENABLER0 (covers IRQs 0..31).
        core::ptr::write_volatile(GICD_ISENABLER0 as *mut u32, 1u32 << 30);
        // Accept all priorities at the CPU interface.
        core::ptr::write_volatile(GICC_PMR  as *mut u32, 0xFF);
        // Enable CPU interface.
        core::ptr::write_volatile(GICC_CTLR as *mut u32, 1);
    }
    uart::puts("  [arch] GICv2 initialized (timer PPI 30 enabled)\n");
}

pub fn init_timer() {
    // Initialize the GIC first so the CPU actually receives the
    // timer IRQ. Without this, CNTP_CTL fires but no IRQ vector
    // is taken — preemption is dead in the water.
    init_gicv2();
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let interval = freq / 100;
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) interval);
        core::arch::asm!("mov x0, #1", "msr cntp_ctl_el0, x0", out("x0") _);
        core::arch::asm!("msr daifclr, #0x2");
    }
    uart::puts("  [arch] Timer configured (100Hz)\n");
}

fn reset_timer() {
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let interval = freq / 100;
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) interval);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_irq(frame: *mut TrapFrame) {
    // V-ASAHI-2.2: on Apple Silicon, IRQs come through AIC2 instead of
    // a per-CPU GIC. We MUST drain the AIC event queue every entry,
    // otherwise level-triggered IRQs (timer, UART, SPI, etc.) re-fire
    // immediately and we livelock in the exception handler.
    if crate::platform::is_apple_silicon() {
        // Drain every pending event (AIC may have queued more than one).
        while crate::drivers::apple::aic::dispatch_one() {}
        return;
    }

    // QEMU virt path: ARM Generic Timer wired directly via the GIC,
    // no indirection. Just check the timer-fired flag.
    let ctl: u64;
    unsafe { core::arch::asm!("mrs {}, cntp_ctl_el0", out(reg) ctl); }
    if ctl & 0b100 != 0 {
        reset_timer();
        crate::kernel::scheduler::tick();

        // CHROMIUM-PHASE-D / REAL PREEMPTION: invoke the threads
        // layer's on_tick(). It snapshots the current thread's
        // user-mode state (from `frame` + live MSRs), picks the
        // next runnable thread, and returns a pointer to that
        // thread's saved regs. We then blit those into the trap
        // frame and update the user MSRs (SP_EL0, TPIDR_EL0,
        // TTBR0_EL1) so eret resumes the new thread.
        //
        // This replaces the old "set request_preempt flag, yield
        // at next syscall boundary" model — workers stuck in user-
        // mode loops with no syscalls would otherwise never get
        // preempted, starving Chromium's worker pool.
        unsafe {
            let frame_ptr = frame as *mut crate::batcave::linux::threads::SavedRegs;
            let new_regs = crate::batcave::linux::threads::on_tick(frame_ptr);
            if let Some(new_ptr) = new_regs {
                // Blit the new thread's user-mode state into the
                // trap frame so eret picks it up.
                let new = &*new_ptr;
                // Frame layout: x[0..31] @ 0..248, elr @ 248, spsr @ 256.
                let tf_x = frame as *mut [u64; 31];
                core::ptr::write(tf_x, new.x);
                let tf_elr = (frame as *mut u8).add(248) as *mut u64;
                core::ptr::write(tf_elr, new.elr_el1);
                let tf_spsr = (frame as *mut u8).add(256) as *mut u64;
                core::ptr::write(tf_spsr, new.spsr_el1);
                // Live MSRs: SP_EL0, TPIDR_EL0, TTBR0_EL1.
                core::arch::asm!("msr sp_el0, {}", in(reg) new.user_sp_el0);
                core::arch::asm!("msr tpidr_el0, {}", in(reg) new.x[18]);
                if new.user_ttbr0 != 0 {
                    let cur_ttbr0: u64;
                    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) cur_ttbr0);
                    if (cur_ttbr0 & !1u64) != new.user_ttbr0 {
                        core::arch::asm!("msr ttbr0_el1, {}", in(reg) new.user_ttbr0);
                        core::arch::asm!("isb");
                        core::arch::asm!("tlbi vmalle1");
                        core::arch::asm!("dsb ish");
                        core::arch::asm!("isb");
                    }
                }
            }
        }
        // Belt-and-suspenders: also set the preempt flag so the
        // syscall-boundary fallback fires if something prevents
        // the IRQ from completing the swap.
        crate::batcave::linux::threads::request_preempt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_exception(frame: *mut TrapFrame) {
    let esr: u64;
    unsafe { core::arch::asm!("mrs {}, esr_el1", out(reg) esr); }
    let ec = (esr >> 26) & 0x3F;


    match ec {
        0x15 => {
            let svc_num = (esr & 0xFFFF) as u16;

            if svc_num == 0 {
                unsafe {
                    let f = &mut *frame;
                    let syscall_num = f.x[8];
                    let args = [f.x[0], f.x[1], f.x[2], f.x[3], f.x[4], f.x[5]];

                    // Record the pre-syscall register state into the
                    // history ring. This runs BEFORE the dispatcher
                    // touches f.x[0] so we capture the caller's LR
                    // (x30) and FP (x29), plus the arguments — which
                    // is exactly the forensic data the UNHANDLED dump
                    // needs when a `ret` eventually lands in the cage.
                    let tid_now = crate::batcave::linux::threads::current_tid();
                    crate::batcave::linux::syscall_history::record(
                        tid_now, syscall_num, &f.x, f.elr,
                    );

                    // EXIT: child exit → eret back to parent at clone return
                    if syscall_num == 93 || syscall_num == 94 {
                        let in_child = crate::batcave::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed);

                        crate::batcave::linux::syscall::handle(0, syscall_num, args);

                        if in_child {
                            let is_thread = crate::batcave::linux::syscall::IS_THREAD_CHILD
                                .load(core::sync::atomic::Ordering::Relaxed);
                            let busybox_sp = core::ptr::read_volatile(
                                core::ptr::addr_of!(SAVED_BUSYBOX_SP));

                            if !is_thread {
                                // Fork-style child: restore parent stack contents
                                // (child corrupted parent's stack by sharing it)
                                let stack_src = core::ptr::addr_of!(SAVED_STACK) as usize;
                                for i in (0..STACK_SAVE_SIZE).step_by(8) {
                                    let val: u64;
                                    core::arch::asm!(
                                        "ldr {v}, [{a}]",
                                        a = in(reg) stack_src + i,
                                        v = out(reg) val,
                                    );
                                    core::arch::asm!(
                                        "str {v}, [{a}]",
                                        a = in(reg) busybox_sp as usize + i,
                                        v = in(reg) val,
                                    );
                                }
                            }
                            // Thread-style child: skip stack restore (child had own stack)

                            // Eret from saved clone frame → parent resumes
                            let saved_ptr = core::ptr::addr_of!(SAVED_FRAME) as u64;
                            // Get the child TID to return to parent
                            let child_tid = crate::batcave::linux::syscall::LAST_CHILD_TID
                                .load(core::sync::atomic::Ordering::Relaxed) as u64;
                            // Restore main thread TID
                            crate::batcave::linux::syscall::restore_parent_tid();

                            core::arch::asm!(
                                // V6-CHAIN-002 fix: write the saved
                                // busybox (user) SP to SP_EL0, not the
                                // current SP_EL1. The eret below uses
                                // SPSR.M to pick which SP becomes
                                // active for EL0; SP_EL1 must NOT be
                                // overwritten with a user-derived
                                // value or every subsequent exception
                                // pushes its trap frame to that
                                // attacker-influenced address.
                                "msr sp_el0, {sp_val}",
                                // x16 = pointer to saved frame data
                                "mov x16, {ptr}",
                                // Restore ELR and SPSR from saved frame
                                "ldp x0, x1, [x16, #248]",
                                "msr elr_el1, x0",
                                // Clear DAIF bits in SPSR so interrupts work after eret
                                "and x1, x1, #0xFFFFFFFFFFFFFC3F",
                                "msr spsr_el1, x1",
                                // Restore all GPRs from saved frame
                                "ldr x1, [x16, #8]",
                                "ldp x2, x3, [x16, #16]",
                                "ldp x4, x5, [x16, #32]",
                                "ldp x6, x7, [x16, #48]",
                                "ldp x8, x9, [x16, #64]",
                                "ldp x10, x11, [x16, #80]",
                                "ldp x12, x13, [x16, #96]",
                                "ldp x14, x15, [x16, #112]",
                                "ldr x17, [x16, #136]",
                                "ldp x18, x19, [x16, #144]",
                                "ldp x20, x21, [x16, #160]",
                                "ldp x22, x23, [x16, #176]",
                                "ldp x24, x25, [x16, #192]",
                                "ldp x26, x27, [x16, #208]",
                                "ldp x28, x29, [x16, #224]",
                                "ldr x30, [x16, #240]",
                                // Load x16 last (destroys our pointer)
                                "ldr x16, [x16, #128]",
                                // x0 = child TID (parent return from clone)
                                "mov x0, {tid}",
                                "eret",
                                ptr = in(reg) saved_ptr,
                                sp_val = in(reg) busybox_sp,
                                tid = in(reg) child_tid,
                                options(noreturn),
                            );
                        }

                        // REAL-FORK: if this exit_group is from a forked
                        // child cave (its TTBR0 differs from the host cave's),
                        // tear down ONLY the child cave and let the
                        // scheduler resume the parent. Otherwise it's
                        // the main process exiting and we go all the
                        // way back to the desktop shell.
                        let cur_ttbr0: u64;
                        core::arch::asm!("mrs {}, ttbr0_el1", out(reg) cur_ttbr0);
                        let cur_ttbr0 = cur_ttbr0 & !1u64;
                        let host_l1 = crate::batcave::linux::mmu::host_cave_l1() as u64;
                        if host_l1 != 0 && cur_ttbr0 != host_l1 {
                            // Forked-child exit. Mark this thread Exited
                            // (parent can wait4 it), then schedule out.
                            // The cooperative-switch asm will activate the
                            // parent's TTBR0 when it picks the parent up.
                            //
                            // (We don't free the child's page tables /
                            // frames yet — wait4 will do that when
                            // implemented. Leak for now; reboot recovers.)
                            crate::batcave::linux::threads::exit_current(
                                args[0] as i32);
                            // exit_current never returns — it schedules
                            // another thread and wfi's if none.
                        }

                        // Real exit (not a forked child — leave BatCave entirely).
                        // V2-NEW-024: DO NOT call mmu::disable here — it
                        // clears SCTLR.M which the next switch_to_cave does
                        // not re-enable, leaving subsequent caves running
                        // with no AP/UXN/PXN enforcement at all. Switch to
                        // primary TTBR0 instead; MMU stays on.
                        crate::batcave::linux::mmu::switch_to_primary();
                        // Restore the kernel SP that the loader stashed before
                        // erets to EL0. See KERNEL_SP_SAVE above.
                        let save_addr = kernel_sp_save_addr();
                        core::arch::asm!(
                            "ldr x0, [{addr}]",
                            "mov sp, x0",
                            addr = in(reg) save_addr,
                            out("x0") _,
                        );
                        crate::ui::desktop::resume();
                    }

                    // CLONE: save exception frame for later parent-resume
                    if syscall_num == 220
                        && !crate::batcave::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed)
                    {
                        let frame_addr = frame as usize;
                        let save_dst = core::ptr::addr_of_mut!(SAVED_FRAME) as usize;
                        // TrapFrame is exactly 33 * 8 = 264 bytes (31 x regs +
                        // ELR + SPSR). The old loop ran for 272 bytes — an
                        // 8-byte overread of the kernel stack into SAVED_FRAME
                        // that got restored to the parent's registers on
                        // child exit. Cap at sizeof(TrapFrame).
                        let tf_size = core::mem::size_of::<TrapFrame>();
                        for i in (0..tf_size).step_by(8) {
                            let val: u64;
                            core::arch::asm!(
                                "ldr {v}, [{a}]",
                                a = in(reg) frame_addr + i,
                                v = out(reg) val,
                            );
                            core::arch::asm!(
                                "str {v}, [{a}]",
                                a = in(reg) save_dst + i,
                                v = in(reg) val,
                            );
                        }
                        core::arch::asm!("dsb sy");
                        // Save busybox SP = frame_addr + 272
                        let sp_val = (frame_addr + 272) as u64;
                        core::ptr::write_volatile(
                            core::ptr::addr_of_mut!(SAVED_BUSYBOX_SP), sp_val,
                        );
                        // Save stack contents above SP (child shares stack)
                        let stack_dst = core::ptr::addr_of_mut!(SAVED_STACK) as usize;
                        for i in (0..STACK_SAVE_SIZE).step_by(8) {
                            let val: u64;
                            core::arch::asm!(
                                "ldr {v}, [{a}]",
                                a = in(reg) sp_val as usize + i,
                                v = out(reg) val,
                            );
                            core::arch::asm!(
                                "str {v}, [{a}]",
                                a = in(reg) stack_dst + i,
                                v = in(reg) val,
                            );
                        }
                    }

                    // EXECVE: if child is calling execve for a busybox applet,
                    // jump to the worker busybox copy for real applet execution
                    if syscall_num == 221
                        && crate::batcave::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed)
                    {
                        let worker_entry = crate::batcave::linux::loader::WORKER_ENTRY
                            .load(core::sync::atomic::Ordering::Relaxed);
                        if worker_entry != 0 {
                            // Read the path to check if it's a busybox applet.
                            // V2-007: gate path_ptr to userspace before ldrb.
                            let path_ptr = f.x[0] as usize;
                            let argv_ptr = f.x[1] as usize;
                            if !crate::batcave::linux::uaccess::is_user_range(path_ptr, 1) {
                                f.x[0] = (-14i64) as u64; // EFAULT
                                return;
                            }
                            let mut path_buf = [0u8; 128];
                            let mut plen = 0usize;
                            for i in 0..127 {
                                if !crate::batcave::linux::uaccess::is_user_range(path_ptr + i, 1) { break; }
                                let b: u32;
                                core::arch::asm!("ldrb {v:w}, [{a}]",
                                    a = in(reg) path_ptr + i, v = out(reg) b);
                                if b == 0 { break; }
                                path_buf[i] = b as u8;
                                plen += 1;
                            }

                            // Debug: log what path execve is trying
                            crate::drivers::uart::puts("[execve] path='");
                            for i in 0..plen.min(60) {
                                crate::drivers::uart::putc(path_buf[i]);
                            }
                            crate::drivers::uart::puts("' len=");
                            crate::kernel::mm::print_num(plen);
                            crate::drivers::uart::puts("\n");

                            // Check for standalone binaries (not busybox applets)
                            let is_hello = (plen == 10 && &path_buf[..10] == b"/bin/hello")
                                || (plen == 5 && &path_buf[..5] == b"hello");

                            if is_hello {
                                // Load the hello ELF binary
                                let hello_data = crate::batcave::linux::runner::hello_elf();
                                match crate::batcave::linux::loader::load_hello_elf(hello_data) {
                                    Ok((phys_entry, _phys_base, _orig_entry)) => {
                                        // Build a minimal stack for the hello binary
                                        let stack_page = crate::kernel::mm::frame::alloc_frame();
                                        if let Some(stack_base) = stack_page {
                                            for _ in 0..15 {
                                                crate::kernel::mm::frame::alloc_frame();
                                            }
                                            let mut sp = stack_base + 16 * 4096;

                                            // Write argv[0] = "hello"
                                            sp -= 6; // "hello\0"
                                            let arg0_addr = sp;
                                            for (j, &b) in b"hello".iter().enumerate() {
                                                core::arch::asm!("strb {v:w}, [{a}]",
                                                    a = in(reg) sp + j,
                                                    v = in(reg) b as u32);
                                            }
                                            core::arch::asm!("strb wzr, [{a}]",
                                                a = in(reg) sp + 5);

                                            // envp string
                                            sp -= 10;
                                            let env0 = sp;
                                            for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
                                                core::arch::asm!("strb {v:w}, [{a}]",
                                                    a = in(reg) sp + j, v = in(reg) b as u32);
                                            }

                                            sp = (sp - 64) & !0xF;

                                            // auxv: AT_NULL
                                            sp -= 16;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp + 8);
                                            // AT_PAGESZ
                                            sp -= 16;
                                            let k6: u64 = 6; let v4096: u64 = 4096;
                                            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k6);
                                            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v4096);

                                            // envp NULL + pointer
                                            sp -= 8;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            sp -= 8;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) env0 as u64);

                                            // argv NULL + pointer
                                            sp -= 8;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            sp -= 8;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) arg0_addr as u64);

                                            // argc = 1
                                            sp -= 8;
                                            let one: u64 = 1;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) one);

                                            // V6-CHAIN-002: enter hello binary at EL0 via
                                            // eret. Writing to SP_EL1 then `br {entry}`
                                            // ran the binary at EL1 with attacker SP —
                                            // every IRQ corrupted the kernel stack.
                                            let entry = phys_entry;
                                            core::arch::asm!(
                                                "msr sp_el0, {sp_val}",
                                                "msr elr_el1, {entry}",
                                                "msr spsr_el1, xzr",  // EL0t, AIF clear
                                                "isb",
                                                "eret",
                                                sp_val = in(reg) sp as u64,
                                                entry = in(reg) entry,
                                                options(noreturn),
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        // Load failed — fall through to ENOENT
                                    }
                                }
                            }

                            // Check if it's in /bin or /usr/bin (busybox applet)
                            let is_bb = plen > 5 && (&path_buf[..5] == b"/bin/"
                                || (plen > 9 && &path_buf[..9] == b"/usr/bin/"));

                            if is_bb {
                                // Re-initialize worker's writable segments
                                // (previous applet run may have corrupted globals)
                                let wbase = crate::batcave::linux::loader::WORKER_PHYS_BASE
                                    .load(core::sync::atomic::Ordering::Relaxed);
                                crate::batcave::linux::loader::reinit_elf(
                                    crate::batcave::linux::runner::busybox_elf(),
                                    wbase,
                                );

                                // Read argv from userspace (up to 8 args).
                                // V2-008: gate argv_ptr (8 × 8-byte pointers)
                                // and each argv[i] string range. Without these,
                                // a cave could pass argv_ptr = 0x40400000 and
                                // have the worker emit 63-byte chunks of
                                // kernel RAM via busybox echo → UART.
                                let _arg_ptrs = [0usize; 8];
                                let mut arg_bufs = [[0u8; 64]; 8];
                                let mut arg_lens = [0usize; 8];
                                let mut argc = 0usize;
                                if argv_ptr != 0
                                    && crate::batcave::linux::uaccess::is_user_range(argv_ptr, 8 * 8)
                                {
                                    for i in 0..8 {
                                        let ap: u64;
                                        core::arch::asm!("ldr {v}, [{a}]",
                                            a = in(reg) argv_ptr + i * 8, v = out(reg) ap);
                                        if ap == 0 { break; }
                                        if !crate::batcave::linux::uaccess::is_user_range(ap as usize, 1) { break; }
                                        for j in 0..63 {
                                            if !crate::batcave::linux::uaccess::is_user_range(ap as usize + j, 1) { break; }
                                            let b: u32;
                                            core::arch::asm!("ldrb {v:w}, [{a}]",
                                                a = in(reg) ap as usize + j, v = out(reg) b);
                                            if b == 0 { break; }
                                            arg_bufs[i][j] = b as u8;
                                            arg_lens[i] = j + 1;
                                        }
                                        argc += 1;
                                    }
                                }

                                // Build a fresh stack for the worker
                                let stack_page = crate::kernel::mm::frame::alloc_frame();
                                if let Some(stack_base) = stack_page {
                                    // Allocate 16 pages for stack
                                    for _ in 0..15 {
                                        crate::kernel::mm::frame::alloc_frame();
                                    }
                                    let mut sp = stack_base + 16 * 4096;

                                    // Write arg strings to stack
                                    let mut str_addrs = [0usize; 8];
                                    for i in (0..argc).rev() {
                                        sp -= arg_lens[i] + 1;
                                        str_addrs[i] = sp;
                                        for j in 0..arg_lens[i] {
                                            core::arch::asm!("strb {v:w}, [{a}]",
                                                a = in(reg) sp + j,
                                                v = in(reg) arg_bufs[i][j] as u32);
                                        }
                                        core::arch::asm!("strb wzr, [{a}]",
                                            a = in(reg) sp + arg_lens[i]);
                                    }

                                    // envp string
                                    sp -= 10;
                                    let env0 = sp;
                                    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
                                        core::arch::asm!("strb {v:w}, [{a}]",
                                            a = in(reg) sp + j, v = in(reg) b as u32);
                                    }

                                    sp = (sp - 64) & !0xF;

                                    // auxv: AT_NULL
                                    sp -= 16;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp + 8);
                                    // AT_PAGESZ
                                    sp -= 16;
                                    let k6: u64 = 6; let v4096: u64 = 4096;
                                    core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k6);
                                    core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v4096);

                                    // envp NULL + pointer
                                    sp -= 8;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    sp -= 8;
                                    core::arch::asm!("str {v}, [{a}]",
                                        a = in(reg) sp, v = in(reg) env0 as u64);

                                    // argv NULL + pointers
                                    sp -= 8;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    for i in (0..argc).rev() {
                                        sp -= 8;
                                        core::arch::asm!("str {v}, [{a}]",
                                            a = in(reg) sp, v = in(reg) str_addrs[i] as u64);
                                    }

                                    // argc
                                    sp -= 8;
                                    core::arch::asm!("str {v}, [{a}]",
                                        a = in(reg) sp, v = in(reg) argc as u64);

                                    // V6-CHAIN-002: enter worker busybox at EL0 via eret.
                                    let entry = worker_entry as u64;
                                    core::arch::asm!(
                                        "msr sp_el0, {sp_val}",
                                        "msr elr_el1, {entry}",
                                        "msr spsr_el1, xzr",
                                        "isb",
                                        "eret",
                                        sp_val = in(reg) sp as u64,
                                        entry = in(reg) entry,
                                        options(noreturn),
                                    );
                                }
                            }
                        }
                    }

                    // For the threading-model clone path, stash the
                    // parent's post-svc return address and saved PSTATE
                    // so threads::set_child_resume can seed the child's
                    // eret-target. ELR_EL1 at SVC entry already points
                    // at the instruction *after* svc, so it's what the
                    // child resumes at.
                    if syscall_num == 220
                        && crate::batcave::linux::threads::is_enabled()
                    {
                        crate::batcave::linux::threads::PARENT_SYSCALL_ELR
                            .store(f.elr, core::sync::atomic::Ordering::Release);
                        crate::batcave::linux::threads::PARENT_SYSCALL_SPSR
                            .store(f.spsr, core::sync::atomic::Ordering::Release);
                        // Snapshot all of the parent's GPRs at svc entry
                        // so set_child_resume can seed the child's full
                        // register state. glibc / musl pthread trampolines
                        // on AArch64 stash fn in x10 and arg in x12 before
                        // svc and expect them to survive into the child's
                        // post-svc code (`blr x10` / `mov x0, x12`). Without
                        // this carry, the child indirect-branch lands at
                        // PC=0.
                        for i in 0..31 {
                            crate::batcave::linux::threads::PARENT_SYSCALL_REGS[i]
                                .store(f.x[i], core::sync::atomic::Ordering::Release);
                        }
                    }

                    // rt_sigreturn (syscall 139) must run against the
                    // trap frame directly — it restores every GPR,
                    // ELR, SPSR, and SP_EL0 from the ucontext the
                    // handler's stack. The regular syscall dispatcher
                    // can't see the frame, so short-circuit it here.
                    let result: i64 = if syscall_num == 139 {
                        let sf = unsafe {
                            &mut *(frame as *mut crate::batcave::linux::signal::TrapFrame)
                        };
                        crate::batcave::linux::signal::complete_rt_sigreturn(sf)
                        // NB: do NOT overwrite f.x[0] below — every
                        // register has just been restored from the
                        // ucontext and the subsequent `f.x[0] = result`
                        // would clobber x0. We short-circuit with a
                        // direct return after the syscall trace.
                    } else {
                        let r = crate::batcave::linux::syscall::handle(0, syscall_num, args);
                        f.x[0] = r as u64;
                        r
                    };
                    // After complete_rt_sigreturn the frame has fresh
                    // contents; any further post-processing (CLONE
                    // child_stack path, EXIT special handling, …)
                    // doesn't apply to rt_sigreturn. Bail out.
                    if syscall_num == 139 {
                        return;
                    }

                    // CLONE with child_stack: jump child to new stack via manual eret
                    if syscall_num == 220 && result == 0 {
                        let child_sp = crate::batcave::linux::syscall::CLONE_CHILD_STACK
                            .load(core::sync::atomic::Ordering::Relaxed);
                        if child_sp != 0 {
                            // Clear the child_stack flag (one-shot)
                            crate::batcave::linux::syscall::CLONE_CHILD_STACK
                                .store(0, core::sync::atomic::Ordering::Relaxed);
                            // Resume the child at the next instruction (after svc)
                            // with SP = child_stack and x0 = 0.
                            // We use x16 as frame pointer (like parent-resume code)
                            // and load x16 itself last from the frame.
                            let frame_ptr = frame as u64;
                            let elr_val = f.elr;
                            let spsr_val = f.spsr;
                            // Ensure 16-byte SP alignment (ARM64 ABI requirement)
                            let child_sp_aligned = child_sp & !0xF;
                            core::arch::asm!(
                                // V6-CHAIN-002 FIX: write the attacker-supplied
                                // child SP into SP_EL0 (the user stack pointer
                                // that eret activates), NOT SP_EL1 (the kernel
                                // SP we're currently using). The previous
                                // `mov sp, {csp}` clobbered the kernel stack
                                // pointer with a value chosen entirely by the
                                // calling cave — every subsequent IRQ/SVC then
                                // pushed its 272-byte trap frame at that
                                // attacker-chosen address. Direct kernel write
                                // primitive on every exception. We use SPSR
                                // configured for EL0t and let eret restore.
                                "msr sp_el0, {csp}",
                                // Set ELR and SPSR for child return
                                "msr elr_el1, {elr}",
                                "msr spsr_el1, {spsr}",
                                // Use only LDR (not LDP) to avoid alignment faults
                                // — trap frame may not be 16-byte aligned
                                "mov x16, {fp}",
                                "ldr x1, [x16, #8]",
                                "ldr x2, [x16, #16]",
                                "ldr x3, [x16, #24]",
                                "ldr x4, [x16, #32]",
                                "ldr x5, [x16, #40]",
                                "ldr x6, [x16, #48]",
                                "ldr x7, [x16, #56]",
                                "ldr x8, [x16, #64]",
                                "ldr x9, [x16, #72]",
                                "ldr x10, [x16, #80]",
                                "ldr x11, [x16, #88]",
                                "ldr x12, [x16, #96]",
                                "ldr x13, [x16, #104]",
                                "ldr x14, [x16, #112]",
                                "ldr x15, [x16, #120]",
                                "ldr x17, [x16, #136]",
                                "ldr x18, [x16, #144]",
                                "ldr x19, [x16, #152]",
                                "ldr x20, [x16, #160]",
                                "ldr x21, [x16, #168]",
                                "ldr x22, [x16, #176]",
                                "ldr x23, [x16, #184]",
                                "ldr x24, [x16, #192]",
                                "ldr x25, [x16, #200]",
                                "ldr x26, [x16, #208]",
                                "ldr x27, [x16, #216]",
                                "ldr x28, [x16, #224]",
                                "ldr x29, [x16, #232]",
                                "ldr x30, [x16, #240]",
                                "ldr x16, [x16, #128]",
                                // x0 = 0 (child return from clone)
                                "mov x0, #0",
                                "eret",
                                elr = in(reg) elr_val,
                                spsr = in(reg) spsr_val,
                                fp = in(reg) frame_ptr,
                                csp = in(reg) child_sp_aligned,
                                options(noreturn),
                            );
                        }
                    }

                    // Async signal poll. If `sys_tgkill` / `sys_kill`
                    // queued a signal on this thread during the
                    // current syscall (or before it — the poll also
                    // picks up anything that accumulated while we
                    // were blocked in `futex_wait` / `ppoll` / etc.),
                    // redirect the trap frame into the registered
                    // user handler on the way back to EL0. On
                    // success the caller's x0 / ELR have been
                    // rewritten to the handler's entry arguments;
                    // on SIG_DFL-with-fatal-default the helper
                    // `terminate_cave_fatal`s instead of returning.
                    //
                    // Skip for rt_sigreturn (syscall 139) — the
                    // frame has just been restored from a ucontext
                    // and polling on top of it would re-raise a
                    // signal we're literally in the middle of
                    // completing.
                    if syscall_num != 139 {
                        let sf = &mut *(frame as *mut crate::batcave::linux::signal::TrapFrame);
                        let _ = crate::batcave::linux::signal::try_deliver_pending(sf);
                    }
                }
            } else {
                unsafe {
                    crate::kernel::syscall::handle(svc_num, &mut *frame);
                }
            }
        }
        0x25 => {
            let far: u64;
            unsafe { core::arch::asm!("mrs {}, far_el1", out(reg) far); }
            let elr = unsafe { (*frame).elr };

            // CHROMIUM-PHASE-D: kernel uaccess (e.g. pipe_buf::write
            // copying from a user iov) can hit a USER VA whose page
            // hasn't been demand-committed yet. The user-side handler
            // for this is EC=0x24, but when the KERNEL is the one
            // touching it the EC is 0x25 (data abort from current EL).
            // Try the lazy-commit path first; if demand_page accepts,
            // retry the faulting instruction.
            if crate::batcave::linux::demand_page::try_handle(far, esr) {
                return;
            }

            let in_code_range = (elr < 0x1400000)
                || (elr >= 0x40000000 && elr < 0x50000000);
            if in_code_range {
                let instr: u32 = unsafe {
                    let val: u32;
                    core::arch::asm!("ldr {v:w}, [{a}]",
                        a = in(reg) elr, v = out(reg) val);
                    val
                };

                // Emulate alignment faults (DFSC=0x21) — HVF enforces strict alignment
                // Use FAR as the exact faulting address, decode instruction for size/direction
                let dfsc = esr & 0x3F;
                if dfsc == 0x21 {
                    unsafe {
                        // ESR ISS fields for data abort tell us load vs store and size
                        let iss = esr & 0x1FFFFFF;
                        let wnr = (iss >> 6) & 1; // 0=read, 1=write
                        let sas = (iss >> 22) & 3; // access size: 0=byte,1=half,2=word,3=dword
                        let srt = (iss >> 16) & 0x1F; // transfer register
                        let isv = (iss >> 24) & 1; // ISV bit — if 1, SAS/SRT are valid

                        if isv == 1 {
                            // ISV valid: use ESR fields (more reliable than decoding instruction)
                            let nbytes = 1u64 << sas;
                            let rt = srt as usize;

                            // V2-002 gate: refuse to proxy kernel-address
                            // accesses for EL0. Before this check, a user
                            // instruction faulting on an unaligned access
                            // to any kernel VA would have us obediently
                            // load/store on its behalf at EL1.
                            let in_user = crate::batcave::linux::uaccess::is_user_range(
                                far as usize, nbytes as usize);
                            if !in_user {
                                // Skip the faulting instruction and deliver
                                // 0/NOP — avoids kernel R/W primitive.
                                (*frame).elr = elr + 4;
                                if wnr == 0 && rt < 31 { (*frame).x[rt] = 0; }
                                return;
                            }

                            if wnr == 0 {
                                // Load: read bytes from FAR
                                let mut val = 0u64;
                                for i in 0..nbytes {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) far.wrapping_add(i), v = out(reg) b);
                                    val |= (b as u64) << (i * 8);
                                }
                                if rt < 31 { (*frame).x[rt] = val; }
                            } else {
                                // Store: write bytes to FAR
                                let val = if rt < 31 { (*frame).x[rt] } else { 0 };
                                for i in 0..nbytes {
                                    let b = ((val >> (i * 8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) far.wrapping_add(i), v = in(reg) b);
                                }
                            }
                            (*frame).elr = elr + 4;
                            return;
                        }

                        // ISV=0: LDP/STP (pair instructions don't set ISV)
                        // Decode instruction manually
                        if (instr & 0x3A000000) == 0x28000000 {
                            let is_64 = (instr >> 31) & 1 == 1;
                            let is_load = (instr >> 22) & 1 == 1;
                            let rt = (instr & 0x1F) as usize;
                            let rt2 = ((instr >> 10) & 0x1F) as usize;
                            let rn = ((instr >> 5) & 0x1F) as usize;
                            let scale: u64 = if is_64 { 8 } else { 4 };
                            // FAR is the exact address the CPU tried to access
                            let addr = far;

                            if is_load {
                                let mut v1 = 0u64; let mut v2 = 0u64;
                                for i in 0..scale {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(i), v = out(reg) b);
                                    v1 |= (b as u64) << (i * 8);
                                }
                                for i in 0..scale {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(scale + i), v = out(reg) b);
                                    v2 |= (b as u64) << (i * 8);
                                }
                                if rt < 31 { (*frame).x[rt] = v1; }
                                if rt2 < 31 { (*frame).x[rt2] = v2; }
                            } else {
                                let v1 = if rt < 31 { (*frame).x[rt] } else { 0 };
                                let v2 = if rt2 < 31 { (*frame).x[rt2] } else { 0 };
                                for i in 0..scale {
                                    let b = ((v1 >> (i*8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(i), v = in(reg) b);
                                }
                                for i in 0..scale {
                                    let b = ((v2 >> (i*8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(scale + i), v = in(reg) b);
                                }
                            }
                            // Handle pre/post index writeback
                            let wb = (instr >> 23) & 3;
                            if wb == 0b01 || wb == 0b11 {
                                let imm7 = ((instr >> 15) & 0x7F) as i32;
                                let simm = if imm7 & 0x40 != 0 { imm7 | !0x7F } else { imm7 };
                                let offset = simm as i64 * scale as i64;
                                // For pre-index: FAR = base + offset, so base = FAR - offset
                                // For post-index: FAR = base, new_base = base + offset
                                if wb == 0b11 {
                                    // Pre-index: writeback = FAR (which is base+offset already)
                                    if rn < 31 { (*frame).x[rn] = addr; }
                                    // rn=31 = SP: can't update SP from trap frame, skip
                                } else {
                                    // Post-index: writeback = base + offset = FAR + offset
                                    let new_val = (addr as i64 + offset) as u64;
                                    if rn < 31 { (*frame).x[rn] = new_val; }
                                }
                            }
                            (*frame).elr = elr + 4;
                            return;
                        }

                        // Fallback: skip instruction
                        (*frame).elr = elr + 4;
                    }
                    return;
                }

                // Emulate atomic load/store exclusive (HVF doesn't support)
                // Single-core → always succeeds, safe to emulate with plain ops
                if (instr & 0x3F000000) == 0x08000000 {
                    let size = (instr >> 30) & 3;
                    let o2 = (instr >> 23) & 1;
                    let is_load = (instr >> 22) & 1;
                    let o1 = (instr >> 21) & 1;
                    let rs = ((instr >> 16) & 0x1F) as usize;
                    let rn = ((instr >> 5) & 0x1F) as usize;
                    let rt = (instr & 0x1F) as usize;

                    unsafe {
                        let f = &mut *frame;
                        let addr = if rn < 31 { f.x[rn] } else { 0 } as usize;

                        if o1 == 0 && o2 == 0 {
                            // LDXR/LDAXR or STXR/STLXR
                            if is_load == 1 {
                                let val = emulate_load(addr, size);
                                if rt < 31 { f.x[rt] = val; }
                            } else {
                                let val = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, val);
                                if rs < 31 { f.x[rs] = 0; }
                            }
                        } else if o1 == 1 && o2 == 1 {
                            // CAS — compare and swap
                            let old = emulate_load(addr, size);
                            let cmp = if rs < 31 { f.x[rs] } else { 0 };
                            let mask: u64 = match size { 0=>0xFF, 1=>0xFFFF, 2=>0xFFFFFFFF, _=>u64::MAX };
                            if (old & mask) == (cmp & mask) {
                                let nv = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, nv);
                            }
                            if rs < 31 { f.x[rs] = old; }
                        } else if o1 == 0 && o2 == 1 {
                            // LDAR/STLR — acquire/release
                            if is_load == 1 {
                                let val = emulate_load(addr, size);
                                if rt < 31 { f.x[rt] = val; }
                            } else {
                                let val = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, val);
                            }
                        } else {
                            // SWP or other atomic — swap
                            let old = emulate_load(addr, size);
                            let nv = if rs < 31 { f.x[rs] } else { 0 };
                            emulate_store(addr, size, nv);
                            if rt < 31 { f.x[rt] = old; }
                        }
                        f.elr = elr + 4;
                    }
                    return;
                }

                // Cache maintenance ops (DC/IC) — skip
                if (instr & 0xFFF80000) == 0xD5080000 {
                    unsafe { (*frame).elr = elr + 4; }
                    return;
                }
            }

            // Log detailed fault info
            let dfsc = esr & 0x3F; // Data Fault Status Code
            uart::puts("!!! DATA ABORT (DFSC=0x");
            let hex = b"0123456789abcdef";
            uart::putc(hex[((dfsc >> 4) & 0xf) as usize]);
            uart::putc(hex[(dfsc & 0xf) as usize]);
            uart::puts(") !!!\n");
            uart::puts("  FAR: 0x"); print_hex(far);
            uart::puts("  ELR: 0x"); print_hex(elr);
            // Log TTBR0 to see which page table is active
            let ttbr0: u64;
            unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
            uart::puts("  TTBR0: 0x"); print_hex(ttbr0);
            uart::puts("\n");
            // Skip instruction to prevent infinite loop (first time only)
            static mut ABORT_COUNT: u32 = 0;
            unsafe {
                ABORT_COUNT += 1;
                if ABORT_COUNT > 3 {
                    uart::puts("[abort] too many — halting binary\n");
                    // Return to shell by setting ELR to a known-good address
                    ABORT_COUNT = 0;
                    return; // just return, the shell is already gone (noreturn)
                }
            }
        }
        0x3C => {
            let elr = unsafe { (*frame).elr };
            let in_child = crate::batcave::linux::syscall::IN_CHILD
                .load(core::sync::atomic::Ordering::Relaxed);

            // If abort/brk from busybox code range, skip the instruction
            // (worker cleanup, musl assertions, etc. — non-fatal)
            let in_code = (elr < 0x1400000)
                || (elr >= 0x40000000 && elr < 0x50000000);
            if in_code && !in_child {
                // Worker or busybox cleanup BRK — just skip it
                unsafe { (*frame).elr = elr + 4; }
                return;
            }

            if in_child {
                uart::puts("[linux] child abort — returning to parent\n");
                crate::batcave::linux::syscall::IN_CHILD
                    .store(false, core::sync::atomic::Ordering::Relaxed);
                unsafe {
                    let busybox_sp = core::ptr::read_volatile(
                        core::ptr::addr_of!(SAVED_BUSYBOX_SP));
                    let stack_src = core::ptr::addr_of!(SAVED_STACK) as usize;
                    for i in (0..STACK_SAVE_SIZE).step_by(8) {
                        let val: u64;
                        core::arch::asm!("ldr {v}, [{a}]", a = in(reg) stack_src + i, v = out(reg) val);
                        core::arch::asm!("str {v}, [{a}]", a = in(reg) busybox_sp as usize + i, v = in(reg) val);
                    }

                    let saved_ptr = core::ptr::addr_of!(SAVED_FRAME) as u64;
                    core::arch::asm!(
                        // V6-CHAIN-002: SP_EL0 not SP_EL1.
                        "msr sp_el0, {sp_val}",
                        "mov x16, {ptr}",
                        "ldp x0, x1, [x16, #248]",
                        "msr elr_el1, x0",
                        "and x1, x1, #0xFFFFFFFFFFFFFC3F",
                        "msr spsr_el1, x1",
                        "ldr x1, [x16, #8]",
                        "ldp x2, x3, [x16, #16]",
                        "ldp x4, x5, [x16, #32]",
                        "ldp x6, x7, [x16, #48]",
                        "ldp x8, x9, [x16, #64]",
                        "ldp x10, x11, [x16, #80]",
                        "ldp x12, x13, [x16, #96]",
                        "ldp x14, x15, [x16, #112]",
                        "ldr x17, [x16, #136]",
                        "ldp x18, x19, [x16, #144]",
                        "ldp x20, x21, [x16, #160]",
                        "ldp x22, x23, [x16, #176]",
                        "ldp x24, x25, [x16, #192]",
                        "ldp x26, x27, [x16, #208]",
                        "ldp x28, x29, [x16, #224]",
                        "ldr x30, [x16, #240]",
                        "ldr x16, [x16, #128]",
                        "mov x0, #2",
                        "eret",
                        ptr = in(reg) saved_ptr,
                        sp_val = in(reg) busybox_sp,
                        options(noreturn),
                    );
                }
            }

            // Only reach here for non-busybox BRK (real shell exit).
            // V2-NEW-024: switch TTBR0 back to primary instead of disabling
            // the MMU so subsequent caves keep W^X / UXN / PXN protection.
            uart::puts("[linux] exit — returning to desktop\n");
            unsafe {
                crate::batcave::linux::mmu::switch_to_primary();
                // Restore the kernel SP that the loader stashed before
                // erets to EL0. See KERNEL_SP_SAVE above.
                let save_addr = kernel_sp_save_addr();
                core::arch::asm!(
                    "ldr x0, [{addr}]",
                    "mov sp, x0",
                    addr = in(reg) save_addr,
                    out("x0") _,
                );
                crate::ui::desktop::resume();
            }
        }
        0x19 | 0x1c | 0x1d => {
            // 0x19 = SVE functionality trapped,
            // 0x1c = FPAC (pointer-authentication failure),
            // 0x1d = SME functionality trapped.
            // When user control-flow strays into data (V8 cage, string
            // literals, etc.) the CPU decoder often matches the random
            // bytes against one of these encodings and traps, instead
            // of the plain EC=0 "unknown instruction" that would make
            // the symptom obvious. Skip the faulting word so the next
            // truly-invalid fetch surfaces with a cleaner diagnostic.
            let elr_raw = unsafe { (*frame).elr };
            let elr = elr_raw & 0x00FF_FFFF_FFFF_FFFF;
            unsafe { (*frame).elr = elr + 4; }
            static SVE_PAC_SME_SKIPS: core::sync::atomic::AtomicU64 =
                core::sync::atomic::AtomicU64::new(0);
            let n = SVE_PAC_SME_SKIPS.fetch_add(
                1, core::sync::atomic::Ordering::Relaxed);
            if n < 4 || (n & 0xFFFF) == 0 {
                uart::puts("[sve/pac/sme-skip] ec=0x");
                let hex = b"0123456789abcdef";
                uart::putc(hex[((ec >> 4) & 0xF) as usize]);
                uart::putc(hex[(ec & 0xF) as usize]);
                uart::puts(" ELR=0x"); print_hex(elr);
                uart::puts(" n="); crate::kernel::mm::print_num(n as usize);
                uart::puts("\n");
            }
            return;
        }
        0x00 => {
            // Unknown/undefined instruction — might be HVF-unsupported atomics
            // (LDADD, LDSET, LDCLR, SWP, etc. at encoding 0x38/0xB8/0xF8)
            // Strip TBI tag so tagged-pointer user code still matches the
            // "in code" range. Our TCR_EL1 has TBI0=1, so the CPU ignores
            // bits 63:56 during translation but reports them in ELR.
            let elr_raw = unsafe { (*frame).elr };
            let elr = elr_raw & 0x00FF_FFFF_FFFF_FFFF;
            // Accept any ELR that lives inside our 39-bit user VA window
            // (< 2^39). The previous check pinned it to busybox ranges
            // and missed V8 JIT trampolines in the pointer-compression
            // cage (0x30_0000_0000..0x38_0000_0000) and friends.
            let in_code = elr < (1u64 << 39);
            if in_code {
                let instr: u32 = unsafe {
                    let val: u32;
                    core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) elr, v = out(reg) val);
                    val
                };

                // Atomic memory ops: size[31:30] 111 V[26] 00 A[23] R[22] 1 Rs[20:16] o3[15] opc[14:12] 00 Rn[9:5] Rt[4:0]
                // LDADD/STADD: opc=000
                // LDSET/STSET: opc=011
                // LDCLR/STCLR: opc=001
                // LDEOR/STEOR: opc=010
                // SWP: opc=100 (bit pattern 111000 with different prefix)
                let top6 = (instr >> 26) & 0x3F;
                if top6 == 0x38 || top6 == 0x39 || top6 == 0x3C || top6 == 0x3D
                    || top6 == 0x3E || top6 == 0x3F || top6 == 0x2E || top6 == 0x2F
                {
                    // Atomic memory operation — emulate
                    let size = (instr >> 30) & 3;
                    let _a_bit = (instr >> 23) & 1;
                    let _r_bit = (instr >> 22) & 1;
                    let rs = ((instr >> 16) & 0x1F) as usize;
                    let opc = (instr >> 12) & 7;
                    let rn = ((instr >> 5) & 0x1F) as usize;
                    let rt = (instr & 0x1F) as usize;

                    unsafe {
                        let f = &mut *frame;
                        let addr = if rn < 31 { f.x[rn] } else { 0 } as usize;
                        let rs_val = if rs < 31 { f.x[rs] } else { 0 };
                        let old = emulate_load(addr, size);

                        let new_val = match opc {
                            0 => old.wrapping_add(rs_val), // LDADD
                            1 => old & !rs_val,            // LDCLR
                            2 => old ^ rs_val,             // LDEOR
                            3 => old | rs_val,             // LDSET
                            4 => rs_val,                   // SWP
                            _ => old,                      // unknown — nop
                        };
                        emulate_store(addr, size, new_val);
                        if rt < 31 { f.x[rt] = old; }
                        f.elr = elr + 4;
                    }
                    return;
                }

                // UDF / unknown instruction that isn't an atomic we can
                // emulate. V8 / WASM / ASan etc. all emit `udf #X` as a
                // deliberate trap sentinel and rely on SIGILL delivery
                // to a registered handler for recovery. Route through
                // the signal layer first; only if no handler is
                // installed do we fall back to the legacy "silently
                // skip" behaviour (kept so the busybox cleanup path
                // still works).
                let sf = unsafe {
                    &mut *(frame as *mut crate::batcave::linux::signal::TrapFrame)
                };
                if crate::batcave::linux::signal::try_deliver_synchronous(
                    sf,
                    crate::batcave::linux::signal::SIGILL,
                    crate::batcave::linux::signal::ILL_ILLOPC,
                    elr,
                ) {
                    return;
                }
                // Other unknown instr in busybox — skip
                unsafe { (*frame).elr = elr + 4; }
                return;
            }
            uart::puts("!!! UNHANDLED EC=0 !!!\n");
            uart::puts("  tid=t");
            crate::kernel::mm::print_num(
                crate::batcave::linux::threads::current_tid() as usize);
            uart::puts(" ELR: 0x"); print_hex(elr);
            let ttbr0: u64; let sctlr: u64; let far: u64;
            unsafe {
                core::arch::asm!("mrs {}, ttbr0_el1",  out(reg) ttbr0);
                core::arch::asm!("mrs {}, sctlr_el1",  out(reg) sctlr);
                core::arch::asm!("mrs {}, far_el1",    out(reg) far);
            }
            uart::puts("  ESR_full=0x"); print_hex(esr);
            uart::puts("  TTBR0=0x"); print_hex(ttbr0);
            uart::puts("  FAR=0x"); print_hex(far);
            uart::puts("\n");
            // Look up the L2_low entry for ELR and read phys bytes directly.
            let l1_phys = ttbr0 & !1u64;
            let l2_low = unsafe {
                core::ptr::read_volatile((l1_phys) as *const u64)
            };
            let l2_low_phys = l2_low & 0x0000_FFFF_FFFF_F000;
            let l2_idx = (elr >> 21) & 0x1FF;
            let l2_entry = unsafe {
                core::ptr::read_volatile(
                    (l2_low_phys + l2_idx * 8) as *const u64)
            };
            let mapped_phys_block = l2_entry & 0x0000_FFFF_FFE0_0000; // 2 MB aligned
            let offset_in_block = elr & 0x1F_FFFF;
            let direct_phys = mapped_phys_block + offset_in_block;
            let direct_word: u32 = unsafe {
                core::ptr::read_volatile(direct_phys as *const u32)
            };
            uart::puts("  l1[0]=0x"); print_hex(l2_low);
            uart::puts("  l2[");
            crate::kernel::mm::print_num(l2_idx as usize);
            uart::puts("]=0x"); print_hex(l2_entry);
            uart::puts("\n  direct_phys=0x"); print_hex(direct_phys);
            uart::puts("  bytes_there=0x"); print_hex(direct_word as u64);
            uart::puts("\n");
            // Two ways to read insn at ELR — asm vs volatile — to
            // cross-check whether we're really reading what we think.
            // Dump 6 instructions around ELR — helps see what computed
            // the bad pointer that the faulting LDR then dereferenced.
            uart::puts("  code around ELR:");
            for off in [-12i64, -8, -4, 0, 4, 8].iter() {
                let addr = (elr as i64 + off) as usize;
                let word: u32 = unsafe { core::ptr::read_volatile(addr as *const u32) };
                uart::puts("\n    ["); print_hex(addr as u64);
                uart::puts("] 0x"); print_hex(word as u64);
            }
            // CHROMIUM-PHASE-B: dump general-purpose registers so we can
            // tell where a jump to a zeroed/unmapped page came from. LR
            // in particular tells us the caller — when an EC=0 UDF fires
            // inside libc/ld-linux, the caller PC is almost always the
            // key clue (what computed the bad target).
            unsafe {
                let f = &*frame;
                uart::puts("\n  x0 =0x"); print_hex(f.x[0]);
                uart::puts("  x1 =0x"); print_hex(f.x[1]);
                uart::puts("  x2 =0x"); print_hex(f.x[2]);
                uart::puts("\n  x3 =0x"); print_hex(f.x[3]);
                uart::puts("  x4 =0x"); print_hex(f.x[4]);
                uart::puts("  x5 =0x"); print_hex(f.x[5]);
                uart::puts("\n  x16=0x"); print_hex(f.x[16]);
                uart::puts("  x17=0x"); print_hex(f.x[17]);
                uart::puts("  x18=0x"); print_hex(f.x[18]);
                uart::puts("\n  x19=0x"); print_hex(f.x[19]);
                uart::puts("  x20=0x"); print_hex(f.x[20]);
                uart::puts("  x21=0x"); print_hex(f.x[21]);
                uart::puts("\n  x29=0x"); print_hex(f.x[29]);
                uart::puts("  x30(LR)=0x"); print_hex(f.x[30]);
                uart::puts("\n");
                // Dump the 4 instructions before LR — that tells us what
                // the caller intended. If LR points right after a BLR xN
                // then xN held the bad target; if after a BL #imm, the
                // jump was direct (and therefore a reloc/relink issue).
                if f.x[30] >= 16 {
                    uart::puts("  code around LR:");
                    for off in [-12i64, -8, -4, 0].iter() {
                        let addr = (f.x[30] as i64 + off) as usize;
                        let word: u32 = core::ptr::read_volatile(
                            addr as *const u32);
                        uart::puts("\n    ["); print_hex(addr as u64);
                        uart::puts("] 0x"); print_hex(word as u64);
                    }
                    uart::puts("\n");
                }
            }
            uart::puts("  ISS=0x");  print_hex(esr & 0x01FF_FFFF);
            uart::puts("\n");
            loop { unsafe { core::arch::asm!("wfe") }; }
        }
        _ => {
            // Demand-paging: EC=0x24 (data abort from lower EL) may
            // be a legitimate lazy-commit for a huge mmap reservation.
            // Ask `demand_page::try_handle` first — if it commits a
            // page, just return so eret retries the faulting insn.
            let far: u64;
            unsafe { core::arch::asm!("mrs {}, far_el1", out(reg) far); }
            // EC=0x24 = data abort from lower EL (user touched
            // uncommitted page). EC=0x25 = data abort from current
            // EL — happens when the kernel reads/writes user memory
            // (uaccess) into a not-yet-committed page. Both can be
            // a legitimate lazy commit. Try to back the page; if
            // demand_page accepts, retry the instruction by returning.
            if (ec == 0x24 || ec == 0x25)
                && crate::batcave::linux::demand_page::try_handle(far, esr)
            {
                return;
            }

            // Synchronous faults from a lower EL that we can't
            // service transparently: try to deliver them as a POSIX
            // signal so user-registered handlers (V8's WASM trap
            // handler, libc assertions, etc.) get a chance to
            // recover. EC → signal mapping:
            //   0x20 / 0x21 (instruction abort)     → SIGSEGV
            //   0x22       (PC alignment)           → SIGBUS
            //   0x24 / 0x25 (data abort)            → SIGSEGV
            //   0x26       (SP alignment)           → SIGBUS
            //   0x2C / 0x2D (FP trap)               → SIGFPE
            // Everything else falls through to the UNHANDLED dump.
            {
                let (signo, si_code): (u32, i32) = match ec {
                    0x20 | 0x21 => {
                        // Instruction abort: ISS DFSC bits 5:0. 0b0001xx
                        // = translation fault, 0b0011xx = access flag,
                        // 0b0111xx = permission. MAPERR vs ACCERR.
                        let iss = (esr & 0x3F) as u32;
                        let si_code = if (iss >> 2) == 0b0011
                            || (iss >> 2) == 0b0001
                        {
                            crate::batcave::linux::signal::SEGV_MAPERR
                        } else {
                            crate::batcave::linux::signal::SEGV_ACCERR
                        };
                        (crate::batcave::linux::signal::SIGSEGV, si_code)
                    }
                    0x22 => (
                        crate::batcave::linux::signal::SIGBUS,
                        crate::batcave::linux::signal::BUS_ADRALN,
                    ),
                    0x24 | 0x25 => {
                        let iss = (esr & 0x3F) as u32;
                        let si_code = if (iss >> 2) == 0b0011
                            || (iss >> 2) == 0b0001
                        {
                            crate::batcave::linux::signal::SEGV_MAPERR
                        } else {
                            crate::batcave::linux::signal::SEGV_ACCERR
                        };
                        (crate::batcave::linux::signal::SIGSEGV, si_code)
                    }
                    0x26 => (
                        crate::batcave::linux::signal::SIGBUS,
                        crate::batcave::linux::signal::BUS_ADRALN,
                    ),
                    _ => (0, 0),
                };
                if signo != 0 {
                    let sf = unsafe {
                        &mut *(frame as *mut crate::batcave::linux::signal::TrapFrame)
                    };
                    if crate::batcave::linux::signal::try_deliver_synchronous(
                        sf, signo, si_code, far,
                    ) {
                        return;
                    }
                }
            }

            uart::puts("!!! UNHANDLED SYNC EXCEPTION !!!\n");
            uart::puts("  tid=t");
            crate::kernel::mm::print_num(
                crate::batcave::linux::threads::current_tid() as usize);
            uart::puts("\n");
            uart::puts("  EC: 0x"); print_hex(ec);
            uart::puts("  ISS: 0x"); print_hex(esr & 0x01FF_FFFF);
            uart::puts("\n");
            // Extra sanity: read TTBR0 + SCTLR to make sure the EL1
            // context is what we think.
            {
                let ttbr0: u64; let sctlr: u64;
                unsafe {
                    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0);
                    core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
                }
                uart::puts("  TTBR0: 0x"); print_hex(ttbr0);
                uart::puts("  SCTLR: 0x"); print_hex(sctlr);
                uart::puts("\n");
            }
            let sp_el0: u64; let tp: u64;
            unsafe {
                core::arch::asm!("mrs {}, sp_el0",     out(reg) sp_el0);
                core::arch::asm!("mrs {}, tpidr_el0",  out(reg) tp);
            }
            let elr = unsafe { (*frame).elr };
            uart::puts("  ELR: 0x"); print_hex(elr);
            uart::puts("  FAR: 0x"); print_hex(far);
            uart::puts("  SP:  0x"); print_hex(sp_el0);
            uart::puts("  TP:  0x"); print_hex(tp);
            uart::puts("\n");
            // Dump 6 instructions around ELR + x9..x28 so we can see
            // where the bad pointer argument came from.
            uart::puts("  code around ELR:");
            for off in [-16i64, -12, -8, -4, 0, 4, 8].iter() {
                let addr = (elr as i64 + off) as usize;
                let word: u32 = unsafe { core::ptr::read_volatile(addr as *const u32) };
                uart::puts("\n    ["); print_hex(addr as u64);
                uart::puts("] 0x"); print_hex(word as u64);
            }
            uart::puts("\n");
            // x0..x8 dump — x8 is syscall number on ARM64; x0..x7 are
            // either syscall args or scratch. Useful to tell a corrupt
            // function-pointer call from an almost-working syscall.
            unsafe {
                // x0..x28 so we can trace x21/x26 (TLS ptr chain) and
                // anything else the faulting instruction needed.
                for i in 0..30 {
                    if i > 0 && i % 3 == 0 { uart::puts("\n"); }
                    uart::puts("  x");
                    if i < 10 {
                        uart::putc(b'0' + i as u8);
                        uart::puts(" ");
                    } else {
                        uart::putc(b'0' + (i / 10) as u8);
                        uart::putc(b'0' + (i % 10) as u8);
                    }
                    uart::puts("=0x"); print_hex((*frame).x[i]);
                }
                uart::puts("\n  LR(x30)=0x"); print_hex((*frame).x[30]);
                uart::puts("\n");
                // Scan the user stack for plausible saved-LR slots
                // (values whose `v-4` decodes as a BL/BLR). Useful for
                // tracing the caller of a function that ret'd to a bad
                // address — the fp-chain can't be trusted if x29 is
                // already corrupted.
                uart::puts("  stack LR candidates (SP+0 .. SP+0x200):");
                let sp_val = sp_el0;
                for i in 0..64usize {
                    let addr = sp_val + (i as u64) * 8;
                    let v: u64 = core::ptr::read_volatile(addr as *const u64);
                    if v >= 0x10000000 && v < 0x1f000000 && (v & 3) == 0 {
                        let pc = v.wrapping_sub(4);
                        let ins: u32 = core::ptr::read_volatile(pc as *const u32);
                        let top6 = (ins >> 26) & 0x3F;
                        let is_bl = top6 == 0x25;
                        let is_blr = (ins & 0xFFFE0000) == 0xD63F0000;
                        if is_bl || is_blr {
                            uart::puts("\n    [sp+0x");
                            let off = i * 8;
                            uart::putc(b"0123456789abcdef"[(off >> 8) & 0xF]);
                            uart::putc(b"0123456789abcdef"[(off >> 4) & 0xF]);
                            uart::putc(b"0123456789abcdef"[off & 0xF]);
                            uart::puts("]=0x"); print_hex(v);
                            uart::puts(if is_bl { " BL" } else { " BLR" });
                        }
                    }
                }
                uart::puts("\n");
                // Dump 4 instructions before LR so we can tell what the
                // function's call site looked like. LR points at the
                // insn AFTER the BL/BLR, so [-4] is the actual jump.
                if (*frame).x[30] >= 16 {
                    uart::puts("  code around LR:");
                    for off in [-16i64, -12, -8, -4, 0].iter() {
                        let addr = ((*frame).x[30] as i64 + off) as usize;
                        let word: u32 = core::ptr::read_volatile(
                            addr as *const u32);
                        uart::puts("\n    ["); print_hex(addr as u64);
                        uart::puts("] 0x"); print_hex(word as u64);
                    }
                    uart::puts("\n");
                }
                // CHROMIUM-PHASE-B: dump 64 bytes of the object at x19.
                // x19 is callee-saved in the AArch64 AAPCS, so functions
                // often use it to hold `this`. When a crash happens
                // inside a method, dumping [x19..x19+64] reveals the
                // object's state and is more informative than just
                // the register file. Also dump [x19-32..x19] for context.
                if (*frame).x[19] > 0x1000
                    && (*frame).x[19] < 0x0000_4000_0000_0000
                {
                    uart::puts("  memory around x19 (32 before + 64 after):");
                    let obj = (*frame).x[19] as usize;
                    for i in -4i64..8i64 {
                        uart::puts("\n    ");
                        if i < 0 { uart::putc(b'-'); } else { uart::putc(b'+'); }
                        let off_abs = (i * 8).unsigned_abs() as usize;
                        uart::puts("0x");
                        uart::putc(b"0123456789abcdef"[(off_abs >> 4) & 0xF]);
                        uart::putc(b"0123456789abcdef"[off_abs & 0xF]);
                        uart::puts(": ");
                        let row_base = (obj as i64 + i * 8) as usize;
                        for j in 0..8usize {
                            let byte: u8 = core::ptr::read_volatile(
                                (row_base + j) as *const u8);
                            uart::putc(b"0123456789abcdef"[(byte >> 4) as usize]);
                            uart::putc(b"0123456789abcdef"[(byte & 0xF) as usize]);
                            uart::putc(b' ');
                        }
                        uart::puts(" | ");
                        for j in 0..8usize {
                            let byte: u8 = core::ptr::read_volatile(
                                (row_base + j) as *const u8);
                            if (0x20..=0x7e).contains(&byte) { uart::putc(byte); }
                            else { uart::putc(b'.'); }
                        }
                    }
                    uart::puts("\n");
                }
                // Also dump the user stack — finds the call chain when
                // a function faults mid-execution. SP points at the
                // top of the active stack frame; walking up lets us
                // see return addresses.
                if sp_el0 > 0x1000 && sp_el0 < 0x0000_4000_0000_0000 {
                    uart::puts("  user stack around SP (0x");
                    print_hex(sp_el0);
                    uart::puts("):");
                    // Dump SP-0x80..SP+0x100: covers the just-popped
                    // frame (negative offsets) plus a few caller
                    // frames. Each row = 4 u64s = 32 bytes.
                    let base = (sp_el0 as i64 - 0x80) as u64;
                    for i in 0..48usize {
                        let off = i * 8;
                        let addr = base + off as u64;
                        let signed_off = (addr as i64) - (sp_el0 as i64);
                        if off % 32 == 0 {
                            uart::puts("\n    ");
                            if signed_off < 0 {
                                uart::puts("-0x");
                                let v = (-signed_off) as u64;
                                uart::putc(b"0123456789abcdef"[((v >> 8) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[((v >> 4) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[(v & 0xF) as usize]);
                            } else {
                                uart::puts("+0x");
                                let v = signed_off as u64;
                                uart::putc(b"0123456789abcdef"[((v >> 8) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[((v >> 4) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[(v & 0xF) as usize]);
                            }
                            uart::puts(":");
                        } else {
                            uart::puts(" ");
                        }
                        let qword: u64 = core::ptr::read_volatile(addr as *const u64);
                        uart::puts("0x");
                        for sh in (0..16).rev() {
                            uart::putc(b"0123456789abcdef"[((qword >> (sh*4)) & 0xF) as usize]);
                        }
                    }
                    uart::puts("\n");
                }
            }
            // Dump the syscall-history ring so we can correlate the
            // fault with the last few svc calls — invaluable for
            // tracking how x29 / x30 got populated with a cage
            // pointer before the crashing `ret`.
            crate::batcave::linux::syscall_history::dump();

            // If the fault came from EL0 (SPSR.M[3:0] == 0b0000 = EL0t)
            // and the EC maps to a synchronous-fault signal whose
            // default disposition is terminate, we can give Chromium /
            // the test harness a soft landing: tear the cave down and
            // drop back into the shell instead of wedging the whole
            // kernel on `wfe`. Real EL1-origin faults (genuine kernel
            // bugs) still `wfe` so the operator can investigate.
            let spsr_m = unsafe { (*frame).spsr & 0xF };
            let from_el0 = spsr_m == 0;
            let fatal_signo: u32 = match ec {
                0x20 | 0x21 | 0x24 | 0x25 => {
                    crate::batcave::linux::signal::SIGSEGV
                }
                0x22 | 0x26 => {
                    crate::batcave::linux::signal::SIGBUS
                }
                _ => 0,
            };
            if from_el0 && fatal_signo != 0 {
                let far_now: u64;
                unsafe {
                    core::arch::asm!("mrs {}, far_el1", out(reg) far_now);
                }
                crate::batcave::linux::signal::terminate_cave_fatal(
                    fatal_signo, far_now,
                );
                // never returns
            }
            loop { unsafe { core::arch::asm!("wfe") }; }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_unhandled_exception(_frame: *mut TrapFrame) {
    uart::puts("!!! UNHANDLED EXCEPTION !!!\n");
    loop { unsafe { core::arch::asm!("wfe") }; }
}

// Emulate load for atomic instruction emulation (HVF workaround)
// V2-002/003/004 gate: only emulate accesses that target user space.
// Before this gate, a guest could craft an unaligned / LDXR / atomic
// instruction with addr=<any kernel address> and the emulator would
// faithfully load/store on its behalf — arbitrary EL1 R/W primitive.
fn emul_addr_ok(addr: usize, nbytes: usize) -> bool {
    crate::batcave::linux::uaccess::is_user_range(addr, nbytes)
}

unsafe fn emulate_load(addr: usize, size: u32) -> u64 {
    let nbytes = 1usize << (size as usize);
    if !emul_addr_ok(addr, nbytes) { return 0; } // safe-zero on bad addr
    unsafe {
        match size {
            0 => { let v: u32; core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            1 => { let v: u32; core::arch::asm!("ldrh {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            2 => { let v: u32; core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            _ => { let v: u64; core::arch::asm!("ldr {v}, [{a}]", a = in(reg) addr, v = out(reg) v); v }
        }
    }
}

// Emulate store for atomic instruction emulation (HVF workaround)
unsafe fn emulate_store(addr: usize, size: u32, val: u64) {
    let nbytes = 1usize << (size as usize);
    if !emul_addr_ok(addr, nbytes) { return; } // silently drop kernel-target writes
    unsafe {
        match size {
            0 => core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            1 => core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            2 => core::arch::asm!("str {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            _ => core::arch::asm!("str {v}, [{a}]", a = in(reg) addr, v = in(reg) val),
        }
    }
}

fn print_hex(val: u64) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        uart::putc(hex[nibble]);
    }
}
