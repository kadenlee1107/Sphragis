//! Wave 4 NET cockpit. Live activity dashboard.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

extern crate alloc;

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_status_panel, StatusPanel, StatusField,
    paint_activity_log, ActivityEntry,
    paint_action_strip, action_strip_hit_test, Action,
};
use crate::ui::wm::WindowRect;
use crate::net;
use crate::net::activity::{self, ActivityKind};

static mut VIEWPORT_START: usize = 0;

fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}
fn set_viewport_start(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    let strip_h: u32 = 28;
    let strip_rect = WindowRect { x: body.x, y: body.y, w: body.w, h: strip_h };
    paint_stats_strip(strip_rect);
    crate::ui::gpu::fill_rect(body.x, body.y + strip_h, body.w, 1, p::HAIRLINE);

    let panel_y = body.y + strip_h + 12;
    let panel_h: u32 = 88;
    let gap: u32 = 12;
    let mode_w  = (body.w - 28 - gap) * 2 / 5;
    let fw_w    = (body.w - 28 - gap) - mode_w;
    let mode_rect = WindowRect { x: body.x + 14, y: panel_y, w: mode_w, h: panel_h };
    let fw_rect   = WindowRect { x: body.x + 14 + mode_w + gap, y: panel_y, w: fw_w, h: panel_h };
    paint_mode_panel(mode_rect);
    paint_firewall_panel(fw_rect);

    let log_y = panel_y + panel_h + 12;
    let log_h = body.h.saturating_sub(log_y - body.y + 50);
    let log_rect = WindowRect { x: body.x + 14, y: log_y, w: body.w - 28, h: log_h };
    paint_activity_block(log_rect);

    crate::ui::gpu::fill_rect(body.x + 14, body.y + body.h - 32, body.w - 28, 1, p::HAIRLINE);
    let action_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    paint_action_strip(action_rect, &actions());
}

fn paint_stats_strip(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let mut left_buf = [0u8; 64];
    let mut ln = 0;
    push_bytes(&mut left_buf, &mut ln, b"RX ");
    format_rate(net::rx_rate(), &mut left_buf, &mut ln);
    push_bytes(&mut left_buf, &mut ln, b"/s  \xc2\xb7  TX ");
    format_rate(net::tx_rate(), &mut left_buf, &mut ln);
    push_bytes(&mut left_buf, &mut ln, b"/s");
    let left = unsafe { core::str::from_utf8_unchecked(&left_buf[..ln]) };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 6, left, p::MID, p::BG);

    let mut right_buf = [0u8; 64];
    let mut rn = 0;
    push_bytes(&mut right_buf, &mut rn, b"PEAK ");
    format_bytes(net::peak_bytes(), &mut right_buf, &mut rn);
    push_bytes(&mut right_buf, &mut rn, b"  \xc2\xb7  UPTIME ");
    format_hms(net::uptime_secs(), &mut right_buf, &mut rn);
    let right = unsafe { core::str::from_utf8_unchecked(&right_buf[..rn]) };
    let right_w = rn as u32 * 8;
    font::draw_str(fb, screen_w, rect.x + rect.w.saturating_sub(right_w + 14), rect.y + 6, right, p::MID, p::BG);
}

fn paint_mode_panel(rect: WindowRect) {
    let mode_label = if net::is_isolated() { "ISOLATED" } else { "ROUTED" };
    let mode_sub = if net::is_isolated() {
        "no outbound to non-cave routes"
    } else {
        "default route via host"
    };
    let panel = StatusPanel {
        label: "MODE",
        header_right: None,
        body: &[],
    };
    paint_status_panel(rect, &panel);

    use crate::ui::font;
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x + 10, rect.y + 36, mode_label, p::INK, p::PANEL);
    font::draw_str(fb, screen_w, rect.x + 10, rect.y + 60, mode_sub,   p::MID, p::PANEL);
}

