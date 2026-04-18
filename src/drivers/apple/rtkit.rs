#![allow(dead_code)]
// Bat_OS — RTKit protocol layer (over ASC mailboxes)
//
// RTKit is the typed message framing Apple coprocessors use on top of
// the raw ASC mailbox. Every Apple coprocessor we care about (ANS, DCP,
// SEP, SMC, SIO, AOP) speaks RTKit; only the SERVICE endpoints above
// the management endpoint differ per coproc.
//
// Wire format (inside the 16-byte ASC message):
//   msg0 (u64)  — payload (endpoint-specific)
//     bits 52..=59 = MGMT message type (only for EP 0)
//   msg1 (u32, low byte) — destination endpoint ID
//
// Well-known endpoints:
//   0  MGMT      — handshake, start/stop, power state, epmap
//   1  CRASHLOG  — crash dumps
//   2  SYSLOG    — diagnostic text
//   3  DEBUG     — debugger attach
//   4  IOREPORT  — I/O stats
//   8  OSLOG     — OS-level logging
//   32+         — per-coproc service endpoints (ANS NVMe is EP 32)
//
// Boot handshake:
//   1. Coproc sends HELLO [min_ver..max_ver]
//   2. We reply HELLO_ACK with the version we picked
//   3. Coproc sends EPMAP (1+ messages) declaring its endpoints
//   4. We reply EPMAP_REPLY with DONE bit on the last ack
//   5. We send START_EP for each endpoint we want
//   6. We send AP_PWR_STATE(ON) → coproc acks
//   7. We send IOP_PWR_STATE(ON) → coproc acks
//   8. Coproc is now fully up; service endpoints are usable.
//
// Reference: m1n1/src/rtkit.c (MIT — protocol only).

use super::asc::{Asc, AscError, AscMessage};

// ─── Constants ─────────────────────────────────────────────────────

pub const EP_MGMT:     u8 = 0;
pub const EP_CRASHLOG: u8 = 1;
pub const EP_SYSLOG:   u8 = 2;
pub const EP_DEBUG:    u8 = 3;
pub const EP_IOREPORT: u8 = 4;
pub const EP_OSLOG:    u8 = 8;

// Management message types (live in msg0 bits [59:52]).
const MGMT_TYPE_SHIFT: u32 = 52;
const MGMT_TYPE_MASK:  u64 = 0xFF << 52;

const MGMT_MSG_HELLO:          u8 = 1;
const MGMT_MSG_HELLO_ACK:      u8 = 2;
const MGMT_MSG_START_EP:       u8 = 5;
const MGMT_MSG_IOP_PWR_STATE:  u8 = 6;
const MGMT_MSG_IOP_PWR_ACK:    u8 = 7;
const MGMT_MSG_EPMAP:          u8 = 8;  // coproc → AP
const MGMT_MSG_EPMAP_REPLY:    u8 = 8;  // AP → coproc (same type, different context)
const MGMT_MSG_AP_PWR_STATE:   u8 = 0xb;

// Version range we speak. RTKit has been stable at 11..12 across M1-M4.
const MIN_VERSION: u16 = 11;
const MAX_VERSION: u16 = 12;

// Power states.
pub const PWR_OFF:      u16 = 0x00;
pub const PWR_SLEEP:    u16 = 0x01;
pub const PWR_QUIESCED: u16 = 0x10;
pub const PWR_ON:       u16 = 0x20;
pub const PWR_INIT:     u16 = 0x220;

// EPMAP message bit layout.
const EPMAP_DONE:         u64 = 1 << 51;
const EPMAP_BASE_SHIFT:   u32 = 32;
const EPMAP_BASE_MASK:    u64 = 0x7 << 32;
const EPMAP_BITMAP_MASK:  u64 = 0xFFFF_FFFF;

// START_EP message bit layout.
const START_EP_IDX_SHIFT: u32 = 32;
const START_EP_IDX_MASK:  u64 = 0xFF << 32;
const START_EP_FLAG:      u64 = 1 << 1;

// Timeouts (spin iterations). At ~1 ns/iter a budget of 10M = 10 ms.
const BOOT_SPIN_BUDGET: u32 = 10_000_000;

// ─── State ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Brand new — haven't spoken to the coproc yet.
    Idle,
    /// Sent HELLO_ACK, waiting for EPMAP.
    WaitEpmap,
    /// Have EPMAP, not yet started endpoints.
    HaveEpmap,
    /// All requested endpoints started, power = ON.
    Up,
    /// Protocol error; coproc unusable.
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RtkitError {
    Asc(AscError),
    /// HELLO's version range didn't overlap ours.
    VersionMismatch { coproc_min: u16, coproc_max: u16 },
    /// Expected MGMT message, got something else.
    ProtocolViolation,
    /// Timed out waiting for a specific coproc message.
    Timeout,
    /// Endpoint out of range (0..256).
    BadEndpoint,
    /// Boot already complete / not in expected state for this call.
    BadState,
}
impl From<AscError> for RtkitError {
    fn from(e: AscError) -> Self { RtkitError::Asc(e) }
}

