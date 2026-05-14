// Sphragis — shared paint primitives.
//
// Extracted from `security/boot_screen.rs` so the desktop
// chrome can re-use the same scanline polygon-fill + line
// rasterizer for the title-bar project glyph. Anything else that needs
// vector-ish drawing on top of `gpu::fill_rect` should live here.

#![allow(dead_code)]

use crate::ui::gpu;

// ─── Generic line / polygon / border helpers ────────────────────────

/// Bresenham line. Fine for 1-pixel diagonals; switch to a thicker
/// stamping primitive if we ever need bold strokes.
pub fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
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

/// Scanline polygon fill, no_std-friendly. Uses a fixed 32-slot
/// intersection buffer per scanline — the polygons we use have at
/// most 19 vertices, so 32 is plenty.
pub fn fill_polygon(points: &[(i32, i32)], origin_x: i32, origin_y: i32, color: u32) {
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
            let crosses = (y1 <= y && y < y2) || (y2 <= y && y < y1);
            if crosses {
                let x = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                if n < x_inters.len() { x_inters[n] = x; n += 1; }
            }
        }
        // Bubble sort the intersections (n is tiny).
        for i in 0..n {
            for j in 0..n.saturating_sub(i + 1) {
                if x_inters[j] > x_inters[j + 1] {
                    x_inters.swap(j, j + 1);
                }
            }
        }
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

/// Hollow 1-px rectangle border in the given color. Cheap helper that
/// keeps the four-fill_rect dance off every caller.
pub fn draw_border(x: u32, y: u32, w: u32, h: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    gpu::fill_rect(x, y, w, 1, color);
    gpu::fill_rect(x, y + h - 1, w, 1, color);
    gpu::fill_rect(x, y, 1, h, color);
    gpu::fill_rect(x + w - 1, y, 1, h, color);
}

// ─── Lock-screen project glyph (120×72 Σ monogram) ────────────────────────
//
// Renders a large Σ (Greek sigma — first letter of σφραγίς, "the seal")
// as the project monogram. The kernel's bitmap font is ASCII-only, so
// we rasterize Σ directly from two horizontal bars + two diagonal
// parallelograms, using the same `fill_polygon` primitive that used
// to draw the bat silhouette.
//
// Glyph is 64×64 pixels, centered in the 120×72 slot
// (28 px horizontal padding, 4 px vertical padding).
// Stroke width: 8 px.

pub const PROJECT_GLYPH_FULL_W: u32 = 120;
pub const PROJECT_GLYPH_FULL_H: u32 = 72;

const SIGMA_W:      u32 = 64;
const SIGMA_H:      u32 = 64;
const SIGMA_STROKE: u32 = 8;

// Σ has two diagonal strokes meeting at the right-center. We render
// each diagonal as a parallelogram so the stroke has constant
// horizontal width along its length.
//
// Coordinate frame: Σ's top-left at (0, 0), 64×64 box.
//   Top bar:    y=0..8        (filled as a rectangle, not in this list)
//   Upper diag: (0, 8)  →  (32, 32)  — 8px-wide parallelogram
//   Lower diag: (32, 32) →  (0, 56)  — 8px-wide parallelogram
//   Bottom bar: y=56..64      (filled as a rectangle)
const SIGMA_UPPER_DIAG: &[(i32, i32)] = &[
    ( 0,  8),
    ( 8,  8),
    (32, 32),
    (24, 32),
];
const SIGMA_LOWER_DIAG: &[(i32, i32)] = &[
    (24, 32),
    (32, 32),
    ( 8, 56),
    ( 0, 56),
];

