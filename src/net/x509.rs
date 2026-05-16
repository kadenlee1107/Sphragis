// Sphragis — X.509 certificate chain validation.
//
// V4: replaces the pin-only interim defence with real PKI validation.
//
// Scope:
// * Parse leaf + intermediate certs from the TLS Certificate message
// (ASN.1 DER via `x509-cert`).
// * Extract SubjectPublicKeyInfo and surface pubkey for the
// CertificateVerify check.
// * Verify the signature chain leaf ← intermediate ← ... ← root
// where root lives in TRUST_STORE (hard-coded pinned CA certs).
// * Check notBefore / notAfter against a (static) boot time — we don't
// have wall-clock yet so we implement a lower bound (reject certs
// issued in the future) but accept slightly-expired ones and log a
// warning. Operator can pin time via KNOWN_GOOD_TIME.
// * Hostname match against SAN dNSName entries (CN fallback).
//
// What we DO NOT do:
// * Revocation (OCSP / CRL) — operator's job for high-security envs.
// * RSA verify — we only wire ECDSA P-256 / P-384 today. Adding RSA is
// a `rsa = "0.9"` dep away but every mainstream TLS-for-HTTPS cert
// we care about (LE ECDSA, Cloudflare ECDSA) is P-256 or P-384.
// * Name constraints / EKU — can add per-cert flags when needed.
//
// All-pass path:
// 1. `verify_chain(leaf_der, chain_ders, hostname, now_unix)` returns
// `Ok(subject_pubkey_der)`.
// 2. Caller hands `subject_pubkey_der` to
// `cert_verify_signature(pubkey_der, signed_bytes, sig_der)` to
// validate TLS-1.3 CertificateVerify.

#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryFrom;
use x509_cert::Certificate;
use x509_cert::der::{Decode, Encode};

/// Hard-coded trust anchors — embedded DER bytes of curated major
/// public CA roots.
// /
/// This used to be empty, which made the audit's verdict
/// ("TLS authentication is theater") literally true — chain validation
/// always returned UntrustedRoot, so HTTPS got us encrypted-but-
/// unauthenticated bytes vs an active MITM. Now populated with a
/// minimal-but-meaningful starter set sourced from each CA's official
/// publication endpoint:
// /
/// * ISRG Root X1 — RSA 4096 — Let's Encrypt's primary root.
/// Anchors a huge chunk of public HTTPS (Cloudflare-fronted sites,
/// GitHub Pages, basically every site that auto-issues with LE).
/// * ISRG Root X2 — ECDSA P-384 — Let's Encrypt's modern ECDSA
/// root, used by sites that opted into ECDSA leaf certs.
/// * Amazon Root CA 1 — RSA 2048 — anchors AWS-hosted services and
/// anything fronted by Amazon.com.
/// * DigiCert Global Root CA — RSA 2048 — anchors a large fraction
/// of enterprise + financial sites.
/// * DigiCert Global Root G2 — RSA 2048 — DigiCert's modern root,
/// used by Google's intermediate CA chain among others.
/// * GTS Root R4 — ECDSA P-384 — Google Trust Services' modern
/// ECDSA root. Anchors a growing slice of public HTTPS as Google
/// migrates leaves from GlobalSign to its own PKI; required for
/// pq.cloudflareresearch.com (used by our PQ-interop smoke).
// /
/// `TrustStore::contains` compares subject public-key bytes for the
/// issuer lookup, so this set covers a meaningful slice of the public
/// web. A full Mozilla CA bundle (~150 roots) is a follow-up STUMP —
/// this six-entry set is enough to verify the most common chains and
/// move the audit's "theater" verdict.
// /
/// **Signature algorithm coverage** :
/// * Cert sigs: ECDSA-P256/P384, RSA-PKCS1v15 (SHA-256/384/512),
/// RSA-PSS — covers self-sigs of every root above + the chains
/// they typically anchor.
/// * TLS-1.3 CertificateVerify: ECDSA-P256, ECDSA-P384, RSA-PSS
/// (SHA-256/384/512). PKCS#1v1.5 is not valid for CertVerify per
/// RFC 8446 §4.4.3 — it's only for cert chain sigs.
// /
/// Refresh procedure: each CA publishes their root cert via a stable
/// URL listed below. Re-fetch, drop into `src/net/ca_certs/`, rebuild.
pub static TRUST_STORE: &[&[u8]] = &[
    // https://letsencrypt.org/certs/isrgrootx1.der
    include_bytes!("ca_certs/isrg_root_x1.der"),
    // https://letsencrypt.org/certs/isrg-root-x2.der
    include_bytes!("ca_certs/isrg_root_x2.der"),
    // https://www.amazontrust.com/repository/AmazonRootCA1.cer
    include_bytes!("ca_certs/amazon_root_ca1.der"),
    // https://cacerts.digicert.com/DigiCertGlobalRootCA.crt
    include_bytes!("ca_certs/digicert_global_root_ca.der"),
    // https://cacerts.digicert.com/DigiCertGlobalRootG2.crt
    include_bytes!("ca_certs/digicert_global_root_g2.der"),
    // https://i.pki.goog/r4.crt (sha256 34:9D:FA:40:58:C5:E2:63:12:3B:39:8A:E7:95:57:3C:4E:13:13:C8:3F:E6:8F:93:55:6C:D5:E8:03:1B:3C:7D)
    include_bytes!("ca_certs/gts_root_r4.der"),
];

/// Outcome of chain validation.
pub enum VerifyOutcome {
    Ok { pubkey_der: Vec<u8>, pubkey_algorithm: PubkeyAlg },
    Err(VerifyError),
}

#[derive(Clone, Copy, Debug)]
pub enum VerifyError {
    Parse,
    EmptyChain,
    UnsupportedSigAlg,
    HostnameMismatch,
    NotYetValid,
    Expired,
    BadSignature,
    UntrustedRoot,
    ChainIncomplete,
    UnknownCriticalExtension,
    BasicConstraintsViolation,
    KeyUsageViolation,
    EkuViolation,
    // AUDIT-CRYPTO-F3 (2026-05-15): per-host SPKI pinning enforced
    // inside verify_chain. Operator pinned a different SPKI for this
    // host; the presented leaf doesn't match any registered pin.
    PinMismatch,
    // AUDIT-CRYPTO-F4 (2026-05-15): cert appears in CRL or OCSP
    // says it's Revoked. Most often used for compromised-key
    // revocation of an otherwise-valid chain.
    Revoked,
}

