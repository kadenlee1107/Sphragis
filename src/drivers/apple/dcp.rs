#![allow(dead_code)]
// Sphragis — Apple Display Controller (DCP) Driver
// On Apple Silicon, the display is managed by a coprocessor (DCP).
// Communication happens via mailbox messages.
//
// For initial bring-up, we use the simple framebuffer that m1n1
// sets up for us — this avoids needing the full DCP mailbox protocol
// which is extremely complex (Asahi's DCP driver is ~10K lines).
//
// Phase 7 strategy:
// 1. Use m1n1's pre-configured framebuffer (THIS FILE)
// 2. Later: implement full DCP mailbox protocol for mode switching
//
// Reference: Asahi Linux drivers/gpu/drm/apple/

use super::soc;
use core::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the display using m1n1's pre-configured framebuffer.
/// m1n1 already set up the display pipeline — we just write pixels
/// to the framebuffer address it gave us.
pub fn init_simple_fb() -> bool {
    let fb = soc::fb_base();
    let width = soc::fb_width();
    let height = soc::fb_height();

    if fb == 0 || width == 0 || height == 0 {
        return false;
    }

    INITIALIZED.store(true, Ordering::Relaxed);
    true
}

/// Get the framebuffer pointer.
pub fn framebuffer() -> *mut u32 {
    soc::fb_base() as *mut u32
}

pub fn width() -> u32 { soc::fb_width() }
pub fn height() -> u32 { soc::fb_height() }
pub fn stride() -> u32 { soc::fb_stride() }
pub fn is_ready() -> bool { INITIALIZED.load(Ordering::Relaxed) }

/// Set a pixel (accounting for stride which may differ from width*4).
pub fn set_pixel(x: u32, y: u32, color: u32) {
    if !is_ready() || x >= width() || y >= height() { return; }
    let stride_pixels = stride() / 4;
    let offset = (y * stride_pixels + x) as usize;
    unsafe {
        core::ptr::write_volatile(framebuffer().add(offset), color);
    }
}

/// Fill a rectangle.
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    if !is_ready() { return; }
    let stride_pixels = stride() / 4;
    let fb = framebuffer();
    for row in y..(y + h).min(height()) {
        for col in x..(x + w).min(width()) {
            unsafe {
                let offset = (row * stride_pixels + col) as usize;
                core::ptr::write_volatile(fb.add(offset), color);
            }
        }
    }
}

/// Fill the entire screen.
pub fn fill_screen(color: u32) {
    if !is_ready() { return; }
    let stride_pixels = stride() / 4;
    let fb = framebuffer();
    for y in 0..height() {
        for x in 0..width() {
            unsafe {
                let offset = (y * stride_pixels + x) as usize;
                core::ptr::write_volatile(fb.add(offset), color);
            }
        }
    }
    // M4 note: without this barrier, subsequent draw_str calls can
    // interleave with the tail of the fill — observed as m1n1
    // boot-log text bleeding through the boot_splash background.
    // `dsb sy` drains the write queue so every pixel of the wipe
    // lands before the first character of the title is written.
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
}

/// Flush — on the simple framebuffer, writes are immediately visible.
/// No explicit flush needed (unlike virtio-gpu).
pub fn flush(_x: u32, _y: u32, _w: u32, _h: u32) {
    // m1n1 simple framebuffer is direct-mapped.
    // Cache maintenance might be needed on real hardware:
    unsafe {
        // Data Synchronization Barrier — ensure all writes are visible
        core::arch::asm!("dsb sy");
    }
}

// V-ASAHI-2.1: boot splash. Renders a centered "SPHRAGIS / Apple Silicon"
// banner so the very first thing the operator sees on the M4's display
// is unambiguous proof that our kernel — not macOS, not Asahi, not the
// m1n1 splash — owns the screen. Uses ONLY the framebuffer m1n1 set up
// for us (no DCP mailbox traffic, no DART), so this works the moment
// we have a valid fb_base + width + height.

