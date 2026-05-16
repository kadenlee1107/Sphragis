//! Wave 8 AGENT — calm Wave-4 register Q&A panel.
//! See `docs/superpowers/specs/2026-05-15-agent-app-design.md`.

#![allow(dead_code)]

extern crate alloc;

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::draw_strip;
use crate::ui::wm::WindowRect;
use crate::ui::{font, gpu};
use crate::ai::{AgentSession, AgentError, StreamEvent};

const MAX_TURNS:    usize = 32;
const MAX_QUESTION: usize = 256;
const MAX_RESPONSE: usize = 1024;
const HEADER_H:     u32   = 32;
const COMPOSER_H:   u32   = 28;
const ROW_H:        u32   = 18;
const CHAR_W:       u32   = 8;
// [timestamp + you] / question / spacer / [timestamp + agent] / response
const ROWS_PER_TURN: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppState {
    Idle,
    Streaming,
    Error,
}

#[derive(Copy, Clone)]
struct Turn {
    active: bool,
    timestamp: u64,
    question: [u8; MAX_QUESTION],
    question_len: u16,
    response: [u8; MAX_RESPONSE],
    response_len: u16,
    is_stub: bool,
}

impl Turn {
    const fn empty() -> Self {
        Self {
            active: false,
            timestamp: 0,
            question: [0u8; MAX_QUESTION],
            question_len: 0,
            response: [0u8; MAX_RESPONSE],
            response_len: 0,
            is_stub: false,
        }
    }
}

static mut TURNS: [Turn; MAX_TURNS] = [Turn::empty(); MAX_TURNS];
static mut TURN_COUNT: usize = 0;
static mut COMPOSE_BUF: [u8; MAX_QUESTION] = [0u8; MAX_QUESTION];
static mut COMPOSE_LEN: usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut APP_STATE: AppState = AppState::Idle;
static mut SESSION: Option<AgentSession> = None;
static mut LAST_ERROR: [u8; 64] = [0u8; 64];
static mut LAST_ERROR_LEN: usize = 0;
static mut SESSION_ID: u64 = 0;

// ── Public entry points ──────────────────────────────────────────

pub fn paint(body: WindowRect) {
    gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    paint_header(WindowRect { x: body.x, y: body.y, w: body.w, h: HEADER_H });
    gpu::fill_rect(body.x, body.y + HEADER_H, body.w, 1, p::HAIRLINE);

    let composer_y = body.y + body.h - COMPOSER_H;
    gpu::fill_rect(body.x, composer_y - 1, body.w, 1, p::HAIRLINE);

    let hist_y = body.y + HEADER_H + 1;
    let hist_h = composer_y.saturating_sub(hist_y + 1);
    paint_history(WindowRect { x: body.x, y: hist_y, w: body.w, h: hist_h });

    paint_composer(WindowRect { x: body.x, y: composer_y, w: body.w, h: COMPOSER_H });
}

pub fn handle_key(c: u8) -> AppEvent {
    let state = unsafe { *core::ptr::addr_of!(APP_STATE) };
    match c {
        0x1B | 0x03 => {
            // Esc / Ctrl+C — if Streaming: signal interrupt (composer retained).
            // Otherwise: clear composer (and dismiss any error).
            if state == AppState::Streaming {
                unsafe {
                    if let Some(s) = &mut *core::ptr::addr_of_mut!(SESSION) {
                        s.interrupt();
                    }
                }
            } else {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), 0); }
                if state == AppState::Error {
                    unsafe {
                        *core::ptr::addr_of_mut!(APP_STATE) = AppState::Idle;
                        core::ptr::write_volatile(core::ptr::addr_of_mut!(LAST_ERROR_LEN), 0);
                    }
                }
            }
            AppEvent::Repaint
        }
        0x90 => { // Up — scroll viewport up
            let vp = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) };
            if vp > 0 {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), vp - 1); }
            }
            AppEvent::Repaint
        }
        0x91 => { // Down — scroll viewport down
            let vp = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) };
            let total = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TURN_COUNT)) };
            if vp + 1 < total {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), vp + 1); }
            }
            AppEvent::Repaint
        }
        0x92 | 0x93 => AppEvent::Repaint,  // Left/Right ignored
        0x08 | 0x7F => {
            // Backspace
            if state == AppState::Idle || state == AppState::Error {
                let n = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN)) };
                if n > 0 {
                    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), n - 1); }
                }
            }
            AppEvent::Repaint
        }
        b'\r' | b'\n' => {
            // Enter — send if not Streaming. Idle or Error both queue a new
            // turn (Error → retry semantics; the error label clears on send).
            if state != AppState::Streaming {
                send_question();
            }
            AppEvent::Repaint
        }
        0x20..=0x7E => {
            if state == AppState::Streaming {
                return AppEvent::Consumed;
            }
            // Printable ASCII — append to composer if room.
            let n = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN)) };
            if n < MAX_QUESTION - 1 {
                unsafe {
                    (*core::ptr::addr_of_mut!(COMPOSE_BUF))[n] = c;
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), n + 1);
                }
            }
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

