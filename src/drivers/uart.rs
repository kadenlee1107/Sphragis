#![allow(dead_code)]
// Bat_OS — PL011 UART Driver
// Bidirectional serial I/O. Safe on platforms without UART.

use core::sync::atomic::{AtomicBool, Ordering};

const UART_BASE: usize = 0x0900_0000;
const UART_FR: usize = 0x018;
const FR_RXFE: u32 = 1 << 4;

static UART_ENABLED: AtomicBool = AtomicBool::new(false);

/// Call this after confirming UART exists (QEMU detection).
pub fn enable() {
    UART_ENABLED.store(true, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    UART_ENABLED.load(Ordering::Relaxed)
}

pub fn putc(c: u8) {
    if !UART_ENABLED.load(Ordering::Relaxed) { return; }
    unsafe {
        // Use 32-bit write for HVF compatibility
        core::ptr::write_volatile(UART_BASE as *mut u32, c as u32);
    }
}

pub fn puts(s: &str) {
    if !UART_ENABLED.load(Ordering::Relaxed) { return; }
    for byte in s.bytes() {
        if byte == b'\n' { putc(b'\r'); }
        putc(byte);
    }
}

/// STUMP #111 (audit M-uart-untrusted): print a slice that may have
/// originated from page DOM / network traffic — i.e. attacker-
/// influenced bytes. Replaces ANSI escapes / CR / NUL / other
/// control bytes with `?` before emit, defeating the
/// `\x1B[2J\x1B[H` clearscreen and `\r faked-line` overwrite
/// attacks. Use this for href values, form-action URLs, page
/// titles, anything where an attacker controls the bytes. The
/// trusted `puts` path stays untouched so kernel-internal status
/// lines keep their full formatting.
pub fn puts_safe(s: &str) {
    if !UART_ENABLED.load(Ordering::Relaxed) { return; }
    for byte in s.bytes() {
        if byte == b'\n' {
            putc(b'\r');
            putc(b'\n');
        } else if byte == b'\t' || (byte >= 0x20 && byte < 0x7F) {
            putc(byte);
        } else {
            putc(b'?');
        }
    }
}

pub fn has_char() -> bool {
    if !UART_ENABLED.load(Ordering::Relaxed) { return false; }
    unsafe {
        let flags = core::ptr::read_volatile((UART_BASE + UART_FR) as *const u32);
        flags & FR_RXFE == 0
    }
}

pub fn getc() -> Option<u8> {
    // Check virtio keyboard first (GUI window input)
    crate::drivers::virtio::keyboard::poll();
    if let Some(c) = crate::drivers::virtio::keyboard::getc() {
        return Some(c);
    }
    // Fall back to serial UART
    if !UART_ENABLED.load(Ordering::Relaxed) { return None; }
    if has_char() {
        unsafe { Some(core::ptr::read_volatile(UART_BASE as *const u32) as u8) }
    } else {
        None
    }
}
