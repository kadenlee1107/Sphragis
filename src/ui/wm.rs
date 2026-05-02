#![allow(dead_code)]
// Bat_OS — Tiling Window Manager
// Keyboard-driven, no floating windows.
// Ctrl+1-5 switches apps. Ctrl+H/L splits horizontal/vertical.
// Sharp angular borders, bat aesthetic.

use crate::ui::gpu;
use super::font;
use super::draw;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

// STUMP #120 — Claude-Design desktop chrome. Palette mirrors the
// lock-screen + spec sheet so apps inherit a single visual language.
const BG:        u32 = 0xFF0A0A0A;
const PANEL:     u32 = 0xFF0E0E0E;
const HAIR:      u32 = 0xFF1A1A1A;
const HAIR_HI:   u32 = 0xFF262626;
const INK:       u32 = 0xFFE5E7EB;
const MID:       u32 = 0xFF9CA3AF;
const DIM_TXT:   u32 = 0xFF4B5563;
const FAINT:     u32 = 0xFF374151;
const CYAN:      u32 = 0xFF22D3EE;
const CYAN_DIM:  u32 = 0xFF0E7490;
const GREEN:     u32 = 0xFF22C55E;
const GREEN_DIM: u32 = 0xFF14532D;
const AMBER:     u32 = 0xFFF59E0B;
const AMBER_DIM: u32 = 0xFF78350F;
const RED:       u32 = 0xFFEF4444;
const RED_DIM:   u32 = 0xFF7F1D1D;

// Legacy aliases — kept so existing call sites don't have to be
// rewritten in lockstep. Map onto the new palette.
const BLACK: u32 = BG;
const WHITE: u32 = INK;
const BORDER: u32 = HAIR;
const TITLE_BG: u32 = BG;
const FG: u32 = MID;
const FG_HI: u32 = INK;
const FG_DIM: u32 = DIM_TXT;
const STATUS_BG: u32 = BG;

// Title-bar segment widths (per spec: brand 132 · 9 tabs × 64 · cave 168).
const BRAND_W:    u32 = 132;
const TAB_W:      u32 = 64;
const CAVE_W_MIN: u32 = 168;
const CHAR_W:     u32 = 8;
const CHAR_H:     u32 = 16;

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

// 2026-04-20 21:45 — Close-button focus. When the user has tabbed
// past the last app onto the title-bar X, this is true. Tab cycles
// app 0 → 1 → … → 8 → close-button → app 0 …
// Enter while CLOSE_FOCUSED triggers shutdown.
static CLOSE_FOCUSED: AtomicBool = AtomicBool::new(false);

pub fn focus_close_button() {
    CLOSE_FOCUSED.store(true, Ordering::Relaxed);
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}
pub fn unfocus_close_button() {
    CLOSE_FOCUSED.store(false, Ordering::Relaxed);
    NEEDS_REDRAW.store(true, Ordering::Relaxed);
}
pub fn is_close_focused() -> bool {
    CLOSE_FOCUSED.load(Ordering::Relaxed)
}

pub fn init_panes_pub() { init_panes(); }

