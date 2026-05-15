//! Wave 2 desktop. State machine + event loop.

#![allow(dead_code)]

use crate::platform;
use crate::ui::draw;
use crate::ui::gpu;
use crate::ui::launcher::{self, LauncherMode};
use crate::ui::sigma_bitmap::{SIGMA_BITMAP_96, SIGMA_BITMAP_W, SIGMA_BITMAP_H};
use crate::ui::topbar::{self, TopBarHit};
use crate::ui::topbar_config;
use crate::ui::wm;

const BG:        u32 = 0xFF0D0D10;
const WATERMARK: u32 = 0xFF1C1D22;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LockReason { UserRequest }

#[derive(Copy, Clone, PartialEq, Eq)]
enum State { Launcher, Active, Overlay }

static mut OVERLAY_OPEN: bool = false;

fn overlay_open() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(OVERLAY_OPEN)) }
}

fn set_overlay_open(v: bool) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(OVERLAY_OPEN), v) }
}

pub fn init() {
    // Idempotent — safe to call from main.rs at startup AND from run()
    // on entry. Restores badge config from BatFS if
    // /system/desktop/topbar.cfg exists; silently keeps defaults if not.
    topbar_config::load();
}

pub fn run() -> LockReason {
    init();
    loop {
        let state = current_state();
        paint(state);
        gpu::flush(0, 0, gpu::width(), gpu::height());

        match poll_event() {
            Event::Lock    => return LockReason::UserRequest,
            Event::Repaint => continue,
            Event::None    => {
                // Periodic security check: dead man's switch expiry →
                // wipe::execute(). Must run on every idle tick.
                crate::security::periodic_check();
                // Drain virtio-net + NAT forward for caves. Without
                // this, cave network connections stall after the ring
                // fills.
                let _ = crate::net::nat::tick();
                core::hint::spin_loop();
            }
        }
    }
}

/// Diverging entry point for non-main.rs callers of the desktop.
///
/// Body is `loop { let _ = run(); }` — runs the desktop, discards
/// `LockReason::UserRequest` on Lock, re-enters. This means **lock
/// requests are silently dropped on this path** until each caller is
/// upgraded to a real lock/unlock cycle (Task 8 work).
///
/// Current callers (each needs a Task-8 upgrade):
///   * `src/caves/linux/signal.rs:908` — Linux cave abnormal-exit handler
///   * `src/kernel/arch/mod.rs:804`    — arch-level recovery path
///   * `src/kernel/arch/mod.rs:2369`   — arch-level reentry path
pub fn resume() -> ! {
    loop { let _ = run(); }
}

fn current_state() -> State {
    if overlay_open() { return State::Overlay; }
    if wm::count() == 0 { return State::Launcher; }
    State::Active
}

fn paint(state: State) {
    let w = gpu::width();
    let h = gpu::height();

    gpu::fill_rect(0, 0, w, h, BG);

    let glyph_x = (w / 2) as i32 - (SIGMA_BITMAP_W as i32) / 2;
    let glyph_y = (h / 2) as i32 - (SIGMA_BITMAP_H as i32) / 2;
    draw::blit_alpha_bitmap(
        gpu::framebuffer(),
        w, h,
        glyph_x, glyph_y,
        &SIGMA_BITMAP_96,
        SIGMA_BITMAP_W, SIGMA_BITMAP_H,
        WATERMARK,
    );

    match state {
        State::Launcher => launcher::paint(LauncherMode::Canvas),
        State::Active   => { launcher::paint(LauncherMode::Background); wm::paint_all(); }
        State::Overlay  => { wm::paint_all(); launcher::paint(LauncherMode::Overlay); }
    }

    topbar::paint();
    topbar::paint_config_sheet();
    paint_cursor();
}

