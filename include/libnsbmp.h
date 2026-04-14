/*
 * Bat_OS — libnsbmp.h stub for NetSurf
 * Minimal BMP/ICO decoder interface.
 */
#ifndef _BATOS_LIBNSBMP_H
#define _BATOS_LIBNSBMP_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    BMP_OK                = 0,
    BMP_INSUFFICIENT_MEMORY = 1,
    BMP_INSUFFICIENT_DATA   = 2,
    BMP_DATA_ERROR          = 3,
} bmp_result;

typedef struct {
    void *(*bitmap_create)(int width, int height, unsigned int state);
    void  (*bitmap_destroy)(void *bitmap);
    unsigned char *(*bitmap_get_buffer)(void *bitmap);
    size_t (*bitmap_get_bpp)(void *bitmap);
    void   (*bitmap_modified)(void *bitmap);
    void   (*bitmap_set_opaque)(void *bitmap, bool opaque);
    bool   (*bitmap_test_opaque)(void *bitmap);
} bmp_bitmap_callback_vt;

typedef struct bmp_image {
    uint32_t width;
    uint32_t height;
    bool     decoded;
    void    *bitmap;
    /* opaque internals */
    void    *_priv;
} bmp_image;

typedef struct ico_collection {
    uint32_t width;
    uint32_t height;
    /* opaque internals */
    void    *_priv;
} ico_collection;

void       bmp_create(bmp_image *bmp, bmp_bitmap_callback_vt *callbacks);
bmp_result bmp_analyse(bmp_image *bmp, size_t size, const uint8_t *data);
bmp_result bmp_decode(bmp_image *bmp);
void       bmp_finalise(bmp_image *bmp);

void       ico_create(ico_collection *ico, bmp_bitmap_callback_vt *callbacks);
bmp_result ico_analyse(ico_collection *ico, size_t size, const uint8_t *data);
bmp_image *ico_find(ico_collection *ico, uint16_t width, uint16_t height);
void       ico_finalise(ico_collection *ico);

#define BMP_CLEAR_MEMORY 2
#define BMP_OPAQUE 1
#endif /* _BATOS_LIBNSBMP_H */
