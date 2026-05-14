#![allow(dead_code)]
// Sphragis — System Call Handler
// Tasks use SVC instruction to request kernel services.
// Each syscall is capability-checked before execution.

use crate::kernel::arch::TrapFrame;
use crate::drivers::uart;

// Syscall numbers
pub const SYS_YIELD: u16 = 0;
pub const SYS_SEND:  u16 = 1;
pub const SYS_RECV:  u16 = 2;
pub const SYS_PRINT: u16 = 3; // Debug only — will be removed in hardened build
pub const SYS_EXIT:  u16 = 4;
pub const SYS_PIPE:    u16 = 5;
pub const SYS_READ:    u16 = 6;
pub const SYS_WRITE:   u16 = 7;
pub const SYS_CLOSE:   u16 = 8;
pub const SYS_SOCKET:  u16 = 9;  // AF_UNIX SOCK_STREAM
pub const SYS_BIND:    u16 = 10;
pub const SYS_LISTEN:  u16 = 11;
pub const SYS_CONNECT: u16 = 12;
pub const SYS_ACCEPT:  u16 = 13;
pub const SYS_SHM_OPEN: u16 = 14;  // POSIX-ish shm_open
pub const SYS_SHM_SIZE: u16 = 15;
pub const SYS_SHM_PTR:  u16 = 16;  // kernel-address into the region

/// Negative return values are -errno encoded as a two's-complement u64
/// so the caller can branch on `(x0 as i64) < 0`.
fn err(code: i64) -> u64 { (-code) as u64 }
const EBADF:  i64 = 9;
const EPIPE:  i64 = 32;
const EINVAL: i64 = 22;
const ENFILE: i64 = 23;
const EFAULT: i64 = 14;
const EADDRINUSE: i64 = 98;
const ECONNREFUSED: i64 = 111;
const EAGAIN: i64 = 11;

