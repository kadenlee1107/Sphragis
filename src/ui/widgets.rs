// Sphragis — shared app widgets .
//
// Higher-level composites built on top of `ui::draw` + `ui::font`,
// used by the DS / NM / SK apps (and likely the rest later). Lives
// here so the panel chrome, KV rows, tiles, status dots, etc. stay
// pixel-identical across apps without copy/paste drift.
//
// Source-of-truth for all dimensions / colors:
// docs/design/apps-ds-nm-sk/apps-specs.jsx

#![allow(dead_code)]

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;
use crate::ui::palette as p;

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
    x: u32, y: u32, _w: u32, h: u32,
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

// bumped from 110 to 160. Spec called for 110 but
// at our 8x16 bitmap font that only fits 13 chars per sub-caption,
// which truncated "3 PINS . 0 MISMATCH" to "3 PINS . 0 MI" and
// "origin allowlist" to "origin allowl". 160px = 20 chars fits the
// longest sub we have. 6 boxes × 160 + 5 arrows × 32 = 1120px,
// well under the 1248px available content width.
pub const FLOW_BOX_W: u32 = 160;
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

// ─── Wave 3: strip / conn pill / syntax tokens ──────────────────────

/// Fill a horizontal strip + draw optional 1px hairlines on top/bottom.
/// Returns the inner-content rect after a small left padding.
pub struct StripRect {
    pub x: u32, pub y: u32, pub w: u32, pub h: u32,
}
pub fn draw_strip(x: u32, y: u32, w: u32, h: u32, top_border: bool, bottom_border: bool) -> StripRect {
    gpu::fill_rect(x, y, w, h, BG);
    if top_border    { gpu::fill_rect(x, y, w, 1, HAIR); }
    if bottom_border { gpu::fill_rect(x, y + h - 1, w, 1, HAIR); }
    StripRect { x, y, w, h }
}

/// 1px vertical separator (between strip segments).
pub fn draw_seg_separator(x: u32, y: u32, h: u32) {
    gpu::fill_rect(x, y, 1, h, HAIR_HI);
}

/// Bordered "connection pill" with a colored dot + label + optional value.
/// Reuses the same look as the desktop status-bar pills.
/// Returns the pill's right-edge x.
pub fn draw_conn_pill(
    x: u32, y: u32,
    label: &str, value: Option<&str>,
    state: State,
) -> u32 {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let pad_x: u32 = 10;
    let dot_size: u32 = 6;
    let label_w = label.len() as u32 * CHAR_W;
    let value_w = value.map_or(0, |v| v.len() as u32 * CHAR_W + CHAR_W);
    let pill_w = pad_x + dot_size + 8 + label_w + value_w + pad_x;
    let pill_h: u32 = 22;
    gpu::fill_rect(x, y, pill_w, pill_h, PANEL);
    draw::draw_border(x, y, pill_w, pill_h, HAIR_HI);
    let (dot_fg, dot_ring) = state_dot_colors(state);
    let dot_x = x + pad_x;
    let dot_y = y + (pill_h - dot_size) / 2;
    if dot_x >= 1 && dot_y >= 1 {
        gpu::fill_rect(dot_x - 1, dot_y - 1, dot_size + 2, dot_size + 2, dot_ring);
    }
    gpu::fill_rect(dot_x, dot_y, dot_size, dot_size, dot_fg);
    let text_y = y + (pill_h - CHAR_H) / 2;
    let label_x = x + pad_x + dot_size + 8;
    font::draw_str(fb, sw, label_x, text_y, label, INK, PANEL);
    if let Some(v) = value {
        let value_x = label_x + label_w + CHAR_W;
        font::draw_str(fb, sw, value_x, text_y, v, DIM_TXT, PANEL);
    }
    x + pill_w
}

/// Syntax token kinds for the Editor (Rust-flavored).
#[derive(Clone, Copy)]
pub enum Tok {
    Keyword,
    String,
    Comment,
    Attr,
    Ident,
    Punct,
}

pub fn tok_color(t: Tok) -> u32 {
    match t {
        Tok::Keyword => CYAN,
        Tok::String  => GREEN,
        Tok::Comment => FAINT,
        Tok::Attr    => AMBER,
        Tok::Ident   => INK,
        Tok::Punct   => MID,
    }
}

