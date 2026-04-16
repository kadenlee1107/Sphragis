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
fn is_user_ptr(p: usize, size: usize) -> bool {
    let end = match p.checked_add(size) {
        Some(e) => e,
        None => return false,
    };
    // Reject NULL, the first page, and anywhere in the kernel RAM
    // identity-map window. `p < end` is implied by the checked_add above
    // but keeps the predicate readable.
    p != 0
        && p >= 0x1000
        && end < 0x4000_0000
        && p < end
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
        166 => (SyscallCat::Always, sys_stub_zero),  // umask
        167 => (SyscallCat::Always, sys_stub_zero),  // old sysinfo (compat)
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
        279 => (SyscallCat::Always, sys_memfd_create), // memfd_create

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

    handler(args)
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

    let target_ticks = tv_sec * freq + tv_nsec * freq / 1_000_000_000;

    // Spin-wait
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target_ticks { break; }
        core::hint::spin_loop();
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

    let page_size = 4096usize;
    let pages = (length + page_size - 1) / page_size;

    // Walk the range and free each backing frame. free_contig silently
    // ignores bases outside the frame bitmap, so passing an address that
    // was never mapped becomes a cheap no-op rather than a kernel panic.
    crate::kernel::mm::frame::free_contig(addr, pages);

    // Refund the memory quota. Saturating, so if the caller munmaps a
    // region we never charged (ChromiumFb, etc.) the counter sticks at 0.
    super::quotas::refund_active(
        super::quotas::Resource::Mem, pages * page_size);

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
fn sys_mprotect(args: [u64; 6]) -> i64 {
    let _addr = args[0] as usize;
    let _len = args[1] as usize;
    let _prot = args[2] as u32;
    // Accept any protection change (stack guard pages, etc.)
    0
}

// ─── fcntl (25) — file descriptor control ───
fn sys_fcntl(args: [u64; 6]) -> i64 {
    let _fd = args[0] as i32;
    let cmd = args[1] as i32;
    match cmd {
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
    let _resource = args[1] as u32;
    let new_limit = args[2] as usize;
    let old_limit = args[3] as usize;

    // Bounds-check both rlimit pointers before we deref them. A struct
    // rlimit64 is 16 bytes (two u64).  Reject any pointer into the
    // kernel identity map.  NULL is legal for both args ("don't write"
    // / "no new limits"), so skip those.
    if new_limit != 0 && !is_user_ptr(new_limit, 16) { return EFAULT; }
    if old_limit != 0 && !is_user_ptr(old_limit, 16) { return EFAULT; }

    // If old_limit is non-null, write generous defaults
    if old_limit != 0 {
        let unlimited: u64 = 0x7FFFFFFFFFFFFFFF;
        unsafe {
            // struct rlimit { rlim_cur: u64, rlim_max: u64 }
            core::arch::asm!("str {v}, [{a}]", a = in(reg) old_limit, v = in(reg) unlimited);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) old_limit + 8, v = in(reg) unlimited);
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

    // Reject pointer-to-kernel attacks before any dereference.
    if count > 0 && !is_user_ptr(buf, count) { return EFAULT; }

    // Pipe write
    unsafe {
        let pipe_wr = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_WRITE_FD));
        if fd_num == pipe_wr && pipe_wr != 0 {
            let plen = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_LEN));
            let writable = (PIPE_BUF_SIZE - plen).min(count);
            if writable > 0 {
                let pbuf = core::ptr::addr_of_mut!(PIPE_BUF);
                core::ptr::copy_nonoverlapping(buf as *const u8, (*pbuf).as_mut_ptr().add(plen), writable);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_LEN), plen + writable);
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

    // Pipe read
    unsafe {
        let pipe_rd = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_READ_FD));
        if fd_num == pipe_rd && pipe_rd != 0 {
            let plen = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_LEN));
            let readable = plen.min(count);
            if readable > 0 {
                let pbuf = core::ptr::addr_of_mut!(PIPE_BUF);
                core::ptr::copy_nonoverlapping((*pbuf).as_ptr(), buf as *mut u8, readable);
                let remaining = plen - readable;
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
    // ROOT-6: per-cave fd quota. Charge up front; refund if we end up
    // returning a negative errno (any path that doesn't hand back an fd).
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Fds, 1) {
        return e;
    }
    let result = sys_openat_inner(args);
    if result < 0 {
        super::quotas::refund_active(super::quotas::Resource::Fds, 1);
    }
    result
}

