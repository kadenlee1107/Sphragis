// Sphragis — VZ Diagnostic Kernel
// Minimal kernel that tries every possible output method.
// Used to discover the correct device addresses in a VZ VM.
//
// VZLinuxBootLoader entry:
//   x0 = DTB pointer
//   Kernel at EL1

.section .text.boot
.global _start

_start:
    b       _diag_start

    // ARM64 Image header
    .align  3
    .quad   0
    .quad   0x10000         // image_size (64KB)
    .quad   0x0A
    .quad   0
    .quad   0
    .quad   0
    .long   0x644d5241      // ARM\x64
    .long   0

_diag_start:
    msr     daifset, #0xf

    // Save DTB pointer
    mov     x20, x0

    // Enable FP/NEON
    mov     x1, #(3 << 20)
    msr     cpacr_el1, x1
    isb

    // Set up a simple stack
    adr     x1, _start
    add     x1, x1, #0x8000    // Stack 32KB above start
    mov     sp, x1

    // Write a status marker to a known memory location
    // The VM host can read this to confirm we're running
    adr     x1, _start
    add     x1, x1, #0x4000    // Status area at start+16KB
    ldr     x2, =0x4241545F4F53    // "SPHRAGIS" in ASCII
    str     x2, [x1]
    mov     x2, #1              // Stage = 1 (boot started)
    str     x2, [x1, #8]

    // Try DTB parse — check if x20 has valid FDT magic
    mov     x2, #0              // DTB valid flag
    cbz     x20, no_dtb
    ldr     w3, [x20]
    // FDT magic = 0xD00DFEED (big-endian)
    ldr     w4, =0xEDFE0DD0    // Little-endian representation
    cmp     w3, w4
    b.ne    no_dtb
    mov     x2, #1
no_dtb:
    // Store DTB status
    adr     x1, _start
    add     x1, x1, #0x4000
    str     x2, [x1, #16]      // Offset 16: DTB valid (0/1)
    str     x20, [x1, #24]     // Offset 24: DTB address

    // Now scan for any writable device
    // Try writing 'B' to many possible MMIO addresses
    mov     w5, #0x42           // 'B' for Bat

    // QEMU PL011 UART
    ldr     x6, =0x09000000
    strb    w5, [x6]

    // Try common ARM virt serial bases
    ldr     x6, =0x09000000
    strb    w5, [x6]

    // Update status = 2 (probing done)
    adr     x1, _start
    add     x1, x1, #0x4000
    mov     x2, #2
    str     x2, [x1, #8]

    // Halt
1:  wfe
    b       1b
