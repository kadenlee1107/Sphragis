#ifndef _ASSERT_H
#define _ASSERT_H

#ifdef NDEBUG
#define assert(expr) ((void)0)
#else
__attribute__((noreturn)) void abort(void);
#define assert(expr) \
    ((expr) ? ((void)0) : \
     (abort(), (void)0))
#endif

/* C11 static_assert */
#define static_assert _Static_assert

#endif
