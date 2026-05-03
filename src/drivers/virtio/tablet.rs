#![allow(dead_code)]
// Bat_OS — VirtIO tablet (absolute pointer) driver.
//
// Sprint 1.5 (STUMP #98). Mirrors `keyboard.rs`. QEMU exposes the
// tablet at the same `virtio-input` device-type as the keyboard, so
// we probe ALL input devices and take the SECOND one — the contract
// is that QEMU is launched with `-device virtio-keyboard-device
// -device virtio-tablet-device` in that order. (See
// scripts/render_live.py.)
//
// Wire format is the standard 8-byte Linux evdev event:
//   u16 type, u16 code, u32 value
// The events we care about:
//   * EV_ABS (3) + ABS_X (0)/ABS_Y (1): absolute position in
//     0..=device_max (default 32767 for QEMU). We rescale to the
//     virtio-gpu framebuffer dimensions before exposing to callers.
//   * EV_KEY (1) + BTN_LEFT (0x110): left mouse button. value=1 down,
//     value=0 up. (BTN_RIGHT/BTN_MIDDLE plumbed but never fire in our
//     flows yet.)
//   * EV_SYN (0) + SYN_REPORT (0): end of event group; we use this
//     as the commit point and emit a single `Move` (or `ButtonDown`/
//     `ButtonUp` if the group also crossed a button-state edge).

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, AtomicI32, Ordering};

static TBL_BASE: AtomicUsize = AtomicUsize::new(0);
static TBL_QUEUE: AtomicUsize = AtomicUsize::new(0);
static TBL_READY: AtomicBool = AtomicBool::new(false);
static TBL_BUFS: AtomicUsize = AtomicUsize::new(0);

const EVENT_SIZE: usize = 8;
const NUM_BUFS: usize = 32;

// Event-type constants per linux/input-event-codes.h
const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_REL: u16 = 2;
const EV_ABS: u16 = 3;
const ABS_X: u16 = 0;
const ABS_Y: u16 = 1;
const REL_X: u16 = 0;
const REL_Y: u16 = 1;
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;

// QEMU virtio-tablet default range. The protocol allows querying
// absinfo at config time, but the QEMU default is consistent enough
// that hard-coding the max keeps the driver tiny.
const TABLET_MAX: i32 = 32767;

/// Last-committed cursor state. Updated on SYN_REPORT.
static CURSOR_X: AtomicI32 = AtomicI32::new(0);
static CURSOR_Y: AtomicI32 = AtomicI32::new(0);
static BUTTON_DOWN: AtomicBool = AtomicBool::new(false);

/// Pending state inside an event group (between EV_ABS calls and the
/// final EV_SYN). We collect axis updates here and commit on SYN.
static PENDING_X: AtomicI32 = AtomicI32::new(0);
static PENDING_Y: AtomicI32 = AtomicI32::new(0);
static PENDING_BTN: AtomicI32 = AtomicI32::new(-1); // -1 = no edge, 0/1 = up/down

// STUMP #100b: QEMU routes ALL keyboard input to virtio-tablet when
// the tablet device is attached (the tablet model claims the key-
// event capability and steals it from virtio-keyboard). So this
// driver also has to decode EV_KEY events into ASCII and surface
// them via getc_key() so the interactive loop sees them. Same shape
// as keyboard.rs.
const KEY_BUF_SIZE: usize = 64;
static mut KEY_BUF: [u8; KEY_BUF_SIZE] = [0; KEY_BUF_SIZE];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;
static mut CTRL_HELD: bool = false;
static mut SHIFT_HELD: bool = false;
static mut ALT_HELD: bool = false;
// STUMP #119: caps lock toggle, mirrored from keyboard.rs because
// QEMU's input multiplexer routes EV_KEY to whichever device claimed
// the capability first — caps-lock presses can land here on Mac
// cocoa just like alpha keys do.
static mut CAPS_LOCK_ON: bool = false;

const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_CAPSLOCK: u16 = 58;
// STUMP #130: arrow keys, mirroring keyboard.rs.
const KEY_UP_C:    u16 = 103;
const KEY_DOWN_C:  u16 = 108;
const KEY_LEFT_C:  u16 = 105;
const KEY_RIGHT_C: u16 = 106;

pub fn caps_active() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CAPS_LOCK_ON)) }
}

// Linux evdev keycode → ASCII (subset that matters for the renderer).
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
    map[1] = 27; // ESC
    map
};

fn push_key(ch: u8) {
    unsafe {
        let next = (KEY_HEAD + 1) % KEY_BUF_SIZE;
        if next != KEY_TAIL {
            KEY_BUF[KEY_HEAD] = ch;
            KEY_HEAD = next;
        }
    }
}

