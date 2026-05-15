# Wave 5 — EDITOR Redesign + TT Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `src/ui/apps/editor.rs` with a Wave-4-register single-buffer text editor that opens files from FILES, edits them in place, and saves back to BatFS. Bundle a tiny commit deleting the orphaned `src/ui/truetype.rs` (zero call sites since the no-browser pivot).

**Architecture:** One file does the work — `src/ui/apps/editor.rs` carries the buffer + cursor + paint + input handlers + cross-app handoff state. Custom 3-row layout (status strip, gutter+text, action strip) composed from existing Wave-3/4 widgets (`paint_state_dot`, `paint_action_strip`, `ConfirmModal`). Cross-app handoff via a `pub static`-backed `set_pending_file` / `take_pending_file` pair that FILES writes and EDITOR consumes on first paint. App switching reuses the existing `wm::iter + wm::focus + wm::open` pattern from `src/ui/desktop.rs:323`.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, alloc available via linked-list allocator). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: `cargo clippy -- -D warnings` clean + QEMU walk-through against the spec.

**Verification reality check.** Same as Waves 1–4: this crate is `#![no_std] #![no_main]`, no `lib.rs`, no test harness. `cargo test` doesn't run kernel code. Every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus a QEMU walk-through at the end. There is no unit-test step.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/editor-redesign
```
Expected: clean tree before checkout; on branch `feat/editor-redesign` after.

- [ ] **Step 0b: Confirm the current build is clean before any edits.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both finish with `Finished release profile`, zero warnings.

- [ ] **Step 0c: Verify kernel API surface (no code change — investigation only).**

The spec flagged 5 API gaps. Pre-flight confirms them — these are all known to exist per the Wave-5 brainstorm but verify with grep:

```bash
# 1. WM app-switching pattern (used in desktop.rs:323 — the editor reuses it)
grep -nE 'pub fn (open|focus|iter|focused)' src/ui/wm.rs | head -6

# 2. AppId enum variants
grep -nE 'AppId::(Files|Editor)' src/ui/apps_registry.rs | head -3

# 3. BatFS overwrite pattern
grep -nE 'pub fn ns_(create|delete|read|list|stats)' src/fs/batfs.rs | head -6

# 4. Keyboard scancodes for arrows / Enter / Backspace / Tab / Esc
grep -nE 'KEY_ARROW|0x08|0x09|0x0D|0x1B' src/drivers/virtio/keyboard.rs | head -8

# 5. Truetype delete safety (must return only the file itself + mod.rs entry)
grep -rnE 'truetype' src/ tests/ 2>&1 | grep -v -E '(comment|//|//)'
```

Expected resolutions:
- `wm::open(AppId, Option<&str>) -> Option<WindowId>`, `wm::focus(WindowId)`, `wm::iter()`, `wm::focused() -> Option<WindowId>` all exist.
- `AppId::Files` and `AppId::Editor` exist in `src/ui/apps_registry.rs`.
- `batfs::ns_create(&str, &[u8]) -> Result<(), &'static str>`, `batfs::ns_delete(&str) -> Result<(), &'static str>`, `batfs::ns_read(&str, &mut [u8]) -> Result<usize, &'static str>` all exist (Wave 4 confirmed).
- Backspace=`0x08`, Tab=`0x09`, Enter=`0x0D`, Esc=`0x1B`, Arrows=`0x90`–`0x93`, Shift+Arrows=`0x94`–`0x97`. **PgUp/PgDn/Home/End do NOT exist as constants** — Wave-5 EDITOR scrolls via arrow keys; spec's PgUp/PgDn mention is dropped here.
- `src/ui/truetype.rs` has zero non-comment references outside the file itself and the `pub mod truetype;` declaration in `src/ui/mod.rs`. Deleting both is safe.

If any of the above grep outputs differ, **stop and reconcile before proceeding** — the task code assumes these signatures.

---

## File structure

This plan creates and modifies the following files:

| File | Status | Responsibility |
|------|--------|----------------|
| `src/ui/apps/editor.rs` | **REPLACED** | Wave-5 single-buffer text editor + handoff state |
| `src/ui/apps/filemanager.rs` | **MODIFY** | Wire Enter / [E] to open selected file in EDITOR |
| `src/ui/truetype.rs` | **DELETED** | Orphaned TrueType rasterizer, no call sites |
| `src/ui/mod.rs` | **MODIFY** | Remove `pub mod truetype;` declaration |

---

## Task 1: EDITOR app rewrite

Replace `src/ui/apps/editor.rs` entirely. This is the largest task — the whole new app lands in one commit since splitting would leave a non-compiling intermediate state.

**Files:**
- Replace: `src/ui/apps/editor.rs`

- [ ] **Step 1: Overwrite `src/ui/apps/editor.rs` with the Wave-5 implementation.**

