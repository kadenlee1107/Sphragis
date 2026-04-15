// Bat_OS — FreeType Font Rendering Test
// Loads a TrueType font, renders glyphs with proper anti-aliasing
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ft2build.h>
#include FT_FREETYPE_H

// Embedded font data — linked in by the test harness
extern const unsigned char _binary_font_ttf_start[];
extern const unsigned char _binary_font_ttf_end[];

void _start(void) {
    printf("=== FreeType Font Rendering Test ===\n\n");

    FT_Library library;
    FT_Error error;

    // Initialize FreeType
    error = FT_Init_FreeType(&library);
    if (error) {
        printf("[FAIL] FT_Init_FreeType: error %d\n", error);
        exit(1);
    }
    printf("[1] FreeType initialized\n");

    // Load font from memory
    FT_Face face;
    unsigned long font_size = (unsigned long)(_binary_font_ttf_end - _binary_font_ttf_start);
    printf("    Font data: %lu bytes\n", font_size);

    error = FT_New_Memory_Face(library,
        _binary_font_ttf_start, (FT_Long)font_size,
        0, &face);
    if (error) {
        printf("[FAIL] FT_New_Memory_Face: error %d\n", error);
        exit(1);
    }
    printf("[2] Font loaded: %s %s\n", face->family_name, face->style_name);
    printf("    Glyphs: %ld, Units/EM: %d\n",
        face->num_glyphs, face->units_per_EM);

    // Set character size (16px)
    error = FT_Set_Pixel_Sizes(face, 0, 16);
    if (error) {
        printf("[FAIL] FT_Set_Pixel_Sizes: error %d\n", error);
        exit(1);
    }
    printf("[3] Pixel size set to 16px\n");

    // Render some characters
    const char *test = "Bat_OS";
    printf("\n[4] Rendering \"%s\":\n\n", test);

    for (int c = 0; test[c]; c++) {
        FT_UInt glyph_index = FT_Get_Char_Index(face, test[c]);
        error = FT_Load_Glyph(face, glyph_index, FT_LOAD_RENDER);
        if (error) {
            printf("    '%c': FAILED (err=%d)\n", test[c], error);
            continue;
        }

        FT_GlyphSlot slot = face->glyph;
        FT_Bitmap *bmp = &slot->bitmap;

        printf("    '%c': %dx%d bitmap, bearingX=%d bearingY=%d advance=%ld\n",
            test[c],
            bmp->width, bmp->rows,
            slot->bitmap_left, slot->bitmap_top,
            slot->advance.x >> 6);

        // Print ASCII art preview of the glyph (first 8 rows)
        int show_rows = bmp->rows < 12 ? bmp->rows : 12;
        for (int y = 0; y < show_rows; y++) {
            printf("      ");
            int show_cols = bmp->width < 20 ? bmp->width : 20;
            for (int x = 0; x < show_cols; x++) {
                unsigned char pixel = bmp->buffer[y * bmp->pitch + x];
                if (pixel > 200) printf("#");
                else if (pixel > 128) printf("*");
                else if (pixel > 64) printf(".");
                else if (pixel > 16) printf(",");
                else printf(" ");
            }
            printf("\n");
        }
    }

    FT_Done_Face(face);
    FT_Done_FreeType(library);

    printf("\n=== FreeType Test PASSED ===\n");
    exit(0);
}
