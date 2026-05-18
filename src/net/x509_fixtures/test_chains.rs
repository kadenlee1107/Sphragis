// GENERATED — do not edit. Regenerate via scripts/gen_x509_test_chains.py.
//
// X.509 chain-validator test fixtures backing the 6 scenarios from
// the 2026-05-17 multi-team-push plan §3 (Eng-1).
//
// All certs are ECDSA P-256 with SHA-256 signatures. The leaf SAN
// is `selftest.sphragis.test` so the hostname check passes for the
// valid chain. Each fixture group is independent: a different
// synthetic root anchors each chain, so swapping which root is in
// the test-trust-store flips a chain between Untrusted and Ok.

// Scenario 1 — valid 3-level chain.
pub const VALID_ROOT_DER:         &[u8] = include_bytes!("valid_root.der");
pub const VALID_INTERMEDIATE_DER: &[u8] = include_bytes!("valid_intermediate.der");
pub const VALID_LEAF_DER:         &[u8] = include_bytes!("valid_leaf.der");

// Scenario 2 — signature mismatch (leaf signed by a stranger).
pub const BADSIG_ROOT_DER:         &[u8] = include_bytes!("badsig_root.der");
pub const BADSIG_INTERMEDIATE_DER: &[u8] = include_bytes!("badsig_intermediate.der");
pub const BADSIG_LEAF_DER:         &[u8] = include_bytes!("badsig_leaf.der");

// Scenario 3 — expired intermediate.
pub const EXPIRED_ROOT_DER:         &[u8] = include_bytes!("expired_root.der");
pub const EXPIRED_INTERMEDIATE_DER: &[u8] = include_bytes!("expired_intermediate.der");
pub const EXPIRED_LEAF_DER:         &[u8] = include_bytes!("expired_leaf.der");

// Scenario 4 — unknown root (root NOT in test trust store).
pub const UNKNOWN_ROOT_DER:         &[u8] = include_bytes!("unknown_root.der");
pub const UNKNOWN_INTERMEDIATE_DER: &[u8] = include_bytes!("unknown_intermediate.der");
pub const UNKNOWN_LEAF_DER:         &[u8] = include_bytes!("unknown_leaf.der");

// Scenario 5 — BasicConstraints violation (leaf with CA:TRUE).
pub const BCLEAF_ROOT_DER:         &[u8] = include_bytes!("bcleaf_root.der");
pub const BCLEAF_INTERMEDIATE_DER: &[u8] = include_bytes!("bcleaf_intermediate.der");
pub const BCLEAF_LEAF_DER:         &[u8] = include_bytes!("bcleaf_leaf.der");

// Hostname every leaf SAN covers.
pub const SELFTEST_HOSTNAME: &[u8] = b"selftest.sphragis.test";
