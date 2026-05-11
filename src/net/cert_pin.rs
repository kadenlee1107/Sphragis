//! Per-host certificate pinning.
//!
//! The X.509 chain validator in `crate::net::x509` already pins the
//! six trust anchors. This module adds a second, finer-grained
//! defense: per-host pins on the leaf certificate's SubjectPublicKeyInfo
//! (SPKI) — the same primitive HPKP (RFC 7469, now-deprecated browser
//! header) and Android's NetworkSecurityConfig use.
//!
//! Why a second layer:
//!
//! - A misissued cert from any of our six anchors is still
//!   chain-valid. Anchor pinning won't catch it. SPKI pinning binds
//!   "host foo.example.com MUST present a leaf with this SPKI hash"
//!   — independent of which CA signed it.
//! - SPKI survives certificate rotation as long as the operator
//!   keeps the same keypair. Recommended: pin two SPKIs at a time
//!   (current + next) so rotation is non-fatal.
//!
//! Anti-foot-gun: pinning policy is **fail-open by default** — if a
//! host has no pins configured, the chain validator's anchor check
//! still applies but the SPKI check is skipped. Pinning a host that
//! you don't actively rotate is how you brick yourself.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::crypto::sha256;

/// Max number of host pins held in the in-kernel table. Sized for
/// the inference host + a handful of other operator-configured
/// services (audit forwarder, time source, update server).
const MAX_HOSTS: usize = 16;
/// Max SPKI pins per host. Keep two: current + next rotation key.
const MAX_PINS_PER_HOST: usize = 4;
/// Maximum hostname length (RFC 1035 max).
const HOST_BUF_LEN: usize = 255;

#[derive(Clone, Copy)]
struct HostEntry {
    host: [u8; HOST_BUF_LEN],
    host_len: u8,
    pin_count: u8,
    pins: [[u8; 32]; MAX_PINS_PER_HOST],
}

impl HostEntry {
    const fn empty() -> Self {
        Self {
            host: [0u8; HOST_BUF_LEN],
            host_len: 0,
            pin_count: 0,
            pins: [[0u8; 32]; MAX_PINS_PER_HOST],
        }
    }
}

static mut TABLE: [HostEntry; MAX_HOSTS] = [HostEntry::empty(); MAX_HOSTS];
static COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub enum PinError {
    HostTooLong,
    TableFull,
    PinTableFull,
    /// Returned by `check_pin` when the host IS pinned and no pin matches.
    /// This is the "active denial" case. Constant-cost — we compare every
    /// pin slot regardless of where (if anywhere) a mismatch was found.
    Mismatch,
}

/// Register one SPKI pin (raw 32-byte SHA-256) for a host. Idempotent:
/// re-adding the same pin is a no-op.
pub fn add_pin(host: &str, spki_sha256: &[u8; 32]) -> Result<(), PinError> {
    if host.len() > HOST_BUF_LEN {
        return Err(PinError::HostTooLong);
    }
    // SAFETY: TABLE is single-threaded today; cluster G will add a
    // proper lock when SMP lands.
    unsafe {
        // Find existing host entry first.
        for i in 0..COUNT.load(Ordering::Relaxed) {
            let entry = &mut TABLE[i];
            if entry.host_len as usize == host.len()
                && &entry.host[..host.len()] == host.as_bytes()
            {
                // Already exists — append or no-op.
                for j in 0..entry.pin_count as usize {
                    if &entry.pins[j] == spki_sha256 {
                        return Ok(());
                    }
                }
                if (entry.pin_count as usize) >= MAX_PINS_PER_HOST {
                    return Err(PinError::PinTableFull);
                }
                entry.pins[entry.pin_count as usize] = *spki_sha256;
                entry.pin_count += 1;
                return Ok(());
            }
        }
        // New host.
        let n = COUNT.load(Ordering::Relaxed);
        if n >= MAX_HOSTS {
            return Err(PinError::TableFull);
        }
        let entry = &mut TABLE[n];
        entry.host[..host.len()].copy_from_slice(host.as_bytes());
        entry.host_len = host.len() as u8;
        entry.pins[0] = *spki_sha256;
        entry.pin_count = 1;
        COUNT.store(n + 1, Ordering::Relaxed);
    }
    Ok(())
}

