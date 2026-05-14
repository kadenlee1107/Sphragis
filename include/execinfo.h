// Sphragis — execinfo.h stub (no backtrace support)
#ifndef _EXECINFO_H
#define _EXECINFO_H

static inline int backtrace(void **buf, int size) { (void)buf; (void)size; return 0; }
static inline char **backtrace_symbols(void *const *buf, int size) { (void)buf; (void)size; return 0; }
static inline void backtrace_symbols_fd(void *const *buf, int size, int fd) { (void)buf; (void)size; (void)fd; }

#endif
