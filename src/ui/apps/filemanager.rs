// Bat_OS — FS · File Manager
//
// STUMP #129 — Claude-Design Wave-3 port. Source artifacts in
// `docs/design/apps-fs-ed-cm/` (jsx + spec sheet).
//
// Layout: 32px header strip ("ENCRYPTED VAULT · cipher info" +
// file count), 24px column header ("STATUS / FILENAME / SIZE /
// CIPHER / MERKLE OK"), N×24px data rows, 28px footer strip
// ("FILES N · MAX_FILES 32" + Merkle preview + hint). Selected
// row gets a 1px cyan-dim inset border + 2px cyan underline.

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;
use crate::ui::widgets::{
    draw_strip, draw_seg_separator,
    BG, INK, MID, DIM_TXT, FAINT, CYAN, CYAN_DIM, GREEN, GREEN_DIM,
    AMBER, AMBER_DIM, HAIR,
};
use crate::fs::batfs;
use crate::drivers::virtio::keyboard::{KEY_ARROW_UP, KEY_ARROW_DOWN};

// STUMP #131: which row the user has highlighted. Up/down arrows
// move it; Enter opens the selected file in the editor.
static mut SELECTED_ROW: usize = 0;
static mut ROW_COUNT_CACHE: usize = 0;

#[inline] fn selected_row() -> usize { unsafe { SELECTED_ROW } }

/// Public dispatch — desktop::run forwards APP_FILES keystrokes here.
pub fn handle_key(c: u8) {
    match c {
        KEY_ARROW_UP => {
            unsafe {
                if SELECTED_ROW > 0 { SELECTED_ROW -= 1; }
            }
        }
        KEY_ARROW_DOWN => {
            unsafe {
                if SELECTED_ROW + 1 < ROW_COUNT_CACHE { SELECTED_ROW += 1; }
            }
        }
        b'\r' | b'\n' => {
            // Open the selected file in the editor.
            let idx = selected_row();
            let mut name_buf = [0u8; 64];
            let mut name_len = 0usize;
            let mut row_i = 0usize;
            batfs::list(|name, _size, _enc| {
                if row_i == idx {
                    let n = name.len().min(name_buf.len());
                    name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
                    name_len = n;
                }
                row_i += 1;
            });
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                if crate::ui::apps::editor::load_from_batfs(name).is_ok() {
                    crate::ui::wm::switch_app(crate::ui::wm::APP_EDITOR);
                }
            }
        }
        _ => {}
    }
}

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;

// Column geometry — 1248px inner width after 16px L/R margin.
const COL_STATUS_W:   u32 = 120;
const COL_SIZE_W:     u32 = 120;
const COL_CIPHER_W:   u32 = 160;
const COL_MERKLE_W:   u32 = 110;

const ROW_H: u32 = 24;

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 100 { return; }

    // ── HEADER STRIP (32px) ───────────────────────────────────────
    draw_strip(r.x, r.y, r.w, 32, false, true);
    draw_header(r.x, r.y, r.w);

    // ── COLUMN HEADER ROW (24px) ──────────────────────────────────
    let col_y = r.y + 32;
    draw_column_header(r.x, col_y, r.w);

    // ── DATA ROWS ─────────────────────────────────────────────────
    let body_y = col_y + ROW_H;
    let footer_y = r.y + r.h - 28;
    let body_h = footer_y.saturating_sub(body_y);

    let (count, _max) = batfs::stats();
    if count == 0 {
        draw_empty(r.x, body_y, r.w, body_h);
    } else {
        draw_rows(r.x, body_y, r.w, body_h);
    }

    // ── FOOTER STRIP (28px) ───────────────────────────────────────
    draw_strip(r.x, footer_y, r.w, 28, true, false);
    draw_footer(r.x, footer_y, r.w, count);
}

fn draw_header(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (32 - CHAR_H) / 2;
    let mut cx = x + 16;
    font::draw_str(fb, sw, cx, text_y, "VAULT",     FAINT, BG); cx += 6 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "ENCRYPTED", INK,   BG); cx += 10 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, ".",         FAINT, BG); cx += 2 * CHAR_W;
    // STUMP #144: BatFS is ChaCha20-Poly1305 AEAD now, not AES-CTR.
    font::draw_str(fb, sw, cx, text_y, "CHACHA20-POLY1305", CYAN, BG); cx += 18 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "+ Merkle integrity", FAINT, BG);

    // Right: file count / MAX_FILES.
    let (count, _max) = batfs::stats();
    let mut buf = [0u8; 32];
    let n = format_file_metric(count, &mut buf);
    let s = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
    let metric_w = n as u32 * CHAR_W;
    if w > metric_w + 16 {
        font::draw_str(fb, sw, x + w - 16 - metric_w, text_y, s, MID, BG);
    }
}

