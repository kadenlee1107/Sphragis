/*
 * Bat_OS -- Comprehensive C standard library implementation
 *
 * Provides ACTUAL function bodies for all functions declared in our
 * sysroot headers at /Users/kadenlee/Bat_OS/include/.
 * Linked with NetSurf object files.
 *
 * All functions use Linux aarch64 syscalls via raw svc #0.
 * Compile with:
 *   clang --target=aarch64-linux-gnu -ffreestanding -nostdlib -O2 \
 *         -mstrict-align -isystem /Users/kadenlee/Bat_OS/include \
 *         -c libc.c -o libc.o
 */

/* ===== Include all sysroot headers ===== */
#include <stddef.h>
#include <stdint.h>
#include <stdarg.h>
#include <stdbool.h>
#include <limits.h>
#include <errno.h>
#include <string.h>
#include <strings.h>
#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include <math.h>
#include <time.h>
#include <unistd.h>
#include <fcntl.h>
#include <signal.h>
#include <setjmp.h>
#include <locale.h>
#include <assert.h>
#include <inttypes.h>
#include <endian.h>
#include <iconv.h>
#include <regex.h>
#include <dirent.h>
#include <sys/types.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <sys/param.h>
#include <sys/utsname.h>
#include <netinet/in.h>
#include <arpa/inet.h>

/* ===================================================================
 *  SECTION 1: Syscall helpers
 * =================================================================== */

int errno;

static inline long __syscall0(long nr)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0");
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x8) : "memory");
    return x0;
}

static inline long __syscall1(long nr, long a0)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8) : "memory");
    return x0;
}

static inline long __syscall2(long nr, long a0, long a1)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1) : "memory");
    return x0;
}

static inline long __syscall3(long nr, long a0, long a1, long a2)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2) : "memory");
    return x0;
}

static inline long __syscall4(long nr, long a0, long a1, long a2, long a3)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x3 __asm__("x3") = a3;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3) : "memory");
    return x0;
}

static inline long __syscall5(long nr, long a0, long a1, long a2, long a3, long a4)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x3 __asm__("x3") = a3;
    register long x4 __asm__("x4") = a4;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4) : "memory");
    return x0;
}

static inline long __syscall6(long nr, long a0, long a1, long a2, long a3, long a4, long a5)
{
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x3 __asm__("x3") = a3;
    register long x4 __asm__("x4") = a4;
    register long x5 __asm__("x5") = a5;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5) : "memory");
    return x0;
}

/* Syscall numbers -- aarch64 Linux */
#define SYS_read           63
#define SYS_write          64
#define SYS_close          57
#define SYS_lseek          62
#define SYS_exit           93
#define SYS_exit_group     94
#define SYS_mmap          222
#define SYS_munmap        215
#define SYS_mprotect      226
#define SYS_clock_gettime 113
#define SYS_nanosleep     101
#define SYS_openat        56
#define SYS_mkdirat       34
#define SYS_unlinkat      35
#define SYS_fstat          80
#define SYS_fstatat        79
#define SYS_dup            23
#define SYS_dup3           24
#define SYS_pipe2          59
#define SYS_fcntl          25
#define SYS_getpid        172
#define SYS_getppid       173
#define SYS_getuid        174
#define SYS_getgid        176
#define SYS_socket        198
#define SYS_bind          200
#define SYS_listen        201
#define SYS_accept        202
#define SYS_connect       203
#define SYS_sendto        206
#define SYS_recvfrom      207
#define SYS_setsockopt    208
#define SYS_getsockopt    209
#define SYS_shutdown      210
#define SYS_getpeername   205
#define SYS_getsockname   204
#define SYS_pselect6       72
#define SYS_readlinkat     78
#define SYS_ftruncate      46
#define SYS_fsync          82
#define SYS_kill          129
#define SYS_sigaction     134
#define SYS_sigprocmask   135
#define SYS_uname         160
#define SYS_getcwd         17
#define SYS_chdir          49
#define SYS_linkat         37
#define SYS_symlinkat      36
#define SYS_renameat2      276
#define SYS_getdents64      61
#define SYS_msync          227
#define SYS_sysconf       (0)  /* not a real syscall -- handled in code */

#define AT_FDCWD (-100)

/* Helper: set errno and return -1 on error */
static long __set_errno(long r)
{
    if (r < 0 && r > -4096) {
        errno = (int)(-r);
        return -1;
    }
    return r;
}

/* ===================================================================
 *  SECTION 2: Memory operations (string.h, strings.h)
 * =================================================================== */

void *memcpy(void *dest, const void *src, size_t n)
{
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    while (n--) *d++ = *s++;
    return dest;
}

void *memmove(void *dest, const void *src, size_t n)
{
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    if (d < s) {
        while (n--) *d++ = *s++;
    } else if (d > s) {
        d += n; s += n;
        while (n--) *--d = *--s;
    }
    return dest;
}

void *memset(void *s, int c, size_t n)
{
    unsigned char *p = (unsigned char *)s;
    while (n--) *p++ = (unsigned char)c;
    return s;
}

int memcmp(const void *s1, const void *s2, size_t n)
{
    const unsigned char *a = (const unsigned char *)s1;
    const unsigned char *b = (const unsigned char *)s2;
    while (n--) {
        if (*a != *b) return (int)*a - (int)*b;
        a++; b++;
    }
    return 0;
}
int bcmp(const void *s1, const void *s2, size_t n) { return memcmp(s1, s2, n); }

void *memchr(const void *s, int c, size_t n)
{
    const unsigned char *p = (const unsigned char *)s;
    while (n--) {
        if (*p == (unsigned char)c) return (void *)p;
        p++;
    }
    return NULL;
}

/* strings.h */
void bzero(void *s, size_t n)   { memset(s, 0, n); }
void bcopy(const void *src, void *dest, size_t n) { memmove(dest, src, n); }

/* ===================================================================
 *  SECTION 3: String operations (string.h)
 * =================================================================== */

size_t strlen(const char *s)
{
    const char *p = s;
    while (*p) p++;
    return (size_t)(p - s);
}

size_t strnlen(const char *s, size_t maxlen)
{
    size_t n = 0;
    while (n < maxlen && s[n]) n++;
    return n;
}

char *strcpy(char *dest, const char *src)
{
    char *d = dest;
    while ((*d++ = *src++));
    return dest;
}

char *strncpy(char *dest, const char *src, size_t n)
{
    size_t i;
    for (i = 0; i < n && src[i]; i++) dest[i] = src[i];
    for (; i < n; i++) dest[i] = '\0';
    return dest;
}

char *strcat(char *dest, const char *src)
{
    char *d = dest + strlen(dest);
    while ((*d++ = *src++));
    return dest;
}

char *strncat(char *dest, const char *src, size_t n)
{
    char *d = dest + strlen(dest);
    while (n-- && *src) *d++ = *src++;
    *d = '\0';
    return dest;
}

int strcmp(const char *s1, const char *s2)
{
    while (*s1 && *s1 == *s2) { s1++; s2++; }
    return (int)(unsigned char)*s1 - (int)(unsigned char)*s2;
}

int strncmp(const char *s1, const char *s2, size_t n)
{
    while (n && *s1 && *s1 == *s2) { s1++; s2++; n--; }
    return n ? (int)(unsigned char)*s1 - (int)(unsigned char)*s2 : 0;
}

static int __tolower_internal(int c)
{
    return (c >= 'A' && c <= 'Z') ? c + 32 : c;
}

int strcasecmp(const char *s1, const char *s2)
{
    while (*s1 && __tolower_internal(*s1) == __tolower_internal(*s2)) { s1++; s2++; }
    return __tolower_internal((unsigned char)*s1) - __tolower_internal((unsigned char)*s2);
}

int strncasecmp(const char *s1, const char *s2, size_t n)
{
    while (n && *s1 && __tolower_internal(*s1) == __tolower_internal(*s2)) { s1++; s2++; n--; }
    return n ? __tolower_internal((unsigned char)*s1) - __tolower_internal((unsigned char)*s2) : 0;
}

char *strchr(const char *s, int c)
{
    while (*s) {
        if (*s == (char)c) return (char *)s;
        s++;
    }
    return (c == '\0') ? (char *)s : NULL;
}

char *strrchr(const char *s, int c)
{
    const char *last = NULL;
    while (*s) {
        if (*s == (char)c) last = s;
        s++;
    }
    if (c == '\0') return (char *)s;
    return (char *)last;
}

char *strstr(const char *haystack, const char *needle)
{
    size_t nlen = strlen(needle);
    if (!nlen) return (char *)haystack;
    while (*haystack) {
        if (*haystack == *needle && strncmp(haystack, needle, nlen) == 0)
            return (char *)haystack;
        haystack++;
    }
    return NULL;
}

