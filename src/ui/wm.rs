//! Wave 2 floating-window manager.
//!
//! Holds a fixed-size store of windows ([Option<Window>; 16]),
//! tracks z-order implicitly (window order in the array — back-most
//! first, focused window last), tracks focus by Option<WindowId>,
//! exposes open/close/focus/cycle/iter API. Paint and event handling
//! live alongside the data model in subsequent tasks.

#![allow(dead_code)]

use crate::ui::apps_registry::AppId;

pub const MAX_WINDOWS: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct WindowId(pub u32);

#[derive(Copy, Clone, Debug)]
pub struct WindowRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Window {
    pub id: WindowId,
    pub app: AppId,
    pub rect: WindowRect,
    pub cave_name: Option<[u8; 16]>,
}

// Static-mut access pattern:
//
// FOCUSED and NEXT_ID are scalar — the optimizer is free to cache
// their values across function calls, so reads/writes go through
// `read_volatile` / `write_volatile` to force a real memory access
// every time. WINDOWS is a `[Option<Window>; 16]`; slot mutations
// compile to whole-slot stores (Option<Window> is Copy, ~28 bytes),
// which the optimizer can't reorder past the surrounding raw-pointer
// access — so the plain `*slot = Some(...)` style is safe without
// volatile. Whole-array writes in `reset_all()` still use volatile
// to be defensive against the same caching concern.
static mut WINDOWS: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];
static mut NEXT_ID: u32 = 1;
static mut FOCUSED: Option<WindowId> = None;

pub fn count() -> usize {
    unsafe {
        let wins = &*core::ptr::addr_of!(WINDOWS);
        wins.iter().filter(|w| w.is_some()).count()
    }
}

pub fn focused() -> Option<WindowId> {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FOCUSED)) }
}

pub fn iter() -> impl Iterator<Item = Window> {
    unsafe {
        let wins = &*core::ptr::addr_of!(WINDOWS);
        wins.iter().filter_map(|w| *w).collect::<alloc::vec::Vec<_>>().into_iter()
    }
}

pub fn get(id: WindowId) -> Option<Window> {
    unsafe {
        let wins = &*core::ptr::addr_of!(WINDOWS);
        wins.iter().find_map(|w| match w {
            Some(x) if x.id == id => Some(*x),
            _ => None,
        })
    }
}

pub fn open(app: AppId, cave_name: Option<&str>) -> Option<WindowId> {
    let id = unsafe {
        let i = core::ptr::read_volatile(core::ptr::addr_of!(NEXT_ID));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(NEXT_ID), i.wrapping_add(1));
        WindowId(i)
    };
    // Truncate cave_name to fit in 16 bytes, but snap back to the
    // nearest char boundary so we never split a multi-byte UTF-8
    // sequence — downstream consumers (Task 3 paint path) read these
    // bytes back through `from_utf8_unchecked`, which is UB on
    // invalid UTF-8.
    let cave = cave_name.map(|s| {
        let mut buf = [0u8; 16];
        let mut n = s.len().min(16);
        while n > 0 && !s.is_char_boundary(n) {
            n -= 1;
        }
        buf[..n].copy_from_slice(&s.as_bytes()[..n]);
        buf
    });
    let i = count() as u32;
    let rect = WindowRect {
        x: 80 + i * 24,
        y: 60 + i * 24,
        w: 720,
        h: 480,
    };
    let window = Window { id, app, rect, cave_name: cave };

    // Place the new window in the first free slot, then re-route
    // through focus() so the compact-to-end path runs. Without this,
    // a slot freed by an earlier close() in the middle of the array
    // would leave the new window mid-array — violating the
    // "focused window last" z-order invariant the painter relies on.
    let mut placed = false;
    unsafe {
        let wins = &mut *core::ptr::addr_of_mut!(WINDOWS);
        for slot in wins.iter_mut() {
            if slot.is_none() {
                *slot = Some(window);
                placed = true;
                break;
            }
        }
    }
    if placed {
        focus(id);
        Some(id)
    } else {
        None
    }
}

pub fn close(id: WindowId) {
    unsafe {
        let wins = &mut *core::ptr::addr_of_mut!(WINDOWS);
        for slot in wins.iter_mut() {
            if slot.map(|w| w.id) == Some(id) {
                *slot = None;
                let focused = core::ptr::read_volatile(core::ptr::addr_of!(FOCUSED));
                if focused == Some(id) {
                    let new_focus = wins.iter().rev().find_map(|w| w.map(|x| x.id));
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(FOCUSED), new_focus);
                }
                return;
            }
        }
    }
}

pub fn focus(id: WindowId) {
    unsafe {
        let wins = &mut *core::ptr::addr_of_mut!(WINDOWS);
        let mut taken: Option<Window> = None;
        for slot in wins.iter_mut() {
            if slot.map(|w| w.id) == Some(id) {
                taken = slot.take();
                break;
            }
        }
        if let Some(w) = taken {
            let mut compacted: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];
            let mut j = 0;
            for x in wins.iter().flatten() {
                compacted[j] = Some(*x);
                j += 1;
            }
            compacted[j] = Some(w);
            *wins = compacted;
            core::ptr::write_volatile(core::ptr::addr_of_mut!(FOCUSED), Some(id));
        }
    }
}

pub fn cycle_focus() {
    let ids: alloc::vec::Vec<WindowId> = iter().map(|w| w.id).collect();
    if ids.len() < 2 { return; }
    let cur = focused();
    let next_idx = match cur {
        Some(id) => {
            let idx = ids.iter().position(|x| *x == id).unwrap_or(0);
            (idx + 1) % ids.len()
        }
        None => 0,
    };
    focus(ids[next_idx]);
}

pub fn set_rect(id: WindowId, rect: WindowRect) {
    unsafe {
        let wins = &mut *core::ptr::addr_of_mut!(WINDOWS);
        for slot in wins.iter_mut() {
            if slot.map(|w| w.id) == Some(id) {
                if let Some(w) = slot.as_mut() { w.rect = rect; }
                return;
            }
        }
    }
}

/// Called when the active cave changes. Wave 2: no-op stub; Task 7
/// or a later wave will decide whether cave switching should close
/// cave-scoped windows, refresh chrome titles, or leave existing
/// windows in place with stale cave_name.
///
/// Kept as a public entry point so `caves/cave.rs` can re-enable the
/// call site (currently `// XXX Wave-2-temp:`) without another round
/// of API surgery once the policy is decided.
pub fn reset_for_cave_switch() {
    // Intentionally empty — see doc.
}

/// Reset all WM state. Only called by security::wipe — NOT by the
/// lock/unlock cycle (which preserves the workspace).
pub fn reset_all() {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(WINDOWS), [None; MAX_WINDOWS]);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FOCUSED), None);
    }
}
