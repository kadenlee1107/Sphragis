#ifndef UTILS_UTF8_H
#define UTILS_UTF8_H
#include <stdint.h>
#include <stddef.h>
static inline size_t utf8_char_byte_length(const char *s) {
    uint8_t c = (uint8_t)*s;
    if (c < 0x80) return 1;
    if ((c & 0xE0) == 0xC0) return 2;
    if ((c & 0xF0) == 0xE0) return 3;
    return 4;
}
static inline uint32_t utf8_to_ucs4(const char *s, size_t len) {
    (void)len;
    return (uint8_t)*s;
}
#endif
