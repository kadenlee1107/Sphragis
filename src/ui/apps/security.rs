//! Wave 4 SECURITY — operator panic console.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

extern crate alloc;

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_status_panel, StatusPanel, StatusField,
    paint_activity_log, ActivityEntry,
    paint_big_metric,
    paint_action_strip, action_strip_hit_test, Action,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
};
use crate::ui::wm::WindowRect;
use crate::security::{deadman, integrity_counts, auth, audit, audit_chain, wipe};

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    ConfirmWipe,
}

static mut APP_MODE: AppMode = AppMode::Viewing;
static mut VIEWPORT_START: usize = 0;
static mut CHAIN_STATUS: [u8; 96] = [0; 96];
static mut CHAIN_STATUS_LEN: usize = 0;

fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    let panel_y = body.y + 12;
    let panel_h: u32 = 100;
    let gap: u32 = 12;
    let half_w = (body.w - 28 - gap) / 2;
    let dm_rect   = WindowRect { x: body.x + 14, y: panel_y, w: half_w, h: panel_h };
    let auth_rect = WindowRect { x: body.x + 14 + half_w + gap, y: panel_y, w: half_w, h: panel_h };
    paint_deadman_panel(dm_rect);
    paint_auth_panel(auth_rect);

    let log_y = panel_y + panel_h + 12;
    let log_h = body.h.saturating_sub(log_y - body.y + panel_h + 24 + 50);
    let log_rect = WindowRect { x: body.x + 14, y: log_y, w: body.w - 28, h: log_h };
    paint_audit_block(log_rect);

    let bot_y = log_y + log_h + 12;
    let bot_h: u32 = panel_h;
    let taint_rect = WindowRect { x: body.x + 14, y: bot_y, w: half_w, h: bot_h };
    let integ_rect = WindowRect { x: body.x + 14 + half_w + gap, y: bot_y, w: half_w, h: bot_h };
    paint_taint_panel(taint_rect);
    paint_integrity_panel(integ_rect);

    crate::ui::gpu::fill_rect(body.x + 14, body.y + body.h - 32, body.w - 28, 1, p::HAIRLINE);
    let strip_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    paint_action_strip(strip_rect, &actions());

    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        let modal = ConfirmModal {
            title: "Wipe entire system?",
            body_lines: &[
                "  zero all cave keys",
                "  wipe BatFS",
                "  zero the audit ring",
                "  clear MLS labels + taint records",
                "  halt the kernel",
                "",
                "IRREVERSIBLE.",
            ],
            commit_key: 'W',
        };
        paint_confirm_modal(&modal);
    }
}

fn paint_deadman_panel(rect: WindowRect) {
    let panel = StatusPanel { label: "DEADMAN", header_right: Some("ARMED"), body: &[] };
    paint_status_panel(rect, &panel);

    let secs = deadman::seconds_remaining();
    let mut value_buf = [0u8; 16];
    let mut vn = 0;
    if secs >= 3600 {
        write_dec(&mut value_buf, &mut vn, (secs / 3600) as u32);
        push_bytes(&mut value_buf, &mut vn, b":");
        write_pad2(&mut value_buf, &mut vn, ((secs / 60) % 60) as u32);
    } else {
        write_pad2(&mut value_buf, &mut vn, (secs / 60) as u32);
        push_bytes(&mut value_buf, &mut vn, b":");
        write_pad2(&mut value_buf, &mut vn, (secs % 60) as u32);
    }
    let value = unsafe { core::str::from_utf8_unchecked(&value_buf[..vn]) };

    let mut sub_buf = [0u8; 48];
    let mut sn = 0;
    push_bytes(&mut sub_buf, &mut sn, b"wipe in ");
    write_dec(&mut sub_buf, &mut sn, (secs / 60) as u32);
    push_bytes(&mut sub_buf, &mut sn, b" min");
    let sub = unsafe { core::str::from_utf8_unchecked(&sub_buf[..sn]) };

    let inner = WindowRect { x: rect.x + 10, y: rect.y + 32, w: rect.w - 20, h: rect.h - 42 };
    paint_big_metric(inner, "", value, sub);
}

fn paint_auth_panel(rect: WindowRect) {
    let remaining = auth::attempts_remaining();
    let ok = remaining >= 3;
    let panel = StatusPanel {
        label: "AUTH",
        header_right: Some(if ok { "ok" } else { "low" }),
        body: &[],
    };
    paint_status_panel(rect, &panel);

    let mut value_buf = [0u8; 16];
    let mut vn = 0;
    write_dec(&mut value_buf, &mut vn, remaining as u32);
    push_bytes(&mut value_buf, &mut vn, b" / 5");
    let value = unsafe { core::str::from_utf8_unchecked(&value_buf[..vn]) };

    let inner = WindowRect { x: rect.x + 10, y: rect.y + 32, w: rect.w - 20, h: rect.h - 42 };
    paint_big_metric(inner, "", value, "attempts remaining");
}

