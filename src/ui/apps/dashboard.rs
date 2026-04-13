// Bat_OS — System Dashboard App
// Real-time system overview: CPU, memory, security, uptime.

use crate::drivers::virtio::gpu;
use crate::ui::font;
use crate::ui::wm;

const BG: u32 = 0xFF000000;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const DIM: u32 = 0xFF5A5A5A;
const PANEL_BG: u32 = 0xFF0A0A0A;
const BORDER: u32 = 0xFF1E1E1E;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();

    // Clear content area
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 4;
    let ymax = r.y + r.h;
    let xmax = r.x + r.w;
    let ln = 18u32;

    // Use single-column if pane is narrow (< 400px), two columns if wide
    let two_col = r.w >= 400;
    let panel_w = if two_col { r.w / 2 - 12 } else { r.w - 8 };
    let col2 = if two_col { r.x + r.w / 2 + 4 } else { x };

    // Helper: max chars that fit in panel width
    let max_chars = (panel_w / 8) as usize;

    let mut y = r.y + 4;

    // ─── Security Status ───
    if y + 20 < ymax { draw_panel(x, y, panel_w, (ymax - y).min(200), "SECURITY"); }
    y += 22;
    if y+ln < ymax { draw_kv(fb, w, x+4, y, "Encrypt", "AES-CTR", GREEN); y+=ln; }
    if y+ln < ymax { draw_kv(fb, w, x+4, y, "Hash", "SHA-256", GREEN); y+=ln; }
    if y+ln < ymax { draw_kv(fb, w, x+4, y, "Firewall", "DENY ALL", GREEN); y+=ln; }
    if y+ln < ymax { draw_kv(fb, w, x+4, y, "Auth", "pass+Yubi", FG); y+=ln; }
    if y+ln < ymax { draw_kv(fb, w, x+4, y, "Caps", "ENFORCED", GREEN); y+=ln; }

    // ─── System Info (right col or below) ───
    let mut y2 = if two_col { r.y + 4 } else { y + 4 };
    let cx = if two_col { col2 } else { x };
    let pw2 = if two_col { panel_w } else { r.w - 8 };
    if y2 + 20 < ymax { draw_panel(cx, y2, pw2, (ymax - y2).min(180), "SYSTEM"); }
    y2 += 22;

    let (mins, _) = get_uptime();
    if y2+ln < ymax { draw_kv_num(fb, w, cx+4, y2, "Up", mins as usize, "m"); y2+=ln; }
    let (used, total) = crate::kernel::mm::frame::stats();
    if y2+ln < ymax { draw_kv_num(fb, w, cx+4, y2, "Free", (total-used)*4, "KB"); y2+=ln; }
    if y2+ln < ymax { draw_kv_num(fb, w, cx+4, y2, "Frames", total, ""); y2+=ln; }
    let net_ok = crate::drivers::virtio::net::is_ready();
    if y2+ln < ymax { draw_kv(fb, w, cx+4, y2, "Net", if net_ok {"ON"} else {"OFF"},
            if net_ok { GREEN } else { RED }); y2+=ln; }
    let (allowed, blocked) = crate::net::firewall::stats();
    if y2+ln < ymax { draw_kv_num(fb, w, cx+4, y2, "FW OK", allowed as usize, ""); y2+=ln; }
    if y2+ln < ymax { draw_kv_num(fb, w, cx+4, y2, "FW Blk", blocked as usize, ""); }

    // ─── Architecture (only if space) ───
    let arch_y = y.max(y2) + 4;
    if arch_y + 40 < ymax {
        let arch_w = if two_col { r.w - 8 } else { r.w - 8 };
        draw_panel(x, arch_y, arch_w, (ymax - arch_y).min(140), "ARCH");
        let mut ay = arch_y + 22;
        if ay+ln < ymax { font::draw_str(fb, w, x+4, ay, "Bat_OS v0.3.0 Microkernel", FG_HI, BG); ay+=ln; }
        if ay+ln < ymax { font::draw_str(fb, w, x+4, ay, "ARM64 Rust + ASM", FG, BG); ay+=ln; }
        if ay+ln < ymax { font::draw_str(fb, w, x+4, ay, "seL4 caps, AES+SHA", FG, BG); ay+=ln; }
        if ay+ln < ymax { font::draw_str(fb, w, x+4, ay, "Zero deps. Zero trust.", FG_HI, BG); }
    }
}

fn draw_panel(x: u32, y: u32, w: u32, h: u32, title: &str) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();

    // Panel border
    gpu::fill_rect(x, y, w, 1, BORDER);
    gpu::fill_rect(x, y + h, w, 1, BORDER);
    gpu::fill_rect(x, y, 1, h, BORDER);
    gpu::fill_rect(x + w, y, 1, h, BORDER);

    // Title background
    gpu::fill_rect(x + 1, y + 1, w - 1, 20, PANEL_BG);
    font::draw_str(fb, sw, x + 8, y + 3, title, FG_HI, PANEL_BG);

    // Separator under title
    gpu::fill_rect(x + 1, y + 21, w - 1, 1, BORDER);
}

fn draw_kv(fb: *mut u32, w: u32, x: u32, y: u32, key: &str, val: &str, val_color: u32) {
    font::draw_str(fb, w, x, y, key, DIM, BG);
    font::draw_str(fb, w, x + 160, y, val, val_color, BG);
}

fn draw_kv_num(fb: *mut u32, w: u32, x: u32, y: u32, key: &str, val: usize, suffix: &str) {
    font::draw_str(fb, w, x, y, key, DIM, BG);

    // Number to string
    let mut buf = [b' '; 12];
    let mut n = val;
    let mut i = 11;
    if n == 0 { buf[11] = b'0'; }
    else {
        while n > 0 && i > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i -= 1; }
    }
    let s = unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) };
    font::draw_str(fb, w, x + 160, y, s, FG_HI, BG);

    if !suffix.is_empty() {
        let sx = x + 160 + (s.len() as u32 + 1) * 8;
        font::draw_str(fb, w, sx, y, suffix, DIM, BG);
    }
}

fn get_uptime() -> (u64, u64) {
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let secs = count / freq;
    (secs / 60, secs % 60)
}
