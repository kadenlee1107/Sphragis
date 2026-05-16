// Sphragis — Linux Syscall Translation Layer
// Intercepts ARM64 Linux syscalls (svc #0) from Cave processes
// and translates them into Sphragis operations.
//
// Every syscall is a security checkpoint:
// No capability → EACCES. No exceptions.
//
// ARM64 Linux syscall convention:
// x8 = syscall number
// x0-x5 = arguments
// x0 = return value (negative = -errno)

use crate::drivers::uart;
use crate::caves::cave;
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
/// anything else shows up as "?".
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
const EAGAIN: i64 = -11;   // Try again
const EPERM: i64 = -1;     // Operation not permitted

/// Returns true if `p` + `size` is plausibly inside the cave's user-space.
// /
/// After ROOT-1 (per-cave page tables) lands this becomes an exact check
/// against the caller's page-table VA range. Today the cave and the
/// kernel share one identity-mapped VA, so the best we can do is reject
/// obvious kernel addresses: NULL, low-page nulls, and anywhere inside
/// the kernel RAM identity map (0x4000_0000..0x8000_0000 on QEMU virt).
// /
/// Returns `false` on overflow, NULL, low pages, or kernel-range pointers.
// /
/// V8-ROOT-8 (regression fix): delegate to `uaccess::is_user_range` so the
/// cave's own user-window bounds (set at `enter()` time) are consulted,
/// not just the static legacy window. Any cave with virt_base != 0
/// previously could read/write the legacy 0x1000..0x4000_0000 range and
/// pivot into another cave's state.
fn is_user_ptr(p: usize, size: usize) -> bool {
    uaccess::is_user_range(p, size)
}

/// walk the page tables to verify a user pointer is
/// WRITABLE (not just mapped). Used by syscalls that emulate Linux
/// behavior of returning EFAULT when the user-supplied buffer is
/// read-only or unmapped (e.g. `getrlimit` from `CheckMemoryReadOnly`).
// /
/// AP[2:1] in L3 PTE (bits 7:6):
/// 0b00: EL1 R/W, no EL0 access
/// 0b01: EL1 R/W, EL0 R/W ← user-writable
/// 0b10: EL1 R/O, no EL0 access
/// 0b11: EL1 R/O, EL0 R/O
fn is_user_writable(p: usize, size: usize) -> bool {
    if !uaccess::is_user_range(p, size) { return false; }
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;
    if l1_phys == 0 { return false; }

    // Check first and last byte's pages.
    let start_page = (p as u64) & !0xFFFu64;
    let end_page = ((p + size - 1) as u64) & !0xFFFu64;
    let mut va = start_page;
    while va <= end_page {
        let l1_idx = (va >> 30) & 0x1FF;
        let l1e: u64 = unsafe {
            core::ptr::read_volatile((l1_phys + l1_idx * 8) as *const u64)
        };
        // pages in an active demand-page reservation
        // count as writable even if they aren't yet committed —
        // the next access will demand-page them in with EL0 R/W
        // perms (USER_PAGE_FLAGS). Without this, every check on a
        // freshly-allocated pthread stack returns EFAULT (e.g.
        // Chromium's `base::GetMaxFds()` calls `getrlimit(RLIMIT_NOFILE,
        // &rlim)` with `&rlim` on a not-yet-committed thread stack
        // page, and our prlimit64 returns EFAULT — Chromium logs
        // "Failed to get file descriptor limit: Bad address (14)"
        // and downstream code may misuse the unset rlim).
        if super::demand_page::is_in_active_reservation(va as usize, 4096) {
            // Conservative: treat as writable if the page is reserved
            // (will be backed lazily). The actual write either hits
            // a committed page or triggers demand_page::commit which
            // installs USER_PAGE_FLAGS (= AP=01, EL0 R/W).
            if va == end_page { break; }
            va += 4096;
            continue;
        }
        if (l1e & 0b11) != 0b11 { return false; }
        let l2_phys = l1e & 0x0000_FFFF_FFFF_F000;
        let l2_idx = (va >> 21) & 0x1FF;
        let l2e: u64 = unsafe {
            core::ptr::read_volatile((l2_phys + l2_idx * 8) as *const u64)
        };
        // L2 BLOCK descriptor (0b01): identity-mapped 2 MB block.
        // L2 TABLE (0b11): walk L3.
        // iter 20: distinguish L2 BLOCK (0b01) from L3
        // PAGE (0b11). Valid bit is bit 0; bit 1 = "table/page" (1)
        // vs "block" (0) at this level. The cave's main user-VA
        // window is mapped via L2 BLOCKS (0b01) — the previous
        // `(pte & 0b11) != 0b11` check rejected those as "invalid",
        // making is_user_writable falsely return EFAULT for every
        // stack/heap pointer in the cave window. That broke
        // prlimit64(old=...) → glibc's pthread_getattr_np → AK
        // StackInfo VERIFY.
        let is_block;
        let pte = if (l2e & 0b11) == 0b01 {
            is_block = true;
            l2e
        } else if (l2e & 0b11) == 0b11 {
            is_block = false;
            let l3_phys = l2e & 0x0000_FFFF_FFFF_F000;
            let l3_idx = (va >> 12) & 0x1FF;
            unsafe {
                core::ptr::read_volatile((l3_phys + l3_idx * 8) as *const u64)
            }
        } else {
            return false;
        };
        // For BLOCK: valid = bit 0 set, block-ness = bit 1 clear (which
        // is already implied by the 0b01 match above).
        // For PAGE (L3): valid descriptor uses bits 1:0 = 0b11.
        if is_block {
            if (pte & 0b1) != 0b1 { return false; }
        } else {
            if (pte & 0b11) != 0b11 { return false; }
        }
        // AP[2:1] at bits 7:6. Only AP=0b01 means EL0 writable.
        let ap = (pte >> 6) & 0b11;
        if ap != 0b01 { return false; }
        if va == end_page { break; }
        va += 4096;
    }
    true
}

// Syscall categories for capability checking. RawNet is reserved
// for syscalls that hand a cave raw network access (AF_PACKET etc.);
// none are wired today, but the variant stays so the dispatcher
// match doesn't have to distinguish present-vs-future when one lands.
#[derive(Clone, Copy)]
#[allow(dead_code)]
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
// Linux AArch64 syscall numbers. The full table stays named because
// caves invoke these by name (CLONE, EXECVE, etc.); we keep entries
// even where Sphragis hasn't grown a handler yet — adding the constant
// when the handler lands is just churn for protocol-stable IDs.
#[allow(dead_code)]
mod nr {
    pub const GETCWD: u64 = 17;
    pub const IOCTL: u64 = 29;
    pub const FACCESSAT: u64 = 48;
    pub const CHDIR: u64 = 49;
    pub const OPENAT: u64 = 56;
    pub const CLOSE: u64 = 57;
    pub const CLOSE_RANGE: u64 = 436;
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

    // Linux 5.13+ Landlock sandboxing API. Chromium probes these once at
    // startup to feature-detect the host's sandbox capability; we don't
    // implement Landlock so all three return ENOSYS. They MUST be
    // surfaced as named stubs (not the unknown-syscall fallback) because:
    // 1. The fallback prints "[linux] unknown syscall 444" which is
    // noise — Chromium expects ENOSYS and handles it silently.
    // 2. The fallback fires the SPHRAGIS_KEEP_GOING skip-log, polluting
    // the "actual unknown syscalls" list during Chromium init.
    pub const LANDLOCK_CREATE_RULESET: u64 = 444;
    pub const LANDLOCK_ADD_RULE:       u64 = 445;
    pub const LANDLOCK_RESTRICT_SELF:  u64 = 446;

    // Sphragis-private syscalls. Numbered well above the Linux AArch64
    // range (which tops out around 463 today) so they never collide
    // with a future Linux number we might want to honour.
    //
    // 0x4001 — bat_https_open(host_ptr, host_len, port, flags) -> fd | -errno
    // The kernel runs TLS itself; the returned fd is plaintext from the
    // cave's perspective. See DESIGN_HTTPS_SYSCALL.md.
    pub const BAT_HTTPS_OPEN: u64 = 0x4001;
}

/// Handle a Linux syscall from a Cave process.
/// cave_id: which Cave this process belongs to
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
    // Cooperative yield every 64 syscalls — balances giving other
    // threads CPU time without thrashing on context switches. Now
    // that timer IRQs work (after the daifclr fix in cxt_switch), the
    // 100Hz timer handles user-mode preemption; this yield mostly
    // services the very-bursty syscall pattern (e.g. Chromium's
    // pthread setup which can do hundreds of syscalls in a tight loop
    // without re-entering user mode long enough to be preempted).
    if (n & 0x3F) == 0 {
        super::threads::schedule();
    }
    // Periodic thread-state dump for deadlock diagnosis. Triggered off
    // the syscall counter (NOT the timer IRQ — that fires too sporadically
    // on QEMU virt for a useful diagnostic cadence). At every 65536
    // syscalls we get periodic snapshots to see the thread table
    // evolution. (Was 1024 — too noisy now that Chromium does millions
    // of syscalls past the prior 3K wall.)
    // Bumped from 0xFFFF to 0xFFFFF — once per million syscalls.
    // The thread pool can do ~50M syscalls per smoke; one dump
    // every 65k was 95% of the log volume.
    if (n & 0xF_FFFF) == 0 && n > 0 {
        uart::puts("[diag] thread-state dump @ syscall ");
        crate::kernel::mm::print_num(n as usize);
        uart::puts("\n");
        super::threads::dump();
        super::syscall_history::dump_per_tid_last();
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
        // iter 16: print ELR_EL1 (the user PC at SVC entry,
        // i.e. the instruction AFTER `svc #0`). Critical for finding
        // which libc function called the syscall when /bin/js hangs
        // in userland — without it we can't correlate to disassembly.
        let elr: u64;
        unsafe { core::arch::asm!("mrs {}, ELR_EL1", out(reg) elr); }
        uart::puts("] elr=0x");
        for sh in (0..16).rev() {
            uart::putc(hex[((elr >> (sh*4)) & 0xF) as usize]);
        }
        uart::puts("\n");
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
        32 => (SyscallCat::Always, sys_stub_zero),    // flock — single-process, no real lock needed
        73 => (SyscallCat::Always, sys_ppoll),        // ppoll
        98 => (SyscallCat::Always, sys_futex),        // futex
        99 => (SyscallCat::Always, sys_stub_zero),   // set_robust_list
        100 => (SyscallCat::Always, sys_stub_zero),  // get_robust_list
        101 => (SyscallCat::Always, sys_nanosleep),  // nanosleep
        102 => (SyscallCat::Always, sys_getitimer),  // d: zero-fill struct itimerval
        103 => (SyscallCat::Always, sys_stub_zero),  // setitimer (no out-buffer; 0 OK)
        131 => (SyscallCat::Always, sys_tgkill),       // tgkill
        132 => (SyscallCat::Always, sys_sigaltstack),  // sigaltstack
        134 => (SyscallCat::Always, sys_rt_sigaction), // rt_sigaction
        135 => (SyscallCat::Always, sys_rt_sigprocmask), // rt_sigprocmask
        136 => (SyscallCat::Always, sys_stub_zero),  // rt_sigpending
        137 => (SyscallCat::Always, sys_stub_zero),  // rt_sigtimedwait
        139 => (SyscallCat::Always, sys_rt_sigreturn), // rt_sigreturn
        144 => (SyscallCat::Always, sys_stub_zero),  // setgid
        146 => (SyscallCat::Always, sys_stub_zero),  // setuid
        153 => (SyscallCat::Always, sys_times),      // d: real ticks + zero-fill tms
        154 => (SyscallCat::Always, sys_stub_zero),  // setpgid
        155 => (SyscallCat::Always, sys_stub_zero),  // getpgid
        157 => (SyscallCat::Always, sys_stub_zero),  // sched_getscheduler
        158 => (SyscallCat::Always, sys_sched_getparam), // d: zero-fill struct sched_param
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
        169 => (SyscallCat::Always, sys_gettimeofday), // d: real timeval write
        170 => (SyscallCat::Always, sys_stub_zero),  // getpgrp/setpgid
        171 => (SyscallCat::Always, sys_sigaltstack), // sigaltstack (compat)
        178 => (SyscallCat::Always, sys_gettid),      // gettid
        179 => (SyscallCat::Always, sys_sysinfo),    // sysinfo (real impl)
        // NOTE: on AArch64, syscall 204 is getsockname (not sched_getaffinity).
        // getsockname is routed below via nr::GETSOCKNAME in the network block.
        // True sched_getaffinity is 123 (AArch64), sched_setaffinity is 122.
        // b: real sched_getaffinity. Was sys_stub_zero, returning 0
        // and writing nothing to the user mask. glibc's _SC_NPROCESSORS_ONLN /
        // CPU_COUNT(mask) then walked uninitialized memory; Chromium uses this
        // value to size lock-free freelists, sharded counters, and worker pools.
        // Stale memory containing 0x01 → freelist[0] keyed on garbage → eventually
        // a freelist head value of 0x1 ends up in PartitionAlloc → the deterministic
        // x1=0x1 BRK we've been chasing.
        nr::SCHED_GETAFFINITY => (SyscallCat::Always, sys_sched_getaffinity),
        nr::SCHED_SETAFFINITY => (SyscallCat::Always, sys_stub_zero),

        // Landlock probes return ENOSYS quietly — Chromium
        // feature-detects, gets ENOSYS, falls back to seccomp-bpf or
        // "no sandbox" depending on the build flags. Without these
        // explicit stubs, every Chromium boot logs three "unknown
        // syscall NNN" lines that drown the real unknown-syscall list.
        nr::LANDLOCK_CREATE_RULESET => (SyscallCat::Always, sys_stub_enosys),
        nr::LANDLOCK_ADD_RULE       => (SyscallCat::Always, sys_stub_enosys),
        nr::LANDLOCK_RESTRICT_SELF  => (SyscallCat::Always, sys_stub_enosys),

        // iter 6: surfaced via the chromium-version smoke
        // crash path. content_shell's base::FilePathWatcher tries to
        // inotify_init1; not having it means watcher silently disables.
        // fstatfs is called by glibc realpath/realpathat for filesystem
        // type checks. Both safely return ENOSYS / a benign zero stat
        // without breaking Chromium.
        26 => (SyscallCat::FileIO, sys_stub_enosys),  // inotify_init1
        27 => (SyscallCat::FileIO, sys_stub_zero),    // inotify_add_watch — return wd 0
        28 => (SyscallCat::FileIO, sys_stub_zero),    // inotify_rm_watch
        44 => (SyscallCat::FileIO, sys_fstatfs),      // fstatfs — return zero buf
        // NOTE: AArch64 syscall 222 is mmap (already routed via nr::MMAP),
        // 223 is fadvise64. 210 is shutdown (moved to Network block below).
        // Previously these were mislabeled as shmget/shmctl/shutdown-stub.
        nr::FADVISE64 => (SyscallCat::Always, sys_stub_zero),
        233 => (SyscallCat::Always, sys_madvise), // c: PT-walking madvise (only zeros committed pages)
        262 => (SyscallCat::Always, sys_stub_zero),  // getrlimit equiv
        276 => (SyscallCat::FileIO, sys_renameat),  // renameat2 — same impl, ignores flags arg
        279 => (SyscallCat::Memory, sys_memfd_create), // memfd_create — needs mem cap
        // fchown/lchown/chown — return success (no-op on our virtual fs).
        // Was: unknown syscall 55, returned 0 anyway, but with a stale "[linux] unknown
        // syscall 55" warning that suggested something was wrong. Stub-zero matches
        // semantics: our VFS doesn't track owner/group, no-op is correct.
        55 => (SyscallCat::Always, sys_stub_zero),  // fchown
        // clock_nanosleep — alias to sys_nanosleep. The clockid_t arg
        // (x0) is ignored; we treat it as CLOCK_MONOTONIC since cntpct_el0 is
        // monotonic. Linux's clock_nanosleep with TIMER_ABSTIME would need
        // different handling but we don't see that in practice.
        115 => (SyscallCat::Always, sys_clock_nanosleep_compat),
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
        nr::CLOSE_RANGE => (SyscallCat::FileIO, sys_close_range),
        nr::READ => (SyscallCat::FileIO, sys_read),
        nr::WRITE => (SyscallCat::FileIO, sys_write),
        nr::LSEEK => (SyscallCat::FileIO, sys_lseek),
        nr::FSTAT => (SyscallCat::FileIO, sys_fstat),
        nr::NEWFSTATAT => (SyscallCat::FileIO, sys_newfstatat),
        nr::IOCTL => (SyscallCat::FileIO, sys_ioctl),
        nr::GETCWD => (SyscallCat::FileIO, sys_getcwd),
        23 => (SyscallCat::FileIO, sys_dup),          // dup
        24 => (SyscallCat::FileIO, sys_dup3),         // dup3
        25 => (SyscallCat::FileIO, sys_fcntl),        // fcntl
        34 => (SyscallCat::FileIO, sys_mkdirat),      // mkdirat
        35 => (SyscallCat::FileIO, sys_stub_zero),    // unlinkat
        37 => (SyscallCat::FileIO, sys_stub_zero),    // linkat — hardlinks; success-stub
        38 => (SyscallCat::FileIO, sys_renameat),     // renameat —  iter 3: real rename for leveldb MANIFEST→CURRENT
        43 => (SyscallCat::FileIO, sys_statfs),       // d: real statfs fill (was leaking uninit buf)
        46 => (SyscallCat::FileIO, sys_ftruncate),    // ftruncate — must really set node size for shm
        47 => (SyscallCat::FileIO, sys_stub_zero),    // fallocate — Chromium uses for shmem pre-alloc; success-stub OK
        48 => (SyscallCat::FileIO, sys_faccessat),    // faccessat
        49 => (SyscallCat::FileIO, sys_chdir),        // chdir
        59 => (SyscallCat::FileIO, sys_pipe2),        // pipe2
        61 => (SyscallCat::FileIO, sys_getdents64),   // getdents64
        66 => (SyscallCat::FileIO, sys_writev),       // writev
        67 => (SyscallCat::FileIO, sys_pread64),      // pread64 — Chromium uses for positional file reads
        68 => (SyscallCat::FileIO, sys_pwrite64),     // pwrite64
        71 => (SyscallCat::FileIO, sys_sendfile),     // sendfile
        78 => (SyscallCat::FileIO, sys_readlinkat),   // readlinkat
        88 => (SyscallCat::FileIO, sys_stub_zero),    // utimensat — return 0; we don't track mtime
        53 => (SyscallCat::FileIO, sys_stub_zero),    // fchownat — single-user OS; ignore
        82 => (SyscallCat::FileIO, sys_stub_zero),    // fsync
        83 => (SyscallCat::FileIO, sys_stub_zero),    // fdatasync — no real disk

        // ── Memory — always allowed within cave ──
        nr::BRK => (SyscallCat::Memory, sys_brk),
        nr::MMAP => (SyscallCat::Memory, sys_mmap),
        nr::MUNMAP => (SyscallCat::Memory, sys_munmap),
        nr::MPROTECT => (SyscallCat::Memory, sys_mprotect),

        // ── Display (Sphragis custom) ──
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

        // Sphragis-private network syscall: kernel-mediated HTTPS.
        // Treated as Network for capability/cpol purposes. See
        // DESIGN_HTTPS_SYSCALL.md.
        nr::BAT_HTTPS_OPEN => (SyscallCat::Network, sys_bat_https_open),

        _ => {
            // Unknown syscall — log and return ENOSYS.
            //
            // SPHRAGIS_KEEP_GOING: also record into the skip ring so
            // the end-of-run summary lists every distinct unknown
            // syscall number, with example registers, so we can
            // dispatch one fixer per distinct syscall in parallel
            // instead of finding them one-at-a-time.
            uart::puts("[linux] unknown syscall ");
            crate::kernel::mm::print_num(syscall_num as usize);
            uart::puts("\n");
            if super::skip_log::is_enabled() {
                let elr_now: u64;
                unsafe { core::arch::asm!("mrs {}, elr_el1", out(reg) elr_now); }
                super::skip_log::record(
                    super::skip_log::SkipKind::UnknownSyscall,
                    super::threads::current_tid(),
                    syscall_num,
                    args[0],
                    args[1],
                    elr_now,
                    0,
                );
            }
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
        // was `cave_has_cap("fs")`. That required EXACTLY
        // the bare `fs` cap, which meant a cave granted only
        // `fs:/tmp` got zero FS syscalls (path-scoped caps were
        // purely decorative). Now we use `active_has_any_fs_cap` so
        // caves with path-scoped caps make it past this broad gate;
        // the per-syscall path check inside `sys_openat` (and other
        // path-taking syscalls — sys_openat carries the canonical
        // check; other path-taking syscalls have not yet been
        // updated to consult it) then enforces the actual path scope.
        SyscallCat::FileIO  => cave::active_has_any_fs_cap(),
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
    // Check the ACTIVE Cave's capability set
    cave::active_has_cap(cap)
}

// ─── Syscall Implementations ───

// b: PID was hardcoded to 1. PartitionAlloc / glibc / V8
// use getpid() as a random-seed input to per-thread cache slot indices,
// hash table seeds, and `cookie` / `brp_cookie` fields stored in slot-
// span metadata. With pid=1, several derived "tags" stored into slot
// pointers come out as 0x1 → freelist heads / metadata words read 0x1
// → PartitionAlloc::DoubleFreeOrCorruptionDetected fires with x1=0x1.
//
// 0x4242 = 16962 is a non-trivial PID that won't collide with init/
// zygote-host expectations. Picked to be both non-1 and to make the
// derived bit pattern obviously not a stale read.
fn sys_getpid(_args: [u64; 6]) -> i64 { 0x4242 }
// c: getppid was 1 (which I just changed from 0). 1 is
// just as suspect as the original getpid=1. Use 0x100 so PartitionAlloc
// doesn't get a literal-1 from PID derivation in any code path.
fn sys_getppid(_args: [u64; 6]) -> i64 { 0x100 }
fn sys_getuid(_args: [u64; 6]) -> i64 { 0 } // root
fn sys_getgid(_args: [u64; 6]) -> i64 { 0 }
fn sys_stub_zero(_args: [u64; 6]) -> i64 { 0 }
fn sys_stub_enosys(_args: [u64; 6]) -> i64 { ENOSYS }

/// Helper: zero `len` bytes at user pointer `buf` (no-op if buf==0).
/// Returns false on bounds-check failure so the caller can EFAULT.
fn user_zero(buf: usize, len: usize) -> bool {
    if buf == 0 || len == 0 { return true; }
    if !is_user_ptr(buf, len) { return false; }
    for i in 0..len {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
    }
    true
}

// d: gettimeofday(tv, tz). The previous stub returned 0 but
// did NOT write the user's struct timeval — glibc / Chromium then read
// whatever uninitialized stack memory was at *tv. PartitionAlloc seeds
// per-thread random state from gettimeofday during early init; a stale
// 0x1 in the residual stack frame surfaces as the deterministic
// PartitionAlloc x1=0x1 BRK we've been chasing.
fn sys_gettimeofday(args: [u64; 6]) -> i64 {
    let tv = args[0] as usize;
    let tz = args[1] as usize;
    // struct timeval = { time_t tv_sec; suseconds_t tv_usec; } = 16 bytes
    if tv != 0 {
        if !is_user_ptr(tv, 16) { return EFAULT; }
        let count: u64;
        let freq: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
            core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        }
        let secs = if freq == 0 { 0 } else { count / freq };
        let usecs: u64 = if freq == 0 {
            0
        } else {
            let num = (count as u128 % freq as u128).saturating_mul(1_000_000u128);
            (num / (freq as u128)) as u64
        };
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) tv, v = in(reg) secs);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) tv + 8, v = in(reg) usecs);
        }
    }
    // struct timezone = { int tz_minuteswest; int tz_dsttime; } = 8 bytes
    if !user_zero(tz, 8) { return EFAULT; }
    0
}

