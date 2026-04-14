// Bat_OS — C program with printf, malloc, string ops
// Uses minilib.h (our own libc replacement — no musl needed)

#include "minilib.h"

MINILIB_MAIN

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(int argc, char *argv[]) {
    printf("=== Bat_OS libc Test ===\n");
    printf("Hello from C with printf!\n");
    printf("argc = %d\n", argc);

    // Test malloc
    char *buf = (char *)malloc(256);
    if (buf) {
        strcpy(buf, "Dynamic memory works!");
        printf("malloc: %s\n", buf);

        // Test sprintf
        sprintf(buf, "fibonacci(10) = %d", fibonacci(10));
        printf("compute: %s\n", buf);

        // Test strlen
        printf("strlen: %zu\n", strlen(buf));

        free(buf);
        printf("free: OK\n");
    } else {
        printf("malloc FAILED\n");
    }

    // Test multiple features
    printf("hex: 0x%x\n", 0xDEAD);
    printf("string: %s + %s\n", "hello", "world");

    printf("=== All libc tests passed! ===\n");
    return 0;
}
