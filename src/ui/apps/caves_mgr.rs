//! Wave 3 Caves Manager. State-machine app composed of the
//! `src/ui/widgets.rs` Wave-3 widget set.
//!
//! See `docs/superpowers/specs/2026-05-14-caves-manager-design.md`.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_state_dot, paint_status_field_list, StatusField,
    paint_action_strip, action_strip_hit_test, Action,
    InspectorLayout,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
    paint_inline_edit_form, handle_form_key, handle_form_click,
    FieldKind, FormField, FormAction,
};
use crate::ui::wm::WindowRect;

// ── App state ────────────────────────────────────────────────────

const NAME_MAX: usize = 16;
const MOUNT_MAX: usize = 64;

/// Per-mode form scratch. Lives across paint calls when the user is
/// typing in CREATE / CONFIGURE; reset on entry to the mode.
struct FormScratch {
    name_buf:        [u8; NAME_MAX],
    name_len:        usize,
    net_mode_sel:    usize,  // 0=Isolated, 1=Routed, 2=Custom
    mls_sens_sel:    usize,  // 0=U, 1=C, 2=S, 3=TS
    mls_integ_sel:   usize,  // 0=Untrusted, 1=Sandboxed, 2=SystemTrusted, 3=HighIntegrity
    mount_buf:       [u8; MOUNT_MAX],
    mount_len:       usize,
    mount_user_dirty: bool,   // false = auto-derived from name
    taint:           u32,
    focused_field:   usize,
}

impl FormScratch {
    fn empty() -> Self {
        Self {
            name_buf: [0; NAME_MAX],
            name_len: 0,
            net_mode_sel: 0,
            mls_sens_sel: 1,    // Confidential
            mls_integ_sel: 1,   // Sandboxed
            mount_buf: [0; MOUNT_MAX],
            mount_len: 0,
            mount_user_dirty: false,
            taint: 0,
            focused_field: 0,
        }
    }
}

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    Creating,
    Configuring(usize),         // index of cave being configured
    ConfirmDestroy(usize),
}

// Static state. Volatile access matches Wave 2 / 3 convention.
static mut SELECTED_CAVE: usize = 0;
// Non-Copy: assign via `unsafe { *core::ptr::addr_of_mut!(APP_MODE) = new_mode; }`
// Do NOT use write_volatile (requires Copy).
static mut APP_MODE: AppMode = AppMode::Viewing;
// Non-Copy: assign via `unsafe { *core::ptr::addr_of_mut!(FORM) = Some(scratch); }`
// Do NOT use write_volatile (requires Copy).
static mut FORM: Option<FormScratch> = None;

fn selected_cave() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECTED_CAVE)) }
}
fn set_selected_cave(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_CAVE), v) }
}

fn mode_is_creating() -> bool {
    matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::Creating)
}

// ── Public API (wired in apps_registry.rs) ───────────────────────

pub fn paint(body: WindowRect) {
    // Background fill.
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    // Layout.
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    layout.paint_divider();
    paint_sidebar(layout.sidebar_rect());

    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing            => paint_detail_view(layout.detail_rect()),
        AppMode::Creating           => paint_detail_create(layout.detail_rect()),
        AppMode::Configuring(_)     => paint_detail_configure(layout.detail_rect()),
        AppMode::ConfirmDestroy(_)  => {
            // Scaffold only — Task 13 replaces this arm with the
            // real modal that resolves the cave name from the index.
            paint_detail_view(layout.detail_rect());
        }
    }
}

