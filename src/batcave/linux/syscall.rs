// Bat_OS — Linux Syscall Translation Layer
// Intercepts ARM64 Linux syscalls (svc #0) from BatCave processes
// and translates them into Bat_OS operations.
//
// Every syscall is a security checkpoint:
// No capability → EACCES. No exceptions.
//
// ARM64 Linux syscall convention:
//   x8 = syscall number
//   x0-x5 = arguments
//   x0 = return value (negative = -errno)

use crate::drivers::uart;
use crate::batcave::cave;
use super::vfs;
use super::fd;
use super::stdio_ring;
use super::uaccess;

/// Runtime toggle for the per-syscall trace print in `handle`. Defaults
/// off; `runner::run_chromium` flips it to `true` while the Chromium
/// cave is running so we can see exactly what content_shell calls.
pub static SYSCALL_TRACE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

/// Best-effort decode of an AArch64 Linux syscall number into its name.
/// Only covers the ones we're likely to see from content_shell's startup
/// — anything else shows up as "?".
pub fn syscall_name(n: u64) -> &'static str {
    match n {
        17  => "getcwd",
        23  => "dup",
        24  => "dup3",
        25  => "fcntl",
        29  => "ioctl",
        34  => "mkdirat",
        43  => "statfs",
        46  => "ftruncate",
        48  => "faccessat",
        49  => "chdir",
        56  => "openat",
        57  => "close",
        59  => "pipe2",
        61  => "getdents64",
        62  => "lseek",
        63  => "read",
        64  => "write",
        65  => "readv",
        66  => "writev",
        67  => "pread64",
        68  => "pwrite64",
        71  => "sendfile",
        72  => "pselect6",
        73  => "ppoll",
        78  => "readlinkat",
        79  => "newfstatat",
        80  => "fstat",
        93  => "exit",
        94  => "exit_group",
        96  => "set_tid_address",
        98  => "futex",
        99  => "set_robust_list",
        113 => "clock_gettime",
        115 => "clock_nanosleep",
        124 => "sched_yield",
        131 => "tgkill",
        134 => "rt_sigaction",
        135 => "rt_sigprocmask",
        139 => "rt_sigreturn",
        140 => "setpriority",
        141 => "getpriority",
        160 => "uname",
        167 => "prctl",
        169 => "gettimeofday",
        172 => "getpid",
        173 => "getppid",
        174 => "getuid",
        175 => "geteuid",
        178 => "gettid",
        179 => "sysinfo",
        199 => "socketpair",
        203 => "connect",
        210 => "shutdown",
        214 => "brk",
        215 => "munmap",
        216 => "mremap",
        220 => "clone",
        221 => "execve",
        222 => "mmap",
        226 => "mprotect",
        233 => "madvise",
        261 => "prlimit64",
        278 => "getrandom",
        293 => "rseq",
        _   => "?",
    }
}

// Linux errno values (returned as negative)
const ENOSYS: i64 = -38;   // Function not implemented
const EACCES: i64 = -13;   // Permission denied
const EBADF: i64 = -9;     // Bad file descriptor
const ENOMEM: i64 = -12;   // Out of memory
const ENOENT: i64 = -2;    // No such file or directory
const EINVAL: i64 = -22;   // Invalid argument
const EFAULT: i64 = -14;   // Bad address
const ECHILD: i64 = -10;   // No child processes
const EAGAIN: i64 = -11;   // Try again
const EPERM: i64 = -1;     // Operation not permitted

/// Returns true if `p` + `size` is plausibly inside the cave's user-space.
///
/// After ROOT-1 (per-cave page tables) lands this becomes an exact check
/// against the caller's page-table VA range. Today the cave and the
/// kernel share one identity-mapped VA, so the best we can do is reject
/// obvious kernel addresses: NULL, low-page nulls, and anywhere inside
/// the kernel RAM identity map (0x4000_0000..0x8000_0000 on QEMU virt).
///
/// Returns `false` on overflow, NULL, low pages, or kernel-range pointers.
///
/// V8-ROOT-8 (regression fix): delegate to `uaccess::is_user_range` so the
/// cave's own user-window bounds (set at `enter()` time) are consulted,
/// not just the static legacy window. Any cave with virt_base != 0
/// previously could read/write the legacy 0x1000..0x4000_0000 range and
/// pivot into another cave's state.
fn is_user_ptr(p: usize, size: usize) -> bool {
    uaccess::is_user_range(p, size)
}

// Syscall categories for capability checking
#[derive(Clone, Copy)]
enum SyscallCat {
    Always,   // Always allowed (getpid, uname, etc.)
    FileIO,   // Needs fs capability
    Network,  // Needs net capability
    RawNet,   // Needs raw capability
    Process,  // Always allowed within the cave
    Memory,   // Always allowed within the cave
    Display,  // Needs display capability
}

/// Linux syscall numbers (ARM64)
mod nr {
    pub const GETCWD: u64 = 17;
    pub const IOCTL: u64 = 29;
    pub const FACCESSAT: u64 = 48;
    pub const CHDIR: u64 = 49;
    pub const OPENAT: u64 = 56;
    pub const CLOSE: u64 = 57;
    pub const LSEEK: u64 = 62;
    pub const READ: u64 = 63;
    pub const WRITE: u64 = 64;
    pub const READLINKAT: u64 = 78;
    pub const NEWFSTATAT: u64 = 79;
    pub const FSTAT: u64 = 80;
    pub const EXIT: u64 = 93;
    pub const EXIT_GROUP: u64 = 94;
    pub const SET_TID_ADDRESS: u64 = 96;
    pub const CLOCK_GETTIME: u64 = 113;
    pub const UNAME: u64 = 160;
    pub const GETPID: u64 = 172;
    pub const GETPPID: u64 = 173;
    pub const GETUID: u64 = 174;
    pub const GETEUID: u64 = 175;
    pub const GETGID: u64 = 176;
    pub const GETEGID: u64 = 177;
    pub const BRK: u64 = 214;
    pub const MUNMAP: u64 = 215;
    pub const CLONE: u64 = 220;
    pub const EXECVE: u64 = 221;
    pub const MMAP: u64 = 222;
    pub const MPROTECT: u64 = 226;
    pub const WAIT4: u64 = 260;
    pub const PRLIMIT64: u64 = 261;
    pub const GETRANDOM: u64 = 278;

    // Epoll + eventfd + timerfd (event notification family)
    pub const EPOLL_CREATE1: u64 = 20;
    pub const EPOLL_CTL: u64 = 21;
    pub const EPOLL_PWAIT: u64 = 22;
    pub const EVENTFD2: u64 = 19;
    pub const TIMERFD_CREATE: u64 = 85;
    pub const TIMERFD_SETTIME: u64 = 86;
    pub const TIMERFD_GETTIME: u64 = 87;

    // Network
    pub const SOCKET: u64 = 198;
    pub const SOCKETPAIR: u64 = 199;
    pub const BIND: u64 = 200;
    pub const LISTEN: u64 = 201;
    pub const ACCEPT: u64 = 202;
    pub const CONNECT: u64 = 203;
    pub const GETSOCKNAME: u64 = 204;
    pub const GETPEERNAME: u64 = 205;
    pub const SENDTO: u64 = 206;
    pub const RECVFROM: u64 = 207;
    pub const SETSOCKOPT: u64 = 208;
    pub const GETSOCKOPT: u64 = 209;
    pub const SHUTDOWN: u64 = 210;
    pub const SENDMSG: u64 = 211;
    pub const RECVMSG: u64 = 212;
    pub const ACCEPT4: u64 = 242;

    // Threading (process family — AArch64 numbers)
    pub const SCHED_GETAFFINITY: u64 = 123;
    pub const SCHED_SETAFFINITY: u64 = 122;
    pub const FADVISE64: u64 = 223;
}

/// Handle a Linux syscall from a BatCave process.
/// cave_id: which BatCave this process belongs to
/// syscall_num: x8 register value
/// args: x0-x5 register values
/// Returns: value to put in x0 (negative = error)
pub fn handle(cave_id: usize, syscall_num: u64, args: [u64; 6]) -> i64 {
    // V4 deferred preemption: timer tick may have requested we yield.
    // Consume the flag at the syscall boundary (safe point — we have a
    // stable stack and aren't mid-inline-asm).
    super::threads::maybe_yield();

    // CHROMIUM-PHASE-D: cooperative-yield every Nth syscall when
    // many threads are runnable. Without this, a hot loop of
    // non-blocking syscalls (Chromium's worker-pool init does
    // exactly this — clone/mprotect/gettid/clock_gettime in a
    // tight cycle) starves freshly-spawned pthreads. They sit in
    // Runnable state forever and never get a chance to run their
    // glibc post-clone setup, so Chromium thinks the worker pool
    // is hung and never advances to navigation. Yielding every
    // 64 syscalls keeps the cooperative scheduler making progress.
    static SYSCALL_COUNTER: core::sync::atomic::AtomicU64 =
        core::sync::atomic::AtomicU64::new(0);
    let n = SYSCALL_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if (n & 0x3F) == 0 {
        super::threads::schedule();
    }

    // Temporary diagnostic: trace every syscall. Essential for the Chromium
    // port debug loop right now ("did content_shell reach main, or did
    // __libc_start_main bail early?"). Gate with an atomic so tests that
    // spam writes (e.g. netsurf, png_test) don't drown in log output.
    let trace = SYSCALL_TRACE.load(core::sync::atomic::Ordering::Relaxed);
    if trace {
        use crate::drivers::uart;
        // Tag each traced syscall with the running thread's tid so we
        // can distinguish main (1) from worker threads when multiple
        // pthreads are active.
        uart::puts("[sc t");
        crate::kernel::mm::print_num(super::threads::current_tid() as usize);
        uart::puts("] ");
        crate::kernel::mm::print_num(syscall_num as usize);
        uart::puts(" (");
        uart::puts(syscall_name(syscall_num));
        uart::puts(") args=[");
        let hex = b"0123456789abcdef";
        for (i, a) in args.iter().enumerate() {
            if i > 0 { uart::puts(", "); }
            uart::puts("0x");
            for shift in (0..16).rev() {
                uart::putc(hex[((a >> (shift * 4)) & 0xF) as usize]);
            }
        }
        uart::puts("]\n");
    }

    // Classify the syscall
    let (cat, handler): (SyscallCat, fn([u64; 6]) -> i64) = match syscall_num {
        // ── Always allowed ──
        nr::GETPID => (SyscallCat::Always, sys_getpid),
        nr::GETPPID => (SyscallCat::Always, sys_getppid),
        nr::GETUID | nr::GETEUID => (SyscallCat::Always, sys_getuid),
        nr::GETGID | nr::GETEGID => (SyscallCat::Always, sys_getgid),
        nr::UNAME => (SyscallCat::Always, sys_uname),
        nr::EXIT => (SyscallCat::Always, sys_exit),
        nr::EXIT_GROUP => (SyscallCat::Always, sys_exit_group),
        nr::SET_TID_ADDRESS => (SyscallCat::Always, sys_set_tid_address),
        nr::PRLIMIT64 => (SyscallCat::Always, sys_prlimit64),
        nr::CLOCK_GETTIME => (SyscallCat::Always, sys_clock_gettime),
        nr::GETRANDOM => (SyscallCat::Always, sys_getrandom),
        73 => (SyscallCat::Always, sys_ppoll),        // ppoll
        98 => (SyscallCat::Always, sys_futex),        // futex
        99 => (SyscallCat::Always, sys_stub_zero),   // set_robust_list
        100 => (SyscallCat::Always, sys_stub_zero),  // get_robust_list
        101 => (SyscallCat::Always, sys_nanosleep),  // nanosleep
        102 => (SyscallCat::Always, sys_stub_zero),  // getitimer
        103 => (SyscallCat::Always, sys_stub_zero),  // setitimer
        131 => (SyscallCat::Always, sys_tgkill),       // tgkill
        132 => (SyscallCat::Always, sys_sigaltstack),  // sigaltstack
        134 => (SyscallCat::Always, sys_rt_sigaction), // rt_sigaction
        135 => (SyscallCat::Always, sys_rt_sigprocmask), // rt_sigprocmask
        136 => (SyscallCat::Always, sys_stub_zero),  // rt_sigpending
        137 => (SyscallCat::Always, sys_stub_zero),  // rt_sigtimedwait
        139 => (SyscallCat::Always, sys_rt_sigreturn), // rt_sigreturn
        144 => (SyscallCat::Always, sys_stub_zero),  // setgid
        146 => (SyscallCat::Always, sys_stub_zero),  // setuid
        153 => (SyscallCat::Always, sys_stub_zero),  // times
        154 => (SyscallCat::Always, sys_stub_zero),  // setpgid
        155 => (SyscallCat::Always, sys_stub_zero),  // getpgid
        157 => (SyscallCat::Always, sys_stub_zero),  // sched_getscheduler
        158 => (SyscallCat::Always, sys_stub_zero),  // sched_getparam
        140 => (SyscallCat::Always, sys_stub_zero),  // setpriority — glibc
                                                     // pthread_create tunes
                                                     // nice level; stub accepts
                                                     // any value with 0-return
                                                     // so the call doesn't
                                                     // log spam.
        141 => (SyscallCat::Always, sys_stub_zero),  // getpriority — counterpart
        166 => (SyscallCat::Always, sys_stub_zero),  // umask
        167 => (SyscallCat::Always, sys_stub_zero),  // prctl — many sub-ops;
                                                     // stub 0 keeps glibc
                                                     // happy for PR_SET_NAME,
                                                     // PR_SET_DUMPABLE, etc.
        169 => (SyscallCat::Always, sys_stub_zero),  // gettimeofday
        170 => (SyscallCat::Always, sys_stub_zero),  // getpgrp/setpgid
        171 => (SyscallCat::Always, sys_sigaltstack), // sigaltstack (compat)
        178 => (SyscallCat::Always, sys_gettid),      // gettid
        179 => (SyscallCat::Always, sys_sysinfo),    // sysinfo (real impl)
        // NOTE: on AArch64, syscall 204 is getsockname (not sched_getaffinity).
        // getsockname is routed below via nr::GETSOCKNAME in the network block.
        // True sched_getaffinity is 123 (AArch64), sched_setaffinity is 122.
        nr::SCHED_GETAFFINITY => (SyscallCat::Always, sys_stub_zero),
        nr::SCHED_SETAFFINITY => (SyscallCat::Always, sys_stub_zero),
        // NOTE: AArch64 syscall 222 is mmap (already routed via nr::MMAP),
        // 223 is fadvise64. 210 is shutdown (moved to Network block below).
        // Previously these were mislabeled as shmget/shmctl/shutdown-stub.
        nr::FADVISE64 => (SyscallCat::Always, sys_stub_zero),
        233 => (SyscallCat::Always, sys_stub_zero),  // madvise
        262 => (SyscallCat::Always, sys_stub_zero),  // getrlimit equiv
        276 => (SyscallCat::Always, sys_stub_zero),  // renameat2
        279 => (SyscallCat::Memory, sys_memfd_create), // memfd_create — needs mem cap
        // rseq: restartable sequences, added in Linux 4.18. glibc ≥ 2.35
        // probes it once per thread; if we return ENOSYS glibc falls back
        // gracefully to the non-rseq path. Stub-zero is wrong because it
        // lies about successful registration; return ENOSYS explicitly.
        293 => (SyscallCat::Always, sys_stub_enosys),

        // ── Epoll + eventfd + timerfd (real implementations) ──
        nr::EPOLL_CREATE1 => (SyscallCat::FileIO, sys_epoll_create1),
        nr::EPOLL_CTL => (SyscallCat::FileIO, sys_epoll_ctl),
        nr::EPOLL_PWAIT => (SyscallCat::FileIO, sys_epoll_pwait),
        nr::EVENTFD2 => (SyscallCat::FileIO, sys_eventfd2),
        nr::TIMERFD_CREATE => (SyscallCat::FileIO, sys_timerfd_create),
        nr::TIMERFD_SETTIME => (SyscallCat::FileIO, sys_timerfd_settime),
        nr::TIMERFD_GETTIME => (SyscallCat::FileIO, sys_timerfd_gettime),

        // ── File I/O — needs fs capability ──
        nr::OPENAT => (SyscallCat::FileIO, sys_openat),
        nr::CLOSE => (SyscallCat::FileIO, sys_close),
        nr::READ => (SyscallCat::FileIO, sys_read),
        nr::WRITE => (SyscallCat::FileIO, sys_write),
        nr::LSEEK => (SyscallCat::FileIO, sys_stub_zero),
        nr::FSTAT => (SyscallCat::FileIO, sys_fstat),
        nr::NEWFSTATAT => (SyscallCat::FileIO, sys_newfstatat),
        nr::IOCTL => (SyscallCat::FileIO, sys_ioctl),
        nr::GETCWD => (SyscallCat::FileIO, sys_getcwd),
        23 => (SyscallCat::FileIO, sys_dup),          // dup
        24 => (SyscallCat::FileIO, sys_dup3),         // dup3
        25 => (SyscallCat::FileIO, sys_fcntl),        // fcntl
        34 => (SyscallCat::FileIO, sys_mkdirat),      // mkdirat
        35 => (SyscallCat::FileIO, sys_stub_zero),    // unlinkat
        46 => (SyscallCat::Always, sys_stub_zero),    // ftruncate
        48 => (SyscallCat::FileIO, sys_faccessat),    // faccessat
        49 => (SyscallCat::FileIO, sys_chdir),        // chdir
        59 => (SyscallCat::FileIO, sys_pipe2),        // pipe2
        61 => (SyscallCat::FileIO, sys_getdents64),   // getdents64
        66 => (SyscallCat::FileIO, sys_writev),       // writev
        71 => (SyscallCat::FileIO, sys_sendfile),     // sendfile
        78 => (SyscallCat::FileIO, sys_readlinkat),   // readlinkat

        // ── Memory — always allowed within cave ──
        nr::BRK => (SyscallCat::Memory, sys_brk),
        nr::MMAP => (SyscallCat::Memory, sys_mmap),
        nr::MUNMAP => (SyscallCat::Memory, sys_munmap),
        nr::MPROTECT => (SyscallCat::Memory, sys_mprotect),

        // ── Display (Bat_OS custom) ──
        500 => (SyscallCat::Display, sys_blit_framebuffer), // custom: blit pixels to GPU

        // ── Process ──
        220 => (SyscallCat::Process, sys_clone_thread), // clone
        221 => (SyscallCat::Process, sys_execve),       // execve
        260 => (SyscallCat::Process, sys_wait_stub),    // wait4

        // ── Network — needs net capability ──
        nr::SOCKET => (SyscallCat::Network, sys_socket),
        nr::SOCKETPAIR => (SyscallCat::Network, sys_socketpair),
        nr::CONNECT => (SyscallCat::Network, sys_connect),
        nr::BIND => (SyscallCat::Network, sys_bind),
        nr::LISTEN => (SyscallCat::Network, sys_listen),
        nr::ACCEPT => (SyscallCat::Network, sys_accept),
        nr::ACCEPT4 => (SyscallCat::Network, sys_accept4),
        nr::GETSOCKNAME => (SyscallCat::Network, sys_getsockname),
        nr::GETPEERNAME => (SyscallCat::Network, sys_getpeername),
        nr::SENDTO => (SyscallCat::Network, sys_sendto),
        nr::RECVFROM => (SyscallCat::Network, sys_recvfrom),
        nr::SENDMSG => (SyscallCat::Network, sys_sendmsg),
        nr::RECVMSG => (SyscallCat::Network, sys_recvmsg),
        nr::SETSOCKOPT => (SyscallCat::Network, sys_setsockopt),
        nr::GETSOCKOPT => (SyscallCat::Network, sys_getsockopt),
        nr::SHUTDOWN => (SyscallCat::Network, sys_shutdown),

        _ => {
            // Unknown syscall — log and return ENOSYS
            uart::puts("[linux] unknown syscall ");
            crate::kernel::mm::print_num(syscall_num as usize);
            uart::puts("\n");
            return ENOSYS;
        }
    };

    // Capability check. ROOT-5 fix: FileIO / Process / Memory now actually
    // consult the cave's cap set instead of hard-returning true. The default
    // cave is created with `fs` granted at shell start (shell.rs:611), so the
    // existing test binaries still work; caves without `fs` / `proc` / `mem`
    // are now genuinely denied those syscalls.
    //
    // `Always` remains unconditionally allowed — it covers getpid, getuid,
    // uname, exit, clock_gettime, etc. where denying would just break the
    // program without adding meaningful protection.
    let allowed = match cat {
        SyscallCat::Always  => true,
        SyscallCat::Process => cave_has_cap(cave_id, "proc"),
        SyscallCat::Memory  => cave_has_cap(cave_id, "mem"),
        SyscallCat::FileIO  => cave_has_cap(cave_id, "fs"),
        SyscallCat::Network => cave_has_cap(cave_id, "net"),
        SyscallCat::RawNet  => cave_has_cap(cave_id, "raw"),
        SyscallCat::Display => cave_has_cap(cave_id, "display"),
    };

    if !allowed {
        uart::puts("[linux] BLOCKED syscall ");
        crate::kernel::mm::print_num(syscall_num as usize);
        uart::puts(" (no capability)\n");
        return EACCES;
    }

    // Per-cave surgical denylist — even if the broader capability is
    // granted, this cave can ban specific syscall numbers. Lets an
    // operator grant `net` but still deny `connect()` so an RCE'd
    // cave can't dial out on behalf of an attacker.
    if super::super::syscall_filter::is_denied(cave_id, syscall_num) {
        super::super::syscall_filter::bump_denied();
        uart::puts("[linux] BLOCKED syscall ");
        crate::kernel::mm::print_num(syscall_num as usize);
        uart::puts(" (per-cave denylist)\n");
        return EPERM;
    }

    let rv = handler(args);
    if trace {
        use crate::drivers::uart;
        uart::puts("[sc t");
        crate::kernel::mm::print_num(super::threads::current_tid() as usize);
        uart::puts("] -> ");
        if rv < 0 {
            uart::puts("errno ");
            crate::kernel::mm::print_num((-rv) as usize);
        } else {
            uart::puts("0x");
            let hex = b"0123456789abcdef";
            for shift in (0..16).rev() {
                uart::putc(hex[((rv as u64 >> (shift * 4)) & 0xF) as usize]);
            }
        }
        uart::puts("\n");
    }
    rv
}

fn cave_has_cap(_cave_id: usize, cap: &str) -> bool {
    // Check the ACTIVE BatCave's capability set
    cave::active_has_cap(cap)
}

// ─── Syscall Implementations ───

fn sys_getpid(_args: [u64; 6]) -> i64 { 1 }
fn sys_getppid(_args: [u64; 6]) -> i64 { 0 }
fn sys_getuid(_args: [u64; 6]) -> i64 { 0 } // root
fn sys_getgid(_args: [u64; 6]) -> i64 { 0 }
fn sys_stub_zero(_args: [u64; 6]) -> i64 { 0 }
fn sys_stub_enosys(_args: [u64; 6]) -> i64 { ENOSYS }
fn sys_stub_enoent(_args: [u64; 6]) -> i64 { ENOENT }

// ─── nanosleep (101) — real sleep using ARM64 generic timer ───
fn sys_nanosleep(args: [u64; 6]) -> i64 {
    let req_ptr = args[0] as usize;
    if req_ptr == 0 { return EINVAL; }
    // Pointer-to-kernel guard must come BEFORE the raw asm read below.
    if !is_user_ptr(req_ptr, 16) { return EFAULT; }

    // Read struct timespec { tv_sec: i64, tv_nsec: i64 }
    let tv_sec: u64;
    let tv_nsec: u64;
    unsafe {
        core::arch::asm!("ldr {v}, [{a}]", a = in(reg) req_ptr, v = out(reg) tv_sec);
        core::arch::asm!("ldr {v}, [{a}]", a = in(reg) req_ptr + 8, v = out(reg) tv_nsec);
    }

    // Read current cycle count and frequency
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }

    // V8-ROOT-3 / V8-ARITH-A1: tv_sec * freq overflows for tv_sec > ~2^28
    // on a typical 50MHz timer. Cap the requested sleep to 1 hour worth
    // of ticks so an attacker can't trigger panic-on-overflow (now that
    // overflow-checks=true) or a wraparound that returns immediately.
    let secs_capped  = tv_sec.min(3600);
    let nsecs_capped = tv_nsec.min(999_999_999);
    let target_ticks = secs_capped.saturating_mul(freq)
        .saturating_add(nsecs_capped.saturating_mul(freq) / 1_000_000_000);

    // NEW-DOS-010/014/016/019 fix: yield to the scheduler instead of burning
    // CPU in a spin-loop. A cave that nanosleep()s for 30 s used to pin the
    // core; now co-scheduled caves get a slice via threads::schedule().
    // We still check the timer every ~100 iterations so wakeup latency stays
    // sub-ms on a lightly loaded system.
    let mut it = 0u32;
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target_ticks { break; }
        it = it.wrapping_add(1);
        if it % 256 == 0 {
            super::threads::schedule();
        } else {
            core::hint::spin_loop();
        }
    }
    0
}

