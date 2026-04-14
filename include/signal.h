/*
 * Bat_OS — signal.h stub for NetSurf
 * Minimal POSIX signal declarations.
 */
#ifndef _BATOS_SIGNAL_H
#define _BATOS_SIGNAL_H

/* Signal numbers (Linux AArch64 values) */
#define SIGHUP     1
#define SIGINT     2
#define SIGQUIT    3
#define SIGILL     4
#define SIGTRAP    5
#define SIGABRT    6
#define SIGIOT     SIGABRT
#define SIGBUS     7
#define SIGFPE     8
#define SIGKILL    9
#define SIGUSR1   10
#define SIGSEGV   11
#define SIGUSR2   12
#define SIGPIPE   13
#define SIGALRM   14
#define SIGTERM   15
#define SIGCHLD   17
#define SIGCONT   18
#define SIGSTOP   19
#define SIGTSTP   20
#define SIGTTIN   21
#define SIGTTOU   22
#define SIGURG    23
#define SIGXCPU   24
#define SIGXFSZ   25
#define SIGVTALRM 26
#define SIGPROF   27
#define SIGWINCH  28
#define SIGIO     29
#define SIGPOLL   SIGIO
#define SIGPWR    30
#define SIGSYS    31

#define _NSIG     64

/* Handler dispositions */
#define SIG_DFL ((void (*)(int))0)
#define SIG_IGN ((void (*)(int))1)
#define SIG_ERR ((void (*)(int))-1)

/* Minimal sigset_t */
typedef struct {
    unsigned long __val[_NSIG / (8 * sizeof(unsigned long))];
} sigset_t;

/* Signal action (SA_*) flags */
#define SA_NOCLDSTOP  1
#define SA_NOCLDWAIT  2
#define SA_SIGINFO    4
#define SA_RESTART   0x10000000
#define SA_NODEFER   0x40000000
#define SA_RESETHAND 0x80000000

typedef union sigval {
    int   sival_int;
    void *sival_ptr;
} sigval_t;

typedef struct {
    int      si_signo;
    int      si_errno;
    int      si_code;
    int      si_pid;
    int      si_uid;
    void    *si_addr;
    int      si_status;
    sigval_t si_value;
} siginfo_t;

struct sigaction {
    union {
        void (*sa_handler)(int);
        void (*sa_sigaction)(int, siginfo_t *, void *);
    };
    sigset_t sa_mask;
    int      sa_flags;
    void   (*sa_restorer)(void);
};

/* Core functions */
void (*signal(int sig, void (*handler)(int)))(int);
int    sigaction(int sig, const struct sigaction *act,
                 struct sigaction *oact);
int    kill(int pid, int sig);
int    raise(int sig);

/* Signal set manipulation */
int    sigemptyset(sigset_t *set);
int    sigfillset(sigset_t *set);
int    sigaddset(sigset_t *set, int signum);
int    sigdelset(sigset_t *set, int signum);
int    sigismember(const sigset_t *set, int signum);

/* Signal mask */
#define SIG_BLOCK   0
#define SIG_UNBLOCK 1
#define SIG_SETMASK 2

int    sigprocmask(int how, const sigset_t *set, sigset_t *oldset);

#endif /* _BATOS_SIGNAL_H */