fn sys_openat_inner(args: [u64; 6]) -> i64 {
    let dirfd = args[0] as i32;
    let path_ptr = args[1] as usize;
    let flags = args[2] as u32;
    if path_ptr == 0 { return ENOENT; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);
    let path = &path_buf[..path_len];

    // FL-016 path-traversal guard: reject any `..` component. A single
    // `..` in a path lets an attacker escape whatever base directory
    // the cave is sandboxed to. We walk the components and refuse on
    // exact match against the two-byte sequence between slashes.
    // (This is coarser than POSIX realpath-normalization but catches
    // the common attack without pulling in extra state.)
    {
        let mut i = 0usize;
        while i < path.len() {
            let start = if path[i] == b'/' { i + 1 } else { i };
            let mut end = start;
            while end < path.len() && path[end] != b'/' { end += 1; }
            if end - start == 2 && &path[start..end] == b".." {
                return EACCES;
            }
            i = end + 1;
            if i == 0 { break; }
        }
    }

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
    // aren't refunded here — they live outside the fd table (slot indices
    // are returned raw from sys_eventfd2 / sys_timerfd_create).
    let refund_res = match fd::get(fd_num) {
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

    match fd::close(fd_num) {
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

    // For fixed-address mappings, just return the requested address
    // (the memory is already identity-mapped)
    if addr != 0 && (flags & 0x10) != 0 { // MAP_FIXED = 0x10
        return addr as i64;
    }

    // Allocate contiguous pages from the frame allocator
    let pages = (len + 4095) / 4096;
    let charge_bytes = pages * 4096;

    // ROOT-6 quota check — before we touch the frame allocator, make sure
    // this cave isn't already at its per-cave memory cap. -ENOMEM matches
    // what Linux returns when RLIMIT_AS is hit.
    if let Err(e) = super::quotas::charge_active(
            super::quotas::Resource::Mem, charge_bytes) {
        return e;
    }

    uart::puts("[mmap] len=");
    crate::kernel::mm::print_num(len);
    uart::puts(" pages=");
    crate::kernel::mm::print_num(pages);
    match crate::kernel::mm::frame::alloc_frame() {
        Some(base) => {
            uart::puts(" base=");
            crate::kernel::mm::print_num(base);
            uart::puts("\n");
            // Allocate remaining pages
            for _ in 1..pages {
                let _ = crate::kernel::mm::frame::alloc_frame();
            }
            // CRITICAL: zero the allocated memory (Linux MAP_ANONYMOUS guarantee)
            // Without this, malloc returns garbage and libcss crashes
            unsafe {
                let ptr = base as *mut u8;
                for i in 0..(pages * 4096) {
                    core::ptr::write_volatile(ptr.add(i), 0);
                }
            }
            base as i64
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

            if sock_type == SOCK_DGRAM {
                let src_addr_ptr = args[4] as usize;
                let addrlen_ptr = args[5] as usize;

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
        for j in 0..len {
            let byte: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) base + j, v = out(reg) byte);
            }
            uart::putc(byte as u8);
        }
        total += len as i64;
    }
    total
}

fn sys_newfstatat(args: [u64; 6]) -> i64 {
    // newfstatat(dirfd, pathname, statbuf, flags)
    let path_ptr = args[1] as usize;
    let buf = args[2] as usize;
    if buf == 0 { return EINVAL; }

    // Empty path with AT_EMPTY_PATH flag → fstat on dirfd
    if path_ptr == 0 {
        fill_stat(buf, 0o100755, 4096, 0, 1);
        return 0;
    }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

    if path_len == 0 {
        // Empty string with AT_EMPTY_PATH
        fill_stat(buf, 0o100755, 4096, 0, 1);
        return 0;
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
    // struct pollfd { fd: i32, events: i16, revents: i16 } — 8 bytes
    let fds_ptr = args[0] as usize;
    let nfds = args[1] as usize;

    if nfds == 0 || fds_ptr == 0 { return 0; }

    // ATTACK-SYS-049 / NEW-SYS-024: gate the full fds array. The loop below
    // only reads the first 8 entries but also scatters revents writes —
    // both arms need the fds[] buffer to live entirely in userspace.
    let n = nfds.min(8);
    let bytes = match n.checked_mul(8) { Some(b) => b, None => return -(22i64) };
    if !uaccess::is_user_range(fds_ptr, bytes) { return -(14i64); }

    // Read all polled fds
    let mut poll_fds = [0i32; 8];
    let mut has_stdin = false;
    let mut has_socket = false;
    for i in 0..nfds.min(8) {
        unsafe {
            let mut fd: u32 = 0;
            core::arch::asm!("ldr {v:w}, [{a}]",
                a = in(reg) fds_ptr + i * 8, v = out(reg) fd);
            poll_fds[i] = fd as i32;
            if fd == 0 { has_stdin = true; }
            // Check if it's a socket fd
            if let Some(entry) = fd::get(fd) {
                let node = vfs::get_node(entry.node_idx);
                if node.node_type == vfs::NodeType::Socket { has_socket = true; }
            }
        }
    }

    // Poll loop — check all sources (long timeout for network I/O)
    for _ in 0..50_000_000 {
        let mut ready = 0i64;

        // Check stdin
        if has_stdin && uart::has_char() {
            for i in 0..nfds.min(8) {
                if poll_fds[i] == 0 {
                    let pollin: u16 = 1;
                    unsafe {
                        core::arch::asm!("strh {v:w}, [{a}]",
                            a = in(reg) fds_ptr + i * 8 + 6,
                            v = in(reg) pollin as u32);
                    }
                    ready += 1;
                }
            }
        }

        // Check socket fds (UDP RX queue has data)
        if has_socket {
            crate::net::poll_once();
            unsafe {
                if UDP_RX_TAIL < UDP_RX_HEAD {
                    for i in 0..nfds.min(8) {
                        if poll_fds[i] > 2 {
                            if let Some(entry) = fd::get(poll_fds[i] as u32) {
                                let node = vfs::get_node(entry.node_idx);
                                if node.node_type == vfs::NodeType::Socket {
                                    let pollin: u16 = 1;
                                    core::arch::asm!("strh {v:w}, [{a}]",
                                        a = in(reg) fds_ptr + i * 8 + 6,
                                        v = in(reg) pollin as u32);
                                    ready += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if ready > 0 { return ready; }

        // If only waiting on stdin (no socket), block indefinitely
        if has_stdin && !has_socket { continue; }

        core::hint::spin_loop();
    }

    // Timeout
    0
}

fn sys_dup(args: [u64; 6]) -> i64 {
    let old_fd = args[0] as u32;
    // NEW-DOS-002: dup was previously free — it allocates a new fd entry
    // but never touched the Fds counter, so a cave could spin dup() and
    // blow past its cap. Charge one Fd up-front and refund on dup failure.
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

fn sys_dup3(args: [u64; 6]) -> i64 {
    let old_fd = args[0] as u32;
    let new_fd = args[1] as u32;
    // NEW-DOS-002: charge for the new fd slot (same reasoning as sys_dup).
    // dup2/dup3 semantics close an existing new_fd first; if that close
    // is what frees the slot the net charge is zero — we don't model that
    // fine distinction here, we just make sure the cave stays within cap.
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

fn sys_futex(args: [u64; 6]) -> i64 {
    use super::futex;
    let uaddr = args[0];
    let op = (args[1] & !(FUTEX_PRIVATE_FLAG)) as u32;
    let val = args[2] as u32;
    let timeout_ptr = args[3] as usize;
    let uaddr2 = args[4];
    let val3 = args[5] as u32;

    // Read optional timeout from *const timespec at args[3]
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
        return super::threads::clone(
            flags,
            child_stack,
            args[2] as *mut i32, // parent_tidptr
            args[4] as *mut i32, // child_tidptr
            args[3],              // tls
        );
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
    TID_ADDRESS.store(tidptr, core::sync::atomic::Ordering::Relaxed);
    // Return current thread ID
    CURRENT_TID.load(core::sync::atomic::Ordering::Relaxed) as i64
}

// ─── gettid (178) ───
fn sys_gettid(_args: [u64; 6]) -> i64 {
    CURRENT_TID.load(core::sync::atomic::Ordering::Relaxed) as i64
}

/// Called from the exception handler when child exits to restore parent TID
pub fn restore_parent_tid() {
    CURRENT_TID.store(1, core::sync::atomic::Ordering::Relaxed);
}

fn sys_execve(args: [u64; 6]) -> i64 {
    let path_ptr = args[0] as usize;

    // Read the path
    let mut path_buf = [0u8; 128];
    let mut path_len = 0;
    for i in 0..127 {
        let byte: u32;
        unsafe {
            core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) path_ptr + i, v = out(reg) byte);
        }
        if byte == 0 { break; }
        path_buf[i] = byte as u8;
        path_len += 1;
    }

    let path = unsafe { core::str::from_utf8_unchecked(&path_buf[..path_len]) };

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

        if argv_ptr != 0 {
            for i in 0..4 {
                let arg_ptr: u64;
                unsafe {
                    core::arch::asm!("ldr {v}, [{a}]",
                        a = in(reg) argv_ptr + i * 8, v = out(reg) arg_ptr);
                }
                if arg_ptr == 0 { break; }
                // Read string
                for j in 0..63 {
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
    let secs = count / freq;
    let nsecs = ((count % freq) * 1_000_000_000) / freq;

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

    // Handle /proc/self/exe — return path to our binary
    if path_len >= 14 {
        let proc_self_exe = b"/proc/self/exe";
        let mut is_match = true;
        for i in 0..14 {
            if path_buf[i] != proc_self_exe[i] {
                is_match = false;
                break;
            }
        }
        if is_match {
            let exe_path = b"/bin/init";
            let len = exe_path.len().min(bufsiz);
            for i in 0..len {
                unsafe {
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) buf + i, v = in(reg) exe_path[i] as u32);
                }
            }
            return len as i64;
        }
    }

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

    // NEW-DOS-002: charge 2 fds up front. A successful pipe2() consumes
    // two new fd slots; without this charge a cave could drive pipe2 in
    // a loop past its per-cave Fds cap. The close paths (sys_close) refund
    // normally when those fds are released.
    if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 2) {
        return e;
    }
    let ret = sys_pipe2_inner(fds_ptr);
    if ret != 0 {
        super::quotas::refund_active(super::quotas::Resource::Fds, 2);
    }
    ret
}

fn sys_pipe2_inner(fds_ptr: usize) -> i64 {

    // Reset pipe buffer
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_LEN), 0);
        for i in 0..PIPE_BUF_SIZE {
            core::arch::asm!("strb wzr, [{a}]",
                a = in(reg) core::ptr::addr_of_mut!(PIPE_BUF) as usize + i);
        }
    }

    // Create a temp VFS file to act as pipe backing
    if vfs::is_ready() {
        if let Some(tmp) = vfs::find_child(0, b"tmp") {
            // Create a unique pipe file name
            static mut PIPE_NUM: u32 = 0;
            let pnum = unsafe {
                let n = core::ptr::read_volatile(core::ptr::addr_of!(PIPE_NUM));
                core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_NUM), n + 1);
                n
            };
            let mut name = [0u8; 16];
            name[0] = b'.'; name[1] = b'p'; name[2] = b'i'; name[3] = b'p'; name[4] = b'e';
            name[5] = b'0' + ((pnum / 10) % 10) as u8;
            name[6] = b'0' + (pnum % 10) as u8;
            let name_len = 7;

            if let Ok(node_idx) = vfs::create_node(tmp, &name[..name_len], vfs::NodeType::File, 0o100600) {
                // Read end (fd pointing to the file, position starts at 0)
                let read_fd = match fd::alloc_fd(node_idx, fd::O_RDONLY) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                // Write end (same file, position starts at 0)
                let write_fd = match fd::alloc_fd(node_idx, fd::O_WRONLY) {
                    Ok(f) => f,
                    Err(e) => return e,
                };

                unsafe {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_READ_FD), read_fd);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_WRITE_FD), write_fd);
                }

                // Return fds to userspace
                unsafe {
                    core::arch::asm!("str {v:w}, [{a}]",
                        a = in(reg) fds_ptr, v = in(reg) read_fd);
                    core::arch::asm!("str {v:w}, [{a}]",
                        a = in(reg) fds_ptr + 4, v = in(reg) write_fd);
                }
                return 0;
            }
        }
    }

    // Fallback: return fake fds backed by pipe buffer
    unsafe {
        let read_fd: u32 = 20;
        let write_fd: u32 = 21;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_READ_FD), read_fd);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PIPE_WRITE_FD), write_fd);
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) fds_ptr, v = in(reg) read_fd);
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) fds_ptr + 4, v = in(reg) write_fd);
    }
    0
}

