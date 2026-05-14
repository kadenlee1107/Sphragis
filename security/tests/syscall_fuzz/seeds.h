/*
 * syscall fuzzer seed inputs.
 *
 * Each SEED(nr, a0, a1, a2, a3, a4, a5) entry is one attack tuple.
 * Syscall numbers match src/caves/linux/syscall.rs (AArch64 Linux ABI,
 * plus Sphragis custom #500).
 *
 * Convention:
 *   KADDR = 0xffff000040000000  — a kernel-like high-half address.
 *           Not dereferenced by the fuzzer itself; passed to the kernel
 *           which (on a vulnerable build) will gladly dereference.
 *   UNALIGN(p) = ((p)|1)        — one-byte-off pointer.
 *   BADFD = 0x7fffffff, NEGFD = (unsigned long)-1
 *
 * Every syscall gets >= 10 seeds.
 */

#ifndef SEEDS_H
#define SEEDS_H

#include <stdint.h>

#define KADDR  0xffff000040000000UL
#define KADDR2 0xffffffffff000000UL
#define HUGE   0x7fffffffffffffffUL
#define NEG1   0xffffffffffffffffUL

struct seed { long nr; unsigned long a[6]; };

/* A scratch buffer in user-space the fuzzer populates before use. */
extern unsigned char scratch[4096];
#define UBUF  ((unsigned long)scratch)
#define UBUF8 ((unsigned long)(scratch + 8))
#define UBAD  (UBUF | 1)      /* misaligned user pointer */

/* ============= READ (63) / WRITE (64) ================================ */
#define SEEDS_READ_WRITE \
    { 63, { 0,      UBUF,   16,     0, 0, 0 } }, \
    { 63, { 0,      0,      16,     0, 0, 0 } }, \
    { 63, { 0,      KADDR,  4096,   0, 0, 0 } },   /* ATTACK-SYS-001 */ \
    { 63, { 0,      UBUF,   NEG1,   0, 0, 0 } },   /* ATTACK-SYS-004 */ \
    { 63, { 2,      UBUF,   16,     0, 0, 0 } }, \
    { 63, { 40,     UBUF,   16,     0, 0, 0 } },   /* /proc pseudo fd */ \
    { 63, { 0x7fffffff, UBUF, 16,   0, 0, 0 } },   /* fd oob */ \
    { 63, { 0,      UBAD,   16,     0, 0, 0 } },   /* unaligned */ \
    { 63, { 0,      UBUF,   0,      0, 0, 0 } }, \
    { 63, { 0,      UBUF,   HUGE,   0, 0, 0 } }, \
    { 64, { 1,      UBUF,   16,     0, 0, 0 } }, \
    { 64, { 1,      0,      16,     0, 0, 0 } }, \
    { 64, { 2,      KADDR,  512,    0, 0, 0 } },   /* ATTACK-SYS-002 */ \
    { 64, { 1,      UBUF,   NEG1,   0, 0, 0 } }, \
    { 64, { 1,      KADDR2, 64,     0, 0, 0 } }, \
    { 64, { 2,      UBAD,   64,     0, 0, 0 } }, \
    { 64, { 0x7fffffff, UBUF, 16,   0, 0, 0 } }, \
    { 64, { 9,      UBUF,   16,     0, 0, 0 } },   /* closed fd */ \
    { 64, { 1,      UBUF,   0,      0, 0, 0 } }, \
    { 64, { 1,      UBUF,   HUGE,   0, 0, 0 } },

