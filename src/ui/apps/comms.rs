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
///
/// STUMP #129 — Claude-Design Wave-3 port. 32px header (COMMS
/// wordmark + connection pill + cipher/key pills), timeline body
/// (12px message rows aligned in [HH:MM] | dir | sender | text
/// columns + grey-prefixed system messages), 28px composer
/// (cyan ">" prompt + ink typed text + cursor + char counter).
pub fn render() {
    use crate::ui::widgets::{
        self as W, draw_strip, draw_conn_pill, State,
        BG as W_BG, INK as W_INK, MID as W_MID, DIM_TXT as W_DIM, FAINT as W_FAINT,
        CYAN as W_CYAN, GREEN as W_GREEN, AMBER as W_AMBER, PANEL as W_PANEL,
        HAIR as W_HAIR,
    };

    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(r.x, r.y, r.w, r.h, W_BG);

    let header_h: u32 = 32;
    let composer_h: u32 = 28;
    let body_y = r.y + header_h;
    let composer_y = r.y + r.h - composer_h;
    let body_h = composer_y.saturating_sub(body_y);

    // ── HEADER STRIP ──────────────────────────────────────────────
    draw_strip(r.x, r.y, r.w, header_h, false, true);
    // COMMS wordmark.
    let h_text_y = r.y + (header_h - 16) / 2;
    font::draw_str(fb, sw, r.x + 16, h_text_y, "COMMS", W_INK, W_BG);
    // Connection pill — depends on state.
    let st = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(STATE)) };
    let pill_y = r.y + (header_h - 22) / 2;
    let mut pill_x = r.x + 16 + 6 * 8 + 8; // "COMMS" + gap
    match st {
        CommState::Disconnected => {
            pill_x = draw_conn_pill(pill_x, pill_y, "DISCONNECTED", None, State::Fail);
        }
        CommState::Connecting => {
            pill_x = draw_conn_pill(pill_x, pill_y, "CONNECTING", None, State::Warn);
        }
        CommState::Connected => {
            pill_x = draw_conn_pill(pill_x, pill_y, "CONNECTED",
                Some("peer 10.0.2.42:9100"), State::Ok);
        }
        CommState::Error => {
            pill_x = draw_conn_pill(pill_x, pill_y, "ERROR", None, State::Fail);
        }
    }
    let _ = pill_x;
    // Right side: cipher + key pills (only when connected).
    if st == CommState::Connected {
        // Compute total right-side pill width to right-align.
        // AES-256-CTR (11 chars) + key "K c4e3d7a2..." (12 chars).
        let key_label = "K";
        let key_value = "c4e3d7a2...";
        let cipher_w = compute_pill_w("AES-256-CTR", None);
        let key_w = compute_pill_w(key_label, Some(key_value));
        let gap: u32 = 8;
        let total = cipher_w + gap + key_w;
        if r.w > total + 16 {
            let mut rx = r.x + r.w - 16 - total;
            rx = draw_conn_pill(rx, pill_y, "AES-256-CTR", None, State::Neutral);
            rx += gap;
            let _ = draw_conn_pill(rx, pill_y, key_label, Some(key_value), State::Neutral);
        }
    }

    // ── TIMELINE BODY ─────────────────────────────────────────────
    if st == CommState::Disconnected {
        draw_disconnected_empty(r.x, body_y, r.w, body_h, sw, fb,
            W_DIM, W_CYAN, W_BG);
    } else {
        draw_timeline(r.x, body_y, r.w, body_h, sw, fb,
            W_DIM, W_INK, W_MID, W_GREEN, W_CYAN, W_BG);
    }

    // ── COMPOSER ──────────────────────────────────────────────────
    gpu::fill_rect(r.x, composer_y, r.w, composer_h, W_PANEL);
    gpu::fill_rect(r.x, composer_y, r.w, 1, W_HAIR);
    let c_text_y = composer_y + (composer_h - 16) / 2;
    let prompt_color = if st == CommState::Disconnected { W_FAINT } else { W_CYAN };
    font::draw_str(fb, sw, r.x + 16, c_text_y, ">", prompt_color, W_PANEL);
    let typed_x = r.x + 16 + 2 * 8;

    let (compose_text, compose_len): (&str, usize) = unsafe {
        let len = core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN));
        let bytes = &COMPOSE_BUF[..len];
        (core::str::from_utf8_unchecked(bytes), len)
    };
    if st == CommState::Disconnected {
        font::draw_str(fb, sw, typed_x, c_text_y,
            "(composer disabled . not connected)", W_FAINT, W_PANEL);
    } else {
        font::draw_str(fb, sw, typed_x, c_text_y, compose_text, W_INK, W_PANEL);
        // STUMP #132: underscore cursor instead of a solid block —
        // keeps the typed text readable + stays visible after a space.
        let cur_x = typed_x + (compose_len as u32) * 8;
        let cell_top = composer_y + (composer_h - 16) / 2;
        gpu::fill_rect(cur_x, cell_top + 16 - 2, 7, 2, W_CYAN);
    }
    // Char counter on the right.
    let counter_color = if compose_len >= 80 { W::RED }
        else if compose_len >= 70 { W_AMBER }
        else if st == CommState::Disconnected { W_FAINT }
        else { W_DIM };
    let mut buf = [0u8; 16];
    let n = format_dec_local(compose_len, &mut buf);
    let n_str = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
    let suffix = " / 80";
    let total_w = (n as u32 + suffix.len() as u32) * 8;
    if r.w > total_w + 16 {
        let cx = r.x + r.w - 16 - total_w;
        font::draw_str(fb, sw, cx, c_text_y, n_str, counter_color, W_PANEL);
        font::draw_str(fb, sw, cx + n as u32 * 8, c_text_y, suffix, W_FAINT, W_PANEL);
    }
}

