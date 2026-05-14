// Sphragis Thread Test — thread creation via clone syscall
#include "minilib.h"

MINILIB_MAIN

static long sys_clone(long flags, void *child_stack) {
    register long x0 __asm__("x0") = flags;
    register long x1 __asm__("x1") = (long)child_stack;
    register long x2 __asm__("x2") = 0;
    register long x3 __asm__("x3") = 0;
    register long x4 __asm__("x4") = 0;
    register long x8 __asm__("x8") = 220;
    __asm__ volatile("svc #0" : "=r"(x0)
        : "r"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x8) : "memory");
    return x0;
}

volatile int shared_counter = 0;

int main(int argc, char *argv[]) {
    printf("=== Thread Test ===\n");

    // Allocate child stack
    char *child_stack = (char *)malloc(65536);
    if (!child_stack) {
        printf("ERROR: stack alloc failed\n");
        return 1;
    }
    // Stack grows DOWN on ARM64, so pass the TOP
    void *stack_top = child_stack + 65536;
    printf("[parent] stack at 0x%x\n", (unsigned)(unsigned long)stack_top);

    // CLONE_VM(0x100) | CLONE_FS(0x200) | CLONE_FILES(0x400) | CLONE_SIGHAND(0x800)
    long tid = sys_clone(0x100 | 0x200 | 0x400 | 0x800, stack_top);

    if (tid == 0) {
        // CHILD — runs on child_stack
        printf("  [child] I am the child!\n");
        for (int i = 0; i < 10; i++) {
            shared_counter++;
        }
        printf("  [child] counter = %d\n", shared_counter);
        exit(0);
    } else if (tid > 0) {
        // PARENT
        printf("[parent] created child tid=%d\n", (int)tid);
        // Simple wait: spin until child increments counter
        for (int i = 0; i < 1000000; i++) {
            if (shared_counter >= 10) break;
        }
        printf("[parent] counter = %d\n", shared_counter);
    } else {
        printf("[parent] clone FAILED: %d\n", (int)tid);
    }

    printf("=== Thread Test Done ===\n");
    return 0;
}
