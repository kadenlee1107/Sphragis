#![allow(dead_code)]
// Sphragis — Apple System Coprocessor (ASC) mailbox driver
//
// ASC is Apple's generic coprocessor mailbox. It shows up across the SoC
// in front of every peripheral that runs its own firmware:
//
//   * ANS  — NVMe controller → ANS2 firmware
//   * DCP  — Display Controller Processor firmware
//   * SEP  — Secure Enclave Processor (yes, the real SEP)
//   * SMC  — System Management Controller
//   * SIO  — System I/O coprocessor (audio, sensors)
//   * AOP  — Always-On Processor (mic, motion)
//
// All of them speak a 16-byte "ASC message" over a small pair of 64-bit
// mailbox registers (A2I = AP→coprocessor, I2A = coprocessor→AP). The
// higher-level protocol on top (RTKit) carries typed endpoint messages
// inside those 16 bytes.
//
// This module is the LOWEST layer: raw mailbox send/receive + CPU
// start/stop control. Protocol layers (RTKit, per-peripheral commands)
// go on top.
//
// Reference: m1n1/src/asc.c (MIT, used for protocol reference only).

// ─── Register offsets ──────────────────────────────────────────────
//
// These live at two MMIO regions per ASC instance:
//   `asc.base`      — mailbox regs (0x000..~0x840)
//   `asc.cpu_base`  — CPU-control regs (0x44: CPU_CONTROL)
//
// On M4 the CPU-control region is usually `base + 0x44_0000` or a
// separate region given by a second `reg` entry in the ADT. Callers
// pass both bases explicitly.

const CPU_CONTROL:       usize = 0x44;
const CPU_CONTROL_START: u32 = 0x10;

// Mailbox (A = Application Processor, I = Internal coprocessor).
// A2I = we send, coprocessor receives.  I2A = coprocessor sends, we rx.
const MBOX_A2I_CONTROL: usize = 0x110;
const MBOX_I2A_CONTROL: usize = 0x114;
const MBOX_A2I_SEND0:   usize = 0x800;
const MBOX_A2I_SEND1:   usize = 0x808;
const MBOX_A2I_RECV0:   usize = 0x810; // diagnostic: the slot the coproc last consumed
const MBOX_A2I_RECV1:   usize = 0x818;
const MBOX_I2A_SEND0:   usize = 0x820; // diagnostic
const MBOX_I2A_SEND1:   usize = 0x828;
const MBOX_I2A_RECV0:   usize = 0x830;
const MBOX_I2A_RECV1:   usize = 0x838;

const CONTROL_FULL:  u32 = 1 << 16;
const CONTROL_EMPTY: u32 = 1 << 17;

// ─── Message type ───────────────────────────────────────────────────

/// One ASC mailbox message. 16 bytes total: 8 bytes of "payload" (msg0)
/// + 8 bytes where only the low 32 bits carry data (msg1 is a u32 in
/// the wire format; higher bits reserved).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AscMessage {
    pub msg0: u64,
    pub msg1: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AscError {
    /// MMIO base is zero / not yet resolved from ADT.
    NotReady,
    /// Send path: mailbox stayed FULL past our polling budget.
    SendTimeout,
    /// Recv path: mailbox EMPTY — nothing to receive (non-blocking).
    RxEmpty,
    /// CPU control register never reflected our start command.
    CpuStartTimeout,
}

// ─── MMIO helpers ──────────────────────────────────────────────────

#[inline]
fn read32(base: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + off) as *const u32) }
}
#[inline]
fn write32(base: usize, off: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u32, val); }
}
#[inline]
fn read64(base: usize, off: usize) -> u64 {
    unsafe { core::ptr::read_volatile((base + off) as *const u64) }
}
#[inline]
fn write64(base: usize, off: usize, val: u64) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u64, val); }
}

// ─── Public handle ──────────────────────────────────────────────────

/// Handle to a single ASC instance. `base` is the mailbox MMIO region;
/// `cpu_base` is the CPU-control region (usually base + a big offset
/// but can be a separate region — callers look it up in the ADT).
#[derive(Debug, Clone, Copy)]
pub struct Asc {
    pub base: usize,
    pub cpu_base: usize,
}

impl Asc {
    /// Construct a handle from the two base addresses. Does NOT touch
    /// MMIO — safe to call at any time. Call `ready()` to confirm the
    /// addresses are plausible before any I/O.
    pub const fn new(base: usize, cpu_base: usize) -> Self {
        Asc { base, cpu_base }
    }

    /// True iff both base addresses are non-null + non-low-page.
    pub fn ready(&self) -> bool {
        self.base >= 0x1000 && self.cpu_base >= 0x1000
    }

    // ── CPU control ───────────────────────────────────────────────