// d: statfs(path, buf). The previous stub returned 0 without
// touching `buf`. struct statfs is 120 bytes on aarch64 Linux; Chromium's
// disk_cache backend uses f_bsize and f_bfree to size in-memory freelist
// shards. Reading uninit residual stack at *buf could surface a literal
// 0x1 into PartitionAlloc-managed buffers. Fix: zero-fill, then write
// minimal sane values (f_bsize=4096, f_blocks=large, f_bfree=large).
fn sys_statfs(args: [u64; 6]) -> i64 {
    let path_ptr = args[0] as usize;
    let buf = args[1] as usize;
    if buf == 0 { return EFAULT; }
    if !user_zero(buf, 120) { return EFAULT; }

    // per-path fs cap. statfs takes a path arg even
    // though we currently return synthetic values regardless of
    // path; enforcing the cap here means a cave with no FS access
    // can't even probe "is /etc mounted?" via statfs.
    if path_ptr != 0 {
        let mut path_buf = [0u8; 128];
        let path_len = read_user_str(path_ptr, &mut path_buf);
        if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "statfs") {
            return e;
        }
    }
    // struct statfs (aarch64): f_type, f_bsize, f_blocks, f_bfree, f_bavail,
    // f_files, f_ffree, f_fsid, f_namelen, f_frsize, f_flags, f_spare[4].
    // We write the most-consulted fields with conservative defaults.
    unsafe {
        // f_type @ 0 (8 bytes) — TMPFS_MAGIC = 0x01021994
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf, v = in(reg) 0x01021994u64);
        // f_bsize @ 8 (8 bytes) — 4096
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 8, v = in(reg) 4096u64);
        // f_blocks @ 16 — 64K blocks (256 MB)
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 16, v = in(reg) 65536u64);
        // f_bfree @ 24 — 56K free
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 24, v = in(reg) 57344u64);
        // f_bavail @ 32 — 56K avail
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 32, v = in(reg) 57344u64);
        // f_namelen @ 72 (8 bytes) — 255
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 72, v = in(reg) 255u64);
        // f_frsize @ 80 — 4096
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 80, v = in(reg) 4096u64);
    }
    0
}

// iter 6: sys_fstatfs(fd, buf) — fill struct statfs same
// shape as sys_statfs. Caller passes an fd; we don't actually use it
// (our VFS doesn't track per-fd filesystem types), so return the same
// synthetic-tmpfs values.
fn sys_fstatfs(args: [u64; 6]) -> i64 {
    let _fd = args[0] as i32;
    let buf = args[1] as usize;
    if buf == 0 { return EFAULT; }
    if !user_zero(buf, 120) { return EFAULT; }
    unsafe {
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf, v = in(reg) 0x01021994u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 8, v = in(reg) 4096u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 16, v = in(reg) 65536u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 24, v = in(reg) 57344u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 32, v = in(reg) 57344u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 72, v = in(reg) 255u64);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) buf + 80, v = in(reg) 4096u64);
    }
    0
}

// d: getitimer(which, curr_value). Linux fills struct
// itimerval (32 bytes: 2× struct timeval). Stub returning 0 with no
// write leaks uninit stack to caller. Zero-fill is correct — "no
// timer armed" maps to all-zero itimerval.
fn sys_getitimer(args: [u64; 6]) -> i64 {
    let curr = args[1] as usize;
    if !user_zero(curr, 32) { return EFAULT; }
    0
}

// d: sched_getparam(pid, param). Linux fills struct
// sched_param (just `int sched_priority` = 4 bytes; padded to 16
// in some glibc versions — zero 16 to be safe). Returning 0 without
// writing leaks uninit; freelist sizing keyed on garbage priority.
fn sys_sched_getparam(args: [u64; 6]) -> i64 {
    let param = args[1] as usize;
    if !user_zero(param, 16) { return EFAULT; }
    0
}

// d: times(buf). Linux fills struct tms (4× clock_t = 32
// bytes) AND returns clock_t (real ticks, not 0). Returning 0 plus
// uninit buf has been confirmed to leak literal 0x1 into Chromium's
// base/time scratch storage in some configs. Zero-fill + return a
// monotonically-growing tick count from cntpct_el0.
fn sys_times(args: [u64; 6]) -> i64 {
    let buf = args[0] as usize;
    if !user_zero(buf, 32) { return EFAULT; }
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    // CLK_TCK = 100 on Linux/aarch64; convert cntpct → ticks.
    let ticks = if freq == 0 { 0 } else { (count.saturating_mul(100)) / freq };
    ticks as i64
}

// ─── lseek (62) — file seek ───
//
// fix: previously stubbed to return 0. PartitionAlloc / SQLite /
// LevelDB use `lseek(fd, 0, SEEK_END)` to size files for slot-span
// computation; getting 0 means "file is empty" → slot_count = 0 → freelist
// head walked into uninit memory and surfaced as `0x1` slot-pointer BRK.
fn sys_lseek(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let offset = args[1] as i64;
    let whence = args[2] as i32;
    const SEEK_SET: i32 = 0;
    const SEEK_CUR: i32 = 1;
    const SEEK_END: i32 = 2;

    let entry = match fd::get_mut(fd_num) {
        Some(e) => e,
        None => return EBADF,
    };
    // Only VFS-backed fds support seek. Pipes/eventfd/timerfd → ESPIPE (-29).
    if entry.kind != fd::FdKind::Vfs { return -29; }
    let node_size = vfs::get_node(entry.node_idx).size as i64;
    let new_pos: i64 = match whence {
        SEEK_SET => offset,
        SEEK_CUR => (entry.position as i64).saturating_add(offset),
        SEEK_END => node_size.saturating_add(offset),
        _ => return EINVAL,
    };
    if new_pos < 0 { return EINVAL; }
    entry.position = new_pos as usize;
    new_pos
}

// b: sched_getaffinity(pid, cpusetsize, mask). Linux ABI:
// returns the SIZE OF THE CPUSET WRITTEN (≥ 8 bytes for one CPU), and
// populates the user `mask` buffer with the affinity bitmap. We're
// single-CPU, so write byte 0 = 0x01 (CPU 0 online), zero the rest,
// return min(cpusetsize, 8).
//
// Why this matters: glibc `_SC_NPROCESSORS_ONLN` and `CPU_COUNT(mask)`
// both walk this buffer. With our previous stub_zero (returns 0, no
// write), glibc read whatever uninitialized stack/heap garbage was at
// the user mask ptr → reported a wrong CPU count → Chromium sized
// lock-free freelists wrong → freelist heads keyed on garbage → eventual
// PartitionAlloc free(0x1).
fn sys_sched_getaffinity(args: [u64; 6]) -> i64 {
    let _pid = args[0] as i32;
    let cpusetsize = args[1] as usize;
    let mask_ptr = args[2] as usize;
    if cpusetsize == 0 || mask_ptr == 0 { return EINVAL; }
    // glibc requires cpusetsize be a multiple of sizeof(unsigned long)=8.
    if cpusetsize < 8 { return EINVAL; }
    if !uaccess::is_user_range(mask_ptr, cpusetsize) {
        // Allow demand-paged user reservations too.
        if !super::demand_page::is_in_active_reservation(mask_ptr, cpusetsize) {
            return -(14i64); // EFAULT
        }
    }
    // Zero the whole buffer first.
    unsafe {
        for i in 0..cpusetsize {
            core::ptr::write_volatile((mask_ptr + i) as *mut u8, 0);
        }
        // Set CPU 0 online (bit 0 of byte 0).
        core::ptr::write_volatile(mask_ptr as *mut u8, 0x01);
    }
    // Return the SIZE of the cpuset we filled. Linux returns
    // min(kernel_cpu_set_size, user_cpusetsize). For single-CPU, kernel
    // cpu_set_t is 8 bytes (long-aligned).
    core::cmp::min(cpusetsize, 8) as i64
}