```rust
//! Wave 5 EDITOR — single-buffer text editor in the calm Wave-4 register.
//! See `docs/superpowers/specs/2026-05-15-editor-redesign-design.md`.

#![allow(dead_code, unused_imports)]

use crate::ui::apps_registry::{AppEvent, AppId};
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_state_dot,
    paint_action_strip, action_strip_hit_test, Action,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
};
use crate::ui::wm::{self, WindowRect};
use crate::fs::batfs;
use crate::drivers::virtio::keyboard::{
    KEY_ARROW_UP, KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT,
    KEY_SHIFT_ARROW_UP, KEY_SHIFT_ARROW_DOWN,
};

const NAME_MAX:     usize = 64;
const MAX_LINES:    usize = 1024;
const MAX_LINE_LEN: usize = 256;
const CHAR_W:       u32   = 8;
const CHAR_H:       u32   = 16;
const STATUS_H:     u32   = 28;
const ACTION_H:     u32   = 28;
const GUTTER_W:     u32   = 36;

// ── State ────────────────────────────────────────────────────────

#[derive(PartialEq, Eq)]
enum AppMode {
    Editing,
    ConfirmRevert,
    ConfirmDiscard,   // Esc pressed with dirty buffer
}

struct Buffer {
    lines:      [[u8; MAX_LINE_LEN]; MAX_LINES],
    line_lens:  [u16; MAX_LINES],
    line_count: usize,
}

impl Buffer {
    const fn empty() -> Self {
        Self {
            lines:      [[0u8; MAX_LINE_LEN]; MAX_LINES],
            line_lens:  [0u16; MAX_LINES],
            line_count: 0,
        }
    }
}

static mut BUFFER: Buffer = Buffer::empty();
static mut APP_MODE: AppMode = AppMode::Editing;
static mut CURSOR_ROW:     usize = 0;
static mut CURSOR_COL:     usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut DIRTY:          bool  = false;
static mut FILE_NAME:      [u8; NAME_MAX] = [0; NAME_MAX];
static mut FILE_NAME_LEN:  usize = 0;
static mut PENDING_FILE:   [u8; NAME_MAX] = [0; NAME_MAX];
static mut PENDING_LEN:    usize = 0;
static mut LOAD_ERR:       [u8; 64] = [0; 64];
static mut LOAD_ERR_LEN:   usize = 0;
static mut SAVE_ERR:       [u8; 64] = [0; 64];
static mut SAVE_ERR_LEN:   usize = 0;
static mut TRUNCATED:      bool  = false;

// ── Cross-app handoff ────────────────────────────────────────────

/// Called by FILES (or any other app) to hand a file off to EDITOR.
/// EDITOR consumes the hint on its next paint.
pub fn set_pending_file(name: &str) {
    let bytes = name.as_bytes();
    let n = bytes.len().min(NAME_MAX);
    let mut snap = n;
    while snap > 0 && !name.is_char_boundary(snap) { snap -= 1; }
    unsafe {
        let dst = core::ptr::addr_of_mut!(PENDING_FILE) as *mut u8;
        for i in 0..snap {
            core::ptr::write(dst.add(i), bytes[i]);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PENDING_LEN), snap);
    }
}

fn take_pending_file() -> Option<usize> {
    let n = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PENDING_LEN)) };
    if n == 0 { return None; }
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(PENDING_LEN), 0); }
    Some(n)
}

// ── Public app entry points ──────────────────────────────────────

pub fn paint(body: WindowRect) {
    // Consume any pending file handoff before painting.
    if let Some(n) = take_pending_file() {
        let bytes = unsafe { &*core::ptr::addr_of!(PENDING_FILE) };
        let name = unsafe { core::str::from_utf8_unchecked(&bytes[..n]) };
        load_from_batfs(name);
    }

    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    let status_rect = WindowRect { x: body.x, y: body.y, w: body.w, h: STATUS_H };
    paint_status_strip(status_rect);
    crate::ui::gpu::fill_rect(body.x, body.y + STATUS_H, body.w, 1, p::HAIRLINE);

    let action_y = body.y + body.h - ACTION_H;
    crate::ui::gpu::fill_rect(body.x, action_y - 1, body.w, 1, p::HAIRLINE);
    let edit_rect = WindowRect {
        x: body.x,
        y: body.y + STATUS_H + 1,
        w: body.w,
        h: action_y.saturating_sub(body.y + STATUS_H + 2),
    };
    paint_edit_region(edit_rect);

    let action_rect = WindowRect { x: body.x + 14, y: action_y, w: body.w - 28, h: ACTION_H };
    paint_action_strip(action_rect, &actions());

    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Editing => {}
        AppMode::ConfirmRevert => {
            paint_confirm_modal(&ConfirmModal {
                title: "Discard unsaved changes and revert?",
                body_lines: &[
                    "  reload from BatFS",
                    "  discard the current buffer",
                ],
                commit_key: 'R',
            });
        }
        AppMode::ConfirmDiscard => {
            paint_confirm_modal(&ConfirmModal {
                title: "Discard unsaved changes?",
                body_lines: &[
                    "  return to FILES",
                    "  the buffer's changes are lost",
                ],
                commit_key: 'D',
            });
        }
    }
}

// ── Painting ─────────────────────────────────────────────────────

fn paint_status_strip(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let name_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FILE_NAME_LEN)) };
    let dirty = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DIRTY)) };

    // Left: dirty-dot + filename + status caption
    paint_state_dot(rect.x + 14, rect.y + (rect.h / 2) - 3, dirty);
    if name_len > 0 {
        let name = unsafe {
            core::str::from_utf8_unchecked(&(*core::ptr::addr_of!(FILE_NAME))[..name_len])
        };
        font::draw_str(fb, screen_w, rect.x + 28, rect.y + 6, name, p::INK, p::BG);
        let after = rect.x + 28 + (name.len() as u32) * CHAR_W;
        let caption: &str = if dirty { " \xc2\xb7 modified" } else { " \xc2\xb7 saved" };
        font::draw_str(fb, screen_w, after, rect.y + 6, caption, p::MID, p::BG);
    } else {
        font::draw_str(fb, screen_w, rect.x + 28, rect.y + 6,
            "No file open \xc2\xb7 press 2 to open from FILES", p::MID, p::BG);
    }

    // Right: cursor pos + truncation banner + load/save errors
    let cur_row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let cur_col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    let truncated = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TRUNCATED)) };

    let mut buf = [0u8; 96];
    let mut n = 0;
    if truncated {
        push_bytes(&mut buf, &mut n, b"truncated to 1024 lines \xc2\xb7 ");
    }
    let save_err_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SAVE_ERR_LEN)) };
    let load_err_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(LOAD_ERR_LEN)) };
    if save_err_len > 0 {
        push_bytes(&mut buf, &mut n, b"save failed: ");
        let e = unsafe { &(*core::ptr::addr_of!(SAVE_ERR))[..save_err_len] };
        push_bytes(&mut buf, &mut n, e);
        push_bytes(&mut buf, &mut n, b" \xc2\xb7 ");
    } else if load_err_len > 0 {
        push_bytes(&mut buf, &mut n, b"load failed: ");
        let e = unsafe { &(*core::ptr::addr_of!(LOAD_ERR))[..load_err_len] };
        push_bytes(&mut buf, &mut n, e);
        push_bytes(&mut buf, &mut n, b" \xc2\xb7 ");
    }
    push_bytes(&mut buf, &mut n, b"L ");
    write_dec(&mut buf, &mut n, (cur_row + 1) as u32);
    push_bytes(&mut buf, &mut n, b" : C ");
    write_dec(&mut buf, &mut n, (cur_col + 1) as u32);

    let right = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
    let right_w = (n as u32) * CHAR_W;
    font::draw_str(
        fb, screen_w,
        rect.x + rect.w.saturating_sub(right_w + 14),
        rect.y + 6, right, p::MID, p::BG);
}

fn paint_edit_region(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let line_count = unsafe { (*core::ptr::addr_of!(BUFFER)).line_count };
    let cursor_row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let cursor_col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    let viewport = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) };

    let visible_rows = (rect.h / CHAR_H) as usize;
    let text_x = rect.x + GUTTER_W + 6;

    // Vertical divider after gutter.
    gpu::fill_rect(rect.x + GUTTER_W, rect.y, 1, rect.h, p::HAIRLINE);

    for i in 0..visible_rows {
        let line_idx = viewport + i;
        if line_idx >= line_count.max(1) { break; }
        let row_y = rect.y + (i as u32) * CHAR_H;

        // Current-line PANEL tint across gutter + text.
        if line_idx == cursor_row {
            gpu::fill_rect(rect.x, row_y, rect.w, CHAR_H, p::PANEL);
            gpu::fill_rect(rect.x + GUTTER_W, row_y, 1, CHAR_H, p::HAIRLINE);
        }
        let row_bg = if line_idx == cursor_row { p::PANEL } else { p::BG };

        // Gutter: right-aligned line number.
        let mut ln_buf = [b' '; 4];
        let mut ln = (line_idx + 1) as u32;
        let mut j = 4;
        while ln > 0 && j > 0 { j -= 1; ln_buf[j] = b'0' + (ln % 10) as u8; ln /= 10; }
        let ln_str = unsafe { core::str::from_utf8_unchecked(&ln_buf) };
        let ln_color = if line_idx == cursor_row { p::INK } else { p::FAINT };
        font::draw_str(fb, screen_w, rect.x + 2, row_y, ln_str, ln_color, row_bg);

        // Text content (if any) with comment-line tokenization.
        if line_idx < line_count {
            let line_bytes = unsafe {
                let len = (*core::ptr::addr_of!(BUFFER)).line_lens[line_idx] as usize;
                let raw = &(*core::ptr::addr_of!(BUFFER)).lines[line_idx][..len];
                raw
            };
            let line_str = unsafe { core::str::from_utf8_unchecked(line_bytes) };
            let color = if is_comment_line(line_bytes) { p::MID } else { p::INK };
            font::draw_str(fb, screen_w, text_x, row_y, line_str, color, row_bg);
        }

        // Cursor: 1-px wide INK block at the cursor column, ONLY on cursor row.
        if line_idx == cursor_row {
            let cur_x = text_x + (cursor_col as u32) * CHAR_W;
            gpu::fill_rect(cur_x, row_y, CHAR_W, CHAR_H, p::INK);
            // Re-paint the char under the cursor (if any) in BG-on-INK.
            if cursor_col < unsafe { (*core::ptr::addr_of!(BUFFER)).line_lens[line_idx] as usize } {
                let c = unsafe { (*core::ptr::addr_of!(BUFFER)).lines[line_idx][cursor_col] };
                let s = unsafe { core::str::from_utf8_unchecked(core::slice::from_ref(&c)) };
                font::draw_str(fb, screen_w, cur_x, row_y, s, p::BG, p::INK);
            }
        }
    }
}

fn is_comment_line(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() { return false; }
    let rest = &bytes[i..];
    rest.starts_with(b"//") ||
        rest.starts_with(b"#")  ||
        rest.starts_with(b";")  ||
        rest.starts_with(b"--")
}

fn actions() -> [Action<'static>; 3] {
    let dirty = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DIRTY)) };
    let has_file = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FILE_NAME_LEN)) } > 0;
    [
        Action { hotkey: 'S', label: "Save",            enabled: dirty && has_file },
        Action { hotkey: 'R', label: "Revert",          enabled: dirty && has_file },
        Action { hotkey: 'X', label: "Esc back to FILES", enabled: true },
    ]
}

// ── Input ────────────────────────────────────────────────────────

pub fn handle_key(c: u8) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::ConfirmRevert  => handle_key_modal_revert(c),
        AppMode::ConfirmDiscard => handle_key_modal_discard(c),
        AppMode::Editing        => handle_key_editing(c),
    }
}

fn handle_key_editing(c: u8) -> AppEvent {
    let has_file = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FILE_NAME_LEN)) } > 0;
    match c {
        0x1B => {
            // Esc: if dirty, ConfirmDiscard; else switch to FILES.
            if dirty() {
                unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmDiscard; }
                AppEvent::Repaint
            } else {
                switch_to_files();
                AppEvent::Repaint
            }
        }
        KEY_ARROW_LEFT  => { move_cursor_left();  AppEvent::Repaint }
        KEY_ARROW_RIGHT => { move_cursor_right(); AppEvent::Repaint }
        KEY_ARROW_UP    => { move_cursor_up(1);   AppEvent::Repaint }
        KEY_ARROW_DOWN  => { move_cursor_down(1); AppEvent::Repaint }
        KEY_SHIFT_ARROW_UP   => { move_cursor_up(8);   AppEvent::Repaint }
        KEY_SHIFT_ARROW_DOWN => { move_cursor_down(8); AppEvent::Repaint }
        0x08 => { backspace(); AppEvent::Repaint }
        0x09 => {
            for _ in 0..4 { insert_char(b' '); }
            AppEvent::Repaint
        }
        0x0D => { newline(); AppEvent::Repaint }
        b's' | b'S' if has_file && dirty() => { save_to_batfs(); AppEvent::Repaint }
        b'r' | b'R' if has_file && dirty() => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmRevert; }
            AppEvent::Repaint
        }
        // Printable ASCII inserts. Reserved keys (S/R/Esc handled above)
        // also fall through here when no file is open or buffer not dirty,
        // which intentionally lets the operator type 's' or 'r' literally.
        0x20..=0x7E => { insert_char(c); AppEvent::Repaint }
        _ => AppEvent::Unhandled,
    }
}

fn handle_key_modal_revert(c: u8) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'R' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            let name = current_file_name_owned();
            if let Some(n) = name.as_ref() {
                load_from_batfs(n.as_str());
            }
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing; }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing; }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

fn handle_key_modal_discard(c: u8) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'D' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing; }
            switch_to_files();
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing; }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    if !matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::Editing) {
        // Any click cancels modals.
        unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing; }
        return AppEvent::Repaint;
    }
    let action_rect = WindowRect {
        x: body.x + 14,
        y: body.y + body.h - ACTION_H,
        w: body.w - 28,
        h: ACTION_H,
    };
    if let Some(key) = action_strip_hit_test(action_rect, mx, my, &actions()) {
        if key == 'X' {
            // The "Esc back to FILES" entry — route to the Esc path.
            return handle_key(0x1B);
        }
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

// ── Buffer edits ─────────────────────────────────────────────────

fn dirty() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DIRTY)) }
}

fn set_dirty() {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(DIRTY), true); }
}

fn move_cursor_left() {
    let mut row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let mut col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    if col > 0 {
        col -= 1;
    } else if row > 0 {
        row -= 1;
        col = unsafe { (*core::ptr::addr_of!(BUFFER)).line_lens[row] as usize };
    }
    set_cursor(row, col);
}

fn move_cursor_right() {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    let line_len = unsafe { (*core::ptr::addr_of!(BUFFER)).line_lens[row] as usize };
    let line_count = unsafe { (*core::ptr::addr_of!(BUFFER)).line_count };
    if col < line_len {
        set_cursor(row, col + 1);
    } else if row + 1 < line_count {
        set_cursor(row + 1, 0);
    }
}

fn move_cursor_up(n: usize) {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    let new_row = row.saturating_sub(n);
    let max_col = unsafe { (*core::ptr::addr_of!(BUFFER)).line_lens[new_row] as usize };
    set_cursor(new_row, col.min(max_col));
}

fn move_cursor_down(n: usize) {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    let line_count = unsafe { (*core::ptr::addr_of!(BUFFER)).line_count };
    let new_row = (row + n).min(line_count.saturating_sub(1));
    let max_col = unsafe { (*core::ptr::addr_of!(BUFFER)).line_lens[new_row] as usize };
    set_cursor(new_row, col.min(max_col));
}

fn set_cursor(row: usize, col: usize) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_ROW), row);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_COL), col);
    }
    scroll_viewport_to_cursor();
}

fn scroll_viewport_to_cursor() {
    // The visible-rows count depends on the body height which we don't
    // have here; paint() will adjust on the next paint cycle. We just
    // make sure VIEWPORT_START <= CURSOR_ROW so the cursor isn't
    // above the viewport.
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let vp = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) };
    if row < vp {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), row); }
    } else if row >= vp + 24 {
        // Conservative bottom edge — paint may have more rows visible;
        // worst case the viewport jumps but the cursor stays in view.
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), row - 23); }
    }
}

fn insert_char(c: u8) {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
        if buf.line_count == 0 { buf.line_count = 1; }
        let line_len = buf.line_lens[row] as usize;
        if line_len >= MAX_LINE_LEN { return; }
        // Shift right.
        for j in (col..line_len).rev() {
            buf.lines[row][j + 1] = buf.lines[row][j];
        }
        buf.lines[row][col] = c;
        buf.line_lens[row] = (line_len + 1) as u16;
    }
    set_cursor(row, col + 1);
    set_dirty();
}

fn backspace() {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    if col > 0 {
        unsafe {
            let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
            let line_len = buf.line_lens[row] as usize;
            for j in col..line_len {
                buf.lines[row][j - 1] = buf.lines[row][j];
            }
            buf.line_lens[row] = (line_len - 1) as u16;
        }
        set_cursor(row, col - 1);
        set_dirty();
    } else if row > 0 {
        // Join with previous line — cursor ends up at the original
        // length of the previous line (the join point).
        let prev_len_before = unsafe {
            (*core::ptr::addr_of!(BUFFER)).line_lens[row - 1] as usize
        };
        unsafe {
            let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
            let prev_len = buf.line_lens[row - 1] as usize;
            let cur_len  = buf.line_lens[row] as usize;
            let combined = prev_len + cur_len;
            if combined > MAX_LINE_LEN { return; }
            for j in 0..cur_len {
                buf.lines[row - 1][prev_len + j] = buf.lines[row][j];
            }
            buf.line_lens[row - 1] = combined as u16;
            for r in row..buf.line_count.saturating_sub(1) {
                buf.lines[r]     = buf.lines[r + 1];
                buf.line_lens[r] = buf.line_lens[r + 1];
            }
            buf.line_count = buf.line_count.saturating_sub(1);
        }
        set_cursor(row - 1, prev_len_before);
        set_dirty();
    }
}

fn newline() {
    let row = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)) };
    let col = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)) };
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
        if buf.line_count >= MAX_LINES { return; }
        if buf.line_count == 0 { buf.line_count = 1; }
        // Shift lines down to make room.
        for r in (row + 1..=buf.line_count).rev() {
            buf.lines[r]     = buf.lines[r - 1];
            buf.line_lens[r] = buf.line_lens[r - 1];
        }
        // Split current line at col.
        let cur_len = buf.line_lens[row] as usize;
        let tail_len = cur_len.saturating_sub(col);
        for j in 0..tail_len {
            buf.lines[row + 1][j] = buf.lines[row][col + j];
        }
        buf.line_lens[row + 1] = tail_len as u16;
        buf.line_lens[row]     = col as u16;
        buf.line_count += 1;
    }
    set_cursor(row + 1, 0);
    set_dirty();
}

// ── BatFS I/O ────────────────────────────────────────────────────

fn load_from_batfs(name: &str) {
    // Reset state.
    unsafe {
        *core::ptr::addr_of_mut!(BUFFER) = Buffer::empty();
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_ROW), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_COL), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DIRTY), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(TRUNCATED), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LOAD_ERR_LEN), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SAVE_ERR_LEN), 0);
    }
    // Save current file name.
    let bytes = name.as_bytes();
    let n = bytes.len().min(NAME_MAX);
    unsafe {
        let dst = core::ptr::addr_of_mut!(FILE_NAME) as *mut u8;
        for i in 0..n {
            core::ptr::write(dst.add(i), bytes[i]);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FILE_NAME_LEN), n);
    }
    // Read into a temp staging buffer, then split on \n into lines.
    let mut staging = [0u8; MAX_LINES * MAX_LINE_LEN];
    let bytes_read = match batfs::ns_read(name, &mut staging) {
        Ok(n) => n,
        Err(e) => {
            store_load_err(e.as_bytes());
            unsafe {
                let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
                buf.line_count = 1;
                buf.line_lens[0] = 0;
            }
            return;
        }
    };
    parse_into_lines(&staging[..bytes_read]);
}

fn parse_into_lines(bytes: &[u8]) {
    let mut row = 0usize;
    let mut col = 0usize;
    let mut truncated = false;
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(BUFFER);
        buf.line_lens[0] = 0;
        for &b in bytes {
            if b == b'\n' {
                buf.line_lens[row] = col as u16;
                row += 1;
                col = 0;
                if row >= MAX_LINES {
                    truncated = true;
                    row = MAX_LINES - 1;
                    break;
                }
                buf.line_lens[row] = 0;
            } else if b == b'\r' {
                // Strip CR (treat CRLF as LF).
            } else if col < MAX_LINE_LEN {
                buf.lines[row][col] = b;
                col += 1;
            } else {
                // Line too long — silently clip the tail of this line.
            }
        }
        buf.line_lens[row] = col as u16;
        buf.line_count = row + 1;
    }
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(TRUNCATED), truncated); }
}

fn save_to_batfs() {
    let name_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FILE_NAME_LEN)) };
    if name_len == 0 { return; }
    let name_bytes = unsafe { &(*core::ptr::addr_of!(FILE_NAME))[..name_len] };
    let name = unsafe { core::str::from_utf8_unchecked(name_bytes) };

    // Serialize buffer into one contiguous byte stream (lines joined by \n).
    let mut tmp = [0u8; MAX_LINES * MAX_LINE_LEN];
    let mut n = 0;
    unsafe {
        let buf = &*core::ptr::addr_of!(BUFFER);
        for r in 0..buf.line_count {
            let line_len = buf.line_lens[r] as usize;
            if n + line_len >= tmp.len() { break; }
            tmp[n..n + line_len].copy_from_slice(&buf.lines[r][..line_len]);
            n += line_len;
            if r + 1 < buf.line_count {
                tmp[n] = b'\n';
                n += 1;
            }
        }
    }

    // Overwrite via delete + create (matches shell.rs's `write` command).
    let _ = batfs::ns_delete(name);
    match batfs::ns_create(name, &tmp[..n]) {
        Ok(()) => {
            unsafe {
                core::ptr::write_volatile(core::ptr::addr_of_mut!(DIRTY), false);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(SAVE_ERR_LEN), 0);
            }
        }
        Err(e) => store_save_err(e.as_bytes()),
    }
}

fn store_load_err(bytes: &[u8]) {
    let n = bytes.len().min(64);
    unsafe {
        let dst = core::ptr::addr_of_mut!(LOAD_ERR) as *mut u8;
        for i in 0..n { core::ptr::write(dst.add(i), bytes[i]); }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LOAD_ERR_LEN), n);
    }
}

fn store_save_err(bytes: &[u8]) {
    let n = bytes.len().min(64);
    unsafe {
        let dst = core::ptr::addr_of_mut!(SAVE_ERR) as *mut u8;
        for i in 0..n { core::ptr::write(dst.add(i), bytes[i]); }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SAVE_ERR_LEN), n);
    }
}

fn current_file_name_owned() -> Option<alloc::string::String> {
    let n = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(FILE_NAME_LEN)) };
    if n == 0 { return None; }
    let bytes = unsafe { &(*core::ptr::addr_of!(FILE_NAME))[..n] };
    let s = unsafe { core::str::from_utf8_unchecked(bytes) };
    Some(alloc::string::String::from(s))
}

// ── Cross-app switch ─────────────────────────────────────────────

fn switch_to_files() {
    let existing = wm::iter().find(|w| w.app == AppId::Files).map(|w| w.id);
    match existing {
        Some(id) => wm::focus(id),
        None     => { wm::open(AppId::Files, None); }
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return; }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
}

extern crate alloc;
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -5
```

