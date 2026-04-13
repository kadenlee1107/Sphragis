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
    unsafe { core::ptr::read_volatile((soc::UART0_BASE + offset) as *const u32) }
}

fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::UART0_BASE + offset) as *mut u32, val) }
}

/// Initialize the UART (assumes m1n1 already configured baud rate).
pub fn init() {
    // UART is typically already initialized by m1n1.
    // We just ensure FIFO is enabled and TX/RX are active.
    write32(UFCON, 0x1); // Enable FIFO
    write32(UCON, 0x5);  // Enable TX and RX, polling mode
}

/// Wait for TX buffer space and send a byte.
pub fn putc(c: u8) {
    // Wait for TX buffer empty
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
