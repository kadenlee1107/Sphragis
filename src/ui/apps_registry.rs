//! App registry — the static table of apps the launcher can show.
//!
//! Each `AppDescriptor` carries the identity, the label rendered on
//! the launcher tile, the chrome title, a paint callback, and two
//! input-dispatch callbacks (`handle_key`, `handle_click`). The
//! paint callback takes a `WindowRect` (the body region inside the
//! window's chrome) and is responsible for drawing the app's
//! contents into that rect. The input callbacks let apps intercept
//! keyboard and pointer events from the focused window before the
//! desktop's own fallback table runs. Apps are stateful — most of
//! them keep state in their own `static`s; this registry just wires
//! up the draw and input entry points.

#![allow(dead_code)]

use crate::ui::wm::WindowRect;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AppId {
    Caves    = 0,
    Files    = 1,
    Net      = 2,
    Security = 3,
    Shell    = 4,
    Editor   = 5,
    Comms    = 6,
    Agent    = 7,
}

/// Result of an app's input handler. Tri-state so the desktop knows
/// whether to repaint, consume silently, or fall through to its own
/// keyboard/pointer table.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AppEvent {
    /// App handled the event; no repaint needed (e.g. quiet text input).
    Consumed,
    /// App handled the event; caller should repaint the desktop.
    Repaint,
    /// App did not handle the event; caller should run its own fallback
    /// (Tab cycle, ^D close, Esc dismiss, etc.).
    Unhandled,
}

pub struct AppDescriptor {
    pub id: AppId,
    pub label: &'static str,
    pub title: &'static str,
    pub paint: fn(WindowRect),
    pub handle_key: fn(u8) -> AppEvent,
    pub handle_click: fn(mx: i32, my: i32, body_rect: WindowRect) -> AppEvent,
}

fn default_handle_key(_c: u8) -> AppEvent { AppEvent::Unhandled }
fn default_handle_click(_mx: i32, _my: i32, _rect: WindowRect) -> AppEvent { AppEvent::Unhandled }

const _: () = assert!(APPS.len() == 8, "APPS length must match AppId variant count");

pub static APPS: [AppDescriptor; 8] = [
    AppDescriptor { id: AppId::Caves,    label: "CAVES",    title: "CAVES",    paint: paint_caves,    handle_key: crate::ui::apps::caves_mgr::handle_key, handle_click: crate::ui::apps::caves_mgr::handle_click },
    AppDescriptor { id: AppId::Files,    label: "FILES",    title: "FILES",    paint: paint_files,    handle_key: crate::ui::apps::filemanager::handle_key, handle_click: crate::ui::apps::filemanager::handle_click },
    AppDescriptor { id: AppId::Net,      label: "NET",      title: "NET",      paint: paint_net,      handle_key: crate::ui::apps::netmon::handle_key, handle_click: crate::ui::apps::netmon::handle_click },
    AppDescriptor { id: AppId::Security, label: "SECURITY", title: "SECURITY", paint: paint_security, handle_key: crate::ui::apps::security::handle_key, handle_click: crate::ui::apps::security::handle_click },
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Editor,   label: "EDITOR",   title: "EDITOR",   paint: paint_editor,   handle_key: crate::ui::apps::editor::handle_key, handle_click: crate::ui::apps::editor::handle_click },
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms,    handle_key: default_handle_key, handle_click: default_handle_click },
    AppDescriptor { id: AppId::Agent,    label: "AGENT",    title: "AGENT",    paint: paint_agent,    handle_key: default_handle_key, handle_click: default_handle_click },
];

pub fn descriptor(id: AppId) -> &'static AppDescriptor {
    &APPS[id as usize]
}

// ── Paint callbacks ──────────────────────────────────────────────

fn paint_caves(rect: WindowRect)    { crate::ui::apps::caves_mgr::paint(rect); }
fn paint_files(rect: WindowRect)    { crate::ui::apps::filemanager::paint(rect); }
fn paint_net(rect: WindowRect)      { crate::ui::apps::netmon::paint(rect); }
fn paint_security(rect: WindowRect) { crate::ui::apps::security::paint(rect); }
fn paint_shell(rect: WindowRect)    { crate::ui::shell::paint(rect); }
fn paint_editor(rect: WindowRect)   { crate::ui::apps::editor::paint(rect); }
fn paint_comms(rect: WindowRect)    { crate::ui::apps::comms::paint(rect); }

fn paint_agent(rect: WindowRect) {
    use crate::ui::font;
    let msg = "AGENT - coming soon";
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + rect.h / 2;
    let tx = cx.saturating_sub((msg.len() as u32 * 8) / 2);
    let ty = cy.saturating_sub(8);
    font::draw_str(
        crate::ui::gpu::framebuffer(),
        crate::ui::gpu::width(),
        tx, ty,
        msg,
        0xFFE5E7EB,
        0xFF0D0D10,
    );
}