/// Draw one tokenized code line. `spans` is a slice of (kind, text)
/// pairs; the function paints them left-to-right with the right
/// color per token.
pub fn draw_code_line(x: u32, y: u32, spans: &[(Tok, &str)]) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let mut cx = x;
    for (tok, text) in spans {
        font::draw_str(fb, sw, cx, y, text, tok_color(*tok), BG);
        cx += text.len() as u32 * CHAR_W;
    }
}

// ─── Wave 4 helpers (browser + caves) ─────────────────────────────

/// Hash a host string to a deterministic 12x12 swatch color.
/// 8-color palette: cyan/green/amber/red plus their dim variants.
pub fn host_swatch_color(host: &str) -> u32 {
    let palette: [u32; 8] = [
        CYAN, GREEN, AMBER, RED,
        CYAN_DIM, GREEN_DIM, AMBER_DIM, RED_DIM,
    ];
    let mut h: u32 = 0;
    for &b in host.as_bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    palette[(h as usize) % palette.len()]
}

/// 28×28 nav button. `glyph` is a single ASCII char ("<" ">" "R" "*").
/// `state`: Ok = enabled cyan, Plan = disabled faint, Warn = amber
/// (used during loading for the reload button).
pub fn draw_nav_btn(x: u32, y: u32, glyph: u8, state: State) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let color = match state {
        State::Plan => FAINT,
        State::Warn => AMBER,
        State::Fail => RED,
        _ => CYAN,
    };
    gpu::fill_rect(x, y, 28, 28, PANEL);
    draw::draw_border(x, y, 28, 28, HAIR_HI);
    let glyph_str = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(&glyph, 1)) };
    font::draw_str(fb, sw, x + (28 - CHAR_W) / 2, y + (28 - CHAR_H) / 2,
        glyph_str, color, PANEL);
}

/// Bookmark chip — 12x12 hash-derived swatch + host name in ink.
/// Returns the chip's right-edge x for chaining.
pub fn draw_bookmark_chip(x: u32, y: u32, host: &str) -> u32 {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let pad: u32 = 12;
    let swatch_size: u32 = 12;
    let gap: u32 = 8;
    let host_w = host.len() as u32 * CHAR_W;
    let chip_w = pad + swatch_size + gap + host_w + pad;
    // 12x12 swatch with 1px hair ring, vertically centered in 24px chip.
    let sw_x = x + pad;
    let sw_y = y + (24 - swatch_size) / 2;
    if sw_x >= 1 && sw_y >= 1 {
        gpu::fill_rect(sw_x - 1, sw_y - 1, swatch_size + 2, swatch_size + 2, HAIR);
    }
    gpu::fill_rect(sw_x, sw_y, swatch_size, swatch_size, host_swatch_color(host));
    let text_y = y + (24 - CHAR_H) / 2;
    font::draw_str(fb, sw, sw_x + swatch_size + gap, text_y, host, INK, BG);
    x + chip_w
}

