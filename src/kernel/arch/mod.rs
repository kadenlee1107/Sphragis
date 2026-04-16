#![allow(dead_code)]
// Bat_OS — Architecture-specific kernel code (ARM64)

use crate::drivers::uart;

core::arch::global_asm!(include_str!("../../batcave/linux/forkjmp.s"));

unsafe extern "C" {
    fn fork_save(buf: *mut u64) -> u64;
    fn fork_restore(buf: *const u64, retval: u64) -> !;
}

// Saved busybox SP at clone time (for eret back to parent)
static mut SAVED_BUSYBOX_SP: u64 = 0;

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
    }
    uart::puts("  [arch] Exception vectors installed\n");
}

pub fn init_timer() {
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
pub extern "C" fn handle_irq(_frame: *mut TrapFrame) {
    let ctl: u64;
    unsafe { core::arch::asm!("mrs {}, cntp_ctl_el0", out(reg) ctl); }
    if ctl & 0b100 != 0 {
        reset_timer();
        crate::kernel::scheduler::tick();
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
                                // Set SP to clone-time busybox SP FIRST
                                // (before we clobber any input regs)
                                "mov sp, {sp_val}",
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

                        // Real exit (not a child — leave BatCave entirely)
                        crate::batcave::linux::mmu::disable();
                        core::arch::asm!(
                            "movz x0, #0x1000",
                            "movk x0, #0x4000, lsl #16",
                            "ldr x0, [x0]",
                            "mov sp, x0",
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
                        for i in (0..272).step_by(8) {
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
                            // Read the path to check if it's a busybox applet
                            let path_ptr = f.x[0] as usize;
                            let argv_ptr = f.x[1] as usize;
                            let mut path_buf = [0u8; 128];
                            let mut plen = 0usize;
                            for i in 0..127 {
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

                                            // Jump to hello binary entry (identity-mapped)
                                            let entry = phys_entry;
                                            core::arch::asm!(
                                                "mov sp, {sp_val}",
                                                "br {entry}",
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

                                // Read argv from userspace (up to 8 args)
                                let _arg_ptrs = [0usize; 8];
                                let mut arg_bufs = [[0u8; 64]; 8];
                                let mut arg_lens = [0usize; 8];
                                let mut argc = 0usize;
                                if argv_ptr != 0 {
                                    for i in 0..8 {
                                        let ap: u64;
                                        core::arch::asm!("ldr {v}, [{a}]",
                                            a = in(reg) argv_ptr + i * 8, v = out(reg) ap);
                                        if ap == 0 { break; }
                                        for j in 0..63 {
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

                                    // Jump to worker busybox entry (via identity mapping)
                                    let entry = worker_entry as u64;
                                    core::arch::asm!(
                                        "mov sp, {sp_val}",
                                        "br {entry}",
                                        sp_val = in(reg) sp as u64,
                                        entry = in(reg) entry,
                                        options(noreturn),
                                    );
                                }
                            }
                        }
                    }

                    let result = crate::batcave::linux::syscall::handle(0, syscall_num, args);
                    f.x[0] = result as u64;

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
                                // Set child stack SP (16-byte aligned)
                                "mov sp, {csp}",
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
                        "mov sp, {sp_val}",
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

            // Only reach here for non-busybox BRK (real shell exit)
            uart::puts("[linux] exit — returning to desktop\n");
            unsafe {
                crate::batcave::linux::mmu::disable();
                core::arch::asm!(
                    "movz x0, #0x1000",
                    "movk x0, #0x4000, lsl #16",
                    "ldr x0, [x0]",
                    "mov sp, x0",
                );
                crate::ui::desktop::resume();
            }
        }
        0x00 => {
            // Unknown/undefined instruction — might be HVF-unsupported atomics
            // (LDADD, LDSET, LDCLR, SWP, etc. at encoding 0x38/0xB8/0xF8)
            let elr = unsafe { (*frame).elr };
            let in_code = (elr < 0x1400000) || (elr >= 0x40000000 && elr < 0x50000000);
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

                // Other unknown instr in busybox — skip
                unsafe { (*frame).elr = elr + 4; }
                return;
            }
            uart::puts("!!! UNHANDLED EC=0 !!!\n");
            uart::puts("  ELR: 0x"); print_hex(elr);
            uart::puts("\n");
            loop { unsafe { core::arch::asm!("wfe") }; }
        }
        _ => {
            uart::puts("!!! UNHANDLED SYNC EXCEPTION !!!\n");
            uart::puts("  EC: 0x"); print_hex(ec);
            uart::puts("\n");
            let elr = unsafe { (*frame).elr };
            uart::puts("  ELR: 0x"); print_hex(elr);
            uart::puts("\n");
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
unsafe fn emulate_load(addr: usize, size: u32) -> u64 {
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
