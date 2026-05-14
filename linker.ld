/* Sphragis Linker Script — ARM64 bare metal for QEMU virt machine */
/* QEMU virt with `-kernel <Image-header binary>` loads at PA 0x40200000.
 * (Empirically verified: see V8-PT-DIAG output 2026-04-25 where
 * *0x40080000=0 but *0x40200000=0x14000010 = `b _real_start`.) Linker
 * VMA must match runtime PA so identity MMU mappings + absolute address
 * loads against linker-resolved VMAs both find the correct bytes.
 * Previously . = 0x40080000 — kernel "worked" via PC-relative branches
 * but every kernel-address absolute load (e.g. `ldr x0, =KERNEL_FN`) was
 * off by 0x180000, which surfaced as Chromium SIGBUS at fake PC
 * 0x4020113c after enough indirect calls landed on misread function
 * pointers. */

ENTRY(_start)

SECTIONS
{
    . = 0x40200000;

    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
        /* INITRD-FIX: Rust emits cold paths into `.text.cold.*` /
         * `.text.unlikely.*`, which in some toolchain versions land in the
         * rodata PT_LOAD. If that happens, blocks past __text_end (flagged
         * BLOCK_KERNEL_DATA with PXN set) trap on the first EL1 fetch after
         * MMU-enable. Catch them explicitly so all executable bytes stay
         * below the W^X boundary. See the matching TODO (V9) in
         * src/caves/linux/mmu.rs for the transitional widen-all-RAM
         * fallback we use until this linker hint plus a rebuild confirm
         * the sections really are gathered here. */
        *(.text.cold.*)
        *(.text.unlikely.*)
        *(.text.hot.*)
        *(.text.startup.*)
    }

    . = ALIGN(0x200000);    /* 2 MB align so MMU 2 MB blocks can split here */
    __text_end = .;         /* end of executable kernel code (W^X boundary) */

    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4096);
    .data : {
        *(.data .data.*)
    }

    . = ALIGN(4096);
    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4096);
    __stack_end = .;
    /* 🎯 STUMP #86: 8 MB stack (was 512KB). The JS engine's
       compile_script returns a 1+MB tuple by value (FunctionProto
       has bytecode: [u8; 8192]; the return type is
       (FunctionProto, [FunctionProto; MAX_PROTOS=128], usize)
       = ~1MB). 512KB stack silently overflowed and corrupted the
       caller's frame; the visible symptom was the compiler hanging
       immediately after the parser produced the AST. 8MB gives
       plenty of headroom. */
    . += 0x800000; /* 8 MB kernel stack */
    __stack_start = .;

    . = ALIGN(4096);
    __kernel_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
