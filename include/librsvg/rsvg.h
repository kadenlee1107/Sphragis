/*
 * Bat_OS — librsvg/rsvg.h stub for NetSurf
 * Minimal declarations; SVG not supported in Bat_OS browser.
 */
#ifndef _BATOS_RSVG_H
#define _BATOS_RSVG_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct _RsvgHandle       RsvgHandle;
typedef struct _RsvgDimensionData {
    int    width;
    int    height;
    double em;
    double ex;
} RsvgDimensionData;

RsvgHandle *rsvg_handle_new(void);
RsvgHandle *rsvg_handle_new_from_data(const uint8_t *data, size_t data_len,
                                       void **error);
bool        rsvg_handle_write(RsvgHandle *handle, const uint8_t *buf,
                               size_t count, void **error);
bool        rsvg_handle_close(RsvgHandle *handle, void **error);
void        rsvg_handle_get_dimensions(RsvgHandle *handle,
                                        RsvgDimensionData *dimension_data);
bool        rsvg_handle_render_cairo(RsvgHandle *handle, void *cr);
void        rsvg_handle_free(RsvgHandle *handle);

/* Replaced by g_object_unref in real glib builds */
#define g_object_unref(obj) ((void)(obj))

typedef struct {} cairo_surface_t;
typedef struct {} cairo_t;
#endif /* _BATOS_RSVG_H */
