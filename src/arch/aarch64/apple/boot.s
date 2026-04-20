// Bat_OS — Apple Silicon Boot Stub
// Entry point when loaded by m1n1 (Asahi bootloader).
//
// m1n1 passes boot arguments in x0:
//   x0 = pointer to M1n1BootArgs structure
//
// This stub:
// 1. Saves boot args pointer
// 2. Paints a proof-of-life band to the m1n1 framebuffer
// 3. Disables interrupts
// 4. Enables FP/NEON
// 5. Sets up stack (PC-relative — see below)
// 6. Zeros BSS (PC-relative — see below)
// 7. Jumps to kernel_main_apple
//
// On Apple Silicon, we enter at EL2 (m1n1 drops us here) or EL1
// depending on m1n1 configuration.

// V-HANDOFF-2: dedicated section so the linker places _apple_start
// at offset 0 of the Apple raw binary. Previously shared `.text.boot`
// with linux_header.s, so the Linux kernel Image header landed at
// offset 0 and m1n1's chainload jumped into that — which interprets
// x0 as an FDT pointer, faults on the first deref, and machine-resets.
.section .text.apple_boot
.global _apple_start

_apple_start:
    // x0 = pointer to boot args from m1n1
    mov     x20, x0

    // ────────────────────────────────────────────────────────────
    // V-HV-GUEST-1: under m1n1's hypervisor (run_guest.py), m1n1
    // drops the guest at EL1 after calling fb_shutdown(true), which
    // free()s the framebuffer backing memory. Writing 16 MiB to the
    // old FB physical address clobbers m1n1's heap (stage-2 pass
    // through covers RAM-HIGH) and the Mac resets within seconds.
    // Direct m1n1 chainload enters at EL2 and the FB is still live;
    // paint for proof-of-life there. Skip the paint at EL1.
    // ────────────────────────────────────────────────────────────
    mrs     x19, CurrentEL            // x19[3:2] = CurrentEL
    cmp     x19, #(2 << 2)
    b.ne    fb_fill_skip

    // V-HANDOFF-3: framebuffer proof-of-life (EL2 direct chainload only).
    // M4 FB pixel format is ARGB2101010 (30 bpp, not ARGB8888):
    //   bits [31:30] = A, [29:20] = R, [19:10] = G, [9:0] = B.
    // 0xFFFF0000 decoded in that format is A=3, R=0x3FF, G=0x3C0,
    // B=0 → yellow, which is what you'll see on the M4 screen.
    ldr     x9, =0x103e0050000       // framebuffer base (observed on M4)
    mov     w10, #0x0000
    movk    w10, #0xffff, lsl #16
    mov     x11, #0x01000000          // 16 MiB / 4 = 4 M pixels
fb_fill:
    str     w10, [x9], #4
    subs    x11, x11, #4
    b.ne    fb_fill
    dsb     sy
fb_fill_skip:

    // Disable interrupts
    msr     daifset, #0xf

    // NOTE on "only boot on primary core" gate: removed. m1n1 chainload
    // is invoked with --skip-secondary-cpus (-S) on M4, so exactly one
    // CPU enters here. The M4 boot P-core has nonzero MPIDR.Aff0
    // (observed smp_id = 0x6), so the classic
    //   and x1, mpidr, #0xff ; cbnz x1, apple_halt
    // silently WFE-halted every chainload.

    // Enable FP/NEON
    mov     x1, #(3 << 20)
    msr     cpacr_el1, x1
    isb

    // Set up stack — PC-relative (adrp + :lo12:) so it resolves to
    // the LOADED binary's __stack_start, not the linker's link-time
    // absolute address. Under m1n1 chainload our binary lives at
    // ~0x1000xxxxxx while the linker base is 0x8_1000_0000.
    adrp    x1, __stack_start
    add     x1, x1, #:lo12:__stack_start
    mov     sp, x1

    // Zero BSS — same PC-relative requirement. Previously the
    // literal-pool `ldr x1, =__bss_start` was emitting the link-time
    // absolute, so the BSS-zero loop was writing zeros to unmapped
    // memory while our real BSS (containing every atomic static in
    // the kernel) stayed whatever random bytes m1n1 had left there
    // — the first Rust static write then faulted.
    adrp    x1, __bss_start
    add     x1, x1, #:lo12:__bss_start
    adrp    x2, __bss_end
    add     x2, x2, #:lo12:__bss_end
apple_bss_clear:
    cmp     x1, x2
    b.ge    apple_bss_done
    str     xzr, [x1], #8
    b       apple_bss_clear

apple_bss_done:
    mov     x0, x20
    bl      kernel_main_apple

apple_halt:
    wfe
    b       apple_halt
