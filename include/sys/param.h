/*
 * Sphragis — sys/param.h stub for NetSurf
 * Common constants and utility macros.
 */
#ifndef _SPHRAGIS_SYS_PARAM_H
#define _SPHRAGIS_SYS_PARAM_H

#include <limits.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

#ifndef MAXPATHLEN
#define MAXPATHLEN PATH_MAX
#endif

#ifndef PAGE_SIZE
#define PAGE_SIZE 4096
#endif

#ifndef MIN
#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#endif

#ifndef MAX
#define MAX(a, b) (((a) > (b)) ? (a) : (b))
#endif

#ifndef CLAMP
#define CLAMP(val, lo, hi) MIN(MAX((val), (lo)), (hi))
#endif

#ifndef howmany
#define howmany(x, y) (((x) + ((y) - 1)) / (y))
#endif

#ifndef roundup
#define roundup(x, y) ((((x) + ((y) - 1)) / (y)) * (y))
#endif

#endif /* _SPHRAGIS_SYS_PARAM_H */
