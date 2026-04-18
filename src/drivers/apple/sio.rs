#![allow(dead_code)]
// Bat_OS — Apple SIO (System I/O Processor) driver skeleton
//
// SIO is Apple's audio + low-rate I/O coprocessor. It runs the audio
// pipeline (samples → codec → speakers/headphones/mic) via RTKit
// messaging, same ASC/RTKit stack as ANS / DCP.
//
// On M4, SIO handles:
//   * Speaker output (MCA → audio codec)
//   * Microphone input (ADMAC → codec → buffer)
//   * Jack detect / headphone/headset events
//   * Some low-speed sensors (ambient light, temp)
//
// This is a SKELETON. Real audio needs:
//   1. ADMAC DMA engine (Apple's audio DMA controller, separate from DART)
//   2. MCA (Multi-Channel Audio) block setup + codec quirks
//   3. RTKit endpoint handshake for the "audio" service
//   4. Buffer queue management (circular buffer of audio samples in
//      DART-mapped memory; hardware pulls from head, software writes
//      to tail; ring doorbells on new data)
//
// Reference: m1n1/proxyclient/m1n1/hw/admac.py + mca.py (RE scripts).

use super::asc::Asc;
use super::dart::Dart;
use super::rtkit::{Rtkit, RtkitError};

// SIO's MMIO region and DART are resolved from the ADT at boot:
//   /arm-io/sio       — coproc mailbox + CPU control
//   /arm-io/dart-sio  — DART in front of SIO
//
// We expose accessors on soc.rs once we add those paths.

/// SIO service endpoint indices (post-RTKit boot, these are what we
/// send audio commands on). Exact values are firmware-specific and
/// only discoverable via EPMAP at runtime.
pub const SIO_EP_AUDIO_OUT: u8 = 32;  // typical first service endpoint
pub const SIO_EP_AUDIO_IN:  u8 = 33;

/// Audio sample formats that the SIO firmware understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    S16Le,        // 16-bit signed little-endian (most common)
    S24Le,
    S32Le,
    F32Le,        // 32-bit float
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SioState {
    Idle,
    BootFailed,
    Ready,
}

#[derive(Debug, Copy, Clone)]
pub struct Sio {
    pub asc: Asc,
    pub dart: Dart,
    pub rtkit: Rtkit,
    pub state: SioState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SioError {
    NotReady,
    Rtkit(RtkitError),
    AudioEpNotFound,
    NotImplemented,
}
impl From<RtkitError> for SioError {
    fn from(e: RtkitError) -> Self { SioError::Rtkit(e) }
}

impl Sio {
    pub fn new(asc: Asc, dart: Dart) -> Self {
        Sio { asc, dart, rtkit: Rtkit::new(asc), state: SioState::Idle }
    }

    pub fn ready(&self) -> bool { self.asc.ready() && self.dart.ready() }

    /// Bring up the SIO coprocessor: DART bypass + RTKit handshake.
    /// On success, the audio-output endpoint is started. Actual audio
    /// play/record requires ADMAC DMA setup which is Phase 4.1b.
    pub fn bring_up(&mut self) -> Result<(), SioError> {
        if !self.ready() { return Err(SioError::NotReady); }
        self.dart.set_bypass(0).map_err(|_| SioError::NotReady)?;
        self.rtkit.boot()?;

        if !self.rtkit.has_endpoint(SIO_EP_AUDIO_OUT) {
            return Err(SioError::AudioEpNotFound);
        }
        self.rtkit.start_endpoint(SIO_EP_AUDIO_OUT)?;
        self.state = SioState::Ready;
        // Real audio buffer queues + ADMAC setup = Phase 4.1b.
        Err(SioError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_on_construct() {
        let a = Asc::new(0x1000, 0x1000);
        let d = Dart::at(0x1000);
        let s = Sio::new(a, d);
        assert_eq!(s.state, SioState::Idle);
    }
}