/// V11-state-sweep: reset pane layout + rendered-target tracking on cave
/// switch. Without this, cave A's rendered pane contents (text drawn
/// into the framebuffer backing the pane) and pane topology survive
/// into cave B's first render pass.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for p in (&mut *core::ptr::addr_of_mut!(PANES)).iter_mut() {
            *p = Pane::empty();
        }
        PANE_COUNT = 0;
        FOCUSED_PANE = 0;
        LAYOUT_ROWS = 1;
        LAYOUT_COLS = 1;
        RENDER_TARGET = 0;
    }
    NEEDS_REDRAW.store(true, Ordering::Release);
    ACTIVE_APP.store(APP_SHELL, Ordering::Release);
    init_panes();
}

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
///
/// STUMP #120 — Claude-Design desktop-chrome port. Layout per
/// `docs/design/desktop-shell/shell-specs.jsx`:
///   * Title bar 24px: brand block (132px) | tab strip (9 × 64px,
///     centered between brand and cave) | cave block (168px right).
///   * Content area: hairline top + bottom, apps own internal layout.
///   * Status bar 28px: 5 live-state segments (ENCRYPTED · NET · TLS
///     · JS · AUDIT) + right-aligned uptime.
pub fn draw_frame() {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    let active_app = ACTIVE_APP.load(Ordering::Relaxed) as usize;

    // Background.
    gpu::fill_screen(BG);

    // ── TITLE BAR ───────────────────────────────────────────────
    gpu::fill_rect(0, 0, w, TITLE_H, BG);

    // 1) Brand block — bat-mini + "BAT_OS" wordmark.
    let brand_text_y = (TITLE_H - CHAR_H) / 2;
    let bat_y = (TITLE_H - 12) / 2;
    draw::draw_bat_mini(14, bat_y, CYAN);
    let wordmark_x = 14 + 18 + 8; // bat width + gap
    font::draw_str(fb, w, wordmark_x, brand_text_y, "BAT", INK, BG);
    font::draw_str(fb, w, wordmark_x + 3 * CHAR_W, brand_text_y, "_", CYAN, BG);
    font::draw_str(fb, w, wordmark_x + 4 * CHAR_W, brand_text_y, "OS", INK, BG);
    // 1px right separator on the brand block.
    gpu::fill_rect(BRAND_W, 0, 1, TITLE_H, HAIR);

    // 2) Tab strip — 9 × 64px = 576px, centered between brand and cave.
    let tabs_total = NUM_APPS as u32 * TAB_W;
    let tabs_start = (w - tabs_total) / 2;
    let labels = ["SH", "DS", "FS", "NM", "ED", "SK", "CM", "WB", "BC"];
    let pane_count_now = unsafe { PANE_COUNT };
    let split_app = unsafe { if pane_count_now > 1 { PANES[1].app as usize } else { usize::MAX } };
    let focus = unsafe { FOCUSED_PANE };
    for i in 0..NUM_APPS as usize {
        let tx = tabs_start + (i as u32) * TAB_W;
        let is_primary = i == active_app;
        let is_secondary = pane_count_now > 1 && i == split_app;
        let is_active = is_primary || is_secondary;

        // Vertical 1px hairline between tabs (skip the last one).
        if i < (NUM_APPS as usize) - 1 {
            gpu::fill_rect(tx + TAB_W, 0, 1, TITLE_H, HAIR);
        }

        // Letter pair, vertically centered in the title-bar
        // (minus the 2px reserved for the active-state underline).
        // No digit hint — the cyan underline + ink-vs-dim coloring
        // already shows which tab is active, and the shell banner
        // tells the operator about the Ctrl+N chord bindings.
        let label_color = if is_active { INK } else { DIM_TXT };
        let label_x = tx + (TAB_W - 2 * CHAR_W) / 2;
        let label_y = (TITLE_H - 2 - CHAR_H) / 2;
        font::draw_str(fb, w, label_x, label_y, labels[i], label_color, BG);

        // Active underline strip (2px tall, 6px inset L/R).
        if is_active {
            // Primary pane gets full cyan; secondary gets cyan-dim if
            // the focus is on the other pane (so the user can still
            // tell which is which).
            let underline = if (is_primary && focus == 0) || (is_secondary && focus == 1) {
                CYAN
            } else {
                CYAN_DIM
            };
            gpu::fill_rect(tx + 6, TITLE_H - 2, TAB_W - 12, 2, underline);
        }
    }

    // 3) Cave block — right-aligned, "CAVE" label + name + status dot.
    let cave_w = CAVE_W_MIN;
    let cave_x = w - cave_w;
    gpu::fill_rect(cave_x, 0, 1, TITLE_H, HAIR);
    let cave_text_y = (TITLE_H - CHAR_H) / 2;
    font::draw_str(fb, w, cave_x + 14, cave_text_y, "CAVE", DIM_TXT, BG);
    let (cave_name, cave_dot, cave_dot_ring) = active_cave_indicator();
    let name_x = cave_x + 14 + 4 * CHAR_W + CHAR_W;
    font::draw_str(fb, w, name_x, cave_text_y, cave_name, INK, BG);
    // 6px dot with 1px ring, far right.
    let dot_x = w - 14 - 6;
    let dot_y = (TITLE_H - 6) / 2;
    gpu::fill_rect(dot_x - 1, dot_y - 1, 8, 8, cave_dot_ring);
    gpu::fill_rect(dot_x, dot_y, 6, 6, cave_dot);

    // ── CONTENT AREA HAIRLINES ──────────────────────────────────
    gpu::fill_rect(0, TITLE_H, w, 1, HAIR);
    gpu::fill_rect(0, h - STATUS_H - 1, w, 1, HAIR);

    // ── PANE DIVIDERS (when split) ──────────────────────────────
    unsafe {
        let total_w = w - BORDER_W * 2;
        let total_h = h - TITLE_H - STATUS_H - BORDER_W * 2;
        let rows = LAYOUT_ROWS.max(1) as u32;
        let cols = LAYOUT_COLS.max(1) as u32;
        for c in 1..cols {
            let div_x = BORDER_W + c * (total_w / cols);
            gpu::fill_rect(div_x, TITLE_H + BORDER_W, 1, total_h, HAIR);
        }
        for r in 1..rows {
            let div_y = TITLE_H + BORDER_W + r * (total_h / rows);
            gpu::fill_rect(BORDER_W, div_y, total_w, 1, HAIR);
        }
        // Focused pane: 1px cyan-dim border so the user can tell
        // which pane keystrokes go to.
        let f = FOCUSED_PANE as usize;
        if PANE_COUNT > 1 && f < MAX_PANES {
            let rect = pane_rect(f);
            draw::draw_border(rect.x, rect.y, rect.w, rect.h, CYAN_DIM);
        }
    }

    // ── STATUS BAR ──────────────────────────────────────────────
    let sy = h - STATUS_H;
    gpu::fill_rect(0, sy, w, STATUS_H, BG);
    gpu::fill_rect(0, sy, w, 1, HAIR);
    draw_status_bar(fb, w, sy);
}

