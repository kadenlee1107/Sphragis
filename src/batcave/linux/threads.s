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
        // (NB: this is SP_EL1 at cooperative-yield time — field name
        // is historical.)
        mov     x2, sp
        str     x2, [x0, #248]

        // ROOT-FIX: save USER SP_EL0 into old.user_sp_el0 (offset 800).
        // Previously SP_EL0 was never saved across a cooperative
        // switch, so whenever thread A yielded, thread B ran, and
        // thread A was later resumed, the `eret` took A back to EL0
        // with B's SP_EL0 — reading the wrong stack. Chromium crashed
        // with x30 loaded from t2's stack slot holding a V8 cage
        // pointer (the ret landed in the cage = SIGSEGV).
        mrs     x2, sp_el0
        str     x2, [x0, #800]

        // REAL-FORK: save the user TTBR0 into old.user_ttbr0
        // (offset 808). Thread might have been moved to a forked
        // cave; capturing TTBR0 here means the next time we resume
        // it, the asm below restores the same address space.
        mrs     x2, ttbr0_el1
        bic     x2, x2, #1              // strip CnP bit
        str     x2, [x0, #808]

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

        // ROOT-FIX: restore USER SP_EL0 from new.user_sp_el0 so the
        // eventual `eret` back to EL0 puts the resumed thread on its
        // OWN user stack, not whatever stack the predecessor happened
        // to leave in the SP_EL0 MSR.
        ldr     x2, [x1, #800]
        msr     sp_el0, x2

        // REAL-FORK: restore user TTBR0 from new.user_ttbr0 if it
        // differs from the currently-active TTBR0. Crossing into a
        // different cave (different process address space) requires
        // a TLB flush to drop stale translations. Same-cave threads
        // skip the swap to avoid the TLB hit.
        //
        // user_ttbr0 == 0 means the thread inherits whatever was
        // active (used for early-init phases before init_main_thread
        // captures the live TTBR0).
        ldr     x2, [x1, #808]
        cbz     x2, 1f                  // 0 → don't touch TTBR0
        mrs     x3, ttbr0_el1
        bic     x3, x3, #1              // strip CnP for compare
        cmp     x2, x3
        b.eq    1f                      // same — skip
        msr     ttbr0_el1, x2
        isb
        tlbi    vmalle1
        dsb     ish
        isb
1:

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

// ---------------------------------------------------------------------------
// thread_first_run — landing pad for a freshly-cloned thread the first
// time the scheduler picks it up.
//
// cxt_switch_cooperative just restored callee-saved regs (x19..x30) from
// the new thread's saved_regs and `ret`'d to x30. We arranged for x30
// to point at this function, with x19..x22 carrying the user-mode
// resume parameters:
//
//     x19 = user_pc   (elr_el1 after eret)
//     x20 = user_sp   (sp_el0 after eret)
//     x21 = spsr      (spsr_el1 after eret; 0 = EL0t, IRQs on)
//     x22 = x0_val    (what x0 is in EL0; 0 for a clone() child)
//
// tpidr_el0 was already restored by cxt_switch_cooperative from the
// saved .x[18] slot. sp_el1 was set to a dedicated kernel stack page
// by the same routine so any IRQ between here and eret has room for
// its trap frame.
//
// We deliberately zero the rest of the GPRs before eret so the child
// can't peek at whatever residue cxt_switch_cooperative left behind
// (side-channel hygiene + matches what Linux does in its ret_from_fork).
// ---------------------------------------------------------------------------
// cxt_switch_first_run(old: *mut SavedRegs, new: *const SavedRegs, user_sp)
//
// Like cxt_switch_cooperative, but the NEW thread has never run. We
// save OLD's callee-saved state as usual (so the outgoing thread can
// later be resumed cooperatively), then restore the NEW thread's FULL
// GPR snapshot from new.saved_regs.x[0..30] — which was seeded from
// the parent's svc-entry trap frame so glibc/musl __clone trampolines
// find fn/arg in x10/x12 intact — and eret to EL0. The caller is
// responsible for overwriting x[0] with 0 (clone's child return)
// before calling.
//
// Register params:
//   x0 — old SavedRegs pointer
//   x1 — new SavedRegs pointer (destroyed by the full restore)
//   x2 — user sp_el0
//
// SavedRegs layout: x[0..30] @ 0..240, sp_el0 @ 248, elr_el1 @ 256,
// spsr_el1 @ 264.
// ---------------------------------------------------------------------------
        .globl  cxt_switch_first_run
        .type   cxt_switch_first_run, @function
        .align  4

cxt_switch_first_run:
        // ─── Save callee-saved regs of OLD thread into *x0 (same as coop) ───
        // schedule() tail-calls us via `b` (not `bl`) after popping its
        // own stack frame and restoring x30 to its original caller-LR,
        // so the current x30 is the address OLD's eventual `ret` from
        // the next cooperative switch should land on (park_slot,
        // ppoll, etc.). SP already points to OLD's caller's frame top.
        stp     x19, x20, [x0, #152]
        stp     x21, x22, [x0, #168]
        stp     x23, x24, [x0, #184]
        stp     x25, x26, [x0, #200]
        stp     x27, x28, [x0, #216]
        stp     x29, x30, [x0, #232]
        mov     x3, sp
        str     x3, [x0, #248]
        // ROOT-FIX: also capture OLD's user SP_EL0 so the next
        // cooperative resume of OLD eret's with the right user sp.
        mrs     x3, sp_el0
        str     x3, [x0, #800]
        // REAL-FORK: capture OLD's user TTBR0 so cross-cave
        // resumes restore the right address space.
        mrs     x3, ttbr0_el1
        bic     x3, x3, #1
        str     x3, [x0, #808]
        mrs     x3, tpidr_el0
        str     x3, [x0, #144]
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
        mrs     x3, fpsr
        str     x3, [x0, #784]
        mrs     x3, fpcr
        str     x3, [x0, #792]

        // ─── Activate the NEW thread's kernel stack ───
        // Once we mov sp, we're running on the fresh thread's kernel
        // stack so any IRQ arriving between here and the eret has a
        // stable place to push its trap frame.
        ldr     x3, [x1, #248]          // kernel_sp from new.sp_el0 field
        mov     sp, x3

        // ─── Restore tpidr_el0 (TLS) from new.x[18] ───
        ldr     x3, [x1, #144]
        msr     tpidr_el0, x3

        // REAL-FORK: activate NEW thread's user TTBR0 if non-zero
        // and different from the current. Forked children get a
        // fresh L1 (their own address space); without this swap
        // the child would eret into the parent's page table and
        // see parent's memory.
        ldr     x3, [x1, #808]          // new.user_ttbr0
        cbz     x3, 2f                  // 0 → leave TTBR0 alone
        mrs     x4, ttbr0_el1
        bic     x4, x4, #1
        cmp     x3, x4
        b.eq    2f                      // same — skip
        msr     ttbr0_el1, x3
        isb
        tlbi    vmalle1
        dsb     ish
        isb
2:

        // ─── Set up the exception-return registers ───
        ldr     x3, [x1, #256]          // elr_el1 (user_pc)
        msr     elr_el1, x3
        ldr     x3, [x1, #264]          // spsr_el1
        msr     spsr_el1, x3
        msr     sp_el0, x2              // user_sp from arg

        // ─── Restore the full GPR snapshot ───
        // We can't use x1 both as base and as a load destination in
        // the final ldp, so stash the pointer in x30 (we already
        // spilled OLD's x30 into OLD's saved_regs above). Load x30
        // last to overwrite the scratch pointer.
        mov     x30, x1
        ldp     x0,  x1,  [x30, #0]
        ldp     x2,  x3,  [x30, #16]
        ldp     x4,  x5,  [x30, #32]
        ldp     x6,  x7,  [x30, #48]
        ldp     x8,  x9,  [x30, #64]
        ldp     x10, x11, [x30, #80]
        ldp     x12, x13, [x30, #96]
        ldp     x14, x15, [x30, #112]
        ldp     x16, x17, [x30, #128]
        ldp     x18, x19, [x30, #144]
        ldp     x20, x21, [x30, #160]
        ldp     x22, x23, [x30, #176]
        ldp     x24, x25, [x30, #192]
        ldp     x26, x27, [x30, #208]
        ldp     x28, x29, [x30, #224]
        ldr     x30,      [x30, #240]

        isb
        eret
        .size   cxt_switch_first_run, . - cxt_switch_first_run
