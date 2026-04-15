#ifndef FP16_H
#define FP16_H
#include <stdint.h>

// Half-precision float stubs
static inline float fp16_ieee_to_fp32_value(uint16_t h) {
    // Simple conversion: sign + exponent + mantissa
    uint32_t sign = (h & 0x8000) << 16;
    uint32_t exp = (h >> 10) & 0x1F;
    uint32_t mant = h & 0x3FF;
    uint32_t f;
    if (exp == 0) { f = sign; }
    else if (exp == 31) { f = sign | 0x7F800000 | (mant << 13); }
    else { f = sign | ((exp + 112) << 23) | (mant << 13); }
    float result;
    __builtin_memcpy(&result, &f, 4);
    return result;
}

static inline uint16_t fp16_ieee_from_fp32_value(float val) {
    uint32_t f;
    __builtin_memcpy(&f, &val, 4);
    uint32_t sign = (f >> 16) & 0x8000;
    int exp = ((f >> 23) & 0xFF) - 112;
    uint32_t mant = (f >> 13) & 0x3FF;
    if (exp <= 0) return sign;
    if (exp >= 31) return sign | 0x7C00;
    return sign | (exp << 10) | mant;
}

#endif
