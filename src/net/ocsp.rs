//! Online Certificate Status Protocol (OCSP) — RFC 6960.
//!
//! OCSP is the *online* revocation channel; the in-tree [`crl`]
//! module is the offline one. Where CRL says "here are 1000 revoked
//! serials under this issuer," OCSP says "this specific (issuer,
//! serial) is good / revoked / unknown" — fresher, much narrower.
//!
//! Design choices:
//!
//! - **In-memory status cache** keyed by `(issuer_key_hash[32],
//!   serial[≤20])` → `Status::{Good, Revoked, Unknown}`. Same
//!   sizing posture as the CRL store: 8 issuers × 256 serials each.
//! - **No fetching**. The auto-fetch path on validation is exactly
//!   the kind of side-effect we avoid; OCSP responses are
//!   operator-pushed (or future arc: stapled in the TLS handshake).
//! - **No in-tree signature verification** of the response. The
//!   caller is expected to verify the responder's certificate is
//!   signed by the trust anchor and that the response itself is
//!   signed by the responder — same delegation as the CRL module.
//!   `record_status` and `ingest_basic_response` are the entry
//!   points; the chain validator that calls them is the policy
//!   point.
//! - **CertID note**: OCSP keys revocation by `issuer_key_hash`
//!   (SHA-256 of the issuer's `subjectPublicKey` BIT STRING
//!   contents), NOT by the SHA-256-of-SubjectPublicKeyInfo we use
//!   in the CRL store. They are deliberately separate caches.
//!
//! API:
//!
//!     status(issuer_key_hash, serial) -> Option<Status>
//!     record_status(issuer_key_hash, serial, Status)
//!     ingest_basic_response(der_bytes) -> Result<usize, OcspError>
//!     stats() -> (issuers, entries)
//!
//! Constant-cost lookup: every slot scanned regardless of match.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use der::Decode;
use x509_ocsp::{BasicOcspResponse, CertStatus, OcspResponse, OcspResponseStatus};

const MAX_ISSUERS:     usize = 8;
const MAX_ENTRIES_PER: usize = 256;
const MAX_SERIAL_LEN:  usize = 20;

/// OCSP cert status as recorded in the cache. Maps 1:1 to RFC 6960
/// `CertStatus` choice, dropping `RevokedInfo` details (we only
/// gate trust on the revocation bit, not on the reason or time).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Good,
    Revoked,
    Unknown,
}

#[derive(Clone, Copy)]
struct Entry {
    serial_len: u8,
    serial: [u8; MAX_SERIAL_LEN],
    status: u8, // 0 = empty slot, 1 = Good, 2 = Revoked, 3 = Unknown
}

