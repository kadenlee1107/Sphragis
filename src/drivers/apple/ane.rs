#![allow(dead_code)]
// Bat_OS — Apple Neural Engine (ANE) driver (skeleton only)
//
// ANE is Apple's on-SoC AI/ML accelerator. Dedicated matrix/convolution
// engine; massively parallel for inference workloads. Every M-series
// chip has one (16 TOPS on M1, up to 38 TOPS on M4).
//
// Like AGX, ANE is firmware-driven (ASC/RTKit coproc) with its own
// command stream format and a dedicated DART. Real use requires:
//   1. Firmware load (proprietary Apple blob)
//   2. Model compilation — ANE executes MIL (Model Intermediate
//      Language) bytecode which is output by CoreML's compiler. The
//      MIL spec is undocumented; Asahi has partially reverse-engineered
//      the encoding.
//   3. Input/output tensor DART-mapping
//   4. Command submission via RTKit messages
//
// ANE access from a non-Apple OS is genuinely novel research territory
// — Linux has no mainline ANE driver. Asahi has RE notes but no
// production driver yet.
//
// This file reserves the module slot + documents scope. Every operation
// returns `NotImplemented`.
//
// References:
//   * m1n1/proxyclient/m1n1/hw/ane.py
//   * Asahi docs: hw/soc/accelerators.md

use super::asc::Asc;
use super::dart::Dart;
use super::rtkit::Rtkit;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AneState { Idle, Unimplemented }

#[derive(Debug, Copy, Clone)]
pub struct Ane {
    pub base: usize,
    pub asc: Asc,
    pub dart: Dart,
    pub rtkit: Rtkit,
    pub state: AneState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AneError { NotImplemented }

impl Ane {
    pub fn new(base: usize, asc: Asc, dart: Dart) -> Self {
        Ane { base, asc, dart, rtkit: Rtkit::new(asc), state: AneState::Idle }
    }

    pub fn bring_up(&mut self) -> Result<(), AneError> {
        self.state = AneState::Unimplemented;
        Err(AneError::NotImplemented)
    }
}
