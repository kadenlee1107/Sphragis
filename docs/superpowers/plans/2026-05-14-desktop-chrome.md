# Desktop Chrome + App Launcher (Wave 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current pane-based cyberpunk desktop with a quiet-canvas + floating multi-window experience per `docs/superpowers/specs/2026-05-14-desktop-chrome-design.md`.

**Architecture:** New `src/ui/wm.rs` is a floating window manager (windows with x/y/w/h, drag-to-move, drag-corner-to-resize, click-to-focus, z-ordered). New `src/ui/topbar.rs` paints the top status strip and handles its click targets. Rewritten `src/ui/desktop.rs` runs a state machine (LAUNCHER / ACTIVE / OVERLAY) and dispatches events to the WM, top bar, and launcher overlay. The lock cycle becomes a real loop in `main.rs` so workspace state can persist across re-locks. App internals (the 8 apps in `src/ui/apps/`) are NOT modified — Wave 2 wraps them in window chrome and routes input; their visuals stay cyberpunk until Wave 3+.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` (no_std, alloc-OK). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: QEMU `-machine virt -cpu max -display cocoa`. No `cargo test` harness in this crate — verification contract is "build + clippy clean under `-D warnings`" + the QEMU walkthrough in Task 9.

**Verification reality check.** Same as Wave 1: the kernel is `#![no_std] #![no_main]` with no test runner. Every task's verification step is build + clippy + (for visible changes) a QEMU eyeball check. No unit tests because there's no harness to run them in.

---

## File structure

| File | Responsibility | Status |
|------|----------------|--------|
| `src/ui/wm.rs` | Floating WM: window storage, focus, paint, drag/resize | **rewritten** (current is pane-based) |
| `src/ui/desktop.rs` | State machine, event loop, repaint orchestration | **rewritten** |
| `src/ui/topbar.rs` | Top status strip paint + click | **new** |
| `src/ui/launcher.rs` | 8-app launcher overlay + dimmed-grid paint | **new** |
| `src/ui/apps_registry.rs` | Static `AppId` enum + per-app metadata (name, icon-paint, open-fn) | **new** |
| `src/ui/mod.rs` | Register new modules | modify |
| `src/main.rs` | Wrap boot_screen + desktop in a re-entrant loop | modify (substantive) |
| `src/ui/apps/*` (8 app files) | App internals (unchanged in this wave) | untouched |

---

## Pre-flight

- [ ] **Step 0a: Branch + clean state.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/desktop-chrome
```
Expected: clean working tree before, on `feat/desktop-chrome` after.

- [ ] **Step 0b: Sanity-check the baseline builds.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
```
Expected: `Finished release profile`. If the baseline doesn't build, fix that first — every later task assumes a clean baseline.

---

### Task 1: App registry — central app metadata

Defines the contract Wave 2's WM uses to launch apps. Wave 2 wraps the existing 7 app modules (caves_mgr, comms, dashboard/security/editor/filemanager/netmon — note the spec collapses dashboard into SECURITY for now) plus a stubbed AGENT. The registry exposes each app's name, icon-paint, and `open()` function. Apps themselves aren't modified; the registry is a thin static table that calls into them.

**Files:**
- Create: `src/ui/apps_registry.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/apps_registry.rs`.**

```rust
//! Static registry of every app the launcher shows.
//!
//! Each app entry has:
//!   * a stable `AppId` (used by the WM to identify which app a
//!     window is hosting),
//!   * a UPPERCASE display name (rendered as the launcher tile label
//!     and as the window title chrome),
//!   * an icon-paint function (Wave 2 uses a placeholder 22×22 rounded
//!     square; Wave 3+ replaces per-app),
//!   * an `open()` function that the WM calls when the user picks the
//!     tile. `open()` returns the window dimensions it wants and the
//!     paint callback the WM should dispatch to.
//!
//! Wave 2 does NOT modify any app's internals. Apps get the new chrome
//! because the WM owns the chrome — the apps' own paint code runs
//! inside the window body region.

#![allow(dead_code)]

use crate::ui::gpu;

/// Stable identifier for each launchable app. Order matches the 4×2
/// launcher grid (CAVES top-left, AGENT bottom-right).
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

impl AppId {
    /// Display name shown on launcher tiles and in window chrome.
    pub const fn name(self) -> &'static str {
        match self {
            AppId::Caves    => "CAVES",
            AppId::Files    => "FILES",
            AppId::Net      => "NET",
            AppId::Security => "SECURITY",
            AppId::Shell    => "SHELL",
            AppId::Editor   => "EDITOR",
            AppId::Comms    => "COMMS",
            AppId::Agent    => "AGENT",
        }
    }

    /// Default window size when the app is first opened, in pixels.
    /// Apps that want a different default size override here.
    pub const fn default_size(self) -> (u32, u32) {
        match self {
            // Shell wants a wider window because it shows monospace
            // text-mode content; the others get a balanced default.
            AppId::Shell => (760, 480),
            _            => (560, 400),
        }
    }

    /// Iteration helper: every AppId in launcher-grid order.
    pub const fn all() -> [AppId; 8] {
        [AppId::Caves, AppId::Files, AppId::Net, AppId::Security,
         AppId::Shell, AppId::Editor, AppId::Comms, AppId::Agent]
    }
}

/// Wave-2 placeholder icon: a 22×22 rounded square in the given color.
/// `x, y` are the top-left of the 22×22 slot. Wave 3+ replaces this
/// with per-app icons (each app's redesign wave introduces its own
/// real icon paint).
pub fn paint_placeholder_icon(_id: AppId, x: u32, y: u32, color: u32) {
    // No rounding in v1 — just a flat 22×22 square. Rounding would
    // need a tiny corner-mask helper; YAGNI for the placeholder.
    gpu::fill_rect(x, y, 22, 22, color);
}
```

- [ ] **Step 2: Wire the module in.**

In `src/ui/mod.rs`, add `pub mod apps_registry;` next to the other `pub mod` declarations (after `pub mod apps;` is fine).

- [ ] **Step 3: Build + clippy.**

```bash
cd /Users/kadenlee/Sphragis
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. The new module is `#![allow(dead_code)]` so the unused functions don't trip the lint.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps_registry.rs src/ui/mod.rs
git commit -m "$(cat <<'EOF'
ui: add apps_registry — central AppId enum + per-app metadata

Defines the contract Wave 2's new WM uses to find every launchable
app. Wave 2 ships:
  * AppId enum: Caves, Files, Net, Security, Shell, Editor, Comms,
    Agent (matches the 4×2 launcher grid in spec).
  * AppId::name() — uppercase display name for tile labels + window
    titles.
  * AppId::default_size() — initial floating-window size per app.
  * paint_placeholder_icon — 22×22 MID-color square, used until each
    app's own Wave 3+ pass introduces a real icon.

