//! Wave 4 Files Manager. Inspector layout + file viewer.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_state_dot, paint_status_field_list, StatusField,
    paint_action_strip, action_strip_hit_test, Action,
    InspectorLayout,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
    paint_file_preview,
};
use crate::ui::wm::WindowRect;
use crate::fs::batfs;

const NAME_MAX: usize = 64;

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    ConfirmDelete(usize),
}

static mut APP_MODE: AppMode = AppMode::Viewing;
static mut SELECTED_FILE: usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut PREVIEW_BUF:  [u8; 8192] = [0; 8192];
static mut PREVIEW_LEN:  usize = 0;
static mut PREVIEW_VALID_FOR: usize = usize::MAX;

fn selected_file() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECTED_FILE)) }
}
fn set_selected_file(v: usize) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_FILE), v);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_VALID_FOR), usize::MAX);
    }
}
fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}
fn set_viewport_start(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    layout.paint_divider();
    paint_sidebar(layout.sidebar_rect());

    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing             => paint_detail_view(layout.detail_rect()),
        AppMode::ConfirmDelete(idx)  => {
            paint_detail_view(layout.detail_rect());
            paint_delete_modal(*idx);
        }
    }
}

fn paint_sidebar(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let (count, _max) = batfs::ns_stats();
    let mut hdr_buf = [0u8; 24];
    hdr_buf[..7].copy_from_slice(b"FILES (");
    let digits = u32_dec(count as u32, &mut hdr_buf, 7);
    hdr_buf[7 + digits] = b')';
    let hdr_len = 7 + digits + 1;
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_len]) };
    font::draw_str(fb, screen_w, rect.x + 8, rect.y + 6, hdr, p::MID, p::BG);
    gpu::fill_rect(rect.x, rect.y + 24, rect.w, 1, p::HAIRLINE);

    let row_h: u32 = 22;
    let sel = selected_file();
    let mut row_index: usize = 0;

    batfs::ns_list(|name, _size, encrypted| {
        let row_y = rect.y + 28 + (row_index as u32) * row_h;
        if row_y + row_h > rect.y + rect.h { return; }

        let is_sel = row_index == sel;
        if is_sel {
            gpu::fill_rect(rect.x, row_y, rect.w, row_h, p::PANEL);
            font::draw_str(fb, screen_w, rect.x + 4, row_y + 3, ">", p::INK, p::PANEL);
        }
        paint_state_dot(rect.x + 18, row_y + 7, encrypted);
        font::draw_str(
            fb, screen_w, rect.x + 30, row_y + 3,
            name,
            if is_sel { p::INK } else { p::MID },
            if is_sel { p::PANEL } else { p::BG },
        );
        row_index += 1;
    });
}

fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let (count, _max) = batfs::ns_stats();
    if count == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No files. Create one via SHELL or EDITOR.", p::MID, p::BG);
        return;
    }

    let sel = selected_file();
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut size: usize = 0;
    let mut encrypted = false;
    let mut row_index: usize = 0;
    batfs::ns_list(|n, s, e| {
        if row_index == sel {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
            size = s;
            encrypted = e;
        }
        row_index += 1;
    });
    if name_len == 0 { return; }
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, name, p::INK, p::BG);
    let mut meta_buf = [0u8; 64];
    let mut mn = 0;
    let (sz_n, sz_u) = format_size(size, &mut meta_buf[mn..]);
    mn += sz_n;
    push_bytes(&mut meta_buf, &mut mn, sz_u.as_bytes());
    push_bytes(&mut meta_buf, &mut mn, b" \xc2\xb7 ");
    push_bytes(&mut meta_buf, &mut mn, if encrypted { b"encrypted" } else { b"plain" });
    let meta = unsafe { core::str::from_utf8_unchecked(&meta_buf[..mn]) };
    let meta_x = rect.x + rect.w.saturating_sub(meta.len() as u32 * 8 + 14);
    font::draw_str(fb, screen_w, meta_x, rect.y + 8, meta, p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 28, rect.w - 28, 1, p::HAIRLINE);

    let preview_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 36,
        w: rect.w - 28,
        h: rect.h.saturating_sub(80),
    };
    let cached_for = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PREVIEW_VALID_FOR)) };
    if cached_for != sel {
        load_preview(name, encrypted);
    }
    let len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PREVIEW_LEN)) };
    let buf = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(PREVIEW_BUF) as *const u8, len) };
    if len == 0 && encrypted {
        font::draw_str(fb, screen_w, preview_rect.x + 4, preview_rect.y + 4,
                       "encrypted; preview requires cave context", p::MID, p::BG);
    } else {
        paint_file_preview(preview_rect, buf, viewport_start());
    }

    gpu::fill_rect(rect.x + 14, rect.y + rect.h - 32, rect.w - 28, 1, p::HAIRLINE);
    let strip_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + rect.h - 28,
        w: rect.w - 28,
        h: 24,
    };
    let actions = actions_for_file(encrypted);
    paint_action_strip(strip_rect, &actions);
}

