#![allow(dead_code)]
// Bat_OS — Apple Silicon UART Driver
// Samsung S5L-derived UART found on all Apple Silicon SoCs.
// m1n1 exposes serial over USB-C, this driver talks directly to the UART block.
// Reference: Asahi Linux drivers/tty/serial/samsung_s5l.c

use super::soc;

// S5L UART Register Offsets
const ULCON: usize = 0x000;    // Line control
const UCON: usize = 0x004;     // Control
const UFCON: usize = 0x008;    // FIFO control
const UTRSTAT: usize = 0x010;  // TX/RX status
const UTXH: usize = 0x020;     // TX buffer
const URXH: usize = 0x024;     // RX buffer

// Status bits
const UTRSTAT_TXBE: u32 = 1 << 1;  // TX buffer empty
const UTRSTAT_RXDA: u32 = 1 << 0;  // RX data available

fn read32(offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((soc::uart0_base() + offset) as *const u32) }
}

fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::uart0_base() + offset) as *mut u32, val) }
}

/// M4 bring-up: this driver targets the Samsung S5L UART layout
/// used on M1/M2. M4 uses the dockchannel UART at 0x3_8812_8000 with
/// a different register layout. Until we port a dockchannel driver,
/// `init()` and `putc()` are no-ops gated by `UART_READY`; callers
/// (many `uart::puts(...)` lines in kernel_main_apple) stay in the
/// code but silently do nothing, which is better than writing
/// S5L-layout config bytes to dockchannel MMIO.
use core::sync::atomic::{AtomicBool, Ordering};
static UART_READY: AtomicBool = AtomicBool::new(false);

pub fn init() {
    // UART disabled for M4 bring-up — see note above. Leaving this
    // function intact so existing `drivers::apple::uart::init()`
    // call sites don't need to change.
    if !UART_READY.load(Ordering::Relaxed) {
        return;
    }
    write32(UFCON, 0x1); // Enable FIFO
    write32(UCON, 0x5);  // Enable TX and RX, polling mode
}

/// Wait for TX buffer space and send a byte.
pub fn putc(c: u8) {
    if !UART_READY.load(Ordering::Relaxed) {
        return;
    }
    while read32(UTRSTAT) & UTRSTAT_TXBE == 0 {
        core::hint::spin_loop();
    }
    write32(UTXH, c as u32);
}

/// Print a string.
pub fn puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            putc(b'\r');
        }
        putc(byte);
    }
}

/// Print `val` as 8 hex digits (upper case, no prefix).
pub fn puthex32(val: u32) {
    const HX: &[u8; 16] = b"0123456789abcdef";
    for i in (0..8).rev() {
        let nib = ((val >> (i * 4)) & 0xF) as usize;
        putc(HX[nib]);
    }
}

/// Print `val` as 16 hex digits.
pub fn puthex64(val: u64) {
    puthex32((val >> 32) as u32);
    puthex32(val as u32);
}

/// Check if a character is available.
pub fn has_char() -> bool {
    read32(UTRSTAT) & UTRSTAT_RXDA != 0
}

/// Read a character (non-blocking).
pub fn getc() -> Option<u8> {
    if has_char() {
        Some((read32(URXH) & 0xFF) as u8)
    } else {
        None
    }
}
