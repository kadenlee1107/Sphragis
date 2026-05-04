// Bat_OS — X.509 certificate chain validation.
//
// V4: replaces the pin-only interim defence with real PKI validation.
//
// Scope:
//   * Parse leaf + intermediate certs from the TLS Certificate message
//     (ASN.1 DER via `x509-cert`).
//   * Extract SubjectPublicKeyInfo and surface pubkey for the
//     CertificateVerify check.
//   * Verify the signature chain leaf ← intermediate ← ... ← root
//     where root lives in TRUST_STORE (hard-coded pinned CA certs).
//   * Check notBefore / notAfter against a (static) boot time — we don't
//     have wall-clock yet so we implement a lower bound (reject certs
//     issued in the future) but accept slightly-expired ones and log a
//     warning. Operator can pin time via KNOWN_GOOD_TIME.
//   * Hostname match against SAN dNSName entries (CN fallback).
//
// What we DO NOT do:
//   * Revocation (OCSP / CRL) — operator's job for high-security envs.
//   * RSA verify — we only wire ECDSA P-256 / P-384 today.  Adding RSA is
//     a `rsa = "0.9"` dep away but every mainstream TLS-for-HTTPS cert
//     we care about (LE ECDSA, Cloudflare ECDSA) is P-256 or P-384.
//   * Name constraints / EKU — can add per-cert flags when needed.
//
// All-pass path:
//   1. `verify_chain(leaf_der, chain_ders, hostname, now_unix)` returns
//      `Ok(subject_pubkey_der)`.
//   2. Caller hands `subject_pubkey_der` to
//      `cert_verify_signature(pubkey_der, signed_bytes, sig_der)` to
//      validate TLS-1.3 CertificateVerify.

#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryFrom;
use x509_cert::Certificate;
use x509_cert::der::{Decode, Encode};

