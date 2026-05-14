//! Wave-2 top bar (22-px strip).

#![allow(dead_code)]

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

fn uptime_seconds() -> u64 {
    let now: u64; let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    if freq == 0 { 0 } else { now / freq }
}

