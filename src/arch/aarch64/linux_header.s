// Sphragis — ARM64 Boot Stub (Universal: QEMU + VZ VM)
// Works with both QEMU -kernel and VZLinuxBootLoader.
// Entry: x0 = DTB pointer (from bootloader)

.section .text.boot
.global _start

_start:
    b       _real_start

    // ARM64 Image header (required by VZLinuxBootLoader)
    .align  3
    .quad   0                       // text_offset
    .quad   __kernel_end - _start   // image_size
    .quad   0x0A                    // flags
    .quad   0
    .quad   0
    .quad   0
    .long   0x644d5241              // "ARM\x64" magic
    .long   0

_real_start:
    msr     daifset, #0xf

    // Save DTB pointer from bootloader
    mov     x20, x0

    // Only primary core
    mrs     x1, mpidr_el1
    and     x1, x1, #0xff
    cbnz    x1, halt

    // Enable FP/NEON
    mov     x1, #(3 << 20)
    msr     cpacr_el1, x1
    isb

    // Set up stack
    ldr     x1, =__stack_start
    mov     sp, x1

    // Zero BSS
    ldr     x1, =__bss_start
    ldr     x2, =__bss_end
bss_clear:
    cmp     x1, x2
    b.ge    bss_done
    str     xzr, [x1], #8
    b       bss_clear

bss_done:
    mov     x20, x0       // Save DTB pointer

    // QEMU always has UART — skip fault-based probing
    // (fault probing confuses HVF on real M4)
    mov     x21, #1

    // Jump to Rust: kernel_main(uart_available, dtb_ptr)
    mov     x0, x21
    mov     x1, x20
    bl      kernel_main

halt:
    wfe
    b       halt

// Safe exit point for Cave Linux processes
// When a Linux binary calls exit(), the exception handler
// redirects ELR here instead of back to the binary.
.global _cave_exit_loop
_cave_exit_loop:
    wfe
    b       _cave_exit_loop
