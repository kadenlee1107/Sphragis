//! WORM (Write-Once-Read-Many) audit segment export to SealFS.
//!
//! Per `DESIGN_AUDIT_WORM.md` (SP-AUD-002). Closes audit finding
//! FS-H7. Sealed segments are persisted to SealFS as
//! `audit/worm/segment-NNNNNNNNNN.bin` and an operator anchor lives
//! at `audit/worm/LATEST_SEAL.bin`. Each sealed segment carries an
//! HMAC-SHA-384 trailer chained to the previous segment's head hash;
//! the verifier (`tools/audit-verifier/audit_verifier.py --worm-dir`)
//! walks segments forward and detects truncation, modification, and
//! missing-segment attacks.
//!
//! Substrate-honesty note: SealFS today exposes only `create()` /
//! `read()` semantics (no append, no fsync). The WORM property here
//! is "once sealed, a segment is never overwritten by the kernel".
//! An attacker who can write SealFS can still corrupt sealed files,
//! but cannot forge new valid chain links without the kernel-only
//! `AUDIT_HMAC_KEY`. The off-platform anchor (operator copies
//! `LATEST_SEAL.bin` periodically) closes the rewind/truncation
//! attack window.

#![allow(static_mut_refs)]

use crate::security::audit::{Entry, MSG_LEN};
use crate::security::audit_chain::CHAIN_HASH_LEN;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub const SEGMENT_CAP_BYTES: usize = 64 * 1024;
pub const HEADER_LEN: usize = 32;                           // magic(24) + seq(8)
pub const RECORD_LEN: usize = 8 + 1 + 1 + 2 + MSG_LEN;      // ts + cat + mlen + cave_id + msg
pub const TRAILER_LEN: usize = 24 + 8 + CHAIN_HASH_LEN + 8; // seal_magic + count + hash + prev_first8 = 88

pub const SEGMENT_MAGIC: &[u8; 24] = b"SPHRAGIS_WORM_SEGMENT_V1";
pub const SEAL_MAGIC: &[u8; 24] = b"WORM_SEGMENT_SEAL_V1\0\0\0\0";
pub const LATEST_MAGIC: &[u8; 24] = b"SPHRAGIS_WORM_LATEST_V1\0";

static mut SEGMENT_BUF: [u8; SEGMENT_CAP_BYTES] = [0u8; SEGMENT_CAP_BYTES];
static SEGMENT_LEN: AtomicUsize = AtomicUsize::new(0);
static SEGMENT_RECORDS: AtomicU64 = AtomicU64::new(0);
static SEGMENT_SEQ: AtomicU64 = AtomicU64::new(0);
static INITED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
static mut PREV_HEAD_HASH: [u8; CHAIN_HASH_LEN] = [0u8; CHAIN_HASH_LEN];

/// Initialize WORM state. Sets up segment 1 with header written. Safe
/// to call repeatedly; only the first call takes effect.
pub fn init() {
    if INITED.swap(true, Ordering::AcqRel) { return; }
    init_segment_header(1);
}

fn init_segment_header(seq: u64) {
    unsafe {
        SEGMENT_BUF[..24].copy_from_slice(SEGMENT_MAGIC);
        SEGMENT_BUF[24..32].copy_from_slice(&seq.to_be_bytes());
    }
    SEGMENT_LEN.store(HEADER_LEN, Ordering::Release);
    SEGMENT_RECORDS.store(0, Ordering::Release);
    SEGMENT_SEQ.store(seq, Ordering::Release);
}

/// Append a single audit entry to the current WORM segment. If the
/// segment is full, it's sealed (flushed to SealFS) and a new one
/// started. Failure modes: SealFS write errors during auto-seal
/// propagate as `Err`. Caller (audit::record) ignores errors today —
/// the in-RAM ring + chain remain the source of truth for live audit.
pub fn worm_append(entry: &Entry) -> Result<(), &'static str> {
    if !INITED.load(Ordering::Acquire) { return Ok(()); }
    let len = SEGMENT_LEN.load(Ordering::Acquire);
    if len + RECORD_LEN + TRAILER_LEN > SEGMENT_CAP_BYTES {
        worm_seal_current()?;
    }
    let len = SEGMENT_LEN.load(Ordering::Acquire);
    unsafe {
        let buf = &mut SEGMENT_BUF;
        buf[len..len + 8].copy_from_slice(&entry.ts.to_be_bytes());
        buf[len + 8] = entry.cat;
        buf[len + 9] = entry.mlen;
        buf[len + 10..len + 12].copy_from_slice(&entry.cave_id.to_be_bytes());
        buf[len + 12..len + 12 + MSG_LEN].copy_from_slice(&entry.msg);
    }
    SEGMENT_LEN.fetch_add(RECORD_LEN, Ordering::Release);
    SEGMENT_RECORDS.fetch_add(1, Ordering::Release);
    Ok(())
}