pub fn handle_key(c: u8) -> AppEvent {
    use crate::caves::cave;

    // Mode-specific handlers first.
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::ConfirmDestroy(idx) => {
            return handle_key_destroy_modal(c, *idx);
        }
        AppMode::Creating | AppMode::Configuring(_) => {
            return handle_key_form(c);
        }
        AppMode::Viewing => {} // fall through below
    }

    // Viewing-mode keys.
    match c {
        0x90 => {  // Arrow Up
            let sel = selected_cave();
            if sel > 0 { set_selected_cave(sel - 1); }
            AppEvent::Repaint
        }
        0x91 => {  // Arrow Down
            let sel = selected_cave();
            let cnt = cave::count();
            if cnt > 0 && sel + 1 < cnt { set_selected_cave(sel + 1); }
            AppEvent::Repaint
        }
        b'n' | b'N' => {
            enter_create_mode();
            AppEvent::Repaint
        }
        b'e' | b'E' => {
            let mut name_buf = [0u8; NAME_MAX];
            let name_len = cave_name_at_selected(&mut name_buf);
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                let _ = cave::enter(name);
            }
            AppEvent::Repaint
        }
        b's' | b'S' => {
            let mut name_buf = [0u8; NAME_MAX];
            let name_len = cave_name_at_selected(&mut name_buf);
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                let _ = cave::stop(name);
            }
            AppEvent::Repaint
        }
        b'c' | b'C' => {
            enter_configure_mode();
            AppEvent::Repaint
        }
        b'd' | b'D' => {
            let sel = selected_cave();
            unsafe {
                *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmDestroy(sel);
            }
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

// Per-mode dispatch wired in Tasks 10-13.
pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing => handle_click_viewing(mx, my, body),
        AppMode::Creating | AppMode::Configuring(_) => handle_click_form(mx, my, body),
        AppMode::ConfirmDestroy(_) => AppEvent::Consumed, // ignore clicks behind modal
    }
}

// ── Sidebar paint ────────────────────────────────────────────────

fn paint_sidebar(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    // Header
    let count = crate::caves::cave::count() as u32;
    let mut hdr_buf = [0u8; 24];
    hdr_buf[..7].copy_from_slice(b"CAVES (");
    let digits = u32_decimal(count, &mut hdr_buf, 7);
    hdr_buf[7 + digits] = b')';
    let hdr_len = 7 + digits + 1;
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_len]) };
    font::draw_str(fb, screen_w, rect.x + 8, rect.y + 6, hdr, p::MID, p::BG);
    gpu::fill_rect(rect.x, rect.y + 24, rect.w, 1, p::HAIRLINE);

    // List rows.
    let row_h: u32 = 22;
    let sel = selected_cave();
    let creating = mode_is_creating();
    let mut row_index: usize = 0;

    crate::caves::cave::list(|cave| {
        let row_y = rect.y + 28 + (row_index as u32) * row_h;
        if row_y + row_h > rect.y + rect.h { return; }

        let is_sel = !creating && row_index == sel;
        if is_sel {
            gpu::fill_rect(rect.x, row_y, rect.w, row_h, p::PANEL);
            font::draw_str(fb, screen_w, rect.x + 4, row_y + 3, "›", p::INK, p::PANEL);
        }

        paint_state_dot(rect.x + 18, row_y + 7, cave.is_running());
        font::draw_str(
            fb, screen_w, rect.x + 30, row_y + 3,
            cave.name_str(),
            if is_sel { p::INK } else { p::MID },
            if is_sel { p::PANEL } else { p::BG },
        );
        row_index += 1;
    });

    // Pinned "+ new cave" row at the bottom.
    let pin_y = rect.y + rect.h - row_h - 2;
    let pin_sel = creating;
    if pin_sel {
        gpu::fill_rect(rect.x, pin_y, rect.w, row_h, p::PANEL);
        font::draw_str(fb, screen_w, rect.x + 4, pin_y + 3, "›", p::INK, p::PANEL);
    }
    font::draw_str(
        fb, screen_w, rect.x + 18, pin_y + 3,
        "+ new cave",
        if pin_sel { p::INK } else { p::MID },
        if pin_sel { p::PANEL } else { p::BG },
    );
}

// Helper: format a u32 in decimal at `buf[offset..]`. Returns the number of digits written.
fn u32_decimal(mut n: u32, buf: &mut [u8], offset: usize) -> usize {
    if n == 0 { buf[offset] = b'0'; return 1; }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in 0..i {
        buf[offset + j] = tmp[i - j - 1];
    }
    i
}

// ── Detail-view paint ─────────────────────────────────────────────

fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    use crate::caves::cave::{self, Sensitivity, Integrity, NetMode};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    if cave::count() == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No caves yet. Press N to create.", p::MID, p::BG);
        return;
    }

    // Resolve the selected cave by index.
    let sel = selected_cave();
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut is_running = false;
    let mut sensitivity = Sensitivity::Unclassified;
    let mut integrity = Integrity::Untrusted;
    let mut net_mode = NetMode::Isolated;
    let mut row_index: usize = 0;

    cave::list(|c| {
        if row_index == sel {
            let n = c.name_len;
            name_len = n;
            name_buf[..n].copy_from_slice(&c.name[..n]);
            is_running = c.is_running();
            sensitivity = Sensitivity::from_u8(c.sensitivity);
            integrity = Integrity::from_u8(c.integrity);
            net_mode = NetMode::from_u8(c.net_mode);
        }
        row_index += 1;
    });
    if name_len == 0 { return; }

    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    // Resolve cave_id for taint lookup (by name, after capturing from list).
    let cave_id = cave::find_id(name).unwrap_or(usize::MAX);
    let taint_value = if cave_id != usize::MAX {
        cave::taint_of(cave_id as u16)
    } else {
        0
    };

    // Name header.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, name, p::INK, p::BG);
    // State line.
    let state_str = if is_running { "RUNNING" } else { "STOPPED" };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 28, state_str, p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    // Build status fields.
    let mut pid_buf = [0u8; 16];
    let pid_digits = u32_decimal(cave_id as u32, &mut pid_buf, 0);
    let pid_str = unsafe { core::str::from_utf8_unchecked(&pid_buf[..pid_digits]) };

    let mut mls_buf = [0u8; 48];
    let mls_str = format_mls(&mut mls_buf, sensitivity, integrity);

    let mut taint_buf = [0u8; 12];
    let taint_str = format_hex32(&mut taint_buf, taint_value);

    // MOUNT: derived from cave name — no stored field (Wave 4+).
    // Display as "<name>:" per pre-flight Gap 2 decision.
    let mut mount_buf = [0u8; NAME_MAX + 1];
    let mn = name_len.min(NAME_MAX);
    mount_buf[..mn].copy_from_slice(&name_buf[..mn]);
    mount_buf[mn] = b':';
    let mount_str = unsafe { core::str::from_utf8_unchecked(&mount_buf[..mn + 1]) };

    let fields = [
        StatusField { key: "PID",   value: pid_str },
        StatusField { key: "NET",   value: net_mode.as_str() },
        StatusField { key: "MLS",   value: mls_str },
        StatusField { key: "MOUNT", value: mount_str },
        StatusField { key: "TAINT", value: taint_str },
        StatusField { key: "AUDIT", value: "—" },  // Wave 4 hooks audit count
    ];
    let fields_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 60,
        w: rect.w - 28,
        h: rect.h - 110,
    };
    paint_status_field_list(fields_rect, &fields);

    // Action strip.
    let actions = [
        Action { hotkey: 'E', label: "Enter",     enabled: true },
        Action { hotkey: 'S', label: "Stop",      enabled: is_running },
        Action { hotkey: 'C', label: "Configure", enabled: true },
        Action { hotkey: 'D', label: "Destroy",   enabled: true },
    ];
    let strip_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + rect.h - 28,
        w: rect.w - 28,
        h: 24,
    };
    gpu::fill_rect(strip_rect.x, strip_rect.y - 4, strip_rect.w, 1, p::HAIRLINE);
    paint_action_strip(strip_rect, &actions);
}

fn paint_detail_create(rect: WindowRect)    { let _ = rect; /* Task 11 */ }
fn paint_detail_configure(rect: WindowRect) { let _ = rect; /* Task 12 */ }

// ── Handle-click helpers ──────────────────────────────────────────

fn handle_click_viewing(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let sidebar = layout.sidebar_rect();
    let detail = layout.detail_rect();

    // Sidebar click.
    if mx >= sidebar.x as i32 && mx < (sidebar.x + sidebar.w) as i32 {
        let row_h: u32 = 22;
        let header_h: u32 = 28;
        let pin_y = sidebar.y + sidebar.h - row_h - 2;
        if my >= pin_y as i32 && my < (pin_y + row_h) as i32 {
            enter_create_mode();
            return AppEvent::Repaint;
        }
        if my >= (sidebar.y + header_h) as i32 {
            let row_idx = ((my as u32 - sidebar.y - header_h) / row_h) as usize;
            let cnt = crate::caves::cave::count();
            if row_idx < cnt {
                set_selected_cave(row_idx);
                return AppEvent::Repaint;
            }
        }
        return AppEvent::Consumed;
    }

    // Detail click — action strip hit-test.
    if mx >= detail.x as i32 && mx < (detail.x + detail.w) as i32 {
        // Determine is_running for the currently-selected cave.
        let sel = selected_cave();
        let mut is_running = false;
        let mut row_index: usize = 0;
        crate::caves::cave::list(|c| {
            if row_index == sel { is_running = c.is_running(); }
            row_index += 1;
        });

        let strip_rect = WindowRect {
            x: detail.x + 14,
            y: detail.y + detail.h - 28,
            w: detail.w - 28,
            h: 24,
        };
        let actions = [
            Action { hotkey: 'E', label: "Enter",     enabled: true },
            Action { hotkey: 'S', label: "Stop",      enabled: is_running },
            Action { hotkey: 'C', label: "Configure", enabled: true },
            Action { hotkey: 'D', label: "Destroy",   enabled: true },
        ];
        if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions) {
            return handle_key(key as u8);
        }
        return AppEvent::Consumed;
    }

    AppEvent::Consumed
}

