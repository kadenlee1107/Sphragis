//! TOTP — Time-based One-Time Password (RFC 6238).
//!
//! TOTP is HOTP with `counter = floor((unix_time - T0) / period)`.
//! Standard config: T0=0, period=30 seconds, 6 digits.
//!
//! Sphragis does not have a wall clock today (see Concept note
//! "Time Without a Clock"). Until cluster D lands NTP, callers must
//! supply a Unix-epoch time externally — typically from a verified
//! HTTPS Date header or via an attested time service.

#![allow(dead_code)]

use crate::crypto::hotp::{hotp, verify_with_window, HotpAlg, HotpError};

pub const TOTP_PERIOD_DEFAULT: u64 = 30;
pub const TOTP_DIGITS_DEFAULT: u32 = 6;

/// Compute the TOTP code at `unix_time` seconds since epoch.
pub fn totp(alg: HotpAlg, secret: &[u8], unix_time: u64, period: u64,
            digits: u32) -> Result<u32, HotpError> {
    let counter = unix_time / period;
    hotp(alg, secret, counter, digits)
}

/// Verify a presented code at `unix_time`, with `window` periods of
/// tolerance in either direction (typical: 1 = accept the previous,
/// current, and next 30-second window — handles clock drift).
/// Returns `Some(matched_counter)` on success.
pub fn verify(alg: HotpAlg, secret: &[u8], unix_time: u64, period: u64,
              digits: u32, window: u32, presented: u32) -> Option<u64> {
    let counter = unix_time / period;
    let start = counter.saturating_sub(window as u64);
    let span  = 2 * window;
    verify_with_window(alg, secret, start, presented, digits, span)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drifts_advance_codes() {
        let secret = b"12345678901234567890";
        let a = totp(HotpAlg::Sha256, secret, 100, 30, 6).unwrap();
        let b = totp(HotpAlg::Sha256, secret, 130, 30, 6).unwrap();
        let c = totp(HotpAlg::Sha256, secret, 160, 30, 6).unwrap();
        // Three different 30s windows should yield three codes.
        // Collisions are possible but vanishingly rare for distinct counters.
        assert!(!(a == b && b == c), "advancing time must mostly change codes");
    }

    #[test]
    fn verify_within_window() {
        let secret = b"abcdefghijabcdefghij";
        let t = 1_700_000_000u64;
        let code = totp(HotpAlg::Sha256, secret, t, 30, 6).unwrap();
        // Drift of -29 / +29 seconds is well within window=1.
        assert!(verify(HotpAlg::Sha256, secret, t - 29, 30, 6, 1, code).is_some());
        assert!(verify(HotpAlg::Sha256, secret, t + 29, 30, 6, 1, code).is_some());
    }
}
