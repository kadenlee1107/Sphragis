// Bat_OS — shared app widgets (STUMP #125).
//
// Higher-level composites built on top of `ui::draw` + `ui::font`,
// used by the DS / NM / SK apps (and likely the rest later). Lives
// here so the panel chrome, KV rows, tiles, status dots, etc. stay
// pixel-identical across apps without copy/paste drift.
//
// Source-of-truth for all dimensions / colors:
//   docs/design/apps-ds-nm-sk/apps-specs.jsx

#![allow(dead_code)]

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;

// ─── Palette (mirrors lock-screen + desktop chrome) ─────────────────

pub const BG:        u32 = 0xFF0A0A0A;
pub const PANEL:     u32 = 0xFF0E0E0E;
pub const HAIR:      u32 = 0xFF1A1A1A;
pub const HAIR_HI:   u32 = 0xFF262626;
pub const INK:       u32 = 0xFFE5E7EB;
pub const MID:       u32 = 0xFF9CA3AF;
pub const DIM_TXT:   u32 = 0xFF4B5563;
pub const FAINT:     u32 = 0xFF374151;
pub const CYAN:      u32 = 0xFF22D3EE;
pub const CYAN_DIM:  u32 = 0xFF0E7490;
pub const GREEN:     u32 = 0xFF22C55E;
pub const GREEN_DIM: u32 = 0xFF14532D;
pub const AMBER:     u32 = 0xFFF59E0B;
pub const AMBER_DIM: u32 = 0xFF78350F;
pub const RED:       u32 = 0xFFEF4444;
pub const RED_DIM:   u32 = 0xFF7F1D1D;

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;

// ─── Status enum ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Ok,      // green — ACTIVE / READY / VERIFIED
    Warn,    // amber — STANDBY / WARN / RESEARCH
    Fail,    // red   — DENIED / OFFLINE / FAIL
    Plan,    // dim   — NOT WIRED / IDLE
    Neutral, // cyan  — live values (IPs, modes, hashes)
}

pub fn state_value_color(s: State) -> u32 {
    match s {
        State::Ok      => GREEN,
        State::Warn    => AMBER,
        State::Fail    => RED,
        State::Plan    => DIM_TXT,
        State::Neutral => CYAN,
    }
}

pub fn state_dot_colors(s: State) -> (u32, u32) {
    match s {
        State::Ok      => (GREEN, GREEN_DIM),
        State::Warn    => (AMBER, AMBER_DIM),
        State::Fail    => (RED,   RED_DIM),
        State::Plan    => (DIM_TXT, FAINT),
        State::Neutral => (CYAN,  CYAN_DIM),
    }
}

// ─── Panel chrome ───────────────────────────────────────────────────

/// Inner content rect of a panel after the title strip + body padding.
/// Apps draw their KV rows / tiles / etc. inside this rect.
pub struct PanelInner {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Draw the standard panel chrome — 1px hairline+ border, 24px title
/// strip with uppercase faint label + optional right-aligned dim
/// metric, 1px separator under the title. Returns the inner content
/// rect (after body padding 16px L/R · 12px T/B).
pub fn draw_panel(x: u32, y: u32, w: u32, h: u32, title: &str, metric: Option<&str>) -> PanelInner {
    if w < 4 || h < 4 {
        return PanelInner { x: x + 2, y: y + 2, w: w.saturating_sub(4), h: h.saturating_sub(4) };
    }
    // Border (square corners, no fill — panel sits on bg).
    draw::draw_border(x, y, w, h, HAIR_HI);
    // Title strip — 24px tall.
    let strip_h: u32 = 24;
    // Label, 12px L padding, vertically centered.
    let label_y = y + (strip_h - CHAR_H) / 2;
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    font::draw_str(fb, sw, x + 12, label_y, title, FAINT, BG);
    // Metric, right-aligned dim.
    if let Some(m) = metric {
        let m_w = m.len() as u32 * CHAR_W;
        if x + w > 12 + m_w {
            let m_x = x + w - 12 - m_w;
            font::draw_str(fb, sw, m_x, label_y, m, DIM_TXT, BG);
        }
    }
    // 1px separator under title.
    if h > strip_h + 1 {
        gpu::fill_rect(x + 1, y + strip_h, w - 2, 1, HAIR);
    }
    // Inner rect — 16px L/R, 12px T/B body padding.
    let pad_x: u32 = 16;
    let pad_y: u32 = 12;
    let inner_x = x + pad_x;
    let inner_y = y + strip_h + pad_y;
    let inner_w = w.saturating_sub(pad_x * 2);
    let inner_h = h.saturating_sub(strip_h + pad_y * 2);
    PanelInner { x: inner_x, y: inner_y, w: inner_w, h: inner_h }
}

// ─── Status dot ─────────────────────────────────────────────────────

/// 6×6 colored square + 1px outer ring, drawn at (x, y).
pub fn draw_status_dot(x: u32, y: u32, s: State) {
    let (fg, ring) = state_dot_colors(s);
    if x >= 1 && y >= 1 {
        gpu::fill_rect(x - 1, y - 1, 8, 8, ring);
    }
    gpu::fill_rect(x, y, 6, 6, fg);
}

// ─── Type A · KV row ────────────────────────────────────────────────

/// One key/value row.
/// `label` paints in MID at column 0.
/// `value` paints in `state_value_color(state)` at column `label_w`.
/// If `with_dot` is true, an 8px-wide dot precedes the label.
/// Row height is the standard 22px (16px line + 6px breathing).
pub fn draw_kv_row(
    x: u32, y: u32,
    label_w: u32,
    label: &str, value: &str,
    state: State,
    with_dot: bool,
) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + 3; // 6px breathing / 2 = 3 at top, 3 at bottom

