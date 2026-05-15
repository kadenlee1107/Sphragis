# Caves Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Wave 3 — redesign the CAVES app to the Wave 1/2 calm register (Inspector layout, pointer+keyboard parity, inline editable create/configure form, double-tap-D destroy confirm, pure-mono state dot) per `docs/superpowers/specs/2026-05-14-caves-manager-design.md`. Surface 6 reusable widgets in `src/ui/widgets.rs` so Wave 4 apps inherit them directly.

**Architecture:** Extend `AppDescriptor` in `src/ui/apps_registry.rs` with `handle_key` and `handle_click` fn pointers + a tri-state `AppEvent` return. The desktop event loop in `src/ui/desktop.rs` routes input to the focused window's app first; the app returns `Unhandled` to fall through to the desktop's own state machine. Six new widgets live alongside the existing legacy widgets in `src/ui/widgets.rs` — old widgets stay for now (Wave 4 migrates other apps + deletes legacy as it goes). `caves_mgr.rs` is replaced entirely with a state-machine app composed of the new widgets.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, alloc available via linked-list allocator). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: QEMU with `-display cocoa -device virtio-keyboard-device -device virtio-mouse-device`.

**Verification reality check.** Same as Waves 1–2: this crate is `#![no_std] #![no_main]`, no `lib.rs`, no test harness. `cargo test` doesn't run kernel code. Every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus a QEMU walk-through against the spec at the end. There is no unit-test step. The QEMU walk-through is Task 14.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/caves-mgr-redesign
```
Expected: working tree clean before checkout; on branch `feat/caves-mgr-redesign` after.

- [ ] **Step 0b: Confirm the current build is clean before any edits.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both clean. If not, fix before proceeding.

- [ ] **Step 0c: Resolve the 5 kernel API gaps from the spec.**

Read the spec at `docs/superpowers/specs/2026-05-14-caves-manager-design.md` §"Kernel API gaps". For each gap, run the indicated `grep` against the existing kernel source and record the resolution in a fresh file `docs/superpowers/plans/2026-05-14-caves-manager-preflight.md`:

```bash
# 1. NET MODE per cave
grep -nE 'net_mode|NetMode|set_net|cave_net' src/caves/cave.rs src/net/mod.rs

# 2. MOUNT editing
grep -nE 'set_mount|mount_override|cave_mount' src/caves/cave.rs

# 3. TAINT setter
grep -nE 'pub fn (stamp|taint_stamp|set_taint)' src/caves/taint.rs src/caves/cave.rs 2>&1

# 4. Cave rename
grep -nE 'pub fn rename|set_name|cave_rename' src/caves/cave.rs

# 5. cave::list signature (already documented in spec — confirm line)
grep -nE 'pub fn list' src/caves/cave.rs
```

For each, record one of three resolutions in the pre-flight file:
- **EXISTS** — line number + signature; use as-is.
- **STUB IN TASK 2** — doesn't exist; Task 2 will add a minimal stub setter that stores the value without enforcement.
- **DEFER TO WAVE 4+** — read-only in the Wave 3 form; documented in the pre-flight file.

Expected resolution baseline (verify; adjust if grep finds otherwise):
- NET MODE: STUB IN TASK 2 (new `set_net_mode_by_name`).
- MOUNT: STUB IN TASK 2 (new `set_mount_by_name`).
- TAINT: EXISTS — `taint::stamp` per the 2026-05-13 journal. Confirm signature.
- Rename: DEFER TO WAVE 4+. NAME read-only in Configure form.
- `cave::list`: EXISTS at `src/caves/cave.rs:2034` — use directly.

Commit the pre-flight file:

```bash
git add docs/superpowers/plans/2026-05-14-caves-manager-preflight.md
git commit -m "$(cat <<'EOF'
plan: Wave 3 — kernel API gap pre-flight resolutions

Records how each of the 5 gaps from the spec was resolved before
implementation begins. Task 2 lands any kernel-side stubs the
investigation surfaced; later tasks assume those stubs exist.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## File structure

This plan creates and modifies the following files. Each new module has one clear responsibility.

| File | Status | Responsibility |
|------|--------|----------------|
| `docs/superpowers/plans/2026-05-14-caves-manager-preflight.md` | **NEW** | Records the Step 0c resolutions (read-only after Pre-flight). |
| `src/ui/palette.rs` | **NEW** | Pub `pub const BG / PANEL / HAIRLINE / INK / MID / FAINT` for Wave 1/2/3 palette. Shared across new widgets. |
| `src/ui/apps_registry.rs` | **MODIFY** | Add `handle_key` / `handle_click` fn pointers + `AppEvent` enum. |
| `src/ui/desktop.rs` | **MODIFY** | Route keyboard + pointer events to the focused window's app before desktop fallback. |
| `src/ui/wm.rs` | **MODIFY (minor)** | Add `pub fn body_rect(id: WindowId) -> Option<WindowRect>` so `desktop.rs` can compute body-local pointer coordinates. |
| `src/caves/cave.rs` | **MODIFY** | Stub setters per Step 0c (typically `set_net_mode_by_name`, `set_mount_by_name`, plus a `NetMode` enum). |
| `src/ui/widgets.rs` | **MODIFY** | Append 6 new widgets: `paint_state_dot`, `paint_status_field_list`, `paint_action_strip` + `action_strip_hit_test`, `InspectorLayout`, `ConfirmModal` (+ `paint_confirm_modal` / `confirm_modal_key`), `InlineEditForm` (+ supporting types). Existing legacy widgets stay; new ones use Wave-2 palette via `crate::ui::palette::*`. |
| `src/ui/apps/caves_mgr.rs` | **REPLACED** | New state-machine app: `AppState` enum, `paint`, `handle_key`, `handle_click`, dispatched per the new AppDescriptor contract. |

The legacy widgets at the top of `widgets.rs` (`draw_strip`, `draw_kv_row`, etc., plus the cyberpunk palette constants) stay untouched. Wave 4 migrates other apps off them and deletes legacy when nothing imports.

---

## Task 1: AppDescriptor input dispatch + WM body_rect

Extend the Wave-2 AppDescriptor so apps can receive keyboard and pointer events. Add the WM helper that lets `desktop.rs` know an event's body-local coordinates.

**Files:**
- Modify: `src/ui/apps_registry.rs`
- Modify: `src/ui/wm.rs`
- Modify: `src/ui/desktop.rs`
- Modify: each `src/ui/apps/*.rs` + `src/ui/shell.rs` to add no-op input handlers

- [ ] **Step 1: Add `AppEvent` enum + extended `AppDescriptor` in `src/ui/apps_registry.rs`.**

Replace the existing `AppDescriptor` struct with:

```rust
/// Result of an app's input handler. Tri-state so the desktop knows
/// whether to repaint, consume silently, or fall through to its own
/// keyboard/pointer table.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AppEvent {
    /// App handled the event; no repaint needed (e.g. quiet text input).
    Consumed,
    /// App handled the event; caller should repaint the desktop.
    Repaint,
    /// App did not handle the event; caller should run its own fallback
    /// (Tab cycle, ^D close, Esc dismiss, etc.).
    Unhandled,
}

pub struct AppDescriptor {
    pub id: AppId,
    pub label: &'static str,
    pub title: &'static str,
    pub paint: fn(WindowRect),
    pub handle_key: fn(u8) -> AppEvent,
    pub handle_click: fn(mx: i32, my: i32, body_rect: WindowRect) -> AppEvent,
}
```

- [ ] **Step 2: Update the `APPS` table with default no-op handlers.**

In `src/ui/apps_registry.rs`, add module-private default handlers and reference them in every `APPS` entry:

```rust
fn default_handle_key(_c: u8) -> AppEvent { AppEvent::Unhandled }
fn default_handle_click(_mx: i32, _my: i32, _rect: WindowRect) -> AppEvent { AppEvent::Unhandled }

pub static APPS: [AppDescriptor; 8] = [
    AppDescriptor { id: AppId::Caves,    label: "CAVES",    title: "CAVES",    paint: paint_caves,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Files,    label: "FILES",    title: "FILES",    paint: paint_files,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Net,      label: "NET",      title: "NET",      paint: paint_net,      handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Security, label: "SECURITY", title: "SECURITY", paint: paint_security, handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Editor,   label: "EDITOR",   title: "EDITOR",   paint: paint_editor,   handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Agent,    label: "AGENT",    title: "AGENT",    paint: paint_agent,    handle_key: default_handle_key, handle_click: default_handle_click },
];
```

CAVES will be updated to point at its real handlers in Task 9.

- [ ] **Step 3: Add `wm::body_rect` helper in `src/ui/wm.rs`.**

The chrome strip is `CHROME_H = 22` pixels tall and the 1-pixel border eats 1 pixel on each side. Body rect = window rect minus chrome and borders. Append near `pub fn get`:

```rust
/// Returns the body rect (inside chrome, inside borders) of a window
/// by id. None if no such window. Used by `desktop.rs` to compute
/// body-local pointer coordinates when forwarding clicks to apps.
pub fn body_rect(id: WindowId) -> Option<WindowRect> {
    let w = get(id)?;
    let r = w.rect;
    Some(WindowRect {
        x: r.x + 1,
        y: r.y + CHROME_H + 1,
        w: r.w.saturating_sub(2),
        h: r.h.saturating_sub(CHROME_H + 2),
    })
}
```

- [ ] **Step 4: Route key events to focused window's app in `desktop.rs::handle_key`.**

Insert app dispatch BETWEEN the system-priority shortcuts (Ctrl+K / Ctrl+L) and the desktop fallback table (Tab / Ctrl+D / Esc / 1-8 / N). Modify the function so the order is:

1. `check_panic_hotkey(c)` (already first)
2. System-priority shortcuts that always win: `0x0B` (Ctrl+K), `0x0C` (Ctrl+L)
3. App dispatch: if a window is focused, call its app's `handle_key`. If `Consumed` → `Event::None`; if `Repaint` → `Event::Repaint`; if `Unhandled` → fall through.
4. Desktop fallback: existing `0x09` (Tab), `0x04` (Ctrl+D), `0x1B` (Esc), `b'1'..=b'8'` (slot launch).

The body of `handle_key` becomes (paste verbatim):