/// Geometric Cave glyph — 64×48 stylized container box. Renders:
/// * outer container rectangle (1px stroke in `color`)
/// * dashed inner seal rect
/// * 4 corner notches (small L-shapes)
/// * single center node (2x2 dot)
pub fn draw_cave_glyph(origin_x: u32, origin_y: u32, color: u32) {
    let dim = match color {
        c if c == CYAN  => CYAN_DIM,
        c if c == AMBER => AMBER_DIM,
        c if c == GREEN => GREEN_DIM,
        c if c == RED   => RED_DIM,
        _               => FAINT,
    };
    // Outer container rectangle (stroke).
    draw::draw_border(origin_x + 6, origin_y + 8, 52, 32, color);
    // Dashed inner seal — 3px dash, 2px gap.
    let inner_x = origin_x + 12;
    let inner_y = origin_y + 14;
    let inner_w = 40u32;
    let inner_h = 20u32;
    let mut dx = 0u32;
    while dx < inner_w {
        let seg = (3u32).min(inner_w - dx);
        gpu::fill_rect(inner_x + dx, inner_y, seg, 1, dim);
        gpu::fill_rect(inner_x + dx, inner_y + inner_h - 1, seg, 1, dim);
        dx += 5;
    }
    let mut dy = 0u32;
    while dy < inner_h {
        let seg = (3u32).min(inner_h - dy);
        gpu::fill_rect(inner_x, inner_y + dy, 1, seg, dim);
        gpu::fill_rect(inner_x + inner_w - 1, inner_y + dy, 1, seg, dim);
        dy += 5;
    }
    // 4 corner notches (L-shapes).
    gpu::fill_rect(origin_x + 2,  origin_y + 4,  6, 1, color);
    gpu::fill_rect(origin_x + 2,  origin_y + 4,  1, 6, color);
    gpu::fill_rect(origin_x + 56, origin_y + 4,  6, 1, color);
    gpu::fill_rect(origin_x + 61, origin_y + 4,  1, 6, color);
    gpu::fill_rect(origin_x + 2,  origin_y + 43, 6, 1, color);
    gpu::fill_rect(origin_x + 2,  origin_y + 38, 1, 6, color);
    gpu::fill_rect(origin_x + 56, origin_y + 43, 6, 1, color);
    gpu::fill_rect(origin_x + 61, origin_y + 38, 1, 6, color);
    // Center node.
    gpu::fill_rect(origin_x + 31, origin_y + 23, 2, 2, color);
}