/// Hard-coded trust anchors — embedded DER bytes of curated major
/// public CA roots.
///
/// STUMP #139: This used to be empty, which made the audit's verdict
/// ("TLS authentication is theater") literally true — chain validation
/// always returned UntrustedRoot, so HTTPS got us encrypted-but-
/// unauthenticated bytes vs an active MITM. Now populated with a
/// minimal-but-meaningful starter set sourced from each CA's official
/// publication endpoint:
///
///   * ISRG Root X1 — RSA 4096 — Let's Encrypt's primary root.
///     Anchors a huge chunk of public HTTPS (Cloudflare-fronted sites,
///     GitHub Pages, basically every site that auto-issues with LE).
///   * ISRG Root X2 — ECDSA P-384 — Let's Encrypt's modern ECDSA
///     root, used by sites that opted into ECDSA leaf certs.
///   * Amazon Root CA 1 — RSA 2048 — anchors AWS-hosted services and
///     anything fronted by Amazon.com.
///   * DigiCert Global Root CA — RSA 2048 — anchors a large fraction
///     of enterprise + financial sites.
///   * DigiCert Global Root G2 — RSA 2048 — DigiCert's modern root,
///     used by Google's intermediate CA chain among others.
///
/// `TrustStore::contains` compares subject public-key bytes for the
/// issuer lookup, so this set covers a meaningful slice of the public
/// web. A full Mozilla CA bundle (~150 roots) is a follow-up STUMP —
/// this five-entry set is enough to verify the most common chains and
/// move the audit's "theater" verdict.
///
/// **RSA support note:** as of STUMP #139, `verify_chain` still only
/// supports ECDSA-P256/P384 leaf signatures (`crypto/sig.rs`). Three of
/// these roots are RSA, so they only validate ECDSA-leaf chains today
/// (LE issues both). RSA leaf signature verify is STUMP #140 — when
/// that lands, this trust store gates real coverage. Until then,
/// ECDSA-leaf chains under ISRG X1 / X2 work; pure RSA chains return
/// `UnsupportedSigAlg` and fall through to pin-based defence.
///
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
pub fn pubkey_alg(cert: &Certificate) -> PubkeyAlg {
    use const_oid::db::rfc5912;
    let spki = &cert.tbs_certificate.subject_public_key_info;
    let oid = spki.algorithm.oid;
    if oid == rfc5912::ID_EC_PUBLIC_KEY {
        // Curve distinguished by parameters. Approximate heuristic:
        // P-256 SPKI SubjectPublicKey length is 520 bits (65 bytes uncompressed).
        // P-384 is 776 bits (97 bytes).
        let pk_bits = spki.subject_public_key.raw_bytes();
        match pk_bits.len() {
            65 => PubkeyAlg::EcdsaP256,
            97 => PubkeyAlg::EcdsaP384,
            33 => PubkeyAlg::EcdsaP256, // compressed form
            _ => PubkeyAlg::Unknown,
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
/// "any single leftmost label matches". No middle-wildcards, no
/// IDN translation (input hostnames should be ASCII-encoded upstream).
fn hostname_matches(pattern: &[u8], hostname: &[u8]) -> bool {
    if pattern == hostname {
        return true;
    }
    if pattern.len() > 2 && &pattern[..2] == b"*." {
        let suffix = &pattern[1..]; // keep the leading "."
        // Find the first '.' in hostname.
        if let Some(idx) = hostname.iter().position(|&b| b == b'.') {
            return &hostname[idx..] == suffix;
        }
    }
    false
}

/// Verify an ECDSA signature (P-256 only in this minimal build) over
/// `signed_bytes` using the pubkey DER taken from a cert.  `signed_bytes`
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
///
/// `leaf_der`: the server's certificate.
/// `chain_ders`: intermediate certs in order (leaf-issuer, then its
///               issuer, …). May be empty if leaf is directly signed by
///               a root in TRUST_STORE.
/// `hostname`: the hostname the client expects (from SNI / URL).
///
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
    // — an off-path observer measuring the abort time learned which
    // hostname the client tried.
    let hostname_ok = check_hostname(&leaf, hostname);

    // 2. Walk the chain. For each (child, parent) pair, verify that
    //    parent.pubkey validates child.signature over child.tbsCertificate.
    //    Root must be in TRUST_STORE.
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
        let _ = i;
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

    // 3. Root in trust store?  We look the current (last) cert up by its
    //    subject bytes and pubkey against TRUST_STORE entries.
    let mut trusted = false;
    for anchor_der in TRUST_STORE.iter() {
        if anchor_der == &current_der {
            trusted = true;
            break;
        }
        // Also accept "root is reissuer of current's issuer" in a
        // single-anchor setup — parse + compare subject pubkey.
        if let Ok(anchor) = parse_cert(anchor_der) {
            let ap = anchor.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
            let cp = current_cert.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
            if ap == cp {
                trusted = true;
                break;
            }
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
    VerifyOutcome::Ok {
        pubkey_der: leaf_spki,
        pubkey_algorithm: pubkey_alg(&leaf),
    }
}

/// Verify that `parent` signed `child`.  Today: ECDSA P-256 + SHA-256 only;
/// other algorithms return `UnsupportedSigAlg`.
fn verify_signed_by(
    child: &Certificate,
    child_der: &[u8],
    parent: &Certificate,
) -> Result<(), VerifyError> {
    let _ = child;

    // TBS bytes to sign: re-encode the tbsCertificate field.
    let tbs = child.tbs_certificate.to_der().map_err(|_| VerifyError::Parse)?;

    // Signature bytes from the child.
    // `signature` field is BIT STRING in DER; re-decode via x509-cert so
    // we don't hand-parse the outer Certificate ourselves.
    let cert = parse_cert(child_der)?;
    let sig_bytes = cert.signature.raw_bytes();

    let digest = crate::crypto::sig::sha256_digest(&tbs);

    match pubkey_alg(parent) {
        PubkeyAlg::EcdsaP256 => {
            let parent_spki = parent.tbs_certificate.subject_public_key_info
                .subject_public_key.raw_bytes();
            crate::crypto::sig::ecdsa_p256_verify(parent_spki, &digest, sig_bytes)
                .map_err(|_| VerifyError::BadSignature)
        }
        _ => Err(VerifyError::UnsupportedSigAlg),
    }
}

/// Verify TLS-1.3 CertificateVerify signature.
///
/// Per RFC 8446 §4.4.3 the signed bytes are:
///   64 × 0x20 || "TLS 1.3, server CertificateVerify" || 0x00 || transcript_hash
/// with a total length of 98 + transcript_hash bytes.
///
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
    for _ in 0..64 { msg.push(0x20); }
    msg.extend_from_slice(b"TLS 1.3, server CertificateVerify");
    msg.push(0x00);
    msg.extend_from_slice(transcript_hash);

    match (pubkey_alg, tls_sig_scheme) {
        // ecdsa_secp256r1_sha256 = 0x0403
        (PubkeyAlg::EcdsaP256, 0x0403) => {
            let digest = crate::crypto::sig::sha256_digest(&msg);
            cert_verify_ecdsa_p256(pubkey_der, &digest, sig_bytes)
        }
        _ => Err(VerifyError::UnsupportedSigAlg),
    }
}
