/* test_blit_nocap.c — call syscall 500 (blit_framebuffer) without the
 * `display` capability. Under a correct sandbox this returns EACCES (-13).
 * Under the current code (ESC-007) it paints a red bar at (0,0).
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

#define W 512
#define H 384
static unsigned int pixels[W * H];

void _start(void) {
    puts_("[blit] Cave display-bypass probe\n");
    for (int i = 0; i < W*H; i++) pixels[i] = 0xFFFF0000U; /* ARGB red */

    /* syscall 500: x0=ptr, x1=w, x2=h, x3=dst_x, x4=dst_y */
    register long x0 __asm__("x0") = (long)pixels;
    register long x1 __asm__("x1") = W;
    register long x2 __asm__("x2") = H;
    register long x3 __asm__("x3") = 0;
    register long x4 __asm__("x4") = 0;
    register long x8 __asm__("x8") = 500;
    __asm__ volatile ("svc #0"
        : "+r"(x0)
        : "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x8));

    if (x0 == -13) {
        puts_("[blit] PASS — EACCES without display cap\n");
    } else {
        puts_("[blit] FAIL — blit succeeded without display cap\n");
    }

    register long rx0 __asm__("x0") = 0;
    register long rx8 __asm__("x8") = 93;
    __asm__ volatile ("svc #0" : : "r"(rx0), "r"(rx8));
    for (;;) {}
}
