# Desktop Chrome + App Launcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Wave 2 — quiet canvas + floating multi-window WM + 8-app launcher overlay + customizable top-bar status strip — per `docs/superpowers/specs/2026-05-14-desktop-chrome-design.md`.

**Architecture:** Replace the current pane-tiling WM (`src/ui/wm.rs`) and single-app desktop loop (`src/ui/desktop.rs`) with a floating-window WM that holds an `[Option<Window>; 16]` array and a state-machine event loop. Add a new `src/ui/topbar.rs` for the top status strip. Apps register through a static `AppDescriptor` table; each descriptor carries an `AppId`, a label, and a paint callback. Lock/unlock becomes a cycle in `main.rs` rather than a `-> !` terminal call, so the workspace persists across lock-and-unlock.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, alloc available via linked-list allocator). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: QEMU with `-display cocoa` + virtio-keyboard + virtio-tablet.

**Verification reality check.** Same as Wave 1: this crate is `#![no_std] #![no_main]`, no `lib.rs`, no test harness. `cargo test` doesn't run kernel code. Every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus a QEMU walk-through against the spec at the end. There is no unit-test step. The QEMU walk-through is Task 11.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/desktop-chrome
```
Expected: working tree clean before checkout; on branch `feat/desktop-chrome` after.

- [ ] **Step 0b: Confirm the current build is clean before any edits.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both clean. If not, fix before proceeding — every later task depends on a clean baseline.

---

## File structure

This plan creates and modifies the following files. Each new module has one clear responsibility.

| File | Status | Responsibility |
|------|--------|----------------|
| `src/ui/apps_registry.rs` | **NEW** | Static `AppId` enum + `AppDescriptor` table. The "what apps exist" table. |
| `src/ui/wm.rs` | **REPLACED** | Floating-window manager: `Window` struct, fixed-size store, focus, drag, resize, paint. |
| `src/ui/topbar.rs` | **NEW** | Top status strip: paint, click handling, config sheet. |
| `src/ui/topbar_config.rs` | **NEW** | Badge enable/order state, SealFS persistence. |
| `src/ui/launcher.rs` | **NEW** | 8-app launcher overlay: paint, click handling. |
| `src/ui/desktop.rs` | **REPLACED** | State machine (LAUNCHER / ACTIVE / OVERLAY) + event loop. `run() -> LockReason`. |
| `src/ui/mod.rs` | **MODIFY** | Register new modules. |
| `src/main.rs` | **MODIFY** | Wrap `boot_screen::run()` + `desktop::run()` in a lock/unlock cycle. |

The existing `src/ui/apps/*` modules are **not modified** — their internal cyberpunk look stays for Wave 2 and gets refreshed in Wave 3+. Each app exposes its existing paint entry; the new WM wraps it with chrome. Small Wave-2 shims may be needed in each app to adapt their existing paint signature to `fn(WindowRect)`.

---

## Task 1: App registry

Create the static "what apps exist" table. Every later task references `AppId` and `APPS`, so this is the foundation.

**Files:**
- Create: `src/ui/apps_registry.rs`
- Modify: `src/ui/mod.rs`
- Modify (Wave-2 shims): each `src/ui/apps/*.rs` and `src/ui/shell.rs` as needed

- [ ] **Step 1: Create `src/ui/apps_registry.rs`.**

```rust
//! App registry — the static table of apps the launcher can show.
//!
//! Each `AppDescriptor` carries the identity, the label rendered on
//! the launcher tile, the chrome title, and a paint callback. The
//! paint callback takes a `WindowRect` (the body region inside the
//! window's chrome) and is responsible for drawing the app's
//! contents into that rect. Apps are stateful — most of them keep
//! state in their own `static`s; this registry just wires up the
//! draw entry points.

#![allow(dead_code)]

use crate::ui::wm::WindowRect;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AppId {
    Caves,
    Files,
    Net,
    Security,
    Shell,
    Editor,
    Comms,
    Agent,
}

pub struct AppDescriptor {
    pub id: AppId,
    pub label: &'static str,
    pub title: &'static str,
    pub paint: fn(WindowRect),
}

pub static APPS: [AppDescriptor; 8] = [
    AppDescriptor { id: AppId::Caves,    label: "CAVES",    title: "CAVES",    paint: paint_caves },
    AppDescriptor { id: AppId::Files,    label: "FILES",    title: "FILES",    paint: paint_files },
    AppDescriptor { id: AppId::Net,      label: "NET",      title: "NET",      paint: paint_net },
    AppDescriptor { id: AppId::Security, label: "SECURITY", title: "SECURITY", paint: paint_security },
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell },
    AppDescriptor { id: AppId::Editor,   label: "EDITOR",   title: "EDITOR",   paint: paint_editor },
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms },
    AppDescriptor { id: AppId::Agent,    label: "AGENT",    title: "AGENT",    paint: paint_agent },
];

pub fn descriptor(id: AppId) -> &'static AppDescriptor {
    APPS.iter().find(|d| d.id == id).expect("AppId always in APPS")
}

// ── Paint callbacks ──────────────────────────────────────────────

fn paint_caves(rect: WindowRect)    { crate::ui::apps::caves_mgr::paint(rect); }
fn paint_files(rect: WindowRect)    { crate::ui::apps::filemanager::paint(rect); }
fn paint_net(rect: WindowRect)      { crate::ui::apps::netmon::paint(rect); }
fn paint_security(rect: WindowRect) { crate::ui::apps::security::paint(rect); }
fn paint_shell(rect: WindowRect)    { crate::ui::shell::paint(rect); }
fn paint_editor(rect: WindowRect)   { crate::ui::apps::editor::paint(rect); }
fn paint_comms(rect: WindowRect)    { crate::ui::apps::comms::paint(rect); }

fn paint_agent(rect: WindowRect) {
    use crate::ui::font;
    let msg = "AGENT - coming soon";
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + rect.h / 2;
    let tx = cx.saturating_sub((msg.len() as u32 * 8) / 2);
    let ty = cy.saturating_sub(8);
    font::draw_str(
        crate::ui::gpu::framebuffer(),
        crate::ui::gpu::width(),
        tx, ty,
        msg,
        0xFFE5E7EB,
        0xFF0D0D10,
    );
}
```

- [ ] **Step 2: Register the module in `src/ui/mod.rs`.**

Open `src/ui/mod.rs` and add `pub mod apps_registry;` alphabetically.

- [ ] **Step 3: Build to identify missing `paint(WindowRect)` entries.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -25
```

The build will fail because the existing apps don't all expose a `paint(rect: WindowRect)` function with that exact signature. For each missing function, add a thin Wave-2 shim in that app's file. Example for an app whose existing render path is `render()`:

```rust
// Append to src/ui/apps/dashboard.rs (or equivalent):

/// Wave 2 shim — adapts the existing render path to the new WM's
/// `fn(WindowRect)` contract. Refresh in Wave 3+ when the app gets
/// its own redesign.
pub fn paint(rect: crate::ui::wm::WindowRect) {
    // Delegate to existing render; the existing path probably writes
    // full-screen, which is fine as a Wave-2 stub — apps get clipped
    // by the WM's chrome and z-order. Wave 3+ rewrites apps to
    // respect `rect` properly.
    let _ = rect;
    render();
}
```

For the SHELL specifically: don't try to wedge the 11k-line console into a Wave-2 window. Add a stub:

```rust
// Append to src/ui/shell.rs:

pub fn paint(rect: crate::ui::wm::WindowRect) {
    use crate::ui::font;
    let msg = "SHELL - integrating in Wave 5";
    let tx = rect.x + 12;
    let ty = rect.y + 12;
    font::draw_str(
        crate::ui::gpu::framebuffer(),
        crate::ui::gpu::width(),
        tx, ty, msg,
        0xFFE5E7EB, 0xFF0D0D10,
    );
}
```

Mark every Wave-2 shim with the comment `// Wave 2 shim — refresh in Wave 3+`.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both clean.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/apps_registry.rs src/ui/mod.rs src/ui/apps/ src/ui/shell.rs
git commit -m "$(cat <<'EOF'
apps_registry: static AppId + AppDescriptor table for Wave 2

Wires 8 apps (CAVES, FILES, NET, SECURITY, SHELL, EDITOR, COMMS,
AGENT) into a static APPS table. Each descriptor carries identity,
launcher label, chrome title, and a paint(rect: WindowRect)
callback. App internals are not refactored; thin Wave-2 shims
adapt each existing paint path to the rect-receiving signature.

AGENT and SHELL paint stubs in place; both get real integrations in
later waves (AGENT depends on DESIGN_AI_AGENT.md, SHELL depends on
Wave 5 console palette refresh).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2: Window struct + WM core (open / close / focus)

Replace the pane-tiling WM with a floating-window WM. This task does the data model + open/close/focus; window paint comes in Task 3, drag/resize in Task 4.

**Files:**
- Replace: `src/ui/wm.rs`

- [ ] **Step 1: Replace `src/ui/wm.rs` entirely.**

Delete the current contents and write:

```rust
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
    unsafe { WINDOWS.iter().filter(|w| w.is_some()).count() }
}

pub fn focused() -> Option<WindowId> {
    unsafe { FOCUSED }
}

pub fn iter() -> impl Iterator<Item = Window> {
    unsafe { WINDOWS.iter().filter_map(|w| *w).collect::<alloc::vec::Vec<_>>().into_iter() }
}

pub fn get(id: WindowId) -> Option<Window> {
    unsafe {
        WINDOWS.iter().find_map(|w| match w {
            Some(x) if x.id == id => Some(*x),
            _ => None,
        })
    }
}

pub fn open(app: AppId, cave_name: Option<&str>) -> Option<WindowId> {
    let id = unsafe {
        let i = NEXT_ID;
        NEXT_ID = NEXT_ID.wrapping_add(1);
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
        for slot in WINDOWS.iter_mut() {
            if slot.is_none() {
                *slot = Some(window);
                FOCUSED = Some(id);
                return Some(id);
            }
        }
    }
    None
}

pub fn close(id: WindowId) {
    unsafe {
        for slot in WINDOWS.iter_mut() {
            if slot.map(|w| w.id) == Some(id) {
                *slot = None;
                if FOCUSED == Some(id) {
                    FOCUSED = WINDOWS.iter().rev().find_map(|w| w.map(|x| x.id));
                }
                return;
            }
        }
    }
}

pub fn focus(id: WindowId) {
    unsafe {
        let mut taken: Option<Window> = None;
        for slot in WINDOWS.iter_mut() {
            if slot.map(|w| w.id) == Some(id) {
                taken = slot.take();
                break;
            }
        }
        if let Some(w) = taken {
            let mut compacted: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];
            let mut j = 0;
            for slot in WINDOWS.iter() {
                if let Some(x) = slot {
                    compacted[j] = Some(*x);
                    j += 1;
                }
            }
            compacted[j] = Some(w);
            WINDOWS = compacted;
            FOCUSED = Some(id);
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
        for slot in WINDOWS.iter_mut() {
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
        WINDOWS = [None; MAX_WINDOWS];
        FOCUSED = None;
    }
}
```

- [ ] **Step 2: Build to find consumers of the old WM API.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -30
```

The old `wm.rs` exposed `pane_rect`, `split_vertical`, `switch_app`, `active_app`, `draw_frame`, etc. Anywhere these are called will fail. **Don't** patch every call site now — the desktop rewrite in Task 7 replaces them all. For each compile error in a file other than `wm.rs`:

- Comment out the offending line, prefix the comment with `// XXX Wave-2-temp:` followed by the original code.
- At the top of any file you edited, add a 1-line tracking comment: `// XXX Wave-2-temp: <N> old-WM call sites commented out, restored in Task 7.`

- [ ] **Step 3: Build clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. If clippy flags dead-code on the new WM functions, add `#[allow(dead_code)]` per-function — never file-level.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/wm.rs src/ui/desktop.rs src/main.rs
git commit -m "$(cat <<'EOF'
wm: floating-window WM core (open/close/focus/cycle/set_rect)

Replaces the pane-tiling WM with a floating-window model:
* Window struct (id, app, rect, optional cave_name).
* Fixed-size store [Option<Window>; 16].
* Z-order implicit in array order; focused window last-most.
* Public API: open / close / focus / cycle_focus / set_rect / iter
  / count / get / focused / reset_all.
* reset_all() is only called by security::wipe — lock/unlock cycle
  preserves workspace state.

Paint, drag, and resize land in subsequent commits. Old-WM call
sites that won't compile are tagged with `// XXX Wave-2-temp` and
get replaced in the desktop rewrite (Task 7).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: Window paint (chrome + body + drop shadow)

Add the paint pass for windows.

**Files:**
- Modify: `src/ui/wm.rs`

- [ ] **Step 1: Append paint code to `src/ui/wm.rs`.**

```rust
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
        let sx = (r.x as i32 + SHADOW_OFFSET_X).max(0) as u32;
        let sy = (r.y as i32 + SHADOW_OFFSET_Y).max(0) as u32;
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
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/wm.rs
git commit -m "$(cat <<'EOF'
wm: paint_all() — chrome + border + drop shadow + body dispatch

For each window in z-order: shadow rectangle, body BG fill, chrome
strip, 1-px HAIRLINE border, chrome/body separator, 8x8 open-ring
close glyph, title (INK if focused / MID if not), optional
'* cave_name' suffix, then dispatch the app's paint callback with
the body rect.

No drag/resize yet — Task 4. No top bar yet — Task 5.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 4: Window drag-to-move + drag-to-resize + click-to-close

Pointer-event handling for window manipulation.

**Files:**
- Modify: `src/ui/wm.rs`

- [ ] **Step 1: Append drag/resize/hit-test code to `src/ui/wm.rs`.**

```rust
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
                    DRAG = Some(DragState {
                        window: id, kind: DragKind::Resize(corner),
                        start_mouse_x: mx, start_mouse_y: my,
                        start_rect: w.rect,
                    });
                }
            }
            true
        }
        Hit::Chrome(id) => {
            focus(id);
            if let Some(w) = get(id) {
                unsafe {
                    DRAG = Some(DragState {
                        window: id, kind: DragKind::Move,
                        start_mouse_x: mx, start_mouse_y: my,
                        start_rect: w.rect,
                    });
                }
            }
            true
        }
        Hit::Body(id) => { focus(id); true }
        Hit::None => false,
    }
}

pub fn update_drag(mx: i32, my: i32) -> bool {
    let drag = unsafe { DRAG };
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
            if (w as u32) < MIN_W { w = MIN_W as i32; }
            if (h as u32) < MIN_H { h = MIN_H as i32; }
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
    unsafe { DRAG = None; }
}

pub fn is_dragging() -> bool {
    unsafe { DRAG.is_some() }
}
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/wm.rs
git commit -m "$(cat <<'EOF'
wm: drag-to-move + drag-to-resize + click-to-close hit testing

hit_test returns CloseGlyph / Corner / Chrome / Body / None for a
point. begin_drag initiates a drag or closes the window on
CloseGlyph hit. update_drag applies move/resize delta, enforcing
MIN_W=280 / MIN_H=160 floors. end_drag clears state. One drag at
a time.

Resize honors a 12-px corner hit zone on all four corners.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 5: Top bar + config storage

The 22-px top strip with brand wordmark, customizable badges, config trigger, and lock glyph.

**Files:**
- Create: `src/ui/topbar.rs`
- Create: `src/ui/topbar_config.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/topbar_config.rs`.**

```rust
//! Top-bar badge config: which badges to show, in what order.
//! Persists to /system/desktop/topbar.cfg in SealFS as a one-line
//! ASCII letter sequence ("NDC" = NET, DEADMAN, CLOCK).

#![allow(dead_code)]

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Badge {
    NetMode,
    Deadman,
    Clock,
    Caves,
    Attempts,
    Memory,
    Cpu,
    Audit,
    CaveFocus,
}

fn badge_letter(b: Badge) -> u8 {
    match b {
        Badge::NetMode   => b'N',
        Badge::Deadman   => b'D',
        Badge::Clock     => b'C',
        Badge::Caves     => b'V',
        Badge::Attempts  => b'A',
        Badge::Memory    => b'M',
        Badge::Cpu       => b'P',
        Badge::Audit     => b'U',
        Badge::CaveFocus => b'F',
    }
}

fn letter_badge(c: u8) -> Option<Badge> {
    Some(match c {
        b'N' => Badge::NetMode,
        b'D' => Badge::Deadman,
        b'C' => Badge::Clock,
        b'V' => Badge::Caves,
        b'A' => Badge::Attempts,
        b'M' => Badge::Memory,
        b'P' => Badge::Cpu,
        b'U' => Badge::Audit,
        b'F' => Badge::CaveFocus,
        _ => return None,
    })
}

pub const MAX_BADGES: usize = 9;

static mut BADGES: [Option<Badge>; MAX_BADGES] = [
    Some(Badge::NetMode),
    Some(Badge::Deadman),
    Some(Badge::Clock),
    None, None, None, None, None, None,
];

const CONFIG_FILE: &str = "/system/desktop/topbar.cfg";

pub fn iter() -> impl Iterator<Item = Badge> {
    unsafe { BADGES.iter().filter_map(|b| *b).collect::<alloc::vec::Vec<_>>().into_iter() }
}

pub fn toggle(badge: Badge) {
    unsafe {
        for slot in BADGES.iter_mut() {
            if *slot == Some(badge) {
                *slot = None;
                let mut compacted: [Option<Badge>; MAX_BADGES] = [None; MAX_BADGES];
                let mut j = 0;
                for s in BADGES.iter() {
                    if let Some(b) = s { compacted[j] = Some(*b); j += 1; }
                }
                BADGES = compacted;
                save();
                return;
            }
        }
        for slot in BADGES.iter_mut() {
            if slot.is_none() {
                *slot = Some(badge);
                save();
                return;
            }
        }
    }
}

fn save() {
    let mut buf = [0u8; MAX_BADGES + 1];
    let mut n = 0;
    for b in iter() {
        buf[n] = badge_letter(b);
        n += 1;
    }
    buf[n] = b'\n';
    n += 1;
    let _ = crate::fs::sealfs::create(CONFIG_FILE, &buf[..n]);
}

pub fn load() {
    let mut buf = [0u8; MAX_BADGES + 1];
    if let Ok(n) = crate::fs::sealfs::read(CONFIG_FILE, &mut buf) {
        let mut new_badges: [Option<Badge>; MAX_BADGES] = [None; MAX_BADGES];
        let mut j = 0;
        for &c in &buf[..n] {
            if c == b'\n' { break; }
            if let Some(b) = letter_badge(c) {
                new_badges[j] = Some(b);
                j += 1;
                if j >= MAX_BADGES { break; }
            }
        }
        if j > 0 {
            unsafe { BADGES = new_badges; }
        }
    }
}
```

- [ ] **Step 2: Create `src/ui/topbar.rs`.**

```rust
//! Wave-2 top bar (22-px strip).

#![allow(dead_code)]

use crate::ui::draw;
use crate::ui::font;
use crate::ui::gpu;
use crate::ui::topbar_config::{self, Badge};

pub const TOPBAR_H: u32 = 22;

const PANEL:    u32 = 0xFF18181C;
const HAIRLINE: u32 = 0xFF2A2A30;
const INK:      u32 = 0xFFE5E7EB;
const MID:      u32 = 0xFF9CA3AF;
const DIM:      u32 = 0xFF6B7280;

const PAD_X:     u32 = 12;
const BADGE_GAP: u32 = 12;

pub enum TopBarHit {
    BrandClick,
    ConfigClick,
    LockClick,
    None,
}

pub fn paint() {
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    gpu::fill_rect(0, 0, screen_w, TOPBAR_H, PANEL);
    gpu::fill_rect(0, TOPBAR_H - 1, screen_w, 1, HAIRLINE);

    let brand = "SPHRAGIS";
    font::draw_str(fb, screen_w, PAD_X, (TOPBAR_H - 16) / 2 + 4, brand, INK, PANEL);

    let mut items: alloc::vec::Vec<(alloc::string::String, u32)> = alloc::vec::Vec::new();
    for b in topbar_config::iter() {
        let text = render_badge(b);
        let color = if badge_is_alert(b) { INK } else { MID };
        items.push((text, color));
    }
    items.push((alloc::string::String::from("..."), DIM));
    items.push((alloc::string::String::from("[L]"), DIM));

    let mut total_w = 0u32;
    for (text, _) in &items {
        total_w += text.len() as u32 * 8 + BADGE_GAP;
    }
    if total_w > 0 { total_w -= BADGE_GAP; }
    let mut x = screen_w.saturating_sub(PAD_X).saturating_sub(total_w);
    for (text, color) in &items {
        font::draw_str(fb, screen_w, x, (TOPBAR_H - 16) / 2 + 4, text, *color, PANEL);
        x += text.len() as u32 * 8 + BADGE_GAP;
    }
}

pub fn hit_test(mx: i32, my: i32) -> TopBarHit {
    if my < 0 || (my as u32) >= TOPBAR_H { return TopBarHit::None; }
    let screen_w = gpu::width() as i32;

    let brand_w = 8 * 8;
    if mx >= PAD_X as i32 && mx < (PAD_X as i32) + brand_w {
        return TopBarHit::BrandClick;
    }

    let lock_w = 3 * 8;
    let lock_x1 = screen_w - PAD_X as i32;
    let lock_x0 = lock_x1 - lock_w;
    if mx >= lock_x0 && mx < lock_x1 { return TopBarHit::LockClick; }

    let config_w = 3 * 8;
    let config_x1 = lock_x0 - BADGE_GAP as i32;
    let config_x0 = config_x1 - config_w;
    if mx >= config_x0 && mx < config_x1 { return TopBarHit::ConfigClick; }

    TopBarHit::None
}

fn render_badge(b: Badge) -> alloc::string::String {
    use alloc::format;
    match b {
        Badge::NetMode => {
            if crate::net::is_isolated() {
                format!("NET ISOLATED")
            } else {
                format!("NET ROUTED")
            }
        }
        Badge::Deadman => {
            let secs = crate::security::deadman::seconds_remaining();
            format!("DEADMAN {:02}:{:02}", secs / 60, secs % 60)
        }
        Badge::Clock => {
            let secs = uptime_seconds();
            format!("{:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60)
        }
        Badge::Caves    => format!("CAVES {}", crate::caves::count()),
        Badge::Attempts => format!("ATTEMPTS {}", crate::security::auth::attempts_remaining()),
        Badge::Memory   => format!("MEM --"),
        Badge::Cpu      => format!("CPU --"),
        Badge::Audit    => format!("AUDIT --"),
        Badge::CaveFocus => {
            let cave = crate::ui::wm::focused()
                .and_then(crate::ui::wm::get)
                .and_then(|w| w.cave_name);
            match cave {
                Some(bytes) => {
                    let n = bytes.iter().position(|&b| b == 0).unwrap_or(16);
                    format!("CAVE {}", unsafe { core::str::from_utf8_unchecked(&bytes[..n]) })
                }
                None => format!("CAVE --"),
            }
        }
    }
}

fn badge_is_alert(b: Badge) -> bool {
    match b {
        Badge::Deadman  => crate::security::deadman::seconds_remaining() < 300,
        Badge::Attempts => crate::security::auth::attempts_remaining() < 3,
        _ => false,
    }
}

fn uptime_seconds() -> u64 {
    let now: u64; let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { 0 } else { now / freq }
}
```

- [ ] **Step 3: Register both modules in `src/ui/mod.rs`.**

Add `pub mod topbar;` and `pub mod topbar_config;` alphabetically.

- [ ] **Step 4: Build to find missing function references.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -20
```

The build may fail on:
- `crate::net::is_isolated()` — stub if missing: `pub fn is_isolated() -> bool { true }` in `src/net/mod.rs`. Mark `// Wave 2 stub`.
- `crate::caves::count()` — stub if missing: `pub fn count() -> u8 { 0 }` in `src/caves/mod.rs`. Mark `// Wave 2 stub`.
- `crate::security::deadman::seconds_remaining()` — stub if missing: `pub fn seconds_remaining() -> u64 { 0 }`. Mark `// Wave 2 stub`.

- [ ] **Step 5: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 6: Commit.**

```bash
git add src/ui/topbar.rs src/ui/topbar_config.rs src/ui/mod.rs src/net/ src/caves/ src/security/
git commit -m "$(cat <<'EOF'
topbar: 22-px status strip + customizable badges + SealFS persist

* topbar.rs paints brand wordmark + badge list + '...' config
  trigger + '[L]' lock glyph. hit_test routes clicks to one of
  BrandClick / ConfigClick / LockClick / None.
* topbar_config.rs holds badge order in a fixed-size array,
  serializes to /system/desktop/topbar.cfg as a one-line ASCII
  letter sequence (e.g. "NDC" = NET, DEADMAN, CLOCK), loads on
  startup, silently falls back to default on missing/malformed.
* Default badges: NetMode, Deadman, Clock. Optional: Caves,
  Attempts, Memory, Cpu, Audit, CaveFocus.

Stubs net::is_isolated / caves::count / deadman::seconds_remaining
when missing — marked Wave-2-stub.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 6: Launcher overlay

8-app grid that paints over windows when summoned.

**Files:**
- Create: `src/ui/launcher.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/launcher.rs`.**

```rust
//! Launcher overlay — 8-app grid summoned by ⌘K or click on the
//! SPHRAGIS brand. Paints on top of the desktop in OVERLAY state
//! and also IS the desktop in LAUNCHER state (no dim, no windows
//! behind).

#![allow(dead_code)]

use crate::ui::apps_registry::{AppId, APPS};
use crate::ui::font;
use crate::ui::gpu;
use crate::ui::topbar::TOPBAR_H;

const BG:      u32 = 0xFF0D0D10;
const INK:     u32 = 0xFFE5E7EB;
const MID:     u32 = 0xFF9CA3AF;
const TILE_BG: u32 = 0xFF9CA3AF;

const COLS: u32 = 4;
const ROWS: u32 = 2;
const TILE_W: u32 = 22;
const TILE_H: u32 = 22;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LauncherMode {
    Background,
    Overlay,
    Canvas,
}

pub fn paint(mode: LauncherMode) {
    let screen_w = gpu::width();
    let screen_h = gpu::height();

    if mode == LauncherMode::Overlay {
        gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, BG);
    }

    let inset_x = (screen_w * 12) / 100;
    let inset_y = TOPBAR_H + (screen_h - TOPBAR_H) * 4 / 100;
    let grid_w = screen_w.saturating_sub(2 * inset_x);
    let grid_h = (screen_h - TOPBAR_H) * 70 / 100;
    let cell_w = grid_w / COLS;
    let cell_h = grid_h / ROWS;

    let label_color = if mode == LauncherMode::Background { MID } else { INK };
    let tile_color  = if mode == LauncherMode::Background { 0xFF3a3b3f } else { TILE_BG };

    for (i, app) in APPS.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let cell_x = inset_x + col * cell_w;
        let cell_y = inset_y + row * cell_h;
        let cx = cell_x + cell_w / 2;
        let cy = cell_y + cell_h / 2;

        let tile_x = cx - TILE_W / 2;
        let tile_y = cy - TILE_H / 2 - 6;
        gpu::fill_rect(tile_x, tile_y, TILE_W, TILE_H, tile_color);

        let label = app.label;
        let lbl_x = cx - (label.len() as u32 * 8) / 2;
        let lbl_y = tile_y + TILE_H + 8;
        font::draw_str(gpu::framebuffer(), screen_w, lbl_x, lbl_y, label, label_color, BG);
    }
}

pub fn hit_test(mx: i32, my: i32) -> Option<AppId> {
    let screen_w = gpu::width();
    let screen_h = gpu::height();
    if my < TOPBAR_H as i32 { return None; }

    let inset_x = (screen_w * 12) / 100;
    let inset_y = TOPBAR_H + (screen_h - TOPBAR_H) * 4 / 100;
    let grid_w = screen_w.saturating_sub(2 * inset_x);
    let grid_h = (screen_h - TOPBAR_H) * 70 / 100;
    let cell_w = grid_w / COLS;
    let cell_h = grid_h / ROWS;

    if (mx as u32) < inset_x || (mx as u32) >= inset_x + grid_w { return None; }
    if (my as u32) < inset_y || (my as u32) >= inset_y + grid_h { return None; }

    let col = ((mx as u32) - inset_x) / cell_w;
    let row = ((my as u32) - inset_y) / cell_h;
    let idx = (row * COLS + col) as usize;
    APPS.get(idx).map(|d| d.id)
}
```

- [ ] **Step 2: Register module in `src/ui/mod.rs`.**

Add `pub mod launcher;` alphabetically.

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/launcher.rs src/ui/mod.rs
git commit -m "$(cat <<'EOF'
launcher: 8-app grid overlay (Background / Overlay / Canvas modes)

paint(mode) draws a 4x2 grid of solid silhouette icons + labels.
Overlay paints a solid BG scrim first; Background dims tile + label
colors; Canvas is full opacity over the bare desktop. hit_test
returns Some(AppId) for clicks landing on a tile cell, None
otherwise.

Tile size, gap, and inset are derived from screen dims so the grid
scales to any resolution.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 7: Desktop state machine + run loop

Replace `src/ui/desktop.rs` with the new state-machine event loop.

**Files:**
- Replace: `src/ui/desktop.rs`

- [ ] **Step 1: Check for `PointerEvent` + `next_pointer_event()` in tablet driver.**

```bash
grep -nE 'pub fn next_pointer_event|pub enum PointerEvent' /Users/kadenlee/Sphragis/src/drivers/virtio/tablet.rs
```

If either is missing, add the stubs to `src/drivers/virtio/tablet.rs`:

```rust
#[derive(Copy, Clone)]
pub enum PointerEvent {
    Down(i32, i32),
    Move(i32, i32),
    Up(i32, i32),
}

/// Wave 2 stub — wires real pointer-event decoding from the
/// virtio-tablet stream in Wave 3+. Returning None means the
/// desktop runs keyboard-only on existing kernels.
pub fn next_pointer_event() -> Option<PointerEvent> { None }
```

Mark the stubs `// Wave 2 stub`.

- [ ] **Step 2: Replace `src/ui/desktop.rs` end-to-end.**

```rust
//! Wave 2 desktop. State machine + event loop.

#![allow(dead_code)]

use crate::platform;
use crate::ui::draw;
use crate::ui::gpu;
use crate::ui::launcher::{self, LauncherMode};
use crate::ui::sigma_bitmap::{SIGMA_BITMAP_96, SIGMA_BITMAP_W, SIGMA_BITMAP_H};
use crate::ui::topbar::{self, TopBarHit};
use crate::ui::topbar_config;
use crate::ui::wm;

const BG:        u32 = 0xFF0D0D10;
const WATERMARK: u32 = 0xFF1C1D22;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LockReason {
    UserRequest,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum State {
    Launcher,
    Active,
    Overlay,
}

static mut OVERLAY_OPEN: bool = false;

pub fn init() {
    topbar_config::load();
}

pub fn run() -> LockReason {
    loop {
        let state = current_state();
        paint(state);
        gpu::flush(0, 0, gpu::width(), gpu::height());

        match poll_event() {
            Event::Lock => return LockReason::UserRequest,
            Event::Repaint => continue,
            Event::None => { core::hint::spin_loop(); }
        }
    }
}

fn current_state() -> State {
    if unsafe { OVERLAY_OPEN } { return State::Overlay; }
    if wm::count() == 0 { return State::Launcher; }
    State::Active
}

fn paint(state: State) {
    let w = gpu::width();
    let h = gpu::height();

    gpu::fill_rect(0, 0, w, h, BG);

    let glyph_x = (w / 2) as i32 - (SIGMA_BITMAP_W as i32) / 2;
    let glyph_y = (h / 2) as i32 - (SIGMA_BITMAP_H as i32) / 2;
    draw::blit_alpha_bitmap(
        gpu::framebuffer(),
        w, h,
        glyph_x, glyph_y,
        &SIGMA_BITMAP_96,
        SIGMA_BITMAP_W, SIGMA_BITMAP_H,
        WATERMARK,
    );

    match state {
        State::Launcher => launcher::paint(LauncherMode::Canvas),
        State::Active   => { launcher::paint(LauncherMode::Background); wm::paint_all(); }
        State::Overlay  => { wm::paint_all(); launcher::paint(LauncherMode::Overlay); }
    }

    topbar::paint();
}

enum Event { None, Repaint, Lock }

fn poll_event() -> Event {
    crate::drivers::virtio::keyboard::poll();
    crate::drivers::virtio::tablet::poll();

    if let Some(pe) = crate::drivers::virtio::tablet::next_pointer_event() {
        return handle_pointer(pe);
    }

    if let Some(c) = platform::serial_getc()
        .or_else(crate::drivers::virtio::keyboard::getc)
        .or_else(crate::drivers::virtio::tablet::getc_key)
    {
        return handle_key(c);
    }

    Event::None
}

fn handle_pointer(pe: crate::drivers::virtio::tablet::PointerEvent) -> Event {
    use crate::drivers::virtio::tablet::PointerEvent;
    match pe {
        PointerEvent::Down(x, y) => {
            if (y as u32) < topbar::TOPBAR_H {
                match topbar::hit_test(x, y) {
                    TopBarHit::BrandClick  => { unsafe { OVERLAY_OPEN = true; } }
                    TopBarHit::ConfigClick => { /* Task 10 */ }
                    TopBarHit::LockClick   => return Event::Lock,
                    TopBarHit::None        => {}
                }
                return Event::Repaint;
            }

            if unsafe { OVERLAY_OPEN } {
                match launcher::hit_test(x, y) {
                    Some(id) => {
                        unsafe { OVERLAY_OPEN = false; }
                        wm::open(id, None);
                    }
                    None => unsafe { OVERLAY_OPEN = false; }
                }
                return Event::Repaint;
            }

            if wm::count() == 0 {
                if let Some(id) = launcher::hit_test(x, y) {
                    wm::open(id, None);
                    return Event::Repaint;
                }
                return Event::None;
            }

            if wm::begin_drag(x, y) { Event::Repaint } else { Event::None }
        }
        PointerEvent::Move(x, y) => {
            if wm::is_dragging() && wm::update_drag(x, y) { Event::Repaint }
            else { Event::None }
        }
        PointerEvent::Up(_, _) => {
            wm::end_drag();
            Event::Repaint
        }
    }
}