/// Action-hint line: "[shell] cmd # comment". Used by BC's detail
/// panel. `danger` paints the cmd in amber instead of cyan (for
/// irreversible actions like seal / destroy).
pub fn draw_action_hint(x: u32, y: u32, w: u32, cmd: &str, comment: &str, danger: bool) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let prefix = "[shell]";
    font::draw_str(fb, sw, x, y, prefix, FAINT, BG);
    let cmd_x = x + (prefix.len() as u32 + 1) * CHAR_W;
    let cmd_color = if danger { AMBER } else { CYAN };
    font::draw_str(fb, sw, cmd_x, y, cmd, cmd_color, BG);
    if !comment.is_empty() {
        // Right-align "# comment".
        let mut buf = [0u8; 64];
        buf[0] = b'#';
        buf[1] = b' ';
        let n = comment.len().min(buf.len() - 2);
        buf[2..2 + n].copy_from_slice(&comment.as_bytes()[..n]);
        let total = 2 + n;
        let total_w = total as u32 * CHAR_W;
        if w > total_w + 16 {
            let cmt_x = x + w - 8 - total_w;
            font::draw_str(fb, sw, cmt_x, y,
                unsafe { core::str::from_utf8_unchecked(&buf[..total]) }, FAINT, BG);
        }
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

// ── Wave 3 widgets ───────────────────────────────────────────────
// Used by caves_mgr (Wave 3) and inherited by FILES/NET/SECURITY/
// EDITOR/COMMS in Wave 4. New widgets import from `crate::ui::palette`;
// legacy widgets above keep their local cyberpunk palette.

/// 6x6 px state indicator. `filled` = INK solid circle (running state).
/// `!filled` = MID 1-px ring over BG fill (idle/stopped state).
/// Renders inside a 6x6 bounding box at (x, y).
pub fn paint_state_dot(x: u32, y: u32, filled: bool) {
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
    // Clear the 6x6 bounding box first so callers don't have to pre-paint
    // BG (e.g. when a dot transitions from filled to hollow on redraw).
    gpu::fill_rect(x, y, 6, 6, p::BG);
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

/// One key/value row in a status field list.
#[derive(Copy, Clone)]
pub struct StatusField<'a> {
    pub key:   &'a str,
    pub value: &'a str,
}

// ─── Wave 3: Caves Manager widgets ──────────────────────────────────────────

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

/// Paint a row of `[K]ey label · ...` actions across `rect`. Items are
/// separated by a `·` glyph in MID. The whole strip is left-aligned
/// starting at `rect.x + 8`.
pub fn paint_action_strip(rect: crate::ui::wm::WindowRect, actions: &[Action]) {
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
        debug_assert!(act.hotkey.is_ascii(), "Action::hotkey must be ASCII");
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
pub fn action_strip_hit_test(rect: crate::ui::wm::WindowRect, mx: i32, my: i32, actions: &[Action]) -> Option<char> {
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

/// Inspector-style sidebar + detail split. Caller decides what to paint
/// in each rect; this widget computes geometry and paints the 1-px
/// HAIRLINE divider between them.
#[derive(Copy, Clone)]
pub struct InspectorLayout {
    pub body_rect:   crate::ui::wm::WindowRect,
    pub sidebar_pct: u32,   // 0..100; default 38
}

impl InspectorLayout {
    /// Create an InspectorLayout with the default 38% sidebar.
    pub fn new(body_rect: crate::ui::wm::WindowRect) -> Self {
        Self { body_rect, sidebar_pct: 38 }
    }

    pub fn with_sidebar_pct(mut self, pct: u32) -> Self {
        self.sidebar_pct = pct.min(80).max(20);
        self
    }

    pub fn sidebar_rect(&self) -> crate::ui::wm::WindowRect {
        let w = (self.body_rect.w * self.sidebar_pct) / 100;
        crate::ui::wm::WindowRect {
            x: self.body_rect.x,
            y: self.body_rect.y,
            w,
            h: self.body_rect.h,
        }
    }

    pub fn detail_rect(&self) -> crate::ui::wm::WindowRect {
        let sw = (self.body_rect.w * self.sidebar_pct) / 100;
        crate::ui::wm::WindowRect {
            x: self.body_rect.x + sw + 1, // +1 for the divider
            y: self.body_rect.y,
            w: self.body_rect.w.saturating_sub(sw + 1),
            h: self.body_rect.h,
        }
    }

    /// Paint the 1-px HAIRLINE vertical divider.
    pub fn paint_divider(&self) {
        let sw = (self.body_rect.w * self.sidebar_pct) / 100;
        gpu::fill_rect(self.body_rect.x + sw, self.body_rect.y, 1, self.body_rect.h, p::HAIRLINE);
    }
}

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
    const TOPBAR_H: u32 = 22;
    const CHAR_H:   u32 = 16;
    const PAD_X:    u32 = 24;
    const PAD_Y:    u32 = 18;

    let screen_w = gpu::width();
    let screen_h = gpu::height();
    let fb = gpu::framebuffer();

    debug_assert!(modal.commit_key.is_ascii(), "ConfirmModal::commit_key must be ASCII");

    // Re-fill everything below the topbar with BG (effectively dims the
    // background — there's no alpha blend, so just clear). The modal
    // panel paints on top.
    gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, p::BG);

    // Compute modal size from content.
    let mut max_line_w = modal.title.len() as u32;
    for line in modal.body_lines {
        if (line.len() as u32) > max_line_w { max_line_w = line.len() as u32; }
    }
    let body_h_lines = modal.body_lines.len() as u32;
    let panel_w = (max_line_w * CHAR_W) + 2 * PAD_X;
    let panel_w = panel_w.max(35 * CHAR_W + 2 * PAD_X);  // footer hint is ~33 chars
    let panel_h = CHAR_H                          // title
                + 8                               // title-body gap
                + body_h_lines * (CHAR_H + 4)     // body lines
                + 18                              // body-footer gap
                + CHAR_H                          // footer hint
                + 2 * PAD_Y;

    let px = (screen_w.saturating_sub(panel_w)) / 2;
    let py = (screen_h.saturating_sub(panel_h)) / 2;

    // Panel fill + 1-px HAIRLINE border.
    gpu::fill_rect(px, py, panel_w, panel_h, p::PANEL);
    gpu::fill_rect(px, py, panel_w, 1, p::HAIRLINE);
    gpu::fill_rect(px, py + panel_h - 1, panel_w, 1, p::HAIRLINE);
    gpu::fill_rect(px, py, 1, panel_h, p::HAIRLINE);
    gpu::fill_rect(px + panel_w - 1, py, 1, panel_h, p::HAIRLINE);

    let inner_x = px + PAD_X;
    let mut y = py + PAD_Y;

    font::draw_str(fb, screen_w, inner_x, y, modal.title, p::INK, p::PANEL);
    y += CHAR_H + 8;

    for line in modal.body_lines {
        font::draw_str(fb, screen_w, inner_x, y, line, p::MID, p::PANEL);
        y += CHAR_H + 4;
    }

    y += 14;
    // Footer hint: <KEY> "again to confirm  " "Esc to cancel"
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
    debug_assert!(modal.commit_key.is_ascii(), "ConfirmModal::commit_key must be ASCII");
    let lower = c.to_ascii_lowercase();
    let key_lower = (modal.commit_key as u8).to_ascii_lowercase();
    if lower == key_lower { return ModalAction::Commit; }
    if c == 0x1B { return ModalAction::Cancel; }
    ModalAction::None
}

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
pub fn paint_inline_edit_form(rect: crate::ui::wm::WindowRect, fields: &[FormField], focused: usize) {
    const CHAR_H:    u32 = 16;
    const ROW_H:     u32 = 42;       // 10 px key + 2 px gap + 22 px field + 8 px row gap
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
                // Right-edge hint "< >" (cycle indicator)
                if focused == i && !field.readonly {
                    let hint = "< >";
                    let hx = box_x + box_w.saturating_sub(3 * CHAR_W + 4);
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
pub fn handle_form_click(fields: &[FormField], focused: &mut usize, rect: crate::ui::wm::WindowRect, mx: i32, my: i32) {
    const ROW_H: u32 = 42;
    const KEY_H: u32 = 10;
    const FIELD_H: u32 = 22;
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

/// Paint a vertical list of key/value rows. Key column auto-sized to
/// the longest key + 2-char padding. Keys render in MID; values in INK.
/// Row pitch is 18 px (16 px glyph height + 1 px top inset + 1 px gap).
///
/// Caller is responsible for clipping — this paints `fields.len()` rows
/// at row pitch 18 starting at `rect.y + 1`. Total footprint is
/// `fields.len() * 18 - 1` px.
pub fn paint_status_field_list(rect: crate::ui::wm::WindowRect, fields: &[StatusField]) {
    const ROW_H: u32 = 18;

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
        font::draw_str(fb, screen_w, rect.x,      row_y, f.key,   p::MID, p::BG);
        font::draw_str(fb, screen_w, value_col_x, row_y, f.value, p::INK, p::BG);
    }
}

// ── Wave 4 widgets ───────────────────────────────────────────────

/// One row in a `paint_activity_log` rendering.
#[derive(Copy, Clone)]
pub struct ActivityEntry<'a> {
    pub timestamp_str: &'a str,  // pre-formatted, e.g. "14:32:01"
    pub kind:          &'a str,  // pre-formatted, e.g. "dns" / "tcp_open"
    pub summary:       &'a str,
}

/// Paint a paginated activity log. Timestamp + kind render in MID
/// with the kind padded to 12 chars; summary in INK. Top-right of
/// rect shows `N..M of T`. Caller owns viewport_start + total.
pub fn paint_activity_log(
    rect: crate::ui::wm::WindowRect,
    entries: &[ActivityEntry],
    viewport_start: usize,
    total: usize,
) {
    const ROW_H: u32 = 18;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // Header: "N..M of T" right-aligned.
    let visible = entries.len();
    let last_idx = viewport_start + visible;
    let mut hdr_buf = [0u8; 48];
    let mut pos = 0;
    pos += write_dec(viewport_start + 1, &mut hdr_buf[pos..]);
    hdr_buf[pos] = b'.'; pos += 1;
    hdr_buf[pos] = b'.'; pos += 1;
    pos += write_dec(last_idx, &mut hdr_buf[pos..]);
    for &c in b" of " { hdr_buf[pos] = c; pos += 1; }
    pos += write_dec(total, &mut hdr_buf[pos..]);
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..pos]) };
    let hdr_w = (pos as u32) * CHAR_W;
    let hdr_x = rect.x + rect.w.saturating_sub(hdr_w + 8);
    font::draw_str(fb, screen_w, hdr_x, rect.y, hdr, p::MID, p::BG);

    // Body rows from rect.y + 20.
    let body_y = rect.y + 20;
    let max_rows = ((rect.h.saturating_sub(20)) / ROW_H) as usize;
    let rows_to_paint = entries.len().min(max_rows);

    for (i, entry) in entries.iter().take(rows_to_paint).enumerate() {
        let row_y = body_y + (i as u32) * ROW_H + 1;
        font::draw_str(fb, screen_w, rect.x + 4, row_y, entry.timestamp_str, p::MID, p::BG);
        let after_ts = rect.x + 4 + (entry.timestamp_str.len() as u32) * CHAR_W + 2 * CHAR_W;
        font::draw_str(fb, screen_w, after_ts, row_y, entry.kind, p::MID, p::BG);
        let after_kind = after_ts + 12 * CHAR_W;
        font::draw_str(fb, screen_w, after_kind, row_y, entry.summary, p::INK, p::BG);
    }
}

