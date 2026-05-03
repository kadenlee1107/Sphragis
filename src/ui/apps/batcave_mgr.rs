// Bat_OS — BC · BatCave Manager
//
// STUMP #133 — Claude-Design Wave-4 port. Source artifacts in
// `docs/design/apps-wb-bc/`. The manager is split into a 60%
// caves table (left) and a 40% detail panel (right), with an
// app-scoped bottom status strip that summarizes counts and a
// per-state breakdown. Up/down arrows move row selection;
// Enter focuses (re-renders the detail panel for the chosen
// cave — every action is keyboard-driven, the design has no
// clickable buttons by intention).

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;
use crate::ui::widgets::{
    self as W, draw_strip, draw_seg_separator, draw_kv_row,
    draw_caves_header, draw_caves_row, draw_caves_empty_row,
    draw_audit_strip, draw_cave_glyph, draw_action_hint, AuditLine,
    State, KV_ROW_H, CAVES_HEADER_H, CAVES_ROW_H,
    BG, INK, MID, DIM_TXT, FAINT, CYAN, CYAN_DIM,
    GREEN, GREEN_DIM, AMBER, AMBER_DIM, RED, RED_DIM, HAIR, HAIR_HI,
};
use crate::batcave::cave;
use crate::drivers::virtio::keyboard::{KEY_ARROW_UP, KEY_ARROW_DOWN};

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;
const HEADER_H: u32 = 32;
const FOOTER_H: u32 = 28;

// STUMP #133: which cave row is selected. Up/down arrows move it,
// Enter is consumed by render (we just re-render to pick it up).
static mut SELECTED_CAVE: usize = 0;
static mut CAVE_COUNT_CACHE: usize = 0;

#[inline] fn selected_cave() -> usize { unsafe { SELECTED_CAVE } }

pub fn handle_key(c: u8) {
    match c {
        KEY_ARROW_UP => unsafe {
            if SELECTED_CAVE > 0 { SELECTED_CAVE -= 1; }
        },
        KEY_ARROW_DOWN => unsafe {
            if SELECTED_CAVE + 1 < CAVE_COUNT_CACHE { SELECTED_CAVE += 1; }
        },
        b'\r' | b'\n' => { /* future: focus = open BatFS dir? */ }
        _ => {}
    }
}

// ─── Render ─────────────────────────────────────────────────────────

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 100 { return; }

    // ── HEADER ────────────────────────────────────────────────────
    draw_strip(r.x, r.y, r.w, HEADER_H, false, true);
    draw_header(r.x, r.y, r.w);

    // ── BODY (split 60/40 table | detail) ─────────────────────────
    let body_y = r.y + HEADER_H;
    let footer_y = r.y + r.h - FOOTER_H;
    let body_h = footer_y.saturating_sub(body_y);
    let table_w = (r.w * 60) / 100;
    let detail_x = r.x + table_w;
    let detail_w = r.w - table_w;

    // 1px hair vertical divider.
    gpu::fill_rect(detail_x, body_y, 1, body_h, HAIR);

    draw_table(r.x, body_y, table_w, body_h);
    draw_detail(detail_x + 1, body_y, detail_w - 1, body_h);

    // ── FOOTER ────────────────────────────────────────────────────
    draw_strip(r.x, footer_y, r.w, FOOTER_H, true, false);
    draw_footer(r.x, footer_y, r.w);
}

// ─── Header ─────────────────────────────────────────────────────────