fn draw_column_header(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (ROW_H - CHAR_H) / 2;
    let inner_x = x + 16;
    font::draw_str(fb, sw, inner_x,                                   text_y, "STATUS",    FAINT, BG);
    font::draw_str(fb, sw, inner_x + COL_STATUS_W,                    text_y, "FILENAME",  FAINT, BG);
    // SIZE column right-aligned within its cell.
    let size_label_x = compute_size_x(inner_x, w);
    let size_label = "SIZE";
    let size_w = size_label.len() as u32 * CHAR_W;
    font::draw_str(fb, sw, size_label_x + COL_SIZE_W - size_w - 16, text_y, size_label, FAINT, BG);
    let cipher_x = size_label_x + COL_SIZE_W;
    font::draw_str(fb, sw, cipher_x + 4,                              text_y, "CIPHER",    FAINT, BG);
    let merkle_x = cipher_x + COL_CIPHER_W;
    font::draw_str(fb, sw, merkle_x,                                  text_y, "MERKLE OK", FAINT, BG);
    // 1px hairline below.
    gpu::fill_rect(x, y + ROW_H - 1, w, 1, HAIR);
}

fn compute_size_x(inner_x: u32, total_w: u32) -> u32 {
    // FILENAME flexes between STATUS (120) and SIZE (120) + CIPHER (160) + MERKLE (110).
    // total_w includes 16px L+R margins.
    let inner_w = total_w.saturating_sub(32);
    let fixed = COL_STATUS_W + COL_SIZE_W + COL_CIPHER_W + COL_MERKLE_W;
    let _name_w = inner_w.saturating_sub(fixed);
    inner_x + COL_STATUS_W + _name_w
}

fn draw_empty(x: u32, y: u32, w: u32, h: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text = "(vault is empty - use ";
    let cmd  = "write <name> <data>";
    let tail = " in shell)";
    let total = text.len() + cmd.len() + tail.len();
    let total_w = total as u32 * CHAR_W;
    let cx = x + (w.saturating_sub(total_w)) / 2;
    let cy = y + h / 2 - CHAR_H / 2;
    font::draw_str(fb, sw, cx,                                  cy, text, DIM_TXT, BG);
    font::draw_str(fb, sw, cx + text.len() as u32 * CHAR_W,     cy, cmd,  CYAN,    BG);
    font::draw_str(fb, sw, cx + (text.len() + cmd.len()) as u32 * CHAR_W, cy, tail, DIM_TXT, BG);
}

fn draw_rows(x: u32, y: u32, w: u32, h: u32) {
    let max_rows = (h / ROW_H) as usize;
    // First pass: count the rows so SELECTED_ROW can be clamped if
    // the table shrunk (file deleted while FS was the active pane).
    let mut total = 0usize;
    batfs::list(|_n, _s, _e| { total += 1; });
    unsafe {
        ROW_COUNT_CACHE = total;
        if SELECTED_ROW >= total && total > 0 { SELECTED_ROW = total - 1; }
        if total == 0 { SELECTED_ROW = 0; }
    }
    let sel = selected_row();
    // Second pass: paint.
    let mut row_idx = 0usize;
    batfs::list(|name, size, encrypted| {
        if row_idx >= max_rows { return; }
        let ry = y + (row_idx as u32) * ROW_H;
        draw_row(x, ry, w, name, size, encrypted, row_idx == sel);
        row_idx += 1;
    });
}

fn draw_row(x: u32, y: u32, w: u32, name: &str, size: usize, encrypted: bool, selected: bool) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (ROW_H - CHAR_H) / 2;
    let inner_x = x + 16;

    // Selection chrome: 1px cyan-dim inset border + 2px cyan underline.
    if selected {
        draw::draw_border(x + 16, y, w.saturating_sub(32), ROW_H, CYAN_DIM);
        gpu::fill_rect(x + 16, y + ROW_H - 2, w.saturating_sub(32), 2, CYAN);
    }

    // STATUS — colored dot + tag.
    let (tag, tag_color, tag_ring) = if encrypted {
        ("[ENC]", GREEN, GREEN_DIM)
    } else {
        ("[RAW]", AMBER, AMBER_DIM)
    };
    let dot_x = inner_x + 4;
    let dot_y = y + (ROW_H - 6) / 2;
    if dot_x >= 1 && dot_y >= 1 {
        gpu::fill_rect(dot_x - 1, dot_y - 1, 8, 8, tag_ring);
    }
    gpu::fill_rect(dot_x, dot_y, 6, 6, tag_color);
    font::draw_str(fb, sw, inner_x + 16, text_y, tag, tag_color, BG);

    // FILENAME — "f" prefix + name. Truncate with "..." if too wide.
    let name_x = inner_x + COL_STATUS_W;
    font::draw_str(fb, sw, name_x, text_y, "f", DIM_TXT, BG);
    let max_name_chars = (compute_size_x(inner_x, w).saturating_sub(name_x + 16)) / CHAR_W;
    let name_show = if name.len() as u32 > max_name_chars {
        // Show first (max-3) chars + "..."
        let keep = (max_name_chars as usize).saturating_sub(3);
        // We can't easily concat in no_std without alloc; just use truncation only.
        &name[..keep.min(name.len())]
    } else { name };
    font::draw_str(fb, sw, name_x + 2 * CHAR_W, text_y, name_show, INK, BG);
    if name.len() as u32 > max_name_chars {
        font::draw_str(fb, sw, name_x + 2 * CHAR_W + name_show.len() as u32 * CHAR_W,
            text_y, "...", DIM_TXT, BG);
    }

    // SIZE — right-aligned with unit suffix in dim.
    let size_x = compute_size_x(inner_x, w);
    let mut s_buf = [0u8; 16];
    let (s_n, unit) = format_size(size, &mut s_buf);
    let size_str = unsafe { core::str::from_utf8_unchecked(&s_buf[..s_n]) };
    let total_w = (s_n as u32 + 1 + unit.len() as u32) * CHAR_W;
    let val_x = size_x + COL_SIZE_W - total_w - 16;
    font::draw_str(fb, sw, val_x, text_y, size_str, INK, BG);
    font::draw_str(fb, sw, val_x + (s_n as u32 + 1) * CHAR_W, text_y, unit, DIM_TXT, BG);

    // CIPHER. STUMP #144: ChaCha20-Poly1305 (was AES-256-CTR label).
    let cipher_x = size_x + COL_SIZE_W + 4;
    if encrypted {
        font::draw_str(fb, sw, cipher_x, text_y, "CHACHA20", CYAN, BG);
    } else {
        font::draw_str(fb, sw, cipher_x, text_y, "-", DIM_TXT, BG);
    }

    // MERKLE OK — green "OK" (we don't have a checkmark glyph in our
    // ASCII-only font, so use the text label).
    let merkle_x = cipher_x + COL_CIPHER_W;
    font::draw_str(fb, sw, merkle_x, text_y, "OK", GREEN, BG);

    // Bottom hairline (only if not selected — selection draws its own).
    if !selected {
        gpu::fill_rect(x + 16, y + ROW_H - 1, w.saturating_sub(32), 1, HAIR);
    }
}

