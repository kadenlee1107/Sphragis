//! Platform-neutral GPU facade for the UI layer.
//!
//! All existing ui::* code targets `drivers::virtio::gpu` directly,
//! which only works on QEMU. This module re-exports the same API but
//! dispatches to either `virtio::gpu` (QEMU) or `apple::dcp` (Apple
//! Silicon) based on `platform::current()`.
//!
//! Colour handling: callers pass ARGB8888 (`0xAARRGGBB`) — the same
//! colour constants the whole UI already uses. On Apple M4 we pipe
//! them through `apple::dcp::argb8888_to_m4()` which packs them into
//! the panel's native ARGB2101010 word before writing. On QEMU the
//! colour is written as-is.

use crate::platform::{self, Platform};

#[inline]
fn convert_color(color_argb8888: u32) -> u32 {
    match platform::current() {
        Platform::AppleSilicon => {
            crate::drivers::apple::dcp::argb8888_to_m4(color_argb8888)
        }
        Platform::QemuVirt => color_argb8888,
    }
}

pub fn width() -> u32 {
    platform::display_width()
}

pub fn height() -> u32 {
    platform::display_height()
}

pub fn framebuffer() -> *mut u32 {
    platform::display_framebuffer()
}

pub fn fill_screen(color_argb8888: u32) {
    platform::display_fill_screen(convert_color(color_argb8888))
}

pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color_argb8888: u32) {
    platform::display_fill_rect(x, y, w, h, convert_color(color_argb8888))
}

pub fn flush(x: u32, y: u32, w: u32, h: u32) {
    platform::display_flush(x, y, w, h)
}