fn paint_delete_modal(idx: usize) {
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut row_index: usize = 0;
    batfs::ns_list(|n, _s, _e| {
        if row_index == idx {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
        }
        row_index += 1;
    });
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    let mut title_buf = [0u8; 80];
    let mut tn = 0;
    push_bytes(&mut title_buf, &mut tn, b"Delete ");
    push_bytes(&mut title_buf, &mut tn, name.as_bytes());
    push_bytes(&mut title_buf, &mut tn, b"?");
    let title = unsafe { core::str::from_utf8_unchecked(&title_buf[..tn]) };

    let modal = ConfirmModal {
        title,
        body_lines: &[
            "  remove the file from BatFS",
            "  zero its encrypted blocks",
            "  add a tombstone to the audit chain",
            "",
            "IRREVERSIBLE.",
        ],
        commit_key: 'D',
    };
    paint_confirm_modal(&modal);
}

fn actions_for_file(_encrypted: bool) -> [Action<'static>; 2] {
    [
        Action { hotkey: 'D', label: "Delete", enabled: true },
        Action { hotkey: 'E', label: "Edit",   enabled: true },
    ]
}

fn load_preview(name: &str, _encrypted: bool) {
    let buf_ptr = core::ptr::addr_of_mut!(PREVIEW_BUF) as *mut u8;
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, 8192) };
    let len = batfs::ns_read(name, buf).unwrap_or(0);
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_LEN), len);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_VALID_FOR), selected_file());
    }
}

pub fn handle_key(c: u8) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::ConfirmDelete(idx) => handle_key_delete_modal(c, *idx),
        AppMode::Viewing            => handle_key_viewing(c),
    }
}

fn handle_key_viewing(c: u8) -> AppEvent {
    let (count, _max) = batfs::ns_stats();
    match c {
        0x90 => {
            let v = viewport_start();
            if v > 0 { set_viewport_start(v.saturating_sub(8)); }
            AppEvent::Repaint
        }
        0x91 => {
            let v = viewport_start();
            set_viewport_start(v + 8);
            AppEvent::Repaint
        }
        b'j' | b'J' => {
            let sel = selected_file();
            if sel + 1 < count { set_selected_file(sel + 1); }
            AppEvent::Repaint
        }
        b'k' | b'K' => {
            let sel = selected_file();
            if sel > 0 { set_selected_file(sel - 1); }
            AppEvent::Repaint
        }
        b'd' | b'D' => {
            if count > 0 {
                unsafe {
                    *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmDelete(selected_file());
                }
            }
            AppEvent::Repaint
        }
        b'e' | b'E' => open_selected_in_editor(),
        0x0D        => open_selected_in_editor(),
        _ => AppEvent::Unhandled,
    }
}

fn open_selected_in_editor() -> AppEvent {
    let (count, _) = batfs::ns_stats();
    if count == 0 { return AppEvent::Consumed; }

    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let sel = selected_file();
    let mut row_index: usize = 0;
    batfs::ns_list(|n, _, _| {
        if row_index == sel {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
        }
        row_index += 1;
    });
    if name_len == 0 { return AppEvent::Consumed; }
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    crate::ui::apps::editor::set_pending_file(name);
    let existing = crate::ui::wm::iter()
        .find(|w| w.app == crate::ui::apps_registry::AppId::Editor)
        .map(|w| w.id);
    match existing {
        Some(id) => crate::ui::wm::focus(id),
        None     => { crate::ui::wm::open(crate::ui::apps_registry::AppId::Editor, None); }
    }
    AppEvent::Repaint
}

fn handle_key_delete_modal(c: u8, idx: usize) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'D' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            batfs::ns_list(|n, _s, _e| {
                if row_index == idx {
                    let l = n.len().min(NAME_MAX);
                    name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
                    name_len = l;
                }
                row_index += 1;
            });
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                let _ = batfs::ns_delete(name);
            }
            set_selected_file(0);
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
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing => handle_click_viewing(mx, my, body),
        AppMode::ConfirmDelete(_) => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
    }
}

fn handle_click_viewing(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let sidebar = layout.sidebar_rect();
    let detail = layout.detail_rect();

    if mx >= sidebar.x as i32 && mx < (sidebar.x + sidebar.w) as i32 {
        let row_h: u32 = 22;
        let header_h: u32 = 28;
        if my >= (sidebar.y + header_h) as i32 {
            let row_idx = ((my as u32 - sidebar.y - header_h) / row_h) as usize;
            let (count, _max) = batfs::ns_stats();
            if row_idx < count {
                set_selected_file(row_idx);
                return AppEvent::Repaint;
            }
        }
        return AppEvent::Consumed;
    }

    let strip_rect = WindowRect {
        x: detail.x + 14,
        y: detail.y + detail.h - 28,
        w: detail.w - 28,
        h: 24,
    };
    let actions = actions_for_file(false);
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

fn u32_dec(mut v: u32, buf: &mut [u8], offset: usize) -> usize {
    if v == 0 { buf[offset] = b'0'; return 1; }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while v > 0 { tmp[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    for j in 0..i { buf[offset + j] = tmp[i - j - 1]; }
    i
}

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn format_size(bytes: usize, out: &mut [u8]) -> (usize, &'static str) {
    if bytes < 1024 {
        let n = u32_dec(bytes as u32, out, 0);
        (n, "B")
    } else if bytes < 1024 * 1024 {
        let n = u32_dec((bytes / 1024) as u32, out, 0);
        (n, "K")
    } else {
        let n = u32_dec((bytes / (1024 * 1024)) as u32, out, 0);
        (n, "M")
    }
}
