/*
 * Sphragis — librosprite.h stub for NetSurf
 * RISC OS sprite format — not applicable to Sphragis.
 */
#ifndef _SPHRAGIS_LIBROSPRITE_H
#define _SPHRAGIS_LIBROSPRITE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    ROSPRITE_OK = 0,
    ROSPRITE_NOMEM = 1,
    ROSPRITE_BADMODE = 2,
    ROSPRITE_EOF = 3,
} rosprite_error;

typedef struct rosprite_area    rosprite_area;
typedef struct rosprite         rosprite;
typedef struct rosprite_header  rosprite_header;

struct rosprite {
    uint32_t  width;
    uint32_t  height;
    bool      has_alpha;
    bool      has_mask;
    uint32_t *image;
};

struct rosprite_area {
    int         sprite_count;
    rosprite  **sprites;
};

rosprite_error rosprite_create_mem_context(const uint8_t *data, size_t len,
                                           void **ctx);
rosprite_error rosprite_load(void *ctx, rosprite_area **result);
void           rosprite_destroy_mem_context(void *ctx);
void           rosprite_destroy_area(rosprite_area *area);

typedef int rosprite_mem_reader;
#endif /* _SPHRAGIS_LIBROSPRITE_H */