/// Draw the lock-screen Σ monogram in `accent` color.
/// `_dim` and `_bg` are unused (Σ is a single-color solid mark) but
/// kept in the signature so the boot-screen caller's color-palette
/// plumbing stays unchanged.
pub fn draw_project_glyph_full(origin_x: i32, origin_y: i32, accent: u32, _dim: u32, _bg: u32) {
    // Center the 64×64 Σ inside the 120×72 slot.
    let pad_x = (PROJECT_GLYPH_FULL_W - SIGMA_W) as i32 / 2;
    let pad_y = (PROJECT_GLYPH_FULL_H - SIGMA_H) as i32 / 2;
    let cx = origin_x + pad_x;
    let cy = origin_y + pad_y;
    if cx < 0 || cy < 0 { return; }
    let ux = cx as u32;
    let uy = cy as u32;

    // Top bar.
    gpu::fill_rect(ux, uy, SIGMA_W, SIGMA_STROKE, accent);
    // Bottom bar.
    gpu::fill_rect(ux, uy + SIGMA_H - SIGMA_STROKE, SIGMA_W, SIGMA_STROKE, accent);
    // Two diagonals as parallelograms.
    fill_polygon(SIGMA_UPPER_DIAG, cx, cy, accent);
    fill_polygon(SIGMA_LOWER_DIAG, cx, cy, accent);
}

// ─── Title-bar mini glyph (18×12 stylized "S") ──────────────────────
//
// 18×12 is too small for the 8×16 bitmap font; we'd either clip the
// bottom four rows or shrink the character below legibility. So
// the title bar gets a hand-drawn 8×7 stylized "S" instead — same
// brand mark, sized for the slot.
//
// Replaced the earlier bat-silhouette raster as part of the Tier 3
// brand cleanup.

pub const PROJECT_GLYPH_MINI_W: u32 = 18;
pub const PROJECT_GLYPH_MINI_H: u32 = 12;

/// Stylized "S" pattern in an 8×7 grid. Each byte is a row;
/// bit 7 = leftmost pixel.
const S_PATTERN_8X7: [u8; 7] = [
    0b01111110, //  ######
    0b11000000, // ##
    0b11000000, // ##
    0b01111100, //  #####
    0b00000011, //       ##
    0b00000011, //       ##
    0b01111110, //  ######
];

/// Draw the title-bar "S" at native 18×12 resolution.
/// The S pattern is 8×7 — centered with `(18-8)/2 = 5` horizontal
/// padding and `(12-7)/2 = 2` vertical padding.
pub fn draw_project_glyph_mini(origin_x: u32, origin_y: u32, color: u32) {
    let ox = origin_x + (PROJECT_GLYPH_MINI_W - 8) / 2;
    let oy = origin_y + (PROJECT_GLYPH_MINI_H - 7) / 2;
    for (row_idx, &bits) in S_PATTERN_8X7.iter().enumerate() {
        for col in 0..8u32 {
            if bits & (0x80 >> col) != 0 {
                gpu::fill_rect(ox + col, oy + row_idx as u32, 1, 1, color);
            }
        }
    }
}

// Title-bar callers that want the "S" at full source resolution
// (36×24, i.e. 2x scale) for clarity at high pixel density.
pub const PROJECT_GLYPH_MINI_FULL_W: u32 = 36;
pub const PROJECT_GLYPH_MINI_FULL_H: u32 = 24;

/// Draw the title-bar "S" at 2x scale into the 36×24 slot.
pub fn draw_project_glyph_mini_full(origin_x: i32, origin_y: i32, color: u32) {
    if origin_x < 0 || origin_y < 0 { return; }
    let ox = (origin_x as u32) + (PROJECT_GLYPH_MINI_FULL_W - 16) / 2;
    let oy = (origin_y as u32) + (PROJECT_GLYPH_MINI_FULL_H - 14) / 2;
    for (row_idx, &bits) in S_PATTERN_8X7.iter().enumerate() {
        for col in 0..8u32 {
            if bits & (0x80 >> col) != 0 {
                // 2×2 block per source pixel.
                gpu::fill_rect(ox + col * 2, oy + (row_idx as u32) * 2, 2, 2, color);
            }
        }
    }
}
