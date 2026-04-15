#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

void *malloc(size_t size);
void *calloc(size_t n, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);

__attribute__((noreturn)) void exit(int status);
__attribute__((noreturn)) void abort(void);

int abs(int x);
long labs(long x);
long long llabs(long long x);

long strtol(const char *s, char **endp, int base);
unsigned long strtoul(const char *s, char **endp, int base);
long long strtoll(const char *s, char **endp, int base);
unsigned long long strtoull(const char *s, char **endp, int base);
double strtod(const char *s, char **endp);
float strtof(const char *s, char **endp);

int atoi(const char *s);
long atol(const char *s);
double atof(const char *s);

void qsort(void *base, size_t n, size_t size, int (*cmp)(const void *, const void *));
void *bsearch(const void *key, const void *base, size_t n, size_t size,
              int (*cmp)(const void *, const void *));

char *getenv(const char *name);
int system(const char *command);

int atexit(void (*func)(void));
int rand(void);
void srand(unsigned int seed);

typedef struct {
    int quot;
    int rem;
} div_t;

typedef struct {
    long quot;
    long rem;
} ldiv_t;

#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1
int posix_memalign(void **memptr, size_t alignment, size_t size);

#define RAND_MAX     0x7FFFFFFF

#ifdef __cplusplus
}
#endif

#endif