Expected: clean. If clippy flags any of the following, fix in place per existing Wave-4 patterns:
- `clippy::needless_range_loop` — restructure with `.iter().enumerate()` or add `#[allow]`.
- `clippy::manual_div_ceil` — `Cargo.toml` already allows this.
- Dead-code on `Buffer::empty`, `take_pending_file`, etc. — the file's `#![allow(dead_code)]` should cover, but if not, add `#[allow(dead_code)]` per-item.

- [ ] **Step 3: Commit.**

```bash
git add src/ui/apps/editor.rs
git commit -m "$(cat <<'EOF'
editor: Wave 5 — single-buffer text editor in calm Wave-4 register

Replaces the 826-line legacy cyberpunk editor. New shape:
* 1024-line × 256-col buffer with cursor + arrow nav (Shift+↑/↓ for
  8-line jumps). Insert / Backspace / Enter / Tab(=4 spaces).
* Custom 3-row layout — status strip (filename + dirty-dot + cursor
  pos) on top, gutter+text region in the middle, action strip on
  bottom. Composed from Wave-3/4 widgets (paint_state_dot,
  paint_action_strip, ConfirmModal).
* Light comment-line awareness — lines starting with //, #, ;, --
  render in MID; everything else in INK. No per-token parsing, no
  extension sniffing.
* Cross-app handoff via pub fn set_pending_file(name) — FILES calls
  this then switches the active app to EDITOR; EDITOR consumes the
  hint on first paint and loads via batfs::ns_read.
* Save via batfs::ns_delete + batfs::ns_create (matches the existing
  shell.rs `write` overwrite pattern).
* Revert + Esc-with-dirty both protected by ConfirmModal.
* Files > 1024 lines paint a "truncated to 1024 lines" banner so
  the operator knows save would overwrite with the truncated content.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2: Wire FILES → EDITOR open flow

Replace the Wave-4 FAINT `[E]dit (W5)` stub in FILES with a real handoff to EDITOR. Also bind Enter on the selected file.

**Files:**
- Modify: `src/ui/apps/filemanager.rs` — lines 213-219, ~269

- [ ] **Step 1: Add the open-in-editor helper near the bottom of the file (after `handle_click_viewing`).**

Append this function to `src/ui/apps/filemanager.rs` just before the `// ── helpers ───` section:

```rust
fn open_selected_in_editor() -> AppEvent {
    let (count, _) = batfs::ns_stats();
    if count == 0 { return AppEvent::Consumed; }

    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let sel = selected_file();
    let mut row_index: usize = 0;
    batfs::ns_list(|n, _, _| {
        if row_index == sel {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
        }
        row_index += 1;
    });
    if name_len == 0 { return AppEvent::Consumed; }
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    crate::ui::apps::editor::set_pending_file(name);
    let existing = crate::ui::wm::iter()
        .find(|w| w.app == crate::ui::apps_registry::AppId::Editor)
        .map(|w| w.id);
    match existing {
        Some(id) => crate::ui::wm::focus(id),
        None     => { crate::ui::wm::open(crate::ui::apps_registry::AppId::Editor, None); }
    }
    AppEvent::Repaint
}
```

- [ ] **Step 2: Update the `actions_for_file` function (around line 213-219) to enable the E action.**

Replace:

```rust
fn actions_for_file(_encrypted: bool) -> [Action<'static>; 2] {
    [
        Action { hotkey: 'D', label: "Delete",     enabled: true  },
        Action { hotkey: 'E', label: "Edit (W5)",  enabled: false },
    ]
}
```

With:

```rust
fn actions_for_file(_encrypted: bool) -> [Action<'static>; 2] {
    [
        Action { hotkey: 'D', label: "Delete", enabled: true },
        Action { hotkey: 'E', label: "Edit",   enabled: true },
    ]
}
```