char *strpbrk(const char *s, const char *accept)
{
    while (*s) {
        const char *a = accept;
        while (*a) {
            if (*s == *a) return (char *)s;
            a++;
        }
        s++;
    }
    return NULL;
}

size_t strspn(const char *s, const char *accept)
{
    size_t n = 0;
    while (s[n]) {
        const char *a = accept;
        int found = 0;
        while (*a) { if (s[n] == *a) { found = 1; break; } a++; }
        if (!found) break;
        n++;
    }
    return n;
}

size_t strcspn(const char *s, const char *reject)
{
    size_t n = 0;
    while (s[n]) {
        const char *r = reject;
        while (*r) { if (s[n] == *r) return n; r++; }
        n++;
    }
    return n;
}

char *strdup(const char *s)
{
    size_t len = strlen(s) + 1;
    char *d = (char *)malloc(len);
    if (d) memcpy(d, s, len);
    return d;
}

char *strndup(const char *s, size_t n)
{
    size_t len = strnlen(s, n);
    char *d = (char *)malloc(len + 1);
    if (d) { memcpy(d, s, len); d[len] = '\0'; }
    return d;
}

static char *__strtok_state;

char *strtok(char *s, const char *delim)
{
    return strtok_r(s, delim, &__strtok_state);
}

char *strtok_r(char *s, const char *delim, char **saveptr)
{
    if (!s) s = *saveptr;
    if (!s) return NULL;
    /* skip leading delimiters */
    s += strspn(s, delim);
    if (!*s) { *saveptr = NULL; return NULL; }
    char *tok = s;
    s = strpbrk(tok, delim);
    if (s) { *s = '\0'; *saveptr = s + 1; }
    else   { *saveptr = NULL; }
    return tok;
}

char *strerror(int errnum)
{
    (void)errnum;
    return (char *)"error";
}

/* ===================================================================
 *  SECTION 4: Memory allocation (stdlib.h, sys/mman.h)
 * =================================================================== */

/* Allocation header: store the actual mmap size so free/realloc can work */
#define ALLOC_HEADER_SIZE 16  /* keep 16-byte aligned */
#define PAGE_ALIGN(x) (((x) + 4095UL) & ~4095UL)

void *mmap(void *addr, size_t length, int prot, int flags, int fd, long offset)
{
    long r = __syscall6(SYS_mmap, (long)addr, (long)length, (long)prot,
                        (long)flags, (long)fd, offset);
    if (r < 0 && r > -4096) { errno = (int)(-r); return MAP_FAILED; }
    return (void *)r;
}

int munmap(void *addr, size_t length)
{
    return (int)__set_errno(__syscall2(SYS_munmap, (long)addr, (long)length));
}

int mprotect(void *addr, size_t len, int prot)
{
    return (int)__set_errno(__syscall3(SYS_mprotect, (long)addr, (long)len, (long)prot));
}

int msync(void *addr, size_t length, int flags)
{
    return (int)__set_errno(__syscall3(SYS_msync, (long)addr, (long)length, (long)flags));
}

