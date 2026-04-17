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
        mov     x2, sp
        str     x2, [x0, #248]

        // Save tpidr_el0 into old.x[18] slot (offset 144).
        mrs     x2, tpidr_el0
        str     x2, [x0, #144]

        // V5-SIDE-003: save Q0..Q31 + FPSR + FPCR.
        // SavedRegs.q @ 272 (after sp_el0/elr_el1/spsr_el1 = 272/16=17 pair slots).
        stp     q0,  q1,  [x0, #272]
        stp     q2,  q3,  [x0, #272+32]
        stp     q4,  q5,  [x0, #272+64]
        stp     q6,  q7,  [x0, #272+96]
        stp     q8,  q9,  [x0, #272+128]
        stp     q10, q11, [x0, #272+160]
        stp     q12, q13, [x0, #272+192]
        stp     q14, q15, [x0, #272+224]
        stp     q16, q17, [x0, #272+256]
        stp     q18, q19, [x0, #272+288]
        stp     q20, q21, [x0, #272+320]
        stp     q22, q23, [x0, #272+352]
        stp     q24, q25, [x0, #272+384]
        stp     q26, q27, [x0, #272+416]
        stp     q28, q29, [x0, #272+448]
        stp     q30, q31, [x0, #272+480]
        mrs     x2, fpsr
        str     x2, [x0, #784]
        mrs     x2, fpcr
        str     x2, [x0, #792]

        // ─── Restore callee-saved regs of NEW thread from *x1 ───
        ldp     x19, x20, [x1, #152]
        ldp     x21, x22, [x1, #168]
        ldp     x23, x24, [x1, #184]
        ldp     x25, x26, [x1, #200]
        ldp     x27, x28, [x1, #216]
        ldp     x29, x30, [x1, #232]

        ldr     x2, [x1, #248]
        mov     sp, x2

        ldr     x2, [x1, #144]
        msr     tpidr_el0, x2

        // V5-SIDE-003: restore Q0..Q31 + FPSR + FPCR.
        ldp     q0,  q1,  [x1, #272]
        ldp     q2,  q3,  [x1, #272+32]
        ldp     q4,  q5,  [x1, #272+64]
        ldp     q6,  q7,  [x1, #272+96]
        ldp     q8,  q9,  [x1, #272+128]
        ldp     q10, q11, [x1, #272+160]
        ldp     q12, q13, [x1, #272+192]
        ldp     q14, q15, [x1, #272+224]
        ldp     q16, q17, [x1, #272+256]
        ldp     q18, q19, [x1, #272+288]
        ldp     q20, q21, [x1, #272+320]
        ldp     q22, q23, [x1, #272+352]
        ldp     q24, q25, [x1, #272+384]
        ldp     q26, q27, [x1, #272+416]
        ldp     q28, q29, [x1, #272+448]
        ldp     q30, q31, [x1, #272+480]
        ldr     x2, [x1, #784]
        msr     fpsr, x2
        ldr     x2, [x1, #792]
        msr     fpcr, x2

        isb

        ret
        .size   cxt_switch_cooperative, . - cxt_switch_cooperative
