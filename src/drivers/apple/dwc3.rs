#![allow(dead_code)]
// Bat_OS — Apple USB controller (Synopsys DesignWare USB3 core)
//
// Apple Silicon ships USB via the Synopsys DWC3 core (NOT a bare XHCI
// controller). DWC3 is dual-mode:
//   * Host mode: the controller behaves like a standard XHCI host,
//     exposing the XHCI register interface; software enumerates and
//     drives USB peripherals (keyboards, thumbdrives, hubs, etc.).
//   * Device mode: the controller is the peripheral side; software
//     implements USB classes (CDC-ACM serial, HID, MSC) to be seen
//     by a host. m1n1 uses this mode to expose its USB proxy.
//
// DWC3 sits behind a DART (dart-usb). All DMA buffers (TRB rings, data
// buffers) must be DART-mapped IOVAs, not raw physical addresses.
//
// This module is the SKELETON for Phase 3.4: register layout, reset +
// mode-selection primitives, TRB structure, handle type. Full host-side
// enumeration (XHCI + URBs) and device-side class drivers are follow-up
// work — gated on real hardware since DWC3 has M4-specific quirks we
// can't verify under HVF.
//
// Reference: m1n1/src/usb_dwc3.c (MIT — protocol/register reference only).

use super::dart::Dart;

// ─── Global / core registers ────────────────────────────────────────
// These live at fixed offsets within DWC3's MMIO region.

pub const GSBUSCFG0:   usize = 0xc100;
pub const GCTL:        usize = 0xc110;
pub const GSTS:        usize = 0xc118;
pub const GSNPSID:     usize = 0xc120; // Synopsys ID ("55330" etc.)
pub const GUID:        usize = 0xc128;
pub const GUSB2PHYCFG: usize = 0xc200;
pub const GUSB3PIPECTL:usize = 0xc2c0;

// GCTL bits
pub const GCTL_PRTCAPDIR_SHIFT: u32 = 12;
pub const GCTL_PRTCAPDIR_MASK:  u32 = 0x3 << 12;
pub const GCTL_PRTCAPDIR_HOST:  u32 = 0x1 << 12;
pub const GCTL_PRTCAPDIR_DEVICE:u32 = 0x2 << 12;
pub const GCTL_CORESOFTRESET:   u32 = 1 << 11;

// ─── Device-mode registers ─────────────────────────────────────────

pub const DCTL:    usize = 0xc704;
pub const DSTS:    usize = 0xc70c;
pub const DEVTEN:  usize = 0xc708;
pub const DGCMD:   usize = 0xc714;
pub const DGCMDPAR:usize = 0xc710;

// DCTL bits
pub const DCTL_RUN_STOP:    u32 = 1 << 31;
pub const DCTL_CSFTRST:     u32 = 1 << 30;
pub const DCTL_INITU1ENA:   u32 = 1 << 10;

pub const DGCMD_CMDACT: u32 = 1 << 10;

// Per-endpoint command registers — there are 2 * NUM_EPS of these.
// DWC3 has up to 32 physical endpoints (16 IN + 16 OUT).
pub const DEPCMD_BASE:    usize = 0xc80c;
pub const DEPCMD_STRIDE:  usize = 0x10;
#[inline] pub const fn depcmd(ep: u8) -> usize {
    DEPCMD_BASE + DEPCMD_STRIDE * ep as usize
}
#[inline] pub const fn depcmdpar0(ep: u8) -> usize {
    DEPCMD_BASE - 0xc + DEPCMD_STRIDE * ep as usize
}

pub const DEPCMD_CMDACT: u32 = 1 << 10;

// ─── TRB (Transfer Request Block) ───────────────────────────────────
// 16 bytes; forms rings. Software pushes TRBs, DWC3 pops them.

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Trb {
    pub bpl: u32,     // Buffer pointer low (IOVA)
    pub bph: u32,     // Buffer pointer high
    pub size: u32,    // Transfer size (bits 23:0) + chain flag (bit 28 etc.)
    pub ctrl: u32,    // Hardware ownership (bit 0) + type + IOC
}

// TRB control bits
pub const TRB_HWO:     u32 = 1 << 0;  // Hardware Owner — software clears, HW sets
pub const TRB_LST:     u32 = 1 << 1;  // Last TRB in sequence
pub const TRB_CHN:     u32 = 1 << 2;  // Chain bit — more TRBs follow
pub const TRB_CSP:     u32 = 1 << 3;  // Continue on short packet
pub const TRB_TYPE_SHIFT: u32 = 4;
pub const TRB_TYPE_MASK:  u32 = 0x3f << 4;
pub const TRB_TYPE_NORMAL: u32 = 1 << 4;
pub const TRB_TYPE_CTRL_SETUP: u32 = 2 << 4;
pub const TRB_TYPE_CTRL_STATUS2: u32 = 3 << 4;
pub const TRB_TYPE_CTRL_STATUS3: u32 = 4 << 4;
pub const TRB_TYPE_CTRL_DATA: u32 = 5 << 4;
pub const TRB_TYPE_ISOC_FIRST: u32 = 6 << 4;
pub const TRB_TYPE_ISOC: u32 = 7 << 4;
pub const TRB_TYPE_LINK: u32 = 8 << 4;
pub const TRB_IOC:     u32 = 1 << 11; // Interrupt on completion

