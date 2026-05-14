// Sphragis — NM · Network Monitor
// XXX Wave-2-temp: 1 old-WM call site commented out, restored in Task 7.
//
// Claude-Design port. Source artifacts in
// `docs/design/apps-ds-nm-sk/` (jsx + spec sheet).
//
// Layout: 2-column grid for INTERFACE + FIREWALL (top, 280px tall),
// full-width SECURITY STACK flow diagram below. Narrow (<512px)
// collapses 2-col grids to 1-col and turns the flow strip into a
// 3×2 grid of FlowBox cells (no arrows).

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::widgets::{
    self as W, draw_panel, draw_kv_row, draw_flow_box, draw_flow_arrow,
    KV_ROW_H, State, FLOW_BOX_W, FLOW_BOX_H, FLOW_ARROW_W,
    BG,
};
use crate::net;

pub fn render() {
    // XXX Wave-2-temp: let r = wm::content_rect();
    let r = wm::WindowRect { x: 0, y: 0, w: gpu::width(), h: gpu::height() };
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 200 { return; }

    let pad: u32 = 16;
    let gutter: u32 = 16;
    let narrow = r.w < 720;
    let top_h: u32 = 280;
    let flow_h: u32 = 200;

    // ── Top row: INTERFACE (left) + FIREWALL (right) ──────────────
    let (if_x, if_y, if_w);
    let (fw_x, fw_y, fw_w);
    if narrow {
        if_x = r.x + pad; if_y = r.y + pad; if_w = r.w.saturating_sub(pad * 2);
        fw_x = r.x + pad; fw_y = if_y + top_h + gutter; fw_w = r.w.saturating_sub(pad * 2);
    } else {
        let half = (r.w - pad * 2 - gutter) / 2;
        if_x = r.x + pad;             if_y = r.y + pad; if_w = half;
        fw_x = if_x + half + gutter;  fw_y = r.y + pad; fw_w = half;
    }

    let if_inner = draw_panel(if_x, if_y, if_w, top_h, "INTERFACE", Some("LINK"));
    draw_interface_kvs(&if_inner);

    let fw_inner = draw_panel(fw_x, fw_y, fw_w, top_h, "FIREWALL", Some("DENY ALL"));
    draw_firewall_kvs(&fw_inner);

    // ── Bottom: SECURITY STACK flow diagram ───────────────────────
    let stack_y = if narrow {
        fw_y + top_h + gutter
    } else {
        r.y + pad + top_h + gutter
    };
    let stack_x = r.x + pad;
    let stack_w = r.w.saturating_sub(pad * 2);
    let stack_h = r.h.saturating_sub(stack_y - r.y + pad).max(flow_h);
    let stack_inner = draw_panel(stack_x, stack_y, stack_w, stack_h,
        "SECURITY STACK", Some("REQUEST FLOW . LIVE"));
    draw_flow_strip(&stack_inner, narrow);
}

fn draw_interface_kvs(p: &W::PanelInner) {
    let label_w: u32 = 56;
    let mut y = p.y;

    let net_ok = crate::drivers::virtio::net::is_ready();
    draw_kv_row(p.x, y, label_w, "LINK",
        if net_ok { "UP . 1 Gbps full-duplex" } else { "DOWN" },
        if net_ok { State::Ok } else { State::Fail }, true);
    y += KV_ROW_H;

    // MAC.
    let mac = crate::drivers::virtio::net::mac();
    let mut mac_buf = [0u8; 18];
    write_mac(&mac, &mut mac_buf);
    draw_kv_row(p.x, y, label_w, "MAC",
        unsafe { core::str::from_utf8_unchecked(&mac_buf[..17]) },
        State::Neutral, false);
    y += KV_ROW_H;

    // IPv4 + " / 24" suffix (QEMU SLIRP default subnet).
    let ip = net::ip::our_ip();
    let mut ip_arr = [0u8; 16];
    let ip_n = net::ip::ip_to_str(ip, &mut ip_arr);
    let mut ip_buf = [0u8; 32];
    ip_buf[..ip_n].copy_from_slice(&ip_arr[..ip_n]);
    let suffix = b" / 24";
    ip_buf[ip_n..ip_n + suffix.len()].copy_from_slice(suffix);
    let ip_with_mask = unsafe {
        core::str::from_utf8_unchecked(&ip_buf[..ip_n + suffix.len()])
    };
    draw_kv_row(p.x, y, label_w, "IPv4", ip_with_mask, State::Neutral, false);
    y += KV_ROW_H;

    // GW.
    let gw = net::ip::gateway();
    let mut gw_buf = [0u8; 16];
    let gw_n = net::ip::ip_to_str(gw, &mut gw_buf);
    draw_kv_row(p.x, y, label_w, "GW",
        unsafe { core::str::from_utf8_unchecked(&gw_buf[..gw_n]) },
        State::Neutral, false);
    y += KV_ROW_H;

    draw_kv_row(p.x, y, label_w, "DNS", "10.0.2.3 (DoH)", State::Neutral, false);
    y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "MTU", "1500", State::Plan, false);
}

