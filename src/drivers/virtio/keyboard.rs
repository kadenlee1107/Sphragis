#![allow(dead_code)]
// Bat_OS — VirtIO Keyboard Driver (HVF-safe)
// Reads key events from virtio-keyboard-device in QEMU GUI window.

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

static KBD_BASE: AtomicUsize = AtomicUsize::new(0);
static KBD_QUEUE: AtomicUsize = AtomicUsize::new(0);
static KBD_READY: AtomicBool = AtomicBool::new(false);
static KBD_BUFS: AtomicUsize = AtomicUsize::new(0);

/// Diagnostic counters — incremented on every event received from the
/// device. Surface them via the `kbd-stats` shell command so the
/// operator can answer "is QEMU sending keystrokes at all?"
static DBG_EVENTS_TOTAL: AtomicUsize = AtomicUsize::new(0);
static DBG_EVKEY_DOWN: AtomicUsize = AtomicUsize::new(0);
static DBG_EVKEY_UP: AtomicUsize = AtomicUsize::new(0);
static DBG_EVSYN: AtomicUsize = AtomicUsize::new(0);
static DBG_OTHER: AtomicUsize = AtomicUsize::new(0);
static DBG_LAST_CODE: AtomicUsize = AtomicUsize::new(0);
static DBG_LAST_TYPE: AtomicUsize = AtomicUsize::new(0);
static DBG_PUSHES: AtomicUsize = AtomicUsize::new(0);
static DBG_TRACE: AtomicBool = AtomicBool::new(false);

pub fn set_trace(on: bool) {
    DBG_TRACE.store(on, Ordering::Relaxed);
}

pub fn dbg_stats() -> (usize, usize, usize, usize, usize, usize, usize, usize) {
    (
        DBG_EVENTS_TOTAL.load(Ordering::Relaxed),
        DBG_EVKEY_DOWN.load(Ordering::Relaxed),
        DBG_EVKEY_UP.load(Ordering::Relaxed),
        DBG_EVSYN.load(Ordering::Relaxed),
        DBG_OTHER.load(Ordering::Relaxed),
        DBG_LAST_TYPE.load(Ordering::Relaxed),
        DBG_LAST_CODE.load(Ordering::Relaxed),
        DBG_PUSHES.load(Ordering::Relaxed),
    )
}

// Each event buffer is 8 bytes: type(u16) + code(u16) + value(u32)
const EVENT_SIZE: usize = 8;
const NUM_BUFS: usize = 32;

// Linux evdev keycode → ASCII
static KEYMAP: [u8; 128] = {
    let mut map = [0u8; 128];
    map[2] = b'1'; map[3] = b'2'; map[4] = b'3'; map[5] = b'4';
    map[6] = b'5'; map[7] = b'6'; map[8] = b'7'; map[9] = b'8';
    map[10] = b'9'; map[11] = b'0'; map[12] = b'-'; map[13] = b'=';
    map[14] = 0x08; map[15] = 0x09;
    map[16] = b'q'; map[17] = b'w'; map[18] = b'e'; map[19] = b'r';
    map[20] = b't'; map[21] = b'y'; map[22] = b'u'; map[23] = b'i';
    map[24] = b'o'; map[25] = b'p'; map[26] = b'['; map[27] = b']';
    map[28] = b'\r';
    map[30] = b'a'; map[31] = b's'; map[32] = b'd'; map[33] = b'f';
    map[34] = b'g'; map[35] = b'h'; map[36] = b'j'; map[37] = b'k';
    map[38] = b'l'; map[39] = b';'; map[40] = b'\''; map[41] = b'`';
    map[43] = b'\\';
    map[44] = b'z'; map[45] = b'x'; map[46] = b'c'; map[47] = b'v';
    map[48] = b'b'; map[49] = b'n'; map[50] = b'm'; map[51] = b',';
    map[52] = b'.'; map[53] = b'/';
    map[57] = b' ';
    map
};

const KEY_BUF_SIZE: usize = 64;
static mut KEY_BUF: [u8; KEY_BUF_SIZE] = [0; KEY_BUF_SIZE];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;

// Modifier key tracking
static mut CTRL_HELD: bool = false;
static mut SHIFT_HELD: bool = false;
static mut ALT_HELD: bool = false;

// STUMP #119: Caps Lock is a TOGGLE, not a held modifier. Each
// KEY_CAPSLOCK DOWN event flips the state. We expose `caps_active()`
// so the lock screen can render an honest "CAPS ON / OFF" indicator
// instead of the static label the design mocked up.
static mut CAPS_LOCK_ON: bool = false;