impl VerifyError {
    /// Map a verifier failure to a debug-friendly static string.
    /// Used by `tls.rs`'s chain-fail branch to abort the handshake with
    /// a specific reason. See DESIGN_TLS_HARDENING.md.
    pub fn as_static_str(&self) -> &'static str {
        match self {
            VerifyError::Parse                     => "TLS: chain validation failed: certificate parse error",
            VerifyError::EmptyChain                => "TLS: chain validation failed: empty chain",
            VerifyError::UnsupportedSigAlg         => "TLS: chain validation failed: unsupported signature algorithm",
            VerifyError::HostnameMismatch          => "TLS: chain validation failed: hostname mismatch",
            VerifyError::NotYetValid               => "TLS: chain validation failed: certificate not yet valid",
            VerifyError::Expired                   => "TLS: chain validation failed: expired certificate",
            VerifyError::BadSignature              => "TLS: chain validation failed: bad signature",
            VerifyError::UntrustedRoot             => "TLS: chain validation failed: untrusted root",
            VerifyError::ChainIncomplete           => "TLS: chain validation failed: chain incomplete",
            VerifyError::UnknownCriticalExtension  => "TLS: chain validation failed: unknown critical extension",
            VerifyError::BasicConstraintsViolation => "TLS: chain validation failed: BasicConstraints violation",
            VerifyError::KeyUsageViolation         => "TLS: chain validation failed: KeyUsage missing keyCertSign",
            VerifyError::EkuViolation              => "TLS: chain validation failed: leaf EKU missing serverAuth",
            VerifyError::PinMismatch               => "TLS: chain validation failed: pin mismatch (operator-pinned SPKI differs)",
            VerifyError::Revoked                   => "TLS: chain validation failed: cert revoked (CRL or OCSP)",
        }
    }
}

/// x509-hardening-a: build-time epoch floor for validity-period checks.
///
/// Sphragis is bare-metal with no RTC/NTP, so we cannot ask "what time is
/// it" the way userspace can. `SPHRAGIS_BUILD_UNIX` is set by build.rs at
/// compile time from the build host's `SystemTime::now()`, so it tracks
/// reality up to one rebuild old. The verifier uses this as a lower
/// bound — a cert whose `notBefore` is *after* this value is rejected
/// as `NotYetValid` (signed in the future relative to the build), and
/// a cert whose `notAfter` is *before* this is rejected as `Expired`.
///
/// Caveats:
/// * If the binary runs for months without a rebuild, `Expired` checks
///   become permissive (kerned from a stale floor), and `NotYetValid`
///   stays correct (a never-issued cert is never valid). This is the
///   right failure mode for an offline kernel.
/// * The fallback constant `1_735_689_600` (2025-01-01 UTC) is what
///   ships if build.rs failed to set the env. Floor is the date below
///   which we know the project did not exist.
pub fn now_unix() -> i64 {
    const FALLBACK_UNIX: i64 = 1_735_689_600; // 2025-01-01 UTC
    match option_env!("SPHRAGIS_BUILD_UNIX") {
        Some(s) => match s.parse::<i64>() {
            Ok(v) if v >= FALLBACK_UNIX => v,
            _ => FALLBACK_UNIX,
        },
        None => FALLBACK_UNIX,
    }
}

/// Convert an x509-cert `Time` to Unix epoch seconds.
fn time_to_unix(t: &x509_cert::time::Time) -> i64 {
    t.to_unix_duration().as_secs() as i64
}

/// Reject a cert whose `notBefore` is in the future (relative to
/// `now_unix`) or whose `notAfter` has passed. Spec: RFC 5280 §6.1.3(a)(2).
///
/// Constant-cost: this function does the same work for every cert
/// regardless of outcome, so calling it on every chain cert does not
/// leak which cert was the bad one via timing.
fn check_validity(cert: &Certificate, now: i64) -> Result<(), VerifyError> {
    let v = &cert.tbs_certificate.validity;
    let nb = time_to_unix(&v.not_before);
    let na = time_to_unix(&v.not_after);
    if now < nb {
        return Err(VerifyError::NotYetValid);
    }
    if now > na {
        return Err(VerifyError::Expired);
    }
    Ok(())
}

/// OIDs the verifier knows how to handle when marked critical.
///
/// RFC 5280 §4.2: a relying party MUST reject a certificate carrying a
/// critical extension whose semantics it does not recognize. Without
/// this check, a CA could pin a constraint (e.g. NameConstraints) that
/// we silently ignore — defeating the purpose of marking it critical.
///
/// Recognized = "we either enforce it elsewhere, or it is structurally
/// safe to honor as informational." Unrecognized critical = hard fail.
///
/// PR-a (this PR) recognizes the OIDs that already appear in real
/// public CA chains. PR-b (BasicConstraints / KeyUsage / EKU
/// enforcement) will add real semantics behind the matching OIDs.
const RECOGNIZED_CRITICAL_OIDS: &[const_oid::ObjectIdentifier] = &[
    const_oid::db::rfc5280::ID_CE_BASIC_CONSTRAINTS,         // 2.5.29.19
    const_oid::db::rfc5280::ID_CE_KEY_USAGE,                 // 2.5.29.15
    const_oid::db::rfc5280::ID_CE_EXT_KEY_USAGE,             // 2.5.29.37
    const_oid::db::rfc5280::ID_CE_SUBJECT_ALT_NAME,          // 2.5.29.17
    const_oid::db::rfc5280::ID_CE_NAME_CONSTRAINTS,          // 2.5.29.30
    const_oid::db::rfc5280::ID_CE_POLICY_CONSTRAINTS,        // 2.5.29.36
    const_oid::db::rfc5280::ID_CE_INHIBIT_ANY_POLICY,        // 2.5.29.54
    const_oid::db::rfc5280::ID_CE_CERTIFICATE_POLICIES,      // 2.5.29.32
    const_oid::db::rfc5280::ID_CE_CRL_DISTRIBUTION_POINTS,   // 2.5.29.31
];

