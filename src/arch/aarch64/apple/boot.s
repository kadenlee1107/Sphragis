// Bat_OS — Apple Silicon Boot Stub
// Entry point when loaded by m1n1 (Asahi bootloader).
//
// m1n1 passes boot arguments in x0:
//   x0 = pointer to M1n1BootArgs structure
//
// This stub:
// 1. Saves boot args pointer
// 2. Sets up the stack
// 3. Enables FPU
// 4. Zeros BSS
// 5. Jumps to Rust entry point
//
// On Apple Silicon, we enter at EL2 (m1n1 drops us here)
// or EL1 depending on m1n1 configuration.

.section .text.boot
.global _apple_start

_apple_start:
    // x0 = pointer to boot args from m1n1
    // Save it — we'll pass it to Rust
    mov     x20, x0

    // Disable interrupts
    msr     daifset, #0xf

    // Only boot on primary core
    mrs     x1, mpidr_el1
    and     x1, x1, #0xff
    cbnz    x1, apple_halt

    // Enable FP/NEON
    mov     x1, #(3 << 20)
    msr     cpacr_el1, x1
    isb

    // Set up stack (use a region after the kernel)
    ldr     x1, =__stack_start
    mov     sp, x1

    // Zero BSS
    ldr     x1, =__bss_start
    ldr     x2, =__bss_end
apple_bss_clear:
    cmp     x1, x2
    b.ge    apple_bss_done
    str     xzr, [x1], #8
    b       apple_bss_clear

apple_bss_done:
    // x0 = boot args pointer (restore from x20)
    mov     x0, x20

    // Jump to Rust — kernel_main_apple(boot_args: *const M1n1BootArgs)
    bl      kernel_main_apple

apple_halt:
    wfe
    b       apple_halt
