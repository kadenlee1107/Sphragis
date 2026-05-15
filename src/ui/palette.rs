//! Shared Wave-1/2/3 palette constants.
//!
//! Modules predating this file (wm.rs, topbar.rs, launcher.rs) define
//! their own private palette constants with the same hex values.
//! Refactoring those to import from here is a Wave-4+ cleanup; for
//! now this module is the canonical home for new widget code.

#![allow(dead_code)]

pub const BG:       u32 = 0xFF0D0D10;
pub const PANEL:    u32 = 0xFF18181C;
pub const HAIRLINE: u32 = 0xFF2A2A30;
pub const INK:      u32 = 0xFFE5E7EB;
pub const MID:      u32 = 0xFF6B7280;

/// Disabled-action color. ~50% of MID; used by the action strip when
/// a hotkey is contextually unavailable (e.g. Stop on a stopped cave).
pub const FAINT:    u32 = 0xFF4A4D55;
