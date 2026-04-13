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

    let x = r.x + 4;
    let ymax = r.y + r.h;
    let ln = 18u32;
    let two_col = r.w >= 400;
    let panel_w = if two_col { r.w / 2 - 12 } else { r.w - 8 };
    let col2 = if two_col { r.x + r.w / 2 + 4 } else { x };
    let kv_off = 56u32; // label→value offset (shorter for narrow panes)

    let mut y = r.y + 4;

    // ─── Interface ───
    if y + 20 < ymax { draw_panel(x, y, panel_w, (ymax - y).min(120), "NETWORK"); }
    let mut py = y + 22;

    let mac = crate::drivers::virtio::net::mac();
    let mut mac_str = [0u8; 17];
    for i in 0..6 {
        let hex = b"0123456789abcdef";
        mac_str[i * 3] = hex[(mac[i] >> 4) as usize];
        mac_str[i * 3 + 1] = hex[(mac[i] & 0xf) as usize];
        if i < 5 { mac_str[i * 3 + 2] = b':'; }
    }
    if py+ln < ymax { font::draw_str(fb, w, x+4, py, "MAC", DIM, BG);
        font::draw_str(fb, w, x+kv_off, py, unsafe { core::str::from_utf8_unchecked(&mac_str[..17]) }, FG_HI, BG); py+=ln; }

    let mut ip_buf = [0u8; 16];
    let ip = net::ip::our_ip();
    let ip_len = net::ip::ip_to_str(ip, &mut ip_buf);
    if py+ln < ymax { font::draw_str(fb, w, x+4, py, "IP", DIM, BG);
        font::draw_str(fb, w, x+kv_off, py, unsafe { core::str::from_utf8_unchecked(&ip_buf[..ip_len]) }, FG_HI, BG); py+=ln; }

    let gw = net::ip::gateway();
    let gw_len = net::ip::ip_to_str(gw, &mut ip_buf);
    if py+ln < ymax { font::draw_str(fb, w, x+4, py, "GW", DIM, BG);
        font::draw_str(fb, w, x+kv_off, py, unsafe { core::str::from_utf8_unchecked(&ip_buf[..gw_len]) }, FG_HI, BG); py+=ln; }

    let net_ok = crate::drivers::virtio::net::is_ready();
    if py+ln < ymax { font::draw_str(fb, w, x+4, py, "Net", DIM, BG);
        font::draw_str(fb, w, x+kv_off, py, if net_ok {"ONLINE"} else {"OFF"},
                       if net_ok { GREEN } else { RED }, BG); py+=ln; }

    // ─── Firewall ───
    let mut fy = if two_col { r.y + 4 } else { py + 4 };
    let cx = if two_col { col2 } else { x };
    let pw2 = if two_col { panel_w } else { r.w - 8 };
    if fy + 20 < ymax { draw_panel(cx, fy, pw2, (ymax - fy).min(120), "FIREWALL"); }
    fy += 22;

    if fy+ln < ymax { font::draw_str(fb, w, cx+4, fy, "Policy", DIM, BG);
        font::draw_str(fb, w, cx+kv_off, fy, "DENY ALL", RED, BG); fy+=ln; }
    if fy+ln < ymax { font::draw_str(fb, w, cx+4, fy, "Mode", DIM, BG);
        font::draw_str(fb, w, cx+kv_off, fy, "ALLOWLIST", FG_HI, BG); fy+=ln; }

    let (allowed, blocked) = net::firewall::stats();
    if fy+ln < ymax { font::draw_str(fb, w, cx+4, fy, "OK", DIM, BG);
        draw_num(fb, w, cx+kv_off, fy, allowed as usize, GREEN); fy+=ln; }
    if fy+ln < ymax { font::draw_str(fb, w, cx+4, fy, "Blk", DIM, BG);
        draw_num(fb, w, cx+kv_off, fy, blocked as usize, RED); }

    // ─── Security Stack (only if space) ───
    let stack_y = py.max(fy) + 4;
    if stack_y + 30 < ymax {
        let sw = if two_col { r.w - 8 } else { r.w - 8 };
        draw_panel(x, stack_y, sw, (ymax - stack_y).min(160), "SEC STACK");
        let mut sy = stack_y + 22;
        if sy+ln < ymax { font::draw_str(fb, w, x+4, sy, "TLS 1.3   [plan]", FG, BG); sy+=ln; }
        if sy+ln < ymax { font::draw_str(fb, w, x+4, sy, "VPN       [plan]", FG, BG); sy+=ln; }
        if sy+ln < ymax { font::draw_str(fb, w, x+4, sy, "Tor       [plan]", FG, BG); sy+=ln; }
        if sy+ln < ymax { font::draw_str(fb, w, x+4, sy, "FW        [LIVE]", GREEN, BG); }
    }
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
