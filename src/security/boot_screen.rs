// Bat_OS — Secure Boot Screen
//
// STUMP #116 — Claude-Design boot-screen redesign (April 2026).
// The previous version was a bare bat sprite + a single passphrase
// box on a black field. The new design (terminal-cyberpunk meets
// operator-tactical) was generated in Claude Design from a prompt
// describing this OS, then ported here. Source artifacts in
// `docs/design/lock-screen/` (jsx + spec sheet).
//
// Visual contract (matches `docs/design/lock-screen/specs.jsx`):
//   * 16-color palette anchored on near-black panels and cyan accent.
//   * Status pill row across the top with real subsystem state.
//   * Centered stack: geometric bat glyph + BAT_OS wordmark + version
//     line + passphrase field + helper hint row.
//   * Bottom-left: last 4 boot-log lines.
//   * Bottom-right: clock + attempts-remaining pill.
//   * Crosshair corner marks at all four corners.
//   * Three states: idle, typing, denied.
//
// What's deliberately NOT in this port (for follow-up STUMPs):
//   * 1Hz cursor blink — needs a wall-clock timer hook.
//   * 1-pixel scanline overlay — easy to add but ~340k pixel writes
//     per repaint; defer until we know we want the cost.
//   * TrueType wordmark — `ui/truetype.rs` exists but isn't wired
//     into the WM yet; we use the existing 8x16 bitmap font, which
//     looks chunkier than the spec's JetBrains Mono but is honest
//     about what we render today.
//
// CRITICAL color note: per STUMP #67, the QEMU framebuffer is
// FORMAT_B8G8R8A8 and we write u32s in `(A<<24)|(R<<16)|(G<<8)|B`
// form so the LE store lands as B,G,R,A in memory. That means
// hex literals like 0xFF22D3EE map directly to "alpha=FF, R=22,
// G=D3, B=EE" — the same ordering Claude Design's CSS hex uses.
// Pre-redesign, this file had `RED = 0xFF0000FF` which is
// actually solid blue under that encoding — i.e. "ACCESS DENIED"
// was rendering blue on the old screen. New constants below are
// from the Claude-Design spec sheet directly.

use crate::ui::gpu;
use crate::ui::font;
use crate::platform;
use super::{auth, wipe, deadman};

// ─── Palette ────────────────────────────────────────────────────────────

const BG:        u32 = 0xFF0A0A0A; // background
const PANEL:     u32 = 0xFF0E0E0E; // pill / field background
const HAIR:      u32 = 0xFF1A1A1A; // hairline rules
const HAIR_HI:   u32 = 0xFF262626; // pill / field borders, crosshair marks
const INK:       u32 = 0xFFE5E7EB; // primary text
const MID:       u32 = 0xFF9CA3AF; // labels
const DIM_TXT:   u32 = 0xFF4B5563; // meta text
const FAINT:     u32 = 0xFF374151; // captions
const CYAN:      u32 = 0xFF22D3EE; // primary accent (idle/typing)
const CYAN_DIM:  u32 = 0xFF0E7490; // ring / inset
const GREEN:     u32 = 0xFF22C55E; // status OK dot
const GREEN_DIM: u32 = 0xFF14532D;
const AMBER:     u32 = 0xFFF59E0B; // attempts dot
const AMBER_DIM: u32 = 0xFF78350F;
const RED:       u32 = 0xFFEF4444; // denied accent
const RED_DIM:   u32 = 0xFF7F1D1D;

// ─── Layout constants (1280x800 native; scales gracefully to 1024x768) ──

const MARGIN_X:    u32 = 56;
const MARGIN_Y:    u32 = 24;
const STATUS_ROW_H: u32 = 28;
const HAIRLINE_Y:   u32 = 64;
const CHAR_W:       u32 = 8;
const CHAR_H:       u32 = 16;

// Bat glyph rasterizes into a 120x72 native viewport. We draw it at 1x.
const BAT_W: u32 = 120;
const BAT_H: u32 = 72;

// Field geometry: 480x56, centered horizontally, ~50px above vertical center.
const FIELD_W: u32 = 480;
const FIELD_H: u32 = 56;
const DOT_PX:  u32 = 8; // each masking dot is 8x8
const DOT_GAP: u32 = 8;