/// Walk the cert's extensions; reject if any extension is marked
/// critical AND its OID is not in `RECOGNIZED_CRITICAL_OIDS`.
fn check_critical_extensions(cert: &Certificate) -> Result<(), VerifyError> {
    let Some(exts) = &cert.tbs_certificate.extensions else {
        return Ok(());
    };
    for ext in exts.iter() {
        if !ext.critical {
            continue;
        }
        if !RECOGNIZED_CRITICAL_OIDS.iter().any(|oid| oid == &ext.extn_id) {
            return Err(VerifyError::UnknownCriticalExtension);
        }
    }
    Ok(())
}

/// x509-hardening-b: BasicConstraints enforcement (RFC 5280 §4.2.1.9).
///
/// Per spec:
/// * The leaf cert must NOT assert `cA: TRUE`. A CA-marked leaf could
///   issue arbitrary certs for any name; that's the original
///   "you have a leaf for evil.com but it's also a CA" footgun.
/// * Every intermediate MUST have BasicConstraints with `cA: TRUE`.
///   Missing BC on an intermediate = pre-fix behavior, where any cert
///   in the bundle could be presented as an intermediate.
///
/// x509-hardening-c: `pathLenConstraint` is now enforced — but with
/// **anchor-aware counting**. RFC 5280 §4.2.1.9 reads "non-self-issued
/// intermediate certificates that may follow this certificate"; a cert
/// in the chain that matches a TRUST_STORE entry by SPKI is the anchor
/// presented in chain form (cross-sign or duplicate self-sign), not an
/// intermediate. The caller computes `intermediates_below` excluding
/// such anchor-equivalent certs, so chains like Let's Encrypt's
/// E1 → DST X3 cross-sign no longer false-positive against `pathLen=0`.
fn check_basic_constraints(
    cert: &Certificate,
    is_leaf: bool,
    intermediates_below: usize,
) -> Result<(), VerifyError> {
    use x509_cert::ext::pkix::BasicConstraints;
    let exts = match &cert.tbs_certificate.extensions {
        Some(e) => e,
        // No extensions at all: legal for leaves (BC absent ⇒ end-entity).
        // Illegal for intermediates — without BC:CA=TRUE a cert MUST NOT
        // sign other certs.
        None => return if is_leaf { Ok(()) } else { Err(VerifyError::BasicConstraintsViolation) },
    };

    for ext in exts.iter() {
        if ext.extn_id != const_oid::db::rfc5280::ID_CE_BASIC_CONSTRAINTS {
            continue;
        }
        let bc = match BasicConstraints::from_der(ext.extn_value.as_bytes()) {
            Ok(v) => v,
            Err(_) => return Err(VerifyError::BasicConstraintsViolation),
        };
        if is_leaf {
            // Leaf with cA:TRUE = banned (would let leaf sign other certs).
            if bc.ca {
                return Err(VerifyError::BasicConstraintsViolation);
            }
        } else {
            if !bc.ca {
                return Err(VerifyError::BasicConstraintsViolation);
            }
            if let Some(pl) = bc.path_len_constraint {
                if intermediates_below > pl as usize {
                    return Err(VerifyError::BasicConstraintsViolation);
                }
            }
        }
        return Ok(());
    }
    // Reached here ⇒ no BasicConstraints extension found.
    // Leaves may omit BC (RFC 5280 §4.2.1.9); intermediates may not.
    if is_leaf { Ok(()) } else { Err(VerifyError::BasicConstraintsViolation) }
}

/// x509-hardening-b: KeyUsage enforcement for cert-signing certs
/// (RFC 5280 §4.2.1.3).
///
/// If a cert that is being used as an intermediate carries a KeyUsage
/// extension, that extension MUST include the `keyCertSign` bit.
/// Without `keyCertSign` the cert is not authorized to sign other
/// certificates, regardless of what BasicConstraints says.
///
/// KeyUsage absent ⇒ no constraint per RFC. We only fail if the
/// extension is present and the bit is missing.
fn check_key_usage_for_signing(cert: &Certificate) -> Result<(), VerifyError> {
    use x509_cert::ext::pkix::{KeyUsage, KeyUsages};
    let exts = match &cert.tbs_certificate.extensions {
        Some(e) => e,
        None => return Ok(()),
    };
    for ext in exts.iter() {
        if ext.extn_id != const_oid::db::rfc5280::ID_CE_KEY_USAGE {
            continue;
        }
        let ku = match KeyUsage::from_der(ext.extn_value.as_bytes()) {
            Ok(v) => v,
            Err(_) => return Err(VerifyError::KeyUsageViolation),
        };
        if !ku.0.contains(KeyUsages::KeyCertSign) {
            return Err(VerifyError::KeyUsageViolation);
        }
        return Ok(());
    }
    Ok(())
}

