#![allow(dead_code)]
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
pub const APP_SECURITY: u8 = 5;
pub const APP_COMMS: u8 = 6;
pub const APP_BROWSER: u8 = 7;
pub const APP_BATCAVE: u8 = 8;

const NUM_APPS: u8 = 9;
const APP_NAMES: [&str; 9] = ["Term", "Dash", "File", "Net", "Edit", "Sec", "Chat", "Web", "Cave"];

static NEEDS_REDRAW: AtomicBool = AtomicBool::new(true);

// ─── Multi-pane system (up to 4 panes) ───
// Each pane has: app_id, position (row, col in a grid)
// Layout is defined by LAYOUT_ROWS × LAYOUT_COLS grid
// Examples:
//   1×1 = single pane       2×1 = 2 horizontal stacked
//   1×2 = 2 vertical side   2×2 = 4 quad grid
//   1×3 = 3 vertical cols   1×4 = 4 vertical cols
//   4×1 = 4 horizontal rows

const MAX_PANES: usize = 4;

#[derive(Clone, Copy)]
struct Pane {
    active: bool,
    app: u8,
    // Grid position (row, col) and span
    row: u8,
    col: u8,
    row_span: u8, // how many rows this pane occupies
    col_span: u8, // how many cols this pane occupies
}

impl Pane {
    const fn empty() -> Self {
        Pane { active: false, app: 0, row: 0, col: 0, row_span: 1, col_span: 1 }
    }
}

static mut PANES: [Pane; MAX_PANES] = [Pane::empty(); MAX_PANES];
static mut PANE_COUNT: u8 = 1;
static mut FOCUSED_PANE: u8 = 0;
static mut LAYOUT_ROWS: u8 = 1;
static mut LAYOUT_COLS: u8 = 1;
static mut RENDER_TARGET: u8 = 0;

// Legacy compat
static ACTIVE_APP: AtomicU8 = AtomicU8::new(APP_SHELL);

pub fn init_panes_pub() { init_panes(); }

fn init_panes() {
    unsafe {
        PANES[0] = Pane { active: true, app: APP_SHELL, row: 0, col: 0, row_span: 1, col_span: 1 };
        PANE_COUNT = 1;
        FOCUSED_PANE = 0;
        LAYOUT_ROWS = 1;
        LAYOUT_COLS = 1;
    }
}

/// Window region (content area inside the border).
pub struct WindowRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn active_app() -> u8 {
    unsafe { PANES[FOCUSED_PANE as usize].app }
}

pub fn switch_app(app: u8) {
    if app < NUM_APPS {
        unsafe { PANES[FOCUSED_PANE as usize].app = app; }
        ACTIVE_APP.store(app, Ordering::Relaxed);
        NEEDS_REDRAW.store(true, Ordering::Relaxed);
    }
}