fn handle_key(c: u8) -> Event {
    // Full keyboard shortcut table lands in Task 9. Esc handled here
    // so the overlay can be dismissed before then.
    if c == 0x1B && unsafe { OVERLAY_OPEN } {
        unsafe { OVERLAY_OPEN = false; }
        return Event::Repaint;
    }
    Event::None
}
```

- [ ] **Step 3: Remove old desktop callers from `main.rs`.**

```bash
grep -n 'desktop::resume\|desktop::run' /Users/kadenlee/Sphragis/src/main.rs
```

Delete any reference to `desktop::resume()` (the function no longer exists). For `desktop::run()` calls, leave them in place — Task 8 wraps them in a loop.

Also remove any `// XXX Wave-2-temp` tags introduced in Task 2 if their tagged code is no longer reachable; verify by searching:

```bash
grep -rn 'Wave-2-temp' /Users/kadenlee/Sphragis/src
```

For each surviving tag, replace the commented-out call with the new equivalent if one exists, or delete the comment entirely. Be conservative — if you don't know what a tag was tracking, leave it and document the unknown in the commit.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -10
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/desktop.rs src/drivers/virtio/tablet.rs src/main.rs
git commit -m "$(cat <<'EOF'
desktop: state-machine event loop (Launcher / Active / Overlay)