void *malloc(size_t size)
{
    if (size == 0) size = 1;
    size_t total = PAGE_ALIGN(size + ALLOC_HEADER_SIZE);
    void *p = mmap(NULL, total, PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (p == MAP_FAILED) return NULL;
    /* Store the total allocation size in the header */
    *(size_t *)p = total;
    return (char *)p + ALLOC_HEADER_SIZE;
}

void free(void *ptr)
{
    if (!ptr) return;
    void *base = (char *)ptr - ALLOC_HEADER_SIZE;
    size_t total = *(size_t *)base;
    munmap(base, total);
}

void *calloc(size_t n, size_t size)
{
    size_t total = n * size;
    void *p = malloc(total);
    if (p) memset(p, 0, total);
    return p;
}

void *realloc(void *ptr, size_t size)
{
    if (!ptr) return malloc(size);
    if (size == 0) { free(ptr); return NULL; }

    void *base = (char *)ptr - ALLOC_HEADER_SIZE;
    size_t old_total = *(size_t *)base;
    size_t old_usable = old_total - ALLOC_HEADER_SIZE;
    size_t new_total = PAGE_ALIGN(size + ALLOC_HEADER_SIZE);

    /* If already large enough, no need to reallocate */
    if (new_total <= old_total) return ptr;

    void *new_ptr = malloc(size);
    if (!new_ptr) return NULL;
    memcpy(new_ptr, ptr, old_usable < size ? old_usable : size);
    free(ptr);
    return new_ptr;
}

/* ===================================================================
 *  SECTION 5: I/O -- read, write, printf family (unistd.h, stdio.h)
 * =================================================================== */

ssize_t read(int fd, void *buf, size_t count)
{
    return (ssize_t)__set_errno(__syscall3(SYS_read, fd, (long)buf, (long)count));
}

ssize_t write(int fd, const void *buf, size_t count)
{
    return (ssize_t)__set_errno(__syscall3(SYS_write, fd, (long)buf, (long)count));
}

int close(int fd)
{
    return (int)__set_errno(__syscall1(SYS_close, fd));
}

off_t lseek(int fd, off_t offset, int whence)
{
    return (off_t)__set_errno(__syscall3(SYS_lseek, fd, (long)offset, whence));
}

/* --- Core vsnprintf implementation --- */

/* Helper: write a single char to buffer with bounds checking */
static int __buf_putc(char *buf, size_t size, size_t pos, char c)
{
    if (buf && pos < size) buf[pos] = c;
    return 1;
}

/* Helper: write a string to buffer */
static int __buf_puts(char *buf, size_t size, size_t pos, const char *s)
{
    int n = 0;
    while (*s) {
        __buf_putc(buf, size, pos + n, *s++);
        n++;
    }
    return n;
}

/* Helper: output an unsigned 64-bit number in a given base */
static int __fmt_uint64(char *buf, size_t size, size_t pos,
                        unsigned long long val, int base, int uppercase,
                        int width, int zero_pad, int left_align)
{
    char tmp[24];
    const char *digits = uppercase ? "0123456789ABCDEF" : "0123456789abcdef";
    int i = 0, n = 0;

    if (val == 0) { tmp[i++] = '0'; }
    else {
        while (val) { tmp[i++] = digits[val % base]; val /= base; }
    }

    int pad = width - i;
    if (!left_align) {
        char pc = zero_pad ? '0' : ' ';
        while (pad-- > 0) { __buf_putc(buf, size, pos + n, pc); n++; }
    }
    while (i--) { __buf_putc(buf, size, pos + n, tmp[i]); n++; }
    if (left_align) {
        while (pad-- > 0) { __buf_putc(buf, size, pos + n, ' '); n++; }
    }
    return n;
}

/* Helper: output a signed 64-bit number */
static int __fmt_int64(char *buf, size_t size, size_t pos,
                       long long val, int width, int zero_pad, int left_align)
{
    int n = 0;
    if (val < 0) {
        __buf_putc(buf, size, pos + n, '-'); n++;
        if (width > 0) width--;
        /* Handle LLONG_MIN carefully: cannot negate it */
        if (val == (-9223372036854775807LL - 1LL)) {
            /* special case: LLONG_MIN */
            const char *s = "9223372036854775808";
            int slen = 19;
            int pad = width - slen;
            if (!left_align) {
                char pc = zero_pad ? '0' : ' ';
                while (pad-- > 0) { __buf_putc(buf, size, pos + n, pc); n++; }
            }
            n += __buf_puts(buf, size, pos + n, s);
            if (left_align) {
                while (pad-- > 0) { __buf_putc(buf, size, pos + n, ' '); n++; }
            }
            return n;
        }
        val = -val;
    }
    n += __fmt_uint64(buf, size, pos + n, (unsigned long long)val, 10, 0, width, zero_pad, left_align);
    return n;
}

int vsnprintf(char *buf, size_t size, const char *fmt, va_list ap)
{
    size_t pos = 0;
    int n;

    while (*fmt) {
        if (*fmt != '%') {
            __buf_putc(buf, size, pos, *fmt);
            pos++;
            fmt++;
            continue;
        }
        fmt++; /* skip '%' */

        /* Flags */
        int zero_pad = 0;
        int left_align = 0;
        int alt_form = 0;
        int show_sign = 0;
        int space_sign = 0;
        while (1) {
            if (*fmt == '0') { zero_pad = 1; fmt++; }
            else if (*fmt == '-') { left_align = 1; fmt++; }
            else if (*fmt == '#') { alt_form = 1; fmt++; }
            else if (*fmt == '+') { show_sign = 1; fmt++; }
            else if (*fmt == ' ') { space_sign = 1; fmt++; }
            else break;
        }
        if (left_align) zero_pad = 0;

        /* Width */
        int width = 0;
        if (*fmt == '*') { width = va_arg(ap, int); fmt++; }
        else { while (*fmt >= '0' && *fmt <= '9') { width = width * 10 + (*fmt - '0'); fmt++; } }

        /* Precision */
        int precision = -1;
        if (*fmt == '.') {
            fmt++;
            precision = 0;
            if (*fmt == '*') { precision = va_arg(ap, int); fmt++; }
            else { while (*fmt >= '0' && *fmt <= '9') { precision = precision * 10 + (*fmt - '0'); fmt++; } }
        }

        /* Length modifier */
        int is_long = 0, is_longlong = 0, is_size = 0, is_short = 0;
        if (*fmt == 'l') {
            fmt++;
            if (*fmt == 'l') { is_longlong = 1; fmt++; }
            else { is_long = 1; }
        } else if (*fmt == 'z') { is_size = 1; fmt++; }
        else if (*fmt == 'h') {
            fmt++;
            if (*fmt == 'h') { is_short = 2; fmt++; } /* hh */
            else { is_short = 1; }
        }
        else if (*fmt == 'j') { is_longlong = 1; fmt++; }
        else if (*fmt == 't') { is_long = 1; fmt++; }

        (void)alt_form; (void)show_sign; (void)space_sign;
        (void)is_short;

        switch (*fmt) {
        case 'd': case 'i': {
            long long val;
            if (is_longlong)     val = va_arg(ap, long long);
            else if (is_long || is_size) val = va_arg(ap, long);
            else                 val = va_arg(ap, int);
            n = __fmt_int64(buf, size, pos, val, width, zero_pad, left_align);
            pos += n;
            break;
        }
        case 'u': {
            unsigned long long val;
            if (is_longlong)     val = va_arg(ap, unsigned long long);
            else if (is_long || is_size) val = (unsigned long long)va_arg(ap, unsigned long);
            else                 val = va_arg(ap, unsigned int);
            n = __fmt_uint64(buf, size, pos, val, 10, 0, width, zero_pad, left_align);
            pos += n;
            break;
        }
        case 'x': case 'X': {
            unsigned long long val;
            if (is_longlong)     val = va_arg(ap, unsigned long long);
            else if (is_long || is_size) val = (unsigned long long)va_arg(ap, unsigned long);
            else                 val = va_arg(ap, unsigned int);
            n = __fmt_uint64(buf, size, pos, val, 16, (*fmt == 'X'), width, zero_pad, left_align);
            pos += n;
            break;
        }
        case 'o': {
            unsigned long long val;
            if (is_longlong)     val = va_arg(ap, unsigned long long);
            else if (is_long || is_size) val = (unsigned long long)va_arg(ap, unsigned long);
            else                 val = va_arg(ap, unsigned int);
            n = __fmt_uint64(buf, size, pos, val, 8, 0, width, zero_pad, left_align);
            pos += n;
            break;
        }
        case 'p': {
            unsigned long long val = (unsigned long long)(uintptr_t)va_arg(ap, void *);
            __buf_putc(buf, size, pos, '0'); pos++;
            __buf_putc(buf, size, pos, 'x'); pos++;
            n = __fmt_uint64(buf, size, pos, val, 16, 0, 0, 0, 0);
            pos += n;
            break;
        }
        case 's': {
            const char *s = va_arg(ap, const char *);
            if (!s) s = "(null)";
            int slen = (int)strlen(s);
            if (precision >= 0 && precision < slen) slen = precision;
            int pad = width - slen;
            if (!left_align) while (pad-- > 0) { __buf_putc(buf, size, pos, ' '); pos++; }
            for (int i = 0; i < slen; i++) { __buf_putc(buf, size, pos, s[i]); pos++; }
            if (left_align) while (pad-- > 0) { __buf_putc(buf, size, pos, ' '); pos++; }
            break;
        }
        case 'c': {
            char c = (char)va_arg(ap, int);
            int pad = width - 1;
            if (!left_align) while (pad-- > 0) { __buf_putc(buf, size, pos, ' '); pos++; }
            __buf_putc(buf, size, pos, c); pos++;
            if (left_align) while (pad-- > 0) { __buf_putc(buf, size, pos, ' '); pos++; }
            break;
        }
        case 'f': case 'F': {
            /* Simple float formatting: integer part + up to 6 decimal digits */
            double val = va_arg(ap, double);
            if (precision < 0) precision = 6;
            if (val < 0) { __buf_putc(buf, size, pos, '-'); pos++; val = -val; }
            unsigned long long ipart = (unsigned long long)val;
            n = __fmt_uint64(buf, size, pos, ipart, 10, 0, 0, 0, 0);
            pos += n;
            if (precision > 0) {
                __buf_putc(buf, size, pos, '.'); pos++;
                double frac = val - (double)ipart;
                for (int i = 0; i < precision; i++) {
                    frac *= 10.0;
                    int digit = (int)frac;
                    __buf_putc(buf, size, pos, '0' + digit); pos++;
                    frac -= digit;
                }
            }
            break;
        }
        case 'e': case 'E': case 'g': case 'G': {
            /* Fallback: just print as %f */
            double val = va_arg(ap, double);
            if (precision < 0) precision = 6;
            if (val < 0) { __buf_putc(buf, size, pos, '-'); pos++; val = -val; }
            unsigned long long ipart = (unsigned long long)val;
            n = __fmt_uint64(buf, size, pos, ipart, 10, 0, 0, 0, 0);
            pos += n;
            if (precision > 0) {
                __buf_putc(buf, size, pos, '.'); pos++;
                double frac = val - (double)ipart;
                for (int i = 0; i < precision; i++) {
                    frac *= 10.0;
                    int digit = (int)frac;
                    __buf_putc(buf, size, pos, '0' + digit); pos++;
                    frac -= digit;
                }
            }
            break;
        }
        case '%':
            __buf_putc(buf, size, pos, '%'); pos++;
            break;
        case 'n': {
            int *np = va_arg(ap, int *);
            if (np) *np = (int)pos;
            break;
        }
        case '\0':
            goto done;
        default:
            /* Unknown specifier -- just emit it */
            __buf_putc(buf, size, pos, '%'); pos++;
            __buf_putc(buf, size, pos, *fmt); pos++;
            break;
        }
        fmt++;
    }
done:
    if (buf) {
        if (pos < size) buf[pos] = '\0';
        else if (size > 0) buf[size - 1] = '\0';
    }
    return (int)pos;
}

int snprintf(char *buf, size_t size, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(buf, size, fmt, ap);
    va_end(ap);
    return n;
}

int vsprintf(char *buf, const char *fmt, va_list ap)
{
    return vsnprintf(buf, (size_t)-1, fmt, ap);
}

int sprintf(char *buf, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(buf, (size_t)-1, fmt, ap);
    va_end(ap);
    return n;
}

int vprintf(const char *fmt, va_list ap)
{
    char buf[4096];
    int n = vsnprintf(buf, sizeof(buf), fmt, ap);
    if (n > 0) write(1, buf, n < (int)sizeof(buf) ? n : (int)sizeof(buf));
    return n;
}

int printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vprintf(fmt, ap);
    va_end(ap);
    return n;
}

/* FILE stubs -- we only have fd-based I/O */
struct _FILE { int fd; int eof; int err; int ungot; };
static struct _FILE __stdin_file  = { 0, 0, 0, -1 };
static struct _FILE __stdout_file = { 1, 0, 0, -1 };
static struct _FILE __stderr_file = { 2, 0, 0, -1 };

FILE *stdin  = &__stdin_file;
FILE *stdout = &__stdout_file;
FILE *stderr = &__stderr_file;

static int __file_fd(FILE *f)
{
    if (f == stdin)  return 0;
    if (f == stdout) return 1;
    if (f == stderr) return 2;
    if (f) return f->fd;
    return 1;
}

int vfprintf(FILE *stream, const char *fmt, va_list ap)
{
    char buf[4096];
    int n = vsnprintf(buf, sizeof(buf), fmt, ap);
    int fd = __file_fd(stream);
    if (n > 0) write(fd, buf, n < (int)sizeof(buf) ? n : (int)sizeof(buf));
    return n;
}

int fprintf(FILE *stream, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vfprintf(stream, fmt, ap);
    va_end(ap);
    return n;
}

int puts(const char *s)
{
    int n = (int)strlen(s);
    write(1, s, n);
    write(1, "\n", 1);
    return n + 1;
}

int fputs(const char *s, FILE *stream)
{
    int fd = __file_fd(stream);
    int n = (int)strlen(s);
    write(fd, s, n);
    return n;
}

int putchar(int c)
{
    char ch = (char)c;
    write(1, &ch, 1);
    return c;
}

int fputc(int c, FILE *stream)
{
    char ch = (char)c;
    write(__file_fd(stream), &ch, 1);
    return (unsigned char)c;
}

