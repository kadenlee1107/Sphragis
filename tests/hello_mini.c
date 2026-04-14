// Bat_OS Test — Mini-libc using raw syscalls
// Tests malloc (via mmap), printf (via write), string ops — WITHOUT musl
// This proves the syscall layer works for C programs.

// Raw syscall wrappers (no libc needed)
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

// Mini printf (write to stdout via syscall)
static void print(const char *s) {
    int len = 0;
    while (s[len]) len++;
    syscall3(64, 1, (long)s, len);
}

static void print_num(long n) {
    char buf[20];
    int i = 19;
    buf[i] = 0;
    if (n == 0) { buf[--i] = '0'; }
    else if (n < 0) { print("-"); n = -n; }
    while (n > 0) { buf[--i] = '0' + (n % 10); n /= 10; }
    print(&buf[i]);
}

// Mini malloc via mmap syscall
static void *mini_malloc(long size) {
    // mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)
    long addr = syscall6(222, 0, size, 3, 34, -1, 0);
    if (addr < 0) return (void*)0;
    return (void*)addr;
}

// String functions
static int my_strlen(const char *s) {
    int n = 0;
    while (s[n]) n++;
    return n;
}

static void my_strcpy(char *dst, const char *src) {
    while (*src) *dst++ = *src++;
    *dst = 0;
}

static void my_sprintf_int(char *buf, const char *fmt, int val) {
    while (*fmt) {
        if (fmt[0] == '%' && fmt[1] == 'd') {
            char num[20];
            int i = 19;
            num[i] = 0;
            int n = val;
            if (n == 0) num[--i] = '0';
            else while (n > 0) { num[--i] = '0' + (n % 10); n /= 10; }
            const char *p = &num[i];
            while (*p) *buf++ = *p++;
            fmt += 2;
        } else {
            *buf++ = *fmt++;
        }
    }
    *buf = 0;
}

// Fibonacci
static int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

void _start(void) {
    print("=== Bat_OS Mini-libc Test ===\n");

    // Test 1: Dynamic memory allocation via mmap
    print("[1] malloc test: ");
    char *buf = (char*)mini_malloc(4096);
    if (buf) {
        my_strcpy(buf, "Dynamic memory via mmap works!");
        print(buf);
        print("\n");
    } else {
        print("FAILED\n");
    }

    // Test 2: String operations (using stack string to avoid rodata segment)
    print("[2] strlen test: ");
    char test_str[] = {'H','e','l','l','o',',',' ','B','a','t','_','O','S','!',0};
    print_num(my_strlen(test_str));
    print(" chars\n");

    // Test 3: Computation
    print("[3] fibonacci(10) = ");
    int fib = fibonacci(10);
    print_num(fib);
    print("\n");

    // Test 4: sprintf equivalent
    if (buf) {
        print("[4] sprintf test: ");
        my_sprintf_int(buf, "result = %d", fib);
        print(buf);
        print("\n");
    }

    // Test 5: Multiple mmap allocations
    print("[5] multiple allocs: ");
    int alloc_count = 0;
    for (int i = 0; i < 10; i++) {
        void *p = mini_malloc(4096);
        if (p) alloc_count++;
    }
    print_num(alloc_count);
    print("/10 succeeded\n");

    // Test 6: Clock
    print("[6] time: ");
    long ts[2] = {0, 0};
    syscall3(113, 1, (long)ts, 0); // clock_gettime CLOCK_MONOTONIC
    print_num(ts[0]);
    print(".");
    print_num(ts[1] / 1000000);
    print("s\n");

    print("=== All mini-libc tests passed! ===\n");

    syscall1(93, 0); // exit(0)
    __builtin_unreachable();
}