fn draw_firewall_kvs(p: &W::PanelInner) {
    let label_w: u32 = 88;
    let mut y = p.y;
    draw_kv_row(p.x, y, label_w, "POLICY",   "DENY ALL . default-drop",       State::Fail,    false); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "MODE",     "ALLOWLIST",                     State::Neutral, false); y += KV_ROW_H;

    let (allowed, blocked) = net::firewall::stats();
    let mut a_buf = [0u8; 16];
    let an = format_dec(allowed as usize, &mut a_buf);
    draw_kv_row(p.x, y, label_w, "ALLOWED",
        unsafe { core::str::from_utf8_unchecked(&a_buf[..an]) },
        State::Ok, false);
    y += KV_ROW_H;

    let mut b_buf = [0u8; 16];
    let bn = format_dec(blocked as usize, &mut b_buf);
    draw_kv_row(p.x, y, label_w, "BLOCKED",
        unsafe { core::str::from_utf8_unchecked(&b_buf[..bn]) },
        State::Fail, false);
    y += KV_ROW_H;

    draw_kv_row(p.x, y, label_w, "LAST EVT",  "OUT 443 -> cdn.example.com", State::Neutral, false); y += KV_ROW_H;
    draw_kv_row(p.x, y, label_w, "LAST DROP", "IN 22 <- 10.0.2.99 (no rule)", State::Fail, false);
}

fn draw_flow_strip(p: &W::PanelInner, narrow: bool) {
    // 6 boxes: APP -> TLS 1.3 -> PIN VRFY -> SOP -> FIREWALL -> WIRE
    // In narrow mode: 3x2 grid, no arrows.
    let boxes: [(&str, &str, State); 6] = [
        ("APP",      "sphragis shell",        State::Ok),
        ("TLS 1.3",  "LOCKDOWN",            State::Ok),
        ("PIN VRFY", "3 PINS . 0 MISMATCH", State::Ok),
        ("SOP",      "origin allowlist",    State::Ok),
        ("FIREWALL", "DENY ALL",            State::Ok),
        ("WIRE",     "virtio-net",          State::Ok),
    ];
    if narrow {
        // 3 columns × 2 rows.
        let cell_w = (p.w - 32) / 3;
        let cell_h = FLOW_BOX_H + 16;
        for (i, (label, sub, state)) in boxes.iter().enumerate() {
            let row = (i / 3) as u32;
            let col = (i % 3) as u32;
            let bx = p.x + col * cell_w + (cell_w - FLOW_BOX_W) / 2;
            let by = p.y + row * cell_h + 8;
            draw_flow_box(bx, by, label, sub, *state);
        }
    } else {
        // Single horizontal row centered vertically.
        let total_w = 6 * FLOW_BOX_W + 5 * FLOW_ARROW_W;
        let start_x = p.x + p.w.saturating_sub(total_w) / 2;
        let by = p.y + (p.h.saturating_sub(FLOW_BOX_H)) / 2;
        let mut bx = start_x;
        for (i, (label, sub, state)) in boxes.iter().enumerate() {
            draw_flow_box(bx, by, label, sub, *state);
            bx += FLOW_BOX_W;
            if i < boxes.len() - 1 {
                draw_flow_arrow(bx, by, FLOW_ARROW_W);
                bx += FLOW_ARROW_W;
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────

fn write_mac(mac: &[u8; 6], out: &mut [u8; 18]) {
    let hex = b"0123456789abcdef";
    for i in 0..6 {
        out[i * 3]     = hex[(mac[i] >> 4) as usize];
        out[i * 3 + 1] = hex[(mac[i] & 0xf) as usize];
        if i < 5 { out[i * 3 + 2] = b':'; }
    }
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}

// Wave 2 shim — refresh in Wave 3+
/// Adapts the existing render path to the WM's `fn(WindowRect)` contract.
pub fn paint(rect: crate::ui::wm::WindowRect) {
    let _ = rect;
    render();
}
