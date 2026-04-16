/*
 * Bat_OS — hello_shm: /batos/fb0 MAP_SHARED smoke test.
 *
 * Purpose
 *   Exercise the Chromium display bridge without dragging in Chromium.
 *   Opens /batos/fb0, mmap()s it MAP_SHARED, fills the pixel buffer with
 *   a color gradient, bumps the `seq` counter, and polls `last_seen_seq`
 *   to confirm the kernel blit kthread observed and consumed the frame.
 *
 * Expected behavior
 *   - After mmap, we can read the 'BFB1' magic in the header.
 *   - After writing pixels + incrementing seq, within a few 60 Hz ticks the
 *     kernel copies our pixels to the virtio-gpu scanout and echoes our seq
 *     into the header's `last_seen_seq` field (offset 28).
 *   - The visible display should show a smooth red→blue horizontal gradient.
 *
 * Build (after BatCave provides a musl cross-compiler)
 *   aarch64-linux-musl-gcc -static -O2 \
 *       -o tests/hello_shm tests/hello_shm.c
 *
 * Run (from the Bat_OS shell, once loader/execve support is wired)
 *   /batos/hello_shm
 *
 * Exit codes
 *   0   success — kernel ack observed
 *   1   open() failed
 *   2   mmap() failed
 *   3   bad magic in header
 *   4   kernel didn't ack within timeout
 *
 * Notes on the current BatCave shape (as of Phase 5)
 *   - O_CREAT is not required: /batos/fb0 is pre-created by VFS init.
 *   - ftruncate is a no-op for ChromiumFb nodes (the region is pre-sized).
 *   - MAP_SHARED returns the pre-allocated physical base — same VA as the
 *     kernel blit kthread uses. Single-process content_shell is fine.
 */

#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define FB_PATH          "/batos/fb0"
#define FB_MAGIC         0x42464231u   /* 'BFB1' */
#define FB_HEADER_SIZE   128u
#define FB_WIDTH         1280u
#define FB_HEIGHT        1024u
#define FB_REGION_SIZE   (FB_HEADER_SIZE + FB_WIDTH * FB_HEIGHT * 4u)

/* Header layout — must match src/batcave/linux/vfs.rs and
 * src/drivers/display/chromium_blit.rs. */
struct fb_header {
    uint32_t magic;          /* 0  */
    uint32_t version;        /* 4  */
    uint32_t width;          /* 8  */
    uint32_t height;         /* 12 */
    uint32_t stride;         /* 16 */
    uint32_t format;         /* 20 */
    uint32_t seq;            /* 24  — we bump this */
    uint32_t last_seen_seq;  /* 28  — kernel writes this after blit */
    uint32_t damage_x;       /* 32 */
    uint32_t damage_y;       /* 36 */
    uint32_t damage_w;       /* 40 */
    uint32_t damage_h;       /* 44 */
    uint64_t pts_ns;         /* 48 */
    uint32_t reserved[8];    /* 56..88 */
};

int main(void) {
    int fd = open(FB_PATH, O_RDWR);
    if (fd < 0) {
        perror("open /batos/fb0");
        return 1;
    }

    void *p = mmap(NULL, FB_REGION_SIZE, PROT_READ | PROT_WRITE,
                   MAP_SHARED, fd, 0);
    if (p == MAP_FAILED) {
        perror("mmap");
        close(fd);
        return 2;
    }
    close(fd);  /* mapping survives; same as Linux semantics */

    volatile struct fb_header *hdr = (volatile struct fb_header *)p;
    uint8_t *pixels = (uint8_t *)p + FB_HEADER_SIZE;

    if (hdr->magic != FB_MAGIC) {
        fprintf(stderr, "bad magic: 0x%08x (want 0x%08x)\n",
                hdr->magic, FB_MAGIC);
        return 3;
    }
    printf("hello_shm: mapped %u x %u, stride=%u, format=%u\n",
           hdr->width, hdr->height, hdr->stride, hdr->format);

    /* Paint a red→blue horizontal gradient in BGRA8888 premul.
     * At x=0: (B=0, G=0, R=255, A=255)  → red
     * At x=W-1: (B=255, G=0, R=0, A=255) → blue */
    for (uint32_t y = 0; y < hdr->height; ++y) {
        uint8_t *row = pixels + y * hdr->stride;
        for (uint32_t x = 0; x < hdr->width; ++x) {
            uint8_t *px = row + x * 4;
            uint32_t t = (x * 255u) / (hdr->width - 1);
            px[0] = (uint8_t)t;         /* B */
            px[1] = 0;                   /* G */
            px[2] = (uint8_t)(255u - t); /* R */
            px[3] = 255;                 /* A */
        }
    }

    /* Publish: full-screen damage, then release-bump seq. On aarch64 we
     * rely on mmap's PROT_WRITE mappings being cache-coherent with the
     * kernel blit kthread — they are, because /batos/fb0 is identity-
     * mapped into the same kernel address space. */
    hdr->damage_x = 0;
    hdr->damage_y = 0;
    hdr->damage_w = hdr->width;
    hdr->damage_h = hdr->height;
    __atomic_thread_fence(__ATOMIC_RELEASE);

    uint32_t my_seq = __atomic_add_fetch(&hdr->seq, 1, __ATOMIC_RELEASE);
    printf("hello_shm: published seq=%u, waiting for kernel ack...\n", my_seq);

    /* Wait up to ~2 s for the kernel to blit and echo `last_seen_seq`.
     * The blit kthread ticks at ~scheduler-tick frequency (100 Hz nominal),
     * so one tick is well under 2 s. */
    for (int i = 0; i < 200; ++i) {
        uint32_t ack = __atomic_load_n(&hdr->last_seen_seq, __ATOMIC_ACQUIRE);
        if (ack >= my_seq) {
            printf("hello_shm: kernel ack=%u (ok)\n", ack);
            munmap(p, FB_REGION_SIZE);
            return 0;
        }
        /* 10 ms nap — tune if we don't have usleep yet in BatCave. */
        struct timespec ts = { 0, 10 * 1000 * 1000 };
        nanosleep(&ts, NULL);
    }

    fprintf(stderr, "hello_shm: kernel ack timeout (last_seen_seq stuck at %u)\n",
            hdr->last_seen_seq);
    munmap(p, FB_REGION_SIZE);
    return 4;
}