pub fn handle_click(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Consumed
}

// ── Render helpers ───────────────────────────────────────────────

fn paint_header(rect: WindowRect) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let _ = draw_strip(rect.x, rect.y, rect.w, rect.h, false, true);
    let h_text_y = rect.y + (rect.h - 16) / 2;

    font::draw_str(fb, sw, rect.x + 16, h_text_y, "AGENT", p::INK, p::BG);

    let state = unsafe { *core::ptr::addr_of!(APP_STATE) };
    let state_str = match state {
        AppState::Idle      => "READY",
        AppState::Streaming => "THINKING",
        AppState::Error     => "ERROR",
    };
    let state_x = rect.x + 16 + 6 * CHAR_W;
    font::draw_str(fb, sw, state_x, h_text_y, state_str, p::INK, p::BG);

    // Error reason (if any) trailing the state.
    let err_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(LAST_ERROR_LEN)) };
    if state == AppState::Error && err_len > 0 {
        let last_err = unsafe { &*core::ptr::addr_of!(LAST_ERROR) };
        let err_bytes = &last_err[..err_len];
        let err_str = unsafe { core::str::from_utf8_unchecked(err_bytes) };
        let err_x = state_x + (state_str.len() as u32) * CHAR_W + 2 * CHAR_W;
        font::draw_str(fb, sw, err_x, h_text_y, ": ", p::MID, p::BG);
        font::draw_str(fb, sw, err_x + 2 * CHAR_W, h_text_y, err_str, p::MID, p::BG);
    }

    // Right side: session id + optional "stub" tag.
    let session_id = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SESSION_ID)) };
    let mut buf = [0u8; 64];
    let mut n = 0;
    push_bytes(&mut buf, &mut n, b"session ");
    write_dec(&mut buf, &mut n, session_id as u32);
    // SESSION_TOKENS rendering removed until Phase-5 wires real counts
    // through StreamingResponse metadata. Re-add as " \xc2\xb7 N tokens"
    // alongside the stub tag when that lands.

    let count = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TURN_COUNT)) };
    let last_stub = if count > 0 {
        let idx = (count - 1) % MAX_TURNS;
        let turns = unsafe { &*core::ptr::addr_of!(TURNS) };
        turns[idx].is_stub
    } else {
        false
    };
    if last_stub {
        push_bytes(&mut buf, &mut n, b" \xc2\xb7 stub");
    }

    let right = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
    let right_w = (n as u32) * CHAR_W;
    if rect.w > right_w + 16 {
        font::draw_str(fb, sw,
            rect.x + rect.w.saturating_sub(right_w + 16),
            h_text_y, right, p::MID, p::BG);
    }
}

fn paint_history(rect: WindowRect) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let total = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TURN_COUNT)) };

    if total == 0 {
        let msg = "Type a question and Enter to ask the agent.";
        font::draw_str(fb, sw, rect.x + 16, rect.y + 16, msg, p::FAINT, p::BG);
        return;
    }

    let viewport = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) };
    let visible_rows = (rect.h / ROW_H) as usize;
    let visible_turns = visible_rows / ROWS_PER_TURN;

    let start = viewport.min(total);
    let count = (total - start).min(visible_turns);
    let anchored_y = (rect.y + rect.h).saturating_sub(4 + (count as u32) * (ROWS_PER_TURN as u32) * ROW_H);
    let mut row_y = anchored_y;

    for i in start..start + count {
        let idx = i % MAX_TURNS;
        let turns = unsafe { &*core::ptr::addr_of!(TURNS) };
        let turn = turns[idx];
        if !turn.active { continue; }

        let mut ts_buf = [b' '; 10];
        ts_buf[0] = b'[';
        let h = (turn.timestamp / 3600) % 24;
        let m = (turn.timestamp / 60) % 60;
        let s = turn.timestamp % 60;
        ts_buf[1] = b'0' + ((h / 10) % 10) as u8;
        ts_buf[2] = b'0' + (h % 10) as u8;
        ts_buf[3] = b':';
        ts_buf[4] = b'0' + ((m / 10) % 10) as u8;
        ts_buf[5] = b'0' + (m % 10) as u8;
        ts_buf[6] = b':';
        ts_buf[7] = b'0' + ((s / 10) % 10) as u8;
        ts_buf[8] = b'0' + (s % 10) as u8;
        ts_buf[9] = b']';
        let ts_str = unsafe { core::str::from_utf8_unchecked(&ts_buf) };

        font::draw_str(fb, sw, rect.x + 16, row_y, ts_str, p::MID, p::BG);
        font::draw_str(fb, sw, rect.x + 16 + 11 * CHAR_W, row_y, "you", p::INK, p::BG);
        row_y += ROW_H;

        let q_len = (turn.question_len as usize).min(turn.question.len());
        let q_str = unsafe { core::str::from_utf8_unchecked(&turn.question[..q_len]) };
        font::draw_str(fb, sw, rect.x + 16, row_y, q_str, p::INK, p::BG);
        row_y += ROW_H;

        row_y += ROW_H;  // spacer between you-block and agent-block

        font::draw_str(fb, sw, rect.x + 16, row_y, ts_str, p::MID, p::BG);
        font::draw_str(fb, sw, rect.x + 16 + 11 * CHAR_W, row_y, "agent", p::INK, p::BG);
        row_y += ROW_H;

        let r_len = (turn.response_len as usize).min(turn.response.len());
        if turn.is_stub {
            let stub_msg = "(stub mode -- wire src/ai/client.rs for live inference)";
            font::draw_str(fb, sw, rect.x + 16, row_y, stub_msg, p::FAINT, p::BG);
        } else if r_len > 0 {
            let r_str = unsafe { core::str::from_utf8_unchecked(&turn.response[..r_len]) };
            font::draw_str(fb, sw, rect.x + 16, row_y, r_str, p::INK, p::BG);
        }
        row_y += ROW_H;
    }
}

