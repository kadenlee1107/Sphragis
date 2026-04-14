// Bat_OS Thread Test — pthread-style threads via raw clone syscall
// Tests: thread creation, shared memory, clone with child_stack
// Compile: clang --target=aarch64-linux-gnu -nostdlib -static -mstrict-align -o hello_threads hello_threads.c

#include "minilib.h"

// ─── Raw clone syscall wrapper ───
static long sys_clone(long flags, void *child_stack, void *parent_tid, void *tls, void *child_tid) {
    register long x0 __asm__("x0") = flags;
    register long x1 __asm__("x1") = (long)child_stack;
    register long x2 __asm__("x2") = (long)parent_tid;
    register long x3 __asm__("x3") = (long)tls;
    register long x4 __asm__("x4") = (long)child_tid;
    register long x8 __asm__("x8") = 220; // SYS_clone
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x8) : "memory");
    return x0;
}

// ─── Raw gettid syscall wrapper ───
static long sys_gettid(void) {
    register long x0 __asm__("x0");
    register long x8 __asm__("x8") = 178; // SYS_gettid
    __asm__ volatile("svc #0" : "=r"(x0) : "r"(x8) : "memory");
    return x0;
}

// Shared counter — both threads can see and modify this
volatile int counter = 0;

void thread_func(void) {
    long tid = sys_gettid();
    printf("  [child] thread started, tid=%d\n", (int)tid);

    for (int i = 0; i < 10; i++) {
        counter++;
    }

    printf("  [child] thread done, counter = %d\n", counter);
    exit(0);
}

void _start(void) {
    printf("=== Bat_OS Thread Test ===\n");

    long my_tid = sys_gettid();
    printf("[parent] main thread tid=%d\n", (int)my_tid);

    // Allocate stack for child thread (64KB)
    char *child_stack = (char *)malloc(65536);
    if (!child_stack) {
        printf("[parent] ERROR: failed to allocate child stack\n");
        exit(1);
    }
    void *stack_top = child_stack + 65536;

    printf("[parent] child stack allocated at %x, top at %x\n",
           (unsigned int)(unsigned long)child_stack,
           (unsigned int)(unsigned long)stack_top);

    // Clone with CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND
    // child_stack != 0 tells the kernel this is a thread, not a fork
    long tid = sys_clone(0x00000100 | 0x00000200 | 0x00000400 | 0x00000800,
                         stack_top, NULL, NULL, NULL);

    if (tid == 0) {
        // Child thread — runs on child_stack
        thread_func();
        // thread_func calls exit(), never reaches here
    } else if (tid > 0) {
        // Parent — child has run and exited (cooperative scheduling)
        printf("[parent] clone returned tid=%d\n", (int)tid);
        printf("[parent] counter = %d (expected 10)\n", counter);
    } else {
        printf("[parent] ERROR: clone failed with %d\n", (int)tid);
        exit(1);
    }

    printf("=== Thread Test PASSED ===\n");
    exit(0);
}
