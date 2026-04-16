#![allow(dead_code)]
// Bat_OS — Apple Display Controller (DCP) Driver
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
