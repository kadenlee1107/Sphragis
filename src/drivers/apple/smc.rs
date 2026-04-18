#![allow(dead_code)]
// Bat_OS — Apple SMC (System Management Controller) driver skeleton
//
// SMC is Apple's "everything else" coprocessor. On M-series Macs it
// handles:
//   * Battery state (charge %, voltage, temperature, wear)
//   * Fan speeds + temp sensors across the SoC and board
//   * Keyboard backlight brightness
//   * Power button + lid sensors
//   * Charger detection + PD negotiation (via a sub-coprocessor)
//   * AC power state / DC input
//   * Ambient light sensor reads (some boards)
//
// Communication: ASC + RTKit (same substrate as ANS/DCP/SIO). SMC
// exposes a key-value database ("SMC keys" — 4-char FourCC like "TB0T"
// for battery temp, "B0FC" for charge) over a specific endpoint.
//
// This skeleton handles:
//   * Coproc bring-up (ASC start + RTKit handshake)
//   * Generic read_key(fourcc) / write_key(fourcc, value) interface
//
// Full SMC keyspace discovery is runtime — each M-series chip/board
// exposes a different set. We can query "keys" (4-byte fourcc 'keys')
// to enumerate, but the semantic meaning of each key is machine-specific.
//
// Reference: m1n1 doesn't have a full SMC driver; linux-asahi has the
// comprehensive one (drivers/platform/apple/smc.c).

use super::asc::Asc;
use super::rtkit::{Rtkit, RtkitError};

/// SMC service endpoint index (typically the first service EP).
pub const SMC_EP: u8 = 32;

/// SMC key (4-char ASCII, big-endian when sent to coproc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(pub [u8; 4]);

impl Key {
    pub const fn new(s: &[u8; 4]) -> Self { Key(*s) }
    pub fn as_u32_be(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }
}

// A few well-known keys (same across M-series chips).
pub const KEY_BAT_CHARGE:   Key = Key(*b"B0FC"); // Battery current charge (%)
pub const KEY_BAT_CAPACITY: Key = Key(*b"B0FM"); // Battery full-charge capacity
pub const KEY_BAT_TEMP:     Key = Key(*b"TB0T"); // Battery temp (°C)
pub const KEY_BAT_VOLTAGE:  Key = Key(*b"B0AV"); // Battery voltage (V)
pub const KEY_POWER_STATE:  Key = Key(*b"AC-W"); // AC wall power present?
pub const KEY_KEY_COUNT:    Key = Key(*b"#KEY"); // Total number of keys

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SmcState { Idle, Ready, Failed }

#[derive(Debug, Copy, Clone)]
pub struct Smc {
    pub asc: Asc,
    pub rtkit: Rtkit,
    pub state: SmcState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SmcError {
    NotReady,
    Rtkit(RtkitError),
    KeyNotFound,
    Timeout,
    NotImplemented,
}
impl From<RtkitError> for SmcError {
    fn from(e: RtkitError) -> Self { SmcError::Rtkit(e) }
}

impl Smc {
    pub fn new(asc: Asc) -> Self {
        Smc { asc, rtkit: Rtkit::new(asc), state: SmcState::Idle }
    }

    pub fn ready(&self) -> bool { self.asc.ready() }

    /// Bring up the SMC coprocessor (ASC start if not already + RTKit
    /// handshake + SMC endpoint start). On success, `read_key`/
    /// `write_key` can be called.
    pub fn bring_up(&mut self) -> Result<(), SmcError> {
        if !self.ready() {
            self.state = SmcState::Failed;
            return Err(SmcError::NotReady);
        }
        if self.rtkit.state == super::rtkit::State::Idle {
            self.rtkit.boot()?;
        }
        if !self.rtkit.has_endpoint(SMC_EP) {
            self.state = SmcState::Failed;
            return Err(SmcError::NotReady);
        }
        self.rtkit.start_endpoint(SMC_EP)?;
        self.state = SmcState::Ready;
        Ok(())
    }

    /// Read a raw SMC key. Writes a request message to the SMC endpoint
    /// and spins for a response. Returns the raw response payload (up
    /// to 8 bytes — SMC values are small scalars).
    ///
    /// NOT YET IMPLEMENTED: requires the SMC-specific message format
    /// inside the RTKit payload. That's documented in linux-asahi's
    /// smc.c (GPL — we'd clean-room from the register/protocol details).
    pub fn read_key(&mut self, _key: Key) -> Result<u64, SmcError> {
        if self.state != SmcState::Ready { return Err(SmcError::NotReady); }
        // Phase 4.4b: implement SMC-specific message-frame format
        // on top of `rtkit.send_on(SMC_EP, payload)`.
        Err(SmcError::NotImplemented)
    }

    pub fn write_key(&mut self, _key: Key, _value: u64) -> Result<(), SmcError> {
        if self.state != SmcState::Ready { return Err(SmcError::NotReady); }
        Err(SmcError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_keys_are_4_bytes() {
        assert_eq!(KEY_BAT_CHARGE.0.len(), 4);
        assert_eq!(KEY_BAT_CHARGE.as_u32_be(), u32::from_be_bytes(*b"B0FC"));
    }

    #[test]
    fn idle_state_on_construct() {
        let a = Asc::new(0x1000, 0x1000);
        let s = Smc::new(a);
        assert_eq!(s.state, SmcState::Idle);
    }
}
