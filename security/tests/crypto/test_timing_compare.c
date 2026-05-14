// test_timing_compare.c — Timing oracle test for hash comparison
//
// Sphragis src/security/auth.rs:112-118 defines constant_time_eq as:
//
//     let mut diff: u8 = 0;
//     for i in 0..32 { diff |= a[i] ^ b[i]; }
//     diff == 0
//
// Good: no early exit. This harness reproduces that loop in C and
// compares wall-clock timing against memcmp (which short-circuits) for
// three candidate hashes that differ from the target at byte 0, byte
// 16, and byte 31 respectively. If constant_time_eq is truly CT the
// means should overlap within a few stddev.
//
// Under the current Sphragis build this should confirm the comparison
// itself is CT. The RISK (see ATTACK-CRYPTO-009, -010) is not the
// compare but the primitives FEEDING the compare (AES/GHASH leak
// timing via table/branch).
//
// Build: cc -O0 -fno-inline test_timing_compare.c -o tcmp
// (use -O0 so the compiler doesn't optimize the hot loop away)

#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <time.h>

#define ITERS 10000000

static inline uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

/* Model of Sphragis constant_time_eq(&[u8;32], &[u8;32]) */
static int ct_eq(const uint8_t a[32], const uint8_t b[32]) {
    uint8_t d = 0;
    for (int i = 0; i < 32; i++) d |= a[i] ^ b[i];
    return d == 0;
}

static uint64_t time_many(const uint8_t t[32], const uint8_t c[32], int use_memcmp) {
    volatile int sink = 0;
    uint64_t s = now_ns();
    for (int i = 0; i < ITERS; i++) {
        sink ^= use_memcmp ? (memcmp(t, c, 32) == 0) : ct_eq(t, c);
    }
    uint64_t e = now_ns();
    (void)sink;
    return e - s;
}

int main(void) {
    uint8_t target[32];
    for (int i = 0; i < 32; i++) target[i] = (uint8_t)(i * 7 + 13);

    uint8_t differ_at_0[32];   memcpy(differ_at_0, target, 32);   differ_at_0[0] ^= 1;
    uint8_t differ_at_16[32];  memcpy(differ_at_16, target, 32);  differ_at_16[16] ^= 1;
    uint8_t differ_at_31[32];  memcpy(differ_at_31, target, 32);  differ_at_31[31] ^= 1;

    printf("test: constant_time_eq vs memcmp, ITERS=%d\n", ITERS);

    for (int pass = 0; pass < 3; pass++) {
        printf("\n== pass %d ==\n", pass);

        uint64_t ct0  = time_many(target, differ_at_0,  0);
        uint64_t ct16 = time_many(target, differ_at_16, 0);
        uint64_t ct31 = time_many(target, differ_at_31, 0);

        uint64_t mc0  = time_many(target, differ_at_0,  1);
        uint64_t mc16 = time_many(target, differ_at_16, 1);
        uint64_t mc31 = time_many(target, differ_at_31, 1);

        printf("  ct_eq  byte0=%llu ns  byte16=%llu ns  byte31=%llu ns\n",
            (unsigned long long)ct0, (unsigned long long)ct16, (unsigned long long)ct31);
        printf("  memcmp byte0=%llu ns  byte16=%llu ns  byte31=%llu ns\n",
            (unsigned long long)mc0, (unsigned long long)mc16, (unsigned long long)mc31);
    }

    return 0;
}
