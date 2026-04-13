// Bat_OS — ARM64 Boot Stub
// First code that executes on the CPU.
// Sets up the stack and jumps to Rust.

.section .text.boot
.global _start

_start:
    // Disable all interrupts
    msr     daifset, #0xf

    // Read processor ID — only core 0 boots, others halt
    mrs     x0, mpidr_el1
    and     x0, x0, #0xff
    cbz     x0, primary_core
    b       halt

primary_core:
    // Enable FP/NEON (CPACR_EL1.FPEN = 0b11)
    mov     x0, #(3 << 20)
    msr     cpacr_el1, x0
    isb

    // Set up the stack pointer
    ldr     x0, =__stack_start
    mov     sp, x0

    // Zero out the BSS section
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end
bss_clear:
    cmp     x0, x1
    b.ge    bss_done
    str     xzr, [x0], #8
    b       bss_clear

bss_done:
    // Jump to Rust entry point — no return
    bl      kernel_main

halt:
    wfe
    b       halt
