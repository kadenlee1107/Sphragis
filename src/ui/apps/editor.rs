#![allow(dead_code)]
// Bat_OS — ED · Code Editor
//
// STUMP #129 (Wave 3) shipped a pure-demo Editor that painted a
// hardcoded sample of kernel_main.rs. STUMP #130 makes it
// actually editable: real text buffer, real cursor, arrow-key
// navigation, character insertion / deletion / Enter, on-the-fly
// Rust syntax tokenization for color.
//
// Still missing (intentionally — separate STUMPs):
//   * Save / load to BatFS (use shell `write` / `read` for now)
//   * Multi-tab buffers (the 3 tabs are visual; only one is real)
//   * Scrolling past the visible region (buffer caps at visible)
//   * Selection / copy / paste

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::widgets::{
    self as W, draw_strip, draw_seg_separator, draw_code_line, Tok,
    BG, INK, MID, DIM_TXT, FAINT, CYAN, AMBER, HAIR, HAIR_HI,
};
use crate::drivers::virtio::keyboard::{
    KEY_ARROW_UP, KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT,
};

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;
const TAB_BAR_H: u32 = 24;
const STATUS_H:  u32 = 28;
const GUTTER_W:  u32 = 56;
const TAB_W:     u32 = 168;
const NEW_TAB_W: u32 = 32;
const GUTTER_BG:   u32 = 0xFF080808;
const LINE_NUM:    u32 = 0xFF3A3A3A;
const CUR_LINE_BG: u32 = 0xFF0E1F22;

// ─── Text buffer (in-memory, single-tab) ────────────────────────────

const MAX_LINES:    usize = 200;
const MAX_LINE_LEN: usize = 256;

struct Buffer {
    lines:      [[u8; MAX_LINE_LEN]; MAX_LINES],
    line_lens:  [u16; MAX_LINES],
    line_count: usize,
    cur_line:   usize,
    cur_col:    usize,
    name:       [u8; 64],
    name_len:   usize,
    dirty:      bool,
}

impl Buffer {
    const fn empty() -> Self {
        Buffer {
            lines: [[0u8; MAX_LINE_LEN]; MAX_LINES],
            line_lens: [0u16; MAX_LINES],
            line_count: 1,
            cur_line: 0,
            cur_col: 0,
            name: [0u8; 64],
            name_len: 0,
            dirty: false,
        }
    }
}

static mut BUF: Buffer = Buffer::empty();

#[inline] fn buf() -> &'static mut Buffer {
    unsafe { &mut *core::ptr::addr_of_mut!(BUF) }
}

/// Drop a line at `line` after column `col` and start a new line below.
fn split_line_at_cursor() {
    let b = buf();
    if b.line_count >= MAX_LINES { return; }
    let line = b.cur_line;
    let col  = b.cur_col;
    let len  = b.line_lens[line] as usize;
    if col > len { return; }
    // Shift all rows below down by one.
    let mut i = b.line_count;
    while i > line + 1 {
        b.lines[i] = b.lines[i - 1];
        b.line_lens[i] = b.line_lens[i - 1];
        i -= 1;
    }
    // New line gets the tail of the current one.
    let tail_len = len - col;
    let mut new_line = [0u8; MAX_LINE_LEN];
    new_line[..tail_len].copy_from_slice(&b.lines[line][col..len]);
    b.lines[line + 1] = new_line;
    b.line_lens[line + 1] = tail_len as u16;
    // Truncate current line.
    b.line_lens[line] = col as u16;
    b.line_count += 1;
    b.cur_line = line + 1;
    b.cur_col = 0;
    b.dirty = true;
}

/// Backspace: if cursor at col 0 and not on the first line, merge
/// current line into previous. Otherwise delete the char to the left.
fn backspace() {
    let b = buf();
    if b.cur_col > 0 {
        let line = b.cur_line;
        let col  = b.cur_col;
        let len  = b.line_lens[line] as usize;
        // Shift bytes left by 1.
        for i in (col - 1)..(len - 1) {
            b.lines[line][i] = b.lines[line][i + 1];
        }
        b.line_lens[line] = (len - 1) as u16;
        b.cur_col = col - 1;
        b.dirty = true;
    } else if b.cur_line > 0 {
        // Merge current line tail into the previous line.
        let prev = b.cur_line - 1;
        let prev_len = b.line_lens[prev] as usize;
        let cur = b.cur_line;
        let cur_len = b.line_lens[cur] as usize;
        let copy = cur_len.min(MAX_LINE_LEN - prev_len);
        for i in 0..copy {
            b.lines[prev][prev_len + i] = b.lines[cur][i];
        }
        b.line_lens[prev] = (prev_len + copy) as u16;
        // Shift all rows below up by one.
        for i in cur..(b.line_count - 1) {
            b.lines[i] = b.lines[i + 1];
            b.line_lens[i] = b.line_lens[i + 1];
        }
        b.line_count -= 1;
        b.cur_line = prev;
        b.cur_col = prev_len;
        b.dirty = true;
    }
}