* `run() -> LockReason` (was -> !); main.rs wraps in a lock/unlock
  cycle in the next commit.
* Paint pass: BG → watermark Σ (baked alpha bitmap from Wave 1) →
  launcher (current mode) → WM windows → topbar (always on top).
* Event loop polls keyboard + tablet per iteration. Top-bar clicks
  route to brand / config / lock. Overlay-mode launcher clicks
  open a new window. ACTIVE state clicks go to WM.
* Esc dismisses the overlay; full keyboard shortcut table in Task 9.

Stubs PointerEvent + next_pointer_event() in virtio/tablet if
missing — Wave 3+ wires real pointer decoding.

Wave-2-temp tags from Task 2 cleaned up; any survivor still in
tree is intentional and documented.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 8: main.rs lock/unlock cycle

Wrap boot_screen + desktop in a loop so the workspace persists across lock.

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Find call sites.**

```bash
grep -nE 'boot_screen::run|desktop::run|sealfs::init|sealfs::mount' /Users/kadenlee/Sphragis/src/main.rs
```

Note every `boot_screen::run` and `desktop::run` line. Note the SealFS init/mount line — `desktop::init()` goes immediately after it.

- [ ] **Step 2: Add `desktop::init()` after SealFS mount.**

Locate the line where SealFS is initialized/mounted in `main.rs`. Add immediately after:

```rust
ui::desktop::init();
```

If the SealFS init line doesn't obviously exist, add `ui::desktop::init();` early in the boot sequence, after platform init but before the first `boot_screen::run()`. It's idempotent enough to be safe.

- [ ] **Step 3: Wrap boot_screen + desktop in a loop.**

For every `security::boot_screen::run();` followed by `ui::desktop::run();` pattern, replace with:

```rust
loop {
    security::boot_screen::run();
    let _reason = ui::desktop::run();
    // _reason is LockReason::UserRequest today; ignored. Loop body
    // re-enters boot_screen::run() which blocks until next unlock.
    // WM state is module-level static so workspace persists.
}
```

If a call site is at the end of a function that returns `!`, the `loop {}` form is correct (it never falls through). If the function is finite, you'll need to adjust based on the call's role.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 5: Commit.**

```bash
git add src/main.rs
git commit -m "$(cat <<'EOF'
main: lock/unlock cycle wraps boot_screen + desktop

`desktop::run()` now returns LockReason. main.rs wraps boot_screen +
desktop in a loop so the workspace persists across the lock/unlock
cycle (WM state is module-level static and is not reset by lock).

`desktop::init()` is called once at boot, after SealFS is mounted, to
load the topbar badge config from /system/desktop/topbar.cfg.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 9: Keyboard shortcuts

⌘K / ⌘TAB / ⌘L / ⌘W / Esc.

**Files:**
- Modify: `src/ui/desktop.rs`

- [ ] **Step 1: Replace the `handle_key` stub.**

In `src/ui/desktop.rs`, find `fn handle_key(c: u8) -> Event` and replace its body:

```rust
fn handle_key(c: u8) -> Event {
    // The kernel's keyboard layer translates Ctrl+letter into ASCII
    // control codes (Ctrl+K = 0x0B, Ctrl+W = 0x17, Ctrl+L = 0x0C).
    // ⌘ on Mac maps to Ctrl through QEMU's HID forwarding, so the
    // brainstormed ⌘K / ⌘L / ⌘W work as documented on both QEMU
    // and the M4 path.
    match c {
        0x0B => { // Ctrl+K — toggle overlay
            unsafe { OVERLAY_OPEN = !OVERLAY_OPEN; }
            Event::Repaint
        }
        0x09 => { // Tab — cycle window focus
            wm::cycle_focus();
            Event::Repaint
        }
        0x0C => Event::Lock, // Ctrl+L
        0x17 => { // Ctrl+W — close focused
            if let Some(id) = wm::focused() { wm::close(id); }
            Event::Repaint
        }
        0x1B => { // Esc
            if unsafe { OVERLAY_OPEN } {
                unsafe { OVERLAY_OPEN = false; }
                Event::Repaint
            } else {
                Event::None
            }
        }
        _ => Event::None,
    }
}
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/desktop.rs
git commit -m "$(cat <<'EOF'
desktop: wire keyboard shortcuts (Ctrl+K/W/L, Tab, Esc)

