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
        AppMode::ConfirmDestroy(idx)  => {
            paint_detail_view(layout.detail_rect());

            // Resolve cave name for the modal title.
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            crate::caves::cave::list(|c| {
                if row_index == *idx {
                    let n = (c.name_len as usize).min(NAME_MAX);
                    name_len = n;
                    name_buf[..n].copy_from_slice(&c.name[..n]);
                }
                row_index += 1;
            });
            let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

            // Build modal title "Destroy <name>?"
            let mut title_buf = [0u8; 32];
            let prefix = b"Destroy ";
            let mut tn = 0;
            for &b in prefix { title_buf[tn] = b; tn += 1; }
            for &b in name.as_bytes() {
                if tn < title_buf.len() { title_buf[tn] = b; tn += 1; }
            }
            if tn < title_buf.len() { title_buf[tn] = b'?'; tn += 1; }
            let title = unsafe { core::str::from_utf8_unchecked(&title_buf[..tn]) };

            let modal = ConfirmModal {
                title,
                body_lines: &[
                    "  kill all processes inside the cave",
                    "  zero the cave's encryption keys",
                    "  wipe its BatFS subtree",
                    "  clear MLS labels + taint records",
                    "",
                    "IRREVERSIBLE.",
                ],
                commit_key: 'D',
            };
            paint_confirm_modal(&modal);
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
        AppMode::ConfirmDestroy(_) => {
            // Any click cancels the destroy (spec: "click outside modal → Cancel").
            unsafe {
                *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
            }
            AppEvent::Repaint
        }
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
            let n = (c.name_len as usize).min(NAME_MAX);
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

    // Resolve cave_id for taint + PID lookup (by name, after capturing from list).
    // `None` => render the sentinel ("—") in PID and skip taint_of (defaults to 0).
    let cave_id_opt: Option<u32> = cave::find_id(name).map(|id| id as u32);
    let taint_value = cave_id_opt.map(|id| cave::taint_of(id as u16)).unwrap_or(0);

    // Name header.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, name, p::INK, p::BG);
    // State line.
    let state_str = if is_running { "RUNNING" } else { "STOPPED" };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 28, state_str, p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    // Build status fields.
    let mut pid_buf = [0u8; 16];
    let pid_str = match cave_id_opt {
        Some(id) => {
            let digits = u32_decimal(id, &mut pid_buf, 0);
            unsafe { core::str::from_utf8_unchecked(&pid_buf[..digits]) }
        }
        None => "—",
    };

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
    let actions = actions_for_cave(is_running);
    let strip_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + rect.h - 28,
        w: rect.w - 28,
        h: 24,
    };
    gpu::fill_rect(strip_rect.x, strip_rect.y - 4, strip_rect.w, 1, p::HAIRLINE);
    paint_action_strip(strip_rect, &actions);
}

fn paint_detail_create(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    // Guard: form must be initialized.
    let form_scratch_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_scratch_ptr).as_mut() {
        Some(f) => f,
        None    => return,
    }};

    // Header.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, "New cave", p::INK, p::BG);
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 30,
                   "TAB ADVANCES · SPACE CYCLES · ENTER CREATES", p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    let focused = f.focused_field;
    let fields = build_form_fields(f, false);

    let form_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 56,
        w: rect.w - 28,
        h: rect.h - 100,
    };
    paint_inline_edit_form(form_rect, &fields, focused);

    // Footer hint.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + rect.h - 18,
                   "Enter to Create  ·  Esc to cancel", p::MID, p::BG);
}

fn paint_detail_configure(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = match unsafe { (*form_ptr).as_mut() } {
        Some(f) => f,
        None => return,
    };

    // Header: "Configure <name>"
    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };
    let mut hdr_buf = [0u8; 64];
    let prefix = b"Configure ";
    let mut n = 0;
    for &b in prefix { hdr_buf[n] = b; n += 1; }
    for &b in name.as_bytes() { if n < hdr_buf.len() { hdr_buf[n] = b; n += 1; } }
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..n]) };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, hdr, p::INK, p::BG);
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 30,
                   "TAB ADVANCES · SPACE CYCLES · ENTER APPLIES", p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 50, rect.w - 28, 1, p::HAIRLINE);

    let focused = f.focused_field;
    let fields = build_form_fields(f, true);

    let form_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 56,
        w: rect.w - 28,
        h: rect.h - 100,
    };
    paint_inline_edit_form(form_rect, &fields, focused);

    // Footer hint.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + rect.h - 18,
                   "Enter to Apply  ·  Esc to cancel", p::MID, p::BG);
}

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
        // Determine is_running for the currently-selected cave — same
        // walk paint_detail_view uses, so the painted strip and the
        // hit-test strip stay in sync.
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
        let actions = actions_for_cave(is_running);
        if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions) {
            return handle_key(key as u8);
        }
        return AppEvent::Consumed;
    }

    AppEvent::Consumed
}

