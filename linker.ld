/* Bat_OS Linker Script — ARM64 bare metal for QEMU virt machine */
/* QEMU virt loads kernel at 0x40080000 */

ENTRY(_start)

SECTIONS
{
    . = 0x40080000;

    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
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
