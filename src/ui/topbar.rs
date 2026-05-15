//! Wave-2 top bar (22-px strip).

#![allow(dead_code)]

use crate::ui::draw;
use crate::ui::font;
use crate::ui::gpu;
use crate::ui::topbar_config::{self, Badge};

pub const TOPBAR_H: u32 = 22;

const PANEL:    u32 = 0xFF18181C;
const HAIRLINE: u32 = 0xFF2A2A30;
const INK:      u32 = 0xFFE5E7EB;
const MID:      u32 = 0xFF9CA3AF;
const DIM:      u32 = 0xFF6B7280;

const PAD_X:     u32 = 12;
const BADGE_GAP: u32 = 12;

pub enum TopBarHit { BrandClick, ConfigClick, LockClick, None }

pub fn paint() {
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    gpu::fill_rect(0, 0, screen_w, TOPBAR_H, PANEL);
    gpu::fill_rect(0, TOPBAR_H - 1, screen_w, 1, HAIRLINE);

    let brand = "SPHRAGIS";
    font::draw_str(fb, screen_w, PAD_X, (TOPBAR_H - 16) / 2 + 4, brand, INK, PANEL);

    let mut items: alloc::vec::Vec<(alloc::string::String, u32)> = alloc::vec::Vec::new();
    for b in topbar_config::iter() {
        let text = render_badge(b);
        let color = if badge_is_alert(b) { INK } else { MID };
        items.push((text, color));
    }
    items.push((alloc::string::String::from("..."), DIM));
    items.push((alloc::string::String::from("[L]"), DIM));

    let mut total_w = 0u32;
    for (text, _) in &items {
        total_w += text.len() as u32 * 8 + BADGE_GAP;
    }
    if total_w > 0 { total_w -= BADGE_GAP; }
    let mut x = screen_w.saturating_sub(PAD_X).saturating_sub(total_w);
    for (text, color) in &items {
        font::draw_str(fb, screen_w, x, (TOPBAR_H - 16) / 2 + 4, text, *color, PANEL);
        x += text.len() as u32 * 8 + BADGE_GAP;
    }
}

pub fn hit_test(mx: i32, my: i32) -> TopBarHit {
    if my < 0 || (my as u32) >= TOPBAR_H { return TopBarHit::None; }
    let screen_w = gpu::width() as i32;

    let brand_w = 8 * 8;
    if mx >= PAD_X as i32 && mx < (PAD_X as i32) + brand_w {
        return TopBarHit::BrandClick;
    }

    let lock_w = 3 * 8;
    let lock_x1 = screen_w - PAD_X as i32;
    let lock_x0 = lock_x1 - lock_w;
    if mx >= lock_x0 && mx < lock_x1 { return TopBarHit::LockClick; }

    let config_w = 3 * 8;
    let config_x1 = lock_x0 - BADGE_GAP as i32;
    let config_x0 = config_x1 - config_w;
    if mx >= config_x0 && mx < config_x1 { return TopBarHit::ConfigClick; }

    TopBarHit::None
}

fn render_badge(b: Badge) -> alloc::string::String {
    use alloc::format;
    match b {
        Badge::NetMode => {
            if crate::net::is_isolated() {
                alloc::string::String::from("NET ISOLATED")
            } else {
                alloc::string::String::from("NET ROUTED")
            }
        }
        Badge::Deadman => {
            let secs = crate::security::deadman::seconds_remaining();
            format!("DEADMAN {:02}:{:02}", secs / 60, secs % 60)
        }
        Badge::Clock => {
            // Wave-2 placeholder: HH:MM derived from uptime (mod 24h), not
            // real wall clock. The user-facing badge name stays "CLOCK" per
            // spec; the displayed value gains true clock semantics in a
            // later wave when an RTC source is wired up.
            let secs = uptime_seconds();
            format!("{:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60)
        }
        Badge::Caves    => format!("CAVES {}", crate::caves::count()),
        Badge::Attempts => format!("ATTEMPTS {}", crate::security::auth::attempts_remaining()),
        Badge::Memory   => alloc::string::String::from("MEM --"),
        Badge::Cpu      => alloc::string::String::from("CPU --"),
        Badge::Audit    => alloc::string::String::from("AUDIT --"),
        Badge::CaveFocus => {
            let cave = crate::ui::wm::focused()
                .and_then(|id| crate::ui::wm::get(id))
                .and_then(|w| w.cave_name);
            match cave {
                Some(bytes) => {
                    let n = bytes.iter().position(|&b| b == 0).unwrap_or(16);
                    format!("CAVE {}", unsafe { core::str::from_utf8_unchecked(&bytes[..n]) })
                }
                None => alloc::string::String::from("CAVE --"),
            }
        }
    }
}

fn badge_is_alert(b: Badge) -> bool {
    match b {
        Badge::Deadman  => crate::security::deadman::seconds_remaining() < 300,
        Badge::Attempts => crate::security::auth::attempts_remaining() < 3,
        _ => false,
    }
}

/// Seconds elapsed since boot via `CNTPCT_EL0` / `CNTFRQ_EL0`.
/// **NOT wall-clock time.** Used by `Badge::Clock` as a placeholder
/// until real RTC support lands (Wave 3+).
fn uptime_seconds() -> u64 {
    let now: u64; let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { 0 } else { now / freq }
}

// ── Config sheet (modal) ─────────────────────────────────────────

static mut CONFIG_SHEET_OPEN: bool = false;

pub fn config_sheet_open() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CONFIG_SHEET_OPEN)) }
}
pub fn open_config_sheet() {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(CONFIG_SHEET_OPEN), true) }
}
pub fn close_config_sheet() {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(CONFIG_SHEET_OPEN), false) }
}