/* ============= OPENAT (56) / CLOSE (57) / FACCESSAT (48) ============= */
#define SEEDS_FILE \
    { 56, { (unsigned long)-100, UBUF,  0, 0, 0, 0 } }, /* normal, UBUF=path */ \
    { 56, { (unsigned long)-100, 0,     0, 0, 0, 0 } }, \
    { 56, { (unsigned long)-100, KADDR, 0, 0, 0, 0 } }, /* ATTACK-SYS-052 */ \
    { 56, { (unsigned long)-100, UBAD,  0, 0, 0, 0 } }, \
    { 56, { (unsigned long)-100, UBUF,  0x40, 0,   0, 0 } }, /* O_CREAT */ \
    { 56, { (unsigned long)-100, UBUF,  0x10000, 0, 0, 0 } }, /* O_DIRECTORY */ \
    { 56, { 0x7fffffff, UBUF, 0, 0, 0, 0 } }, \
    { 56, { (unsigned long)-100, UBUF, NEG1, 0, 0, 0 } }, \
    { 56, { (unsigned long)-100, UBUF, 0, NEG1, 0, 0 } }, \
    { 56, { (unsigned long)-100, HUGE, 0, 0, 0, 0 } }, \
    { 57, { 0, 0, 0, 0, 0, 0 } }, \
    { 57, { 1, 0, 0, 0, 0, 0 } }, \
    { 57, { 2, 0, 0, 0, 0, 0 } }, \
    { 57, { 63, 0, 0, 0, 0, 0 } }, \
    { 57, { 64, 0, 0, 0, 0, 0 } }, \
    { 57, { NEG1, 0, 0, 0, 0, 0 } }, \
    { 48, { (unsigned long)-100, UBUF, 0, 0, 0, 0 } }, \
    { 48, { (unsigned long)-100, KADDR, 0, 0, 0, 0 } }, /* ATTACK-SYS-052 */ \
    { 48, { (unsigned long)-100, UBAD, 0, 0, 0, 0 } }, \
    { 48, { (unsigned long)-100, 0, 0, 0, 0, 0 } },

/* ============= FSTAT (80) / NEWFSTATAT (79) / GETCWD (17) ============= */
#define SEEDS_STAT \
    { 80, { 1, UBUF, 0, 0, 0, 0 } }, \
    { 80, { 0, 0, 0, 0, 0, 0 } },            /* EINVAL expected */ \
    { 80, { 0, KADDR, 0, 0, 0, 0 } },        /* ATTACK-SYS-003 */ \
    { 80, { 0, UBAD, 0, 0, 0, 0 } }, \
    { 80, { 0x7fffffff, UBUF, 0, 0, 0, 0 } }, \
    { 80, { NEG1, UBUF, 0, 0, 0, 0 } }, \
    { 80, { 40, UBUF, 0, 0, 0, 0 } }, \
    { 80, { 64, UBUF, 0, 0, 0, 0 } }, \
    { 80, { 0, UBUF, 0, 0, 0, 0 } }, \
    { 80, { 1, 1, 0, 0, 0, 0 } },            /* addr=1, not pages */ \
    { 79, { (unsigned long)-100, 0, UBUF, 0x1000, 0, 0 } }, /* AT_EMPTY_PATH */ \
    { 79, { (unsigned long)-100, UBUF, KADDR, 0, 0, 0 } }, \
    { 79, { (unsigned long)-100, KADDR, UBUF, 0, 0, 0 } }, \
    { 79, { (unsigned long)-100, UBUF, UBAD, 0, 0, 0 } }, \
    { 17, { UBUF, 128, 0, 0, 0, 0 } }, \
    { 17, { 0, 128, 0, 0, 0, 0 } }, \
    { 17, { KADDR, 128, 0, 0, 0, 0 } }, \
    { 17, { UBUF, 1, 0, 0, 0, 0 } },         /* size<2 EINVAL */ \
    { 17, { UBUF, NEG1, 0, 0, 0, 0 } }, \
    { 17, { UBAD, 128, 0, 0, 0, 0 } },

/* ============= WRITEV (66) ============================================ */
/* iovec laid out in scratch: iov[0]={scratch+64, 8}, iov[1]={...}       */
#define SEEDS_WRITEV \
    { 66, { 1, UBUF,  2, 0, 0, 0 } },       /* iov from user */ \
    { 66, { 1, KADDR, 2, 0, 0, 0 } },       /* ATTACK-SYS-041 */ \
    { 66, { 1, 0,     2, 0, 0, 0 } }, \
    { 66, { 1, UBUF,  NEG1, 0, 0, 0 } },    /* iovcnt huge */ \
    { 66, { 1, UBUF,  0x40000000, 0, 0, 0 } }, /* ATTACK-SYS-042 */ \
    { 66, { 2, UBUF,  2, 0, 0, 0 } }, \
    { 66, { 1, UBAD,  2, 0, 0, 0 } }, \
    { 66, { 0x7fffffff, UBUF, 2, 0, 0, 0 } }, \
    { 66, { 1, UBUF,  0, 0, 0, 0 } }, \
    { 66, { NEG1, UBUF, 2, 0, 0, 0 } },

