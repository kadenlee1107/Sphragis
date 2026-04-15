#ifndef _MATH_H
#define _MATH_H

#define M_PI       3.14159265358979323846
#define M_PI_2     1.57079632679489661923
#define M_PI_4     0.78539816339744830962
#define M_E        2.71828182845904523536
#define M_LOG2E    1.44269504088896340736
#define M_LOG10E   0.43429448190325182765
#define M_LN2      0.69314718055994530942
#define M_LN10     2.30258509299404568402
#define M_SQRT2    1.41421356237309504880

#define INFINITY   __builtin_inf()
#define NAN        __builtin_nan("")
#define HUGE_VAL   __builtin_huge_val()
#define HUGE_VALF  __builtin_huge_valf()

#ifndef __cplusplus
#define isnan(x)   __builtin_isnan(x)
#define isinf(x)   __builtin_isinf(x)
#define isfinite(x) __builtin_isfinite(x)
#define signbit(x) __builtin_signbit(x)
#endif
#define fpclassify(x) __builtin_fpclassify(0, 1, 4, 3, 2, (x))

#define FP_NAN       0
#define FP_INFINITE  1
#define FP_ZERO      2
#define FP_SUBNORMAL 3
#define FP_NORMAL    4

/* Trigonometric */
double sin(double x);
double cos(double x);
double tan(double x);
double asin(double x);
double acos(double x);
double atan(double x);
double atan2(double y, double x);

float sinf(float x);
float cosf(float x);
float tanf(float x);
float asinf(float x);
float acosf(float x);
float atanf(float x);
float atan2f(float y, float x);

/* Hyperbolic */
double sinh(double x);
double cosh(double x);
double tanh(double x);

/* Exponential and logarithmic */
double exp(double x);
double exp2(double x);
double log(double x);
double log2(double x);
double log10(double x);
double frexp(double x, int *exp);
double ldexp(double x, int exp);

float expf(float x);
float exp2f(float x);
float logf(float x);
float log2f(float x);
float log10f(float x);

/* Power */
double pow(double base, double exp);
double sqrt(double x);
double cbrt(double x);
double hypot(double x, double y);

float powf(float base, float exp);
float sqrtf(float x);
float cbrtf(float x);
float hypotf(float x, float y);

/* Rounding */
double ceil(double x);
double floor(double x);
double round(double x);
double trunc(double x);
double rint(double x);
long lround(double x);
long long llround(double x);
long lrint(double x);

float ceilf(float x);
float floorf(float x);
float roundf(float x);
float truncf(float x);

/* Remainder */
double fmod(double x, double y);
double remainder(double x, double y);

float fmodf(float x, float y);

/* Absolute value */
double fabs(double x);
float fabsf(float x);

/* Min/max */
double fmin(double x, double y);
double fmax(double x, double y);
float fminf(float x, float y);
float fmaxf(float x, float y);

/* Decomposition */
double modf(double x, double *iptr);

/* Copy sign */
double copysign(double x, double y);
float copysignf(float x, float y);

/* Scaling */
double scalbn(double x, int n);
float scalbnf(float x, int n);
int ilogb(double x);
double logb(double x);

/* Next representable value */
double nextafter(double x, double y);
float nextafterf(float x, float y);
double nexttoward(double x, long double y);

/* NaN generation */
double nan(const char *tagp);
float nanf(const char *tagp);

/* Classification functions (C99 — also available as macros above) */
#ifdef __cplusplus
extern "C" {
#endif
static inline int __isnan_fn(double x) { return __builtin_isnan(x); }
static inline int __isinf_fn(double x) { return __builtin_isinf(x); }
static inline int __isfinite_fn(double x) { return __builtin_isfinite(x); }
static inline int __isnanf_fn(float x) { return __builtin_isnan(x); }
static inline int __isinff_fn(float x) { return __builtin_isinf(x); }
static inline int __isfinitef_fn(float x) { return __builtin_isfinite(x); }
#ifdef __cplusplus
}
#endif

/* FMA (fused multiply-add) */
double fma(double x, double y, double z);
float fmaf(float x, float y, float z);
long lrint(double x);
long lrintf(float x);
long long llrint(double x);

/* Error function */
double erf(double x);
double erfc(double x);

/* Gamma */
double lgamma(double x);
double tgamma(double x);

float nearbyintf(float x);
double nearbyint(double x);

#endif