fn paint_audit_block(rect: WindowRect) {
    use crate::ui::font;
    use alloc::{format, string::String, vec::Vec};
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x, rect.y, "AUDIT", p::MID, p::BG);

    // Chain status (right-aligned).
    let status_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CHAIN_STATUS_LEN)) };
    let status_default: &[u8] = b"chain: ready \xc2\xb7 press V to verify";
    let (chain_status, status_w) = if status_len == 0 {
        let s = unsafe { core::str::from_utf8_unchecked(status_default) };
        // Middle-dot is 2 bytes (\xc2\xb7) but counts as 1 glyph; approximate display width
        // conservatively as byte-len minus 1 (the extra continuation byte).
        (s, (status_default.len() as u32 - 1) * 8)
    } else {
        let buf = unsafe { &*core::ptr::addr_of!(CHAIN_STATUS) };
        let s = unsafe { core::str::from_utf8_unchecked(&buf[..status_len]) };
        (s, status_len as u32 * 8)
    };
    font::draw_str(fb, screen_w,
        rect.x + rect.w.saturating_sub(status_w + 8),
        rect.y, chain_status, p::MID, p::BG);

    // Build display entries by pulling the last 32 audit entries (oldest..newest)
    // and iterating in reverse for newest-first display.
    let mut tmp: [audit::Entry; 32] = [audit::Entry::empty(); 32];
    let n = audit::recent(&mut tmp);
    let viewport = viewport_start();
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for (row_index, i) in (0..n).rev().enumerate() {
        if row_index >= viewport {
            let e = &tmp[i];
            let ts = format!("{:02}:{:02}:{:02}", e.ts / 3600, (e.ts / 60) % 60, e.ts % 60);
            let kind = String::from(cat_label(e.cat));
            let mlen = (e.mlen as usize).min(e.msg.len());
            let summary = unsafe { core::str::from_utf8_unchecked(&e.msg[..mlen]) };
            entries.push((ts, kind, String::from(summary)));
        }
    }
    let total = audit::count();
    let refs: Vec<ActivityEntry> = entries.iter().map(|(t, k, s)| ActivityEntry {
        timestamp_str: t.as_str(),
        kind: k.as_str(),
        summary: s.as_str(),
    }).collect();
    let log_rect = WindowRect { x: rect.x, y: rect.y + 4, w: rect.w, h: rect.h.saturating_sub(4) };
    paint_activity_log(log_rect, &refs, viewport, total);
}

fn cat_label(cat: u8) -> &'static str {
    match cat {
        1  => "fetch",
        2  => "script",
        3  => "click",
        4  => "nav",
        5  => "form",
        6  => "mode",
        7  => "auth",
        8  => "boot",
        9  => "cave",
        10 => "ai",
        11 => "pipe",
        12 => "sock",
        13 => "shm",
        _  => "?",
    }
}