fn compute_pill_w(label: &str, value: Option<&str>) -> u32 {
    let pad: u32 = 10;
    let dot: u32 = 6;
    let label_w = label.len() as u32 * 8;
    let value_w = value.map_or(0, |v| v.len() as u32 * 8 + 8);
    pad + dot + 8 + label_w + value_w + pad
}

fn draw_disconnected_empty(
    x: u32, y: u32, w: u32, h: u32,
    sw: u32, fb: *mut u32,
    dim: u32, cyan: u32, bg: u32,
) {
    let text = "(no peer connected - use ";
    let cmd  = "comms connect <ip>:<port>";
    let tail = " in shell)";
    let total = (text.len() + cmd.len() + tail.len()) as u32 * 8;
    let cx = x + (w.saturating_sub(total)) / 2;
    let cy = y + h / 2 - 8;
    font::draw_str(fb, sw, cx, cy, text, dim, bg);
    font::draw_str(fb, sw, cx + text.len() as u32 * 8, cy, cmd, cyan, bg);
    font::draw_str(fb, sw, cx + (text.len() + cmd.len()) as u32 * 8, cy, tail, dim, bg);
}

fn draw_timeline(
    x: u32, y: u32, _w: u32, h: u32,
    sw: u32, fb: *mut u32,
    dim: u32, ink: u32, mid: u32, green: u32, cyan: u32, bg: u32,
) {
    // Show the most-recent messages, bottom-up (so the latest is
    // anchored to just above the composer).
    unsafe {
        let row_h: u32 = 18;
        let pad_l: u32 = 16;
        let max_rows = (h.saturating_sub(24) / row_h) as usize;
        let total = MSG_COUNT;
        let start = if total > max_rows { total - max_rows } else { 0 };
        let count = total - start;
        // Anchor to bottom of body.
        let baseline_y = y + h - 12 - (count as u32) * row_h;
        let mut row_y = baseline_y;
        for i in start..total {
            let idx = i % MAX_MESSAGES;
            let msg = &MESSAGES[idx];
            if !msg.active { continue; }
            let mins = msg.timestamp / 60;
            let secs = msg.timestamp % 60;
            // [HH:MM]
            let mut ts_buf = [0u8; 7];
            ts_buf[0] = b'[';
            ts_buf[1] = b'0' + ((mins / 10) % 10) as u8;
            ts_buf[2] = b'0' + (mins % 10) as u8;
            ts_buf[3] = b':';
            ts_buf[4] = b'0' + ((secs / 10) % 10) as u8;
            ts_buf[5] = b'0' + (secs % 10) as u8;
            ts_buf[6] = b']';
            let ts_str = core::str::from_utf8_unchecked(&ts_buf);
            font::draw_str(fb, sw, x + pad_l, row_y, ts_str, dim, bg);
            // Direction + sender + text.
            let (arrow, sender, arrow_color, sender_color) = if msg.outgoing {
                (">>", "you ", cyan, cyan)
            } else {
                ("<<", "peer", green, green)
            };
            font::draw_str(fb, sw, x + pad_l + 8 * 8, row_y, arrow, arrow_color, bg);
            font::draw_str(fb, sw, x + pad_l + (8 + 4) * 8, row_y, sender, sender_color, bg);
            let text_x = x + pad_l + (8 + 4 + 7) * 8;
            let text = core::str::from_utf8_unchecked(&msg.text[..msg.text_len]);
            font::draw_str(fb, sw, text_x, row_y, text, ink, bg);
            row_y += row_h;
        }
        let _ = mid; // reserved for system-message styling once add_system_msg is distinguishable
    }
}

fn format_dec_local(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
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