```rust
fn handle_key(c: u8) -> Event {
    // Panic hotkey first — see existing comment block.
    if crate::security::check_panic_hotkey(c) {
        return Event::None;
    }

    // System-priority shortcuts: Ctrl+K (overlay) and Ctrl+L (lock)
    // are never overrideable by an app — security and global flow.
    match c {
        0x0B => {
            if topbar::config_sheet_open() { topbar::close_config_sheet(); }
            set_overlay_open(!overlay_open());
            return Event::Repaint;
        }
        0x0C => return Event::Lock,
        _ => {}
    }

    // App dispatch: if a window is focused, the app sees the key first.
    if let Some(focused_id) = wm::focused() {
        if let Some(w) = wm::get(focused_id) {
            if let Some(body) = wm::body_rect(focused_id) {
                let _ = body;
                let desc = crate::ui::apps_registry::descriptor(w.app);
                match (desc.handle_key)(c) {
                    crate::ui::apps_registry::AppEvent::Consumed => return Event::None,
                    crate::ui::apps_registry::AppEvent::Repaint  => return Event::Repaint,
                    crate::ui::apps_registry::AppEvent::Unhandled => { /* fall through */ }
                }
            }
        }
    }

    // Desktop fallback (existing table, minus the Ctrl+K/Ctrl+L cases
    // that moved above).
    match c {
        0x09 => { wm::cycle_focus(); Event::Repaint }
        0x04 => { if let Some(id) = wm::focused() { wm::close(id); } Event::Repaint }
        b'1'..=b'8' => {
            let slot = (c - b'1') as usize;
            let app = crate::ui::apps_registry::APPS[slot].id;
            let existing = wm::iter().find(|w| w.app == app).map(|w| w.id);
            match existing {
                Some(id) => wm::focus(id),
                None     => { wm::open(app, None); }
            }
            if overlay_open() { set_overlay_open(false); }
            Event::Repaint
        }
        0x1B => {
            if topbar::config_sheet_open() {
                topbar::close_config_sheet();
                return Event::Repaint;
            }
            if overlay_open() {
                set_overlay_open(false);
                Event::Repaint
            } else {
                Event::None
            }
        }
        _ => Event::None,
    }
}
```

- [ ] **Step 5: Route pointer Down events to focused window's app body in `desktop.rs::handle_pointer`.**

In the `PointerEvent::Down(x, y)` arm, AFTER the existing top-bar / overlay / empty-desktop checks and BEFORE `wm::begin_drag`, add app dispatch when the click lands inside a window body (not chrome / corner / close-glyph). Concretely, replace the tail of the Down arm:

```rust
            // (existing checks for topbar, overlay, empty-desktop unchanged)

            // App-first dispatch on body clicks.
            // wm::hit_test classifies the click; only Hit::Body forwards
            // to the app (corners/chrome/close-glyph stay with the WM
            // for drag/resize/close).
            match wm::hit_test(x, y) {
                wm::Hit::Body(id) => {
                    wm::focus(id);
                    if let Some(body) = wm::body_rect(id) {
                        if let Some(w) = wm::get(id) {
                            let desc = crate::ui::apps_registry::descriptor(w.app);
                            match (desc.handle_click)(x, y, body) {
                                crate::ui::apps_registry::AppEvent::Consumed => return Event::None,
                                crate::ui::apps_registry::AppEvent::Repaint  => return Event::Repaint,
                                crate::ui::apps_registry::AppEvent::Unhandled => { /* fall through */ }
                            }
                        }
                    }
                    return Event::Repaint;  // focus changed, repaint
                }
                _ => {}
            }

            // Fallback: existing WM drag/close/corner handling.
            if wm::begin_drag(x, y) { Event::Repaint } else { Event::None }
```

The exact integration depends on the current shape of the `Down` arm (Wave 2's Task 7 + later patches). Read `src/ui/desktop.rs::handle_pointer` before editing; if the structure has drifted, adapt while preserving the spec'd order.

- [ ] **Step 6: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both clean.

- [ ] **Step 7: Commit.**

```bash
git add src/ui/apps_registry.rs src/ui/wm.rs src/ui/desktop.rs
git commit -m "$(cat <<'EOF'
apps: AppDescriptor handle_key/handle_click + WM body_rect

Extends AppDescriptor with two fn-pointer fields (handle_key,
handle_click) and a tri-state AppEvent return so apps can intercept
input from the focused window. All 8 apps wired with default no-op
handlers that return Unhandled — current behavior preserved.

desktop.rs reorders handle_key + handle_pointer to route input to
the focused window's app between system-priority shortcuts and the
desktop fallback table. wm::body_rect gives the desktop body-local
pointer coords for app dispatch.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2: Kernel-side stubs for missing setters

Land any setters the spec needs that Step 0c flagged as STUB IN TASK 2. Conservative defaults assume net mode + mount are missing; verify against the pre-flight file before patching.

**Files:**
- Modify: `src/caves/cave.rs`
- Modify (if needed): `src/net/mod.rs`

- [ ] **Step 1: Add `NetMode` enum + `Cave.net_mode` field.**

In `src/caves/cave.rs`, add near the existing `Sensitivity` / `Integrity` enums:

```rust
/// Cave-scoped network policy. Stored per-cave. Wave 3 surfaces this
/// in the UI; the net subsystem doesn't enforce per-cave policy yet
/// (global `net::is_isolated()` still wins). Per-cave enforcement is
/// a Wave 4+ kernel item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NetMode {
    Isolated = 0,
    Routed   = 1,
    Custom   = 2,
}

impl NetMode {
    pub fn as_str(self) -> &'static str {
        match self {
            NetMode::Isolated => "isolated",
            NetMode::Routed   => "routed",
            NetMode::Custom   => "custom",
        }
    }
    pub fn from_u8(b: u8) -> NetMode {
        match b {
            1 => NetMode::Routed,
            2 => NetMode::Custom,
            _ => NetMode::Isolated,
        }
    }
}
```

In the `pub struct Cave { ... }` definition (around line 83), add a `net_mode: u8` field with default `0` (Isolated). The default applies in `create()`.

- [ ] **Step 2: Add `set_net_mode_by_name` setter.**

Right below `set_integrity_by_name` (around line 363):

```rust
pub fn set_net_mode_by_name(name: &str, mode: NetMode) -> Result<(), &'static str> {
    let cave_id = name_to_id(name).ok_or("cave not found")?;
    let caves = CAVES.lock();
    if let Some(cave) = caves.get_mut(cave_id) {
        cave.net_mode = mode as u8;
        Ok(())
    } else {
        Err("cave slot empty")
    }
}

pub fn net_mode_of(cave_id: u16) -> NetMode {
    let caves = CAVES.lock();
    caves.get(cave_id as usize)
        .map(|c| NetMode::from_u8(c.net_mode))
        .unwrap_or(NetMode::Isolated)
}
```

The `name_to_id` and `CAVES.lock()` shapes should already exist in `cave.rs`; verify the exact pattern (might be `cave_by_name` + an inner `unsafe { ... }` mutex). Match the file's prevailing style.

- [ ] **Step 3: Add `set_mount_by_name` setter.**

Caves currently derive mount paths from name. Wave 3 lets the user override. Same module:

```rust
/// Override the cave's mount string. Default (None) means "derive from
/// name: `/ /home/<name>`". Wave 3 stub — the actual mount mechanism
/// in BatFS isn't dynamically reconfigurable yet; this just stores
/// the user's intent.
pub fn set_mount_by_name(name: &str, mount: &str) -> Result<(), &'static str> {
    let cave_id = name_to_id(name).ok_or("cave not found")?;
    let caves = CAVES.lock();
    if let Some(cave) = caves.get_mut(cave_id) {
        let bytes = mount.as_bytes();
        let n = bytes.len().min(64);
        cave.mount_override[..n].copy_from_slice(&bytes[..n]);
        cave.mount_override_len = n as u8;
        Ok(())
    } else {
        Err("cave slot empty")
    }
}

pub fn mount_of(cave_id: u16) -> Option<&'static str> {
    let caves = CAVES.lock();
    let cave = caves.get(cave_id as usize)?;
    if cave.mount_override_len == 0 { return None; }
    // SAFETY: mount_override is ASCII-only (set_mount_by_name doesn't
    // do char-boundary checks; tighten in a future wave if non-ASCII
    // names arrive).
    let n = cave.mount_override_len as usize;
    Some(unsafe { core::str::from_utf8_unchecked(&cave.mount_override[..n]) })
}
```

Add `mount_override: [u8; 64]` and `mount_override_len: u8` to the `Cave` struct.

- [ ] **Step 4: Verify `taint::stamp` signature.**

If the pre-flight investigation showed `taint::stamp(cave_id: u16, value: u32) -> Result<...>` exists, no work — the create form just calls it after `cave::create`. If signature differs, adapt the call site in Task 11 to match.

- [ ] **Step 5: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 6: Commit.**

```bash
git add src/caves/cave.rs
git commit -m "$(cat <<'EOF'
caves: per-cave NetMode + mount-override stubs

Adds:
* NetMode enum (Isolated/Routed/Custom) + Cave.net_mode field
* set_net_mode_by_name, net_mode_of
* Cave.mount_override + mount_override_len
* set_mount_by_name, mount_of
* Default at create(): net_mode=Isolated, mount_override empty
  (caller derives from name).

Both are Wave 3 stubs: storage only, no enforcement. The net
subsystem still uses global `is_isolated()`; BatFS mounts are still
derived from name. Per-cave enforcement of both is a Wave 4+ kernel
item. Surfaced now so the Wave 3 create/configure form can capture
the user's intent without losing it.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: Palette module + StateDot widget

Foundation for every Wave 3 widget: a shared palette module so the new widgets aren't pasting `0xFFE5E7EB` everywhere, plus the simplest widget (`paint_state_dot`).

**Files:**
- Create: `src/ui/palette.rs`
- Modify: `src/ui/mod.rs` (register module)
- Modify: `src/ui/widgets.rs` (append StateDot section)

- [ ] **Step 1: Create `src/ui/palette.rs`.**

```rust
//! Shared Wave-1/2/3 palette constants.
//!
//! Modules predating this file (wm.rs, topbar.rs, launcher.rs) define
//! their own private palette constants with the same hex values.
//! Refactoring those to import from here is a Wave-4+ cleanup; for
//! now this module is the canonical home for new widget code.

#![allow(dead_code)]

pub const BG:       u32 = 0xFF0D0D10;
pub const PANEL:    u32 = 0xFF18181C;
pub const HAIRLINE: u32 = 0xFF2A2A30;
pub const INK:      u32 = 0xFFE5E7EB;
pub const MID:      u32 = 0xFF6B7280;

/// Disabled-action color. ~50% of MID; used by the action strip when
/// a hotkey is contextually unavailable (e.g. Stop on a stopped cave).
pub const FAINT:    u32 = 0xFF4A4D55;
```

- [ ] **Step 2: Register the module in `src/ui/mod.rs`.**

Add `pub mod palette;` alphabetically (likely between `launcher` and `shell`). Verify with:

```bash
grep -n 'pub mod' src/ui/mod.rs
```

- [ ] **Step 3: Append the StateDot widget to `src/ui/widgets.rs`.**

Append below the existing widgets:

```rust
// ── Wave 3 widgets ───────────────────────────────────────────────
// Used by caves_mgr (Wave 3) and inherited by FILES/NET/SECURITY/
// EDITOR/COMMS in Wave 4. New widgets import from `crate::ui::palette`;
// legacy widgets above keep their local cyberpunk palette.

use crate::ui::palette as p;

/// 6x6 px state indicator. `filled` = INK solid circle (running state).
/// `!filled` = MID 1-px ring over BG fill (idle/stopped state).
/// Renders inside a 6x6 bounding box at (x, y).
pub fn paint_state_dot(x: u32, y: u32, filled: bool) {
    use crate::ui::gpu;
    // Pre-baked 6x6 dot bitmaps. 1 = pixel set, 0 = transparent.
    // Filled (●) and hollow (○) shapes hand-tuned for 6x6 grid.
    const FILLED: [[u8; 6]; 6] = [
        [0, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 1, 0],
    ];
    const HOLLOW: [[u8; 6]; 6] = [
        [0, 1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [0, 1, 1, 1, 1, 0],
    ];
    let bm = if filled { &FILLED } else { &HOLLOW };
    let color = if filled { p::INK } else { p::MID };
    for dy in 0..6u32 {
        for dx in 0..6u32 {
            if bm[dy as usize][dx as usize] == 1 {
                gpu::fill_rect(x + dx, y + dy, 1, 1, color);
            }
        }
    }
}
```

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/palette.rs src/ui/mod.rs src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui: shared palette module + paint_state_dot widget

* New `src/ui/palette.rs` with pub Wave-1/2/3 palette constants.
  Wave 1/2 modules keep their local private constants; refactoring
  to import from here is a Wave-4+ cleanup item.
* First Wave-3 widget: `paint_state_dot(x, y, filled)`. 6x6 px.
  Filled = INK solid circle (running); hollow = MID ring over BG
  (idle/stopped). Used by caves_mgr sidebar; inherited by NET (link
  up/down), FILES (saved/unsaved), SECURITY (armed/disarmed).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 4: StatusFieldList widget

Vertical list of `KEY  value` rows. Auto-aligned key column. Used by caves_mgr's detail panel and inherited by every Wave 4 app's status surface.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget to `src/ui/widgets.rs`.**

Below the StateDot section:

```rust
/// One key/value row in a status field list.
#[derive(Copy, Clone)]
pub struct StatusField<'a> {
    pub key:   &'a str,
    pub value: &'a str,
}

/// Paint a vertical list of key/value rows. Key column auto-sized to
/// the longest key + 2-char padding. Keys render in MID; values in INK.
/// Row height matches the bitmap font (~18 px including 2 px padding).
///
/// Caller is responsible for clipping — this paints exactly
/// `fields.len() * 18` pixels of height starting at `rect.y`.
pub fn paint_status_field_list(rect: WindowRect, fields: &[StatusField]) {
    use crate::ui::{font, gpu};

    const ROW_H:  u32 = 18;
    const CHAR_W: u32 = 8;

    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // Compute key column width.
    let mut max_key_len: u32 = 0;
    for f in fields {
        let n = f.key.len() as u32;
        if n > max_key_len { max_key_len = n; }
    }
    let value_col_x = rect.x + (max_key_len + 2) * CHAR_W;

    for (i, f) in fields.iter().enumerate() {
        let row_y = rect.y + (i as u32) * ROW_H + 1; // +1 to push below row top
        font::draw_str(fb, screen_w, rect.x,       row_y, f.key,   p::MID, p::BG);
        font::draw_str(fb, screen_w, value_col_x,  row_y, f.value, p::INK, p::BG);
    }
}
```