pub fn handle(num: u16, frame: &mut TrapFrame) {
    match num {
        SYS_YIELD => {
            crate::kernel::scheduler::yield_now();
        }
        SYS_PRINT => {
            // Debug syscall: print a byte to UART
            let byte = frame.x[0] as u8;
            uart::putc(byte);
        }
        SYS_EXIT => {
            let current = crate::kernel::process::current();
            current.state = crate::kernel::process::TaskState::Dead;
            uart::puts("[syscall] Task exited: ");
            uart::puts(current.name_str());
            uart::puts("\n");
            crate::kernel::scheduler::yield_now();
        }
        SYS_PIPE => {
            // x0 ← (rfd in low 16) | (wfd in 16..32), or -errno on failure.
            match crate::kernel::pipe::create() {
                Ok((rfd, wfd)) => {
                    frame.x[0] = (rfd as u64) | ((wfd as u64) << 16);
                }
                Err(_) => {
                    frame.x[0] = err(ENFILE);
                }
            }
        }
        SYS_READ => {
            // x0 = fd, x1 = buf ptr, x2 = len.
            let fd  = frame.x[0] as u16;
            let ptr = frame.x[1] as usize;
            let len = frame.x[2] as usize;
            frame.x[0] = do_read(fd, ptr, len);
        }
        SYS_WRITE => {
            // x0 = fd, x1 = buf ptr, x2 = len.
            let fd  = frame.x[0] as u16;
            let ptr = frame.x[1] as usize;
            let len = frame.x[2] as usize;
            frame.x[0] = do_write(fd, ptr, len);
        }
        SYS_CLOSE => {
            let fd = frame.x[0] as u16;
            let task = crate::kernel::process::current();
            match task.fd_take(fd) {
                Some(entry) => {
                    match entry.kind {
                        crate::kernel::process::FdKind::Pipe { id, end } => {
                            crate::kernel::pipe::release_end(id, end);
                        }
                        crate::kernel::process::FdKind::Socket { id, .. } => {
                            crate::kernel::unix_sock::close(id);
                        }
                        crate::kernel::process::FdKind::Shm { id } => {
                            crate::kernel::shm::release(id);
                        }
                    }
                    frame.x[0] = 0;
                }
                None => {
                    frame.x[0] = err(EBADF);
                }
            }
        }
        SYS_SOCKET => {
            // No arguments today: AF_UNIX SOCK_STREAM is implicit.
            frame.x[0] = match crate::kernel::unix_sock::create() {
                Ok(fd)  => fd as u64,
                Err(_)  => err(ENFILE),
            };
        }
        SYS_BIND => {
            // x0 = fd, x1 = name ptr, x2 = name len.
            let fd  = frame.x[0] as u16;
            let ptr = frame.x[1] as usize;
            let len = frame.x[2] as usize;
            frame.x[0] = sock_op_with_name(fd, ptr, len, |sid, name| {
                crate::kernel::unix_sock::bind(sid, name)
            });
        }
        SYS_LISTEN => {
            let fd = frame.x[0] as u16;
            frame.x[0] = match resolve_sock_fd(fd) {
                Some(sid) => match crate::kernel::unix_sock::listen(sid) {
                    Ok(()) => 0,
                    Err(_) => err(EINVAL),
                },
                None => err(EBADF),
            };
        }
        SYS_CONNECT => {
            let fd  = frame.x[0] as u16;
            let ptr = frame.x[1] as usize;
            let len = frame.x[2] as usize;
            frame.x[0] = sock_op_with_name(fd, ptr, len, |sid, name| {
                crate::kernel::unix_sock::connect(sid, name)
            });
        }
        SYS_ACCEPT => {
            let fd = frame.x[0] as u16;
            frame.x[0] = match resolve_sock_fd(fd) {
                Some(sid) => match crate::kernel::unix_sock::accept(sid) {
                    Ok(new_fd) => new_fd as u64,
                    Err(_)     => err(EINVAL),
                },
                None => err(EBADF),
            };
        }
        SYS_SHM_OPEN => {
            // x0 = name_ptr, x1 = name_len, x2 = size_or_zero
            //   (size > 0 → create; size == 0 → open existing)
            let name_ptr = frame.x[0] as usize;
            let name_len = frame.x[1] as usize;
            let size     = frame.x[2] as usize;
            frame.x[0] = if name_ptr == 0 || name_len == 0
                || name_len > crate::kernel::shm::MAX_NAME_LEN {
                err(EINVAL)
            } else {
                let name = unsafe {
                    core::slice::from_raw_parts(name_ptr as *const u8, name_len)
                };
                let res = if size > 0 {
                    crate::kernel::shm::create(name, size)
                } else {
                    crate::kernel::shm::open(name)
                };
                match res {
                    Ok(fd) => fd as u64,
                    Err(e) => match e {
                        "name taken"   => err(EADDRINUSE),
                        "no such name" => err(ECONNREFUSED),
                        "fd table full" | "no free region" | "out of memory"
                                       => err(ENFILE),
                        _              => err(EINVAL),
                    }
                }
            };
        }
        SYS_SHM_SIZE => {
            let fd = frame.x[0] as u16;
            frame.x[0] = match resolve_shm_fd(fd) {
                Some(id) => crate::kernel::shm::region_size(id)
                    .map(|n| n as u64).unwrap_or(err(EBADF)),
                None => err(EBADF),
            };
        }
        SYS_SHM_PTR => {
            // Returns the kernel-address pointer to the region's
            // bytes. Safe in Phase 2 (all tasks share kernel
            // address space). Phase 3 will replace this with
            // proper page-table mapping.
            let fd = frame.x[0] as u16;
            frame.x[0] = match resolve_shm_fd(fd) {
                Some(id) => crate::kernel::shm::region_bytes_mut(id)
                    .map(|b| b.as_mut_ptr() as u64).unwrap_or(err(EBADF)),
                None => err(EBADF),
            };
        }
        _ => {
            uart::puts("[syscall] Unknown syscall: ");
            uart::putc(b'0' + (num / 10) as u8);
            uart::putc(b'0' + (num % 10) as u8);
            uart::puts("\n");
        }
    }
}