/// Pop the next decoded ASCII keystroke that arrived through the
/// tablet's mis-routed EV_KEY stream. Mirrors keyboard::getc.
pub fn getc_key() -> Option<u8> {
    unsafe {
        if KEY_HEAD == KEY_TAIL { return None; }
        let ch = KEY_BUF[KEY_TAIL];
        KEY_TAIL = (KEY_TAIL + 1) % KEY_BUF_SIZE;
        Some(ch)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    Move { x: i32, y: i32 },
    ButtonDown { x: i32, y: i32, button: u8 },
    ButtonUp   { x: i32, y: i32, button: u8 },
}

const RING_CAP: usize = 64;
static mut EV_RING: [Option<InputEvent>; RING_CAP] = [None; RING_CAP];
static mut EV_HEAD: usize = 0;
static mut EV_TAIL: usize = 0;

pub fn init() -> Option<()> {
    let devices = mmio::probe(18);
    // Take the SECOND input device — the first is the keyboard. If
    // only one input device was attached we have no tablet (most
    // headless test runs).
    let base = devices[1]?;
    uart::puts("  [tbl] Found virtio-tablet at slot 1\n");

    let device = VirtioMmio::new(base);
    device.init_device().ok()?;

    let queue_mem = frame::alloc_frame()?;
    let queue_ptr = queue_mem as *mut Virtqueue;
    let queue = Virtqueue::new()?;
    unsafe { core::ptr::write(queue_ptr, queue); }

    TBL_QUEUE.store(queue_mem, Ordering::Relaxed);
    TBL_BASE.store(base, Ordering::Relaxed);

    let vq = unsafe { &mut *(queue_mem as *mut Virtqueue) };
    device.setup_queue(0, vq);
    device.driver_ok();

    let buf_page = frame::alloc_frame()?;
    TBL_BUFS.store(buf_page, Ordering::Relaxed);
    for i in 0..NUM_BUFS {
        let buf_addr = buf_page + i * EVENT_SIZE;
        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
    }
    device.notify(0);

    TBL_READY.store(true, Ordering::Relaxed);
    uart::puts("  [tbl] Tablet ready\n");
    Some(())
}

pub fn is_ready() -> bool { TBL_READY.load(Ordering::Relaxed) }

pub fn cursor_xy() -> (i32, i32) {
    (CURSOR_X.load(Ordering::Relaxed), CURSOR_Y.load(Ordering::Relaxed))
}

pub fn poll() {
    if !TBL_READY.load(Ordering::Relaxed) { return; }
    let queue_addr = TBL_QUEUE.load(Ordering::Relaxed);
    let vq = unsafe { &mut *(queue_addr as *mut Virtqueue) };
    let buf_base = TBL_BUFS.load(Ordering::Relaxed);
    let base = TBL_BASE.load(Ordering::Relaxed);

    while let Some((id, _len)) = vq.poll_used() {
        let buf_addr = buf_base + (id as usize % NUM_BUFS) * EVENT_SIZE;
        let etype = super::virtqueue::safe_read16(buf_addr);
        let code  = super::virtqueue::safe_read16(buf_addr + 2);
        let value = super::virtqueue::safe_read32(buf_addr + 4) as i32;

        // Diagnostic: log every event so we can tell whether QMP-
        // injected motion is arriving at the virtio-mouse device.
        // Drop once the bridge is confirmed end-to-end.
        if etype == EV_REL || etype == EV_ABS || (etype == EV_KEY && code >= 0x110) {
            uart::puts("    [tbl] ev type=");
            crate::kernel::mm::print_num(etype as usize);
            uart::puts(" code=");
            crate::kernel::mm::print_num(code as usize);
            uart::puts(" value=");
            crate::kernel::mm::print_num(value as usize);
            uart::puts("\n");
        }

        match etype {
            EV_ABS => {
                // Rescale tablet coords to GPU framebuffer coords.
                // Cap at FB-1 so a click on the right edge maps to a
                // valid pixel.
                let fb_w = super::gpu::width().max(1) as i32;
                let fb_h = super::gpu::height().max(1) as i32;
                if code == ABS_X {
                    let sx = (value.max(0) * (fb_w - 1)) / TABLET_MAX;
                    PENDING_X.store(sx, Ordering::Relaxed);
                } else if code == ABS_Y {
                    let sy = (value.max(0) * (fb_h - 1)) / TABLET_MAX;
                    PENDING_Y.store(sy, Ordering::Relaxed);
                }
            }
            EV_REL => {
                // STUMP #109 — relative motion from virtio-mouse-device.
                // QEMU cocoa drops EV_ABS events to virtio-tablet on Mac
                // (host cursor hides + motion never fires) but virtio-
                // mouse uses a different code path that DOES deliver REL
                // motion. Track an absolute (x,y) internally by
                // accumulating the deltas; clamp to FB bounds.
                let fb_w = super::gpu::width().max(1) as i32;
                let fb_h = super::gpu::height().max(1) as i32;
                if code == REL_X {
                    let cur = PENDING_X.load(Ordering::Relaxed);
                    PENDING_X.store((cur + value).clamp(0, fb_w - 1), Ordering::Relaxed);
                } else if code == REL_Y {
                    let cur = PENDING_Y.load(Ordering::Relaxed);
                    PENDING_Y.store((cur + value).clamp(0, fb_h - 1), Ordering::Relaxed);
                }
            }
            EV_KEY => {
                if code == BTN_LEFT {
                    PENDING_BTN.store(value & 1, Ordering::Relaxed);
                } else if code == BTN_RIGHT || code == BTN_MIDDLE {
                    // Plumbed but not surfaced.
                } else {
                    // STUMP #100b: alphanumeric / control key — QEMU
                    // mis-routed it to us instead of virtio-keyboard.
                    // Decode and stash so the interactive loop can
                    // pick it up via tablet::getc_key().
                    unsafe {
                        match code {
                            KEY_LEFTCTRL | KEY_RIGHTCTRL  => CTRL_HELD  = value == 1,
                            KEY_LEFTSHIFT | KEY_RIGHTSHIFT => SHIFT_HELD = value == 1,
                            KEY_LEFTALT | KEY_RIGHTALT   => ALT_HELD   = value == 1,
                            // STUMP #119: caps-lock toggles on DOWN.
                            KEY_CAPSLOCK => {
                                if value == 1 { CAPS_LOCK_ON = !CAPS_LOCK_ON; }
                            }
                            _ => {}
                        }
                    }
                    if value == 1 {
                        // STUMP #130: arrow keys → 0x90..0x93 in the
                        // tablet ring too (QEMU may route keys here).
                        let arrow = match code {
                            KEY_UP_C    => Some(0x90u8),
                            KEY_DOWN_C  => Some(0x91u8),
                            KEY_LEFT_C  => Some(0x92u8),
                            KEY_RIGHT_C => Some(0x93u8),
                            _ => None,
                        };
                        if let Some(b) = arrow {
                            push_key(b);
                            // Re-post buffer + notify before falling
                            // through to alpha-key handling.
                            vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
                            let device = VirtioMmio::new(base);
                            device.notify(0);
                            continue;
                        }
                        let idx = code as usize;
                        if idx < KEYMAP.len() {
                            let mut ch = KEYMAP[idx];
                            if ch != 0 {
                                unsafe {
                                    if CTRL_HELD && ch >= b'a' && ch <= b'z' {
                                        ch = ch - b'a' + 1;
                                    } else {
                                        // STUMP #119: caps XOR shift on alpha;
                                        // shift-only on number-row symbols.
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
            }
            EV_SYN => {
                // Commit: pending → cursor, emit ring entry.
                let nx = PENDING_X.load(Ordering::Relaxed);
                let ny = PENDING_Y.load(Ordering::Relaxed);
                let prev_x = CURSOR_X.load(Ordering::Relaxed);
                let prev_y = CURSOR_Y.load(Ordering::Relaxed);
                CURSOR_X.store(nx, Ordering::Relaxed);
                CURSOR_Y.store(ny, Ordering::Relaxed);

                let btn_edge = PENDING_BTN.swap(-1, Ordering::Relaxed);
                if btn_edge == 1 {
                    BUTTON_DOWN.store(true, Ordering::Relaxed);
                    push_event(InputEvent::ButtonDown { x: nx, y: ny, button: 0 });
                } else if btn_edge == 0 {
                    BUTTON_DOWN.store(false, Ordering::Relaxed);
                    push_event(InputEvent::ButtonUp { x: nx, y: ny, button: 0 });
                } else if nx != prev_x || ny != prev_y {
                    push_event(InputEvent::Move { x: nx, y: ny });
                }
            }
            _ => {}
        }

        vq.add_writable(buf_addr as *mut u8, EVENT_SIZE as u32);
        let device = VirtioMmio::new(base);
        device.notify(0);
    }
}

fn push_event(ev: InputEvent) {
    unsafe {
        let next = (EV_HEAD + 1) % RING_CAP;
        if next != EV_TAIL {
            EV_RING[EV_HEAD] = Some(ev);
            EV_HEAD = next;
        }
    }
}

/// Pop the next pending input event, if any.
pub fn pop_event() -> Option<InputEvent> {
    unsafe {
        if EV_HEAD == EV_TAIL { return None; }
        let ev = EV_RING[EV_TAIL].take()?;
        EV_TAIL = (EV_TAIL + 1) % RING_CAP;
        Some(ev)
    }
}

/// Same security sweep as keyboard.rs — wipe pending pointer state on
/// cave switch so a privileged cursor position doesn't leak.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..RING_CAP {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(EV_RING[i]), None);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(EV_HEAD), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(EV_TAIL), 0);
    }
    CURSOR_X.store(0, Ordering::Relaxed);
    CURSOR_Y.store(0, Ordering::Relaxed);
    BUTTON_DOWN.store(false, Ordering::Relaxed);
    PENDING_X.store(0, Ordering::Relaxed);
    PENDING_Y.store(0, Ordering::Relaxed);
    PENDING_BTN.store(-1, Ordering::Relaxed);
    // STUMP #119: reset modifier + caps-lock state too.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CTRL_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SHIFT_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(ALT_HELD), false);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CAPS_LOCK_ON), false);
    }
}
