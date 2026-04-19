// Bat_OS — Apple Silicon Boot Stub
// Entry point when loaded by m1n1 (Asahi bootloader).
//
// m1n1 passes boot arguments in x0:
//   x0 = pointer to M1n1BootArgs structure
//
// M4 framebuffer format is ARGB2101010 at FB 0x103e0050000, stride 0x2f40,
// 3024x1964 pixels. Pixel encoding:
//   bits [31:30] = A (2 bits), [29:20] = R (10), [19:10] = G (10), [9:0] = B (10)
//
// Bring-up uses full-FB re-paints as stage markers: each stage overwrites
// the whole screen with a distinctive color, so the final visible color
// on a capture tells us the last stage reached before halt.

.section .text.apple_boot
.global _apple_start

// Paint-loop macro: fill 16 MiB starting at FB base with the 32-bit pixel
// value currently in w10. Clobbers x9, x10, x11. Takes a unique label
// suffix to avoid label collision across the file.
.macro FB_FILL label
    ldr     x9, =0x103e0050000
    mov     x11, #0x01000000
\label :
    str     w10, [x9], #4
    subs    x11, x11, #4
    b.ne    \label
    dsb     sy
.endm

_apple_start:
    mov     x20, x0                       // save boot_args

    // ─── Stage 1: reached _apple_start ─── YELLOW (our bug-pixel from before)
    // 0xFFFF0000 decoded in ARGB2101010: A=3, R=0x3FF, G=0x3C0, B=0 → yellow.
    // Leaving as-is because we've already visually validated this works.
    mov     w10, #0x0000
    movk    w10, #0xffff, lsl #16
    FB_FILL fb1

    msr     daifset, #0xf                 // disable interrupts

    // ─── Stage 2: post-daifset ─── BLUE (0xC00003FF)
    mov     w10, #0x03ff
    movk    w10, #0xc000, lsl #16
    FB_FILL fb2

    // Enable FP/NEON
    mov     x1, #(3 << 20)
    msr     cpacr_el1, x1
    isb

    // ─── Stage 3: FPU enabled ─── GREEN (0xC00FFC00)
    mov     w10, #0xfc00
    movk    w10, #0xc00f, lsl #16
    FB_FILL fb3

    // Set up stack
    ldr     x1, =__stack_start
    mov     sp, x1

    // ─── Stage 4: stack set ─── MAGENTA (0xFFF003FF)
    mov     w10, #0x03ff
    movk    w10, #0xfff0, lsl #16
    FB_FILL fb4

    // Zero BSS
    ldr     x1, =__bss_start
    ldr     x2, =__bss_end
apple_bss_clear:
    cmp     x1, x2
    b.ge    apple_bss_done
    str     xzr, [x1], #8
    b       apple_bss_clear

apple_bss_done:
    // ─── Stage 5: BSS zeroed ─── WHITE (0xFFFFFFFF), about to call Rust
    mov     w10, #0xffff
    movk    w10, #0xffff, lsl #16
    FB_FILL fb5

    mov     x0, x20
    bl      kernel_main_apple

    // ─── Stage 6: Rust returned (shouldn't happen) ─── CYAN (0xC00FFFFF)
    mov     w10, #0xffff
    movk    w10, #0xc00f, lsl #16
    FB_FILL fb6

apple_halt:
    wfe
    b       apple_halt