- [ ] **Step 3: Update `handle_key_viewing` (around line 269) — the `b'e' | b'E'` arm and a new Enter arm.**

Find this block:

```rust
        b'e' | b'E' => AppEvent::Consumed,
```

Replace with:

```rust
        b'e' | b'E' => open_selected_in_editor(),
        0x0D        => open_selected_in_editor(),  // Enter
```

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -5
```

- [ ] **Step 5: Commit.**

```bash
git add src/ui/apps/filemanager.rs
git commit -m "$(cat <<'EOF'
filemanager: wire Enter / [E] to open selected file in EDITOR

Wave 4 left [E]dit as a FAINT (W5) stub; this lights it up. Pressing
Enter or [E] on a selected file (or clicking the [E]dit action strip
button) calls editor::set_pending_file(name) and switches the active
app to EDITOR via the existing wm::iter/focus/open pattern.

EDITOR consumes the hint on its next paint and loads the file via
batfs::ns_read. Esc-back-to-FILES (handled in EDITOR) returns focus
with the prior selection preserved.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: Delete orphaned TrueType rasterizer

`src/ui/truetype.rs` is 1327 lines with `#![allow(dead_code)]` at the top. It was built during the in-tree-browser era; the no-browser pivot (2026-05-08) orphaned it. Brainstorm confirmed zero non-comment references in the tree outside the file itself and its `mod` declaration.

