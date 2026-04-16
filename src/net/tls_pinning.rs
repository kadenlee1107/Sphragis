// Bat_OS — TLS certificate pinning.
//
// NEW-CRYPTO-010 / NET2-001: full X.509 chain validation is a multi-day
// project (ASN.1 DER parser, OID dispatch, signature verifier per algo,
// CA trust store, name-constraints, basic-constraints, EKU, revocation).
// As an interim defense we ship strict SHA-256 pinning: the operator
// embeds the expected leaf cert fingerprint(s) for each host they trust,
// and the TLS handshake aborts on mismatch.
//
// Threat model addressed: an active MITM presenting a different cert
// (self-signed, different real cert, etc.) is rejected. A MITM that
// somehow obtained the genuine cert + private key is NOT addressed —
// that's a private-key compromise, not a TLS issue.
//
// Operator workflow:
//   1. `openssl s_client -connect host:443 < /dev/null \
//        | openssl x509 -outform DER | openssl dgst -sha256 -binary \
//        | xxd -i`
//   2. Paste the 32-byte array into `PINS` below with the matching
//      hostname.
//   3. Rebuild — pin is enforced from next boot.
//
// Behaviour when no pin is configured for a hostname:
//   * `STRICT_MODE = true`: abort the handshake (recommended).
//   * `STRICT_MODE = false`: allow but log "[tls] WARN no pin for HOST".

#![allow(dead_code)]

use crate::drivers::uart;

/// Refuse handshakes to hostnames that don't have a pin entry. Default
/// `false` so dev builds still work; flip to `true` for production.
pub const STRICT_MODE: bool = false;

/// One pin entry. Hostname is matched literally (no wildcards).
pub struct Pin {
    pub host: &'static [u8],
    pub sha256: [u8; 32],
}

/// Static pin set. Add real values here before relying on TLS auth.
pub static PINS: &[Pin] = &[
    // Example (do not ship): "example.com" with placeholder hash.
    // Pin {
    //     host: b"example.com",
    //     sha256: [
    //         0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
    //         0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    //         0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
    //         0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    //     ],
    // },
];

/// Look up the pin for a hostname. Returns `Some(&[u8;32])` if pinned,
/// `None` otherwise.
pub fn pin_for(host: &[u8]) -> Option<&'static [u8; 32]> {
    for p in PINS.iter() {
        if p.host == host { return Some(&p.sha256); }
    }
    None
}

/// Decision returned by `check_cert`.
pub enum PinDecision {
    /// Pin matched — proceed.
    Match,
    /// Pin mismatched — abort the handshake.
    Mismatch,
    /// No pin for this host. In strict mode this is an abort; in
    /// permissive mode the caller logs and proceeds.
    NoPin,
}

/// Verify the leaf cert's SHA-256 against the pin for `host`.
/// `cert_der` is the raw DER bytes of the leaf certificate as taken
/// from the TLS Certificate message.
pub fn check_cert(host: &[u8], cert_der: &[u8]) -> PinDecision {
    let actual = crate::crypto::sig::sha256_digest(cert_der);
    match pin_for(host) {
        Some(expected) => {
            // Constant-time compare so a partial-match doesn't leak.
            let mut diff: u8 = 0;
            for i in 0..32 { diff |= expected[i] ^ actual[i]; }
            if diff == 0 {
                uart::puts("[tls] cert pin OK\n");
                PinDecision::Match
            } else {
                uart::puts("[tls] cert pin MISMATCH — aborting handshake\n");
                PinDecision::Mismatch
            }
        }
        None => PinDecision::NoPin,
    }
}
