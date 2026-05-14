#![allow(dead_code)]
// Sphragis — Broadcom Wi-Fi (BCM43xx / BCM4378) driver skeleton
//
// Apple Silicon MacBooks ship with a Broadcom Wi-Fi/Bluetooth combo
// chip (variously BCM4378, BCM4377, BCM4387 on M4). It's connected
// over Apple's PCIe block (APCIE) — the PCIe root complex, not the
// standard ECAM/AHB designs seen elsewhere.
//
// Stack layout:
//
//   Application
//   ----------------------
//   bcmfmac  (FullMAC driver, issues Wi-Fi commands as "ioctl"
//             message pairs to the chip firmware)
//   ----------------------
//   msgbuf   (message-buffer protocol on top of a shared memory ring)
//   ----------------------
//   pcie     (transport — Apple's APCIE root complex)
//   ----------------------
//   hardware
//
// This module is the SKELETON. Real Wi-Fi needs:
//   1. APCIE root complex init + enumeration
//   2. Broadcom firmware blob loading (FW blob is non-redistributable
//      Broadcom proprietary binary; Sphragis will need an install-time
//      path to fetch it from macOS WiFi.framework or the Asahi firmware
//      bundle — NOT included in the Sphragis repo)
//   3. Message-buffer protocol implementation (shared rings + doorbells)
//   4. FullMAC command surface (scan, join, tx, rx, encryption key mgmt)
//
// Reference: Linux `brcmfmac` driver + Asahi's `bcmdhd` notes.
// This is the single largest piece of firmware-dependent code in a
// modern Apple Silicon OS. Full implementation is a multi-week effort.

use super::dart::Dart;

// ─── Register offsets (BCM4378 / generic PCIe message-buffer) ──────

/// Chip identification register (always readable).
pub const BCM_CHIPID: usize = 0x0000;
/// Mailbox doorbell — writing 1 pokes the firmware.
pub const BCM_MBOX_DATA:      usize = 0x140; // writes to this raise an IRQ on the chip
pub const BCM_INTR_MAILBOX0:  usize = 0x150; // host-side IRQ cause register

/// Shared-memory protocol "scratchpad" addresses — set by firmware at
/// boot; host reads via BCM_SCRATCH to learn ring-buffer IOVAs.
pub const BCM_SCRATCH: usize = 0x10;

// ─── Known chip IDs ────────────────────────────────────────────────

pub const CHIPID_BCM4377: u32 = 0x4377;
pub const CHIPID_BCM4378: u32 = 0x4378;
pub const CHIPID_BCM4387: u32 = 0x4387;
pub const CHIPID_BCM4389: u32 = 0x4389;

// ─── Handle ─────────────────────────────────────────────────────────

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WifiState {
    Idle,
    PcieLinked,
    FirmwareLoaded,
    Ready,
    Failed,
}

#[derive(Debug, Copy, Clone)]
pub struct BcmWifi {
    pub base: usize,
    pub dart: Dart,
    pub state: WifiState,
    pub chip_id: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WifiError {
    NotReady,
    UnknownChip(u32),
    FirmwareMissing,
    FirmwareLoadFailed,
    DartErr(super::dart::DartError),
    NotImplemented,
}
impl From<super::dart::DartError> for WifiError {
    fn from(e: super::dart::DartError) -> Self { WifiError::DartErr(e) }
}

#[inline]
fn read32(base: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + off) as *const u32) }
}

impl BcmWifi {
    pub fn new(base: usize, dart: Dart) -> Self {
        BcmWifi { base, dart, state: WifiState::Idle, chip_id: 0 }
    }

    pub fn ready(&self) -> bool { self.base >= 0x1000 && self.dart.ready() }

    /// Probe the chip-ID register. Returns the raw u32; the low 16
    /// bits are the Broadcom chip ID (0x4378, etc.).
    pub fn probe_chip_id(&mut self) -> Result<u32, WifiError> {
        if !self.ready() { return Err(WifiError::NotReady); }
        let id = read32(self.base, BCM_CHIPID) & 0xFFFF;
        if !matches!(id, CHIPID_BCM4377 | CHIPID_BCM4378
                         | CHIPID_BCM4387 | CHIPID_BCM4389) {
            return Err(WifiError::UnknownChip(id));
        }
        self.chip_id = id;
        Ok(id)
    }

    /// Bring-up sequence (skeleton):
    ///   1. DART bypass for PCIe DMA.
    ///   2. Probe chip ID.
    ///   3. Load firmware blob (stubbed — Broadcom FW is non-redistributable).
    ///   4. Start chip, exchange protocol-version handshake.
    ///   5. Configure rings, register IRQ handler.
    ///
    /// Steps 3-5 are `NotImplemented` until we have:
    ///   (a) A firmware-loader path (install-time or boot-arg path to the
    ///       proprietary BCM FW blob)
    ///   (b) A full message-buffer protocol implementation
    pub fn bring_up(&mut self) -> Result<(), WifiError> {
        if !self.ready() {
            self.state = WifiState::Failed;
            return Err(WifiError::NotReady);
        }
        self.dart.set_bypass(0)?;
        self.state = WifiState::PcieLinked;

        self.probe_chip_id()?;
        // Steps 3-5: not yet implemented.
        Err(WifiError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_on_construct() {
        let d = Dart::at(0x1000);
        let w = BcmWifi::new(0x1000, d);
        assert_eq!(w.state, WifiState::Idle);
        assert_eq!(w.chip_id, 0);
    }

    #[test]
    fn known_chip_ids_accept() {
        assert!(matches!(CHIPID_BCM4378, 0x4378));
    }
}