int putc(int c, FILE *stream)   { return fputc(c, stream); }
int getchar(void)               { return fgetc(stdin); }
int getc(FILE *stream)          { return fgetc(stream); }

int fgetc(FILE *stream)
{
    if (stream && stream->ungot >= 0) {
        int c = stream->ungot;
        stream->ungot = -1;
        return c;
    }
    unsigned char c;
    ssize_t r = read(__file_fd(stream), &c, 1);
    if (r <= 0) { if (stream) stream->eof = 1; return EOF; }
    return (int)c;
}

int ungetc(int c, FILE *stream)
{
    if (c == EOF || !stream) return EOF;
    stream->ungot = c;
    return c;
}

char *fgets(char *buf, int size, FILE *stream)
{
    if (size <= 0) return NULL;
    int i = 0;
    while (i < size - 1) {
        int c = fgetc(stream);
        if (c == EOF) { if (i == 0) return NULL; break; }
        buf[i++] = (char)c;
        if (c == '\n') break;
    }
    buf[i] = '\0';
    return buf;
}

/* Formatted input stubs */
int scanf(const char *fmt, ...)      { (void)fmt; return 0; }
int fscanf(FILE *stream, const char *fmt, ...) { (void)stream; (void)fmt; return 0; }
int sscanf(const char *buf, const char *fmt, ...) { (void)buf; (void)fmt; return 0; }

/* File operations -- return stubs */
FILE *fopen(const char *path, const char *mode)  { (void)path; (void)mode; return NULL; }
FILE *freopen(const char *path, const char *mode, FILE *stream) { (void)path; (void)mode; (void)stream; return NULL; }
int fclose(FILE *stream) { (void)stream; return 0; }
int fflush(FILE *stream) { (void)stream; return 0; }

size_t fread(void *buf, size_t size, size_t count, FILE *stream)
{
    (void)buf; (void)size; (void)count; (void)stream;
    return 0;
}

size_t fwrite(const void *buf, size_t size, size_t count, FILE *stream)
{
    if (!stream) return 0;
    int fd = __file_fd(stream);
    size_t total = size * count;
    ssize_t r = write(fd, buf, total);
    if (r < 0) return 0;
    return (size_t)r / size;
}

int fseek(FILE *stream, long offset, int whence)
{
    (void)stream; (void)offset; (void)whence;
    return -1;
}

long ftell(FILE *stream) { (void)stream; return -1; }
void rewind(FILE *stream) { (void)stream; }
int feof(FILE *stream) { return stream ? stream->eof : 1; }
int ferror(FILE *stream) { return stream ? stream->err : 0; }
void clearerr(FILE *stream) { if (stream) { stream->eof = 0; stream->err = 0; } }

int remove(const char *path)
{
    return (int)__set_errno(__syscall3(SYS_unlinkat, AT_FDCWD, (long)path, 0));
}

int rename(const char *oldpath, const char *newpath)
{
    return (int)__set_errno(__syscall5(SYS_renameat2, AT_FDCWD, (long)oldpath,
                                      AT_FDCWD, (long)newpath, 0));
}

void perror(const char *s)
{
    if (s && *s) { write(2, s, strlen(s)); write(2, ": ", 2); }
    const char *msg = "error\n";
    write(2, msg, 6);
}

FILE *tmpfile(void) { return NULL; }
char *tmpnam(char *s) { (void)s; return NULL; }

/* ===================================================================
 *  SECTION 6: Process (stdlib.h)
 * =================================================================== */

void exit(int status)
{
    /* Run atexit handlers in reverse */
    extern void __run_atexit(void);
    __run_atexit();
    __syscall1(SYS_exit_group, status);
    __builtin_unreachable();
}

void abort(void)
{
    exit(134);
}

static void (*__atexit_funcs[32])(void);
static int __atexit_count = 0;

int atexit(void (*func)(void))
{
    if (__atexit_count >= 32) return -1;
    __atexit_funcs[__atexit_count++] = func;
    return 0;
}

void __run_atexit(void)
{
    while (__atexit_count > 0) {
        __atexit_count--;
        if (__atexit_funcs[__atexit_count])
            __atexit_funcs[__atexit_count]();
    }
}

int system(const char *command) { (void)command; return -1; }

/* ===================================================================
 *  SECTION 7: Number conversion (stdlib.h)
 * =================================================================== */

long strtol(const char *s, char **endp, int base)
{
    const char *p = s;
    long result = 0;
    int neg = 0;

    /* skip whitespace */
    while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;

    /* sign */
    if (*p == '-') { neg = 1; p++; }
    else if (*p == '+') { p++; }

    /* auto-detect base */
    if (base == 0) {
        if (*p == '0') {
            p++;
            if (*p == 'x' || *p == 'X') { base = 16; p++; }
            else { base = 8; }
        } else { base = 10; }
    } else if (base == 16 && p[0] == '0' && (p[1] == 'x' || p[1] == 'X')) {
        p += 2;
    }

    while (*p) {
        int digit;
        if (*p >= '0' && *p <= '9') digit = *p - '0';
        else if (*p >= 'a' && *p <= 'f') digit = *p - 'a' + 10;
        else if (*p >= 'A' && *p <= 'F') digit = *p - 'A' + 10;
        else break;
        if (digit >= base) break;
        result = result * base + digit;
        p++;
    }

    if (endp) *endp = (char *)p;
    return neg ? -result : result;
}

unsigned long strtoul(const char *s, char **endp, int base)
{
    const char *p = s;
    unsigned long result = 0;

    while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;

    int neg = 0;
    if (*p == '-') { neg = 1; p++; }
    else if (*p == '+') { p++; }

    if (base == 0) {
        if (*p == '0') {
            p++;
            if (*p == 'x' || *p == 'X') { base = 16; p++; }
            else { base = 8; }
        } else { base = 10; }
    } else if (base == 16 && p[0] == '0' && (p[1] == 'x' || p[1] == 'X')) {
        p += 2;
    }

    while (*p) {
        int digit;
        if (*p >= '0' && *p <= '9') digit = *p - '0';
        else if (*p >= 'a' && *p <= 'f') digit = *p - 'a' + 10;
        else if (*p >= 'A' && *p <= 'F') digit = *p - 'A' + 10;
        else break;
        if (digit >= base) break;
        result = result * base + digit;
        p++;
    }

    if (endp) *endp = (char *)p;
    return neg ? (unsigned long)(-(long)result) : result;
}

long long strtoll(const char *s, char **endp, int base)
{
    return (long long)strtol(s, endp, base);
}

unsigned long long strtoull(const char *s, char **endp, int base)
{
    return (unsigned long long)strtoul(s, endp, base);
}

double strtod(const char *s, char **endp)
{
    const char *p = s;
    double result = 0.0;
    int neg = 0;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') { neg = 1; p++; }
    else if (*p == '+') { p++; }

    /* Integer part */
    while (*p >= '0' && *p <= '9') {
        result = result * 10.0 + (*p - '0');
        p++;
    }
    /* Fractional part */
    if (*p == '.') {
        p++;
        double frac = 0.1;
        while (*p >= '0' && *p <= '9') {
            result += (*p - '0') * frac;
            frac *= 0.1;
            p++;
        }
    }
    /* Exponent part */
    if (*p == 'e' || *p == 'E') {
        p++;
        int eneg = 0;
        int exp_val = 0;
        if (*p == '-') { eneg = 1; p++; }
        else if (*p == '+') { p++; }
        while (*p >= '0' && *p <= '9') { exp_val = exp_val * 10 + (*p - '0'); p++; }
        double mult = 1.0;
        for (int i = 0; i < exp_val; i++) mult *= 10.0;
        if (eneg) result /= mult; else result *= mult;
    }

    if (endp) *endp = (char *)p;
    return neg ? -result : result;
}

float strtof(const char *s, char **endp)
{
    return (float)strtod(s, endp);
}

int atoi(const char *s)   { return (int)strtol(s, NULL, 10); }
long atol(const char *s)  { return strtol(s, NULL, 10); }
double atof(const char *s) { return strtod(s, NULL); }

intmax_t strtoimax(const char *s, char **endp, int base)
{
    return (intmax_t)strtol(s, endp, base);
}

uintmax_t strtoumax(const char *s, char **endp, int base)
{
    return (uintmax_t)strtoul(s, endp, base);
}

int abs(int x)             { return x < 0 ? -x : x; }
long labs(long x)          { return x < 0 ? -x : x; }
long long llabs(long long x) { return x < 0 ? -x : x; }

/* ===================================================================
 *  SECTION 8: ctype.h
 * =================================================================== */