**Files:**
- Delete: `src/ui/truetype.rs`
- Modify: `src/ui/mod.rs` — remove `pub mod truetype;`

- [ ] **Step 1: Confirm safety with one more grep.**

```bash
grep -rnE 'crate::ui::truetype|use.*truetype|truetype::|TrueTypeFont|TtFont' src/ tests/ 2>&1 | grep -v -E '^src/ui/truetype\.rs:|^src/ui/mod\.rs:'
```

Expected: zero hits. The references in `src/ui/draw.rs:85` and `src/security/boot_screen.rs` are all in `//` comments documenting historical context; the grep filter above excludes the truetype.rs file itself and the mod declaration. If any hit appears in actual code (not comments), **stop and document** — the deletion is not safe.

- [ ] **Step 2: Delete the file.**

```bash
git rm src/ui/truetype.rs
```

- [ ] **Step 3: Remove the module declaration from `src/ui/mod.rs`.**

Find the line `pub mod truetype;` (around line 16 per the brainstorm) and delete it. Use `grep -n 'pub mod truetype' src/ui/mod.rs` to find the exact line first, then use the Edit tool to remove just that one line.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -5
```

Expected: clean. If anything in the tree was secretly depending on truetype.rs (e.g. via a re-export we missed), the build fails here with a clear "module not found" error and we can put it back.

- [ ] **Step 5: Commit.**

```bash
git add -A src/ui/truetype.rs src/ui/mod.rs
git commit -m "$(cat <<'EOF'
ui: delete orphaned TrueType rasterizer (1327 lines, dead since no-browser pivot)

