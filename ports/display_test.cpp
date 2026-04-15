// Bat_OS — Full Display Test
// FreeType fonts + Skia colors + GPU framebuffer
// Renders anti-aliased text and shapes on screen
extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
    #include <ft2build.h>
    #include FT_FREETYPE_H
}

#include "include/core/SkColor.h"
#include "include/core/SkRect.h"
#include "include/core/SkImageInfo.h"

// Embedded font
extern "C" {
    extern const unsigned char _binary_font_ttf_start[];
    extern const unsigned char _binary_font_ttf_end[];
}

// Syscall helper
static long syscall5(long nr, long a0, long a1, long a2, long a3, long a4) {
    register long x8 __asm__("x8") = nr;
    register long x0 __asm__("x0") = a0;
    register long x1 __asm__("x1") = a1;
    register long x2 __asm__("x2") = a2;
    register long x3 __asm__("x3") = a3;
    register long x4 __asm__("x4") = a4;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4) : "memory");
    return x0;
}
#define SYS_BLIT 500

// Screen buffer (section of screen)
const int SCR_W = 800, SCR_H = 500;
static uint32_t screen[800 * 500];

// Alpha blend a pixel onto the screen
static inline void blend_pixel(int x, int y, uint32_t color, uint8_t alpha) {
    if (x < 0 || x >= SCR_W || y < 0 || y >= SCR_H) return;
    if (alpha == 0) return;

    uint32_t dst = screen[y * SCR_W + x];
    int dr = (dst >> 16) & 0xFF, dg = (dst >> 8) & 0xFF, db = dst & 0xFF;
    int sr = (color >> 16) & 0xFF, sg = (color >> 8) & 0xFF, sb = color & 0xFF;

    int r = dr + ((sr - dr) * alpha) / 255;
    int g = dg + ((sg - dg) * alpha) / 255;
    int b = db + ((sb - db) * alpha) / 255;

    screen[y * SCR_W + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
}

// Draw a filled rectangle
static void draw_rect(int x, int y, int w, int h, uint32_t color) {
    for (int dy = 0; dy < h; dy++)
        for (int dx = 0; dx < w; dx++)
            blend_pixel(x + dx, y + dy, color, 255);
}

// Draw a filled circle with anti-aliasing
static void draw_circle(int cx, int cy, int r, uint32_t color) {
    for (int dy = -r - 1; dy <= r + 1; dy++) {
        for (int dx = -r - 1; dx <= r + 1; dx++) {
            float dist = (float)(dx * dx + dy * dy);
            float edge = (float)(r * r);
            if (dist <= edge - r) {
                blend_pixel(cx + dx, cy + dy, color, 255);
            } else if (dist <= edge + r) {
                // Anti-alias the edge
                float aa = 1.0f - (dist - edge + r) / (2.0f * r);
                if (aa > 0) blend_pixel(cx + dx, cy + dy, color, (uint8_t)(aa * 255));
            }
        }
    }
}

// Render text with FreeType
static void draw_text(FT_Face face, const char *text, int x, int y, int size, uint32_t color) {
    FT_Set_Pixel_Sizes(face, 0, size);
    int pen_x = x;
    for (int i = 0; text[i]; i++) {
        FT_UInt gi = FT_Get_Char_Index(face, text[i]);
        if (FT_Load_Glyph(face, gi, FT_LOAD_RENDER)) continue;

        FT_GlyphSlot slot = face->glyph;
        FT_Bitmap *bmp = &slot->bitmap;

        int bx = pen_x + slot->bitmap_left;
        int by = y - slot->bitmap_top;

        for (unsigned int row = 0; row < bmp->rows; row++) {
            for (unsigned int col = 0; col < bmp->width; col++) {
                uint8_t alpha = bmp->buffer[row * bmp->pitch + col];
                if (alpha > 0) {
                    blend_pixel(bx + col, by + row, color, alpha);
                }
            }
        }
        pen_x += slot->advance.x >> 6;
    }
}

extern "C" void _start() {
    printf("=== Bat_OS Display Demo ===\n\n");

    // Init FreeType
    FT_Library ft;
    FT_Init_FreeType(&ft);
    FT_Face face;
    unsigned long font_size = (unsigned long)(_binary_font_ttf_end - _binary_font_ttf_start);
    FT_New_Memory_Face(ft, _binary_font_ttf_start, (FT_Long)font_size, 0, &face);
    printf("[1] FreeType: %s loaded (%ld bytes)\n", face->family_name, font_size);

    // Clear screen to dark background
    SkColor bg = SkColorSetARGB(255, 18, 18, 32);
    uint32_t bg32 = 0xFF201212; // BGRA
    for (int i = 0; i < SCR_W * SCR_H; i++) screen[i] = bg32;

    // Draw header bar
    draw_rect(0, 0, SCR_W, 60, 0xFF3D1A1A);
    printf("[2] Header bar drawn\n");

    // Draw title: "Bat_OS" in large white text
    draw_text(face, "Bat_OS", 30, 45, 40, 0xFFFFFFFF);
    printf("[3] Title rendered: Bat_OS (40px Verdana)\n");

    // Draw subtitle
    draw_text(face, "Bare-Metal ARM64 OS with Chromium Graphics", 30, 100, 18, 0xFFA0A0A0);
    printf("[4] Subtitle rendered (18px)\n");

    // Draw Skia-colored shapes
    // Red rectangle
    draw_rect(50, 140, 200, 80, 0xFF0000FF);
    draw_text(face, "Skia SkRect", 70, 190, 16, 0xFFFFFFFF);

    // Blue circle
    draw_circle(420, 180, 50, 0xFFFF8000);
    draw_text(face, "SkCircle", 390, 250, 14, 0xFFA0A0A0);

    // Green rectangle
    draw_rect(550, 140, 180, 80, 0xFF80FF00);
    draw_text(face, "FreeType", 580, 190, 16, 0xFF000000);

    printf("[5] Shapes drawn (rect, circle, rect)\n");

    // Draw feature list
    const char *features[] = {
        "TLS 1.3 + HTTPS",
        "CSS Engine (libcss)",
        "FreeType Fonts",
        "Skia 2D Graphics",
        "C++ Runtime",
        "POSIX Layer",
    };
    for (int i = 0; i < 6; i++) {
        int fy = 290 + i * 28;
        draw_circle(55, fy + 8, 6, 0xFF00FF88);
        draw_text(face, features[i], 75, fy + 14, 16, 0xFFD0D0D0);
    }
    printf("[6] Feature list rendered (6 items)\n");

    // Draw bottom bar
    draw_rect(0, SCR_H - 40, SCR_W, 40, 0xFF3D1A1A);
    draw_text(face, "Security: Zero Dependencies. Zero Trust.", 30, SCR_H - 12, 14, 0xFF808080);
    printf("[7] Footer drawn\n");

    // Blit to GPU!
    printf("[8] Blitting %dx%d to GPU framebuffer...\n", SCR_W, SCR_H);
    long result = syscall5(SYS_BLIT, (long)screen, SCR_W, SCR_H, 240, 100);
    if (result == 0) {
        printf("    SUCCESS — Skia + FreeType graphics on GPU display!\n");
    } else {
        printf("    Blit returned %ld\n", result);
    }

    FT_Done_Face(face);
    FT_Done_FreeType(ft);

    printf("\n=== Display Demo COMPLETE ===\n");

    // Don't exit — keep display visible
    printf("Display will remain visible. Press Ctrl+A X to exit QEMU.\n");
    while (1) { __asm__ volatile("wfe"); }
}
