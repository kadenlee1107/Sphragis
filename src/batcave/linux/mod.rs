// Linux-ABI compatibility shim for caves. Many syscalls/constants are
// staged ahead of the cave that exercises them — keeping a complete
// Linux table cuts review churn when a new cave needs a syscall we
// hadn't wired yet. dead_code is silenced module-wide since
// individual #[allow] tags would just clutter every file.
#![allow(dead_code)]

pub mod async_fds;
pub mod demand_page;
pub mod elf;
pub mod epoll;
pub mod fd;
pub mod futex;
pub mod loader;
pub mod mmu;
pub mod pipe_buf;
pub mod quotas;
pub mod runner;
pub mod signal;
pub mod skip_log;
pub mod sockets;
pub mod stdio_ring;
pub mod syscall;
pub mod syscall_history;
pub mod threads;
pub mod uaccess;
pub mod vfs;
