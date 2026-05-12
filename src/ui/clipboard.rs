//! Bat_OS — system clipboard.
//!
//! Single-slot byte buffer with set/get/clear. The shell uses it for
//! Ctrl+V (paste) and Ctrl+Y (yank current line). Commands that need
//! to hand the operator a long string they'll want to round-trip
//! (e.g. `comms identify` discovering a server pubkey) populate the
//! clipboard programmatically so a follow-up `comms pin <Ctrl+V>` is
//! all the user has to type.
//!
//! Wiped on cave switch — a logged-out cave must not leave its
//! clipboard for the next tenant. The reset_for_cave_switch path is
//! called from the same place that scrubs scrollback and message
//! history.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

pub const CLIPBOARD_CAP: usize = 1024;

static mut BUF: [u8; CLIPBOARD_CAP] = [0u8; CLIPBOARD_CAP];
static LEN: AtomicUsize = AtomicUsize::new(0);

/// Replace clipboard contents with `data`. Truncates anything beyond
/// `CLIPBOARD_CAP` — callers are responsible for keeping their
/// payloads in range.
pub fn set(data: &[u8]) {
    let n = data.len().min(CLIPBOARD_CAP);
    unsafe {
        let dst = core::ptr::addr_of_mut!(BUF) as *mut u8;
        for i in 0..n {
            core::ptr::write_volatile(dst.add(i), data[i]);
        }
        // Zero the tail so a shorter set doesn't leave previous bytes
        // visible to a future longer get() that mis-sizes.
        for i in n..CLIPBOARD_CAP {
            core::ptr::write_volatile(dst.add(i), 0);
        }
    }
    LEN.store(n, Ordering::Release);
}

/// Number of bytes currently held.
pub fn len() -> usize {
    LEN.load(Ordering::Acquire)
}

/// True when the clipboard is empty (nothing to paste).
pub fn is_empty() -> bool {
    len() == 0
}

/// Copy clipboard contents into `out`. Returns the number of bytes
/// written, capped by both clipboard size and `out.len()`.
pub fn copy_into(out: &mut [u8]) -> usize {
    let n = len().min(out.len());
    unsafe {
        let src = core::ptr::addr_of!(BUF) as *const u8;
        for i in 0..n {
            out[i] = core::ptr::read_volatile(src.add(i));
        }
    }
    n
}

/// Read one byte at index `i`. Used by the paste hotkey to stream
/// bytes one at a time into the input handler without allocating an
/// intermediate buffer.
pub fn byte_at(i: usize) -> Option<u8> {
    if i >= len() { return None; }
    unsafe {
        let src = core::ptr::addr_of!(BUF) as *const u8;
        Some(core::ptr::read_volatile(src.add(i)))
    }
}

/// Empty the clipboard and wipe the underlying buffer. Used by
/// `clip clear` and on cave switch.
pub fn clear() {
    unsafe {
        let dst = core::ptr::addr_of_mut!(BUF) as *mut u8;
        for i in 0..CLIPBOARD_CAP {
            core::ptr::write_volatile(dst.add(i), 0);
        }
    }
    LEN.store(0, Ordering::Release);
}

/// Wipe everything on cave switch so the new tenant can't read the
/// previous tenant's clipboard. Single source of truth — anything
/// the clipboard owns should also be wiped from anywhere it leaked
/// (none today, but if/when paste hooks copy elsewhere this is
/// where the rule lives).
pub fn reset_for_cave_switch() {
    clear();
}
