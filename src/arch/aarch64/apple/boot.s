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

// V-HANDOFF-2: dedicated section so the linker places _apple_start
// at offset 0 of the Apple raw binary. Previously shared `.text.boot`
// with linux_header.s, so the Linux kernel Image header landed at
// offset 0 and m1n1's chainload jumped into that — which interprets
// x0 as an FDT pointer, faults on the first deref, and machine-resets.
.section .text.apple_boot
.global _apple_start

_apple_start:
    // x0 = pointer to boot args from m1n1
    // Save it — we'll pass it to Rust
    mov     x20, x0

    // ────────────────────────────────────────────────────────────
    // V-HANDOFF-3: framebuffer proof-of-life.
    // BEFORE we touch the boot args, before any Rust, before stack
    // setup — paint a band of red pixels at the top of the
    // framebuffer m1n1 left running. If this visibly appears on the
    // display after chainload, we KNOW _apple_start actually ran
    // and the problem is later in boot.
    //
    // Framebuffer base 0x103e0050000 is observed from m1n1 on this
    // specific M4 (Mac16,1). m1n1 logs it at boot as:
    //   "FB: 0x103e0050000 ..."
    // Hardcoding is fine for bring-up — production will read from
    // boot_args.video.base via Rust.
    //
    // Writes ~16 MiB of 0xFFFF0000 (opaque red in little-endian ARGB8
    // or AARRGGBB, visible as red on M4's 30bpp FB).
    // ────────────────────────────────────────────────────────────
    ldr     x9, =0x103e0050000       // framebuffer base
    mov     w10, #0x0000              // low 16 bits: blue+green = 0
    movk    w10, #0xffff, lsl #16     // high 16 bits: red = 0xffff
    mov     x11, #0x01000000          // 16 MiB / 4 = 4 MiB pixels
fb_fill:
    str     w10, [x9], #4
    subs    x11, x11, #4
    b.ne    fb_fill
    // DSB so the pixels actually hit RAM before we potentially fault
    // in the boot-args code below.
    dsb     sy

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