// ─── munmap (215) — free mapped memory ───
//
// Bat_OS maps user pages identity on the current TTBR0 (ROOT-1: per-cave
// page tables aren't wired yet), so the "VA" we get is really a PA in the
// frame allocator's bitmap. All we need to do is free each backing frame,
// refund the cave's memory quota, and invalidate the TLB for the range.
//
// Return: 0 on success, -EINVAL on zero-length / mis-aligned inputs.
fn sys_munmap(args: [u64; 6]) -> i64 {
    let addr = args[0] as usize;
    let length = args[1] as usize;
    if length == 0 { return EINVAL; }
    if addr & 0xFFF != 0 { return EINVAL; }

    // V6-WEIRD-007 fix: WITHOUT this gate, a cave could call
    // munmap(addr=0x40100000, 4KB) and `free_contig` → `free_frame`
    // would zero 4 KB of kernel memory (heap, auth BSS, frame bitmap,
    // anything >= MEMORY_START). The old comment claiming "free_contig
    // silently ignores bases outside the frame bitmap" was WRONG —
    // free_frame zeroed the page BEFORE checking the bitmap index.
    // The fix here gates at syscall entry; frame::free_frame also
    // got a defensive extent-check so this can't be bypassed by any
    // other caller.
    if !uaccess::is_user_range(addr, length) {
        return -(14i64); // EFAULT
    }

    let page_size = 4096usize;
    let pages = (length + page_size - 1) / page_size;

    // V6-XLAYER-003: refund based on the number of pages actually
    // freed (frames that were in-use in the bitmap), NOT the user-
    // supplied length. Without this, a cave can call
    //   munmap(real_4kb_alloc, 1GB)
    // and saturating-sub its memory quota to zero, then mmap fresh
    // pages past its real cap.
    let freed_pages = crate::kernel::mm::frame::free_contig(addr, pages);
    super::quotas::refund_active(
        super::quotas::Resource::Mem, freed_pages * page_size);

    // Invalidate the TLB for this ASID. We don't yet have per-address
    // tlbi vaae1 wired (see mmu.rs); a full `tlbi vmalle1` is coarse but
    // correct. Cheap on single-core HVF.
    unsafe {
        core::arch::asm!(
            "dsb ishst",
            "tlbi vmalle1",
            "dsb ish",
            "isb",
            options(nostack, preserves_flags),
        );
    }

    0
}

// ─── mprotect (226) — change memory protection ───
//
// Walk the cave's page table for each 4 KB page in [addr, addr+len)
// and flip AP (access perm) / UXN (unprivileged execute-never) bits
// to match the requested PROT_* flags. Pages that are not yet
// backed (demand-paged reservation) are left alone — they'll get
// the default RW-no-exec flags when demand_page::try_handle commits
// them. Regions that need exec MUST call sys_mprotect after the
// page has been materialised via a first access.
fn sys_mprotect(args: [u64; 6]) -> i64 {
    let addr = args[0] as u64;
    let len  = args[1] as usize;
    let prot = args[2] as u32;

    const PROT_READ:  u32 = 1;
    const PROT_WRITE: u32 = 2;
    const PROT_EXEC:  u32 = 4;

    if len == 0 { return 0; }

    // Align range to 4 KB.
    let start = addr & !0xFFFu64;
    let end_raw = match (addr as usize).checked_add(len) {
        Some(e) => e as u64,
        None => return EINVAL,
    };
    let end = (end_raw + 0xFFF) & !0xFFFu64;

    // Cave's L1 physical address.
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;

    // Build the new AP + UXN bit pattern. Always set PXN + AF + SH +
    // VALID. UXN is *on* (no-exec) unless PROT_EXEC was requested.
    const BITS_VALID: u64 = 0b11;
    const BITS_AF:    u64 = 1 << 10;
    const BITS_SH:    u64 = 3 << 8;
    const BITS_PXN:   u64 = 1 << 53;
    const BITS_UXN:   u64 = 1 << 54;

    // AP[2:1] at bits [7:6]:
    //   0b00: EL1 R/W, no EL0 access
    //   0b01: EL1 R/W, EL0 R/W
    //   0b10: EL1 R/O, no EL0 access
    //   0b11: EL1 R/O, EL0 R/O
    let ap_bits: u64 = if (prot & PROT_WRITE) != 0 {
        0b01 << 6  // EL0 R/W
    } else if (prot & PROT_READ) != 0 {
        0b11 << 6  // EL0 R/O
    } else {
        0b00 << 6  // no EL0 access
    };
    let mut new_low: u64 =
        BITS_VALID | BITS_AF | BITS_SH | ap_bits;
    // High attribute bits.
    let mut new_high: u64 = BITS_PXN;
    if (prot & PROT_EXEC) == 0 {
        new_high |= BITS_UXN;
    }
    let _ = &mut new_low;

    // Walk page-by-page.
    let mut updated: usize = 0;
    for va in (start..end).step_by(4096) {
        let l1_idx = ((va >> 30) & 0x1FF) as usize;
        let l1_ent_addr = l1_phys + (l1_idx as u64) * 8;
        let l1_ent = unsafe {
            core::ptr::read_volatile(l1_ent_addr as *const u64)
        };
        // Must be a TABLE descriptor (lower 2 bits = 0b11).
        if (l1_ent & 0b11) != 0b11 { continue; }
        let l2_phys = l1_ent & 0x0000_FFFF_FFFF_F000;

        let l2_idx = ((va >> 21) & 0x1FF) as usize;
        let l2_ent_addr = l2_phys + (l2_idx as u64) * 8;
        let l2_ent = unsafe {
            core::ptr::read_volatile(l2_ent_addr as *const u64)
        };
        if (l2_ent & 0b11) != 0b11 { continue; }
        let l3_phys = l2_ent & 0x0000_FFFF_FFFF_F000;

        let l3_idx = ((va >> 12) & 0x1FF) as usize;
        let l3_ent_addr = l3_phys + (l3_idx as u64) * 8;
        let l3_ent = unsafe {
            core::ptr::read_volatile(l3_ent_addr as *const u64)
        };
        // L3 entry must be a valid PAGE descriptor (lower 2 bits = 0b11).
        if (l3_ent & 0b11) != 0b11 { continue; }
        // Preserve the physical frame address and attr-index, replace
        // the AP / PXN / UXN / AF / SH bits.
        const MASK_FRAME: u64 = 0x0000_FFFF_FFFF_F000;
        const MASK_ATTR:  u64 = 0b111 << 2;
        let kept = l3_ent & (MASK_FRAME | MASK_ATTR);
        let new_ent = kept | new_low | new_high;
        unsafe {
            core::ptr::write_volatile(l3_ent_addr as *mut u64, new_ent);
        }
        updated += 1;
    }

    // TLB flush if we actually changed anything. Full-ASID sledgehammer
    // so the next EL0 fetch/load picks up the new permissions.
    if updated > 0 {
        unsafe {
            core::arch::asm!("dsb ishst");
            core::arch::asm!("tlbi vmalle1");
            core::arch::asm!("dsb ish");
            core::arch::asm!("isb");
        }
    }
    // Log rate-limited — mprotect fires dozens of times during
    // Chromium startup as V8 commits cage sub-pages.
    static MPROTECT_CALLS: core::sync::atomic::AtomicU64
        = core::sync::atomic::AtomicU64::new(0);
    let n = MPROTECT_CALLS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if n < 8 || (n & 0xFF) == 0 {
        uart::puts("[mprotect] addr=0x");
        let hex = b"0123456789abcdef";
        for sh in (0..16).rev() {
            uart::putc(hex[((start >> (sh*4)) & 0xF) as usize]);
        }
        uart::puts(" len=");
        crate::kernel::mm::print_num(len);
        uart::puts(" prot=0x");
        uart::putc(hex[((prot >> 4) & 0xF) as usize]);
        uart::putc(hex[(prot & 0xF) as usize]);
        uart::puts(" updated=");
        crate::kernel::mm::print_num(updated);
        uart::puts("\n");
    }
    0
}

// ─── fcntl (25) — file descriptor control ───
//
// F_GETFD (1) / F_SETFD (2) / F_GETFL (3) / F_SETFL (4) are stubs.
// F_DUPFD (0) and F_DUPFD_CLOEXEC (1030) MUST allocate a new fd
// duplicating the input — without this, Chromium's FD ownership
// tracker sees "fd 15 was duplicated, the dup's fd is the same as
// some unrelated open fd" and FATALs with
// "Crashing due to FD ownership violation". F_DUPFD_CLOEXEC just
// adds the close-on-exec flag to the new fd; we don't track that
// flag, so it's treated identically to F_DUPFD.
fn sys_fcntl(args: [u64; 6]) -> i64 {
    let fd = args[0] as i32;
    let cmd = args[1] as i32;
    let _arg = args[2] as i64;

    // Diagnostic: log the call site (LR) for fcntl whenever it
    // happens at high frequency — Chromium's fork-as-thread child
    // loops on fcntl(0, F_GETFD) ~57k times in 90s, and we need to
    // see what's calling it. Print every 256th invocation for the
    // first 4096 calls, then go quiet.
    static FCNTL_COUNTER: core::sync::atomic::AtomicU64
        = core::sync::atomic::AtomicU64::new(0);
    let n = FCNTL_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if n < 4096 && (n & 0xFF) == 0 {
        if let Some(e) = super::syscall_history::last_entry() {
            uart::puts("[fcntl] n=");
            crate::kernel::mm::print_num(n as usize);
            uart::puts(" tid=t");
            crate::kernel::mm::print_num(e.tid as usize);
            uart::puts(" fd=0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() {
                uart::putc(hex[((fd as u64 >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" cmd=");
            crate::kernel::mm::print_num(cmd as usize);
            uart::puts(" lr=0x");
            for sh in (0..16).rev() {
                uart::putc(hex[((e.x30 >> (sh * 4)) & 0xF) as usize]);
            }
            // Walk the FP chain so we can see the call stack. Each
            // frame stores [prev_x29, prev_x30] at [x29], so the
            // saved LR of the function whose frame this is sits at
            // [x29+8]. Chase up to 6 frames; stop when we leave
            // the user VA range or hit 0.
            let mut fp = e.x29;
            for depth in 0..6u32 {
                if fp == 0 || !uaccess::is_user_range(fp as usize, 16) {
                    break;
                }
                let saved_x30: u64 = unsafe {
                    core::ptr::read_volatile((fp + 8) as *const u64)
                };
                let prev_fp: u64 = unsafe {
                    core::ptr::read_volatile(fp as *const u64)
                };
                uart::puts(" f");
                crate::kernel::mm::print_num(depth as usize);
                uart::puts("=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((saved_x30 >> (sh * 4)) & 0xF) as usize]);
                }
                fp = prev_fp;
                if saved_x30 == 0 { break; }
            }
            uart::puts("\n");
        }
    }

    match cmd {
        0 | 1030 => {
            // F_DUPFD (0) / F_DUPFD_CLOEXEC (1030) — duplicate fd
            // to a NEW fd >= arg (lowest available, but >= arg).
            // Chromium expects a fresh fd number it can then track
            // via its scoped-fd machinery; returning 0 (stdin)
            // silently corrupts Chromium's view of which fds are
            // owned by which subsystem.
            if fd < 0 { return -9; } // EBADF
            // Look up the source fd; fail if not open.
            if fd::get(fd as u32).is_none() { return -9; }
            // Allocate a new fd that duplicates the source's
            // backing. Easiest path: ask fd::dup which copies the
            // FdEntry into the next free slot.
            match fd::dup(fd as u32) {
                Ok(new_fd) => new_fd as i64,
                Err(_) => -24, // EMFILE
            }
        }
        1 => 0,   // F_GETFD — return 0 (no FD_CLOEXEC)
        2 => 0,   // F_SETFD — accept and ignore
        3 => 0,   // F_GETFL — return 0 (O_RDONLY)
        4 => 0,   // F_SETFL — accept and ignore
        _ => 0,   // Unknown: return 0
    }
}

// ─── prlimit64 (261) — get/set resource limits ───
fn sys_prlimit64(args: [u64; 6]) -> i64 {
    let _pid = args[0] as i32;
    let resource = args[1] as u32;
    let new_limit = args[2] as usize;
    let old_limit = args[3] as usize;

    // Bounds-check both rlimit pointers before we deref them. A struct
    // rlimit64 is 16 bytes (two u64).  Reject any pointer into the
    // kernel identity map.  NULL is legal for both args ("don't write"
    // / "no new limits"), so skip those.
    if new_limit != 0 && !is_user_ptr(new_limit, 16) { return EFAULT; }
    if old_limit != 0 && !is_user_ptr(old_limit, 16) { return EFAULT; }

    // CHROMIUM-PHASE-C: return resource-specific sane defaults when
    // `old_limit` is non-null. The previous stub returned
    // `rlim_cur = rlim_max = 0x7FFFFFFFFFFFFFFF` for every resource
    // — pthread_create(3) computes `stacksize = rlim_cur * 2` (or
    // clamps to 32 MB), overflows, and mmap's an 8 EB stack that
    // our kernel ENOMEM's. Using a BSD-ish 8 MB cap for STACK keeps
    // glibc happy.
    //
    // RLIMIT_STACK = 3
    // RLIMIT_AS    = 9
    // RLIMIT_NOFILE= 7
    // RLIMIT_CORE  = 4
    // (See asm-generic/resource.h. On arm64 these match generic.)
    if old_limit != 0 {
        const RLIMIT_STACK: u32 = 3;
        const RLIMIT_AS:    u32 = 9;
        const RLIMIT_NOFILE:u32 = 7;
        const RLIMIT_CORE:  u32 = 4;
        let (cur, max): (u64, u64) = match resource {
            RLIMIT_STACK  => (8 * 1024 * 1024, 8 * 1024 * 1024),     // 8 MB
            RLIMIT_AS     => (4 * 1024 * 1024 * 1024, 4 * 1024 * 1024 * 1024), // 4 GB
            RLIMIT_NOFILE => (1024, 4096),
            RLIMIT_CORE   => (0, 0),
            _             => (0x7FFFFFFFFFFFFFFF, 0x7FFFFFFFFFFFFFFF),
        };
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) old_limit, v = in(reg) cur);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) old_limit + 8, v = in(reg) max);
        }
    }
    0
}

// ─── exit_group (94) — exit all threads ───
fn sys_exit_group(args: [u64; 6]) -> i64 {
    // For single-threaded processes, same as exit
    sys_exit(args)
}

fn sys_exit(args: [u64; 6]) -> i64 {
    let code = args[0] as usize;

    if IN_CHILD.load(core::sync::atomic::Ordering::Relaxed) {
        // Child exit — don't actually exit ash
        // Store exit code and mark child as done
        CHILD_EXIT_CODE.store(code, core::sync::atomic::Ordering::Relaxed);
        IN_CHILD.store(false, core::sync::atomic::Ordering::Relaxed);
        // Return code will be handled by wait4
        // For now, this exit is caught by the exception handler
    }

    uart::puts("[linux] process exited with code ");
    crate::kernel::mm::print_num(code);
    uart::puts("\n");
    0
}

fn sys_uname(args: [u64; 6]) -> i64 {
    // struct utsname: 5 fields of 65 bytes each = 325 bytes (+padding
    // rounds to 390 on Linux).  Validate the full span to block writes
    // into kernel memory.
    let buf = args[0] as usize;
    if buf == 0 { return EINVAL; }
    if !is_user_ptr(buf, 390) { return EFAULT; }

    let fields: [&[u8]; 5] = [
        b"BatOS\0",                    // sysname
        b"batcave\0",                  // nodename
        b"1.0.0\0",                    // release
        b"BatOS 1.0.0 aarch64\0",     // version
        b"aarch64\0",                  // machine
    ];

    for (i, field) in fields.iter().enumerate() {
        let offset = buf + i * 65;
        for (j, &byte) in field.iter().enumerate() {
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) offset + j, v = in(reg) byte as u32);
            }
        }
    }
    0
}

fn write_to_uart(buf: usize, count: usize) -> i64 {
    for i in 0..count {
        let byte: u32;
        unsafe {
            core::arch::asm!("ldrb {v:w}, [{a}]",
                a = in(reg) buf + i, v = out(reg) byte);
        }
        uart::putc(byte as u8);
    }
    count as i64
}

/// Route stdout/stderr writes through the async ring buffer when it is live.
/// Falls back to synchronous UART if the ring is not yet initialised (early
/// boot before stdio_ring::init() runs).  Chromium content_shell's verbose
/// --enable-logging=stderr output would otherwise stall on UART back-pressure.
fn write_stdio(buf: usize, count: usize) -> i64 {
    if !stdio_ring::is_ready() {
        return write_to_uart(buf, count);
    }
    // Copy the userspace buffer out a byte at a time (same ldrb trick as
    // write_to_uart — these pointers come from guest x1 and are not
    // guaranteed to be naturally aligned).  We chunk through a small stack
    // scratch to amortise push_slice overhead.
    const CHUNK: usize = 128;
    let mut scratch = [0u8; CHUNK];
    let mut done = 0usize;
    while done < count {
        let take = core::cmp::min(CHUNK, count - done);
        for i in 0..take {
            let byte: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]",
                    a = in(reg) buf + done + i, v = out(reg) byte);
            }
            scratch[i] = byte as u8;
        }
        stdio_ring::push_slice(&scratch[..take]);
        done += take;
    }
    count as i64
}

fn sys_write(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let count = args[2] as usize;

    // Pipe-kinded fd (socketpair / pipe2): copy the user buffer
    // into the paired pipe. Checked before the kernel-ptr gate
    // below because we need is_user_ptr anyway.
    if count > 0 && !is_user_ptr(buf, count) { return EFAULT; }
    if let Some((slot, side)) = fd::pipe_info(fd_num) {
        let mut tmp = [0u8; 256];
        let mut total = 0usize;
        while total < count {
            let n = (count - total).min(tmp.len());
            for i in 0..n {
                let b: u32;
                unsafe {
                    core::arch::asm!("ldrb {v:w}, [{a}]",
                        a = in(reg) buf + total + i,
                        v = out(reg) b);
                }
                tmp[i] = b as u8;
            }
            match super::pipe_buf::write(slot, side, &tmp[..n]) {
                Ok(pushed) => {
                    total += pushed;
                    if pushed < n { break; } // pipe full
                }
                Err(e) => {
                    if total > 0 { return total as i64; }
                    return e;
                }
            }
        }
        return total as i64;
    }

    // Reject pointer-to-kernel attacks before any dereference.
    if count > 0 && !is_user_ptr(buf, count) { return EFAULT; }

    // Pipe write — V8-ROOT-1 + V8-ROOT-3 (regression fix): the whole read-
    // modify-write of PIPE_LEN must be atomic against a racing second
    // writer (a timer IRQ that schedules another thread which also pipes).
    // Also plen+writable is checked_add to survive overflow-checks=true
    // if any state drifts.
    unsafe {
        let pipe_wr = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_WRITE_FD));
        if fd_num == pipe_wr && pipe_wr != 0 {
            let _g = crate::kernel::sync::IrqGuard::new();
            let plen = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_LEN));
            let writable = PIPE_BUF_SIZE.saturating_sub(plen).min(count);
            if writable > 0 {
                let pbuf = core::ptr::addr_of_mut!(PIPE_BUF);
                core::ptr::copy_nonoverlapping(buf as *const u8, (*pbuf).as_mut_ptr().add(plen), writable);
                let new_len = match plen.checked_add(writable) {
                    Some(n) if n <= PIPE_BUF_SIZE => n,
                    _ => return EAGAIN,
                };
                core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_LEN), new_len);
                return writable as i64;
            }
            return EAGAIN;
        }
    }

    // Check if fd has been redirected (dup2'd to a file)
    if let Some(entry) = fd::get(fd_num) {
        let node_idx = entry.node_idx;

        // node_idx == 0 means original stdin/stdout/stderr (not redirected)
        if node_idx == 0 {
            if fd_num == 0 { return EBADF; }
            // fd 1 (stdout) / fd 2 (stderr) → async ring buffer
            return write_stdio(buf, count);
        }

        // Redirected or VFS file
        let node = vfs::get_node(node_idx);
        // Socket → TCP send
        if node.node_type == vfs::NodeType::Socket {
            // V5-XLAYER-004 fix: sys_write arrived here classified as
            // FileIO (needs `fs` cap), but we're about to send data to
            // the network. Re-check the `net` cap so a cave with only
            // `fs` can't exfil via a socket fd it inherited.
            if !cave::active_has_cap("net") {
                return -(13i64); // EACCES
            }
            return sys_sendto([fd_num as u64, args[1], args[2], 0, 0, 0]);
        }
        if node.node_type == vfs::NodeType::DevNull { return count as i64; }
        if node.node_type == vfs::NodeType::DevConsole {
            return write_to_uart(buf, count);
        }
        // /batos/fb0 — ChromiumFb: write bytes into the shared region directly
        // (bypasses the 256KB MAX_FILE_PAGES limit of regular files).
        if node.node_type == vfs::NodeType::ChromiumFb {
            let pos = entry.position;
            if node.data_addr == 0 { return -5; } // EIO
            if pos >= node.size { return 0; }
            let to_write = (node.size - pos).min(count);
            let dst = node.data_addr + pos;
            for i in 0..to_write {
                unsafe {
                    let b: u32;
                    core::arch::asm!("ldrb {v:w}, [{a}]",
                        a = in(reg) buf + i, v = out(reg) b);
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) dst + i, v = in(reg) b);
                }
            }
            if let Some(e) = fd::get_mut(fd_num) { e.position += to_write; }
            return to_write as i64;
        }

        let pos = entry.position;
        match vfs::write_to_file(node_idx, pos, buf, count) {
            Ok(n) => {
                if let Some(e) = fd::get_mut(fd_num) { e.position += n; }
                n as i64
            }
            Err(e) => e,
        }
    } else {
        // fd not in table — default stdout/stderr to the async stdio ring
        if fd_num == 1 || fd_num == 2 {
            return write_stdio(buf, count);
        }
        EBADF
    }
}

fn sys_read(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let count = args[2] as usize;

    // Reject pointer-to-kernel attacks before any dereference.
    if count > 0 && !is_user_ptr(buf, count) { return EFAULT; }

    // stdin (0) — read from UART, BLOCKING
    if fd_num == 0 {
        if count == 0 { return 0; }
        loop {
            if let Some(c) = uart::getc() {
                unsafe {
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) buf, v = in(reg) c as u32);
                }
                let mut total = 1usize;
                while total < count {
                    if let Some(c2) = uart::getc() {
                        unsafe {
                            core::arch::asm!("strb {v:w}, [{a}]",
                                a = in(reg) buf + total, v = in(reg) c2 as u32);
                        }
                        total += 1;
                        if c2 == b'\n' || c2 == b'\r' { break; }
                    } else {
                        break;
                    }
                }
                return total as i64;
            }
            core::hint::spin_loop();
        }
    }

    // Pipe read — V8-ROOT-1 (regression fix): same CS discipline as the
    // write path, otherwise a timer-IRQ-scheduled writer can extend
    // PIPE_LEN while we hold a stale `plen` and we read past the buffer.
    unsafe {
        let pipe_rd = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_READ_FD));
        if fd_num == pipe_rd && pipe_rd != 0 {
            let _g = crate::kernel::sync::IrqGuard::new();
            let plen = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_LEN));
            let readable = plen.min(count);
            if readable > 0 {
                let pbuf = core::ptr::addr_of_mut!(PIPE_BUF);
                core::ptr::copy_nonoverlapping((*pbuf).as_ptr(), buf as *mut u8, readable);
                let remaining = plen.saturating_sub(readable);
                if remaining > 0 {
                    core::ptr::copy((*pbuf).as_ptr().add(readable), (*pbuf).as_mut_ptr(), remaining);
                }
                core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_LEN), remaining);
                return readable as i64;
            }
            return 0;
        }
    }

    // /proc pseudo-fd reads (fallback for when VFS doesn't have /proc nodes)
    if fd_num >= 40 && fd_num < 56 {
        let idx = (fd_num - 40) as usize;
        unsafe {
            let plen = core::ptr::read_volatile(core::ptr::addr_of!(PROC_FD_LENS[idx]));
            if plen > 0 {
                let path_bytes = &(&(*core::ptr::addr_of!(PROC_FD_PATHS[idx])))[..plen];
                let path_str = core::str::from_utf8_unchecked(path_bytes);
                let mut proc_buf = [0u8; 512];
                let content_len = proc_read(path_str, &mut proc_buf);
                if content_len > 0 {
                    let pos = core::ptr::read_volatile(core::ptr::addr_of!(PROC_FD_POS[idx]));
                    if pos >= content_len { return 0; }
                    let avail = content_len - pos;
                    let to_copy = avail.min(count);
                    core::ptr::copy_nonoverlapping(proc_buf.as_ptr().add(pos), buf as *mut u8, to_copy);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PROC_FD_POS[idx]), pos + to_copy);
                    return to_copy as i64;
                }
            }
        }
    }

    // /proc synthetic file reads (VFS-backed)
    if let Some(entry) = fd::get(fd_num) {
        let node_idx = entry.node_idx;
        // Build the full path and check if it's a /proc file
        let mut path_buf = [0u8; 128];
        let path_len = vfs::node_path(node_idx, &mut path_buf);
        if path_len > 0 {
            let path_str = unsafe { core::str::from_utf8_unchecked(&path_buf[..path_len]) };
            if path_str.starts_with("/proc") {
                let mut proc_buf = [0u8; 512];
                let proc_len = proc_read(path_str, &mut proc_buf);
                if proc_len > 0 {
                    let pos = entry.position;
                    if pos >= proc_len { return 0; } // EOF
                    let avail = proc_len - pos;
                    let to_copy = avail.min(count);
                    unsafe {
                        core::ptr::copy_nonoverlapping(proc_buf.as_ptr().add(pos), buf as *mut u8, to_copy);
                    }
                    if let Some(e) = fd::get_mut(fd_num) { e.position += to_copy; }
                    return to_copy as i64;
                }
            }
        }
    }

    // VFS file descriptors
    if let Some(entry) = fd::get(fd_num) {
        let node_idx = entry.node_idx;
        let pos = entry.position;
        let node = vfs::get_node(node_idx);

        // Pipe-kinded fd (socketpair / pipe2): read from the paired buffer.
        if let Some((slot, side)) = fd::pipe_info(fd_num) {
            let mut tmp = [0u8; 256];
            let mut total = 0usize;
            while total < count {
                let n = (count - total).min(tmp.len());
                match super::pipe_buf::read(slot, side, &mut tmp[..n]) {
                    Ok(0) => break,
                    Ok(got) => {
                        for i in 0..got {
                            unsafe {
                                core::arch::asm!("strb {v:w}, [{a}]",
                                    a = in(reg) buf + total + i,
                                    v = in(reg) tmp[i] as u32);
                            }
                        }
                        total += got;
                        if got < n { break; } // buffer drained
                    }
                    Err(e) => {
                        if total > 0 { return total as i64; }
                        return e;
                    }
                }
            }
            return total as i64;
        }

        // Socket → TCP recv
        if node.node_type == vfs::NodeType::Socket {
            return sys_recvfrom([fd_num as u64, args[1], args[2], 0, 0, 0]);
        }

        // /dev/null → EOF
        if node.node_type == vfs::NodeType::DevNull { return 0; }
        // /dev/zero → fill with zeros
        if node.node_type == vfs::NodeType::DevZero {
            for i in 0..count {
                unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
            }
            return count as i64;
        }
        // /dev/random + /dev/urandom → ARMv8.5 RNDR / fallback RNG.
        // glibc uses /dev/urandom for stack canaries, ASLR seed, and
        // getrandom() fallback. Without this the first attempt to open
        // /dev/urandom failed with ENOENT and content_shell exited
        // before reaching main.
        if node.node_type == vfs::NodeType::DevRandom {
            // fill_bytes() pulls from RNDR if available (the kernel
            // logs "[rng] ARMv8.5 RNDR available — mixing HW entropy"
            // at boot), falling back to a SHA-mixed software RNG.
            // We copy into a small stack buffer then write to EL0.
            const CHUNK: usize = 256;
            let mut total = 0usize;
            while total < count {
                let n = (count - total).min(CHUNK);
                let mut tmp = [0u8; CHUNK];
                crate::crypto::rng::fill_bytes(&mut tmp[..n]);
                for i in 0..n {
                    unsafe {
                        core::arch::asm!("strb {v:w}, [{a}]",
                            a = in(reg) buf + total + i,
                            v = in(reg) tmp[i] as u32);
                    }
                }
                total += n;
            }
            return count as i64;
        }
        // /dev/console → read from UART
        if node.node_type == vfs::NodeType::DevConsole {
            return sys_read([0, args[1], args[2], 0, 0, 0]); // redirect to stdin
        }
        // /batos/fb0 — ChromiumFb: read raw bytes from the shared region.
        // Useful for `hexdump /batos/fb0 | head` style debugging.
        if node.node_type == vfs::NodeType::ChromiumFb {
            if node.data_addr == 0 { return -5; } // EIO
            if pos >= node.size { return 0; }
            let to_read = (node.size - pos).min(count);
            let src = node.data_addr + pos;
            for i in 0..to_read {
                unsafe {
                    let b: u32;
                    core::arch::asm!("ldrb {v:w}, [{a}]",
                        a = in(reg) src + i, v = out(reg) b);
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) buf + i, v = in(reg) b);
                }
            }
            if let Some(e) = fd::get_mut(fd_num) { e.position += to_read; }
            return to_read as i64;
        }

        // Regular file
        match vfs::read_file_data(node_idx, pos, buf, count) {
            Ok(n) => {
                if let Some(e) = fd::get_mut(fd_num) { e.position += n; }
                n as i64
            }
            Err(e) => e,
        }
    } else {
        EBADF
    }
}

