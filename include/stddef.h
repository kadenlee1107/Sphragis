#ifndef _STDDEF_H
#define _STDDEF_H

typedef unsigned long size_t;
typedef long ptrdiff_t;
#ifndef __cplusplus
typedef int wchar_t;
#endif

#define NULL ((void *)0)
#define offsetof(type, member) __builtin_offsetof(type, member)

typedef long double max_align_t;

#endif
