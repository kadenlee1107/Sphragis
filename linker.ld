/* Bat_OS Linker Script — ARM64 bare metal for QEMU virt machine */
/* QEMU virt loads kernel at 0x40080000 */

ENTRY(_start)

SECTIONS
{
    . = 0x40080000;

    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
        /* INITRD-FIX: Rust emits cold paths into `.text.cold.*` /
         * `.text.unlikely.*`, which in some toolchain versions land in the
         * rodata PT_LOAD. If that happens, blocks past __text_end (flagged
         * BLOCK_KERNEL_DATA with PXN set) trap on the first EL1 fetch after
         * MMU-enable. Catch them explicitly so all executable bytes stay
         * below the W^X boundary. See the matching TODO (V9) in
         * src/batcave/linux/mmu.rs for the transitional widen-all-RAM
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
    . += 0x80000; /* 512KB kernel stack */
    __stack_start = .;

    . = ALIGN(4096);
    __kernel_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
