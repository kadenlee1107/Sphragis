/* test_pt_write.c — attempt to locate and corrupt a page-table entry.
 *
 * Strategy: scan kernel RAM looking for 4-KB pages that "look like" an L2.
 * An L2 entry for a 2-MB block has low bits 0x1 (VALID) and block/table
 * distinction in bit[1]. We pattern-match on AF (bit 10) and ATTR_NORMAL
 * (bits 4:2 = 0) to find L2s, then patch entry 50 to point at an arbitrary
 * physical address. On success the cave gains a new VA.
 *
 * Under ESC-003 this succeeds (kernel RAM is EL0-writable). Under a fixed
 * implementation the first str should take a permission fault.
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

/* Looks-like-L2-block-entry test. */
static int looks_like_block(unsigned long v) {
    if ((v & 0x3) != 0x1) return 0;             /* VALID && block descriptor */
    if ((v & (1UL << 10)) == 0) return 0;       /* AF set */
    if ((v & (0x7UL << 2)) != 0) return 0;      /* ATTR_NORMAL (index 0) */
    /* Address bits for a 2 MB block must be 2 MB aligned. */
    if (v & ((1UL << 21) - 1) & ~0x1FFFUL) return 0;
    return 1;
}

void _start(void) {
    puts_("[pt] BatCave page-table-write probe\n");

    /* Scan the kernel RAM window in 4 KB steps. */
    unsigned long base = 0x40000000UL;
    unsigned long end  = 0x40200000UL; /* first 2 MB */
    unsigned long found = 0;

    for (unsigned long p = base; p < end; p += 4096) {
        int hits = 0;
        for (int i = 0; i < 16; i++) {
            unsigned long v;
            __asm__ volatile ("ldr %0, [%1]" : "=r"(v) : "r"(p + i*8));
            if (looks_like_block(v)) hits++;
        }
        if (hits >= 8) { found = p; break; }
    }

    if (!found) {
        puts_("[pt] PASS — no plausible L2 in first 2 MB (or reads faulted)\n");
    } else {
        puts_("[pt] candidate L2 at "); puthex(found);
        /* Attempt to overwrite entry 50 — maps VA [100 MB .. 102 MB). */
        unsigned long new_entry = 0x40000000UL | 0x401UL; /* block + AF */
        __asm__ volatile ("str %0, [%1]" :: "r"(new_entry), "r"(found + 50 * 8));
        puts_("[pt] FAIL — wrote L2[50], kernel is corruptable\n");
    }

    register long x0 __asm__("x0") = 0;
    register long x8 __asm__("x8") = 93;
    __asm__ volatile ("svc #0" : : "r"(x0), "r"(x8));
    for (;;) {}
}