Note: `WindowRect` import already at the top of widgets.rs (it's used by legacy widgets too). If not, add `use crate::ui::wm::WindowRect;` near the existing imports.

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. If clippy flags dead-code on the new widget, leave it — Task 9 wires the first caller.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_status_field_list

Generic key/value row list. Key column auto-sized to longest key
plus 2 chars padding; keys in MID, values in INK. Used by caves_mgr
detail panel (PID/NET/MLS/MOUNT/TAINT/AUDIT) and inherited by every
Wave 4 app's status surface.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 5: ActionStrip widget

Bottom-of-panel hotkey row: `[E]nter · [S]top · [C]onfigure · [D]estroy`. Letter in INK, rest in MID. Whole token clickable; hotkey on the keyboard equivalent.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget to `src/ui/widgets.rs`.**

```rust
/// A single entry in an action strip.
#[derive(Copy, Clone)]
pub struct Action<'a> {
    /// Uppercase ASCII hotkey, e.g. 'E'. Matches the bracketed letter
    /// in `label`. Caller decides what `b'E'` (or 'e' lowercased)
    /// means; this widget only paints.
    pub hotkey:  char,
    /// Action label rendered as `[<hotkey>]<rest>`, e.g. "Enter"
    /// renders as `[E]nter`.
    pub label:   &'a str,
    /// false → painted in FAINT, hit-tested as a miss (caller's
    /// keyboard handler also ignores the hotkey when disabled).
    pub enabled: bool,
}

/// Paint a row of `[K]ey label · ...` actions across the bottom of
/// `rect`. Items are separated by a `·` glyph in MID. The whole strip
/// is left-aligned starting at `rect.x + 8`.
pub fn paint_action_strip(rect: WindowRect, actions: &[Action]) {
    use crate::ui::{font, gpu};

    const CHAR_W: u32 = 8;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    let y = rect.y + (rect.h.saturating_sub(16)) / 2;
    let mut x = rect.x + 8;

    for (i, act) in actions.iter().enumerate() {
        if i > 0 {
            font::draw_str(fb, screen_w, x, y, " · ", p::MID, p::BG);
            x += 3 * CHAR_W;
        }
        let (letter_color, rest_color) = if act.enabled {
            (p::INK, p::MID)
        } else {
            (p::FAINT, p::FAINT)
        };
        // "[X]"
        let mut bracket_buf = [0u8; 3];
        bracket_buf[0] = b'[';
        bracket_buf[1] = act.hotkey as u8;
        bracket_buf[2] = b']';
        let bracket = unsafe { core::str::from_utf8_unchecked(&bracket_buf) };
        font::draw_str(fb, screen_w, x, y, bracket, letter_color, p::BG);
        x += 3 * CHAR_W;
        // "rest" (label minus the first character, which the bracket replaced)
        if act.label.len() > 1 {
            let rest = &act.label[1..];
            font::draw_str(fb, screen_w, x, y, rest, rest_color, p::BG);
            x += rest.len() as u32 * CHAR_W;
        }
    }
}

/// Hit-test the action strip. Returns the hotkey of the clicked
/// action if the click landed on a label, None otherwise.
/// Disabled actions return None even if clicked on.
pub fn action_strip_hit_test(rect: WindowRect, mx: i32, my: i32, actions: &[Action]) -> Option<char> {
    const CHAR_W: u32 = 8;

    let strip_y0 = rect.y as i32;
    let strip_y1 = (rect.y + rect.h) as i32;
    if my < strip_y0 || my >= strip_y1 { return None; }

    let mut x = rect.x as i32 + 8;
    for (i, act) in actions.iter().enumerate() {
        if i > 0 {
            x += 3 * CHAR_W as i32; // separator " · "
        }
        let token_w = 3 * CHAR_W as i32  // "[X]"
                    + act.label.len().saturating_sub(1) as i32 * CHAR_W as i32;
        if mx >= x && mx < x + token_w {
            return if act.enabled { Some(act.hotkey) } else { None };
        }
        x += token_w;
    }
    None
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
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_action_strip + action_strip_hit_test

Bottom-of-panel hotkey row. Each entry renders as `[K]label` with
the bracketed letter in INK (enabled) or FAINT (disabled); rest of
label in MID. Items separated by ` · ` in MID. hit_test returns the
hotkey of the clicked enabled action, None on miss or disabled.

Used by caves_mgr's detail-panel footer (Enter / Stop / Configure /
Destroy). Inherited by every Wave 4 app's action surface.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 6: InspectorLayout widget

Sidebar + detail split helper. Mostly geometry math; returns rects the caller paints into.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget to `src/ui/widgets.rs`.**

```rust
/// Inspector-style sidebar + detail split. Caller decides what to paint
/// in each rect; this widget just computes geometry and paints the
/// 1-px HAIRLINE divider between them.
#[derive(Copy, Clone)]
pub struct InspectorLayout {
    pub body_rect:   WindowRect,
    pub sidebar_pct: u32,   // 0..100; default 38
}

impl InspectorLayout {
    pub fn new(body_rect: WindowRect) -> Self {
        Self { body_rect, sidebar_pct: 38 }
    }

    pub fn with_sidebar_pct(mut self, pct: u32) -> Self {
        self.sidebar_pct = pct.min(80).max(20);
        self
    }

    pub fn sidebar_rect(&self) -> WindowRect {
        let w = (self.body_rect.w * self.sidebar_pct) / 100;
        WindowRect {
            x: self.body_rect.x,
            y: self.body_rect.y,
            w,
            h: self.body_rect.h,
        }
    }

    pub fn detail_rect(&self) -> WindowRect {
        let sw = (self.body_rect.w * self.sidebar_pct) / 100;
        WindowRect {
            x: self.body_rect.x + sw + 1, // +1 for the divider
            y: self.body_rect.y,
            w: self.body_rect.w.saturating_sub(sw + 1),
            h: self.body_rect.h,
        }
    }

    /// Paint the 1-px HAIRLINE vertical divider.
    pub fn paint_divider(&self) {
        use crate::ui::gpu;
        let sw = (self.body_rect.w * self.sidebar_pct) / 100;
        gpu::fill_rect(self.body_rect.x + sw, self.body_rect.y, 1, self.body_rect.h, p::HAIRLINE);
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
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: InspectorLayout

Sidebar + detail geometry helper. Default 38% sidebar, clamped to
20..=80. Provides sidebar_rect / detail_rect / paint_divider; caller
paints into the returned rects. Used by caves_mgr (Wave 3) and
inherited by FILES / NET / SECURITY (Wave 4).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 7: ConfirmModal widget

Centered destructive-action overlay. Dims the area below TOPBAR_H, draws a PANEL-filled panel with HAIRLINE border. Caller supplies title + body lines + commit-key.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget to `src/ui/widgets.rs`.**

```rust
/// Confirmation modal — used wherever an action is destructive.
/// Double-tap the `commit_key` to confirm. Esc cancels.
pub struct ConfirmModal<'a> {
    pub title:       &'a str,
    pub body_lines:  &'a [&'a str],
    pub commit_key:  char,   // uppercase ASCII, e.g. 'D' for Destroy
}

/// Result of routing a key event to the modal.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModalAction {
    None,
    Commit,
    Cancel,
}

/// Paint the modal centered on screen, dimming everything below TOPBAR_H.
/// The TOPBAR strip stays live (lock glyph still works).
pub fn paint_confirm_modal(modal: &ConfirmModal) {
    use crate::ui::{font, gpu};

    const TOPBAR_H: u32 = 22;
    const CHAR_W:   u32 = 8;
    const CHAR_H:   u32 = 16;
    const PAD_X:    u32 = 24;
    const PAD_Y:    u32 = 18;

    let screen_w = gpu::width();
    let screen_h = gpu::height();
    let fb = gpu::framebuffer();

    // Dim everything below the topbar (alpha-blend toward BG at ~35%).
    // No real alpha — we just overdraw with BG and trust the modal
    // panel to repaint on top. The dim layer is implicit: the modal
    // covers most of the screen; behind it we re-fill with BG.
    gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, p::BG);

    // Compute modal size from content.
    let mut max_line_w = modal.title.len() as u32;
    for line in modal.body_lines {
        if (line.len() as u32) > max_line_w { max_line_w = line.len() as u32; }
    }
    let body_h_lines = modal.body_lines.len() as u32;
    let panel_w = (max_line_w * CHAR_W) + 2 * PAD_X;
    let panel_h = CHAR_H                            // title
                + 8                                 // title-body gap
                + body_h_lines * (CHAR_H + 4)       // body lines
                + 18                                // body-footer gap
                + CHAR_H                            // footer hint
                + 2 * PAD_Y;

    let px = (screen_w.saturating_sub(panel_w)) / 2;
    let py = (screen_h.saturating_sub(panel_h)) / 2;

    // Panel fill + border.
    gpu::fill_rect(px, py, panel_w, panel_h, p::PANEL);
    // 1-px HAIRLINE border
    gpu::fill_rect(px, py, panel_w, 1, p::HAIRLINE);
    gpu::fill_rect(px, py + panel_h - 1, panel_w, 1, p::HAIRLINE);
    gpu::fill_rect(px, py, 1, panel_h, p::HAIRLINE);
    gpu::fill_rect(px + panel_w - 1, py, 1, panel_h, p::HAIRLINE);

    let inner_x = px + PAD_X;
    let mut y = py + PAD_Y;

    // Title.
    font::draw_str(fb, screen_w, inner_x, y, modal.title, p::INK, p::PANEL);
    y += CHAR_H + 8;

    // Body lines.
    for line in modal.body_lines {
        font::draw_str(fb, screen_w, inner_x, y, line, p::MID, p::PANEL);
        y += CHAR_H + 4;
    }

    // Footer hint.
    y += 14;
    let mut hint = [0u8; 64];
    let mut n = 0;
    let prefix = b" again to confirm  ";
    hint[n] = modal.commit_key as u8; n += 1;
    for &b in prefix { if n < hint.len() { hint[n] = b; n += 1; } }
    let mid = b"  Esc to cancel";
    for &b in mid { if n < hint.len() { hint[n] = b; n += 1; } }
    // Render hint with the commit-key letter standing out in INK.
    let key_glyph = [modal.commit_key as u8];
    let key_str = unsafe { core::str::from_utf8_unchecked(&key_glyph) };
    font::draw_str(fb, screen_w, inner_x,            y, key_str, p::INK, p::PANEL);
    font::draw_str(fb, screen_w, inner_x + CHAR_W,   y, " again to confirm  ", p::MID, p::PANEL);
    font::draw_str(fb, screen_w, inner_x + 21 * CHAR_W, y, "Esc to cancel", p::MID, p::PANEL);
}

/// Route a key event to the modal. Returns Commit on the commit key,
/// Cancel on Esc, None otherwise. Caller is responsible for tracking
/// whether the modal is open — this fn doesn't.
pub fn confirm_modal_key(modal: &ConfirmModal, c: u8) -> ModalAction {
    let lower = if c >= b'A' && c <= b'Z' { c + 32 } else { c };
    let key_lower = (modal.commit_key as u8).to_ascii_lowercase();
    if lower == key_lower { return ModalAction::Commit; }
    if c == 0x1B { return ModalAction::Cancel; }
    ModalAction::None
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
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: ConfirmModal + paint_confirm_modal + confirm_modal_key

Centered destructive-action overlay. Dims everything below TOPBAR_H,
draws a PANEL-filled panel with HAIRLINE border. Title in INK, body
lines in MID, footer hint with the commit-key letter highlighted.
Double-tap the commit key (Commit) or Esc (Cancel). Caller tracks
modal-open state.

Used by caves_mgr Destroy flow; inherited by every Wave 4 app with
destructive actions (FILES delete, SECURITY wipe, etc.).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 8: InlineEditForm widget

The most complex widget: editable form with Tab field cycling, Space/← → enum cycling, text input.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the field types + form to `src/ui/widgets.rs`.**

```rust
/// One field in an inline edit form.
pub enum FieldKind<'a> {
    /// Text field; caller owns the buffer + length. `max` is the
    /// hard cap (no more characters accepted once `len == max`).
    Text { buf: &'a mut [u8], len: &'a mut usize, max: usize },
    /// Single-select enum. Selected index cycles via Space / ← → ;
    /// caller controls the variant list.
    Enum { values: &'a [&'static str], selected: &'a mut usize },
    /// 32-bit hex value. Caller stores the value; widget handles
    /// in-place editing of hex digits.
    Hex32 { value: &'a mut u32 },
}

pub struct FormField<'a> {
    pub key:      &'a str,
    pub kind:     FieldKind<'a>,
    pub readonly: bool,
}

/// Result of a key dispatched to the form.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FormAction {
    None,
    Submit,
    Cancel,
}

/// Paint the form into `rect`. `focused` is the index of the field
/// currently focused (highlighted with INK border instead of HAIRLINE).
pub fn paint_inline_edit_form(rect: WindowRect, fields: &[FormField], focused: usize) {
    use crate::ui::{font, gpu};

    const CHAR_W:    u32 = 8;
    const CHAR_H:    u32 = 16;
    const ROW_H:     u32 = 42;       // 10 px key + 24 px field + 8 px gap
    const KEY_H:     u32 = 10;
    const FIELD_H:   u32 = 22;

    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    for (i, field) in fields.iter().enumerate() {
        let row_y = rect.y + (i as u32) * ROW_H;
        // Key label
        font::draw_str(fb, screen_w, rect.x, row_y, field.key, p::MID, p::BG);
        // Field box
        let box_x = rect.x;
        let box_y = row_y + KEY_H + 2;
        let box_w = rect.w;
        let box_h = FIELD_H;
        gpu::fill_rect(box_x, box_y, box_w, box_h, p::PANEL);
        let border = if focused == i { p::INK } else { p::HAIRLINE };
        // 1-px border
        gpu::fill_rect(box_x, box_y, box_w, 1, border);
        gpu::fill_rect(box_x, box_y + box_h - 1, box_w, 1, border);
        gpu::fill_rect(box_x, box_y, 1, box_h, border);
        gpu::fill_rect(box_x + box_w - 1, box_y, 1, box_h, border);

        // Field contents
        let text_y = box_y + (box_h - CHAR_H) / 2;
        let text_x = box_x + 8;
        let val_color = if field.readonly { p::MID } else { p::INK };

        match &field.kind {
            FieldKind::Text { buf, len, .. } => {
                let n = **len;
                let s = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
                font::draw_str(fb, screen_w, text_x, text_y, s, val_color, p::PANEL);
                // Cursor caret (only when focused, not readonly).
                if focused == i && !field.readonly {
                    let cx = text_x + (n as u32) * CHAR_W;
                    gpu::fill_rect(cx, text_y, CHAR_W, CHAR_H, p::INK);
                }
            }
            FieldKind::Enum { values, selected } => {
                let s = values.get(**selected).copied().unwrap_or("?");
                font::draw_str(fb, screen_w, text_x, text_y, s, val_color, p::PANEL);
                // Right-edge hint "← →"
                if focused == i && !field.readonly {
                    let hint = "← →";
                    let hx = box_x + box_w - 4 * CHAR_W;
                    font::draw_str(fb, screen_w, hx, text_y, hint, p::MID, p::PANEL);
                }
            }
            FieldKind::Hex32 { value } => {
                let mut buf = [b'0'; 10];
                buf[1] = b'x';
                for j in 0..8 {
                    let nibble = (**value >> ((7 - j) * 4)) & 0xF;
                    buf[2 + j] = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
                }
                let s = unsafe { core::str::from_utf8_unchecked(&buf) };
                font::draw_str(fb, screen_w, text_x, text_y, s, val_color, p::PANEL);
            }
        }
    }
}

/// Route a key to the form. Returns Submit on Enter (when valid),
/// Cancel on Esc, None otherwise. Updates `*focused` on Tab/Shift+Tab,
/// mutates the focused field's storage on text/enum/hex edits.
pub fn handle_form_key(fields: &mut [FormField], focused: &mut usize, c: u8) -> FormAction {
    if c == 0x1B { return FormAction::Cancel; }
    if c == b'\r' || c == b'\n' { return FormAction::Submit; }
    if c == b'\t' {
        let n = fields.len();
        if n > 0 { *focused = (*focused + 1) % n; }
        return FormAction::None;
    }
    // Shift+Tab arrives as 0x90..0x97 range from the kernel keyboard
    // layer when Shift is held with arrows. Plain Shift+Tab is rare;
    // we don't handle it for Wave 3.

    let i = *focused;
    if i >= fields.len() { return FormAction::None; }
    if fields[i].readonly { return FormAction::None; }

    match &mut fields[i].kind {
        FieldKind::Text { buf, len, max } => {
            if c == 0x08 || c == 0x7F {  // Backspace / Delete
                if **len > 0 { **len -= 1; }
            } else if c >= b' ' && c < 0x7F {
                if **len < *max && **len < buf.len() {
                    buf[**len] = c;
                    **len += 1;
                }
            }
        }
        FieldKind::Enum { values, selected } => {
            if c == b' ' {
                let n = values.len();
                if n > 0 { **selected = (**selected + 1) % n; }
            }
            // ← arrow = 0x92, → arrow = 0x93 per the kernel arrow mapping.
            if c == 0x92 {
                let n = values.len();
                if n > 0 { **selected = (**selected + n - 1) % n; }
            }
            if c == 0x93 {
                let n = values.len();
                if n > 0 { **selected = (**selected + 1) % n; }
            }
        }
        FieldKind::Hex32 { value } => {
            // Hex edit: shift left, accept new low nibble.
            let nibble = match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            };
            if let Some(n) = nibble {
                **value = (**value << 4) | (n as u32);
            }
            if c == 0x08 {  // Backspace
                **value >>= 4;
            }
        }
    }
    FormAction::None
}