int isalpha(int c)  { return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'); }
int isdigit(int c)  { return c >= '0' && c <= '9'; }
int isalnum(int c)  { return isalpha(c) || isdigit(c); }
int isspace(int c)  { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v'; }
int isupper(int c)  { return c >= 'A' && c <= 'Z'; }
int islower(int c)  { return c >= 'a' && c <= 'z'; }
int isprint(int c)  { return c >= 0x20 && c <= 0x7e; }
int isgraph(int c)  { return c > 0x20 && c <= 0x7e; }
int ispunct(int c)  { return isgraph(c) && !isalnum(c); }
int iscntrl(int c)  { return (c >= 0 && c < 0x20) || c == 0x7f; }
int isxdigit(int c) { return isdigit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F'); }
int isblank(int c)  { return c == ' ' || c == '\t'; }
int isascii(int c)  { return (unsigned)c <= 0x7f; }

int toupper(int c)  { return (c >= 'a' && c <= 'z') ? c - 32 : c; }
int tolower(int c)  { return (c >= 'A' && c <= 'Z') ? c + 32 : c; }

/* ===================================================================
 *  SECTION 9: Math (soft-float, no libm)
 * =================================================================== */

double fabs(double x)  { return x < 0 ? -x : x; }
float fabsf(float x)   { return x < 0 ? -x : x; }

double floor(double x)
{
    long i = (long)x;
    if (x < 0 && x != (double)i) return (double)(i - 1);
    return (double)i;
}

float floorf(float x)
{
    long i = (long)x;
    if (x < 0 && x != (float)i) return (float)(i - 1);
    return (float)i;
}

double ceil(double x)
{
    long i = (long)x;
    if (x > 0 && x != (double)i) return (double)(i + 1);
    return (double)i;
}

float ceilf(float x)
{
    long i = (long)x;
    if (x > 0 && x != (float)i) return (float)(i + 1);
    return (float)i;
}

double round(double x)
{
    return x >= 0 ? floor(x + 0.5) : ceil(x - 0.5);
}

float roundf(float x)
{
    return x >= 0 ? floorf(x + 0.5f) : ceilf(x - 0.5f);
}

double trunc(double x)
{
    return (double)(long)x;
}

float truncf(float x)
{
    return (float)(long)x;
}

double rint(double x)
{
    return round(x);
}

long lround(double x) { return (long)round(x); }
long long llround(double x) { return (long long)round(x); }
long lrint(double x) { return (long)rint(x); }

double fmod(double x, double y)
{
    if (y == 0.0) return x;
    return x - floor(x / y) * y;
}

float fmodf(float x, float y)
{
    if (y == 0.0f) return x;
    return x - floorf(x / y) * y;
}

double remainder(double x, double y) { return fmod(x, y); }

double fmin(double x, double y) { return x < y ? x : y; }
double fmax(double x, double y) { return x > y ? x : y; }
float fminf(float x, float y) { return x < y ? x : y; }
float fmaxf(float x, float y) { return x > y ? x : y; }

double copysign(double x, double y)
{
    double ax = fabs(x);
    return y < 0 ? -ax : ax;
}

float copysignf(float x, float y)
{
    float ax = fabsf(x);
    return y < 0 ? -ax : ax;
}

double modf(double x, double *iptr)
{
    double i = trunc(x);
    if (iptr) *iptr = i;
    return x - i;
}

double sqrt(double x)
{
    if (x < 0) return 0.0; /* NaN in real life */
    if (x == 0) return 0.0;
    double guess = x * 0.5;
    for (int i = 0; i < 50; i++) {
        guess = 0.5 * (guess + x / guess);
    }
    return guess;
}

float sqrtf(float x)
{
    return (float)sqrt((double)x);
}

double cbrt(double x)
{
    if (x == 0) return 0;
    int neg = (x < 0);
    if (neg) x = -x;
    double guess = x / 3.0;
    for (int i = 0; i < 50; i++) {
        guess = (2.0 * guess + x / (guess * guess)) / 3.0;
    }
    return neg ? -guess : guess;
}

float cbrtf(float x) { return (float)cbrt((double)x); }

double hypot(double x, double y) { return sqrt(x * x + y * y); }
float hypotf(float x, float y) { return sqrtf(x * x + y * y); }

double pow(double base, double exponent)
{
    if (exponent == 0.0) return 1.0;
    if (base == 0.0) return 0.0;
    if (base == 1.0) return 1.0;

    /* Integer exponent fast path */
    int iexp = (int)exponent;
    if ((double)iexp == exponent) {
        int neg = 0;
        if (iexp < 0) { neg = 1; iexp = -iexp; }
        double result = 1.0;
        double b = base;
        while (iexp > 0) {
            if (iexp & 1) result *= b;
            b *= b;
            iexp >>= 1;
        }
        return neg ? 1.0 / result : result;
    }

    /* For non-integer exponents: exp(exponent * log(base)) */
    /* Using our stub log/exp -- limited accuracy */
    return exp(exponent * log(base));
}

float powf(float base, float exponent) { return (float)pow((double)base, (double)exponent); }

/* Logarithm -- series expansion: log(x) for x > 0 */
double log(double x)
{
    if (x <= 0.0) return -HUGE_VAL;
    if (x == 1.0) return 0.0;

    /* Reduce: x = m * 2^e, where 0.5 <= m < 1.0 */
    int e = 0;
    double m = x;
    while (m >= 2.0) { m *= 0.5; e++; }
    while (m < 0.5) { m *= 2.0; e--; }

    /* log(x) = e * log(2) + log(m) */
    /* log(m): use series around 1: log(1+t) = t - t^2/2 + t^3/3 - ... */
    double t = (m - 1.0) / (m + 1.0);
    double t2 = t * t;
    double sum = 0.0;
    double term = t;
    for (int i = 0; i < 30; i++) {
        sum += term / (2 * i + 1);
        term *= t2;
    }
    sum *= 2.0;

    return sum + (double)e * 0.693147180559945309;
}

float logf(float x)     { return (float)log((double)x); }
double log2(double x)   { return log(x) * 1.44269504088896341; }
float log2f(float x)    { return (float)log2((double)x); }
double log10(double x)  { return log(x) * 0.434294481903251828; }
float log10f(float x)   { return (float)log10((double)x); }

double exp(double x)
{
    if (x == 0.0) return 1.0;
    if (x > 709.0) return HUGE_VAL;
    if (x < -709.0) return 0.0;

    /* exp(x) = 1 + x + x^2/2! + x^3/3! + ... */
    double sum = 1.0;
    double term = 1.0;
    for (int i = 1; i < 60; i++) {
        term *= x / i;
        sum += term;
        if (fabs(term) < 1e-15 * fabs(sum)) break;
    }
    return sum;
}

float expf(float x) { return (float)exp((double)x); }
double exp2(double x) { return pow(2.0, x); }
float exp2f(float x) { return (float)exp2((double)x); }

double frexp(double x, int *e)
{
    if (x == 0.0) { *e = 0; return 0.0; }
    *e = 0;
    double m = x < 0 ? -x : x;
    while (m >= 1.0) { m *= 0.5; (*e)++; }
    while (m < 0.5) { m *= 2.0; (*e)--; }
    return x < 0 ? -m : m;
}

double ldexp(double x, int e)
{
    double r = x;
    if (e > 0) { while (e--) r *= 2.0; }
    else       { while (e++) r *= 0.5; }
    return r;
}

double scalbn(double x, int n) { return ldexp(x, n); }

int ilogb(double x)
{
    if (x == 0.0) return INT_MIN;
    int e;
    frexp(x, &e);
    return e - 1;
}

double logb(double x) { return (double)ilogb(x); }

/* Trigonometric -- Taylor series */
static double __reduce_angle(double x)
{
    /* Reduce x to [-pi, pi] */
    while (x > M_PI) x -= 2.0 * M_PI;
    while (x < -M_PI) x += 2.0 * M_PI;
    return x;
}

double sin(double x)
{
    x = __reduce_angle(x);
    double sum = 0.0, term = x;
    for (int i = 0; i < 20; i++) {
        sum += term;
        term *= -x * x / ((2 * i + 2) * (2 * i + 3));
    }
    return sum;
}

double cos(double x)
{
    x = __reduce_angle(x);
    double sum = 0.0, term = 1.0;
    for (int i = 0; i < 20; i++) {
        sum += term;
        term *= -x * x / ((2 * i + 1) * (2 * i + 2));
    }
    return sum;
}

double tan(double x)
{
    double c = cos(x);
    if (c == 0.0) return HUGE_VAL;
    return sin(x) / c;
}

float sinf(float x)  { return (float)sin((double)x); }
float cosf(float x)  { return (float)cos((double)x); }
float tanf(float x)  { return (float)tan((double)x); }

double asin(double x)
{
    /* Newton's method on sin(y) = x */
    if (x >= 1.0) return M_PI_2;
    if (x <= -1.0) return -M_PI_2;
    double y = x;
    for (int i = 0; i < 30; i++) {
        double sy = sin(y);
        double cy = cos(y);
        if (fabs(cy) < 1e-15) break;
        y -= (sy - x) / cy;
    }
    return y;
}

double acos(double x) { return M_PI_2 - asin(x); }

double atan(double x)
{
    return asin(x / sqrt(1.0 + x * x));
}

double atan2(double y, double x)
{
    if (x > 0.0) return atan(y / x);
    if (x < 0.0 && y >= 0.0) return atan(y / x) + M_PI;
    if (x < 0.0 && y < 0.0) return atan(y / x) - M_PI;
    if (x == 0.0 && y > 0.0) return M_PI_2;
    if (x == 0.0 && y < 0.0) return -M_PI_2;
    return 0.0;
}

float asinf(float x)  { return (float)asin((double)x); }
float acosf(float x)  { return (float)acos((double)x); }
float atanf(float x)  { return (float)atan((double)x); }
float atan2f(float y, float x) { return (float)atan2((double)y, (double)x); }

/* Hyperbolic */
double sinh(double x) { return (exp(x) - exp(-x)) * 0.5; }
double cosh(double x) { return (exp(x) + exp(-x)) * 0.5; }
double tanh(double x)
{
    double ep = exp(x), em = exp(-x);
    return (ep - em) / (ep + em);
}

/* ===================================================================
 *  SECTION 10: Time (time.h, sys/time.h)
 * =================================================================== */

int clock_gettime(int clk_id, struct timespec *tp)
{
    return (int)__set_errno(__syscall2(SYS_clock_gettime, clk_id, (long)tp));
}

time_t time(time_t *t)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) < 0) return (time_t)-1;
    if (t) *t = ts.tv_sec;
    return ts.tv_sec;
}

