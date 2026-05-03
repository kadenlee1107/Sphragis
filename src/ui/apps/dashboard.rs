// Bat_OS — DS · System Dashboard
//
// STUMP #125 — Claude-Design port. Source artifacts in
// `docs/design/apps-ds-nm-sk/` (jsx + spec sheet).
//
// Layout: 2-column grid for SYSTEM + SECURITY (top, 360px tall),
// full-width ARCHITECTURE panel below. Narrow (<512px) collapses
// to single column.

use crate::ui::wm;
use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;
use crate::ui::widgets::{
    self as W, draw_panel, draw_kv_row, draw_tile,
    KV_ROW_H, State,
    BG, INK, MID, DIM_TXT, FAINT, CYAN, CYAN_DIM, GREEN,
};

pub fn render() {
    let r = wm::content_rect();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);
    if r.w < 200 || r.h < 200 { return; }

    let pad: u32 = 16;
    let gutter: u32 = 16;
    let narrow = r.w < 720;

    // ── Top row: SYSTEM (left) + SECURITY (right) ─────────────────
    let top_h: u32 = 360;
    let (sys_x, sys_y, sys_w);
    let (sec_x, sec_y, sec_w);
    if narrow {
        // Stack vertically.
        sys_x = r.x + pad; sys_y = r.y + pad; sys_w = r.w.saturating_sub(pad * 2);
        sec_x = r.x + pad; sec_y = sys_y + 200 + gutter; sec_w = r.w.saturating_sub(pad * 2);
    } else {
        let half = (r.w - pad * 2 - gutter) / 2;
        sys_x = r.x + pad;             sys_y = r.y + pad; sys_w = half;
        sec_x = sys_x + half + gutter; sec_y = r.y + pad; sec_w = half;
    }
    let panel_top_h = if narrow { 200 } else { top_h.min(r.h.saturating_sub(pad * 2 + 200)) };

    // SYSTEM panel — 2x2 tiles.
    let sys_inner = draw_panel(sys_x, sys_y, sys_w, panel_top_h,
        "SYSTEM", Some("ARM64 / APPLE-M4"));
    draw_system_tiles(&sys_inner);

    // SECURITY panel — KV rows.
    let sec_inner = draw_panel(sec_x, sec_y, sec_w, panel_top_h,
        "SECURITY", Some("7 SUBSYSTEMS"));
    draw_security_kvs(&sec_inner);

    // ── Bottom: ARCHITECTURE (full width) ─────────────────────────
    let arch_y = if narrow {
        sec_y + panel_top_h + gutter
    } else {
        r.y + pad + panel_top_h + gutter
    };
    let arch_x = r.x + pad;
    let arch_w = r.w.saturating_sub(pad * 2);
    let arch_h = r.h.saturating_sub(arch_y - r.y + pad);
    if arch_h < 80 { return; }
    let arch_inner = draw_panel(arch_x, arch_y, arch_w, arch_h,
        "ARCHITECTURE", Some("/etc/release"));
    draw_architecture(&arch_inner);
}

fn draw_system_tiles(p: &W::PanelInner) {
    // 2x2 grid with 1px hairline grid lines between tiles.
    let half_w = p.w / 2;
    let half_h = p.h / 2;

    // Hairline cross.
    gpu::fill_rect(p.x + half_w, p.y, 1, p.h, W::HAIR);
    gpu::fill_rect(p.x, p.y + half_h, p.w, 1, W::HAIR);

    let (uptime_min, _) = uptime();
    let mut ut_buf = [0u8; 16];
    let ut_n = format_uptime_short(uptime_min, &mut ut_buf);
    let ut_s = unsafe { core::str::from_utf8_unchecked(&ut_buf[..ut_n]) };

    let (used_frames, total_frames) = crate::kernel::mm::frame::stats();
    let free_kb = (total_frames.saturating_sub(used_frames)) * 4;
    let mut mem_buf = [0u8; 16];
    let mem_n = format_mem(free_kb, &mut mem_buf);
    let mem_s = unsafe { core::str::from_utf8_unchecked(&mem_buf[..mem_n]) };

    let audit_n = crate::security::audit::count();
    let mut audit_buf = [0u8; 16];
    let audit_w = format_dec(audit_n, &mut audit_buf);
    let audit_s = unsafe { core::str::from_utf8_unchecked(&audit_buf[..audit_w]) };

    let net_ok = crate::drivers::virtio::net::is_ready();

    draw_tile(p.x,            p.y,            half_w, half_h,
        "UPTIME", ut_s, "since boot", None);
    draw_tile(p.x + half_w + 1, p.y,          half_w - 1, half_h,
        "FREE MEM", mem_s, "GiB . 4 GiB total", None);
    draw_tile(p.x,            p.y + half_h + 1, half_w, half_h - 1,
        "AUDIT", audit_s, "/ 1024 ring", Some(State::Neutral));
    draw_tile(p.x + half_w + 1, p.y + half_h + 1, half_w - 1, half_h - 1,
        "NETWORK",
        if net_ok { "ONLINE" } else { "OFFLINE" },
        if net_ok { "10.0.2.15" } else { "no link" },
        if net_ok { Some(State::Ok) } else { Some(State::Fail) });
}