/* ============= MMAP (222) / MPROTECT (226) / MUNMAP (215) / BRK (214) */
#define SEEDS_MEM \
    { 222, { 0, 4096, 3, 0x22, (unsigned long)-1, 0 } }, /* normal anon */ \
    { 222, { 0, 0, 3, 0x22, (unsigned long)-1, 0 } },    /* len=0 EINVAL */ \
    { 222, { 0, NEG1, 3, 0x22, (unsigned long)-1, 0 } }, /* len overflow; ATTACK-SYS-005 */ \
    { 222, { KADDR, 4096, 3, 0x12, (unsigned long)-1, 0 } }, /* MAP_FIXED kaddr; ATTACK-SYS-044 */ \
    { 222, { 0, 4096, 0, 0x22, (unsigned long)-1, 0 } }, \
    { 222, { 0, 4096, 7, 0x10, (unsigned long)-1, 0 } }, /* MAP_FIXED addr=0 */ \
    { 222, { 0, HUGE, 3, 0x22, (unsigned long)-1, 0 } }, \
    { 222, { 0, 4096, 3, 0x22, 0, 0 } },                /* fd=0 with flags */ \
    { 222, { 0, 4096, 3, 0x22, 1, 0 } }, \
    { 222, { 0, 4096, 3, 0x22, 40, 0 } }, \
    { 226, { 0, 4096, 3, 0, 0, 0 } }, \
    { 226, { KADDR, 4096, 7, 0, 0, 0 } },                /* ATTACK-SYS-045 */ \
    { 226, { 0, NEG1, 3, 0, 0, 0 } }, \
    { 226, { UBAD, 4096, 3, 0, 0, 0 } }, \
    { 215, { 0, 4096, 0, 0, 0, 0 } }, \
    { 215, { KADDR, 4096, 0, 0, 0, 0 } }, \
    { 215, { UBAD, 4096, 0, 0, 0, 0 } }, \
    { 214, { 0, 0, 0, 0, 0, 0 } }, \
    { 214, { KADDR, 0, 0, 0, 0, 0 } }, \
    { 214, { HUGE, 0, 0, 0, 0, 0 } },

/* ============= FUTEX (98) ============================================ */
#define SEEDS_FUTEX \
    { 98, { UBUF,   0, 0, 0, 0, 0 } },            /* FUTEX_WAIT val=0 */ \
    { 98, { KADDR,  0, 0, 0, 0, 0 } },            /* ATTACK-SYS-006 */ \
    { 98, { UBUF|1, 0, 0, 0, 0, 0 } },            /* unaligned; ATTACK-SYS-009 */ \
    { 98, { 0,      0, 0, 0, 0, 0 } },            /* null uaddr */ \
    { 98, { UBUF,   1, 0, 0, 0, 0 } },            /* FUTEX_WAKE */ \
    { 98, { UBUF,   9999, 0, 0, 0, 0 } },         /* unknown op; ATTACK-SYS-008 */ \
    { 98, { UBUF,   9, 0, 0, 0, 0 } },            /* WAIT_BITSET */ \
    { 98, { UBUF,   10, 0, 0, 0, 0 } },           /* WAKE_BITSET bitset=0; ATTACK-SYS-007 */ \
    { 98, { UBUF,   3, 0, KADDR, 0, 0 } },        /* REQUEUE uaddr2=kaddr */ \
    { 98, { UBUF,   0, 0, KADDR, 0, 0 } },        /* timeout_ptr=kaddr */ \
    { 98, { UBUF,   0, 0, UBAD, 0, 0 } },

