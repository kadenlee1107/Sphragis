// Simple mmap test
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>

void _start(void) {
    printf("=== mmap test ===\n");

    // Try malloc (which calls mmap internally)
    void *p = malloc(64);
    if (p) {
        printf("[PASS] malloc(64) = %p\n", p);
        free(p);
    } else {
        printf("[FAIL] malloc(64) returned NULL\n");
    }

    // Try direct mmap
    void *m = mmap(0, 4096, 3, 0x22, -1, 0); // PROT_READ|WRITE, MAP_PRIVATE|ANONYMOUS
    if (m != (void*)-1 && m != 0) {
        printf("[PASS] mmap(4096) = %p\n", m);
    } else {
        printf("[FAIL] mmap returned %p\n", m);
    }

    printf("=== done ===\n");
    exit(0);
}
