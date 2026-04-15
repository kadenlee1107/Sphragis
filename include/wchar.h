#ifndef _WCHAR_H
#define _WCHAR_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#include <bits/types/mbstate_t.h>

typedef unsigned int wint_t;
#define WEOF ((wint_t)-1)

size_t wcslen(const wchar_t *s);
int wcscmp(const wchar_t *s1, const wchar_t *s2);
wchar_t *wcscpy(wchar_t *dest, const wchar_t *src);
wchar_t *wcsncpy(wchar_t *dest, const wchar_t *src, size_t n);
wchar_t *wcschr(const wchar_t *s, wchar_t c);
wchar_t *wcsrchr(const wchar_t *s, wchar_t c);
wchar_t *wcsstr(const wchar_t *s1, const wchar_t *s2);
wchar_t *wmemchr(const wchar_t *s, wchar_t c, size_t n);
int wcsncmp(const wchar_t *s1, const wchar_t *s2, size_t n);
wchar_t *wmemcpy(wchar_t *dest, const wchar_t *src, size_t n);
wchar_t *wmemmove(wchar_t *dest, const wchar_t *src, size_t n);
wchar_t *wmemset(wchar_t *dest, wchar_t c, size_t n);
int wmemcmp(const wchar_t *s1, const wchar_t *s2, size_t n);
int swprintf(wchar_t *s, size_t n, const wchar_t *fmt, ...);
int vswprintf(wchar_t *s, size_t n, const wchar_t *fmt, __builtin_va_list ap);
long wcstol(const wchar_t *s, wchar_t **endp, int base);
unsigned long wcstoul(const wchar_t *s, wchar_t **endp, int base);
float wcstof(const wchar_t *s, wchar_t **endp);
double wcstod(const wchar_t *s, wchar_t **endp);
long double wcstold(const wchar_t *s, wchar_t **endp);
long long wcstoll(const wchar_t *s, wchar_t **endp, int base);
unsigned long long wcstoull(const wchar_t *s, wchar_t **endp, int base);
int wctob(wint_t c);
wint_t btowc(int c);
wchar_t *wcscat(wchar_t *dest, const wchar_t *src);
size_t wcsspn(const wchar_t *s, const wchar_t *accept);
size_t wcscspn(const wchar_t *s, const wchar_t *reject);
wchar_t *wcspbrk(const wchar_t *s, const wchar_t *accept);
wchar_t *wcstok(wchar_t *s, const wchar_t *delim, wchar_t **saveptr);
size_t wcsftime(wchar_t *s, size_t max, const wchar_t *fmt, const void *tm);
size_t mbsrtowcs(wchar_t *dest, const char **src, size_t len, mbstate_t *ps);
size_t wcsrtombs(char *dest, const wchar_t **src, size_t len, mbstate_t *ps);
int wcwidth(wchar_t c);
wint_t getwc(void *f);
wint_t putwc(wchar_t c, void *f);
wint_t ungetwc(wint_t c, void *f);
wint_t fgetwc(void *f);
wint_t fputwc(wchar_t c, void *f);
size_t wcstombs(char *dest, const wchar_t *src, size_t n);
size_t mbstowcs(wchar_t *dest, const char *src, size_t n);
int mbtowc(wchar_t *pwc, const char *s, size_t n);
int wctomb(char *s, wchar_t wchar);
size_t mbrlen(const char *s, size_t n, mbstate_t *ps);
size_t mbrtowc(wchar_t *pwc, const char *s, size_t n, mbstate_t *ps);
size_t wcrtomb(char *s, wchar_t wc, mbstate_t *ps);
int mbsinit(const mbstate_t *ps);

#ifdef __cplusplus
}
#endif

#endif
