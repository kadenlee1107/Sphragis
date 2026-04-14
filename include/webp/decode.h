#ifndef _WEBP_DECODE_H
#define _WEBP_DECODE_H
#include <stddef.h>
#include <stdint.h>
uint8_t *WebPDecodeRGBA(const uint8_t *data, size_t sz, int *w, int *h);
void WebPFree(void *ptr);
typedef int VP8StatusCode;
typedef struct { int width; int height; int has_alpha; } WebPBitstreamFeatures;
int WebPGetFeatures(const uint8_t *data, size_t sz, WebPBitstreamFeatures *f);
#endif