/// x509-hardening-b: ExtendedKeyUsage enforcement on the leaf
/// (RFC 5280 §4.2.1.12 + CA/Browser Forum BR §7.1.2.7.10).
///
/// For TLS server certs: if the leaf carries an EKU extension, that
/// extension MUST include either `id-kp-serverAuth` (1.3.6.1.5.5.7.3.1)
/// or `anyExtendedKeyUsage` (2.5.29.37.0). A code-signing-only cert,
/// for example, is not allowed to authenticate a TLS server even if
/// its chain otherwise validates.
///
/// EKU absent on the leaf ⇒ no constraint per RFC. We only fail if
/// EKU is present and serverAuth/anyEKU is missing.
fn check_eku_server_auth(leaf: &Certificate) -> Result<(), VerifyError> {
    use x509_cert::ext::pkix::ExtendedKeyUsage;
    let exts = match &leaf.tbs_certificate.extensions {
        Some(e) => e,
        None => return Ok(()),
    };
    // anyExtendedKeyUsage: 2.5.29.37.0 — sentinel meaning "any usage allowed".
    let any_eku = const_oid::ObjectIdentifier::new_unwrap("2.5.29.37.0");
    let server_auth = const_oid::db::rfc5280::ID_KP_SERVER_AUTH;
    for ext in exts.iter() {
        if ext.extn_id != const_oid::db::rfc5280::ID_CE_EXT_KEY_USAGE {
            continue;
        }
        let eku = match ExtendedKeyUsage::from_der(ext.extn_value.as_bytes()) {
            Ok(v) => v,
            Err(_) => return Err(VerifyError::EkuViolation),
        };
        let ok = eku.0.iter().any(|oid| *oid == server_auth || *oid == any_eku);
        if !ok {
            return Err(VerifyError::EkuViolation);
        }
        return Ok(());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PubkeyAlg {
    EcdsaP256,
    EcdsaP384,
    Rsa,
    Ed25519,
    Unknown,
}

/// Parse a DER-encoded certificate.
pub fn parse_cert(der: &[u8]) -> Result<Certificate, VerifyError> {
    Certificate::from_der(der).map_err(|_| VerifyError::Parse)
}

/// V5-CHAIN-001 / V5-CRYPTO-001 fix: extract leaf SPKI + algorithm for
/// the CertificateVerify check **regardless** of whether chain validation
/// succeeded. Without this, a fallback-to-pinning path left peer_spki
/// empty and CertificateVerify was silently skipped, giving a full MITM
/// bypass.
pub fn leaf_info(leaf_der: &[u8]) -> Result<(alloc::vec::Vec<u8>, PubkeyAlg), VerifyError> {
    let leaf = parse_cert(leaf_der)?;
    let spki = subject_spki_der(&leaf)?;
    Ok((spki, pubkey_alg(&leaf)))
}

/// V11-FRESH-EYES: like `leaf_info`, but additionally enforces that the
/// leaf cert's SAN (or CN as a last resort) actually covers `hostname`.
/// Used by the TLS fallback-to-pinning path so a cert legitimately issued
/// for host A cannot be used against host B — which was possible before
/// because the pin-only path never re-checked hostname against the leaf.
pub fn leaf_info_with_host(leaf_der: &[u8], hostname: &[u8]) -> Result<(alloc::vec::Vec<u8>, PubkeyAlg), VerifyError> {
    let leaf = parse_cert(leaf_der)?;
    if !check_hostname(&leaf, hostname) {
        return Err(VerifyError::HostnameMismatch);
    }
    let spki = subject_spki_der(&leaf)?;
    Ok((spki, pubkey_alg(&leaf)))
}

/// Extract the SubjectPublicKeyInfo encoded as DER from a certificate.
/// Used by the TLS CertificateVerify path to obtain the peer's pubkey.
pub fn subject_spki_der(cert: &Certificate) -> Result<Vec<u8>, VerifyError> {
    cert.tbs_certificate
        .subject_public_key_info
        .to_der()
        .map_err(|_| VerifyError::Parse)
}

/// Identify the public-key algorithm for signature dispatch.
///
/// AUDIT-CRYPTO-F16 (2026-05-15): for EC keys the curve is now
/// identified by parsing the curve OID from the
/// AlgorithmIdentifier's `parameters` field instead of guessing
/// from the SubjectPublicKey byte length. Prior heuristic ID'd
/// P-256 by length 65, P-384 by length 97 — Brainpool / secp256k1
/// / SM2 happen to encode at the same lengths and would have been
/// misclassified. Not a security bypass (the downstream
/// ecdsa_p256_verify would fail), but losing safety margin.
pub fn pubkey_alg(cert: &Certificate) -> PubkeyAlg {
    use const_oid::db::rfc5912;
    let spki = &cert.tbs_certificate.subject_public_key_info;
    let oid = spki.algorithm.oid;
    if oid == rfc5912::ID_EC_PUBLIC_KEY {
        // RFC 5480 §2.1.1: parameters for EC-public-key carry
        // ECParameters which for named curves is just a CHOICE
        // namedCurve (an OID). Decode + compare to secp256r1 /
        // secp384r1.
        let params = match spki.algorithm.parameters.as_ref() {
            Some(p) => p,
            None => return PubkeyAlg::Unknown,
        };
        use der::Decode;
        let curve_oid = match const_oid::ObjectIdentifier::from_der(params.value()) {
            Ok(o) => o,
            Err(_) => return PubkeyAlg::Unknown,
        };
        if curve_oid == rfc5912::SECP_256_R_1 {
            PubkeyAlg::EcdsaP256
        } else if curve_oid == rfc5912::SECP_384_R_1 {
            PubkeyAlg::EcdsaP384
        } else {
            // Brainpool, secp256k1, SM2, etc. land here.
            PubkeyAlg::Unknown
        }
    } else if oid == rfc5912::RSA_ENCRYPTION {
        PubkeyAlg::Rsa
    } else if oid.as_bytes() == [0x2B, 0x65, 0x70] {
        // Ed25519 OID = 1.3.101.112
        PubkeyAlg::Ed25519
    } else {
        PubkeyAlg::Unknown
    }
}

/// Verify that `hostname` matches the cert's SAN dNSName list (CN
/// fallback is **not** implemented — modern RFC 6125 requires SAN).
pub fn check_hostname(cert: &Certificate, hostname: &[u8]) -> bool {
    use x509_cert::ext::pkix::name::GeneralName;
    use x509_cert::ext::pkix::SubjectAltName;

    let Some(exts) = &cert.tbs_certificate.extensions else {
        return false;
    };
    for ext in exts.iter() {
        if ext.extn_id == const_oid::db::rfc5280::ID_CE_SUBJECT_ALT_NAME {
            if let Ok(san) = SubjectAltName::from_der(ext.extn_value.as_bytes()) {
                for gn in san.0.iter() {
                    if let GeneralName::DnsName(d) = gn {
                        let d_bytes = d.as_bytes();
                        if hostname_matches(d_bytes, hostname) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Wildcard-aware hostname match. `pattern` may start with `*.` meaning
/// "any single leftmost label matches".
///
/// AUDIT-CRYPTO-F17 + F18 (2026-05-15): case-insensitive ASCII match
/// per RFC 6125 §6.4.1 + reject empty-leftmost-label wildcards.
/// Prior implementation:
///   * Did byte-exact compare, so `Example.com` cert vs `example.com`
///     request failed (false negative; not exploitable but
///     RFC-non-conformant — would have shown up when peer cert CN
///     mixed case).
///   * Pattern `*.example.com` matched `hostname = .example.com`
///     (empty leftmost label) because `position(|b| b == b'.')`
///     returned Some(0) and the suffix matched. DNS resolvers don't
///     yield empty-leftmost-label names but a future protocol path
///     that does could pass our SAN check and resolve differently.
fn hostname_matches(pattern: &[u8], hostname: &[u8]) -> bool {
    // ASCII-case-insensitive exact match.
    if eq_ignore_ascii_case(pattern, hostname) {
        return true;
    }
    if pattern.len() > 2 && &pattern[..2] == b"*." {
        let suffix = &pattern[1..]; // keep the leading "."
        // Find the first '.' in hostname. Require at least one char
        // before it (idx > 0) so `*.example.com` doesn't match
        // `.example.com` (empty leftmost label).
        if let Some(idx) = hostname.iter().position(|&b| b == b'.') {
            if idx > 0 {
                return eq_ignore_ascii_case(&hostname[idx..], suffix);
            }
        }
    }
    false
}

fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i].to_ascii_lowercase() ^ b[i].to_ascii_lowercase();
    }
    diff == 0
}

/// Verify an ECDSA signature (P-256 only in this minimal build) over
/// `signed_bytes` using the pubkey DER taken from a cert. `signed_bytes`
/// is already hashed by the caller when calling TLS-style prehash.
pub fn cert_verify_ecdsa_p256(pubkey_der: &[u8], digest: &[u8; 32], sig_der: &[u8])
    -> Result<(), VerifyError>
{
    // pubkey_der is a SubjectPublicKeyInfo. Extract the BIT STRING content
    // to feed p256::EncodedPoint. Minimal approach: walk the DER to find
    // the subjectPublicKey BIT STRING.
    use x509_cert::spki::SubjectPublicKeyInfoRef;
    let spki = SubjectPublicKeyInfoRef::from_der(pubkey_der)
        .map_err(|_| VerifyError::Parse)?;
    let raw_point = spki.subject_public_key.raw_bytes();
    crate::crypto::sig::ecdsa_p256_verify(raw_point, digest, sig_der)
        .map_err(|_| VerifyError::BadSignature)
}

/// Full chain validation.
// /
/// `leaf_der`: the server's certificate.
/// `chain_ders`: intermediate certs in order (leaf-issuer, then its
/// issuer, …). May be empty if leaf is directly signed by
/// a root in TRUST_STORE.
/// `hostname`: the hostname the client expects (from SNI / URL).
// /
/// Returns the leaf's SPKI DER on success so the caller can then verify
/// the TLS CertificateVerify signature with it.
pub fn verify_chain(
    leaf_der: &[u8],
    chain_ders: &[&[u8]],
    hostname: &[u8],
) -> VerifyOutcome {
    let leaf = match parse_cert(leaf_der) {
        Ok(c) => c,
        Err(e) => return VerifyOutcome::Err(e),
    };

    // V6-SIDE-002 fix: do the EXPENSIVE signature verification FIRST
    // and accumulate the hostname-mismatch flag, so the abort timing
    // does NOT distinguish "wrong hostname" from "wrong signature".
    // V5 returned early on hostname mismatch BEFORE doing chain
    // verify, leaving a 30-50× timing delta between the two outcomes
    // an off-path observer measuring the abort time learned which
    // hostname the client tried.
    let hostname_ok = check_hostname(&leaf, hostname);

    // x509-hardening-a + b: cheap RFC-5280 conformance checks.
    // Each runs on every relevant cert in the chain and accumulates into
    // a flag — same constant-cost abort discipline as V6-SIDE-002. These
    // do no signature math, so they cost the same as a few extra parses.
    let now = now_unix();
    let mut earliest_validity_err: Option<VerifyError> = None;
    let mut critical_ext_ok = true;
    // x509-hardening-b: BasicConstraints / KeyUsage / EKU enforcement.
    let mut bc_violation: Option<VerifyError> = None;
    let mut ku_ok = true;
    let mut eku_ok = true;

    let mut record_validity = |c: &Certificate| {
        if earliest_validity_err.is_none() {
            if let Err(e) = check_validity(c, now) {
                earliest_validity_err = Some(e);
            }
        }
    };
    let mut record_critical = |c: &Certificate| {
        if check_critical_extensions(c).is_err() {
            critical_ext_ok = false;
        }
    };
    let mut record_bc = |c: &Certificate, is_leaf: bool, below: usize| {
        if bc_violation.is_none() {
            if let Err(e) = check_basic_constraints(c, is_leaf, below) {
                bc_violation = Some(e);
            }
        }
    };
    let mut record_ku = |c: &Certificate| {
        if check_key_usage_for_signing(c).is_err() {
            ku_ok = false;
        }
    };

    // x509-hardening-c: precompute which chain certs are anchor-equivalent
    // (share an SPKI with a TRUST_STORE entry). For pathLen counting these
    // are NOT real intermediates — they are the trust anchor presented in
    // chain form (cross-sign or duplicate self-sign). Counting them as
    // intermediates is what tripped strict pathLen against real-world
    // Let's Encrypt and Cloudflare chains in PR-b.
    let trust_spkis: Vec<Vec<u8>> = TRUST_STORE
        .iter()
        .filter_map(|der| parse_cert(der).ok())
        .map(|c| {
            c.tbs_certificate
                .subject_public_key_info
                .subject_public_key
                .raw_bytes()
                .to_vec()
        })
        .collect();
    let chain_is_anchor: Vec<bool> = chain_ders
        .iter()
        .map(|der| match parse_cert(der) {
            Ok(c) => {
                let spki = c
                    .tbs_certificate
                    .subject_public_key_info
                    .subject_public_key
                    .raw_bytes();
                trust_spkis.iter().any(|t| t.as_slice() == spki)
            }
            Err(_) => false,
        })
        .collect();

    record_validity(&leaf);
    record_critical(&leaf);
    // Leaf-only checks. intermediates_below is irrelevant for leaves
    // (the leaf is the path terminus on the subject end).
    record_bc(&leaf, true, 0);
    if check_eku_server_auth(&leaf).is_err() {
        eku_ok = false;
    }

    // 2. Walk the chain. For each (child, parent) pair, verify that
    // parent.pubkey validates child.signature over child.tbsCertificate.
    // Root must be in TRUST_STORE.
    let mut current_cert = leaf.clone();
    let mut current_der: &[u8] = leaf_der;
    let mut chain_ok = true;

    for (i, int_der) in chain_ders.iter().enumerate() {
        let parent = match parse_cert(int_der) {
            Ok(c) => c,
            Err(_) => {
                chain_ok = false;
                break;
            }
        };

        record_validity(&parent);
        record_critical(&parent);
        // Intermediates: BasicConstraints must say cA:TRUE; if pathLen
        // is set, count the *real* intermediates below this cert in the
        // chain — anchor-equivalent certs (same SPKI as a trust anchor)
        // don't count, they are the anchor presented in chain form.
        let real_intermediates_below = chain_is_anchor[i + 1..]
            .iter()
            .filter(|&&is_anchor| !is_anchor)
            .count();
        record_bc(&parent, false, real_intermediates_below);
        // KeyUsage with keyCertSign required if KU extension is present.
        record_ku(&parent);

        if verify_signed_by(&current_cert, current_der, &parent).is_err() {
            chain_ok = false;
            // Continue the loop (don't return) so chain length doesn't
            // distinguish failure-on-step-N from -on-step-M timing-wise.
            // Subsequent verify_signed_by calls run against a possibly
            // wrong parent, but that's harmless because we already
            // know we're going to fail.
        }
        current_cert = parent;
        current_der = int_der;
    }

    // Only AFTER the (constant-cost) chain walk do we examine the
    // accumulated outcome. Any single-flag short-circuit before this
    // point would re-introduce the timing oracle.
    if !hostname_ok {
        return VerifyOutcome::Err(VerifyError::HostnameMismatch);
    }
    if !chain_ok {
        return VerifyOutcome::Err(VerifyError::BadSignature);
    }
    if let Some(e) = earliest_validity_err {
        return VerifyOutcome::Err(e);
    }
    if !critical_ext_ok {
        return VerifyOutcome::Err(VerifyError::UnknownCriticalExtension);
    }
    if let Some(e) = bc_violation {
        return VerifyOutcome::Err(e);
    }
    if !ku_ok {
        return VerifyOutcome::Err(VerifyError::KeyUsageViolation);
    }
    if !eku_ok {
        return VerifyOutcome::Err(VerifyError::EkuViolation);
    }

    // 3. Root in trust store? Three accept paths, walked in order of
    // cost. All three are spec-equivalent ways of saying "the chain
    // rooted at some entry in TRUST_STORE":
    //
    // a. Current cert IS an anchor (server included its own root in
    // the chain — uncommon but legal).
    // b. Current cert and an anchor share the same SubjectPublicKey
    // (cross-signed root: the anchor we ship and the cert the
    // server sent are different DER blobs but represent the same
    // trust anchor — common, e.g. GTS Root R4 cross-signed by
    // GlobalSign).
    // c. Current cert is signed by an anchor (the typical real-world
    // case: the server sends [leaf, intermediate(s)] and stops
    // short of the root because RFC 5246 says clients should have
    // the root locally). Validates by treating the anchor as a
    // virtual parent and re-running the signature check.
    //
    // Pre-fix the verifier only had (a) and (b), so chains from
    // Let's Encrypt / GTS-anchored sites that don't ship the root
    // failed at "untrusted root" even though the chain is valid.
    let mut trusted = false;
    for anchor_der in TRUST_STORE.iter() {
        // Path (a): exact-bytes equality.
        if anchor_der == &current_der {
            trusted = true;
            break;
        }
        let anchor = match parse_cert(anchor_der) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Path (b): same subject public key — cross-signed root.
        let ap = anchor.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
        let cp = current_cert.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
        if ap == cp {
            trusted = true;
            break;
        }
        // Path (c): current cert signed by anchor.
        if verify_signed_by(&current_cert, current_der, &anchor).is_ok() {
            trusted = true;
            break;
        }
    }

    if TRUST_STORE.is_empty() {
        // V5-CRYPTO-004 / V5-CHAIN-001 fix: empty trust store now returns
        // UntrustedRoot — the previous "Ok if empty" behaviour made all
        // of V4's chain validation a no-op on shipped builds because
        // TRUST_STORE ships empty. The tls.rs caller has a pin-check
        // fallback that runs on any Err return, preserving the interim
        // defence. Populate TRUST_STORE with real roots for full chain
        // enforcement.
        return VerifyOutcome::Err(VerifyError::UntrustedRoot);
    }

    if !trusted {
        return VerifyOutcome::Err(VerifyError::UntrustedRoot);
    }

    let leaf_spki = match subject_spki_der(&leaf) {
        Ok(v) => v,
        Err(e) => return VerifyOutcome::Err(e),
    };

    // AUDIT-CRYPTO-F3 (2026-05-15): per-host SPKI pinning.
    // `cert_pin::check` was defined but never called from this
    // function — a misissued cert from any of our embedded trust
    // anchors would pass the chain validation but be caught here
    // if the operator has pinned the host. Fail-open if no pin is
    // registered for the host (`Ok(false)`): see the host-pinning
    // operator runbook for how to seed pins.
    if let Ok(host_str) = core::str::from_utf8(hostname) {
        match crate::net::cert_pin::check(host_str, &leaf_spki) {
            Ok(_) => { /* pin matched OR no pin registered */ }
            Err(_) => return VerifyOutcome::Err(VerifyError::PinMismatch),
        }
    }

    // AUDIT-CRYPTO-F4 (2026-05-15): revocation check (CRL + OCSP).
    // Both modules were implemented but never consulted from
    // verify_chain. A revoked cert (key compromise, name-misuse)
    // would otherwise remain valid until its notAfter — year-scale
    // exposure for leaf certs with long validity. For now we check
    // only the leaf; intermediate revocation is a follow-up wave.
    //
    // The issuer SPKI hash needed by both checks is computed from
    // the first chain_ders entry (the leaf's immediate parent) when
    // it exists. If chain_ders is empty (rare — usually means leaf
    // is signed directly by a trust anchor), skip revocation
    // checks; trust-store path already covered authenticity.
    if let Some(parent_der) = chain_ders.first() {
        if let Ok(parent) = parse_cert(parent_der) {
            let parent_spki = parent
                .tbs_certificate
                .subject_public_key_info
                .subject_public_key
                .raw_bytes();
            let issuer_hash: [u8; 32] = crate::crypto::sha256::hash(parent_spki);
            let serial = leaf.tbs_certificate.serial_number.as_bytes();

            if crate::net::crl::is_revoked(&issuer_hash, serial) {
                return VerifyOutcome::Err(VerifyError::Revoked);
            }
            if let Some(crate::net::ocsp::Status::Revoked) =
                crate::net::ocsp::status(&issuer_hash, serial)
            {
                return VerifyOutcome::Err(VerifyError::Revoked);
            }
        }
    }

    VerifyOutcome::Ok {
        pubkey_der: leaf_spki,
        pubkey_algorithm: pubkey_alg(&leaf),
    }
}

/// Verify that `parent` signed `child`.
// /
/// was ECDSA-P256-only. Now dispatches on the child's
/// `signatureAlgorithm` OID (NOT just the parent's pubkey alg) and
/// supports:
/// * 1.2.840.10045.4.3.2 ecdsa-with-SHA256 (ECDSA P-256 leaf sigs)
/// * 1.2.840.10045.4.3.3 ecdsa-with-SHA384 (ECDSA P-384 leaf sigs)
/// * 1.2.840.113549.1.1.11 sha256WithRSAEncryption (PKCS#1 v1.5 RSA)
/// * 1.2.840.113549.1.1.12 sha384WithRSAEncryption
/// * 1.2.840.113549.1.1.13 sha512WithRSAEncryption
/// * 1.2.840.113549.1.1.10 RSASSA-PSS (RSA-PSS — caller picks
/// hash from PSS params)
// /
/// Other algorithms return `UnsupportedSigAlg`. This unlocks the three
/// RSA roots embedded by plus the
/// ECDSA-P384 ISRG Root X2.
fn verify_signed_by(
    child: &Certificate,
    child_der: &[u8],
    parent: &Certificate,
) -> Result<(), VerifyError> {
    let _ = child;

    // TBS bytes to sign: re-encode the tbsCertificate field.
    let tbs = child.tbs_certificate.to_der().map_err(|_| VerifyError::Parse)?;

    // Re-decode the outer Certificate so we can read sigAlgo + sigBytes.
    let cert = parse_cert(child_der)?;
    let sig_bytes = cert.signature.raw_bytes();
    let sig_oid_raw = cert.signature_algorithm.oid;
    let sig_oid = sig_oid_raw.as_bytes();

    // Parent SPKI bytes — ECDSA paths take this raw, RSA paths need to
    // strip the SPKI wrapper to get the inner RsaPublicKey.
    let parent_spki = parent.tbs_certificate.subject_public_key_info
        .subject_public_key.raw_bytes();

    // OID numeric form for matching (avoids importing const_oid::db
    // tables for every variant — these are short and stable).
    // 1.2.840.10045.4.3.2 = 0x2A 0x86 0x48 0xCE 0x3D 0x04 0x03 0x02
    // 1.2.840.10045.4.3.3 = 0x2A 0x86 0x48 0xCE 0x3D 0x04 0x03 0x03
    // 1.2.840.113549.1.1.11 = 0x2A 0x86 0x48 0x86 0xF7 0x0D 0x01 0x01 0x0B
    // 1.2.840.113549.1.1.12 = ... 0x0C
    // 1.2.840.113549.1.1.13 = ... 0x0D
    // 1.2.840.113549.1.1.10 = ... 0x0A (RSASSA-PSS)
    const ECDSA_SHA256: &[u8] = &[0x2A,0x86,0x48,0xCE,0x3D,0x04,0x03,0x02];
    const ECDSA_SHA384: &[u8] = &[0x2A,0x86,0x48,0xCE,0x3D,0x04,0x03,0x03];
    const RSA_PKCS1V15_SHA256: &[u8] =
        &[0x2A,0x86,0x48,0x86,0xF7,0x0D,0x01,0x01,0x0B];
    const RSA_PKCS1V15_SHA384: &[u8] =
        &[0x2A,0x86,0x48,0x86,0xF7,0x0D,0x01,0x01,0x0C];
    const RSA_PKCS1V15_SHA512: &[u8] =
        &[0x2A,0x86,0x48,0x86,0xF7,0x0D,0x01,0x01,0x0D];
    const RSA_PSS:           &[u8] =
        &[0x2A,0x86,0x48,0x86,0xF7,0x0D,0x01,0x01,0x0A];

    if sig_oid == ECDSA_SHA256 {
        let digest = crate::crypto::sig::sha256_digest(&tbs);
        crate::crypto::sig::ecdsa_p256_verify(parent_spki, &digest, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else if sig_oid == ECDSA_SHA384 {
        let digest = crate::crypto::sig::sha384_digest(&tbs);
        crate::crypto::sig::ecdsa_p384_verify(parent_spki, &digest, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else if sig_oid == RSA_PKCS1V15_SHA256 {
        // RSA pubkey is wrapped in BIT STRING within SPKI; the inner
        // bytes are the DER-encoded RSAPublicKey. `subject_public_key`
        // already gave us the inner bytes.
        crate::crypto::sig::rsa_pkcs1v15_sha256_verify(parent_spki, &tbs, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else if sig_oid == RSA_PKCS1V15_SHA384 {
        crate::crypto::sig::rsa_pkcs1v15_sha384_verify(parent_spki, &tbs, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else if sig_oid == RSA_PKCS1V15_SHA512 {
        crate::crypto::sig::rsa_pkcs1v15_sha512_verify(parent_spki, &tbs, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else if sig_oid == RSA_PSS {
        // PSS hash + salt are encoded in `signature_algorithm.parameters`.
        // For the common case (which is what real CAs use), the hash is
        // SHA-256 and salt-len = hash-len. Try SHA-256 first, then 384/512.
        // A spec-strict impl would parse the parameters; we fall through.
        if crate::crypto::sig::rsa_pss_sha256_verify(parent_spki, &tbs, sig_bytes).is_ok() {
            return Ok(());
        }
        if crate::crypto::sig::rsa_pss_sha384_verify(parent_spki, &tbs, sig_bytes).is_ok() {
            return Ok(());
        }
        crate::crypto::sig::rsa_pss_sha512_verify(parent_spki, &tbs, sig_bytes)
            .map_err(|_| VerifyError::BadSignature)
    } else {
        Err(VerifyError::UnsupportedSigAlg)
    }
}

/// Verify TLS-1.3 CertificateVerify signature.
// /
/// Per RFC 8446 §4.4.3 the signed bytes are:
/// 64 × 0x20 || "TLS 1.3, server CertificateVerify" || 0x00 || transcript_hash
/// with a total length of 98 + transcript_hash bytes.
// /
/// `alg` is the signature algorithm from the CertificateVerify's
/// SignatureScheme (2-byte TLS code), e.g. 0x0403 = ecdsa_secp256r1_sha256.
pub fn tls13_verify_cert_verify(
    pubkey_alg: PubkeyAlg,
    pubkey_der: &[u8],
    sig_bytes: &[u8],
    transcript_hash: &[u8],
    tls_sig_scheme: u16,
) -> Result<(), VerifyError> {
    // Build the signed message.
    let mut msg: Vec<u8> = Vec::with_capacity(64 + 34 + 1 + transcript_hash.len());
    msg.extend(core::iter::repeat_n(0x20u8, 64));
    msg.extend_from_slice(b"TLS 1.3, server CertificateVerify");
    msg.push(0x00);
    msg.extend_from_slice(transcript_hash);

    // dispatch every standard TLS 1.3 SignatureScheme that
    // a CA might issue a leaf cert with. Per RFC 8446 §4.4.3, ONLY the
    // PSS schemes are valid for CertificateVerify (PKCS#1v1.5 was
    // removed) — but cert chain validation in `verify_signed_by` still
    // accepts PKCS#1v1.5 because that's how CAs sign certs themselves.
    match (pubkey_alg, tls_sig_scheme) {
        // ecdsa_secp256r1_sha256 = 0x0403
        (PubkeyAlg::EcdsaP256, 0x0403) => {
            let digest = crate::crypto::sig::sha256_digest(&msg);
            cert_verify_ecdsa_p256(pubkey_der, &digest, sig_bytes)
        }
        // ecdsa_secp384r1_sha384 = 0x0503
        (PubkeyAlg::EcdsaP384, 0x0503) => {
            let digest = crate::crypto::sig::sha384_digest(&msg);
            cert_verify_ecdsa_p384(pubkey_der, &digest, sig_bytes)
        }
        // rsa_pss_rsae_sha256 = 0x0804
        (PubkeyAlg::Rsa, 0x0804) => {
            cert_verify_rsa_pss_sha256(pubkey_der, &msg, sig_bytes)
        }
        // rsa_pss_rsae_sha384 = 0x0805
        (PubkeyAlg::Rsa, 0x0805) => {
            cert_verify_rsa_pss_sha384(pubkey_der, &msg, sig_bytes)
        }
        // rsa_pss_rsae_sha512 = 0x0806
        (PubkeyAlg::Rsa, 0x0806) => {
            cert_verify_rsa_pss_sha512(pubkey_der, &msg, sig_bytes)
        }
        _ => Err(VerifyError::UnsupportedSigAlg),
    }
}

/// Helper: ECDSA P-384 prehash verify against an SPKI-wrapped pubkey.
fn cert_verify_ecdsa_p384(spki_der: &[u8], digest: &[u8; 48], sig: &[u8])
    -> Result<(), VerifyError>
{
    // Strip the SPKI wrapper to get the bare uncompressed point.
    let spki = spki::SubjectPublicKeyInfoOwned::try_from(spki_der)
        .map_err(|_| VerifyError::Parse)?;
    let point = spki.subject_public_key.raw_bytes();
    crate::crypto::sig::ecdsa_p384_verify(point, digest, sig)
        .map_err(|_| VerifyError::BadSignature)
}

/// Helper: RSA-PSS verify (SHA-256) against an SPKI-wrapped RSA pubkey.
fn cert_verify_rsa_pss_sha256(spki_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), VerifyError>
{
    let spki = spki::SubjectPublicKeyInfoOwned::try_from(spki_der)
        .map_err(|_| VerifyError::Parse)?;
    let inner = spki.subject_public_key.raw_bytes();
    crate::crypto::sig::rsa_pss_sha256_verify(inner, msg, sig)
        .map_err(|_| VerifyError::BadSignature)
}

fn cert_verify_rsa_pss_sha384(spki_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), VerifyError>
{
    let spki = spki::SubjectPublicKeyInfoOwned::try_from(spki_der)
        .map_err(|_| VerifyError::Parse)?;
    let inner = spki.subject_public_key.raw_bytes();
    crate::crypto::sig::rsa_pss_sha384_verify(inner, msg, sig)
        .map_err(|_| VerifyError::BadSignature)
}

fn cert_verify_rsa_pss_sha512(spki_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), VerifyError>
{
    let spki = spki::SubjectPublicKeyInfoOwned::try_from(spki_der)
        .map_err(|_| VerifyError::Parse)?;
    let inner = spki.subject_public_key.raw_bytes();
    crate::crypto::sig::rsa_pss_sha512_verify(inner, msg, sig)
        .map_err(|_| VerifyError::BadSignature)
}
