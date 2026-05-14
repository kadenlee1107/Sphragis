/*
 * Sphragis syscall fuzzer.
 *
 * Runs as a BatCave guest. Walks `seeds.h` and issues each syscall with
 * its seed tuple, plus a few LCG-mutated variants. Prints a single line
 * per call to stdout via write(1,...) so the Sphragis UART log preserves
 * the full trace — if the kernel panics, the last printed "TRY" line
 * identifies the culprit.
 *
 * Zero external deps: we make syscalls via `svc #0` directly. Doesn't
 * need libc startup — _start is the entry point.
 *
 * Build: see Makefile.
 */

#include <stdint.h>

typedef unsigned long ul;
typedef long          sl;

/* ------------------------- inline syscall helpers ------------------------- */

static inline sl do_syscall6(long nr, ul a0, ul a1, ul a2, ul a3, ul a4, ul a5) {
    register ul x8 __asm__("x8") = (ul)nr;
    register ul x0 __asm__("x0") = a0;
    register ul x1 __asm__("x1") = a1;
    register ul x2 __asm__("x2") = a2;
    register ul x3 __asm__("x3") = a3;
    register ul x4 __asm__("x4") = a4;
    register ul x5 __asm__("x5") = a5;
    __asm__ volatile("svc #0"
                     : "+r"(x0)
                     : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5)
                     : "memory", "cc");
    return (sl)x0;
}

#define sys_write(fd,b,c) do_syscall6(64, (ul)(fd), (ul)(b), (ul)(c), 0, 0, 0)
#define sys_exit(c)       do_syscall6(93, (ul)(c), 0, 0, 0, 0, 0)

/* ------------------------- tiny print helpers ---------------------------- */

static void put_str(const char *s) {
    const char *p = s;
    unsigned long n = 0;
    while (*p) { p++; n++; }
    sys_write(1, s, n);
}

static void put_hex(ul v) {
    static const char hex[] = "0123456789abcdef";
    char buf[18];
    buf[0] = '0'; buf[1] = 'x';
    for (int i = 0; i < 16; i++) {
        buf[2 + i] = hex[(v >> ((15 - i) * 4)) & 0xf];
    }
    sys_write(1, buf, 18);
}

static void put_dec(sl v) {
    char buf[24];
    int neg = 0;
    ul u;
    if (v < 0) { neg = 1; u = (ul)(-v); } else { u = (ul)v; }
    int n = 0;
    if (u == 0) { buf[n++] = '0'; }
    while (u) { buf[n++] = '0' + (int)(u % 10); u /= 10; }
    if (neg)  { buf[n++] = '-'; }
    for (int i = 0; i < n / 2; i++) {
        char t = buf[i]; buf[i] = buf[n - 1 - i]; buf[n - 1 - i] = t;
    }
    sys_write(1, buf, (ul)n);
}

/* ------------------------- shared user scratch --------------------------- */

unsigned char scratch[4096] __attribute__((aligned(16)));

/* Populate the scratch with some "looks valid" content so short paths /
 * iovecs / sockaddrs aren't trivially rejected on content grounds. */
