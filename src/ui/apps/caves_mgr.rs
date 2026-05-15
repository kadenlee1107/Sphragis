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
static mut APP_MODE: AppMode = AppMode::Viewing;
static mut FORM: Option<FormScratch> = None;

fn selected_cave() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECTED_CAVE)) }
}
fn set_selected_cave(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_CAVE), v) }
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

pub fn handle_key(_c: u8) -> AppEvent {
    // Wired per-mode in Tasks 10-13. For now return Unhandled so the
    // desktop fallback continues to handle every key.
    AppEvent::Unhandled
}

pub fn handle_click(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Unhandled
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

    // Cave list rendered in Task 10. Pinned "+ new cave" rendered in Task 10.
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

// ── Detail-view paint (stub for now; Task 10 fills in) ───────────

fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    if crate::caves::cave::count() == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No caves yet. Press N to create.", p::MID, p::BG);
        return;
    }

    // Task 10 fills in the cave-detail rendering.
    let _ = (fb, screen_w);
    let _ = rect;
}

fn paint_detail_create(rect: WindowRect)    { let _ = rect; /* Task 11 */ }
fn paint_detail_configure(rect: WindowRect) { let _ = rect; /* Task 12 */ }