src/ui/truetype.rs was built during the in-tree browser era (pre
2026-05-08 no-browser pivot). It has #![allow(dead_code)] at the top
and zero non-comment call sites in the tree — confirmed by grep
during Wave 5 pre-flight.

The 8×16 bitmap font (src/ui/font.rs) covers every UI text need; the
proper kernel-level boot screen rationale lives in
src/security/boot_screen.rs comments which mention truetype only
historically. Nothing breaks.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 4: QEMU walk-through

Manual visual confirmation. **No commit.**

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

- [ ] **Step 2: Verify EDITOR empty state.**

Press `6` to open EDITOR (no file loaded yet). Confirm:
- Status strip shows hollow MID dot + "No file open · press 2 to open from FILES" in MID.
- Edit region is empty (no lines).
- Action strip shows [S]ave (FAINT), [R]evert (FAINT), [Esc] back to FILES (INK).

- [ ] **Step 3: Verify FILES → EDITOR open flow.**

Press `2` to open FILES. Use J/K to move selection to an existing file (need at least one file in BatFS first — if there's no file, press Ctrl+D to close FILES, press `5` for SHELL, type `write notes.txt "## Test\nhello world\n// a comment line\n"`, then `2` to reopen FILES).

Press Enter on the selected file. Confirm:
- EDITOR window comes to focus (or opens if it wasn't open yet).
- Status strip shows hollow MID dot + filename + " · saved".
- Edit region shows the file content; comment lines (those starting with `//`, `#`, `;`, `--`) render in MID, the rest in INK.
- Cursor is at line 1, col 1, current-line PANEL tint visible.

Press Esc, then press FILES `2`, select the same file, click the `[E]dit` button in the action strip. Confirm same behavior.

- [ ] **Step 4: Verify editing + dirty marker.**

In EDITOR with a file open:
- Type a few printable chars (e.g. `abc`). Confirm:
  - Status dirty marker turns filled INK + " · modified" caption appears.
  - Cursor advances; chars appear.
- Press Backspace. Confirm char deletes, cursor moves left.
- Press Enter. Confirm line splits; cursor moves to col 1 of the new line.
- Move cursor to a comment line (// or #). Confirm it still renders in MID.
- Type a character on a comment line. Confirm the line stays MID (the marker is still present).

- [ ] **Step 5: Verify Save.**

Press S. Confirm:
- Dirty dot turns back to hollow MID + " · saved" caption.
- Status strip shows no error.

Press Esc to go back to FILES. The selection should be preserved (same file highlighted).

Reopen the file. Confirm the edits persisted (the file's content reflects what you typed and saved).

- [ ] **Step 6: Verify Revert.**

Open the file again. Type a few chars (dirty). Press R. Confirm:
- ConfirmModal appears with "Discard unsaved changes and revert?".
- Press R again to commit. Buffer reloads from BatFS; dirty marker clears; cursor at L 1 C 1.

Press R again without dirty changes. Confirm nothing happens (the action is FAINT-gated).

- [ ] **Step 7: Verify Esc-with-dirty discard prompt.**

Type a few chars (dirty). Press Esc. Confirm:
- ConfirmModal appears with "Discard unsaved changes?".
- Press D to commit → returns to FILES with selection preserved; the buffer's changes are gone.

Repeat: type chars, press Esc, Esc again (cancel) → returns to EDITOR with the chars still there and dirty marker still set.

- [ ] **Step 8: Verify cross-app keyboard parity.**

- `1` opens CAVES — unchanged.
- `2` opens FILES — unchanged.
- `3`, `4` open NET, SECURITY — unchanged (Wave 4 work).
- Tab cycles focus.
- Ctrl+L returns to lock screen; passphrase re-unlocks; workspace persists.

- [ ] **Step 9: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 10: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 5: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/editor-redesign
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

Same as Waves 1/2/3/4. Recommended choice: "Merge back to main locally" — full pattern: checkout main → --no-ff merge → verify build/clippy → delete local branch → push origin main → delete origin's feature branch → journal entry.

---

## Spec coverage map (self-review)

| Spec section | Task |
|--------------|------|
| §Scope — In: rewrite editor.rs to Wave-4 register | Task 1 |
| §Scope — In: 1024×256 buffer | Task 1 (constants `MAX_LINES`, `MAX_LINE_LEN`) |
| §Scope — In: cursor nav + insert / delete / Enter / Backspace | Task 1 (`handle_key_editing`, `move_cursor_*`, `insert_char`, `backspace`, `newline`) |
| §Scope — In: scrolling viewport | Task 1 (`scroll_viewport_to_cursor` + paint's viewport check) |
| §Scope — In: comment-line tokenization | Task 1 (`is_comment_line`) |
| §Scope — In: open flow via FILES Enter / [E] | Task 2 (`open_selected_in_editor` + key arms) |
| §Scope — In: save flow | Task 1 (`save_to_batfs`) |
| §Scope — In: revert flow + ConfirmModal | Task 1 (`AppMode::ConfirmRevert` + `handle_key_modal_revert`) |
| §Scope — In: Esc with dirty prompt | Task 1 (`AppMode::ConfirmDiscard` + `handle_key_modal_discard`) |
| §Scope — In: empty state | Task 1 (paint_status_strip "No file open" branch) |
| §Scope — In: delete truetype.rs + mod.rs entry | Task 3 |
| §Visual system | Task 1 (uses `palette as p`; no new constants) |
| §Layout — 3-row | Task 1 (status strip + edit region + action strip) |
| §Status strip | Task 1 (`paint_status_strip`) |
| §Gutter + text | Task 1 (`paint_edit_region`) |
| §Action strip | Task 1 (`actions()` + `paint_action_strip`) |
| §Cross-app handoff — PENDING_FILE slot | Task 1 (`set_pending_file` / `take_pending_file`) |
| §Cross-app handoff — FILES changes | Task 2 |
| §Buffer + edit semantics — key bindings table | Task 1 (`handle_key_editing`) |
| §Open API gaps — all 5 | Pre-flight 0c |
| §Failure modes — load fails / save fails / truncated | Task 1 (`store_load_err`, `store_save_err`, `TRUNCATED` banner) |
| §Reuse from Wave 4 — paint_state_dot / paint_action_strip / ConfirmModal | Task 1 imports |
| §Out-of-scope — selection / multi-tab / search / undo / new-file | Not implemented (correct per spec) |

No gaps.

## Out-of-scope reminders

Do NOT implement in this plan:
- Selection / copy / paste / clipboard
- Search / replace
- Multi-buffer or visual tabs (the legacy 3-tab visual is gone — single buffer only)
- Per-language syntax highlighting beyond comment-line awareness
- Undo / redo
- New-file creation in EDITOR (operator uses SHELL `write`)
- "Save as" rename
- Indentation-aware Tab (just 4 spaces)
- Horizontal scroll for long lines (they clip at the right edge)
- PgUp / PgDn / Home / End key bindings (constants don't exist in `virtio::keyboard`; ↑/↓ + Shift+↑/↓ cover the cases)