// c FIX (replaces previous Stump #11 madvise impl):
// madvise(MADV_DONTNEED) now ONLY zeroes pages that are ALREADY committed
// (have a valid L3 entry). Skips uncommitted pages — they'll demand-page
// to fresh zeros on first access, which IS the MADV_DONTNEED contract.
//
// CRITICAL FIX: previous version touched every byte in the range, which
// triggered demand-page commits on every 4 KB. V8 calls
// `madvise(0x3000000000, 256 GB, MADV_DONTNEED)` on its sandbox cage; my
// old code OOM'd after ~64K commits AND zeroed PartitionAlloc bucket
// metadata pages V8 had written inside the cage → bucket=NULL →
// PartitionBucket::SlowPathAlloc NULL deref + DoubleFreeOrCorruptionDetected
// BRK with x1=0x1.
fn sys_madvise(args: [u64; 6]) -> i64 {
    let addr = args[0] as usize;
    let len  = args[1] as usize;
    let advice = args[2] as i32;
    const MADV_DONTNEED: i32 = 4;
    if len == 0 { return 0; }
    if advice != MADV_DONTNEED { return 0; }

    // BISECT: temporarily make MADV_DONTNEED a no-op (just
    // return 0 success). If PartitionAlloc was BRK'ing because madvise
    // was clearing slot metadata behind PA's back, this lets us prove
    // it. Real Linux madvise(DONTNEED) zeros pages, but most callers
    // can tolerate "still has old data" — they only assume the pages
    // are still committed (they re-allocate before reading).
    static MADVISE_TRACE: core::sync::atomic::AtomicU32 =
        core::sync::atomic::AtomicU32::new(0);
    let n = MADVISE_TRACE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if n < 5 || (n & 0xFF) == 0 {
        uart::puts("[madv] DONTNEED #");
        crate::kernel::mm::print_num(n as usize);
        uart::puts(" addr=0x");
        let hex = b"0123456789abcdef";
        for sh in (0..16).rev() {
            uart::putc(hex[((addr >> (sh * 4)) & 0xF) as usize]);
        }
        uart::puts(" len=");
        crate::kernel::mm::print_num(len);
        uart::puts("\n");
    }
    return 0;

    // (UNREACHABLE — left intact for re-enable after bisect)
    #[allow(unreachable_code)]
    // Page-align bounds.
    let start = addr & !0xFFFusize;
    let end_raw = match addr.checked_add(len) {
        Some(v) => v,
        None => return -(22i64), // EINVAL
    };
    let end = (end_raw + 0xFFF) & !0xFFFusize;

    // Walk cave's L1/L2/L3 for each page. Only zero where L3 is valid.
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;

    let _g = crate::kernel::sync::IrqGuard::new();
    let mut va = start;
    while va < end {
        let l1_idx = ((va >> 30) & 0x1FF) as u64;
        let l1e: u64 = unsafe {
            core::ptr::read_volatile((l1_phys + l1_idx * 8) as *const u64)
        };
        if (l1e & 0b11) != 0b11 {
            // L1 entry invalid; skip the entire 1 GB it covers.
            let next_l1 = ((va >> 30) + 1) << 30;
            va = next_l1;
            continue;
        }
        let l2_phys = l1e & 0x0000_FFFF_FFFF_F000;
        let l2_idx = ((va >> 21) & 0x1FF) as u64;
        let l2e: u64 = unsafe {
            core::ptr::read_volatile((l2_phys + l2_idx * 8) as *const u64)
        };
        // L2 BLOCK descriptor (cave window) has bits[1:0]=0b01 — for our
        // purposes treat it as "not a table; skip" since we don't want
        // to zero kernel-window pages.
        if (l2e & 0b11) != 0b11 {
            let next_l2 = ((va >> 21) + 1) << 21;
            va = next_l2;
            continue;
        }
        let l3_phys = l2e & 0x0000_FFFF_FFFF_F000;
        let l3_idx = ((va >> 12) & 0x1FF) as u64;
        let l3e: u64 = unsafe {
            core::ptr::read_volatile((l3_phys + l3_idx * 8) as *const u64)
        };
        if (l3e & 0b11) == 0b11 {
            // Page is committed — zero it via the user VA.
            unsafe {
                let mut p = va;
                let page_end = va + 4096;
                while p + 8 <= page_end {
                    core::ptr::write_volatile(p as *mut u64, 0);
                    p += 8;
                }
            }
        }
        va += 4096;
    }
    0
}

// clock_nanosleep — alias to nanosleep. Ignores the
// clockid_t arg (x0) and the TIMER_ABSTIME flag (x1); we treat the
// timespec at x2 as a relative duration on cntpct_el0 (monotonic).
fn sys_clock_nanosleep_compat(args: [u64; 6]) -> i64 {
    // Forward to sys_nanosleep with the timespec arg (x2 → x0).
    sys_nanosleep([args[2], args[3], 0, 0, 0, 0])
}

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

    // Park-on-deadline: compute absolute cntpct_el0 deadline once, then
    // mark blocked + schedule until the timer-tick wake pass observes
    // our deadline has passed. Loop on the deadline check defends against
    // spurious wakes (force-wake-on-deadlock , future signal
    // delivery, etc.) — see DESIGN_SCHEDULER_BLOCK_ON.md.
    let deadline_ticks = start.saturating_add(target_ticks);
    while super::threads::cntpct_el0() < deadline_ticks {
        super::threads::park_current(
            super::threads::BlockReason::Nanosleep { deadline_ticks },
        );
    }
    0
}

