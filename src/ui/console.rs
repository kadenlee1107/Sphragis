#![allow(dead_code)]
// Bat_OS — GPU Console
// Terminal emulator rendered to the framebuffer.
// Handles text output and cursor management.

use crate::ui::gpu;
use super::font::{self, CHAR_W, CHAR_H};
use core::sync::atomic::{AtomicU32, Ordering};

// STUMP #120: palette matches the WM chrome and lock screen.
const BG: u32 = 0xFF0A0A0A;
const FG: u32 = 0xFFE5E7EB;     // INK
const FG_HI: u32 = 0xFFE5E7EB;
const FG_DIM: u32 = 0xFF4B5563;
const ACCENT_CYAN: u32 = 0xFF22D3EE;
const ACCENT_CYAN_DIM: u32 = 0xFF0E7490;
const ACCENT_GREEN: u32 = 0xFF22C55E;
const ACCENT_RED: u32 = 0xFFEF4444;

// STUMP #124 — bumped MARGIN_Y from 16 (which sat *inside* the new
// 24px title bar) to 32 (clears title bar + 8px inset) and
// STATUS_BAR_H from 32 to 28 to match the redesigned wm chrome.
const MARGIN_X: u32 = 16;
const MARGIN_Y: u32 = 32;
const STATUS_BAR_H: u32 = 28;

static CURSOR_X: AtomicU32 = AtomicU32::new(0);
static CURSOR_Y: AtomicU32 = AtomicU32::new(0);

// ─── Scrollback buffer (STUMP #124) ───────────────────────────────────
//
// 2D cell grid that mirrors the visible console region. Every char
// written via `putc` lands in BOTH the framebuffer and a Cell here, so
// `redraw_content` can replay the buffer when the WM clears the FB on
// tab switch / split / etc. Replaying chars also auto-fixes the
// cmd_buf-vs-screen consistency quirk: typed-but-not-Entered chars
// are buffer cells just like everything else, so on tab return the
// user sees their in-progress command exactly as they left it.
//
// 160 cols × 50 rows × 5 bytes/cell ≈ 40KB BSS. Rows scroll up via
// memmove of the buffer rows when content overflows.
const SB_COLS: usize = 160;
const SB_ROWS: usize = 50;

#[derive(Clone, Copy)]
struct Cell {
    ch: u8,
    fg: u32,
}
const SB_EMPTY: Cell = Cell { ch: 0, fg: 0 };

static mut SB_BUF: [[Cell; SB_COLS]; SB_ROWS] = [[SB_EMPTY; SB_COLS]; SB_ROWS];

/// Current "pen" color used by the next `putc` write. Callers swap
/// it with `set_pen()` to color sections of output (e.g. the prompt's
/// "bat_os" in INK, " > " in CYAN). Defaults to FG.
static mut PEN_COLOR: u32 = FG;

/// Set the foreground color used by subsequent `putc` calls.
pub fn set_pen(color: u32) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(PEN_COLOR), color); }
}

/// Reset pen to the default INK color. Call after a colored region.
pub fn reset_pen() { set_pen(FG); }

/// Move the cursor to an explicit (col, row). Used by `shell_banner`
/// to land the prompt below the bat-glyph banner row.
pub fn set_cursor(col: u32, row: u32) {
    CURSOR_X.store(col, Ordering::Relaxed);
    CURSOR_Y.store(row, Ordering::Relaxed);
}

/// Read the current cursor as (col, row). Callers that need to
/// restore cursor across a banner repaint use this.
pub fn cursor() -> (u32, u32) {
    (CURSOR_X.load(Ordering::Relaxed), CURSOR_Y.load(Ordering::Relaxed))
}

fn write_cell(cx: u32, cy: u32, ch: u8, fg: u32) {
    if (cx as usize) < SB_COLS && (cy as usize) < SB_ROWS {
        unsafe {
            let row_ptr = core::ptr::addr_of_mut!(SB_BUF[cy as usize][cx as usize]);
            core::ptr::write_volatile(row_ptr, Cell { ch, fg });
        }
    }
}

fn clear_cell(cx: u32, cy: u32) {
    if (cx as usize) < SB_COLS && (cy as usize) < SB_ROWS {
        unsafe {
            let row_ptr = core::ptr::addr_of_mut!(SB_BUF[cy as usize][cx as usize]);
            core::ptr::write_volatile(row_ptr, SB_EMPTY);
        }
    }
}

fn scroll_buffer_up() {
    // Shift all rows up by one; clear the bottom row.
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(SB_BUF);
        for r in 1..SB_ROWS {
            buf[r - 1] = buf[r];
        }
        for c in 0..SB_COLS {
            buf[SB_ROWS - 1][c] = SB_EMPTY;
        }
    }
}

fn cols() -> u32 {
    (gpu::width() - MARGIN_X * 2) / CHAR_W
}