fn handle_click_form(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_mut() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};

    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let detail = layout.detail_rect();
    let form_rect = WindowRect {
        x: detail.x + 14,
        y: detail.y + 56,
        w: detail.w - 28,
        h: detail.h - 100,
    };

    let mut focused = f.focused_field;
    {
        let name_ro = matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::Configuring(_));
        let fields = build_form_fields(f, name_ro);
        handle_form_click(&fields, &mut focused, form_rect, mx, my);
    } // `fields`' borrow on `f` ends here.
    f.focused_field = focused;
    AppEvent::Repaint
}

// ── Handle-key helpers ────────────────────────────────────────────

fn handle_key_destroy_modal(c: u8, idx: usize) -> AppEvent {
    // A thin ConfirmModal for key dispatch — title/body don't affect
    // key routing, only commit_key does.
    let modal = ConfirmModal {
        title: "",
        body_lines: &[],
        commit_key: 'D',
    };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            // Resolve cave name by index, then destroy.
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            crate::caves::cave::list(|c| {
                if row_index == idx {
                    let n = (c.name_len as usize).min(NAME_MAX);
                    name_len = n;
                    name_buf[..n].copy_from_slice(&c.name[..n]);
                }
                row_index += 1;
            });
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                // TODO(Wave 4): per spec, if destroy returns Err, stay in
                // ConfirmDestroy mode and surface the error in the modal footer.
                // Wave 3 silently swallows the Err and dismisses the modal.
                let _ = crate::caves::cave::destroy(name);
            }
            // Reset selection to 0 and return to Viewing.
            set_selected_cave(0);
            unsafe {
                *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
            }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe {
                *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
            }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

fn handle_key_form(c: u8) -> AppEvent {
    let form_ptr = core::ptr::addr_of_mut!(FORM);
    let f = unsafe { match (*form_ptr).as_mut() {
        Some(f) => f,
        None    => return AppEvent::Unhandled,
    }};

    let mut focused = f.focused_field;
    let action = {
        let name_ro = matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::Configuring(_));
        let mut fields = build_form_fields(f, name_ro);
        handle_form_key(&mut fields, &mut focused, c)
    }; // `fields`' borrow on `f` ends here.
    f.focused_field = focused;

    match action {
        FormAction::Submit => {
            match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
                AppMode::Creating           => submit_create_form(f),
                AppMode::Configuring(_idx)  => submit_configure_form(f),
                _                           => AppEvent::Unhandled,
            }
        }
        FormAction::Cancel => {
            unsafe {
                *core::ptr::addr_of_mut!(FORM)     = None;
                *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
            }
            AppEvent::Repaint
        }
        FormAction::None => {
            // Auto-derive MOUNT from NAME on every keystroke
            // (MOUNT is readonly; no dirty tracking needed).
            regenerate_mount_from_name(f);
            AppEvent::Repaint
        }
    }
}

fn regenerate_mount_from_name(f: &mut FormScratch) {
    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };
    let prefix = b"/ /home/";
    f.mount_len = 0;
    for &b in prefix {
        if f.mount_len < f.mount_buf.len() {
            f.mount_buf[f.mount_len] = b;
            f.mount_len += 1;
        }
    }
    for &b in name.as_bytes() {
        if f.mount_len < f.mount_buf.len() {
            f.mount_buf[f.mount_len] = b;
            f.mount_len += 1;
        }
    }
}

fn submit_create_form(f: &FormScratch) -> AppEvent {
    use crate::caves::cave::{self, NetMode, Sensitivity, Integrity};
    if f.name_len == 0 { return AppEvent::Repaint; }  // invalid; repaint shows it

    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };

    // 1. cave::create
    let create_res = cave::create(name, false);
    if create_res.is_err() {
        // Wave 3 limitation: errors silently swallowed; user can adjust + retry.
        return AppEvent::Repaint;
    }

    // 2-4. Per-field setters (partial-failure tolerant).
    let _ = cave::set_sensitivity_by_name(name, Sensitivity::from_u8(f.mls_sens_sel as u8));
    let _ = cave::set_integrity_by_name(name, Integrity::from_u8(f.mls_integ_sel as u8));
    let _ = cave::set_net_mode_by_name(name, NetMode::from_u8(f.net_mode_sel as u8));

    // 5. MOUNT — skipped per pre-flight Gap 2 (no set_mount_by_name).
    //    MOUNT is readonly in the form; mount is name-derived at access time.

    // 6. TAINT — only when non-zero; find_id lookup post-create.
    if f.taint != 0 {
        if let Some(cave_id) = cave::find_id(name) {
            cave::set_taint(cave_id as u16, f.taint);
        }
    }

    // 7. Exit Create mode.
    unsafe {
        *core::ptr::addr_of_mut!(FORM)     = None;
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
    }
    AppEvent::Repaint
}

