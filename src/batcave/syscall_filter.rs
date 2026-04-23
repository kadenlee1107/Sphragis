//! Per-cave Linux-syscall denylist.
//!
//! Second enforcement layer on top of `cave::active_has_cap`. The
//! cap system gates coarse categories (fs, net, proc, mem, raw,
//! display) — a cave with `net` can call every socket-family
//! syscall. That's fine for most operators, but if the cave gets
//! RCE'd the attacker now has e.g. `connect()` and can pivot.
//! Per-cave denylist lets operators surgically remove individual
//! syscall numbers (e.g. 203 = CONNECT) while keeping the broader
//! capability — the cave can `socket()` + `bind()` + `listen()` to
//! serve traffic but cannot dial out on behalf of an attacker.
//!
//! Fixed-size tables so this is alloc-free and works in the hot
//! syscall path without taking a lock.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, Ordering};

use super::cave::{MAX_CAVES, get_active};

const MAX_DENY_PER_CAVE: usize = 32;

#[derive(Clone, Copy)]
struct CaveFilter {
    len: u8,
    /// u16 is enough for any Linux syscall number we see (max ~450
    /// in the AArch64 table; we keep headroom).
    denied: [u16; MAX_DENY_PER_CAVE],
}

const EMPTY_FILTER: CaveFilter = CaveFilter {
    len: 0,
    denied: [0; MAX_DENY_PER_CAVE],
};

static mut FILTERS: [CaveFilter; MAX_CAVES] = [EMPTY_FILTER; MAX_CAVES];
static SYSCALL_DENIED_TOTAL: AtomicU32 = AtomicU32::new(0);

pub fn deny(cave_id: usize, nr: u64) {
    if cave_id >= MAX_CAVES || nr > u16::MAX as u64 { return; }
    let n = nr as u16;
    unsafe {
        let t = core::ptr::addr_of_mut!(FILTERS);
        let f = &mut (*t)[cave_id];
        for i in 0..(f.len as usize) {
            if f.denied[i] == n { return; }
        }
        if (f.len as usize) < MAX_DENY_PER_CAVE {
            f.denied[f.len as usize] = n;
            f.len += 1;
        }
    }
}

pub fn allow(cave_id: usize, nr: u64) {
    if cave_id >= MAX_CAVES || nr > u16::MAX as u64 { return; }
    let n = nr as u16;
    unsafe {
        let t = core::ptr::addr_of_mut!(FILTERS);
        let f = &mut (*t)[cave_id];
        for i in 0..(f.len as usize) {
            if f.denied[i] == n {
                // Shift left.
                for j in i..(f.len as usize) - 1 {
                    f.denied[j] = f.denied[j + 1];
                }
                f.len -= 1;
                return;
            }
        }
    }
}

pub fn clear(cave_id: usize) {
    if cave_id >= MAX_CAVES { return; }
    unsafe {
        let t = core::ptr::addr_of_mut!(FILTERS);
        (*t)[cave_id].len = 0;
    }
}

pub fn is_denied(cave_id: usize, nr: u64) -> bool {
    if cave_id >= MAX_CAVES || nr > u16::MAX as u64 { return false; }
    let n = nr as u16;
    unsafe {
        let t = core::ptr::addr_of!(FILTERS);
        let f = &(*t)[cave_id];
        for i in 0..(f.len as usize) {
            if f.denied[i] == n { return true; }
        }
    }
    false
}

/// Convenience for the syscall hot path.
pub fn is_denied_active(nr: u64) -> bool {
    let id = get_active();
    if id == usize::MAX { return false; }
    is_denied(id, nr)
}

/// Increment the "syscall was denied by the per-cave filter" counter.
/// Called from `syscall::handle` when is_denied_active returns true.
pub fn bump_denied() {
    SYSCALL_DENIED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn denied_count() -> u32 {
    SYSCALL_DENIED_TOTAL.load(Ordering::Relaxed)
}

pub fn denied_count_reset() {
    SYSCALL_DENIED_TOTAL.store(0, Ordering::Relaxed);
}

/// Copy this cave's denylist into `out` and return the number of
/// entries written. Used by shell `cave-syscall-list`.
pub fn list(cave_id: usize, out: &mut [u16]) -> usize {
    if cave_id >= MAX_CAVES { return 0; }
    unsafe {
        let t = core::ptr::addr_of!(FILTERS);
        let f = &(*t)[cave_id];
        let n = (f.len as usize).min(out.len());
        for i in 0..n { out[i] = f.denied[i]; }
        n
    }
}

pub fn len_for(cave_id: usize) -> usize {
    if cave_id >= MAX_CAVES { return 0; }
    unsafe {
        let t = core::ptr::addr_of!(FILTERS);
        (*t)[cave_id].len as usize
    }
}

// ── Self-test ────────────────────────────────────────────────────

pub struct SyscallFilterReport {
    pub installed: usize,
    pub is_denied_203: bool,
    pub is_denied_204: bool,
    pub after_clear: usize,
}

/// Install a few syscall denials on cave 0, check lookups, clear.
pub fn selftest() -> Result<SyscallFilterReport, &'static str> {
    clear(0);
    denied_count_reset();

    deny(0, 203);   // CONNECT
    deny(0, 211);   // SENDMSG
    deny(0, 221);   // EXECVE
    // Duplicate should not double-count.
    deny(0, 203);

    let installed = len_for(0);
    if installed != 3 { return Err("expected 3 denies installed (dup suppressed)"); }

    let c = is_denied(0, 203);
    let g = is_denied(0, 204);   // getsockname — should NOT be denied
    if !c { return Err("CONNECT should be denied"); }
    if g  { return Err("GETSOCKNAME should not be denied"); }

    allow(0, 211);
    if len_for(0) != 2 { return Err("allow() should shrink to 2"); }

    clear(0);
    let after = len_for(0);
    if after != 0 { return Err("clear should empty the list"); }

    Ok(SyscallFilterReport {
        installed,
        is_denied_203: c,
        is_denied_204: g,
        after_clear: after,
    })
}