fn handle_click_form(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Unhandled
}

// ── Handle-key helpers ────────────────────────────────────────────

fn handle_key_destroy_modal(_c: u8, _idx: usize) -> AppEvent {
    // Task 13 implements destroy commit / cancel.
    AppEvent::Unhandled
}

fn handle_key_form(_c: u8) -> AppEvent {
    // Task 11 / 12 implement form key dispatch.
    AppEvent::Unhandled
}

// ── Mode-transition helpers ───────────────────────────────────────

fn enter_create_mode() {
    let scratch = FormScratch::empty();
    unsafe {
        *core::ptr::addr_of_mut!(FORM) = Some(scratch);
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Creating;
    }
}

fn enter_configure_mode() {
    let sel = selected_cave();
    let mut scratch = FormScratch::empty();
    // Pre-fill from the selected cave.
    let mut row_index: usize = 0;
    crate::caves::cave::list(|c| {
        if row_index == sel {
            let n = c.name_len;
            scratch.name_buf[..n].copy_from_slice(&c.name[..n]);
            scratch.name_len = n;
            scratch.net_mode_sel = c.net_mode as usize;
            scratch.mls_sens_sel = c.sensitivity as usize;
            scratch.mls_integ_sel = c.integrity as usize;
            // taint: read from side-table after list exits (can't call find_id here safely).
        }
        row_index += 1;
    });
    // Taint pre-fill from side-table using the captured name.
    let name = unsafe { core::str::from_utf8_unchecked(&scratch.name_buf[..scratch.name_len]) };
    if let Some(id) = crate::caves::cave::find_id(name) {
        scratch.taint = crate::caves::cave::taint_of(id as u16);
    }
    unsafe {
        *core::ptr::addr_of_mut!(FORM) = Some(scratch);
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Configuring(sel);
    }
}

// ── Cave-iteration helpers ────────────────────────────────────────

/// Fill `buf` with the name of the selected cave. Returns the length
/// (0 if no cave is at the selected index).
fn cave_name_at_selected(buf: &mut [u8; NAME_MAX]) -> usize {
    let sel = selected_cave();
    let mut name_len = 0;
    let mut row_index: usize = 0;
    crate::caves::cave::list(|c| {
        if row_index == sel {
            let n = c.name_len;
            buf[..n].copy_from_slice(&c.name[..n]);
            name_len = n;
        }
        row_index += 1;
    });
    name_len
}

// ── Format helpers ────────────────────────────────────────────────

fn format_mls(
    buf: &mut [u8],
    sens: crate::caves::cave::Sensitivity,
    integ: crate::caves::cave::Integrity,
) -> &str {
    let mut n = 0;
    for &b in b"sens=" { buf[n] = b; n += 1; }
    for &b in sens.as_str().as_bytes() { buf[n] = b; n += 1; }
    for &b in b" integ=" { buf[n] = b; n += 1; }
    for &b in integ_short(integ).as_bytes() { buf[n] = b; n += 1; }
    unsafe { core::str::from_utf8_unchecked(&buf[..n]) }
}

fn integ_short(integ: crate::caves::cave::Integrity) -> &'static str {
    use crate::caves::cave::Integrity;
    match integ {
        Integrity::Untrusted     => "Untrusted",
        Integrity::Sandboxed     => "Sandboxed",
        Integrity::SystemTrusted => "SystemTrusted",
        Integrity::HighIntegrity => "HighIntegrity",
    }
}

fn format_hex32(buf: &mut [u8; 12], value: u32) -> &str {
    buf[0] = b'0';
    buf[1] = b'x';
    for j in 0..8 {
        let nibble = (value >> ((7 - j) * 4)) & 0xF;
        buf[2 + j] = if nibble < 10 { b'0' + nibble as u8 } else { b'A' + (nibble - 10) as u8 };
    }
    unsafe { core::str::from_utf8_unchecked(&buf[..10]) }
}
