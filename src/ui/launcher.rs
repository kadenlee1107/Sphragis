//! Launcher overlay — 8-app grid summoned by ⌘K or click on the
//! SPHRAGIS brand. Paints on top of the desktop in OVERLAY state
//! and also IS the desktop in LAUNCHER state (no dim, no windows
//! behind).

#![allow(dead_code)]

use crate::ui::apps_registry::{AppId, APPS};
use crate::ui::font;
use crate::ui::gpu;
use crate::ui::topbar::TOPBAR_H;

const BG:      u32 = 0xFF0D0D10;
const INK:     u32 = 0xFFE5E7EB;
const MID:     u32 = 0xFF9CA3AF; // dimmed label text in Background mode
// TILE_BG shares MID's hex by coincidence — distinct semantic
// (tile fill, not text). Don't merge them in a future palette pass.
const TILE_BG: u32 = 0xFF9CA3AF;

const COLS: u32 = 4;
const ROWS: u32 = 2;
const TILE_W: u32 = 22;
const TILE_H: u32 = 22;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LauncherMode {
    Background,
    Overlay,
    Canvas,
}

struct GridLayout {
    inset_x: u32,
    inset_y: u32,
    grid_w:  u32,
    grid_h:  u32,
    cell_w:  u32,
    cell_h:  u32,
}

fn grid_layout(screen_w: u32, screen_h: u32) -> GridLayout {
    let inset_x = (screen_w * 12) / 100;
    let inset_y = TOPBAR_H + (screen_h - TOPBAR_H) * 4 / 100;
    let grid_w = screen_w.saturating_sub(2 * inset_x);
    let grid_h = (screen_h - TOPBAR_H) * 70 / 100;
    GridLayout {
        inset_x,
        inset_y,
        grid_w,
        grid_h,
        cell_w: grid_w / COLS,
        cell_h: grid_h / ROWS,
    }
}

pub fn paint(mode: LauncherMode) {
    let screen_w = gpu::width();
    let screen_h = gpu::height();

    if mode == LauncherMode::Overlay {
        gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, BG);
    }

    let GridLayout { inset_x, inset_y, cell_w, cell_h, .. } = grid_layout(screen_w, screen_h);

    let label_color = if mode == LauncherMode::Background { MID } else { INK };
    let tile_color  = if mode == LauncherMode::Background { 0xFF3A3B3F } else { TILE_BG };

    for (i, app) in APPS.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let cell_x = inset_x + col * cell_w;
        let cell_y = inset_y + row * cell_h;
        let cx = cell_x + cell_w / 2;
        let cy = cell_y + cell_h / 2;

        let tile_x = cx - TILE_W / 2;
        let tile_y = cy - TILE_H / 2 - 6;
        gpu::fill_rect(tile_x, tile_y, TILE_W, TILE_H, tile_color);

        let label = app.label;
        let lbl_x = cx - (label.len() as u32 * 8) / 2;
        let lbl_y = tile_y + TILE_H + 8;
        font::draw_str(gpu::framebuffer(), screen_w, lbl_x, lbl_y, label, label_color, BG);
    }
}

pub fn hit_test(mx: i32, my: i32) -> Option<AppId> {
    let screen_w = gpu::width();
    let screen_h = gpu::height();
    if my < TOPBAR_H as i32 { return None; }

    let GridLayout { inset_x, inset_y, grid_w, grid_h, cell_w, cell_h } = grid_layout(screen_w, screen_h);

    if (mx as u32) < inset_x || (mx as u32) >= inset_x + grid_w { return None; }
    if (my as u32) < inset_y || (my as u32) >= inset_y + grid_h { return None; }

    let col = ((mx as u32) - inset_x) / cell_w;
    let row = ((my as u32) - inset_y) / cell_h;
    let idx = (row * COLS + col) as usize;
    APPS.get(idx).map(|d| d.id)
}
