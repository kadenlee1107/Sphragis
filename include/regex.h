/*
 * Bat_OS — regex.h stub for NetSurf
 * Minimal POSIX regex declarations.
 */
#ifndef _BATOS_REGEX_H
#define _BATOS_REGEX_H

#include <stddef.h>

/* Compile flags */
#define REG_EXTENDED  1
#define REG_ICASE     2
#define REG_NOSUB     4
#define REG_NEWLINE   8

/* Execution flags */
#define REG_NOTBOL    1
#define REG_NOTEOL    2

/* Error codes */
#define REG_NOMATCH   1
#define REG_BADPAT    2
#define REG_ECOLLATE  3
#define REG_ECTYPE    4
#define REG_EESCAPE   5
#define REG_ESUBREG   6
#define REG_EBRACK    7
#define REG_EPAREN    8
#define REG_EBRACE    9
#define REG_BADBR    10
#define REG_ERANGE   11
#define REG_ESPACE   12
#define REG_BADRPT   13

typedef struct {
    size_t re_nsub;
    /* opaque internals */
    void  *_internal;
} regex_t;

typedef struct {
    int rm_so;   /* start offset */
    int rm_eo;   /* end offset   */
} regmatch_t;

int    regcomp(regex_t *preg, const char *pattern, int cflags);
int    regexec(const regex_t *preg, const char *string,
               size_t nmatch, regmatch_t pmatch[], int eflags);
size_t regerror(int errcode, const regex_t *preg,
                char *errbuf, size_t errbuf_size);
void   regfree(regex_t *preg);

#endif /* _BATOS_REGEX_H */