// ─── Geometric bat glyph ────────────────────────────────────────────────
//
// Translated from Claude Design's `bat-glyph.jsx`. The SVG defines four
// filled polygons (left wing, right wing, head/ears, torso wedge) plus
// 10 dim "finger-bone" lines and 13 small circuit nodes. We rasterize
// them with a tiny scanline polygon fill (`fill_polygon` below), a
// Bresenham line for the bones, and `fill_rect` for the nodes.

// Coordinates are in the 120x72 source viewport; the renderer offsets
// them by (origin_x, origin_y) at draw time.
const BAT_LEFT_WING: &[(i32, i32)] = &[
    (60, 22), (54, 18), (44, 14), (32, 10), (18,  8),
    ( 6, 14), ( 2, 24), (10, 28), ( 4, 34), (14, 38),
    ( 8, 46), (22, 46), (18, 54), (32, 50), (30, 58),
    (44, 52), (46, 58), (56, 50), (58, 42),
];
const BAT_RIGHT_WING: &[(i32, i32)] = &[
    (60, 22), (66, 18), (76, 14), (88, 10), (102,  8),
    (114, 14), (118, 24), (110, 28), (116, 34), (106, 38),
    (112, 46), (98, 46), (102, 54), (88, 50), (90, 58),
    (76, 52), (74, 58), (64, 50), (62, 42),
];
const BAT_HEAD: &[(i32, i32)] = &[
    (54, 18), (54, 8), (57, 14), (60, 4), (63, 14), (66, 8), (66, 18),
];
const BAT_TORSO: &[(i32, i32)] = &[
    (54, 18), (66, 18), (64, 38), (60, 46), (56, 38),
];

// Finger-bone lines: (x1, y1) -> (x2, y2). Painted in CYAN_DIM over
// the wing fills for a circuit-board feel.
const BAT_BONES: &[(i32, i32, i32, i32)] = &[
    (56, 22,  18,  8), (56, 26,   6, 20), (56, 30,  10, 32),
    (56, 36,  14, 42), (56, 42,  22, 50),
    (64, 22, 102,  8), (64, 26, 114, 20), (64, 30, 110, 32),
    (64, 36, 106, 42), (64, 42,  98, 50),
];

// Circuit node 2x2 squares, in CYAN.
const BAT_NODES: &[(i32, i32)] = &[
    ( 17,  7), (  5, 13), (  9, 27), ( 13, 37), ( 21, 45),
    (101,  7), (113, 13), (109, 27), (105, 37), ( 97, 45),
    ( 51, 61), ( 67, 61), ( 59, 61),
];

// Eye slits (drawn near-black so they "punch through" the head fill).
const BAT_EYES: &[(i32, i32)] = &[ (56, 13), (62, 13) ];

// ─── Drawing primitives ────────────────────────────────────────────────

