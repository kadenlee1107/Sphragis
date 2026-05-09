//! Cave-policy entry for the AI agent's outbound TLS connection.
//!
//! The agent runs as a privileged kernel-side caller (it lives
//! in the kernel image, not in a cave), so its egress doesn't go
//! through the per-cave allowlist. Instead it has its own pinning
//! check enforced here: the inference host's self-signed cert is
//! pinned by SHA-256, and any deviation aborts the connection
//! before bytes flow.
//!
//! Phase 2 stub: holds the pinned-cert constant slot and a
//! placeholder check function. Real work lands in Phase 6.

#![allow(dead_code)]

use crate::ai::AgentError;

/// SHA-256 fingerprint of the operator's self-signed cert. Set at
/// deploy time by writing a 32-byte hex value into this slot —
/// `xxd -r -p` from the `openssl x509 -fingerprint -sha256 -noout`
/// output. Default zeros mean "not configured": all connections
/// fail closed.
pub const PINNED_CERT_SHA256: [u8; 32] = [0u8; 32];

/// Validate that the given peer cert SHA-256 matches our pin.
/// Constant-cost — always compares all 32 bytes regardless of
/// where the first mismatch is, to avoid timing leaks.
pub fn check_pin(observed: &[u8; 32]) -> Result<(), AgentError> {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= PINNED_CERT_SHA256[i] ^ observed[i];
    }
    if diff == 0 {
        Ok(())
    } else {
        Err(AgentError::PolicyDenied)
    }
}

/// Ensure the inference host:port pair matches the build-time
/// configuration. Belt-and-suspenders against accidental
/// reconfiguration at the call site.
pub fn ensure_allowlisted(host: &str, port: u16) -> Result<(), AgentError> {
    if host == crate::ai::client::HOST && port == crate::ai::client::PORT {
        Ok(())
    } else {
        Err(AgentError::PolicyDenied)
    }
}
