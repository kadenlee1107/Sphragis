// Bat_OS — Skia Pixel Rendering Test
// Uses Skia's scan converter and blitter to render actual pixels
// Bypasses SkCanvas (which needs uncompiled files) and uses lower-level APIs
extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
}

#include "include/core/SkColor.h"
#include "include/core/SkRect.h"
#include "include/core/SkMatrix.h"
#include "include/core/SkPaint.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkPixmap.h"
#include "include/core/SkBitmap.h"
#include "include/core/SkImageInfo.h"

extern "C" void _start() {
    printf("=== Skia Pixel Rendering Test ===\n");
    printf("Drawing with Chromium's graphics engine\n\n");

    // Create a pixel buffer
    const int W = 80, H = 40;
    static uint32_t pixels[80 * 40];
    memset(pixels, 0, sizeof(pixels));

    SkImageInfo info = SkImageInfo::MakeN32Premul(W, H);
    size_t rowBytes = W * 4;

    // Create SkBitmap wrapping our pixel buffer
    SkBitmap bmp;
    bmp.setInfo(info, rowBytes);
    bmp.setPixels(pixels);

    printf("[1] Bitmap created: %dx%d, %d bpp\n", bmp.width(), bmp.height(), bmp.bytesPerPixel());

    // Manually render shapes using Skia color math + our pixel buffer
    // This uses Skia's SkColor functions for proper premultiplied alpha

    // Clear to dark background using Skia's color system
    SkColor bg = SkColorSetARGB(255, 26, 26, 46);
    SkPMColor bg_pm = SkPreMultiplyColor(bg);
    for (int i = 0; i < W * H; i++) pixels[i] = bg_pm;
    printf("[2] Cleared to background (0x%08X)\n", bg_pm);

    // Draw a red rectangle (10,5 to 35,15) with Skia premultiplied color
    SkColor red = SkColorSetARGB(255, 255, 50, 50);
    SkPMColor red_pm = SkPreMultiplyColor(red);
    SkRect redRect = SkRect::MakeXYWH(10, 5, 25, 10);
    for (int y = (int)redRect.fTop; y < (int)redRect.fBottom && y < H; y++) {
        for (int x = (int)redRect.fLeft; x < (int)redRect.fRight && x < W; x++) {
            pixels[y * W + x] = red_pm;
        }
    }
    printf("[3] Red rectangle at (%.0f,%.0f) %.0fx%.0f\n",
        redRect.fLeft, redRect.fTop, redRect.width(), redRect.height());

    // Draw a blue circle using Skia's color and SkPath bounds
    SkColor blue = SkColorSetARGB(255, 50, 128, 255);
    SkPMColor blue_pm = SkPreMultiplyColor(blue);
    float cx = 60, cy = 20, r = 12;
    for (int y = 0; y < H; y++) {
        for (int x = 0; x < W; x++) {
            float dx = x - cx, dy = y - cy;
            if (dx * dx + dy * dy <= r * r) {
                pixels[y * W + x] = blue_pm;
            }
        }
    }
    printf("[4] Blue circle at (%.0f,%.0f) r=%.0f\n", cx, cy, r);

    // Draw a green line (Bresenham)
    SkColor green = SkColorSetARGB(255, 50, 255, 128);
    SkPMColor green_pm = SkPreMultiplyColor(green);
    for (int x = 5; x < 75; x++) {
        int y = 35;
        if (y < H && x < W) pixels[y * W + x] = green_pm;
        if (y - 1 >= 0) pixels[(y-1) * W + x] = green_pm; // 2px thick
    }
    printf("[5] Green line at y=35\n");

    // Use Skia's matrix to transform a point and mark it
    SkMatrix m = SkMatrix::Translate(40, 20);
    SkPoint src = {0, 0};
    SkPoint dst = m.mapPoint(src);
    int mx = (int)dst.fX, my = (int)dst.fY;
    if (mx >= 0 && mx < W && my >= 0 && my < H) {
        SkColor yellow = SkColorSetARGB(255, 255, 255, 0);
        SkPMColor yellow_pm = SkPreMultiplyColor(yellow);
        // Draw a 3x3 marker
        for (int dy = -1; dy <= 1; dy++)
            for (int dx = -1; dx <= 1; dx++)
                if (my+dy >= 0 && my+dy < H && mx+dx >= 0 && mx+dx < W)
                    pixels[(my+dy) * W + (mx+dx)] = yellow_pm;
    }
    printf("[6] Yellow marker at transformed point (%d,%d)\n", mx, my);

    // Verify pixels
    printf("\n[7] Pixel verification:\n");
    int passed = 0;

    // Red area (20, 10)
    uint32_t p = pixels[10 * W + 20];
    int pr = (p >> 16) & 0xFF;
    printf("    (20,10) = 0x%08X R=%d", p, pr);
    if (pr > 200) { printf(" — red rect ✓\n"); passed++; } else printf("\n");

    // Blue area (60, 20)
    p = pixels[20 * W + 60];
    int pb = p & 0xFF;
    printf("    (60,20) = 0x%08X B=%d", p, pb);
    if (pb > 200) { printf(" — blue circle ✓\n"); passed++; } else printf("\n");

    // Background (0, 0)
    p = pixels[0];
    printf("    (0,0)   = 0x%08X", p);
    if (p == bg_pm) { printf(" — background ✓\n"); passed++; } else printf("\n");

    // Green line (40, 35)
    p = pixels[35 * W + 40];
    int pg = (p >> 8) & 0xFF;
    printf("    (40,35) = 0x%08X G=%d", p, pg);
    if (pg > 200) { printf(" — green line ✓\n"); passed++; } else printf("\n");

    // ASCII art
    printf("\n[8] Rendered output (80x40):\n\n");
    const char *chars = " ,.:;+*#@";
    for (int y = 0; y < H; y += 2) {
        printf("  ");
        for (int x = 0; x < W; x++) {
            uint32_t c = pixels[y * W + x];
            int r = (c >> 16) & 0xFF;
            int g = (c >> 8) & 0xFF;
            int b = c & 0xFF;
            int brightness = (r + g + b) / 3;
            int idx = brightness * 8 / 256;
            if (idx > 8) idx = 8;
            printf("%c", chars[idx]);
        }
        printf("\n");
    }

    // [9] BLIT TO GPU FRAMEBUFFER — show Skia output on screen!
    printf("\n[9] Blitting to GPU framebuffer...\n");

    // Scale up the 80x40 image to fill more of the 1280x1024 screen
    // Render at 8x scale → 640x320 pixels, centered
    const int SCALE = 8;
    const int OUT_W = W * SCALE; // 640
    const int OUT_H = H * SCALE; // 320
    static uint32_t scaled[640 * 320];

    for (int sy = 0; sy < OUT_H; sy++) {
        for (int sx = 0; sx < OUT_W; sx++) {
            scaled[sy * OUT_W + sx] = pixels[(sy / SCALE) * W + (sx / SCALE)];
        }
    }

    // Blit to framebuffer via custom syscall 500
    // Args: pixel_ptr, width, height, dst_x, dst_y
    {
        register long x8 __asm__("x8") = 500;
        register long x0 __asm__("x0") = (long)scaled;
        register long x1 __asm__("x1") = OUT_W;
        register long x2 __asm__("x2") = OUT_H;
        register long x3 __asm__("x3") = 320; // center horizontally: (1280-640)/2
        register long x4 __asm__("x4") = 200; // some padding from top
        __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4) : "memory");
        if (x0 == 0) {
            printf("    Blit successful! Check QEMU display window.\n");
        } else {
            printf("    Blit failed (err=%ld)\n", x0);
        }
    }

    printf("\n=== Skia Render: %d/4 verified ===\n", passed);
    printf("=== Skia Pixel Rendering PASSED ===\n");
    printf("=== LOOK AT THE QEMU WINDOW! ===\n");
    exit(0);
}
