// Bat_OS — Minimal kernel-mode printf for Blink
// Writes to UART via the Rust uart_putc function.
#include <stdarg.h>

// Declared in Rust — writes one byte to UART
extern void uart_putc_bridge(unsigned char c);

static void put_char(char c) {
    // Write directly to UART MMIO (PL011 at 0x09000000)
    volatile unsigned int* uart = (volatile unsigned int*)0x09000000ULL;
    *uart = (unsigned int)c;
}

static void put_str(const char* s) {
    while (*s) put_char(*s++);
}

static void put_num(long long n) {
    if (n < 0) { put_char('-'); n = -n; }
    if (n == 0) { put_char('0'); return; }
    char buf[20]; int i = 0;
    while (n > 0) { buf[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) put_char(buf[--i]);
}

static void put_hex(unsigned long long n) {
    const char* h = "0123456789abcdef";
    if (n == 0) { put_char('0'); return; }
    char buf[16]; int i = 0;
    while (n > 0) { buf[i++] = h[n & 0xf]; n >>= 4; }
    while (i > 0) put_char(buf[--i]);
}

int printf(const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    while (*fmt) {
        if (*fmt == '%') {
            fmt++;
            // Handle length modifiers
            int is_long = 0;
            if (*fmt == 'l') { is_long = 1; fmt++; }
            if (*fmt == 'l') { is_long = 2; fmt++; }

            switch (*fmt) {
                case 's': put_str(va_arg(ap, const char*)); break;
                case 'd': case 'i':
                    if (is_long >= 2) put_num(va_arg(ap, long long));
                    else if (is_long) put_num(va_arg(ap, long));
                    else put_num(va_arg(ap, int));
                    break;
                case 'u':
                    if (is_long >= 2) put_num(va_arg(ap, unsigned long long));
                    else if (is_long) put_num(va_arg(ap, unsigned long));
                    else put_num(va_arg(ap, unsigned));
                    break;
                case 'x': case 'X':
                    if (is_long >= 2) put_hex(va_arg(ap, unsigned long long));
                    else if (is_long) put_hex(va_arg(ap, unsigned long));
                    else put_hex(va_arg(ap, unsigned));
                    break;
                case 'p': put_str("0x"); put_hex((unsigned long long)va_arg(ap, void*)); break;
                case 'c': put_char((char)va_arg(ap, int)); break;
                case '%': put_char('%'); break;
                case 'f': va_arg(ap, double); put_str("?f"); break;
                default: put_char('?'); break;
            }
        } else {
            put_char(*fmt);
        }
        fmt++;
    }
    va_end(ap);
    return 0;
}

int snprintf(char* buf, unsigned long n, const char* fmt, ...) {
    // Stub — just copy format string
    unsigned long i = 0;
    while (i < n - 1 && fmt[i]) { buf[i] = fmt[i]; i++; }
    buf[i] = 0;
    return (int)i;
}

int sprintf(char* buf, const char* fmt, ...) {
    return snprintf(buf, 4096, fmt);
}

int vprintf(const char* fmt, va_list ap) {
    (void)ap;
    put_str(fmt);
    return 0;
}

void abort(void) {
    put_str("[blink] ABORT\n");
    while(1) { __asm__ volatile("wfe"); }
}

int puts(const char* s) {
    put_str(s);
    put_char('\n');
    return 0;
}
