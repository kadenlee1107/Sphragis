/*
 * Sphragis — nsgif.h stub for NetSurf
 * Minimal GIF decoder interface (libnsgif).
 */
#ifndef _SPHRAGIS_NSGIF_H
#define _SPHRAGIS_NSGIF_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    NSGIF_OK            = 0,
    NSGIF_ERR_OOM       = 1,
    NSGIF_ERR_DATA      = 2,
    NSGIF_ERR_BAD_FRAME = 3,
    NSGIF_ERR_DATA_FRAME = 4,
    NSGIF_ERR_END_OF_DATA = 5,
    NSGIF_ERR_ANIMATION  = 6,
} nsgif_error;

typedef enum {
    NSGIF_BITMAP_FMT_R8G8B8A8 = 0,
    NSGIF_BITMAP_FMT_B8G8R8A8 = 1,
    NSGIF_BITMAP_FMT_A8R8G8B8 = 2,
    NSGIF_BITMAP_FMT_A8B8G8R8 = 3,
} nsgif_bitmap_fmt;

typedef struct {
    nsgif_bitmap_fmt bitmap_fmt;
} nsgif_bitmap_cb_vt;

typedef struct nsgif nsgif_t;

typedef struct {
    uint32_t width;
    uint32_t height;
    uint32_t frame_count;
    bool     loop;
    int      loop_max;
    uint32_t background;
} nsgif_info_t;

typedef struct {
    uint32_t delay;       /* in centiseconds */
    uint32_t rect_x;
    uint32_t rect_y;
    uint32_t rect_w;
    uint32_t rect_h;
    bool     opaque;
    bool     disposal;
    bool     transparency;
} nsgif_frame_info_t;

nsgif_error    nsgif_create(const nsgif_bitmap_cb_vt *bitmap_vt,
                            nsgif_bitmap_fmt fmt,
                            nsgif_t **gif_out);
void           nsgif_destroy(nsgif_t *gif);
nsgif_error    nsgif_data_scan(nsgif_t *gif, size_t size, const uint8_t *data);
void           nsgif_data_complete(nsgif_t *gif);
const nsgif_info_t *nsgif_get_info(const nsgif_t *gif);
nsgif_error    nsgif_frame_prepare(nsgif_t *gif, uint32_t *area,
                                    uint32_t *delay_cs, uint32_t *frame);
nsgif_error    nsgif_frame_decode(nsgif_t *gif, uint32_t frame,
                                   uint32_t **bitmap);
const nsgif_frame_info_t *nsgif_get_frame_info(const nsgif_t *gif,
                                                uint32_t frame);
const char    *nsgif_strerror(nsgif_error err);

#endif /* _SPHRAGIS_NSGIF_H */
