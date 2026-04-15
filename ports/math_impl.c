// Bat_OS — Math function implementations
// Force function symbols to be emitted (not inlined to builtins)
#include <stddef.h>

// Use __attribute__((noinline)) and volatile to prevent optimization away

__attribute__((noinline,visibility("default")))
double ceil(double x) { double r; __asm__ volatile("frintp %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double floor(double x) { double r; __asm__ volatile("frintm %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double fabs(double x) { double r; __asm__ volatile("fabs %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double sqrt(double x) { double r; __asm__ volatile("fsqrt %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double round(double x) { double r; __asm__ volatile("frinta %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double trunc(double x) { double r; __asm__ volatile("frintz %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
double rint(double x) { double r; __asm__ volatile("frintx %d0, %d1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float ceilf(float x) { float r; __asm__ volatile("frintp %s0, %s1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float floorf(float x) { float r; __asm__ volatile("frintm %s0, %s1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float fabsf(float x) { float r; __asm__ volatile("fabs %s0, %s1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float sqrtf(float x) { float r; __asm__ volatile("fsqrt %s0, %s1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float roundf(float x) { float r; __asm__ volatile("frinta %s0, %s1" : "=w"(r) : "w"(x)); return r; }

__attribute__((noinline,visibility("default")))
float truncf(float x) { float r; __asm__ volatile("frintz %s0, %s1" : "=w"(r) : "w"(x)); return r; }

// Functions that can't use single instructions — need real implementations
#define NOINLINE __attribute__((noinline,visibility("default")))

NOINLINE double fmod(double x, double y) { return x - trunc(x / y) * y; }
NOINLINE float fmodf(float x, float y) { return x - truncf(x / y) * y; }
NOINLINE double exp(double x) { /* Taylor series approximation */
    double sum = 1.0, term = 1.0;
    for (int i = 1; i < 20; i++) { term *= x / i; sum += term; }
    return sum;
}
NOINLINE float expf(float x) { return (float)exp((double)x); }
NOINLINE double log(double x) {
    if (x <= 0) return -1e308;
    double r = 0, y = (x - 1) / (x + 1);
    double y2 = y * y, term = y;
    for (int i = 0; i < 20; i++) { r += term / (2 * i + 1); term *= y2; }
    return 2 * r;
}
NOINLINE float logf(float x) { return (float)log((double)x); }
NOINLINE double log2(double x) { return log(x) * 1.4426950408889634; }
NOINLINE float log2f(float x) { return (float)log2((double)x); }
NOINLINE double log10(double x) { return log(x) * 0.4342944819032518; }
NOINLINE float log10f(float x) { return (float)log10((double)x); }
NOINLINE double exp2(double x) { return exp(x * 0.6931471805599453); }
NOINLINE float exp2f(float x) { return (float)exp2((double)x); }
NOINLINE double pow(double x, double y) { return exp(y * log(x)); }
NOINLINE float powf(float x, float y) { return (float)pow((double)x, (double)y); }
NOINLINE double sin(double x) {
    while (x > 3.14159265) x -= 6.28318530; while (x < -3.14159265) x += 6.28318530;
    double x2 = x*x, r = x;
    double t = x; t *= -x2/(2*3); r += t; t *= -x2/(4*5); r += t;
    t *= -x2/(6*7); r += t; t *= -x2/(8*9); r += t; t *= -x2/(10*11); r += t;
    return r;
}
NOINLINE float sinf(float x) { return (float)sin((double)x); }
NOINLINE double cos(double x) { return sin(x + 1.5707963267948966); }
NOINLINE float cosf(float x) { return (float)cos((double)x); }
NOINLINE double tan(double x) { return sin(x) / cos(x); }
NOINLINE float tanf(float x) { return (float)tan((double)x); }
NOINLINE double atan(double x) {
    if (x > 1) return 1.5707963 - atan(1/x);
    if (x < -1) return -1.5707963 - atan(1/x);
    double x2 = x*x, r = x, t = x;
    t *= -x2/3; r += t; t *= -x2*3/5; r += t;
    t *= -x2*5/7; r += t; t *= -x2*7/9; r += t;
    return r;
}
NOINLINE float atanf(float x) { return (float)atan((double)x); }
NOINLINE double atan2(double y, double x) {
    if (x > 0) return atan(y/x);
    if (x < 0 && y >= 0) return atan(y/x) + 3.14159265;
    if (x < 0 && y < 0) return atan(y/x) - 3.14159265;
    if (y > 0) return 1.5707963;
    return -1.5707963;
}
NOINLINE float atan2f(float y, float x) { return (float)atan2((double)y, (double)x); }
NOINLINE double asin(double x) { return atan2(x, sqrt(1 - x*x)); }
NOINLINE float asinf(float x) { return (float)asin((double)x); }
NOINLINE double acos(double x) { return atan2(sqrt(1 - x*x), x); }
NOINLINE float acosf(float x) { return (float)acos((double)x); }
NOINLINE double copysign(double x, double y) { return fabs(x) * (y < 0 ? -1.0 : 1.0); }
NOINLINE float copysignf(float x, float y) { return fabsf(x) * (y < 0 ? -1.0f : 1.0f); }
NOINLINE double hypot(double x, double y) { return sqrt(x*x + y*y); }
NOINLINE float hypotf(float x, float y) { return sqrtf(x*x + y*y); }
NOINLINE double cbrt(double x) { return (x < 0) ? -pow(-x, 1.0/3.0) : pow(x, 1.0/3.0); }
NOINLINE float cbrtf(float x) { return (float)cbrt((double)x); }
NOINLINE double sinh(double x) { return (exp(x) - exp(-x)) / 2; }
NOINLINE double cosh(double x) { return (exp(x) + exp(-x)) / 2; }
NOINLINE double tanh(double x) { return sinh(x) / cosh(x); }
NOINLINE double frexp(double x, int *e) { *e = 0; return x; }
NOINLINE double ldexp(double x, int e) { for(int i=0;i<e;i++) x*=2; for(int i=0;i>e;i--) x/=2; return x; }
NOINLINE double scalbn(double x, int n) { return ldexp(x, n); }
NOINLINE float scalbnf(float x, int n) { return (float)ldexp((double)x, n); }
NOINLINE double modf(double x, double *i) { *i = trunc(x); return x - *i; }
NOINLINE double remainder(double x, double y) { return x - round(x/y) * y; }
NOINLINE double nextafter(double x, double y) { (void)y; return x; }
NOINLINE float nextafterf(float x, float y) { (void)y; return x; }
NOINLINE double nexttoward(double x, long double y) { (void)y; return x; }
NOINLINE double fma(double x, double y, double z) { return x*y+z; }
NOINLINE float fmaf(float x, float y, float z) { return x*y+z; }
NOINLINE double nan(const char *t) { (void)t; return 0.0/0.0; }
NOINLINE float nanf(const char *t) { (void)t; return 0.0f/0.0f; }
NOINLINE double fmin(double x, double y) { return x<y?x:y; }
NOINLINE double fmax(double x, double y) { return x>y?x:y; }
NOINLINE float fminf(float x, float y) { return x<y?x:y; }
NOINLINE float fmaxf(float x, float y) { return x>y?x:y; }
NOINLINE long lrint(double x) { return (long)(x+0.5); }
NOINLINE long lrintf(float x) { return (long)(x+0.5f); }
NOINLINE long lround(double x) { return (long)round(x); }
NOINLINE long long llround(double x) { return (long long)round(x); }
NOINLINE double erf(double x) { (void)x; return 0; }
NOINLINE double erfc(double x) { (void)x; return 1; }
NOINLINE double lgamma(double x) { (void)x; return 0; }
NOINLINE double tgamma(double x) { return x; }
NOINLINE int ilogb(double x) { (void)x; return 0; }
NOINLINE double logb(double x) { return log2(fabs(x)); }
NOINLINE int abs(int x) { return x<0?-x:x; }

NOINLINE long sysconf(int name) { (void)name; return 1; }
NOINLINE int tolower(int c) { return (c>='A'&&c<='Z') ? c+32 : c; }
NOINLINE int toupper(int c) { return (c>='a'&&c<='z') ? c-32 : c; }