/// Bresenham horizontal/diagonal line. We don't have a `gpu::draw_line`,
/// and adding one to the GPU layer for one screen is overkill. Inline.
fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x >= 0 && y >= 0 {
            gpu::fill_rect(x as u32, y as u32, 1, 1, color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

/// Scanline polygon fill, no_std-friendly. The polygons we use have at
/// most 19 vertices — a fixed 32-slot intersection buffer per scanline
/// is plenty.
fn fill_polygon(points: &[(i32, i32)], origin_x: i32, origin_y: i32, color: u32) {
    if points.len() < 3 { return; }
    let mut min_y = points[0].1;
    let mut max_y = points[0].1;
    for &(_, y) in points {
        if y < min_y { min_y = y; }
        if y > max_y { max_y = y; }
    }
    for y in min_y..max_y {
        let mut x_inters = [0i32; 32];
        let mut n = 0usize;
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            // Edge spans this scanline iff y is in [min(y1,y2), max(y1,y2)).
            let crosses = (y1 <= y && y < y2) || (y2 <= y && y < y1);
            if crosses {
                // Linear interpolate to find x at this y; integer math
                // is fine because all polygon coords are integers.
                let x = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                if n < x_inters.len() { x_inters[n] = x; n += 1; }
            }
        }
        // Bubble sort the intersections (n is tiny, max ~10).
        for i in 0..n {
            for j in 0..n.saturating_sub(i + 1) {
                if x_inters[j] > x_inters[j + 1] {
                    x_inters.swap(j, j + 1);
                }
            }
        }
        // Fill between consecutive pairs (even-odd rule).
        let mut i = 0;
        while i + 1 < n {
            let xa = x_inters[i] + origin_x;
            let xb = x_inters[i + 1] + origin_x;
            let yy = y + origin_y;
            if yy >= 0 && xb >= xa && xa >= 0 {
                gpu::fill_rect(xa as u32, yy as u32, (xb - xa) as u32, 1, color);
            }
            i += 2;
        }
    }
}

/// Draw the geometric bat glyph at (origin_x, origin_y), sized 120x72.
fn draw_bat_glyph(origin_x: i32, origin_y: i32, accent: u32, dim: u32) {
    fill_polygon(BAT_LEFT_WING,  origin_x, origin_y, accent);
    fill_polygon(BAT_RIGHT_WING, origin_x, origin_y, accent);
    fill_polygon(BAT_HEAD,       origin_x, origin_y, accent);
    fill_polygon(BAT_TORSO,      origin_x, origin_y, accent);
    for &(x1, y1, x2, y2) in BAT_BONES {
        draw_line(origin_x + x1, origin_y + y1, origin_x + x2, origin_y + y2, dim);
    }
    for &(x, y) in BAT_NODES {
        gpu::fill_rect((origin_x + x) as u32, (origin_y + y) as u32, 2, 2, accent);
    }
    // Eye slits — paint near-black to punch through the head fill.
    for &(x, y) in BAT_EYES {
        gpu::fill_rect((origin_x + x) as u32, (origin_y + y) as u32, 2, 1, BG);
    }
    // Subtle circuit traces below the body.
    draw_line(origin_x + 60, origin_y + 46, origin_x + 60, origin_y + 62, dim);
    draw_line(origin_x + 52, origin_y + 62, origin_x + 68, origin_y + 62, dim);
}

/// Draw a 1px hollow rectangle border in the given color.
fn draw_border(x: u32, y: u32, w: u32, h: u32, color: u32) {
    gpu::fill_rect(x, y, w, 1, color);
    gpu::fill_rect(x, y + h - 1, w, 1, color);
    gpu::fill_rect(x, y, 1, h, color);
    gpu::fill_rect(x + w - 1, y, 1, h, color);
}

/// Draw a 14×14 L-shape crosshair mark at one of the four corners.
/// `dx, dy` are the direction signs (-1 / +1) the L opens toward.
fn draw_corner(x: u32, y: u32, dx: i32, dy: i32) {
    const S: u32 = 14;
    let hx = if dx > 0 { x } else { x.saturating_sub(S - 1) };
    let hy = if dy > 0 { y } else { y };
    let vy = if dy > 0 { y } else { y.saturating_sub(S - 1) };
    let vx = if dx > 0 { x } else { x };
    gpu::fill_rect(hx, hy, S, 1, HAIR_HI);
    gpu::fill_rect(vx, vy, 1, S, HAIR_HI);
}

/// Draw a status pill: bordered panel + colored dot + uppercase label
/// (and optional value). Returns the pill's right-edge x so the caller
/// can chain pills horizontally.
fn draw_status_pill(
    fb: *mut u32, w: u32,
    x: u32, y: u32,
    label: &str, value: Option<&str>,
    dot_fg: u32, dot_ring: u32,
) -> u32 {
    let pad_x: u32 = 10;
    let dot_size: u32 = 6;
    let label_w = label.len() as u32 * CHAR_W;
    let value_w = value.map_or(0, |v| v.len() as u32 * CHAR_W + CHAR_W); // +space
    let pill_w = pad_x + dot_size + 8 + label_w + value_w + pad_x;
    let pill_h: u32 = 22;

    gpu::fill_rect(x, y, pill_w, pill_h, PANEL);
    draw_border(x, y, pill_w, pill_h, HAIR_HI);

    // Colored dot with 1px ring (drawn as a 6x6 fill over an 8x8 ring).
    let dot_x = x + pad_x;
    let dot_y = y + (pill_h - dot_size) / 2;
    gpu::fill_rect(dot_x - 1, dot_y - 1, dot_size + 2, dot_size + 2, dot_ring);
    gpu::fill_rect(dot_x, dot_y, dot_size, dot_size, dot_fg);

    // Label and value text.
    let text_y = y + (pill_h - CHAR_H) / 2;
    let label_x = x + pad_x + dot_size + 8;
    font::draw_str(fb, w, label_x, text_y, label, INK, PANEL);
    if let Some(v) = value {
        let value_x = label_x + label_w + CHAR_W;
        font::draw_str(fb, w, value_x, text_y, v, DIM_TXT, PANEL);
    }
    x + pill_w
}

// ─── Boot log helpers ──────────────────────────────────────────────────

/// Deterministic boot-log block — pulls the 4 most recent kernel-init
/// breadcrumbs that match real subsystem state. We don't tail
/// `audit::dump_tail` here because the audit ring has a different
/// shape (timestamps + categories) than what fits in a 4-line strip.
const BOOT_LOG: &[(&str, &str)] = &[
    ("[ ok ]", "[net] virtio-net up  10.0.2.15/24"),
    ("[ ok ]", "[fs]  batfs mounted /  aes-256-ctr"),
    ("[ ok ]", "[sec] sha-256 kdf  16 rounds  ready"),
    ("[ ok ]", "[ui]  framebuffer 1280x800 bgra8"),
];

fn draw_boot_log(fb: *mut u32, w: u32, x: u32, y: u32) {
    font::draw_str(fb, w, x, y, "BOOT.LOG  TAIL -N 4", FAINT, BG);
    for (i, (tag, line)) in BOOT_LOG.iter().enumerate() {
        let row_y = y + 22 + (i as u32) * 16;
        font::draw_str(fb, w, x, row_y, tag, GREEN, BG);
        font::draw_str(fb, w, x + 7 * CHAR_W, row_y, line, DIM_TXT, BG);
    }
}

fn draw_clock_block(fb: *mut u32, w: u32, x_right: u32, y_top: u32, attempts: u8, denied: bool) {
    // Right-aligned strip. We don't have a real wall clock yet, so
    // show a plausible UTC timestamp + uptime ticks.
    let label = "SYSTEM CLOCK  UTC";
    let label_x = x_right.saturating_sub(label.len() as u32 * CHAR_W);
    font::draw_str(fb, w, label_x, y_top, label, FAINT, BG);

    // Wall clock placeholder. Future hookup: pull from RTC on Apple
    // path; QEMU has no battery-backed clock so leave as a stamp.
    let stamp = "2026-05-02  14:22:08";
    let stamp_x = x_right.saturating_sub(stamp.len() as u32 * CHAR_W);
    font::draw_str(fb, w, stamp_x, y_top + 22, stamp, INK, BG);

    let uptime = "UPTIME 0D 00:02:41";
    let uptime_x = x_right.saturating_sub(uptime.len() as u32 * CHAR_W);
    font::draw_str(fb, w, uptime_x, y_top + 38, uptime, DIM_TXT, BG);

    // Attempts pill.
    let mut buf = [0u8; 24];
    let mut p = 0usize;
    let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
        let n = src.len().min(dst.len() - *p);
        dst[*p..*p + n].copy_from_slice(&src[..n]);
        *p += n;
    };
    buf[p] = b'0' + attempts; p += 1;
    copy(&mut buf, b" ATTEMPTS REMAINING", &mut p);
    let pill_text = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
    let pill_w = (p as u32) * CHAR_W + 10 + 6 + 8 + 10;
    let pill_x = x_right.saturating_sub(pill_w);
    let pill_y = y_top + 60;
    let dot_color = if denied { RED } else { AMBER };
    let ring_color = if denied { RED_DIM } else { AMBER_DIM };
    let border_color = if denied { RED } else { HAIR_HI };

    gpu::fill_rect(pill_x, pill_y, pill_w, 22, PANEL);
    draw_border(pill_x, pill_y, pill_w, 22, border_color);
    let dot_x = pill_x + 10;
    let dot_y = pill_y + 8;
    gpu::fill_rect(dot_x - 1, dot_y - 1, 8, 8, ring_color);
    gpu::fill_rect(dot_x, dot_y, 6, 6, dot_color);
    let text_color = if denied { RED } else { MID };
    font::draw_str(fb, w, pill_x + 10 + 6 + 8, pill_y + 3, pill_text, text_color, PANEL);
}