/// Route a click to the form. Updates `*focused` to the clicked field
/// if the click landed on a field box. Does NOT submit (caller handles
/// submit-button click separately if it has one).
pub fn handle_form_click(fields: &[FormField], focused: &mut usize, rect: WindowRect, mx: i32, my: i32) {
    const ROW_H: u32 = 42;
    const KEY_H: u32 = 10;
    const FIELD_H: u32 = 22;
    let mx = mx as i32;
    let my = my as i32;
    if mx < rect.x as i32 || mx >= (rect.x + rect.w) as i32 { return; }
    if my < rect.y as i32 { return; }
    for (i, _) in fields.iter().enumerate() {
        let box_y = rect.y as i32 + (i as i32) * ROW_H as i32 + KEY_H as i32 + 2;
        if my >= box_y && my < box_y + FIELD_H as i32 {
            *focused = i;
            return;
        }
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
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: InlineEditForm (Tab/Space/Enter + Text/Enum/Hex32)

Editable form with three field kinds: Text (buf+len+max), Enum
(values+selected), Hex32 (u32 in-place hex edit). Tab cycles fields
(wraps); Space cycles enums; ← → also cycle enums; printable chars
fill Text; Backspace deletes; hex digits + Backspace edit Hex32.
Focused field renders INK border vs HAIRLINE; readonly fields ignore
edits but render in MID. Click hit-tests select the field clicked.

Caller manages submit/cancel logic: handle_form_key returns Submit
on Enter / Cancel on Esc / None otherwise.

Used by caves_mgr create + configure forms. Inherited by FILES
properties, SECURITY deadman config, etc. in Wave 4.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 9: caves_mgr scaffolding (AppState + paint dispatch)

Replace `src/ui/apps/caves_mgr.rs` with the new state machine and wire its handlers into `apps_registry.rs`. Empty state for now; later tasks fill in each mode.

**Files:**
- Replace: `src/ui/apps/caves_mgr.rs`
- Modify: `src/ui/apps_registry.rs`

- [ ] **Step 1: Replace `src/ui/apps/caves_mgr.rs` entirely.**

```rust
//! Wave 3 Caves Manager. State-machine app composed of the
//! `src/ui/widgets.rs` Wave-3 widget set.
//!
//! See `docs/superpowers/specs/2026-05-14-caves-manager-design.md`.

#![allow(dead_code)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_state_dot, paint_status_field_list, StatusField,
    paint_action_strip, action_strip_hit_test, Action,
    InspectorLayout,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
    paint_inline_edit_form, handle_form_key, handle_form_click,
    FieldKind, FormField, FormAction,
};
use crate::ui::wm::WindowRect;

// ── App state ────────────────────────────────────────────────────

const NAME_MAX: usize = 16;
const MOUNT_MAX: usize = 64;

/// Per-mode form scratch. Lives across paint calls when the user is
/// typing in CREATE / CONFIGURE; reset on entry to the mode.
struct FormScratch {
    name_buf:        [u8; NAME_MAX],
    name_len:        usize,
    net_mode_sel:    usize,  // 0=Isolated, 1=Routed, 2=Custom
    mls_sens_sel:    usize,  // 0=U, 1=C, 2=S, 3=TS
    mls_integ_sel:   usize,  // 0=Untrusted, 1=Sandboxed, 2=SystemTrusted, 3=HighIntegrity
    mount_buf:       [u8; MOUNT_MAX],
    mount_len:       usize,
    mount_user_dirty: bool,   // false = auto-derived from name
    taint:           u32,
    focused_field:   usize,
}

impl FormScratch {
    fn empty() -> Self {
        Self {
            name_buf: [0; NAME_MAX],
            name_len: 0,
            net_mode_sel: 0,
            mls_sens_sel: 1,    // Confidential
            mls_integ_sel: 1,   // Sandboxed
            mount_buf: [0; MOUNT_MAX],
            mount_len: 0,
            mount_user_dirty: false,
            taint: 0,
            focused_field: 0,
        }
    }
}

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    Creating,
    Configuring(usize),         // index of cave being configured
    ConfirmDestroy(usize),
}

// Static state. Volatile access matches Wave 2 / 3 convention.
static mut SELECTED_CAVE: usize = 0;
static mut APP_MODE: AppMode = AppMode::Viewing;
static mut FORM: Option<FormScratch> = None;
static mut LAST_DESTROY_TAP: bool = false;  // tracks the first D vs the commit D

fn selected_cave() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECTED_CAVE)) }
}
fn set_selected_cave(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_CAVE), v) }
}
// NOTE: AppMode is not Copy because of Configuring(usize); access via
// addr_of! for reads and replace() for writes.
fn mode_is(m: &AppMode) -> bool {
    unsafe { &*core::ptr::addr_of!(APP_MODE) == m }
}

// ── Public API (wired in apps_registry.rs) ───────────────────────

pub fn paint(body: WindowRect) {
    // Background fill.
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    // Layout.
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    layout.paint_divider();
    paint_sidebar(layout.sidebar_rect());

    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing            => paint_detail_view(layout.detail_rect()),
        AppMode::Creating           => paint_detail_create(layout.detail_rect()),
        AppMode::Configuring(_)     => paint_detail_configure(layout.detail_rect()),
        AppMode::ConfirmDestroy(_)  => {
            // Scaffold only — Task 13 replaces this arm with the
            // real modal that resolves the cave name from the index.
            paint_detail_view(layout.detail_rect());
        }
    }
}

pub fn handle_key(_c: u8) -> AppEvent {
    // Wired per-mode in Tasks 10-13. For now return Unhandled so the
    // desktop fallback continues to handle every key.
    AppEvent::Unhandled
}

pub fn handle_click(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Unhandled
}

// ── Sidebar paint ────────────────────────────────────────────────

fn paint_sidebar(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    // Header
    let count = crate::caves::cave::count() as u32;
    let mut hdr_buf = [0u8; 24];
    hdr_buf[..7].copy_from_slice(b"CAVES (");
    let digits = u32_decimal(count, &mut hdr_buf, 7);
    hdr_buf[7 + digits] = b')';
    let hdr_len = 7 + digits + 1;
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_len]) };
    font::draw_str(fb, screen_w, rect.x + 8, rect.y + 6, hdr, p::MID, p::BG);
    gpu::fill_rect(rect.x, rect.y + 24, rect.w, 1, p::HAIRLINE);

    // Cave list rendered in Task 10. Pinned "+ new cave" rendered in Task 10.
}

// Helper: format a u32 in decimal at `buf[offset..]`. Returns the number of digits written.
fn u32_decimal(mut n: u32, buf: &mut [u8], offset: usize) -> usize {
    if n == 0 { buf[offset] = b'0'; return 1; }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in 0..i {
        buf[offset + j] = tmp[i - j - 1];
    }
    i
}

// ── Detail-view paint (stub for now; Task 10 fills in) ───────────

fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    if crate::caves::cave::count() == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No caves yet. Press N to create.", p::MID, p::BG);
        return;
    }

    // Task 10 fills in the cave-detail rendering.
    let _ = (font, gpu, fb, screen_w);
    let _ = rect;
}

fn paint_detail_create(rect: WindowRect)    { let _ = rect; /* Task 11 */ }
fn paint_detail_configure(rect: WindowRect) { let _ = rect; /* Task 12 */ }
```

The Wave-2 shim that adapts `render()` → `paint(rect)` no longer exists; the new module owns `paint` directly.

- [ ] **Step 2: Wire the new handlers in `src/ui/apps_registry.rs`.**

Change the CAVES entry in `APPS`:

```rust
AppDescriptor {
    id: AppId::Caves,
    label: "CAVES",
    title: "CAVES",
    paint: paint_caves,
    handle_key:   crate::ui::apps::caves_mgr::handle_key,
    handle_click: crate::ui::apps::caves_mgr::handle_click,
},
```

And update the `paint_caves` callback to pass through the rect:

```rust
fn paint_caves(rect: WindowRect) {
    crate::ui::apps::caves_mgr::paint(rect);
}
```

(The Wave-2 shim was `crate::ui::apps::caves_mgr::paint(rect)` already — Task 1's app rewrite drops the Wave-2 shim; this step just confirms the callback resolves.)

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. Empty state should now render correctly in QEMU (cave list shows just the header; detail panel says "No caves yet. Press N to create.").

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps/caves_mgr.rs src/ui/apps_registry.rs
git commit -m "$(cat <<'EOF'
caves_mgr: Wave 3 scaffolding — state machine + paint dispatch

Replaces the legacy 508-line cyberpunk caves_mgr with the new
state-machine app. AppMode (Viewing/Creating/Configuring/
ConfirmDestroy) lives in static-volatile state; paint dispatches per
mode. Wave-3 widget composition: InspectorLayout split, sidebar with
CAVES (n) header + divider, empty-state message in the detail panel.

handle_key + handle_click return Unhandled for now — Tasks 10-13 wire
each mode's keyboard + pointer behavior. apps_registry's CAVES entry
points at the new handlers.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 10: caves_mgr Viewing mode

Sidebar list with cave rows + state dots; detail panel with name header + state line + status fields + action strip. Up/Down + click selection. Hotkeys for E/S/C/D + N.

**Files:**
- Modify: `src/ui/apps/caves_mgr.rs`

- [ ] **Step 1: Implement sidebar list + pinned `+ new cave` row.**

Replace `paint_sidebar` and add the helper. The list paints each cave with a state dot + name; selected row gets PANEL highlight + INK selection arrow. The `+ new cave` row is pinned at the bottom.

```rust
fn paint_sidebar(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let count = crate::caves::cave::count() as u32;
    let mut hdr_buf = [0u8; 24];
    hdr_buf[..7].copy_from_slice(b"CAVES (");
    let digits = u32_decimal(count, &mut hdr_buf, 7);
    hdr_buf[7 + digits] = b')';
    let hdr_len = 7 + digits + 1;
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_len]) };
    font::draw_str(fb, screen_w, rect.x + 8, rect.y + 6, hdr, p::MID, p::BG);
    gpu::fill_rect(rect.x, rect.y + 24, rect.w, 1, p::HAIRLINE);

    // List rows.
    let row_h: u32 = 22;
    let sel = selected_cave();
    let creating = mode_is(&AppMode::Creating);
    let mut row_index: usize = 0;

    crate::caves::cave::list(|cave| {
        let row_y = rect.y + 28 + (row_index as u32) * row_h;
        if row_y + row_h > rect.y + rect.h { return; }

        let is_sel = !creating && row_index == sel;
        if is_sel {
            gpu::fill_rect(rect.x, row_y, rect.w, row_h, p::PANEL);
            font::draw_str(fb, screen_w, rect.x + 4, row_y + 3, "›", p::INK, p::PANEL);
        }

        paint_state_dot(rect.x + 18, row_y + 7, cave.is_running());
        font::draw_str(
            fb, screen_w, rect.x + 30, row_y + 3,
            cave.name_str(),
            if is_sel { p::INK } else { p::MID },
            if is_sel { p::PANEL } else { p::BG },
        );
        row_index += 1;
    });

    // Pinned "+ new cave" row at the bottom.
    let pin_y = rect.y + rect.h - row_h - 2;
    let pin_sel = creating;
    if pin_sel {
        gpu::fill_rect(rect.x, pin_y, rect.w, row_h, p::PANEL);
        font::draw_str(fb, screen_w, rect.x + 4, pin_y + 3, "›", p::INK, p::PANEL);
    }
    font::draw_str(
        fb, screen_w, rect.x + 18, pin_y + 3,
        "+ new cave",
        if pin_sel { p::INK } else { p::MID },
        if pin_sel { p::PANEL } else { p::BG },
    );
}
```

This relies on two helper methods on `Cave`: `is_running()` and `name_str()`. If they don't exist in `src/caves/cave.rs`, add them as inherent methods:

```rust
impl Cave {
    pub fn is_running(&self) -> bool { matches!(self.state, CaveState::Running) }
    pub fn name_str(&self) -> &str {
        let n = self.name_len as usize;
        unsafe { core::str::from_utf8_unchecked(&self.name[..n]) }
    }
}
```

Verify the actual field names (might be `state: u8` etc) and the existing convention before adding.

- [ ] **Step 2: Implement detail-view paint.**

Replace the `paint_detail_view` stub:

```rust
fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    use crate::caves::cave::{self, Sensitivity, Integrity, NetMode};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    if cave::count() == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No caves yet. Press N to create.", p::MID, p::BG);
        return;
    }

    // Resolve the selected cave by index.
    let sel = selected_cave();
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut is_running = false;
    let mut cave_id_seen: u16 = 0;
    let mut sensitivity = Sensitivity::Unclassified;
    let mut integrity = Integrity::Untrusted;
    let mut taint_value: u32 = 0;
    let mut net_mode = NetMode::Isolated;
    let mut row_index: usize = 0;
    cave::list(|c| {
        if row_index == sel {
            let n = c.name_len as usize;
            name_len = n;
            name_buf[..n].copy_from_slice(&c.name[..n]);
            is_running = c.is_running();
            cave_id_seen = c.id as u16;
            sensitivity = Sensitivity::from_u8(c.sensitivity);
            integrity = Integrity::from_u8(c.integrity);
            taint_value = c.taint;
            net_mode = NetMode::from_u8(c.net_mode);
        }
        row_index += 1;
    });
    if name_len == 0 { return; }

    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    // Name header.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, name, p::INK, p::BG);
    // State line.
    let state_str = if is_running { "RUNNING" } else { "STOPPED" };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 28, state_str, p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    // Status field list.
    let mut pid_buf = [0u8; 16];
    let pid_digits = u32_decimal(cave_id_seen as u32, &mut pid_buf, 0);
    let pid_str = unsafe { core::str::from_utf8_unchecked(&pid_buf[..pid_digits]) };

    let mut mls_buf = [0u8; 32];
    let mls_str = format_mls(&mut mls_buf, sensitivity, integrity);

    let mut taint_buf = [0u8; 16];
    let taint_str = format_hex32(&mut taint_buf, taint_value);

    let mount_str = cave::mount_of(cave_id_seen).unwrap_or("/ (derived)");

    let fields = [
        StatusField { key: "PID",    value: pid_str },
        StatusField { key: "NET",    value: net_mode.as_str() },
        StatusField { key: "MLS",    value: mls_str },
        StatusField { key: "MOUNT",  value: mount_str },
        StatusField { key: "TAINT",  value: taint_str },
        StatusField { key: "AUDIT",  value: "—" },  // Wave 4 hooks audit count
    ];
    let fields_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 60,
        w: rect.w - 28,
        h: rect.h - 110,
    };
    paint_status_field_list(fields_rect, &fields);

    // Action strip.
    let actions = [
        Action { hotkey: 'E', label: "Enter",     enabled: true },         // always available
        Action { hotkey: 'S', label: "Stop",      enabled: is_running },   // disabled on stopped caves
        Action { hotkey: 'C', label: "Configure", enabled: true },
        Action { hotkey: 'D', label: "Destroy",   enabled: true },
    ];
    let strip_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + rect.h - 28,
        w: rect.w - 28,
        h: 24,
    };
    gpu::fill_rect(strip_rect.x, strip_rect.y - 4, strip_rect.w, 1, p::HAIRLINE);
    paint_action_strip(strip_rect, &actions);
}

fn format_mls(buf: &mut [u8], sens: crate::caves::cave::Sensitivity, integ: crate::caves::cave::Integrity) -> &str {
    let mut n = 0;
    let prefix = b"sens=";
    for &b in prefix { buf[n] = b; n += 1; }
    let s_str = sens.as_str().as_bytes();
    for &b in s_str { buf[n] = b; n += 1; }
    let mid = b" integ=";
    for &b in mid { buf[n] = b; n += 1; }
    let i_str = integ_short(integ);
    for &b in i_str.as_bytes() { buf[n] = b; n += 1; }
    unsafe { core::str::from_utf8_unchecked(&buf[..n]) }
}

fn integ_short(integ: crate::caves::cave::Integrity) -> &'static str {
    match integ {
        crate::caves::cave::Integrity::Untrusted      => "Untrusted",
        crate::caves::cave::Integrity::Sandboxed      => "Sandboxed",
        crate::caves::cave::Integrity::SystemTrusted  => "SystemTrusted",
        crate::caves::cave::Integrity::HighIntegrity  => "HighIntegrity",
    }
}

fn format_hex32(buf: &mut [u8], value: u32) -> &str {
    buf[0] = b'0';
    buf[1] = b'x';
    for j in 0..8 {
        let nibble = (value >> ((7 - j) * 4)) & 0xF;
        buf[2 + j] = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
    }
    unsafe { core::str::from_utf8_unchecked(&buf[..10]) }
}
```

- [ ] **Step 3: Implement `handle_key` for Viewing mode.**

Replace the `handle_key` stub:

```rust
pub fn handle_key(c: u8) -> AppEvent {
    use crate::caves::cave;

    // Mode-specific handlers first.
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::ConfirmDestroy(idx) => {
            return handle_key_destroy_modal(c, *idx);
        }
        AppMode::Creating | AppMode::Configuring(_) => {
            return handle_key_form(c);
        }
        AppMode::Viewing => {} // fall through below
    }

    // Viewing-mode keys.
    match c {
        0x90 => {  // Arrow Up
            let sel = selected_cave();
            if sel > 0 { set_selected_cave(sel - 1); }
            AppEvent::Repaint
        }
        0x91 => {  // Arrow Down
            let sel = selected_cave();
            let cnt = cave::count();
            if cnt > 0 && sel + 1 < cnt { set_selected_cave(sel + 1); }
            AppEvent::Repaint
        }
        b'n' | b'N' => {
            // Plain N: enter Create mode.
            enter_create_mode();
            AppEvent::Repaint
        }
        b'e' | b'E' => {
            if let Some(name) = cave_name_at_selected() {
                let _ = cave::enter(&name);
            }
            AppEvent::Repaint
        }
        b's' | b'S' => {
            if let Some(name) = cave_name_at_selected() {
                let _ = cave::stop(&name);
            }
            AppEvent::Repaint
        }
        b'c' | b'C' => {
            enter_configure_mode();
            AppEvent::Repaint
        }
        b'd' | b'D' => {
            let sel = selected_cave();
            unsafe {
                core::ptr::write_volatile(
                    core::ptr::addr_of_mut!(APP_MODE),
                    AppMode::ConfirmDestroy(sel),
                );
            }
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

fn cave_name_at_selected() -> Option<alloc::string::String> {
    use alloc::string::String;
    let sel = selected_cave();
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut row_index: usize = 0;
    crate::caves::cave::list(|c| {
        if row_index == sel {
            let n = c.name_len as usize;
            name_len = n;
            name_buf[..n].copy_from_slice(&c.name[..n]);
        }
        row_index += 1;
    });
    if name_len == 0 { return None; }
    let s = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
    Some(String::from(s))
}

fn enter_create_mode() {
    let scratch = FormScratch::empty();
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FORM), Some(scratch));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Creating);
    }
}

fn enter_configure_mode() {
    let sel = selected_cave();
    let mut scratch = FormScratch::empty();
    // Pre-fill from the selected cave.
    let mut row_index: usize = 0;
    crate::caves::cave::list(|c| {
        if row_index == sel {
            let n = c.name_len as usize;
            scratch.name_buf[..n].copy_from_slice(&c.name[..n]);
            scratch.name_len = n;
            scratch.net_mode_sel = c.net_mode as usize;
            scratch.mls_sens_sel = c.sensitivity as usize;
            scratch.mls_integ_sel = c.integrity as usize;
            // mount_override / taint pre-fill, if present.
            if c.mount_override_len > 0 {
                let n = c.mount_override_len as usize;
                scratch.mount_buf[..n].copy_from_slice(&c.mount_override[..n]);
                scratch.mount_len = n;
                scratch.mount_user_dirty = true;
            }
            scratch.taint = c.taint;
        }
        row_index += 1;
    });
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FORM), Some(scratch));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Configuring(sel));
    }
}

fn handle_key_destroy_modal(_c: u8, _idx: usize) -> AppEvent {
    // Task 13 implements destroy commit / cancel.
    AppEvent::Unhandled
}

fn handle_key_form(_c: u8) -> AppEvent {
    // Task 11 / 12 implement form key dispatch.
    AppEvent::Unhandled
}
```

The `Cave` struct field names (`taint`, `name_len`, `name`) assume the existing kernel cave struct. Verify with `grep -n 'pub.*name\|pub.*name_len\|pub taint' src/caves/cave.rs` and adapt.

- [ ] **Step 4: Implement `handle_click` for Viewing mode.**

```rust
pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing => handle_click_viewing(mx, my, body),
        AppMode::Creating | AppMode::Configuring(_) => handle_click_form(mx, my, body),
        AppMode::ConfirmDestroy(_) => AppEvent::Consumed, // ignore clicks behind modal
    }
}

fn handle_click_viewing(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let sidebar = layout.sidebar_rect();
    let detail = layout.detail_rect();

    // Sidebar click — find which row was clicked.
    if mx >= sidebar.x as i32 && mx < (sidebar.x + sidebar.w) as i32 {
        let row_h: u32 = 22;
        let header_h: u32 = 28;
        let pin_y = sidebar.y + sidebar.h - row_h - 2;
        if my >= pin_y as i32 && my < (pin_y + row_h) as i32 {
            enter_create_mode();
            return AppEvent::Repaint;
        }
        if my >= (sidebar.y + header_h) as i32 {
            let row_idx = ((my as u32 - sidebar.y - header_h) / row_h) as usize;
            let cnt = crate::caves::cave::count();
            if row_idx < cnt {
                set_selected_cave(row_idx);
                return AppEvent::Repaint;
            }
        }
        return AppEvent::Consumed;
    }

    // Detail click — action strip hit-test.
    let strip_rect = WindowRect {
        x: detail.x + 14,
        y: detail.y + detail.h - 28,
        w: detail.w - 28,
        h: 24,
    };
    let actions = [
        Action { hotkey: 'E', label: "Enter",     enabled: true },
        Action { hotkey: 'S', label: "Stop",      enabled: true },
        Action { hotkey: 'C', label: "Configure", enabled: true },
        Action { hotkey: 'D', label: "Destroy",   enabled: true },
    ];
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

fn handle_click_form(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Unhandled
}
```

- [ ] **Step 5: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. In QEMU: open CAVES → sidebar lists existing caves with state dots → arrow keys move selection → detail panel updates with PID / NET / MLS / MOUNT / TAINT / AUDIT.

- [ ] **Step 6: Commit.**

```bash
git add src/ui/apps/caves_mgr.rs src/caves/cave.rs
git commit -m "$(cat <<'EOF'
caves_mgr: Viewing mode — sidebar list + detail panel + actions

Sidebar renders the cave list with state dots, selection highlight,
and the pinned `+ new cave` row. Detail panel shows the selected
cave's name + state line + status fields (PID, NET, MLS, MOUNT,
TAINT, AUDIT) + the action strip (Enter / Stop / Configure / Destroy).

Keyboard: Up/Down arrows move selection; E/S/C/D fire actions; N
enters Create mode; D opens the destroy-confirm overlay.

Pointer: sidebar rows clickable for selection; pinned row enters
Create mode; action strip tokens forward to handle_key.

Cave struct gains is_running() + name_str() helpers if they didn't
already exist.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 11: caves_mgr Creating mode

Inline editable form: NAME / NET MODE / MLS SENS / MLS INTEG / MOUNT / TAINT. Tab cycles fields; Space cycles enums; Enter submits; Esc cancels.

**Files:**
- Modify: `src/ui/apps/caves_mgr.rs`

- [ ] **Step 1: Implement `paint_detail_create`.**

Replace the stub:

```rust
fn paint_detail_create(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let form = match unsafe { &*core::ptr::addr_of!(FORM) } {
        Some(f) => f,
        None => return,
    };

    // Header.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, "New cave", p::INK, p::BG);
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 30,
                   "TAB ADVANCES · SPACE CYCLES · ENTER CREATES", p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    // Build form-field views from scratch.
    let net_values = ["isolated", "routed", "custom"];
    let sens_values = ["Unclassified", "Confidential", "Secret", "TopSecret"];
    let integ_values = ["Untrusted", "Sandboxed", "SystemTrusted", "HighIntegrity"];

    // The form mutates buf/len/selected through &mut refs, but we paint
    // off a read-only view, so we re-bind here.
    let form_scratch_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { (*form_scratch_ptr).as_mut().unwrap() };

    let fields: [FormField; 6] = [
        FormField {
            key: "NAME",
            kind: FieldKind::Text { buf: &mut f.name_buf[..], len: &mut f.name_len, max: NAME_MAX },
            readonly: false,
        },
        FormField {
            key: "NET MODE",
            kind: FieldKind::Enum { values: &net_values, selected: &mut f.net_mode_sel },
            readonly: false,
        },
        FormField {
            key: "MLS SENS",
            kind: FieldKind::Enum { values: &sens_values, selected: &mut f.mls_sens_sel },
            readonly: false,
        },
        FormField {
            key: "MLS INTEG",
            kind: FieldKind::Enum { values: &integ_values, selected: &mut f.mls_integ_sel },
            readonly: false,
        },
        FormField {
            key: "MOUNT",
            kind: FieldKind::Text { buf: &mut f.mount_buf[..], len: &mut f.mount_len, max: MOUNT_MAX },
            readonly: false,
        },
        FormField {
            key: "TAINT",
            kind: FieldKind::Hex32 { value: &mut f.taint },
            readonly: false,
        },
    ];

    let form_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 56,
        w: rect.w - 28,
        h: rect.h - 100,
    };
    paint_inline_edit_form(form_rect, &fields, f.focused_field);

    // Footer hint.
    let hint_y = rect.y + rect.h - 18;
    font::draw_str(fb, screen_w, rect.x + 14, hint_y,
                   "Enter to Create  ·  Esc to cancel", p::MID, p::BG);
}
```

- [ ] **Step 2: Implement `handle_key_form` for Creating mode.**

Replace the stub:

```rust
fn handle_key_form(c: u8) -> AppEvent {
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_mut() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};

    let net_values: [&str; 3] = ["isolated", "routed", "custom"];
    let sens_values: [&str; 4] = ["Unclassified", "Confidential", "Secret", "TopSecret"];
    let integ_values: [&str; 4] = ["Untrusted", "Sandboxed", "SystemTrusted", "HighIntegrity"];

    let mut fields: [FormField; 6] = [
        FormField {
            key: "NAME",
            kind: FieldKind::Text { buf: &mut f.name_buf[..], len: &mut f.name_len, max: NAME_MAX },
            readonly: false,
        },
        FormField {
            key: "NET MODE",
            kind: FieldKind::Enum { values: &net_values, selected: &mut f.net_mode_sel },
            readonly: false,
        },
        FormField {
            key: "MLS SENS",
            kind: FieldKind::Enum { values: &sens_values, selected: &mut f.mls_sens_sel },
            readonly: false,
        },
        FormField {
            key: "MLS INTEG",
            kind: FieldKind::Enum { values: &integ_values, selected: &mut f.mls_integ_sel },
            readonly: false,
        },
        FormField {
            key: "MOUNT",
            kind: FieldKind::Text { buf: &mut f.mount_buf[..], len: &mut f.mount_len, max: MOUNT_MAX },
            readonly: false,
        },
        FormField {
            key: "TAINT",
            kind: FieldKind::Hex32 { value: &mut f.taint },
            readonly: false,
        },
    ];

    let action = handle_form_key(&mut fields, &mut f.focused_field, c);
    match action {
        FormAction::Submit => submit_create_form(),
        FormAction::Cancel => {
            unsafe {
                core::ptr::write_volatile(core::ptr::addr_of_mut!(FORM), None);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Viewing);
            }
            AppEvent::Repaint
        }
        FormAction::None => {
            // Auto-derive MOUNT from NAME unless user has edited it.
            if !f.mount_user_dirty {
                regenerate_mount_from_name(f);
            }
            // Track whether user typed into MOUNT.
            if f.focused_field == 4 && matches!(c, b' '..=0x7E | 0x08 | 0x7F) {
                f.mount_user_dirty = true;
            }
            AppEvent::Repaint
        }
    }
}

