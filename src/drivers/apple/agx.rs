#![allow(dead_code)]
// Sphragis — Apple AGX GPU driver (skeleton only — Phase 4 placeholder)
//
// AGX is Apple's proprietary GPU architecture, present across M1-M4.
// It's the single most complex piece of hardware on the SoC:
//
//   * Custom ISA (not ARM, not RDNA, not any public spec). Shaders
//     compile to AGX machine code. Metal and IOKit provide the public
//     API; the ISA + command encoding are reverse-engineered.
//   * Firmware-loaded GPU (several MB of proprietary Apple firmware).
//     The GPU is useless without this firmware loaded correctly.
//   * UAT: "Unified Addressing Table" — a second IOMMU (distinct from
//     DART) that the GPU uses for its own virtual addressing.
//   * Work-queue model: software builds command streams in shared
//     memory; GPU pulls them via an ASC/RTKit coproc interface + its
//     own doorbell registers.
//
// Asahi has a working open-source AGX driver (`apple/drivers/gpu/drm/
// asahi` — Lina Asahi's work) that took ~2 years to build. A full
// Sphragis AGX driver would be a comparable effort, and only makes sense
// once we've shipped the simpler peripherals.
//
// This module defines ONLY the register offsets and handle skeleton.
// Every actual operation returns `NotImplemented`. Its purpose is:
//   1. Reserve the module slot so downstream code can refer to `agx::`.
//   2. Document the scope of what's missing.
//   3. Provide the ASC/RTKit + DART + UAT integration points the real
//      driver will plug into.
//
// References:
//   * Asahi docs: hw/soc/agx.md (overview)
//   * Linux: drivers/gpu/drm/asahi/ (GPL — we'd clean-room the ISA
//     decoders; register layouts are facts, not copyrightable)
//   * m1n1: proxyclient/m1n1/agx/ (RE scripts)

use super::asc::Asc;
use super::dart::Dart;
use super::rtkit::Rtkit;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AgxState {
    Idle,
    Reserved,    // module linked, no hardware touched
    Unimplemented,
}

#[derive(Debug, Copy, Clone)]
pub struct Agx {
    pub base: usize,
    pub asc: Asc,
    pub dart: Dart,
    pub rtkit: Rtkit,
    pub state: AgxState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AgxError {
    NotImplemented,
}

impl Agx {
    pub fn new(base: usize, asc: Asc, dart: Dart) -> Self {
        Agx {
            base, asc, dart,
            rtkit: Rtkit::new(asc),
            state: AgxState::Idle,
        }
    }

    pub fn bring_up(&mut self) -> Result<(), AgxError> {
        // Deliberately unimplemented — see module docs for scope.
        self.state = AgxState::Unimplemented;
        Err(AgxError::NotImplemented)
    }

    /// Submit a command buffer to the GPU. Real implementation would
    /// build an AGX command stream in a UAT-mapped buffer, write the
    /// doorbell, and wait for completion.
    pub fn submit_cmdbuf(&mut self, _iova: u64, _size: usize) -> Result<(), AgxError> {
        Err(AgxError::NotImplemented)
    }
}
