/*
 * canary.c — second-cave observer. Runs in a DIFFERENT Cave from the
 * attacker and reports whether normal operations still succeed.
 *
 * A healthy system prints "canary: ok N". A DoS'd system prints the
 * errno of the first operation that fails.
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <time.h>

int main(void) {
    unsigned long n = 0;
    for (;;) {
        /* 1. mmap a page */
        void *p = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                       MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        if (p == MAP_FAILED) {
            fprintf(stderr, "canary: mmap FAILED iter=%lu errno=%d\n", n, errno);
            return 1;
        }
        munmap(p, 4096);

        /* 2. open-and-close a file */
        int fd = open("/", O_RDONLY);
        if (fd < 0) {
            fprintf(stderr, "canary: open FAILED iter=%lu errno=%d\n", n, errno);
            return 2;
        }
        close(fd);

        /* 3. allocate a socket */
        int s = socket(AF_INET, SOCK_STREAM, 0);
        if (s < 0) {
            fprintf(stderr, "canary: socket FAILED iter=%lu errno=%d\n", n, errno);
            return 3;
        }
        close(s);

        n++;
        if ((n & 0xFF) == 0)
            fprintf(stderr, "canary: ok %lu\n", n);
        struct timespec ts = { 0, 10 * 1000 * 1000 }; /* 10 ms */
        nanosleep(&ts, NULL);
    }
}