fn regenerate_mount_from_name(f: &mut FormScratch) {
    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };
    let prefix = b"/ /home/";
    f.mount_len = 0;
    for &b in prefix { if f.mount_len < f.mount_buf.len() { f.mount_buf[f.mount_len] = b; f.mount_len += 1; } }
    for b in name.as_bytes() { if f.mount_len < f.mount_buf.len() { f.mount_buf[f.mount_len] = *b; f.mount_len += 1; } }
}

fn submit_create_form() -> AppEvent {
    use crate::caves::cave::{self, NetMode, Sensitivity, Integrity};
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_ref() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};
    if f.name_len == 0 { return AppEvent::Repaint; }  // invalid, repaint shows it

    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };

    // 1. create
    let create_res = cave::create(name, false);
    if create_res.is_err() {
        // Wave 3 limitation: errors are silently swallowed. Wave 4 adds
        // a footer-error surface; for now a failed create just leaves
        // the form open and the user can adjust + retry.
        return AppEvent::Repaint;
    }

    // 2-5. configure
    let _ = cave::set_sensitivity_by_name(name, Sensitivity::from_u8(f.mls_sens_sel as u8));
    let _ = cave::set_integrity_by_name(name, Integrity::from_u8(f.mls_integ_sel as u8));
    let _ = cave::set_net_mode_by_name(name, NetMode::from_u8(f.net_mode_sel as u8));
    if f.mount_user_dirty && f.mount_len > 0 {
        let mount = unsafe { core::str::from_utf8_unchecked(&f.mount_buf[..f.mount_len]) };
        let _ = cave::set_mount_by_name(name, mount);
    }
    if f.taint != 0 {
        // Verify the taint API per Step 0c.
        let cave_id = create_res.unwrap() as u16;
        let _ = crate::caves::taint::stamp(cave_id, f.taint);
    }

    // 6. exit Create mode.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FORM), None);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Viewing);
    }
    AppEvent::Repaint
}
```

- [ ] **Step 3: Wire `handle_click_form` to dispatch to the form widget.**

Replace the stub:

```rust
fn handle_click_form(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_mut() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};

    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let detail = layout.detail_rect();
    let form_rect = WindowRect {
        x: detail.x + 14,
        y: detail.y + 56,
        w: detail.w - 28,
        h: detail.h - 100,
    };

    let net_values: [&str; 3] = ["isolated", "routed", "custom"];
    let sens_values: [&str; 4] = ["Unclassified", "Confidential", "Secret", "TopSecret"];
    let integ_values: [&str; 4] = ["Untrusted", "Sandboxed", "SystemTrusted", "HighIntegrity"];
    let fields: [FormField; 6] = [
        FormField { key: "NAME",      kind: FieldKind::Text { buf: &mut f.name_buf[..], len: &mut f.name_len, max: NAME_MAX }, readonly: false },
        FormField { key: "NET MODE",  kind: FieldKind::Enum { values: &net_values,  selected: &mut f.net_mode_sel },          readonly: false },
        FormField { key: "MLS SENS",  kind: FieldKind::Enum { values: &sens_values, selected: &mut f.mls_sens_sel },          readonly: false },
        FormField { key: "MLS INTEG", kind: FieldKind::Enum { values: &integ_values, selected: &mut f.mls_integ_sel },        readonly: false },
        FormField { key: "MOUNT",     kind: FieldKind::Text { buf: &mut f.mount_buf[..], len: &mut f.mount_len, max: MOUNT_MAX }, readonly: false },
        FormField { key: "TAINT",     kind: FieldKind::Hex32 { value: &mut f.taint }, readonly: false },
    ];
    handle_form_click(&fields, &mut f.focused_field, form_rect, mx, my);
    AppEvent::Repaint
}
```

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. In QEMU: press N → form appears → type a name → Tab → cycle through enums with Space → Enter → new cave appears in sidebar.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/apps/caves_mgr.rs
git commit -m "$(cat <<'EOF'
caves_mgr: Creating mode — inline editable form + submit sequence

Press N (or click `+ new cave`) → right panel becomes a 6-field
form (NAME / NET MODE / MLS SENS / MLS INTEG / MOUNT / TAINT). Tab
cycles fields; Space cycles enums; Backspace edits text/hex; printable
chars fill text; Enter submits; Esc cancels.

Submit calls cave::create(name, false), then chains the per-field
setters: set_sensitivity_by_name, set_integrity_by_name,
set_net_mode_by_name, set_mount_by_name (when user-dirty), and
taint::stamp (when non-zero). Partial-failure tolerant — user can
re-Configure later.

MOUNT auto-derives from NAME until the user types into MOUNT; once
edited, NAME changes leave MOUNT alone.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 12: caves_mgr Configuring mode

Same form, pre-filled with the selected cave's values. NAME is read-only (no rename API). Submit skips `create()` and runs only the setters.

**Files:**
- Modify: `src/ui/apps/caves_mgr.rs`

- [ ] **Step 1: Implement `paint_detail_configure`.**

```rust
fn paint_detail_configure(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = match unsafe { (*form_ptr).as_mut() } {
        Some(f) => f,
        None => return,
    };

    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };
    let mut hdr_buf = [0u8; 64];
    let prefix = b"Configure ";
    let mut n = 0;
    for &b in prefix { hdr_buf[n] = b; n += 1; }
    for &b in name.as_bytes() { if n < hdr_buf.len() { hdr_buf[n] = b; n += 1; } }
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..n]) };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, hdr, p::INK, p::BG);
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 30,
                   "TAB ADVANCES · SPACE CYCLES · ENTER APPLIES", p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    let net_values: [&str; 3] = ["isolated", "routed", "custom"];
    let sens_values: [&str; 4] = ["Unclassified", "Confidential", "Secret", "TopSecret"];
    let integ_values: [&str; 4] = ["Untrusted", "Sandboxed", "SystemTrusted", "HighIntegrity"];

    let fields: [FormField; 6] = [
        // NAME is readonly: no rename API in Wave 3.
        FormField { key: "NAME",      kind: FieldKind::Text { buf: &mut f.name_buf[..], len: &mut f.name_len, max: NAME_MAX }, readonly: true },
        FormField { key: "NET MODE",  kind: FieldKind::Enum { values: &net_values,  selected: &mut f.net_mode_sel },          readonly: false },
        FormField { key: "MLS SENS",  kind: FieldKind::Enum { values: &sens_values, selected: &mut f.mls_sens_sel },          readonly: false },
        FormField { key: "MLS INTEG", kind: FieldKind::Enum { values: &integ_values, selected: &mut f.mls_integ_sel },        readonly: false },
        FormField { key: "MOUNT",     kind: FieldKind::Text { buf: &mut f.mount_buf[..], len: &mut f.mount_len, max: MOUNT_MAX }, readonly: false },
        FormField { key: "TAINT",     kind: FieldKind::Hex32 { value: &mut f.taint }, readonly: false },
    ];

    let form_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 56,
        w: rect.w - 28,
        h: rect.h - 100,
    };
    paint_inline_edit_form(form_rect, &fields, f.focused_field);

    let hint_y = rect.y + rect.h - 18;
    font::draw_str(fb, screen_w, rect.x + 14, hint_y,
                   "Enter to Apply  ·  Esc to cancel", p::MID, p::BG);
}
```

- [ ] **Step 2: Update `handle_key_form` to dispatch Submit to either `submit_create_form` or `submit_configure_form`.**

Find the match arm `FormAction::Submit => submit_create_form(),` and replace with:

```rust
        FormAction::Submit => {
            match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
                AppMode::Creating => submit_create_form(),
                AppMode::Configuring(idx) => submit_configure_form(*idx),
                _ => AppEvent::Unhandled,
            }
        }