/// Handle to one RTKit-speaking coprocessor. Holds the ASC handle plus
/// a bitmap of endpoints the coproc told us about in the EPMAP phase.
#[derive(Debug, Clone, Copy)]
pub struct Rtkit {
    pub asc: Asc,
    pub state: State,
    /// Bit N set iff the coproc advertised endpoint N. Up to 256 eps.
    pub endpoints: [u32; 8],
    /// Selected RTKit protocol version from the HELLO handshake.
    pub version: u16,
}

impl Rtkit {
    pub const fn new(asc: Asc) -> Self {
        Rtkit {
            asc,
            state: State::Idle,
            endpoints: [0; 8],
            version: 0,
        }
    }

    #[inline]
    pub fn has_endpoint(&self, ep: u8) -> bool {
        let word = (ep as usize) >> 5;
        let bit = (ep as usize) & 0x1f;
        self.endpoints[word] & (1 << bit) != 0
    }

    fn record_endpoints(&mut self, base: u8, bitmap: u32) {
        let word = (base as usize) >> 5;
        if word < self.endpoints.len() {
            self.endpoints[word] |= bitmap;
        }
    }

    // ── Message framing helpers ──────────────────────────────────

    fn build_mgmt(msg_type: u8, payload: u64) -> AscMessage {
        let msg0 = ((msg_type as u64) << MGMT_TYPE_SHIFT) | (payload & !MGMT_TYPE_MASK);
        AscMessage { msg0, msg1: EP_MGMT as u32 }
    }

    fn send_mgmt(&self, msg_type: u8, payload: u64) -> Result<(), RtkitError> {
        let msg = Self::build_mgmt(msg_type, payload);
        self.asc.send(&msg)?;
        Ok(())
    }

    /// Send a raw message on a user endpoint (not management).
    pub fn send_on(&self, ep: u8, payload: u64) -> Result<(), RtkitError> {
        if ep == EP_MGMT { return Err(RtkitError::BadEndpoint); }
        self.asc.send(&AscMessage { msg0: payload, msg1: ep as u32 })?;
        Ok(())
    }

    fn recv_with_budget(&self, budget: u32) -> Result<AscMessage, RtkitError> {
        self.asc.recv(budget).map_err(RtkitError::Asc)
    }

    /// Poll for one inbound message (non-blocking). Service-endpoint
    /// drivers call this in their poll loop; returns (ep, payload) or
    /// None if nothing ready.
    pub fn poll(&self) -> Option<(u8, u64)> {
        let msg = self.asc.try_recv().ok()?;
        let ep = (msg.msg1 & 0xff) as u8;
        Some((ep, msg.msg0))
    }

    // ── Boot sequence ────────────────────────────────────────────