const BG: u32 = 0xFF0D0D10;
const ALL_BADGES: &[(Badge, &str)] = &[
    (Badge::NetMode,   "NET MODE"),
    (Badge::Deadman,   "DEADMAN"),
    (Badge::Clock,     "CLOCK"),
    (Badge::Caves,     "CAVES COUNT"),
    (Badge::Attempts,  "ATTEMPTS"),
    (Badge::Memory,    "MEMORY"),
    (Badge::Cpu,       "CPU"),
    (Badge::Audit,     "AUDIT TAIL"),
    (Badge::CaveFocus, "CAVE FOCUS"),
];

pub fn paint_config_sheet() {
    if !config_sheet_open() { return; }

    let screen_w = gpu::width();
    let screen_h = gpu::height();
    let fb = gpu::framebuffer();

    gpu::fill_rect(0, TOPBAR_H, screen_w, screen_h - TOPBAR_H, BG);

    let (px, py, panel_w, panel_h, row_h) = config_sheet_geometry(screen_w, screen_h);
    gpu::fill_rect(px, py, panel_w, panel_h, PANEL);
    draw::draw_border(px, py, panel_w, panel_h, HAIRLINE);

    font::draw_str(fb, screen_w, px + 14, py + 10, "TOP BAR BADGES", INK, PANEL);

    for (i, (badge, name)) in ALL_BADGES.iter().enumerate() {
        let ry = py + 40 + (i as u32) * row_h;
        let enabled = topbar_config::iter().any(|b| b == *badge);
        let marker = if enabled { "[x]" } else { "[ ]" };
        let color = if enabled { INK } else { DIM };
        font::draw_str(fb, screen_w, px + 14, ry,             marker, color, PANEL);
        font::draw_str(fb, screen_w, px + 14 + 4 * 8, ry,     name,   color, PANEL);
    }

    font::draw_str(fb, screen_w, px + 14, py + panel_h - 16, "ESC TO CLOSE", DIM, PANEL);
}

/// Returns `true` if a badge row was toggled (repaint needed).
/// Returns `false` if the click did NOT land on a row — the caller
/// should treat this as a close-sheet request.
pub fn config_sheet_click(mx: i32, my: i32) -> bool {
    if !config_sheet_open() { return false; }
    let screen_w = gpu::width();
    let screen_h = gpu::height();

    let (px, py, panel_w, _panel_h, row_h) = config_sheet_geometry(screen_w, screen_h);

    if (mx as u32) < px || (mx as u32) >= px + panel_w { return false; }
    if (my as u32) < py + 40 || (my as u32) >= py + 40 + (ALL_BADGES.len() as u32) * row_h {
        return false;
    }

    let row_idx = (((my as u32) - py - 40) / row_h) as usize;
    if row_idx < ALL_BADGES.len() {
        topbar_config::toggle(ALL_BADGES[row_idx].0);
        return true;
    }
    false
}

#[inline(always)]
fn config_sheet_geometry(screen_w: u32, screen_h: u32) -> (u32, u32, u32, u32, u32) {
    let panel_w: u32 = 360;
    let row_h:   u32 = 24;
    let panel_h  = (ALL_BADGES.len() as u32) * row_h + 50;
    let px = (screen_w - panel_w) / 2;
    let py = (screen_h - panel_h) / 2;
    (px, py, panel_w, panel_h, row_h)
}

