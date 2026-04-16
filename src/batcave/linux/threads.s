// Bat_OS — ARM64 cooperative context switch for BatCave threads.
// Called by threads::schedule() when a task yields voluntarily
// (blocking syscall, explicit yield).
//
// x0 = *mut SavedRegs       old thread (snapshot current callee-saved here)
// x1 = *const SavedRegs     new thread (restore callee-saved from here)
//
// SavedRegs layout (see threads.rs, #[repr(C)]):
//   x[31] @ offsets 0..248    (x0..x30, 8 bytes each)
//   sp_el0     @ 248
//   elr_el1    @ 256
//   spsr_el1   @ 264
//
// For cooperative switch, only callee-saved registers need to be preserved:
//     x19..x28  general callee-saved
//     x29       frame pointer
//     x30       link register
//     sp        stack pointer
// Caller-saved regs (x0..x18) are spilled by the compiler before it
// calls us if they're live, per AAPCS64.
//
// Note: this runs at EL1 (kernel mode). We are switching between kernel
// execution contexts for threads that share a single address space (all
// Chromium threads live in one BatCave address space). We do NOT touch
// TTBR0, so no TLB flush is needed.
//
// tpidr_el0 (TLS base) IS swapped — each thread has its own TLS region.

        .section .text
        .globl  cxt_switch_cooperative
        .type   cxt_switch_cooperative, @function
        .align  4

cxt_switch_cooperative:
        // ─── Save callee-saved regs of OLD thread into *x0 ───
        // x[19] through x[30] go at offsets 152..248 in the SavedRegs struct
        stp     x19, x20, [x0, #152]
        stp     x21, x22, [x0, #168]
        stp     x23, x24, [x0, #184]
        stp     x25, x26, [x0, #200]
        stp     x27, x28, [x0, #216]
        stp     x29, x30, [x0, #232]

        // Save current SP into old.sp_el0 (offset 248).
        // (We treat SP_EL1 as the save target because we're in kernel mode
        // running this on behalf of the user thread.)
        mov     x2, sp
        str     x2, [x0, #248]

        // Save tpidr_el0 (user TLS base) into old.x[18] slot — we don't
        // have a dedicated field so reuse the otherwise-unused x18 slot.
        // x18 is the ARM64 "platform register" / shadow stack reg; we
        // don't use it. Offset = 18*8 = 144.
        mrs     x2, tpidr_el0
        str     x2, [x0, #144]

        // ─── Restore callee-saved regs of NEW thread from *x1 ───
        ldp     x19, x20, [x1, #152]
        ldp     x21, x22, [x1, #168]
        ldp     x23, x24, [x1, #184]
        ldp     x25, x26, [x1, #200]
        ldp     x27, x28, [x1, #216]
        ldp     x29, x30, [x1, #232]

        // Restore SP
        ldr     x2, [x1, #248]
        mov     sp, x2

        // Restore tpidr_el0 (TLS base) for the new thread
        ldr     x2, [x1, #144]
        msr     tpidr_el0, x2

        // Synchronize: make sure the TLS change is visible before we return
        // to code that may read tpidr_el0.
        isb

        // Return — x30 was just restored to the new thread's LR, so ret
        // jumps to wherever that thread was about to resume from.
        ret
        .size   cxt_switch_cooperative, . - cxt_switch_cooperative