/// Resolve the cave indicator for the title bar's right-hand block.
fn active_cave_indicator() -> (&'static str, u32, u32) {
    let id = crate::batcave::cave::get_active();
    if id == usize::MAX {
        // No active cave — kernel context.
        return ("kernel", GREEN, GREEN_DIM);
    }
    // We only render the first 16 chars of the cave name into a tiny
    // static slot so we can return a `'static`-shaped reference. Most
    // cave names are well within 16 ASCII chars.
    static mut NAME_BUF: [u8; 16] = [b' '; 16];
    let name_len = unsafe {
        let buf_ptr = core::ptr::addr_of_mut!(NAME_BUF) as *mut u8;
        // Fill with spaces first.
        for i in 0..16 { core::ptr::write_volatile(buf_ptr.add(i), b' '); }
        // Try a few likely accessor patterns. We don't have a generic
        // "current_cave_name" helper, so call into the active cave
        // table directly.
        let raw = crate::batcave::cave::active_name_str();
        let n = raw.len().min(16);
        for i in 0..n { core::ptr::write_volatile(buf_ptr.add(i), raw.as_bytes()[i]); }
        n
    };
    let name = unsafe {
        core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(core::ptr::addr_of!(NAME_BUF) as *const u8, name_len)
        )
    };
    (name, GREEN, GREEN_DIM)
}

