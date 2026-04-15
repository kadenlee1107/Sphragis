// Bat_OS — POSIX Infrastructure Test
// Tests: signals, pipes, epoll, shared memory, /proc filesystem
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Linux syscall wrappers
static long syscall1(long nr, long a) {
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8) : "memory");
    return x0;
}
static long syscall2(long nr, long a, long b) {
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1) : "memory");
    return x0;
}
static long syscall3(long nr, long a, long b, long c) {
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    register long x2 __asm__("x2") = c;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2) : "memory");
    return x0;
}
static long syscall4(long nr, long a, long b, long c, long d) {
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    register long x2 __asm__("x2") = c;
    register long x3 __asm__("x3") = d;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3) : "memory");
    return x0;
}

// Syscall numbers (ARM64 Linux)
#define SYS_read        63
#define SYS_write       64
#define SYS_close       57
#define SYS_openat      56
#define SYS_pipe2       59
#define SYS_rt_sigaction    134
#define SYS_rt_sigprocmask  135
#define SYS_tgkill      131
#define SYS_memfd_create 279
#define SYS_epoll_create1 20
#define SYS_epoll_ctl    21
#define SYS_epoll_pwait  22

// Signal numbers
#define SIGUSR1 10
#define SIGCHLD 17

// Open flags
#define AT_FDCWD -100
#define O_RDONLY 0

int passed = 0;
int failed = 0;

void check(const char *name, int cond) {
    if (cond) { printf("  [PASS] %s\n", name); passed++; }
    else      { printf("  [FAIL] %s\n", name); failed++; }
}

void _start(void) {
    printf("=== POSIX Infrastructure Test ===\n\n");

    // ── #5: Signals ──
    printf("[5] Signals:\n");
    {
        // rt_sigaction — set a handler for SIGUSR1
        unsigned long sa[4] = {1, 0, 0, 0}; // SIG_IGN
        unsigned long old_sa[4] = {0};
        long ret = syscall4(SYS_rt_sigaction, SIGUSR1, (long)sa, (long)old_sa, 8);
        check("rt_sigaction(SIGUSR1, SIG_IGN)", ret == 0);

        // rt_sigprocmask — block SIGCHLD
        unsigned long mask = (1UL << SIGCHLD);
        unsigned long old_mask = 0;
        ret = syscall4(SYS_rt_sigprocmask, 0/*SIG_BLOCK*/, (long)&mask, (long)&old_mask, 8);
        check("rt_sigprocmask(SIG_BLOCK, SIGCHLD)", ret == 0);

        // Unblock
        ret = syscall4(SYS_rt_sigprocmask, 1/*SIG_UNBLOCK*/, (long)&mask, 0, 8);
        check("rt_sigprocmask(SIG_UNBLOCK)", ret == 0);
    }

    // ── #6: Pipes ──
    printf("\n[6] Pipes:\n");
    {
        int fds[2] = {-1, -1};
        long ret = syscall2(SYS_pipe2, (long)fds, 0);
        check("pipe2() returns 0", ret == 0);
        check("pipe2() read fd >= 0", fds[0] >= 0);
        check("pipe2() write fd >= 0", fds[1] >= 0);

        if (fds[0] >= 0 && fds[1] >= 0) {
            // Write to pipe
            const char *msg = "hello pipe";
            long written = syscall3(SYS_write, fds[1], (long)msg, 10);
            check("write(pipe) = 10", written == 10);

            // Read from pipe
            char buf[32] = {0};
            long rd = syscall3(SYS_read, fds[0], (long)buf, 32);
            check("read(pipe) = 10", rd == 10);
            check("pipe data correct", memcmp(buf, "hello pipe", 10) == 0);

            syscall1(SYS_close, fds[0]);
            syscall1(SYS_close, fds[1]);
        }
    }

    // ── #7: Epoll ──
    printf("\n[7] Epoll:\n");
    {
        long epfd = syscall1(SYS_epoll_create1, 0);
        check("epoll_create1() >= 0", epfd >= 0);

        if (epfd >= 0) {
            // Add stdin to epoll (EPOLLIN)
            struct { unsigned int events; unsigned long long data; } ev;
            ev.events = 1; // EPOLLIN
            ev.data = 0;
            long ret = syscall4(SYS_epoll_ctl, epfd, 1/*EPOLL_CTL_ADD*/, 0/*stdin*/, (long)&ev);
            check("epoll_ctl(ADD, stdin)", ret == 0);

            syscall1(SYS_close, epfd);
        }
    }

    // ── #8: Shared Memory ──
    printf("\n[8] Shared Memory:\n");
    {
        long fd = syscall2(SYS_memfd_create, (long)"test", 0);
        check("memfd_create() >= 0", fd >= 0);
        if (fd >= 0) syscall1(SYS_close, fd);
    }

    // ── #9: /proc Filesystem ──
    printf("\n[9] /proc Filesystem:\n");
    {
        // Open /proc/version
        long fd = syscall4(SYS_openat, AT_FDCWD, (long)"/proc/version", O_RDONLY, 0);
        check("/proc/version opens", fd >= 0);
        if (fd >= 0) {
            char buf[128] = {0};
            long rd = syscall3(SYS_read, fd, (long)buf, 127);
            check("/proc/version readable", rd > 0);
            if (rd > 0) {
                buf[rd < 127 ? rd : 127] = 0;
                printf("    content: %s", buf);
            }
            syscall1(SYS_close, fd);
        }

        // Open /proc/cpuinfo
        fd = syscall4(SYS_openat, AT_FDCWD, (long)"/proc/cpuinfo", O_RDONLY, 0);
        check("/proc/cpuinfo opens", fd >= 0);
        if (fd >= 0) {
            char buf[256] = {0};
            long rd = syscall3(SYS_read, fd, (long)buf, 255);
            check("/proc/cpuinfo readable", rd > 0);
            syscall1(SYS_close, fd);
        }

        // Open /proc/meminfo
        fd = syscall4(SYS_openat, AT_FDCWD, (long)"/proc/meminfo", O_RDONLY, 0);
        check("/proc/meminfo opens", fd >= 0);
        if (fd >= 0) syscall1(SYS_close, fd);
    }

    printf("\n=== Results: %d passed, %d failed ===\n", passed, failed);
    if (failed == 0) printf("=== POSIX Infrastructure Test PASSED ===\n");
    else printf("=== POSIX Infrastructure Test FAILED ===\n");
    exit(0);
}