/// Insert one printable byte at the cursor.
fn insert_char(c: u8) {
    let b = buf();
    let line = b.cur_line;
    let col  = b.cur_col;
    let len  = b.line_lens[line] as usize;
    if len + 1 >= MAX_LINE_LEN { return; }
    // Shift bytes right by 1 from col.
    let mut i = len;
    while i > col {
        b.lines[line][i] = b.lines[line][i - 1];
        i -= 1;
    }
    b.lines[line][col] = c;
    b.line_lens[line] = (len + 1) as u16;
    b.cur_col = col + 1;
    b.dirty = true;
}

fn move_cursor_left() {
    let b = buf();
    if b.cur_col > 0 {
        b.cur_col -= 1;
    } else if b.cur_line > 0 {
        b.cur_line -= 1;
        b.cur_col = b.line_lens[b.cur_line] as usize;
    }
}

fn move_cursor_right() {
    let b = buf();
    let len = b.line_lens[b.cur_line] as usize;
    if b.cur_col < len {
        b.cur_col += 1;
    } else if b.cur_line + 1 < b.line_count {
        b.cur_line += 1;
        b.cur_col = 0;
    }
}

fn move_cursor_up() {
    let b = buf();
    if b.cur_line > 0 {
        b.cur_line -= 1;
        let len = b.line_lens[b.cur_line] as usize;
        if b.cur_col > len { b.cur_col = len; }
    }
}

fn move_cursor_down() {
    let b = buf();
    if b.cur_line + 1 < b.line_count {
        b.cur_line += 1;
        let len = b.line_lens[b.cur_line] as usize;
        if b.cur_col > len { b.cur_col = len; }
    }
}

/// Public entry — desktop::run dispatches keystrokes here when the
/// active app is APP_EDITOR.
pub fn handle_key(c: u8) {
    match c {
        KEY_ARROW_UP    => move_cursor_up(),
        KEY_ARROW_DOWN  => move_cursor_down(),
        KEY_ARROW_LEFT  => move_cursor_left(),
        KEY_ARROW_RIGHT => move_cursor_right(),
        b'\r' | b'\n'   => split_line_at_cursor(),
        0x08 | 0x7F     => backspace(),
        c if c >= 0x20 && c <= 0x7E => insert_char(c),
        _ => {}
    }
}

/// Public — let other code (like a `edit <file>` shell command,
/// future) seed the buffer with content.
pub fn load_text(text: &[u8], name: &str) {
    let b = buf();
    // Reset.
    b.line_count = 1;
    b.cur_line = 0;
    b.cur_col = 0;
    b.dirty = false;
    for r in 0..MAX_LINES {
        b.line_lens[r] = 0;
    }
    // Walk text, splitting on '\n'.
    let mut row = 0usize;
    let mut col = 0usize;
    for &byte in text {
        if byte == b'\n' {
            row += 1;
            col = 0;
            if row >= MAX_LINES { return; }
        } else if col < MAX_LINE_LEN && byte >= 0x20 && byte < 0x7F {
            b.lines[row][col] = byte;
            col += 1;
            b.line_lens[row] = col as u16;
        }
    }
    b.line_count = (row + 1).max(1);
    let nlen = name.len().min(64);
    b.name[..nlen].copy_from_slice(&name.as_bytes()[..nlen]);
    b.name_len = nlen;
}

// ─── Tiny Rust tokenizer ───────────────────────────────────────────