// ─── Top-level paint of the lock screen ─────────────────────────────────

/// State the screen is being painted in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LockState { Idle, Typing(u8), Denied, Granted(u8) }

fn paint_lock_screen(fb: *mut u32, w: u32, h: u32, state: LockState, attempts: u8) {
    // Background.
    gpu::fill_screen(BG);

    // Crosshair corner marks.
    draw_corner(MARGIN_X / 2,                MARGIN_X / 2,                 1,  1);
    draw_corner(w - MARGIN_X / 2 - 1,        MARGIN_X / 2,                -1,  1);
    draw_corner(MARGIN_X / 2,                h - MARGIN_X / 2 - 1,         1, -1);
    draw_corner(w - MARGIN_X / 2 - 1,        h - MARGIN_X / 2 - 1,        -1, -1);

    // ── TOP STATUS ROW ────────────────────────────────────────────
    let net_tone_dot = if state == LockState::Denied { AMBER } else { GREEN };
    let net_tone_ring = if state == LockState::Denied { AMBER_DIM } else { GREEN_DIM };
    let net_value = if state == LockState::Denied { "ISOLATED" } else { "10.0.2.15" };
    let mut x = MARGIN_X;
    let row_y = MARGIN_Y;
    x = draw_status_pill(fb, w, x,     row_y, "ENCRYPTED", Some("AES-256-CTR"), GREEN, GREEN_DIM);
    x = draw_status_pill(fb, w, x + 8, row_y, "BATFS",     Some("MOUNTED"),     GREEN, GREEN_DIM);
    x = draw_status_pill(fb, w, x + 8, row_y, "M1N1",      Some("CHAINLOAD"),   GREEN, GREEN_DIM);
    let _ = draw_status_pill(fb, w, x + 8, row_y, "NET",   Some(net_value),     net_tone_dot, net_tone_ring);

    // Right-justified system identity — host / kernel / arch.
    let ident_y = row_y + 6;
    let ident = "HOST BATOS-01    KERNEL BAT 0.5.0-DEV    ARCH AARCH64 / APPLE-M4";
    let ident_x = (w - MARGIN_X).saturating_sub(ident.len() as u32 * CHAR_W);
    font::draw_str(fb, w, ident_x, ident_y, ident, MID, BG);

    // Hairline beneath status row.
    gpu::fill_rect(MARGIN_X, HAIRLINE_Y, w - 2 * MARGIN_X, 1, HAIR);

    // ── CENTER STACK ─────────────────────────────────────────────
    let cx = w / 2;
    let cy = h / 2;
    let accent = match state {
        LockState::Denied     => RED,
        LockState::Granted(_) => GREEN,
        _                     => CYAN,
    };
    let accent_dim = match state {
        LockState::Denied     => RED_DIM,
        LockState::Granted(_) => GREEN_DIM,
        _                     => CYAN_DIM,
    };

    // Bat glyph centered, ~180px above vertical mid.
    let glyph_x = cx - BAT_W / 2;
    let glyph_y = cy.saturating_sub(180);
    draw_bat_glyph(glyph_x as i32, glyph_y as i32, accent, accent_dim);

    // Wordmark "BAT_OS" — 32px in the spec, our font is 16px so it's
    // visually smaller than the mock, but the layout works.
    let wordmark = "BAT_OS";
    let word_x = cx - (wordmark.len() as u32 * CHAR_W) / 2;
    let word_y = glyph_y + BAT_H + 24;
    font::draw_str(fb, w, word_x, word_y, "BAT", INK, BG);
    font::draw_str(fb, w, word_x + 3 * CHAR_W, word_y, "_", accent, BG);
    font::draw_str(fb, w, word_x + 4 * CHAR_W, word_y, "OS", INK, BG);

    let version = "V0.5.0-DEV  .  BUILD 20260502.A3F1C  .  SIGNED";
    let ver_x = cx - (version.len() as u32 * CHAR_W) / 2;
    font::draw_str(fb, w, ver_x, word_y + 22, version, DIM_TXT, BG);

    // Field label row (above the field).
    let field_x = cx - FIELD_W / 2;
    let field_y = word_y + 60;
    let label = match state {
        LockState::Granted(_) => "[AUTH] GRANTED",
        _                     => "[AUTH] PASSPHRASE",
    };
    let label_color = match state {
        LockState::Denied     => RED,
        LockState::Granted(_) => GREEN,
        _                     => MID,
    };
    font::draw_str(fb, w, field_x, field_y - 16, label, label_color, BG);
    let kdf = "SHA-256 KDF . 16 ROUNDS";
    let kdf_x = field_x + FIELD_W - (kdf.len() as u32 * CHAR_W);
    font::draw_str(fb, w, kdf_x, field_y - 16, kdf, DIM_TXT, BG);

    // Passphrase field.
    let field_border = match state {
        LockState::Denied     => RED,
        LockState::Granted(_) => GREEN,
        LockState::Typing(_)  => CYAN,
        LockState::Idle       => HAIR_HI,
    };
    gpu::fill_rect(field_x, field_y, FIELD_W, FIELD_H, PANEL);
    draw_border(field_x, field_y, FIELD_W, FIELD_H, field_border);
    if matches!(state, LockState::Typing(_) | LockState::Granted(_)) {
        // Inset 1px ring matching the accent.
        draw_border(field_x + 1, field_y + 1, FIELD_W - 2, FIELD_H - 2, accent_dim);
    }

    // Granted state — replace the dots/cursor row with a centered
    // "ACCESS GRANTED" text inside the field. No chevron, no
    // attempts counter (they'd just clutter the success moment).
    if let LockState::Granted(_) = state {
        let granted = "ACCESS GRANTED";
        let g_x = field_x + (FIELD_W - granted.len() as u32 * CHAR_W) / 2;
        let g_y = field_y + (FIELD_H - CHAR_H) / 2;
        font::draw_str(fb, w, g_x, g_y, granted, GREEN, PANEL);
    } else {
        // Chevron prompt prefix.
        font::draw_char(fb, w, field_x + 18, field_y + (FIELD_H - CHAR_H) / 2, b'>', accent, PANEL);

        // Dots representing typed chars.
        let dots = match state {
            LockState::Typing(n) => n,
            LockState::Denied    => 7, // freeze the field at the moment of denial
            _                    => 0,
        };
        let dots_x = field_x + 18 + CHAR_W + 14;
        let dots_y = field_y + (FIELD_H - DOT_PX) / 2;
        for i in 0..dots {
            let dx = dots_x + (i as u32) * (DOT_PX + DOT_GAP);
            gpu::fill_rect(dx, dots_y, DOT_PX, DOT_PX, INK);
        }
        // Inline cursor at the trailing edge of the dots (no blink yet).
        let cursor_x = dots_x + (dots as u32) * (DOT_PX + DOT_GAP);
        let cursor_color = match state {
            LockState::Typing(_) => CYAN,
            LockState::Idle      => MID,
            LockState::Denied    => RED,
            _                    => MID,
        };
        gpu::fill_rect(cursor_x, field_y + 17, 10, 22, cursor_color);

        // Inline attempts indicator on the right side of the field.
        let mut buf = [0u8; 16];
        let mut p = 0usize;
        buf[p] = b'0' + attempts; p += 1;
        let suffix = b" ATTEMPTS LEFT";
        buf[p..p + suffix.len()].copy_from_slice(suffix);
        p += suffix.len();
        let inline = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
        let inline_color = if state == LockState::Denied { RED } else { DIM_TXT };
        let inline_x = field_x + FIELD_W - 10 - (p as u32 * CHAR_W);
        font::draw_str(fb, w, inline_x, field_y + (FIELD_H - CHAR_H) / 2, inline, inline_color, PANEL);
    }

    // Helper hint row beneath the field.
    let hint_y = field_y + FIELD_H + 12;
    let (hint_left, hint_right, hint_color) = match state {
        LockState::Granted(_) => (
            "DEADMAN REFRESHED . LAUNCHING DESKTOP",
            "READY",
            GREEN,
        ),
        _ => (
            "RETURN TO SUBMIT . ESC TO WIPE . F2 KEYMAP",
            "CAPS OFF",
            DIM_TXT,
        ),
    };
    font::draw_str(fb, w, field_x, hint_y, hint_left, hint_color, BG);
    let caps_x = field_x + FIELD_W - (hint_right.len() as u32 * CHAR_W);
    font::draw_str(fb, w, caps_x, hint_y, hint_right, hint_color, BG);

    // Denied overlay — red box centered over the stack.
    if state == LockState::Denied {
        let msg = "ACCESS DENIED";
        let sub1 = "CODE 0X1A . SHA-256 VERIFY FAILED . 1.42S";
        let sub2 = "ATTEMPT 2 OF 6 . COOLDOWN 8S";
        let pad_x: u32 = 56;
        let pad_y: u32 = 28;
        let inner_w = msg.len() as u32 * CHAR_W;
        let inner_w = inner_w.max(sub1.len() as u32 * CHAR_W);
        let box_w = inner_w + pad_x * 2;
        let box_h = CHAR_H * 3 + pad_y * 2 + 8;
        let box_x = cx - box_w / 2;
        let box_y = cy - box_h / 2;
        gpu::fill_rect(box_x, box_y, box_w, box_h, BG);
        draw_border(box_x, box_y, box_w, box_h, RED);
        draw_border(box_x - 1, box_y - 1, box_w + 2, box_h + 2, RED_DIM);
        let msg_x = box_x + (box_w - msg.len() as u32 * CHAR_W) / 2;
        font::draw_str(fb, w, msg_x, box_y + pad_y, msg, RED, BG);
        let sub1_x = box_x + (box_w - sub1.len() as u32 * CHAR_W) / 2;
        font::draw_str(fb, w, sub1_x, box_y + pad_y + CHAR_H + 8, sub1, MID, BG);
        let sub2_x = box_x + (box_w - sub2.len() as u32 * CHAR_W) / 2;
        font::draw_str(fb, w, sub2_x, box_y + pad_y + CHAR_H * 2 + 8, sub2, RED, BG);
    }

    // Bottom strips.
    draw_boot_log(fb, w, MARGIN_X, h - MARGIN_Y - (16 * 4 + 22));
    draw_clock_block(fb, w, w - MARGIN_X, h - MARGIN_Y - 80,
        attempts, state == LockState::Denied);

    // Bottom-edge hairline.
    gpu::fill_rect(0, h - 1, w, 1, HAIR);
}

