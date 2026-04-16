/* test_mmio_probe.c — BatCave MMIO probe.
 *
 * Attempts direct reads from UART (0x09000000) and virtio control register
 * (0x0A000000). Under correct per-cave page tables (ESC-005/006) these MUST
 * fault. Under the current primary-table design they succeed.
 */

static long sys_write(int fd, const void *buf, unsigned long n) {
    register long x0 __asm__("x0") = fd;
    register long x1 __asm__("x1") = (long)buf;
    register long x2 __asm__("x2") = n;
    register long x8 __asm__("x8") = 64;
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

void _start(void) {
    puts_("[mmio] BatCave MMIO probe\n");

    unsigned long uart = 0x09000000UL;
    unsigned long gpu  = 0x0A000000UL;
    unsigned long v;

    /* UART ID register area */
    __asm__ volatile ("ldr %w0, [%1]" : "=r"(v) : "r"(uart + 0x18));
    puts_("[mmio] UART[0x18]="); puthex(v);

    /* Try to write UART TX FIFO directly — bypass any logging policy */
    unsigned int c = 'X';
    __asm__ volatile ("str %w0, [%1]" :: "r"(c), "r"(uart));

    /* virtio-mmio magic value register */
    __asm__ volatile ("ldr %w0, [%1]" : "=r"(v) : "r"(gpu));
    puts_("[mmio] virtio magic="); puthex(v); /* expect 0x74726976 ("virt") */

    puts_("[mmio] FAIL — MMIO reachable from EL0\n");

    register long x0 __asm__("x0") = 0;
    register long x8 __asm__("x8") = 93;
    __asm__ volatile ("svc #0" : : "r"(x0), "r"(x8));
    for (;;) {}
}
