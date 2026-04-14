#ifndef _SYS_STAT_H
#define _SYS_STAT_H
#include <sys/types.h>
struct stat { off_t st_size; mode_t st_mode; };
int stat(const char *path, struct stat *buf);
int fstat(int fd, struct stat *buf);
int mkdir(const char *path, mode_t mode);
#define S_ISDIR(m) (((m) & 0170000) == 0040000)
#define S_ISREG(m) (((m) & 0170000) == 0100000)
#endif