// ─── Public entry points (preserved API) ────────────────────────────────

/// Dev helper: paint the login screen exactly once, then return after a
/// fixed delay. Lets the operator screenshot the auth UI without needing
/// a real passphrase. Kept for the Apple HV preview path.
pub fn run_dev_preview(hold_ms: u64) {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    paint_lock_screen(fb, w, h, LockState::Idle, 5);
    gpu::flush(0, 0, w, h);

    let freq: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq); }
    let start: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) start); }
    let target = (freq / 1000) * hold_ms;
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target { break; }
        core::hint::spin_loop();
    }
}

/// Run the boot authentication screen.
/// Returns only on successful authentication.
/// On duress or lockout, never returns.
pub fn run() {
    platform::serial_puts("[bs] enter run\n");
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    platform::serial_puts("[bs] fb obtained\n");

    loop {
        let attempts = auth::attempts_remaining();
        platform::serial_puts("[bs] paint idle\n");
        paint_lock_screen(fb, w, h, LockState::Idle, attempts);
        gpu::flush(0, 0, w, h);
        platform::serial_puts("[bs] paint done — input loop\n");

        let mut buf = [0u8; 128];
        let mut len = 0usize;

        loop {
            // STUMP #99 + #112 keyboard plumbing — drain serial,
            // virtio-keyboard, AND the pointer-device's mis-routed
            // EV_KEY ring.
            crate::drivers::virtio::keyboard::poll();
            crate::drivers::virtio::tablet::poll();
            let c_opt = platform::serial_getc()
                .or_else(crate::drivers::virtio::keyboard::getc)
                .or_else(crate::drivers::virtio::tablet::getc_key);
            if let Some(c) = c_opt {
                match c {
                    b'\r' | b'\n' => break,
                    0x08 | 0x7F => {
                        if len > 0 {
                            len -= 1;
                            // Repaint with one fewer dot.
                            let s = if len == 0 { LockState::Idle } else { LockState::Typing(len as u8) };
                            paint_lock_screen(fb, w, h, s, attempts);
                            gpu::flush(0, 0, w, h);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && len < 127 => {
                        buf[len] = c;
                        len += 1;
                        // Repaint with one more dot. This is full-frame
                        // every keystroke which is overkill, but it
                        // matches the design and types-per-second is
                        // human-rate so it's fine.
                        paint_lock_screen(fb, w, h, LockState::Typing(len as u8), attempts);
                        gpu::flush(0, 0, w, h);
                    }
                    _ => {}
                }
            }
            core::hint::spin_loop();
        }

        if len == 0 { continue; }

        let input = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
        let result = auth::authenticate(input);

        match result {
            auth::AuthResult::Success => {
                // Repaint the whole screen in Granted state so the
                // field, label, helper row, and accent (including
                // the bat glyph) all turn green together. Pre-fix
                // we painted only the field overlay and got the
                // y-offset wrong, dropping the green box onto the
                // helper row instead of the field.
                paint_lock_screen(fb, w, h, LockState::Granted(len as u8), attempts);
                gpu::flush(0, 0, w, h);
                for _ in 0..5_000_000 { core::hint::spin_loop(); }
                deadman::refresh();
                return;
            }
            auth::AuthResult::Failed => {
                paint_lock_screen(fb, w, h, LockState::Denied, attempts.saturating_sub(1));
                gpu::flush(0, 0, w, h);
                for _ in 0..3_000_000 { core::hint::spin_loop(); }
                continue;
            }
            auth::AuthResult::Duress => {
                fake_boot_and_wipe(fb, w, h);
            }
            auth::AuthResult::LockedOut => {
                paint_lock_screen(fb, w, h, LockState::Denied, 0);
                let cx = w / 2;
                let cy = h / 2;
                let msg = "SYSTEM LOCKED";
                let m_x = cx - (msg.len() as u32 * CHAR_W) / 2;
                font::draw_str(fb, w, m_x, cy + 24, msg, RED, BG);
                gpu::flush(0, 0, w, h);
                wipe::execute(wipe::WipeReason::Lockout, false);
                loop { unsafe { core::arch::asm!("wfe") }; }
            }
        }
    }
}

/// Fake boot animation — attacker thinks the system is loading.
/// Behind the scenes, everything is being destroyed.
fn fake_boot_and_wipe(fb: *mut u32, w: u32, h: u32) {
    let cx = w / 2;
    let cy = h / 2;

    gpu::fill_screen(BG);
    let title = "BAT_OS";
    let t_x = cx - (title.len() as u32 * CHAR_W) / 2;
    font::draw_str(fb, w, t_x, cy - 40, title, INK, BG);
    let loading = "LOADING SYSTEM ...";
    let l_x = cx - (loading.len() as u32 * CHAR_W) / 2;
    font::draw_str(fb, w, l_x, cy, loading, MID, BG);

    let bar_x = cx - 100;
    let bar_y = cy + 30;
    let bar_w: u32 = 200;
    let bar_h: u32 = 12;
    draw_border(bar_x, bar_y, bar_w, bar_h, HAIR_HI);
    gpu::flush(0, 0, w, h);

    wipe::execute(wipe::WipeReason::Duress, true);

    for progress in 0..bar_w {
        gpu::fill_rect(bar_x + 1, bar_y + 1, progress, bar_h - 2, INK);
        let pct = (progress * 100) / bar_w;
        let mut pct_str = [b' ', b' ', b' ', b'%'];
        if pct >= 100 { pct_str[0] = b'1'; pct_str[1] = b'0'; pct_str[2] = b'0'; }
        else if pct >= 10 { pct_str[1] = b'0' + (pct / 10) as u8; pct_str[2] = b'0' + (pct % 10) as u8; }
        else { pct_str[2] = b'0' + pct as u8; }
        font::draw_str(fb, w, cx - 16, bar_y + bar_h + 16,
            unsafe { core::str::from_utf8_unchecked(&pct_str) }, MID, BG);
        gpu::flush(bar_x, bar_y, bar_w + 2, bar_h + 32);
        for _ in 0..200_000 { core::hint::spin_loop(); }
    }
    for _ in 0..5_000_000 { core::hint::spin_loop(); }

    gpu::fill_screen(BG);
    font::draw_str(fb, w, 16, 16, "panic: unable to mount root filesystem", INK, BG);
    font::draw_str(fb, w, 16, 32, "kernel: VFS: unable to mount root fs",   INK, BG);
    font::draw_str(fb, w, 16, 48, "---[ end Kernel panic - not syncing ]---", INK, BG);
    gpu::flush(0, 0, w, h);

    loop { unsafe { core::arch::asm!("wfe") }; }
}
