#![allow(dead_code)]
// Bat_OS — ED · Code Editor
//
// STUMP #129 — Claude-Design Wave-3 port. Source artifacts in
// `docs/design/apps-fs-ed-cm/`. Read-only viewer for now —
// shows a hardcoded sample of kernel_main.rs with syntax-aware
// coloring per the spec sheet's KEYWORD/STRING/COMMENT/ATTR/
// IDENT/PUNCT mapping.
//
// Layout: 24px tab bar (3 tabs + "+" slot), 56px gutter (line
// numbers + current-line accent), code area (~16px line height),
// 28px status strip (LANG / ENC / POS / LF + READ ONLY).

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::widgets::{
    self as W, draw_strip, draw_seg_separator, draw_code_line, Tok,
    BG, INK, MID, DIM_TXT, FAINT, CYAN, AMBER, HAIR, HAIR_HI,
};

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;
const TAB_BAR_H: u32 = 24;
const STATUS_H:  u32 = 28;
const GUTTER_W:  u32 = 56;
const TAB_W:     u32 = 168;
const NEW_TAB_W: u32 = 32;
const GUTTER_BG: u32 = 0xFF080808;
const LINE_NUM:  u32 = 0xFF3A3A3A;
const CUR_LINE_BG: u32 = 0xFF0E1F22; // approximation of "barely there cyan tint"

// Cursor sits on line 17 per spec (1-indexed).
const CURSOR_LINE: usize = 16; // 0-indexed
const CURSOR_COL:  u32   = 4;  // 5th char (after "    " indent)

// ─── Sample code (hardcoded — full editor is a future STUMP) ─────

