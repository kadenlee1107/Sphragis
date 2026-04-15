// Bat_OS — malloc.h (GNU extension header)
#ifndef _MALLOC_H
#define _MALLOC_H

#include <stdlib.h>
#include <stddef.h>

// GNU extension: malloc_usable_size
static inline size_t malloc_usable_size(void *ptr) {
    if (!ptr) return 0;
    // Our malloc stores size in a header 16 bytes before the pointer
    size_t total = *((size_t *)((char *)ptr - 16));
    return total > 16 ? total - 16 : 0;
}

#endif
