/* Bat_OS Linker Script — Apple Silicon bare metal */
/* m1n1 loads the kernel image at a high address in DRAM */
/* The actual load address is determined by m1n1, but we */
/* link at a reasonable default and let m1n1 relocate if needed. */

ENTRY(_apple_start)

SECTIONS
{
    /* Apple Silicon unified memory typically starts at 0x800000000 */
    /* m1n1 loads payload at ~0x810000000 */
    . = 0x810000000;

    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    . = ALIGN(4096);
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
    . += 0x100000; /* 1MB kernel stack for Apple Silicon */
    __stack_start = .;

    __kernel_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
