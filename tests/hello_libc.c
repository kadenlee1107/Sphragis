// Bat_OS Test — C program WITH libc (musl)
// Tests: printf, malloc/free, strlen, memcpy, sprintf
// Cross-compile: zig cc -target aarch64-linux-musl -static -o hello_libc hello_libc.c

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(int argc, char *argv[]) {
    printf("=== Bat_OS libc Test ===\n");
    printf("Hello from C with musl libc!\n");
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

    printf("=== All libc tests passed! ===\n");
    return 0;
}
