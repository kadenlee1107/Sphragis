/*
 * mem_bomb.c — DoS stress: memory exhaustion.
 *
 * Guest-side harness for Bat_OS BatCave Linux runner. Intended to be
 * cross-compiled for aarch64-linux-musl and loaded by the ELF loader.
 *
 * Covers:
 *   ATTACK-DOS-001  mmap storm  (4 KiB allocations until ENOMEM)
 *   ATTACK-DOS-002  mmap/munmap silent leak (munmap is a no-op in Bat_OS)
 *   ATTACK-DOS-005  huge MAP_STACK (1 GiB single alloc)
 *
 * Usage (inside a cave):
 *   ./mem_bomb small   # DOS-001
 *   ./mem_bomb leak    # DOS-002
 *   ./mem_bomb huge    # DOS-005
 *
 * Success criterion for the ATTACKER: eventually get ENOMEM for ANY
 * allocation from any other cave. Observation requires a second cave
 * running a "canary" process — see run_all.sh.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <errno.h>

static void mem_bomb_small(void) {
    unsigned long count = 0;
    for (;;) {
        void *p = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                       MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        if (p == MAP_FAILED) {
            fprintf(stderr, "[mem_bomb/small] mmap failed after %lu allocs, errno=%d\n",
                    count, errno);
            return;
        }
        /* Touch every page so lazy-fault kernels still commit. Bat_OS
           commits eagerly so this is redundant but cheap. */
        memset(p, 0xA5, 4096);
        count++;
        if ((count & 0x3FF) == 0)
            fprintf(stderr, "[mem_bomb/small] %lu pages allocated\n", count);
    }
}

static void mem_bomb_leak(void) {
    /* Because sys_munmap is a no-op in Bat_OS, this loop drains the
       frame pool while *looking* well-behaved. */
    unsigned long count = 0;
    for (;;) {
        void *p = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                       MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        if (p == MAP_FAILED) {
            fprintf(stderr, "[mem_bomb/leak] mmap failed after %lu cycles, errno=%d\n",
                    count, errno);
            return;
        }
        ((volatile char*)p)[0] = 1;
        /* Pretend to clean up. Bat_OS silently does nothing. */
        munmap(p, 4096);
        count++;
        if ((count & 0xFFF) == 0)
            fprintf(stderr, "[mem_bomb/leak] %lu cycles (frames actually leaked)\n", count);
    }
}

static void mem_bomb_huge(void) {
    size_t sz = (size_t)1 << 30;   /* 1 GiB */
    void *p = mmap(NULL, sz, PROT_READ|PROT_WRITE,
                   MAP_PRIVATE|MAP_ANONYMOUS|MAP_STACK, -1, 0);
    if (p == MAP_FAILED) {
        fprintf(stderr, "[mem_bomb/huge] mmap 1 GiB failed errno=%d\n", errno);
        return;
    }
    fprintf(stderr, "[mem_bomb/huge] got 1 GiB at %p — 50%% of the 2 GiB pool\n", p);
    /* Hold. Another cave that tries to allocate 128 KiB should now fail. */
    for (;;) { /* park */ }
}

int main(int argc, char **argv) {
    const char *mode = (argc > 1) ? argv[1] : "small";
    if      (!strcmp(mode, "small")) mem_bomb_small();
    else if (!strcmp(mode, "leak"))  mem_bomb_leak();
    else if (!strcmp(mode, "huge"))  mem_bomb_huge();
    else {
        fprintf(stderr, "usage: %s [small|leak|huge]\n", argv[0]);
        return 2;
    }
    return 0;
}
