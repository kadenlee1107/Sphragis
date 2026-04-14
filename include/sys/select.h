#ifndef _SYS_SELECT_H
#define _SYS_SELECT_H
typedef struct { unsigned long fds_bits[16]; } fd_set;
#define FD_ZERO(s) do { int i; for(i=0;i<16;i++) (s)->fds_bits[i]=0; } while(0)
#define FD_SET(fd, s) ((s)->fds_bits[(fd)/64] |= (1UL << ((fd)%64)))
#define FD_ISSET(fd, s) ((s)->fds_bits[(fd)/64] & (1UL << ((fd)%64)))
int select(int nfds, fd_set *rd, fd_set *wr, fd_set *ex, struct timeval *tv);
int pselect(int nfds, fd_set *rd, fd_set *wr, fd_set *ex, const struct timespec *ts, const void *sigmask);
#endif