fn draw_status_bar(fb: *mut u32, w: u32, sy: u32) {
    // Each segment: dot? + 12px L/R padding + label (10px-equivalent
    // tracking 1.5) + value (11px-equivalent tracking 1). Separators
    // are 1px hair-hi vertical strips between segments.
    let mut x = 0u32;

    x = draw_status_segment(fb, w, x, sy, "ENCRYPTED", None, INK,
        Some((GREEN, GREEN_DIM)));

    let net_ok = crate::drivers::virtio::net::is_ready();
    let mut net_buf = [0u8; 16];
    let net_value: &str = if net_ok {
        let ip = crate::net::ip::our_ip();
        let n = crate::net::ip::ip_to_str(ip, &mut net_buf);
        unsafe { core::str::from_utf8_unchecked(&net_buf[..n]) }
    } else {
        "OFFLINE"
    };
    let net_color = if net_ok { INK } else { RED };
    x = draw_status_segment(fb, w, x, sy, "NET", Some(net_value), net_color, None);

    let mode = crate::net::tls_pinning::current_mode();
    let (tls_label, tls_color) = match mode {
        crate::net::tls_pinning::Mode::Lockdown => ("LOCKDOWN", CYAN),
        crate::net::tls_pinning::Mode::Research => ("RESEARCH", AMBER),
        crate::net::tls_pinning::Mode::Open     => ("OPEN",     RED),
    };
    x = draw_status_segment(fb, w, x, sy, "TLS", Some(tls_label), tls_color, None);

    let js_on = crate::browser::js::is_enabled();
    x = draw_status_segment(fb, w, x, sy, "JS",
        Some(if js_on { "ON" } else { "OFF" }),
        if js_on { AMBER } else { INK }, None);

    let audit_count = crate::security::audit::count();
    let mut audit_buf = [0u8; 24];
    let audit_n = format_audit(audit_count, &mut audit_buf);
    let audit_text = unsafe { core::str::from_utf8_unchecked(&audit_buf[..audit_n]) };
    let _ = draw_status_segment(fb, w, x, sy, "AUDIT", Some(audit_text), INK, None);

    // Right-anchored UPTIME segment.
    let (mins, secs) = get_uptime();
    let mut up_buf = [0u8; 24];
    let up_n = format_uptime(mins, secs, &mut up_buf);
    let up_text = unsafe { core::str::from_utf8_unchecked(&up_buf[..up_n]) };
    let label_w = 6 * CHAR_W;     // "UPTIME"
    let value_w = up_n as u32 * CHAR_W;
    let pad: u32 = 12;
    let seg_w = pad + label_w + CHAR_W + value_w + pad;
    let seg_x = w - seg_w;
    gpu::fill_rect(seg_x, sy, 1, STATUS_H, HAIR_HI);
    let text_y = sy + (STATUS_H - CHAR_H) / 2;
    font::draw_str(fb, w, seg_x + pad, text_y, "UPTIME", MID, BG);
    font::draw_str(fb, w, seg_x + pad + label_w + CHAR_W, text_y, up_text, INK, BG);
}

/// Paint a single status-bar segment (dot? + label + optional value).
/// Returns the segment's right edge so the caller can chain.
fn draw_status_segment(
    fb: *mut u32, w: u32,
    x: u32, sy: u32,
    label: &str, value: Option<&str>,
    value_color: u32,
    dot: Option<(u32, u32)>,
) -> u32 {
    let pad: u32 = 12;
    let label_w = label.len() as u32 * CHAR_W;
    let value_w = value.map_or(0, |v| v.len() as u32 * CHAR_W + CHAR_W);
    let dot_w: u32 = if dot.is_some() { 6 + 8 } else { 0 };
    let seg_w = pad + dot_w + label_w + value_w + pad;
    let text_y = sy + (STATUS_H - CHAR_H) / 2;

    if let Some((dot_color, dot_ring)) = dot {
        let dot_x = x + pad;
        let dot_y = sy + (STATUS_H - 6) / 2;
        gpu::fill_rect(dot_x - 1, dot_y - 1, 8, 8, dot_ring);
        gpu::fill_rect(dot_x, dot_y, 6, 6, dot_color);
    }
    let label_x = x + pad + dot_w;
    font::draw_str(fb, w, label_x, text_y, label, MID, BG);
    if let Some(v) = value {
        let value_x = label_x + label_w + CHAR_W;
        font::draw_str(fb, w, value_x, text_y, v, value_color, BG);
    }
    // Right separator.
    gpu::fill_rect(x + seg_w, sy, 1, STATUS_H, HAIR_HI);
    x + seg_w
}

fn format_audit(count: usize, out: &mut [u8]) -> usize {
    let mut p = 0;
    p += write_dec(count, &mut out[p..]);
    out[p] = b' '; p += 1;
    out[p] = b'/'; p += 1;
    out[p] = b' '; p += 1;
    p += write_dec(1024, &mut out[p..]);
    p
}

fn format_uptime(mins: u64, secs: u64, out: &mut [u8]) -> usize {
    let days = mins / (60 * 24);
    let hours = (mins / 60) % 24;
    let m = mins % 60;
    let mut p = 0;
    p += write_dec(days as usize, &mut out[p..]);
    out[p] = b'd'; p += 1;
    out[p] = b' '; p += 1;
    p += write_dec_2(hours as usize, &mut out[p..]);
    out[p] = b':'; p += 1;
    p += write_dec_2(m as usize, &mut out[p..]);
    out[p] = b':'; p += 1;
    p += write_dec_2(secs as usize, &mut out[p..]);
    p
}

fn write_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}

fn write_dec_2(n: usize, out: &mut [u8]) -> usize {
    out[0] = b'0' + ((n / 10) % 10) as u8;
    out[1] = b'0' + (n % 10) as u8;
    2
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