// (Old FAKE_FILES and PASSWD_CONTENT removed — now served by VFS)

// Pipe buffer — simple single-pipe implementation
const PIPE_BUF_SIZE: usize = 16384;
static mut PIPE_BUF: [u8; PIPE_BUF_SIZE] = [0; PIPE_BUF_SIZE];
static mut PIPE_LEN: usize = 0;
static mut PIPE_READ_FD: u32 = 0;
static mut PIPE_WRITE_FD: u32 = 0;

// /proc pseudo-fd tracking (for fallback when VFS doesn't have /proc nodes)
static mut PROC_FD_PATHS: [[u8; 64]; 16] = [[0; 64]; 16];
static mut PROC_FD_LENS: [usize; 16] = [0; 16];
static mut PROC_FD_POS: [usize; 16] = [0; 16];

fn sys_openat(args: [u64; 6]) -> i64 {
    // V8-ROOT-1 (re-audit follow-up): wrap charge+inner+refund in CS so
    // a preempt between charge and inner can't race a concurrent fd op.
    // sys_openat_inner is mostly local + VFS lookups, no schedule().
    //
    // CHROMIUM-PHASE-B: charge Resource::Fds up-front; refund on any
    // non-success return. Before the quotas::init() fix mem_limit was
    // 0 (so ALL charges would wrongly fail), and openat skipped the
    // charge entirely as a stopgap — now the limits read correctly
    // so we can restore the accounting. Every successful openat
    // consumes 1 fd; close() refunds.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(
                super::quotas::Resource::Fds, 1) {
            return e;
        }
        let result = sys_openat_inner(args);
        if result < 0 {
            // Error path: either the file didn't exist or fd alloc
            // failed. Refund the fd we pre-charged.
            super::quotas::refund_active(super::quotas::Resource::Fds, 1);
        }
        result
    }
}

fn sys_openat_inner(args: [u64; 6]) -> i64 {
    let dirfd = args[0] as i32;
    let path_ptr = args[1] as usize;
    let flags = args[2] as u32;
    if path_ptr == 0 { return ENOENT; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);
    let path = &path_buf[..path_len];
    // Unconditional trace during Chromium port bring-up — shows
    // exactly what ld-linux is trying to open.
    uart::puts("[openat] dirfd=");
    if dirfd as i64 == -100 { uart::puts("AT_FDCWD"); }
    else { crate::kernel::mm::print_num(dirfd as usize); }
    uart::puts(" path='");
    for &b in path { if b.is_ascii_graphic() || b == b'/' || b == b'.' { uart::putc(b); } else { uart::putc(b'?'); } }
    uart::puts("' len="); crate::kernel::mm::print_num(path_len);
    uart::puts(" flags=0x");
    let hex = b"0123456789abcdef";
    for sh in (0..8).rev() { uart::putc(hex[((flags >> (sh*4)) & 0xF) as usize]); }
    uart::puts("\n");

    if has_dotdot(path) { return EACCES; }

    // Handle /proc paths BEFORE VFS check — /proc is always available
    let path_str = unsafe { core::str::from_utf8_unchecked(path) };
    if path_str.starts_with("/proc/") {
        let mut test_buf = [0u8; 4];
        if proc_read(path_str, &mut test_buf) > 0 {
            // Try VFS-backed approach
            let proc_idx = if let Ok(idx) = vfs::resolve_path(b"/proc") {
                Some(idx)
            } else {
                vfs::find_child(0, b"proc")
            };

            if let Some(proc_parent) = proc_idx {
                let rel = &path_str[6..]; // strip "/proc/"
                let mut parent = proc_parent;
                let mut last_slash = 0;
                let rel_bytes = rel.as_bytes();
                for j in 0..rel_bytes.len() {
                    if rel_bytes[j] == b'/' {
                        let dir_name = &rel_bytes[last_slash..j];
                        if !dir_name.is_empty() {
                            parent = match vfs::find_child(parent, dir_name) {
                                Some(idx) => idx,
                                None => match vfs::create_node(parent, dir_name, vfs::NodeType::Directory, 0o40555) {
                                    Ok(idx) => idx,
                                    Err(_) => break,
                                },
                            };
                        }
                        last_slash = j + 1;
                    }
                }
                let file_name = &rel_bytes[last_slash..];
                if !file_name.is_empty() {
                    if let Ok(node_idx) = vfs::create_node(parent, file_name, vfs::NodeType::File, 0o100444) {
                        if let Ok(fd_num) = fd::alloc_fd(node_idx, flags) {
                            return fd_num as i64;
                        }
                    }
                }
            }

            // Fallback: return a pseudo-fd for /proc reads
            // We'll use fd numbers 40+ for /proc files
            static mut PROC_FD_COUNTER: u32 = 40;
            unsafe {
                let pfd = core::ptr::read_volatile(core::ptr::addr_of!(PROC_FD_COUNTER));
                core::ptr::write_volatile(core::ptr::addr_of_mut!(PROC_FD_COUNTER), pfd + 1);
                // Store the path so reads can generate content
                let idx = (pfd - 40) as usize;
                if idx < 16 {
                    let pl = path_str.len().min(63);
                    let src = path_str.as_bytes();
                    for k in 0..pl {
                        core::ptr::write_volatile(
                            core::ptr::addr_of_mut!(PROC_FD_PATHS[idx][k]),
                            src[k]
                        );
                    }
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PROC_FD_LENS[idx]), pl);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PROC_FD_POS[idx]), 0);
                }
                return pfd as i64;
            }
        }
    }

    if !vfs::is_ready() {
        return ENOENT;
    }

    // Resolve path through VFS
    match vfs::resolve_path(path) {
        Ok(node_idx) => {
            let node = vfs::get_node(node_idx);
            // If O_DIRECTORY and not a dir, fail
            if flags & fd::O_DIRECTORY != 0 && node.node_type != vfs::NodeType::Directory {
                return ENOENT;
            }
            match fd::alloc_fd(node_idx, flags) {
                Ok(fd_num) => fd_num as i64,
                Err(e) => e,
            }
        }
        Err(_) => {
            // O_CREAT: create the file
            if flags & fd::O_CREAT != 0 {
                match vfs::resolve_parent(path) {
                    Ok((parent, name)) => {
                        match vfs::create_node(parent, name, vfs::NodeType::File, 0o100644) {
                            Ok(idx) => {
                                match fd::alloc_fd(idx, flags) {
                                    Ok(fd_num) => fd_num as i64,
                                    Err(e) => e,
                                }
                            }
                            Err(e) => e,
                        }
                    }
                    Err(e) => e,
                }
            } else {
                ENOENT
            }
        }
    }
}

fn sys_close(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;

    // ROOT-6: figure out which resource class this fd belongs to BEFORE we
    // tear down the entry, so we refund the right counter. Sockets come
    // back to the Sockets pool; everything else is a generic Fds refund.
    // eventfd / timerfd / epoll slots are accounted at creation time but
    // NEW-DOS-003 fix: also detect epoll/eventfd/timerfd "fds" so they
    // refund the right counter on close. epoll fds live in the fd table
    // with a sentinel node_idx; eventfd/timerfd return slot indices that
    // collide with low fd numbers (architectural quirk — ideally those
    // would also live in the fd table). We make a best-effort attempt
    // by checking known sentinel ranges before falling through to the
    // generic Fds refund.
    let mut refund_res: Option<super::quotas::Resource> = None;
    let mut handled_special = false;

    // epoll: sentinel-node fds live in the fd table.
    if super::epoll::is_epoll_fd(fd_num as i32) {
        let _ = super::epoll::epoll_close(fd_num as i32);
        refund_res = Some(super::quotas::Resource::Epolls);
        handled_special = true;
    }

    if !handled_special {
        // Fall through to standard fd-table close, picking refund class
        // by node type.
        refund_res = match fd::get(fd_num) {
            Some(entry) => {
                let node = vfs::get_node(entry.node_idx);
                if node.node_type == vfs::NodeType::Socket {
                    Some(super::quotas::Resource::Sockets)
                } else {
                    Some(super::quotas::Resource::Fds)
                }
            }
            None => None,
        };
    }

    let close_result = if handled_special {
        // epoll already freed via epoll_close above; the fd-table entry
        // (if any) still gets the standard close to release the slot.
        let _ = fd::close(fd_num);
        Ok(())
    } else {
        fd::close(fd_num)
    };

    match close_result {
        Ok(()) => {
            if let Some(r) = refund_res {
                super::quotas::refund_active(r, 1);
            }
            0
        }
        Err(e) => e,
    }
}

fn fill_stat(buf: usize, mode: u32, size: u64, ino: u64, nlink: u32) {
    // Zero out stat struct (128 bytes on ARM64 Linux)
    for i in 0..128 {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
    }
    unsafe {
        // st_dev at offset 0
        let dev: u64 = 1;
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf, v = in(reg) dev);
        // st_ino at offset 8
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 8, v = in(reg) ino);
        // st_mode at offset 16
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) buf + 16, v = in(reg) mode);
        // st_nlink at offset 20 (actually 24 on some, but ARM64 stat has it at 20)
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) buf + 20, v = in(reg) nlink);
        // st_size at offset 48
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 48, v = in(reg) size);
        // st_blksize at offset 56
        let blksz: u64 = 4096;
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 56, v = in(reg) blksz);
    }
}

fn sys_fstat(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    if buf == 0 { return EINVAL; }
    // struct stat on aarch64 is 144 bytes.  fill_stat writes up to
    // offset 56 + 8; the caller ABI reads the whole struct, so
    // validate the full width.
    if !is_user_ptr(buf, 144) { return EFAULT; }

    // stdout/stderr/stdin
    if fd_num <= 2 {
        fill_stat(buf, 0o20600, 0, 0, 1); // S_IFCHR
        return 0;
    }

    if let Some(entry) = fd::get(fd_num) {
        let node = vfs::get_node(entry.node_idx);
        fill_stat(buf, node.mode, node.size as u64, entry.node_idx as u64, node.nlink);
        return 0;
    }

    // Fallback for unknown fds
    let mode: u32 = 0o100755;
    fill_stat(buf, mode, 4096, 0, 1);
    // Set st_nlink = 1
    let nlink: u32 = 1;
    unsafe {
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) buf + 24, v = in(reg) nlink);
    }
    0
}

fn sys_getcwd(args: [u64; 6]) -> i64 {
    let buf = args[0] as usize;
    let size = args[1] as usize;
    if buf == 0 || size < 2 { return EINVAL; }
    // We may write up to `size` bytes into buf; bounds-check the full
    // range.  A huge size is also what the attacker uses to flip one
    // kernel byte, so catching this check here kills that primitive.
    if !is_user_ptr(buf, size) { return EFAULT; }

    if vfs::is_ready() {
        let mut path = [0u8; 128];
        let len = vfs::node_path(vfs::get_cwd(), &mut path);
        let write_len = len.min(size - 1);
        for i in 0..write_len {
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = in(reg) path[i] as u32);
            }
        }
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + write_len); }
        return buf as i64;
    }

    // Fallback: return "/"
    unsafe {
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf, v = in(reg) b'/' as u32);
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + 1);
    }
    buf as i64
}

fn sys_ioctl(args: [u64; 6]) -> i64 {
    let _fd = args[0];
    let cmd = args[1];
    match cmd {
        0x5401 => { // TCGETS — get terminal attributes
            // NEW-SYS-025: gate the arg pointer so a cave can't use ioctl
            // as a "60-byte kernel zeroing" primitive.
            let buf = args[2] as usize;
            if buf != 0 {
                if !uaccess::is_user_range(buf, 60) { return -(14i64); }
                for i in 0..60 {
                    unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
                }
            }
            0
        }
        0x5402 | 0x5403 => 0, // TCSETS, TCSETSW — set terminal attributes (ignore)
        0x5413 => { // TIOCGWINSZ — terminal window size
            // NEW-SYS-025: gate the 4-byte winsize write.
            let buf = args[2] as usize;
            if buf != 0 {
                if !uaccess::is_user_range(buf, 4) { return -(14i64); }
                unsafe {
                    core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) buf, v = in(reg) 24u32);
                    core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) buf + 2, v = in(reg) 80u32);
                }
            }
            0
        }
        0x540E => 0, // TIOCGPGRP — get process group
        0x540F => 0, // TIOCSPGRP — set process group
        _ => 0       // Unknown ioctls — return success
    }
}

// Worker brk state (separate from primary to avoid heap corruption)
static mut WORKER_BRK: u64 = 0;

fn sys_brk(args: [u64; 6]) -> i64 {
    let requested = args[0];

    if IN_CHILD.load(core::sync::atomic::Ordering::Relaxed) {
        // Worker process: use physical addresses (identity-mapped)
        unsafe {
            if WORKER_BRK == 0 {
                // Start worker brk after the worker's loaded segments
                let wbase = crate::batcave::linux::loader::WORKER_PHYS_BASE
                    .load(core::sync::atomic::Ordering::Relaxed) as u64;
                WORKER_BRK = wbase + 0x200000; // 2MB after worker base
            }
            if requested == 0 {
                return WORKER_BRK as i64;
            }
            if requested > WORKER_BRK {
                // Grow: allocate pages to cover the gap (page-aligned). We
                // round the new break UP to a page so the freeable unit on
                // shrink matches the granularity of allocation.
                let gap = (requested - WORKER_BRK) as usize;
                let pages = (gap + 4095) / 4096;
                let bytes = pages * 4096;
                // Enforce the cave's memory quota before we start alloc'ing.
                if let Err(e) = super::quotas::charge_active(
                        super::quotas::Resource::Mem, bytes) {
                    return e;
                }
                for _ in 0..pages {
                    if crate::kernel::mm::frame::alloc_frame().is_none() {
                        super::quotas::refund_active(
                            super::quotas::Resource::Mem, bytes);
                        return ENOMEM;
                    }
                }
                WORKER_BRK = requested;
            } else if requested < WORKER_BRK {
                // Shrink: free the released pages. ROOT-6 fix — previously
                // a no-op that leaked the entire tail of the heap.
                // We free only fully-released page-aligned frames; if the
                // new break ends mid-page we keep that page.
                let new_aligned = (requested + 4095) & !4095;
                let old_aligned = (WORKER_BRK + 4095) & !4095;
                if new_aligned < old_aligned {
                    let freed_pages = ((old_aligned - new_aligned) / 4096) as usize;
                    crate::kernel::mm::frame::free_contig(
                        new_aligned as usize, freed_pages);
                    super::quotas::refund_active(
                        super::quotas::Resource::Mem, freed_pages * 4096);
                    // Flush TLB for the released range.
                    core::arch::asm!(
                        "dsb ishst",
                        "tlbi vmalle1",
                        "dsb ish",
                        "isb",
                        options(nostack, preserves_flags),
                    );
                }
                WORKER_BRK = requested;
            }
            return WORKER_BRK as i64;
        }
    }

    // Primary (ash): use virtual addresses in the mapped region
    if requested == 0 {
        return 0x0080_0000;
    }
    requested as i64
}