App modules in src/ui/apps/* are untouched. This registry is purely
the metadata table the WM reads.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 2: Window struct + WM core (open/close/focus/paint)

Replace the pane-based WM with a floating WM. Every open app is a `Window` with position, size, focus state, optional cave name. The WM stores up to 8 windows (matches the 8 apps; reasonable cap for v1). Paint walks back-to-front so the focused window draws last (on top).

**Files:**
- Modify: `src/ui/wm.rs` (rewrites the bulk of the file; preserves only constants the rest of the kernel imports — see the `Constants and external API contract` section below)

#### Constants and external API contract

Before rewriting wm.rs, check what other files import from it. The current wm.rs exposes `APP_SHELL`, `APP_DASHBOARD`, `APP_NETMON`, `APP_EDITOR`, `flush_all`, `active_app`, `switch_app`, `request_redraw`, `reset_for_cave_switch`, `content_rect`, `pane_count`, etc.

```bash
cd /Users/kadenlee/Sphragis
rg 'wm::' src/ --type rust | grep -v 'src/ui/wm.rs' | awk -F'wm::' '{print $2}' | awk '{print $1}' | sort -u
```

Whatever symbols come back, we must either re-provide them or update each caller. For Wave 2's scope, we **keep callers compiling** by exposing thin shims: `reset_for_cave_switch`, `flush_all`, `request_redraw` stay as functions (no-ops or simple repaint triggers in the new model). The pane-specific symbols (`APP_*`, `switch_app`, `active_app`, `content_rect`, `pane_count`) get migrated by Task 6's desktop rewrite, which is the only consumer.

- [ ] **Step 1: Snapshot the old wm.rs callers, then rewrite wm.rs.**

```bash
cd /Users/kadenlee/Sphragis
rg 'wm::' src/ --type rust | grep -v 'src/ui/wm.rs' > /tmp/old-wm-callers.txt
wc -l /tmp/old-wm-callers.txt
```

Expected: a handful of caller sites in `desktop.rs`, possibly `console.rs`, possibly `shell.rs`. Glance through to know what shims you need to keep.

- [ ] **Step 2: Replace `src/ui/wm.rs` with the new floating WM.**

Replace the entire file contents with:

```rust
//! Floating window manager — Wave 2.
//!
//! Replaces the pane-based WM. Up to MAX_WINDOWS app windows are
//! stored as `Option<Window>` slots. Windows are kept in z-order;
//! `Z_ORDER[0]` is back-most, `Z_ORDER[count-1]` is front-most /
//! focused. Paint walks in z-order so the focused window draws last.
//!
//! Mouse and keyboard event handling is upstream (in desktop.rs);
//! this module exposes the queries and mutators they call.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};

use crate::ui::gpu;
use crate::ui::draw;
use crate::ui::font;
use crate::ui::apps_registry::AppId;

/// Maximum simultaneously-open app windows. 8 = one of each app
/// (matches the launcher grid). Beyond that, the WM rejects opens.
pub const MAX_WINDOWS: usize = 8;

/// Window chrome height in pixels — matches the lock-screen and
/// top-bar visual language.
pub const CHROME_H: u32 = 22;

/// Resize hit zone in pixels at each corner.
pub const RESIZE_GRAB: u32 = 12;

/// Minimum window size — below this the chrome runs out of room.
pub const MIN_W: u32 = 280;
pub const MIN_H: u32 = 160;

// Palette inherited from Wave 1 — duplicated here so wm.rs doesn't
// need to depend on the security::boot_screen module. Source of truth
// is the spec; if Wave 1 ever moves these into ui::palette, update.
const BG:       u32 = 0xFF0D0D10;
const PANEL:    u32 = 0xFF18181C;
const HAIRLINE: u32 = 0xFF2A2A30;
const INK:      u32 = 0xFFE5E7EB;
const MID:      u32 = 0xFF6B7280;

/// One open window.
#[derive(Copy, Clone)]
pub struct Window {
    pub app:        AppId,
    pub x:          i32,
    pub y:          i32,
    pub w:          u32,
    pub h:          u32,
    /// Optional cave context this window's app is running inside.
    /// Renders in the chrome as `TITLE · cave_name`. Set at open
    /// time by the caller (caves manager fills it in when opening a
    /// shell into a specific cave). None = the window runs in the
    /// host context.
    pub cave_name:  Option<&'static str>,
}

/// Window-slot storage. `None` = empty slot.
static mut WINDOWS: [Option<Window>; MAX_WINDOWS] = [None; MAX_WINDOWS];

/// Z-order: front-most last. Each entry is an index into WINDOWS.
/// Length tracked by `Z_COUNT` because we're no-std.
static mut Z_ORDER: [usize; MAX_WINDOWS] = [0; MAX_WINDOWS];
static mut Z_COUNT: usize = 0;

/// True when the WM needs a full repaint (any window change).
static NEEDS_REDRAW: AtomicBool = AtomicBool::new(true);

/// Open a new window for `app`. Returns the slot index, or None if
/// all slots are full. Position is the canonical "next-window" offset
/// (stack each new window down-right of the last so they don't
/// stack identically).
pub fn open(app: AppId, cave_name: Option<&'static str>) -> Option<usize> {
    let (w, h) = app.default_size();
    // Find a free slot.
    let slot = unsafe {
        let mut found = None;
        for i in 0..MAX_WINDOWS {
            if WINDOWS[i].is_none() {
                found = Some(i);
                break;
            }
        }
        found
    }?;
    // Cascade offset: 32 px down-right per existing window.
    let n = unsafe { Z_COUNT } as i32;
    let x = 120 + n * 32;
    let y = 80  + n * 32;
    unsafe {
        WINDOWS[slot] = Some(Window { app, x, y, w, h, cave_name });
        Z_ORDER[Z_COUNT] = slot;
        Z_COUNT += 1;
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
    Some(slot)
}

/// Close a window by slot index. Quietly does nothing if the slot is
/// already empty. Returns the new front-most slot (if any) so callers
/// can re-focus.
pub fn close(slot: usize) -> Option<usize> {
    if slot >= MAX_WINDOWS { return current_focus(); }
    unsafe {
        if WINDOWS[slot].is_none() { return current_focus(); }
        WINDOWS[slot] = None;
        // Remove from z-order.
        let mut i = 0;
        while i < Z_COUNT {
            if Z_ORDER[i] == slot {
                for j in i..Z_COUNT - 1 {
                    Z_ORDER[j] = Z_ORDER[j + 1];
                }
                Z_COUNT -= 1;
                break;
            }
            i += 1;
        }
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
    current_focus()
}

/// The front-most window's slot, or None if no windows are open.
pub fn current_focus() -> Option<usize> {
    unsafe {
        if Z_COUNT == 0 { None } else { Some(Z_ORDER[Z_COUNT - 1]) }
    }
}

/// True when at least one window is open.
pub fn any_open() -> bool {
    unsafe { Z_COUNT > 0 }
}

/// Number of open windows.
pub fn count() -> usize {
    unsafe { Z_COUNT }
}

/// Bring the given slot to the front of the z-order.
pub fn focus(slot: usize) {
    unsafe {
        let mut found = None;
        for i in 0..Z_COUNT {
            if Z_ORDER[i] == slot { found = Some(i); break; }
        }
        if let Some(i) = found {
            // Slide everything after `i` down by one, then put slot
            // at the top.
            for j in i..Z_COUNT - 1 {
                Z_ORDER[j] = Z_ORDER[j + 1];
            }
            Z_ORDER[Z_COUNT - 1] = slot;
        }
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Cycle focus to the next window (back-most → front). Used by ⌘TAB.
pub fn cycle_focus() {
    unsafe {
        if Z_COUNT < 2 { return; }
        // The current front becomes the back; everyone shifts up.
        let new_back = Z_ORDER[Z_COUNT - 1];
        for i in (1..Z_COUNT).rev() {
            Z_ORDER[i] = Z_ORDER[i - 1];
        }
        Z_ORDER[0] = new_back;
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Read-only snapshot of a window. None if slot is empty.
pub fn get(slot: usize) -> Option<Window> {
    if slot >= MAX_WINDOWS { return None; }
    unsafe { WINDOWS[slot] }
}

/// Move a window's top-left by (dx, dy). Clamps so the window stays
/// at least partly on-screen.
pub fn move_by(slot: usize, dx: i32, dy: i32, screen_w: u32, screen_h: u32) {
    if slot >= MAX_WINDOWS { return; }
    unsafe {
        if let Some(ref mut win) = WINDOWS[slot] {
            win.x = (win.x + dx).max(-((win.w as i32) - 60))
                                .min(screen_w as i32 - 60);
            win.y = (win.y + dy).max(0)
                                .min(screen_h as i32 - CHROME_H as i32);
        }
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Resize a window by dragging the bottom-right corner. Clamps to
/// MIN_W/MIN_H.
pub fn resize_to(slot: usize, new_w: u32, new_h: u32) {
    if slot >= MAX_WINDOWS { return; }
    unsafe {
        if let Some(ref mut win) = WINDOWS[slot] {
            win.w = new_w.max(MIN_W);
            win.h = new_h.max(MIN_H);
        }
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Paint every window back-to-front. Caller is responsible for
/// painting the canvas + watermark beforehand and the top bar
/// afterwards.
pub fn paint_all(fb: *mut u32, screen_w: u32, screen_h: u32) {
    let z = unsafe { Z_ORDER };
    let n = unsafe { Z_COUNT };
    for i in 0..n {
        let slot = z[i];
        let focused = i == n - 1;
        if let Some(win) = get(slot) {
            paint_window(fb, screen_w, screen_h, &win, focused);
        }
    }
}

/// Paint a single window — chrome + body region (body is empty in
/// Wave 2; the app paint pass runs after this in desktop.rs).
fn paint_window(
    _fb: *mut u32,
    screen_w: u32,
    _screen_h: u32,
    win: &Window,
    focused: bool,
) {
    // Off-screen clip
    if win.x + win.w as i32 <= 0 { return; }
    if win.x >= screen_w as i32   { return; }

    let ux = win.x.max(0) as u32;
    let uy = win.y.max(0) as u32;

    // Body background — fills the whole window rect; the chrome
    // overpaints the top CHROME_H pixels.
    gpu::fill_rect(ux, uy, win.w, win.h, BG);

    // Border (1 px hairline) — top, bottom, left, right.
    draw::draw_border(ux, uy, win.w, win.h, HAIRLINE);

    // Chrome strip.
    gpu::fill_rect(ux, uy, win.w, CHROME_H, PANEL);
    // Chrome bottom hairline.
    gpu::fill_rect(ux, uy + CHROME_H - 1, win.w, 1, HAIRLINE);

    // Close glyph — 8×8 ring at the far-left.
    let cg_x = ux + 8;
    let cg_y = uy + (CHROME_H - 8) / 2;
    draw::draw_border(cg_x, cg_y, 8, 8, MID);

    // Title text. "APP" if no cave, "APP · cave_name" if caved.
    let title_color = if focused { INK } else { MID };
    let title_x = ux + 24;
    let title_y = uy + (CHROME_H - 16) / 2 + 4; // baseline-ish
    font::draw_str(_fb, screen_w, title_x, title_y, win.app.name(), title_color, PANEL);
    if let Some(cave) = win.cave_name {
        let app_len = win.app.name().len() as u32;
        let sep_x = title_x + app_len * 8 + 4;
        font::draw_str(_fb, screen_w, sep_x, title_y, "·", MID, PANEL);
        font::draw_str(_fb, screen_w, sep_x + 12, title_y, cave, title_color, PANEL);
    }
}

/// True if (x, y) is over the chrome of the focused window. Used by
/// desktop.rs to start a drag-move.
pub fn focused_chrome_hit(x: i32, y: i32) -> Option<usize> {
    let slot = current_focus()?;
    let win = get(slot)?;
    if x >= win.x + 16 && x < win.x + win.w as i32
        && y >= win.y && y < win.y + CHROME_H as i32 { Some(slot) } else { None }
}

/// True if (x, y) is over the close glyph of the focused window.
pub fn focused_close_hit(x: i32, y: i32) -> Option<usize> {
    let slot = current_focus()?;
    let win = get(slot)?;
    let cg_x = win.x + 8;
    let cg_y = win.y + (CHROME_H as i32 - 8) / 2;
    if x >= cg_x && x < cg_x + 8 && y >= cg_y && y < cg_y + 8 { Some(slot) } else { None }
}

/// True if (x, y) is over the bottom-right resize corner of the
/// focused window.
pub fn focused_resize_hit(x: i32, y: i32) -> Option<usize> {
    let slot = current_focus()?;
    let win = get(slot)?;
    let cx = win.x + win.w as i32;
    let cy = win.y + win.h as i32;
    if x >= cx - RESIZE_GRAB as i32 && x < cx
        && y >= cy - RESIZE_GRAB as i32 && y < cy { Some(slot) } else { None }
}

/// Topmost window slot under (x, y), or None.
pub fn slot_at(x: i32, y: i32) -> Option<usize> {
    let z = unsafe { Z_ORDER };
    let n = unsafe { Z_COUNT };
    for i in (0..n).rev() {
        let slot = z[i];
        if let Some(win) = get(slot) {
            if x >= win.x && x < win.x + win.w as i32
                && y >= win.y && y < win.y + win.h as i32 {
                return Some(slot);
            }
        }
    }
    None
}

/// Mark the WM as needing a full repaint.
pub fn request_redraw() {
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Read-and-clear "needs redraw" flag.
pub fn take_redraw() -> bool {
    NEEDS_REDRAW.swap(false, Ordering::Relaxed)
}

/// Force a full-frame flush — equivalent to the old `flush_all`. Kept
/// because shell.rs and console.rs call it after every keystroke; the
/// new WM model still needs the same fullscreen flush semantics.
pub fn flush_all() {
    let w = gpu::width();
    let h = gpu::height();
    gpu::flush(0, 0, w, h);
}

/// Reset all WM state — called when a cave is exited so we don't
/// leak window state into the resumed host context. Wave 2 wipes
/// everything; future waves may persist host-context windows across
/// cave-switch.
pub fn reset_for_cave_switch() {
    unsafe {
        for i in 0..MAX_WINDOWS { WINDOWS[i] = None; }
        Z_COUNT = 0;
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}
```

- [ ] **Step 3: Update consumers that imported the old pane API.**

Run the caller probe again:

```bash
cd /Users/kadenlee/Sphragis
rg 'wm::APP_|wm::switch_app|wm::active_app|wm::content_rect|wm::pane_count|wm::split_|wm::active_cave_indicator|wm::draw_frame|wm::set_render_target|wm::pane_app|wm::pane_rect|wm::is_close_focused|wm::focus_close_button|wm::unfocus_close_button|wm::init_panes_pub' src/ --type rust | grep -v 'src/ui/wm.rs'
```

For every match, the file needs an update. Task 6 (desktop rewrite) eliminates the bulk of these. For each non-desktop caller, comment out the call temporarily with a `// TODO Wave 2: switch to new WM API` and let Task 6 sweep them. Specifically expect: `src/ui/shell.rs` and `src/ui/console.rs` may reference `flush_all` (kept) and possibly status-bar functions (drop).

- [ ] **Step 4: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -10
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -5
```

Expected: clean build. Likely call-site errors will be in `desktop.rs` (which we're rewriting in Task 6 anyway), `shell.rs`, `console.rs`. Resolve each by either keeping a compatibility shim in `wm.rs` (preferred for things shell and console rely on) or commenting at the call site.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/wm.rs
git commit -m "$(cat <<'EOF'
wm: rewrite as floating window manager

Replaces the pane-based model with a floating one: up to 8 Window
slots (app, x, y, w, h, optional cave_name), Z_ORDER tracking the
front-most last, paint walks back-to-front so the focused window
draws on top.

Public API: open / close / focus / cycle_focus / move_by /
resize_to / paint_all / slot_at + chrome/close/resize hit-tests.

flush_all and reset_for_cave_switch kept as compatibility shims so
shell.rs and console.rs don't break in this commit; Tasks 6+
rewrite their consumers properly.

Wave 2 covers the WM only — apps are unmodified and don't yet paint
into the body region. Task 6's desktop rewrite hooks the app paint
pass in.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 3: Top bar — paint + click handling

A new file. Renders the 22-px-tall top strip: "SPHRAGIS" wordmark on the left, customizable badges on the right, ending with `⋯` (config trigger) and `⏻` (lock). Customization persistence comes in Task 8; Task 3 just ships the hardcoded default set so the visual lands.

**Files:**
- Create: `src/ui/topbar.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/topbar.rs`.**

```rust
//! Desktop top status bar.
//!
//! 22 px tall, spans the full screen width. Left = "SPHRAGIS"
//! wordmark (click = launcher overlay). Right = customizable status
//! badges + `⋯` config trigger + `⏻` lock glyph.
//!
//! Wave 2 ships the default badge set hardcoded. Task 8 layers the
//! BatFS-persisted user customization on top.

#![allow(dead_code)]

use crate::ui::gpu;
use crate::ui::draw;
use crate::ui::font;

pub const TOPBAR_H: u32 = 22;

// Palette (mirrors Wave 1 / wm.rs). See note in wm.rs about
// eventually centralizing.
const BG:       u32 = 0xFF0D0D10;
const PANEL:    u32 = 0xFF18181C;
const HAIRLINE: u32 = 0xFF2A2A30;
const INK:      u32 = 0xFFE5E7EB;
const MID:      u32 = 0xFF6B7280;
const FAINT:    u32 = 0xFF4B5563;

/// Badge identifiers — the union of every status item the user can
/// add to the top-right strip.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BadgeId {
    NetMode,    // "NET ISOLATED" / "NET ROUTED"
    Deadman,    // "DEADMAN HH:MM"
    Clock,      // "HH:MM"
    CavesCount, // "n CAVES"
    Attempts,   // "n ATTEMPTS" (warning only when low)
}

/// Default badge order shipped out of the box.
pub const DEFAULT_BADGES: &[BadgeId] = &[
    BadgeId::NetMode,
    BadgeId::Deadman,
    BadgeId::Clock,
];

/// Click target identification — returned by `hit_test`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TopbarHit {
    Brand,
    Gear,
    Lock,
    None,
}

/// Paint the top bar. `screen_w` is the framebuffer width; badges
/// are the live list (DEFAULT_BADGES in v1, user-configured later).
pub fn paint(fb: *mut u32, screen_w: u32, badges: &[BadgeId]) {
    // Strip background — 60 % opaque PANEL over the canvas. We can't
    // do true alpha in a u32-RGB framebuffer cheaply, so we paint
    // a solid color one step toward the canvas. (Refine to true
    // alpha after the rasterizer fix in Wave 5.)
    gpu::fill_rect(0, 0, screen_w, TOPBAR_H, PANEL);
    gpu::fill_rect(0, TOPBAR_H - 1, screen_w, 1, HAIRLINE);

    // Brand wordmark — left.
    let brand_x: u32 = 12;
    let brand_y: u32 = (TOPBAR_H - 16) / 2 + 4;
    font::draw_str(fb, screen_w, brand_x, brand_y, "SPHRAGIS", INK, PANEL);

    // Status strip — right, painted right-to-left so the badges'
    // widths don't need to be measured ahead.
    let mut cursor: i32 = screen_w as i32 - 12;

    // ⏻ glyph (always present, far right).
    let lock_label = "⏻";
    let lock_w = utf8_width_px(lock_label);
    cursor -= lock_w as i32;
    draw_label(fb, screen_w, cursor as u32, brand_y, lock_label, MID);
    cursor -= 12;

    // ⋯ glyph (always present, just left of lock).
    let gear_label = "⋯";
    let gear_w = utf8_width_px(gear_label);
    cursor -= gear_w as i32;
    draw_label(fb, screen_w, cursor as u32, brand_y, gear_label, FAINT);
    cursor -= 14;

    // User badges, right-to-left.
    for badge in badges.iter().rev() {
        let (label, warn) = badge_text(*badge);
        let lw = label.len() as u32 * 8;
        cursor -= lw as i32;
        let color = if warn { INK } else { MID };
        font::draw_str(fb, screen_w, cursor as u32, brand_y, &label, color, PANEL);
        cursor -= 12;
    }
}

/// Hit-test a click against the top bar. Returns the hit target or
/// `None` if the click was elsewhere in the top bar.
pub fn hit_test(x: u32, y: u32, screen_w: u32) -> TopbarHit {
    if y >= TOPBAR_H { return TopbarHit::None; }
    // Brand: 8 chars × 8 px + a few px padding either side.
    if x >= 8 && x < 8 + 8 * 8 + 8 { return TopbarHit::Brand; }
    // Lock: rightmost ~12 px.
    if x >= screen_w - 20 && x < screen_w - 8 { return TopbarHit::Lock; }
    // Gear: just left of lock, ~14 px window.
    if x >= screen_w - 38 && x < screen_w - 24 { return TopbarHit::Gear; }
    TopbarHit::None
}

/// Bitmap-font width of an ASCII or single-glyph label in pixels.
fn utf8_width_px(s: &str) -> u32 {
    // The bitmap font is 8 px monospace; non-ASCII glyphs are
    // currently drawn as `?` so their width is also 8.
    s.chars().count() as u32 * 8
}

fn draw_label(fb: *mut u32, screen_w: u32, x: u32, y: u32, label: &str, color: u32) {
    font::draw_str(fb, screen_w, x, y, label, color, PANEL);
}

/// Resolve a badge to its current text + whether it should highlight
/// as a warning (INK color).
fn badge_text(b: BadgeId) -> (heapless::String<32>, bool) {
    use core::fmt::Write;
    let mut s: heapless::String<32> = heapless::String::new();
    match b {
        BadgeId::NetMode => {
            // Wave 2 doesn't yet wire to a live net-mode source;
            // hardcode ISOLATED. Net module exposes
            // `crate::net::is_isolated() -> bool` once the wiring
            // lands (Wave 3+).
            let _ = write!(s, "NET ISOLATED");
            (s, false)
        }
        BadgeId::Deadman => {
            // `crate::security::deadman::remaining_ms() -> u64`
            // exists (boot_screen calls deadman::refresh on grant).
            // Wave 2 stubs the read as a fixed string to keep the
            // visual landing; live read added with the rest of the
            // dynamic-badge work in Task 8.
            let _ = write!(s, "DEADMAN 47:12");
            (s, false)
        }
        BadgeId::Clock => {
            let _ = write!(s, "14:22");
            (s, false)
        }
        BadgeId::CavesCount => {
            let _ = write!(s, "0 CAVES");
            (s, false)
        }
        BadgeId::Attempts => {
            let _ = write!(s, "5 ATTEMPTS");
            (s, false)
        }
    }
}
```

**NOTE:** This depends on the `heapless` crate. If `Cargo.toml` doesn't already include it, add it:

- [ ] **Step 2: Add `heapless` to Cargo.toml dependencies.**

Open `Cargo.toml` and add to `[dependencies]`:

```toml
heapless = { version = "0.8", default-features = false }
```

If `heapless` is already present at any version, skip this step.

- [ ] **Step 3: Wire the module in.**

In `src/ui/mod.rs`, add `pub mod topbar;` next to the other module declarations.

- [ ] **Step 4: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/topbar.rs src/ui/mod.rs Cargo.toml Cargo.lock
git commit -m "$(cat <<'EOF'
ui: add topbar module — paint + hit-test

22-px top strip: SPHRAGIS wordmark on the left, status badges on
the right ending with ⋯ (config trigger) and ⏻ (lock). Wave 2
ships the default badge set (NET MODE, DEADMAN, CLOCK) with
hardcoded values; Task 8 wires real values + BatFS-persisted
customization.

Adds heapless dep for no-alloc small-string formatting.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 4: Launcher overlay — paint + click

New file. Paints the 8-app grid: in LAUNCHER state at full opacity (interactive), in ACTIVE state at 22 % opacity (decorative, click-through), in OVERLAY state at full opacity over open windows.

**Files:**
- Create: `src/ui/launcher.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/launcher.rs`.**

```rust
//! 8-app launcher grid.
//!
//! Renders the 4×2 grid of apps in any of three modes:
//!   * Full opacity, interactive (LAUNCHER state — no windows open).
//!   * Dimmed to ~22 % opacity, non-interactive (ACTIVE state —
//!     visible behind windows but clicks fall through).
//!   * Full opacity, interactive, overlaid on top of windows
//!     (OVERLAY state — summoned by ⌘K / brand click).

#![allow(dead_code)]

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::apps_registry::{AppId, paint_placeholder_icon};
use crate::ui::topbar::TOPBAR_H;

const INK_FULL: u32 = 0xFFE5E7EB;
const INK_DIM:  u32 = 0xFF1F2024; // ~22% INK on BG — derived constant
const MID_FULL: u32 = 0xFF6B7280;
const MID_DIM:  u32 = 0xFF18191B; // ~22% MID on BG

/// Painted modes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LauncherMode {
    /// Full opacity, interactive. The whole grid is clickable.
    Interactive,
    /// Dim, decorative behind windows.
    Dimmed,
}

/// Paint the launcher grid centered on the body region (below the
/// top bar).
pub fn paint(fb: *mut u32, screen_w: u32, screen_h: u32, mode: LauncherMode) {
    // Grid geometry — matches the mockup proportions:
    //   horizontal padding = 12 % of width
    //   top padding = 26 % of (screen_h - TOPBAR_H)
    //   bottom padding = 14 %
    //   4 columns × 2 rows, 14 px gutter
    let body_top = TOPBAR_H;
    let body_h = screen_h - TOPBAR_H;

    let pad_x = screen_w * 12 / 100;
    let grid_top = body_top + body_h * 26 / 100;
    let grid_h = body_h * 60 / 100;

    let col_w = (screen_w - 2 * pad_x - 3 * 14) / 4;
    let row_h = (grid_h - 14) / 2;

    let (icon_color, label_color) = match mode {
        LauncherMode::Interactive => (MID_FULL, MID_FULL),
        LauncherMode::Dimmed       => (MID_DIM,  MID_DIM),
    };

    for (i, app) in AppId::all().iter().enumerate() {
        let col = (i % 4) as u32;
        let row = (i / 4) as u32;
        let cell_x = pad_x + col * (col_w + 14);
        let cell_y = grid_top + row * (row_h + 14);

        // Center the 22-px icon + label inside the cell.
        let icon_x = cell_x + (col_w - 22) / 2;
        let icon_y = cell_y + (row_h / 2) - 22;
        paint_placeholder_icon(*app, icon_x, icon_y, icon_color);

        // Label below the icon. 8-px font.
        let label = app.name();
        let label_w = label.len() as u32 * 8;
        let label_x = cell_x + (col_w.saturating_sub(label_w)) / 2;
        let label_y = icon_y + 22 + 8;
        font::draw_str(fb, screen_w, label_x, label_y, label, label_color, 0xFF0D0D10);
    }
}

/// Hit-test a click against the launcher grid. Returns the AppId of
/// the tile the user hit, or None.
pub fn hit_test(x: u32, y: u32, screen_w: u32, screen_h: u32) -> Option<AppId> {
    let body_top = TOPBAR_H;
    let body_h = screen_h - TOPBAR_H;
    let pad_x = screen_w * 12 / 100;
    let grid_top = body_top + body_h * 26 / 100;
    let grid_h = body_h * 60 / 100;
    let col_w = (screen_w - 2 * pad_x - 3 * 14) / 4;
    let row_h = (grid_h - 14) / 2;

    if y < grid_top || y >= grid_top + 2 * (row_h + 14) { return None; }
    if x < pad_x || x >= screen_w - pad_x { return None; }

    let col = (x - pad_x) / (col_w + 14);
    let row = (y - grid_top) / (row_h + 14);
    if col >= 4 || row >= 2 { return None; }
    let idx = (row * 4 + col) as usize;
    if idx >= 8 { return None; }
    Some(AppId::all()[idx])
}
```

- [ ] **Step 2: Wire the module in.**

In `src/ui/mod.rs`, add `pub mod launcher;`.

- [ ] **Step 3: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/launcher.rs src/ui/mod.rs
git commit -m "$(cat <<'EOF'
ui: add launcher module — 4×2 app grid paint + hit-test

Two render modes: Interactive (full opacity, clicks open the app)
and Dimmed (~22% opacity, decorative behind open windows). Hit-test
maps a (x, y) click to the AppId of the tile under the cursor.

Uses apps_registry::paint_placeholder_icon — Wave 2 placeholder
22×22 squares. Wave 3+ replaces per-app.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 5: Watermark Σ paint helper

Reuses the Wave 1 baked Σ bitmap. Centered behind everything, large, low alpha. Small wrapper because every desktop repaint needs this.

**Files:**
- Modify: `src/ui/draw.rs` (add `paint_centered_watermark`)

- [ ] **Step 1: Add the helper to `src/ui/draw.rs`.**

Append this function near the bottom of the file (after `blit_alpha_bitmap`):

```rust
/// Paint the Σ watermark centered on screen, at a large size with a
/// soft-on-near-black tint. Used by the desktop canvas behind every-
/// thing else. Reuses Wave 1's baked Σ bitmap, scaled by simple
/// nearest-neighbor blow-up — fine for a watermark.
pub fn paint_sigma_watermark(fb: *mut u32, screen_w: u32, screen_h: u32) {
    use crate::ui::sigma_bitmap::{SIGMA_BITMAP_96, SIGMA_BITMAP_W, SIGMA_BITMAP_H};
    // Scale: 4× of the 96×96 source = 384×384, sized for desktop.
    const SCALE: u32 = 4;
    let target_w = SIGMA_BITMAP_W * SCALE;
    let target_h = SIGMA_BITMAP_H * SCALE;
    let origin_x = ((screen_w as i32) - target_w as i32) / 2;
    let origin_y = ((screen_h as i32) - target_h as i32) / 2;
    // Soft mark tint — slightly above BG, well below INK.
    const WATERMARK: u32 = 0xFF1C1D22;
    for row in 0..target_h as i32 {
        for col in 0..target_w as i32 {
            let src_row = (row as u32 / SCALE) as usize;
            let src_col = (col as u32 / SCALE) as usize;
            let cov = SIGMA_BITMAP_96[src_row * SIGMA_BITMAP_W as usize + src_col] as u32;
            if cov == 0 { continue; }
            let sx = origin_x + col;
            let sy = origin_y + row;
            if sx < 0 || sx >= screen_w as i32 { continue; }
            if sy < 0 || sy >= screen_h as i32 { continue; }
            let fb_idx = (sy as u32 * screen_w + sx as u32) as usize;
            unsafe {
                let dst = core::ptr::read_volatile(fb.add(fb_idx));
                let dr = (dst >> 16) & 0xFF;
                let dg = (dst >> 8) & 0xFF;
                let db = dst & 0xFF;
                let wr = (WATERMARK >> 16) & 0xFF;
                let wg = (WATERMARK >> 8) & 0xFF;
                let wb = WATERMARK & 0xFF;
                // Quarter-strength blend toward the watermark color
                // even when coverage is full — keeps the Σ subtle.
                let alpha = cov / 4;
                let r = (dr + (((wr - dr) * alpha) / 255)).min(255);
                let g = (dg + (((wg - dg) * alpha) / 255)).min(255);
                let b = (db + (((wb - db) * alpha) / 255)).min(255);
                core::ptr::write_volatile(
                    fb.add(fb_idx),
                    0xFF000000 | (r << 16) | (g << 8) | b,
                );
            }
        }
    }
}
```

- [ ] **Step 2: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/draw.rs
git commit -m "$(cat <<'EOF'
draw: add paint_sigma_watermark — desktop background mark

4× scaled nearest-neighbor blow-up of the Wave 1 baked Σ bitmap,
quarter-strength alpha blend over the canvas. Subtle behind-
everything brand mark for the new desktop.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 6: Desktop state machine + event loop

Rewrite `src/ui/desktop.rs` end-to-end. Replaces the Ctrl+1-5 single-app switcher with the LAUNCHER / ACTIVE / OVERLAY state machine. Returns to caller on lock (`⌘L` or ⏻ click) so `main.rs` can re-enter the lock screen.

**Files:**
- Modify: `src/ui/desktop.rs`

- [ ] **Step 1: Replace `src/ui/desktop.rs` with the new state-machine version.**

```rust
//! Desktop environment — Wave 2.
//!
//! State machine:
//!
//!     LAUNCHER ──click app──> ACTIVE ──close last window──> LAUNCHER
//!                                │
//!                                └── ⌘K / brand click ──> OVERLAY
//!                                                            │
//!                                                  click app │ click outside / Esc
//!                                                            ▼
//!                                                        ACTIVE
//!
//! `run()` returns `LockReason` to its caller (main.rs) when the
//! user presses ⌘L or clicks ⏻. main.rs is responsible for the
//! re-enter cycle.
//!
//! WM state persists across the run/exit cycle because it lives in
//! `wm.rs` module statics — locking doesn't reset windows.

use crate::platform;
use crate::ui::{wm, topbar, launcher, draw, gpu};
use crate::ui::apps_registry::AppId;
use crate::ui::topbar::{TOPBAR_H, BadgeId, DEFAULT_BADGES, TopbarHit};
use crate::ui::launcher::LauncherMode;

/// Why `run()` returned. Currently only "user requested lock"; future
/// expansions (panic, system shutdown) get new variants.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LockReason {
    UserLocked,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum State {
    Launcher,
    Active,
    Overlay,
}

const BG: u32 = 0xFF0D0D10;

/// Modifier-key state. Updated as the input loop sees Ctrl/⌘ events.
struct Mods {
    cmd: bool,
}

/// Drag state — None when no drag in progress; otherwise the kind
/// + the slot being dragged.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Drag {
    None,
    Move(usize, i32, i32), // slot, last-x, last-y
    Resize(usize),          // slot — corner drag
}

/// Run the desktop event loop until the user locks. Returns the
/// reason so main.rs can decide what to do next.
pub fn run() -> LockReason {
    let mut state = if wm::any_open() { State::Active } else { State::Launcher };
    let mut mods = Mods { cmd: false };
    let mut drag = Drag::None;
    let mut pointer_x: i32 = 0;
    let mut pointer_y: i32 = 0;
    let mut need_repaint = true;

    loop {
        // ── repaint ──────────────────────────────────────────────
        if need_repaint || wm::take_redraw() {
            paint_full(state);
            need_repaint = false;
        }

        // ── drain input ──────────────────────────────────────────
        crate::drivers::virtio::keyboard::poll();
        crate::drivers::virtio::tablet::poll();

        // Keyboard
        let key = platform::serial_getc()
            .or_else(crate::drivers::virtio::keyboard::getc)
            .or_else(crate::drivers::virtio::tablet::getc_key);
        if let Some(c) = key {
            if let Some(reason) = handle_key(c, &mut state, &mods) {
                return reason;
            }
            need_repaint = true;
        }

        // Pointer events from the tablet driver. Wave 2 reads a simple
        // (x, y, button) snapshot each tick; the tablet module
        // already maintains this state.
        if let Some((px, py, btn)) =
            crate::drivers::virtio::tablet::pointer_state()
        {
            pointer_x = px;
            pointer_y = py;
            if handle_pointer(px, py, btn, &mut state, &mut drag) {
                need_repaint = true;
            }
        }

        // Cooperative yield so other kernel work runs.
        core::hint::spin_loop();

        // Suppress the unused-mut warning for `pointer_x/y` — these
        // are intentionally tracked for future hover-state work but
        // not yet read.
        let _ = (pointer_x, pointer_y);
    }
}

/// Paint a full frame for the given state.
fn paint_full(state: State) {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();

    // 1. canvas BG.
    gpu::fill_screen(BG);

    // 2. watermark Σ.
    draw::paint_sigma_watermark(fb, w, h);

    // 3. launcher grid — dimmed if ACTIVE, full opacity otherwise.
    let mode = match state {
        State::Launcher => LauncherMode::Interactive,
        State::Active   => LauncherMode::Dimmed,
        State::Overlay  => LauncherMode::Interactive,
    };
    launcher::paint(fb, w, h, mode);

    // 4. all windows (back-to-front). Skipped in Launcher state
    //    (there are none) and behind the overlay in Overlay state.
    if state == State::Active || state == State::Overlay {
        wm::paint_all(fb, w, h);
    }

    // 5. overlay grid (if Overlay) — on top of windows.
    if state == State::Overlay {
        launcher::paint(fb, w, h, LauncherMode::Interactive);
    }

    // 6. top bar on top.
    topbar::paint(fb, w, DEFAULT_BADGES);

    gpu::flush(0, 0, w, h);
}

/// Handle a single keyboard event. Returns `Some(LockReason)` if the
/// user pressed ⌘L (or otherwise requested a lock).
fn handle_key(c: u8, state: &mut State, mods: &Mods) -> Option<LockReason> {
    // The kernel's keyboard pipe doesn't currently expose modifier
    // chords as separate events; we treat plain ASCII for the
    // bindings the keyboard driver delivers when ⌘ + letter is
    // pressed: ⌘K=0x0B, ⌘L=0x0C, ⌘W=0x17. These are
    // the ctrl-modified ASCII codes virtio-keyboard delivers when
    // Ctrl+K/L/W is pressed.
    match c {
        0x0B => { // ⌘K — toggle launcher overlay
            *state = match *state {
                State::Active   => State::Overlay,
                State::Overlay  => State::Active,
                State::Launcher => State::Launcher,
            };
            None
        }
        0x0C => Some(LockReason::UserLocked), // ⌘L
        0x17 => { // ⌘W — close focused window
            if let Some(slot) = wm::current_focus() {
                wm::close(slot);
                if !wm::any_open() { *state = State::Launcher; }
            }
            None
        }
        0x1B => { // Esc — dismiss overlay
            if *state == State::Overlay { *state = State::Active; }
            None
        }
        0x09 => { // Tab — cycle focus (treated as ⌘TAB for simplicity)
            wm::cycle_focus();
            None
        }
        _ => {
            // Plain text goes to the focused window's app. Wave 2's
            // apps don't yet have an input hook through the WM, so
            // we route keystrokes to the legacy single-app path:
            // shell. Future waves wire each app properly.
            if let Some(slot) = wm::current_focus() {
                if let Some(win) = wm::get(slot) {
                    route_keystroke_to_app(win.app, c);
                }
            }
            let _ = mods;
            None
        }
    }
}

/// Forward a keystroke to a specific app. Wave 2 stub — each app
/// gets its own real handler in its own redesign wave.
fn route_keystroke_to_app(app: AppId, c: u8) {
    if app == AppId::Shell {
        // Existing shell input pipe.
        let _ = c;
        // crate::ui::shell::feed_byte(c); — to wire once shell is
        // refactored to expose a per-byte hook. Wave 2 stub.
    }
}

/// Handle a pointer state snapshot. Returns true if the frame needs
/// repaint.
fn handle_pointer(
    px: i32,
    py: i32,
    pressed: bool,
    state: &mut State,
    drag: &mut Drag,
) -> bool {
    let w = gpu::width();
    let h = gpu::height();

    // Released? End any drag.
    if !pressed {
        if *drag != Drag::None { *drag = Drag::None; return true; }
        return false;
    }

    // Pressed — figure out what was hit, in priority order: top bar
    // first, then overlay/launcher grid (when interactive), then
    // window chrome/close/resize/body.

    // ── Top bar ──────────────────────────────────────────────────
    if py >= 0 && (py as u32) < TOPBAR_H {
        match topbar::hit_test(px as u32, py as u32, w) {
            TopbarHit::Brand => {
                *state = match *state {
                    State::Launcher => State::Launcher,
                    State::Active   => State::Overlay,
                    State::Overlay  => State::Active,
                };
                return true;
            }
            TopbarHit::Lock => {
                // Treat like ⌘L — handled by the run loop reading
                // this as a LockReason. For simplicity, set a
                // module-level "wants lock" flag here.
                LOCK_REQUESTED.store(true, core::sync::atomic::Ordering::Relaxed);
                return true;
            }
            TopbarHit::Gear => {
                // Task 8 wires the config sheet here; Wave 2 ships
                // with no-op gear so the click visibly does nothing.
                return false;
            }
            TopbarHit::None => return false,
        }
    }

    // ── Overlay or Launcher (interactive) ────────────────────────
    if matches!(*state, State::Launcher | State::Overlay) {
        if let Some(app) = launcher::hit_test(px as u32, py as u32, w, h) {
            // Open the app (no cave context from the launcher;
            // caves manager will use a different open path).
            let _ = wm::open(app, None);
            *state = State::Active;
            return true;
        }
        // In OVERLAY, click-outside dismisses.
        if *state == State::Overlay {
            *state = State::Active;
            return true;
        }
        return false;
    }

    // ── Window interactions (state == Active) ─────────────────────
    // Resize corner first (priority over chrome / body).
    if let Some(slot) = wm::focused_resize_hit(px, py) {
        *drag = Drag::Resize(slot);
        return true;
    }
    // Close button.
    if let Some(slot) = wm::focused_close_hit(px, py) {
        wm::close(slot);
        if !wm::any_open() { *state = State::Launcher; }
        return true;
    }
    // Chrome → start a move drag.
    if let Some(slot) = wm::focused_chrome_hit(px, py) {
        *drag = Drag::Move(slot, px, py);
        return true;
    }
    // Body → focus the window under the cursor.
    if let Some(slot) = wm::slot_at(px, py) {
        wm::focus(slot);
        return true;
    }

    false
}

/// Lock-request flag for the ⏻ button (which can't directly return
/// from `handle_pointer`).
static LOCK_REQUESTED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
pub fn lock_requested() -> bool {
    LOCK_REQUESTED.swap(false, core::sync::atomic::Ordering::Relaxed)
}

/// Legacy entry — kept compiling for now. The old caller in main.rs
/// uses `run() -> !`; Task 7 changes the call site to use the new
/// `run() -> LockReason`. Until then, `resume()` and the old
/// `!`-returning fallback live in the call-site comment, not here.
#[allow(dead_code)]
pub fn resume() -> LockReason {
    run()
}
```

**NOTE:** This task assumes `crate::drivers::virtio::tablet::pointer_state()` exists and returns `Option<(i32, i32, bool)>` for (x, y, button_pressed). If it doesn't, see the next step.

- [ ] **Step 2: Confirm `tablet::pointer_state()` exists, or add a thin shim.**

```bash
cd /Users/kadenlee/Sphragis
grep -n 'pub fn pointer_state\|pub fn cursor\|pub fn xy\|pub fn last_position\|pub fn position\|pub fn read_pointer' src/drivers/virtio/tablet.rs 2>/dev/null
```

If a function returning the cursor state already exists with a different name, adjust the call in `handle_pointer`. If none exists, add this near the top of `src/drivers/virtio/tablet.rs`:

```rust
/// Snapshot the current pointer state (x, y, button_pressed) or None
/// if no pointer has reported yet. Wave 2's WM polls this every tick.
pub fn pointer_state() -> Option<(i32, i32, bool)> {
    // The tablet driver already tracks cursor position internally.
    // If it doesn't expose getters yet, replace the bodies below
    // with reads of the appropriate private statics.
    None  // placeholder — replace with the real read
}
```

If `pointer_state` had to be a placeholder, that's a known limitation: pointer-driven drag/resize won't work in QEMU until the tablet driver is properly wired. Keyboard shortcuts still work. Note this in the commit.

- [ ] **Step 3: Update `src/main.rs` for the new return type.**

Find the call site:

```bash
grep -n 'ui::desktop::run' /Users/kadenlee/Sphragis/src/main.rs
```

The current line (around 453) reads `ui::desktop::run();` and assumes `-> !`. Replace it with the lock-loop:

```rust
        // Lock cycle: alternate between the lock screen and the
        // desktop. WM state persists across cycles because wm.rs
        // holds it in module statics.
        loop {
            security::boot_screen::run();
            match ui::desktop::run() {
                ui::desktop::LockReason::UserLocked => continue,
            }
        }
```

Replace whatever code follows the existing `boot_screen::run()` + `desktop::run()` calls so the loop wraps both.

- [ ] **Step 4: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -10
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -5
```

Expected: clean. Likely fixes needed: callers of the old `desktop::resume()` returning `!`, callers of removed pane-API helpers. Resolve each by matching the new return-type signature.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/desktop.rs src/main.rs src/drivers/virtio/tablet.rs
git commit -m "$(cat <<'EOF'
desktop: state machine + lock-cycle loop

Rewrites the desktop event loop end-to-end against the Wave 2 spec.
LAUNCHER / ACTIVE / OVERLAY states; ⌘K toggles overlay; ⌘L locks;
⌘W closes; ⌘TAB cycles. Brand-click and ⏻ click also trigger the
state transitions. Pointer events route to chrome drag, corner
resize, close button, or window-body focus.

main.rs now wraps boot_screen + desktop in a re-entrant loop so
locking returns to the lock screen and re-unlocking resumes the
same workspace (WM state is module-static and survives the cycle).

Per-window app paint + per-window keystroke routing are
stubbed — each app's redesign wave wires its own paint + input
hook. The chrome itself draws.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 7: App-body paint pass — call each app's paint into its window region

The WM owns chrome; apps own body. Wave 2 connects them. Each existing app has its own paint function (already living in `src/ui/apps/*`); we add an `AppId::dispatch_paint(rect)` thin wrapper that calls the right one.

**Files:**
- Modify: `src/ui/apps_registry.rs`
- Modify: `src/ui/wm.rs`

- [ ] **Step 1: Audit each app's existing paint function.**

```bash
cd /Users/kadenlee/Sphragis
for f in src/ui/apps/*.rs; do
    name=$(basename "$f" .rs)
    [ "$name" = "mod" ] && continue
    echo "== $name =="
    grep -nE '^pub fn (paint|render|draw)' "$f" | head -3
done
```

Expected: each app exposes some kind of `render` or `paint` function. Note the function names — you'll dispatch to them in the next step.

- [ ] **Step 2: Add `paint_body` to `AppId` in apps_registry.rs.**

Append to the `impl AppId` block:

```rust
    /// Dispatch to the app's body-paint function. Each app's paint
    /// runs inside the window-body rect (chrome already drawn by
    /// the WM). Apps that don't yet have a Wave 2-compatible paint
    /// callback fall back to a placeholder.
    pub fn paint_body(self, body_x: u32, body_y: u32, body_w: u32, body_h: u32) {
        // Set the GPU clip to the body region so the app can't paint
        // outside its window (apps were written assuming full-screen
        // access; clipping is the simplest way to constrain them
        // without rewriting each one).
        crate::ui::font::set_clip(body_x, body_y, body_w, body_h);
        match self {
            AppId::Caves    => crate::ui::apps::caves_mgr::render(),
            AppId::Files    => crate::ui::apps::filemanager::render(),
            AppId::Net      => crate::ui::apps::netmon::render(),
            AppId::Security => crate::ui::apps::security::render(),
            AppId::Editor   => crate::ui::apps::editor::render(),
            AppId::Comms    => crate::ui::apps::comms::render(),
            AppId::Shell    => {
                // Shell paint lives in src/ui/shell.rs; Wave 3 wires
                // it cleanly. Wave 2 leaves the body blank with a
                // small placeholder string.
                paint_app_stub(body_x, body_y, "shell — wave 3 wire-up");
            }
            AppId::Agent    => {
                paint_app_stub(body_x, body_y, "agent — design pending");
            }
        }
        crate::ui::font::clear_clip();
    }
}

fn paint_app_stub(body_x: u32, body_y: u32, msg: &str) {
    // Use the bitmap font's draw_str on the framebuffer.
    let w = crate::ui::gpu::width();
    let fb = crate::ui::gpu::framebuffer();
    crate::ui::font::draw_str(
        fb, w,
        body_x + 16, body_y + 16,
        msg, 0xFF6B7280, 0xFF0D0D10,
    );
}
```

**NOTE:** This step assumes each app exposes a no-arg `render()` function. If the real names from Step 1 differ, update each match arm accordingly. If any app has a different signature (e.g., takes a `WindowRect`), adapt.

- [ ] **Step 3: Call `paint_body` from the WM's paint_window.**

In `src/ui/wm.rs`, find `paint_window` (currently ends after rendering the title text). Append the body-paint call just before the closing `}`:

```rust
    // Body paint — call the app's paint, clipped to the body region.
    let body_x = ux;
    let body_y = uy + CHROME_H;
    let body_w = win.w;
    let body_h = win.h.saturating_sub(CHROME_H);
    win.app.paint_body(body_x, body_y, body_w, body_h);
```

- [ ] **Step 4: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean. If any app's render function has a different signature, the build fails with a clear mismatch error — adapt the match arm.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/apps_registry.rs src/ui/wm.rs
git commit -m "$(cat <<'EOF'
ui: dispatch each app's body paint into its window region

Adds AppId::paint_body which the WM calls for each open window.
Apps with pre-existing render() functions get clip-set, render
called, then clip-cleared so cyberpunk-styled apps can't paint
outside their window box. Shell and Agent get placeholder
strings — they'll wire their real paint in Wave 3 and beyond.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 8: Top-bar customization sheet + BatFS persistence

Layer the config sheet on top of the static default-badges code. Open via `⋯` click; lists every BadgeId with toggle + reorder; saves to `/system/desktop/topbar.cfg` in BatFS.

**Files:**
- Modify: `src/ui/topbar.rs`
- Modify: `src/ui/desktop.rs`

- [ ] **Step 1: Add the config-sheet UI to `src/ui/topbar.rs`.**

Append to topbar.rs:

```rust
// ─── Config sheet ─────────────────────────────────────────────────
//
// A small modal overlay that opens when the user clicks ⋯. Lists
// every BadgeId with an on/off toggle. The user enables/disables
// + drags to reorder; the desktop event loop owns the modal-mode
// state and forwards clicks here.

const CONFIG_PATH: &str = "/system/desktop/topbar.cfg";

/// All badges in canonical order. The user's active list is a subset
/// of these, persisted to BatFS as a byte sequence of indices.
pub const ALL_BADGES: &[BadgeId] = &[
    BadgeId::NetMode,
    BadgeId::Deadman,
    BadgeId::Clock,
    BadgeId::CavesCount,
    BadgeId::Attempts,
];

impl BadgeId {
    pub const fn as_byte(self) -> u8 {
        match self {
            BadgeId::NetMode    => 0,
            BadgeId::Deadman    => 1,
            BadgeId::Clock      => 2,
            BadgeId::CavesCount => 3,
            BadgeId::Attempts   => 4,
        }
    }

    pub const fn from_byte(b: u8) -> Option<BadgeId> {
        match b {
            0 => Some(BadgeId::NetMode),
            1 => Some(BadgeId::Deadman),
            2 => Some(BadgeId::Clock),
            3 => Some(BadgeId::CavesCount),
            4 => Some(BadgeId::Attempts),
            _ => None,
        }
    }
}

/// Active-badge list, persisted to BatFS. Up to 8 active badges.
static mut ACTIVE: [Option<BadgeId>; 8] = [
    Some(BadgeId::NetMode),
    Some(BadgeId::Deadman),
    Some(BadgeId::Clock),
    None, None, None, None, None,
];

/// Read the active badge list. Used by paint().
pub fn active_badges() -> [Option<BadgeId>; 8] {
    unsafe { ACTIVE }
}

/// Replace the active list and persist to BatFS.
pub fn set_active(new_active: [Option<BadgeId>; 8]) {
    unsafe { ACTIVE = new_active; }
    persist_to_disk(&new_active);
}

fn persist_to_disk(active: &[Option<BadgeId>; 8]) {
    let mut bytes = [0u8; 8];
    let mut n = 0usize;
    for slot in active.iter() {
        if let Some(b) = slot {
            bytes[n] = b.as_byte();
            n += 1;
        }
    }
    // BatFS doesn't expose update — delete-then-create.
    let _ = crate::fs::batfs::delete(CONFIG_PATH);
    let _ = crate::fs::batfs::create(CONFIG_PATH, &bytes[..n]);
}

/// Load the active list from BatFS. Called once at boot. Falls back
/// to DEFAULT_BADGES if the file doesn't exist or is malformed.
pub fn load_from_disk() {
    let mut buf = [0u8; 8];
    match crate::fs::batfs::read(CONFIG_PATH, &mut buf) {
        Ok(n) if n > 0 && n <= 8 => {
            let mut new_active: [Option<BadgeId>; 8] = [None; 8];
            for (i, &b) in buf[..n].iter().enumerate() {
                if let Some(badge) = BadgeId::from_byte(b) {
                    new_active[i] = Some(badge);
                }
            }
            unsafe { ACTIVE = new_active; }
        }
        _ => {
            // First boot or corrupt — leave defaults in place.
        }
    }
}

/// Whether the config sheet is currently visible. Toggled by clicks
/// on the ⋯ glyph (handled in desktop.rs); paint() consults this.
static SHEET_VISIBLE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

pub fn toggle_sheet() {
    SHEET_VISIBLE.fetch_xor(true, core::sync::atomic::Ordering::Relaxed);
}

pub fn sheet_visible() -> bool {
    SHEET_VISIBLE.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn close_sheet() {
    SHEET_VISIBLE.store(false, core::sync::atomic::Ordering::Relaxed);
}

/// Paint the config sheet, centered on the screen. Wave 2 ships the
/// minimal viable form: a vertical list of every BadgeId with the
/// active ones marked. Click a row to toggle.
pub fn paint_sheet(fb: *mut u32, screen_w: u32, screen_h: u32) {
    if !sheet_visible() { return; }
    let w: u32 = 280;
    let h: u32 = 200;
    let x = (screen_w - w) / 2;
    let y = (screen_h - h) / 2;
    gpu::fill_rect(x, y, w, h, PANEL);
    draw::draw_border(x, y, w, h, HAIRLINE);
    font::draw_str(fb, screen_w, x + 16, y + 16, "STATUS STRIP", INK, PANEL);
    let active = active_badges();
    let row_h: u32 = 22;
    for (i, badge) in ALL_BADGES.iter().enumerate() {
        let row_y = y + 44 + (i as u32) * row_h;
        let on = active.iter().flatten().any(|a| a == badge);
        let mark = if on { "[x]" } else { "[ ]" };
        font::draw_str(fb, screen_w, x + 16, row_y, mark, MID, PANEL);
        let label = badge_name(*badge);
        font::draw_str(fb, screen_w, x + 48, row_y, label, INK, PANEL);
    }
    font::draw_str(fb, screen_w, x + 16, y + h - 24, "CLICK ROW TO TOGGLE  ·  ESC TO CLOSE", FAINT, PANEL);
}

const fn badge_name(b: BadgeId) -> &'static str {
    match b {
        BadgeId::NetMode    => "NET MODE",
        BadgeId::Deadman    => "DEADMAN",
        BadgeId::Clock      => "CLOCK",
        BadgeId::CavesCount => "CAVES COUNT",
        BadgeId::Attempts   => "ATTEMPTS",
    }
}

/// Hit-test a click against the open sheet. Returns the BadgeId the
/// user clicked, or None.
pub fn sheet_hit_test(x: u32, y: u32, screen_w: u32, screen_h: u32) -> Option<BadgeId> {
    if !sheet_visible() { return None; }
    let w: u32 = 280;
    let h: u32 = 200;
    let sx = (screen_w - w) / 2;
    let sy = (screen_h - h) / 2;
    if x < sx || x >= sx + w || y < sy || y >= sy + h { return None; }
    let row_h: u32 = 22;
    let local_y = y - sy;
    if local_y < 44 { return None; }
    let row = (local_y - 44) / row_h;
    if row as usize >= ALL_BADGES.len() { return None; }
    Some(ALL_BADGES[row as usize])
}

/// Toggle a badge in the active list, then persist. Inserts at the
/// end if not present; removes if present.
pub fn toggle_badge(b: BadgeId) {
    let mut active = active_badges();
    // Already on? Remove.
    for slot in active.iter_mut() {
        if *slot == Some(b) {
            *slot = None;
            compact(&mut active);
            set_active(active);
            return;
        }
    }
    // Not on — append.
    for slot in active.iter_mut() {
        if slot.is_none() { *slot = Some(b); break; }
    }
    set_active(active);
}

fn compact(active: &mut [Option<BadgeId>; 8]) {
    let mut write = 0;
    for read in 0..8 {
        if active[read].is_some() {
            if write != read { active[write] = active[read]; active[read] = None; }
            write += 1;
        }
    }
}
```

- [ ] **Step 2: Update `topbar::paint` to use the live active list.**

Find the existing `pub fn paint(fb, screen_w, badges: &[BadgeId])` and change the signature + implementation to use `active_badges()` directly (so callers no longer have to thread the list through):

Replace the `pub fn paint(fb: *mut u32, screen_w: u32, badges: &[BadgeId])` signature with:

```rust
pub fn paint(fb: *mut u32, screen_w: u32) {
```

…and change the body's `for badge in badges.iter().rev()` to:

```rust
    let active = active_badges();
    for slot in active.iter().rev() {
        if let Some(badge) = slot {
            let (label, warn) = badge_text(*badge);
            let lw = label.len() as u32 * 8;
            cursor -= lw as i32;
            let color = if warn { INK } else { MID };
            font::draw_str(fb, screen_w, cursor as u32, brand_y, &label, color, PANEL);
            cursor -= 12;
        }
    }
```

- [ ] **Step 3: Update desktop.rs to use the new paint signature + sheet.**

In `src/ui/desktop.rs`:

- Change `topbar::paint(fb, w, DEFAULT_BADGES);` to `topbar::paint(fb, w);`.
- After the topbar paint in `paint_full`, add:
  ```rust
  // Config sheet — overlays on top of everything when open.
  topbar::paint_sheet(fb, w, h);
  ```
- In `handle_pointer`'s `TopbarHit::Gear` arm, change the no-op to:
  ```rust
  TopbarHit::Gear => {
      topbar::toggle_sheet();
      return true;
  }
  ```
- Add new dispatch BEFORE the top-bar hit_test block to handle clicks against the open sheet:
  ```rust
  // Config sheet — if visible, swallow all clicks. Clicks on a row
  // toggle the badge; clicks outside the sheet close it.
  if topbar::sheet_visible() {
      if let Some(badge) = topbar::sheet_hit_test(px as u32, py as u32, w, h) {
          topbar::toggle_badge(badge);
          return true;
      }
      // Click outside → close.
      topbar::close_sheet();
      return true;
  }
  ```
- In `handle_key`, after the existing `0x1B` (Esc) arm, allow Esc to close the sheet too:
  ```rust
  0x1B => {
      if topbar::sheet_visible() { topbar::close_sheet(); }
      else if *state == State::Overlay { *state = State::Active; }
      None
  }
  ```
- Call `topbar::load_from_disk()` once at the start of `run()`:
  ```rust
  pub fn run() -> LockReason {
      topbar::load_from_disk();
      // ... existing body
  }
  ```

- [ ] **Step 4: Build + clippy.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/topbar.rs src/ui/desktop.rs
git commit -m "$(cat <<'EOF'
topbar: customization sheet + BatFS-persisted active list

Click ⋯ opens a centered sheet with every available BadgeId as a
toggle. Click a row to add/remove. Active list persists to
/system/desktop/topbar.cfg in BatFS; loaded once at desktop::run()
entry.

paint() now reads the live active list (drops the `badges`
parameter); desktop.rs updated accordingly.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 9: QEMU walkthrough — visual verification

No code changes. Boot the kernel under QEMU and confirm every state + interaction works.

- [ ] **Step 1: Build + launch.**

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

Expected: lock screen appears (Wave 1 unchanged), unlock with `sphragis-dev`.

- [ ] **Step 2: LAUNCHER state.**

After unlock, confirm:
- Top bar shows "SPHRAGIS" on the left, "NET ISOLATED · DEADMAN 47:12 · 14:22 ⋯ ⏻" on the right.
- The 8-app grid is visible at full opacity (CAVES, FILES, NET, SECURITY, SHELL, EDITOR, COMMS, AGENT).
- Σ watermark is faintly visible behind everything.

- [ ] **Step 3: Open a window (ACTIVE state).**

- Click the CAVES tile.
- Confirm a floating window appears with `CAVES` in the chrome.
- Grid behind it should dim to ~22 %.
- App body shows whatever caves_mgr renders (cyberpunk styling — that's expected, redesign is Wave 3).

- [ ] **Step 4: Open a second window.**

- Press `⌘K` (or click "SPHRAGIS") to open the launcher overlay over the open window.
- Click NET. A second window appears, offset down-right from the first.
- Press `⌘TAB` — focus should cycle.

- [ ] **Step 5: Drag, resize, close.**

- Drag the CAVES window's chrome — it should follow the cursor.
- Drag the bottom-right corner — it should resize, capped at the minimum.
- Click the close circle on each window — they should close.
- After closing the last, screen returns to LAUNCHER state automatically.

- [ ] **Step 6: Config sheet.**

- Click the `⋯` in the top-right.
- Sheet appears with all 5 badges; the 3 default ones marked `[x]`.
- Click `CAVES COUNT` — it should mark `[x]` and `0 CAVES` should appear in the top-right after closing.
- Click outside the sheet or press Esc — it closes.

- [ ] **Step 7: Lock + unlock.**

- Press `⌘L` (or click the `⏻` glyph in the top-right).
- Lock screen returns.
- Unlock with `sphragis-dev`.
- Desktop reappears with the same workspace state (open windows in same positions; topbar badges preserved).

- [ ] **Step 8: Cleanup.**

```bash
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
```

- [ ] **Step 9: No commit. Verification only.**

---

### Task 10: Push the branch and finish

- [ ] **Step 1: Push.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/desktop-chrome
```

- [ ] **Step 2: Use the `superpowers:finishing-a-development-branch` skill** to verify (build/clippy clean) and merge back to main.

---

## Spec coverage map (self-review)

| Spec section | Implemented by |
|--------------|---------------|
| Goal — quiet canvas + multi-window | Task 6 (state machine) + Task 5 (watermark) |
| Mental model — A+C hybrid | Task 4 + Task 6 |
| Visual palette | Task 2 (WM constants) + Task 3 (topbar constants) — single source of truth deferred |
| Typography — bitmap font in top bar / chrome | Tasks 2, 3, 4, 8 (all use `font::draw_str`) |
| Top bar — brand left, customizable right | Task 3 (paint) + Task 8 (customization) |
| Default badges (NET MODE / DEADMAN / CLOCK) | Task 3 (DEFAULT_BADGES) + Task 8 (ACTIVE default) |
| App grid — 4×2, 8 apps | Task 4 |
| Solid silhouette placeholder icons | Task 1 (`paint_placeholder_icon`) |
| Window struct (position, size, cave_name) | Task 2 |
| Floating, drag-move, drag-resize, click-to-focus | Task 2 + Task 6 (`handle_pointer`) |
| Multi-cave reading (`SHELL · cave_name`) | Task 2 (chrome paint includes cave_name) |
| Drop shadow under windows | Not yet (no_std friendly drop-shadow is non-trivial); deferred — captured below |
| State machine LAUNCHER / ACTIVE / OVERLAY | Task 6 |
| ⌘K, ⌘TAB, ⌘L, ⌘W, Esc | Task 6 (`handle_key`) |
| Click SPHRAGIS = launcher | Task 6 (`TopbarHit::Brand`) |
| ⏻ click = lock | Task 6 + Task 8 |
| Lock cycle persists windows | Task 6 (`main.rs` re-entrant loop, WM state static) |
| Customizable badges + BatFS persistence | Task 8 |
| Apps untouched | Whole plan — only `apps_registry.rs` references each app's `render()` |

### Known gaps from the spec

- **Drop shadow under floating windows** is in the spec but not implemented in this plan. Drawing a true soft shadow on a no_std framebuffer is non-trivial (no alpha-compositing primitives); the WM ships windows with the 1-px hairline border only. Filed as a Wave-2-followup or rolled into Wave 5 alongside the rasterizer fix.
- **Live badge values** (real net mode, real deadman, real clock) are stubbed with fixed strings in Task 3. The badges' wiring to live sources is a one-step follow-up: replace the stubs in `badge_text` with reads from `crate::net::is_isolated`, `crate::security::deadman::remaining_ms`, and `cntpct_el0`. Not blocking the Wave 2 ship.
- **`⌘TAB` only cycles to the next window in z-order**, not "back to where you were last" — simple model; refine if user feedback says so.

## Out-of-scope reminders

- Per-app internal redesign (Wave 3+).
- Real per-app icons (Wave 3+).
- Shell + console palette refresh (Wave 5).
- Virtual desktops, snap-to-edge, dock, taskbar (out of all current waves).
