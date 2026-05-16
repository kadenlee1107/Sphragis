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
        load_file(name);
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
            let arr = &*core::ptr::addr_of!(FILE_NAME);
            core::str::from_utf8_unchecked(&arr[..name_len])
        };
        font::draw_str(fb, screen_w, rect.x + 28, rect.y + 6, name, p::INK, p::BG);
        let after = rect.x + 28 + (name.len() as u32) * CHAR_W;
        let caption: &str = if dirty { " \u{00b7} modified" } else { " \u{00b7} saved" };
        font::draw_str(fb, screen_w, after, rect.y + 6, caption, p::MID, p::BG);
    } else {
        font::draw_str(fb, screen_w, rect.x + 28, rect.y + 6,
            "No file open \u{00b7} press 2 to open from FILES", p::MID, p::BG);
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
        let e = unsafe { let arr = &*core::ptr::addr_of!(SAVE_ERR); &arr[..save_err_len] };
        push_bytes(&mut buf, &mut n, e);
        push_bytes(&mut buf, &mut n, b" \xc2\xb7 ");
    } else if load_err_len > 0 {
        push_bytes(&mut buf, &mut n, b"load failed: ");
        let e = unsafe { let arr = &*core::ptr::addr_of!(LOAD_ERR); &arr[..load_err_len] };
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
                let b = &*core::ptr::addr_of!(BUFFER);
                &b.lines[line_idx][..b.line_lens[line_idx] as usize]
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
        Action { hotkey: 'S', label: "^S Save",            enabled: dirty && has_file },
        Action { hotkey: 'R', label: "^R Revert",          enabled: dirty && has_file },
        Action { hotkey: 'X', label: "Esc back to FILES",  enabled: true },
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

    // Empty-state: no file loaded. Only Esc switches back to FILES; every
    // other key falls through so the desktop launcher 1..8 still works.
    // Without this, the "press 2 to open from FILES" hint is a lie because
    // EDITOR's Editing mode would otherwise eat the digit as text input.
    if !has_file {
        return match c {
            0x1B => { switch_to_files(); AppEvent::Repaint }
            _    => AppEvent::Unhandled,
        };
    }

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
        // Ctrl+S / Ctrl+R for save / revert. Unmodified s/S/r/R fall
        // through to the printable-ASCII arm below so the operator can
        // type those letters into the buffer without triggering the
        // action. (Wave 6 fix: unmodified hotkeys collided with text
        // input.)
        0x13 if dirty() => { save_to_batfs(); AppEvent::Repaint }
        0x12 if dirty() => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmRevert; }
            AppEvent::Repaint
        }
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
                load_file(n.as_str());
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
        let byte = match key {
            'X' => 0x1B,  // Esc
            'S' => 0x13,  // Ctrl+S
            'R' => 0x12,  // Ctrl+R
            other => other as u8,
        };
        return handle_key(byte);
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

/// Public shim for the shell's `edit <filename>` command (pre-Wave-5
/// call site in shell.rs). Returns Ok on load or Err with a static
/// message so the shell can print it. The status-strip error slot is
/// also populated on failure.
pub fn load_from_batfs(name: &str) -> Result<(), &'static str> {
    load_file(name);
    // Check whether the load populated an error.
    let err_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(LOAD_ERR_LEN)) };
    if err_len > 0 { Err("load failed") } else { Ok(()) }
}

fn load_file(name: &str) {
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
    let name_bytes = unsafe { let arr = &*core::ptr::addr_of!(FILE_NAME); &arr[..name_len] };
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
    let bytes = unsafe { let arr = &*core::ptr::addr_of!(FILE_NAME); &arr[..n] };
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

/// AUDIT-DRV-C1 (2026-05-15): zero editor state on cave switch so the
/// previous cave's file contents (up to 256 KB of plaintext in
/// `BUFFER`) don't leak to the new cave. Also clears file-name
/// scratch, cursor, viewport, dirty flag, pending save/open names.
/// Held under IrqGuard so the timer IRQ can't observe half-cleared
/// state.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        *core::ptr::addr_of_mut!(BUFFER) = Buffer::empty();
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Editing;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_ROW), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_COL), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DIRTY), false);
        *core::ptr::addr_of_mut!(FILE_NAME) = [0u8; NAME_MAX];
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FILE_NAME_LEN), 0);
        *core::ptr::addr_of_mut!(PENDING_FILE) = [0u8; NAME_MAX];
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PENDING_LEN), 0);
    }
}
