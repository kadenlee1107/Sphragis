#![allow(dead_code)]
// Bat_OS — Apple Silicon Dockchannel UART driver.
//
// On M4 (and M2+ generally) the serial console is the "dockchannel"
// UART — a small FIFO-based block at 0x3_8812_8000 on T8132 with
// u32-register width. It is NOT a Samsung S5L UART (that's the
// M1-era block at a different address with different semantics).
//
// Register offsets and protocol ported from
// `external/m1n1/src/dockchannel_uart.c`:
//
//   DATA_TX8       = 0x4004  (write byte to TX FIFO)
//   DATA_TX_FREE   = 0x4014  (u32: bytes free in TX FIFO — wait for >0)
//   DATA_RX8       = 0x401c  (u32: RX byte in bits [15:8])
//   DATA_RX_COUNT  = 0x402c  (u32: bytes available in RX FIFO)
//
// m1n1 guarantees the block is already clocked and configured when
// it hands off, so we don't need an explicit `init()` — just respect
// TX_FREE before writing.

use super::soc;

const DATA_TX8:       usize = 0x4004;
const DATA_TX_FREE:   usize = 0x4014;
const DATA_RX8:       usize = 0x401c;
const DATA_RX_COUNT:  usize = 0x402c;

#[inline(always)]
fn read32(offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((soc::uart0_base() + offset) as *const u32) }
}

#[inline(always)]
fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::uart0_base() + offset) as *mut u32, val) }
}

/// Initialize the dockchannel UART. Nothing to do — m1n1 already
/// configured the baud rate and enabled it. Kept as a function so
/// existing `drivers::apple::uart::init()` callers still compile.
pub fn init() {}

/// Send one byte. Waits for TX FIFO space but bails after a bounded
/// number of spins so a misconfigured UART doesn't hang the whole
/// kernel. The upper cap is generous — ~a million iterations is
/// microseconds on real hardware but a hundred ms or so at M4's slow
/// pre-cpufreq boot clock.
///
/// Mirrors every byte into the on-screen `fb_console` so char-level
/// emitters like `kernel::mm::print_num` (which drives this via
/// `platform::serial_putc`) also show up on the Mac's display, not
/// just in the void of the dockchannel MMIO.
pub fn putc(c: u8) {
    // 🎯 STUMP #60 (HVF cleanup): on QEMU we MUST NOT touch the
    // dockchannel UART MMIO — it lives at PA 0x3_8812_8000 which is
    // unmapped on QEMU virt and any read faults. The platform layer
    // routes serial_putc to the right driver based on
    // `platform::current()`, but a few callers still reach in
    // directly (panic-adjacent paths, kernel_main_apple). Add a
    // platform gate so direct calls on QEMU silently no-op instead
    // of trying to read DATA_TX_FREE from a phantom address.
    if !crate::platform::is_apple_silicon() {
        return;
    }
    let mut guard: u32 = 1_000_000;
    while read32(DATA_TX_FREE) == 0 {
        guard = guard.saturating_sub(1);
        if guard == 0 {
            break; // give up writing to MMIO, but still mirror to FB
        }
        core::hint::spin_loop();
    }
    if guard > 0 {
        write32(DATA_TX8, c as u32);
    }
    // CR from puts's \n→CRLF translation is a no-op on the on-screen
    // console — fb_console already treats \r as "cursor to left
    // margin" without advancing the line. Passthrough.
    super::fb_console::putc(c);
}

/// Print a string. Translates `\n` to CRLF for the serial side.
/// `putc` already mirrors each byte to `fb_console`, so no explicit
/// fb_console::puts call is needed here (avoids double-rendering).
pub fn puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            putc(b'\r');
        }
        putc(byte);
    }
}

/// Print `val` as 8 hex digits (lower case, no prefix).
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

/// True if a byte is available in the RX FIFO.
pub fn has_char() -> bool {
    read32(DATA_RX_COUNT) != 0
}

/// Non-blocking read — returns `Some(b)` when a byte is ready,
/// `None` otherwise.
pub fn getc() -> Option<u8> {
    if has_char() {
        // m1n1 extracts the byte from bits [15:8]; match their
        // protocol.
        Some(((read32(DATA_RX8) >> 8) & 0xFF) as u8)
    } else {
        None
    }
}
