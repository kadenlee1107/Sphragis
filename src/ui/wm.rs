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

// ── Palette (matches Wave 1) ──────────────────────────────────────

const BG:        u32 = 0xFF0D0D10;
const PANEL:     u32 = 0xFF18181C;
const HAIRLINE:  u32 = 0xFF2A2A30;
const INK:       u32 = 0xFFE5E7EB;
const MID:       u32 = 0xFF6B7280;
const SHADOW:    u32 = 0xFF040408;

const CHROME_H:        u32 = 22;
const SHADOW_OFFSET_X: i32 = 4;
const SHADOW_OFFSET_Y: i32 = 6;

/// Paint all open windows in z-order (back-most first → focused last).
pub fn paint_all() {
    use crate::ui::apps_registry::descriptor;
    use crate::ui::draw;
    use crate::ui::font;
    use crate::ui::gpu;

    let screen_w = gpu::width();
    let focused = focused();
    let snapshot: alloc::vec::Vec<Window> = iter().collect();

    for window in snapshot.iter() {
        let r = window.rect;
        let is_focused = Some(window.id) == focused;

        // Drop shadow.
        let sx = r.x.saturating_add(SHADOW_OFFSET_X as u32);
        let sy = r.y.saturating_add(SHADOW_OFFSET_Y as u32);
        gpu::fill_rect(sx, sy, r.w, r.h, SHADOW);

        // Body fill.
        gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

        // Chrome strip.
        gpu::fill_rect(r.x, r.y, r.w, CHROME_H, PANEL);

        // 1-px border.
        draw::draw_border(r.x, r.y, r.w, r.h, HAIRLINE);

        // Chrome/body separator.
        gpu::fill_rect(r.x, r.y + CHROME_H, r.w, 1, HAIRLINE);

        // 8x8 open circle (close glyph).
        let cx0 = r.x + 10;
        let cy0 = r.y + (CHROME_H - 8) / 2;
        for dy in 0..8u32 {
            for dx in 0..8u32 {
                let fx = dx as i32 - 4;
                let fy = dy as i32 - 4;
                let d2 = fx * fx + fy * fy;
                if d2 >= 6 && d2 <= 13 {
                    gpu::fill_rect(cx0 + dx, cy0 + dy, 1, 1, MID);
                }
            }
        }

        // Title text.
        let desc = descriptor(window.app);
        let title_color = if is_focused { INK } else { MID };
        let title_x = r.x + 28;
        let title_y = r.y + (CHROME_H - 16) / 2;
        font::draw_str(
            gpu::framebuffer(),
            screen_w,
            title_x, title_y,
            desc.title,
            title_color, PANEL,
        );

        // Optional cave-name suffix.
        if let Some(cave) = window.cave_name {
            let n = cave.iter().position(|&b| b == 0).unwrap_or(16);
            let cave_str = unsafe { core::str::from_utf8_unchecked(&cave[..n]) };
            let sep_x = title_x + desc.title.len() as u32 * 8 + 8;
            font::draw_str(gpu::framebuffer(), screen_w, sep_x, title_y, "*", MID, PANEL);
            font::draw_str(gpu::framebuffer(), screen_w, sep_x + 16, title_y, cave_str, MID, PANEL);
        }

        // Body rect → app's paint callback.
        let body = WindowRect {
            x: r.x + 1,
            y: r.y + CHROME_H + 1,
            w: r.w.saturating_sub(2),
            h: r.h.saturating_sub(CHROME_H + 2),
        };
        (desc.paint)(body);
    }
}

// ── Drag/resize state + hit testing ──────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Corner { TL, TR, BL, BR }

#[derive(Copy, Clone)]
enum DragKind {
    Move,
    Resize(Corner),
}

#[derive(Copy, Clone)]
struct DragState {
    window: WindowId,
    kind: DragKind,
    start_mouse_x: i32,
    start_mouse_y: i32,
    start_rect: WindowRect,
}

// DRAG follows the same volatile + addr_of/addr_of_mut access
// convention as FOCUSED and NEXT_ID above. Every read goes through
// read_volatile and every write goes through write_volatile so the
// optimizer cannot cache the value across call boundaries.
static mut DRAG: Option<DragState> = None;

const RESIZE_HIT: u32 = 12;
const MIN_W: u32 = 280;
const MIN_H: u32 = 160;

pub enum Hit {
    CloseGlyph(WindowId),
    Corner(WindowId, Corner),
    Chrome(WindowId),
    Body(WindowId),
    None,
}

