// Linux-ABI compatibility shim for caves. Constants and helpers
// here back the syscall surface caves see; only items with a concrete
// caller stay. Items the squeaky-clean Phase 4 pass found unused
// were deleted outright — not silenced — so the shim's surface
// matches what we actually implement, not what we might one day.

pub mod async_fds;
pub mod demand_page;
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