```

- [ ] **Step 3: Add `submit_configure_form` helper.**

```rust
fn submit_configure_form(_idx: usize) -> AppEvent {
    use crate::caves::cave::{self, NetMode, Sensitivity, Integrity};
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_ref() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};
    if f.name_len == 0 { return AppEvent::Repaint; }

    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };

    let _ = cave::set_sensitivity_by_name(name, Sensitivity::from_u8(f.mls_sens_sel as u8));
    let _ = cave::set_integrity_by_name(name, Integrity::from_u8(f.mls_integ_sel as u8));
    let _ = cave::set_net_mode_by_name(name, NetMode::from_u8(f.net_mode_sel as u8));
    if f.mount_user_dirty && f.mount_len > 0 {
        let mount = unsafe { core::str::from_utf8_unchecked(&f.mount_buf[..f.mount_len]) };
        let _ = cave::set_mount_by_name(name, mount);
    }
    if let Some(cave_id) = cave::id_of_name(name) {
        let _ = crate::caves::taint::stamp(cave_id, f.taint);
    }

    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FORM), None);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Viewing);
    }
    AppEvent::Repaint
}
```

`cave::id_of_name` may not exist with that name — verify and adapt. The pattern in `set_*_by_name` resolves a name to a `cave_id` internally; either expose that helper or inline the lookup here.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. In QEMU: select a cave → press C → form appears pre-filled → change MLS sens → Enter → cave detail panel shows the new value.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/apps/caves_mgr.rs
git commit -m "$(cat <<'EOF'
caves_mgr: Configuring mode — same form, pre-filled, NAME readonly

Press C (or click [C]onfigure) on a selected cave → right panel
becomes the same 6-field form, pre-filled with the cave's current
values. NAME field is readonly (no kernel rename API yet); all other
fields editable.

Submit runs the per-field setters (skips cave::create). Cancel
reverts to View mode on the same cave.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 13: caves_mgr ConfirmDestroy modal

Press `D` → modal appears. Second `D` calls `cave::destroy`; Esc cancels.

**Files:**
- Modify: `src/ui/apps/caves_mgr.rs`

- [ ] **Step 1: Resolve the cave name dynamically in `paint`.**

Update `AppMode::ConfirmDestroy(_)` arm in `paint` to look up the cave name from the index:

```rust
        AppMode::ConfirmDestroy(idx) => {
            paint_detail_view(layout.detail_rect());

            // Resolve cave name for the modal title.
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            crate::caves::cave::list(|c| {
                if row_index == *idx {
                    let n = c.name_len as usize;
                    name_len = n;
                    name_buf[..n].copy_from_slice(&c.name[..n]);
                }
                row_index += 1;
            });
            let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

            let mut title_buf = [0u8; 32];
            let prefix = b"Destroy ";
            let mut tn = 0;
            for &b in prefix { title_buf[tn] = b; tn += 1; }
            for &b in name.as_bytes() { if tn < title_buf.len() { title_buf[tn] = b; tn += 1; } }
            if tn < title_buf.len() { title_buf[tn] = b'?'; tn += 1; }
            let title = unsafe { core::str::from_utf8_unchecked(&title_buf[..tn]) };

            let modal = ConfirmModal {
                title,
                body_lines: &[
                    "  kill all processes inside the cave",
                    "  zero the cave's encryption keys",
                    "  wipe its BatFS subtree",
                    "  clear MLS labels + taint records",
                    "",
                    "IRREVERSIBLE.",
                ],
                commit_key: 'D',
            };
            paint_confirm_modal(&modal);
        }
