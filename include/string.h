#ifndef _STRING_H
#define _STRING_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Copying */
void *memcpy(void *dest, const void *src, size_t n);
void *memmove(void *dest, const void *src, size_t n);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);

/* Concatenation */
char *strcat(char *dest, const char *src);
char *strncat(char *dest, const char *src, size_t n);

/* Comparison */
int memcmp(const void *s1, const void *s2, size_t n);
int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
int strcasecmp(const char *s1, const char *s2);
int strncasecmp(const char *s1, const char *s2, size_t n);

/* Searching */
void *memchr(const void *s, int c, size_t n);
char *strchr(const char *s, int c);
char *strrchr(const char *s, int c);
char *strstr(const char *haystack, const char *needle);
char *strpbrk(const char *s, const char *accept);
size_t strspn(const char *s, const char *accept);
size_t strcspn(const char *s, const char *reject);

/* Other */
void *memset(void *s, int c, size_t n);
size_t strlen(const char *s);
size_t strnlen(const char *s, size_t maxlen);
char *strdup(const char *s);
char *strndup(const char *s, size_t n);
char *strerror(int errnum);
char *strtok(char *s, const char *delim);
char *strtok_r(char *s, const char *delim, char **saveptr);

#ifdef __cplusplus
}
#endif

#endif