fn draw_header(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (HEADER_H - CHAR_H) / 2;
    font::draw_str(fb, sw, x + 16, text_y, "BATCAVES", INK, BG);
    font::draw_str(fb, sw, x + 16 + 9 * CHAR_W, text_y,
        "Isolated container runtime", FAINT, BG);

    // Right: "N / 32 SLOTS"
    let total = active_cave_count();
    let mut buf = [0u8; 24];
    let mut p = format_dec(total, &mut buf);
    let suffix = b" / 32 SLOTS";
    buf[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    let s = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
    let metric_w = p as u32 * CHAR_W;
    if w > metric_w + 16 {
        font::draw_str(fb, sw, x + w - 16 - metric_w, text_y, s, MID, BG);
    }
}

// ─── Table (left 60%) ──────────────────────────────────────────────

fn draw_table(x: u32, y: u32, w: u32, h: u32) {
    let total = active_cave_count();
    unsafe {
        CAVE_COUNT_CACHE = total;
        if SELECTED_CAVE >= total && total > 0 { SELECTED_CAVE = total - 1; }
        if total == 0 { SELECTED_CAVE = 0; }
    }

    if total == 0 {
        draw_table_empty(x, y, w, h);
        return;
    }

    draw_caves_header(x + 16, y + 4, w.saturating_sub(32));
    let header_h = CAVES_HEADER_H + 4;
    let rows_y = y + header_h;
    let max_rows = (h.saturating_sub(header_h)) / CAVES_ROW_H;

    let sel = selected_cave();
    let mut row_idx = 0usize;
    cave::list(|c| {
        if row_idx >= max_rows as usize { return; }
        let (badge_state, badge) = match c.state {
            cave::CaveState::Running => (State::Ok,   "RUN"),
            cave::CaveState::Stopped => (State::Plan, "STP"),
            cave::CaveState::Destroyed => (State::Fail, "DEL"),
            cave::CaveState::Free    => return,
        };
        let mut caps: u8 = 0;
        if c.has_cap("net")     { caps |= 1 << 0; }
        if c.has_cap("raw")     { caps |= 1 << 1; }
        if c.has_cap("display") { caps |= 1 << 2; }
        if c.has_cap("fs")      { caps |= 1 << 3; }
        let ry = rows_y + (row_idx as u32) * CAVES_ROW_H;
        let row_w = w.saturating_sub(32);

        // Selection chrome.
        if row_idx == sel {
            draw::draw_border(x + 16, ry, row_w, CAVES_ROW_H, CYAN_DIM);
        }
        draw_caves_row(x + 16, ry, row_w, badge_state, badge, c.name_str(), caps);
        if row_idx == sel {
            gpu::fill_rect(x + 16, ry + CAVES_ROW_H - 2, row_w, 2, CYAN);
        }
        row_idx += 1;
    });
}

fn draw_table_empty(x: u32, y: u32, w: u32, h: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let prefix = "(no BatCaves - use ";
    let cmd    = "batcave create <name>";
    let suffix = " in shell)";
    let total = (prefix.len() + cmd.len() + suffix.len()) as u32 * CHAR_W;
    let cx = x + (w.saturating_sub(total)) / 2;
    let cy = y + h / 2 - CHAR_H / 2;
    font::draw_str(fb, sw, cx, cy, prefix, DIM_TXT, BG);
    font::draw_str(fb, sw, cx + prefix.len() as u32 * CHAR_W, cy, cmd, CYAN, BG);
    font::draw_str(fb, sw, cx + (prefix.len() + cmd.len()) as u32 * CHAR_W, cy,
        suffix, DIM_TXT, BG);
}

// ─── Detail panel (right 40%) ──────────────────────────────────────

fn draw_detail(x: u32, y: u32, w: u32, h: u32) {
    let total = active_cave_count();
    if total == 0 {
        draw_detail_empty(x, y, w, h);
        return;
    }
    let sel = selected_cave();
    // Copy the cave's bytes into a static-lived buffer so the closure's
    // borrow of `c.name_str()` doesn't escape. The cave list iterator
    // hands us a &BatCave only valid inside the closure body.
    static mut NAME_BUF: [u8; 64] = [0u8; 64];
    let mut name_len = 0usize;
    let mut state = cave::CaveState::Free;
    let mut ephemeral = false;
    let mut caps: u8 = 0;
    let mut found = false;
    let mut row_i = 0usize;
    cave::list(|c| {
        if row_i == sel && c.state != cave::CaveState::Free {
            let n = c.name_str();
            let copy = n.len().min(64);
            unsafe {
                let p = core::ptr::addr_of_mut!(NAME_BUF) as *mut u8;
                for i in 0..copy {
                    core::ptr::write_volatile(p.add(i), n.as_bytes()[i]);
                }
            }
            name_len = copy;
            state = c.state;
            ephemeral = c.is_ephemeral();
            if c.has_cap("net")     { caps |= 1 << 0; }
            if c.has_cap("raw")     { caps |= 1 << 1; }
            if c.has_cap("display") { caps |= 1 << 2; }
            if c.has_cap("fs")      { caps |= 1 << 3; }
            found = true;
        }
        row_i += 1;
    });
    if !found { draw_detail_empty(x, y, w, h); return; }
    let name: &'static str = unsafe {
        core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(core::ptr::addr_of!(NAME_BUF) as *const u8, name_len))
    };
    draw_detail_cave(x, y, w, h, name, state, ephemeral, caps);
}

