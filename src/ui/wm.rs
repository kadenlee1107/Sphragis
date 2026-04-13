// Bat_OS — Tiling Window Manager
// Keyboard-driven, no floating windows.
// Ctrl+1-5 switches apps. Ctrl+H/L splits horizontal/vertical.
// Sharp angular borders, bat aesthetic.

use crate::drivers::virtio::gpu;
use super::font;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

const BLACK: u32 = 0xFF000000;
const WHITE: u32 = 0xFFFFFFFF;
const BORDER: u32 = 0xFF1E1E1E;
const TITLE_BG: u32 = 0xFF0A0A0A;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const FG_DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const STATUS_BG: u32 = 0xFF0A0A0A;

const TITLE_H: u32 = 24;
const STATUS_H: u32 = 28;
const BORDER_W: u32 = 1;

pub const APP_SHELL: u8 = 0;
pub const APP_DASHBOARD: u8 = 1;
pub const APP_FILES: u8 = 2;
pub const APP_NETMON: u8 = 3;
pub const APP_EDITOR: u8 = 4;
pub const APP_BATCAVE: u8 = 5;

const APP_NAMES: [&str; 6] = ["Terminal", "Dashboard", "Files", "NetMonitor", "Editor", "BatCaves"];

static ACTIVE_APP: AtomicU8 = AtomicU8::new(APP_SHELL);
static NEEDS_REDRAW: AtomicBool = AtomicBool::new(true);

/// Window region (content area inside the border).
pub struct WindowRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn active_app() -> u8 {
    ACTIVE_APP.load(Ordering::Relaxed)
}

pub fn switch_app(app: u8) {
    if app < 6 {
        ACTIVE_APP.store(app, Ordering::Relaxed);
        NEEDS_REDRAW.store(true, Ordering::Relaxed);
    }
}

pub fn request_redraw() {
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Get the content area for the main window.
pub fn content_rect() -> WindowRect {
    WindowRect {
        x: BORDER_W,
        y: TITLE_H + BORDER_W,
        w: gpu::width() - BORDER_W * 2,
        h: gpu::height() - TITLE_H - STATUS_H - BORDER_W * 2,
    }
}

/// Draw the full window frame (title bar, borders, status bar).
pub fn draw_frame() {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    let app = ACTIVE_APP.load(Ordering::Relaxed) as usize;

    // Clear everything
    gpu::fill_screen(BLACK);

    // Title bar background
    gpu::fill_rect(0, 0, w, TITLE_H, TITLE_BG);

    // Bat icon + app name
    font::draw_str(fb, w, 8, 4, "\x04", WHITE, TITLE_BG); // bat char placeholder
    font::draw_str(fb, w, 8, 4, ">>", FG_DIM, TITLE_BG);
    font::draw_str(fb, w, 28, 4, APP_NAMES[app], FG_HI, TITLE_BG);

    // Tab indicators
    let tab_start = w / 2 - 100;
    for i in 0..6 {
        let tx = tab_start + (i as u32) * 48;
        let label = match i {
            0 => "1:SH",
            1 => "2:DS",
            2 => "3:FS",
            3 => "4:NM",
            4 => "5:ED",
            5 => "6:BC",
            _ => "",
        };
        if i == app {
            font::draw_str(fb, w, tx, 4, label, BLACK, WHITE);
        } else {
            font::draw_str(fb, w, tx, 4, label, FG_DIM, TITLE_BG);
        }
    }

    // Close/minimize (decorative)
    font::draw_str(fb, w, w - 40, 4, "_ X", FG_DIM, TITLE_BG);

    // Border below title
    gpu::fill_rect(0, TITLE_H, w, BORDER_W, BORDER);

    // Side borders
    gpu::fill_rect(0, TITLE_H, BORDER_W, h - TITLE_H - STATUS_H, BORDER);
    gpu::fill_rect(w - BORDER_W, TITLE_H, BORDER_W, h - TITLE_H - STATUS_H, BORDER);

    // Status bar
    let sy = h - STATUS_H;
    gpu::fill_rect(0, sy, w, STATUS_H, STATUS_BG);
    gpu::fill_rect(0, sy, w, 1, BORDER);

    draw_status_bar(fb, w, sy);
}

fn draw_status_bar(fb: *mut u32, w: u32, sy: u32) {
    let ty = sy + 6;

    // Encryption status
    gpu::fill_rect(8, ty + 2, 8, 8, GREEN);
    font::draw_str(fb, w, 20, ty, "ENCRYPTED", GREEN, STATUS_BG);

    font::draw_str(fb, w, 100, ty, "|", FG_DIM, STATUS_BG);

    // Network status
    let net_ok = crate::drivers::virtio::net::is_ready();
    let net_color = if net_ok { GREEN } else { RED };
    let net_text = if net_ok { "ONLINE" } else { "OFFLINE" };
    gpu::fill_rect(112, ty + 2, 8, 8, net_color);
    font::draw_str(fb, w, 124, ty, net_text, net_color, STATUS_BG);

    font::draw_str(fb, w, 188, ty, "|", FG_DIM, STATUS_BG);

    // Firewall
    font::draw_str(fb, w, 200, ty, "FW:DENY_ALL", FG_DIM, STATUS_BG);

    font::draw_str(fb, w, 300, ty, "|", FG_DIM, STATUS_BG);

    // Uptime
    let (mins, secs) = get_uptime();
    font::draw_str(fb, w, 312, ty, "UP:", FG_DIM, STATUS_BG);
    draw_num_at(fb, w, 336, ty, mins as usize, FG_DIM, STATUS_BG);
    font::draw_str(fb, w, 352, ty, "m", FG_DIM, STATUS_BG);

    // Ctrl+N hint
    font::draw_str(fb, w, w - 200, ty, "Ctrl+1-5: switch app", FG_DIM, STATUS_BG);
}

fn get_uptime() -> (u64, u64) {
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let secs = count / freq;
    (secs / 60, secs % 60)
}

fn draw_num_at(fb: *mut u32, sw: u32, x: u32, y: u32, n: usize, fg: u32, bg: u32) {
    let mut buf = [b' '; 8];
    let mut val = n;
    let mut i = 7;
    if val == 0 {
        buf[7] = b'0';
    } else {
        while val > 0 && i > 0 {
            buf[i] = b'0' + (val % 10) as u8;
            val /= 10;
            i -= 1;
        }
    }
    let start = i + 1;
    let s = unsafe { core::str::from_utf8_unchecked(&buf[start..]) };
    font::draw_str(fb, sw, x, y, s, fg, bg);
}

/// Flush the entire screen.
pub fn flush_all() {
    gpu::flush(0, 0, gpu::width(), gpu::height());
    NEEDS_REDRAW.store(false, Ordering::Relaxed);
}

pub fn needs_redraw() -> bool {
    NEEDS_REDRAW.load(Ordering::Relaxed)
}