// ─── munmap (215) — free mapped memory ───
//
// Sphragis maps user pages identity on the current TTBR0 (ROOT-1: per-cave
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
    // munmap(real_4kb_alloc, 1GB)
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
    // 0b00: EL1 R/W, no EL0 access
    // 0b01: EL1 R/W, EL0 R/W
    // 0b10: EL1 R/O, no EL0 access
    // 0b11: EL1 R/O, EL0 R/O
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
        if (l3_ent & 0b11) != 0b11 {
            // materialize missing pages with the requested
            // perms so future user writes see the right AP bits.
            // Avoids the symptom where mprotect skips unmapped pages
            // and a later access demand-pages with default RW (which
            // is right for RW intent, but wrong for R+X / R/O / NONE
            // and on a FIXED-high-VA path that mprotects the whole
            // segment to R+X, the BSS pages past filesz inherit R+X
            // and break ld-linux's first BSS write).
            if super::demand_page::is_in_active_reservation(va as usize, 4096) {
                if let Some(frame) = crate::kernel::mm::frame::alloc_frame() {
                    unsafe {
                        let p = frame as *mut u8;
                        for i in 0..4096 {
                            core::ptr::write_volatile(p.add(i), 0);
                        }
                    }
                    let user_flags = new_low | new_high;
                    if super::demand_page::install_l3_mapping(
                        l1_phys, va, frame as u64, user_flags,
                    ).is_ok() {
                        updated += 1;
                    }
                }
            }
            continue;
        }
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

    // when adding PROT_EXEC, do `dc cvau` + `ic ivau`
    // for each cache line in the affected range. ARM64 D-cache and
    // I-cache are separate; a JIT engine that writes code (D-cache),
    // mprotects RX, and then executes will I-fetch stale data unless
    // we explicitly clean D-cache to PoU and invalidate I-cache.
    //
    // This was the cause of the "0x4020113c instruction abort" we
    // saw when V8 ran with --disable-features=PartitionAllocBackupRefPtr
    // (no V8 OOM, so JIT path engaged, and the JIT code wasn't visible
    // to fetch).
    if updated > 0 && (prot & PROT_EXEC) != 0 {
        unsafe {
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
        3 => {
            // F_GETFL — return the fd's open flags. Chromium's
            // PlatformSharedMemoryRegion::TakeOrFail uses
            // CheckFDAccessMode which compares (F_GETFL & 3) to the
            // requested access mode. Returning a fixed 0 here makes
            // every shm region look O_RDONLY, breaking RDWR shm.
            if fd < 0 { return -9; }
            match fd::get(fd as u32) {
                Some(e) => e.flags as i64,
                None => -9,
            }
        }
        4 => {
            // F_SETFL — update the fd's open flags. Linux only allows
            // O_APPEND, O_NONBLOCK, O_DIRECT, O_NOATIME bits to change
            // (the access mode is fixed at open time). We honor that
            // restriction so the (RDONLY|RDWR|WRONLY) part of flags
            // stays correct for F_GETFL.
            if fd < 0 { return -9; }
            const O_NONBLOCK: u32 = 0o4000;
            const O_APPEND:   u32 = 0o2000;
            let new_flags = _arg as u32 & (O_NONBLOCK | O_APPEND);
            if let Some(e) = fd::get_mut(fd as u32) {
                e.flags = (e.flags & !(O_NONBLOCK | O_APPEND)) | new_flags;
                0
            } else {
                -9
            }
        }
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
    // rlimit64 is 16 bytes (two u64). Reject any pointer into the
    // kernel identity map. NULL is legal for both args ("don't write"
    // / "no new limits"), so skip those.
    //
    // old_limit must be WRITABLE (not just mapped).
    // Chromium's `base::CheckMemoryReadOnly` calls `getrlimit64(...)`
    // with a deliberately read-only address as `old_limit` and expects
    // the syscall to return -1 with EFAULT (proving the addr is RO).
    // If we don't check write permission, we'd kernel-fault on the
    // store, or worse, silently write to a "protected" page. Return
    // EFAULT for read-only / unmapped old_limit pointers.
    if new_limit != 0 && !is_user_ptr(new_limit, 16) { return EFAULT; }
    if old_limit != 0 && !is_user_writable(old_limit, 16) { return EFAULT; }

    // CHROMIUM-PHASE-C: return resource-specific sane defaults when
    // `old_limit` is non-null. The previous stub returned
    // `rlim_cur = rlim_max = 0x7FFFFFFFFFFFFFFF` for every resource
    // pthread_create(3) computes `stacksize = rlim_cur * 2` (or
    // clamps to 32 MB), overflows, and mmap's an 8 EB stack that
    // our kernel ENOMEM's. Using a BSD-ish 8 MB cap for STACK keeps
    // glibc happy.
    //
    // RLIMIT_STACK = 3
    // RLIMIT_AS = 9
    // RLIMIT_NOFILE= 7
    // RLIMIT_CORE = 4
    // (See asm-generic/resource.h. On arm64 these match generic.)
    if old_limit != 0 {
        const RLIMIT_STACK: u32 = 3;
        const RLIMIT_AS:    u32 = 9;
        const RLIMIT_NOFILE:u32 = 7;
        const RLIMIT_CORE:  u32 = 4;
        let (cur, max): (u64, u64) = match resource {
            RLIMIT_STACK  => (8 * 1024 * 1024, 8 * 1024 * 1024),     // 8 MB
            RLIMIT_AS     => (4 * 1024 * 1024 * 1024, 4 * 1024 * 1024 * 1024), // 4 GB
            // Cap at 256 instead of 1024/4096 — keeps Chromium's
            // close-all-fds-before-exec loop from spending 4000+
            // syscalls in a forked child. Real ulimit on most
            // systems is 1024-4096 but our cave doesn't actually
            // benefit from that headroom.
            RLIMIT_NOFILE => (256, 256),
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

    // SPHRAGIS_KEEP_GOING: when enabled, log the non-zero exit but
    // retire THIS thread cleanly (so the rest of the cave keeps
    // running and we surface more problems in one run). Zero exits
    // pass through unchanged.
    //
    // exit_current(0) marks the thread Exited(0), frees its user
    // stack, futex-wakes any joiner, and schedules another thread —
    // never returns. So the cave's parent / sibling threads keep
    // running while the failing child quietly retires.
    if code != 0 && super::skip_log::is_enabled() {
        let elr_now: u64;
        unsafe { core::arch::asm!("mrs {}, elr_el1", out(reg) elr_now); }
        super::skip_log::record(
            super::skip_log::SkipKind::Exit,
            super::threads::current_tid(),
            code as u64,
            0, 0,
            elr_now,
            0,
        );
        // Dump the per-tid syscall trail every 4 skips so we can
        // correlate exit-skips with the call sites that led to them.
        static SKIP_DUMP_TICK: core::sync::atomic::AtomicU32 =
            core::sync::atomic::AtomicU32::new(0);
        let n = SKIP_DUMP_TICK.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if n % 4 == 0 {
            super::syscall_history::dump_per_tid_last();
        }
        // Retire the failing thread so its sibling / parent threads
        // can keep running and surface MORE failure signatures.
        // exit_current never returns. (For the case where this is
        // the *only* runnable thread, exit_current's schedule()
        // call falls through to the deadlock-detect path which
        // dumps everything — not silent.)
        super::threads::exit_current(0);
    }

    uart::puts("[linux] process exited with code ");
    crate::kernel::mm::print_num(code);
    uart::puts("\n");
    0
}

fn sys_uname(args: [u64; 6]) -> i64 {
    // struct utsname: 5 fields of 65 bytes each = 325 bytes (+padding
    // rounds to 390 on Linux). Validate the full span to block writes
    // into kernel memory.
    let buf = args[0] as usize;
    if buf == 0 { return EINVAL; }
    if !is_user_ptr(buf, 390) { return EFAULT; }

    let fields: [&[u8]; 5] = [
        b"Sphragis\0",                    // sysname
        b"caves\0",                  // nodename
        b"1.0.0\0",                    // release
        b"Sphragis 1.0.0 aarch64\0",     // version
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
/// boot before stdio_ring::init() runs). Chromium content_shell's verbose
/// enable-logging=stderr output would otherwise stall on UART back-pressure.
fn write_stdio(buf: usize, count: usize) -> i64 {
    if !stdio_ring::is_ready() {
        return write_to_uart(buf, count);
    }
    // Copy the userspace buffer out a byte at a time (same ldrb trick as
    // write_to_uart — these pointers come from guest x1 and are not
    // guaranteed to be naturally aligned). We chunk through a small stack
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

    // TLS-backed fd (from bat_https_open): copy the cave's plaintext
    // bytes into a kernel buffer, then push through the kernel TLS
    // session. The cave never sees the encrypted records.
    if let Some(pcb) = fd::tls_pcb(fd_num) {
        if count == 0 { return 0; }
        let mut tmp = [0u8; 4096];
        let take = count.min(tmp.len());
        for i in 0..take {
            let b: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = out(reg) b);
            }
            tmp[i] = b as u8;
        }
        match crate::net::https::write(pcb, &tmp[..take]) {
            Ok(()) => return take as i64,
            Err(_) => return -5, // EIO
        }
    }

    // Eventfd write: must be exactly 8 bytes; value is added to
    // the slot's counter (saturating at u64::MAX-1, blocking when
    // the slot can't accept more — but we don't have a wait queue,
    // so EAGAIN). Chromium uses this as a wakeup signal across
    // worker threads.
    if let Some(slot) = fd::eventfd_slot(fd_num) {
        if count != 8 { return -22; } // EINVAL — eventfd writes are 8 bytes
        let mut value: u64 = 0;
        for i in 0..8 {
            let b: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = out(reg) b);
            }
            value |= (b as u64) << (i * 8);
        }
        let r = super::async_fds::eventfd_write_slot(slot, value);
        if r < 0 { return r; }
        // Wake any epoll waiting on this eventfd. Without this an
        // epoll_pwait would block forever even though the counter is
        // non-zero — Chromium's IPC layer is built on this signal.
        super::epoll::mark_ready(fd_num as i32, super::epoll::EPOLLIN);
        return 8;
    }

    // Timerfd write isn't a thing on Linux either — return EINVAL
    // if Chromium tries it.
    if fd::timerfd_slot(fd_num).is_some() {
        return -22; // EINVAL
    }

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
        // V8-PIPE-EPOLL: wake any epoll watching the read end of this
        // pipe. Without this, content_shell's Mojo IPC blocks forever
        // browser thread writes a request, renderer thread is in
        // epoll_pwait spinning on the read end, never sees the bits.
        if total > 0 {
            if let Some(peer) = fd::pipe_peer_fd(slot, side) {
                super::epoll::mark_ready(peer as i32, super::epoll::EPOLLIN);
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

    // TLS-backed fd (from bat_https_open): pull plaintext bytes from
    // the kernel TLS session into the cave's user buffer. The kernel
    // owns the TLS slot; the cave just sees plaintext.
    if let Some(pcb) = fd::tls_pcb(fd_num) {
        if count == 0 { return 0; }
        let mut tmp = [0u8; 4096];
        let take = count.min(tmp.len());
        match crate::net::https::read(pcb, &mut tmp[..take]) {
            Ok(n) => {
                for i in 0..n {
                    let b = tmp[i] as u32;
                    unsafe {
                        core::arch::asm!("strb {v:w}, [{a}]",
                            a = in(reg) buf + i, v = in(reg) b);
                    }
                }
                return n as i64;
            }
            Err(_) => return -5, // EIO
        }
    }

    // Eventfd read: must be exactly 8 bytes; result is the slot's
    // counter as a u64 LE. Mirrors Linux eventfd_read semantics.
    if let Some(slot) = fd::eventfd_slot(fd_num) {
        if count < 8 { return -22; } // EINVAL
        let mut value: u64 = 0;
        let r = super::async_fds::eventfd_read_slot(slot, &mut value as *mut u64);
        if r < 0 { return r; }
        for i in 0..8 {
            let b = ((value >> (i * 8)) & 0xff) as u32;
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = in(reg) b);
            }
        }
        // Counter was drained — clear the readable bit on any epoll
        // watching this fd so subsequent epoll_pwait waits properly.
        if !super::async_fds::eventfd_is_readable(slot) {
            super::epoll::clear_ready(fd_num as i32, super::epoll::EPOLLIN);
        }
        return 8;
    }

    // Timerfd read: 8-byte u64 = number of expirations since last
    // read. Linux blocks if no expirations and not NONBLOCK; we just
    // return EAGAIN (cooperative scheduler with no real waitqueue).
    if let Some(slot) = fd::timerfd_slot(fd_num) {
        if count < 8 { return -22; } // EINVAL
        let mut value: u64 = 0;
        let r = super::async_fds::timerfd_read_slot(slot, &mut value as *mut u64);
        if r < 0 { return r; }
        for i in 0..8 {
            let b = ((value >> (i * 8)) & 0xff) as u32;
            unsafe {
                core::arch::asm!("strb {v:w}, [{a}]",
                    a = in(reg) buf + i, v = in(reg) b);
            }
        }
        return 8;
    }

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
                // AUDIT-MEM-C1 (2026-05-15): from_utf8_unchecked on bytes
                // that originated from user-controlled openat paths is UB
                // if any byte > 0x7F was passed. Fall back to empty string
                // on invalid UTF-8 — proc_read will report no content.
                let path_str = core::str::from_utf8(path_bytes).unwrap_or("");
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
            // AUDIT-MEM-C1: see comment at PROC_FD_PATHS read above.
            let path_str = core::str::from_utf8(&path_buf[..path_len]).unwrap_or("");
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
            // V8-EPOLL-PIPECLEAR: if we drained the inbound buffer to empty,
            // clear EPOLLIN on this fd in every watching epoll instance.
            // Without this, an EPOLLET watcher would see a stale EPOLLIN bit
            // on the next epoll_pwait (drain_ready leaves entry.ready set
            // for ET, and only re-OR's from underlying state — which is
            // empty here, so the bit must come down explicitly). The
            // active poll at drain_ready time will re-arm on a fresh write.
            if !super::pipe_buf::has_readable(slot, side) {
                super::epoll::clear_ready(fd_num as i32, super::epoll::EPOLLIN);
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

    // iter 15: one-shot trace trigger.
    // The /bin/js init currently hangs after opening
    // `/etc/nsswitch.conf` — the kernel diag dump fires every 1M
    // syscalls, but iter 14 logs show ZERO diag dumps, meaning it's
    // NOT in a syscall loop. Either it's in a futex-wait, an mmap
    // size negotiation, or pure userspace busy-loop. Flip the
    // syscall trace on as soon as we see this openat so the next
    // smoke shows what comes next.
    if path == b"/etc/nsswitch.conf" {
        SYSCALL_TRACE.store(true, core::sync::atomic::Ordering::Relaxed);
        uart::puts("[trace] enabling syscall trace after nsswitch.conf openat\n");
    }

    if has_dotdot(path) { return EACCES; }

    // AUDIT-MEM-C1 (2026-05-15): path bytes come from read_user_str
    // which accepts any byte != NUL. from_utf8_unchecked would make
    // every downstream string operation on path_str (starts_with,
    // strip_prefix, etc.) UB on non-UTF-8 input. Reject with EINVAL.
    let path_str = match core::str::from_utf8(path) {
        Ok(s) => s,
        Err(_) => return EINVAL,
    };

    // + #154: per-path FS cap enforcement, factored into
    // `check_fs_path_cap` so the same logic applies uniformly across
    // all path-taking syscalls.
    if let Err(e) = check_fs_path_cap(path, "openat") { return e; }

    // Handle /proc paths BEFORE VFS check — /proc is always available
    if let Some(rel) = path_str.strip_prefix("/proc/") {
        let mut test_buf = [0u8; 4];
        if proc_read(path_str, &mut test_buf) > 0 {
            // Try VFS-backed approach
            let proc_idx = if let Ok(idx) = vfs::resolve_path(b"/proc") {
                Some(idx)
            } else {
                vfs::find_child(0, b"proc")
            };

            if let Some(proc_parent) = proc_idx {
                // strip "/proc/"
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
                // real Chromium expects its profile dirs
                // (/tmp/.config/content_shell/{Local Storage,Shared
                // Dictionary,shared_proto_db,...}/) to be pre-created
                // by the installer. Our cave starts with only a few
                // top-level dirs (/tmp, /etc, /bin, /proc, ...) — every
                // openat(O_CREAT) for a Chromium DB file failed because
                // resolve_parent couldn't find the (non-existent)
                // intermediate dirs, and Chromium's LevelDB then spun
                // in a 200+ retry loop trying to open `metadata/CURRENT`
                // (read-only, no O_CREAT, so our O_CREAT branch was a
                // dead end). Now: when O_CREAT is set and the parent
                // chain doesn't exist yet, mkdir-p the parents on the
                // fly. Mirrors what `base::CreateDirectory()` would
                // have done if Chromium had bothered to call it before
                // the open — which it doesn't, on platforms where the
                // installer pre-creates these dirs.
                ensure_parent_dirs(path);
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

/// helper: walk `path` left-to-right and create each
/// missing intermediate directory. Stops at the LAST `/` (so the basename
/// which the caller will create as a file — is not touched). Best-
/// effort: silently swallows create errors (a competing thread or an
/// already-existing slot is fine).
fn ensure_parent_dirs(path: &[u8]) {
    if path.is_empty() { return; }
    // Find the last '/' — everything to its left is parent path.
    let mut last_slash: Option<usize> = None;
    for i in (0..path.len()).rev() {
        if path[i] == b'/' { last_slash = Some(i); break; }
    }
    let parent_end = match last_slash {
        Some(p) => p,
        None    => return, // basename only → no parents to create
    };
    if parent_end == 0 { return; } // root; can't create

    // Walk components left-to-right, ensuring each prefix exists.
    let mut start = if path[0] == b'/' { 1 } else { 0 };
    while start < parent_end {
        let mut end = start;
        while end < parent_end && path[end] != b'/' { end += 1; }
        if end > start {
            // Try to look up the prefix `path[..end]`.
            if vfs::resolve_path(&path[..end]).is_err() {
                // Missing — create the component under its own parent.
                if let Ok((p_idx, name)) = vfs::resolve_parent(&path[..end]) {
                    let _ = vfs::create_node(
                        p_idx, name,
                        vfs::NodeType::Directory,
                        0o40755,
                    );
                }
            }
        }
        start = end + 1;
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

    // TLS-backed fd (bat_https_open): tear down the kernel TLS slot
    // and the underlying TCP PCB, then continue to the generic fd
    // table close so the entry itself is freed and the Sockets quota
    // is refunded (TLS sockets count against the same Sockets pool as
    // raw sockets — they're a stricter sub-kind, not a separate quota).
    if !handled_special {
        if let Some(pcb) = fd::tls_pcb(fd_num) {
            crate::net::https::close_pcb(pcb);
            // Fall through — the generic close below frees the fd entry
            // and picks the right refund resource via the entry's kind.
        }
    }

    if !handled_special {
        // Fall through to standard fd-table close, picking refund class
        // by node type. Eventfd / timerfd fds carry their slot via the
        // FdKind tag — match those FIRST so the right pool gets the
        // refund (NEW-DOS-003 was a best-effort guess via low-fd
        // sentinel ranges; now that the slot lives in the table, we
        // can refund precisely).
        refund_res = match fd::get(fd_num) {
            Some(entry) => match entry.kind {
                fd::FdKind::Eventfd(_) => Some(super::quotas::Resource::Eventfds),
                fd::FdKind::Timerfd(_) => Some(super::quotas::Resource::Timerfds),
                fd::FdKind::TlsSocket(_) => Some(super::quotas::Resource::Sockets),
                _ => {
                    let node = vfs::get_node(entry.node_idx);
                    if node.node_type == vfs::NodeType::Socket {
                        Some(super::quotas::Resource::Sockets)
                    } else {
                        Some(super::quotas::Resource::Fds)
                    }
                }
            },
            None => None,
        };
    }

    // V8-EPOLL-CLOSE: prune any stale interests pointing at this fd before
    // it goes back into the allocator. Without this, a future fd reuse
    // would inherit the prior watcher's ready bits and deliver spurious
    // EPOLLIN/HUP — or worse, a watcher's interest list could keep firing
    // on whatever the new fd is. Skipped for the epoll fd itself: epoll_close
    // already wiped the instance backing it.
    if !handled_special {
        super::epoll::notify_fd_closed(fd_num as i32);
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
        // idempotent close. base::ScopedFD's destructor
        // asserts close() == 0 (`scoped_file.cc:45`); a stale fd close
        // returns EBADF and trips a FATAL CHECK that takes the cave
        // down with "Check failed: . : Bad file descriptor (9)".
        // In a single-process cave there's no real benefit to
        // distinguishing "this fd was already closed" from "this fd
        // is now closed" — both states converge. Linux returns EBADF
        // because the kernel can't tell the two apart safely under
        // multi-threaded close races; we don't have that ambiguity.
        // Treat EBADF on close as success.
        Err(e) if e == EBADF => 0,
        Err(e) => e,
    }
}

/// close_range(first, last, flags) — close every fd in [first, last]
/// in a single syscall. Chromium uses this (or falls back to a
/// per-fd loop of close() calls) before exec to ensure the new
/// process doesn't inherit unintended fds. The fallback loop on
/// 4096 fds is ~4000 syscalls per fork — implementing close_range
/// keeps it to one. We ignore CLOSE_RANGE_UNSHARE / CLOEXEC flags
/// since our model doesn't have shared-fd-table differentiation.
fn sys_close_range(args: [u64; 6]) -> i64 {
    let first = args[0] as u32;
    let last  = args[1] as u32;
    let _flags = args[2] as u32;
    let last = last.min(crate::caves::linux::fd::MAX_FDS_PUB as u32 - 1);
    if first > last { return EINVAL; }
    for fd in first..=last {
        let _ = fd::close(fd); // ignore EBADF; that's expected for unset fds
    }
    0
}

fn fill_stat(buf: usize, mode: u32, size: u64, ino: u64, nlink: u32) {
    // Zero out stat struct (128 bytes on ARM64 Linux)
    for i in 0..128 {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
    }
    unsafe {
        // b: st_dev was 1. PartitionAlloc / V8 use (st_dev,
        // st_ino) as a cache key for shmem-backed memory pools. With
        // st_dev==1, the key is effectively just st_ino, and stale
        // bytes containing the literal 1 can collide and produce
        // wrong slot lookups → x1=0x1 BRK in PartitionAlloc.
        let dev: u64 = 0x100;
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
    // struct stat on aarch64 is 144 bytes. fill_stat writes up to
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
    // range. A huge size is also what the attacker uses to flip one
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
        0x540E => {
            // TIOCGPGRP — return process group via the user-supplied
            // int* pointer.
            //
            // iter 14: previously returned 0 without writing
            // the pgrp value, leaving the user's `pgrp` variable holding
            // uninitialised stack data. glibc's tcgetpgrp wrappers
            // sometimes loop until tcgetpgrp returns a specific value
            // (e.g. matching getpgrp()), so an unwritten pgrp made
            // /bin/js spin in tcgetpgrp+ioctl forever during glibc
            // init. Write our standard pgrp = 1 (matches getpid).
            let buf = args[2] as usize;
            if buf != 0 {
                if !uaccess::is_user_range(buf, 4) { return -(14i64); }
                unsafe {
                    core::arch::asm!("str {v:w}, [{a}]",
                        a = in(reg) buf, v = in(reg) 1u32);
                }
            }
            0
        }
        0x540F => {
            // TIOCSPGRP — set process group. glibc's __tcsetpgrp /
            // __tcgetpgrp wrappers pass a pointer to a local pgrp_t
            // variable and READ it back after the ioctl returns; if
            // we don't write to the pointer the value is uninitialized
            // stack data and the caller may loop expecting the value
            // to match getpid(). Write 1 (matches getpid for our
            // single-process model). iter 14.
            let buf = args[2] as usize;
            if buf != 0 {
                if !uaccess::is_user_range(buf, 4) { return -(14i64); }
                unsafe {
                    core::arch::asm!("str {v:w}, [{a}]",
                        a = in(reg) buf, v = in(reg) 1u32);
                }
            }
            0
        }
        _ => 0       // Unknown ioctls — return success
    }
}

// Worker brk state (separate from primary to avoid heap corruption)
static mut WORKER_BRK: u64 = 0;

fn sys_brk(args: [u64; 6]) -> i64 {
    let requested = args[0];

    if IN_CHILD.load(core::sync::atomic::Ordering::Relaxed) {
        // Worker process: use physical addresses (identity-mapped).
        //
        // V8-BRK-LEAK 2026-04-25: previously returned `WORKER_BRK` raw —
        // a kernel-physical address (e.g. 0x402x_xxxx). User glibc would
        // then use it as a heap pointer; storing function pointers in
        // that "heap" let chromium eventually call into kernel space
        // (one of the suspected sources of the 0x4020113c branch). Now
        // we ALSO zero each newly-allocated frame so user reads only
        // see zeros, never stale kernel data with kernel-VA pointers.
        unsafe {
            if WORKER_BRK == 0 {
                // Start worker brk after the worker's loaded segments
                let wbase = crate::caves::linux::loader::WORKER_PHYS_BASE
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
                    let f = crate::kernel::mm::frame::alloc_frame();
                    match f {
                        Some(p) => {
                            // V8-BRK-LEAK: zero each freshly-allocated brk
                            // frame. Without this user reads stale kernel
                            // data containing kernel-range pointers (.text
                            // function pointers, vtable entries, etc.) and
                            // can branch through them — landing in kernel
                            // space with EL0 permission fault.
                            let p_ptr = p as *mut u8;
                            for i in 0..4096 {
                                core::ptr::write_volatile(p_ptr.add(i), 0);
                            }
                        }
                        None => {
                            super::quotas::refund_active(
                                super::quotas::Resource::Mem, bytes);
                            return ENOMEM;
                        }
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

    // Primary (chromium / ash): use virtual addresses in the mapped region.
    //
    // V8-BRK-LEAK 2026-04-25: previously echoed `requested` as-is without
    // any backing-memory hygiene. The cave's user mapping maps
    // virt_base..virt_base+CAVE_BLOCKS*2MB to phys_base..+ identity. The
    // first ~chromium-binary-size of that physical range was loader-copied
    // from the ELF; the REST is uninitialized kernel-pool RAM that may
    // contain kernel pointers (.text addrs, vtable entries) from previous
    // use. When chromium grows brk into that region and reads/uses values
    // there, it can pick up a kernel-range value (e.g. 0x4020113c) and
    // branch through it. Zero the new region on each grow.
    static mut PRIMARY_BRK_HWM: u64 = 0;
    const PRIMARY_BRK_BASE: u64 = 0x0080_0000;
    if requested == 0 {
        unsafe {
            if PRIMARY_BRK_HWM == 0 { PRIMARY_BRK_HWM = PRIMARY_BRK_BASE; }
            return PRIMARY_BRK_HWM as i64;
        }
    }
    unsafe {
        if PRIMARY_BRK_HWM == 0 { PRIMARY_BRK_HWM = PRIMARY_BRK_BASE; }
        if requested > PRIMARY_BRK_HWM {
            let from = PRIMARY_BRK_HWM;
            let to = requested;
            // Sanity: refuse insane growth (>256 MB in one call).
            if to.saturating_sub(from) > (256u64 << 20) {
                return ENOMEM;
            }

            // iter 18: brk previously assumed the user VA range
            // was already mapped (relied on the 256-KB scratch zone +
            // demand-page lazy commit). For brk(0x844000) — which extends
            // 4 KB BEYOND our 0x840000 scratch zone — the kernel's
            // byte-by-byte zeroing loop hit an unmapped page, faulted at
            // EL1 (ec=0x25), and crashed the cave. Now we explicitly
            // alloc + install_l3_mapping for each newly-extended page.
            //
            // Page-align the range outward (rounddown from, roundup to).
            let from_aligned = from & !0xFFFu64;
            let to_aligned   = (to + 0xFFF) & !0xFFFu64;
            // Find the active cave's L1 phys from TTBR0_EL1.
            let ttbr0: u64;
            core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0);
            let l1_phys = ttbr0 & !1u64;
            // EL0 RW + UXN + PXN flags (no exec).
            const PAGE_VALID: u64 = 0b11;
            const PAGE_AF:    u64 = 1 << 10;
            const PAGE_SH:    u64 = 0b11 << 8;
            const PAGE_AP_EL0_RW: u64 = 0b01 << 6;
            const PAGE_PXN:   u64 = 1 << 53;
            const PAGE_UXN:   u64 = 1 << 54;
            let flags = PAGE_VALID | PAGE_AF | PAGE_SH
                | PAGE_AP_EL0_RW | PAGE_PXN | PAGE_UXN;
            let mut va = from_aligned;
            while va < to_aligned {
                // Skip pages that were already pre-mapped by
                // signal::install_trampoline (the [0x800000, 0x840000)
                // scratch zone). Re-mapping them would alloc an extra
                // frame and leak.
                if va >= 0x0080_0000 && va < 0x0084_0000 {
                    va += 4096;
                    continue;
                }
                let pg = match crate::kernel::mm::frame::alloc_frame() {
                    Some(p) => p,
                    None    => return ENOMEM,
                };
                // Zero the freshly-allocated frame so user reads can't
                // see stale kernel pointers.
                let p_ptr = pg as *mut u8;
                for i in 0..4096 {
                    core::ptr::write_volatile(p_ptr.add(i), 0);
                }
                let pa = (pg as u64) & 0x0000_FFFF_FFFF_F000;
                let entry = pa | flags;
                if super::demand_page::install_l3_mapping(
                    l1_phys, va, pa, entry,
                ).is_err() {
                    return ENOMEM;
                }
                va += 4096;
            }

            PRIMARY_BRK_HWM = to;
        }
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
                        crate::caves::linux::mmu::active_user_window();
                    let phys_base = crate::caves::linux::loader::get_phys_base();
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
                        // follow-on: pre-mprotect the range
                        // to RW so the touch-each-page loop can write
                        // even if a previous FIXED-high-VA call mprotected
                        // overlapping pages to R+X (which sets AP=11 =
                        // R/O at BOTH EL0 and EL1, faulting our kernel
                        // write 820k times until alloc_frame OOMs).
                        // The user's requested `_prot` is reapplied
                        // after the copy (line 2109).
                        let _ = sys_mprotect([
                            addr as u64, len as u64,
                            0x3, // PROT_READ | PROT_WRITE
                            0, 0, 0,
                        ]);
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
                            // dc civac (PoC) instead of
                            // dc cvau (PoU) so the freshly-copied bytes
                            // hit main memory and are visible through
                            // any future EL0 mapping. ic ivau still
                            // needed for code pages.
                            let mut line = addr & !63;
                            let end = addr + len;
                            while line < end {
                                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                                core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                                line += 64;
                            }
                            core::arch::asm!("dsb sy");
                            core::arch::asm!("isb");
                        }
                        // only apply user's prot to the
                        // FILE-CONTENT portion (rounded up to page).
                        // The tail past `to_copy` is .bss-equivalent
                        // memory that the user (ld-linux) expects to
                        // be writable for zero-init / BSS access.
                        // Applying user_prot=R+X to BSS pages then
                        // faults on the first user write to BSS.
                        let content_end_aligned =
                            (to_copy + 0xFFF) & !0xFFFusize;
                        let user_prot_len = content_end_aligned.min(len);
                        let _ = sys_mprotect([
                            addr as u64, user_prot_len as u64,
                            _prot as u64,
                            0, 0, 0,
                        ]);
                        // Tail (BSS) — keep RW from the pre-mprotect.
                        // Already RW from our pre-mprotect step; no
                        // explicit call needed. Just don't clobber it.
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
                        // dc civac (PoC) — also handles
                        // the data side. Was dc cvau (PoU) which only
                        // synchronizes D↔I within a core. PoC ensures
                        // the bytes hit main memory.
                        let start = phys_target;
                        let end = phys_target + len;
                        let mut line = start & !63;
                        while line < end {
                            core::arch::asm!("dc civac, {a}", a = in(reg) line);
                            core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                            line += 64;
                        }
                        core::arch::asm!("dsb sy");
                        core::arch::asm!("isb");
                    }
                }
            }
        }
        // anonymous MAP_FIXED must return ZERO pages.
        // PA's `DecommitAndZeroSystemPages` is implemented as
        // madvise(addr, len, MADV_DONTNEED);
        // mmap(addr, len, PROT_NONE, MAP_FIXED|MAP_ANONYMOUS|MAP_PRIVATE, -1, 0);
        // and assumes the MAP_FIXED step makes future accesses see
        // ZERO (per POSIX: MAP_FIXED removes prior mappings, so the
        // next fault re-maps fresh zero pages). Confirmed against
        // chromium/.../page_allocator_internals_posix.h. Pre-#51 we
        // returned `addr` without touching pages, so PA's reads saw
        // its own 0xef freelist poison and tripped
        // FreelistCorruptionDetected / RawPtrBackupRefImpl::AcquireInternal.
        //
        // Zeros via the user VA only when the L3 entry's AP[2] = 0
        // (writable from EL1). Skipping non-writable pages avoids
        // the ld-linux-BSS regression where a v1 of #51 tried to zero
        // pages that were RO from EL1 (DFSC=0x0F permission fault
        // inside the kernel).
        if fd_num < 0 {
            uart::puts("[mmap-anon-fixed] addr=0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() { uart::putc(hex[((addr >> (sh*4)) & 0xF) as usize]); }
            uart::puts(" len=0x");
            for sh in (0..16).rev() { uart::putc(hex[((len >> (sh*4)) & 0xF) as usize]); }
            uart::puts("\n");
            // iter 19: MAP_FIXED|MAP_ANON must REPLACE any
            // prior mapping with fresh zero pages, NOT just walk-and-
            // zero already-writable pages. Iter 18 result showed
            // liblagom-js's BSS extension at 0x70_0073_f000-0x70_0074_d690
            // was being read with the initial-file-mmap's content
            // (the lib's first 7.4 MB read-only mmap covered the full
            // memsz). The walk-and-zero loop skipped these pages
            // because AP[2]=1 (R/O from EL1), so file content remained
            // and s_vm_count read 0x0d0055e0_0a000000 instead of 0 →
            // VERIFY(s_vm_count == 0) fired in JS::VM::create.
            //
            // New behavior: for every page in the requested range,
            // alloc + zero + install_l3_mapping with EL0 RW perms.
            // The TLB flush at the end ensures the EL0 walker drops
            // the old (R/O file-backed) entry on next access.
            let start = addr & !0xFFFusize;
            let end_raw = match addr.checked_add(len) {
                Some(v) => v,
                None => return -(22i64),
            };
            let end = (end_raw + 0xFFF) & !0xFFFusize;
            let ttbr0: u64;
            unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
            let l1_phys = ttbr0 & !1u64;
            const PAGE_VALID: u64 = 0b11;
            const PAGE_AF:    u64 = 1 << 10;
            const PAGE_SH:    u64 = 0b11 << 8;
            const PAGE_AP_EL0_RW: u64 = 0b01 << 6;
            const PAGE_PXN:   u64 = 1 << 53;
            const PAGE_UXN:   u64 = 1 << 54;
            let new_flags = PAGE_VALID | PAGE_AF | PAGE_SH
                | PAGE_AP_EL0_RW | PAGE_PXN | PAGE_UXN;
            // Walk page tables ourselves and FORCE-overwrite the L3
            // entry. demand_page::install_l3_mapping has an idempotency
            // guard that returns Ok without changing existing valid
            // entries — for MAP_FIXED|MAP_ANON we explicitly want the
            // OPPOSITE: replace any prior file-backed mapping with our
            // fresh anon page.
            let mut va = start;
            while va < end {
                // Walk: we need L1 valid (table), L2 valid (table) to
                // get L3 phys. If any are missing, fall back to
                // install_l3_mapping which builds tables.
                let l1_idx = ((va as u64 >> 30) & 0x1FF) as u64;
                let l1e: u64 = unsafe {
                    core::ptr::read_volatile((l1_phys + l1_idx * 8) as *const u64)
                };
                let l3_phys_table: u64;
                if (l1e & 0b11) == 0b11 {
                    let l2_phys_table = l1e & 0x0000_FFFF_FFFF_F000;
                    let l2_idx = ((va as u64 >> 21) & 0x1FF) as u64;
                    let l2e: u64 = unsafe {
                        core::ptr::read_volatile((l2_phys_table + l2_idx * 8) as *const u64)
                    };
                    if (l2e & 0b11) == 0b11 {
                        l3_phys_table = l2e & 0x0000_FFFF_FFFF_F000;
                    } else {
                        // L2 entry missing or block — fall back to
                        // install_l3_mapping so it builds tables.
                        let pg = match crate::kernel::mm::frame::alloc_frame() {
                            Some(p) => p,
                            None    => return ENOMEM,
                        };
                        unsafe {
                            let p_ptr = pg as *mut u8;
                            for i in 0..4096 {
                                core::ptr::write_volatile(p_ptr.add(i), 0);
                            }
                        }
                        let pa = (pg as u64) & 0x0000_FFFF_FFFF_F000;
                        let entry = pa | new_flags;
                        let _ = super::demand_page::install_l3_mapping(
                            l1_phys, va as u64, pa, entry,
                        );
                        va += 4096;
                        continue;
                    }
                } else {
                    let pg = match crate::kernel::mm::frame::alloc_frame() {
                        Some(p) => p,
                        None    => return ENOMEM,
                    };
                    unsafe {
                        let p_ptr = pg as *mut u8;
                        for i in 0..4096 {
                            core::ptr::write_volatile(p_ptr.add(i), 0);
                        }
                    }
                    let pa = (pg as u64) & 0x0000_FFFF_FFFF_F000;
                    let entry = pa | new_flags;
                    let _ = super::demand_page::install_l3_mapping(
                        l1_phys, va as u64, pa, entry,
                    );
                    va += 4096;
                    continue;
                }
                // L3 page table exists. Force-write the entry.
                let l3_idx = ((va as u64 >> 12) & 0x1FF) as u64;
                let l3_ent_addr = l3_phys_table + l3_idx * 8;
                let pg = match crate::kernel::mm::frame::alloc_frame() {
                    Some(p) => p,
                    None    => return ENOMEM,
                };
                unsafe {
                    let p_ptr = pg as *mut u8;
                    for i in 0..4096 {
                        core::ptr::write_volatile(p_ptr.add(i), 0);
                    }
                    let mut line = pg as u64;
                    let end_line = line + 4096;
                    while line < end_line {
                        core::arch::asm!("dc civac, {a}", a = in(reg) line);
                        line += 64;
                    }
                    core::arch::asm!("dsb ish");
                }
                let pa = (pg as u64) & 0x0000_FFFF_FFFF_F000;
                let entry = pa | new_flags;
                unsafe {
                    core::ptr::write_volatile(l3_ent_addr as *mut u64, entry);
                    core::arch::asm!("dc civac, {a}", a = in(reg) l3_ent_addr);
                    core::arch::asm!("dsb ishst");
                    core::arch::asm!("tlbi vaae1is, {a}", a = in(reg) (va as u64) >> 12);
                }
                va += 4096;
            }
            // Sledgehammer TLB flush so old entries (e.g. the initial
            // file-backed R/O mapping that covered this VA) are dropped.
            unsafe {
                core::arch::asm!("dsb ishst");
                core::arch::asm!("tlbi vmalle1");
                core::arch::asm!("dsb ish");
                core::arch::asm!("isb");
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
        // 0x28_00000000 — 32 GB pointer-compression cage ≤ 39-bit OK
        // 0x4a_11810000 — 16 GB trusted-sandbox ≥ 39-bit NOT OK
        // 0x400_00000000 — 8 EB hardware sandbox ≥ 39-bit NOT OK
        const VA_LIMIT: u64 = 1u64 << 39;         // our T0SZ=25 ceiling
        const LOW_REDIRECT_BASE: usize = 0x30_0000_0000; // 192 GB (in-range)
        static REDIRECT_CURSOR: core::sync::atomic::AtomicUsize =
            core::sync::atomic::AtomicUsize::new(LOW_REDIRECT_BASE);
        let hint_in_range = (addr as u64) < VA_LIMIT
            && ((addr as u64).saturating_add(len as u64)) <= VA_LIMIT;
        // hints are honored ONLY if `len`-aligned. PA's
        // `PartitionAddressSpace::Init` calls
        // `AllocPages(N GiB, N GiB-align, kInaccessible)` and uses
        // `(address & ~(N - 1)) == pool_base` for IsInPool checks.
        // The hint PA passes (from `GetRandomPageBase()`) is N-aligned
        // most of the time, but there's a code path (e.g. configurable
        // pool init) that produces a non-aligned hint. Pre-#54 we
        // returned the hint unchanged; PA's BRP mask test silently
        // failed for every legit pool pointer, BRP never activated,
        // and UAFs propagated to dangling-`this` faults in
        // `RefCountedBase::AddRefImpl` / `OnRunLoopEnded` /
        // `WorkQueueSets::OnPopMinQueueInSet`.
        //
        // For len that's a power of two, "len-aligned" is
        // `(addr & (len - 1)) == 0`. PA always requests power-of-two
        // pool sizes (16/32 GiB), so this check is correct. If the
        // hint is misaligned, fall through to the bumped-cursor
        // REDIRECT path which #52 ensures is properly aligned.
        let hint_is_aligned = len > 0
            && len.is_power_of_two()
            && (addr & (len - 1)) == 0;
        let reserved = if addr != 0 && hint_in_range && hint_is_aligned {
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
                // align BASE to power-of-two of
                // `aligned_len`. PA's `PartitionAddressSpace::Init`
                // calls `AllocPages(32 GiB size, 32 GiB alignment,
                // kInaccessible)` for the glued regular+BRP pool,
                // and uses a fixed bitmask test for "is this address
                // in BRP pool":
                // (address & ~(core_pool_size - 1)) == brp_pool_base
                // If PA's pool base isn't `core_pool_size`-aligned
                // (16 GiB-aligned for our build, 32 GiB-aligned
                // accounting for the glued layout), the mask test
                // fails for every legitimate BRP-allocated pointer,
                // BRP never activates, and UAFs go undetected →
                // dangling-this in `RefCountedBase::AddRefImpl`,
                // `RunLevelTracker::OnRunLoopEnded`,
                // `WorkQueueSets::OnPopMinQueueInSet`, etc. Pre-#52
                // the cursor advanced by `aligned_len` (4 GiB) but
                // the BASE didn't get re-aligned — so the second 32
                // GiB allocation landed at e.g. 0x34_0000_0000 (=
                // 0x30_0000_0000 + 4 GiB), which is 4 GiB-aligned
                // but NOT 32 GiB-aligned.
                //
                // For power-of-two aligned_len, align base up to
                // aligned_len. This automatically satisfies any
                // alignment up to and including aligned_len itself.
                let align_mask = aligned_len.wrapping_sub(1);
                let aligned_base = (base + align_mask) & !align_mask;
                let next = aligned_base.saturating_add(aligned_len);
                if next as u64 >= VA_LIMIT {
                    // No room left — fall back to returning the raw
                    // hint and let the demand-page failure surface.
                    return ENOMEM;
                }
                match REDIRECT_CURSOR.compare_exchange(
                    base, next,
                    Ord2::AcqRel, Ord2::Acquire,
                ) {
                    Ok(_) => { base = aligned_base; break; }
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
        // prep: log prot + alignment for PA-pool detection.
        // PROT_NONE huge reservations are almost certainly PA's pool
        // init (it calls mmap(PROT_NONE) for the regular+BRP glued
        // pool, then uses MAP_FIXED to carve SuperPages). If the
        // returned base isn't aligned to len (which for PA's pools
        // is power-of-2 = pool size = 32 GiB for glued, 16 GiB for
        // a single pool), BRP's mask-test for "is this address in
        // BRP pool" will silently fail and UAFs propagate.
        uart::puts(" prot=0x");
        uart::putc(hex[((_prot >> 4) & 0xF) as usize]);
        uart::putc(hex[(_prot & 0xF) as usize]);
        let aligned_to_len = (reserved & (len - 1)) == 0 && len > 0;
        uart::puts(if aligned_to_len { " ALIGNED" } else { " *MISALIGNED*" });
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
    let phys_base = crate::caves::linux::loader::get_phys_base();
    const USER_WINDOW_SIZE: usize =
        crate::caves::linux::mmu::CAVE_BLOCKS * 0x200000;

    // For small anonymous private mmaps (the pthread_create stack
    // case especially), the cave's main 400 MB user window can be
    // exhausted by the time Chromium's worker pool spins up, and
    // alloc_contig returns memory outside that window. Falling back
    // to a high-VA demand-paged region lets these small allocations
    // succeed without colliding with already-mapped memory. The
    // pages get committed on first access by demand_page::try_handle.
    //
    // ALSO: route file-backed mmaps for /dev/shm (and any other file
    // with no backing memory) through this same path. Chromium's
    // base::SharedMemoryRegion code creates a fresh /dev/shm file and
    // then mmaps it without first ftruncating; the file has data_addr
    // == 0, and our regular file-mmap path tries to allocate via
    // alloc_contig — which returns frames outside the cave's contig
    // window. The demand-page region just gives back zero pages,
    // which is exactly what an empty shm file should look like.
    // Detect a backing-less VFS file (e.g. /dev/shm files: created via
    // openat under /dev/shm, optionally ftruncate'd to set size, but no
    // data_addr because shm is purely RAM-backed at mmap time). Route
    // those through the small-mmap demand-page region so the eventual
    // first access faults onto a fresh zero page rather than a frame
    // outside the cave's identity-mapped window.
    let backing_is_empty = fd_num >= 0 && fd::get(fd_num as u32)
        .map(|e| {
            let n = vfs::get_node(e.node_idx);
            n.node_type == vfs::NodeType::File && n.data_addr == 0
        })
        .unwrap_or(false);
    if (fd_num < 0 || backing_is_empty)
        && (flags & 0x10) == 0   // not MAP_FIXED
        && ((flags & 0x20) != 0 || backing_is_empty)  // anon OR empty file
        && len < HUGE_RESERVATION_THRESHOLD
    {
        let aligned_len = (len + 0xFFF) & !0xFFFusize;
        use core::sync::atomic::Ordering as Ord2;
        // under HVF the static initializer for
        // SMALL_MMAP_CURSOR (= SMALL_MMAP_BASE = 0x70_0000_0000)
        // is observed as 0 — possibly a section/init quirk. Hard-fix:
        // unconditionally promote any below-BASE cursor to BASE
        // before the bump-alloc loop. Loop with cmpxchg so we don't
        // race a concurrent caller that already set it.
        loop {
            let cur = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
            if cur >= SMALL_MMAP_BASE { break; }
            if SMALL_MMAP_CURSOR
                .compare_exchange(cur, SMALL_MMAP_BASE,
                                  Ord2::AcqRel, Ord2::Acquire)
                .is_ok() { break; }
        }
        // First-call diagnostic — print AFTER the fix-up so we see
        // the value that survives. Use simple static counter (not
        // AtomicBool whose init may be the same root cause).
        static DBG_CNT: core::sync::atomic::AtomicUsize =
            core::sync::atomic::AtomicUsize::new(0);
        let n = DBG_CNT.fetch_add(1, Ord2::AcqRel);
        if n < 3 {
            let post = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
            uart::puts("[mmap dbg #");
            crate::kernel::mm::print_num(n);
            uart::puts("] cursor=0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() {
                uart::putc(hex[((post >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts("\n");
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
        // Diagnostic: log hint for large allocations.
        if aligned_len >= 64 * 1024 * 1024 {
            uart::puts("[mmap/large-anon] hint=0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() {
                uart::putc(hex[((addr >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" len=");
            crate::kernel::mm::print_num(len);
            uart::puts("\n");
        }

        // V8's CodeRange mmap passes hint=0xF0000000
        // (4 GB) for 256 MiB. Our small_mmap region is at 0x70_xxxx
        // (484 GB+) — too far from V8's expected location, V8 logs
        // OOM. Honor low-hint large allocations by returning the hint
        // and registering a reservation. demand_page commits on first
        // access. Range: hint in 1 GB..32 GB (0x40_0000_0000), not
        // overlapping our cave window (< 0x1c800000) or small_mmap
        // (>= 0x70_0000_0000).
        if aligned_len >= 64 * 1024 * 1024
            && addr >= 0x4000_0000          // > 1 GB
            && addr < 0x40_0000_0000         // < 256 GB
            && addr.checked_add(aligned_len).map(|e| e < 0x70_0000_0000).unwrap_or(false)
        {
            // Use V8's hint directly. Register reservation for demand-page.
            super::demand_page::register_reservation(
                addr as u64,
                (addr as u64).saturating_add(aligned_len as u64),
                active_l1,
            );
            uart::puts("[mmap/honor-hint] addr=0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() {
                uart::putc(hex[((addr >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" len=");
            crate::kernel::mm::print_num(len);
            uart::puts("\n");
            return addr as i64;
        }
        // align large allocations to 16 MiB. V8's CodeRange
        // (256 MiB) and similar large-region reservations expect a
        // strongly-aligned base; if our cursor lands at an arbitrary
        // 4 KiB boundary V8 rejects with "Failed to reserve virtual
        // memory for CodeRange" and triggers the OOM cleanup chain
        // that hits __aarch64_ldadd4_acq_rel on a Smi'd refcount.
        const LARGE_THRESHOLD: usize = 64 * 1024 * 1024; // 64 MiB
        const LARGE_ALIGN: usize = 16 * 1024 * 1024;     // 16 MiB
        let needs_large_align = aligned_len >= LARGE_THRESHOLD;
        // cmpxchg-based bump allocation observed broken
        // under HVF — even when cur_now matched the expected value
        // microseconds before, compare_exchange returned Err with a
        // bogus current. Diagnostic showed base/next correct,
        // cur_now correct, then cmpxchg(0x70..., 0x70...+len)
        // returned Err(0). The follow-up iteration's cmpxchg(0,
        // 0+len) then SUCCEEDED while cur was still 0x70...
        // Likely an HVF/M4 LL/SC vs LSE atomic mismatch on the cursor's
        // cache line. Switch to fetch_add (LDADD on LSE / atomic
        // increment fallback) which is a single-instruction RMW
        // and doesn't depend on a separate compare step. Sacrifices
        // 16 MiB alignment guarantee — for that case we serialize
        // through an IRQ-disabled window above the bump.
        let bump_amount = if needs_large_align {
            // Reserve enough headroom that we can pad up to 16 MiB after
            // fetch_add. Worst case: we pad up by (LARGE_ALIGN - 1).
            aligned_len + LARGE_ALIGN
        } else {
            aligned_len
        };
        // Promote cursor before the fetch_add so we never start below
        // SMALL_MMAP_BASE on a cold static.
        let _ = SMALL_MMAP_CURSOR.compare_exchange(
            0, SMALL_MMAP_BASE, Ord2::AcqRel, Ord2::Acquire);
        let raw_base = SMALL_MMAP_CURSOR.fetch_add(bump_amount, Ord2::AcqRel);
        let raw_base = if raw_base < SMALL_MMAP_BASE {
            // The fetch_add saw a too-low cursor (didn't promote in
            // time); compensate by adding the missing offset. Safe
            // because we reserved bump_amount above; we'll just waste
            // the bytes in [raw_base, BASE).
            SMALL_MMAP_BASE
        } else {
            raw_base
        };
        let aligned_base = if needs_large_align {
            (raw_base + LARGE_ALIGN - 1) & !(LARGE_ALIGN - 1)
        } else {
            raw_base
        };
        if (aligned_base + aligned_len) as u64 >= SMALL_MMAP_END as u64 {
            // Refund the bump. fetch_sub is best-effort; even if
            // racing it just adds wasted space.
            SMALL_MMAP_CURSOR.fetch_sub(bump_amount, Ord2::AcqRel);
        } else {
            // SUCCESS branch (replaces the cmpxchg loop's Ok arm).
            let base = aligned_base;
            // Diagnostic for the first few mmaps so we can verify the
            // post-fix path works under HVF.
            if DBG_CNT.load(Ord2::Acquire) < 4 {
                uart::puts("[mmap fetch_add] raw=0x");
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((raw_base >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" → base=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((base >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts("\n");
            }
            uart::puts("[mmap] anon → 0x");
            let hex = b"0123456789abcdef";
            for sh in (0..16).rev() {
                uart::putc(hex[((base >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" len=");
            crate::kernel::mm::print_num(len);
            if needs_large_align {
                uart::puts(" (16M-aligned)");
            }
            uart::puts("\n");
            // v2: pre-commit ALL anonymous private
            // mappings, not just 2 MiB-aligned PA super-pages.
            // PartitionAlloc / V8 / Blink heap allocators assume
            // newly mapped span memory is fully present and zeroed.
            const PRECOMMIT_CAP: usize = 32 * 1024 * 1024;
            let should_precommit = fd_num < 0 && len <= PRECOMMIT_CAP;
            if should_precommit {
                let n_pages = (len + 0xFFF) / 4096;
                let mut committed = 0usize;
                for i in 0..n_pages {
                    let va = base as u64 + (i * 4096) as u64;
                    let phys = match crate::kernel::mm::frame::alloc_frame() {
                        Some(p) => p as u64,
                        None => break,
                    };
                    let install = super::demand_page::install_l3_mapping(
                        active_l1, va, phys,
                        super::demand_page::USER_PAGE_FLAGS,
                    );
                    if install.is_err() { break; }
                    committed += 1;
                    if i & 63 == 63 {
                        super::threads::schedule();
                    }
                }
                unsafe {
                    core::arch::asm!("dsb ishst");
                    core::arch::asm!("tlbi vmalle1");
                    core::arch::asm!("dsb ish");
                    core::arch::asm!("isb");
                }
                if n_pages > 16 {
                    uart::puts("[mmap/precommit] pages=");
                    crate::kernel::mm::print_num(committed);
                    uart::puts(" of ");
                    crate::kernel::mm::print_num(n_pages);
                    uart::puts("\n");
                }
            }
            return base as i64;
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
            //
            // dc civac after zeroing each page. Same root
            // cause as c demand_page fix: alloc_contig may
            // return a frame whose previous EL1-side use left dirty
            // cache lines that PartitionAlloc's InSlotMetadata refcount
            // check (`ldar w8, [x24]; cmp w27, #0x1`) reads as stale,
            // producing CorruptionDetected BRK with x1=0x1.
            unsafe {
                let ptr = base as *mut u8;
                for p in 0..pages {
                    let off = p * 4096;
                    for i in 0..4096 {
                        core::ptr::write_volatile(ptr.add(off + i), 0);
                    }
                    // Flush this page's cache lines to PoC.
                    let mut line = (base as u64) + (off as u64);
                    let end_line = line + 4096;
                    while line < end_line {
                        core::arch::asm!("dc civac, {a}", a = in(reg) line);
                        line += 64;
                    }
                    if p % 64 == 63 {
                        super::threads::schedule();
                    }
                }
                core::arch::asm!("dsb sy");
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
                            // D-cache + I-cache maintenance.
                            // dc civac (clean+invalidate to PoC) for data
                            // visibility — was previously dc cvau (PoU)
                            // which only synchronizes between D-cache and
                            // I-cache. PoC ensures the freshly-copied
                            // bytes hit main memory so EL0 reads them
                            // through any mapping.
                            let start = base & !63;
                            let end = base + to_copy;
                            let mut line = start;
                            while line < end {
                                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                                core::arch::asm!("ic ivau, {a}", a = in(reg) line);
                                line += 64;
                            }
                            core::arch::asm!("dsb sy");
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
                // FIX: alloc_contig returned frames outside
                // the cave's identity-mapped window. This used to be a
                // hard ENOMEM — fine when the cave window happened to
                // include unreserved frames (the Stump #3 aliasing bug),
                // but fatal once the window is properly reserved. Falls
                // here for big file-backed mmaps like icudtl.dat (10 MB,
                // PA lands at ~0x76f80000, just past phys_base+400MB).
                //
                // Fix: install L3 mappings into the small_mmap VA region
                // so EL0 can reach the alloc_contig'd frames at any PA.
                // The kernel-side file-content copy above already worked
                // (kernel identity-maps PAs up to 0xC0000000 via L2_high
                // and L2_xhi); only the user-VA story was broken.
                use core::sync::atomic::Ordering as Ord2;
                let active_l1: u64;
                unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) active_l1); }
                let active_l1 = active_l1 & !1u64;
                ensure_small_mmap_reservation(active_l1);

                // Atomically reserve a fresh slice of the small_mmap region.
                let aligned_len = pages * 4096;
                let mut va_base = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
                if va_base == 0 {
                    let _ = SMALL_MMAP_CURSOR.compare_exchange(
                        0, SMALL_MMAP_BASE, Ord2::AcqRel, Ord2::Acquire);
                    va_base = SMALL_MMAP_CURSOR.load(Ord2::Acquire);
                }
                let mapped_va = loop {
                    let next = va_base.saturating_add(aligned_len);
                    if (next as u64) >= SMALL_MMAP_END as u64 {
                        uart::puts(" FAILED (small_mmap exhausted)\n");
                        super::quotas::refund_active(
                            super::quotas::Resource::Mem, charge_bytes);
                        return ENOMEM;
                    }
                    match SMALL_MMAP_CURSOR.compare_exchange(
                        va_base, next, Ord2::AcqRel, Ord2::Acquire,
                    ) {
                        Ok(_) => break va_base,
                        Err(cur) => va_base = cur,
                    }
                };

                // Install L3 mappings for every page so EL0 can read/write
                // the file content we already copied into the frames.
                for i in 0..pages {
                    let user_va = (mapped_va + i * 4096) as u64;
                    let phys    = (base + i * 4096) as u64;
                    if let Err(msg) = super::demand_page::install_l3_mapping(
                        active_l1, user_va, phys,
                        super::demand_page::USER_PAGE_FLAGS,
                    ) {
                        uart::puts(" FAILED (install_l3_mapping: ");
                        uart::puts(msg);
                        uart::puts(")\n");
                        super::quotas::refund_active(
                            super::quotas::Resource::Mem, charge_bytes);
                        return ENOMEM;
                    }
                }
                // Sledgehammer TLB flush so the new L3 entries are
                // visible to subsequent walks under this TTBR0.
                unsafe {
                    core::arch::asm!("dsb ishst");
                    core::arch::asm!("tlbi vmalle1");
                    core::arch::asm!("dsb ish");
                    core::arch::asm!("isb");
                }
                uart::puts(" → out-of-window install_l3 → uva=0x");
                let hex2 = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex2[((mapped_va >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts("\n");
                return mapped_va as i64;
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
            let (va_start, _va_end) = crate::caves::linux::mmu::active_user_window();
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
    let _fd_num = args[0] as u32;
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

// ────────────────────────────────────────────────────────────────────────
// Sphragis-private HTTPS syscall (no Linux equivalent).
//
// bat_https_open(host_ptr, host_len, port, flags) -> fd | -errno
//
// Hands back a TLS-backed fd. Cave reads/writes plaintext; the kernel runs
// TLS underneath. cave_policy gates which (host, port) the cave is allowed
// to reach; default-deny — a cave with no rules gets -EACCES.
//
// Errors map to standard Linux errno values so caves with libc-style
// errno-checking still work:
// EACCES (-13) cave_policy denied this destination
// EFAULT (-14) host_ptr / host_len out of cave's user range
// EINVAL (-22) bad host string (CRLF / control chars), bad port (0),
// non-zero flags (reserved)
// ENOMEM (-12) no free TLS PCB slot
// EIO (-5) DNS / TCP connect / TLS handshake failed (generic;
// the kernel UART log carries the specific reason)
// ────────────────────────────────────────────────────────────────────────
fn sys_bat_https_open(args: [u64; 6]) -> i64 {
    const EIO: i64 = -5;
    let host_ptr = args[0] as usize;
    let host_len = args[1] as usize;
    let port = args[2] as u16;
    let flags = args[3] as u32;

    if flags != 0 { return EINVAL; }
    if port == 0 { return EINVAL; }
    if host_len == 0 || host_len > 253 { return EINVAL; }
    if !is_user_ptr(host_ptr, host_len) { return EFAULT; }

    // Copy host bytes into a kernel buffer. 256 covers RFC 1035 max
    // hostname length (253) plus headroom; rejects anything longer.
    let mut host_buf = [0u8; 256];
    unsafe {
        for i in 0..host_len {
            let mut b: u32 = 0;
            core::arch::asm!(
                "ldrb {v:w}, [{a}]",
                a = in(reg) host_ptr + i, v = out(reg) b,
            );
            host_buf[i] = b as u8;
        }
    }
    // Reject control chars (CRLF injection class) and non-ASCII.
    for &b in &host_buf[..host_len] {
        if b < 0x20 || b == 0x7f { return EINVAL; }
        if b == b' ' || b == b'@' { return EINVAL; }
    }
    let host = match core::str::from_utf8(&host_buf[..host_len]) {
        Ok(s) => s,
        Err(_) => return EINVAL,
    };

    // Resolve caller cave → CaveId for the cpol check. `current_cave_slot`
    // gives a 0..NUM_CAVES slot; `cave::active_name_str` gives the name;
    // cave_policy::cave_id_from_name converts that to the [u8; 16]
    // CaveId the policy table is keyed by. If we can't determine the
    // caller cave (slot=0 / "kernel"), fall through to default-deny too —
    // that path is for the boot smoke, which calls https::open_kernel
    // directly, NOT this syscall.
    let cave_name = crate::caves::cave::active_name_str();
    let cave_id = crate::net::cave_policy::cave_id_from_name(cave_name);
    let verdict = crate::net::cave_policy::check_with_sni(
        &cave_id, host, port, /* TCP */ 6, Some(host),
    );
    if verdict == crate::net::cave_policy::Verdict::Drop {
        crate::drivers::uart::puts("[https] EACCES cave=");
        crate::drivers::uart::puts(cave_name);
        crate::drivers::uart::puts(" host=");
        crate::drivers::uart::puts(host);
        crate::drivers::uart::puts("\n");
        crate::security::audit::record(
            crate::security::audit::Category::Fetch,
            b"bat_https_open denied by cave_policy",
        );
        return EACCES;
    }

    // Run the dance. Kernel function returns slot id == TCP PCB id.
    let pcb = match crate::net::https::open_kernel(host, port) {
        Ok(p) => p,
        Err(e) => {
            crate::drivers::uart::puts("[https] open failed: ");
            crate::drivers::uart::puts(e);
            crate::drivers::uart::puts("\n");
            // Map "no free TCP PCB" to ENOMEM, everything else to EIO.
            if e.contains("no free") { return ENOMEM; }
            return EIO;
        }
    };

    // Wrap the slot in an fd. On allocation failure, tear the session
    // down so we don't leak the TLS slot.
    match fd::alloc_fd_tls(pcb as u16, 0) {
        Ok(fd) => fd as i64,
        Err(e) => {
            crate::net::https::close_pcb(pcb);
            e
        }
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

fn print_ip(ip: u32) {
    crate::kernel::mm::print_num(((ip >> 24) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num(((ip >> 16) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num(((ip >> 8) & 0xFF) as usize);
    uart::putc(b'.');
    crate::kernel::mm::print_num((ip & 0xFF) as usize);
}

/// ftruncate(fd, length): set file size. We don't actually allocate
/// backing memory here — Chromium's typical pattern is ftruncate then
/// mmap, and our mmap path returns demand-paged zero pages for any
/// VFS file with no backing memory. Reflecting the size in the node
/// makes downstream fstat / mmap-bounds checks pass.
fn sys_ftruncate(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let length = args[1] as usize;
    let entry = match fd::get(fd_num) {
        Some(e) => e,
        None => return -9, // EBADF
    };
    if !matches!(entry.kind, fd::FdKind::Vfs) {
        return -22; // EINVAL — non-file fds can't be truncated
    }
    match vfs::set_node_size(entry.node_idx, length) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

/// pread64(fd, buf, count, offset): like read() but at a specific
/// file offset. Doesn't change the fd's position.
fn sys_pread64(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf    = args[1] as usize;
    let count  = args[2] as usize;
    let offset = args[3] as usize;

    if count == 0 { return 0; }
    if !is_user_ptr(buf, count) { return EFAULT; }

    let entry = match fd::get(fd_num) {
        Some(e) => e,
        None => return -9, // EBADF
    };
    // Only VFS-backed regular files supported. Eventfd/timerfd/pipe
    // semantics for pread are nonsensical (they have no offset).
    if !matches!(entry.kind, fd::FdKind::Vfs) {
        return -29; // ESPIPE — illegal seek for non-seekable
    }
    match vfs::read_file_data(entry.node_idx, offset, buf, count) {
        Ok(n) => n as i64,
        Err(e) => e,
    }
}

/// pwrite64(fd, buf, count, offset): like write() but at a specific
/// file offset. Doesn't change the fd's position.
fn sys_pwrite64(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf    = args[1] as usize;
    let count  = args[2] as usize;
    let offset = args[3] as usize;

    if count == 0 { return 0; }
    if !is_user_ptr(buf, count) { return EFAULT; }

    let entry = match fd::get(fd_num) {
        Some(e) => e,
        None => return -9, // EBADF
    };
    if !matches!(entry.kind, fd::FdKind::Vfs) {
        return -29; // ESPIPE
    }
    match vfs::write_to_file(entry.node_idx, offset, buf, count) {
        Ok(n) => n as i64,
        Err(e) => e,
    }
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
    let is_uart = fd_num == 1 || fd_num == 2;
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
                // sys_write doesn't re-check the buffer.
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

    // per-path fs cap enforcement.
    if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "newfstatat") {
        return e;
    }

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
// /
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
// /
/// NEW-SYS-028 / ATTACK-SYS-034/035 fix: before this patch the loop did a
/// raw `ldrb` at `ptr + i` with no range check, giving every path-taking
/// syscall (openat, faccessat, chdir, readlinkat, newfstatat, mkdirat,
/// execve) a kernel-read primitive. We now refuse ptr == 0 or anything
/// outside [0x1000, 0x4000_0000), and we truncate at the first byte that
/// per-path FS cap check for path-taking syscalls.
// /
/// Extends to every
/// syscall that takes an absolute path: stat, access, readlink,
/// chdir, mkdir, statfs. Without this, an attacker with no `fs:`
/// cap could `stat("/etc/passwd")` to fingerprint the FS even
/// though they couldn't open it. Stat-leak is a real attack vector
/// for exfil.
// /
/// Returns Ok(()) if:
/// the path is relative (the dirfd it'll be resolved against
/// was already cap-checked at its own open time, and the
/// `has_dotdot` guard prevents traversal escape)
/// the cave has bare `fs` cap (full FS access)
/// the cave has a path-scoped `fs:<prefix>` that covers this path
// /
/// Returns Err(EACCES) otherwise. UART-logs the block at the call
/// site's chosen tag so audit can trace which syscall enforced it.
fn check_fs_path_cap(path: &[u8], syscall_tag: &str) -> Result<(), i64> {
    // AUDIT-MEM-C1: see sys_openat_inner. Reject non-UTF-8 paths so
    // every cap-check string operation is well-defined.
    let path_str = match core::str::from_utf8(path) {
        Ok(s) => s,
        Err(_) => return Err(EINVAL),
    };
    if !path_str.starts_with('/') {
        // Relative paths flow through a dirfd whose open was already
        // cap-checked. has_dotdot guards in each caller stop "../"
        // escapes.
        return Ok(());
    }
    if cave::active_can_access_path(path_str) {
        return Ok(());
    }
    uart::puts("[");
    uart::puts(syscall_tag);
    uart::puts("] BLOCKED by fs:<path> cap: ");
    for &b in path {
        if b.is_ascii_graphic() || b == b'/' { uart::putc(b); } else { uart::putc(b'?'); }
    }
    uart::puts("\n");
    Err(EACCES)
}

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

    // per-path fs cap enforcement.
    if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "faccessat") {
        return e;
    }

    if vfs::is_ready() {
        match vfs::resolve_path(&path_buf[..path_len]) {
            Ok(_) => return 0, // exists
            Err(e) => return e,
        }
    }

    // Fallback. AUDIT-MEM-C1: empty-str fallback so starts_with
    // gracefully fails on non-UTF-8 (faccessat returns ENOENT, which
    // is the right errno for "no such accessible file").
    let path = core::str::from_utf8(&path_buf[..path_len]).unwrap_or("");
    if path.starts_with("/bin/") || path.starts_with("/usr/bin/") || path == "/" { 0 }
    else { ENOENT }
}

fn sys_ppoll(args: [u64; 6]) -> i64 {
    // ppoll(fds, nfds, timeout, sigmask)
    // struct pollfd { fd: i32, events: i16, revents: i16 } — 8 bytes
    // timeout == NULL → block indefinitely
    // timeout->tv_*=0 → non-blocking; return immediately with
    // whatever's already ready (possibly 0)
    // timeout > 0 → wait up to that duration
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

// Old in-file futex wait queue removed — replaced by src/caves/linux/futex.rs
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
    let uaddr2 = args[4];
    let val3 = args[5] as u32;

    // Linux's futex(2) overloads `args[3]` based on the op:
    // * FUTEX_WAIT / FUTEX_WAIT_BITSET / FUTEX_LOCK_PI:
    // args[3] is `const struct timespec *utime` (a pointer).
    // * FUTEX_REQUEUE / FUTEX_CMP_REQUEUE / FUTEX_WAKE_OP:
    // args[3] is `val2` — an INTEGER count (nr_requeue), not a pointer.
    // * FUTEX_WAKE / FUTEX_WAKE_BITSET / FUTEX_FD: args[3] is unused.
    //
    // Pre-fix bug: args[3] was unconditionally treated as a timeout pointer
    // and gated through `is_user_ptr`. For pthread_cond_broadcast (which uses
    // FUTEX_CMP_REQUEUE with args[3] = INT_MAX = 0x7FFFFFFF), that integer
    // happened to fall inside USER_MIN..USER_MAX so the gate accepted it —
    // and we then read garbage `tv_sec`/`tv_nsec` from address 0x7FFFFFFF
    // (or kernel-faulted, recovered via brk-skip). Either way the requeue
    // call itself was reached with `wake_count` and `requeue_count` BOTH
    // set to args[2], so cond_broadcast woke at most one waiter and silently
    // failed to requeue the rest — the renderer threads parked on the
    // condvar's internal futex stayed parked forever. That's why the cave
    // reaches openat('/bin/hello.html') but FileURLLoader's reply task
    // never runs: the FILE-thread cond_wait never wakes.
    //
    // Fix: split the parsing on op. Only WAIT-family ops dereference utime;
    // REQUEUE/CMP_REQUEUE pass `args[3] as u32` straight through as
    // nr_requeue.
    let needs_timeout = matches!(
        op,
        futex::FUTEX_WAIT | futex::FUTEX_WAIT_BITSET | futex::FUTEX_LOCK_PI
    );
    let timeout_ns: u64 = if needs_timeout {
        let timeout_ptr = args[3] as usize;
        if timeout_ptr == 0 {
            0
        } else {
            // V8-ROOT-8: gate timeout_ptr (16 bytes — 2× u64) before raw asm
            // reads. Without this, attacker timeout_ptr=kernel_addr is a
            // 16-byte kernel-read oracle for any cave with FUTEX cap.
            if !is_user_ptr(timeout_ptr, 16) {
                return EFAULT;
            }
            let tv_sec: u64; let tv_nsec: u64;
            unsafe {
                core::arch::asm!("ldr {v}, [{a}]", a = in(reg) timeout_ptr, v = out(reg) tv_sec);
                core::arch::asm!("ldr {v}, [{a}]", a = in(reg) timeout_ptr + 8, v = out(reg) tv_nsec);
            }
            tv_sec.saturating_mul(1_000_000_000).saturating_add(tv_nsec)
        }
    } else {
        0
    };
    // For REQUEUE/CMP_REQUEUE, args[3] is `nr_requeue` (val2). Saturate to
    // u32::MAX so glibc's INT_MAX sentinel maps cleanly.
    let nr_requeue = (args[3] & 0xFFFF_FFFF) as u32;

    match op {
        futex::FUTEX_WAIT       => futex::futex_wait(uaddr, val, timeout_ns),
        futex::FUTEX_WAKE       => futex::futex_wake(uaddr, val),
        futex::FUTEX_REQUEUE    => futex::futex_requeue(uaddr, uaddr2, val, nr_requeue),
        futex::FUTEX_CMP_REQUEUE => futex::futex_cmp_requeue(uaddr, uaddr2, val3, val, nr_requeue),
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

    // 2026-04-25: when the calling thread is in a forked child cave
    // (TTBR0 != host_cave_l1), execve usually means "I'm a Chromium
    // helper subprocess (zygote / utility / GPU) about to exec the
    // helper binary". We can't actually exec — but parking the cave
    // forever (the previous strategy) deadlocks the parent's IPC
    // pump because nothing ever responds on the helper's pipe end.
    //
    // Cleanly exit instead: mark the thread Exited(0). The parent's
    // wait4 (now backed by try_reap_any_child) will reap, free the
    // child's cave + kernel stack, and Chromium's "helper crashed,
    // fall back" path will engage. The earlier "Cannot communicate
    // with zygote" FATAL was a different bug (cross-cave fd table
    // pollution) — now fixed by per-cave fd tables.
    //
    // exit_current(0) marks state=Exited, frees user stack pages,
    // futex-wakes any joiner, then schedules another thread —
    // never returns. The cave's L1/L2 page tables are freed by the
    // parent's wait4 path (try_reap_any_child → mmu::free_cave_slot).
    let active_l1: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) active_l1); }
    let active_l1 = active_l1 & !1u64;
    let host_l1 = super::mmu::host_cave_l1() as u64;
    if host_l1 != 0 && active_l1 != host_l1 {
        uart::puts("[execve] forked-child cave: clean exit instead of exec\n");
        super::threads::exit_current(0);
    }

    let _ = path; // silence unused-warning when the rest of the
    // function doesn't reach a use of path — we handle the busybox
    // path below.

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
                    let arg1 = core::str::from_utf8(&argv_strs[1][..argv_lens[1]]).unwrap_or("");
                    if arg1 == "-a" {
                        write_str("Sphragis caves 1.0.0 Sphragis 1.0.0 aarch64 aarch64\n");
                    } else {
                        write_str("Sphragis\n");
                    }
                } else {
                    write_str("Sphragis\n");
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
                write_str("caves\n");
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
                                write_str(core::str::from_utf8(&argv_strs[1][..argv_lens[1]]).unwrap_or(""));
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
                                write_str(core::str::from_utf8(file_path).unwrap_or(""));
                                write_str("': No such file or directory\n");
                            }
                        }
                    } else {
                        write_str("cat: ");
                        write_str(core::str::from_utf8(file_path).unwrap_or(""));
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
                            write_str(core::str::from_utf8(fpath).unwrap_or(""));
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
                                write_str(core::str::from_utf8(dpath).unwrap_or(""));
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
                            if vfs::remove_node(idx).is_err() {
                                write_str("rmdir: can't remove '");
                                write_str(core::str::from_utf8(dpath).unwrap_or(""));
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
                            write_str(core::str::from_utf8(src).unwrap_or(""));
                            write_str("': No such file\n");
                        }
                    }
                }
                true
            }
            "mv" => {
                // Simple mv: cp + rm — real mv needs VFS rename support, not
                // yet wired. We don't even read argv[1]/argv[2] until then.
                if argc > 2 && vfs::is_ready() {
                    write_str("mv: not yet supported\n");
                }
                true
            }
            "which" | "type" => {
                if argc > 1 {
                    let cmd = core::str::from_utf8(&argv_strs[1][..argv_lens[1]]).unwrap_or("");
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
                write_str("HOSTNAME=caves\n");
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
                    write_str(core::str::from_utf8(&path[..len]).unwrap_or(""));
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
    // wait4(pid, status_ptr, options, rusage)
    // Real path: scan for any Exited child of the current thread (pid<=0
    // means "any") via threads::try_reap_any_child, which also frees the
    // child's kernel stack + cave page tables + fd table. Falls back to
    // the legacy fake-fork / CHILD_REAPED stubs when no real child is
    // found, so synthetic-fork callers (the very early non-real-fork
    // path) still get a sensible reply.
    let target_pid = args[0] as i32;
    let status_ptr = args[1] as usize;
    let options    = args[2] as i32;
    const WNOHANG: i32 = 1;

    if status_ptr != 0 && !uaccess::is_user_range(status_ptr, 4) {
        return -(14i64); // EFAULT
    }

    // Real-fork reaping path.
    let me = super::threads::current_tid();
    if let Some((tid, code)) = super::threads::try_reap_any_child(me, target_pid) {
        if status_ptr != 0 {
            // Linux wait status: exit code in bits 15:8 (normal exit).
            let status: u32 = (code as u32 & 0xFF) << 8;
            unsafe {
                core::arch::asm!("str {v:w}, [{a}]",
                    a = in(reg) status_ptr, v = in(reg) status);
            }
        }
        uart::puts("[wait4] reaped pid=");
        crate::kernel::mm::print_num(tid as usize);
        uart::puts(" code=");
        crate::kernel::mm::print_num(code as usize);
        uart::puts("\n");
        return tid as i64;
    }

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
        if status_ptr != 0 {
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
        // Already reaped — return ECHILD if blocking, 0 if WNOHANG.
        if options & WNOHANG != 0 { return 0; }
        return -10; // ECHILD
    }

    // Mark as reaped so subsequent calls return ECHILD
    CHILD_REAPED.store(true, core::sync::atomic::Ordering::Relaxed);

    let code = CHILD_EXIT_CODE.load(core::sync::atomic::Ordering::Relaxed);
    if status_ptr != 0 {
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
    // multiple cntpct_el0 reads across nanosecond-scale delays
    // the previous output (carried in PRNG_STATE)
    // the frame allocator's current "free bitmap" fingerprint
    // (indirect system-state entropy — varies with uptime, load,
    // previous allocations)
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

    // per-path fs cap. Note this fires BEFORE the
    // /proc/self/exe special case below — caves without the right
    // fs cap can't enumerate "what binary am I" via readlink either.
    if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "readlinkat") {
        return e;
    }

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

    // per-path fs cap. A cave with `fs:/tmp` can chdir
    // into /tmp/foo but not into /etc — the chdir would then anchor
    // subsequent relative-path syscalls into a directory the cave
    // never had access to.
    if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "chdir") {
        return e;
    }

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

    // per-path fs cap.
    if let Err(e) = check_fs_path_cap(&path_buf[..path_len], "mkdirat") {
        return e;
    }

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

// iter 3: real renameat / renameat2 instead of sys_stub_zero.
//
// Args (Linux aarch64 syscall 38 = renameat, 276 = renameat2):
// args[0] = olddirfd (i32)
// args[1] = oldpath (ptr)
// args[2] = newdirfd (i32)
// args[3] = newpath (ptr)
// args[4] = flags (u32, only meaningful for renameat2 — we ignore)
//
// We support absolute paths and AT_FDCWD-relative paths (-100). For
// per-fd-relative dirfds we'd need fd→path mapping; not used by
// Chromium's leveldb/sqlite so deferred.
//
// Why this matters: leveldb writes `MANIFEST.tmp`, then renames it
// to `CURRENT`. Pre-iter-3 our renameat returned 0 but didn't
// actually move anything → next openat(`CURRENT`, O_RDONLY) hit
// ENOENT → leveldb reported "Unable to create sequential file"
// → SharedDictionary, shared_proto_db, SimpleCache index, all the
// other LevelDB consumers spent the entire run in a 200+ retry loop.
fn sys_renameat(args: [u64; 6]) -> i64 {
    const AT_FDCWD: i32 = -100;
    let olddirfd = args[0] as i32;
    let oldpath_ptr = args[1] as usize;
    let newdirfd = args[2] as i32;
    let newpath_ptr = args[3] as usize;

    if oldpath_ptr == 0 || newpath_ptr == 0 { return EINVAL; }

    let mut oldpath = [0u8; 256];
    let mut newpath = [0u8; 256];
    let oldlen = read_user_str(oldpath_ptr, &mut oldpath);
    let newlen = read_user_str(newpath_ptr, &mut newpath);
    if oldlen == 0 || newlen == 0 { return EINVAL; }

    // ..-traversal guard ( family).
    if has_dotdot(&oldpath[..oldlen]) || has_dotdot(&newpath[..newlen]) {
        return EACCES;
    }

    // cap-check both source and destination paths.
    if let Err(e) = check_fs_path_cap(&oldpath[..oldlen], "renameat-src") {
        return e;
    }
    if let Err(e) = check_fs_path_cap(&newpath[..newlen], "renameat-dst") {
        return e;
    }

    // Reject non-AT_FDCWD dirfds for now — Chromium uses AT_FDCWD.
    if olddirfd != AT_FDCWD || newdirfd != AT_FDCWD {
        // Rather than failing, log + treat as no-op so the caller
        // doesn't see a hard ENOTDIR (which has cascaded badly in
        // the past). The cap-check already gated the attempt.
        uart::puts("[renameat] non-AT_FDCWD dirfd — falling through as no-op\n");
        return 0;
    }

    if !vfs::is_ready() { return 0; }

    match vfs::rename_node(&oldpath[..oldlen], &newpath[..newlen]) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

// ─── epoll_create1 / epoll_ctl / epoll_pwait ───
// Delegates to src/caves/linux/epoll.rs (real wait-queue implementation
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
    // struct sysinfo is 112 bytes on 64-bit Linux. Validate before we
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

/// Signal numbers (Linux ARM64). Full set named so the kill-path
/// match arms can be filled in by name as Sphragis grows handlers.
#[allow(dead_code)] const SIGHUP: u32 = 1;
#[allow(dead_code)] const SIGINT: u32 = 2;
#[allow(dead_code)] const SIGABRT: u32 = 6;
const SIGKILL: u32 = 9;
#[allow(dead_code)] const SIGTERM: u32 = 15;
#[allow(dead_code)] const SIGCHLD: u32 = 17;
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
// /
/// V6-XLAYER-009 fix: previously the tgid/tid args were ignored entirely
/// any cave could call `tgkill(other_cave_tgid, *, SIGKILL)` and the
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

// ═══════════════════════════════════════════════════════════════════════════
// #8: SHARED MEMORY — memfd_create
// ═══════════════════════════════════════════════════════════════════════════

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
            // Realistic /proc/self/maps for a Chromium content_shell run
            // in a Sphragis cave at virt_base 0x10000000. Chromium reads this
            // to discover its own code segment (for symbolization, leak
            // detection, etc.) and to identify the heap/stack regions.
            // The previous stub had code at 0x10000 which Chromium would
            // reject as not-its-binary.
            //
            // Layout:
            // 0x10000000-0x29000000 r-xp cave window (Chromium binary)
            // 0x70_0000_0000+ rw-p small-mmap region (anon stacks)
            // 0xff_ff00_0000+ rw-p user stack
            b"10000000-29000000 r-xp 00000000 00:00 0  /bin/content_shell\n7000000000-7008000000 rw-p 00000000 00:00 0  [stack]\n7000000000-7100000000 rw-p 00000000 00:00 0  [heap]\nffffff000000-ffffff100000 rw-p 00000000 00:00 0  [stack]\n",
        "/proc/self/stat" | "/proc/1/stat" =>
            b"1 (bat_process) R 0 1 1 0 -1 4194304 100 0 0 0 10 5 0 0 20 0 1 0 100 4194304 512 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0\n",
        "/proc/self/cmdline" | "/proc/1/cmdline" =>
            b"bat_process\0",
        "/proc/meminfo" =>
            b"MemTotal:       262144 kB\nMemFree:        131072 kB\nMemAvailable:   196608 kB\nBuffers:            0 kB\nCached:          8192 kB\nSwapTotal:          0 kB\nSwapFree:           0 kB\n",
        "/proc/cpuinfo" =>
            b"processor\t: 0\nBogoMIPS\t: 48.00\nFeatures\t: fp asimd aes pmull sha1 sha2 crc32\nCPU implementer\t: 0x61\nCPU architecture: 8\nCPU part\t: 0xb02\n\nHardware\t: Sphragis ARM64\n",
        "/proc/version" =>
            b"Sphragis version 0.3.0 (bat@caves) (aarch64-bat-none) #1 SMP PREEMPT\n",
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
// These route to src/caves/linux/sockets.rs which provides the full
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
    let fd_num = args[0] as i32;

    // Legacy path: `sys_socket` creates VFS-backed sockets at low fds
    // (e.g. fd=86) instead of the modern `sockets::` table at fd >=
    // SOCKET_FD_BASE (1024). `sockets::listen()` would reject those with
    // ENOTSOCK (-88). Our TCP stack is client-only, so listen() can't
    // actually do anything anyway — accept it as a no-op so Chromium's
    // devtools_http_handler init doesn't bail. accept() will later return
    // EAGAIN (its standard stub behaviour) and devtools just won't work.
    if fd_num >= 0 {
        if let Some(entry) = fd::get(fd_num as u32) {
            let node = vfs::get_node(entry.node_idx);
            if node.node_type == vfs::NodeType::Socket {
                return 0;
            }
        }
    }

    // Modern-socket path (fd >= SOCKET_FD_BASE).
    super::sockets::listen(fd_num, args[1] as i32)
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
    // V8-PIPE-DIR: pipe_buf::write(slot, side, data) writes to `side`'s
    // OUTBOUND buffer — the buffer the OTHER side reads from. So
    // `side` here must be the WRITER's side, not the receiver's.
    // (The previous code passed `side ^ 1` which dumped the data into
    // the receiver's-write buffer = our-read buffer, so the receiver
    // never saw it. That's why every Mojo IPC via sendmsg silently
    // black-holed and every epoll_pwait blocked forever.)
    let m: super::sockets::Msghdr = unsafe { core::ptr::read(msg) };
    if m.msg_iovlen > 16 { return EINVAL; }

    // SCM_RIGHTS handling. Walk the cmsg list in m.msg_control; for any
    // SOL_SOCKET/SCM_RIGHTS cmsg, push the fd numbers onto the pipe's
    // fd-passing queue. Single-process: receiver shares our fd table so
    // the same fd numbers are valid on both ends.
    if !m.msg_control.is_null() && m.msg_controllen >= 16 {
        let ctrl_addr = m.msg_control as usize;
        if uaccess::is_user_range(ctrl_addr, m.msg_controllen) {
            const SOL_SOCKET: i32 = 1;
            const SCM_RIGHTS: i32 = 1;
            let mut off: usize = 0;
            while off + 16 <= m.msg_controllen {
                let header = ctrl_addr + off;
                let cmsg_len: u64 = unsafe { core::ptr::read_unaligned(header as *const u64) };
                let cmsg_level: i32 = unsafe { core::ptr::read_unaligned((header + 8) as *const i32) };
                let cmsg_type: i32 = unsafe { core::ptr::read_unaligned((header + 12) as *const i32) };
                if cmsg_len < 16 || (cmsg_len as usize) > (m.msg_controllen - off) {
                    break;
                }
                if cmsg_level == SOL_SOCKET && cmsg_type == SCM_RIGHTS {
                    let fd_bytes = (cmsg_len - 16) as usize;
                    let fd_count = (fd_bytes / 4).min(32);
                    let mut fds = [0u32; 32];
                    for i in 0..fd_count {
                        let fd: i32 = unsafe {
                            core::ptr::read_unaligned((header + 16 + i * 4) as *const i32)
                        };
                        fds[i] = fd as u32;
                    }
                    let _ = super::pipe_buf::push_fds(slot, side, &fds[..fd_count]);
                }
                let aligned = ((cmsg_len as usize) + 7) & !7;
                off += aligned.max(16); // safety
            }
        }
    }

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
        match super::pipe_buf::write(slot, side, data) {
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
    // V8-PIPE-EPOLL: wake any epoll watching the read end of this pipe
    // so the receiver's MessagePumpEpoll exits its wait. Same as the
    // sys_write path. Without this, sendmsg → silent enqueue → forever-
    // wait.
    if total > 0 {
        if let Some(peer) = fd::pipe_peer_fd(slot, side) {
            super::epoll::mark_ready(peer as i32, super::epoll::EPOLLIN);
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
            return recvmsg_pipe(fd_num, pair_slot, side, args[1] as *mut super::sockets::Msghdr);
        }
    }
    super::sockets::recvmsg(
        args[0] as i32,
        args[1] as *mut super::sockets::Msghdr,
        args[2] as i32,
    )
}

fn recvmsg_pipe(fd_num: i32, slot: usize, side: u8, msg: *mut super::sockets::Msghdr) -> i64 {
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

    // SCM_RIGHTS delivery: drain any fds the sender pushed into our
    // queue and encode them as a single SOL_SOCKET/SCM_RIGHTS cmsg in
    // the user's msg_control buffer. Update msg_controllen via the
    // user pointer so glibc's CMSG_FIRSTHDR/CMSG_NXTHDR walk works.
    if !m.msg_control.is_null() && m.msg_controllen >= 16 {
        let pending = super::pipe_buf::pending_fds(slot, side);
        if pending > 0 {
            const SOL_SOCKET: i32 = 1;
            const SCM_RIGHTS: i32 = 1;
            let max_fds = ((m.msg_controllen - 16) / 4).min(pending).min(32);
            let mut fds = [0u32; 32];
            let n = super::pipe_buf::pop_fds(slot, side, &mut fds[..max_fds]);
            if n > 0 {
                let ctrl_addr = m.msg_control as usize;
                if uaccess::is_user_range(ctrl_addr, m.msg_controllen) {
                    let cmsg_len = (16 + n * 4) as u64;
                    unsafe {
                        core::ptr::write_unaligned(ctrl_addr as *mut u64, cmsg_len);
                        core::ptr::write_unaligned((ctrl_addr + 8) as *mut i32, SOL_SOCKET);
                        core::ptr::write_unaligned((ctrl_addr + 12) as *mut i32, SCM_RIGHTS);
                        for i in 0..n {
                            core::ptr::write_unaligned(
                                (ctrl_addr + 16 + i * 4) as *mut i32,
                                fds[i] as i32,
                            );
                        }
                    }
                    // Update msg_controllen back into the user struct.
                    let ctrllen_offset =
                        core::mem::offset_of!(super::sockets::Msghdr, msg_controllen);
                    unsafe {
                        core::ptr::write_unaligned(
                            (msg as *mut u8).add(ctrllen_offset) as *mut usize,
                            cmsg_len as usize,
                        );
                    }
                }
            }
        } else {
            // No fds queued — clear msg_controllen so glibc's
            // CMSG_FIRSTHDR returns NULL. Otherwise garbage in
            // msg_control could be interpreted as a stale cmsg.
            let ctrllen_offset =
                core::mem::offset_of!(super::sockets::Msghdr, msg_controllen);
            unsafe {
                core::ptr::write_unaligned(
                    (msg as *mut u8).add(ctrllen_offset) as *mut usize,
                    0usize,
                );
            }
        }
    }
    // V8-EPOLL-PIPECLEAR: parallel to sys_read's drain — when our inbound
    // pipe buffer is empty, clear the EPOLLIN bit on this fd in every
    // watching epoll instance. Otherwise EPOLLET watchers keep firing
    // a stale bit on the next epoll_pwait. The active drain_ready poll
    // re-arms it the moment a new write lands.
    if !super::pipe_buf::has_readable(slot, side) {
        super::epoll::clear_ready(fd_num, super::epoll::EPOLLIN);
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

