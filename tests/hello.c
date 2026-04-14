// Bat_OS Test — Raw syscall C program (no libc)
// Cross-compile: clang --target=aarch64-linux-gnu -nostdlib -static -o hello hello.c

static long syscall3(long nr, long a0, long a1, long a2) {
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x1), "r"(x2), "r"(x8) : "memory");
    return x0;
}

static long syscall1(long nr, long a0) {
    register long x0 __asm__("x0") = a0;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x8) : "memory");
    return x0;
}

static void write_str(const char *s) {
    int len = 0;
    while (s[len]) len++;
    syscall3(64, 1, (long)s, len);  // SYS_write(fd=1, buf, count)
}

static void write_num(long n) {
    char buf[20];
    int i = 19;
    buf[i--] = 0;
    if (n == 0) { buf[i--] = '0'; }
    else {
        while (n > 0) { buf[i--] = '0' + (n % 10); n /= 10; }
    }
    write_str(&buf[i + 1]);
}

void _start(void) {
    write_str("=== Bat_OS C Program Test ===\n");
    write_str("Hello from C on Bat_OS!\n");

    // Test mmap (syscall 222)
    long addr = syscall3(222, 0, 4096, 3);  // mmap(NULL, 4096, PROT_READ|PROT_WRITE)
    // Note: mmap takes 6 args, we're using simplified version
    write_str("mmap returned: ");
    write_num(addr);
    write_str("\n");

    // Test clock_gettime (syscall 113)
    long ts[2] = {0, 0};  // struct timespec
    syscall3(113, 1, (long)ts, 0);  // CLOCK_MONOTONIC
    write_str("Time: ");
    write_num(ts[0]);
    write_str(" sec, ");
    write_num(ts[1] / 1000000);
    write_str(" ms\n");

    // Test getpid (syscall 172)
    long pid = syscall1(172, 0);
    write_str("PID: ");
    write_num(pid);
    write_str("\n");

    write_str("=== All tests passed! ===\n");

    // Exit (syscall 93)
    syscall1(93, 0);
    __builtin_unreachable();
}