fn paint_taint_panel(rect: WindowRect) {
    use crate::caves::cave;
    use alloc::string::String;
    let mut tainted: u32 = 0;
    let mut system_or: u32 = 0;
    cave::list(|c| {
        if let Some(id) = cave::find_id(c.name_str()) {
            let t = cave::taint_of(id as u16);
            if t != 0 { tainted += 1; system_or |= t; }
        }
    });

    let mut count_buf = [0u8; 24];
    let mut cn = 0;
    write_dec(&mut count_buf, &mut cn, tainted);
    push_bytes(&mut count_buf, &mut cn, b" caves tainted");
    let count_str_bytes = unsafe { core::str::from_utf8_unchecked(&count_buf[..cn]) };
    let count_str_owned: String = String::from(count_str_bytes);

    let mut or_buf = [0u8; 40];
    let mut on = 0;
    push_bytes(&mut or_buf, &mut on, b"0x");
    for k in 0..8 {
        let nibble = ((system_or >> ((7 - k) * 4)) & 0xF) as u8;
        or_buf[on] = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
        on += 1;
    }
    let labels = taint_labels(system_or);
    if !labels.is_empty() {
        push_bytes(&mut or_buf, &mut on, b" \xc2\xb7 ");
        push_bytes(&mut or_buf, &mut on, labels.as_bytes());
    }
    let or_str_bytes = unsafe { core::str::from_utf8_unchecked(&or_buf[..on]) };
    let or_str_owned: String = String::from(or_str_bytes);

    let body = [
        StatusField { key: "system OR", value: or_str_owned.as_str() },
        StatusField { key: "count",     value: count_str_owned.as_str() },
    ];
    let header_badge: &str = if tainted > 0 { count_str_owned.as_str() } else { "clean" };
    let panel = StatusPanel {
        label: "TAINT",
        header_right: Some(header_badge),
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn taint_labels(bits: u32) -> &'static str {
    match bits {
        0x00000000 => "",
        0x00000001 => "PII",
        0x00000002 => "CRYPTO",
        0x00000004 => "AUDIT",
        0x00000008 => "NETWORK",
        _ if (bits & 0x0F) != 0 => "PII|CRYPTO|AUDIT|NETWORK",
        _ => "",
    }
}

fn paint_integrity_panel(rect: WindowRect) {
    use alloc::string::String;
    let blp  = integrity_counts::blp_denies();
    let biba = integrity_counts::biba_denies();
    let te   = integrity_counts::te_denies();
    let clean = blp == 0 && biba == 0 && te == 0;

    let mut blp_buf = [0u8; 24];
    let mut bn = 0;
    write_dec(&mut blp_buf, &mut bn, blp);
    push_bytes(&mut blp_buf, &mut bn, b" in 24h");
    let blp_owned: String = String::from(unsafe { core::str::from_utf8_unchecked(&blp_buf[..bn]) });

    let mut biba_buf = [0u8; 24];
    let mut bbn = 0;
    write_dec(&mut biba_buf, &mut bbn, biba);
    push_bytes(&mut biba_buf, &mut bbn, b" in 24h");
    let biba_owned: String = String::from(unsafe { core::str::from_utf8_unchecked(&biba_buf[..bbn]) });

    let mut te_buf = [0u8; 24];
    let mut tn = 0;
    write_dec(&mut te_buf, &mut tn, te);
    push_bytes(&mut te_buf, &mut tn, b" in 24h");
    let te_owned: String = String::from(unsafe { core::str::from_utf8_unchecked(&te_buf[..tn]) });

    let body = [
        StatusField { key: "BLP denies",  value: blp_owned.as_str() },
        StatusField { key: "Biba denies", value: biba_owned.as_str() },
        StatusField { key: "TE denies",   value: te_owned.as_str() },
    ];
    let panel = StatusPanel {
        label: "INTEGRITY",
        header_right: Some(if clean { "clean" } else { "denies" }),
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn actions() -> [Action<'static>; 3] {
    [
        Action { hotkey: 'R', label: "Re-arm deadman", enabled: true },
        Action { hotkey: 'V', label: "Verify chain",   enabled: true },
        Action { hotkey: 'W', label: "Wipe NOW",       enabled: true },
    ]
}

pub fn handle_key(c: u8) -> AppEvent {
    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        return handle_key_wipe_modal(c);
    }
    match c {
        0x90 => {
            let v = viewport_start();
            if v > 0 {
                unsafe {
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!(VIEWPORT_START),
                        v.saturating_sub(8),
                    );
                }
            }
            AppEvent::Repaint
        }
        0x91 => {
            let total = audit::count();
            let v = viewport_start();
            if v + 8 < total {
                unsafe {
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!(VIEWPORT_START),
                        v + 8,
                    );
                }
            }
            AppEvent::Repaint
        }
        b'r' | b'R' => {
            deadman::arm(48);
            AppEvent::Repaint
        }
        b'v' | b'V' => {
            let outcome = audit_chain::verify_chain();
            let len = audit::count();
            let root = audit_chain::chain_head();

            let mut buf = [0u8; 96];
            let mut n = 0;
            push_bytes(&mut buf, &mut n, b"chain: ");
            write_dec(&mut buf, &mut n, len as u32);
            push_bytes(&mut buf, &mut n, b" \xc2\xb7 root ");
            // First 4 bytes (8 hex chars) of the chain head.
            for k in 0..4 {
                let byte = root[k];
                let hi = (byte >> 4) & 0xF;
                let lo = byte & 0xF;
                if n < buf.len() {
                    buf[n] = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
                    n += 1;
                }
                if n < buf.len() {
                    buf[n] = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
                    n += 1;
                }
            }
            push_bytes(&mut buf, &mut n, b" \xc2\xb7 ");
            let ok = matches!(outcome, audit_chain::VerifyOutcome::Ok);
            push_bytes(&mut buf, &mut n, if ok { b"verified ok" } else { b"verified FAIL" });
            unsafe {
                let dst = core::ptr::addr_of_mut!(CHAIN_STATUS) as *mut u8;
                core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, n);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(CHAIN_STATUS_LEN), n);
            }
            AppEvent::Repaint
        }
        b'w' | b'W' => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmWipe; }
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

fn handle_key_wipe_modal(c: u8) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'W' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            wipe::execute(wipe::WipeReason::Manual, false);
            // Unreachable on hardware; only reached under QEMU stub.
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
        return AppEvent::Repaint;
    }
    let strip_rect = WindowRect {
        x: body.x + 14,
        y: body.y + body.h - 28,
        w: body.w - 28,
        h: 24,
    };
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions()) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() {
            buf[*n] = b;
            *n += 1;
        }
    }
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 {
        if *n < buf.len() {
            buf[*n] = b'0';
            *n += 1;
        }
        return;
    }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 {
        tmp[t] = b'0' + (v % 10) as u8;
        v /= 10;
        t += 1;
    }
    for j in 0..t {
        if *n < buf.len() {
            buf[*n] = tmp[t - j - 1];
            *n += 1;
        }
    }
}

fn write_pad2(buf: &mut [u8], n: &mut usize, v: u32) {
    if v < 10 {
        push_bytes(buf, n, b"0");
    }
    write_dec(buf, n, v);
}
