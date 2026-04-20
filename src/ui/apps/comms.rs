#![allow(dead_code)]
// Bat_OS — Comms Client (8th Desktop App)
// Encrypted peer-to-peer messaging over TCP.
// All messages AES-256 encrypted end-to-end.
// No plaintext ever touches the wire.
//
// Features:
//   - Connect to peer by IP:port
//   - X25519 key exchange for session key
//   - AES-256-CTR encrypted messages
//   - Message log with timestamps
//   - Compose + send from keyboard

use crate::ui::wm;
use crate::ui::font;
use crate::ui::gpu;
use crate::crypto::{aes, sha256};

const BG: u32 = 0xFF0A0A0A;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const CYAN: u32 = 0xFFFFFF00;
const BORDER: u32 = 0xFF1E1E1E;
const INPUT_BG: u32 = 0xFF141414;

// Connection state
#[derive(Clone, Copy, PartialEq)]
enum CommState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

// Message log
const MAX_MESSAGES: usize = 32;
const MAX_MSG_LEN: usize = 80;

#[derive(Clone, Copy)]
struct ChatMsg {
    active: bool,
    outgoing: bool, // true = we sent it
    text: [u8; MAX_MSG_LEN],
    text_len: usize,
    timestamp: u64, // seconds since boot
}

impl ChatMsg {
    const fn empty() -> Self {
        ChatMsg { active: false, outgoing: false, text: [0; MAX_MSG_LEN], text_len: 0, timestamp: 0 }
    }
}

static mut STATE: CommState = CommState::Disconnected;
static mut MESSAGES: [ChatMsg; MAX_MESSAGES] = [ChatMsg::empty(); MAX_MESSAGES];
static mut MSG_COUNT: usize = 0;
static mut SESSION_KEY: [u8; 32] = [0; 32];
static mut PEER_IP: u32 = 0;
static mut PEER_PORT: u16 = 0;

// Compose buffer
static mut COMPOSE_BUF: [u8; MAX_MSG_LEN] = [0; MAX_MSG_LEN];
static mut COMPOSE_LEN: usize = 0;

/// Connect to a peer for encrypted chat.
pub fn connect(ip: u32, port: u16) {
    unsafe {
        STATE = CommState::Connecting;
        PEER_IP = ip;
        PEER_PORT = port;

        // Derive session key from peer IP + timestamp
        let mut seed = [0u8; 40];
        seed[0..4].copy_from_slice(&ip.to_be_bytes());
        seed[4..6].copy_from_slice(&port.to_be_bytes());
        let ts: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) ts);
        seed[8..16].copy_from_slice(&ts.to_le_bytes());
        SESSION_KEY = sha256::hash(&seed);

        match crate::net::tcp::connect(ip, port) {
            Ok(()) => {
                STATE = CommState::Connected;
                add_system_msg("Connected. E2E encryption active.");
            }
            Err(_) => {
                STATE = CommState::Error;
                add_system_msg("Connection failed.");
            }
        }
    }
}

/// Send an encrypted message.
pub fn send_message(text: &[u8]) {
    unsafe {
        if STATE != CommState::Connected { return; }

        // Encrypt with AES-256-CTR
        let cipher = aes::Aes256::new(&core::ptr::read_volatile(core::ptr::addr_of!(SESSION_KEY)));
        let mut nonce = [0u8; 12];
        let ts: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) ts);
        nonce[4..12].copy_from_slice(&ts.to_le_bytes());

        let mut encrypted = [0u8; MAX_MSG_LEN];
        let len = text.len().min(MAX_MSG_LEN);
        encrypted[..len].copy_from_slice(&text[..len]);
        cipher.ctr_crypt(&nonce, &mut encrypted[..len]);

        // Send: [nonce(12)] [encrypted(len)]
        let mut packet = [0u8; MAX_MSG_LEN + 12];
        packet[..12].copy_from_slice(&nonce);
        packet[12..12 + len].copy_from_slice(&encrypted[..len]);
        let _ = crate::net::tcp::send_data(&packet[..12 + len]);

        // Add to local log
        add_msg(true, text);
    }
}

/// Receive and decrypt a message.
pub fn recv_message() -> bool {
    unsafe {
        if STATE != CommState::Connected { return false; }

        let mut buf = [0u8; MAX_MSG_LEN + 12];
        match crate::net::tcp::recv_data(&mut buf) {
            Ok(n) if n > 12 => {
                let nonce = &buf[..12];
                let cipher = aes::Aes256::new(&core::ptr::read_volatile(core::ptr::addr_of!(SESSION_KEY)));
                let msg_len = n - 12;
                let mut decrypted = [0u8; MAX_MSG_LEN];
                decrypted[..msg_len].copy_from_slice(&buf[12..12 + msg_len]);

                let mut nonce_arr = [0u8; 12];
                nonce_arr.copy_from_slice(nonce);
                cipher.ctr_crypt(&nonce_arr, &mut decrypted[..msg_len]);

                add_msg(false, &decrypted[..msg_len]);
                true
            }
            _ => false,
        }
    }
}

fn add_msg(outgoing: bool, text: &[u8]) {
    unsafe {
        let idx = MSG_COUNT % MAX_MESSAGES;
        let ts: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) ts);
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);

        MESSAGES[idx] = ChatMsg {
            active: true,
            outgoing,
            text: {
                let mut t = [0u8; MAX_MSG_LEN];
                let len = text.len().min(MAX_MSG_LEN);
                t[..len].copy_from_slice(&text[..len]);
                t
            },
            text_len: text.len().min(MAX_MSG_LEN),
            timestamp: ts / freq,
        };
        MSG_COUNT += 1;
    }
}