clock_t clock(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * CLOCKS_PER_SEC + ts.tv_nsec / (1000000000L / CLOCKS_PER_SEC);
}

int gettimeofday(struct timeval *tv, struct timezone *tz)
{
    (void)tz;
    struct timespec ts;
    int r = clock_gettime(CLOCK_REALTIME, &ts);
    if (r < 0) return r;
    if (tv) { tv->tv_sec = ts.tv_sec; tv->tv_usec = ts.tv_nsec / 1000; }
    return 0;
}

int settimeofday(const struct timeval *tv, const struct timezone *tz)
{
    (void)tv; (void)tz;
    return -1;
}

double difftime(time_t t1, time_t t0) { return (double)(t1 - t0); }

int nanosleep(const struct timespec *req, struct timespec *rem)
{
    return (int)__set_errno(__syscall2(SYS_nanosleep, (long)req, (long)rem));
}

unsigned int sleep(unsigned int seconds)
{
    struct timespec req = { .tv_sec = seconds, .tv_nsec = 0 };
    struct timespec rem = { 0, 0 };
    nanosleep(&req, &rem);
    return (unsigned int)rem.tv_sec;
}

int usleep(unsigned int usec)
{
    struct timespec req = { .tv_sec = usec / 1000000, .tv_nsec = (usec % 1000000) * 1000L };
    return nanosleep(&req, NULL);
}

/* gmtime -- break time_t into struct tm (UTC) */
static struct tm __gmtime_buf;

static int __is_leap_year(int year)
{
    return (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0));
}

static const int __month_days[12] = { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 };

struct tm *gmtime_r(const time_t *t, struct tm *result)
{
    time_t secs = *t;
    int days = (int)(secs / 86400);
    int rem  = (int)(secs % 86400);
    if (rem < 0) { days--; rem += 86400; }

    result->tm_hour = rem / 3600;
    result->tm_min  = (rem % 3600) / 60;
    result->tm_sec  = rem % 60;

    /* Day of week: 1970-01-01 was Thursday (4) */
    result->tm_wday = (4 + days) % 7;
    if (result->tm_wday < 0) result->tm_wday += 7;

    int year = 1970;
    while (1) {
        int yday = __is_leap_year(year) ? 366 : 365;
        if (days < yday) break;
        days -= yday;
        year++;
    }
    result->tm_year = year - 1900;
    result->tm_yday = days;

    int leap = __is_leap_year(year);
    int mon;
    for (mon = 0; mon < 12; mon++) {
        int md = __month_days[mon] + (mon == 1 && leap ? 1 : 0);
        if (days < md) break;
        days -= md;
    }
    result->tm_mon = mon;
    result->tm_mday = days + 1;
    result->tm_isdst = 0;
    return result;
}

struct tm *gmtime(const time_t *t)
{
    return gmtime_r(t, &__gmtime_buf);
}

struct tm *localtime_r(const time_t *t, struct tm *result)
{
    return gmtime_r(t, result); /* No timezone support, treat as UTC */
}

struct tm *localtime(const time_t *t)
{
    return gmtime(t);
}

time_t mktime(struct tm *tm)
{
    int year = tm->tm_year + 1900;
    int mon  = tm->tm_mon;
    int day  = tm->tm_mday;

    time_t result = 0;
    for (int y = 1970; y < year; y++)
        result += __is_leap_year(y) ? 366 * 86400L : 365 * 86400L;
    for (int m = 0; m < mon; m++)
        result += (__month_days[m] + (m == 1 && __is_leap_year(year) ? 1 : 0)) * 86400L;
    result += (day - 1) * 86400L;
    result += tm->tm_hour * 3600L + tm->tm_min * 60L + tm->tm_sec;
    return result;
}

size_t strftime(char *buf, size_t maxsize, const char *fmt, const struct tm *tm)
{
    /* Minimal: just handle a few common format specifiers */
    size_t pos = 0;
    while (*fmt && pos < maxsize - 1) {
        if (*fmt != '%') { buf[pos++] = *fmt++; continue; }
        fmt++;
        char tmp[32];
        int n = 0;
        switch (*fmt) {
        case 'Y': n = snprintf(tmp, sizeof(tmp), "%04d", tm->tm_year + 1900); break;
        case 'm': n = snprintf(tmp, sizeof(tmp), "%02d", tm->tm_mon + 1); break;
        case 'd': n = snprintf(tmp, sizeof(tmp), "%02d", tm->tm_mday); break;
        case 'H': n = snprintf(tmp, sizeof(tmp), "%02d", tm->tm_hour); break;
        case 'M': n = snprintf(tmp, sizeof(tmp), "%02d", tm->tm_min); break;
        case 'S': n = snprintf(tmp, sizeof(tmp), "%02d", tm->tm_sec); break;
        case '%': tmp[0] = '%'; n = 1; break;
        default: tmp[0] = '%'; tmp[1] = *fmt; n = 2; break;
        }
        for (int i = 0; i < n && pos < maxsize - 1; i++) buf[pos++] = tmp[i];
        fmt++;
    }
    buf[pos] = '\0';
    return pos;
}

static char __asctime_buf[26];

char *asctime(const struct tm *tm)
{
    static const char *wday[] = { "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" };
    static const char *mon[] = { "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec" };
    snprintf(__asctime_buf, sizeof(__asctime_buf), "%.3s %.3s%3d %02d:%02d:%02d %d\n",
             wday[tm->tm_wday], mon[tm->tm_mon], tm->tm_mday,
             tm->tm_hour, tm->tm_min, tm->tm_sec, tm->tm_year + 1900);
    return __asctime_buf;
}

char *ctime(const time_t *t)
{
    return asctime(localtime(t));
}

/* ===================================================================
 *  SECTION 11: qsort, bsearch, rand (stdlib.h)
 * =================================================================== */

/* Simple insertion sort -- good enough for moderate N */
void qsort(void *base, size_t n, size_t size, int (*cmp)(const void *, const void *))
{
    char *arr = (char *)base;
    char *tmp = (char *)malloc(size);
    if (!tmp) return;

    for (size_t i = 1; i < n; i++) {
        memcpy(tmp, arr + i * size, size);
        size_t j = i;
        while (j > 0 && cmp(arr + (j - 1) * size, tmp) > 0) {
            memcpy(arr + j * size, arr + (j - 1) * size, size);
            j--;
        }
        memcpy(arr + j * size, tmp, size);
    }
    free(tmp);
}

void *bsearch(const void *key, const void *base, size_t n, size_t size,
              int (*cmp)(const void *, const void *))
{
    const char *arr = (const char *)base;
    size_t lo = 0, hi = n;
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        int c = cmp(key, arr + mid * size);
        if (c == 0) return (void *)(arr + mid * size);
        if (c < 0) hi = mid;
        else lo = mid + 1;
    }
    return NULL;
}

static unsigned int __rand_seed = 1;

int rand(void)
{
    __rand_seed = __rand_seed * 1103515245 + 12345;
    return (int)((__rand_seed >> 16) & RAND_MAX);
}

