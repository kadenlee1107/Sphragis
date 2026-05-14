/*
 * evil_path_traversal.c — guest-side harness that exercises VFS path
 * handling in src/caves/linux/vfs.rs and sys_openat (ATTACK-FL-016,
 * FL-017, FL-018).
 *
 * This is a Linux userspace program intended to be compiled for AArch64
 * and run under a Sphragis cave (via busybox or direct ELF loader).
 *
 *   aarch64-linux-gnu-gcc -static -Os -o evil_path_traversal \
 *       evil_path_traversal.c
 *
 * It does not escape the host. It is a probe for Cave VFS behavior.
 *
 * Tests:
 *   1. ../../.. escape (should fail, root parent is root).
 *   2. 200-byte path (truncation to 128 bytes).
 *   3. Symlink with absolute target (/tmp/link → /etc/passwd).
 *   4. Symlink cycle at depth 9 (ELOOP expected).
 *   5. O_CREAT on 200-byte path (creates file in WRONG parent if truncated).
 */

#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

static void test_traversal(void) {
    int fd = open("../../../../../../../../etc/passwd", O_RDONLY);
    printf("[1] open ../../etc/passwd -> fd=%d errno=%d\n", fd, errno);
    if (fd >= 0) close(fd);
}

static void test_long_path(void) {
    char path[300];
    memset(path, 'A', sizeof(path));
    strcpy(path, "/tmp/");
    memset(path + 5, 'A', 250);
    path[255] = 0;
    /* 250-byte path; read_user_str caps at 128. Everything after byte 128
     * is silently truncated. */
    int fd = open(path, O_CREAT | O_WRONLY, 0644);
    printf("[2] open 250-byte path -> fd=%d errno=%d (expect truncation)\n",
           fd, errno);
    if (fd >= 0) close(fd);
}

static void test_symlink_escape(void) {
    /* If symlinkat is exposed: /tmp/escape -> /etc/passwd. */
    int r = symlink("/etc/passwd", "/tmp/escape");
    printf("[3] symlink /tmp/escape -> /etc/passwd: r=%d errno=%d\n", r, errno);
    int fd = open("/tmp/escape", O_RDONLY);
    if (fd >= 0) {
        char buf[64];
        ssize_t n = read(fd, buf, sizeof(buf)-1);
        if (n > 0) { buf[n] = 0; printf("    read %zd bytes: %s\n", n, buf); }
        close(fd);
    }
}

static void test_symlink_cycle(void) {
    symlink("/tmp/b", "/tmp/a");
    symlink("/tmp/a", "/tmp/b");
    int fd = open("/tmp/a", O_RDONLY);
    printf("[4] cyclic symlink -> fd=%d errno=%d (expect ELOOP=40)\n",
           fd, errno);
    if (fd >= 0) close(fd);
}

static void test_creat_wrong_parent(void) {
    /* If a 200-byte path's NUL terminator falls after byte 128, the
     * resolved parent directory will be (wrongly) the truncated prefix,
     * not the intended one. O_CREAT will drop the new file in an
     * unexpected location. */
    char path[220];
    memset(path, 0, sizeof(path));
    strcpy(path, "/tmp/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                 "/sneaky_file");
    int fd = open(path, O_CREAT | O_WRONLY, 0644);
    printf("[5] O_CREAT long path -> fd=%d errno=%d\n", fd, errno);
    if (fd >= 0) close(fd);
}

int main(void) {
    test_traversal();
    test_long_path();
    test_symlink_escape();
    test_symlink_cycle();
    test_creat_wrong_parent();
    return 0;
}
