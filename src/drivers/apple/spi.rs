// Bat_OS — Apple SPI Keyboard/Trackpad Driver
// MacBook keyboard and trackpad connect over SPI.
// The keyboard sends HID reports via SPI packets.
// Reference: Asahi Linux drivers/input/keyboard/apple-spi-keyboard.c
//            Asahi Linux drivers/spi/spi-apple.c

use super::soc;
use core::sync::atomic::{AtomicBool, Ordering};

// Apple SPI Controller Registers
const SPI_CTRL: usize = 0x000;
const SPI_CONFIG: usize = 0x004;
const SPI_STATUS: usize = 0x008;
const SPI_PIN: usize = 0x00C;
const SPI_TXDATA: usize = 0x010;
const SPI_RXDATA: usize = 0x020;
const SPI_CLKDIV: usize = 0x030;
const SPI_RXCNT: usize = 0x034;
const SPI_TXCNT: usize = 0x04C;
const SPI_FIFOSTAT: usize = 0x010C;
const SPI_SHIFTCONFIG: usize = 0x0150;
const SPI_PINCONFIG: usize = 0x0160;

// SPI keyboard packet types
const SPI_HID_REPORT_TYPE: u8 = 0x10;
const SPI_KBD_REPORT_ID: u8 = 0x01;

// HID Keyboard scan codes → ASCII (simplified mapping)
const KEY_MAP: [u8; 128] = {
    let mut map = [0u8; 128];
    // Letters (USB HID usage IDs)
    map[4] = b'a'; map[5] = b'b'; map[6] = b'c'; map[7] = b'd';
    map[8] = b'e'; map[9] = b'f'; map[10] = b'g'; map[11] = b'h';
    map[12] = b'i'; map[13] = b'j'; map[14] = b'k'; map[15] = b'l';
    map[16] = b'm'; map[17] = b'n'; map[18] = b'o'; map[19] = b'p';
    map[20] = b'q'; map[21] = b'r'; map[22] = b's'; map[23] = b't';
    map[24] = b'u'; map[25] = b'v'; map[26] = b'w'; map[27] = b'x';
    map[28] = b'y'; map[29] = b'z';
    // Numbers
    map[30] = b'1'; map[31] = b'2'; map[32] = b'3'; map[33] = b'4';
    map[34] = b'5'; map[35] = b'6'; map[36] = b'7'; map[37] = b'8';
    map[38] = b'9'; map[39] = b'0';
    // Special
    map[40] = b'\r'; // Enter
    map[41] = 0x1B;  // Escape
    map[42] = 0x08;  // Backspace
    map[43] = 0x09;  // Tab
    map[44] = b' ';  // Space
    map[45] = b'-'; map[46] = b'=';
    map[47] = b'['; map[48] = b']';
    map[49] = b'\\';
    map[51] = b';'; map[52] = b'\'';
    map[53] = b'`';
    map[54] = b','; map[55] = b'.'; map[56] = b'/';
    map
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Key event buffer
const KEY_BUF_SIZE: usize = 32;
static mut KEY_BUF: [u8; KEY_BUF_SIZE] = [0; KEY_BUF_SIZE];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;

fn read32(offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((soc::SPI0_BASE + offset) as *const u32) }
}

fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::SPI0_BASE + offset) as *mut u32, val) }
}

/// Initialize the SPI controller for keyboard communication.
pub fn init() -> Result<(), &'static str> {
    // Configure SPI controller
    // Apple SPI uses specific shift/pin configurations
    write32(SPI_SHIFTCONFIG, 0x0); // MSB first
    write32(SPI_PINCONFIG, 0x2);   // CS active low
    write32(SPI_CLKDIV, 0x4);     // Clock divider

    // Enable controller
    write32(SPI_CONFIG, 0x1);

    INITIALIZED.store(true, Ordering::Relaxed);
    Ok(())
}

/// Poll the SPI keyboard for key events.
/// Called from the main input loop.
pub fn poll() {
    if !INITIALIZED.load(Ordering::Relaxed) { return; }

    // Check if SPI has data
    let fifo = read32(SPI_FIFOSTAT);
    let rx_count = fifo & 0xFF;

    if rx_count >= 8 {
        // Read SPI packet
        let mut packet = [0u32; 8];
        for i in 0..8 {
            packet[i] = read32(SPI_RXDATA);
        }

        // Parse HID keyboard report
        let report_type = (packet[0] & 0xFF) as u8;
        if report_type == SPI_HID_REPORT_TYPE {
            let report_id = ((packet[0] >> 8) & 0xFF) as u8;
            if report_id == SPI_KBD_REPORT_ID {
                // Standard HID keyboard report: modifier, reserved, keys[6]
                let modifier = ((packet[1] >> 0) & 0xFF) as u8;
                let key0 = ((packet[1] >> 16) & 0xFF) as u8;

                if key0 > 0 && (key0 as usize) < KEY_MAP.len() {
                    let mut ch = KEY_MAP[key0 as usize];
                    if ch != 0 {
                        // Handle Shift modifier
                        let shift = modifier & 0x22 != 0; // Left or Right Shift
                        if shift && ch >= b'a' && ch <= b'z' {
                            ch -= 32; // Uppercase
                        }

                        // Handle Ctrl modifier
                        let ctrl = modifier & 0x11 != 0;
                        if ctrl && ch >= b'a' && ch <= b'z' {
                            ch = ch - b'a' + 1; // Ctrl+A = 0x01, etc.
                        }

                        push_key(ch);
                    }
                }
            }
        }
    }
}

fn push_key(ch: u8) {
    unsafe {
        let next = (KEY_HEAD + 1) % KEY_BUF_SIZE;
        if next != KEY_TAIL {
            KEY_BUF[KEY_HEAD] = ch;
            KEY_HEAD = next;
        }
    }
}

/// Get the next key event (non-blocking).
pub fn getc() -> Option<u8> {
    unsafe {
        if KEY_HEAD == KEY_TAIL {
            return None;
        }
        let ch = KEY_BUF[KEY_TAIL];
        KEY_TAIL = (KEY_TAIL + 1) % KEY_BUF_SIZE;
        Some(ch)
    }
}

pub fn is_ready() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}