fn paint_firewall_panel(rect: WindowRect) {
    let body = [
        StatusField { key: "rules",     value: "12 allow \u{00B7} 0 deny" },
        StatusField { key: "drops",     value: "3 in last 60s" },
        StatusField { key: "last drop", value: "14:31:48 tcp 10.0.0.4:443" },
    ];
    let panel = StatusPanel {
        label: "FIREWALL",
        header_right: Some("default: DENY"),
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn paint_activity_block(rect: WindowRect) {
    use crate::ui::font;
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x, rect.y, "ACTIVITY", p::MID, p::BG);

    let total = activity::count();
    let viewport = viewport_start();
    let mut owned: alloc::vec::Vec<(alloc::string::String, alloc::string::String, alloc::string::String)> =
        alloc::vec::Vec::new();
    let mut row_index: usize = 0;
    activity::iter_newest_first(|entry| {
        if row_index >= viewport {
            use alloc::format;
            let kind = ActivityKind::from_u8(entry.kind);
            let ts = format!("{:02}:{:02}:{:02}",
                entry.ts / 3600,
                (entry.ts / 60) % 60,
                entry.ts % 60,
            );
            owned.push((
                ts,
                alloc::string::String::from(kind.as_str()),
                alloc::string::String::from(entry.summary_str()),
            ));
        }
        row_index += 1;
        true
    });
    let refs: alloc::vec::Vec<ActivityEntry> = owned.iter().map(|(t, k, s)| ActivityEntry {
        timestamp_str: t.as_str(),
        kind: k.as_str(),
        summary: s.as_str(),
    }).collect();
    let log_rect = WindowRect { x: rect.x, y: rect.y + 4, w: rect.w, h: rect.h.saturating_sub(4) };
    paint_activity_log(log_rect, &refs, viewport, total);
}

fn actions() -> [Action<'static>; 2] {
    [
        Action { hotkey: 'T', label: "Toggle isolation", enabled: true },
        Action { hotkey: 'C', label: "Clear counters",   enabled: true },
    ]
}

pub fn handle_key(c: u8) -> AppEvent {
    match c {
        0x90 => {
            let v = viewport_start();
            if v > 0 { set_viewport_start(v.saturating_sub(8)); }
            AppEvent::Repaint
        }
        0x91 => {
            let total = activity::count();
            let v = viewport_start();
            if v + 8 < total { set_viewport_start(v + 8); }
            AppEvent::Repaint
        }
        b't' | b'T' => {
            net::set_isolation(!net::is_isolated());
            AppEvent::Repaint
        }
        b'c' | b'C' => {
            net::clear_counters();
            set_viewport_start(0);
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let action_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    if let Some(key) = action_strip_hit_test(action_rect, mx, my, &actions()) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn format_rate(bps: u32, buf: &mut [u8], n: &mut usize) {
    if bps >= 1024 {
        let kbps = bps / 1024;
        write_dec(buf, n, kbps);
        push_bytes(buf, n, b".");
        write_dec(buf, n, ((bps % 1024) * 10) / 1024);
        push_bytes(buf, n, b" KB");
    } else {
        write_dec(buf, n, bps);
        push_bytes(buf, n, b" B");
    }
}

fn format_bytes(bytes: u64, buf: &mut [u8], n: &mut usize) {
    if bytes >= 1024 * 1024 {
        write_dec(buf, n, (bytes / (1024 * 1024)) as u32);
        push_bytes(buf, n, b".");
        write_dec(buf, n, (((bytes % (1024 * 1024)) * 10) / (1024 * 1024)) as u32);
        push_bytes(buf, n, b" MB");
    } else if bytes >= 1024 {
        write_dec(buf, n, (bytes / 1024) as u32);
        push_bytes(buf, n, b" KB");
    } else {
        write_dec(buf, n, bytes as u32);
        push_bytes(buf, n, b" B");
    }
}

fn format_hms(secs: u64, buf: &mut [u8], n: &mut usize) {
    let h = secs / 3600;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    write_pad2(buf, n, h as u32);
    push_bytes(buf, n, b":");
    write_pad2(buf, n, m as u32);
    push_bytes(buf, n, b":");
    write_pad2(buf, n, s as u32);
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return; }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
}

fn write_pad2(buf: &mut [u8], n: &mut usize, v: u32) {
    if v < 10 { push_bytes(buf, n, b"0"); }
    write_dec(buf, n, v);
}
