// Sphragis — shared paint primitives.
//
// Extracted from `security/boot_screen.rs` so the desktop
// chrome can re-use the same scanline polygon-fill + line
// rasterizer for the title-bar bat glyph. Anything else that needs
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

// ─── Lock-screen bat glyph (120×72 detailed) ────────────────────────
//
// Translated from `docs/design/lock-screen/bat-glyph.jsx`.
// 4 polygons (left wing, right wing, head, torso) +
// 10 "finger-bone" lines + 13 circuit nodes + 2 eye slits.

pub const BAT_FULL_W: u32 = 120;
pub const BAT_FULL_H: u32 = 72;

const BAT_FULL_LEFT_WING: &[(i32, i32)] = &[
    (60, 22), (54, 18), (44, 14), (32, 10), (18,  8),
    ( 6, 14), ( 2, 24), (10, 28), ( 4, 34), (14, 38),
    ( 8, 46), (22, 46), (18, 54), (32, 50), (30, 58),
    (44, 52), (46, 58), (56, 50), (58, 42),
];
const BAT_FULL_RIGHT_WING: &[(i32, i32)] = &[
    (60, 22), (66, 18), (76, 14), (88, 10), (102,  8),
    (114, 14), (118, 24), (110, 28), (116, 34), (106, 38),
    (112, 46), (98, 46), (102, 54), (88, 50), (90, 58),
    (76, 52), (74, 58), (64, 50), (62, 42),
];
const BAT_FULL_HEAD: &[(i32, i32)] = &[
    (54, 18), (54, 8), (57, 14), (60, 4), (63, 14), (66, 8), (66, 18),
];
const BAT_FULL_TORSO: &[(i32, i32)] = &[
    (54, 18), (66, 18), (64, 38), (60, 46), (56, 38),
];
const BAT_FULL_BONES: &[(i32, i32, i32, i32)] = &[
    (56, 22,  18,  8), (56, 26,   6, 20), (56, 30,  10, 32),
    (56, 36,  14, 42), (56, 42,  22, 50),
    (64, 22, 102,  8), (64, 26, 114, 20), (64, 30, 110, 32),
    (64, 36, 106, 42), (64, 42,  98, 50),
];
const BAT_FULL_NODES: &[(i32, i32)] = &[
    ( 17,  7), (  5, 13), (  9, 27), ( 13, 37), ( 21, 45),
    (101,  7), (113, 13), (109, 27), (105, 37), ( 97, 45),
    ( 51, 61), ( 67, 61), ( 59, 61),
];
const BAT_FULL_EYES: &[(i32, i32)] = &[ (56, 13), (62, 13) ];

/// Draw the detailed 120×72 bat. `bg` is the body color the eye slits
/// should "punch through" to (matches the surrounding background).
pub fn draw_bat_full(origin_x: i32, origin_y: i32, accent: u32, dim: u32, bg: u32) {
    fill_polygon(BAT_FULL_LEFT_WING,  origin_x, origin_y, accent);
    fill_polygon(BAT_FULL_RIGHT_WING, origin_x, origin_y, accent);
    fill_polygon(BAT_FULL_HEAD,       origin_x, origin_y, accent);
    fill_polygon(BAT_FULL_TORSO,      origin_x, origin_y, accent);
    for &(x1, y1, x2, y2) in BAT_FULL_BONES {
        draw_line(origin_x + x1, origin_y + y1, origin_x + x2, origin_y + y2, dim);
    }
    for &(x, y) in BAT_FULL_NODES {
        gpu::fill_rect((origin_x + x) as u32, (origin_y + y) as u32, 2, 2, accent);
    }
    for &(x, y) in BAT_FULL_EYES {
        gpu::fill_rect((origin_x + x) as u32, (origin_y + y) as u32, 2, 1, bg);
    }
    draw_line(origin_x + 60, origin_y + 46, origin_x + 60, origin_y + 62, dim);
    draw_line(origin_x + 52, origin_y + 62, origin_x + 68, origin_y + 62, dim);
}

// ─── Title-bar mini bat (18×12 simplified) ──────────────────────────
//
// Translated from `docs/design/desktop-shell/bat-mini.jsx`.
// Membrane silhouette only — no finger bones, no eye slits, no
// circuit nodes (would collapse to noise at this raster).
//
// The source viewBox is 36×24 but the mock renders at 18×12 — half
// scale. We keep the vertices in 36×24 space and let `fill_polygon`
// stamp them; callers pass `origin_x, origin_y` as the glyph's
// top-left at full 36×24 resolution. The title-bar drawer uses a
// custom downscale path further down.

pub const BAT_MINI_W: u32 = 36;
pub const BAT_MINI_H: u32 = 24;