/// Tokenize a single line of Rust source into spans. Returns the
/// number of spans written into `out`. Caller passes a fixed-size
/// buffer of tuples.
fn tokenize_line<'a>(line: &'a [u8], out: &mut [(Tok, &'a [u8])]) -> usize {
    let mut n = 0usize;
    let mut i = 0usize;
    let len = line.len();
    while i < len && n < out.len() {
        let b = line[i];
        // Whitespace runs → punct (so they paint as plain background).
        if b == b' ' || b == b'\t' {
            let start = i;
            while i < len && (line[i] == b' ' || line[i] == b'\t') { i += 1; }
            out[n] = (Tok::Punct, &line[start..i]);
            n += 1;
            continue;
        }
        // Comment to end-of-line (// or //! or ///).
        if b == b'/' && i + 1 < len && line[i + 1] == b'/' {
            out[n] = (Tok::Comment, &line[i..len]);
            n += 1;
            return n;
        }
        // Attribute: '#' followed by '[' or '!'.
        if b == b'#' && i + 1 < len && (line[i + 1] == b'[' || line[i + 1] == b'!') {
            let start = i;
            // Walk until matching ']' or end of line.
            let mut depth = 0i32;
            while i < len {
                if line[i] == b'[' { depth += 1; }
                else if line[i] == b']' { depth -= 1; if depth == 0 { i += 1; break; } }
                i += 1;
            }
            out[n] = (Tok::Attr, &line[start..i]);
            n += 1;
            continue;
        }
        // String literal "..." (no escape support — kernel code rarely
        // contains escaped quotes inside strings).
        if b == b'"' {
            let start = i;
            i += 1;
            while i < len && line[i] != b'"' { i += 1; }
            if i < len { i += 1; }
            out[n] = (Tok::String, &line[start..i]);
            n += 1;
            continue;
        }
        // Identifier / keyword: starts with letter or _, then alnum/_.
        if (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z') || b == b'_' {
            let start = i;
            while i < len {
                let c = line[i];
                let is_alnum = (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
                    || (c >= b'0' && c <= b'9') || c == b'_';
                if !is_alnum { break; }
                i += 1;
            }
            let ident = &line[start..i];
            let kind = if is_keyword(ident) { Tok::Keyword } else { Tok::Ident };
            out[n] = (kind, ident);
            n += 1;
            continue;
        }
        // Anything else: a single-byte punct span.
        out[n] = (Tok::Punct, &line[i..i + 1]);
        n += 1;
        i += 1;
    }
    n
}

fn is_keyword(s: &[u8]) -> bool {
    matches!(s,
        b"as" | b"break" | b"const" | b"continue" | b"crate" | b"else" | b"enum"
        | b"extern" | b"false" | b"fn" | b"for" | b"if" | b"impl" | b"in" | b"let"
        | b"loop" | b"match" | b"mod" | b"move" | b"mut" | b"pub" | b"ref" | b"return"
        | b"self" | b"Self" | b"static" | b"struct" | b"super" | b"trait" | b"true"
        | b"type" | b"unsafe" | b"use" | b"where" | b"while" | b"async" | b"await"
        | b"dyn" | b"box"
    )
}

// ─── Render ─────────────────────────────────────────────────────────

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 100 { return; }

    draw_strip(r.x, r.y, r.w, TAB_BAR_H, false, true);
    draw_tabs(r.x, r.y, r.w);

    let body_y = r.y + TAB_BAR_H;
    let status_y = r.y + r.h - STATUS_H;
    let body_h = status_y.saturating_sub(body_y);
    draw_gutter_and_code(r.x, body_y, r.w, body_h);

    draw_strip(r.x, status_y, r.w, STATUS_H, true, false);
    draw_status_strip(r.x, status_y, r.w);
}

fn buffer_name() -> &'static str {
    let b = buf();
    if b.name_len == 0 { "untitled.rs" }
    else { unsafe { core::str::from_utf8_unchecked(&b.name[..b.name_len]) } }
}

fn draw_tabs(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();

    let primary_name = buffer_name();
    let primary_dirty = buf().dirty;
    let tabs: [(&str, bool, bool); 3] = [
        (primary_name,    true,  primary_dirty),
        ("lib.rs",        false, false),
        ("Cargo.toml",    false, false),
    ];
    let mut tx = x;
    for (name, active, dirty) in tabs.iter() {
        gpu::fill_rect(tx + TAB_W, y, 1, TAB_BAR_H, HAIR);
        let text_y = y + (TAB_BAR_H - CHAR_H) / 2;
        let name_x = tx + 12;
        let name_color = if *active { INK } else { DIM_TXT };
        font::draw_str(fb, sw, name_x, text_y, name, name_color, BG);
        let mut after_name = name_x + name.len() as u32 * CHAR_W;
        if *dirty {
            font::draw_str(fb, sw, after_name + CHAR_W / 2, text_y, ".", AMBER, BG);
            after_name += CHAR_W;
        }
        let close_x = tx + TAB_W - 16;
        let punct_color = FAINT;
        let x_color = if *active { CYAN } else { FAINT };
        font::draw_str(fb, sw, close_x, text_y, ":", punct_color, BG);
        font::draw_str(fb, sw, close_x + CHAR_W, text_y, "x", x_color, BG);
        let _ = after_name;
        if *active {
            gpu::fill_rect(tx, y + TAB_BAR_H - 2, TAB_W, 2, CYAN);
        }
        tx += TAB_W;
    }
    let plus_x = x + w - NEW_TAB_W;
    gpu::fill_rect(plus_x, y, 1, TAB_BAR_H, HAIR);
    font::draw_str(fb, sw, plus_x + (NEW_TAB_W - CHAR_W) / 2,
        y + (TAB_BAR_H - CHAR_H) / 2, "+", DIM_TXT, BG);
}