/* ============= CLONE (220) =========================================== */
/* flags: CLONE_VM|CLONE_THREAD = 0x00000100|0x00010000 = 0x00010100    */
#define SEEDS_CLONE \
    { 220, { 0x10100, UBUF, 0, 0, 0, 0 } },                        /* legit */ \
    { 220, { 0x10100, KADDR, 0, 0, 0, 0 } },                       /* ATTACK-SYS-010 */ \
    { 220, { 0x10100, UBUF, KADDR, 0, 0, 0 } },                    /* parent_tid=kaddr; SYS-012 */ \
    { 220, { 0x10100, UBUF, 0, 0, KADDR, 0 } },                    /* child_tid=kaddr; SYS-011 */ \
    { 220, { 0x10100 | 0x4000, UBUF, 0, 0, 0, 0 } },               /* VFORK EINVAL */ \
    { 220, { 0x10100 | 0x2000, UBUF, 0, 0, 0, 0 } },               /* PTRACE EINVAL */ \
    { 220, { 0, UBUF, 0, 0, 0, 0 } },                              /* no VM|THREAD */ \
    { 220, { 0x10100, 1, 0, 0, 0, 0 } },                           /* unaligned stack */ \
    { 220, { 0x10100, NEG1, 0, 0, 0, 0 } }, \
    { 220, { 0x10100 | 0x40000000, UBUF, 0, 0, 0, 0 } },           /* CLONE_NEWNS */ \
    { 220, { 0x10100 | 0x100000, UBUF, 0, 0, 0, 0 } },             /* CLONE_NEWUSER; ATTACK-SYS-013 */

/* ============= EXECVE (221) ========================================== */
#define SEEDS_EXECVE \
    { 221, { UBUF, UBUF8, 0, 0, 0, 0 } }, \
    { 221, { 0, 0, 0, 0, 0, 0 } }, \
    { 221, { KADDR, UBUF8, 0, 0, 0, 0 } }, \
    { 221, { UBUF, KADDR, 0, 0, 0, 0 } },         /* argv in kernel; SYS-035 */ \
    { 221, { UBAD, UBUF8, 0, 0, 0, 0 } }, \
    { 221, { UBUF, 0, 0, 0, 0, 0 } }, \
    { 221, { UBUF, UBAD, 0, 0, 0, 0 } }, \
    { 221, { HUGE, UBUF8, 0, 0, 0, 0 } }, \
    { 221, { UBUF, HUGE, 0, 0, 0, 0 } }, \
    { 221, { UBUF, UBUF8, KADDR, 0, 0, 0 } },     /* envp in kernel */

