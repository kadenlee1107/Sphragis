#ifndef _BITS_TYPES_MBSTATE_T_H
#define _BITS_TYPES_MBSTATE_T_H

typedef struct {
    int __count;
    union { unsigned int __wch; char __wchb[4]; } __value;
} mbstate_t;

#endif
