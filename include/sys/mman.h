/*
 * Sphragis — sys/mman.h stub for NetSurf
 * Memory mapping declarations.
 */
#ifndef _SPHRAGIS_SYS_MMAN_H
#define _SPHRAGIS_SYS_MMAN_H

#include <stddef.h>
#include <sys/types.h>

/* Protection flags */
#define PROT_NONE   0x0
#define PROT_READ   0x1
#define PROT_WRITE  0x2
#define PROT_EXEC   0x4

/* Map flags */
#define MAP_SHARED      0x01
#define MAP_PRIVATE     0x02
#define MAP_FIXED       0x10
#define MAP_ANONYMOUS   0x20
#define MAP_ANON        MAP_ANONYMOUS

/* Failure sentinel */
#define MAP_FAILED ((void *)-1)

/* Msync flags */
#define MS_ASYNC      1
#define MS_SYNC       2
#define MS_INVALIDATE 4

void *mmap(void *addr, size_t length, int prot, int flags,
           int fd, long offset);
int   munmap(void *addr, size_t length);
int   mprotect(void *addr, size_t len, int prot);
int   msync(void *addr, size_t length, int flags);

#endif /* _SPHRAGIS_SYS_MMAN_H */
