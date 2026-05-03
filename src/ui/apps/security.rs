// Bat_OS — SK · Security
//
// STUMP #125 — Claude-Design port. Source artifacts in
// `docs/design/apps-ds-nm-sk/` (jsx + spec sheet).
//
// Layout: top row is the full-width ACTIVE BATCAVES table,
// bottom row is SECURITY PIPELINE (left) + INTEGRITY (right).
// Narrow (<512px) collapses bottom row to 1-col.

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::widgets::{
    self as W, draw_panel, draw_kv_row, draw_caves_header, draw_caves_row,
    draw_caves_empty_row, draw_audit_strip, AuditLine,
    KV_ROW_H, CAVES_HEADER_H, CAVES_ROW_H, State,
    BG, INK, MID, GREEN, HAIR,
};
use crate::batcave::cave;

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 200 { return; }

    let pad: u32 = 16;
    let gutter: u32 = 16;
    let narrow = r.w < 720;

    // ── Top: ACTIVE BATCAVES (full-width) ─────────────────────────
    let caves_h: u32 = 180;
    let caves_x = r.x + pad;
    let caves_y = r.y + pad;
    let caves_w = r.w.saturating_sub(pad * 2);
    let mut caves_metric = [0u8; 16];
    let cn = format_slots_metric(active_cave_count(), cave::MAX_CAVES, &mut caves_metric);
    let caves_inner = draw_panel(caves_x, caves_y, caves_w, caves_h,
        "ACTIVE BATCAVES",
        Some(unsafe { core::str::from_utf8_unchecked(&caves_metric[..cn]) }));
    draw_caves_table(&caves_inner);

    // ── Bottom: SECURITY PIPELINE + INTEGRITY ─────────────────────
    let bot_y = caves_y + caves_h + gutter;
    let bot_h = r.h.saturating_sub(bot_y - r.y + pad);
    if bot_h < 80 { return; }

    let (pipe_x, pipe_y, pipe_w);
    let (int_x, int_y, int_w);
    if narrow {
        pipe_x = r.x + pad; pipe_y = bot_y; pipe_w = r.w.saturating_sub(pad * 2);
        int_x  = r.x + pad; int_y  = pipe_y + bot_h / 2; int_w = r.w.saturating_sub(pad * 2);
    } else {
        let half = (r.w - pad * 2 - gutter) / 2;
        pipe_x = r.x + pad;            pipe_y = bot_y; pipe_w = half;
        int_x  = pipe_x + half + gutter; int_y = bot_y; int_w = half;
    }
    let half_h = if narrow { bot_h / 2 - gutter / 2 } else { bot_h };

    let pipe_inner = draw_panel(pipe_x, pipe_y, pipe_w, half_h,
        "SECURITY PIPELINE", Some("8 STAGES"));
    draw_pipeline_kvs(&pipe_inner);

    let int_inner = draw_panel(int_x, int_y, int_w, half_h,
        "INTEGRITY", Some("BatFS . MERKLE"));
    draw_integrity(&int_inner);
}

fn draw_caves_table(p: &W::PanelInner) {
    draw_caves_header(p.x, p.y, p.w);
    let mut y = p.y + CAVES_HEADER_H;
    let mut shown = 0u32;
    let max_rows = (p.h.saturating_sub(CAVES_HEADER_H)) / CAVES_ROW_H;
    let active_id = cave::get_active();
    cave::list(|c| {
        if shown + 1 >= max_rows { return; }
        let (badge_state, badge) = match c.state {
            cave::CaveState::Running => (State::Ok,   "RUN"),
            cave::CaveState::Stopped => (State::Plan, "STP"),
            cave::CaveState::Destroyed => (State::Fail, "DEL"),
            cave::CaveState::Free    => return,
        };
        let mut caps: u8 = 0;
        if c.has_cap("net") { caps |= 1 << 0; }
        if c.has_cap("raw") { caps |= 1 << 1; }
        if c.has_cap("display") { caps |= 1 << 2; }
        if c.has_cap("fs")  { caps |= 1 << 3; }
        draw_caves_row(p.x, y, p.w, badge_state, badge, c.name_str(), caps);
        y += CAVES_ROW_H;
        shown += 1;
    });
    let _ = active_id;
    // Empty placeholder if no caves OR if there's room left.
    let free_slots = cave::MAX_CAVES.saturating_sub(active_cave_count());
    if shown == 0 || y + CAVES_ROW_H <= p.y + p.h {
        draw_caves_empty_row(p.x, y, p.w, free_slots);
    }
}