    let mut cursor_x = x;
    if with_dot {
        // Dot vertically centered with text.
        let dot_y = y + (16 - 6) / 2 + 3;
        draw_status_dot(cursor_x, dot_y, state);
        cursor_x += 8 + 8; // dot width + 8px gap
    }
    font::draw_str(fb, sw, cursor_x, text_y, label, MID, BG);
    let value_x = if with_dot { x + 8 + 8 + label_w } else { x + label_w };
    let value_color = state_value_color(state);
    font::draw_str(fb, sw, value_x, text_y, value, value_color, BG);
}

pub const KV_ROW_H: u32 = 22;

// ─── Type B · Tile ──────────────────────────────────────────────────

/// Big-number tile. `label` (faint, 10px-feel) sits top, `value`
/// (ink, scaled-2x for prominence) middle, `unit` (dim) below.
/// Optional left-of-label status dot.
pub fn draw_tile(
    x: u32, y: u32, w: u32, h: u32,
    label: &str, value: &str, unit: &str,
    dot: Option<State>,
) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    // Top: label (with optional dot).
    let label_y = y + 8;
    let mut lx = x + 8;
    if let Some(s) = dot {
        let dy = label_y + (16 - 6) / 2;
        draw_status_dot(lx, dy, s);
        lx += 8 + 6;
    }
    font::draw_str(fb, sw, lx, label_y, label, FAINT, BG);

    // Middle: value at 2x scale (so 8x16 → 16x32).
    let value_y = y + 8 + 16 + 4;
    let value_color = if let Some(s) = dot { state_value_color(s) } else { INK };
    font::draw_str_scaled(fb, sw, x + 8, value_y, value, value_color, BG, 2);

    // Bottom: unit, 4px below the value.
    let unit_y = value_y + 32 + 4;
    if unit_y + CHAR_H <= y + h {
        font::draw_str(fb, sw, x + 8, unit_y, unit, DIM_TXT, BG);
    }
}

// ─── Type C · Caves table ───────────────────────────────────────────

/// Header row: STATE / NAME / CAPABILITIES.
pub fn draw_caves_header(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let ty = y + 1;
    font::draw_str(fb, sw, x + 4, ty, "STATE", FAINT, BG);
    font::draw_str(fb, sw, x + 4 + 48, ty, "NAME", FAINT, BG);
    // Right-aligned "CAPABILITIES" — pin to the cap-pill column.
    let cap_label = "CAPABILITIES";
    if w > (cap_label.len() as u32 * CHAR_W) + 8 {
        let cap_x = x + w - 8 - cap_label.len() as u32 * CHAR_W;
        font::draw_str(fb, sw, cap_x, ty, cap_label, FAINT, BG);
    }
    // Hairline under header.
    gpu::fill_rect(x, y + 18, w, 1, HAIR);
}

pub const CAVES_HEADER_H: u32 = 18;
pub const CAVES_ROW_H:    u32 = 28;

/// One cave row: state badge + name + 4 cap pills (NET RAW DSP FS).
/// `caps` is a 4-bit mask: bit 0 = NET, 1 = RAW, 2 = DSP, 3 = FS.
pub fn draw_caves_row(
    x: u32, y: u32, w: u32,
    badge_state: State, badge: &str,
    name: &str,
    caps: u8,
) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let ty = y + (CAVES_ROW_H - CHAR_H) / 2;

    // State badge — 32x16 outlined cell with the 3-letter state.
    let badge_w: u32 = 32;
    let badge_h: u32 = 16;
    let badge_x = x + 4;
    let badge_y = y + (CAVES_ROW_H - badge_h) / 2;
    let (fg, ring) = state_dot_colors(badge_state);
    draw::draw_border(badge_x, badge_y, badge_w, badge_h, fg);
    font::draw_str(fb, sw, badge_x + (badge_w - badge.len() as u32 * CHAR_W) / 2,
        badge_y + (badge_h - CHAR_H) / 2, badge, fg, BG);
    let _ = ring;

    // Name.
    font::draw_str(fb, sw, badge_x + badge_w + 12, ty, name, INK, BG);

    // 4 cap pills, right-aligned.
    let cap_w: u32 = 38;
    let cap_h: u32 = 16;
    let cap_gap: u32 = 4;
    let labels = ["NET", "RAW", "DSP", "FS"];
    let total_caps_w = 4 * cap_w + 3 * cap_gap;
    if w > total_caps_w + 16 {
        let pills_x = x + w - 4 - total_caps_w;
        for i in 0..4 {
            let on = (caps >> i) & 1 == 1;
            let pill_x = pills_x + (i as u32) * (cap_w + cap_gap);
            let pill_y = y + (CAVES_ROW_H - cap_h) / 2;
            let color = if on { CYAN } else { FAINT };
            draw::draw_border(pill_x, pill_y, cap_w, cap_h, color);
            let lbl = labels[i];
            font::draw_str(fb, sw, pill_x + (cap_w - lbl.len() as u32 * CHAR_W) / 2,
                pill_y + (cap_h - CHAR_H) / 2, lbl, color, BG);
        }
    }
    // 1px bottom separator.
    gpu::fill_rect(x, y + CAVES_ROW_H - 1, w, 1, HAIR);
}

