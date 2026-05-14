/* test_memory_peek.c — Cave escape probe: read above the cave window.
 *
 * Expected if isolation holds: faults / zeros for addresses outside the cave's
 * 200 MB window. Expected under the current design (see ESC-001/011 in
 * security/PENTEST_SANDBOX_ESCAPE.md): successful reads returning arbitrary
 * kernel bytes from 0x40000000..0x4FFFFFFF.
 *
 * Build: aarch64-linux-musl-gcc -static -nostdlib -o test_memory_peek
 *        test_memory_peek.c
 * Wire up: embed ELF via include_bytes!() in runner.rs, dispatch to
 *          run_small_elf / load_elf and call from a cave with no caps.
 */

static long sys_write(int fd, const void *buf, unsigned long n) {
    register long x0 __asm__("x0") = fd;
    register long x1 __asm__("x1") = (long)buf;
    register long x2 __asm__("x2") = n;
    register long x8 __asm__("x8") = 64; /* write */
    __asm__ volatile ("svc #0" : "+r"(x0) : "r"(x1), "r"(x2), "r"(x8));
    return x0;
}

static void puts_(const char *s) {
    unsigned long n = 0; while (s[n]) n++;
    sys_write(1, s, n);
}

static void puthex(unsigned long v) {
    char buf[19]; buf[0]='0'; buf[1]='x';
    for (int i = 0; i < 16; i++) {
        int nib = (v >> ((15 - i) * 4)) & 0xF;
        buf[2 + i] = nib < 10 ? '0' + nib : 'a' + (nib - 10);
    }
    buf[18] = '\n'; sys_write(1, buf, 19);
}

static const unsigned long PROBE[] = {
    0x40000000UL, /* start of kernel RAM identity map */
    0x40000100UL, /* known SP_SAVE_ADDR (ESC-028) */
    0x40001000UL,
    0x40100000UL,
    0x40800000UL, /* middle of kernel BSS region */
    0x4FFFFF00UL, /* near top of 256 MB window */
};

void _start(void) {
    puts_("[peek] Cave memory-peek probe\n");
    for (unsigned i = 0; i < sizeof(PROBE)/sizeof(PROBE[0]); i++) {
        unsigned long a = PROBE[i];
        unsigned long v = 0;
        /* NOTE: if isolation is correct, this ldr will take a permission
         * fault and the cave dies. Under current design it just returns. */
        __asm__ volatile ("ldr %0, [%1]" : "=r"(v) : "r"(a));
        puts_("[peek] addr "); puthex(a);
        puts_("[peek]  val "); puthex(v);
    }
    puts_("[peek] FAIL — isolation did not stop us\n");
    /* exit(0) */
    register long x0 __asm__("x0") = 0;
    register long x8 __asm__("x8") = 93;
    __asm__ volatile ("svc #0" : : "r"(x0), "r"(x8));
    for (;;) {}
}