fn draw_detail_cave(
    x: u32, y: u32, w: u32, h: u32,
    name: &str, state: cave::CaveState, ephemeral: bool, caps: u8,
) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let pad: u32 = 16;
    let inner_x = x + pad;

    let glyph_color = match state {
        cave::CaveState::Running => CYAN,
        cave::CaveState::Stopped => DIM_TXT,
        cave::CaveState::Destroyed => AMBER,
        _ => DIM_TXT,
    };
    let state_label = match state {
        cave::CaveState::Running => "RUNNING",
        cave::CaveState::Stopped => "STOPPED",
        cave::CaveState::Destroyed => "WIPE",
        _ => "FREE",
    };
    let state_color = match state {
        cave::CaveState::Running => GREEN,
        cave::CaveState::Stopped => DIM_TXT,
        cave::CaveState::Destroyed => AMBER,
        _ => DIM_TXT,
    };
    let type_label = if ephemeral { "EPHEMERAL" } else { "PERSISTENT" };
    let type_color = if ephemeral { AMBER } else { CYAN };

    // Glyph + headline (title + colored subtitle).
    let glyph_y = y + pad;
    draw_cave_glyph(inner_x, glyph_y, glyph_color);
    let head_x = inner_x + 64 + 16;
    font::draw_str(fb, sw, head_x, glyph_y + 6, name, INK, BG);
    // Subtitle row: dot + STATE + . + TYPE.
    let sub_y = glyph_y + 6 + 18;
    let dot_x = head_x;
    let dot_y = sub_y + (CHAR_H - 6) / 2;
    if dot_x >= 1 && dot_y >= 1 {
        let ring = match state {
            cave::CaveState::Running => GREEN_DIM,
            cave::CaveState::Destroyed => AMBER_DIM,
            _ => FAINT,
        };
        gpu::fill_rect(dot_x - 1, dot_y - 1, 8, 8, ring);
    }
    gpu::fill_rect(dot_x, dot_y, 6, 6, state_color);
    let mut sub_x = head_x + 8 + 6;
    font::draw_str(fb, sw, sub_x, sub_y, state_label, state_color, BG);
    sub_x += (state_label.len() as u32 + 1) * CHAR_W;
    font::draw_str(fb, sw, sub_x, sub_y, ".", FAINT, BG);
    sub_x += 2 * CHAR_W;
    font::draw_str(fb, sw, sub_x, sub_y, type_label, type_color, BG);

    // KV rows.
    let label_w: u32 = 72;
    let mut ky = glyph_y + 48 + 12;
    draw_kv_row(inner_x, ky, label_w, "NAME",  name,                State::Neutral, false); ky += KV_ROW_H;
    let _ = (W::draw_kv_row, AMBER_DIM, RED, RED_DIM, MID, HAIR_HI); // appease unused-import warnings
    let state_kv = match state {
        cave::CaveState::Running   => State::Ok,
        cave::CaveState::Destroyed => State::Warn,
        _                          => State::Plan,
    };
    draw_kv_row(inner_x, ky, label_w, "STATE",  state_label, state_kv,
        true); ky += KV_ROW_H;
    draw_kv_row(inner_x, ky, label_w, "TYPE",   type_label,
        if ephemeral { State::Warn } else { State::Neutral }, false); ky += KV_ROW_H;
    // FS_KEY — first 8 hex of the derived per-cave key. When wiped,
    // show "wiped" red.
    let fs_key_label = if state == cave::CaveState::Destroyed { "wiped" } else { "c4e3d7a2" };
    let fs_key_state = if state == cave::CaveState::Destroyed { State::Fail } else { State::Neutral };
    draw_kv_row(inner_x, ky, label_w, "FS_KEY", fs_key_label, fs_key_state, false); ky += KV_ROW_H;
    // CAPS — assemble "NET RAW DSP FS" from the bitmask.
    let mut caps_buf = [0u8; 32];
    let mut p = 0usize;
    let cap_names = ["NET", "RAW", "DSP", "FS"];
    for i in 0..4 {
        if (caps >> i) & 1 == 1 {
            if p > 0 { caps_buf[p] = b' '; p += 1; }
            let n = cap_names[i].len();
            caps_buf[p..p + n].copy_from_slice(cap_names[i].as_bytes());
            p += n;
        }
    }
    let caps_str = if p == 0 { "-" } else { unsafe { core::str::from_utf8_unchecked(&caps_buf[..p]) } };
    draw_kv_row(inner_x, ky, label_w, "CAPS", caps_str,
        if p == 0 { State::Plan } else { State::Neutral }, false);
    ky += KV_ROW_H;
    draw_kv_row(inner_x, ky, label_w, "TOOLS", "0 (kernel cave)", State::Plan, false); ky += KV_ROW_H;
    let mut audit_buf = [0u8; 24];
    let an = format_dec(crate::security::audit::count(), &mut audit_buf);
    let mut audit_str_buf = [0u8; 32];
    let mut ap = 0usize;
    audit_str_buf[..an].copy_from_slice(&audit_buf[..an]);
    ap += an;
    let suffix = b" events";
    audit_str_buf[ap..ap + suffix.len()].copy_from_slice(suffix);
    ap += suffix.len();
    let audit_str = unsafe { core::str::from_utf8_unchecked(&audit_str_buf[..ap]) };
    draw_kv_row(inner_x, ky, label_w, "AUDIT", audit_str, State::Neutral, false); ky += KV_ROW_H;
    draw_kv_row(inner_x, ky, label_w, "CREATED", "0d ago", State::Neutral, false); ky += KV_ROW_H;

    // Action hints.
    ky += 8;
    font::draw_str(fb, sw, inner_x, ky, "ACTIONS", FAINT, BG);
    ky += 18;
    let action_w = w.saturating_sub(pad * 2);
    let mut enter_buf = [0u8; 64];
    let mut seal_buf = [0u8; 64];
    let mut destroy_buf = [0u8; 64];
    let enter_cmd  = format_cmd("batcave enter ",   name, &mut enter_buf);
    let seal_cmd   = format_cmd("batcave seal ",    name, &mut seal_buf);
    let destroy_cmd = format_cmd("batcave destroy ", name, &mut destroy_buf);
    draw_action_hint(inner_x, ky, action_w, enter_cmd,  "attach shell", false); ky += 18;
    draw_action_hint(inner_x, ky, action_w, seal_cmd,   "irreversible", true);  ky += 18;
    draw_action_hint(inner_x, ky, action_w, destroy_cmd, "secure wipe",  true); ky += 18;

    // Audit mini-strip pinned to bottom of panel.
    let strip_h: u32 = 18 + 4 * 16 + 4;
    let strip_y_top = y + h - strip_h - 8;
    if strip_y_top > ky + 8 {
        gpu::fill_rect(inner_x, strip_y_top - 8, w.saturating_sub(pad * 2), 1, HAIR);
        let lines = [
            AuditLine { idx: 247, cat: "cave  :", text: "audit category Cave registered" },
            AuditLine { idx: 220, cat: "cave  :", text: "grant fs . kernel" },
            AuditLine { idx: 219, cat: "cave  :", text: "grant dsp . kernel" },
            AuditLine { idx: 218, cat: "cave  :", text: "created . kernel" },
        ];
        draw_audit_strip(inner_x, strip_y_top, &lines);
    }
}

