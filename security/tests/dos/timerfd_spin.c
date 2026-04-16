/*
 * timerfd_spin.c — DoS stress: timerfd with sub-microsecond interval.
 *
 * Covers:
 *   ATTACK-DOS-021  allocate all 64 timerfds, arm each with 1-ns interval,
 *                   then loop polling them — each sweep() path does a
 *                   128-bit divide and counter fetch_add.
 */

#include <stdio.h>
#include <string.h>
#include <sys/timerfd.h>
#include <time.h>
#include <unistd.h>
#include <errno.h>

int main(void) {
    int fds[64];
    int n = 0;
    for (int i = 0; i < 64; i++) {
        int t = timerfd_create(CLOCK_MONOTONIC, 0);
        if (t < 0) {
            fprintf(stderr, "[timerfd_spin] got %d timerfds before EMFILE (errno=%d)\n",
                    i, errno);
            break;
        }
        fds[n++] = t;
        struct itimerspec its = {
            .it_value    = { .tv_sec = 0, .tv_nsec = 1 },
            .it_interval = { .tv_sec = 0, .tv_nsec = 1 },
        };
        if (timerfd_settime(t, 0, &its, NULL) < 0) {
            fprintf(stderr, "[timerfd_spin] settime failed on tfd %d errno=%d\n", t, errno);
        }
    }
    fprintf(stderr, "[timerfd_spin] %d timerfds armed at 1 ns. Polling to force sweep().\n", n);

    uint64_t buf;
    unsigned long iters = 0;
    for (;;) {
        for (int i = 0; i < n; i++) {
            /* Each read triggers sweep() which does a 128-bit divmod.
               With interval=1ns and now=any realistic value, "extra" is
               enormous and the counter keeps growing. */
            ssize_t r = read(fds[i], &buf, sizeof(buf));
            (void)r;
        }
        iters++;
        if ((iters & 0xFFF) == 0)
            fprintf(stderr, "[timerfd_spin] %lu sweep rounds\n", iters);
    }
}