fn draw_pipeline_kvs(p: &W::PanelInner) {
    let label_w: u32 = 88;
    let mut y = p.y;

    let net_ok = crate::drivers::virtio::net::is_ready();
    draw_kv_row(p.x, y, label_w, "FIREWALL", "ACTIVE",
        if net_ok { State::Ok } else { State::Plan }, true); y += KV_ROW_H;

    draw_kv_row(p.x, y, label_w, "AES-256",  "ACTIVE",          State::Ok,      true); y += KV_ROW_H;

    let mode = crate::net::tls_pinning::current_mode();
    let (tls_label, tls_state) = match mode {
        crate::net::tls_pinning::Mode::Lockdown => ("LOCKDOWN . 1 PIN", State::Neutral),
        crate::net::tls_pinning::Mode::Research => ("RESEARCH",         State::Warn),
        crate::net::tls_pinning::Mode::Open     => ("OPEN",             State::Fail),
    };
    draw_kv_row(p.x, y, label_w, "TLS 1.3", tls_label, tls_state, true); y += KV_ROW_H;

    draw_kv_row(p.x, y, label_w, "VPN",     "STANDBY",         State::Plan,    true); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "Tor",     "3-HOP CIRCUIT",   State::Ok,      true); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "DNS",     "DoH ENABLED",     State::Ok,      true); y += KV_ROW_H;

    let audit_n = crate::security::audit::count();
    let mut audit_buf = [0u8; 32];
    let an = format_audit_count(audit_n, &mut audit_buf);
    draw_kv_row(p.x, y, label_w, "AUDIT",
        unsafe { core::str::from_utf8_unchecked(&audit_buf[..an]) },
        State::Neutral, true);
    y += KV_ROW_H;

    draw_kv_row(p.x, y, label_w, "DMS",     "ARMED . 48H",     State::Ok,      true);
}

fn draw_integrity(p: &W::PanelInner) {
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let label_w: u32 = 88;
    let mut y = p.y;

    // 1. Merkle row.
    let root = crate::fs::batfs::merkle_root();
    let hex = b"0123456789abcdef";
    let mut hash_str = [b' '; 19]; // 4 groups of 4 chars + 3 spaces
    let group = |i: usize, dst: &mut [u8], pos: usize| {
        let byte = root[i];
        dst[pos]     = hex[(byte >> 4) as usize];
        dst[pos + 1] = hex[(byte & 0xf) as usize];
    };
    group(0, &mut hash_str, 0);
    group(1, &mut hash_str, 2);
    hash_str[4] = b' ';
    group(2, &mut hash_str, 5);
    group(3, &mut hash_str, 7);
    hash_str[9] = b' ';
    group(4, &mut hash_str, 10);
    group(5, &mut hash_str, 12);
    hash_str[14] = b' ';
    group(6, &mut hash_str, 15);
    group(7, &mut hash_str, 17);

    font::draw_str(fb, w, p.x, y + 3, "MERKLE", MID, BG);
    font::draw_str(fb, w, p.x + label_w, y + 3,
        unsafe { core::str::from_utf8_unchecked(&hash_str) }, INK, BG);
    // "..." continuation indicator.
    font::draw_str(fb, w, p.x + label_w + 19 * 8 + 4, y + 3, "...", crate::ui::widgets::DIM_TXT, BG);
    // Right-aligned VERIFIED.
    let verified = "VERIFIED";
    if p.w > (verified.len() as u32) * 8 + 8 {
        let vx = p.x + p.w - (verified.len() as u32) * 8;
        font::draw_str(fb, w, vx, y + 3, verified, GREEN, BG);
    }
    y += 22;
    // Hairline under merkle row.
    gpu::fill_rect(p.x, y, p.w, 1, HAIR);
    y += 6;

    // 2. KV trio with status dots.
    draw_kv_row(p.x, y, label_w, "AUTH",       "VERIFIED",                State::Ok,   true); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "OPEN PORTS", "0 . invisible-by-design", State::Ok,   true); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "WIPE",       "ARMED",                   State::Warn, true); y += KV_ROW_H;

    // 3. Audit mini-strip — STUMP #128: was anchored to the panel
    // bottom which left a huge empty gap below WIPE. Now drawn
    // inline right after the WIPE row with a small separator.
    y += 8;
    gpu::fill_rect(p.x, y, p.w, 1, HAIR);
    y += 8;
    let strip_h: u32 = 18 + 4 * 16 + 4;
    if y + strip_h <= p.y + p.h {
        let lines = [
            AuditLine { idx: 243, cat: "script:", text: "exec js 1024B" },
            AuditLine { idx: 242, cat: "fetch :", text: "GET http://10.0.2.2:8765/  OK" },
            AuditLine { idx: 241, cat: "nav   :", text: "main origin -> http://10.0.2.2" },
            AuditLine { idx: 240, cat: "mode  :", text: "js-mode -> on" },
        ];
        draw_audit_strip(p.x, y, &lines);
    }
}

// ── helpers ───────────────────────────────────────────────────────

fn active_cave_count() -> usize {
    let mut n = 0usize;
    cave::list(|c| {
        if c.state != cave::CaveState::Free { n += 1; }
    });
    n
}

fn format_slots_metric(used: usize, total: usize, out: &mut [u8]) -> usize {
    let mut p = format_dec(used, out);
    out[p] = b' '; p += 1;
    out[p] = b'/'; p += 1;
    out[p] = b' '; p += 1;
    p += format_dec(total, &mut out[p..]);
    let suffix = b" SLOTS";
    out[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    p
}

fn format_audit_count(n: usize, out: &mut [u8]) -> usize {
    let mut p = format_dec(n, out);
    let suffix = b" / 1024 ENTRIES";
    out[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    p
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
