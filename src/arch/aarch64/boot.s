// Bat_OS — ARM64 Boot Stub
// First code that executes on the CPU.
// Sets up the stack and jumps to Rust.

.section .text.boot
.global _start

_start:
    // QEMU passes the DTB physical address in x0 at kernel entry.
    // Preserve it in a callee-saved register before anything else
    // clobbers it — kernel_main's 2nd arg needs it so the DTB-
    // supplied initrd range reaches initrd::set_range.
    mov     x19, x0

    // Disable all interrupts
    msr     daifset, #0xf

    // Read processor ID — only core 0 boots, others halt.
    // Use x2 instead of x0 so we don't clobber the DTB we just
    // saved into x19.
    mrs     x2, mpidr_el1
    and     x2, x2, #0xff
    cbz     x2, primary_core
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
    // Call Rust: kernel_main(uart_available=1, dtb_ptr=<saved>).
    // First arg: QEMU serial is live.
    // Second arg: the DTB pointer we preserved at entry.
    mov     x0, #1
    mov     x1, x19
    bl      kernel_main

halt:
    wfe
    b       halt
