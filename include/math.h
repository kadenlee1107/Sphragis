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

#define isnan(x)   __builtin_isnan(x)
#define isinf(x)   __builtin_isinf(x)
#define isfinite(x) __builtin_isfinite(x)
#define signbit(x) __builtin_signbit(x)
#define fpclassify(x) __builtin_fpclassify(0, 1, 4, 3, 2, (x))

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
int ilogb(double x);
double logb(double x);

#endif