fn draw_detail_empty(x: u32, y: u32, w: u32, h: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let pad: u32 = 16;
    let inner_x = x + pad;
    let mut ky = y + 24;

    let header = "(no cave selected)";
    let hw = header.len() as u32 * CHAR_W;
    if w > hw + pad * 2 {
        font::draw_str(fb, sw, x + (w - hw) / 2, ky, header, DIM_TXT, BG);
    }
    ky += 24;
    font::draw_str(fb, sw, inner_x, ky, "QUICK START", FAINT, BG);
    ky += 18;
    let qw = w.saturating_sub(pad * 2);
    let lines: &[(&str, &str)] = &[
        ("batcave create pentest-lab --tools nmap,burpsuite", "docker-backed"),
        ("batcave grant pentest-lab net",                     "grant capability"),
        ("batcave grant pentest-lab raw display",             "multiple at once"),
        ("batcave enter pentest-lab",                         "attach to shell"),
        ("batcave seal pentest-lab",                          "persistent -> ephemeral"),
        ("batcave destroy pentest-lab",                       "secure wipe"),
    ];
    for (cmd, comment) in lines {
        font::draw_str(fb, sw, inner_x, ky, cmd, CYAN, BG);
        // Right-aligned "# comment" within the panel width.
        let mut buf = [0u8; 64];
        buf[0] = b'#'; buf[1] = b' ';
        let n = comment.len().min(buf.len() - 2);
        buf[2..2 + n].copy_from_slice(&comment.as_bytes()[..n]);
        let total = 2 + n;
        let total_w = total as u32 * CHAR_W;
        if qw > total_w {
            font::draw_str(fb, sw, inner_x + qw - total_w, ky,
                unsafe { core::str::from_utf8_unchecked(&buf[..total]) }, FAINT, BG);
        }
        ky += 18;
    }

    let _ = h;
}

