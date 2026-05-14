//! App registry — the static table of apps the launcher can show.
//!
//! Each `AppDescriptor` carries the identity, the label rendered on
//! the launcher tile, the chrome title, and a paint callback. The
//! paint callback takes a `WindowRect` (the body region inside the
//! window's chrome) and is responsible for drawing the app's
//! contents into that rect. Apps are stateful — most of them keep
//! state in their own `static`s; this registry just wires up the
//! draw entry points.

#![allow(dead_code)]

use crate::ui::wm::WindowRect;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AppId {
    Caves,
    Files,
    Net,
    Security,
    Shell,
    Editor,
    Comms,
    Agent,
}

pub struct AppDescriptor {
    pub id: AppId,
    pub label: &'static str,
    pub title: &'static str,
    pub paint: fn(WindowRect),
}

pub static APPS: [AppDescriptor; 8] = [
    AppDescriptor { id: AppId::Caves,    label: "CAVES",    title: "CAVES",    paint: paint_caves },
    AppDescriptor { id: AppId::Files,    label: "FILES",    title: "FILES",    paint: paint_files },
    AppDescriptor { id: AppId::Net,      label: "NET",      title: "NET",      paint: paint_net },
    AppDescriptor { id: AppId::Security, label: "SECURITY", title: "SECURITY", paint: paint_security },
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell },
    AppDescriptor { id: AppId::Editor,   label: "EDITOR",   title: "EDITOR",   paint: paint_editor },
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms },
    AppDescriptor { id: AppId::Agent,    label: "AGENT",    title: "AGENT",    paint: paint_agent },
];

pub fn descriptor(id: AppId) -> &'static AppDescriptor {
    APPS.iter().find(|d| d.id == id).expect("AppId always in APPS")
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