pub fn request_redraw() {
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Add a pane. Always uses 2x2 grid layout.
/// 1 pane = full, 2 = top-left + top-right, 3 = +bottom-left, 4 = +bottom-right
pub fn split_vertical() {
    add_pane();
}

pub fn split_horizontal() {
    add_pane();
}

fn add_pane() {
    unsafe {
        if PANE_COUNT >= 4 { return; }
        let idx = PANE_COUNT as usize;
        let next_app = (PANES[FOCUSED_PANE as usize].app + 1) % NUM_APPS;
        PANES[idx] = Pane { active: true, app: next_app, row: 0, col: 0, row_span: 1, col_span: 1 };
        PANE_COUNT += 1;
        rebuild_grid();
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Close the focused pane. If last pane, do nothing.
pub fn close_pane() {
    unsafe {
        if PANE_COUNT <= 1 { return; }
        let f = FOCUSED_PANE as usize;
        // Shift remaining panes down
        for i in f..(MAX_PANES - 1) {
            PANES[i] = PANES[i + 1];
        }
        PANES[MAX_PANES - 1] = Pane::empty();
        PANE_COUNT -= 1;
        if FOCUSED_PANE >= PANE_COUNT { FOCUSED_PANE = PANE_COUNT - 1; }
        // Recalculate grid
        rebuild_grid();
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

/// Rebuild grid — always 2×2 quad layout.
fn rebuild_grid() {
    unsafe {
        let n = PANE_COUNT as usize;
        // Grid positions: top-left, top-right, bottom-left, bottom-right
        LAYOUT_ROWS = if n <= 2 { 1 } else { 2 };
        LAYOUT_COLS = if n == 1 { 1 } else { 2 };

        match n {
            1 => {
                LAYOUT_ROWS = 1; LAYOUT_COLS = 1;
                PANES[0].row = 0; PANES[0].col = 0;
                PANES[0].row_span = 1; PANES[0].col_span = 1;
            }
            2 => {
                // Two panes: top-left, top-right
                LAYOUT_ROWS = 1; LAYOUT_COLS = 2;
                PANES[0].row = 0; PANES[0].col = 0;
                PANES[1].row = 0; PANES[1].col = 1;
                for i in 0..2 { PANES[i].row_span = 1; PANES[i].col_span = 1; }
            }
            3 => {
                // Three panes: top-left, top-right, bottom-left (bottom-right empty)
                LAYOUT_ROWS = 2; LAYOUT_COLS = 2;
                PANES[0].row = 0; PANES[0].col = 0;
                PANES[1].row = 0; PANES[1].col = 1;
                PANES[2].row = 1; PANES[2].col = 0;
                for i in 0..3 { PANES[i].row_span = 1; PANES[i].col_span = 1; }
            }
            4 => {
                // Four panes: 2×2 quad
                LAYOUT_ROWS = 2; LAYOUT_COLS = 2;
                PANES[0].row = 0; PANES[0].col = 0;
                PANES[1].row = 0; PANES[1].col = 1;
                PANES[2].row = 1; PANES[2].col = 0;
                PANES[3].row = 1; PANES[3].col = 1;
                for i in 0..4 { PANES[i].row_span = 1; PANES[i].col_span = 1; }
            }
            _ => {}
        }
    }
}

/// Cycle focus to next pane (Option+Tab).
pub fn split_toggle_focus() {
    unsafe {
        if PANE_COUNT <= 1 { return; }
        FOCUSED_PANE = (FOCUSED_PANE + 1) % PANE_COUNT;
        ACTIVE_APP.store(PANES[FOCUSED_PANE as usize].app, Ordering::Relaxed);
    }
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}

// Legacy compat
pub fn split_mode() -> u8 { unsafe { if PANE_COUNT > 1 { 1 } else { 0 } } }
pub fn split_app() -> u8 { unsafe { if PANE_COUNT > 1 { PANES[1].app } else { 0 } } }
pub fn set_split_app(app: u8) { unsafe { if PANE_COUNT > 1 { PANES[1].app = app; } } NEEDS_REDRAW.store(true, Ordering::Relaxed); }
pub fn split_focus() -> u8 { unsafe { FOCUSED_PANE } }

/// Set which pane is being rendered (for content_rect routing).
pub fn set_render_target(target: u8) {
    unsafe { RENDER_TARGET = target; }
}

/// Get pane count.
pub fn pane_count() -> u8 { unsafe { PANE_COUNT } }

/// Get pane app by index.
pub fn pane_app(idx: u8) -> u8 { unsafe { if (idx as usize) < MAX_PANES { PANES[idx as usize].app } else { 0 } } }

/// Get content rect for the current render target pane.
pub fn content_rect() -> WindowRect {
    let target = unsafe { RENDER_TARGET } as usize;
    pane_rect(target)
}

/// Get the content rect for a specific pane index.
pub fn pane_rect(idx: usize) -> WindowRect {
    let total_w = gpu::width() - BORDER_W * 2;
    let total_h = gpu::height() - TITLE_H - STATUS_H - BORDER_W * 2;

    unsafe {
        if idx >= PANE_COUNT as usize {
            return WindowRect { x: 0, y: 0, w: 0, h: 0 };
        }
        let rows = LAYOUT_ROWS.max(1) as u32;
        let cols = LAYOUT_COLS.max(1) as u32;
        let p = &PANES[idx];

        let cell_w = total_w / cols;
        let cell_h = total_h / rows;

        WindowRect {
            x: BORDER_W + (p.col as u32) * cell_w,
            y: TITLE_H + BORDER_W + (p.row as u32) * cell_h,
            w: cell_w * (p.col_span as u32) - 2, // -2 for divider gap
            h: cell_h * (p.row_span as u32) - 2,
        }
    }
}

// Legacy compat
pub fn content_rect_secondary() -> WindowRect { pane_rect(1) }

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

    // Tab indicators — spread evenly across the title bar
    let mode = unsafe { if PANE_COUNT > 1 { 1u8 } else { 0u8 } };
    let focus = unsafe { FOCUSED_PANE };
    let split_app_id = unsafe { if PANE_COUNT > 1 { PANES[1].app as usize } else { 0 } };

    // Center the tabs in the title bar
    let tab_spacing = 36u32;
    let total_tabs_w = NUM_APPS as u32 * tab_spacing;
    let tab_start = (w - total_tabs_w) / 2;
    for i in 0..NUM_APPS as usize {
        let tx = tab_start + (i as u32) * tab_spacing;
        let label = match i {
            0 => "SH",
            1 => "DS",
            2 => "FS",
            3 => "NM",
            4 => "ED",
            5 => "SK",
            6 => "CM",
            7 => "WB",
            8 => "BC",
            _ => "",
        };
        // Highlight: active app in primary panel
        let is_primary = i == app;
        let is_secondary = mode > 0 && i == split_app_id;

        if is_primary && is_secondary {
            // Both panels show same app (unlikely but handle it)
            font::draw_str(fb, w, tx, 4, label, BLACK, WHITE);
        } else if is_primary {
            let bg = if focus == 0 { WHITE } else { FG_DIM };
            font::draw_str(fb, w, tx, 4, label, BLACK, bg);
        } else if is_secondary {
            let bg = if focus == 1 { WHITE } else { FG_DIM };
            font::draw_str(fb, w, tx, 4, label, BLACK, bg);
        } else {
            font::draw_str(fb, w, tx, 4, label, FG_DIM, TITLE_BG);
        }
    }

    // Pane count indicator
    let n_panes = unsafe { PANE_COUNT };
    if n_panes > 1 {
        let mut px = tab_start + NUM_APPS as u32 * tab_spacing + 8;
        font::draw_str(fb, w, px, 4, "[", FG_DIM, TITLE_BG);
        px += 8;
        for i in 0..n_panes as usize {
            let pane_app = unsafe { PANES[i].app } as usize;
            let short = match pane_app {
                0 => "SH", 1 => "DS", 2 => "FS", 3 => "NM", 4 => "ED", 5 => "BC", _ => "??",
            };
            let color = if i == focus as usize { FG_HI } else { FG_DIM };
            font::draw_str(fb, w, px, 4, short, color, TITLE_BG);
            px += 20;
            if (i as u8) < n_panes - 1 {
                font::draw_str(fb, w, px, 4, "|", FG_DIM, TITLE_BG);
                px += 12;
            }
        }
        font::draw_str(fb, w, px, 4, "]", FG_DIM, TITLE_BG);
    }

    // Close/minimize (decorative)
    font::draw_str(fb, w, w - 40, 4, "_ X", FG_DIM, TITLE_BG);

    // Border below title
    gpu::fill_rect(0, TITLE_H, w, BORDER_W, BORDER);

    // Side borders
    gpu::fill_rect(0, TITLE_H, BORDER_W, h - TITLE_H - STATUS_H, BORDER);
    gpu::fill_rect(w - BORDER_W, TITLE_H, BORDER_W, h - TITLE_H - STATUS_H, BORDER);

    // Pane dividers
    unsafe {
        let total_w = w - BORDER_W * 2;
        let total_h = h - TITLE_H - STATUS_H - BORDER_W * 2;
        let rows = LAYOUT_ROWS.max(1) as u32;
        let cols = LAYOUT_COLS.max(1) as u32;

        // Vertical dividers
        for c in 1..cols {
            let div_x = BORDER_W + c * (total_w / cols);
            gpu::fill_rect(div_x, TITLE_H + BORDER_W, 2, total_h, BORDER);
        }
        // Horizontal dividers
        for r in 1..rows {
            let div_y = TITLE_H + BORDER_W + r * (total_h / rows);
            gpu::fill_rect(BORDER_W, div_y, total_w, 2, BORDER);
        }

        // Draw focused pane border highlight
        let f = FOCUSED_PANE as usize;
        if PANE_COUNT > 1 && f < MAX_PANES {
            let rect = pane_rect(f);
            let highlight: u32 = 0xFF3A3A3A;
            gpu::fill_rect(rect.x, rect.y, rect.w, 1, highlight); // top
            gpu::fill_rect(rect.x, rect.y + rect.h, rect.w, 1, highlight); // bottom
            gpu::fill_rect(rect.x, rect.y, 1, rect.h, highlight); // left
            gpu::fill_rect(rect.x + rect.w, rect.y, 1, rect.h, highlight); // right
        }
    }

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
    let (mins, _secs) = get_uptime();
    font::draw_str(fb, w, 312, ty, "UP:", FG_DIM, STATUS_BG);
    draw_num_at(fb, w, 336, ty, mins as usize, FG_DIM, STATUS_BG);
    font::draw_str(fb, w, 352, ty, "m", FG_DIM, STATUS_BG);

    // Keyboard hints
    unsafe {
        if PANE_COUNT > 1 {
            font::draw_str(fb, w, w - 320, ty, "Opt+Tab:focus Tab:app ^L:vsplit ^K:hsplit ^Q:close", FG_DIM, STATUS_BG);
        } else {
            font::draw_str(fb, w, w - 300, ty, "Tab:app ^L:vsplit ^K:hsplit", FG_DIM, STATUS_BG);
        }
    }
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
