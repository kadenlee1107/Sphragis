#![allow(dead_code)]
// Bat_OS — Display subsystem.
//
// Re-exports display-related bridges. Today: chromium_blit (the /batos/fb0
// shared-memory → virtio-gpu scanout kthread). Future: Wayland-like
// compositor glue, Apple DCP wiring.

pub mod chromium_blit;