/// Re-encode an ARGB8888 literal into the M4 framebuffer's native
/// ARGB2101010 layout (see docs/M4_GROUND_TRUTH.md §3.1b). Writing an
/// ARGB8888 value directly to the FB produces the wrong color: e.g.
/// `0xFF00_0000` ("opaque black" in 8888) decodes on M4 as A=3,
/// R=0x3F0, G=0, B=0 — bright red. Keep color literals authored in
/// ARGB8888 for readability and run them through this at const-eval
/// time.
pub const fn argb8888_to_m4(argb8888: u32) -> u32 {
    let a8 = (argb8888 >> 24) & 0xFF;
    let r8 = (argb8888 >> 16) & 0xFF;
    let g8 = (argb8888 >>  8) & 0xFF;
    let b8 =  argb8888        & 0xFF;
    // 8→10 bit expansion: replicate the top 2 bits into the low 2
    // so saturated channels stay saturated (0xFF → 0x3FF, not 0x3FC).
    let a2  = a8 >> 6;
    let r10 = (r8 << 2) | (r8 >> 6);
    let g10 = (g8 << 2) | (g8 >> 6);
    let b10 = (b8 << 2) | (b8 >> 6);
    (a2 << 30) | (r10 << 20) | (g10 << 10) | b10
}

/// Draw `s` at pixel (x, y) using the kernel's 8x16 bitmap font.
/// `fg` / `bg` are passed through to the framebuffer as-is — callers
/// are responsible for encoding in whatever pixel format the FB
/// expects (ARGB2101010 on M4; see `argb8888_to_m4`).
pub fn draw_text(x: u32, y: u32, s: &str, fg: u32, bg: u32) {
    if !is_ready() { return; }
    let stride_pixels = stride() / 4;
    crate::ui::font::draw_str(framebuffer(), stride_pixels, x, y, s, fg, bg);
}

/// Render the boot splash. Safe to call multiple times; will redraw
/// the centered banner over a black background.
pub fn boot_splash() {
    if !is_ready() { return; }
    let w = width();
    let h = height();
    if w == 0 || h == 0 { return; }

    // Color literals authored as ARGB8888 for readability; re-encoded
    // to the M4 FB's native ARGB2101010 at const-eval time. Writing
    // ARGB8888 directly to this FB gives the wrong color (see
    // argb8888_to_m4 above / docs/M4_GROUND_TRUTH.md §3.1b).
    const BG:      u32 = argb8888_to_m4(0xFF00_0000); // black
    const C_TITLE: u32 = argb8888_to_m4(0xFFFF_C000); // amber bat-signal
    const C_SUB:   u32 = argb8888_to_m4(0xFF40_C0FF); // cool blue
    const C_DIM:   u32 = argb8888_to_m4(0xFF80_8080); // dim gray

    // Solid-black background.
    fill_screen(BG);

    // Title sized 4x normal (32x64 per char). We do this by drawing
    // the same glyph 4 times offset (cheap "scaling").
    let title = "SPHRAGIS";
    let title_w_px = title.len() as u32 * crate::ui::font::CHAR_W * 4;
    let tx = w.saturating_sub(title_w_px) / 2;
    let ty = h / 3;
    for sy in 0..4 {
        for sx in 0..4 {
            for (i, b) in title.bytes().enumerate() {
                let cx = tx + (i as u32) * crate::ui::font::CHAR_W * 4 + sx;
                let cy = ty + sy;
                let mut buf = [0u8; 1];
                buf[0] = b;
                let s = unsafe { core::str::from_utf8_unchecked(&buf) };
                let stride_pixels = stride() / 4;
                crate::ui::font::draw_str(
                    framebuffer(), stride_pixels,
                    cx, cy, s, C_TITLE, BG
                );
            }
        }
    }

    // Subtitle (normal size).
    let sub = "Bare Metal // Apple Silicon (M4 / T8132)";
    let sub_w_px = sub.len() as u32 * crate::ui::font::CHAR_W;
    let sx = w.saturating_sub(sub_w_px) / 2;
    let sy = ty + crate::ui::font::CHAR_H * 4 + 16;
    draw_text(sx, sy, sub, C_SUB, BG);

    // Footer.
    let foot = "[booted via m1n1 chainload]";
    let foot_w = foot.len() as u32 * crate::ui::font::CHAR_W;
    let fx = w.saturating_sub(foot_w) / 2;
    let fy = sy + crate::ui::font::CHAR_H * 2;
    draw_text(fx, fy, foot, C_DIM, BG);

    flush(0, 0, w, h);
}
