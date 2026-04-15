#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>
#include <sys/types.h>

#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

#define _SC_NPROCESSORS_ONLN 84
#define _SC_PAGE_SIZE        30

long sysconf(int name);

/* Access mode flags for access() */
#define F_OK 0
#define R_OK 4
#define W_OK 2
#define X_OK 1

/* Read/write */
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);

/* File operations */
int close(int fd);
off_t lseek(int fd, off_t offset, int whence);
int dup(int oldfd);
int dup2(int oldfd, int newfd);
int pipe(int pipefd[2]);
int unlink(const char *path);
int rmdir(const char *path);
int access(const char *path, int mode);
int link(const char *oldpath, const char *newpath);
int symlink(const char *target, const char *linkpath);
ssize_t readlink(const char *path, char *buf, size_t bufsiz);
int truncate(const char *path, off_t length);
int ftruncate(int fd, off_t length);
int fsync(int fd);

/* Directory */
char *getcwd(char *buf, size_t size);
int chdir(const char *path);

/* Process */
pid_t getpid(void);
pid_t getppid(void);
uid_t getuid(void);
gid_t getgid(void);

/* Sleep */
unsigned int sleep(unsigned int seconds);
int usleep(unsigned int usec);

/* Misc */
int isatty(int fd);
long sysconf(int name);

#define _SC_PAGE_SIZE 30
#define _SC_PAGESIZE _SC_PAGE_SIZE

char *realpath(const char *path, char *resolved);
#endif
