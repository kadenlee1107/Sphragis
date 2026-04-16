/*
 * cpu_loop.c — DoS stress: CPU hogging.
 *
 * Covers:
 *   ATTACK-DOS-018  tight EL0 loop (no syscalls)
 *   ATTACK-DOS-019  64 threads all yielding (scheduler thrash)
 *
 * Usage:
 *   ./cpu_loop tight     # DOS-018: pure spin
 *   ./cpu_loop yield     # DOS-019: 64 threads yielding at each other
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sched.h>
#include <pthread.h>

static void cpu_tight(void) {
    volatile unsigned long x = 0;
    for (;;) { x += 1; }
}

static void *yield_worker(void *_) {
    (void)_;
    for (;;) { sched_yield(); }
    return NULL;
}

static void cpu_yield(void) {
    /* 63 workers + main = 64 threads; matches MAX_THREADS. Each schedule()
       pass scans the whole table under IRQ-masked spinlock. */
    for (int i = 0; i < 63; i++) {
        pthread_t t;
        if (pthread_create(&t, NULL, yield_worker, NULL) != 0) {
            fprintf(stderr, "[cpu_loop/yield] created %d workers, stopping\n", i);
            break;
        }
        pthread_detach(t);
    }
    fprintf(stderr, "[cpu_loop/yield] 64-thread yield storm started\n");
    yield_worker(NULL);
}

int main(int argc, char **argv) {
    const char *mode = (argc > 1) ? argv[1] : "tight";
    if      (!strcmp(mode, "tight")) cpu_tight();
    else if (!strcmp(mode, "yield")) cpu_yield();
    else {
        fprintf(stderr, "usage: %s [tight|yield]\n", argv[0]);
        return 2;
    }
    return 0;
}