fn draw_security_kvs(p: &W::PanelInner) {
    let label_w: u32 = 96;
    let mut y = p.y;
    let net_ok = crate::drivers::virtio::net::is_ready();
    let mode = crate::net::tls_pinning::current_mode();
    let _ = (mode, net_ok); // referenced if we expand the rows later

    draw_kv_row(p.x, y,            label_w, "ENCRYPT",  "AES-256-CTR",                     State::Neutral, false); y += KV_ROW_H;
    draw_kv_row(p.x, y,            label_w, "HASH",     "SHA-256",                         State::Neutral, false); y += KV_ROW_H;
    draw_kv_row(p.x, y,            label_w, "KDF",      "16 ROUNDS . pre-Argon2id",        State::Plan,    false); y += KV_ROW_H;
    draw_kv_row(p.x, y,            label_w, "FIREWALL", "DENY ALL",                        State::Ok,      true);  y += KV_ROW_H;
    draw_kv_row(p.x, y,            label_w, "AUTH",     "PASSPHRASE",                      State::Neutral, false); y += KV_ROW_H;
    draw_kv_row(p.x, y,            label_w, "CAPS",     "ENFORCED",                        State::Ok,      false); y += KV_ROW_H;
    let mut audit_buf = [0u8; 24];
    let n = format_audit_count(crate::security::audit::count(), &mut audit_buf);
    draw_kv_row(p.x, y,            label_w, "AUDIT",
        unsafe { core::str::from_utf8_unchecked(&audit_buf[..n]) },                        State::Neutral, false);
}

fn draw_architecture(p: &W::PanelInner) {
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let mut y = p.y;
    // Headline: "Bat_OS  v0.5.0-DEV"
    font::draw_str(fb, w, p.x, y, "Bat_OS  ", INK, BG);
    font::draw_str(fb, w, p.x + 8 * 8, y, "v0.5.0-DEV", CYAN, BG);
    y += 18;
    font::draw_str(fb, w, p.x, y, "Bare-metal AArch64 microkernel . zero external deps", MID, BG);
    y += 18;
    font::draw_str(fb, w, p.x, y, "BatCave isolation . BatFS encrypted . audit-everything", MID, BG);
    y += 18;
    font::draw_str(fb, w, p.x, y, "Built 20260502.a3f1c . signed", DIM_TXT, BG);
    y += 18;
    font::draw_str(fb, w, p.x, y, "compiled with rustc 1.81-nightly . target aarch64-unknown-none-softfloat", FAINT, BG);

    // Decorative bat at right edge — 36x24 simplified glyph.
    if p.w > 200 {
        let bat_x = p.x + p.w - 40;
        let bat_y = p.y + (p.h.saturating_sub(24)) / 2;
        draw::draw_bat_mini_full(bat_x as i32, bat_y as i32, CYAN);
        // Subtle dim accent line under it.
        gpu::fill_rect(bat_x, bat_y + 28, 36, 1, CYAN_DIM);
    }
    let _ = GREEN;
}

// ── helpers ───────────────────────────────────────────────────────

fn uptime() -> (u64, u64) {
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let secs = if freq == 0 { 0 } else { count / freq };
    (secs / 60, secs % 60)
}

fn format_uptime_short(mins: u64, out: &mut [u8]) -> usize {
    let days = mins / (60 * 24);
    let hours = (mins / 60) % 24;
    let m = mins % 60;
    let mut p = 0;
    p += format_dec(days as usize, &mut out[p..]);
    out[p] = b'd'; p += 1;
    out[p] = b' '; p += 1;
    if hours > 0 {
        p += format_dec(hours as usize, &mut out[p..]);
        out[p] = b'h'; p += 1;
        out[p] = b' '; p += 1;
    }
    p += format_dec(m as usize, &mut out[p..]);
    out[p] = b'm'; p += 1;
    p
}

/// Format a KiB value as "X.Y" GiB (one decimal place). Raw KiB
/// readouts get unwieldy fast (3.7 GiB free reads as "3709292 KiB"),
/// so we collapse to GiB at the cost of <0.1 GiB precision. Caller
/// pairs this with the unit string "GiB . X GiB total".
fn format_mem(kb: usize, out: &mut [u8]) -> usize {
    let mib = kb / 1024;
    let gib_int = mib / 1024;
    let gib_dec = ((mib * 10) / 1024) % 10; // tenths
    let mut p = format_dec(gib_int, out);
    out[p] = b'.'; p += 1;
    out[p] = b'0' + gib_dec as u8; p += 1;
    p
}

fn format_audit_count(n: usize, out: &mut [u8]) -> usize {
    let mut p = format_dec(n, out);
    let suffix = b" / 1024";
    out[p..p + suffix.len()].copy_from_slice(suffix);
    p += suffix.len();
    p
}

fn format_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}