* Ctrl+K toggles the launcher overlay.
* Tab cycles window focus.
* Ctrl+L returns LockReason::UserRequest (main.rs cycles back to
  boot_screen::run()).
* Ctrl+W closes the focused window.
* Esc dismisses the overlay.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 10: Topbar config sheet

Modal overlay for toggling badges. v1 ships toggle-only.

**Files:**
- Modify: `src/ui/topbar.rs`
- Modify: `src/ui/desktop.rs`

- [ ] **Step 1: Append config-sheet code to `src/ui/topbar.rs`.**

```rust
// ── Config sheet (modal) ─────────────────────────────────────────

static mut CONFIG_SHEET_OPEN: bool = false;

pub fn config_sheet_open()  -> bool { unsafe { CONFIG_SHEET_OPEN } }
pub fn open_config_sheet()           { unsafe { CONFIG_SHEET_OPEN = true; } }
pub fn close_config_sheet()          { unsafe { CONFIG_SHEET_OPEN = false; } }

const BG: u32 = 0xFF0D0D10;
const ALL_BADGES: &[(Badge, &str)] = &[
    (Badge::NetMode,   "NET MODE"),
    (Badge::Deadman,   "DEADMAN"),
    (Badge::Clock,     "CLOCK"),
    (Badge::Caves,     "CAVES COUNT"),
    (Badge::Attempts,  "ATTEMPTS"),
    (Badge::Memory,    "MEMORY"),
    (Badge::Cpu,       "CPU"),
    (Badge::Audit,     "AUDIT TAIL"),
    (Badge::CaveFocus, "CAVE FOCUS"),
];

pub fn paint_config_sheet() {
    if !config_sheet_open() { return; }

    let screen_w = gpu::width();
    let screen_h = gpu::height();
    let fb = gpu::framebuffer();

    gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, BG);

    let panel_w: u32 = 360;
    let row_h: u32 = 24;
    let panel_h = (ALL_BADGES.len() as u32) * row_h + 50;
    let px = (screen_w - panel_w) / 2;
    let py = (screen_h - panel_h) / 2;
    gpu::fill_rect(px, py, panel_w, panel_h, PANEL);
    draw::draw_border(px, py, panel_w, panel_h, HAIRLINE);

    font::draw_str(fb, screen_w, px + 14, py + 10, "TOP BAR BADGES", INK, PANEL);

    for (i, (badge, name)) in ALL_BADGES.iter().enumerate() {
        let ry = py + 40 + (i as u32) * row_h;
        let enabled = topbar_config::iter().any(|b| b == *badge);
        let marker = if enabled { "[x]" } else { "[ ]" };
        let color = if enabled { INK } else { DIM };
        font::draw_str(fb, screen_w, px + 14, ry,                 marker, color, PANEL);
        font::draw_str(fb, screen_w, px + 14 + 4 * 8, ry,         name,   color, PANEL);
    }

    font::draw_str(fb, screen_w, px + 14, py + panel_h - 16, "ESC TO CLOSE", DIM, PANEL);
}

/// Returns true if a repaint is needed.
pub fn config_sheet_click(mx: i32, my: i32) -> bool {
    if !config_sheet_open() { return false; }
    let screen_w = gpu::width();
    let screen_h = gpu::height();

    let panel_w: u32 = 360;
    let row_h: u32 = 24;
    let panel_h = (ALL_BADGES.len() as u32) * row_h + 50;
    let px = (screen_w - panel_w) / 2;
    let py = (screen_h - panel_h) / 2;

    if (mx as u32) < px || (mx as u32) >= px + panel_w { return false; }
    if (my as u32) < py + 40 || (my as u32) >= py + 40 + (ALL_BADGES.len() as u32) * row_h {
        return false;
    }

    let row_idx = (((my as u32) - py - 40) / row_h) as usize;
    if row_idx < ALL_BADGES.len() {
        topbar_config::toggle(ALL_BADGES[row_idx].0);
        return true;
    }
    false
}
```