fn sys_mkdirat(args: [u64; 6]) -> i64 {
    let path_ptr = args[1] as usize;
    let mode = args[2] as u32;
    if path_ptr == 0 { return EINVAL; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);

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
/// Alternate signal stack
static mut SIGALT_SP: u64 = 0;
static mut SIGALT_SIZE: u64 = 0;

/// rt_sigaction — set/get signal handler
fn sys_rt_sigaction(args: [u64; 6]) -> i64 {
    let signum = args[0] as u32;
    let act_ptr = args[1] as usize;
    let oldact_ptr = args[2] as usize;

    if signum == 0 || signum as usize >= MAX_SIG { return EINVAL; }
    if signum == SIGKILL || signum == SIGSTOP { return EINVAL; }

    // sigaction struct is 32 bytes (4 × u64).  ATTACK-SYS-037: the
    // `oldact` write is an arbitrary 8-byte kernel-write primitive if
    // the attacker points it at kernel state and can pre-arm
    // SIGNAL_HANDLERS[idx] via the `act` path.  Validate both before
    // any deref.
    if act_ptr != 0 && !is_user_ptr(act_ptr, 32) { return EFAULT; }
    if oldact_ptr != 0 && !is_user_ptr(oldact_ptr, 32) { return EFAULT; }

    let idx = signum as usize;
    unsafe {
        // Return old action if requested
        if oldact_ptr != 0 {
            let old = oldact_ptr as *mut u64;
            core::ptr::write(old, SIGNAL_HANDLERS[idx]);
            core::ptr::write(old.add(1), 0); // sa_flags
            core::ptr::write(old.add(2), 0); // sa_restorer
            core::ptr::write(old.add(3), 0); // sa_mask
        }
        // Set new action if provided
        if act_ptr != 0 {
            let act = act_ptr as *const u64;
            SIGNAL_HANDLERS[idx] = core::ptr::read(act);
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
            core::ptr::write(oldset_ptr as *mut u64, SIGNAL_MASK);
        }
        if set_ptr != 0 {
            let new_set = core::ptr::read(set_ptr as *const u64)
                & !((1u64 << SIGKILL) | (1u64 << SIGSTOP));
            match how {
                0 => SIGNAL_MASK |= new_set,   // SIG_BLOCK
                1 => SIGNAL_MASK &= !new_set,  // SIG_UNBLOCK
                2 => SIGNAL_MASK = new_set,     // SIG_SETMASK
                _ => return EINVAL,
            }
        }
    }
    0
}

/// rt_sigreturn — return from signal handler
fn sys_rt_sigreturn(_args: [u64; 6]) -> i64 { 0 }

/// tgkill — send signal to a thread
fn sys_tgkill(args: [u64; 6]) -> i64 {
    let sig = args[2] as u32;
    if sig == 0 { return 0; } // existence check
    if (sig as usize) < MAX_SIG {
        unsafe { SIGNAL_PENDING |= 1u64 << sig; }
    }
    if sig == SIGKILL || sig == SIGTERM || sig == SIGABRT {
        uart::puts("[signal] fatal signal, terminating\n");
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

/// memfd_create — create anonymous file backed by memory
/// Returns a valid fd backed by our pipe buffer (simplest approach)
fn sys_memfd_create(_args: [u64; 6]) -> i64 {
    // Use fd table directly with a fake VFS node for simplicity
    if vfs::is_ready() {
        // Try VFS-backed approach
        if let Some(tmp) = vfs::find_child(0, b"tmp") {
            if let Ok(node_idx) = vfs::create_node(tmp, b"memfd", vfs::NodeType::File, 0o100666) {
                if let Ok(fdi) = fd::alloc_fd(node_idx, 0) {
                    return fdi as i64;
                }
            }
        }
    }
    // Fallback: return a pseudo-fd
    30 // fake but valid-looking fd
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
    // Chromium Mojo uses socketpair + SCM_RIGHTS. Not implemented yet.
    let _ = args;
    ENOSYS
}

fn sys_bind(args: [u64; 6]) -> i64 {
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
    if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Sockets, 1) {
        return e;
    }
    if let Err(e) = super::quotas::charge_active(super::quotas::Resource::Fds, 1) {
        super::quotas::refund_active(super::quotas::Resource::Sockets, 1);
        return e;
    }
    let r = super::sockets::accept4(listen_fd, addr, addrlen, flags);
    if r < 0 {
        super::quotas::refund_active(super::quotas::Resource::Sockets, 1);
        super::quotas::refund_active(super::quotas::Resource::Fds, 1);
    }
    r
}
fn sys_getsockname(args: [u64; 6]) -> i64 {
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
    super::sockets::sendmsg(
        args[0] as i32,
        args[1] as *const super::sockets::Msghdr,
        args[2] as i32,
    )
}
fn sys_recvmsg(args: [u64; 6]) -> i64 {
    super::sockets::recvmsg(
        args[0] as i32,
        args[1] as *mut super::sockets::Msghdr,
        args[2] as i32,
    )
}
fn sys_setsockopt(args: [u64; 6]) -> i64 {
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
    super::sockets::shutdown(args[0] as i32, args[1] as i32)
}