fn draw_gutter_and_code(x: u32, y: u32, w: u32, h: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(x, y, GUTTER_W, h, GUTTER_BG);
    gpu::fill_rect(x + GUTTER_W, y, 1, h, HAIR_HI);

    let pad_top: u32 = 8;
    let line_x = x + GUTTER_W + 16;
    let visible_lines = ((h.saturating_sub(pad_top)) / CHAR_H) as usize;
    let b = buf();
    let n_lines = b.line_count.min(visible_lines);

    let mut span_buf: [(Tok, &[u8]); 64] = [(Tok::Punct, &[]); 64];

    for i in 0..n_lines {
        let ly = y + pad_top + (i as u32) * CHAR_H;
        let is_cur = i == b.cur_line;

        if is_cur {
            gpu::fill_rect(x + GUTTER_W + 1, ly, w - GUTTER_W - 1, CHAR_H, CUR_LINE_BG);
            gpu::fill_rect(x + GUTTER_W, ly, 1, CHAR_H, CYAN);
        }

        // Line number.
        let mut buf_n = [0u8; 8];
        let n = format_dec(i + 1, &mut buf_n);
        let ln_str = unsafe { core::str::from_utf8_unchecked(&buf_n[..n]) };
        let ln_w = n as u32 * CHAR_W;
        let ln_x = x + GUTTER_W - 12 - ln_w;
        let ln_color = if is_cur { INK } else { LINE_NUM };
        font::draw_str(fb, sw, ln_x, ly, ln_str, ln_color,
            if is_cur { CUR_LINE_BG } else { GUTTER_BG });

        // Tokenize and paint the line.
        let line_len = b.line_lens[i] as usize;
        if line_len > 0 {
            let line = &b.lines[i][..line_len];
            let n_spans = tokenize_line(line, &mut span_buf);
            // Convert byte slices to &str for draw_code_line.
            let mut converted: [(Tok, &str); 64] = [(Tok::Punct, ""); 64];
            for j in 0..n_spans {
                let (t, slice) = span_buf[j];
                converted[j] = (t, unsafe { core::str::from_utf8_unchecked(slice) });
            }
            draw_code_line(line_x, ly, &converted[..n_spans]);
        }

        // Cursor block.
        if is_cur {
            let cur_x = line_x + (b.cur_col as u32) * CHAR_W;
            gpu::fill_rect(cur_x, ly + 1, 8, 14, CYAN);
        }
    }
}

fn draw_status_strip(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (STATUS_H - CHAR_H) / 2;
    let b = buf();

    let mut cx = x + 16;
    font::draw_str(fb, sw, cx, text_y, "LANG", FAINT, BG); cx += 5 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "RUST", INK, BG);   cx += 5 * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;
    font::draw_str(fb, sw, cx, text_y, "ENC", FAINT, BG);  cx += 4 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "UTF-8", INK, BG);  cx += 6 * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;

    // POS (live cursor).
    font::draw_str(fb, sw, cx, text_y, "POS", FAINT, BG);  cx += 4 * CHAR_W;
    let mut pos_buf = [0u8; 24];
    let mut p = 0usize;
    pos_buf[p] = b'L'; p += 1;
    pos_buf[p] = b'n'; p += 1;
    pos_buf[p] = b' '; p += 1;
    p += format_dec(b.cur_line + 1, &mut pos_buf[p..]);
    pos_buf[p] = b','; p += 1;
    pos_buf[p] = b' '; p += 1;
    pos_buf[p] = b'C'; p += 1;
    pos_buf[p] = b'o'; p += 1;
    pos_buf[p] = b'l'; p += 1;
    pos_buf[p] = b' '; p += 1;
    p += format_dec(b.cur_col + 1, &mut pos_buf[p..]);
    let pos_s = unsafe { core::str::from_utf8_unchecked(&pos_buf[..p]) };
    font::draw_str(fb, sw, cx, text_y, pos_s, INK, BG);
    cx += (p as u32 + 1) * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;

    font::draw_str(fb, sw, cx, text_y, "LF", FAINT, BG); cx += 3 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "UNIX", INK, BG);

    // Right: dirty marker + EDIT badge (no longer "READ ONLY").
    let badge = if b.dirty { "MODIFIED" } else { "READY" };
    let badge_color = if b.dirty { AMBER } else { CYAN };
    let badge_w = badge.len() as u32 * CHAR_W;
    if w > badge_w + 16 {
        font::draw_str(fb, sw, x + w - 16 - badge_w, text_y, badge, badge_color, BG);
    }

    let _ = (W::draw_kv_row, MID);
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