fn sample_lines() -> &'static [&'static [(Tok, &'static str)]] {
    static LINES: &[&[(Tok, &str)]] = &[
        &[(Tok::Comment, "//! Bat_OS bare-metal kernel entry")],
        &[(Tok::Comment, "//! v0.5.0-DEV . aarch64-unknown-none-softfloat")],
        &[],
        &[(Tok::Attr, "#![no_std]")],
        &[(Tok::Attr, "#![no_main]")],
        &[],
        &[(Tok::Keyword, "use"), (Tok::Punct, " "), (Tok::Ident, "core"), (Tok::Punct, "::"), (Tok::Ident, "panic"), (Tok::Punct, "::"), (Tok::Ident, "PanicInfo"), (Tok::Punct, ";")],
        &[(Tok::Keyword, "use"), (Tok::Punct, " "), (Tok::Ident, "crate"), (Tok::Punct, "::{"), (Tok::Ident, "kernel"), (Tok::Punct, ", "), (Tok::Ident, "drivers"), (Tok::Punct, ", "), (Tok::Ident, "fs"), (Tok::Punct, ", "), (Tok::Ident, "net"), (Tok::Punct, ", "), (Tok::Ident, "ui"), (Tok::Punct, "};")],
        &[],
        &[(Tok::Comment, "/// Entry point - called from boot.S after stack + MMU.")],
        &[(Tok::Attr, "#[no_mangle]")],
        &[(Tok::Keyword, "pub"), (Tok::Punct, " "), (Tok::Keyword, "extern"), (Tok::Punct, " "), (Tok::String, "\"C\""), (Tok::Punct, " "), (Tok::Keyword, "fn"), (Tok::Punct, " "), (Tok::Ident, "kernel_main"), (Tok::Punct, "("), (Tok::Ident, "master_key"), (Tok::Punct, ": &["), (Tok::Ident, "u8"), (Tok::Punct, "; "), (Tok::Ident, "32"), (Tok::Punct, "]) -> "), (Tok::Ident, "!"), (Tok::Punct, " {")],
        &[(Tok::Punct, "    "), (Tok::Ident, "kernel"), (Tok::Punct, "::"), (Tok::Ident, "mm"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[(Tok::Punct, "    "), (Tok::Ident, "kernel"), (Tok::Punct, "::"), (Tok::Ident, "process"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[(Tok::Punct, "    "), (Tok::Ident, "kernel"), (Tok::Punct, "::"), (Tok::Ident, "scheduler"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[(Tok::Punct, "    "), (Tok::Ident, "kernel"), (Tok::Punct, "::"), (Tok::Ident, "ipc"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[(Tok::Punct, "    "), (Tok::Ident, "kernel"), (Tok::Punct, "::"), (Tok::Ident, "arch"), (Tok::Punct, "::"), (Tok::Ident, "init_exceptions"), (Tok::Punct, "();")],
        &[],
        &[(Tok::Punct, "    "), (Tok::Comment, "// storage + net come up after the core is alive")],
        &[(Tok::Punct, "    "), (Tok::Ident, "fs"), (Tok::Punct, "::"), (Tok::Ident, "batfs"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "(&"), (Tok::Ident, "master_key"), (Tok::Punct, ");")],
        &[(Tok::Punct, "    "), (Tok::Ident, "drivers"), (Tok::Punct, "::"), (Tok::Ident, "virtio"), (Tok::Punct, "::"), (Tok::Ident, "net"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[(Tok::Punct, "    "), (Tok::Ident, "net"), (Tok::Punct, "::"), (Tok::Ident, "init"), (Tok::Punct, "();")],
        &[],
        &[(Tok::Punct, "    "), (Tok::Comment, "// hand off to the operator shell - never returns")],
        &[(Tok::Punct, "    "), (Tok::Ident, "ui"), (Tok::Punct, "::"), (Tok::Ident, "shell"), (Tok::Punct, "::"), (Tok::Ident, "run"), (Tok::Punct, "();")],
        &[(Tok::Punct, "}")],
        &[],
        &[(Tok::Attr, "#[panic_handler]")],
        &[(Tok::Keyword, "fn"), (Tok::Punct, " "), (Tok::Ident, "panic"), (Tok::Punct, "("), (Tok::Ident, "_info"), (Tok::Punct, ": &"), (Tok::Ident, "PanicInfo"), (Tok::Punct, ") -> "), (Tok::Ident, "!"), (Tok::Punct, " { "), (Tok::Keyword, "loop"), (Tok::Punct, " {} }")],
    ];
    LINES
}

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 100 { return; }

    // ── TAB BAR ───────────────────────────────────────────────────
    draw_strip(r.x, r.y, r.w, TAB_BAR_H, false, true);
    draw_tabs(r.x, r.y, r.w);

    // ── BODY: gutter + code ──────────────────────────────────────
    let body_y = r.y + TAB_BAR_H;
    let status_y = r.y + r.h - STATUS_H;
    let body_h = status_y.saturating_sub(body_y);
    draw_gutter_and_code(r.x, body_y, r.w, body_h);

    // ── STATUS STRIP ──────────────────────────────────────────────
    draw_strip(r.x, status_y, r.w, STATUS_H, true, false);
    draw_status_strip(r.x, status_y, r.w);
}

fn draw_tabs(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();

    // Three tabs.
    let tabs: &[(&str, bool, bool)] = &[
        ("kernel_main.rs", true,  false),
        ("lib.rs",         false, false),
        ("Cargo.toml",     false, true),  // dirty
    ];
    let mut tx = x;
    for (name, active, dirty) in tabs {
        // Right border.
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
        // Close glyph at right edge.
        let close_x = tx + TAB_W - 16;
        let punct_color = if *active { FAINT } else { FAINT };
        let x_color = if *active { CYAN } else { FAINT };
        font::draw_str(fb, sw, close_x, text_y, ":", punct_color, BG);
        font::draw_str(fb, sw, close_x + CHAR_W, text_y, "x", x_color, BG);
        let _ = after_name;

        // Active underline.
        if *active {
            gpu::fill_rect(tx, y + TAB_BAR_H - 2, TAB_W, 2, CYAN);
        }
        tx += TAB_W;
    }
    // "+" new-tab slot at far right.
    let plus_x = x + w - NEW_TAB_W;
    gpu::fill_rect(plus_x, y, 1, TAB_BAR_H, HAIR);
    font::draw_str(fb, sw, plus_x + (NEW_TAB_W - CHAR_W) / 2,
        y + (TAB_BAR_H - CHAR_H) / 2, "+", DIM_TXT, BG);
}

fn draw_gutter_and_code(x: u32, y: u32, w: u32, h: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();

    // Gutter background + right hairline.
    gpu::fill_rect(x, y, GUTTER_W, h, GUTTER_BG);
    gpu::fill_rect(x + GUTTER_W, y, 1, h, HAIR_HI);

    let lines = sample_lines();
    let pad_top: u32 = 8;
    let line_x = x + GUTTER_W + 16;

    for (i, spans) in lines.iter().enumerate() {
        let ly = y + pad_top + (i as u32) * CHAR_H;
        if ly + CHAR_H > y + h { break; }
        let is_cur = i == CURSOR_LINE;

        // Current-line tint across the code area.
        if is_cur {
            gpu::fill_rect(x + GUTTER_W + 1, ly, w - GUTTER_W - 1, CHAR_H, CUR_LINE_BG);
            // 1px cyan accent at gutter's right edge.
            gpu::fill_rect(x + GUTTER_W, ly, 1, CHAR_H, CYAN);
        }

        // Line number, right-aligned in gutter (12px right padding).
        let line_no = (i + 1) as usize;
        let mut buf = [0u8; 8];
        let n = format_dec(line_no, &mut buf);
        let ln_str = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
        let ln_w = n as u32 * CHAR_W;
        let ln_x = x + GUTTER_W - 12 - ln_w;
        let ln_color = if is_cur { INK } else { LINE_NUM };
        font::draw_str(fb, sw, ln_x, ly,
            ln_str, ln_color, if is_cur { CUR_LINE_BG } else { GUTTER_BG });

        // Code line.
        if !spans.is_empty() {
            // The line draw helper paints with BG bg color; on the
            // current line we want CUR_LINE_BG instead. Cheap path:
            // paint a base layer over the current line, then call
            // draw_code_line to overlay text.
            draw_code_line(line_x, ly, spans);
        }

        // Cursor — 8x14 cyan block, only on the current line.
        if is_cur {
            let cur_x = line_x + CURSOR_COL * CHAR_W;
            gpu::fill_rect(cur_x, ly + 1, 8, 14, CYAN);
        }
    }
}

fn draw_status_strip(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (STATUS_H - CHAR_H) / 2;

    let mut cx = x + 16;
    // LANG.
    font::draw_str(fb, sw, cx, text_y, "LANG", FAINT, BG); cx += 5 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "RUST", INK, BG);   cx += 5 * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;
    // ENC.
    font::draw_str(fb, sw, cx, text_y, "ENC", FAINT, BG);  cx += 4 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "UTF-8", INK, BG);  cx += 6 * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;
    // POS.
    font::draw_str(fb, sw, cx, text_y, "POS", FAINT, BG);  cx += 4 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "Ln 17, Col 5", INK, BG); cx += 13 * CHAR_W;
    draw_seg_separator(cx, y, STATUS_H); cx += 12;
    // LF.
    font::draw_str(fb, sw, cx, text_y, "LF", FAINT, BG); cx += 3 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "UNIX", INK, BG);

    // Right: READ ONLY in amber.
    let label = "READ ONLY";
    let label_w = label.len() as u32 * CHAR_W;
    if w > label_w + 16 {
        font::draw_str(fb, sw, x + w - 16 - label_w, text_y, label, AMBER, BG);
    }

    // Suppress unused-import warnings.
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
