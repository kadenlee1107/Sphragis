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
    let cave = cave_name.map(|s| {
        let mut buf = [0u8; 16];
        let bytes = s.as_bytes();
        let n = bytes.len().min(16);
        buf[..n].copy_from_slice(&bytes[..n]);
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

    unsafe {
        let wins = &mut *core::ptr::addr_of_mut!(WINDOWS);
        for slot in wins.iter_mut() {
            if slot.is_none() {
                *slot = Some(window);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(FOCUSED), Some(id));
                return Some(id);
            }
        }
    }
    None
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

/// Reset all WM state. Only called by security::wipe — NOT by the
/// lock/unlock cycle (which preserves the workspace).
pub fn reset_all() {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(WINDOWS), [None; MAX_WINDOWS]);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FOCUSED), None);
    }
}