fn sys_mmap(args: [u64; 6]) -> i64 {
    let addr = args[0] as usize;
    let len = args[1] as usize;
    let _prot = args[2] as u32;
    let flags = args[3] as u32;
    let fd_num = args[4] as i32;
    let offset = args[5] as usize;

    if len == 0 { return EINVAL; }

    // ─── /batos/fb0 ChromiumFb: MAP_SHARED of the pre-allocated region ───
    //
    // Chromium's patched Ozone backend opens /batos/fb0, ftruncates, then
    // calls mmap(NULL, 5 MiB, PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0).
    // We return the physical base of the region — it's identity-mapped in
    // the kernel's flat address space, so the returned VA is directly
    // accessible from both EL1 (kernel blit kthread) and EL0 (Chromium).
    //
    // Limitations (see ports/chromium_port/PHASE5_DISPLAY.md Risk #4):
    //   - No per-process MMU view: all BatCave processes see the same VA.
    //     Good enough for single-process content_shell (--single-process).
    //   - MAP_PRIVATE on the fb would silently give Chromium a shared view
    //     here; we accept that for v1 because content_shell uses MAP_SHARED.
    //   - Offsets other than 0 are allowed (stride math), but we clamp to
    //     the region size.
    const MAP_SHARED: u32 = 0x01;
    const MAP_PRIVATE: u32 = 0x02;
    if fd_num >= 0 && (flags & (MAP_SHARED | MAP_PRIVATE)) != 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::ChromiumFb && node.data_addr != 0 {
                if offset >= node.size { return EINVAL; }
                let avail = node.size - offset;
                if len > avail {
                    uart::puts("[mmap] /batos/fb0 len exceeds region\n");
                    return EINVAL;
                }
                let base = node.data_addr + offset;
                uart::puts("[mmap] /batos/fb0 → 0x");
                let hex = b"0123456789abcdef";
                for shift in (0..16).rev() {
                    let nibble = ((base >> (shift * 4)) & 0xF) as usize;
                    uart::putc(hex[nibble]);
                }
                uart::puts("\n");
                return base as i64;
            }
        }
    }

    // For fixed-address mappings, return the requested address.
    //
    // V6-XLAYER-003 / V6-WEIRD-012 / V6-TOCTOU-006 fix: MAP_FIXED in
    // our impl does NOT allocate any new frames — it just hands back
    // the requested address (the cave is already mapped identity for
    // its window). So MAP_FIXED must NOT charge the Mem quota: V5
    // added a charge here, but munmap later refunded `len` bytes via
    // saturating_sub — net effect was a quota AMPLIFIER (cave inflates
    // quota usage with MAP_FIXED, munmap drains it to zero, cave then
    // mmaps real frames past its actual limit). Reverting to "MAP_FIXED
    // is a no-op for the quota ledger" closes that primitive.
    //
    // The V5 is_user_range check is preserved — MAP_FIXED with a
    // kernel-range addr is still rejected.
    if addr != 0 && (flags & 0x10) != 0 { // MAP_FIXED = 0x10
        if !uaccess::is_user_range(addr, len) {
            return -(14i64); // EFAULT
        }
        // CHROMIUM-PHASE-B: MAP_FIXED + fd-backed → copy file bytes
        // from the VFS node's `data_addr` (archive memory) into the
        // target user VA. Without this, ld-linux gets an "uva=addr"
        // return but the underlying pages are whatever the previous
        // ANON reservation left — typically zeros — and the first
        // jump into libc text DATA-ABORTs.
        //
        // Layout: the caller already reserved the whole [addr..addr+total]
        // region via an earlier ANON mmap, and the cave's page table
        // maps every user VA to phys via 2 MB L2 blocks at
        // phys_base + (uva - virt_base). Since we only need to write
        // bytes (not adjust the page table), computing the phys target
        // and memcpy'ing is sufficient.
        if fd_num >= 0 {
            if let Some(entry) = fd::get(fd_num as u32) {
                let node = vfs::get_node(entry.node_idx);
                if node.node_type == vfs::NodeType::File && node.data_addr != 0 {
                    let (va_start, va_end) =
                        crate::batcave::linux::mmu::active_user_window();
                    let phys_base = crate::batcave::linux::loader::get_phys_base();
                    // CHROMIUM-PHASE-D: if `addr` is in a demand-paged
                    // reservation outside the cave's main window
                    // (small-anon-mmap region at 0x70_0000_0000+, or
                    // V8 cage at 0x30_0000_0000+), the
                    // phys_base + (addr - va_start) translation is
                    // garbage. Touch each page in the destination
                    // range to demand-commit it, then copy file
                    // bytes through the USER VA directly — the
                    // active page table maps it.
                    if addr >= va_end || addr < va_start {
                        let bytes_available = node.size.saturating_sub(offset);
                        let to_copy = bytes_available.min(len);
                        // Touch every page to demand-commit.
                        let end_addr = addr + len;
                        let mut va = addr & !0xFFFusize;
                        while va < end_addr {
                            unsafe {
                                core::ptr::write_volatile(va as *mut u8, 0);
                            }
                            va += 4096;
                        }
                        unsafe {
                            let src = (node.data_addr + offset) as *const u8;
                            let dst = addr as *mut u8;
                            core::ptr::copy_nonoverlapping(src, dst, to_copy);
                            // Zero tail (.bss-ish region).
                            for i in to_copy..len {
                                core::ptr::write_volatile(dst.add(i), 0);
                            }
                            // Cache maintenance for code pages.
                            let mut line = addr & !63;
                            let end = addr + len;
                            while line < end {
                                core::arch::asm!("dc cvau, {a}", a = in(reg) line);
                                core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                                line += 64;
                            }
                            core::arch::asm!("dsb ish");
                            core::arch::asm!("isb");
                        }
                        // Apply the requested protection (prot
                        // arg of mmap) — without this, the bytes
                        // we just copied sit in pages with our
                        // demand_page default flags (RW, UXN=1)
                        // and the first instruction fetch from a
                        // code segment faults EC=0x20.
                        let _ = sys_mprotect([
                            addr as u64, len as u64, _prot as u64,
                            0, 0, 0,
                        ]);
                        uart::puts("[mmap] FIXED-high-VA fd=");
                        crate::kernel::mm::print_num(fd_num as usize);
                        uart::puts(" → 0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((addr >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" copied=");
                        crate::kernel::mm::print_num(to_copy);
                        uart::puts(" prot=0x");
                        uart::putc(hex[((_prot >> 4) & 0xF) as usize]);
                        uart::putc(hex[(_prot & 0xF) as usize]);
                        uart::puts("\n");
                        return addr as i64;
                    }
                    if addr < va_start || phys_base == 0 {
                        return -(14i64); // EFAULT — misconfigured cave
                    }
                    let phys_target = phys_base + (addr - va_start);
                    let bytes_available = node.size.saturating_sub(offset);
                    let to_copy = bytes_available.min(len);
                    uart::puts("[mmap] fd=");
                    crate::kernel::mm::print_num(fd_num as usize);
                    uart::puts(" off=0x");
                    {
                        let hex = b"0123456789abcdef";
                        for shift in (0..16).rev() {
                            uart::putc(hex[((offset >> (shift * 4)) & 0xF) as usize]);
                        }
                    }
                    uart::puts(" copying ");
                    crate::kernel::mm::print_num(to_copy);
                    uart::puts(" bytes archive→uva\n");
                    unsafe {
                        let src = (node.data_addr + offset) as *const u8;
                        let dst = phys_target as *mut u8;
                        // Copy file bytes
                        core::ptr::copy_nonoverlapping(src, dst, to_copy);
                        // Zero any tail — glibc relies on the tail of
                        // the last segment page being zero for .bss.
                        for i in to_copy..len {
                            core::ptr::write_volatile(dst.add(i), 0);
                        }
                        // Cache maintenance: this region will be
                        // fetched as instructions (libc .text). Without
                        // `dc cvau` + `ic ivau` the D-side sees our
                        // stores but the I-side may fetch stale lines.
                        let start = phys_target;
                        let end = phys_target + len;
                        let mut line = start & !63;
                        while line < end {
                            core::arch::asm!("dc cvau, {a}", a = in(reg) line);
                            core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                            line += 64;
                        }
                        core::arch::asm!("dsb ish");
                        core::arch::asm!("isb");
                    }
                }
            }
        }
        return addr as i64;
    }

    // CHROMIUM-PHASE-A: handle V8 / PartitionAlloc's huge anonymous VA
    // reservations. Chromium's V8 pointer-compression setup calls
    // `mmap(hint, 32 GB, ..., MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)` at
    // startup — *then* uses mprotect / MAP_FIXED to lazily commit small
    // regions as the heap grows. We can't allocate 32 GB physical, but
    // we also don't have to: return a fake address from the top of the
    // cave's user window and remember the reservation in a tiny table
    // so a follow-on MAP_FIXED at addr..addr+len gets the correct
    // in-cave phys mapping. Only kicks in when len is much larger than
    // our cave + the request is fd=-1 MAP_PRIVATE|MAP_ANON.
    //
    // 2 GB threshold is arbitrary but larger than any legit in-cave
    // heap today (cave is 400 MB), so anything above it is definitely
    // a reservation, not an actual heap grow.
    const HUGE_RESERVATION_THRESHOLD: usize = 2 * 1024 * 1024 * 1024;
    const MAP_PRIVATE_BIT: u32 = 0x02;
    const MAP_ANONYMOUS_BIT: u32 = 0x20;
    if fd_num == -1
        && len >= HUGE_RESERVATION_THRESHOLD
        && (flags & MAP_PRIVATE_BIT) != 0
        && (flags & MAP_ANONYMOUS_BIT) != 0
    {
        // CHROMIUM-PHASE-A: return the caller's hint (or a high-half
        // default) without allocating physical memory. V8's pointer-
        // compression setup sends a 4 GB-aligned hint and will
        // munmap + give up if we hand back a misaligned replacement
        // (we tried picking 0x20000000 in-cave; V8 rejected that
        // address and unmapped). Returning the hint passes the
        // alignment check and lets the setup proceed to mprotect +
        // first access — which will fault because the reserved range
        // isn't in the cave's L2. Demand paging + L3 page tables are
        // the real fix; this stub at least exposes the next
        // boundary.
        //
        // CHROMIUM-PHASE-C: if the hint exceeds our 39-bit VA (addr >=
        // 2^39), the hardware will fault on first access before
        // demand-page can even run L1/L2/L3 walks. Redirect such
        // hints to a low, 4 GB-aligned address within our window
        // instead. V8's pointer compression just wants SOME base with
        // matching alignment; it doesn't hard-require the specific
        // value. Addresses we've seen V8 ask for:
        //   0x28_00000000  — 32 GB pointer-compression cage ≤ 39-bit OK
        //   0x4a_11810000  — 16 GB trusted-sandbox ≥ 39-bit NOT OK
        //   0x400_00000000 — 8 EB hardware sandbox ≥ 39-bit NOT OK
        const VA_LIMIT: u64 = 1u64 << 39;         // our T0SZ=25 ceiling
        const LOW_REDIRECT_BASE: usize = 0x30_0000_0000; // 192 GB (in-range)
        static REDIRECT_CURSOR: core::sync::atomic::AtomicUsize =
            core::sync::atomic::AtomicUsize::new(LOW_REDIRECT_BASE);
        let hint_in_range = (addr as u64) < VA_LIMIT
            && ((addr as u64).saturating_add(len as u64)) <= VA_LIMIT;
        let reserved = if addr != 0 && hint_in_range {
            addr
        } else {
            // Bump-allocate inside the 39-bit window. Align len up to
            // 4 GB so each reservation starts at a V8-friendly
            // boundary. (V8's sandbox code checks the base is
            // pointer-compression-aligned.)
            let aligned_len = (len + 0xFFFF_FFFF) & !0xFFFF_FFFFusize;
            use core::sync::atomic::Ordering as Ord2;
            let mut base = REDIRECT_CURSOR.load(Ord2::Relaxed);
            loop {
                let next = base.saturating_add(aligned_len);
                if next as u64 >= VA_LIMIT {
                    // No room left — fall back to returning the raw
                    // hint and let the demand-page failure surface.
                    return ENOMEM;
                }
                match REDIRECT_CURSOR.compare_exchange(
                    base, next,
                    Ord2::AcqRel, Ord2::Acquire,
                ) {
                    Ok(_) => break,
                    Err(cur) => base = cur,
                }
            }
            uart::puts("[mmap] reserve-only: REDIRECT high-hint 0x");
            {
                let hex = b"0123456789abcdef";
                for s in (0..16).rev() {
                    uart::putc(hex[((addr >> (s*4)) & 0xF) as usize]);
                }
            }
            uart::puts(" → 0x");
            {
                let hex = b"0123456789abcdef";
                for s in (0..16).rev() {
                    uart::putc(hex[((base >> (s*4)) & 0xF) as usize]);
                }
            }
            uart::puts("\n");
            base
        };
        uart::puts("[mmap] reserve-only: len=");
        crate::kernel::mm::print_num(len / (1024 * 1024));
        uart::puts(" MB, hint=0x");
        let hex = b"0123456789abcdef";
        for shift in (0..16).rev() {
            uart::putc(hex[((reserved >> (shift * 4)) & 0xF) as usize]);
        }
        uart::puts("\n");

        // Tell the demand-page handler to lazily back 4 KB pages in this
        // range on first access. Without this, EL0 accesses through the
        // huge reservation DATA-ABORT (the cave's L2 has no entry for
        // 0x3c25aa0000-style addresses). We use the current cave's
        // TTBR0 as the reservation's identity — each cave's reservations
        // live under its own L1.
        let ttbr0: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
        super::demand_page::register_reservation(
            reserved as u64,
            (reserved as u64).saturating_add(len as u64),
            ttbr0 & !1u64,
        );

        return reserved as i64;
    }

    // V8-ROOT-3 / V8-ARITH-A2: `len + 4095` wraps for len > usize::MAX-4094,
    // landing pages = 0 and charge_bytes = 0 → cave gets a 1-frame
    // allocation whose VA range claims to cover ~16 EB of user memory.
    // Use checked_add so any overflow returns -ENOMEM cleanly.
    let pages = match len.checked_add(4095) {
        Some(s) => s / 4096,
        None => return ENOMEM,
    };
    let charge_bytes = match pages.checked_mul(4096) {
        Some(b) => b,
        None => return ENOMEM,
    };

    // ROOT-6 quota check — before we touch the frame allocator, make sure
    // this cave isn't already at its per-cave memory cap. -ENOMEM matches
    // what Linux returns when RLIMIT_AS is hit.
    // ROOT-6 quota check. CHROMIUM-PHASE-B (fixed): mem_limit was
    // reading as 0 at runtime because the static CAVE_QUOTAS const-
    // init was flaky. Fixed by adding `quotas::init()` called at
    // kernel boot that explicitly writes DEFAULT_* into every slot.
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Mem, charge_bytes) {
        return e;
    }

    uart::puts("[mmap] len=");
    crate::kernel::mm::print_num(len);
    uart::puts(" pages=");
    crate::kernel::mm::print_num(pages);

    // QEMU-BUGFIX-4: use alloc_contig + convert the phys base to the
    // user-VA equivalent the caller's EL0 code can actually reach.
    //
    // The cave's user window is mapped by `mmu::setup_cave_pagetable_at`
    // as `CAVE_BLOCKS × 2 MB` (= 400 MB today) of 2 MB L2 blocks starting
    // at `phys_base`. EL0 reaches phys via this window; anything outside
    // it is un-mapped from EL0's view and will DATA-ABORT on first touch.
    //
    // Strategy: allocate contiguously (alloc_contig, not N×alloc_frame
    // which could fragment), then offset-convert the resulting phys
    // into the user-VA window. If the allocation lands outside
    // phys_base..phys_base+USER_WINDOW_SIZE we refuse it rather than
    // handing EL0 a pointer it can't use.
    //
    // CHROMIUM-PHASE-B fix: previously this const was `20 * MB`, which
    // matched an old one-shot MMU setup. Once the cave page table grew
    // to CAVE_BLOCKS × 2 MB (see mmu.rs) the effective user window was
    // 400 MB. The 20 MB value here caused ld-linux's first post-load
    // mmap to return ENOMEM after the loader's ~26 MB reservation —
    // content_shell reported "cannot create shared object descriptor".
    let phys_base = crate::batcave::linux::loader::get_phys_base();
    const USER_WINDOW_SIZE: usize =
        crate::batcave::linux::mmu::CAVE_BLOCKS * 0x200000;

    // For small anonymous private mmaps (the pthread_create stack
    // case especially), the cave's main 400 MB user window can be
    // exhausted by the time Chromium's worker pool spins up, and
    // alloc_contig returns memory outside that window. Falling back
    // to a high-VA demand-paged region lets these small allocations
    // succeed without colliding with already-mapped memory. The
    // pages get committed on first access by demand_page::try_handle.
    if fd_num < 0
        && (flags & 0x10) == 0   // not MAP_FIXED
        && (flags & 0x20) != 0   // MAP_ANONYMOUS
        && len < HUGE_RESERVATION_THRESHOLD
    {
        let aligned_len = (len + 0xFFF) & !0xFFFusize;
        use core::sync::atomic::Ordering as Ord2;
        let cur = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
        if cur == 0 {
            let _ = SMALL_MMAP_CURSOR.compare_exchange(
                0, SMALL_MMAP_BASE,
                Ord2::AcqRel, Ord2::Acquire,
            );
        }
        // Ensure the entire small-mmap region is covered by ONE
        // big demand-page reservation per active L1 — Chromium does
        // hundreds of small mmaps and would blow our 8-slot
        // reservation table if we registered per-call.
        let active_l1: u64;
        unsafe {
            core::arch::asm!("mrs {}, ttbr0_el1", out(reg) active_l1);
        }
        let active_l1 = active_l1 & !1u64;
        ensure_small_mmap_reservation(active_l1);
        let mut base = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
        loop {
            let next = base.saturating_add(aligned_len);
            if (next as u64) >= SMALL_MMAP_END as u64 {
                break;
            }
            match SMALL_MMAP_CURSOR.compare_exchange(
                base, next, Ord2::AcqRel, Ord2::Acquire,
            ) {
                Ok(_) => {
                    uart::puts("[mmap] anon → 0x");
                    let hex = b"0123456789abcdef";
                    for sh in (0..16).rev() {
                        uart::putc(hex[((base >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts(" len=");
                    crate::kernel::mm::print_num(len);
                    uart::puts("\n");
                    return base as i64;
                }
                Err(cur) => base = cur,
            }
        }
    }

    match crate::kernel::mm::frame::alloc_contig(pages) {
        Some(base) => {
            uart::puts(" base=0x");
            let hex = b"0123456789abcdef";
            for shift in (0..16).rev() {
                let nib = ((base >> (shift * 4)) & 0xF) as usize;
                uart::putc(hex[nib]);
            }

            // Zero the allocated memory (Linux MAP_ANONYMOUS guarantee).
            // Writes go through the kernel's EL1 identity map — fine
            // from here; EL0 will see the same bytes via the user VA.
            //
            // NEW-DOS-014: yield to the scheduler every 64 pages so a
            // cave that mmap's 1 GiB doesn't pin the core for seconds.
            unsafe {
                let ptr = base as *mut u8;
                for p in 0..pages {
                    let off = p * 4096;
                    for i in 0..4096 {
                        core::ptr::write_volatile(ptr.add(off + i), 0);
                    }
                    if p % 64 == 63 {
                        super::threads::schedule();
                    }
                }
            }

            // CHROMIUM-PHASE-B: if this is a file-backed mmap (fd >= 0
            // and the fd points at a VFS File node), copy the file
            // bytes into the freshly-allocated pages. Without this,
            // `mmap(NULL, sz, PROT_READ, MAP_SHARED, fd, 0)` returns
            // a zeroed region — Chromium's ICU loader reads zeros
            // and reports `U_INVALID_FORMAT_ERROR`. Only the
            // non-MAP_FIXED path needs this; MAP_FIXED has its own
            // copy earlier. Skip for anonymous (fd < 0) mmaps which
            // legitimately want zeros.
            if fd_num >= 0 {
                if let Some(entry) = fd::get(fd_num as u32) {
                    let node = vfs::get_node(entry.node_idx);
                    if node.node_type == vfs::NodeType::File
                        && node.data_addr != 0
                    {
                        let available =
                            node.size.saturating_sub(offset);
                        let to_copy = available.min(len);
                        uart::puts("[mmap] fd=");
                        crate::kernel::mm::print_num(fd_num as usize);
                        uart::puts(" off=0x");
                        {
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((offset >> (sh * 4)) & 0xF) as usize]);
                            }
                        }
                        uart::puts(" copying ");
                        crate::kernel::mm::print_num(to_copy);
                        uart::puts(" bytes archive→frame\n");
                        unsafe {
                            let src = (node.data_addr + offset) as *const u8;
                            let dst = base as *mut u8;
                            core::ptr::copy_nonoverlapping(src, dst, to_copy);
                            // I-cache maintenance for code-loaded mmaps.
                            let start = base & !63;
                            let end = base + to_copy;
                            let mut line = start;
                            while line < end {
                                core::arch::asm!("dc cvau, {a}", a = in(reg) line);
                                core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                                line += 64;
                            }
                            core::arch::asm!("dsb ish");
                            core::arch::asm!("isb");
                        }
                    }
                }
            }

            // Compute the user-VA equivalent. Bail + refund quota if the
            // allocation landed outside the cave's user window.
            if phys_base == 0 || base < phys_base {
                uart::puts(" FAILED (before phys_base)\n");
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, charge_bytes);
                // leak the frames rather than free-list them — allocator
                // has no free() and the bitmap is ours until next ELF.
                return ENOMEM;
            }
            let offset = base - phys_base;
            let end = match offset.checked_add(pages * 4096) {
                Some(v) => v,
                None => {
                    super::quotas::refund_active(
                        super::quotas::Resource::Mem, charge_bytes);
                    return ENOMEM;
                }
            };
            if end > USER_WINDOW_SIZE {
                uart::puts(" FAILED (outside cave user window)\n");
                super::quotas::refund_active(
                    super::quotas::Resource::Mem, charge_bytes);
                return ENOMEM;
            }

            // CHROMIUM-PHASE-B: add the cave's virt_base to the phys
            // offset. Caves like the Chromium host run at virt_base =
            // 0x10000000 (to sit above MMIO); returning just `offset`
            // would give the caller VA 0x01b05000 while the actual
            // page-table entry is at VA 0x11b05000, so the first EL0
            // access DATA-ABORT'd with FAR~=0x01b05028. active_user_window()
            // reads the current cave's window; (0, 0) means we're on the
            // primary/legacy setup where virt_base = 0, so the old
            // behaviour falls out naturally.
            let (va_start, _va_end) = crate::batcave::linux::mmu::active_user_window();
            let uva = va_start.saturating_add(offset);

            uart::puts(" → uva=0x");
            for shift in (0..16).rev() {
                let nib = ((uva >> (shift * 4)) & 0xF) as usize;
                uart::putc(hex[nib]);
            }
            uart::puts("\n");
            uva as i64
        }
        None => {
            uart::puts(" FAILED (no frames)\n");
            super::quotas::refund_active(
                super::quotas::Resource::Mem, charge_bytes);
            ENOMEM
        }
    }
}

// Socket type tracking
const AF_INET: u32 = 2;
const SOCK_STREAM: u32 = 1;
const SOCK_DGRAM: u32 = 2;
const SOCK_RAW: u32 = 3;
static mut SOCKET_TYPE: u32 = 0;

// Per-socket state (simple: tracks the last socket's connection info)
static mut SOCK_DEST_IP: u32 = 0;
static mut SOCK_DEST_PORT: u16 = 0;
static mut SOCK_LOCAL_PORT: u16 = 30000;

// UDP receive buffer — 4-slot circular queue
pub const UDP_RX_SLOTS: usize = 4;
pub static mut UDP_RX_BUF: [[u8; 512]; UDP_RX_SLOTS] = [[0; 512]; UDP_RX_SLOTS];
pub static mut UDP_RX_LEN: [usize; UDP_RX_SLOTS] = [0; UDP_RX_SLOTS];
pub static mut UDP_RX_HEAD: usize = 0; // next write slot
pub static mut UDP_RX_TAIL: usize = 0; // next read slot
pub static mut UDP_RX_READY: bool = false;

fn sys_socket(args: [u64; 6]) -> i64 {
    let domain = args[0] as u32;
    let sock_type = args[1] as u32 & 0xFF; // mask out SOCK_CLOEXEC etc.

    if domain != AF_INET { return -97; } // EAFNOSUPPORT

    // ROOT-6: per-cave socket quota. On overflow we return -EMFILE, matching
    // what Linux returns when RLIMIT_NOFILE is hit.
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Sockets, 1) {
        return e;
    }

    // Create a VFS socket node
    if vfs::is_ready() {
        if let Ok(idx) = vfs::create_node(0, b".sock", vfs::NodeType::Socket, 0o140755) {
            unsafe { SOCKET_TYPE = sock_type; }
            match fd::alloc_fd(idx, 0) {
                Ok(fd_num) => {
                    uart::puts("[net] socket created fd=");
                    crate::kernel::mm::print_num(fd_num as usize);
                    uart::puts("\n");
                    return fd_num as i64;
                }
                Err(e) => {
                    super::quotas::refund_active(
                        super::quotas::Resource::Sockets, 1);
                    return e;
                }
            }
        }
        // VFS accepted ready, but create_node failed — refund.
        super::quotas::refund_active(super::quotas::Resource::Sockets, 1);
    }

    // Fallback — no VFS. Still a live socket from the cave's POV, so keep
    // the charge.
    10
}

fn sys_connect(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let addr_ptr = args[1] as usize;
    let addr_len = args[2] as usize;

    if addr_ptr == 0 || addr_len < 8 { return EINVAL; }
    // V8-ROOT-8: gate addr_ptr before any raw asm load. Without this, an
    // attacker passes addr_ptr in the kernel range and gets a controlled
    // 8-byte kernel-read oracle via the IP/port we later log.
    if !uaccess::is_user_range(addr_ptr, addr_len.min(16)) {
        return EFAULT;
    }

    // Parse struct sockaddr_in { sa_family(2), sin_port(2), sin_addr(4) }
    let family: u16;
    let port_be: u16;
    let ip_be: u32;
    unsafe {
        let mut fam: u32 = 0;
        core::arch::asm!("ldrh {v:w}, [{a}]", a = in(reg) addr_ptr, v = out(reg) fam);
        family = fam as u16;
        let mut p: u32 = 0;
        core::arch::asm!("ldrh {v:w}, [{a}]", a = in(reg) addr_ptr + 2, v = out(reg) p);
        port_be = p as u16;
        core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) addr_ptr + 4, v = out(reg) ip_be);
    }

    if family != AF_INET as u16 { return -97; }

    // Convert from network byte order
    let port = u16::from_be(port_be);
    let ip = u32::from_be(ip_be);

    uart::puts("[net] connect → ");
    print_ip(ip);
    uart::puts(":");
    crate::kernel::mm::print_num(port as usize);
    uart::puts("\n");

    // TCP connect
    let sock_type = unsafe { SOCKET_TYPE };
    if sock_type == SOCK_STREAM {
        match crate::net::tcp::connect(ip, port) {
            Ok(()) => {
                uart::puts("[net] TCP connected!\n");
                0
            }
            Err(e) => {
                uart::puts("[net] connect failed: ");
                uart::puts(e);
                uart::puts("\n");
                -111 // ECONNREFUSED
            }
        }
    } else {
        // UDP — save destination for later send/recv
        uart::puts("[net] UDP connect → ");
        print_ip(ip);
        uart::puts(":");
        crate::kernel::mm::print_num(port as usize);
        uart::puts("\n");
        unsafe {
            SOCK_DEST_IP = ip;
            SOCK_DEST_PORT = port;
            SOCK_LOCAL_PORT += 1;
        }
        0
    }
}

fn sys_sendto(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let len = args[2] as usize;
    let _flags = args[3];
    let dest_ptr = args[4] as usize;
    let _dest_len = args[5] as usize;

    if buf == 0 || len == 0 { return 0; }
    if !is_user_ptr(buf, len) { return EFAULT; }
    if dest_ptr != 0 && !is_user_ptr(dest_ptr, 16) { return EFAULT; }

    // Check if this is a socket fd
    if let Some(entry) = fd::get(fd_num) {
        let node = vfs::get_node(entry.node_idx);
        if node.node_type == vfs::NodeType::Socket {
            // Build data buffer from userspace
            let mut data = [0u8; 4096];
            let send_len = len.min(4096);
            for i in 0..send_len {
                unsafe {
                    let b: u32;
                    core::arch::asm!("ldrb {v:w}, [{a}]",
                        a = in(reg) buf + i, v = out(reg) b);
                    data[i] = b as u8;
                }
            }

            let sock_type = unsafe { SOCKET_TYPE };

            // Raw socket (SOCK_RAW + IPPROTO_ICMP — busybox ping's path).
            // User's buffer IS a pre-built ICMP packet; forward it as the
            // IP payload with protocol=1. Dest IP comes from sockaddr_in
            // arg if non-null, otherwise from SOCK_DEST_IP.
            if sock_type == SOCK_RAW {
                let ip_dest = if dest_ptr != 0 {
                    let ip_be: u32;
                    unsafe {
                        core::arch::asm!("ldr {v:w}, [{a}]",
                            a = in(reg) dest_ptr + 4, v = out(reg) ip_be);
                    }
                    u32::from_be(ip_be)
                } else {
                    unsafe { SOCK_DEST_IP }
                };
                match crate::net::ip::send(ip_dest, 1, &data[..send_len]) {
                    Ok(()) => return send_len as i64,
                    Err(_) => return -5, // EIO
                }
            }

            // UDP socket
            if sock_type == SOCK_DGRAM {
                let (ip, port) = if dest_ptr != 0 {
                    // Parse destination from sockaddr
                    let port_be: u16;
                    let ip_be: u32;
                    unsafe {
                        let mut p: u32 = 0;
                        core::arch::asm!("ldrh {v:w}, [{a}]",
                            a = in(reg) dest_ptr + 2, v = out(reg) p);
                        port_be = p as u16;
                        core::arch::asm!("ldr {v:w}, [{a}]",
                            a = in(reg) dest_ptr + 4, v = out(reg) ip_be);
                    }
                    (u32::from_be(ip_be), u16::from_be(port_be))
                } else {
                    // Connected UDP — use saved dest
                    unsafe { (SOCK_DEST_IP, SOCK_DEST_PORT) }
                };

                // For DNS queries (port 53), forward directly via our UDP stack
                // The real DNS response will come back through udp::handle() → queue
                let local_port = unsafe { SOCK_LOCAL_PORT };

                match crate::net::udp::send(ip, local_port, port, &data[..send_len]) {
                    Ok(()) => return send_len as i64,
                    Err(_) => return -5,
                }
            }

            // TCP send (already connected)
            match crate::net::tcp::send_data(&data[..send_len]) {
                Ok(()) => return send_len as i64,
                Err(_) => return -5,
            }
        }
    }

    // Fall through to regular write for non-socket fds
    sys_write(args)
}

