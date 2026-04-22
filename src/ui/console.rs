#![allow(dead_code)]
// Bat_OS — GPU Console
// Terminal emulator rendered to the framebuffer.
// Handles text output and cursor management.

use crate::ui::gpu;
use super::font::{self, CHAR_W, CHAR_H};
use core::sync::atomic::{AtomicU32, Ordering};

const BG: u32 = 0xFF000000;    // Black
const FG: u32 = 0xFFA0A0A0;    // text-mid gray
const FG_HI: u32 = 0xFFFFFFFF; // White
const FG_DIM: u32 = 0xFF5A5A5A; // Dim
const ACCENT_GREEN: u32 = 0xFF00FF00;
const ACCENT_RED: u32 = 0xFF0000FF; // BGRA red

const MARGIN_X: u32 = 16;
const MARGIN_Y: u32 = 16;
const STATUS_BAR_H: u32 = 32;

static CURSOR_X: AtomicU32 = AtomicU32::new(0);
static CURSOR_Y: AtomicU32 = AtomicU32::new(0);

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
pub fn reset_for_cave_switch() {
    CURSOR_X.store(0, Ordering::Release);
    CURSOR_Y.store(0, Ordering::Release);
}

/// Redraw existing console content (placeholder — clears area).
pub fn redraw_content() {
    // In a full implementation, we'd store a scrollback buffer.
    // For now, the console content persists in the framebuffer.
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
pub fn putc(c: u8) {
    let mut cx = CURSOR_X.load(Ordering::Relaxed);
    let mut cy = CURSOR_Y.load(Ordering::Relaxed);
    let max_cols = cols();
    let max_rows = rows();

    match c {
        b'\n' | b'\r' => {
            cx = 0;
            cy += 1;
        }
        0x08 | 0x7F => {
            // Backspace
            if cx > 0 {
                cx -= 1;
                let px = MARGIN_X + cx * CHAR_W;
                let py = MARGIN_Y + cy * CHAR_H;
                font::draw_char(gpu::framebuffer(), gpu::width(), px, py, b' ', BG, BG);
                gpu::flush(px, py, CHAR_W, CHAR_H);
            }
        }
        _ => {
            let px = MARGIN_X + cx * CHAR_W;
            let py = MARGIN_Y + cy * CHAR_H;
            font::draw_char(gpu::framebuffer(), gpu::width(), px, py, c, FG, BG);
            gpu::flush(px, py, CHAR_W, CHAR_H);
            cx += 1;
            if cx >= max_cols {
                cx = 0;
                cy += 1;
            }
        }
    }

    // Scroll if needed
    if cy >= max_rows {
        scroll_up();
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
pub fn puts_hi(s: &str) {
    let cx = CURSOR_X.load(Ordering::Relaxed);
    let cy = CURSOR_Y.load(Ordering::Relaxed);
    let w = gpu::width();
    let fb = gpu::framebuffer();

    let mut x = cx;
    let mut y = cy;
    let max_cols = cols();

    for b in s.bytes() {
        if b == b'\n' || b == b'\r' {
            x = 0;
            y += 1;
        } else {
            let px = MARGIN_X + x * CHAR_W;
            let py = MARGIN_Y + y * CHAR_H;
            font::draw_char(fb, w, px, py, b, FG_HI, BG);
            x += 1;
            if x >= max_cols {
                x = 0;
                y += 1;
            }
        }
    }

    CURSOR_X.store(x, Ordering::Relaxed);
    CURSOR_Y.store(y, Ordering::Relaxed);

    // Flush the area we drew
    gpu::flush(0, MARGIN_Y, w, gpu::height() - STATUS_BAR_H);

    mirror_to_serial(s);
}

/// Print the shell prompt.
pub fn prompt() {
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let _cx = CURSOR_X.load(Ordering::Relaxed);
    let cy = CURSOR_Y.load(Ordering::Relaxed);
    let py = MARGIN_Y + cy * CHAR_H;

    font::draw_str(fb, w, MARGIN_X, py, "bat_os", FG_HI, BG);
    font::draw_str(fb, w, MARGIN_X + 6 * CHAR_W, py, " > ", FG_DIM, BG);
    CURSOR_X.store(9, Ordering::Relaxed);

    gpu::flush(MARGIN_X, py, 12 * CHAR_W, CHAR_H);

    // Echo the prompt to serial on QEMU so test harnesses have a
    // deterministic readiness marker between commands.
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