```

- [ ] **Step 2: Implement `handle_key_destroy_modal`.**

Replace the stub:

```rust
fn handle_key_destroy_modal(c: u8, idx: usize) -> AppEvent {
    let modal = ConfirmModal {
        title: "",
        body_lines: &[],
        commit_key: 'D',
    };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            // Look up cave name and destroy.
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            crate::caves::cave::list(|c| {
                if row_index == idx {
                    let n = c.name_len as usize;
                    name_len = n;
                    name_buf[..n].copy_from_slice(&c.name[..n]);
                }
                row_index += 1;
            });
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                let _ = crate::caves::cave::destroy(name);
            }
            // Reset selection to 0 (or empty state).
            set_selected_cave(0);
            unsafe {
                core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Viewing);
            }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe {
                core::ptr::write_volatile(core::ptr::addr_of_mut!(APP_MODE), AppMode::Viewing);
            }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}
```

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. In QEMU: select a cave → press D → modal appears → press D again → cave gone from sidebar; or Esc → modal dismisses, cave stays.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps/caves_mgr.rs
git commit -m "$(cat <<'EOF'
caves_mgr: ConfirmDestroy modal — double-tap D commits, Esc cancels

Press D (or click [D]estroy) on a selected cave → centered modal
listing the irreversible consequences. Second D press calls
cave::destroy(name); Esc cancels and returns to View mode. Click
outside the modal also cancels (handle_click consumes the click in
ConfirmDestroy mode).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 14: QEMU walk-through

Manual visual confirmation of the entire Wave 3 surface. **No commit.**

- [ ] **Step 1: Rebuild + relaunch QEMU.**

```bash
cd /Users/kadenlee/Sphragis
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
qemu-system-aarch64 \
  -machine virt -cpu max -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -device virtio-mouse-device \
  -netdev user,id=net0 \
  -device virtio-net-device,netdev=net0 \
  -serial none \
  -kernel target/aarch64-unknown-none/release/sphragis &
```

Unlock with `sphragis-dev`.

- [ ] **Step 2: Verify CAVES app launches.**

Press `1` to open CAVES. Confirm:
- Window opens with Wave-2 chrome (hairline border, drop shadow, "CAVES" title).
- Inside the window: sidebar with `CAVES (0)` header, pinned `+ new cave` row.
- Right panel: "No caves yet. Press N to create."

- [ ] **Step 3: Verify Create flow.**

Press `N` → right panel becomes the form. Type a name. Tab through fields. Space cycles enums. Enter creates. Confirm:
- Sidebar updates with a `● <name>` row.
- Right panel returns to View mode showing the new cave's PID / NET / MLS / MOUNT / TAINT / AUDIT.

- [ ] **Step 4: Verify Configure flow.**

Press `C` (or click `[C]onfigure`). Confirm:
- Form re-appears pre-filled.
- NAME field shows as readonly (no border highlight).
- Change MLS SENS via Space. Enter applies.
- View mode shows updated value.

- [ ] **Step 5: Verify Destroy flow.**

Press `D` → modal appears. Press Esc → modal dismisses, cave intact. Press `D` again → modal re-appears. Press `D` again (within the modal) → cave destroyed; sidebar shows the next cave selected (or empty state).

- [ ] **Step 6: Verify pointer parity.**

Click `+ new cave` in the sidebar → enters Create mode. Click cave rows → selection moves. Click `[E]nter` / `[C]onfigure` / `[D]estroy` tokens → equivalent to hotkey.

- [ ] **Step 7: Verify state dot rendering.**

After creating a cave: `●` filled INK in the sidebar. After pressing `S`: `○` hollow MID. After re-Enter: `●` filled again.

- [ ] **Step 8: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 9: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 15: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/caves-mgr-redesign
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

That skill verifies the build is clean, then presents merge / PR / keep / discard options. Recommended choice for this branch is "merge back to main locally" — same pattern as Waves 1 and 2.

---

## Spec coverage map (self-review)

| Spec section | Tasks |
|--------------|-------|
| §Visual system (palette + typography) | Task 3 (palette module) — all subsequent widgets import from it |
| §Layout — Inspector | Task 6 (InspectorLayout widget); Tasks 9–13 use it |
| §Sidebar | Tasks 9 + 10 |
| §Detail panel (3 modes) | Tasks 10 (View), 11 (Create), 12 (Configure), 13 (ConfirmDestroy) |
| §Action set | Task 10 (action strip wired) |
| §Cave creation form | Task 11 |
| §Configure flow | Task 12 |
| §Destroy confirmation | Task 13 |
| §State dot semantics | Task 3 (widget); Task 10 (used in sidebar) |
| §Empty state | Task 9 |
| §Pattern primitives | Tasks 3–8 (each widget) |
| §Kernel API gaps | Task 0c (resolve), Task 2 (stubs) |
| §State machine | Task 9 (enum); Tasks 10–13 (each mode) |
| §Implementation outline §1 (new widgets) | Tasks 3–8 |
| §Implementation outline §2 (replace caves_mgr) | Tasks 9–13 |
| §Implementation outline §3 (WM/desktop input dispatch) | Task 1 |
| §Implementation outline §4 (kernel-API verification) | Task 0c + Task 2 |
| §Implementation outline §5 (QEMU walk-through) | Task 14 |

## Out-of-scope reminders

Per the spec §"What's NOT in v1," none of the following land in this plan:
- Drag-to-reorder, multi-select, search, rename, process list, live audit, templates, per-cave color/icon, animations, stop-with-grace, right-click context menu.

The spec's §"Scope boundary" describes how Wave 4 picks up FILES / NET / SECURITY / EDITOR / COMMS once these widgets are merged.