/// Resolve `fd` to a shm region id. Returns None if the fd is
/// unmapped or not a Shm kind.
fn resolve_shm_fd(fd: u16) -> Option<u16> {
    use crate::kernel::process::FdKind;
    let task = crate::kernel::process::current();
    let entry = task.fd_get(fd)?;
    match entry.kind {
        FdKind::Shm { id } => Some(id),
        _ => None,
    }
}

/// Resolve `fd` to a socket id. Returns None if the fd is unmapped
/// or not a Socket kind.
fn resolve_sock_fd(fd: u16) -> Option<u16> {
    use crate::kernel::process::FdKind;
    let task = crate::kernel::process::current();
    let entry = task.fd_get(fd)?;
    match entry.kind {
        FdKind::Socket { id, .. } => Some(id),
        _ => None,
    }
}

/// SYS_BIND / SYS_CONNECT helper: validate the buffer, copy the name
/// out, run the socket op, and encode the errno.
fn sock_op_with_name<F>(fd: u16, ptr: usize, len: usize, op: F) -> u64
where F: FnOnce(u16, &[u8]) -> Result<(), &'static str>
{
    if ptr == 0 { return err(EFAULT); }
    if len == 0 || len > crate::kernel::unix_sock::SOCK_NAME_MAX {
        return err(EINVAL);
    }
    let sid = match resolve_sock_fd(fd) {
        Some(s) => s,
        None    => return err(EBADF),
    };
    let name = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    match op(sid, name) {
        Ok(())  => 0,
        Err(e)  => match e {
            "name in use"            => err(EADDRINUSE),
            "no such name"           => err(ECONNREFUSED),
            "backlog full"           => err(EAGAIN),
            _                        => err(EINVAL),
        },
    }
}

/// Combined READ dispatch: pipe-read for FdKind::Pipe(Read end),
/// socket-read for FdKind::Socket(Connected).
fn do_read(fd: u16, ptr: usize, len: usize) -> u64 {
    use crate::kernel::process::{FdKind, PipeEnd, SocketRole};
    if ptr == 0 { return err(EFAULT); }
    if len == 0 { return 0; }
    let task = crate::kernel::process::current();
    let Some(entry) = task.fd_get(fd) else { return err(EBADF); };
    let out = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, len) };
    match entry.kind {
        FdKind::Pipe { id, end: PipeEnd::Read } => {
            match crate::kernel::pipe::read(id, out) {
                Ok(n)  => n as u64,
                Err(_) => err(EPIPE),
            }
        }
        FdKind::Pipe { .. } => err(EBADF),
        FdKind::Socket { id, role: SocketRole::Connected } => {
            match crate::kernel::unix_sock::read(id, out) {
                Ok(n)  => n as u64,
                Err(_) => err(EPIPE),
            }
        }
        FdKind::Socket { .. } => err(EBADF),
        FdKind::Shm { .. } => err(EBADF),  // use SYS_SHM_PTR + SYS_SHM_SIZE
    }
}

/// Combined WRITE dispatch: mirror of `do_read`.
fn do_write(fd: u16, ptr: usize, len: usize) -> u64 {
    use crate::kernel::process::{FdKind, PipeEnd, SocketRole};
    if ptr == 0 { return err(EFAULT); }
    if len == 0 { return 0; }
    let task = crate::kernel::process::current();
    let Some(entry) = task.fd_get(fd) else { return err(EBADF); };
    let buf = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    match entry.kind {
        FdKind::Pipe { id, end: PipeEnd::Write } => {
            match crate::kernel::pipe::write(id, buf) {
                Ok(n) => n as u64,
                Err("EPIPE") => err(EPIPE),
                Err(_) => err(EINVAL),
            }
        }
        FdKind::Pipe { .. } => err(EBADF),
        FdKind::Socket { id, role: SocketRole::Connected } => {
            match crate::kernel::unix_sock::write(id, buf) {
                Ok(n) => n as u64,
                Err("EPIPE") => err(EPIPE),
                Err(_) => err(EINVAL),
            }
        }
        FdKind::Socket { .. } => err(EBADF),
        FdKind::Shm { .. } => err(EBADF),  // use SYS_SHM_PTR + SYS_SHM_SIZE
    }
}