const BAT_MINI_LEFT_WING: &[(i32, i32)] = &[
    (18, 8), (16, 6), (12, 4), ( 6, 3), ( 1, 5),
    ( 0, 9), ( 3,10), ( 1,12), ( 5,13), ( 3,16),
    ( 8,16), ( 6,19), (11,17), (10,20), (15,18),
    (16,20), (17,17),
];
const BAT_MINI_RIGHT_WING: &[(i32, i32)] = &[
    (18, 8), (20, 6), (24, 4), (30, 3), (35, 5),
    (36, 9), (33,10), (35,12), (31,13), (33,16),
    (28,16), (30,19), (25,17), (26,20), (21,18),
    (20,20), (19,17),
];
const BAT_MINI_HEAD: &[(i32, i32)] = &[
    (16, 6), (16, 2), (17, 4), (18, 1), (19, 4), (20, 2), (20, 6),
];
const BAT_MINI_TORSO: &[(i32, i32)] = &[
    (16, 6), (20, 6), (19,14), (18,16), (17,14),
];

/// Draw the simplified 36×24 bat at full source resolution.
pub fn draw_bat_mini_full(origin_x: i32, origin_y: i32, color: u32) {
    fill_polygon(BAT_MINI_LEFT_WING,  origin_x, origin_y, color);
    fill_polygon(BAT_MINI_RIGHT_WING, origin_x, origin_y, color);
    fill_polygon(BAT_MINI_HEAD,       origin_x, origin_y, color);
    fill_polygon(BAT_MINI_TORSO,      origin_x, origin_y, color);
}

/// Draw the simplified bat at the title-bar's intended 18×12 raster.
/// Implementation: rasterize at 36×24 into a tiny stack buffer, then
/// downsample 2:1 with a "any source pixel set → output set" rule
/// (cheap and preserves the silhouette better than nearest-neighbor
/// sampling at this scale).
pub fn draw_bat_mini(origin_x: u32, origin_y: u32, color: u32) {
    // 36×24 = 864 cells. Pack as a static stack array — the function
    // only runs from the title-bar redraw which happens at most a few
    // times a second.
    let mut tile = [0u8; (BAT_MINI_W * BAT_MINI_H) as usize];

    // Inline scanline fill that writes 1 into `tile` for each filled
    // pixel. Could share code with fill_polygon, but tile-write vs
    // gpu::fill_rect is different enough that duplicating is clearer.
    let stamp = |poly: &[(i32, i32)], tile: &mut [u8]| {
        if poly.len() < 3 { return; }
        let mut min_y = poly[0].1;
        let mut max_y = poly[0].1;
        for &(_, y) in poly {
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }
        for y in min_y..max_y {
            let mut x_inters = [0i32; 16];
            let mut n = 0usize;
            for i in 0..poly.len() {
                let (x1, y1) = poly[i];
                let (x2, y2) = poly[(i + 1) % poly.len()];
                let crosses = (y1 <= y && y < y2) || (y2 <= y && y < y1);
                if crosses {
                    let x = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                    if n < x_inters.len() { x_inters[n] = x; n += 1; }
                }
            }
            for i in 0..n {
                for j in 0..n.saturating_sub(i + 1) {
                    if x_inters[j] > x_inters[j + 1] {
                        x_inters.swap(j, j + 1);
                    }
                }
            }
            let mut i = 0;
            while i + 1 < n {
                let xa = x_inters[i].max(0) as u32;
                let xb = (x_inters[i + 1].max(0) as u32).min(BAT_MINI_W);
                if (y as u32) < BAT_MINI_H {
                    for xx in xa..xb {
                        tile[((y as u32) * BAT_MINI_W + xx) as usize] = 1;
                    }
                }
                i += 2;
            }
        }
    };

    stamp(BAT_MINI_LEFT_WING,  &mut tile);
    stamp(BAT_MINI_RIGHT_WING, &mut tile);
    stamp(BAT_MINI_HEAD,       &mut tile);
    stamp(BAT_MINI_TORSO,      &mut tile);

    // Downsample 2:1 — any of the 4 source pixels in a 2×2 block being
    // set means the destination pixel is on. Preserves silhouette
    // better than picking a single sample at this size.
    let dst_w = BAT_MINI_W / 2;
    let dst_h = BAT_MINI_H / 2;
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = dx * 2;
            let sy = dy * 2;
            let any = tile[(sy * BAT_MINI_W + sx) as usize] != 0
                   || tile[(sy * BAT_MINI_W + sx + 1) as usize] != 0
                   || tile[((sy + 1) * BAT_MINI_W + sx) as usize] != 0
                   || tile[((sy + 1) * BAT_MINI_W + sx + 1) as usize] != 0;
            if any {
                gpu::fill_rect(origin_x + dx, origin_y + dy, 1, 1, color);
            }
        }
    }
}