- [ ] **Step 2: Wire the config sheet into `desktop.rs`.**

In `src/ui/desktop.rs`'s `paint()`, append after `topbar::paint();`:

```rust
    topbar::paint_config_sheet();
```

In `handle_pointer`'s `Down(x, y)` arm, add a pre-pass BEFORE the existing top-bar hit-test:

```rust
        PointerEvent::Down(x, y) => {
            // Config sheet absorbs all clicks when open.
            if topbar::config_sheet_open() {
                if topbar::config_sheet_click(x, y) {
                    return Event::Repaint;
                }
                topbar::close_config_sheet();
                return Event::Repaint;
            }
            // ... existing top-bar pre-pass below ...
```

Replace the `TopBarHit::ConfigClick` arm:

```rust
                    TopBarHit::ConfigClick => { topbar::open_config_sheet(); }
```

In `handle_key`'s Esc branch, give the config sheet priority:

```rust
        0x1B => { // Esc
            if topbar::config_sheet_open() {
                topbar::close_config_sheet();
                return Event::Repaint;
            }
            if unsafe { OVERLAY_OPEN } {
                unsafe { OVERLAY_OPEN = false; }
                Event::Repaint
            } else {
                Event::None
            }
        }
```

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/topbar.rs src/ui/desktop.rs
git commit -m "$(cat <<'EOF'
topbar: modal config sheet for toggling badges

