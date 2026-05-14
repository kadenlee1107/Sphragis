// Sphragis — dlfcn.h stub (no dynamic loading)
#ifndef _DLFCN_H
#define _DLFCN_H

#define RTLD_LAZY   1
#define RTLD_NOW    2
#define RTLD_LOCAL  0
#define RTLD_GLOBAL 256

static inline void *dlopen(const char *f, int flags) { (void)f; (void)flags; return 0; }
static inline void *dlsym(void *h, const char *s) { (void)h; (void)s; return 0; }
static inline int dlclose(void *h) { (void)h; return 0; }
static inline char *dlerror(void) { return "dlopen not supported"; }

#endif
