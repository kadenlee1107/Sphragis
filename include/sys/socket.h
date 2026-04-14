/*
 * Bat_OS — sys/socket.h stub for NetSurf
 * BSD-style socket declarations.
 */
#ifndef _BATOS_SYS_SOCKET_H
#define _BATOS_SYS_SOCKET_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

/* Address families */
#define AF_UNSPEC   0
#define AF_UNIX     1
#define AF_LOCAL    AF_UNIX
#define AF_INET     2
#define AF_INET6   10
#define PF_UNSPEC   AF_UNSPEC
#define PF_INET     AF_INET
#define PF_INET6    AF_INET6

/* Socket types */
#define SOCK_STREAM    1
#define SOCK_DGRAM     2
#define SOCK_RAW       3

/* Socket options / levels */
#define SOL_SOCKET     1
#define SO_REUSEADDR   2
#define SO_KEEPALIVE   9
#define SO_RCVTIMEO   20
#define SO_SNDTIMEO   21
#define SO_ERROR      4

/* Shutdown how */
#define SHUT_RD    0
#define SHUT_WR    1
#define SHUT_RDWR  2

/* Message flags */
#define MSG_PEEK       0x02
#define MSG_DONTWAIT   0x40
#define MSG_NOSIGNAL   0x4000

typedef unsigned int   socklen_t;
typedef unsigned short sa_family_t;

struct sockaddr {
    sa_family_t sa_family;
    char        sa_data[14];
};

struct sockaddr_storage {
    sa_family_t ss_family;
    char        _ss_pad[126];
};

struct in_addr {
    uint32_t s_addr;
};

struct sockaddr_in {
    sa_family_t    sin_family;
    uint16_t       sin_port;
    struct in_addr sin_addr;
    char           sin_zero[8];
};

struct in6_addr {
    uint8_t s6_addr[16];
};

struct sockaddr_in6 {
    sa_family_t     sin6_family;
    uint16_t        sin6_port;
    uint32_t        sin6_flowinfo;
    struct in6_addr sin6_addr;
    uint32_t        sin6_scope_id;
};

/* Ancillary / cmsghdr (minimal) */
struct msghdr {
    void         *msg_name;
    socklen_t     msg_namelen;
    struct iovec *msg_iov;
    int           msg_iovlen;
    void         *msg_control;
    socklen_t     msg_controllen;
    int           msg_flags;
};

struct iovec {
    void   *iov_base;
    size_t  iov_len;
};

/* Network byte-order helpers (no-op on big-endian; NetSurf expects these) */
#ifndef htons
uint16_t htons(uint16_t hostshort);
uint16_t ntohs(uint16_t netshort);
uint32_t htonl(uint32_t hostlong);
uint32_t ntohl(uint32_t netlong);
#endif

int    socket(int domain, int type, int protocol);
int    bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
int    listen(int sockfd, int backlog);
int    accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);
int    connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
long   send(int sockfd, const void *buf, size_t len, int flags);
long   recv(int sockfd, void *buf, size_t len, int flags);
long   sendto(int sockfd, const void *buf, size_t len, int flags,
              const struct sockaddr *dest_addr, socklen_t addrlen);
long   recvfrom(int sockfd, void *buf, size_t len, int flags,
                struct sockaddr *src_addr, socklen_t *addrlen);
int    setsockopt(int sockfd, int level, int optname,
                  const void *optval, socklen_t optlen);
int    getsockopt(int sockfd, int level, int optname,
                  void *optval, socklen_t *optlen);
int    shutdown(int sockfd, int how);
int    getpeername(int sockfd, struct sockaddr *addr, socklen_t *addrlen);
int    getsockname(int sockfd, struct sockaddr *addr, socklen_t *addrlen);

#endif /* _BATOS_SYS_SOCKET_H */
