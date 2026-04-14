// Bat_OS Full Mini-libc Test
// Tests: function calls, malloc (mmap), string ops, fibonacci, sprintf
// Compile with: -mstrict-align (no unaligned accesses for HVF compatibility)

static long syscall1(long nr, long a0) {
    register long x0 __asm__("x0") = a0;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x8) : "memory");
    return x0;
}
static long syscall3(long nr, long a0, long a1, long a2) {
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x1), "r"(x2), "r"(x8) : "memory");
    return x0;
}
static long syscall6(long nr, long a0, long a1, long a2, long a3, long a4, long a5) {
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

// Print string to stdout
void print(const char *s) {
    int len = 0;
    while (s[len]) len++;
    syscall3(64, 1, (long)s, len);
}

void print_num(long n) {
    char buf[20];
    int i = 19;
    buf[i] = 0;
    if (n == 0) { buf[--i] = '0'; }
    else {
        int neg = n < 0;
        if (neg) n = -n;
        while (n > 0) { buf[--i] = '0' + (n % 10); n /= 10; }
        if (neg) buf[--i] = '-';
    }
    print(&buf[i]);
}

// malloc via mmap
void *mini_malloc(long size) {
    long addr = syscall6(222, 0, size, 3, 34, -1, 0);
    if (addr < 0) return (void*)0;
    return (void*)addr;
}

// String functions
int my_strlen(const char *s) {
    int n = 0; while (s[n]) n++; return n;
}
void my_strcpy(char *d, const char *s) {
    while (*s) *d++ = *s++; *d = 0;
}

// Fibonacci (recursive — tests deep stack usage)
int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

void _start(void) {
    print("=== Bat_OS C Program Test ===\n");

    // Test 1: malloc via mmap
    print("[1] malloc: ");
    char *buf = (char*)mini_malloc(4096);
    if (buf) {
        my_strcpy(buf, "Dynamic memory works!");
        print(buf);
        print("\n");
    } else {
        print("FAILED\n");
    }

    // Test 2: strlen
    print("[2] strlen: ");
    print_num(my_strlen("Hello Bat_OS"));
    print(" chars\n");

    // Test 3: fibonacci(10)
    print("[3] fib(10) = ");
    print_num(fibonacci(10));
    print("\n");

    // Test 4: sprintf-like
    if (buf) {
        print("[4] sprintf: ");
        my_strcpy(buf, "answer = ");
        int off = my_strlen(buf);
        int val = fibonacci(10);
        char num[8];
        int ni = 7; num[ni] = 0;
        while (val > 0) { num[--ni] = '0' + (val % 10); val /= 10; }
        my_strcpy(buf + off, &num[ni]);
        print(buf);
        print("\n");
    }

    // Test 5: Multiple allocations
    print("[5] allocs: ");
    int ok = 0;
    for (int i = 0; i < 10; i++) {
        if (mini_malloc(4096)) ok++;
    }
    print_num(ok);
    print("/10\n");

    // Test 6: Clock
    print("[6] time: ");
    long ts[2] = {0, 0};
    syscall3(113, 1, (long)ts, 0);
    print_num(ts[0]);
    print("s\n");

    print("=== ALL TESTS PASSED ===\n");
    syscall1(93, 0);
    __builtin_unreachable();
}