    /// Start the coprocessor CPU (sets CPU_CONTROL.START).
    ///
    /// Note: most coprocessors are already started by iBoot / m1n1 by
    /// the time we see them. Calling `start()` on an already-running
    /// ASC is a no-op on the hardware level (the bit just latches to
    /// 1). If the coproc isn't responding, try a `stop()` + `start()`
    /// cycle. But be careful: stopping DCP mid-flight corrupts the
    /// display; stopping SEP is impossible (locked); stopping ANS
    /// loses in-flight NVMe I/O.
    pub fn start(&self) -> Result<(), AscError> {
        if !self.ready() { return Err(AscError::NotReady); }
        let cur = read32(self.cpu_base, CPU_CONTROL);
        write32(self.cpu_base, CPU_CONTROL, cur | CPU_CONTROL_START);
        // Confirm the bit stuck.
        for _ in 0..1000 {
            if read32(self.cpu_base, CPU_CONTROL) & CPU_CONTROL_START != 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(AscError::CpuStartTimeout)
    }

    /// Stop the coprocessor CPU. USE WITH CAUTION (see `start` docs).
    pub fn stop(&self) -> Result<(), AscError> {
        if !self.ready() { return Err(AscError::NotReady); }
        let cur = read32(self.cpu_base, CPU_CONTROL);
        write32(self.cpu_base, CPU_CONTROL, cur & !CPU_CONTROL_START);
        Ok(())
    }

    /// True if the coprocessor CPU is currently marked started.
    pub fn is_running(&self) -> bool {
        if !self.ready() { return false; }
        read32(self.cpu_base, CPU_CONTROL) & CPU_CONTROL_START != 0
    }

    // ── Mailbox: A2I (we → coproc) ────────────────────────────────

    /// True iff we CAN currently send (A2I mailbox is not full).
    pub fn can_send(&self) -> bool {
        if !self.ready() { return false; }
        read32(self.base, MBOX_A2I_CONTROL) & CONTROL_FULL == 0
    }

    /// Send a message to the coprocessor. Spins until the A2I mailbox
    /// reports not-full (up to ~200k iterations ≈ few ms on M4).
    /// Returns `SendTimeout` if the coproc never drained it.
    pub fn send(&self, msg: &AscMessage) -> Result<(), AscError> {
        if !self.ready() { return Err(AscError::NotReady); }
        for _ in 0..200_000 {
            if self.can_send() { break; }
            core::hint::spin_loop();
        }
        if !self.can_send() { return Err(AscError::SendTimeout); }

        write64(self.base, MBOX_A2I_SEND0, msg.msg0);
        // Writing to SEND1 commits the message (hardware latches both
        // SEND0 and SEND1 into the FIFO on SEND1's rising edge). Hence
        // SEND0 must be written BEFORE SEND1.
        write64(self.base, MBOX_A2I_SEND1, msg.msg1 as u64);
        Ok(())
    }

    // ── Mailbox: I2A (coproc → us) ────────────────────────────────

    /// True iff a message is waiting in the I2A mailbox.
    pub fn has_recv(&self) -> bool {
        if !self.ready() { return false; }
        read32(self.base, MBOX_I2A_CONTROL) & CONTROL_EMPTY == 0
    }

    /// Try to receive one message. Non-blocking — returns `RxEmpty` if
    /// the mailbox is empty.
    pub fn try_recv(&self) -> Result<AscMessage, AscError> {
        if !self.ready() { return Err(AscError::NotReady); }
        if !self.has_recv() { return Err(AscError::RxEmpty); }
        // Order matters: RECV0 THEN RECV1. Reading RECV1 commits the
        // dequeue on the hardware side.
        let msg0 = read64(self.base, MBOX_I2A_RECV0);
        let msg1 = read64(self.base, MBOX_I2A_RECV1) as u32;
        Ok(AscMessage { msg0, msg1 })
    }

    /// Blocking receive with a spin budget. Returns `RxEmpty` if no
    /// message arrived within `spin_budget` iterations.
    pub fn recv(&self, spin_budget: u32) -> Result<AscMessage, AscError> {
        for _ in 0..spin_budget {
            if self.has_recv() {
                return self.try_recv();
            }
            core::hint::spin_loop();
        }
        Err(AscError::RxEmpty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_ready_on_zero() {
        let a = Asc::new(0, 0);
        assert!(!a.ready());
        assert_eq!(a.send(&AscMessage::default()), Err(AscError::NotReady));
        assert_eq!(a.try_recv(), Err(AscError::NotReady));
        assert_eq!(a.start(), Err(AscError::NotReady));
    }

    #[test]
    fn msg_layout_16_bytes() {
        // msg0 is u64, msg1 is u32 — C layout puts them back-to-back.
        // The hardware expects exactly this shape.
        assert_eq!(core::mem::size_of::<AscMessage>(), 16);
    }
}