void srand(unsigned int seed) { __rand_seed = seed; }

/* ===================================================================
 *  SECTION 12: File operations (fcntl.h, unistd.h, dirent.h, stat)
 * =================================================================== */

int open(const char *path, int flags, ...)
{
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        mode = (mode_t)va_arg(ap, int);
        va_end(ap);
    }
    return (int)__set_errno(__syscall4(SYS_openat, AT_FDCWD, (long)path, flags, mode));
}

int creat(const char *path, mode_t mode)
{
    return open(path, O_WRONLY | O_CREAT | O_TRUNC, mode);
}

int fcntl(int fd, int cmd, ...)
{
    va_list ap;
    va_start(ap, cmd);
    long arg = va_arg(ap, long);
    va_end(ap);
    return (int)__set_errno(__syscall3(SYS_fcntl, fd, cmd, arg));
}

int dup(int oldfd)
{
    return (int)__set_errno(__syscall1(SYS_dup, oldfd));
}

int dup2(int oldfd, int newfd)
{
    return (int)__set_errno(__syscall3(SYS_dup3, oldfd, newfd, 0));
}

int pipe(int pipefd[2])
{
    return (int)__set_errno(__syscall2(SYS_pipe2, (long)pipefd, 0));
}

int unlink(const char *path)
{
    return (int)__set_errno(__syscall3(SYS_unlinkat, AT_FDCWD, (long)path, 0));
}

int rmdir(const char *path)
{
    /* AT_REMOVEDIR = 0x200 */
    return (int)__set_errno(__syscall3(SYS_unlinkat, AT_FDCWD, (long)path, 0x200));
}

int access(const char *path, int mode)
{
    /* Use faccessat (syscall 48) */
    return (int)__set_errno(__syscall3(48, AT_FDCWD, (long)path, mode));
}

int link(const char *oldpath, const char *newpath)
{
    return (int)__set_errno(__syscall5(SYS_linkat, AT_FDCWD, (long)oldpath,
                                      AT_FDCWD, (long)newpath, 0));
}

int symlink(const char *target, const char *linkpath)
{
    return (int)__set_errno(__syscall3(SYS_symlinkat, (long)target, AT_FDCWD, (long)linkpath));
}

ssize_t readlink(const char *path, char *buf, size_t bufsiz)
{
    return (ssize_t)__set_errno(__syscall4(SYS_readlinkat, AT_FDCWD, (long)path, (long)buf, (long)bufsiz));
}

int truncate(const char *path, off_t length)
{
    int fd = open(path, O_WRONLY);
    if (fd < 0) return -1;
    int r = ftruncate(fd, length);
    close(fd);
    return r;
}

int ftruncate(int fd, off_t length)
{
    return (int)__set_errno(__syscall2(SYS_ftruncate, fd, length));
}

int fsync(int fd)
{
    return (int)__set_errno(__syscall1(SYS_fsync, fd));
}

int stat(const char *path, struct stat *buf)
{
    /* Use fstatat (syscall 79) */
    return (int)__set_errno(__syscall4(SYS_fstatat, AT_FDCWD, (long)path, (long)buf, 0));
}

int fstat(int fd, struct stat *buf)
{
    return (int)__set_errno(__syscall2(SYS_fstat, fd, (long)buf));
}

int mkdir(const char *path, mode_t mode)
{
    return (int)__set_errno(__syscall3(SYS_mkdirat, AT_FDCWD, (long)path, mode));
}

char *getcwd(char *buf, size_t size)
{
    long r = __syscall2(SYS_getcwd, (long)buf, (long)size);
    if (r < 0) { errno = (int)(-r); return NULL; }
    return buf;
}

int chdir(const char *path)
{
    return (int)__set_errno(__syscall1(SYS_chdir, (long)path));
}

char *realpath(const char *path, char *resolved)
{
    /* Simple stub: just copy the path */
    if (!resolved) {
        resolved = (char *)malloc(PATH_MAX);
        if (!resolved) return NULL;
    }
    size_t len = strlen(path);
    if (len >= PATH_MAX) { errno = ENAMETOOLONG; return NULL; }
    memcpy(resolved, path, len + 1);
    return resolved;
}

/* dirent */
DIR *opendir(const char *name)  { (void)name; return NULL; }
struct dirent *readdir(DIR *dir) { (void)dir; return NULL; }
int closedir(DIR *dir) { (void)dir; return 0; }

/* ===================================================================
 *  SECTION 13: Process info (unistd.h)
 * =================================================================== */

pid_t getpid(void)   { return (pid_t)__syscall0(SYS_getpid); }
pid_t getppid(void)  { return (pid_t)__syscall0(SYS_getppid); }
uid_t getuid(void)   { return (uid_t)__syscall0(SYS_getuid); }
gid_t getgid(void)   { return (gid_t)__syscall0(SYS_getgid); }
int isatty(int fd)   { (void)fd; return 0; }

long sysconf(int name)
{
    if (name == _SC_PAGE_SIZE) return 4096;
    return -1;
}

/* ===================================================================
 *  SECTION 14: Locale (locale.h)
 * =================================================================== */

static char __locale_name[] = "C";
static struct lconv __lconv = {
    .decimal_point = ".",
    .thousands_sep = "",
    .grouping = "",
    .int_curr_symbol = "",
    .currency_symbol = "",
    .mon_decimal_point = "",
    .mon_thousands_sep = "",
    .mon_grouping = "",
    .positive_sign = "",
    .negative_sign = "",
    .int_frac_digits = 127,
    .frac_digits = 127,
    .p_cs_precedes = 127,
    .p_sep_by_space = 127,
    .n_cs_precedes = 127,
    .n_sep_by_space = 127,
    .p_sign_posn = 127,
    .n_sign_posn = 127,
};

char *setlocale(int category, const char *locale)
{
    (void)category; (void)locale;
    return __locale_name;
}

struct lconv *localeconv(void) { return &__lconv; }

/* ===================================================================
 *  SECTION 15: Environment (stdlib.h)
 * =================================================================== */

char *getenv(const char *name) { (void)name; return NULL; }

/* ===================================================================
 *  SECTION 16: Signal (signal.h)
 * =================================================================== */

static void (*__signal_handlers[64])(int);

void (*signal(int sig, void (*handler)(int)))(int)
{
    if (sig < 1 || sig >= 64) return SIG_ERR;
    void (*old)(int) = __signal_handlers[sig];
    __signal_handlers[sig] = handler;
    return old;
}

int sigaction(int sig, const struct sigaction *act, struct sigaction *oact)
{
    (void)sig; (void)act; (void)oact;
    return 0;
}

int kill(int pid, int sig)
{
    return (int)__set_errno(__syscall2(SYS_kill, pid, sig));
}

int raise(int sig)
{
    return kill(getpid(), sig);
}

int sigemptyset(sigset_t *set)
{
    memset(set, 0, sizeof(*set));
    return 0;
}

int sigfillset(sigset_t *set)
{
    memset(set, 0xff, sizeof(*set));
    return 0;
}

int sigaddset(sigset_t *set, int signum)
{
    if (signum < 1 || signum >= _NSIG) { errno = EINVAL; return -1; }
    set->__val[(signum - 1) / (8 * sizeof(unsigned long))] |= 1UL << ((signum - 1) % (8 * sizeof(unsigned long)));
    return 0;
}

int sigdelset(sigset_t *set, int signum)
{
    if (signum < 1 || signum >= _NSIG) { errno = EINVAL; return -1; }
    set->__val[(signum - 1) / (8 * sizeof(unsigned long))] &= ~(1UL << ((signum - 1) % (8 * sizeof(unsigned long))));
    return 0;
}

int sigismember(const sigset_t *set, int signum)
{
    if (signum < 1 || signum >= _NSIG) { errno = EINVAL; return -1; }
    return (set->__val[(signum - 1) / (8 * sizeof(unsigned long))] >> ((signum - 1) % (8 * sizeof(unsigned long)))) & 1;
}

int sigprocmask(int how, const sigset_t *set, sigset_t *oldset)
{
    (void)how; (void)set; (void)oldset;
    return 0;
}

/* ===================================================================
 *  SECTION 17: setjmp / longjmp -- ARM64 assembly
 * =================================================================== */

/*
 * setjmp: save callee-saved registers x19-x30, sp, d8-d15
 * AArch64 calling convention: x0 = jmp_buf
 * Returns 0 on initial call, val on longjmp
 */
