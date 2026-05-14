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

pub fn init() { topbar_config::load(); }

pub fn run() -> LockReason {
    loop {
        let state = current_state();
        paint(state);
        gpu::flush(0, 0, gpu::width(), gpu::height());

        match poll_event() {
            Event::Lock => return LockReason::UserRequest,
            Event::Repaint => continue,
            Event::None => { core::hint::spin_loop(); }
        }
    }
}

/// Resume desktop after a cave exits — diverging so callers in
/// signal.rs / arch/mod.rs don't need to change. Runs the standard
/// desktop loop in a lock/unlock cycle; Task 8 will replace this with
/// the full lock-screen round-trip.
pub fn resume() -> ! {
    loop {
        let _ = run();
        // run() only returns on LockReason::UserRequest.
        // With Task 8 wired, control would go to the lock screen here.
        // Until then, re-enter the desktop immediately.
    }
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
            if (y as u32) < topbar::TOPBAR_H {
                match topbar::hit_test(x, y) {
                    TopBarHit::BrandClick  => { set_overlay_open(true); }
                    TopBarHit::ConfigClick => { /* Task 10 */ }
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

            if wm::begin_drag(x, y) { Event::Repaint } else { Event::None }
        }
        PointerEvent::Move(x, y) => {
            if wm::is_dragging() && wm::update_drag(x, y) { Event::Repaint } else { Event::None }
        }
        PointerEvent::Up(_, _) => { wm::end_drag(); Event::Repaint }
    }
}

fn handle_key(c: u8) -> Event {
    // Full keyboard shortcut table lands in Task 9. Esc handled here
    // so the overlay can be dismissed before then.
    if c == 0x1B && overlay_open() {
        set_overlay_open(false);
        return Event::Repaint;
    }
    Event::None
}