/// Draws the kernel-managed cursor on top of everything else.
///
/// Cocoa + `virtio-mouse-device` delivers relative motion deltas (EV_REL);
/// the host cursor is hidden under the grab. We accumulate deltas in
/// `tablet::cursor_xy()` and paint a small arrow at that position every
/// frame so the user can see where they're aiming. `virtio-tablet` on
/// Cocoa silently drops EV_ABS, so the host-cursor + absolute path isn't
/// viable here.
fn paint_cursor() {
    let (cx, cy) = crate::drivers::virtio::tablet::cursor_xy();
    if cx < 0 || cy < 0 { return; }
    let cx = cx as u32;
    let cy = cy as u32;

    // 10x10 arrow bitmap. 1 = INK, 2 = HAIRLINE outline, 0 = transparent.
    // Outline keeps the arrow visible against light or dark panels.
    const ARROW: [[u8; 10]; 10] = [
        [1, 2, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 2, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 2, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 2, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 2, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 2, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
        [1, 1, 1, 1, 2, 2, 2, 0, 0, 0],
        [1, 1, 2, 1, 1, 2, 0, 0, 0, 0],
        [2, 2, 0, 2, 1, 1, 2, 0, 0, 0],
    ];

    const INK_HEX:      u32 = 0xFFE5E7EB;
    const HAIRLINE_HEX: u32 = 0xFF2A2A30;

    for dy in 0..10u32 {
        for dx in 0..10u32 {
            let color = match ARROW[dy as usize][dx as usize] {
                1 => INK_HEX,
                2 => HAIRLINE_HEX,
                _ => continue,
            };
            gpu::fill_rect(cx + dx, cy + dy, 1, 1, color);
        }
    }
}

enum Event { None, Repaint, Lock }

fn poll_event() -> Event {
    crate::drivers::virtio::keyboard::poll();
    crate::drivers::virtio::tablet::poll();

    if let Some(pe) = crate::drivers::virtio::tablet::next_pointer_event() {
        return handle_pointer(pe);
    }

    if let Some(c) = platform::serial_getc()
        .or_else(crate::drivers::virtio::keyboard::getc)
        .or_else(crate::drivers::virtio::tablet::getc_key)
    {
        return handle_key(c);
    }

    Event::None
}

fn handle_pointer(pe: crate::drivers::virtio::tablet::PointerEvent) -> Event {
    use crate::drivers::virtio::tablet::PointerEvent;
    match pe {
        PointerEvent::Down(x, y) => {
            // Config sheet absorbs all clicks when open; click outside
            // the badge rows closes the sheet.
            if topbar::config_sheet_open() {
                let repaint = topbar::config_sheet_click(x, y);
                if !repaint {
                    topbar::close_config_sheet();
                }
                return Event::Repaint;
            }

            if (y as u32) < topbar::TOPBAR_H {
                match topbar::hit_test(x, y) {
                    TopBarHit::BrandClick  => { set_overlay_open(true); }
                    TopBarHit::ConfigClick => { topbar::open_config_sheet(); }
                    TopBarHit::LockClick   => return Event::Lock,
                    TopBarHit::None        => {}
                }
                return Event::Repaint;
            }

            if overlay_open() {
                match launcher::hit_test(x, y) {
                    Some(id) => { set_overlay_open(false); wm::open(id, None); }
                    None     => set_overlay_open(false),
                }
                return Event::Repaint;
            }

            if wm::count() == 0 {
                if let Some(id) = launcher::hit_test(x, y) {
                    wm::open(id, None);
                    return Event::Repaint;
                }
                return Event::None;
            }

            // App-first dispatch on body clicks: focus the window, give
            // the app a chance to handle the click, then fall through to
            // the WM drag/close/corner handling.
            match wm::hit_test(x, y) {
                wm::Hit::Body(id) => {
                    wm::focus(id);
                    if let Some(body) = wm::body_rect(id) {
                        if let Some(w) = wm::get(id) {
                            let desc = crate::ui::apps_registry::descriptor(w.app);
                            match (desc.handle_click)(x, y, body) {
                                crate::ui::apps_registry::AppEvent::Consumed => return Event::None,
                                crate::ui::apps_registry::AppEvent::Repaint  => return Event::Repaint,
                                crate::ui::apps_registry::AppEvent::Unhandled => { /* fall through */ }
                            }
                        }
                    }
                    return Event::Repaint;  // focus changed, repaint
                }
                _ => {}
            }

            // Fallback: WM drag/close/corner handling.
            if wm::begin_drag(x, y) { Event::Repaint } else { Event::None }
        }
        PointerEvent::Move(x, y) => {
            if wm::is_dragging() && wm::update_drag(x, y) { Event::Repaint } else { Event::None }
        }
        PointerEvent::Up(_, _) => { wm::end_drag(); Event::Repaint }
    }
}

fn handle_key(c: u8) -> Event {
    // Panic hotkey first — 0x17 (Ctrl+W) calls wipe::execute which
    // halts the SoC via the SEP mailbox on real M4 hardware. Under
    // QEMU the wipe stub returns normally; the early `return
    // Event::None` below catches that case so the spin loop doesn't
    // burn cycles after a simulated wipe. (wipe::execute is currently
    // typed `-> ()`, not `-> !`; the `-> !` retrofit is a Wave-3+
    // security-audit follow-up.)
    //
    // 0x17 (Ctrl+W) is intercepted by check_panic_hotkey above —
    // emergency wipe. Do NOT re-add a "close on Ctrl+W" shortcut.
    // Wave-2 close-window UX lives in the chrome close glyph
    // (Tasks 3/4); there is no keyboard shortcut for close because
    // the panic key has prior claim on Ctrl+W.
    if crate::security::check_panic_hotkey(c) {
        return Event::None;
    }

    // The kernel's keyboard layer translates Ctrl+letter into ASCII
    // control codes (Ctrl+K = 0x0B, Ctrl+L = 0x0C). ⌘ on Mac maps to
    // Ctrl through QEMU's HID forwarding, so the brainstormed ⌘K /
    // ⌘L work as documented on both QEMU and the M4 path.
    //
    // System-priority shortcuts: Ctrl+K (overlay) and Ctrl+L (lock)
    // are never overrideable by an app — security and global flow must
    // always win regardless of which app has focus.
    match c {
        0x0B => {
            // Ctrl+K — close the config sheet (if open) and toggle the
            // launcher overlay. Single state change visible per keystroke,
            // matching macOS Cmd+Space precedent (dismisses modals to open
            // the launcher).
            if topbar::config_sheet_open() { topbar::close_config_sheet(); }
            set_overlay_open(!overlay_open());
            return Event::Repaint;
        }
        0x0C => return Event::Lock,                                    // Ctrl+L — lock screen
        _ => {}
    }

    // App dispatch: if a window is focused, the app sees the key first.
    // Consumed → swallow silently; Repaint → repaint; Unhandled → fall
    // through to the desktop's own shortcut table below.
    if let Some(focused_id) = wm::focused() {
        if let Some(w) = wm::get(focused_id) {
            if wm::body_rect(focused_id).is_some() {
                let desc = crate::ui::apps_registry::descriptor(w.app);
                match (desc.handle_key)(c) {
                    crate::ui::apps_registry::AppEvent::Consumed => return Event::None,
                    crate::ui::apps_registry::AppEvent::Repaint  => return Event::Repaint,
                    crate::ui::apps_registry::AppEvent::Unhandled => { /* fall through */ }
                }
            }
        }
    }

    // Desktop fallback (Ctrl+K / Ctrl+L moved to system-priority above).
    match c {
        0x09 => { wm::cycle_focus(); Event::Repaint }                  // Tab — cycle window focus
        0x04 => {                                                      // Ctrl+D — close focused window
            if let Some(id) = wm::focused() { wm::close(id); }
            Event::Repaint
        }
        b'1'..=b'8' => {
            // 1..8 — open (or refocus) the app at that launcher slot.
            // Wave 2 keyboard-driven launcher: QEMU on Cocoa doesn't
            // deliver virtio-input pointer events, so without a number
            // shortcut the launcher is unreachable from the keyboard.
            // 1..8 maps to APPS slots 0..7 (CAVES, FILES, NET,
            // SECURITY, SHELL, EDITOR, COMMS, AGENT).
            let slot = (c - b'1') as usize;
            let app = crate::ui::apps_registry::APPS[slot].id;
            // If the app already has an open window, focus it; otherwise open.
            let existing = wm::iter().find(|w| w.app == app).map(|w| w.id);
            match existing {
                Some(id) => wm::focus(id),
                None     => { wm::open(app, None); }
            }
            // Dismiss the overlay if it was open so the new/focused
            // window is visible immediately.
            if overlay_open() { set_overlay_open(false); }
            Event::Repaint
        }
        0x1B => {                                                      // Esc — dismiss config sheet or overlay
            if topbar::config_sheet_open() {
                topbar::close_config_sheet();
                return Event::Repaint;
            }
            if overlay_open() {
                set_overlay_open(false);
                Event::Repaint
            } else {
                Event::None
            }
        }
        _ => Event::None,
    }
}