fn paint_composer(rect: WindowRect) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, p::PANEL);

    let state = unsafe { *core::ptr::addr_of!(APP_STATE) };
    let c_text_y = rect.y + (rect.h - 16) / 2;
    let disabled = state == AppState::Streaming;

    let prompt_color = if disabled { p::FAINT } else { p::INK };
    font::draw_str(fb, sw, rect.x + 16, c_text_y, ">", prompt_color, p::PANEL);

    let typed_x = rect.x + 16 + 2 * CHAR_W;
    let compose_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN)) };

    // Counter string built first so we can reserve its width before
    // laying out the typed-text scroll window.
    let mut cbuf = [0u8; 16];
    let mut cn = 0;
    write_dec(&mut cbuf, &mut cn, compose_len as u32);
    push_bytes(&mut cbuf, &mut cn, b" / 256");
    let counter = unsafe { core::str::from_utf8_unchecked(&cbuf[..cn]) };
    let counter_w = (cn as u32) * CHAR_W;

    if disabled {
        font::draw_str(fb, sw, typed_x, c_text_y,
            "(querying -- Esc to interrupt)", p::FAINT, p::PANEL);
    } else {
        // Compute how many chars fit between the prompt and the counter.
        // Reserve one CHAR_W gap before the counter and one CHAR_W for
        // the cursor cell itself (so the block never overlaps the
        // counter on the right edge).
        let text_end_x = (rect.x + rect.w).saturating_sub(16 + counter_w + CHAR_W);
        let visible_px = text_end_x.saturating_sub(typed_x);
        let max_visible_chars = (visible_px / CHAR_W) as usize;
        let max_visible_chars = max_visible_chars.saturating_sub(1); // cursor cell reserve

        let buf = unsafe { &*core::ptr::addr_of!(COMPOSE_BUF) };
        // Scroll so the cursor stays at the right edge once we overflow.
        let start = compose_len.saturating_sub(max_visible_chars);
        let visible_str = unsafe { core::str::from_utf8_unchecked(&buf[start..compose_len]) };
        let visible_len = (compose_len - start) as u32;

        font::draw_str(fb, sw, typed_x, c_text_y, visible_str, p::INK, p::PANEL);

        let cur_x = typed_x + visible_len * CHAR_W;
        let cell_top = rect.y + (rect.h - 16) / 2;
        gpu::fill_rect(cur_x, cell_top, CHAR_W, 16, p::INK);
    }

    if rect.w > counter_w + 16 {
        let cx = rect.x + rect.w - 16 - counter_w;
        let color = if disabled { p::FAINT } else { p::MID };
        font::draw_str(fb, sw, cx, c_text_y, counter, color, p::PANEL);
    }
}

// ── Q&A dispatch ─────────────────────────────────────────────────