fn rows() -> u32 {
    (gpu::height() - MARGIN_Y * 2 - STATUS_BAR_H) / CHAR_H
}

/// Initialize the console: clear screen, draw status bar.
pub fn init() {
    gpu::fill_screen(BG);
    draw_status_bar();
    CURSOR_X.store(0, Ordering::Relaxed);
    CURSOR_Y.store(0, Ordering::Relaxed);
    gpu::flush(0, 0, gpu::width(), gpu::height());
}

/// Initialize console within the window manager frame.
pub fn init_in_window() {
    CURSOR_X.store(0, Ordering::Relaxed);
    CURSOR_Y.store(0, Ordering::Relaxed);
}

/// V12: reset console cursor on cave switch (minor UX / read-pointer leak).
/// STUMP #124: also wipe the scrollback buffer so a logged-out cave
/// doesn't leave its command output / typed input visible to the
/// next tenant.
///
/// STUMP #147 (visual artifact fix): the buffer wipe alone left stale
/// pixels on the framebuffer because the desktop only redraws the SH
/// pane on tab-switch. When a `batcave enter <name>` ran from the
/// shell, the new (empty) console buffer rendered the cave's prompt
/// at row 0 while the bottom of the screen still showed the old
/// `bat_os >` history that was on the framebuffer before the reset.
/// Confused users into thinking input was going "to a new line above."
/// Now we explicitly fill the active pane with BG and replay the
/// (post-wipe, empty) buffer — which paints the cleared rect and
/// leaves nothing behind. Almost-always on SH at cave-switch time so
/// this fixes the SH artifact; if some other tab happens to be active
/// the wrong pane gets cleared but the next tab-switch repaints it
/// from the app's own state, so no permanent damage.
pub fn reset_for_cave_switch() {
    CURSOR_X.store(0, Ordering::Release);
    CURSOR_Y.store(0, Ordering::Release);
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(SB_BUF);
        for r in 0..SB_ROWS {
            for c in 0..SB_COLS {
                buf[r][c] = SB_EMPTY;
            }
        }
    }
    reset_pen();
    // STUMP #147: flush the SH-pane wipe to the framebuffer.
    // Chrome redraw (for the CAVE indicator) is done separately by
    // the caller AFTER `set_active(id)` runs, so the indicator
    // reads the NEW cave name not the old one. See `cave::enter`.
    redraw_content();
}

/// STUMP #124: replay the scrollback buffer to the framebuffer.
/// Called on every tab switch back to SH so the shell content
/// survives the FB clear in `wm::draw_frame`.
pub fn redraw_content() {
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let pr = crate::ui::wm::content_rect();

    // Clear the pane content rect so we paint over a clean slate.
    gpu::fill_rect(pr.x, pr.y, pr.w, pr.h, BG);

    // Replay every non-empty cell at its absolute (MARGIN_X+col*W, MARGIN_Y+row*H).
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for cy in 0..SB_ROWS {
            for cx in 0..SB_COLS {
                let cell = buf[cy][cx];
                if cell.ch == 0 { continue; }
                let px = MARGIN_X + (cx as u32) * CHAR_W;
                let py = MARGIN_Y + (cy as u32) * CHAR_H;
                if px >= pr.x && py >= pr.y
                    && px + CHAR_W <= pr.x + pr.w
                    && py + CHAR_H <= pr.y + pr.h
                {
                    font::draw_char(fb, w, px, py, cell.ch, cell.fg, BG);
                }
            }
        }
    }
}

fn draw_status_bar() {
    let w = gpu::width();
    let h = gpu::height();
    let bar_y = h - STATUS_BAR_H;
    let fb = gpu::framebuffer();

    // Bar background
    gpu::fill_rect(0, bar_y, w, STATUS_BAR_H, 0xFF0A0A0A);
    // Separator line
    gpu::fill_rect(0, bar_y, w, 1, 0xFF1E1E1E);

    // Status text
    let text_y = bar_y + 8;
    font::draw_str(fb, w, 12, text_y, "ENCRYPTED", ACCENT_GREEN, 0xFF0A0A0A);
    font::draw_str(fb, w, 108, text_y, "|", FG_DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, 120, text_y, "SECURE", ACCENT_GREEN, 0xFF0A0A0A);
    font::draw_str(fb, w, 180, text_y, "|", FG_DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, 192, text_y, "BAT_OS v0.3", FG_DIM, 0xFF0A0A0A);
}

