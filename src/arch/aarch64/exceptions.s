// Bat_OS — ARM64 Exception Vector Table
// Handles interrupts, syscalls, and faults.
// Each vector entry is 128 bytes (32 instructions).

.section .text

// Macro: save all general-purpose registers to stack
.macro SAVE_REGS
    sub     sp, sp, #272
    stp     x0,  x1,  [sp, #0]
    stp     x2,  x3,  [sp, #16]
    stp     x4,  x5,  [sp, #32]
    stp     x6,  x7,  [sp, #48]
    stp     x8,  x9,  [sp, #64]
    stp     x10, x11, [sp, #80]
    stp     x12, x13, [sp, #96]
    stp     x14, x15, [sp, #112]
    stp     x16, x17, [sp, #128]
    stp     x18, x19, [sp, #144]
    stp     x20, x21, [sp, #160]
    stp     x22, x23, [sp, #176]
    stp     x24, x25, [sp, #192]
    stp     x26, x27, [sp, #208]
    stp     x28, x29, [sp, #224]
    str     x30, [sp, #240]
    mrs     x0, elr_el1
    mrs     x1, spsr_el1
    stp     x0, x1, [sp, #248]
.endm

// Macro: restore all general-purpose registers from stack
.macro RESTORE_REGS
    ldp     x0, x1, [sp, #248]
    msr     elr_el1, x0
    msr     spsr_el1, x1
    ldp     x0,  x1,  [sp, #0]
    ldp     x2,  x3,  [sp, #16]
    ldp     x4,  x5,  [sp, #32]
    ldp     x6,  x7,  [sp, #48]
    ldp     x8,  x9,  [sp, #64]
    ldp     x10, x11, [sp, #80]
    ldp     x12, x13, [sp, #96]
    ldp     x14, x15, [sp, #112]
    ldp     x16, x17, [sp, #128]
    ldp     x18, x19, [sp, #144]
    ldp     x20, x21, [sp, #160]
    ldp     x22, x23, [sp, #176]
    ldp     x24, x25, [sp, #192]
    ldp     x26, x27, [sp, #208]
    ldp     x28, x29, [sp, #224]
    ldr     x30, [sp, #240]
    add     sp, sp, #272
    eret
.endm


// Vector entry macro — branch to handler
.macro VECTOR_ENTRY label
    .balign 128
    b       \label
.endm

// Exception vector table — must be 2KB aligned
.balign 2048
.global exception_vectors
exception_vectors:
    // Current EL with SP_EL0 (not used)
    VECTOR_ENTRY sync_unhandled
    VECTOR_ENTRY irq_unhandled
    VECTOR_ENTRY fiq_unhandled
    VECTOR_ENTRY serror_unhandled

    // Current EL with SP_ELx (kernel mode interrupts)
    VECTOR_ENTRY sync_el1h
    VECTOR_ENTRY irq_el1h
    VECTOR_ENTRY fiq_unhandled
    VECTOR_ENTRY serror_unhandled

    // Lower EL using AArch64 (userspace interrupts)
    VECTOR_ENTRY sync_el0_64
    VECTOR_ENTRY irq_el0_64
    VECTOR_ENTRY fiq_unhandled
    VECTOR_ENTRY serror_unhandled

    // Lower EL using AArch32 (not supported)
    VECTOR_ENTRY sync_unhandled
    VECTOR_ENTRY irq_unhandled
    VECTOR_ENTRY fiq_unhandled
    VECTOR_ENTRY serror_unhandled

// Handlers

sync_el1h:
    SAVE_REGS
    mov     x0, sp
    bl      handle_sync_exception
    RESTORE_REGS

irq_el1h:
    SAVE_REGS
    mov     x0, sp
    bl      handle_irq
    RESTORE_REGS

sync_el0_64:
    SAVE_REGS
    mov     x0, sp
    bl      handle_sync_exception
    RESTORE_REGS

irq_el0_64:
    SAVE_REGS
    mov     x0, sp
    bl      handle_irq
    RESTORE_REGS

sync_unhandled:
    SAVE_REGS
    mov     x0, sp
    bl      handle_unhandled_exception
    RESTORE_REGS

irq_unhandled:
    SAVE_REGS
    mov     x0, sp
    bl      handle_irq
    RESTORE_REGS

fiq_unhandled:
    b       fiq_unhandled

serror_unhandled:
    b       serror_unhandled

// Context switch: switch_context(old_ctx: *mut CpuContext, new_ctx: *const CpuContext)
.global switch_context
switch_context:
    // Save callee-saved regs to old context
    stp     x19, x20, [x0, #0]
    stp     x21, x22, [x0, #16]
    stp     x23, x24, [x0, #32]
    stp     x25, x26, [x0, #48]
    stp     x27, x28, [x0, #64]
    stp     x29, x30, [x0, #80]
    mov     x2, sp
    str     x2, [x0, #96]

    // Restore callee-saved regs from new context
    ldp     x19, x20, [x1, #0]
    ldp     x21, x22, [x1, #16]
    ldp     x23, x24, [x1, #32]
    ldp     x25, x26, [x1, #48]
    ldp     x27, x28, [x1, #64]
    ldp     x29, x30, [x1, #80]
    ldr     x2, [x1, #96]
    mov     sp, x2

    ret