    /// Run the full RTKit boot handshake through to power = ON.
    /// On success `self.state` = Up, `self.endpoints` populated.
    pub fn boot(&mut self) -> Result<(), RtkitError> {
        if self.state != State::Idle { return Err(RtkitError::BadState); }

        // 1. Wait for coproc HELLO.
        let hello = self.recv_with_budget(BOOT_SPIN_BUDGET)?;
        let hello_type = ((hello.msg0 >> MGMT_TYPE_SHIFT) & 0xff) as u8;
        let hello_ep = (hello.msg1 & 0xff) as u8;
        if hello_ep != EP_MGMT || hello_type != MGMT_MSG_HELLO {
            self.state = State::Failed;
            return Err(RtkitError::ProtocolViolation);
        }
        let coproc_min = hello.msg0 as u16;
        let coproc_max = (hello.msg0 >> 16) as u16;
        // Pick the highest common version.
        let chosen = core::cmp::min(coproc_max, MAX_VERSION);
        if chosen < core::cmp::max(coproc_min, MIN_VERSION) {
            self.state = State::Failed;
            return Err(RtkitError::VersionMismatch { coproc_min, coproc_max });
        }
        self.version = chosen;

        // 2. HELLO_ACK with chosen version.
        // Payload: min in [15:0], max in [31:16] (we echo chosen in both).
        let payload = (chosen as u64) | ((chosen as u64) << 16);
        self.send_mgmt(MGMT_MSG_HELLO_ACK, payload)?;
        self.state = State::WaitEpmap;

        // 3. Consume EPMAP message(s) until DONE bit set.
        loop {
            let msg = self.recv_with_budget(BOOT_SPIN_BUDGET)?;
            let ty = ((msg.msg0 >> MGMT_TYPE_SHIFT) & 0xff) as u8;
            let ep = (msg.msg1 & 0xff) as u8;
            if ep != EP_MGMT || ty != MGMT_MSG_EPMAP {
                // Coproc may interleave other MGMT messages we don't
                // care about (e.g. IOP_PWR init-time chatter). Skip.
                continue;
            }
            let base = ((msg.msg0 & EPMAP_BASE_MASK) >> EPMAP_BASE_SHIFT) as u8;
            let bitmap = (msg.msg0 & EPMAP_BITMAP_MASK) as u32;
            self.record_endpoints(base * 32, bitmap);

            // Ack this batch. Mirror DONE bit back.
            let done = msg.msg0 & EPMAP_DONE;
            let reply = done
                | ((base as u64) << EPMAP_BASE_SHIFT)
                | (bitmap as u64);
            self.send_mgmt(MGMT_MSG_EPMAP_REPLY, reply)?;
            if done != 0 { break; }
        }
        self.state = State::HaveEpmap;

        // 4. (Service endpoints to start happen via start_endpoint
        //     after boot(); we don't auto-start them here.)
        // 5. Request AP pwr state ON.
        self.send_mgmt(MGMT_MSG_AP_PWR_STATE, PWR_ON as u64)?;
        // Wait for ACK. The ack type in m1n1 is the same as the req.
        self.wait_mgmt(MGMT_MSG_AP_PWR_STATE)?;

        // 6. Request IOP (coproc) pwr state ON.
        self.send_mgmt(MGMT_MSG_IOP_PWR_STATE, PWR_ON as u64)?;
        self.wait_mgmt(MGMT_MSG_IOP_PWR_ACK)?;

        self.state = State::Up;
        Ok(())
    }

    /// After boot(), tell the coproc to start the specific endpoint
    /// we want to use. Fails if the endpoint wasn't in the EPMAP.
    pub fn start_endpoint(&self, ep: u8) -> Result<(), RtkitError> {
        if self.state != State::HaveEpmap && self.state != State::Up {
            return Err(RtkitError::BadState);
        }
        if !self.has_endpoint(ep) { return Err(RtkitError::BadEndpoint); }
        let payload = ((ep as u64) << START_EP_IDX_SHIFT) | START_EP_FLAG;
        self.send_mgmt(MGMT_MSG_START_EP, payload)?;
        Ok(())
    }

    /// Drain the inbound mailbox until we see a management message of
    /// a specific type. Used for ACKs. Non-MGMT messages are dropped.
    fn wait_mgmt(&self, want_type: u8) -> Result<AscMessage, RtkitError> {
        for _ in 0..BOOT_SPIN_BUDGET {
            if self.asc.has_recv() {
                let m = self.asc.try_recv()?;
                let ty = ((m.msg0 >> MGMT_TYPE_SHIFT) & 0xff) as u8;
                let ep = (m.msg1 & 0xff) as u8;
                if ep == EP_MGMT && ty == want_type {
                    return Ok(m);
                }
                // Not our ACK — keep draining.
            } else {
                core::hint::spin_loop();
            }
        }
        Err(RtkitError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mgmt_build_packs_type_in_bits_52_59() {
        let m = Rtkit::build_mgmt(MGMT_MSG_HELLO_ACK, 0x1234);
        let ty = (m.msg0 >> MGMT_TYPE_SHIFT) & 0xff;
        assert_eq!(ty as u8, MGMT_MSG_HELLO_ACK);
        assert_eq!(m.msg0 & 0xFFFF_FFFF_FFFF, 0x1234);
        assert_eq!(m.msg1, EP_MGMT as u32);
    }

    #[test]
    fn has_endpoint_bitmap_logic() {
        let asc = Asc::new(0x1000, 0x1000);
        let mut rk = Rtkit::new(asc);
        rk.record_endpoints(0, 0b111); // endpoints 0, 1, 2
        assert!(rk.has_endpoint(0));
        assert!(rk.has_endpoint(1));
        assert!(rk.has_endpoint(2));
        assert!(!rk.has_endpoint(3));
        rk.record_endpoints(32, 0b10); // endpoint 32 + 1 = 33
        assert!(rk.has_endpoint(33));
        assert!(!rk.has_endpoint(32));
    }

    #[test]
    fn bad_endpoint_zero_rejected() {
        let asc = Asc::new(0x1000, 0x1000);
        let rk = Rtkit::new(asc);
        assert_eq!(rk.send_on(EP_MGMT, 0), Err(RtkitError::BadEndpoint));
    }
}
