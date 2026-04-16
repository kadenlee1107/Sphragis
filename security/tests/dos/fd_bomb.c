/*
 * fd_bomb.c — DoS stress: fd and socket exhaustion.
 *
 * Covers:
 *   ATTACK-DOS-008  open-forever (per-cave fd table, 64 slots)
 *   ATTACK-DOS-009  socket-forever (global socket table, 128 slots)
 *   ATTACK-DOS-012  eventfd/timerfd global exhaustion
 *   ATTACK-DOS-013  epoll interest flood
 *
 * Usage:
 *   ./fd_bomb open      # DOS-008
 *   ./fd_bomb socket    # DOS-009
 *   ./fd_bomb eventfd   # DOS-012
 *   ./fd_bomb epoll     # DOS-013
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/timerfd.h>

static void fd_bomb_open(void) {
    unsigned long count = 0;
    for (;;) {
        int fd = open("/", O_RDONLY | O_DIRECTORY);
        if (fd < 0) {
            fprintf(stderr, "[fd_bomb/open] EMFILE after %lu opens, errno=%d\n",
                    count, errno);
            for (;;) { /* hold the fds */ }
        }
        count++;
    }
}

static void fd_bomb_socket(void) {
    unsigned long count = 0;
    for (;;) {
        int s = socket(AF_INET, SOCK_STREAM, 0);
        if (s < 0) {
            fprintf(stderr, "[fd_bomb/socket] EMFILE after %lu sockets, errno=%d\n",
                    count, errno);
            /* Hold forever so other caves observe the denial. */
            for (;;) { /* park */ }
        }
        count++;
    }
}

static void fd_bomb_eventfd(void) {
    unsigned long count = 0;
    for (;;) {
        int e = eventfd(0, 0);
        if (e < 0) {
            fprintf(stderr, "[fd_bomb/eventfd] EMFILE after %lu eventfds, errno=%d\n",
                    count, errno);
            for (;;) { /* park */ }
        }
        count++;
    }
}

static void fd_bomb_epoll(void) {
    /* Create up to 64 epoll instances and fill each with 256 interests. */
    int epfds[64];
    int n_ep = 0;
    for (int i = 0; i < 64; i++) {
        int ep = epoll_create1(0);
        if (ep < 0) {
            fprintf(stderr, "[fd_bomb/epoll] create instance %d failed errno=%d\n",
                    i, errno);
            break;
        }
        epfds[n_ep++] = ep;
    }
    fprintf(stderr, "[fd_bomb/epoll] allocated %d instances\n", n_ep);

    /* Use a single eventfd as the watched target. */
    int target = eventfd(0, 0);
    for (int e = 0; e < n_ep; e++) {
        for (int i = 0; i < 256; i++) {
            struct epoll_event ev = { .events = EPOLLIN, .data.u64 = i };
            /* Linux forbids duplicate fd registration, so use dup to get
               distinct-but-cheap fds. In Bat_OS dup slot share behavior
               may differ — adjust if the cap trips early. */
            int t = dup(target);
            if (t < 0) break;
            if (epoll_ctl(e, EPOLL_CTL_ADD, t, &ev) < 0) {
                fprintf(stderr, "[fd_bomb/epoll] inst %d interest %d failed errno=%d\n",
                        e, i, errno);
                break;
            }
        }
    }
    fprintf(stderr, "[fd_bomb/epoll] interests filled. Hot mark_ready is now O(64*256).\n");
    for (;;) { /* park */ }
}

int main(int argc, char **argv) {
    const char *mode = (argc > 1) ? argv[1] : "open";
    if      (!strcmp(mode, "open"))    fd_bomb_open();
    else if (!strcmp(mode, "socket"))  fd_bomb_socket();
    else if (!strcmp(mode, "eventfd")) fd_bomb_eventfd();
    else if (!strcmp(mode, "epoll"))   fd_bomb_epoll();
    else {
        fprintf(stderr, "usage: %s [open|socket|eventfd|epoll]\n", argv[0]);
        return 2;
    }
    return 0;
}