/// Check whether `presented_spki` matches a registered pin for `host`.
/// Returns:
///
/// - `Ok(true)` if the host has pins AND the presented SPKI matches one.
/// - `Ok(false)` if the host has NO pins registered (fail-open).
/// - `Err(Mismatch)` if the host HAS pins but the SPKI doesn't match any.
///
/// Constant-cost: scans every pin slot regardless of where the match
/// (if any) lies — matches our constant-cost abort discipline.
pub fn check(host: &str, presented_spki: &[u8]) -> Result<bool, PinError> {
    // Compute the SHA-256 of the presented SPKI.
    let presented_hash: [u8; 32] = sha256::hash(presented_spki);

    unsafe {
        for i in 0..COUNT.load(Ordering::Relaxed) {
            let entry = &TABLE[i];
            if entry.host_len as usize != host.len() {
                continue;
            }
            if &entry.host[..host.len()] != host.as_bytes() {
                continue;
            }
            // Host matched. Constant-cost compare across all pin slots.
            let mut any_match = 0u8;
            for j in 0..MAX_PINS_PER_HOST {
                let mut diff = 0u8;
                for k in 0..32 {
                    diff |= entry.pins[j][k] ^ presented_hash[k];
                }
                if (j as u8) < entry.pin_count && diff == 0 {
                    any_match |= 1;
                }
            }
            return if any_match == 0 {
                Err(PinError::Mismatch)
            } else {
                Ok(true)
            };
        }
    }
    // Host not in table — fail-open.
    Ok(false)
}

/// List all configured pins for diagnostics. Returns
/// `(host, [hex-encoded-spki-hashes])`.
pub fn list_pins() -> Vec<(String, Vec<String>)> {
    use alloc::string::ToString;
    let mut out = Vec::new();
    unsafe {
        for i in 0..COUNT.load(Ordering::Relaxed) {
            let entry = &TABLE[i];
            let host = core::str::from_utf8(&entry.host[..entry.host_len as usize])
                .unwrap_or("?").to_string();
            let mut pins = Vec::new();
            for j in 0..entry.pin_count as usize {
                let mut hex = String::with_capacity(64);
                for &b in entry.pins[j].iter() {
                    let lo = b & 0x0f;
                    let hi = b >> 4;
                    hex.push(if hi < 10 { (b'0' + hi) as char }
                             else        { (b'a' + hi - 10) as char });
                    hex.push(if lo < 10 { (b'0' + lo) as char }
                             else        { (b'a' + lo - 10) as char });
                }
                pins.push(hex);
            }
            out.push((host, pins));
        }
    }
    out
}

/// Test-only: clear the pin table.
#[cfg(test)]
pub fn reset() {
    unsafe {
        TABLE = [HostEntry::empty(); MAX_HOSTS];
    }
    COUNT.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_open_when_no_pins() {
        reset();
        let r = check("nopin.example", b"unused");
        assert!(matches!(r, Ok(false)));
    }

    #[test]
    fn match_when_registered() {
        reset();
        let spki = b"fake-spki-bytes-1234567890";
        let hash = sha256::sha256(spki);
        add_pin("example.com", &hash).unwrap();
        assert!(matches!(check("example.com", spki), Ok(true)));
    }

    #[test]
    fn mismatch_rejected() {
        reset();
        let real = b"real-spki";
        let fake = b"fake-spki";
        add_pin("foo.example", &sha256::hash(real)).unwrap();
        assert!(matches!(check("foo.example", fake), Err(PinError::Mismatch)));
    }
}
