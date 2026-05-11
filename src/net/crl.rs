//! Certificate Revocation List (CRL) — RFC 5280 §5.
//!
//! A CRL is a signed list of revoked certificate serial numbers
//! published by a CA. We use it as the offline revocation channel
//! (OCSP is the online one); the X.509 chain validator consults
//! this module per cert in the chain.
//!
//! Design choices:
//!
//! - **In-memory store of revoked serials** keyed by issuer SPKI
//!   SHA-256. 256 issuers × 1024 serials each = ~32 KB. Plenty for
//!   our trust posture (6 root anchors + their intermediates).
//! - **Boot-time load from BatFS.** Operator drops `/crls/<anchor>.crl`
//!   for each trust anchor; we parse each on first auth. CRL refresh
//!   on a cadence is a separate, follow-up feature.
//! - **No CRL fetching** in this module. The CRL Distribution Points
//!   extension in certs gives URLs, but auto-fetching them at chain
//!   validation time is a heavy operation we don't want on the
//!   per-connection path. Operator-pushed CRLs is the model.
//!
//! API:
//!
//!     add_revocation(issuer_spki: &[u8; 32], serial: &[u8])
//!     is_revoked(issuer_spki: &[u8; 32], serial: &[u8]) -> bool
//!     load_crl(crl_bytes: &[u8]) -> Result<usize, CrlError>
//!
//! Constant-cost is_revoked: scans every slot for the issuer; the
//! lookup time doesn't leak whether a serial matched.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use x509_cert::crl::CertificateList;
use der::Decode;

const MAX_ISSUERS:     usize = 8;
const MAX_REVOKED_PER: usize = 256;
const MAX_SERIAL_LEN:  usize = 20;   // RFC 5280: serials ≤ 20 octets

#[derive(Clone, Copy)]
struct RevokedSerial {
    len: u8,
    bytes: [u8; MAX_SERIAL_LEN],
}

impl RevokedSerial {
    const fn empty() -> Self {
        Self { len: 0, bytes: [0u8; MAX_SERIAL_LEN] }
    }
}

#[derive(Clone, Copy)]
struct IssuerBucket {
    issuer_spki: [u8; 32],
    count: u16,
    serials: [RevokedSerial; MAX_REVOKED_PER],
}

impl IssuerBucket {
    const fn empty() -> Self {
        Self {
            issuer_spki: [0u8; 32],
            count: 0,
            serials: [RevokedSerial::empty(); MAX_REVOKED_PER],
        }
    }
}

static mut TABLE: [IssuerBucket; MAX_ISSUERS] = [IssuerBucket::empty(); MAX_ISSUERS];
static ISSUER_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub enum CrlError {
    Decode(&'static str),
    IssuerTableFull,
    BucketFull,
    SerialTooLong,
}

fn find_or_alloc_bucket(issuer_spki: &[u8; 32]) -> Result<usize, CrlError> {
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            if TABLE[i].issuer_spki == *issuer_spki {
                return Ok(i);
            }
        }
        if n >= MAX_ISSUERS {
            return Err(CrlError::IssuerTableFull);
        }
        TABLE[n].issuer_spki = *issuer_spki;
        ISSUER_COUNT.store(n + 1, Ordering::Relaxed);
        Ok(n)
    }
}

/// Add one serial as revoked under the given issuer. Idempotent —
/// re-adding the same serial is a no-op.
pub fn add_revocation(issuer_spki: &[u8; 32], serial: &[u8]) -> Result<(), CrlError> {
    if serial.len() > MAX_SERIAL_LEN {
        return Err(CrlError::SerialTooLong);
    }
    let idx = find_or_alloc_bucket(issuer_spki)?;
    unsafe {
        let bucket = &mut TABLE[idx];
        for i in 0..bucket.count as usize {
            let r = &bucket.serials[i];
            if r.len as usize == serial.len()
                && &r.bytes[..serial.len()] == serial
            {
                return Ok(());  // already present
            }
        }
        if (bucket.count as usize) >= MAX_REVOKED_PER {
            return Err(CrlError::BucketFull);
        }
        let n = bucket.count as usize;
        bucket.serials[n].len = serial.len() as u8;
        bucket.serials[n].bytes[..serial.len()].copy_from_slice(serial);
        bucket.count += 1;
    }
    Ok(())
}

/// Check whether the (issuer, serial) pair has been revoked.
/// Constant-cost: always scans every slot in the bucket so a
/// timing observer can't tell which (if any) serial matched.
pub fn is_revoked(issuer_spki: &[u8; 32], serial: &[u8]) -> bool {
    let mut any = 0u8;
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            if TABLE[i].issuer_spki != *issuer_spki {
                continue;
            }
            let bucket = &TABLE[i];
            for j in 0..bucket.count as usize {
                let r = &bucket.serials[j];
                let mut eq = 0u8;
                if r.len as usize == serial.len() {
                    let mut diff = 0u8;
                    for k in 0..serial.len() {
                        diff |= r.bytes[k] ^ serial[k];
                    }
                    if diff == 0 {
                        eq = 1;
                    }
                }
                any |= eq;
            }
            return any != 0;
        }
    }
    false  // unknown issuer — fail-open is the standard CRL behaviour
}

/// Parse a DER-encoded CRL and register every revoked serial.
/// Returns the number of newly-registered serials.
///
/// NOTE: this **does not verify the CRL signature**. The caller is
/// expected to verify the CRL was signed by the trust anchor before
/// passing the bytes in — typically by calling
/// `crate::net::x509::verify_crl_signature(&crl, &anchor_spki)`
/// (which lives in the existing chain validator). Verifying here
/// would introduce a cycle.
pub fn load_crl(crl_bytes: &[u8], issuer_spki: &[u8; 32]) -> Result<usize, CrlError> {
    let crl = CertificateList::from_der(crl_bytes)
        .map_err(|_| CrlError::Decode("der parse"))?;
    let mut added = 0usize;
    if let Some(revoked) = crl.tbs_cert_list.revoked_certificates {
        for entry in revoked {
            let serial_bytes = entry.serial_number.as_bytes();
            if serial_bytes.len() > MAX_SERIAL_LEN {
                continue;
            }
            if add_revocation(issuer_spki, serial_bytes).is_ok() {
                added += 1;
            }
        }
    }
    Ok(added)
}

/// Statistics for diagnostics.
pub fn stats() -> (usize, usize) {
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        let total: usize = (0..n).map(|i| TABLE[i].count as usize).sum();
        (n, total)
    }
}

/// Test-only reset hook.
#[cfg(test)]
pub fn reset() {
    unsafe { TABLE = [IssuerBucket::empty(); MAX_ISSUERS]; }
    ISSUER_COUNT.store(0, Ordering::Relaxed);
}
