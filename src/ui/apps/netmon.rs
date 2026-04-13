// Bat_OS — Network Monitor App
// Live network status, firewall activity, connection info.

use crate::drivers::virtio::gpu;
use crate::ui::font;
use crate::ui::wm;
use crate::net;

const BG: u32 = 0xFF000000;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const BORDER: u32 = 0xFF1E1E1E;
const PANEL_BG: u32 = 0xFF0A0A0A;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 16;
    let col2 = r.x + r.w / 2 + 16;
    let mut y = r.y + 8;

    // ─── Interface Panel ───
    draw_panel(x, y, r.w / 2 - 32, 140, "NETWORK INTERFACE");
    let py = y + 28;

    let mac = crate::drivers::virtio::net::mac();
    font::draw_str(fb, w, x + 8, py, "MAC:", DIM, BG);
    let mut mac_str = [0u8; 17];
    for i in 0..6 {
        let hex = b"0123456789abcdef";
        mac_str[i * 3] = hex[(mac[i] >> 4) as usize];
        mac_str[i * 3 + 1] = hex[(mac[i] & 0xf) as usize];
        if i < 5 { mac_str[i * 3 + 2] = b':'; }
    }
    font::draw_str(fb, w, x + 80, py, unsafe { core::str::from_utf8_unchecked(&mac_str[..17]) }, FG_HI, BG);

    let mut ip_buf = [0u8; 16];
    let ip = net::ip::our_ip();
    let ip_len = net::ip::ip_to_str(ip, &mut ip_buf);
    font::draw_str(fb, w, x + 8, py + 18, "IPv4:", DIM, BG);
    font::draw_str(fb, w, x + 80, py + 18, unsafe { core::str::from_utf8_unchecked(&ip_buf[..ip_len]) }, FG_HI, BG);

    let gw = net::ip::gateway();
    let gw_len = net::ip::ip_to_str(gw, &mut ip_buf);
    font::draw_str(fb, w, x + 8, py + 36, "Gateway:", DIM, BG);
    font::draw_str(fb, w, x + 80, py + 36, unsafe { core::str::from_utf8_unchecked(&ip_buf[..gw_len]) }, FG_HI, BG);

    let net_ok = crate::drivers::virtio::net::is_ready();
    font::draw_str(fb, w, x + 8, py + 54, "Status:", DIM, BG);
    font::draw_str(fb, w, x + 80, py + 54, if net_ok { "ONLINE" } else { "OFFLINE" },
                   if net_ok { GREEN } else { RED }, BG);

    font::draw_str(fb, w, x + 8, py + 72, "Driver:", DIM, BG);
    font::draw_str(fb, w, x + 80, py + 72, "virtio-net (MMIO v1)", FG, BG);

    // ─── Firewall Panel ───
    draw_panel(col2, y, r.w / 2 - 32, 140, "FIREWALL");
    let fy = y + 28;

    font::draw_str(fb, w, col2 + 8, fy, "Policy:", DIM, BG);
    font::draw_str(fb, w, col2 + 120, fy, "DEFAULT DENY ALL", RED, BG);

    font::draw_str(fb, w, col2 + 8, fy + 18, "Mode:", DIM, BG);
    font::draw_str(fb, w, col2 + 120, fy + 18, "ALLOWLIST", FG_HI, BG);

    let (allowed, blocked) = net::firewall::stats();
    font::draw_str(fb, w, col2 + 8, fy + 36, "Allowed:", DIM, BG);
    draw_num(fb, w, col2 + 120, fy + 36, allowed as usize, GREEN);
    font::draw_str(fb, w, col2 + 200, fy + 36, "packets", DIM, BG);

    font::draw_str(fb, w, col2 + 8, fy + 54, "Blocked:", DIM, BG);
    draw_num(fb, w, col2 + 120, fy + 54, blocked as usize, RED);
    font::draw_str(fb, w, col2 + 200, fy + 54, "packets", DIM, BG);

    font::draw_str(fb, w, col2 + 8, fy + 72, "Rules:", DIM, BG);
    font::draw_str(fb, w, col2 + 120, fy + 72, "4 active", FG, BG);

    // ─── Security Stack ───
    y += 160;
    draw_panel(x, y, r.w - 32, 180, "NETWORK SECURITY STACK (design)");
    let sy = y + 28;

    font::draw_str(fb, w, x + 8, sy,      "Layer 1: TLS 1.3        [planned]  All connections encrypted", FG, BG);
    font::draw_str(fb, w, x + 8, sy + 20,  "Layer 2: VPN (WireGuard) [planned]  Hides Tor usage from ISP", FG, BG);
    font::draw_str(fb, w, x + 8, sy + 40,  "Layer 3: Tor Circuit     [planned]  3-hop onion routing", FG, BG);
    font::draw_str(fb, w, x + 8, sy + 60,  "Layer 4: DNS-over-HTTPS  [planned]  No plaintext DNS", FG, BG);
    font::draw_str(fb, w, x + 8, sy + 80,  "Layer 5: MAC Randomize   [planned]  Per-session random MAC", FG, BG);
    font::draw_str(fb, w, x + 8, sy + 100, "Layer 6: Allowlist FW    [ACTIVE]   Default deny all inbound", GREEN, BG);
    font::draw_str(fb, w, x + 8, sy + 120, "Stack: Nobody sees the full picture.", FG_HI, BG);
}

fn draw_panel(x: u32, y: u32, w: u32, h: u32, title: &str) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(x, y, w, 1, BORDER);
    gpu::fill_rect(x, y + h, w, 1, BORDER);
    gpu::fill_rect(x, y, 1, h, BORDER);
    gpu::fill_rect(x + w, y, 1, h, BORDER);
    gpu::fill_rect(x + 1, y + 1, w - 1, 20, PANEL_BG);
    font::draw_str(fb, sw, x + 8, y + 3, title, FG_HI, PANEL_BG);
    gpu::fill_rect(x + 1, y + 21, w - 1, 1, BORDER);
}

fn draw_num(fb: *mut u32, w: u32, x: u32, y: u32, n: usize, color: u32) {
    let mut buf = [b' '; 12];
    let mut val = n;
    let mut i = 11;
    if val == 0 { buf[11] = b'0'; }
    else { while val > 0 && i > 0 { buf[i] = b'0' + (val % 10) as u8; val /= 10; i -= 1; } }
    font::draw_str(fb, w, x, y, unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) }, color, BG);
}