fn send_question() {
    let compose_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN)) };
    if compose_len == 0 { return; }

    // New turn starts — clear any stale error from the previous turn.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LAST_ERROR_LEN), 0);
    }

    let session_ready = unsafe { (*core::ptr::addr_of!(SESSION)).is_some() };
    if !session_ready {
        match AgentSession::new() {
            Ok(s) => unsafe {
                *core::ptr::addr_of_mut!(SESSION) = Some(s);
                let id = core::ptr::read_volatile(core::ptr::addr_of!(SESSION_ID));
                core::ptr::write_volatile(core::ptr::addr_of_mut!(SESSION_ID), id + 1);
            },
            Err(e) => {
                store_error(error_label(&e));
                return;
            }
        }
    }

    let buf = unsafe { &*core::ptr::addr_of!(COMPOSE_BUF) };
    let q_bytes = &buf[..compose_len];
    let turn_idx = unsafe {
        let count = core::ptr::read_volatile(core::ptr::addr_of!(TURN_COUNT));
        let idx = count % MAX_TURNS;
        let turns = &mut *core::ptr::addr_of_mut!(TURNS);
        turns[idx] = Turn::empty();
        turns[idx].active = true;
        turns[idx].timestamp = crate::kernel::time::monotonic_secs();
        let n = q_bytes.len();
        turns[idx].question[..n].copy_from_slice(&q_bytes[..n]);
        turns[idx].question_len = n as u16;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(TURN_COUNT), count + 1);
        idx
    };

    unsafe { *core::ptr::addr_of_mut!(APP_STATE) = AppState::Streaming; }
    let q_str = unsafe { core::str::from_utf8_unchecked(q_bytes) };

    let mut text_seen = false;
    let result_state;
    {
        let session_ref = unsafe { (*core::ptr::addr_of_mut!(SESSION)).as_mut().unwrap() };
        let mut stream = session_ref.ask(q_str);
        loop {
            match stream.poll() {
                StreamEvent::Text(s) => {
                    text_seen = true;
                    let s_bytes = s.as_bytes();
                    let turns = unsafe { &mut *core::ptr::addr_of_mut!(TURNS) };
                    let cur_len = turns[turn_idx].response_len as usize;
                    let cap = MAX_RESPONSE - cur_len;
                    let mut take = s_bytes.len().min(cap);
                    // Walk back to the nearest char boundary so we never split a codepoint.
                    while take > 0 && !s.is_char_boundary(take) {
                        take -= 1;
                    }
                    turns[turn_idx].response[cur_len..cur_len + take]
                        .copy_from_slice(&s_bytes[..take]);
                    turns[turn_idx].response_len = (cur_len + take) as u16;
                }
                StreamEvent::ToolCall { .. } => { /* Wave 8: no UI surface */ }
                StreamEvent::Done => {
                    result_state = AppState::Idle;
                    break;
                }
                StreamEvent::Error(e) => {
                    if matches!(e, AgentError::Interrupted) {
                        // User interrupt — return to READY, retain partial response.
                        result_state = AppState::Idle;
                    } else {
                        store_error(error_label(&e));
                        result_state = AppState::Error;
                    }
                    break;
                }
            }
        }
    }

    if !text_seen && result_state == AppState::Idle {
        unsafe {
            let turns = &mut *core::ptr::addr_of_mut!(TURNS);
            turns[turn_idx].is_stub = true;
        }
    }

    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), 0);
        *core::ptr::addr_of_mut!(APP_STATE) = result_state;
    }
}

fn error_label(e: &AgentError) -> &'static [u8] {
    match e {
        AgentError::Interrupted   => b"interrupted",
        AgentError::Network(_)    => b"network",
        AgentError::Protocol(_)   => b"protocol",
        AgentError::Tool(_)       => b"tool",
        AgentError::TokenBudget   => b"token budget",
        AgentError::PolicyDenied  => b"policy denied",
    }
}

/// Stores an error label into the LAST_ERROR static.
///
/// # Safety contract
///
/// `bytes` must be valid UTF-8 — `paint_header` reads via
/// `from_utf8_unchecked`. ASCII suffices; all callers today
/// pass ASCII-only labels from `error_label`.
fn store_error(bytes: &[u8]) {
    let n = bytes.len().min(64);
    unsafe {
        let dst = core::ptr::addr_of_mut!(LAST_ERROR) as *mut u8;
        for i in 0..n { core::ptr::write(dst.add(i), bytes[i]); }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LAST_ERROR_LEN), n);
    }
}

// ── Cave-switch reset ────────────────────────────────────────────

pub fn reset_for_cave_switch() {
    unsafe {
        *core::ptr::addr_of_mut!(TURNS) = [Turn::empty(); MAX_TURNS];
        core::ptr::write_volatile(core::ptr::addr_of_mut!(TURN_COUNT), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), 0);
        *core::ptr::addr_of_mut!(APP_STATE) = AppState::Idle;
        *core::ptr::addr_of_mut!(SESSION) = None;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(LAST_ERROR_LEN), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SESSION_ID), 0);
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return; }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
}