/// Print a character to the console.
///
/// STUMP #124: every char also lands in the scrollback buffer with
/// the current pen color so `redraw_content` can replay the screen
/// after the WM clears the FB.
pub fn putc(c: u8) {
    let mut cx = CURSOR_X.load(Ordering::Relaxed);
    let mut cy = CURSOR_Y.load(Ordering::Relaxed);
    let max_cols = cols();
    let max_rows = rows();
    let pen = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEN_COLOR)) };

    match c {
        b'\n' | b'\r' => {
            cx = 0;
            cy += 1;
        }
        0x08 | 0x7F => {
            // Backspace — paint a space, clear the buffer cell.
            if cx > 0 {
                cx -= 1;
                let px = MARGIN_X + cx * CHAR_W;
                let py = MARGIN_Y + cy * CHAR_H;
                font::draw_char(gpu::framebuffer(), gpu::width(), px, py, b' ', BG, BG);
                gpu::flush(px, py, CHAR_W, CHAR_H);
                clear_cell(cx, cy);
            }
        }
        _ => {
            let px = MARGIN_X + cx * CHAR_W;
            let py = MARGIN_Y + cy * CHAR_H;
            font::draw_char(gpu::framebuffer(), gpu::width(), px, py, c, pen, BG);
            gpu::flush(px, py, CHAR_W, CHAR_H);
            // Write to scrollback. Non-printable ASCII stamps as `?`
            // (matches font::draw_char's own out-of-range fallback).
            let stored = if c >= 0x20 && c < 0x7F { c } else { b'?' };
            write_cell(cx, cy, stored, pen);
            cx += 1;
            if cx >= max_cols {
                cx = 0;
                cy += 1;
            }
        }
    }

    // QEMU-only: mirror single-char writes to PL011 so byte-granular
    // output (like `print_num(n)` which calls putc per digit) is visible
    // to test harnesses. Note: shell input-echo also writes via
    // `console::putc(c)` + `platform::serial_putc(c)` — which together
    // produce a doubled char on serial here. That's cosmetic (test
    // harnesses strip duplicates) and worth the tradeoff: without this
    // mirror, inline numbers are invisible to harnesses.
    if matches!(crate::platform::current(), crate::platform::Platform::QemuVirt) {
        crate::drivers::uart::putc(c);
    }

    // Scroll if needed.
    if cy >= max_rows {
        scroll_up();
        scroll_buffer_up();
        cy = max_rows - 1;
    }

    CURSOR_X.store(cx, Ordering::Relaxed);
    CURSOR_Y.store(cy, Ordering::Relaxed);
}

/// Mirror console writes to PL011 on QEMU so test harnesses can observe
/// shell output over serial. On Apple, `fb_console` already mirrors
/// serial→FB so we don't mirror FB→serial (would double-write).
#[inline]
fn mirror_to_serial(s: &str) {
    if matches!(crate::platform::current(), crate::platform::Platform::QemuVirt) {
        crate::drivers::uart::puts(s);
    }
}

/// Print a string to the console.
pub fn puts(s: &str) {
    for b in s.bytes() {
        putc(b);
    }
    mirror_to_serial(s);
}

/// Print a string in highlight color.
///
/// STUMP #124: route through `putc` so the cells land in the
/// scrollback buffer with the FG_HI color. Pre-fix this drew
/// directly via `font::draw_char` and bypassed the buffer, so
/// banner / prompt content vanished on tab switch.
pub fn puts_hi(s: &str) {
    set_pen(FG_HI);
    for b in s.bytes() { putc(b); }
    reset_pen();
    mirror_to_serial(s);
}

/// Print a string in an arbitrary pen color. Resets pen to FG after.
pub fn puts_color(s: &str, color: u32) {
    set_pen(color);
    for b in s.bytes() { putc(b); }
    reset_pen();
    mirror_to_serial(s);
}

/// Print the shell prompt.
///
/// STUMP #124: switched from direct `font::draw_str` calls to
/// `putc`-via-pen so the prompt cells land in the scrollback buffer.
/// Without this the prompt vanished from the screen the moment the
/// WM cleared the FB on tab switch.
pub fn prompt() {
    set_pen(FG_HI);
    for b in b"bat_os" { putc(*b); }
    set_pen(ACCENT_CYAN);
    for b in b" > " { putc(*b); }
    reset_pen();
    mirror_to_serial("bat_os > ");
}

fn scroll_up() {
    // Move framebuffer content up by one text row
    let fb = gpu::framebuffer();
    let w = gpu::width() as usize;
    let row_pixels = CHAR_H as usize;
    let start_y = MARGIN_Y as usize;
    let end_y = (gpu::height() - STATUS_BAR_H) as usize;

    unsafe {
        for y in start_y..(end_y - row_pixels) {
            let src = (y + row_pixels) * w;
            let dst = y * w;
            core::ptr::copy(fb.add(src), fb.add(dst), w);
        }
        // Clear last row
        for y in (end_y - row_pixels)..end_y {
            for x in 0..w {
                core::ptr::write_volatile(fb.add(y * w + x), BG);
            }
        }
    }
    gpu::flush(0, 0, gpu::width(), gpu::height());
}
