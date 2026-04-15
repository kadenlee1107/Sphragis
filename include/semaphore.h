// Bat_OS — semaphore.h stub
#ifndef _SEMAPHORE_H
#define _SEMAPHORE_H

typedef struct { int __val; } sem_t;

int sem_init(sem_t *sem, int pshared, unsigned int value);
int sem_destroy(sem_t *sem);
int sem_wait(sem_t *sem);
int sem_trywait(sem_t *sem);
int sem_post(sem_t *sem);
int sem_getvalue(sem_t *sem, int *sval);

#endif