// Linux evdev keycodes for modifiers
const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_CAPSLOCK: u16 = 58;
const KEY_TAB: u16 = 15;
// STUMP #130: arrow keys aren't in KEYMAP (which only covers ASCII
// 32-126). Surface them as bytes 0x90..0x93 in the keystroke ring.
// 0x80 was already taken by Option+Tab. Apps that want arrow nav
// match these constants.
const KEY_UP:    u16 = 103;
const KEY_DOWN:  u16 = 108;
const KEY_LEFT:  u16 = 105;
const KEY_RIGHT: u16 = 106;
pub const KEY_ARROW_UP:    u8 = 0x90;
pub const KEY_ARROW_DOWN:  u8 = 0x91;
pub const KEY_ARROW_LEFT:  u8 = 0x92;
pub const KEY_ARROW_RIGHT: u8 = 0x93;

/// Public read of the caps-lock toggle. The lock screen and any
/// other UI that wants a "CAPS ON" badge should read it.
pub fn caps_active() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CAPS_LOCK_ON)) }
}

pub fn init() -> Option<()> {
    let devices = mmio::probe(18); // virtio input device type
    let base = devices[0]?;

    uart::puts("  [kbd] Found virtio-keyboard\n");

    let device = VirtioMmio::new(base);
    device.init_device().ok()?;

    let queue_mem = frame::alloc_frame()?;
    let queue_ptr = queue_mem as *mut Virtqueue;
    let queue = Virtqueue::new()?;
    unsafe { core::ptr::write(queue_ptr, queue); }

    KBD_QUEUE.store(queue_mem, Ordering::Relaxed);
    KBD_BASE.store(base, Ordering::Relaxed);

    let vq = unsafe { &mut *(queue_mem as *mut Virtqueue) };
    device.setup_queue(0, vq);
    device.driver_ok();

    // Allocate event buffers — each is EVENT_SIZE bytes
    let buf_page = frame::alloc_frame()?;
    KBD_BUFS.store(buf_page, Ordering::Relaxed);

    // Post individual receive buffers
    for i in 0..NUM_BUFS {
        let buf_addr = buf_page + i * EVENT_SIZE;
        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
    }
    device.notify(0);

    KBD_READY.store(true, Ordering::Relaxed);
    uart::puts("  [kbd] Keyboard ready (GUI input)\n");
    Some(())
}

