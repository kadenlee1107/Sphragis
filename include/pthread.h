// Sphragis — pthread.h stub
#ifndef _PTHREAD_H
#define _PTHREAD_H

#include <stddef.h>
#include <stdint.h>

typedef unsigned long pthread_t;
typedef struct { int __attr; } pthread_attr_t;
typedef struct { int __lock; } pthread_mutex_t;
typedef struct { int __attr; } pthread_mutexattr_t;
typedef struct { int __cond; } pthread_cond_t;
typedef struct { int __attr; } pthread_condattr_t;
typedef struct { int __once; } pthread_once_t;
typedef unsigned int pthread_key_t;
typedef struct { int __rw; } pthread_rwlock_t;
typedef struct { int __attr; } pthread_rwlockattr_t;

#define PTHREAD_MUTEX_INITIALIZER {0}
#define PTHREAD_COND_INITIALIZER {0}
#define PTHREAD_ONCE_INIT {0}
#define PTHREAD_RWLOCK_INITIALIZER {0}

int pthread_create(pthread_t *t, const pthread_attr_t *a, void*(*fn)(void*), void *arg);
int pthread_join(pthread_t t, void **retval);
int pthread_detach(pthread_t t);
pthread_t pthread_self(void);
int pthread_equal(pthread_t a, pthread_t b);

int pthread_mutex_init(pthread_mutex_t *m, const pthread_mutexattr_t *a);
int pthread_mutex_destroy(pthread_mutex_t *m);
int pthread_mutex_lock(pthread_mutex_t *m);
int pthread_mutex_trylock(pthread_mutex_t *m);
int pthread_mutex_unlock(pthread_mutex_t *m);

int pthread_cond_init(pthread_cond_t *c, const pthread_condattr_t *a);
int pthread_cond_destroy(pthread_cond_t *c);
int pthread_cond_wait(pthread_cond_t *c, pthread_mutex_t *m);
int pthread_cond_signal(pthread_cond_t *c);
int pthread_cond_broadcast(pthread_cond_t *c);

int pthread_once(pthread_once_t *once, void (*fn)(void));

int pthread_key_create(pthread_key_t *key, void (*dtor)(void*));
int pthread_key_delete(pthread_key_t key);
void *pthread_getspecific(pthread_key_t key);
int pthread_setspecific(pthread_key_t key, const void *val);

int pthread_rwlock_init(pthread_rwlock_t *rw, const pthread_rwlockattr_t *a);
int pthread_rwlock_destroy(pthread_rwlock_t *rw);
int pthread_rwlock_rdlock(pthread_rwlock_t *rw);
int pthread_rwlock_wrlock(pthread_rwlock_t *rw);
int pthread_rwlock_unlock(pthread_rwlock_t *rw);

#endif