static void prime_scratch(void) {
    /* scratch[0..]  : a printable path "/tmp/f" */
    scratch[0] = '/'; scratch[1] = 't'; scratch[2] = 'm'; scratch[3] = 'p';
    scratch[4] = '/'; scratch[5] = 'f'; scratch[6] = 0;
    /* scratch[8..15]: another short string / second argv entry */
    scratch[8]  = 'h'; scratch[9]  = 'i'; scratch[10] = 0;
    /* scratch[16..] : a sockaddr_in for 127.0.0.1:53 */
    /* sa_family (2) = AF_INET */
    scratch[16] = 2; scratch[17] = 0;
    /* sin_port = htons(53) */
    scratch[18] = 0; scratch[19] = 53;
    /* sin_addr = 127.0.0.1 */
    scratch[20] = 127; scratch[21] = 0; scratch[22] = 0; scratch[23] = 1;
    /* zero padding */
    for (int i = 24; i < 32; i++) scratch[i] = 0;
    /* scratch[64..] : iov[0]={iov_base=&scratch[80], iov_len=4}, iov[1]={&scratch[88],4} */
    {
        unsigned long *iov = (unsigned long *)(scratch + 64);
        iov[0] = (unsigned long)(scratch + 80);
        iov[1] = 4;
        iov[2] = (unsigned long)(scratch + 88);
        iov[3] = 4;
    }
    scratch[80] = 'H'; scratch[81] = 'I'; scratch[82] = '\n'; scratch[83] = 0;
    scratch[88] = 'B'; scratch[89] = 'Y'; scratch[90] = 'E'; scratch[91] = '\n';
    /* scratch[128..] : an epoll_event: u32 events; u64 data */
    scratch[128] = 1; /* EPOLLIN */
    scratch[129] = 0; scratch[130] = 0; scratch[131] = 0;
    /* data (8 bytes) = 0xdeadbeefcafef00d */
    unsigned long magic = 0xdeadbeefcafef00dUL;
    for (int i = 0; i < 8; i++) scratch[132 + i] = (unsigned char)(magic >> (i * 8));
    /* scratch[160..]: sigset / signal handler 'data' (8 bytes of zeros) */
    for (int i = 160; i < 256; i++) scratch[i] = 0;
    /* scratch[256..]: timespec (tv_sec=0, tv_nsec=1000) */
    unsigned long *ts = (unsigned long *)(scratch + 256);
    ts[0] = 0; ts[1] = 1000;
    /* scratch[320..]: itimerspec (interval + value timespecs, all 0 -> disarm) */
    for (int i = 320; i < 320 + 32; i++) scratch[i] = 0;
}

#include "seeds.h"

/* ----------------------------- LCG mutation ------------------------------ */

static ul lcg_state = 0x1234567890abcdefUL;
static ul lcg(void) {
    lcg_state = lcg_state * 6364136223846793005UL + 1442695040888963407UL;
    return lcg_state;
}

/* Mutate one arg slot to a pseudo-interesting value. */
static ul mutate(ul orig) {
    switch (lcg() & 0x7) {
        case 0: return 0;
        case 1: return 0xffffffffffffffffUL;
        case 2: return 0x7fffffffffffffffUL;
        case 3: return orig | 1;                       /* unalign */
        case 4: return orig ^ 0x1000;                  /* shift */
        case 5: return 0xffff000040000000UL;           /* kernel-like */
        case 6: return (ul)scratch + (lcg() & 0xff0);  /* random user offset */
        default: return orig;
    }
}

/* ------------------------------- main ------------------------------------ */

static void try_seed(const struct seed *s) {
    put_str("TRY sys=");
    put_dec(s->nr);
    put_str(" args=[ ");
    for (int i = 0; i < 6; i++) {
        put_hex(s->a[i]);
        if (i != 5) put_str(", ");
    }
    put_str(" ] -> ret=");
    sl r = do_syscall6(s->nr, s->a[0], s->a[1], s->a[2], s->a[3], s->a[4], s->a[5]);
    put_dec(r);
    put_str("\n");
}

static void try_mutated(const struct seed *s) {
    struct seed m = *s;
    int slot = (int)(lcg() % 6);
    m.a[slot] = mutate(m.a[slot]);
    put_str("MUT sys=");
    put_dec(m.nr);
    put_str(" args=[ ");
    for (int i = 0; i < 6; i++) {
        put_hex(m.a[i]);
        if (i != 5) put_str(", ");
    }
    put_str(" ] -> ret=");
    sl r = do_syscall6(m.nr, m.a[0], m.a[1], m.a[2], m.a[3], m.a[4], m.a[5]);
    put_dec(r);
    put_str("\n");
}

void _start(void) {
    prime_scratch();
    put_str("== Sphragis syscall fuzzer starting ==\n");

    const unsigned long nseeds = sizeof(seeds) / sizeof(seeds[0]);
    for (unsigned long i = 0; i < nseeds; i++) {
        try_seed(&seeds[i]);
        /* Two mutated siblings per seed. */
        try_mutated(&seeds[i]);
        try_mutated(&seeds[i]);
    }

    put_str("== Sphragis syscall fuzzer done ==\n");
    sys_exit(0);
    /* unreachable */
    for (;;) { __asm__ volatile("wfi"); }
}