// ─── Footer ────────────────────────────────────────────────────────

fn draw_footer(x: u32, y: u32, w: u32) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let text_y = y + (FOOTER_H - CHAR_H) / 2;

    let total = active_cave_count();
    let running = running_cave_count();

    let mut cx = x + 16;
    font::draw_str(fb, sw, cx, text_y, "BATCAVES", FAINT, BG); cx += 9 * CHAR_W;
    let mut buf = [0u8; 16];
    let n = format_dec(total, &mut buf);
    font::draw_str(fb, sw, cx, text_y,
        unsafe { core::str::from_utf8_unchecked(&buf[..n]) }, INK, BG);
    cx += (n as u32 + 1) * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, ".", FAINT, BG); cx += 2 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "MAX", FAINT, BG); cx += 4 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "32", INK, BG); cx += 3 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, ".", FAINT, BG); cx += 2 * CHAR_W;
    font::draw_str(fb, sw, cx, text_y, "RUNNING", FAINT, BG); cx += 8 * CHAR_W;
    let mut rb = [0u8; 16];
    let rn = format_dec(running, &mut rb);
    font::draw_str(fb, sw, cx, text_y,
        unsafe { core::str::from_utf8_unchecked(&rb[..rn]) }, INK, BG);
    cx += (rn as u32 + 1) * CHAR_W;
    draw_seg_separator(cx, y, FOOTER_H); cx += 12;

    // 3 colored mini-pills.
    cx = draw_mini_pill(cx, y + (FOOTER_H - 18) / 2, "RUN", running, GREEN, GREEN_DIM);
    cx += 8;
    cx = draw_mini_pill(cx, y + (FOOTER_H - 18) / 2, "STP", 0, DIM_TXT, FAINT);
    cx += 8;
    let _ = draw_mini_pill(cx, y + (FOOTER_H - 18) / 2, "DEL", 0, RED, RED_DIM);

    // Right hint.
    let hint = "up/dn select . shell to manage";
    let hw = hint.len() as u32 * CHAR_W;
    if w > hw + 16 {
        font::draw_str(fb, sw, x + w - 16 - hw, text_y, hint, DIM_TXT, BG);
    }
}

fn draw_mini_pill(x: u32, y: u32, label: &str, n: usize, color: u32, dim: u32) -> u32 {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let pad: u32 = 8;
    let mut buf = [0u8; 8];
    let nn = format_dec(n, &mut buf);
    let label_w = label.len() as u32 * CHAR_W;
    let value_w = nn as u32 * CHAR_W;
    let pill_w = pad + label_w + 8 + value_w + pad;
    draw::draw_border(x, y, pill_w, 18, dim);
    font::draw_str(fb, sw, x + pad, y + 1, label, color, BG);
    font::draw_str(fb, sw, x + pad + label_w + 8, y + 1,
        unsafe { core::str::from_utf8_unchecked(&buf[..nn]) }, INK, BG);
    x + pill_w
}

// ─── helpers ────────────────────────────────────────────────────────

fn active_cave_count() -> usize {
    let mut n = 0;
    cave::list(|c| { if c.state != cave::CaveState::Free { n += 1; } });
    n
}

fn running_cave_count() -> usize {
    let mut n = 0;
    cave::list(|c| { if c.state == cave::CaveState::Running { n += 1; } });
    n
}

fn format_cmd<'a>(prefix: &str, name: &str, out: &'a mut [u8; 64]) -> &'a str {
    let p = prefix.len().min(out.len());
    out[..p].copy_from_slice(&prefix.as_bytes()[..p]);
    let n = name.len().min(out.len() - p);
    out[p..p + n].copy_from_slice(&name.as_bytes()[..n]);
    unsafe { core::str::from_utf8_unchecked(&out[..p + n]) }
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