// ─── Handle ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Controller hasn't been configured yet.
    Unconfigured,
    /// Host mode — software drives enumeration via XHCI semantics.
    Host,
    /// Device mode — software implements USB classes.
    Device,
}

#[derive(Debug, Copy, Clone)]
pub struct Dwc3 {
    pub base: usize,
    pub dart: Dart,
    pub mode: Mode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UsbError {
    NotReady,
    ResetTimeout,
    UnknownSnpsId(u32),
    DartErr(super::dart::DartError),
    NotImplemented,
}
impl From<super::dart::DartError> for UsbError {
    fn from(e: super::dart::DartError) -> Self { UsbError::DartErr(e) }
}

#[inline]
fn read32(base: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + off) as *const u32) }
}
#[inline]
fn write32(base: usize, off: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u32, val); }
}

impl Dwc3 {
    pub fn new(base: usize, dart: Dart) -> Self {
        Dwc3 { base, dart, mode: Mode::Unconfigured }
    }

    pub fn ready(&self) -> bool { self.base >= 0x1000 && self.dart.ready() }

    /// Read the Synopsys ID register. On M4 we expect a value matching
    /// the "55330" DWC3 variant. Sanity-check that we're talking to the
    /// right hardware before proceeding with any other access.
    pub fn snps_id(&self) -> Option<u32> {
        if !self.ready() { return None; }
        Some(read32(self.base, GSNPSID))
    }

    /// Issue a CORE SOFT RESET via GCTL.CORESOFTRESET; block for up to
    /// 100k iterations waiting for the bit to self-clear.
    pub fn core_reset(&self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        let cur = read32(self.base, GCTL);
        write32(self.base, GCTL, cur | GCTL_CORESOFTRESET);
        for _ in 0..100_000 {
            if read32(self.base, GCTL) & GCTL_CORESOFTRESET == 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(UsbError::ResetTimeout)
    }

    /// Place the controller in device mode (PRTCAPDIR = 2).
    pub fn set_device_mode(&mut self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        let cur = read32(self.base, GCTL);
        let new = (cur & !GCTL_PRTCAPDIR_MASK) | GCTL_PRTCAPDIR_DEVICE;
        write32(self.base, GCTL, new);
        self.mode = Mode::Device;
        Ok(())
    }

    /// Place the controller in host mode (PRTCAPDIR = 1). After this,
    /// software drives the XHCI register interface at `base + 0x0`.
    pub fn set_host_mode(&mut self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        let cur = read32(self.base, GCTL);
        let new = (cur & !GCTL_PRTCAPDIR_MASK) | GCTL_PRTCAPDIR_HOST;
        write32(self.base, GCTL, new);
        self.mode = Mode::Host;
        Ok(())
    }

    /// Device-mode soft reset (DCTL.CSFTRST).
    pub fn device_soft_reset(&self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        if self.mode != Mode::Device { return Err(UsbError::NotReady); }
        let cur = read32(self.base, DCTL);
        write32(self.base, DCTL, cur | DCTL_CSFTRST);
        for _ in 0..100_000 {
            if read32(self.base, DCTL) & DCTL_CSFTRST == 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(UsbError::ResetTimeout)
    }

    /// Set the DCTL.RUN_STOP bit — starts the USB device (after all
    /// endpoints are configured + TRB ring is primed).
    pub fn device_start(&self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        if self.mode != Mode::Device { return Err(UsbError::NotReady); }
        let cur = read32(self.base, DCTL);
        write32(self.base, DCTL, cur | DCTL_RUN_STOP);
        Ok(())
    }

    /// Device-mode stop (clear DCTL.RUN_STOP).
    pub fn device_stop(&self) -> Result<(), UsbError> {
        if !self.ready() { return Err(UsbError::NotReady); }
        if self.mode != Mode::Device { return Err(UsbError::NotReady); }
        let cur = read32(self.base, DCTL);
        write32(self.base, DCTL, cur & !DCTL_RUN_STOP);
        Ok(())
    }

    /// Bring-up sequence: DART bypass → core reset → set mode. Use
    /// this for quick "is the hardware alive?" smoke-testing. Full
    /// enumeration / class driver is Phase 3.4b.
    pub fn bring_up(&mut self, mode: Mode) -> Result<(), UsbError> {
        self.dart.set_bypass(0)?;
        self.core_reset()?;
        match mode {
            Mode::Host => self.set_host_mode(),
            Mode::Device => self.set_device_mode(),
            Mode::Unconfigured => Err(UsbError::NotReady),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trb_is_16_bytes() {
        assert_eq!(core::mem::size_of::<Trb>(), 16);
    }

    #[test]
    fn mode_transitions() {
        let d = Dart::at(0x1000);
        let mut u = Dwc3::new(0x1000, d);
        assert_eq!(u.mode, Mode::Unconfigured);
    }

    #[test]
    fn depcmd_offsets_monotonic() {
        assert_eq!(depcmd(0), DEPCMD_BASE);
        assert_eq!(depcmd(1), DEPCMD_BASE + DEPCMD_STRIDE);
        assert_eq!(depcmd(2), DEPCMD_BASE + 2 * DEPCMD_STRIDE);
    }
}