/* ============= SOCKET/BIND/CONNECT/SENDTO/RECVFROM ================== */
#define SEEDS_NET \
    { 198, { 2, 1, 0, 0, 0, 0 } },                /* AF_INET/SOCK_STREAM */ \
    { 198, { 2, 2, 0, 0, 0, 0 } }, \
    { 198, { 10, 1, 0, 0, 0, 0 } },               /* AF_INET6 unsupported */ \
    { 198, { 2, 3, 0, 0, 0, 0 } },                /* SOCK_RAW */ \
    { 198, { NEG1, NEG1, NEG1, 0, 0, 0 } }, \
    { 198, { 2, 1, 0, 0, 0, 0 } }, /* repeat to exhaust; ATTACK-SYS-051 */ \
    { 198, { 2, 1, 0, 0, 0, 0 } }, \
    { 198, { 2, 1, 0, 0, 0, 0 } }, \
    { 198, { 2, 1, 0, 0, 0, 0 } }, \
    { 198, { 2, 1, 0, 0, 0, 0 } }, \
    { 203, { 3, UBUF, 16, 0, 0, 0 } }, \
    { 203, { 3, 0, 16, 0, 0, 0 } }, \
    { 203, { 3, KADDR, 16, 0, 0, 0 } }, \
    { 203, { 3, UBUF, 4, 0, 0, 0 } },             /* addrlen<8 EINVAL */ \
    { 203, { 3, UBAD, 16, 0, 0, 0 } }, \
    { 206, { 3, UBUF, 16, 0, 0, 0 } }, \
    { 206, { 3, KADDR, 16, 0, 0, 0 } },           /* ATTACK-SYS-047 */ \
    { 206, { 3, UBUF, NEG1, 0, 0, 0 } }, \
    { 206, { 3, UBUF, 16, 0, KADDR, 0 } }, \
    { 207, { 3, UBUF, 16, 0, 0, 0 } }, \
    { 207, { 3, KADDR, 16, 0, 0, 0 } },           /* ATTACK-SYS-048 */ \
    { 207, { 3, UBUF, 16, 0, KADDR, 0 } }, \
    { 207, { 3, UBUF, NEG1, 0, 0, 0 } }, \
    { 211, { 3, UBUF, 0, 0, 0, 0 } },             /* sendmsg */ \
    { 211, { 3, 0, 0, 0, 0, 0 } }, \
    { 211, { 3, KADDR, 0, 0, 0, 0 } },            /* ATTACK-SYS-014 */ \
    { 212, { 3, UBUF, 0, 0, 0, 0 } },             /* recvmsg */ \
    { 212, { 3, KADDR, 0, 0, 0, 0 } },            /* ATTACK-SYS-015 */ \
    { 212, { 3, UBAD, 0, 0, 0, 0 } },

/* ============= PPOLL (73) / PIPE2 (59) =============================== */
#define SEEDS_POLL \
    { 73, { UBUF, 2, 0, 0, 0, 0 } }, \
    { 73, { 0, 2, 0, 0, 0, 0 } }, \
    { 73, { KADDR, 2, 0, 0, 0, 0 } },             /* ATTACK-SYS-049 */ \
    { 73, { UBAD, 2, 0, 0, 0, 0 } }, \
    { 73, { UBUF, 0, 0, 0, 0, 0 } }, \
    { 73, { UBUF, NEG1, 0, 0, 0, 0 } }, \
    { 73, { UBUF, 2, KADDR, 0, 0, 0 } },          /* timeout in kernel */ \
    { 73, { UBUF, 2, 0, KADDR, 0, 0 } },          /* sigmask in kernel */ \
    { 59, { UBUF, 0, 0, 0, 0, 0 } }, \
    { 59, { 0, 0, 0, 0, 0, 0 } }, \
    { 59, { KADDR, 0, 0, 0, 0, 0 } },             /* ATTACK-SYS-050 */ \
    { 59, { UBAD, 0, 0, 0, 0, 0 } },