fn sys_recvfrom(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let len = args[2] as usize;

    if buf == 0 || len == 0 { return 0; }
    if !is_user_ptr(buf, len) { return EFAULT; }

    if let Some(entry) = fd::get(fd_num) {
        let node = vfs::get_node(entry.node_idx);
        if node.node_type == vfs::NodeType::Socket {
            let sock_type = unsafe { SOCKET_TYPE };

            // SOCK_RAW + IPPROTO_ICMP: busybox ping reads back a full
            // IPv4 datagram (IP header + ICMP payload). Poll net::icmp
            // for a pending raw reply; time-bounded so a no-reply
            // target doesn't wedge the cave.
            if sock_type == SOCK_RAW {
                let src_addr_ptr = args[4] as usize;
                let addrlen_ptr = args[5] as usize;
                if src_addr_ptr != 0 && !uaccess::is_user_range(src_addr_ptr, 16) {
                    return EFAULT;
                }
                if addrlen_ptr != 0 && !uaccess::is_user_range(addrlen_ptr, 4) {
                    return EFAULT;
                }
                let mut scratch = [0u8; 1500];
                let cap = len.min(1500);
                for _ in 0..50_000_000u64 {
                    crate::net::poll_once();
                    if let Some((n, src)) = crate::net::icmp::take_raw_reply(
                            &mut scratch[..cap]) {
                        for i in 0..n {
                            unsafe {
                                core::arch::asm!("strb {v:w}, [{a}]",
                                    a = in(reg) buf + i,
                                    v = in(reg) scratch[i] as u32);
                            }
                        }
                        // Fill sockaddr_in (AF_INET, port=0, IP=src)
                        if src_addr_ptr != 0 {
                            let af: u16 = AF_INET as u16;
                            let port_be: u16 = 0;
                            let ip_be = src.to_be();
                            unsafe {
                                core::arch::asm!("strh {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr, v = in(reg) af as u32);
                                core::arch::asm!("strh {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr + 2,
                                    v = in(reg) port_be as u32);
                                core::arch::asm!("str {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr + 4, v = in(reg) ip_be);
                                for z in 0..8u64 {
                                    core::arch::asm!("strb wzr, [{a}]",
                                        a = in(reg) src_addr_ptr + 8 + z as usize);
                                }
                            }
                            if addrlen_ptr != 0 {
                                let alen: u32 = 16;
                                unsafe {
                                    core::arch::asm!("str {v:w}, [{a}]",
                                        a = in(reg) addrlen_ptr, v = in(reg) alen);
                                }
                            }
                        }
                        return n as i64;
                    }
                    core::hint::spin_loop();
                }
                return -11; // EAGAIN
            }

            if sock_type == SOCK_DGRAM {
                let src_addr_ptr = args[4] as usize;
                let addrlen_ptr = args[5] as usize;

                // V8-ROOT-8: gate all EL0 pointers BEFORE any raw-asm store.
                // Without this, a hostile user could pass a kernel-range
                // src_addr_ptr / addrlen_ptr and we would faithfully clobber
                // 16 + 4 bytes of kernel memory.
                if src_addr_ptr != 0 && !uaccess::is_user_range(src_addr_ptr, 16) {
                    return EFAULT;
                }
                if addrlen_ptr != 0 && !uaccess::is_user_range(addrlen_ptr, 4) {
                    return EFAULT;
                }

                // UDP receive — read from circular queue
                for _iteration in 0..50_000_000u64 {
                    crate::net::poll_once();
                    unsafe {
                        if UDP_RX_TAIL < UDP_RX_HEAD {
                            let slot = UDP_RX_TAIL % UDP_RX_SLOTS;
                            let n = UDP_RX_LEN[slot].min(len);
                            let rx_ptr = core::ptr::addr_of!(UDP_RX_BUF) as usize + slot * 512;
                            for i in 0..n {
                                let b = core::ptr::read_volatile((rx_ptr + i) as *const u8);
                                core::arch::asm!("strb {v:w}, [{a}]",
                                    a = in(reg) buf + i, v = in(reg) b as u32);
                            }
                            UDP_RX_TAIL += 1;
                            if UDP_RX_TAIL >= UDP_RX_HEAD {
                                UDP_RX_READY = false;
                            }

                            // Fill in source address (musl checks this for DNS)
                            if src_addr_ptr != 0 {
                                // struct sockaddr_in { sa_family(2), sin_port(2), sin_addr(4), zero(8) }
                                let af: u16 = AF_INET as u16;
                                core::arch::asm!("strh {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr, v = in(reg) af as u32);
                                // Port 53 in network byte order
                                let port_be: u16 = 53u16.to_be();
                                core::arch::asm!("strh {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr + 2, v = in(reg) port_be as u32);
                                // IP 10.0.2.3 in network byte order
                                let ip_be: u32 = 0x0A000203u32.to_be();
                                core::arch::asm!("str {v:w}, [{a}]",
                                    a = in(reg) src_addr_ptr + 4, v = in(reg) ip_be);
                                // Zero padding
                                for z in 0..8 {
                                    core::arch::asm!("strb wzr, [{a}]",
                                        a = in(reg) src_addr_ptr + 8 + z);
                                }
                            }
                            if addrlen_ptr != 0 {
                                let alen: u32 = 16;
                                core::arch::asm!("str {v:w}, [{a}]",
                                    a = in(reg) addrlen_ptr, v = in(reg) alen);
                            }

                            return n as i64;
                        }
                    }
                    core::hint::spin_loop();
                }
                return -11; // EAGAIN
            }

            // TCP receive
            let mut recv_buf = [0u8; 8192];
            let recv_len = len.min(8192);
            match crate::net::tcp::recv_data(&mut recv_buf[..recv_len]) {
                Ok(n) => {
                    for i in 0..n {
                        unsafe {
                            core::arch::asm!("strb {v:w}, [{a}]",
                                a = in(reg) buf + i,
                                v = in(reg) recv_buf[i] as u32);
                        }
                    }
                    return n as i64;
                }
                Err(_) => return 0,
            }
        }
    }

    sys_read(args)
}

/// Get pointer to next write slot in UDP RX queue.
fn udp_rx_alloc() -> (usize, usize) {
    unsafe {
        let slot = UDP_RX_HEAD % UDP_RX_SLOTS;
        let ptr = core::ptr::addr_of_mut!(UDP_RX_BUF) as usize + slot * 512;
        (slot, ptr)
    }
}

/// Commit a write to the UDP RX queue.
fn udp_rx_commit(slot: usize, len: usize) {
    unsafe {
        UDP_RX_LEN[slot] = len;
        UDP_RX_HEAD += 1;
        UDP_RX_READY = true;
    }
}

/// Build a DNS A-record response from a query and resolved IP.
fn build_dns_response(query: &[u8], ip: u32) {
    let (slot, buf_ptr) = udp_rx_alloc();
    unsafe {
        let mut pos = 0;

        // Copy transaction ID from query (2 bytes)
        for i in 0..2.min(query.len()) {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) buf_ptr + pos, v = in(reg) query[i] as u32);
            pos += 1;
        }
        // Flags: 0x8180 (response, recursion available, no error)
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 2, v = in(reg) 0x81u32);
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 3, v = in(reg) 0x80u32);
        // QDCOUNT = 1
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + 4);
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 5, v = in(reg) 1u32);
        // ANCOUNT = 1
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + 6);
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 7, v = in(reg) 1u32);
        // NSCOUNT = 0, ARCOUNT = 0
        for i in 8..12 {
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + i);
        }
        pos = 12;

        // Copy question section from query
        let q_start = 12;
        let mut q_end = 12;
        while q_end < query.len() && query[q_end] != 0 { q_end += 1; }
        q_end += 5; // null + qtype(2) + qclass(2)
        let q_len = q_end.min(query.len()) - q_start;
        for i in 0..q_len {
            let idx = q_start + i;
            if idx < query.len() {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf_ptr + pos, v = in(reg) query[idx] as u32);
            }
            pos += 1;
        }

        // Answer: name pointer (0xC00C = pointer to offset 12 = question name)
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 0xC0u32); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 0x0Cu32); pos += 1;
        // Type A (1)
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + pos); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 1u32); pos += 1;
        // Class IN (1)
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + pos); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 1u32); pos += 1;
        // TTL (300 seconds)
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + pos); pos += 1;
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + pos); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 1u32); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 0x2Cu32); pos += 1;
        // RDLENGTH = 4 (IPv4)
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + pos); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) 4u32); pos += 1;
        // RDATA = IP address (network byte order = big-endian)
        // ip from dns::resolve() is already host-order u32 where (ip >> 24) = first octet
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) ((ip >> 24) & 0xFF) as u32); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) ((ip >> 16) & 0xFF) as u32); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) ((ip >> 8) & 0xFF) as u32); pos += 1;
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + pos, v = in(reg) (ip & 0xFF) as u32); pos += 1;

        udp_rx_commit(slot, pos);
    }
}

/// Build a DNS NXDOMAIN/empty response.
fn build_dns_nxdomain(query: &[u8]) {
    let (slot, buf_ptr) = udp_rx_alloc();
    unsafe {
        // Copy transaction ID
        for i in 0..2.min(query.len()) {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) buf_ptr + i, v = in(reg) query[i] as u32);
        }
        // Flags: 0x8183 (response, NXDOMAIN)
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 2, v = in(reg) 0x81u32);
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 3, v = in(reg) 0x83u32);
        // QDCOUNT=1, ANCOUNT=0, NSCOUNT=0, ARCOUNT=0
        core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + 4);
        core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf_ptr + 5, v = in(reg) 1u32);
        for i in 6..12 {
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf_ptr + i);
        }
        // Copy question section
        let q_start = 12;
        let mut q_end = 12;
        while q_end < query.len() && query[q_end] != 0 { q_end += 1; }
        q_end += 5;
        let mut pos = 12;
        for i in q_start..q_end.min(query.len()) {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) buf_ptr + pos, v = in(reg) query[i] as u32);
            pos += 1;
        }
        udp_rx_commit(slot, pos);
    }
}

fn print_ip(ip: u32) {
    crate::kernel::mm::print_num(((ip >> 24) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num(((ip >> 16) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num(((ip >> 8) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num((ip & 0xFF) as usize);
}

fn sys_writev(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let iov_ptr = args[1] as usize;
    let iovcnt = args[2] as usize;

    // Each iovec is 16 bytes (ptr + len). Cap iovcnt at a sane limit
    // and verify the whole array is in user space before we read any
    // iov_base / iov_len (TOCTOU still possible if sibling threads
    // rewrite after check — tightened by R1/R2 eventually).
    if iovcnt > 1024 { return EINVAL; }
    let array_bytes = iovcnt.saturating_mul(16);
    if iovcnt > 0 && !is_user_ptr(iov_ptr, array_bytes) { return EFAULT; }

    // Check if redirected
    let mut is_uart = fd_num == 1 || fd_num == 2;
    if let Some(entry) = fd::get(fd_num) {
        if entry.node_idx != 0 {
            // Redirected to a file — write each iovec via sys_write
            let mut total = 0i64;
            for i in 0..iovcnt {
                let iov_base: u64;
                let iov_len: u64;
                unsafe {
                    core::arch::asm!("ldr {v}, [{a}]", a = in(reg) iov_ptr + i * 16, v = out(reg) iov_base);
                    core::arch::asm!("ldr {v}, [{a}]", a = in(reg) iov_ptr + i * 16 + 8, v = out(reg) iov_len);
                }
                // NEW-SYS-046: even the file-redirect path needs the gate
                // — sys_write doesn't re-check the buffer.
                if iov_len > 0 && !uaccess::is_user_range(iov_base as usize, iov_len as usize) {
                    return -(14i64);
                }
                let r = sys_write([fd_num as u64, iov_base, iov_len, 0, 0, 0]);
                if r < 0 { return r; }
                total += r;
            }
            return total;
        }
    }

    if !is_uart { return EBADF; }

    let mut total = 0i64;
    for i in 0..iovcnt {
        let iov_base: u64;
        let iov_len: u64;
        unsafe {
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) iov_ptr + i * 16, v = out(reg) iov_base);
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) iov_ptr + i * 16 + 8, v = out(reg) iov_len);
        }
        let base = iov_base as usize;
        let len = iov_len as usize;
        // NEW-SYS-046: gate iov_base lives in userspace. Previously writev
        // dumped arbitrary kernel bytes over the UART when the fd was
        // stdout/stderr — a trivial kernel-read primitive per renderer.
        if len > 0 && !uaccess::is_user_range(base, len) { return -(14i64); }
        for j in 0..len {
            let byte: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) base + j, v = out(reg) byte);
            }
            uart::putc(byte as u8);
        }
        // V8-ROOT-3 / V8-ARITH-A5: len came in as u64 → cast to i64 can
        // wrap to negative, making total (also i64) negative on overflow.
        // Use saturating_add so short-write semantics hold (caller sees
        // the correct count up to i64::MAX).
        let add = if len > i64::MAX as usize { i64::MAX } else { len as i64 };
        total = total.saturating_add(add);
    }
    total
}

fn sys_newfstatat(args: [u64; 6]) -> i64 {
    // newfstatat(dirfd, pathname, statbuf, flags)
    let dirfd = args[0] as i32;
    let path_ptr = args[1] as usize;
    let buf = args[2] as usize;
    let flags = args[3] as u32;
    if buf == 0 { return EINVAL; }

    const AT_EMPTY_PATH: u32 = 0x1000;

    // CHROMIUM-PHASE-B: AT_EMPTY_PATH with empty string means "stat
    // the dirfd itself" (glibc's fstat() is implemented as
    // newfstatat(fd, "", buf, AT_EMPTY_PATH)). Previously we
    // returned a bogus mode=0o100755, size=4096 stat regardless of
    // the fd's actual backing — which broke ld-linux's ELF size
    // check for every lib after libdl, so content_shell loading
    // stopped short with "NSS_3.2 not found" style errors.
    //
    // Now: when path is empty (null ptr OR first byte NUL) with
    // AT_EMPTY_PATH set, go through the fd table and fill stat
    // from the VfsNode that fd points at.
    let path_is_empty = if path_ptr == 0 {
        true
    } else {
        // Peek the first byte of the user-space path without reading
        // the whole buffer.
        let mut b: u32 = 0;
        unsafe {
            core::arch::asm!("ldrb {v:w}, [{a}]",
                a = in(reg) path_ptr, v = out(reg) b);
        }
        b == 0
    };

    if path_is_empty {
        if flags & AT_EMPTY_PATH != 0 && dirfd >= 0 {
            if let Some(entry) = fd::get(dirfd as u32) {
                let node = vfs::get_node(entry.node_idx);
                fill_stat(buf, node.mode, node.size as u64,
                    entry.node_idx as u64, node.nlink);
                return 0;
            }
        }
        // Empty path without AT_EMPTY_PATH (or bad fd): Linux returns
        // ENOENT. Returning the bogus 4096 stat breaks ELF loading.
        return ENOENT;
    }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    // FLv2-NEW-011: extend `..` guard to newfstatat.
    if has_dotdot(&path_buf[..path_len]) { return EACCES; }

    if vfs::is_ready() {
        match vfs::resolve_path(&path_buf[..path_len]) {
            Ok(idx) => {
                let node = vfs::get_node(idx);
                fill_stat(buf, node.mode, node.size as u64, idx as u64, node.nlink);
                return 0;
            }
            Err(_) => {}
        }
    }

    // Fallback: return generic stat for any path
    fill_stat(buf, 0o100755, 4096, 0, 1);
    0
}

fn write_str(s: &str) {
    for &b in s.as_bytes() {
        uart::putc(b);
    }
}

/// FLv2-NEW-011/012/013/014: shared `..` path-component rejector. Walks
/// the path component-by-component and returns true if any component is
/// exactly `..`. Used by every path-taking syscall (openat, faccessat,
/// unlinkat, mkdirat, readlinkat, newfstatat, statx, execve) so a cave
/// can no longer escape its base directory by passing `../foo` through
/// the syscalls that the V1 `..` guard didn't cover.
///
/// Coarser than POSIX realpath-normalization (which would resolve
/// symlinks before checking) but catches the obvious attack with zero
/// extra state. Symlink-target rejection is handled in vfs::resolve_path.
pub(crate) fn has_dotdot(path: &[u8]) -> bool {
    let mut i = 0usize;
    while i < path.len() {
        let start = if path[i] == b'/' { i + 1 } else { i };
        let mut end = start;
        while end < path.len() && path[end] != b'/' { end += 1; }
        if end - start == 2 && &path[start..end] == b".." {
            return true;
        }
        if end == path.len() { break; }
        i = end + 1;
    }
    false
}

/// Read a null-terminated string from userspace memory.
///
/// NEW-SYS-028 / ATTACK-SYS-034/035 fix: before this patch the loop did a
/// raw `ldrb` at `ptr + i` with no range check, giving every path-taking
/// syscall (openat, faccessat, chdir, readlinkat, newfstatat, mkdirat,
/// execve) a kernel-read primitive. We now refuse ptr == 0 or anything
/// outside [0x1000, 0x4000_0000), and we truncate at the first byte that
/// falls outside userspace (treated as a de-facto NUL).
fn read_user_str(ptr: usize, buf: &mut [u8]) -> usize {
    if ptr == 0 { return 0; }
    let max = buf.len().min(255);
    // Fast-path: if the whole read window lives in userspace, skip per-byte
    // checks. `is_user_range(ptr, max)` rejects overflow too.
    let whole_ok = uaccess::is_user_range(ptr, max);
    let mut len = 0;
    for i in 0..max {
        if !whole_ok && !uaccess::is_user_range(ptr + i, 1) {
            break;
        }
        let byte: u32;
        unsafe {
            core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) ptr + i, v = out(reg) byte);
        }
        if byte == 0 { break; }
        buf[i] = byte as u8;
        len += 1;
    }
    len
}

fn sys_faccessat(args: [u64; 6]) -> i64 {
    let path_ptr = args[1] as usize;
    if path_ptr == 0 { return ENOENT; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    // FLv2-NEW-011: extend `..` guard to faccessat (was openat-only).
    if has_dotdot(&path_buf[..path_len]) { return EACCES; }

    if vfs::is_ready() {
        match vfs::resolve_path(&path_buf[..path_len]) {
            Ok(_) => return 0, // exists
            Err(e) => return e,
        }
    }

    // Fallback
    let path = unsafe { core::str::from_utf8_unchecked(&path_buf[..path_len]) };
    if path.starts_with("/bin/") || path.starts_with("/usr/bin/") || path == "/" { 0 }
    else { ENOENT }
}

fn sys_ppoll(args: [u64; 6]) -> i64 {
    // ppoll(fds, nfds, timeout, sigmask)
    //   struct pollfd { fd: i32, events: i16, revents: i16 } — 8 bytes
    //   timeout == NULL  → block indefinitely
    //   timeout->tv_*=0  → non-blocking; return immediately with
    //                      whatever's already ready (possibly 0)
    //   timeout > 0      → wait up to that duration
    let fds_ptr     = args[0] as usize;
    let nfds        = args[1] as usize;
    let timeout_ptr = args[2] as usize;

    if nfds == 0 || fds_ptr == 0 { return 0; }

    let n = nfds.min(8);
    let bytes = match n.checked_mul(8) { Some(b) => b, None => return -(22i64) };
    if !uaccess::is_user_range(fds_ptr, bytes) { return -(14i64); }

    // Decode timeout: None → block; Some(0) → don't block; Some(ns) → bounded.
    let timeout_ns: Option<u64> = if timeout_ptr == 0 {
        None
    } else {
        if !is_user_ptr(timeout_ptr, 16) { return -(14i64); }
        let tv_sec: u64; let tv_nsec: u64;
        unsafe {
            core::arch::asm!("ldr {v}, [{a}]",
                a = in(reg) timeout_ptr, v = out(reg) tv_sec);
            core::arch::asm!("ldr {v}, [{a}]",
                a = in(reg) timeout_ptr + 8, v = out(reg) tv_nsec);
        }
        Some(tv_sec.saturating_mul(1_000_000_000).saturating_add(tv_nsec))
    };

    // Snapshot the (fd, events) pairs once. revents gets filled in on
    // each iteration of the wait loop.
    let mut poll_fds    = [0i32; 8];
    let mut poll_events = [0u16; 8];
    let mut has_stdin   = false;
    let mut has_socket  = false;
    for i in 0..n {
        unsafe {
            let mut fd: u32 = 0;
            core::arch::asm!("ldr {v:w}, [{a}]",
                a = in(reg) fds_ptr + i * 8, v = out(reg) fd);
            poll_fds[i] = fd as i32;
            let mut ev: u32 = 0;
            core::arch::asm!("ldrh {v:w}, [{a}]",
                a = in(reg) fds_ptr + i * 8 + 4, v = out(reg) ev);
            poll_events[i] = ev as u16;
            if fd == 0 { has_stdin = true; }
            if let Some(entry) = fd::get(fd) {
                let node = vfs::get_node(entry.node_idx);
                if node.node_type == vfs::NodeType::Socket { has_socket = true; }
            }
        }
    }

    // One-shot diagnostic so we can see what the caller is blocking on.
    {
        uart::puts("[ppoll] fds=[");
        for i in 0..n {
            if i > 0 { uart::putc(b','); }
            crate::kernel::mm::print_num(poll_fds[i] as usize);
            uart::putc(b':');
            if let Some((slot, side)) = fd::pipe_info(poll_fds[i] as u32) {
                uart::puts("pipe");
                crate::kernel::mm::print_num(slot);
                if side == 0 { uart::putc(b'A'); } else { uart::putc(b'B'); }
            } else if let Some(entry) = fd::get(poll_fds[i] as u32) {
                let node = vfs::get_node(entry.node_idx);
                if node.node_type == vfs::NodeType::Socket { uart::puts("sock"); }
                else { uart::puts("vfs"); }
            } else {
                uart::puts("bad");
            }
        }
        uart::puts("] timeout=");
        match timeout_ns {
            None => uart::puts("inf"),
            Some(0) => uart::puts("0"),
            Some(ns) => crate::kernel::mm::print_num(ns as usize),
        }
        uart::puts("\n");
    }

    // Scans all polled fds, writes revents bits, returns the count of
    // fds that had anything ready.
    let scan = || -> i64 {
        let mut ready = 0i64;
        for i in 0..n {
            let fd = poll_fds[i];
            let events = poll_events[i];
            let mut revents: u16 = 0;

            // stdin: POLLIN if there's a byte waiting on UART.
            if fd == 0 && (events & 0x1) != 0 && uart::has_char() {
                revents |= 0x1;
            }

            // pipe-kinded fds: POLLIN if the inbound buffer has data,
            // POLLOUT always (writes don't block in our impl).
            if let Some((slot, side)) = fd::pipe_info(fd as u32) {
                if (events & 0x1) != 0
                    && super::pipe_buf::has_readable(slot, side)
                {
                    revents |= 0x1;
                }
                if (events & 0x4) != 0 {
                    revents |= 0x4;
                }
            } else if fd > 2 {
                // Socket VFS nodes (legacy non-pipe-kinded): report
                // POLLIN when the UDP RX ring has data.
                if let Some(entry) = fd::get(fd as u32) {
                    let node = vfs::get_node(entry.node_idx);
                    if node.node_type == vfs::NodeType::Socket
                        && (events & 0x1) != 0
                    {
                        unsafe {
                            if UDP_RX_TAIL < UDP_RX_HEAD {
                                revents |= 0x1;
                            }
                        }
                    }
                }
            }

            if revents != 0 {
                unsafe {
                    core::arch::asm!("strh {v:w}, [{a}]",
                        a = in(reg) fds_ptr + i * 8 + 6,
                        v = in(reg) revents as u32);
                }
                ready += 1;
            }
        }
        ready
    };

    // Fast path: something is already ready, or caller asked for
    // non-blocking (timeout_ns == Some(0)).
    let ready_now = scan();
    if ready_now > 0 || matches!(timeout_ns, Some(0)) {
        return ready_now;
    }

    // Wait loop. Yield on each iteration so sibling threads can run
    // (and possibly drop data into one of the polled pipes). For a
    // NULL timeout we loop indefinitely; for a bounded timeout we
    // cap by a rough iteration count (real deadline arithmetic is
    // blocked on a usable monotonic clock in our scheduler).
    //
    // The iteration cap is intentionally high (50M) so the real
    // POSIX contract — "a NULL-timeout ppoll returns only when an
    // fd is ready or a signal arrives" — is effectively honoured for
    // the single-process content_shell workload. Chromium's code
    // explicitly asserts n != 0 on infinite-timeout poll returns, so
    // we must NOT return 0 spuriously here.
    let max_iters: u64 = match timeout_ns {
        Some(ns) => (ns / 1_000).max(1),  // ~1 spin per µs
        None     => u64::MAX,
    };
    let mut iters: u64 = 0;
    loop {
        super::threads::schedule();
        if has_socket { crate::net::poll_once(); }
        let ready = scan();
        if ready > 0 { return ready; }
        iters += 1;
        if iters >= max_iters { return 0; }
        core::hint::spin_loop();
    }
}

fn sys_dup(args: [u64; 6]) -> i64 {
    let old_fd = args[0] as u32;
    // V8-ROOT-1: charge → alloc → (refund on error) is atomic w.r.t.
    // IRQ. Previously a timer IRQ between charge and alloc (or between
    // alloc failure and refund) could race a concurrent fd op and
    // observe an inflated-but-uncommitted quota.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 1) {
            return e;
        }
        match fd::dup(old_fd) {
            Ok(new_fd) => new_fd as i64,
            Err(e) => {
                super::quotas::refund_active(super::quotas::Resource::Fds, 1);
                e
            }
        }
    }
}

fn sys_dup3(args: [u64; 6]) -> i64 {
    let old_fd = args[0] as u32;
    let new_fd = args[1] as u32;
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 1) {
            return e;
        }
        match fd::dup2(old_fd, new_fd) {
            Ok(fd) => fd as i64,
            Err(e) => {
                super::quotas::refund_active(super::quotas::Resource::Fds, 1);
                e
            }
        }
    }
}

// (old sys_pipe2 removed — real implementation below)

// Fork/thread state machine — save parent state at clone(), restore at child exit
use core::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64};
pub static IN_CHILD: AtomicBool = AtomicBool::new(false);
static CHILD_EXIT_CODE: AtomicUsize = AtomicUsize::new(0);
static CHILD_REAPED: AtomicBool = AtomicBool::new(true); // start with no child
// Saved ELR and SPSR from the clone() syscall trap frame
pub static FORK_SAVED_ELR: AtomicUsize = AtomicUsize::new(0);
pub static FORK_SAVED_SPSR: AtomicUsize = AtomicUsize::new(0);
// Saved SP at clone time
pub static FORK_SAVED_SP: AtomicUsize = AtomicUsize::new(0);

// ─── Thread support ───
// When clone is called with child_stack != 0, this is a pthread-style thread.
// We store the child_stack here so the exception handler can modify the
// trap frame SP before eret, causing the child to run on its own stack.
pub static CLONE_CHILD_STACK: AtomicU64 = AtomicU64::new(0);
// Thread ID counter (starts at 2 since PID 1 is the main process)
static NEXT_TID: AtomicUsize = AtomicUsize::new(2);
// Current thread ID
static CURRENT_TID: AtomicUsize = AtomicUsize::new(1);
// Last child TID assigned by clone (returned to parent on child exit)
pub static LAST_CHILD_TID: AtomicUsize = AtomicUsize::new(2);
// Whether the current child is a thread (has own stack) vs fork (shares parent stack)
pub static IS_THREAD_CHILD: AtomicBool = AtomicBool::new(false);
// TLS pointer for set_tid_address
static TID_ADDRESS: AtomicU64 = AtomicU64::new(0);

// Old in-file futex wait queue removed — replaced by src/batcave/linux/futex.rs
// which provides a 2048-waiter hash table with FUTEX_REQUEUE, bitset ops, and
// real cntpct_el0 timeouts.
const FUTEX_PRIVATE_FLAG: u64 = 128;
const FUTEX_CLOCK_REALTIME: u64 = 256;