/// A bordered PANEL with a header (label + optional right badge)
/// and a body region rendered via `paint_status_field_list`.
pub struct StatusPanel<'a> {
    pub label: &'a str,
    pub header_right: Option<&'a str>,
    pub body: &'a [StatusField<'a>],
}

pub fn paint_status_panel(rect: crate::ui::wm::WindowRect, panel: &StatusPanel) {
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // PANEL fill + 1-px HAIRLINE border.
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, p::PANEL);
    gpu::fill_rect(rect.x, rect.y, rect.w, 1, p::HAIRLINE);
    gpu::fill_rect(rect.x, rect.y + rect.h - 1, rect.w, 1, p::HAIRLINE);
    gpu::fill_rect(rect.x, rect.y, 1, rect.h, p::HAIRLINE);
    gpu::fill_rect(rect.x + rect.w - 1, rect.y, 1, rect.h, p::HAIRLINE);

    // Header row.
    let hdr_y = rect.y + 10;
    font::draw_str(fb, screen_w, rect.x + 10, hdr_y, panel.label, p::MID, p::PANEL);
    if let Some(badge) = panel.header_right {
        let badge_w = (badge.len() as u32) * CHAR_W;
        let badge_x = rect.x + rect.w.saturating_sub(badge_w + 10);
        font::draw_str(fb, screen_w, badge_x, hdr_y, badge, p::INK, p::PANEL);
    }

    // Body rect under header, 10-px inset.
    let body_rect = crate::ui::wm::WindowRect {
        x: rect.x + 10,
        y: rect.y + 32,
        w: rect.w.saturating_sub(20),
        h: rect.h.saturating_sub(42),
    };
    paint_status_field_list(body_rect, panel.body);
}