/* ============= EPOLL (20,21,22) / EVENTFD (19) / TIMERFD (85-87) ===== */
#define SEEDS_EVTFD \
    { 20, { 0, 0, 0, 0, 0, 0 } }, \
    { 20, { 0x80000, 0, 0, 0, 0, 0 } },           /* EPOLL_CLOEXEC */ \
    { 20, { NEG1, 0, 0, 0, 0, 0 } }, \
    { 21, { 3, 1, 1, UBUF, 0, 0 } },              /* add stdin */ \
    { 21, { 3, 1, 1, 0, 0, 0 } },                 /* null event -> EFAULT */ \
    { 21, { 3, 1, 1, KADDR, 0, 0 } },             /* ATTACK-SYS-017 */ \
    { 21, { 3, 1, 3, UBUF, 0, 0 } },              /* watch self; SYS-020 */ \
    { 21, { 3, 99, 1, UBUF, 0, 0 } },             /* bad op */ \
    { 21, { NEG1, 1, 1, UBUF, 0, 0 } }, \
    { 22, { 3, UBUF, 8, 0, 0, 0 } }, \
    { 22, { 3, 0, 8, 0, 0, 0 } }, \
    { 22, { 3, KADDR, 8, 0, 0, 0 } },             /* ATTACK-SYS-018 */ \
    { 22, { 3, UBUF, NEG1, 0, 0, 0 } },           /* maxevents */ \
    { 22, { 3, UBUF, 0, 0, 0, 0 } },              /* <= 0 EINVAL */ \
    { 22, { 3, UBUF, 0x40000000, 0, 0, 0 } },     /* ATTACK-SYS-019 */ \
    { 19, { 0, 0, 0, 0, 0, 0 } }, \
    { 19, { 1, 0, 0, 0, 0, 0 } }, \
    { 19, { 0, 0x80000, 0, 0, 0, 0 } },           /* EFD_CLOEXEC */ \
    { 19, { NEG1, NEG1, 0, 0, 0, 0 } }, \
    { 85, { 0, 0, 0, 0, 0, 0 } },                 /* timerfd_create */ \
    { 85, { 0, 0x80000, 0, 0, 0, 0 } }, \
    { 85, { NEG1, 0, 0, 0, 0, 0 } }, \
    { 86, { 3, 0, UBUF, UBUF, 0, 0 } }, \
    { 86, { 3, 0, 0, UBUF, 0, 0 } }, \
    { 86, { 3, 0, KADDR, UBUF, 0, 0 } },          /* ATTACK-SYS-022 */ \
    { 86, { 3, 0, UBUF, KADDR, 0, 0 } }, \
    { 87, { 3, UBUF, 0, 0, 0, 0 } }, \
    { 87, { 3, 0, 0, 0, 0, 0 } }, \
    { 87, { 3, KADDR, 0, 0, 0, 0 } },             /* ATTACK-SYS-023 */ \
    { 87, { 3, UBAD, 0, 0, 0, 0 } },

/* ============= SIGNALS (134,135,132,131,139) ========================= */
#define SEEDS_SIG \
    { 134, { 10, UBUF, UBUF, 0, 0, 0 } }, \
    { 134, { 10, KADDR, 0, 0, 0, 0 } },           /* ATTACK-SYS-036 */ \
    { 134, { 10, UBUF, KADDR, 0, 0, 0 } },        /* ATTACK-SYS-037 */ \
    { 134, { 9, UBUF, 0, 0, 0, 0 } },             /* SIGKILL EINVAL */ \
    { 134, { 19, UBUF, 0, 0, 0, 0 } },            /* SIGSTOP EINVAL */ \
    { 134, { 0, UBUF, 0, 0, 0, 0 } }, \
    { 134, { 63, UBUF, 0, 0, 0, 0 } }, \
    { 134, { 64, UBUF, 0, 0, 0, 0 } },            /* out of range */ \
    { 134, { 1, 0, 0, 0, 0, 0 } }, \
    { 134, { 1, UBAD, 0, 0, 0, 0 } }, \
    { 135, { 0, UBUF, UBUF, 0, 0, 0 } }, \
    { 135, { 0, KADDR, 0, 0, 0, 0 } },            /* ATTACK-SYS-038 */ \
    { 135, { 1, UBUF, KADDR, 0, 0, 0 } }, \
    { 135, { 99, UBUF, 0, 0, 0, 0 } },            /* bad how */ \
    { 132, { UBUF, UBUF, 0, 0, 0, 0 } }, \
    { 132, { KADDR, 0, 0, 0, 0, 0 } },            /* ATTACK-SYS-039 */ \
    { 132, { 0, KADDR, 0, 0, 0, 0 } }, \
    { 131, { 1, 1, 9, 0, 0, 0 } },                /* tgkill self SIGKILL; SYS-040 */ \
    { 131, { 1, 1, 15, 0, 0, 0 } }, \
    { 131, { 1, 1, 0, 0, 0, 0 } },