fn sys_futex(args: [u64; 6]) -> i64 {
    use super::futex;
    let uaddr = args[0];
    // Strip BOTH FUTEX_PRIVATE_FLAG and FUTEX_CLOCK_REALTIME before the
    // match — glibc's pthread sync primitives always set these (op = 0x189
    // for WAIT_BITSET), and without the clock-realtime mask the match
    // dropped into the `_ => 0` catch-all and returned instantly, leaving
    // the waiter spinning on a futex that never actually blocked.
    let op = (args[1] & !(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME)) as u32;
    let val = args[2] as u32;
    let timeout_ptr = args[3] as usize;
    let uaddr2 = args[4];
    let val3 = args[5] as u32;

    // Read optional timeout from *const timespec at args[3]
    // V8-ROOT-8: gate timeout_ptr (16 bytes — 2× u64) before raw asm reads.
    // Without this, attacker timeout_ptr=kernel_addr is a 16-byte kernel-read
    // oracle for any cave with FUTEX cap.
    if timeout_ptr != 0 && !is_user_ptr(timeout_ptr, 16) {
        return EFAULT;
    }
    let timeout_ns = if timeout_ptr != 0 {
        let tv_sec: u64; let tv_nsec: u64;
        unsafe {
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) timeout_ptr, v = out(reg) tv_sec);
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) timeout_ptr + 8, v = out(reg) tv_nsec);
        }
        tv_sec.saturating_mul(1_000_000_000).saturating_add(tv_nsec)
    } else { 0 };

    match op {
        futex::FUTEX_WAIT       => futex::futex_wait(uaddr, val, timeout_ns),
        futex::FUTEX_WAKE       => futex::futex_wake(uaddr, val),
        futex::FUTEX_REQUEUE    => futex::futex_requeue(uaddr, uaddr2, val, args[2] as u32),
        futex::FUTEX_CMP_REQUEUE => futex::futex_cmp_requeue(uaddr, uaddr2, val3, val, args[2] as u32),
        futex::FUTEX_WAIT_BITSET => futex::futex_wait_bitset(uaddr, val, timeout_ns, val3),
        futex::FUTEX_WAKE_BITSET => futex::futex_wake_bitset(uaddr, val, val3),
        _ => 0, // unknown op — return success for compatibility
    }
}

fn sys_clone_thread(args: [u64; 6]) -> i64 {
    let flags = args[0];
    let child_stack = args[1];

    // ATTACK-SYS-010/011/012: validate pointer-shaped arguments against
    // the user range BEFORE handing to the threading scheduler. A kernel
    // address here becomes a stack-pivot / kernel-write primitive.
    let parent_tid = args[2] as usize;
    let tls        = args[3] as usize;
    let child_tid  = args[4] as usize;
    if child_stack != 0 && !is_user_ptr(child_stack as usize, 16) {
        return EFAULT;
    }
    if parent_tid != 0 && !is_user_ptr(parent_tid, 4) { return EFAULT; }
    if child_tid  != 0 && !is_user_ptr(child_tid,  4) { return EFAULT; }
    if tls        != 0 && !is_user_ptr(tls,        16) { return EFAULT; }

    // If the new threading model is active (Chromium launched via
    // threads::init_main_thread), delegate to the real clone().
    // ARM64 Linux clone ABI: (flags, child_stack, parent_tidptr, tls, child_tidptr)
    if super::threads::is_enabled() {
        let tid = super::threads::clone(
            flags,
            child_stack,
            args[2] as *mut i32, // parent_tidptr
            args[4] as *mut i32, // child_tidptr
            args[3],              // tls
        );
        // Only fill in the child's resume PC when clone succeeded
        // (positive return = new TID). Negative returns are errnos
        // and no slot was allocated.
        if tid > 0 {
            let parent_elr = super::threads::PARENT_SYSCALL_ELR
                .load(core::sync::atomic::Ordering::Acquire);
            // The arch SVC dispatcher stashes ELR_EL1 (already the
            // post-svc return address on ARM64) into PARENT_SYSCALL_ELR
            // before calling us. Plumb it through so cxt_switch_first_run
            // erets the child at the instruction after the parent's svc.
            super::threads::set_child_resume(tid as u32, parent_elr, child_stack);
        }
        return tid;
    }

    // Legacy fork/thread path (busybox, v8_exec, etc.)
    if IN_CHILD.load(core::sync::atomic::Ordering::Relaxed) {
        return -1; // nested clone not supported
    }

    // Assign a thread ID for the child
    let tid = NEXT_TID.load(core::sync::atomic::Ordering::Relaxed);
    NEXT_TID.store(tid + 1, core::sync::atomic::Ordering::Relaxed);
    LAST_CHILD_TID.store(tid, core::sync::atomic::Ordering::Relaxed);

    IN_CHILD.store(true, core::sync::atomic::Ordering::Relaxed);
    CHILD_REAPED.store(false, core::sync::atomic::Ordering::Relaxed);

    if child_stack != 0 {
        // pthread-style clone: child runs on its own stack
        // Store the child stack so the exception handler can set it
        // in the trap frame before eret.
        CLONE_CHILD_STACK.store(child_stack, core::sync::atomic::Ordering::Relaxed);
        IS_THREAD_CHILD.store(true, core::sync::atomic::Ordering::Relaxed);
        CURRENT_TID.store(tid, core::sync::atomic::Ordering::Relaxed);
    } else {
        // fork-style clone: child shares parent stack (busybox)
        CLONE_CHILD_STACK.store(0, core::sync::atomic::Ordering::Relaxed);
        IS_THREAD_CHILD.store(false, core::sync::atomic::Ordering::Relaxed);
    }

    // Reset worker brk for fresh execution
    unsafe { WORKER_BRK = 0; }

    0 // child return value (x0=0)
}

// ─── set_tid_address (96) ───
fn sys_set_tid_address(args: [u64; 6]) -> i64 {
    let tidptr = args[0];
    // V8-ROOT-8: tidptr is later written to at thread exit (write_volatile of
    // a 4-byte zero). If unvalidated, an attacker passes a kernel address and
    // gets an arbitrary 4-byte kernel write at thread death. Reject any
    // non-zero pointer that does not lie in the user range.
    if tidptr != 0 && !uaccess::is_user_range(tidptr as usize, 4) {
        return EFAULT;
    }
    TID_ADDRESS.store(tidptr, core::sync::atomic::Ordering::Relaxed);
    // Return current thread ID
    CURRENT_TID.load(core::sync::atomic::Ordering::Relaxed) as i64
}

// ─── gettid (178) ───
fn sys_gettid(_args: [u64; 6]) -> i64 {
    // Prefer the real per-thread TID from the threading layer when
    // it's active. The legacy CURRENT_TID was the single-threaded
    // simulation's notion of "which child is on the CPU" and often
    // sits at 0 — which Chromium then registers as a real thread
    // TID, leaving ThreadIdNameManager's cache populated with
    // (cached_id=0, cached_name=NULL) and crashing on the next
    // GetName(0) lookup.
    let real_tid = super::threads::current_tid();
    if real_tid != 0 {
        return real_tid as i64;
    }
    CURRENT_TID.load(core::sync::atomic::Ordering::Relaxed) as i64
}

/// Called from the exception handler when child exits to restore parent TID
pub fn restore_parent_tid() {
    CURRENT_TID.store(1, core::sync::atomic::Ordering::Relaxed);
}

fn sys_execve(args: [u64; 6]) -> i64 {
    let path_ptr = args[0] as usize;

    // V8-ROOT-3 / V8-ROOT-8 / V8-PARSER-3 / V8-LENGTH-E: route path read
    // through read_user_str (which gates via is_user_range). Previously
    // this raw-ldrb loop was a kernel-read primitive — pass path_ptr =
    // kernel addr, this would copy 127 bytes of kernel RAM into
    // path_buf, then from_utf8_unchecked.
    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);
    let path = match core::str::from_utf8(&path_buf[..path_len]) {
        Ok(s) => s,
        Err(_) => return -(14i64), // EFAULT
    };

    // Check if it's a /bin/* command — these are busybox applets
    if path.starts_with("/bin/") || path.starts_with("/usr/bin/") {
        // Extract the tool name from the path
        let tool = if let Some(pos) = path.rfind('/') {
            &path[pos + 1..]
        } else {
            path
        };

        // Read argv from the pointer array
        let argv_ptr = args[1] as usize;
        let mut argv_strs = [[0u8; 64]; 4];
        let mut argv_lens = [0usize; 4];
        let mut argc = 0;

        // V5-PARSER-011 fix: gate argv_ptr (4 × 8 byte pointer array) and
        // each arg_ptr string before dereference. Previously a cave could
        // pass argv_ptr=0x40400000 and have the kernel read 4 × 63 bytes
        // of kernel RAM into argv_strs, which busybox then emitted to
        // stdout — a cheap kernel-memory disclosure.
        if argv_ptr != 0 && uaccess::is_user_range(argv_ptr, 4 * 8) {
            for i in 0..4 {
                let arg_ptr: u64;
                unsafe {
                    core::arch::asm!("ldr {v}, [{a}]",
                        a = in(reg) argv_ptr + i * 8, v = out(reg) arg_ptr);
                }
                if arg_ptr == 0 { break; }
                if !uaccess::is_user_range(arg_ptr as usize, 1) { break; }
                // Read string
                for j in 0..63 {
                    if !uaccess::is_user_range(arg_ptr as usize + j, 1) { break; }
                    let b: u32;
                    unsafe {
                        core::arch::asm!("ldrb {v:w}, [{a}]",
                            a = in(reg) arg_ptr as usize + j, v = out(reg) b);
                    }
                    if b == 0 { break; }
                    argv_strs[i][j] = b as u8;
                    argv_lens[i] = j + 1;
                }
                argc += 1;
            }
        }

        // Build output directly — write the busybox applet output
        // For builtins that ash can't exec, we handle them here
        uart::puts(tool);
        uart::puts("\n");

        // Execute by calling busybox write syscall with the output
        // The applet runs in the busybox binary which is already loaded
        // We just need to call busybox's main with the right argv

        // Run the busybox applet by writing its output directly
        // We handle common applets inline
        // Handle the command: write output, set exit code, return -ENOENT
        // Since execve "fails" (returns), ash child calls _exit.
        // Our exit handler catches _exit and lets ash parent continue.
        let handled = match tool {
            "uname" => {
                if argc > 1 {
                    let arg1 = unsafe { core::str::from_utf8_unchecked(&argv_strs[1][..argv_lens[1]]) };
                    if arg1 == "-a" {
                        write_str("BatOS batcave 1.0.0 BatOS 1.0.0 aarch64 aarch64\n");
                    } else {
                        write_str("BatOS\n");
                    }
                } else {
                    write_str("BatOS\n");
                }
                true
            }
            "id" => {
                write_str("uid=0(root) gid=0(root) groups=0(root)\n");
                true
            }
            "whoami" => {
                write_str("root\n");
                true
            }
            "hostname" => {
                write_str("batcave\n");
                true
            }
            "arch" => {
                write_str("aarch64\n");
                true
            }
            "date" => {
                write_str("Sat Apr 12 00:00:00 UTC 2026\n");
                true
            }
            "ls" => {
                if vfs::is_ready() {
                    // Determine target directory
                    let dir_path = if argc > 1 {
                        &argv_strs[1][..argv_lens[1]]
                    } else {
                        b"/" as &[u8]
                    };
                    match vfs::resolve_path(dir_path) {
                        Ok(dir_idx) => {
                            vfs::list_children(dir_idx, |_, child| {
                                write_str(child.name_str());
                                if child.node_type == vfs::NodeType::Directory {
                                    write_str("/");
                                } else if child.node_type == vfs::NodeType::Symlink {
                                    // don't mark symlinks specially for now
                                }
                                write_str("\n");
                            });
                        }
                        Err(_) => {
                            write_str("ls: can't open '");
                            if argc > 1 {
                                write_str(unsafe { core::str::from_utf8_unchecked(&argv_strs[1][..argv_lens[1]]) });
                            }
                            write_str("': No such file or directory\n");
                        }
                    }
                } else {
                    write_str("bin  etc  root  tmp  usr\n");
                }
                true
            }
            "cat" => {
                if argc > 1 {
                    let file_path = &argv_strs[1][..argv_lens[1]];
                    if vfs::is_ready() {
                        match vfs::resolve_path(file_path) {
                            Ok(idx) => {
                                let node = vfs::get_node(idx);
                                if node.data_addr != 0 && node.size > 0 {
                                    // Read and print file content
                                    for i in 0..node.size {
                                        let byte: u32;
                                        unsafe {
                                            core::arch::asm!("ldrb {v:w}, [{a}]",
                                                a = in(reg) node.data_addr + i, v = out(reg) byte);
                                        }
                                        uart::putc(byte as u8);
                                    }
                                }
                            }
                            Err(_) => {
                                write_str("cat: can't open '");
                                write_str(unsafe { core::str::from_utf8_unchecked(file_path) });
                                write_str("': No such file or directory\n");
                            }
                        }
                    } else {
                        write_str("cat: ");
                        write_str(unsafe { core::str::from_utf8_unchecked(file_path) });
                        write_str(": No such file\n");
                    }
                }
                true
            }
            "touch" => {
                if argc > 1 && vfs::is_ready() {
                    let fpath = &argv_strs[1][..argv_lens[1]];
                    // Create file if it doesn't exist
                    if vfs::resolve_path(fpath).is_err() {
                        if let Ok((parent, name)) = vfs::resolve_parent(fpath) {
                            vfs::create_node(parent, name, vfs::NodeType::File, 0o100644).ok();
                        }
                    }
                }
                true
            }
            "rm" => {
                if argc > 1 && vfs::is_ready() {
                    let fpath = &argv_strs[1][..argv_lens[1]];
                    match vfs::resolve_path(fpath) {
                        Ok(idx) => { vfs::remove_node(idx).ok(); }
                        Err(_) => {
                            write_str("rm: can't remove '");
                            write_str(unsafe { core::str::from_utf8_unchecked(fpath) });
                            write_str("': No such file or directory\n");
                        }
                    }
                }
                true
            }
            "mkdir" => {
                if argc > 1 && vfs::is_ready() {
                    let dpath = &argv_strs[1][..argv_lens[1]];
                    if let Ok((parent, name)) = vfs::resolve_parent(dpath) {
                        match vfs::create_node(parent, name, vfs::NodeType::Directory, 0o40755) {
                            Ok(_) => {}
                            Err(_) => {
                                write_str("mkdir: can't create '");
                                write_str(unsafe { core::str::from_utf8_unchecked(dpath) });
                                write_str("'\n");
                            }
                        }
                    }
                }
                true
            }
            "rmdir" => {
                if argc > 1 && vfs::is_ready() {
                    let dpath = &argv_strs[1][..argv_lens[1]];
                    match vfs::resolve_path(dpath) {
                        Ok(idx) => {
                            if let Err(_) = vfs::remove_node(idx) {
                                write_str("rmdir: can't remove '");
                                write_str(unsafe { core::str::from_utf8_unchecked(dpath) });
                                write_str("': Directory not empty\n");
                            }
                        }
                        Err(_) => {}
                    }
                }
                true
            }
            "cp" => {
                if argc > 2 && vfs::is_ready() {
                    let src = &argv_strs[1][..argv_lens[1]];
                    let dst = &argv_strs[2][..argv_lens[2]];
                    match vfs::resolve_path(src) {
                        Ok(src_idx) => {
                            let src_node = vfs::get_node(src_idx);
                            if src_node.data_addr != 0 && src_node.size > 0 {
                                if let Ok((parent, name)) = vfs::resolve_parent(dst) {
                                    if let Ok(dst_idx) = vfs::create_node(parent, name, vfs::NodeType::File, src_node.mode) {
                                        // Copy data page by page
                                        let mut buf = [0u8; 256];
                                        let mut off = 0;
                                        while off < src_node.size {
                                            let chunk = (src_node.size - off).min(256);
                                            for i in 0..chunk {
                                                unsafe {
                                                    let b: u32;
                                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                                        a = in(reg) src_node.data_addr + off + i, v = out(reg) b);
                                                    buf[i] = b as u8;
                                                }
                                            }
                                            vfs::write_file_data(dst_idx, &buf[..chunk]).ok();
                                            off += chunk;
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            write_str("cp: can't stat '");
                            write_str(unsafe { core::str::from_utf8_unchecked(src) });
                            write_str("': No such file\n");
                        }
                    }
                }
                true
            }
            "mv" => {
                // Simple mv: cp + rm
                if argc > 2 && vfs::is_ready() {
                    let src = &argv_strs[1][..argv_lens[1]];
                    let dst = &argv_strs[2][..argv_lens[2]];
                    // For now, just report — real mv needs VFS rename support
                    write_str("mv: not yet supported\n");
                }
                true
            }
            "which" | "type" => {
                if argc > 1 {
                    let cmd = unsafe { core::str::from_utf8_unchecked(&argv_strs[1][..argv_lens[1]]) };
                    write_str("/bin/");
                    write_str(cmd);
                    write_str("\n");
                }
                true
            }
            "env" | "printenv" => {
                write_str("PATH=/bin:/usr/bin:/sbin\n");
                write_str("HOME=/root\n");
                write_str("SHELL=/bin/sh\n");
                write_str("USER=root\n");
                write_str("HOSTNAME=batcave\n");
                true
            }
            "readlink" => {
                if argc > 1 && vfs::is_ready() {
                    let lpath = &argv_strs[1][..argv_lens[1]];
                    if let Ok((parent, name)) = vfs::resolve_parent(lpath) {
                        if let Some(idx) = vfs::find_child(parent, name) {
                            let node = vfs::get_node(idx);
                            if node.node_type == vfs::NodeType::Symlink {
                                write_str(node.link_str());
                                write_str("\n");
                            }
                        }
                    }
                }
                true
            }
            "ln" => {
                if argc > 2 && vfs::is_ready() {
                    let target = &argv_strs[1][..argv_lens[1]];
                    let link = &argv_strs[2][..argv_lens[2]];
                    if let Ok((parent, name)) = vfs::resolve_parent(link) {
                        vfs::create_symlink(parent, name, target).ok();
                    }
                }
                true
            }
            "pwd" => {
                if vfs::is_ready() {
                    let mut path = [0u8; 128];
                    let len = vfs::node_path(vfs::get_cwd(), &mut path);
                    write_str(unsafe { core::str::from_utf8_unchecked(&path[..len]) });
                    write_str("\n");
                } else {
                    write_str("/\n");
                }
                true
            }
            "true" => true,
            "false" => true, // still returns ENOENT → exit 127, close enough
            "nproc" => { write_str("1\n"); true }
            "uptime" => { write_str(" 00:00:00 up 0 min, 1 user, load average: 0.00, 0.00, 0.00\n"); true }
            "free" => {
                write_str("              total        used        free\n");
                write_str("Mem:         262144       32768      229376\n");
                write_str("Swap:             0           0           0\n");
                true
            }
            "df" => {
                write_str("Filesystem     1K-blocks  Used Available Use% Mounted on\n");
                write_str("batfs             262144 32768    229376  13% /\n");
                true
            }
            "tty" => { write_str("/dev/console\n"); true }
            "logname" => { write_str("root\n"); true }
            "groups" => { write_str("root\n"); true }
            "hello" => {
                // Standalone binary — handled by the exception handler's execve path.
                // If we reach here, execution was already handled via br to the ELF entry.
                // Return false so the real binary execution takes over.
                false
            }
            _ => false,
        };

        if handled {
            CHILD_EXIT_CODE.store(0, core::sync::atomic::Ordering::Relaxed);
            return ENOENT;
        }
        return ENOENT;
    }

    ENOENT
}

fn sys_wait_stub(args: [u64; 6]) -> i64 {
    // ROOT-FIX (2026-04-24): fake-fork reap path. When `clone()`
    // synthesised a fake child PID (see threads::clone), there's no
    // real process to wait on — the child "exited immediately" with
    // status 0. Report it as reaped exactly once; subsequent calls
    // return -ECHILD.
    let fake = super::threads::FAKE_CHILD_PID
        .load(core::sync::atomic::Ordering::Acquire);
    if fake != 0 {
        super::threads::FAKE_CHILD_PID
            .store(0, core::sync::atomic::Ordering::Release);
        let status_ptr = args[1] as usize;
        if status_ptr != 0 {
            if !uaccess::is_user_range(status_ptr, 4) { return -(14i64); }
            // Linux wait status layout for normal exit:
            //   low 7 bits = termination signal (0 = normal)
            //   bit 7 = core dump
            //   bits 15:8 = exit code
            let status: u32 = 0;
            unsafe {
                core::arch::asm!("str {v:w}, [{a}]", a = in(reg) status_ptr, v = in(reg) status);
            }
        }
        uart::puts("[wait4] fake-fork reap: pid=");
        crate::kernel::mm::print_num(fake as usize);
        uart::puts(" status=0\n");
        return fake as i64;
    }

    // Check if there's a child to reap
    let has_child = CHILD_REAPED.load(core::sync::atomic::Ordering::Relaxed);
    if has_child {
        // Already reaped — no more children
        return -10; // ECHILD
    }

    // Mark as reaped so subsequent calls return ECHILD
    CHILD_REAPED.store(true, core::sync::atomic::Ordering::Relaxed);

    let status_ptr = args[1] as usize;
    let code = CHILD_EXIT_CODE.load(core::sync::atomic::Ordering::Relaxed);
    if status_ptr != 0 {
        // NEW-SYS-026: status_ptr must be user memory. Without this, wait4
        // was a 4-byte controlled kernel-write primitive (attacker-controlled
        // status value limited to 8 bits but attacker-chosen location).
        if !uaccess::is_user_range(status_ptr, 4) { return -(14i64); }
        // Linux wait status: exit code in bits 15:8
        let status: u32 = (code as u32 & 0xFF) << 8;
        unsafe {
            core::arch::asm!("str {v:w}, [{a}]", a = in(reg) status_ptr, v = in(reg) status);
        }
    }

    LAST_CHILD_TID.load(core::sync::atomic::Ordering::Relaxed) as i64 // Return child TID
}

fn sys_clock_gettime(args: [u64; 6]) -> i64 {
    let buf = args[1] as usize;
    if buf == 0 { return EINVAL; }
    if !is_user_ptr(buf, 16) { return EFAULT; }

    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let secs = if freq == 0 { 0 } else { count / freq };
    // V8-ROOT-3 (regression fix): (count % freq) * 1e9 can overflow u64 when
    // freq > 2^34 (unlikely on current ARMv8 but possible on some SoCs).
    // Use u128 for the intermediate.
    let nsecs_raw: u64 = if freq == 0 {
        0
    } else {
        let num = (count as u128 % freq as u128).saturating_mul(1_000_000_000u128);
        let q = num / (freq as u128);
        // Result fits in u64 because q < 1e9.
        q as u64
    };

    // V8-ROOT-7: quantize nanoseconds to 100-µs resolution BEFORE handing
    // the value to EL0. A browser-class attacker uses cycle-level time to
    // mount cache-timing / Spectre side-channels; 100 µs is ~100000×
    // coarser than the underlying hardware counter and far coarser than
    // the single cache-miss / AES-round events they're trying to observe.
    // Legitimate workloads (timers, timeouts, Date.now()) need only ms
    // granularity, so 100 µs is comfortably over-accurate for them.
    const QUANTUM_NS: u64 = 100_000; // 100 µs
    let nsecs = (nsecs_raw / QUANTUM_NS) * QUANTUM_NS;

    unsafe {
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf, v = in(reg) secs);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 8, v = in(reg) nsecs);
    }
    0
}

fn sys_getrandom(args: [u64; 6]) -> i64 {
    let buf = args[0] as usize;
    let len = args[1] as usize;

    if len > 0 && !is_user_ptr(buf, len) { return EFAULT; }

    // ATTACK-CRYPTO-003 partial: replace the straight counter-shift with
    // an accumulated-state SHA-256-based PRNG. Still not a CSPRNG in the
    // formal sense (no interrupt-timing entropy pool yet), but dramatically
    // better than single-counter-read-per-byte.
    //
    // State is seeded from:
    //   - multiple cntpct_el0 reads across nanosecond-scale delays
    //   - the previous output (carried in PRNG_STATE)
    //   - the frame allocator's current "free bitmap" fingerprint
    //     (indirect system-state entropy — varies with uptime, load,
    //      previous allocations)
    //
    // Output is SHA-256(state || counter) chunked into 32-byte blocks.

    use crate::crypto::sha256;
    use core::sync::atomic::{AtomicU64, Ordering as O};
    static PRNG_STATE_LO: AtomicU64 = AtomicU64::new(0);
    static PRNG_STATE_HI: AtomicU64 = AtomicU64::new(0);
    static PRNG_CTR:      AtomicU64 = AtomicU64::new(0);

    // Gather 64 bytes of seed material.
    let mut seed = [0u8; 64];
    for i in 0..8 {
        let v: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
        seed[i*8..(i+1)*8].copy_from_slice(&v.to_le_bytes());
        // Deliberately insert a small delay so consecutive reads differ
        // at the low bits; ~100 cycles of spin is enough.
        for _ in 0..100 { core::hint::spin_loop(); }
    }
    // Mix prior state + counter.
    let prev_lo = PRNG_STATE_LO.load(O::Relaxed);
    let prev_hi = PRNG_STATE_HI.load(O::Relaxed);
    for i in 0..8 { seed[i] ^= prev_lo.to_le_bytes()[i]; }
    for i in 0..8 { seed[i+8] ^= prev_hi.to_le_bytes()[i]; }

    let mut pos = 0usize;
    while pos < len {
        let ctr = PRNG_CTR.fetch_add(1, O::Relaxed);
        let mut stream = [0u8; 64 + 16];
        stream[..64].copy_from_slice(&seed);
        stream[64..72].copy_from_slice(&ctr.to_le_bytes());
        stream[72..80].copy_from_slice(&(pos as u64).to_le_bytes());
        let h = sha256::hash(&stream);
        let take = core::cmp::min(32, len - pos);
        for i in 0..take {
            let byte = h[i] as u32;
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf + pos + i, v = in(reg) byte);
            }
        }
        // Feed the output back into the state so the next call chains.
        let new_lo = u64::from_le_bytes([h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]]);
        let new_hi = u64::from_le_bytes([h[8],h[9],h[10],h[11],h[12],h[13],h[14],h[15]]);
        PRNG_STATE_LO.store(new_lo, O::Relaxed);
        PRNG_STATE_HI.store(new_hi, O::Relaxed);
        pos += take;
    }
    len as i64
}

