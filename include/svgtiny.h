/*
 * Sphragis — svgtiny.h stub for NetSurf
 * Minimal SVG Tiny declarations.
 */
#ifndef _SPHRAGIS_SVGTINY_H
#define _SPHRAGIS_SVGTINY_H

#include <stddef.h>
#include <stdint.h>

typedef enum {
    svgtiny_OK             = 0,
    svgtiny_OUT_OF_MEMORY  = 1,
    svgtiny_LIBDOM_ERROR   = 2,
    svgtiny_NOT_SVG        = 3,
    svgtiny_SVG_ERROR      = 4,
} svgtiny_code;

typedef enum {
    svgtiny_PATH_MOVE  = 0,
    svgtiny_PATH_CLOSE = 1,
    svgtiny_PATH_LINE  = 2,
    svgtiny_PATH_BEZIER = 3,
} svgtiny_path_type;

typedef struct {
    float        *path;
    unsigned int  path_length;
    char         *text;
    float         text_x;
    float         text_y;
    uint32_t      fill;
    uint32_t      stroke;
    float         stroke_width;
} svgtiny_shape;

typedef struct svgtiny_diagram {
    unsigned int    width;
    unsigned int    height;
    svgtiny_shape  *shape;
    unsigned int    shape_count;
    unsigned short  error_line;
    const char     *error_message;
} svgtiny_diagram;

svgtiny_diagram *svgtiny_create(void);
svgtiny_code     svgtiny_parse(svgtiny_diagram *diagram,
                                const char *buffer, size_t size,
                                const char *url,
                                int viewport_width, int viewport_height);
void             svgtiny_free(svgtiny_diagram *diagram);

#define svgtiny_TRANSPARENT 0x1000000

#define svgtiny_RED(c) (((c) >> 0) & 0xFF)
#define svgtiny_GREEN(c) (((c) >> 8) & 0xFF)
#define svgtiny_BLUE(c) (((c) >> 16) & 0xFF)
#endif /* _SPHRAGIS_SVGTINY_H */