pub fn hit_test(mx: i32, my: i32) -> Hit {
    let snapshot: alloc::vec::Vec<Window> = iter().collect();
    for window in snapshot.iter().rev() {
        let r = window.rect;
        let rx0 = r.x as i32;
        let ry0 = r.y as i32;
        let rx1 = rx0 + r.w as i32;
        let ry1 = ry0 + r.h as i32;
        if mx < rx0 || mx >= rx1 || my < ry0 || my >= ry1 { continue; }

        let cgx0 = rx0 + 10;
        let cgy0 = ry0 + ((CHROME_H - 8) / 2) as i32;
        if mx >= cgx0 && mx < cgx0 + 8 && my >= cgy0 && my < cgy0 + 8 {
            return Hit::CloseGlyph(window.id);
        }

        let rh = RESIZE_HIT as i32;
        if mx < rx0 + rh && my < ry0 + rh { return Hit::Corner(window.id, Corner::TL); }
        if mx >= rx1 - rh && my < ry0 + rh { return Hit::Corner(window.id, Corner::TR); }
        if mx < rx0 + rh && my >= ry1 - rh { return Hit::Corner(window.id, Corner::BL); }
        if mx >= rx1 - rh && my >= ry1 - rh { return Hit::Corner(window.id, Corner::BR); }

        if my < ry0 + CHROME_H as i32 { return Hit::Chrome(window.id); }

        return Hit::Body(window.id);
    }
    Hit::None
}

/// Begin a drag (or close, on CloseGlyph hit). Returns true if a
/// repaint is needed.
pub fn begin_drag(mx: i32, my: i32) -> bool {
    match hit_test(mx, my) {
        Hit::CloseGlyph(id) => { close(id); true }
        Hit::Corner(id, corner) => {
            focus(id);
            if let Some(w) = get(id) {
                unsafe {
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!(DRAG),
                        Some(DragState {
                            window: id, kind: DragKind::Resize(corner),
                            start_mouse_x: mx, start_mouse_y: my,
                            start_rect: w.rect,
                        }),
                    );
                }
            }
            true
        }
        Hit::Chrome(id) => {
            focus(id);
            if let Some(w) = get(id) {
                unsafe {
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!(DRAG),
                        Some(DragState {
                            window: id, kind: DragKind::Move,
                            start_mouse_x: mx, start_mouse_y: my,
                            start_rect: w.rect,
                        }),
                    );
                }
            }
            true
        }
        Hit::Body(id) => { focus(id); true }
        Hit::None => false,
    }
}

pub fn update_drag(mx: i32, my: i32) -> bool {
    let drag = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DRAG)) };
    let Some(d) = drag else { return false; };
    let dx = mx - d.start_mouse_x;
    let dy = my - d.start_mouse_y;
    let r0 = d.start_rect;
    let new_rect = match d.kind {
        DragKind::Move => WindowRect {
            x: (r0.x as i32 + dx).max(0) as u32,
            y: (r0.y as i32 + dy).max(0) as u32,
            w: r0.w, h: r0.h,
        },
        DragKind::Resize(corner) => {
            let mut x = r0.x as i32;
            let mut y = r0.y as i32;
            let mut w = r0.w as i32;
            let mut h = r0.h as i32;
            match corner {
                Corner::TL => { x += dx; y += dy; w -= dx; h -= dy; }
                Corner::TR => {          y += dy; w += dx; h -= dy; }
                Corner::BL => { x += dx;          w -= dx; h += dy; }
                Corner::BR => {                   w += dx; h += dy; }
            }

            // Bug-fix A: use signed compares so a negative w or h
            // (from aggressive dragging) still triggers the floor.
            // The plan's `(w as u32) < MIN_W` wraps a negative i32
            // to a huge u32, silently bypassing the floor.
            let right  = r0.x as i32 + r0.w as i32;
            let bottom = r0.y as i32 + r0.h as i32;

            if w < MIN_W as i32 { w = MIN_W as i32; }
            if h < MIN_H as i32 { h = MIN_H as i32; }

            // Bug-fix B: re-anchor the opposite (fixed) edge after
            // clamping, so it stays put when the moving edge hits the
            // floor. Without this the far edge drifts with the cursor.
            match corner {
                Corner::TL => { x = right - w;  y = bottom - h; }
                Corner::TR => {                  y = bottom - h; }
                Corner::BL => { x = right - w;                  }
                Corner::BR => {}
            }

            WindowRect {
                x: x.max(0) as u32, y: y.max(0) as u32,
                w: w as u32,        h: h as u32,
            }
        }
    };
    set_rect(d.window, new_rect);
    true
}

pub fn end_drag() {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DRAG), None);
    }
}

pub fn is_dragging() -> bool {
    let drag = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DRAG)) };
    drag.is_some()
}
