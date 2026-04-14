#ifndef _DIRENT_H
#define _DIRENT_H
struct dirent { unsigned long d_ino; char d_name[256]; unsigned char d_type; };
typedef struct { int fd; } DIR;
DIR *opendir(const char *name);
struct dirent *readdir(DIR *dir);
int closedir(DIR *dir);
#define DT_DIR 4
#define DT_REG 8
#endif