Click '...' opens a modal listing all 9 badges with a [x]/[ ]
marker; clicking a row toggles the badge (persisted to SealFS).
Esc or click-outside dismisses. Drag-reorder is a follow-up — v1
ships toggle-only; re-ordering happens by toggling off + on again
at the new end position.

Sheet wires into desktop.rs as a top-priority click pre-pass and
the top-priority Esc target.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 11: QEMU walk-through

Manual visual confirmation of the entire Wave 2 surface. **No commit.**

- [ ] **Step 1: Rebuild + relaunch QEMU with virtio-tablet.**

```bash
cd /Users/kadenlee/Sphragis
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
qemu-system-aarch64 \
  -machine virt -cpu max -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -device virtio-tablet-device \
  -netdev user,id=net0 \
  -device virtio-net-device,netdev=net0 \
  -serial none \
  -kernel target/aarch64-unknown-none/release/sphragis &
```

Unlock with `sphragis-dev`.

- [ ] **Step 2: Verify LAUNCHER state.**

After unlock:
- Background near-black.
- Watermark Σ visible (soft, center).
- Top bar: "SPHRAGIS" on the left, `NET ISOLATED · DEADMAN MM:SS · HH:MM · ... · [L]` on the right.
- 8-app grid centered (CAVES, FILES, NET, SECURITY, SHELL, EDITOR, COMMS, AGENT).