/// "(no further caves · N slots free)" placeholder row.
pub fn draw_caves_empty_row(x: u32, y: u32, w: u32, free: usize) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let mut buf = [0u8; 64];
    let mut p = 0usize;
    let prefix = b"(no further caves . ";
    buf[p..p + prefix.len()].copy_from_slice(prefix);
    p += prefix.len();
    p += write_dec(free, &mut buf[p..]);
    let suffix = b" slots free)";
    buf[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    let s = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
    let ty = y + (CAVES_ROW_H - CHAR_H) / 2;
    font::draw_str(fb, sw, x + 4, ty, s, DIM_TXT, BG);
    let _ = w;
}

// ─── Flow diagram (NetMon SECURITY STACK) ───────────────────────────

pub const FLOW_BOX_W: u32 = 110;
pub const FLOW_BOX_H: u32 = 44;
pub const FLOW_ARROW_W: u32 = 32;

pub fn draw_flow_box(x: u32, y: u32, label: &str, sub: &str, state: State) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let color = state_value_color(state);
    // Panel fill + border.
    gpu::fill_rect(x, y, FLOW_BOX_W, FLOW_BOX_H, PANEL);
    draw::draw_border(x, y, FLOW_BOX_W, FLOW_BOX_H, color);
    // Label, centered, top half.
    let lx = x + (FLOW_BOX_W - label.len() as u32 * CHAR_W) / 2;
    font::draw_str(fb, sw, lx, y + 8, label, color, PANEL);
    // Sub caption, centered, bottom half — truncate to fit.
    let sub_max = (FLOW_BOX_W / CHAR_W) as usize;
    let sub_show = if sub.len() > sub_max { &sub[..sub_max] } else { sub };
    let sx = x + (FLOW_BOX_W - sub_show.len() as u32 * CHAR_W) / 2;
    font::draw_str(fb, sw, sx, y + 8 + CHAR_H + 2, sub_show, DIM_TXT, PANEL);
}

/// 1px line + 6×8 triangle head, drawn from (x, y) horizontally
/// across `w` pixels at the vertical center of FLOW_BOX_H.
pub fn draw_flow_arrow(x: u32, y: u32, w: u32) {
    let cy = y + FLOW_BOX_H / 2;
    // Line.
    gpu::fill_rect(x, cy, w.saturating_sub(8), 1, CYAN_DIM);
    // Triangle head — 4 horizontal rects of decreasing width.
    let head_x = x + w - 8;
    gpu::fill_rect(head_x,     cy - 3, 1, 7, CYAN);
    gpu::fill_rect(head_x + 1, cy - 2, 1, 5, CYAN);
    gpu::fill_rect(head_x + 2, cy - 2, 1, 5, CYAN);
    gpu::fill_rect(head_x + 3, cy - 1, 1, 3, CYAN);
    gpu::fill_rect(head_x + 4, cy - 1, 1, 3, CYAN);
    gpu::fill_rect(head_x + 5, cy,     1, 1, CYAN);
}

// ─── Audit mini-strip (Security INTEGRITY panel) ────────────────────

pub struct AuditLine<'a> {
    pub idx: u32,
    pub cat: &'a str,
    pub text: &'a str,
}

/// Lock-screen-style boot-log strip: "[N] cat: text" per line.
pub fn draw_audit_strip(x: u32, y: u32, lines: &[AuditLine]) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    // Faint header.
    font::draw_str(fb, sw, x, y, "AUDIT . LAST 4", FAINT, BG);
    for (i, line) in lines.iter().enumerate() {
        let row_y = y + 18 + (i as u32) * 16;
        // [NNN] in cyan.
        let mut buf = [0u8; 8];
        buf[0] = b'[';
        let mut p = 1;
        p += write_dec(line.idx as usize, &mut buf[p..]);
        buf[p] = b']'; p += 1;
        let idx_s = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
        font::draw_str(fb, sw, x, row_y, idx_s, CYAN, BG);
        let after_idx = x + (p as u32 + 1) * CHAR_W;
        font::draw_str(fb, sw, after_idx, row_y, line.cat, MID, BG);
        let after_cat = after_idx + (line.cat.len() as u32 + 1) * CHAR_W;
        font::draw_str(fb, sw, after_cat, row_y, line.text, INK, BG);
    }
}

// ─── Internal helpers ───────────────────────────────────────────────

fn write_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