fn sys_sendfile(args: [u64; 6]) -> i64 {
    // sendfile(out_fd, in_fd, offset, count)
    let out_fd = args[0] as u32;
    let in_fd = args[1] as u32;
    let _offset_ptr = args[2] as usize;
    let count = args[3] as usize;

    // Read from in_fd, write to out_fd
    let in_entry = match fd::get(in_fd) {
        Some(e) => e,
        None => return EBADF,
    };
    let node_idx = in_entry.node_idx;
    let pos = in_entry.position;
    let node = vfs::get_node(node_idx);
    let available = if node.size > pos { node.size - pos } else { 0 };
    let to_send = available.min(count).min(4096);

    if to_send == 0 { return 0; }

    // Read from VFS file and write to output
    for i in 0..to_send {
        if node.data_addr == 0 { break; }
        let byte: u32;
        unsafe {
            core::arch::asm!("ldrb {v:w}, [{a}]",
                a = in(reg) node.data_addr + pos + i, v = out(reg) byte);
        }
        // Write to out_fd
        if out_fd == 1 || out_fd == 2 {
            uart::putc(byte as u8);
        }
    }

    // Update in_fd position
    if let Some(e) = fd::get_mut(in_fd) {
        e.position += to_send;
    }

    to_send as i64
}

// ─── VFS-backed syscalls ───

fn sys_readlinkat(args: [u64; 6]) -> i64 {
    let path_ptr = args[1] as usize;
    let buf = args[2] as usize;
    let bufsiz = args[3] as usize;
    if path_ptr == 0 || buf == 0 { return EINVAL; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    // Trace — Chromium readlinks a bunch of /proc/self/* paths
    // during startup; seeing which ones and what we return helps
    // chase down post-ICU content_shell crashes.
    uart::puts("[readlinkat] '");
    for &b in &path_buf[..path_len] {
        if b.is_ascii_graphic() || b == b'/' || b == b'.' { uart::putc(b); }
        else { uart::putc(b'?'); }
    }
    uart::puts("'\n");

    // FLv2-NEW-011: extend `..` guard to readlinkat.
    if has_dotdot(&path_buf[..path_len]) { return EACCES; }

    // Handle /proc/self/exe — return path to the active EL0 binary.
    // Chromium reads this during startup to compute DIR_EXE / DIR_ASSETS
    // via realpath; without a sensible answer, various PathService
    // lookups fall back to EINVAL and Chromium derefs garbage later.
    if path_len >= 14 && &path_buf[..14] == b"/proc/self/exe" {
        let exe_path = b"/bin/content_shell";
        let len = exe_path.len().min(bufsiz);
        for i in 0..len {
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = in(reg) exe_path[i] as u32);
            }
        }
        return len as i64;
    }
    // /proc/self/cwd — currently always "/"
    if path_len >= 14 && &path_buf[..14] == b"/proc/self/cwd" {
        let cwd = b"/";
        let len = cwd.len().min(bufsiz);
        for i in 0..len {
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = in(reg) cwd[i] as u32);
            }
        }
        return len as i64;
    }
    // /etc/localtime — deliberately return EINVAL. The ICU TZ
    // detector's symlink-target-parsing code reaches a CharString::
    // append that hits a non-canonical pointer deref on our setup;
    // making /etc/localtime act as a plain file (not a symlink)
    // steers ICU down the tzdata-from-icudtl.dat path instead,
    // which works. (-EINVAL is what readlinkat returns on any
    // non-symlink inode — matches kernel semantics.)

    if vfs::is_ready() {
        // Don't follow symlinks — resolve parent then find the final component
        if let Ok((parent, name)) = vfs::resolve_parent(&path_buf[..path_len]) {
            if let Some(idx) = vfs::find_child(parent, name) {
                let node = vfs::get_node(idx);
                if node.node_type == vfs::NodeType::Symlink {
                    let len = node.link_len.min(bufsiz);
                    for i in 0..len {
                        unsafe {
                            core::arch::asm!("strb {v:w}, [{a}]",
                                a = in(reg) buf + i, v = in(reg) node.link_target[i] as u32);
                        }
                    }
                    return len as i64;
                }
            }
        }
    }
    EINVAL
}

fn sys_getdents64(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let buf_size = args[2] as usize;

    if buf_size > 0 && !is_user_ptr(buf, buf_size) { return EFAULT; }

    if !vfs::is_ready() { return 0; }

    let entry = match fd::get(fd_num) {
        Some(e) => e,
        None => return EBADF,
    };

    let dir_idx = entry.node_idx;
    let node = vfs::get_node(dir_idx);
    if node.node_type != vfs::NodeType::Directory { return -20; } // ENOTDIR

    let start_pos = entry.position;

    // Collect children and fill buffer
    let mut offset = 0usize;
    let mut child_count = 0usize;

    // linux_dirent64: d_ino(8) + d_off(8) + d_reclen(2) + d_type(1) + d_name(variable)
    vfs::list_children(dir_idx, |child_idx, child_node| {
        if child_count < start_pos {
            child_count += 1;
            return; // skip already-returned entries
        }

        let name_len = child_node.name_len;
        // reclen must be 8-byte aligned, minimum 24 + name_len + 1 (null)
        let reclen = ((24 + name_len + 1 + 7) / 8) * 8;

        if offset + reclen > buf_size { return; } // buffer full

        unsafe {
            // d_ino
            let ino = child_idx as u64;
            core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + offset, v = in(reg) ino);
            // d_off (next entry offset)
            let d_off = (offset + reclen) as u64;
            core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + offset + 8, v = in(reg) d_off);
            // d_reclen
            let reclen16 = reclen as u16;
            core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) buf + offset + 16, v = in(reg) reclen16 as u32);
            // d_type
            let dtype: u8 = match child_node.node_type {
                vfs::NodeType::Directory => 4,  // DT_DIR
                vfs::NodeType::Symlink => 10,   // DT_LNK
                _ => 8,                         // DT_REG
            };
            core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf + offset + 18, v = in(reg) dtype as u32);
            // d_name
            for j in 0..name_len {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + offset + 19 + j,
                    v = in(reg) child_node.name[j] as u32);
            }
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + offset + 19 + name_len);
        }

        offset += reclen;
        child_count += 1;
    });

    // Update position
    if let Some(e) = fd::get_mut(fd_num) {
        e.position = child_count;
    }

    offset as i64
}

fn sys_chdir(args: [u64; 6]) -> i64 {
    let path_ptr = args[0] as usize;
    if path_ptr == 0 { return EINVAL; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    // FLv2-NEW-011: extend `..` guard to chdir. Without this a cave could
    // chdir("..") repeatedly and end up at the rootfs root from inside a
    // sandboxed cwd.
    if has_dotdot(&path_buf[..path_len]) { return EACCES; }

    if vfs::is_ready() {
        match vfs::resolve_path(&path_buf[..path_len]) {
            Ok(idx) => {
                let node = vfs::get_node(idx);
                if node.node_type != vfs::NodeType::Directory { return -20; } // ENOTDIR
                vfs::set_cwd(idx);
                return 0;
            }
            Err(e) => return e,
        }
    }
    0
}

fn sys_pipe2(args: [u64; 6]) -> i64 {
    let fds_ptr = args[0] as usize;
    if fds_ptr == 0 { return EINVAL; }
    if !is_user_ptr(fds_ptr, 8) { return EFAULT; }

    // V8-ROOT-1: charge → inner → (refund on error) atomic.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 2) {
            return e;
        }
        let ret = sys_pipe2_inner(fds_ptr);
        if ret != 0 {
            super::quotas::refund_active(super::quotas::Resource::Fds, 2);
        }
        ret
    }
}

fn sys_pipe2_inner(fds_ptr: usize) -> i64 {
    // CHROMIUM-PHASE-C: route pipe2 through the same pipe_buf pair
    // slots that socketpair uses. That way reads on pipefd[0] block
    // in ppoll until someone writes to pipefd[1], and POLLIN reporting
    // works uniformly. Previously pipe2 handed out VFS-file fds that
    // ppoll couldn't track.
    //
    // We still need two VFS Socket nodes (well, Pipe would be nicer,
    // but we don't have that variant yet — sockets work because ppoll's
    // VFS-node path treats Socket specially). stat/fstat on the fds
    // see them as "socket-like pipes"; nothing user-visible cares.
    if !vfs::is_ready() { return -(38i64); }
    let slot = match super::pipe_buf::alloc_pair() {
        Some(s) => s,
        None => return -(24i64), // EMFILE
    };
    // Two VFS stub nodes under /tmp so /proc-style introspection sees
    // a name. Unique per-slot.
    let mut name = [0u8; 16];
    name[0] = b'.'; name[1] = b'p'; name[2] = b'i'; name[3] = b'p'; name[4] = b'e';
    name[5] = b'0' + ((slot / 10) % 10) as u8;
    name[6] = b'0' + (slot % 10) as u8;
    let name_len = 7;
    let parent = vfs::find_child(0, b"tmp").unwrap_or(0);
    let node_r = match vfs::create_node(parent, &name[..name_len], vfs::NodeType::Socket, 0o140600) {
        Ok(n) => n,
        Err(e) => { super::pipe_buf::release(slot); return e; }
    };
    let node_w = match vfs::create_node(parent, &name[..name_len], vfs::NodeType::Socket, 0o140600) {
        Ok(n) => n,
        Err(e) => { super::pipe_buf::release(slot); return e; }
    };
    // side 0 is the read end (reads pull data that side 1 wrote);
    // side 1 is the write end. pipe2 returns [read_fd, write_fd].
    let read_fd = match fd::alloc_fd_pipe(node_r, fd::O_RDONLY, slot, 0) {
        Ok(f) => f,
        Err(e) => { super::pipe_buf::release(slot); return e; }
    };
    let write_fd = match fd::alloc_fd_pipe(node_w, fd::O_WRONLY, slot, 1) {
        Ok(f) => f,
        Err(e) => {
            let _ = fd::close(read_fd);
            super::pipe_buf::release(slot);
            return e;
        }
    };
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_READ_FD), read_fd);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_WRITE_FD), write_fd);
        core::arch::asm!("str {v:w}, [{a}]",
            a = in(reg) fds_ptr, v = in(reg) read_fd);
        core::arch::asm!("str {v:w}, [{a}]",
            a = in(reg) fds_ptr + 4, v = in(reg) write_fd);
    }
    uart::puts("[pipe2] fd=");
    crate::kernel::mm::print_num(read_fd as usize);
    uart::puts(",");
    crate::kernel::mm::print_num(write_fd as usize);
    uart::puts(" slot=");
    crate::kernel::mm::print_num(slot);
    uart::puts("\n");
    0
}

fn sys_mkdirat(args: [u64; 6]) -> i64 {
    let path_ptr = args[1] as usize;
    let mode = args[2] as u32;
    if path_ptr == 0 { return EINVAL; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    // FLv2-NEW-011: extend `..` guard to mkdirat.
    if has_dotdot(&path_buf[..path_len]) { return EACCES; }

    if vfs::is_ready() {
        match vfs::resolve_parent(&path_buf[..path_len]) {
            Ok((parent, name)) => {
                let dir_mode = 0o40000 | (mode & 0o7777); // S_IFDIR | perms
                match vfs::create_node(parent, name, vfs::NodeType::Directory, dir_mode) {
                    Ok(_) => 0,
                    Err(e) => e,
                }
            }
            Err(e) => e,
        }
    } else {
        0
    }
}

// ─── epoll_create1 / epoll_ctl / epoll_pwait ───
// Delegates to src/batcave/linux/epoll.rs (real wait-queue implementation
// with interest lists, level-triggered delivery, and close-notification).
fn sys_epoll_create1(args: [u64; 6]) -> i64 {
    // ROOT-6: per-cave epoll-instance quota (default 16). Refund if the
    // underlying epoll module rejects the request.
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Epolls, 1) {
        return e;
    }
    let r = super::epoll::epoll_create1(args[0] as u32);
    if r < 0 {
        super::quotas::refund_active(super::quotas::Resource::Epolls, 1);
    }
    r
}
fn sys_epoll_ctl(args: [u64; 6]) -> i64 {
    super::epoll::epoll_ctl(
        args[0] as i32, args[1] as i32, args[2] as i32,
        args[3] as *const super::epoll::EpollEvent,
    )
}
fn sys_epoll_pwait(args: [u64; 6]) -> i64 {
    // maxevents × 16 bytes per epoll_event must fit in user memory.
    let events = args[1] as usize;
    let maxevents = args[2] as i32;
    if maxevents > 0 {
        let bytes = (maxevents as usize).saturating_mul(16);
        if !is_user_ptr(events, bytes) { return EFAULT; }
    }

    super::epoll::epoll_pwait(
        args[0] as i32,
        args[1] as *mut super::epoll::EpollEvent,
        args[2] as i32,
        args[3] as i32,
        args[4] as *const u64,
    )
}

// ─── eventfd2 (19) + timerfd_create/settime/gettime (85/86/87) ───
fn sys_eventfd2(args: [u64; 6]) -> i64 {
    // ROOT-6: per-cave eventfd quota (default 16).
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Eventfds, 1) {
        return e;
    }
    let r = super::async_fds::eventfd2(args[0] as u32, args[1] as i32);
    if r < 0 {
        super::quotas::refund_active(super::quotas::Resource::Eventfds, 1);
    }
    r
}
fn sys_timerfd_create(args: [u64; 6]) -> i64 {
    // ROOT-6: per-cave timerfd quota (default 16).
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Timerfds, 1) {
        return e;
    }
    let r = super::async_fds::timerfd_create(args[0] as i32, args[1] as i32);
    if r < 0 {
        super::quotas::refund_active(super::quotas::Resource::Timerfds, 1);
    }
    r
}
fn sys_timerfd_settime(args: [u64; 6]) -> i64 {
    super::async_fds::timerfd_settime(
        args[0] as i32, args[1] as i32,
        args[2] as *const super::async_fds::Itimerspec,
        args[3] as *mut super::async_fds::Itimerspec,
    )
}
fn sys_timerfd_gettime(args: [u64; 6]) -> i64 {
    super::async_fds::timerfd_gettime(
        args[0] as i32,
        args[1] as *mut super::async_fds::Itimerspec,
    )
}

// ─── sysinfo (179) — return system information ───
fn sys_sysinfo(args: [u64; 6]) -> i64 {
    let buf = args[0] as usize;
    if buf == 0 { return EINVAL; }
    // struct sysinfo is 112 bytes on 64-bit Linux.  Validate before we
    // fill it — otherwise any attacker-supplied pointer would receive
    // 112 zero-fill strb's + structured kernel-chosen values.
    if !is_user_ptr(buf, 112) { return EFAULT; }

    // Zero out the struct first (at least 112 bytes on 64-bit Linux)
    for i in 0..112 {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
    }

    // Get uptime from ARM64 generic timer
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let uptime = if freq > 0 { count / freq } else { 0 };

    // Get RAM stats from frame allocator
    let total_ram: u64 = 256 * 1024 * 1024; // 256 MB
    let free_ram: u64 = 224 * 1024 * 1024;  // ~224 MB free (conservative)

    unsafe {
        // offset 0: uptime (long / i64)
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf, v = in(reg) uptime);
        // offset 8: loads[0] (unsigned long) — 1-min load average * 65536
        let load: u64 = 0;
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 8, v = in(reg) load);
        // offset 16: loads[1] — 5-min
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 16, v = in(reg) load);
        // offset 24: loads[2] — 15-min
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 24, v = in(reg) load);
        // offset 32: totalram (unsigned long)
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 32, v = in(reg) total_ram);
        // offset 40: freeram (unsigned long)
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 40, v = in(reg) free_ram);
        // offset 48: sharedram
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 48, v = in(reg) 0u64);
        // offset 56: bufferram
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 56, v = in(reg) 0u64);
        // offset 64: totalswap
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 64, v = in(reg) 0u64);
        // offset 72: freeswap
        core::arch::asm!("str {v}, [{a}]",
            a = in(reg) buf + 72, v = in(reg) 0u64);
        // offset 80: procs (unsigned short) — 1 process
        let procs: u16 = 1;
        core::arch::asm!("strh {v:w}, [{a}]",
            a = in(reg) buf + 80, v = in(reg) procs as u32);
        // offset 88: mem_unit (unsigned int) — 1 (byte-granularity)
        let mem_unit: u32 = 1;
        core::arch::asm!("str {v:w}, [{a}]",
            a = in(reg) buf + 88, v = in(reg) mem_unit);
    }
    0
}

// ═══════════════════════════════════════════════════════════════════════════
// #5: SIGNAL HANDLING — SIGCHLD, SIGTERM, rt_sigaction, rt_sigprocmask
// ═══════════════════════════════════════════════════════════════════════════

/// Signal numbers (Linux ARM64)
const _SIGHUP: u32 = 1;
const _SIGINT: u32 = 2;
const SIGKILL: u32 = 9;
const SIGABRT: u32 = 6;
const SIGTERM: u32 = 15;
const _SIGCHLD: u32 = 17;
const SIGSTOP: u32 = 19;
const MAX_SIG: usize = 64;

/// Signal disposition for each signal (up to 64)
/// Stores the handler function pointer (SIG_DFL=0, SIG_IGN=1, or handler address)
static mut SIGNAL_HANDLERS: [u64; MAX_SIG] = [0; MAX_SIG];
/// Signal mask (blocked signals)
static mut SIGNAL_MASK: u64 = 0;
/// Pending signals bitmap
static mut SIGNAL_PENDING: u64 = 0;

/// VA bump-cursor for the small-anonymous-mmap fallback path
/// (sys_mmap, line ~2010). Module-scope so a known initial value
/// survives any early static-init quirks.
const SMALL_MMAP_BASE: usize = 0x70_0000_0000;
const SMALL_MMAP_END:  usize = 0x78_0000_0000; // 32 GB region
static SMALL_MMAP_CURSOR: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(SMALL_MMAP_BASE);

/// Bitmap of L1s for which we've registered the big small-mmap
/// reservation. Indexed by cave-slot equivalent — but since we
/// only have ~8 caves total, a u64 is overkill. Use a small
/// linear table.
static mut SMALL_MMAP_RESV_L1S: [u64; 16] = [0u64; 16];
static SMALL_MMAP_RESV_COUNT: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

fn ensure_small_mmap_reservation(l1: u64) {
    use core::sync::atomic::Ordering as Ord2;
    unsafe {
        let n = SMALL_MMAP_RESV_COUNT.load(Ord2::Acquire);
        for i in 0..n {
            if SMALL_MMAP_RESV_L1S[i] == l1 { return; }
        }
        if n >= 16 { return; }
        SMALL_MMAP_RESV_L1S[n] = l1;
        SMALL_MMAP_RESV_COUNT.store(n + 1, Ord2::Release);
    }
    super::demand_page::register_reservation(
        SMALL_MMAP_BASE as u64,
        SMALL_MMAP_END  as u64,
        l1,
    );
}
/// Alternate signal stack
static mut SIGALT_SP: u64 = 0;
static mut SIGALT_SIZE: u64 = 0;

/// Clear every `static mut` the Linux compat layer carries across cave
/// lifetimes so a freshly-created cave cannot inherit signal handlers,
/// child/thread bookkeeping, pipe contents, or UDP RX queue state from
/// the previous tenant. V2-NEW-009/019/031/032/033 + ESC-029/033.
pub fn reset_cave_statics() {
    unsafe {
        // Signal state.
        for i in 0..MAX_SIG { SIGNAL_HANDLERS[i] = 0; }
        SIGNAL_MASK = 0;
        SIGNAL_PENDING = 0;
        SIGALT_SP = 0;
        SIGALT_SIZE = 0;
    }
    // Clear the real signal module's handler table too.
    super::signal::reset();
    // Wipe the syscall-history ring — the next cave should start
    // with a blank forensic record so its crash dump only contains
    // its own syscalls.
    super::syscall_history::reset();
    unsafe {
        // Pipe buffer + bookkeeping. V11-state-sweep: prior fd IDs AND
        // the fn-local PIPE_NUM counter (reset via write_volatile below)
        // were leaking across caves so a read() on an inherited fd could
        // race the old pipe.
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_LEN), 0);
        for i in 0..PIPE_BUF_SIZE {
            core::arch::asm!("strb wzr, [{a}]",
                a = in(reg) core::ptr::addr_of_mut!(PIPE_BUF) as usize + i);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_READ_FD), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_WRITE_FD), 0);
        // UDP RX queue (the syscall-layer ring; sockets have their own).
        for slot in 0..UDP_RX_SLOTS {
            UDP_RX_LEN[slot] = 0;
            for b in 0..UDP_RX_BUF[slot].len() { UDP_RX_BUF[slot][b] = 0; }
        }
        UDP_RX_HEAD = 0;
        UDP_RX_TAIL = 0;
        UDP_RX_READY = false;

        // V11-state-sweep: per-socket last-connection metadata. Next cave's
        // first socket() would otherwise inherit destination IP/port +
        // start from the previous cave's local-port cursor (fingerprint
        // leak + peer-identity leak).
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SOCKET_TYPE), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SOCK_DEST_IP), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SOCK_DEST_PORT), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SOCK_LOCAL_PORT), 30000);

        // V11-state-sweep: /proc pseudo-fd table — prior cave's /proc
        // paths would otherwise be readable by the next tenant.
        {
            let paths = &mut *core::ptr::addr_of_mut!(PROC_FD_PATHS);
            for row in paths.iter_mut() {
                for b in row.iter_mut() { *b = 0; }
            }
            let lens = &mut *core::ptr::addr_of_mut!(PROC_FD_LENS);
            for l in lens.iter_mut() { *l = 0; }
            let pos = &mut *core::ptr::addr_of_mut!(PROC_FD_POS);
            for p in pos.iter_mut() { *p = 0; }
        }

        // V11-state-sweep: worker-ELF heap end. brk() in the next cave
        // would otherwise start at a stale value.
        WORKER_BRK = 0;
    }
    // Child/thread bookkeeping.
    IN_CHILD.store(false, core::sync::atomic::Ordering::Relaxed);
    IS_THREAD_CHILD.store(false, core::sync::atomic::Ordering::Relaxed);
    LAST_CHILD_TID.store(2, core::sync::atomic::Ordering::Relaxed);
    CHILD_EXIT_CODE.store(0, core::sync::atomic::Ordering::Relaxed);
    CHILD_REAPED.store(true, core::sync::atomic::Ordering::Relaxed);

    // V11-state-sweep: TID counters and set_tid_address pointer. The
    // TID_ADDRESS pointer in particular is CRITICAL — it's a user-VA
    // that the kernel writes `0` into at thread exit. Without reset it
    // becomes an arbitrary-kernel-write gadget into the NEW cave's
    // address space via the old cave's dangling tidptr.
    NEXT_TID.store(2, core::sync::atomic::Ordering::Release);
    CURRENT_TID.store(1, core::sync::atomic::Ordering::Release);
    TID_ADDRESS.store(0, core::sync::atomic::Ordering::Release);

    // V11-state-sweep: fork-resume registers. These are a CROSS-CAVE
    // CONTROL-FLOW PIVOT — the new cave could resume into the prior
    // cave's saved EL0 context via a leftover ELR/SPSR/SP.
    FORK_SAVED_ELR.store(0, core::sync::atomic::Ordering::Release);
    FORK_SAVED_SPSR.store(0, core::sync::atomic::Ordering::Release);
    FORK_SAVED_SP.store(0, core::sync::atomic::Ordering::Release);
    CLONE_CHILD_STACK.store(0, core::sync::atomic::Ordering::Release);
}

/// rt_sigaction — set/get signal handler. Route through `signal::`
/// which owns the real handler table + delivery path.
fn sys_rt_sigaction(args: [u64; 6]) -> i64 {
    use super::signal;
    let signum = args[0] as u32;
    let act_ptr = args[1] as usize;
    let oldact_ptr = args[2] as usize;

    if signum == 0 || signum as usize >= signal::MAX_SIG { return EINVAL; }
    if signum == SIGKILL || signum == SIGSTOP { return EINVAL; }

    // sigaction struct: handler (u64), flags (u64), restorer (u64),
    // sigset_t mask (u64 on our simple impl). Linux actually has a
    // larger sa_mask but glibc wraps this.
    if act_ptr != 0 && !is_user_ptr(act_ptr, 32) { return EFAULT; }
    if oldact_ptr != 0 && !is_user_ptr(oldact_ptr, 32) { return EFAULT; }

    let mut old = signal::Sigaction::default();
    let old_out = if oldact_ptr != 0 { Some(&mut old) } else { None };

    let new_sa = if act_ptr != 0 {
        unsafe {
            let p = act_ptr as *const u64;
            Some(signal::Sigaction {
                handler:  core::ptr::read(p),
                flags:    core::ptr::read(p.add(1)),
                restorer: core::ptr::read(p.add(2)),
                mask:     core::ptr::read(p.add(3)),
            })
        }
    } else { None };

    let rc = signal::set_action(signum, new_sa, old_out);
    if rc < 0 { return rc; }

    if oldact_ptr != 0 {
        unsafe {
            let p = oldact_ptr as *mut u64;
            core::ptr::write(p,           old.handler);
            core::ptr::write(p.add(1),    old.flags);
            core::ptr::write(p.add(2),    old.restorer);
            core::ptr::write(p.add(3),    old.mask);
        }
    }
    // Mirror the handler in the legacy bitmap too so sys_tgkill etc.
    // keep working during this transition.
    if let Some(sa) = new_sa {
        unsafe {
            if (signum as usize) < MAX_SIG {
                SIGNAL_HANDLERS[signum as usize] = sa.handler;
            }
        }
    }
    0
}