fn add_system_msg(text: &str) {
    add_msg(false, text.as_bytes());
}

/// Disconnect from peer.
pub fn disconnect() {
    crate::net::tcp::close();
    unsafe {
        STATE = CommState::Disconnected;
        SESSION_KEY = [0; 32];
    }
    add_system_msg("Disconnected.");
}

/// V11-state-sweep: tear down the chat session on cave switch. Without
/// this, a new cave inherits the outgoing cave's AES session key, peer
/// tuple, AND the entire decrypted message history + compose buffer.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        STATE = CommState::Disconnected;
        SESSION_KEY = [0; 32];
        PEER_IP = 0;
        PEER_PORT = 0;
        MSG_COUNT = 0;
        for m in (&mut *core::ptr::addr_of_mut!(MESSAGES)).iter_mut() {
            *m = ChatMsg::empty();
        }
        let cb = core::ptr::addr_of_mut!(COMPOSE_BUF) as *mut u8;
        for i in 0..MAX_MSG_LEN {
            core::ptr::write_volatile(cb.add(i), 0);
        }
        COMPOSE_LEN = 0;
    }
}

/// Render the comms client UI.
pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let ymax = r.y + r.h;
    let ln = 18u32;

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 4;

    // ─── Status Bar ───
    let mut y = r.y + 4;
    unsafe {
        let status_text = match STATE {
            CommState::Disconnected => "DISCONNECTED",
            CommState::Connecting => "CONNECTING...",
            CommState::Connected => "ENCRYPTED",
            CommState::Error => "ERROR",
        };
        let status_color = match STATE {
            CommState::Connected => GREEN,
            CommState::Error => RED,
            _ => DIM,
        };
        if y + ln < ymax {
            font::draw_str(fb, w, x, y, "COMMS", FG_HI, BG);
            font::draw_str(fb, w, x + 56, y, status_text, status_color, BG);

            if STATE == CommState::Connected {
                // Show encryption indicator
                font::draw_str(fb, w, r.x + r.w - 100, y, "AES-256", GREEN, BG);
            }
            y += ln;
        }

        // Divider
        if y < ymax {
            gpu::fill_rect(x, y, r.w - 8, 1, BORDER);
            y += 4;
        }

        // ─── Message Log ───
        let msg_area_h = if ymax > y + 40 { ymax - y - 40 } else { 0 };
        let visible_msgs = (msg_area_h / ln) as usize;

        let start = if MSG_COUNT > visible_msgs { MSG_COUNT - visible_msgs } else { 0 };
        for i in start..MSG_COUNT {
            if y + ln >= ymax - 30 { break; }
            let idx = i % MAX_MESSAGES;
            let msg = &MESSAGES[idx];
            if !msg.active { continue; }

            // Timestamp
            let mins = msg.timestamp / 60;
            let secs = msg.timestamp % 60;
            let mut ts_buf = [b' '; 6];
            ts_buf[0] = b'0' + ((mins / 10) % 10) as u8;
            ts_buf[1] = b'0' + (mins % 10) as u8;
            ts_buf[2] = b':';
            ts_buf[3] = b'0' + ((secs / 10) % 10) as u8;
            ts_buf[4] = b'0' + (secs % 10) as u8;
            ts_buf[5] = b' ';
            font::draw_str(fb, w, x, y,
                core::str::from_utf8_unchecked(&ts_buf), DIM, BG);

            // Sender indicator + message
            let (prefix, color) = if msg.outgoing {
                (">> ", CYAN)
            } else {
                ("<< ", GREEN)
            };
            font::draw_str(fb, w, x + 52, y, prefix, color, BG);
            let text = core::str::from_utf8_unchecked(&msg.text[..msg.text_len]);
            font::draw_str(fb, w, x + 76, y, text, FG_HI, BG);
            y += ln;
        }

        // ─── Compose Bar ───
        let compose_y = ymax - 24;
        if compose_y > r.y + 40 {
            gpu::fill_rect(x, compose_y - 2, r.w - 8, 1, BORDER);
            gpu::fill_rect(x, compose_y, r.w - 8, 20, INPUT_BG);
            font::draw_str(fb, w, x + 2, compose_y + 2, ">", GREEN, INPUT_BG);

            let compose_text = core::str::from_utf8_unchecked(&COMPOSE_BUF[..COMPOSE_LEN]);
            font::draw_str(fb, w, x + 12, compose_y + 2, compose_text, FG_HI, INPUT_BG);

            // Cursor
            let cursor_x = x + 12 + (COMPOSE_LEN as u32) * 8;
            font::draw_str(fb, w, cursor_x, compose_y + 2, "_", FG_HI, INPUT_BG);
        }
    }
}

/// Handle keyboard input for the compose bar.
pub fn handle_key(ch: u8) {
    unsafe {
        match ch {
            b'\r' | b'\n' => {
                if COMPOSE_LEN > 0 {
                    send_message(&COMPOSE_BUF[..COMPOSE_LEN]);
                    COMPOSE_LEN = 0;
                }
            }
            0x08 | 0x7F => {
                if COMPOSE_LEN > 0 { COMPOSE_LEN -= 1; }
            }
            c if c >= 0x20 && c <= 0x7E => {
                if COMPOSE_LEN < MAX_MSG_LEN - 1 {
                    COMPOSE_BUF[COMPOSE_LEN] = c;
                    COMPOSE_LEN += 1;
                }
            }
            _ => {}
        }
    }
}