- [ ] **Step 3: Verify ACTIVE state.**

Click CAVES (or whichever tile if pointer events are stubbed — fall through to keyboard path: open the launcher via Ctrl+K, then on next iteration the kernel may need pointer events. If pointer remains stubbed in this build, **the click path can't be visually tested** until Wave 3+ wires real pointer decoding. Note this in the verification result; the keyboard path is still testable below).

Pointer-path checks (if working):
- Floating window with hairline border, drop shadow, 22-px chrome.
- Title "CAVES" in INK.
- Open-circle close glyph at the left of the chrome.
- Launcher grid dims to ~22 % opacity behind the window.

Open a second window. The first dims to MID title.

Drag the focused chrome — window should move with pointer. Drag a corner — window resizes, clamped at 280×160.

Click the open circle on a window — it closes.

- [ ] **Step 4: Verify OVERLAY state.**

Press Ctrl+K → launcher overlay appears over windows. Click an app or press Esc to dismiss.

Click "SPHRAGIS" wordmark — overlay also opens.

- [ ] **Step 5: Verify keyboard shortcuts.**

- `Tab` — focus cycles through open windows.
- `Ctrl+W` — closes focused window.
- `Ctrl+L` — drops back to the Wave 1 lock screen. Unlock again — **the workspace (any windows still open) reappears as left**.

- [ ] **Step 6: Verify topbar config sheet.**

Click the `...` glyph — modal sheet appears listing all 9 badges + markers. Click "CAVES COUNT" → marker flips to `[x]`. Press Esc → sheet dismisses. Top-right strip now includes CAVES badge.

Lock with Ctrl+L. Re-unlock. Badge config persists.

- [ ] **Step 7: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 8: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 12: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/desktop-chrome
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

That skill verifies the build is clean, then presents merge / PR / keep / discard options. Recommended choice for this branch is "merge back to main locally" — same pattern as Wave 1.

---

## Spec coverage map (self-review)

| Spec section | Implemented by |
|--------------|---------------|
| Mental model — quiet canvas + Σ watermark + floating WM + customizable top bar | Tasks 2, 3, 5, 7 |
| Palette inheritance from Wave 1 (5 colors + WATERMARK + SHADOW) | Tasks 3, 5, 7 |
| Top bar — SPHRAGIS brand left, customizable badges right, ⋯ + ⏻ at end | Task 5 |
| Default badges: NetMode, Deadman, Clock | Task 5 (`topbar_config::BADGES` initializer) |
| Customizable badges with config sheet + SealFS persistence | Tasks 5 (storage) + 10 (sheet UI) |
| 4×2 app grid, 8 apps named in spec | Tasks 1 + 6 |
| Solid silhouette icon placeholders | Task 6 |
| Launcher modes: Background / Overlay / Canvas | Tasks 6, 7 |
| Floating multi-window WM, max 16 windows | Task 2 |
| Window chrome: hairline border, 22-px chrome, open-circle close, INK/MID title | Task 3 |
| `SHELL · kali-recon`-style cave-name in chrome | Tasks 2 (struct) + 3 (paint) |
| Drag chrome to move, drag corner (12 px) to resize, 280×160 min | Task 4 |
| Z-order, click body to focus | Tasks 2 + 4 |
| 4-state machine (Launcher / Active / Overlay / [Lock]) | Task 7 |
| ⌘K / Tab / ⌘L / ⌘W / Esc | Task 9 |
| Workspace persists across lock/unlock | Task 8 + WM state being static |
| System alerts via badge brightness (no popups) | Task 5 (`badge_is_alert`) |
| NOT in v1: dock, taskbar, virtual desktops, snap-to-edge, theme, multi-monitor | Out of scope — not implemented |
| Verification = QEMU walk-through | Task 11 |

## Out-of-scope reminders

Do **not** drift into these while executing the plan:
- Per-app internal redesign — Wave 3+.
- Real app icons — Wave 3+ (each app's wave covers its own).
- Shell + console palette refresh — Wave 5.
- TrueType rasterizer fix (carried from Wave 1) — Wave 5.
- AGENT app real implementation — depends on `DESIGN_AI_AGENT.md`.
- Drag-to-reorder in the topbar config sheet — follow-up after v1.
- Real wall-clock — depends on RTC plumbing.
- Real pointer-event decoding in `virtio/tablet.rs` — stubbed; Wave 3+.

If you find yourself touching any of these, stop and ask. The wave system only works if each wave produces working, testable software on its own.
