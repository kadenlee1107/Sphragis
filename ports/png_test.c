// Bat_OS — libpng Test
// Verifies zlib + libpng compile and link correctly
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <png.h>

void _start(void) {
    printf("=== libpng Test ===\n\n");

    // Check libpng version
    printf("[1] libpng version: %s\n", png_get_libpng_ver(NULL));

    // Create a minimal 4x4 PNG in memory and decode it
    // For now, just verify the library initializes
    png_structp png = png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (!png) {
        printf("[FAIL] png_create_read_struct returned NULL\n");
        exit(1);
    }
    printf("[2] png_create_read_struct: OK\n");

    png_infop info = png_create_info_struct(png);
    if (!info) {
        printf("[FAIL] png_create_info_struct returned NULL\n");
        exit(1);
    }
    printf("[3] png_create_info_struct: OK\n");

    png_destroy_read_struct(&png, &info, NULL);
    printf("[4] Cleanup: OK\n");

    printf("\n=== libpng Test PASSED ===\n");
    exit(0);
}