__asm__(
".global setjmp\n"
".type setjmp, @function\n"
"setjmp:\n"
"    stp x19, x20, [x0, #0]\n"
"    stp x21, x22, [x0, #16]\n"
"    stp x23, x24, [x0, #32]\n"
"    stp x25, x26, [x0, #48]\n"
"    stp x27, x28, [x0, #64]\n"
"    stp x29, x30, [x0, #80]\n"
"    mov x2, sp\n"
"    str x2, [x0, #96]\n"
"    stp d8, d9, [x0, #104]\n"
"    stp d10, d11, [x0, #120]\n"
"    stp d12, d13, [x0, #136]\n"
"    stp d14, d15, [x0, #152]\n"
"    mov x0, #0\n"
"    ret\n"
);

__asm__(
".global longjmp\n"
".type longjmp, @function\n"
"longjmp:\n"
"    ldp x19, x20, [x0, #0]\n"
"    ldp x21, x22, [x0, #16]\n"
"    ldp x23, x24, [x0, #32]\n"
"    ldp x25, x26, [x0, #48]\n"
"    ldp x27, x28, [x0, #64]\n"
"    ldp x29, x30, [x0, #80]\n"
"    ldr x2, [x0, #96]\n"
"    mov sp, x2\n"
"    ldp d8, d9, [x0, #104]\n"
"    ldp d10, d11, [x0, #120]\n"
"    ldp d12, d13, [x0, #136]\n"
"    ldp d14, d15, [x0, #152]\n"
"    /* return val (x1), or 1 if val==0 */\n"
"    cmp x1, #0\n"
"    csinc x0, x1, xzr, ne\n"
"    ret\n"
);

/* ===================================================================
 *  SECTION 18: Sockets (sys/socket.h, arpa/inet.h)
 * =================================================================== */

int socket(int domain, int type, int protocol)
{
    return (int)__set_errno(__syscall3(SYS_socket, domain, type, protocol));
}

int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen)
{
    return (int)__set_errno(__syscall3(SYS_bind, sockfd, (long)addr, addrlen));
}

int listen(int sockfd, int backlog)
{
    return (int)__set_errno(__syscall2(SYS_listen, sockfd, backlog));
}

int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen)
{
    return (int)__set_errno(__syscall3(SYS_accept, sockfd, (long)addr, (long)addrlen));
}

int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen)
{
    return (int)__set_errno(__syscall3(SYS_connect, sockfd, (long)addr, addrlen));
}

long send(int sockfd, const void *buf, size_t len, int flags)
{
    return (long)__set_errno(__syscall6(SYS_sendto, sockfd, (long)buf, (long)len, flags, 0, 0));
}

long recv(int sockfd, void *buf, size_t len, int flags)
{
    return (long)__set_errno(__syscall6(SYS_recvfrom, sockfd, (long)buf, (long)len, flags, 0, 0));
}

long sendto(int sockfd, const void *buf, size_t len, int flags,
            const struct sockaddr *dest_addr, socklen_t addrlen)
{
    return (long)__set_errno(__syscall6(SYS_sendto, sockfd, (long)buf, (long)len,
                                       flags, (long)dest_addr, addrlen));
}

long recvfrom(int sockfd, void *buf, size_t len, int flags,
              struct sockaddr *src_addr, socklen_t *addrlen)
{
    return (long)__set_errno(__syscall6(SYS_recvfrom, sockfd, (long)buf, (long)len,
                                       flags, (long)src_addr, (long)addrlen));
}

int setsockopt(int sockfd, int level, int optname, const void *optval, socklen_t optlen)
{
    return (int)__set_errno(__syscall5(SYS_setsockopt, sockfd, level, optname, (long)optval, optlen));
}

int getsockopt(int sockfd, int level, int optname, void *optval, socklen_t *optlen)
{
    return (int)__set_errno(__syscall5(SYS_getsockopt, sockfd, level, optname, (long)optval, (long)optlen));
}

int shutdown(int sockfd, int how)
{
    return (int)__set_errno(__syscall2(SYS_shutdown, sockfd, how));
}

int getpeername(int sockfd, struct sockaddr *addr, socklen_t *addrlen)
{
    return (int)__set_errno(__syscall3(SYS_getpeername, sockfd, (long)addr, (long)addrlen));
}

int getsockname(int sockfd, struct sockaddr *addr, socklen_t *addrlen)
{
    return (int)__set_errno(__syscall3(SYS_getsockname, sockfd, (long)addr, (long)addrlen));
}

/* Byte-order helpers -- aarch64 is little-endian */
uint16_t htons(uint16_t x) { return __builtin_bswap16(x); }
uint16_t ntohs(uint16_t x) { return __builtin_bswap16(x); }
uint32_t htonl(uint32_t x) { return __builtin_bswap32(x); }
uint32_t ntohl(uint32_t x) { return __builtin_bswap32(x); }

int inet_pton(int af, const char *src, void *dst)
{
    if (af == AF_INET) {
        unsigned char *d = (unsigned char *)dst;
        unsigned int parts[4] = {0};
        int part = 0;
        const char *p = src;
        while (*p && part < 4) {
            if (*p == '.') { part++; p++; continue; }
            if (*p >= '0' && *p <= '9') { parts[part] = parts[part] * 10 + (*p - '0'); }
            p++;
        }
        if (part != 3) return 0;
        for (int i = 0; i < 4; i++) d[i] = (unsigned char)parts[i];
        return 1;
    }
    return 0; /* IPv6 not supported */
}

const char *inet_ntop(int af, const void *src, char *dst, unsigned int size)
{
    if (af == AF_INET && size >= INET_ADDRSTRLEN) {
        const unsigned char *s = (const unsigned char *)src;
        snprintf(dst, size, "%u.%u.%u.%u", s[0], s[1], s[2], s[3]);
        return dst;
    }
    return NULL;
}

/* ===================================================================
 *  SECTION 19: select (sys/select.h)
 * =================================================================== */

int select(int nfds, fd_set *rd, fd_set *wr, fd_set *ex, struct timeval *tv)
{
    struct timespec ts, *pts = NULL;
    if (tv) {
        ts.tv_sec = tv->tv_sec;
        ts.tv_nsec = tv->tv_usec * 1000;
        pts = &ts;
    }
    return pselect(nfds, rd, wr, ex, pts, NULL);
}

int pselect(int nfds, fd_set *rd, fd_set *wr, fd_set *ex,
            const struct timespec *ts, const void *sigmask)
{
    return (int)__set_errno(__syscall6(SYS_pselect6, nfds, (long)rd, (long)wr,
                                      (long)ex, (long)ts, (long)sigmask));
}

/* ===================================================================
 *  SECTION 20: uname (sys/utsname.h)
 * =================================================================== */

int uname(struct utsname *buf)
{
    if (!buf) { errno = EINVAL; return -1; }
    memset(buf, 0, sizeof(*buf));
    strcpy(buf->sysname, "BatOS");
    strcpy(buf->nodename, "bat");
    strcpy(buf->release, "1.0.0");
    strcpy(buf->version, "Bat_OS v1.0");
    strcpy(buf->machine, "aarch64");
    return 0;
}

/* ===================================================================
 *  SECTION 21: iconv (iconv.h) -- stubs
 * =================================================================== */

iconv_t iconv_open(const char *to, const char *from)
{
    (void)to; (void)from;
    return (iconv_t)1; /* non-null = "success" */
}

size_t iconv(iconv_t cd, char **inbuf, size_t *inbytesleft,
             char **outbuf, size_t *outbytesleft)
{
    (void)cd;
    /* Simple passthrough: copy bytes 1:1 */
    if (!inbuf || !*inbuf || !outbuf || !*outbuf) return 0;
    size_t copied = 0;
    while (*inbytesleft > 0 && *outbytesleft > 0) {
        **outbuf = **inbuf;
        (*inbuf)++; (*outbuf)++;
        (*inbytesleft)--; (*outbytesleft)--;
        copied++;
    }
    return copied;
}

int iconv_close(iconv_t cd)
{
    (void)cd;
    return 0;
}

/* ===================================================================
 *  SECTION 22: Regex (regex.h) -- stubs
 * =================================================================== */

int regcomp(regex_t *preg, const char *pattern, int cflags)
{
    (void)preg; (void)pattern; (void)cflags;
    return REG_BADPAT; /* always fail */
}

int regexec(const regex_t *preg, const char *string,
            size_t nmatch, regmatch_t pmatch[], int eflags)
{
    (void)preg; (void)string; (void)nmatch; (void)pmatch; (void)eflags;
    return REG_NOMATCH;
}

size_t regerror(int errcode, const regex_t *preg, char *errbuf, size_t errbuf_size)
{
    (void)errcode; (void)preg;
    const char *msg = "regex not supported";
    size_t len = strlen(msg) + 1;
    if (errbuf && errbuf_size > 0) {
        size_t n = len < errbuf_size ? len : errbuf_size;
        memcpy(errbuf, msg, n);
        errbuf[n - 1] = '\0';
    }
    return len;
}

void regfree(regex_t *preg)
{
    (void)preg;
}
