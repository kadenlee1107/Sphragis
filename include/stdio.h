#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>
#include <stdarg.h>

typedef struct _FILE FILE;

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

#define EOF (-1)

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define BUFSIZ   1024
#define FILENAME_MAX 4096

/* Formatted output */
int printf(const char *fmt, ...);
int fprintf(FILE *stream, const char *fmt, ...);
int sprintf(char *buf, const char *fmt, ...);
int snprintf(char *buf, size_t size, const char *fmt, ...);
int vprintf(const char *fmt, va_list ap);
int vfprintf(FILE *stream, const char *fmt, va_list ap);
int vsprintf(char *buf, const char *fmt, va_list ap);
int vsnprintf(char *buf, size_t size, const char *fmt, va_list ap);

/* Formatted input */
int scanf(const char *fmt, ...);
int fscanf(FILE *stream, const char *fmt, ...);
int sscanf(const char *buf, const char *fmt, ...);

/* Character I/O */
int fgetc(FILE *stream);
int fputc(int c, FILE *stream);
int getc(FILE *stream);
int putc(int c, FILE *stream);
int getchar(void);
int putchar(int c);
int ungetc(int c, FILE *stream);

/* String I/O */
char *fgets(char *buf, int size, FILE *stream);
int fputs(const char *s, FILE *stream);
int puts(const char *s);

/* Direct I/O */
size_t fread(void *buf, size_t size, size_t count, FILE *stream);
size_t fwrite(const void *buf, size_t size, size_t count, FILE *stream);

/* File operations */
FILE *fopen(const char *path, const char *mode);
FILE *freopen(const char *path, const char *mode, FILE *stream);
int fclose(FILE *stream);
int fflush(FILE *stream);

/* File positioning */
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
void rewind(FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
void clearerr(FILE *stream);

/* File removal/rename */
int remove(const char *path);
int rename(const char *oldpath, const char *newpath);

/* Error reporting */
void perror(const char *s);

/* Temporary files */
FILE *tmpfile(void);
char *tmpnam(char *s);

#endif