/* ============= MISC (uname, clock_gettime, getrandom, sysinfo…) ===== */
#define SEEDS_MISC \
    { 160, { UBUF, 0, 0, 0, 0, 0 } }, \
    { 160, { 0, 0, 0, 0, 0, 0 } }, \
    { 160, { KADDR, 0, 0, 0, 0, 0 } }, \
    { 113, { 0, UBUF, 0, 0, 0, 0 } }, \
    { 113, { 0, 0, 0, 0, 0, 0 } }, \
    { 113, { 0, KADDR, 0, 0, 0, 0 } }, \
    { 278, { UBUF, 64, 0, 0, 0, 0 } }, \
    { 278, { KADDR, 64, 0, 0, 0, 0 } },           /* ATTACK-SYS-046 */ \
    { 278, { UBUF, NEG1, 0, 0, 0, 0 } }, \
    { 179, { UBUF, 0, 0, 0, 0, 0 } }, \
    { 179, { 0, 0, 0, 0, 0, 0 } }, \
    { 179, { KADDR, 0, 0, 0, 0, 0 } }, \
    { 261, { 0, 0, 0, UBUF, 0, 0 } },             /* prlimit64 old */ \
    { 261, { 0, 0, 0, KADDR, 0, 0 } },            /* kernel write */ \
    { 261, { 0, 0, UBUF, UBUF, 0, 0 } }, \
    { 29,  { 1, 0x5401, UBUF, 0, 0, 0 } },        /* ioctl TCGETS */ \
    { 29,  { 1, 0x5401, KADDR, 0, 0, 0 } }, \
    { 29,  { 1, 0x5413, KADDR, 0, 0, 0 } }, \
    { 29,  { 1, 0xdeadbeef, UBUF, 0, 0, 0 } }, \
    { 500, { UBUF, 8, 8, 0, 0, 0 } },             /* blit_framebuffer */ \
    { 500, { KADDR, 8, 8, 0, 0, 0 } }, \
    { 500, { UBUF, NEG1, NEG1, 0, 0, 0 } },

/* ============= DIRECTORY / FD TABLE ================================== */
#define SEEDS_FDMISC \
    { 23, { 0, 0, 0, 0, 0, 0 } },                 /* dup */ \
    { 23, { 0x7fffffff, 0, 0, 0, 0, 0 } }, \
    { 23, { NEG1, 0, 0, 0, 0, 0 } }, \
    { 24, { 0, 1, 0, 0, 0, 0 } },                 /* dup3 */ \
    { 24, { 0, 0x7fffffff, 0, 0, 0, 0 } }, \
    { 25, { 0, 1, 0, 0, 0, 0 } },                 /* fcntl */ \
    { 25, { 0, 99, 0, 0, 0, 0 } }, \
    { 34, { (unsigned long)-100, UBUF, 0755, 0, 0, 0 } }, /* mkdirat */ \
    { 34, { (unsigned long)-100, KADDR, 0755, 0, 0, 0 } }, \
    { 49, { UBUF, 0, 0, 0, 0, 0 } },              /* chdir */ \
    { 49, { KADDR, 0, 0, 0, 0, 0 } }, \
    { 61, { 3, UBUF, 4096, 0, 0, 0 } },           /* getdents64 */ \
    { 61, { 3, KADDR, 4096, 0, 0, 0 } }, \
    { 61, { 3, UBUF, NEG1, 0, 0, 0 } }, \
    { 78, { (unsigned long)-100, UBUF, UBUF, 128, 0, 0 } }, /* readlinkat */ \
    { 78, { (unsigned long)-100, UBUF, KADDR, 128, 0, 0 } }, \
    { 71, { 1, 3, 0, 4096, 0, 0 } },              /* sendfile */ \
    { 71, { 1, NEG1, 0, 4096, 0, 0 } }, \
    { 260, { 0, UBUF, 0, 0, 0, 0 } },             /* wait4 */ \
    { 260, { 0, KADDR, 0, 0, 0, 0 } },

#define ALL_SEEDS \
    SEEDS_READ_WRITE \
    SEEDS_FILE \
    SEEDS_STAT \
    SEEDS_WRITEV \
    SEEDS_MEM \
    SEEDS_FUTEX \
    SEEDS_CLONE \
    SEEDS_EXECVE \
    SEEDS_NET \
    SEEDS_POLL \
    SEEDS_EVTFD \
    SEEDS_SIG \
    SEEDS_MISC \
    SEEDS_FDMISC

static const struct seed seeds[] = { ALL_SEEDS };

#endif /* SEEDS_H */
