#![allow(dead_code)]
// Sphragis — Platform Abstraction Layer
// Same kernel/UI code runs on QEMU (virtio) or Apple Silicon (real HW).
// This module provides a unified interface that dispatches to the right driver.

#[derive(Clone, Copy, PartialEq)]
pub enum Platform {
    QemuVirt,
    AppleSilicon,
}

use core::sync::atomic::{AtomicU8, Ordering};
static CURRENT_PLATFORM: AtomicU8 = AtomicU8::new(0); // 0 = QEMU, 1 = Apple

pub fn set_platform(p: Platform) {
    CURRENT_PLATFORM.store(p as u8, Ordering::Relaxed);
}

pub fn current() -> Platform {
    match CURRENT_PLATFORM.load(Ordering::Relaxed) {
        1 => Platform::AppleSilicon,
        _ => Platform::QemuVirt,
    }
}

#[inline]
pub fn is_apple_silicon() -> bool {
    matches!(current(), Platform::AppleSilicon)
}

// ─── Unified Serial I/O ───

pub fn serial_putc(c: u8) {
    match current() {
        Platform::QemuVirt => crate::drivers::uart::putc(c),
        Platform::AppleSilicon => {
            crate::drivers::apple::uart::putc(c);
            // Piggy-back the SMC keepalive on every byte the guest
            // emits. Rate-limited internally to ~10 Hz so heavy
            // output doesn't flood the SoC fabric.
            crate::ui::shell::smc_keepalive_tick();
        }
    }
}

pub fn serial_puts(s: &str) {
    match current() {
        Platform::QemuVirt => crate::drivers::uart::puts(s),
        Platform::AppleSilicon => {
            crate::drivers::apple::uart::puts(s);
            crate::ui::shell::smc_keepalive_tick();
        }
    }
}

pub fn serial_getc() -> Option<u8> {
    match current() {
        Platform::QemuVirt => crate::drivers::uart::getc(),
        Platform::AppleSilicon => {
            // Check SPI keyboard first, then UART
            if let Some(c) = crate::drivers::apple::spi::getc() {
                return Some(c);
            }
            crate::drivers::apple::uart::getc()
        }
    }
}

// ─── Unified Display ───

pub fn display_width() -> u32 {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::width(),
        Platform::AppleSilicon => crate::drivers::apple::dcp::width(),
    }
}

pub fn display_height() -> u32 {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::height(),
        Platform::AppleSilicon => crate::drivers::apple::dcp::height(),
    }
}

pub fn display_framebuffer() -> *mut u32 {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::framebuffer(),
        Platform::AppleSilicon => crate::drivers::apple::dcp::framebuffer(),
    }
}

pub fn display_fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::fill_rect(x, y, w, h, color),
        Platform::AppleSilicon => crate::drivers::apple::dcp::fill_rect(x, y, w, h, color),
    }
}

pub fn display_fill_screen(color: u32) {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::fill_screen(color),
        Platform::AppleSilicon => crate::drivers::apple::dcp::fill_screen(color),
    }
}

pub fn display_flush(x: u32, y: u32, w: u32, h: u32) {
    match current() {
        Platform::QemuVirt => crate::drivers::virtio::gpu::flush(x, y, w, h),
        Platform::AppleSilicon => crate::drivers::apple::dcp::flush(x, y, w, h),
    }
}
