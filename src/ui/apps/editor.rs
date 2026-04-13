// Bat_OS — Code Editor App
// Minimal text editor with syntax-aware display.
// Phase 6 scaffold — full editing comes later.

use crate::drivers::virtio::gpu;
use crate::ui::font;
use crate::ui::wm;

const BG: u32 = 0xFF000000;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const LINE_NUM: u32 = 0xFF3A3A3A;
const BORDER: u32 = 0xFF1E1E1E;
const KEYWORD: u32 = 0xFF6688CC;
const STRING_C: u32 = 0xFF88BB66;
const COMMENT: u32 = 0xFF555555;
const GUTTER: u32 = 0xFF080808;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    // File tab bar
    let x = r.x;
    let mut y = r.y;
    gpu::fill_rect(x, y, r.w, 20, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 8, y + 2, "kernel_main.rs", FG_HI, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 140, y + 2, "x", DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 160, y + 2, "untitled.rs", DIM, 0xFF080808);
    gpu::fill_rect(x, y + 20, r.w, 1, BORDER);
    y += 22;

    // Line gutter width
    let gutter_w: u32 = 48;
    gpu::fill_rect(x, y, gutter_w, r.h - 22, GUTTER);
    gpu::fill_rect(x + gutter_w, y, 1, r.h - 22, BORDER);

    // Sample Rust code display
    let code_x = x + gutter_w + 8;
    let lines: &[(&str, u32)] = &[
        ("#![no_std]", KEYWORD),
        ("#![no_main]", KEYWORD),
        ("", FG),
        ("// Bat_OS — Microkernel Entry Point", COMMENT),
        ("// Zero dependencies. Zero trust.", COMMENT),
        ("", FG),
        ("mod drivers;", KEYWORD),
        ("mod kernel;", KEYWORD),
        ("mod crypto;", KEYWORD),
        ("mod fs;", KEYWORD),
        ("mod net;", KEYWORD),
        ("mod ui;", KEYWORD),
        ("", FG),
        ("use core::panic::PanicInfo;", FG),
        ("", FG),
        ("#[unsafe(no_mangle)]", KEYWORD),
        ("pub extern \"C\" fn kernel_main() -> ! {", FG_HI),
        ("    // Initialize microkernel", COMMENT),
        ("    kernel::mm::init();", FG),
        ("    kernel::process::init();", FG),
        ("    kernel::scheduler::init();", FG),
        ("    kernel::ipc::init();", FG),
        ("    kernel::arch::init_exceptions();", FG),
        ("", FG),
        ("    // Initialize encrypted vault", COMMENT),
        ("    fs::batfs::init(&master_key);", FG),
        ("", FG),
        ("    // Initialize networking", COMMENT),
        ("    drivers::virtio::net::init();", FG),
        ("    net::init();", FG),
        ("", FG),
        ("    // Launch shell", COMMENT),
        ("    ui::shell::run();", FG_HI),
        ("}", FG_HI),
    ];

    for (i, (line, color)) in lines.iter().enumerate() {
        let ly = y + (i as u32) * 16;
        if ly + 16 > r.y + r.h { break; }

        // Line number
        let num = i + 1;
        let mut buf = [b' '; 4];
        let mut n = num;
        let mut j = 3;
        if n == 0 { buf[3] = b'0'; }
        else { while n > 0 && j > 0 { buf[j] = b'0' + (n % 10) as u8; n /= 10; j -= 1; } }
        font::draw_str(fb, w, x + 8, ly, unsafe { core::str::from_utf8_unchecked(&buf) }, LINE_NUM, GUTTER);

        // Code line
        font::draw_str(fb, w, code_x, ly, line, *color, BG);
    }

    // Cursor (blinking block)
    let cursor_y = y + 33 * 16;
    if cursor_y + 16 < r.y + r.h {
        gpu::fill_rect(code_x, cursor_y, 8, 16, FG_HI);
    }

    // Status line
    let sy = r.y + r.h - 20;
    gpu::fill_rect(x, sy, r.w, 20, 0xFF0A0A0A);
    gpu::fill_rect(x, sy, r.w, 1, BORDER);
    font::draw_str(fb, w, x + 8, sy + 2, "Rust", DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 60, sy + 2, "|", BORDER, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 72, sy + 2, "UTF-8", DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 120, sy + 2, "|", BORDER, 0xFF0A0A0A);
    font::draw_str(fb, w, x + 132, sy + 2, "Ln 34, Col 1", DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, x + r.w - 120, sy + 2, "READ ONLY", DIM, 0xFF0A0A0A);
}
