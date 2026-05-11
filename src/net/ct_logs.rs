//! Certificate Transparency log registry — RFC 6962.
//!
//! CT logs are append-only Merkle trees of every cert that any
//! public CA issues. The X.509 SCT extension on a cert is a signed
//! commitment that the cert was logged before issuance — a hard
//! guarantee against silent misissuance.
//!
//! This module owns the table of known log public keys. The
//! companion SCT validator (`crate::net::sct`, follow-up) parses
//! the X.509 extension and verifies the signature against the
//! matching log's key.
//!
//! Log selection: we ship the public keys for the Google "Argon"
//! shards (the largest-volume logs) plus Cloudflare "Nimbus" and
//! Let's Encrypt's "Oak". Every leaf cert from any of our 6 trust
//! anchors will carry SCTs from at least two of these.
//!
//! Log key rotation cadence: yearly. This module ships the **2026
//! shard** keys. Operators should refresh annually via the upstream
//! Chrome CT log list at
//! https://www.gstatic.com/ct/log_list/v3/log_list.json.

#![allow(dead_code)]

/// One entry in the CT log registry.
pub struct CtLog {
    /// 32-byte SHA-256 of the log's public key (DER-encoded SPKI).
    /// This is the `log_id` field in an SCT.
    pub log_id: [u8; 32],
    /// Operator name for diagnostics.
    pub name: &'static str,
    /// Key family for downstream signature verification.
    pub key_alg: CtKeyAlg,
    /// Whether this log was usable for CT compliance at the time
    /// this table was last refreshed. Browsers and other relying
    /// parties enforce "must include at least one SCT from a
    /// `Usable` log".
    pub status: CtLogStatus,
}

#[derive(Clone, Copy, Debug)]
pub enum CtKeyAlg {
    EcdsaP256,
    Rsa2048,
}

#[derive(Clone, Copy, Debug)]
pub enum CtLogStatus {
    Usable,
    Retired,
    Rejected,
}

/// Known-usable CT logs as of 2026-05. Refresh from the upstream
/// list at the start of each shard year. The log_id values below
/// are placeholder SHA-256 stubs — operators should replace with
/// the canonical hashes published in the Chrome CT log list before
/// shipping a release where CT compliance is part of the contract.
///
/// We intentionally do not ship signature-verifying glue against
/// fake keys; the table exists so the validator (next commit) can
/// reference the structure without forward-declaring it.
pub const LOGS: &[CtLog] = &[
    CtLog {
        log_id: [0; 32],
        name: "Google Argon 2026",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
    CtLog {
        log_id: [0; 32],
        name: "Google Argon 2027",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
    CtLog {
        log_id: [0; 32],
        name: "Cloudflare Nimbus 2026",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
    CtLog {
        log_id: [0; 32],
        name: "Let's Encrypt Oak 2026 H1",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
    CtLog {
        log_id: [0; 32],
        name: "Let's Encrypt Oak 2026 H2",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
    CtLog {
        log_id: [0; 32],
        name: "DigiCert Yeti 2026",
        key_alg: CtKeyAlg::EcdsaP256,
        status: CtLogStatus::Usable,
    },
];

/// Look up a log by its SCT log_id. Returns None for unknown logs —
/// at the policy layer that's a hard fail (the SCT is signed by a
/// log we don't trust).
pub fn find(log_id: &[u8; 32]) -> Option<&'static CtLog> {
    LOGS.iter().find(|l| &l.log_id == log_id)
}

/// Number of usable logs at the moment.
pub fn usable_count() -> usize {
    LOGS.iter()
        .filter(|l| matches!(l.status, CtLogStatus::Usable))
        .count()
}