/// Apply form changes to an existing cave. Unlike [`submit_create_form`],
/// this skips `cave::create` (cave already exists) and sets taint
/// unconditionally (clearing to 0 is a valid user action).
fn submit_configure_form(f: &FormScratch) -> AppEvent {
    use crate::caves::cave::{self, NetMode, Sensitivity, Integrity};
    if f.name_len == 0 { return AppEvent::Repaint; }

    let name = unsafe { core::str::from_utf8_unchecked(&f.name_buf[..f.name_len]) };

    let _ = cave::set_sensitivity_by_name(name, Sensitivity::from_u8(f.mls_sens_sel as u8));
    let _ = cave::set_integrity_by_name(name, Integrity::from_u8(f.mls_integ_sel as u8));
    let _ = cave::set_net_mode_by_name(name, NetMode::from_u8(f.net_mode_sel as u8));
    // MOUNT skipped per pre-flight Gap 2 (no set_mount_by_name in Wave 3).
    // TAINT: set unconditionally — user may be clearing to 0, which is meaningful.
    if let Some(cave_id) = cave::find_id(name) {
        cave::set_taint(cave_id as u16, f.taint);
    }

    unsafe {
        *core::ptr::addr_of_mut!(FORM)     = None;
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
    }
    AppEvent::Repaint
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
            let n = (c.name_len as usize).min(NAME_MAX);
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
            let n = (c.name_len as usize).min(NAME_MAX);
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

// ── Action-strip table ────────────────────────────────────────────

/// Build the action-strip entries for a cave in Viewing mode. Used
/// by both `paint_detail_view` and `handle_click_viewing` so the
/// painted strip and the hit-test strip stay in sync.
fn actions_for_cave(is_running: bool) -> [Action<'static>; 4] {
    [
        Action { hotkey: 'E', label: "Enter",     enabled: true },
        Action { hotkey: 'S', label: "Stop",      enabled: is_running },
        Action { hotkey: 'C', label: "Configure", enabled: true },
        Action { hotkey: 'D', label: "Destroy",   enabled: true },
    ]
}

/// Build the form fields for the create / configure flow. MOUNT is
/// readonly per pre-flight Gap 2; the rest are editable in Create
/// mode. `name_readonly` flips NAME's readonly flag for Configure
/// mode (no rename API). Caller passes the FormScratch by mutable
/// reference; the returned array borrows into `f`'s buffers.
fn build_form_fields(f: &mut FormScratch, name_readonly: bool) -> [FormField<'_>; 6] {
    const NET_VALUES:   &[&str] = &["isolated", "routed", "custom"];
    const SENS_VALUES:  &[&str] = &["Unclassified", "Confidential", "Secret", "TopSecret"];
    const INTEG_VALUES: &[&str] = &["Untrusted", "Sandboxed", "SystemTrusted", "HighIntegrity"];

    [
        FormField { key: "NAME",      kind: FieldKind::Text { buf: &mut f.name_buf[..],  len: &mut f.name_len,  max: NAME_MAX  }, readonly: name_readonly },
        FormField { key: "NET MODE",  kind: FieldKind::Enum { values: NET_VALUES,   selected: &mut f.net_mode_sel  },              readonly: false },
        FormField { key: "MLS SENS",  kind: FieldKind::Enum { values: SENS_VALUES,  selected: &mut f.mls_sens_sel  },              readonly: false },
        FormField { key: "MLS INTEG", kind: FieldKind::Enum { values: INTEG_VALUES, selected: &mut f.mls_integ_sel },              readonly: false },
        FormField { key: "MOUNT",     kind: FieldKind::Text { buf: &mut f.mount_buf[..], len: &mut f.mount_len, max: MOUNT_MAX }, readonly: true  },
        FormField { key: "TAINT",     kind: FieldKind::Hex32 { value: &mut f.taint },                                              readonly: false },
    ]
}

/// AUDIT-DRV-C1 (2026-05-15): zero the caves-manager scratch on
/// cave switch. FORM may hold a half-typed new-cave name (which can
/// be sensitive — operator-chosen identifier) along with MLS labels
/// and mount path scratch from the previous cave's edit session.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_CAVE), 0);
        *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing;
        *core::ptr::addr_of_mut!(FORM) = None;
    }
}