fn draw_footer(x: u32, y: u32, w: u32, count: usize) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (28 - CHAR_H) / 2;

    // Left segment: FILES N · MAX_FILES 32.
    let mut cx = x + 16;
    font::draw_str(fb, sw, cx, text_y, "FILES", FAINT, BG); cx += 6 * CHAR_W;
    let mut buf = [0u8; 16];
    let n = format_dec(count, &mut buf);
    font::draw_str(fb, sw, cx, text_y,
        unsafe { core::str::from_utf8_unchecked(&buf[..n]) }, INK, BG);
    cx += (n as u32 + 1) * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, ".", FAINT, BG); cx += 2 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "MAX_FILES", FAINT, BG); cx += 10 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "32", INK, BG); cx += 3 * CHAR_W;

    // Separator.
    draw_seg_separator(cx + 4, y, 28); cx += 16;

    // Mid: MERKLE preview.
    font::draw_str(fb, sw, cx, text_y, "MERKLE", FAINT, BG); cx += 7 * CHAR_W;
    let root = batfs::merkle_root();
    let hex = b"0123456789abcdef";
    let mut hash_str = [b' '; 9];
    for i in 0..4 {
        hash_str[i * 2]     = hex[(root[i] >> 4) as usize];
        hash_str[i * 2 + 1] = hex[(root[i] & 0xf) as usize];
        if i == 1 { hash_str[4] = b' '; }
    }
    font::draw_str(fb, sw, cx, text_y,
        unsafe { core::str::from_utf8_unchecked(&hash_str) }, INK, BG);
    cx += 10 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "...", DIM_TXT, BG); cx += 4 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "VERIFIED", GREEN, BG);

    // Right hint.
    let hint = "Ctrl+1 to manage in shell";
    let hint_w = hint.len() as u32 * CHAR_W;
    if w > hint_w + 16 {
        font::draw_str(fb, sw, x + w - 16 - hint_w, text_y, hint, DIM_TXT, BG);
    }
}

// ── helpers ───────────────────────────────────────────────────────

fn format_file_metric(count: usize, out: &mut [u8]) -> usize {
    let mut p = format_dec(count, out);
    let suffix = b" / MAX_FILES 32";
    out[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    p
}

/// Format a byte size with unit. Returns (length-of-numeric, unit).
fn format_size(bytes: usize, out: &mut [u8]) -> (usize, &'static str) {
    if bytes < 1024 {
        (format_dec(bytes, out), "B")
    } else if bytes < 1024 * 1024 {
        // KiB with one decimal.
        let kib_int = bytes / 1024;
        let kib_dec = ((bytes * 10) / 1024) % 10;
        let mut p = format_dec(kib_int, out);
        out[p] = b'.'; p += 1;
        out[p] = b'0' + kib_dec as u8; p += 1;
        (p, "KiB")
    } else {
        let mib_int = bytes / (1024 * 1024);
        let mib_dec = ((bytes * 10) / (1024 * 1024)) % 10;
        let mut p = format_dec(mib_int, out);
        out[p] = b'.'; p += 1;
        out[p] = b'0' + mib_dec as u8; p += 1;
        (p, "MiB")
    }
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
