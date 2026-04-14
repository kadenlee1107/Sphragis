// Bat_OS Mini C Library — replaces musl for cross-compiled programs
// Provides: printf, malloc, free, strlen, strcpy, strcmp, memcpy, memset, sprintf
// All backed by raw Linux syscalls (no complex init, no TLS, no atomics)

#ifndef MINILIB_H
#define MINILIB_H

typedef unsigned long size_t;
typedef long ssize_t;
#define NULL ((void*)0)

// ─── Syscall wrappers ───
static long __syscall1(long nr, long a0) {
    register long x0 __asm__("x0") = a0;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x8) : "memory");
    return x0;
}
static long __syscall3(long nr, long a0, long a1, long a2) {
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x1), "r"(x2), "r"(x8) : "memory");
    return x0;
}
static long __syscall6(long nr, long a0, long a1, long a2, long a3, long a4, long a5) {
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x3 __asm__("x3") = a3;
    register long x4 __asm__("x4") = a4;
    register long x5 __asm__("x5") = a5;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5), "r"(x8) : "memory");
    return x0;
}

// ─── I/O ───
static ssize_t write(int fd, const void *buf, size_t count) {
    return __syscall3(64, fd, (long)buf, count);
}

static int puts(const char *s) {
    size_t len = 0; while (s[len]) len++;
    write(1, s, len);
    write(1, "\n", 1);
    return 0;
}

// Simple printf (supports %s, %d, %x, %zu, %c, %%)
static int printf(const char *fmt, ...) {
    // Variadic args via register/stack access
    // For simplicity, use a buffer-based approach
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    char buf[512];
    int bi = 0;
    while (*fmt && bi < 500) {
        if (*fmt == '%') {
            fmt++;
            if (*fmt == 's') {
                const char *s = __builtin_va_arg(ap, const char*);
                while (*s && bi < 500) buf[bi++] = *s++;
            } else if (*fmt == 'd') {
                long v = __builtin_va_arg(ap, int);
                if (v < 0) { buf[bi++] = '-'; v = -v; }
                char num[20]; int ni = 19; num[ni] = 0;
                if (v == 0) num[--ni] = '0';
                else while (v > 0) { num[--ni] = '0' + (v % 10); v /= 10; }
                const char *p = &num[ni];
                while (*p && bi < 500) buf[bi++] = *p++;
            } else if (*fmt == 'x') {
                unsigned long v = __builtin_va_arg(ap, unsigned int);
                const char *hex = "0123456789abcdef";
                char num[17]; int ni = 16; num[ni] = 0;
                if (v == 0) num[--ni] = '0';
                else while (v > 0) { num[--ni] = hex[v & 0xf]; v >>= 4; }
                const char *p = &num[ni];
                while (*p && bi < 500) buf[bi++] = *p++;
            } else if (*fmt == 'z' && fmt[1] == 'u') {
                fmt++;
                unsigned long v = __builtin_va_arg(ap, unsigned long);
                char num[20]; int ni = 19; num[ni] = 0;
                if (v == 0) num[--ni] = '0';
                else while (v > 0) { num[--ni] = '0' + (v % 10); v /= 10; }
                const char *p = &num[ni];
                while (*p && bi < 500) buf[bi++] = *p++;
            } else if (*fmt == 'c') {
                buf[bi++] = (char)__builtin_va_arg(ap, int);
            } else if (*fmt == '%') {
                buf[bi++] = '%';
            } else {
                buf[bi++] = '%'; buf[bi++] = *fmt;
            }
            fmt++;
        } else {
            buf[bi++] = *fmt++;
        }
    }
    __builtin_va_end(ap);
    write(1, buf, bi);
    return bi;
}

// ─── Memory ───
static void *malloc(size_t size) {
    size = (size + 4095) & ~4095; // round up to page
    long addr = __syscall6(222, 0, size, 3, 34, -1, 0);
    return (addr < 0) ? NULL : (void*)addr;
}

static void free(void *ptr) {
    // munmap — leaks for now (our kernel stubs it)
    if (ptr) __syscall3(215, (long)ptr, 4096, 0);
}

static void *calloc(size_t n, size_t sz) {
    void *p = malloc(n * sz);
    if (p) {
        char *c = (char*)p;
        for (size_t i = 0; i < n * sz; i++) c[i] = 0;
    }
    return p;
}

// ─── String ───
static size_t strlen(const char *s) {
    size_t n = 0; while (s[n]) n++; return n;
}

static char *strcpy(char *d, const char *s) {
    char *r = d; while ((*d++ = *s++)); return r;
}

static int strcmp(const char *a, const char *b) {
    while (*a && *a == *b) { a++; b++; }
    return *(unsigned char*)a - *(unsigned char*)b;
}

static void *memcpy(void *d, const void *s, size_t n) {
    char *dc = (char*)d; const char *sc = (const char*)s;
    for (size_t i = 0; i < n; i++) dc[i] = sc[i];
    return d;
}

static void *memset(void *d, int c, size_t n) {
    char *dc = (char*)d;
    for (size_t i = 0; i < n; i++) dc[i] = (char)c;
    return d;
}

static int sprintf(char *buf, const char *fmt, ...) {
    // Minimal sprintf
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    int bi = 0;
    while (*fmt) {
        if (*fmt == '%') {
            fmt++;
            if (*fmt == 'd') {
                int v = __builtin_va_arg(ap, int);
                if (v < 0) { buf[bi++] = '-'; v = -v; }
                char num[20]; int ni = 19; num[ni] = 0;
                if (v == 0) num[--ni] = '0';
                else while (v > 0) { num[--ni] = '0' + (v % 10); v /= 10; }
                const char *p = &num[ni];
                while (*p) buf[bi++] = *p++;
            } else if (*fmt == 's') {
                const char *s = __builtin_va_arg(ap, const char*);
                while (*s) buf[bi++] = *s++;
            } else {
                buf[bi++] = '%'; buf[bi++] = *fmt;
            }
            fmt++;
        } else {
            buf[bi++] = *fmt++;
        }
    }
    buf[bi] = 0;
    __builtin_va_end(ap);
    return bi;
}

// ─── Process ───
static void exit(int code) {
    __syscall1(93, code);
    __builtin_unreachable();
}

// Entry point wrapper
#define MINILIB_MAIN \
    extern int main(int argc, char **argv); \
    void _start(void) { exit(main(0, NULL)); }

#endif // MINILIB_H
