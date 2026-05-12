#![allow(dead_code)]
// Bat_OS — System Call Handler
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
pub const SYS_PIPE:  u16 = 5;
pub const SYS_READ:  u16 = 6;
pub const SYS_WRITE: u16 = 7;
pub const SYS_CLOSE: u16 = 8;

/// Negative return values are -errno encoded as a two's-complement u64
/// so the caller can branch on `(x0 as i64) < 0`.
fn err(code: i64) -> u64 { (-code) as u64 }
const EBADF:  i64 = 9;
const EPIPE:  i64 = 32;
const EINVAL: i64 = 22;
const ENFILE: i64 = 23;
const EFAULT: i64 = 14;

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
            frame.x[0] = match resolve_pipe_fd(fd, true) {
                Some(id) => {
                    if ptr == 0 {
                        err(EFAULT)
                    } else if len == 0 {
                        0
                    } else {
                        let out = unsafe {
                            core::slice::from_raw_parts_mut(ptr as *mut u8, len)
                        };
                        match crate::kernel::pipe::read(id, out) {
                            Ok(n)  => n as u64,
                            Err(_) => err(EPIPE),
                        }
                    }
                }
                None => err(EBADF),
            };
        }
        SYS_WRITE => {
            // x0 = fd, x1 = buf ptr, x2 = len.
            let fd  = frame.x[0] as u16;
            let ptr = frame.x[1] as usize;
            let len = frame.x[2] as usize;
            frame.x[0] = match resolve_pipe_fd(fd, false) {
                Some(id) => {
                    if ptr == 0 {
                        err(EFAULT)
                    } else if len == 0 {
                        0
                    } else {
                        let buf = unsafe {
                            core::slice::from_raw_parts(ptr as *const u8, len)
                        };
                        match crate::kernel::pipe::write(id, buf) {
                            Ok(n) => n as u64,
                            Err(e) if e == "EPIPE" => err(EPIPE),
                            Err(_) => err(EINVAL),
                        }
                    }
                }
                None => err(EBADF),
            };
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
                    }
                    frame.x[0] = 0;
                }
                None => {
                    frame.x[0] = err(EBADF);
                }
            }
        }
        _ => {
            uart::puts("[syscall] Unknown syscall: ");
            uart::putc(b'0' + (num / 10) as u8);
            uart::putc(b'0' + (num % 10) as u8);
            uart::puts("\n");
        }
    }
}

/// Resolve `fd` to a pipe id, requiring the end matches the operation.
/// Returns None if the fd is unmapped or points at the wrong end.
fn resolve_pipe_fd(fd: u16, for_read: bool) -> Option<u16> {
    use crate::kernel::process::{FdKind, PipeEnd};
    let task = crate::kernel::process::current();
    let entry = task.fd_get(fd)?;
    match entry.kind {
        FdKind::Pipe { id, end } => {
            let ok = match end {
                PipeEnd::Read  => for_read,
                PipeEnd::Write => !for_read,
            };
            if ok { Some(id) } else { None }
        }
    }
}