pub fn poll() {
    if !KBD_READY.load(Ordering::Relaxed) { return; }

    let queue_addr = KBD_QUEUE.load(Ordering::Relaxed);
    let vq = unsafe { &mut *(queue_addr as *mut Virtqueue) };
    let buf_base = KBD_BUFS.load(Ordering::Relaxed);
    let base = KBD_BASE.load(Ordering::Relaxed);

    while let Some((id, _len)) = vq.poll_used() {
        // Read event from the specific buffer that was returned
        let buf_addr = buf_base + (id as usize % NUM_BUFS) * EVENT_SIZE;

        // Read event fields using safe reads
        let event_type = super::virtqueue::safe_read16(buf_addr);
        let code = super::virtqueue::safe_read16(buf_addr + 2);
        let value = super::virtqueue::safe_read32(buf_addr + 4);

        // Bump diagnostic counters BEFORE we filter, so we can see
        // EV_SYN / EV_REL etc. arriving even if they wouldn't push a
        // char into the keystroke ring.
        DBG_EVENTS_TOTAL.fetch_add(1, Ordering::Relaxed);
        DBG_LAST_TYPE.store(event_type as usize, Ordering::Relaxed);
        DBG_LAST_CODE.store(code as usize, Ordering::Relaxed);
        match event_type {
            0 => { DBG_EVSYN.fetch_add(1, Ordering::Relaxed); }
            1 => {
                if value == 1 { DBG_EVKEY_DOWN.fetch_add(1, Ordering::Relaxed); }
                else if value == 0 { DBG_EVKEY_UP.fetch_add(1, Ordering::Relaxed); }
            }
            _ => { DBG_OTHER.fetch_add(1, Ordering::Relaxed); }
        }
        // Live trace toggle: when DBG_TRACE is true, every event prints
        // a one-line "[kbd] ev type=X code=Y val=Z" to UART. Off by
        // default to keep boot logs clean; flip on with `kbd-trace on`.
        if DBG_TRACE.load(Ordering::Relaxed) {
            uart::puts("[kbd] ev type=");
            crate::kernel::mm::print_num(event_type as usize);
            uart::puts(" code=");
            crate::kernel::mm::print_num(code as usize);
            uart::puts(" val=");
            crate::kernel::mm::print_num(value as usize);
            uart::puts("\n");
        }

        // EV_KEY (type=1)
        if event_type == 1 {
            // Track modifier key state (DOWN=1, UP=0)
            unsafe {
                match code {
                    KEY_LEFTCTRL | KEY_RIGHTCTRL => { CTRL_HELD = value == 1; }
                    KEY_LEFTSHIFT | KEY_RIGHTSHIFT => { SHIFT_HELD = value == 1; }
                    KEY_LEFTALT | KEY_RIGHTALT => { ALT_HELD = value == 1; }
                    // STUMP #119: caps-lock is a press-to-toggle key,
                    // not held. Flip on DOWN only; ignore UP.
                    KEY_CAPSLOCK => {
                        if value == 1 { CAPS_LOCK_ON = !CAPS_LOCK_ON; }
                    }
                    _ => {}
                }
            }

            // Key DOWN (value=1) — generate character
            if value == 1 {
                unsafe {
                    // Option+Tab → send special code 0x80 (split focus switch)
                    if ALT_HELD && code == KEY_TAB {
                        push_key(0x80);
                        // Don't continue — fall through to re-post buffer below
                        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
                        let device = VirtioMmio::new(base);
                        device.notify(0);
                        continue;
                    }
                    // STUMP #130: arrow keys → 0x90..0x93.
                    let arrow = match code {
                        KEY_UP    => Some(KEY_ARROW_UP),
                        KEY_DOWN  => Some(KEY_ARROW_DOWN),
                        KEY_LEFT  => Some(KEY_ARROW_LEFT),
                        KEY_RIGHT => Some(KEY_ARROW_RIGHT),
                        _ => None,
                    };
                    if let Some(b) = arrow {
                        push_key(b);
                        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
                        let device = VirtioMmio::new(base);
                        device.notify(0);
                        continue;
                    }
                }

                let code_idx = code as usize;
                if code_idx < 128 {
                    let mut ch = KEYMAP[code_idx];
                    if ch != 0 {
                        unsafe {
                            if CTRL_HELD && ch >= b'a' && ch <= b'z' {
                                ch = ch - b'a' + 1;
                            } else {
                                // STUMP #119: caps lock XORs with shift
                                // for alpha keys (so caps+shift cancels
                                // out, like every other OS). Number-row
                                // symbols are SHIFT-only — caps lock
                                // doesn't affect them.
                                let alpha_upper = SHIFT_HELD ^ CAPS_LOCK_ON;
                                if alpha_upper && ch >= b'a' && ch <= b'z' {
                                    ch -= 32;
                                }
                                if SHIFT_HELD {
                                    ch = match ch {
                                        b'1' => b'!', b'2' => b'@', b'3' => b'#',
                                        b'4' => b'$', b'5' => b'%', b'6' => b'^',
                                        b'7' => b'&', b'8' => b'*', b'9' => b'(',
                                        b'0' => b')', b'-' => b'_', b'=' => b'+',
                                        b'[' => b'{', b']' => b'}', b'\\' => b'|',
                                        b';' => b':', b'\'' => b'"', b'`' => b'~',
                                        b',' => b'<', b'.' => b'>', b'/' => b'?',
                                        _ => ch,
                                    };
                                }
                            }
                        }
                        push_key(ch);
                    }
                }
            }
        }

        // Re-post this buffer for the next event
        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
        let device = VirtioMmio::new(base);
        device.notify(0);
    }
}

fn push_key(ch: u8) {
    DBG_PUSHES.fetch_add(1, Ordering::Relaxed);
    unsafe {
        let next = (KEY_HEAD + 1) % KEY_BUF_SIZE;
        if next != KEY_TAIL {
            KEY_BUF[KEY_HEAD] = ch;
            KEY_HEAD = next;
        }
    }
}

pub fn getc() -> Option<u8> {
    unsafe {
        if KEY_HEAD == KEY_TAIL { return None; }
        let ch = KEY_BUF[KEY_TAIL];
        KEY_TAIL = (KEY_TAIL + 1) % KEY_BUF_SIZE;
        Some(ch)
    }
}

pub fn is_ready() -> bool {
    KBD_READY.load(Ordering::Relaxed)
}

/// V11-state-sweep: wipe the keystroke ring buffer + modifier flags on
/// cave switch. THIS IS CRITICAL — without this, any typed passphrase,
/// SSH key passphrase, or plaintext message in the outgoing cave remains
/// readable by the new cave's first `getc()` loop.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let p = core::ptr::addr_of_mut!(KEY_BUF) as *mut u8;
        for i in 0..KEY_BUF_SIZE {
            core::ptr::write_volatile(p.add(i), 0);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(KEY_HEAD), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(KEY_TAIL), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CTRL_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SHIFT_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(ALT_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CAPS_LOCK_ON), false);
    }
}
