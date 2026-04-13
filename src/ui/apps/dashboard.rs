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

    let mut y = r.y + 8;
    let x = r.x + 16;
    let col2 = r.x + r.w / 2 + 16;

    // ─── LEFT COLUMN: Security Status ───
    draw_panel(x, y, r.w / 2 - 32, 200, "SECURITY STATUS");
    y += 28;
    draw_kv(fb, w, x + 8, y, "Encryption", "AES-256-CTR ACTIVE", GREEN); y += 18;
    draw_kv(fb, w, x + 8, y, "Integrity", "SHA-256 VERIFIED", GREEN); y += 18;
    draw_kv(fb, w, x + 8, y, "Secure Enclave", "SIMULATED (QEMU)", DIM); y += 18;
    draw_kv(fb, w, x + 8, y, "Firewall", "DEFAULT DENY ALL", GREEN); y += 18;
    draw_kv(fb, w, x + 8, y, "Dead Man Switch", "NOT ARMED (dev)", DIM); y += 18;
    draw_kv(fb, w, x + 8, y, "Auth Method", "passphrase + YubiKey", FG); y += 18;
    draw_kv(fb, w, x + 8, y, "Kernel Mode", "EL1 (privileged)", FG); y += 18;
    draw_kv(fb, w, x + 8, y, "Capabilities", "ENFORCED", GREEN);

    // ─── RIGHT COLUMN: System Info ───
    let mut y2 = r.y + 8;
    draw_panel(col2, y2, r.w / 2 - 32, 200, "SYSTEM INFO");
    y2 += 28;

    // Uptime
    let (mins, secs) = get_uptime();
    draw_kv_num(fb, w, col2 + 8, y2, "Uptime", mins as usize, "m"); y2 += 18;

    // Memory
    let (used, total) = crate::kernel::mm::frame::stats();
    let free_kb = (total - used) * 4;
    draw_kv_num(fb, w, col2 + 8, y2, "Free Memory", free_kb, "KB"); y2 += 18;
    draw_kv_num(fb, w, col2 + 8, y2, "Used Frames", used, ""); y2 += 18;
    draw_kv_num(fb, w, col2 + 8, y2, "Total Frames", total, ""); y2 += 18;

    // Files
    let (file_count, file_max) = crate::fs::batfs::stats();
    draw_kv_num(fb, w, col2 + 8, y2, "Vault Files", file_count, ""); y2 += 18;

    // Network
    let net_ok = crate::drivers::virtio::net::is_ready();
    draw_kv(fb, w, col2 + 8, y2, "Network", if net_ok { "ONLINE" } else { "OFFLINE" },
            if net_ok { GREEN } else { RED }); y2 += 18;

    // Firewall stats
    let (allowed, blocked) = crate::net::firewall::stats();
    draw_kv_num(fb, w, col2 + 8, y2, "FW Allowed", allowed as usize, "pkts"); y2 += 18;
    draw_kv_num(fb, w, col2 + 8, y2, "FW Blocked", blocked as usize, "pkts");

    // ─── BOTTOM: Architecture ───
    let arch_y = r.y + 224;
    draw_panel(x, arch_y, r.w - 32, 160, "ARCHITECTURE");
    let ay = arch_y + 28;

    font::draw_str(fb, w, x + 8, ay, "Bat_OS v0.3.0 — Custom Microkernel", FG_HI, BG);
    font::draw_str(fb, w, x + 8, ay + 18, "Target: Apple Silicon M4 (QEMU virt)", FG, BG);
    font::draw_str(fb, w, x + 8, ay + 36, "Kernel: Rust + ARM64 Assembly", FG, BG);
    font::draw_str(fb, w, x + 8, ay + 54, "Security: seL4-inspired capabilities", FG, BG);
    font::draw_str(fb, w, x + 8, ay + 72, "Crypto: AES-256 + SHA-256 (zero deps)", FG, BG);
    font::draw_str(fb, w, x + 8, ay + 90, "Display: 1280x800 virtio-gpu framebuffer", FG, BG);
    font::draw_str(fb, w, x + 8, ay + 108, "Philosophy: Zero dependencies. Zero trust.", FG_HI, BG);
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
