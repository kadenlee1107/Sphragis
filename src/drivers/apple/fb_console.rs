#![allow(dead_code)]
// Sphragis — Framebuffer console for the Apple M4 path.
//
// Sphragis on M4 currently has no way to ship serial bytes back to
// Ubuntu (m1n1's USB-CDC is gone post-chainload; our own USB-CDC
// stack is future work). So every `apple::uart::puts` call vanishes
// into the dockchannel MMIO with no reader.
//
// Until the USB-CDC class driver lands, this module mirrors every
// outgoing UART write onto the M4 display, rendered via the 8x16
// bitmap font from `ui::font`, in a console region below the boot
// splash. That gives us camera-visible kernel output: you can read
// every boot log line and `apple_serial_shell` prompt off the Mac's
// screen.
//
// Not cosmetic: bytes rendered here are the exact same bytes
// `apple::uart::puts` sends to MMIO, not a hand-crafted UI. When
// the real serial link lands, this stays as a secondary on-screen
// tty (useful without a host attached).
//
// All color constants are authored ARGB8888 and run through
// `dcp::argb8888_to_m4` at const-eval — see `M4_GROUND_TRUTH §3.1b`
// for why that's necessary.

use super::dcp::argb8888_to_m4;
use super::soc;
use crate::ui::font;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Console region on the M4 1964×3024 display: below the boot_splash
// footer (which ends around y ≈ 770), above the bottom edge.
const MARGIN_LEFT: u32 = 48;
const MARGIN_RIGHT: u32 = 48;
const REGION_TOP: u32 = 1000;
const REGION_BOTTOM: u32 = 1920;

// Scale factor for rendering glyphs — the native 8x16 font at M4's
// 3024x1964 resolution is unreadable through a 720p capture card.
// 2x (16x32) is the sweet spot for camera legibility while still
// fitting ~28 lines × 186 chars of log in the region.
const SCALE: u32 = 2;
const CELL_W: u32 = font::CHAR_W * SCALE;
const CELL_H: u32 = font::CHAR_H * SCALE;

const FG_TEXT: u32 = argb8888_to_m4(0xFFC0_C0C0);  // light gray
const FG_DIM:  u32 = argb8888_to_m4(0xFF80_8080);  // dim gray
const BG:      u32 = argb8888_to_m4(0xFF00_0000);  // black

static CURSOR_X: AtomicU32 = AtomicU32::new(MARGIN_LEFT);
static CURSOR_Y: AtomicU32 = AtomicU32::new(REGION_TOP);
static READY: AtomicBool = AtomicBool::new(false);

/// Enable the framebuffer console. Safe to call before any of the
/// other functions — they all no-op until `init` has been called.
/// Requires `soc::set_fb_info` has populated the FB base/dims.
pub fn init() {
    if soc::fb_base() == 0 || soc::fb_width() == 0 || soc::fb_stride() == 0 {
        return;
    }
    clear_region();
    CURSOR_X.store(MARGIN_LEFT, Ordering::Release);
    CURSOR_Y.store(REGION_TOP, Ordering::Release);
    READY.store(true, Ordering::Release);
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

/// Render `s` to the FB console. Honors `\n` (newline), `\r`
/// (carriage return — moves cursor to left margin without
/// advancing y), and prints any byte 0x20..=0x7E as-is. Any other
/// byte is substituted with a visible `?`. Scrolls by wiping the
/// whole region when the cursor overruns the bottom — a real
/// row-copy scroll is a nice-to-have future improvement.
pub fn puts(s: &str) {
    if !is_ready() { return; }
    let fb = soc::fb_base() as *mut u32;
    let stride_pixels = soc::fb_stride() / 4;
    let screen_w = soc::fb_width();
    if stride_pixels == 0 || screen_w == 0 { return; }

    for b in s.bytes() {
        match b {
            b'\n' => {
                CURSOR_X.store(MARGIN_LEFT, Ordering::Relaxed);
                advance_line(stride_pixels);
            }
            b'\r' => {
                CURSOR_X.store(MARGIN_LEFT, Ordering::Relaxed);
            }
            _ => {
                let ch = if (0x20..=0x7E).contains(&b) { b } else { b'?' };
                // Wrap before the right margin.
                let mut cx = CURSOR_X.load(Ordering::Relaxed);
                if cx + CELL_W + MARGIN_RIGHT > screen_w {
                    CURSOR_X.store(MARGIN_LEFT, Ordering::Relaxed);
                    advance_line(stride_pixels);
                    cx = CURSOR_X.load(Ordering::Relaxed);
                }
                let cy = CURSOR_Y.load(Ordering::Relaxed);
                font::draw_char_scaled(fb, stride_pixels, cx, cy, ch, FG_TEXT, BG, SCALE);
                CURSOR_X.store(cx + CELL_W, Ordering::Relaxed);
            }
        }
    }
}

pub fn putc(c: u8) {
    let s = [c];
    puts(unsafe { core::str::from_utf8_unchecked(&s) });
}

fn advance_line(stride_pixels: u32) {
    let new_y = CURSOR_Y.load(Ordering::Relaxed) + CELL_H;
    if new_y + CELL_H > REGION_BOTTOM {
        // Overrun — scroll the console region up by one row of text
        // (CELL_H scanlines), then clear the freshly-opened bottom
        // row. Cursor stays on the new last line.
        scroll_up_one_line(stride_pixels);
        CURSOR_Y.store(new_y - CELL_H, Ordering::Relaxed);
    } else {
        CURSOR_Y.store(new_y, Ordering::Relaxed);
    }
}

/// Copy each scanline in `REGION_TOP..REGION_BOTTOM-CELL_H` down
/// from the next `CELL_H` rows, then clear the `CELL_H` rows at the
/// bottom. Preserves all historical text below the splash.
fn scroll_up_one_line(stride_pixels: u32) {
    let base = soc::fb_base();
    if base == 0 || stride_pixels == 0 { return; }
    let screen_w = soc::fb_width();
    if screen_w == 0 { return; }

    let fb = base as *mut u32;
    let shift = CELL_H;
    // Move rows [top+shift, bottom) → [top, bottom-shift).
    for y in REGION_TOP..(REGION_BOTTOM - shift) {
        let src_off = ((y + shift) * stride_pixels) as usize;
        let dst_off = (y * stride_pixels) as usize;
        for x in 0..screen_w {
            unsafe {
                let v = core::ptr::read_volatile(fb.add(src_off + x as usize));
                core::ptr::write_volatile(fb.add(dst_off + x as usize), v);
            }
        }
    }
    // Clear the last `shift` rows (freshly opened at the bottom).
    for y in (REGION_BOTTOM - shift)..REGION_BOTTOM {
        let off = (y * stride_pixels) as usize;
        for x in 0..screen_w {
            unsafe { core::ptr::write_volatile(fb.add(off + x as usize), BG); }
        }
    }
}

fn clear_region() {
    let base = soc::fb_base();
    let stride_pixels = soc::fb_stride() / 4;
    let screen_w = soc::fb_width();
    if base == 0 || stride_pixels == 0 || screen_w == 0 { return; }
    let fb = base as *mut u32;
    for y in REGION_TOP..REGION_BOTTOM {
        let row_ptr = unsafe { fb.add((y * stride_pixels) as usize) };
        for x in 0..screen_w {
            unsafe { core::ptr::write_volatile(row_ptr.add(x as usize), BG); }
        }
    }
}