impl Entry {
    const fn empty() -> Self {
        Self {
            serial_len: 0,
            serial: [0u8; MAX_SERIAL_LEN],
            status: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct IssuerBucket {
    key_hash: [u8; 32],
    count: u16,
    entries: [Entry; MAX_ENTRIES_PER],
}

impl IssuerBucket {
    const fn empty() -> Self {
        Self {
            key_hash: [0u8; 32],
            count: 0,
            entries: [Entry::empty(); MAX_ENTRIES_PER],
        }
    }
}

static mut TABLE: [IssuerBucket; MAX_ISSUERS] = [IssuerBucket::empty(); MAX_ISSUERS];
static ISSUER_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub enum OcspError {
    Decode(&'static str),
    NotSuccessful,
    NotBasicResponse,
    IssuerKeyHashWrongLen,
    SerialTooLong,
    IssuerTableFull,
    BucketFull,
}

fn find_or_alloc_bucket(key_hash: &[u8; 32]) -> Result<usize, OcspError> {
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            if TABLE[i].key_hash == *key_hash {
                return Ok(i);
            }
        }
        if n >= MAX_ISSUERS {
            return Err(OcspError::IssuerTableFull);
        }
        TABLE[n].key_hash = *key_hash;
        ISSUER_COUNT.store(n + 1, Ordering::Relaxed);
        Ok(n)
    }
}

fn status_to_code(s: Status) -> u8 {
    match s {
        Status::Good => 1,
        Status::Revoked => 2,
        Status::Unknown => 3,
    }
}

fn code_to_status(c: u8) -> Option<Status> {
    match c {
        1 => Some(Status::Good),
        2 => Some(Status::Revoked),
        3 => Some(Status::Unknown),
        _ => None,
    }
}

/// Record a status for the `(issuer_key_hash, serial)` pair. Updates
/// the slot in place if it already exists (so a fresh response
/// overrides a stale one for the same cert).
pub fn record_status(
    issuer_key_hash: &[u8; 32],
    serial: &[u8],
    status: Status,
) -> Result<(), OcspError> {
    if serial.len() > MAX_SERIAL_LEN {
        return Err(OcspError::SerialTooLong);
    }
    let idx = find_or_alloc_bucket(issuer_key_hash)?;
    unsafe {
        let bucket = &mut TABLE[idx];
        // Update-in-place if the serial is already cached.
        for i in 0..bucket.count as usize {
            let e = &mut bucket.entries[i];
            if e.serial_len as usize == serial.len()
                && &e.serial[..serial.len()] == serial
            {
                e.status = status_to_code(status);
                return Ok(());
            }
        }
        if (bucket.count as usize) >= MAX_ENTRIES_PER {
            return Err(OcspError::BucketFull);
        }
        let n = bucket.count as usize;
        bucket.entries[n].serial_len = serial.len() as u8;
        bucket.entries[n].serial[..serial.len()].copy_from_slice(serial);
        bucket.entries[n].status = status_to_code(status);
        bucket.count += 1;
    }
    Ok(())
}

/// Look up the cached status for `(issuer_key_hash, serial)`.
/// Returns `None` if nothing has been recorded — the caller's
/// policy (typically fail-soft for unknown OCSP, fail-closed for
/// the chain anchor) decides what that means.
///
/// Constant-cost: walks every entry in every bucket regardless of
/// match, so a timing observer can't tell which (if any) slot
/// matched. Falls through to the kernel-cache discipline used by
/// the CRL counterpart.
pub fn status(issuer_key_hash: &[u8; 32], serial: &[u8]) -> Option<Status> {
    if serial.len() > MAX_SERIAL_LEN {
        return None;
    }
    let mut matched: u8 = 0;       // 0 = no match, else status code
    let mut any_match: u8 = 0;
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            let issuer_eq = if TABLE[i].key_hash == *issuer_key_hash { 1u8 } else { 0u8 };
            let bucket = &TABLE[i];
            for j in 0..bucket.count as usize {
                let e = &bucket.entries[j];
                let mut len_eq = 0u8;
                if e.serial_len as usize == serial.len() {
                    let mut diff = 0u8;
                    for k in 0..serial.len() {
                        diff |= e.serial[k] ^ serial[k];
                    }
                    if diff == 0 { len_eq = 1; }
                }
                let entry_match = issuer_eq & len_eq;
                // any_match | (entry_match * 1)
                any_match |= entry_match;
                // matched takes the status code from the matching slot
                matched |= e.status & (0u8.wrapping_sub(entry_match));
            }
        }
    }
    if any_match == 0 { None } else { code_to_status(matched) }
}

/// Parse a DER-encoded `OCSPResponse` (RFC 6960 §4.2.1), extract
/// every `SingleResponse` in the inner `BasicOCSPResponse`, and
/// record each `(issuerKeyHash, serialNumber) → status` mapping
/// into the cache. Returns the number of `SingleResponse` entries
/// successfully recorded.
///
/// NOTE: signature verification of the OCSP response (and of the
/// responder's certificate chain back to the trust anchor) is the
/// CALLER's responsibility. Same delegation discipline as the CRL
/// module.
pub fn ingest_basic_response(der_bytes: &[u8]) -> Result<usize, OcspError> {
    let resp = OcspResponse::from_der(der_bytes)
        .map_err(|_| OcspError::Decode("der parse"))?;
    if resp.response_status != OcspResponseStatus::Successful {
        return Err(OcspError::NotSuccessful);
    }
    let rb = match resp.response_bytes {
        Some(r) => r,
        None => return Err(OcspError::NotBasicResponse),
    };
    let basic = BasicOcspResponse::from_der(rb.response.as_bytes())
        .map_err(|_| OcspError::Decode("basic-response der parse"))?;

    let mut recorded = 0usize;
    for sr in basic.tbs_response_data.responses.iter() {
        let key_bytes = sr.cert_id.issuer_key_hash.as_bytes();
        if key_bytes.len() != 32 {
            return Err(OcspError::IssuerKeyHashWrongLen);
        }
        let mut key_hash = [0u8; 32];
        key_hash.copy_from_slice(key_bytes);
        let serial_bytes = sr.cert_id.serial_number.as_bytes();
        let status = match sr.cert_status {
            CertStatus::Good(_) => Status::Good,
            CertStatus::Revoked(_) => Status::Revoked,
            CertStatus::Unknown(_) => Status::Unknown,
        };
        record_status(&key_hash, serial_bytes, status)?;
        recorded += 1;
    }
    Ok(recorded)
}

/// `(issuer_buckets_in_use, total_entries_recorded)` for diagnostics.
pub fn stats() -> (usize, usize) {
    unsafe {
        let n = ISSUER_COUNT.load(Ordering::Relaxed);
        let total: usize = (0..n).map(|i| TABLE[i].count as usize).sum();
        (n, total)
    }
}

/// Test-only reset.
#[cfg(test)]
pub fn reset() {
    unsafe { TABLE = [IssuerBucket::empty(); MAX_ISSUERS]; }
    ISSUER_COUNT.store(0, Ordering::Relaxed);
}
