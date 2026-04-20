// Bat_OS — File Manager App
// Browse the encrypted vault. View file listing with metadata.

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::wm;
use crate::fs::batfs;

const BG: u32 = 0xFF000000;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const BORDER: u32 = 0xFF1E1E1E;
const PANEL_BG: u32 = 0xFF0A0A0A;
const ROW_ALT: u32 = 0xFF080808;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 16;
    let mut y = r.y + 8;

    // Header
    font::draw_str(fb, w, x, y, "ENCRYPTED VAULT", FG_HI, BG);
    font::draw_str(fb, w, x + 160, y, "// AES-256-CTR + SHA-256 integrity", DIM, BG);
    y += 24;

    // Column headers
    gpu::fill_rect(x, y, r.w - 32, 18, PANEL_BG);
    font::draw_str(fb, w, x + 8, y + 1, "STATUS", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 80, y + 1, "FILENAME", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 400, y + 1, "SIZE", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 500, y + 1, "ENCRYPTION", DIM, PANEL_BG);
    y += 20;

    gpu::fill_rect(x, y, r.w - 32, 1, BORDER);
    y += 4;

    // File listing
    let mut row = 0u32;
    let (count, _max) = batfs::stats();

    if count == 0 {
        font::draw_str(fb, w, x + 8, y + 20, "(vault is empty — use 'write' in terminal)", DIM, BG);
    } else {
        batfs::list(|name, size, encrypted| {
            let row_bg = if row % 2 == 0 { BG } else { ROW_ALT };
            let ry = y + row * 20;
            gpu::fill_rect(x, ry, r.w - 32, 20, row_bg);

            // Status icon
            let status_color = if encrypted { GREEN } else { FG };
            gpu::fill_rect(x + 12, ry + 6, 8, 8, status_color);

            // Lock icon text
            font::draw_str(fb, w, x + 30, ry + 2, if encrypted { "[ENC]" } else { "[RAW]" }, status_color, row_bg);

            // Filename
            font::draw_str(fb, w, x + 80, ry + 2, name, FG_HI, row_bg);

            // Size
            let mut buf = [b' '; 12];
            let mut n = size;
            let mut i = 11;
            if n == 0 { buf[11] = b'0'; }
            else { while n > 0 && i > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i -= 1; } }
            let s = unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) };
            font::draw_str(fb, w, x + 400, ry + 2, s, FG, row_bg);
            font::draw_str(fb, w, x + 400 + (s.len() as u32 + 1) * 8, ry + 2, "B", DIM, row_bg);

            // Encryption
            font::draw_str(fb, w, x + 500, ry + 2, "AES-256-CTR", GREEN, row_bg);

            row += 1;
        });
    }

    // Footer
    let fy = r.y + r.h - 40;
    gpu::fill_rect(x, fy, r.w - 32, 1, BORDER);
    font::draw_str(fb, w, x + 8, fy + 8, "Files:", DIM, BG);
    let mut buf = [b' '; 4];
    let mut n = count;
    let mut i = 3;
    if n == 0 { buf[3] = b'0'; } else { while n > 0 && i > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i -= 1; } }
    font::draw_str(fb, w, x + 60, fy + 8, unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) }, FG_HI, BG);

    font::draw_str(fb, w, x + 100, fy + 8, "|  Ctrl+1: Terminal to manage files", DIM, BG);
}