/// A big-value metric tile. Label (MID) at top, 2× value (INK)
/// centered, sub-caption (MID) below. Caller draws the panel chrome.
pub fn paint_big_metric(rect: crate::ui::wm::WindowRect, label: &str, value: &str, sub: &str) {
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // Label at top.
    font::draw_str(fb, screen_w, rect.x, rect.y, label, p::MID, p::PANEL);

    // Big value: 2× scale, left-aligned, vertically centered.
    let value_h = CHAR_H * 2;
    let avail_h = rect.h.saturating_sub(CHAR_H + 4);
    let value_y = rect.y + CHAR_H + 4
        + (avail_h.saturating_sub(value_h + CHAR_H + 4)) / 2;
    font::draw_str_scaled(fb, screen_w, rect.x, value_y, value, p::INK, p::PANEL, 2);

    // Sub-caption.
    let sub_y = value_y + value_h + 4;
    font::draw_str(fb, screen_w, rect.x, sub_y, sub, p::MID, p::PANEL);
}

/// Render a file's bytes as text (printable-ASCII with line numbers)
/// or hex+ASCII dump. Mode chosen by sniffing first 256 bytes.
pub fn paint_file_preview(rect: crate::ui::wm::WindowRect, bytes: &[u8], viewport_start: usize) {
    if bytes.is_empty() {
        let fb = gpu::framebuffer();
        let screen_w = gpu::width();
        font::draw_str(fb, screen_w, rect.x + 4, rect.y + 4, "(empty)", p::MID, p::BG);
        return;
    }
    if is_text(bytes) {
        paint_file_preview_text(rect, bytes, viewport_start);
    } else {
        paint_file_preview_hex(rect, bytes, viewport_start);
    }
}

