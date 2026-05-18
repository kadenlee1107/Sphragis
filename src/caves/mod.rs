pub mod kits;
pub mod bridge;
pub mod cap_mls_selftest;
pub mod cap_token;
pub mod cave;
pub mod cave_private;
pub mod docker_client;
pub mod ipc_session;
pub mod linux;
pub mod mls_ipc;
pub mod mls_label;
pub mod persist;
pub mod pq_comms_session;
pub mod secure_channel;
pub mod secure_ipc;
pub mod sys_caves;
pub mod sys_wg_ipc;
pub mod sys_wg_service;
pub mod syscall_filter;

/// Wave 2 stub — re-exports cave::count() at the crate::caves top level
/// so topbar.rs can call crate::caves::count().
pub fn count() -> u8 { cave::count() as u8 } // Wave 2 stub