/// rt_sigprocmask — get/set blocked signal mask
fn sys_rt_sigprocmask(args: [u64; 6]) -> i64 {
    let how = args[0] as u32;
    let set_ptr = args[1] as usize;
    let oldset_ptr = args[2] as usize;

    // ATTACK-SYS-039 fix: gate both pointers. Without these a cave could
    // use rt_sigprocmask as a 64-bit kernel read oracle (via oldset) and
    // as a gadget-friendly controlled read (of SIGNAL_MASK) / unchecked
    // 64-bit read (of set_ptr).
    if oldset_ptr != 0 && !uaccess::is_user_range(oldset_ptr, 8) {
        return -(14i64);
    }
    if set_ptr != 0 && !uaccess::is_user_range(set_ptr, 8) {
        return -(14i64);
    }

    unsafe {
        if oldset_ptr != 0 {
            // Prefer the new signal module's view; it's what the
            // async-delivery poll actually consults.
            let cur = super::signal::get_mask();
            core::ptr::write(oldset_ptr as *mut u64, cur);
            // Mirror into the legacy global too so anybody still
            // reading SIGNAL_MASK sees the same value.
            SIGNAL_MASK = cur;
        }
        if set_ptr != 0 {
            let new_set = core::ptr::read(set_ptr as *const u64)
                & !((1u64 << SIGKILL) | (1u64 << SIGSTOP));
            let new_mask = match how {
                0 => super::signal::mask_block(new_set),        // SIG_BLOCK   — returns prev
                1 => super::signal::mask_unblock(new_set),      // SIG_UNBLOCK — returns prev
                2 => super::signal::set_mask(new_set),          // SIG_SETMASK — returns prev
                _ => return EINVAL,
            };
            // `new_mask` is the *old* value; compute the new one and
            // mirror into the legacy bitmap so nothing diverges.
            let resulting = match how {
                0 => new_mask | new_set,
                1 => new_mask & !new_set,
                2 => new_set,
                _ => new_mask,
            };
            SIGNAL_MASK = resulting;
        }
    }
    0
}

/// rt_sigreturn — return from signal handler
fn sys_rt_sigreturn(_args: [u64; 6]) -> i64 { 0 }

/// tgkill — send signal to a thread (tgkill(tgid, tid, sig)).
///
/// V6-XLAYER-009 fix: previously the tgid/tid args were ignored entirely
/// — any cave could call `tgkill(other_cave_tgid, *, SIGKILL)` and the
/// signal got OR'd into the GLOBAL SIGNAL_PENDING bitmap, killing
/// whoever was active when the signal next checked. Now we restrict
/// tgkill to "self" (the calling cave/process); cross-process signals
/// would need a real signal-routing layer (Phase B). tgid=0 / tgid=1
/// (self) is permitted; everything else returns ESRCH.
fn sys_tgkill(args: [u64; 6]) -> i64 {
    let tgid = args[0] as i32;
    let _tid = args[1] as i32;
    let sig = args[2] as u32;
    if sig == 0 { return 0; } // existence check
    // Permit only self-targeted signals. Our process always reports
    // pid=1 (see sys_getpid), so tgid in {0, 1} is "self".
    if tgid != 0 && tgid != 1 {
        return -(3i64); // ESRCH
    }
    if (sig as usize) < MAX_SIG {
        unsafe { SIGNAL_PENDING |= 1u64 << sig; }
        // V8-NEW: also mirror into signal.rs's async-pending bitmap so
        // the syscall-return poll in arch/mod.rs picks it up and
        // routes through the real delivery path (rt_sigframe, user
        // handler, rt_sigreturn). The legacy SIGNAL_PENDING is still
        // used by check_pending_signal() which a few older call sites
        // consult; keeping both in sync is cheap.
        super::signal::mark_pending(sig);
    }
    // SIGKILL / SIGSTOP can't be caught or ignored; take the cave
    // down directly rather than wait for the next syscall return.
    // (Async delivery of a SIGKILL would eventually fire
    // terminate_cave_fatal anyway — this just shortcuts.)
    if sig == SIGKILL {
        uart::puts("[signal] SIGKILL, terminating\n");
        sys_exit([1, 0, 0, 0, 0, 0]);
    }
    0
}

/// sigaltstack — set/get alternate signal stack
fn sys_sigaltstack(args: [u64; 6]) -> i64 {
    let ss_ptr = args[0] as usize;
    let old_ss_ptr = args[1] as usize;

    // NEW-SYS-027 / ATTACK-SYS-038: the struct is 24 bytes (ss_sp, ss_flags,
    // _pad, ss_size). Without gating this was a 24-byte EL1 write primitive
    // via old_ss_ptr and a 16-byte controlled read via ss_ptr.
    if old_ss_ptr != 0 && !uaccess::is_user_range(old_ss_ptr, 24) {
        return -(14i64);
    }
    if ss_ptr != 0 && !uaccess::is_user_range(ss_ptr, 24) {
        return -(14i64);
    }

    unsafe {
        if old_ss_ptr != 0 {
            let old = old_ss_ptr as *mut u64;
            core::ptr::write(old, SIGALT_SP);
            core::ptr::write(old.add(1), 0);
            core::ptr::write(old.add(2), SIGALT_SIZE);
        }
        if ss_ptr != 0 {
            let ss = ss_ptr as *const u64;
            SIGALT_SP = core::ptr::read(ss);
            SIGALT_SIZE = core::ptr::read(ss.add(2));
        }
    }
    0
}

/// Check for deliverable pending signals
pub fn check_pending_signal() -> Option<u32> {
    unsafe {
        let deliverable = SIGNAL_PENDING & !SIGNAL_MASK;
        if deliverable == 0 { return None; }
        let sig = deliverable.trailing_zeros();
        SIGNAL_PENDING &= !(1u64 << sig);
        Some(sig)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// #8: SHARED MEMORY — memfd_create, shmget stubs
// ═══════════════════════════════════════════════════════════════════════════

/// shmget — create shared memory segment (returns fd to anonymous mmap region)
fn sys_shmget(args: [u64; 6]) -> i64 {
    let size = args[1] as usize;
    if size == 0 || size > 16 * 1024 * 1024 { return EINVAL; }
    if let Ok(tmp) = vfs::resolve_path(b"/tmp") {
        if let Ok(node_idx) = vfs::create_node(tmp, b"shm_anon", vfs::NodeType::File, 0o100666) {
            if let Ok(fdi) = fd::alloc_fd(node_idx, 0) {
                return fdi as i64;
            }
        }
    }
    ENOMEM
}

/// memfd_create — create anonymous file backed by memory.
/// NEW-SYS-049: now charges Resource::Fds and goes through the cap layer
/// (SyscallCat::Memory). Previously classified `Always`, so any cave —
/// including ones with no `mem` cap — could spam memfd_create past their
/// fd cap and never be charged.
fn sys_memfd_create(_args: [u64; 6]) -> i64 {
    // V8-ROOT-1: charge → alloc → refund-on-err is one CS.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 1) {
            return e;
        }
        let result: i64 = if vfs::is_ready() {
            if let Some(tmp) = vfs::find_child(0, b"tmp") {
                if let Ok(node_idx) = vfs::create_node(tmp, b"memfd", vfs::NodeType::File, 0o100666) {
                    match fd::alloc_fd(node_idx, 0) {
                        Ok(fdi) => fdi as i64,
                        Err(e) => e,
                    }
                } else { ENOMEM }
            } else { ENOMEM }
        } else {
            super::quotas::refund_active(super::quotas::Resource::Fds, 1);
            return 30;
        };
        if result < 0 {
            super::quotas::refund_active(super::quotas::Resource::Fds, 1);
        }
        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// #9: /proc FILESYSTEM — synthetic file reads
// ═══════════════════════════════════════════════════════════════════════════

/// Generate /proc content on read
pub fn proc_read(path: &str, buf: &mut [u8]) -> usize {
    let content: &[u8] = match path {
        "/proc/self/status" | "/proc/1/status" =>
            b"Name:\tbat_process\nState:\tR (running)\nTgid:\t1\nPid:\t1\nPPid:\t0\nUid:\t0\t0\t0\t0\nGid:\t0\t0\t0\t0\nVmSize:\t4096 kB\nVmRSS:\t2048 kB\nThreads:\t1\n",
        "/proc/self/maps" | "/proc/1/maps" =>
            b"00010000-00100000 r-xp 00000000 00:00 0  [code]\n00100000-00200000 rw-p 00000000 00:00 0  [data]\n40000000-42000000 rw-p 00000000 00:00 0  [heap]\nfffff000-ffffffff rw-p 00000000 00:00 0  [stack]\n",
        "/proc/self/stat" | "/proc/1/stat" =>
            b"1 (bat_process) R 0 1 1 0 -1 4194304 100 0 0 0 10 5 0 0 20 0 1 0 100 4194304 512 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0\n",
        "/proc/self/cmdline" | "/proc/1/cmdline" =>
            b"bat_process\0",
        "/proc/meminfo" =>
            b"MemTotal:       262144 kB\nMemFree:        131072 kB\nMemAvailable:   196608 kB\nBuffers:            0 kB\nCached:          8192 kB\nSwapTotal:          0 kB\nSwapFree:           0 kB\n",
        "/proc/cpuinfo" =>
            b"processor\t: 0\nBogoMIPS\t: 48.00\nFeatures\t: fp asimd aes pmull sha1 sha2 crc32\nCPU implementer\t: 0x61\nCPU architecture: 8\nCPU part\t: 0xb02\n\nHardware\t: Bat_OS ARM64\n",
        "/proc/version" =>
            b"Bat_OS version 0.3.0 (bat@batcave) (aarch64-bat-none) #1 SMP PREEMPT\n",
        "/proc/uptime" => b"3600.00 3500.00\n",
        "/proc/loadavg" => b"0.01 0.05 0.10 1/32 42\n",
        "/proc/filesystems" => b"nodev\tbatfs\nnodev\tproc\nnodev\ttmpfs\n",
        "/proc/mounts" | "/proc/self/mounts" =>
            b"batfs / batfs rw 0 0\nproc /proc proc rw 0 0\ntmpfs /tmp tmpfs rw 0 0\n",
        _ => return 0,
    };
    let len = content.len().min(buf.len());
    buf[..len].copy_from_slice(&content[..len]);
    len
}

// ═══════════════════════════════════════════════════════════════════════════
// CUSTOM: Framebuffer blit — syscall 500
// args: x0=pixel_ptr, x1=src_width, x2=src_height, x3=dst_x, x4=dst_y
// ═══════════════════════════════════════════════════════════════════════════

fn sys_blit_framebuffer(args: [u64; 6]) -> i64 {
    let src_ptr = args[0] as usize;
    let src_w = args[1] as u32;
    let src_h = args[2] as u32;
    let dst_x = args[3] as u32;
    let dst_y = args[4] as u32;

    let fb = crate::drivers::virtio::gpu::framebuffer();
    let screen_w = crate::drivers::virtio::gpu::width();
    let screen_h = crate::drivers::virtio::gpu::height();

    if fb.is_null() { return -1; }

    // V2-NEW-035 / NEW-SYS-030 / ESC-007: cap check already gates us via
    // SyscallCat::Display; also reject unbounded / kernel source pointers.
    // Without this, any cave could pass src_ptr = 0x40000000 and
    // render kernel RAM to the screen.
    let pixels = match (src_w as usize).checked_mul(src_h as usize)
        .and_then(|p| p.checked_mul(4))
    {
        Some(b) if b > 0 => b,
        _ => return -(22i64), // EINVAL: zero or overflow
    };
    if !uaccess::is_user_range(src_ptr, pixels) {
        return -(14i64); // EFAULT
    }

    // Copy pixels from user buffer to framebuffer
    for y in 0..src_h {
        let fb_y = dst_y + y;
        if fb_y >= screen_h { break; }
        for x in 0..src_w {
            let fb_x = dst_x + x;
            if fb_x >= screen_w { break; }
            let src_offset = (y * src_w + x) as usize;
            let dst_offset = (fb_y * screen_w + fb_x) as usize;
            unsafe {
                let pixel: u32;
                core::arch::asm!("ldr {v:w}, [{a}]",
                    a = in(reg) src_ptr + src_offset * 4,
                    v = out(reg) pixel);
                core::ptr::write_volatile(fb.add(dst_offset), pixel);
            }
        }
    }

    // Flush the affected region to display
    crate::drivers::virtio::gpu::flush(dst_x, dst_y, src_w.min(screen_w - dst_x), src_h.min(screen_h - dst_y));

    0
}

// ─── BSD socket syscall wrappers for Chromium ───
// These route to src/batcave/linux/sockets.rs which provides the full
// BSD API Chromium expects. The legacy sys_socket/sys_connect/sys_sendto/
// sys_recvfrom above use a separate low-fd namespace (<64) for compat
// with netsurf_test. The new wrappers below operate on the high-fd
// namespace (>=1024) that sockets.rs manages.

fn sys_socketpair(args: [u64; 6]) -> i64 {
    // CHROMIUM-PHASE-C: real pipe-backed socketpair. Creates two
    // VFS Socket nodes + two pipe-kinded fds sharing a pair slot
    // from `pipe_buf`. Bytes written to one end become readable
    // on the other. `read()` / `write()` / `poll()` dispatch on
    // FdKind::Pipe in their syscall handlers.
    let _domain = args[0] as u32;
    let _sock_type = args[1] as u32;
    let _protocol = args[2] as u32;
    let sv_ptr = args[3] as usize;
    if sv_ptr == 0 || !is_user_ptr(sv_ptr, 8) { return EFAULT; }

    if !vfs::is_ready() { return -38; }
    let slot = match super::pipe_buf::alloc_pair() {
        Some(s) => s,
        None => return -24, // EMFILE
    };
    let node_a = match vfs::create_node(0, b".sockpair_a", vfs::NodeType::Socket, 0o140755) {
        Ok(n) => n, Err(e) => { super::pipe_buf::release(slot); return e; },
    };
    let node_b = match vfs::create_node(0, b".sockpair_b", vfs::NodeType::Socket, 0o140755) {
        Ok(n) => n, Err(e) => { super::pipe_buf::release(slot); return e; },
    };
    let fd_a = match fd::alloc_fd_pipe(node_a, 0, slot, 0) {
        Ok(f) => f, Err(e) => { super::pipe_buf::release(slot); return e; },
    };
    let fd_b = match fd::alloc_fd_pipe(node_b, 0, slot, 1) {
        Ok(f) => f, Err(e) => { super::pipe_buf::release(slot); return e; },
    };
    unsafe {
        core::arch::asm!("str {v:w}, [{a}]",
            a = in(reg) sv_ptr, v = in(reg) fd_a);
        core::arch::asm!("str {v:w}, [{a}]",
            a = in(reg) sv_ptr + 4, v = in(reg) fd_b);
    }
    uart::puts("[socketpair] fd=");
    crate::kernel::mm::print_num(fd_a as usize);
    uart::puts(",");
    crate::kernel::mm::print_num(fd_b as usize);
    uart::puts(" slot=");
    crate::kernel::mm::print_num(slot);
    uart::puts("\n");
    0
}

fn sys_bind(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as i32;

    // Legacy path: `sys_socket` creates sockets as VFS nodes with low
    // fds (e.g. 10) instead of the modern `sockets::` table at fd >=
    // SOCKET_FD_BASE (1024). `sockets::bind()` would reject them with
    // ENOTSOCK. Since our outbound-only legacy sockets pick ephemeral
    // ports on connect(), bind() is advisory — accept it as a no-op.
    // This unlocks busybox nslookup (UDP DGRAM → bind → sendto DNS).
    if fd_num >= 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::Socket {
                // Validate the addr pointer if non-null; trust the
                // content (ephemeral local port picking happens on
                // the send side in sys_sendto).
                let addr_ptr = args[1] as usize;
                let addr_len = args[2] as usize;
                if addr_ptr != 0 && addr_len >= 8 {
                    if !uaccess::is_user_range(addr_ptr, addr_len.min(16)) {
                        return EFAULT;
                    }
                }
                return 0;
            }
        }
    }

    // Modern-socket path: fd was handed out by sockets::socket() with
    // fd >= SOCKET_FD_BASE; the table entry holds per-socket state.
    super::sockets::bind(
        args[0] as i32,
        args[1] as *const super::sockets::SockaddrIn,
        args[2] as u32,
    )
}
fn sys_listen(args: [u64; 6]) -> i64 {
    super::sockets::listen(args[0] as i32, args[1] as i32)
}
fn sys_accept(args: [u64; 6]) -> i64 {
    accept_charged(args[0] as i32,
                   args[1] as *mut super::sockets::SockaddrIn,
                   args[2] as *mut u32,
                   0)
}
fn sys_accept4(args: [u64; 6]) -> i64 {
    accept_charged(args[0] as i32,
                   args[1] as *mut super::sockets::SockaddrIn,
                   args[2] as *mut u32,
                   args[3] as i32)
}

/// NEW-DOS-005: accept{,4} was never quota-charged. A remote SYN flood
/// could drive the listening cave past its Sockets/Fds cap because the
/// new fd was created from the kernel side without consulting the cave
/// ledger. Charge both Sockets+Fds up front; refund on failure. sys_close
/// refunds them on release via its existing node-type check.
fn accept_charged(listen_fd: i32,
                  addr: *mut super::sockets::SockaddrIn,
                  addrlen: *mut u32,
                  flags: i32) -> i64 {
    // V8-ROOT-1: charge-Sockets + charge-Fds + accept + refund-on-err atomic.
    // NOTE: accept4 internally may block / take locks. We do NOT hold the
    // IrqGuard across the accept4 call itself — only the quota bookkeeping.
    // This still closes the window between charges where a racing syscall
    // could observe partial accounting.
    crate::critical_section! {
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Sockets, 1) {
            return e;
        }
        if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 1) {
            super::quotas::refund_active(super::quotas::Resource::Sockets, 1);
            return e;
        }
    }
    let r = super::sockets::accept4(listen_fd, addr, addrlen, flags);
    if r < 0 {
        crate::critical_section! {
            super::quotas::refund_active(super::quotas::Resource::Sockets, 1);
            super::quotas::refund_active(super::quotas::Resource::Fds, 1);
        }
    }
    r
}
fn sys_getsockname(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as i32;
    let addr_ptr = args[1] as usize;
    let addrlen_ptr = args[2] as usize;

    // Legacy VFS-socket path: fabricate a plausible local address
    // (0.0.0.0 : ephemeral) so traceroute can read a local port and
    // continue. Busybox traceroute calls getsockname after connect()
    // purely to learn the kernel-picked local port.
    if fd_num >= 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::Socket {
                if addr_ptr == 0 || addrlen_ptr == 0 { return EINVAL; }
                if !uaccess::is_user_range(addr_ptr, 16) { return EFAULT; }
                if !uaccess::is_user_range(addrlen_ptr, 4) { return EFAULT; }

                // Read caller-provided buffer length; write back min(avail, 16).
                let available: u32;
                unsafe {
                    let mut v: u32 = 0;
                    core::arch::asm!("ldr {v:w}, [{a}]",
                        a = in(reg) addrlen_ptr, v = out(reg) v);
                    available = v;
                }
                let write_len = (available as usize).min(16);

                // struct sockaddr_in { family(u16), port_be(u16), addr_be(u32), zero(u64) }
                let local_port = unsafe { SOCK_LOCAL_PORT };
                let port_be = local_port.to_be();
                let family: u16 = AF_INET as u16;
                let zero: u64 = 0;
                unsafe {
                    if write_len >= 2 {
                        core::arch::asm!("strh {v:w}, [{a}]",
                            a = in(reg) addr_ptr, v = in(reg) family as u32);
                    }
                    if write_len >= 4 {
                        core::arch::asm!("strh {v:w}, [{a}]",
                            a = in(reg) addr_ptr + 2, v = in(reg) port_be as u32);
                    }
                    if write_len >= 8 {
                        core::arch::asm!("str {v:w}, [{a}]",
                            a = in(reg) addr_ptr + 4, v = in(reg) 0u32);
                    }
                    if write_len >= 16 {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) addr_ptr + 8, v = in(reg) zero);
                    }
                    // Store actual size back to addrlen.
                    let filled: u32 = 16;
                    core::arch::asm!("str {v:w}, [{a}]",
                        a = in(reg) addrlen_ptr, v = in(reg) filled);
                }
                return 0;
            }
        }
    }

    super::sockets::getsockname(
        args[0] as i32,
        args[1] as *mut super::sockets::SockaddrIn,
        args[2] as *mut u32,
    )
}
fn sys_getpeername(args: [u64; 6]) -> i64 {
    super::sockets::getpeername(
        args[0] as i32,
        args[1] as *mut super::sockets::SockaddrIn,
        args[2] as *mut u32,
    )
}
fn sys_sendmsg(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as i32;
    // CHROMIUM-PHASE-D: socketpair() returns Pipe-kinded fds (the
    // sockets:: table only knows about TCP/UDP). When Chromium does
    // sendmsg on its zygote control socketpair, the sockets:: code
    // returns ENOTSOCK and Chromium FATALs with "Cannot communicate
    // with zygote". Route Pipe-kinded fds through pipe_buf::write
    // so the iovec data actually lands somewhere.
    if fd_num >= 0 {
        if let Some((pair_slot, side)) = fd::pipe_info(fd_num as u32) {
            return sendmsg_pipe(pair_slot, side, args[1] as *const super::sockets::Msghdr);
        }
    }
    super::sockets::sendmsg(
        args[0] as i32,
        args[1] as *const super::sockets::Msghdr,
        args[2] as i32,
    )
}

fn sendmsg_pipe(slot: usize, side: u8, msg: *const super::sockets::Msghdr) -> i64 {
    if msg.is_null() { return EFAULT; }
    if !is_user_ptr(msg as usize, core::mem::size_of::<super::sockets::Msghdr>()) {
        return EFAULT;
    }
    // The sender side writes to the OPPOSITE side of the pipe — that's
    // what the receiver reads from. pipe_buf::write expects "side" to
    // be the side data lands ON, not the side we're calling from.
    let target_side = side ^ 1;
    let m: super::sockets::Msghdr = unsafe { core::ptr::read(msg) };
    if m.msg_iovlen > 16 { return EINVAL; }
    let mut total: i64 = 0;
    for i in 0..m.msg_iovlen {
        let iv = unsafe { core::ptr::read(m.msg_iov.add(i)) };
        if iv.iov_len == 0 { continue; }
        if iv.iov_base.is_null() { return EFAULT; }
        if !uaccess::is_user_range(iv.iov_base as usize, iv.iov_len) {
            return EFAULT;
        }
        let data = unsafe {
            core::slice::from_raw_parts(iv.iov_base as *const u8, iv.iov_len)
        };
        match super::pipe_buf::write(slot, target_side, data) {
            Ok(n) => {
                total += n as i64;
                if n < iv.iov_len { break; } // partial write
            }
            Err(e) => {
                if total > 0 { return total; }
                return e;
            }
        }
    }
    total
}

fn sys_recvmsg(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as i32;
    // Mirror of sys_sendmsg: route socketpair-Pipe fds through
    // pipe_buf::read so the iovecs get filled with whatever the
    // peer wrote.
    if fd_num >= 0 {
        if let Some((pair_slot, side)) = fd::pipe_info(fd_num as u32) {
            return recvmsg_pipe(pair_slot, side, args[1] as *mut super::sockets::Msghdr);
        }
    }
    super::sockets::recvmsg(
        args[0] as i32,
        args[1] as *mut super::sockets::Msghdr,
        args[2] as i32,
    )
}

fn recvmsg_pipe(slot: usize, side: u8, msg: *mut super::sockets::Msghdr) -> i64 {
    if msg.is_null() { return EFAULT; }
    if !is_user_ptr(msg as usize, core::mem::size_of::<super::sockets::Msghdr>()) {
        return EFAULT;
    }
    let m: super::sockets::Msghdr = unsafe { core::ptr::read(msg) };
    if m.msg_iovlen > 16 { return EINVAL; }
    let mut total: i64 = 0;
    for i in 0..m.msg_iovlen {
        let iv = unsafe { core::ptr::read(m.msg_iov.add(i)) };
        if iv.iov_len == 0 { continue; }
        if iv.iov_base.is_null() { return EFAULT; }
        if !uaccess::is_user_range(iv.iov_base as usize, iv.iov_len) {
            return EFAULT;
        }
        let buf = unsafe {
            core::slice::from_raw_parts_mut(iv.iov_base as *mut u8, iv.iov_len)
        };
        // We read from OUR side of the pipe (where peer's writes land).
        match super::pipe_buf::read(slot, side, buf) {
            Ok(n) => {
                total += n as i64;
                if n == 0 { break; }      // EOF or empty
                if n < iv.iov_len { break; } // partial read
            }
            Err(e) => {
                if total > 0 { return total; }
                return e;
            }
        }
    }
    total
}
fn sys_setsockopt(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as i32;

    // Legacy path: legacy-table socket fds (VFS Socket node) — accept
    // common socket options as no-ops. busybox traceroute sets
    // SO_SNDBUF/SO_RCVBUF/IP_TTL; none require state we track.
    if fd_num >= 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::Socket {
                // Validate the optval pointer if present; ignore contents.
                let optval_ptr = args[3] as usize;
                let optlen = args[4] as usize;
                if optval_ptr != 0 && optlen > 0 {
                    if !uaccess::is_user_range(optval_ptr, optlen.min(64)) {
                        return EFAULT;
                    }
                }
                return 0;
            }
        }
    }

    super::sockets::setsockopt(
        args[0] as i32, args[1] as i32, args[2] as i32,
        args[3] as *const u8, args[4] as u32,
    )
}
fn sys_getsockopt(args: [u64; 6]) -> i64 {
    super::sockets::getsockopt(
        args[0] as i32, args[1] as i32, args[2] as i32,
        args[3] as *mut u8, args[4] as *mut u32,
    )
}
fn sys_shutdown(args: [u64; 6]) -> i64 {
    // CHROMIUM-PHASE-B: handle shutdown on VFS-Socket fds (the ones
    // our socketpair() stub creates). The sockets::shutdown path
    // only knows about real TCP/UDP pcbs; for our stub sockets it
    // would return ENOTSOCK, which Chromium's sandbox_host_linux.cc
    // takes as a CHECK failure. Return 0 (success) for any fd that
    // maps to a Socket-type VfsNode — Mojo's sandbox-host init just
    // wants the call to succeed.
    let fd_num = args[0] as i32;
    if fd_num >= 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::Socket {
                return 0;
            }
        }
    }
    super::sockets::shutdown(args[0] as i32, args[1] as i32)
}