fn is_text(bytes: &[u8]) -> bool {
    let sample = &bytes[..bytes.len().min(256)];
    if sample.is_empty() { return true; }
    let printable = sample.iter().filter(|&&b|
        (0x20..=0x7E).contains(&b) || b == b'\n' || b == b'\t' || b == b'\r'
    ).count();
    (printable * 100) >= sample.len() * 90
}

fn paint_file_preview_text(rect: crate::ui::wm::WindowRect, bytes: &[u8], viewport_start: usize) {
    const ROW_H: u32 = 16;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    let max_rows = (rect.h / ROW_H) as usize;
    let line_no_w = 5 * CHAR_W;
    let text_x = rect.x + line_no_w + 4;

    let mut line: usize = 0;
    let mut line_start: usize = 0;
    let mut rendered: usize = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' || i == bytes.len() - 1 {
            let end = if b == b'\n' { i } else { i + 1 };
            if line >= viewport_start {
                if rendered >= max_rows { break; }
                let row_y = rect.y + (rendered as u32) * ROW_H;
                let mut ln_buf = [b' '; 5];
                let mut ln = (line + 1) as u32;
                let mut j = 5;
                while ln > 0 && j > 0 { j -= 1; ln_buf[j] = b'0' + (ln % 10) as u8; ln /= 10; }
                let ln_str = unsafe { core::str::from_utf8_unchecked(&ln_buf) };
                font::draw_str(fb, screen_w, rect.x, row_y, ln_str, p::MID, p::BG);
                let line_str = unsafe { core::str::from_utf8_unchecked(&bytes[line_start..end]) };
                font::draw_str(fb, screen_w, text_x, row_y, line_str, p::INK, p::BG);
                rendered += 1;
            }
            line += 1;
            line_start = i + 1;
        }
    }
}

fn paint_file_preview_hex(rect: crate::ui::wm::WindowRect, bytes: &[u8], viewport_start: usize) {
    const ROW_H: u32 = 14;
    const BYTES_PER_ROW: usize = 16;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    let max_rows = (rect.h / ROW_H) as usize;
    let total_rows = bytes.len().div_ceil(BYTES_PER_ROW);
    let end_row = (viewport_start + max_rows).min(total_rows);

    for (rendered, row_idx) in (viewport_start..end_row).enumerate() {
        let row_y = rect.y + (rendered as u32) * ROW_H;
        let start = row_idx * BYTES_PER_ROW;
        let end = (start + BYTES_PER_ROW).min(bytes.len());

        // 4-digit hex offset.
        let mut off_buf = [b'0'; 4];
        let off = (start as u32) & 0xFFFF;
        for k in 0..4 {
            let nibble = ((off >> ((3 - k) * 4)) & 0xF) as u8;
            off_buf[k] = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
        }
        let off_str = unsafe { core::str::from_utf8_unchecked(&off_buf) };
        font::draw_str(fb, screen_w, rect.x, row_y, off_str, p::MID, p::BG);

        // 16 bytes of hex.
        let mut hex_buf = [b' '; 48];
        for (k, byte) in bytes[start..end].iter().enumerate() {
            let hi = (byte >> 4) & 0xF;
            let lo = byte & 0xF;
            hex_buf[k * 3]     = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
            hex_buf[k * 3 + 1] = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
        }
        let hex_str = unsafe { core::str::from_utf8_unchecked(&hex_buf) };
        font::draw_str(fb, screen_w, rect.x + 5 * CHAR_W, row_y, hex_str, p::INK, p::BG);

        // ASCII gutter.
        let mut ascii_buf = [b'.'; 16];
        for (k, byte) in bytes[start..end].iter().enumerate() {
            ascii_buf[k] = if (0x20..=0x7E).contains(byte) { *byte } else { b'.' };
        }
        let ascii_str = unsafe { core::str::from_utf8_unchecked(&ascii_buf) };
        font::draw_str(fb, screen_w, rect.x + 5 * CHAR_W + 49 * CHAR_W, row_y, ascii_str, p::INK, p::BG);
    }
}
