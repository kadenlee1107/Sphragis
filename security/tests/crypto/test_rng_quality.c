// test_rng_quality.c — Sphragis getrandom quality probe
//
// Calls SYS_getrandom (Sphragis number 278) 1,000,000 times, 1 byte each,
// writes the stream to stdout, and prints a simple entropy / run-test
// summary on stderr.
//
// Expected on a real CSPRNG: each byte value ~1/256 frequency, runs
// distribution ~N(N-1)/2 transitions, Shannon entropy close to 8.0
// bits/byte.
//
// Observed against Sphragis sys_getrandom (see ATTACK-CRYPTO-002): byte
// values cluster near the low byte of cntpct_el0, entropy well below 3
// bits/byte, long runs of identical bytes within each 8-byte counter
// window.
//
// Build (userland harness, not in kernel): cc -O2 test_rng_quality.c -o rngq
// Run:   ./rngq > rng.bin 2> rng.stats
//
// This test is a userland scaffold. To exercise Sphragis itself, rebuild
// the binary to invoke the kernel syscall directly via svc #0 from a
// BatCave guest.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <unistd.h>
#include <sys/syscall.h>

#ifndef SYS_getrandom
#define SYS_getrandom 278
#endif

#define N 1000000

int main(void) {
    unsigned char *buf = malloc(N);
    if (!buf) { perror("malloc"); return 1; }

    // One byte at a time, to match Sphragis sys_getrandom call pattern.
    for (size_t i = 0; i < N; i++) {
        if (syscall(SYS_getrandom, &buf[i], 1, 0) != 1) {
            perror("getrandom");
            return 1;
        }
    }

    fwrite(buf, 1, N, stdout);

    // Byte histogram
    unsigned long hist[256] = {0};
    for (size_t i = 0; i < N; i++) hist[buf[i]]++;

    // Shannon entropy
    double H = 0.0;
    for (int v = 0; v < 256; v++) {
        if (!hist[v]) continue;
        double p = (double)hist[v] / (double)N;
        H -= p * log2(p);
    }

    // Runs (transitions)
    unsigned long runs = 1;
    for (size_t i = 1; i < N; i++) if (buf[i] != buf[i-1]) runs++;

    fprintf(stderr, "entropy  = %.4f bits/byte (ideal 8.0000)\n", H);
    fprintf(stderr, "runs     = %lu (ideal ~%lu)\n", runs, (unsigned long)((N - 1) * 255 / 256));
    fprintf(stderr, "min_byte = %lu  max_byte = %lu\n",
            *(&hist[0] + (hist[0] ? 0 : 1)),
            0UL);

    // Print the 8 most-common byte values
    fprintf(stderr, "top8 bytes (value:count):\n");
    for (int top = 0; top < 8; top++) {
        int best = 0;
        for (int v = 1; v < 256; v++) if (hist[v] > hist[best]) best = v;
        fprintf(stderr, "  0x%02x : %lu\n", best, hist[best]);
        hist[best] = 0;
    }

    free(buf);
    return 0;
}