/// Seal the current segment: compute HMAC-SHA-384 over (seq ||
/// prev_head_hash || records), write trailer, flush to SealFS at
/// `audit/worm/segment-NNNNNNNNNN.bin`. Rotates to seq+1 with a
/// fresh header. Silent no-op if the current segment has no records
/// (avoids creating empty sealed files at boot).
pub fn worm_seal_current() -> Result<(), &'static str> {
    let len = SEGMENT_LEN.load(Ordering::Acquire);
    let rec_count = SEGMENT_RECORDS.load(Ordering::Acquire);
    let seq = SEGMENT_SEQ.load(Ordering::Acquire);
    if rec_count == 0 { return Ok(()); }

    let mut key = crate::security::audit_chain::audit_hmac_key_copy();
    let head_hash = compute_segment_hash(&key, seq, len);
    crate::security::zeroize::zeroize(&mut key);

    let prev_first8: [u8; 8] = {
        let mut p = [0u8; 8];
        unsafe { p.copy_from_slice(&PREV_HEAD_HASH[..8]); }
        p
    };

    unsafe {
        let buf = &mut SEGMENT_BUF;
        let t = len;
        buf[t..t + 24].copy_from_slice(SEAL_MAGIC);
        buf[t + 24..t + 32].copy_from_slice(&rec_count.to_be_bytes());
        buf[t + 32..t + 32 + CHAIN_HASH_LEN].copy_from_slice(&head_hash);
        buf[t + 32 + CHAIN_HASH_LEN..t + 32 + CHAIN_HASH_LEN + 8].copy_from_slice(&prev_first8);
    }
    let total = len + TRAILER_LEN;

    let mut name = [0u8; 40];
    let n = format_segment_name(&mut name, seq);
    let nm = core::str::from_utf8(&name[..n]).map_err(|_| "worm: name utf8")?;
    let payload = unsafe { &SEGMENT_BUF[..total] };
    crate::fs::sealfs::create(nm, payload).map_err(|_| "worm: sealfs create segment failed")?;

    let mut latest = [0u8; HEADER_LEN + CHAIN_HASH_LEN + 8];
    latest[..24].copy_from_slice(LATEST_MAGIC);
    latest[24..32].copy_from_slice(&seq.to_be_bytes());
    latest[32..32 + CHAIN_HASH_LEN].copy_from_slice(&head_hash);
    latest[32 + CHAIN_HASH_LEN..32 + CHAIN_HASH_LEN + 8].copy_from_slice(&prev_first8);
    crate::fs::sealfs::create("audit/worm/LATEST_SEAL.bin", &latest)
        .map_err(|_| "worm: sealfs create LATEST_SEAL failed")?;

    unsafe { PREV_HEAD_HASH = head_hash; }
    init_segment_header(seq + 1);
    Ok(())
}

/// HMAC-SHA-384 over (seq_be8 || PREV_HEAD_HASH || record_bytes).
/// `record_bytes` is the slice from HEADER_LEN..len in SEGMENT_BUF.
fn compute_segment_hash(key: &[u8], seq: u64, len: usize) -> [u8; CHAIN_HASH_LEN] {
    let records_len = len - HEADER_LEN;
    let mut buf = [0u8; 8 + CHAIN_HASH_LEN + SEGMENT_CAP_BYTES];
    buf[..8].copy_from_slice(&seq.to_be_bytes());
    unsafe {
        buf[8..8 + CHAIN_HASH_LEN].copy_from_slice(&PREV_HEAD_HASH);
        buf[8 + CHAIN_HASH_LEN..8 + CHAIN_HASH_LEN + records_len]
            .copy_from_slice(&SEGMENT_BUF[HEADER_LEN..len]);
    }
    crate::crypto::sha384::hmac(key, &buf[..8 + CHAIN_HASH_LEN + records_len])
}

fn format_segment_name(out: &mut [u8], seq: u64) -> usize {
    let prefix = b"audit/worm/segment-";
    out[..prefix.len()].copy_from_slice(prefix);
    let mut n = seq;
    let mut digits = [0u8; 10];
    for i in (0..10).rev() {
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out[prefix.len()..prefix.len() + 10].copy_from_slice(&digits);
    let suffix = b".bin";
    out[prefix.len() + 10..prefix.len() + 10 + suffix.len()].copy_from_slice(suffix);
    prefix.len() + 10 + suffix.len()
}

/// Operator-facing status: (current_seq, records_in_current, prev_head_first8).
pub fn worm_status() -> (u64, u64, [u8; 8]) {
    let seq = SEGMENT_SEQ.load(Ordering::Acquire);
    let recs = SEGMENT_RECORDS.load(Ordering::Acquire);
    let mut p = [0u8; 8];
    unsafe { p.copy_from_slice(&PREV_HEAD_HASH[..8]); }
    (seq, recs, p)
}
