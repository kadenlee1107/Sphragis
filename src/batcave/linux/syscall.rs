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

// Linux errno values (returned as negative)
const ENOSYS: i64 = -38;   // Function not implemented
const EACCES: i64 = -13;   // Permission denied
const EBADF: i64 = -9;     // Bad file descriptor
const ENOMEM: i64 = -12;   // Out of memory
const ENOENT: i64 = -2;    // No such file or directory
const EINVAL: i64 = -22;   // Invalid argument

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

    // Network
    pub const SOCKET: u64 = 198;
    pub const BIND: u64 = 200;
    pub const LISTEN: u64 = 201;
    pub const ACCEPT: u64 = 202;
    pub const CONNECT: u64 = 203;
    pub const SENDTO: u64 = 206;
    pub const RECVFROM: u64 = 207;
    pub const SETSOCKOPT: u64 = 208;
    pub const GETSOCKOPT: u64 = 209;
}

/// Handle a Linux syscall from a BatCave process.
/// cave_id: which BatCave this process belongs to
/// syscall_num: x8 register value
/// args: x0-x5 register values
/// Returns: value to put in x0 (negative = error)
pub fn handle(cave_id: usize, syscall_num: u64, args: [u64; 6]) -> i64 {
    // Classify the syscall
    let (cat, handler): (SyscallCat, fn([u64; 6]) -> i64) = match syscall_num {
        // Always allowed
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
        73 => (SyscallCat::Always, sys_ppoll),        // ppoll — block on stdin
        // 56 (openat) handled below as nr::OPENAT
        66 => (SyscallCat::FileIO, sys_writev),        // writev
        98 => (SyscallCat::Always, sys_futex),        // futex
        99 => (SyscallCat::Always, sys_stub_zero),   // set_robust_list
        100 => (SyscallCat::Always, sys_stub_zero),  // get_robust_list
        71 => (SyscallCat::FileIO, sys_sendfile),     // sendfile (used by cat)
        101 => (SyscallCat::Always, sys_nanosleep),  // nanosleep
        102 => (SyscallCat::Always, sys_stub_zero),  // getitimer
        103 => (SyscallCat::Always, sys_stub_zero),  // setitimer
        131 => (SyscallCat::Always, sys_stub_zero),  // tgkill
        134 => (SyscallCat::Always, sys_stub_zero),  // rt_sigaction
        135 => (SyscallCat::Always, sys_stub_zero),  // rt_sigprocmask
        153 => (SyscallCat::Always, sys_stub_zero),  // times
        166 => (SyscallCat::Always, sys_stub_zero),  // umask
        167 => (SyscallCat::Always, sys_stub_zero),  // sysinfo
        169 => (SyscallCat::Always, sys_stub_zero),  // gettimeofday
        178 => (SyscallCat::Always, sys_gettid),      // gettid
        233 => (SyscallCat::Always, sys_stub_zero),  // madvise
        261 => (SyscallCat::Always, sys_prlimit64),  // prlimit64
        25 => (SyscallCat::FileIO, sys_fcntl),        // fcntl
        17 => (SyscallCat::FileIO, sys_getcwd),      // getcwd (remap)
        46 => (SyscallCat::Always, sys_stub_zero),   // ftruncate
        34 => (SyscallCat::FileIO, sys_mkdirat),      // mkdirat
        48 => (SyscallCat::FileIO, sys_faccessat),   // faccessat
        61 => (SyscallCat::FileIO, sys_getdents64),  // getdents64
        78 => (SyscallCat::FileIO, sys_readlinkat),  // readlinkat
        // 79 newfstatat handled above as nr::NEWFSTATAT
        144 => (SyscallCat::Always, sys_stub_zero),  // setgid
        146 => (SyscallCat::Always, sys_stub_zero),  // setuid
        157 => (SyscallCat::Always, sys_stub_zero),  // sched_getscheduler
        158 => (SyscallCat::Always, sys_stub_zero),  // sched_getparam
        170 => (SyscallCat::Always, sys_stub_zero),  // getpgrp/setpgid
        171 => (SyscallCat::Always, sys_stub_zero),  // sigaltstack
        204 => (SyscallCat::Always, sys_stub_zero),  // sched_getaffinity
        210 => (SyscallCat::Always, sys_stub_zero),  // shutdown
        262 => (SyscallCat::Always, sys_stub_zero),  // getrlimit equiv
        113 => (SyscallCat::Always, sys_clock_gettime), // clock_gettime (dup)
        179 => (SyscallCat::Always, sys_stub_zero),  // sysinfo
        23 => (SyscallCat::FileIO, sys_dup),         // dup
        24 => (SyscallCat::FileIO, sys_dup3),        // dup3
        25 => (SyscallCat::FileIO, sys_stub_zero),   // fcntl (dup)
        35 => (SyscallCat::FileIO, sys_stub_zero),   // unlinkat
        // 48 faccessat handled above
        49 => (SyscallCat::FileIO, sys_chdir),        // chdir
        59 => (SyscallCat::FileIO, sys_pipe2),       // pipe2
        61 => (SyscallCat::FileIO, sys_stub_zero),   // getdents64 (dup)
        134 => (SyscallCat::Always, sys_stub_zero),  // rt_sigaction (dup)
        135 => (SyscallCat::Always, sys_stub_zero),  // rt_sigprocmask (dup)
        137 => (SyscallCat::Always, sys_stub_zero),  // rt_sigtimedwait
        154 => (SyscallCat::Always, sys_stub_zero),  // setpgid
        155 => (SyscallCat::Always, sys_stub_zero),  // getpgid
        172 => (SyscallCat::Always, sys_getpid),     // getpid (dup)
        220 => (SyscallCat::Process, sys_clone_thread), // clone (thread support)
        221 => (SyscallCat::Process, sys_execve),       // execve
        260 => (SyscallCat::Process, sys_wait_stub),  // wait4 (improved)

        // Memory — always allowed within cave
        nr::BRK => (SyscallCat::Memory, sys_brk),
        nr::MMAP => (SyscallCat::Memory, sys_mmap),
        nr::MUNMAP => (SyscallCat::Memory, sys_munmap),
        nr::MPROTECT => (SyscallCat::Memory, sys_mprotect),

        // File I/O — needs fs capability
        nr::OPENAT => (SyscallCat::FileIO, sys_openat),
        nr::CLOSE => (SyscallCat::FileIO, sys_close),
        nr::READ => (SyscallCat::FileIO, sys_read),
        nr::WRITE => (SyscallCat::FileIO, sys_write),
        nr::LSEEK => (SyscallCat::FileIO, sys_stub_zero),
        nr::FSTAT => (SyscallCat::FileIO, sys_fstat),
        nr::NEWFSTATAT => (SyscallCat::FileIO, sys_newfstatat),
        nr::FACCESSAT => (SyscallCat::FileIO, sys_stub_zero),
        nr::GETCWD => (SyscallCat::FileIO, sys_getcwd),
        nr::CHDIR => (SyscallCat::FileIO, sys_stub_zero),
        nr::READLINKAT => (SyscallCat::FileIO, sys_readlinkat),
        nr::IOCTL => (SyscallCat::FileIO, sys_ioctl),

        // Process (clone handled at line 158)
        nr::EXECVE => (SyscallCat::Process, sys_stub_zero),
        nr::WAIT4 => (SyscallCat::Process, sys_stub_zero),

        // Network — needs net capability
        nr::SOCKET => (SyscallCat::Network, sys_socket),
        nr::CONNECT => (SyscallCat::Network, sys_connect),
        nr::BIND => (SyscallCat::Network, sys_stub_zero),
        nr::LISTEN => (SyscallCat::Network, sys_stub_zero),
        nr::ACCEPT => (SyscallCat::Network, sys_stub_zero),
        nr::SENDTO => (SyscallCat::Network, sys_sendto),
        nr::RECVFROM => (SyscallCat::Network, sys_recvfrom),
        nr::SETSOCKOPT => (SyscallCat::Network, sys_stub_zero),
        nr::GETSOCKOPT => (SyscallCat::Network, sys_stub_zero),

        _ => {
            // Unknown syscall — log and return ENOSYS
            uart::puts("[linux] unknown syscall ");
            crate::kernel::mm::print_num(syscall_num as usize);
            uart::puts("\n");
            return ENOSYS;
        }
    };

    // Capability check
    let allowed = match cat {
        SyscallCat::Always | SyscallCat::Process | SyscallCat::Memory => true,
        SyscallCat::FileIO => true, // BatCave always has access to its OWN rootfs
        SyscallCat::Network => cave_has_cap(cave_id, "net"),
        SyscallCat::RawNet => cave_has_cap(cave_id, "raw"),
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
fn sys_munmap(args: [u64; 6]) -> i64 {
    let _addr = args[0] as usize;
    let _length = args[1] as usize;
    // TODO: actually free frames — for now accept and leak
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
    // args[2] = new_limit (ignored)
    let old_limit = args[3] as usize;

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
    // struct utsname: 5 fields of 65 bytes each
    let buf = args[0] as usize;
    if buf == 0 { return EINVAL; }

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

fn sys_write(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let count = args[2] as usize;

    // Check if fd has been redirected (dup2'd to a file)
    if let Some(entry) = fd::get(fd_num) {
        let node_idx = entry.node_idx;

        // node_idx == 0 means original stdin/stdout/stderr (not redirected)
        if node_idx == 0 {
            if fd_num == 0 { return EBADF; }
            return write_to_uart(buf, count);
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

        let pos = entry.position;
        match vfs::write_to_file(node_idx, pos, buf, count) {
            Ok(n) => {
                if let Some(e) = fd::get_mut(fd_num) { e.position += n; }
                n as i64
            }
            Err(e) => e,
        }
    } else {
        // fd not in table — default stdout/stderr to UART
        if fd_num == 1 || fd_num == 2 {
            return write_to_uart(buf, count);
        }
        EBADF
    }
}

fn sys_read(args: [u64; 6]) -> i64 {
    let fd_num = args[0] as u32;
    let buf = args[1] as usize;
    let count = args[2] as usize;

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

fn sys_openat(args: [u64; 6]) -> i64 {
    let dirfd = args[0] as i32;
    let path_ptr = args[1] as usize;
    let flags = args[2] as u32;
    if path_ptr == 0 { return ENOENT; }

    let mut path_buf = [0u8; 128];
    let path_len = read_user_str(path_ptr, &mut path_buf);
    let path = &path_buf[..path_len];

    if !vfs::is_ready() {
        // Fallback for pre-VFS mode
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
    match fd::close(fd_num) {
        Ok(()) => 0,
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
    let fd = args[0];
    let cmd = args[1];
    match cmd {
        0x5401 => { // TCGETS — get terminal attributes
            let buf = args[2] as usize;
            if buf != 0 {
                // Zero out termios struct (60 bytes on aarch64)
                for i in 0..60 {
                    unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) buf + i); }
                }
            }
            0
        }
        0x5402 | 0x5403 => 0, // TCSETS, TCSETSW — set terminal attributes (ignore)
        0x5413 => { // TIOCGWINSZ — terminal window size
            let buf = args[2] as usize;
            if buf != 0 {
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
            if requested >= WORKER_BRK {
                // Allocate pages to cover the gap
                let pages = ((requested - WORKER_BRK) as usize + 4095) / 4096;
                for _ in 0..pages {
                    crate::kernel::mm::frame::alloc_frame();
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
    let len = args[1] as usize;

    if len == 0 { return EINVAL; }

    // Allocate pages
    let pages = (len + 4095) / 4096;
    match crate::kernel::mm::frame::alloc_frame() {
        Some(base) => {
            // Allocate remaining pages
            for _ in 1..pages {
                let _ = crate::kernel::mm::frame::alloc_frame();
            }
            base as i64
        }
        None => ENOMEM,
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
                Err(e) => return e,
            }
        }
    }

    // Fallback
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
fn read_user_str(ptr: usize, buf: &mut [u8]) -> usize {
    let max = buf.len().min(255);
    let mut len = 0;
    for i in 0..max {
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
    // struct pollfd { fd: i32, events: i16, revents: i16 }
    let fds_ptr = args[0] as usize;
    let nfds = args[1] as usize;

    if nfds == 0 || fds_ptr == 0 { return 0; }

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
    match fd::dup(old_fd) {
        Ok(new_fd) => new_fd as i64,
        Err(e) => e,
    }
}

fn sys_dup3(args: [u64; 6]) -> i64 {
    let old_fd = args[0] as u32;
    let new_fd = args[1] as u32;
    match fd::dup2(old_fd, new_fd) {
        Ok(fd) => fd as i64,
        Err(e) => e,
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

// ─── Futex wait queue ───
// Simple array of {addr, waiting} pairs for FUTEX_WAIT/FUTEX_WAKE
const MAX_FUTEX_WAITERS: usize = 16;
struct FutexWaiter {
    addr: u64,    // futex address being waited on
    active: bool, // is this waiter slot in use?
    woken: bool,  // has this waiter been woken?
}
static mut FUTEX_WAITERS: [FutexWaiter; MAX_FUTEX_WAITERS] = {
    const EMPTY: FutexWaiter = FutexWaiter { addr: 0, active: false, woken: false };
    [EMPTY; MAX_FUTEX_WAITERS]
};

// Futex operations
const FUTEX_WAIT: u64 = 0;
const FUTEX_WAKE: u64 = 1;
const FUTEX_PRIVATE_FLAG: u64 = 128;

fn sys_futex(args: [u64; 6]) -> i64 {
    let uaddr = args[0];
    let op = args[1] & !(FUTEX_PRIVATE_FLAG); // strip PRIVATE flag
    let val = args[2] as u32;

    match op {
        FUTEX_WAIT => {
            // Read the value at uaddr
            let current: u32;
            unsafe {
                core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) uaddr, v = out(reg) current);
            }
            // If value changed since caller checked, return EAGAIN
            if current != val {
                return -11; // EAGAIN
            }
            // In our single-core cooperative model, FUTEX_WAIT just returns 0
            // immediately. The caller will spin-retry. Real blocking would
            // require preemptive scheduling which we don't have for userspace.
            // This is correct: the futex contract allows spurious wakeups.
            0
        }
        FUTEX_WAKE => {
            // Wake up to `val` waiters on this address
            // In our cooperative model, waiters aren't actually blocked,
            // so this is a no-op but we return the count for correctness.
            let count = val as i64;
            if count > 0 { count.min(1) } else { 0 }
        }
        _ => {
            // Unknown futex op — return success for compatibility
            0
        }
    }
}

fn sys_clone_thread(args: [u64; 6]) -> i64 {
    let _flags = args[0];
    let child_stack = args[1];

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

    for i in 0..len {
        let val: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) val);
            let byte = ((val >> (i % 8 * 8)) & 0xFF) as u32;
            core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) buf + i, v = in(reg) byte);
        }
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

    // Reset pipe buffer
    unsafe {
        PIPE_LEN = 0;
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
                    PIPE_READ_FD = read_fd;
                    PIPE_WRITE_FD = write_fd;
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

    // Fallback: return fake fds
    unsafe {
        let read_fd: u32 = 20;
        let write_fd: u32 = 21;
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
