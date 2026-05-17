//! Tamper-evident hash chain over the audit ring.
//!
//! Goal: make it computationally infeasible for an attacker (even one
//! who has memory write access to the ring) to delete or modify a past
//! audit entry without changing every subsequent entry's hash. That
//! turns silent log tampering into a detectable event.
//!
//! Mechanism: maintain a parallel array of 32-byte hashes alongside
//! the audit ring. When entry `i` is written, we compute
//!
//!     CHAIN[i % CAP] = HMAC-SHA-384(AUDIT_HMAC_KEY,
//!                                   CHAIN[(i-1) % CAP] || entry_canonical_bytes(i))
//!
//! where `entry_canonical_bytes(i)` is `ts || cat || mlen || cave_id || msg[..mlen]`
//! — a deterministic byte serialization of the public fields.
//!
//! SP-C4.1 (2026-05-16): upgraded from HMAC-SHA-256 (32-byte chain
//! + 40-byte ChainSeal) to HMAC-SHA-384 (48-byte chain + 56-byte
//! ChainSeal) per CNSA 2.0 alignment. In-place swap — no dual-chain
//! transition window since no production deployment has off-platform
//! seal files in maintenance yet. Pre-SP-C4.1 on-disk seal files
//! (40 bytes) are unverifiable under the new schema; operator
//! should re-seal after upgrade.
//!
//! A verifier later walks the ring head -> tail, recomputes each
//! hash from the previous, and aborts at the first mismatch. The
//! offset of the first mismatch tells the operator how far back the
//! tampering reaches.
//!
//! Limitations (documented for the future):
//!
//! - Sealing the *latest* hash off-platform (TPM, hardware key, paper
//!   QR code) is what makes the chain detect head-truncation
//!   attacks. Without an external anchor an attacker who can rewrite
//!   the whole ring can also rewrite the chain. This module gives
//!   you the chain; cluster F adds the seal.
//!
//! - On rollover (HEAD > RING_CAP) the oldest entries get evicted.
//!   Their hashes are lost too. An off-platform anchor in long-running
//!   deployments needs to be checkpointed every few hundred entries
//!   so historical tampering is bounded.

#![allow(dead_code)]

use core::sync::atomic::Ordering;

use crate::crypto::sha384;
use crate::security::audit::{Entry, MSG_LEN, RING_CAP, HEAD};

/// SP-C4.1 (2026-05-16): SHA-384 chain output size (was 32 for
/// SHA-256). Public so callers (chain_head, ChainSeal) can refer to
/// it without re-deriving.
pub const CHAIN_HASH_LEN: usize = 48;

/// Storage for the per-entry chain hashes. Index `i` mirrors the
/// audit ring's slot `i`. Slot 0 chains from the all-zero genesis.
static mut CHAIN: [[u8; CHAIN_HASH_LEN]; RING_CAP] = [[0u8; CHAIN_HASH_LEN]; RING_CAP];

/// AUDIT-CAVE-M1 (2026-05-15): kernel-only HMAC key for chaining.
/// Seeded from RNDR at boot via `init_audit_key()`. Until that runs
/// the key is zero, which still produces a deterministic chain —
/// but a tamperer who can write the static can't forge entries
/// without knowing the key. Once SEP / a sealed storage primitive
/// lands, this should be sourced from there so kernel-write alone
/// is insufficient.
///
/// SP-C4.1: key length grew 32 -> 48 bytes alongside the HMAC
/// upgrade SHA-256 -> SHA-384. HMAC accepts any key length but a
/// key sized to the inner-hash output is conventional.
static mut AUDIT_HMAC_KEY: [u8; CHAIN_HASH_LEN] = [0u8; CHAIN_HASH_LEN];
static AUDIT_KEY_READY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

/// Initialize the audit-chain HMAC key. Should be called once at
/// boot, after crypto::rng::probe_hw_rng. Safe to call again — only
/// the first call seeds the key, subsequent calls are no-ops.
pub fn init_audit_key() {
    use core::sync::atomic::Ordering;
    if AUDIT_KEY_READY.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe {
        let k = core::ptr::addr_of_mut!(AUDIT_HMAC_KEY);
        crate::crypto::rng::fill_bytes(&mut *k);
    }
}

/// Canonicalize an entry for hashing. Fixed-width prefix +
/// variable-length message body. Big-endian for portability.
/// AUDIT-CAVE-M3: cave_id (2 bytes big-endian) is part of the
/// canonical form so the chain covers entry provenance — tampering
/// with cave_id alone now breaks the chain.
fn canonical_bytes(entry: &Entry, out: &mut [u8; CHAIN_HASH_LEN + MSG_LEN]) -> usize {
    // 8 + 1 + 1 + 2 = 12 byte fixed prefix, then up to MSG_LEN body.
    let ts_be = entry.ts.to_be_bytes();
    out[..8].copy_from_slice(&ts_be);
    out[8] = entry.cat;
    out[9] = entry.mlen;
    let cid_be = entry.cave_id.to_be_bytes();
    out[10] = cid_be[0];
    out[11] = cid_be[1];
    let mlen = entry.mlen as usize;
    out[12..12 + mlen].copy_from_slice(&entry.msg[..mlen]);
    12 + mlen
}

/// Return a copy of the AUDIT_HMAC_KEY. Intended for in-kernel
/// modules that need to HMAC audit-derived material (e.g. the WORM
/// segment-chain in `audit_worm`). The caller MUST zeroize the copy
/// after use. Not exported beyond the kernel.
pub(crate) fn audit_hmac_key_copy() -> [u8; CHAIN_HASH_LEN] {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(AUDIT_HMAC_KEY)) }
}

/// Update the chain hash for an entry at slot `slot` after a record.
/// Called from `audit::record` after the entry is in place.
///
/// `head` is the **absolute index** of THIS entry — for the n-th
/// entry recorded since boot, `head == n`. This is the value
/// `HEAD.fetch_add(1)` returns (i.e. the OLD count, before the
/// increment landed). Concretely: the first entry passes
/// `head == 0` and inherits the all-zero genesis hash; the second
/// entry passes `head == 1` and chains off `CHAIN[0]`.
///
/// SAFETY: caller must hold the same exclusion the audit ring assumes
/// (currently: single-writer in main thread).
pub unsafe fn append_chain(slot: usize, entry: &Entry, head: usize) {
    let mut canon = [0u8; CHAIN_HASH_LEN + MSG_LEN];
    let n = canonical_bytes(entry, &mut canon);

    let prev = if head == 0 {
        [0u8; CHAIN_HASH_LEN]
    } else {
        let prev_slot = (head - 1) % RING_CAP;
        unsafe { CHAIN[prev_slot] }
    };

    let mut buf = [0u8; CHAIN_HASH_LEN + CHAIN_HASH_LEN + MSG_LEN];
    buf[..CHAIN_HASH_LEN].copy_from_slice(&prev);
    buf[CHAIN_HASH_LEN..CHAIN_HASH_LEN + n].copy_from_slice(&canon[..n]);
    // SP-C4.1 (2026-05-16): HMAC-SHA-384 (was SHA-256). CNSA 2.0
    // mandates SHA-384/512 for new signing/MAC use; the chain MAC
    // counts as MAC use. Migration is in-place (no dual-chain
    // transition window); pre-SP-C4.1 on-disk seals are
    // unverifiable under the new schema.
    let key = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(AUDIT_HMAC_KEY)) };
    let h = sha384::hmac(&key, &buf[..CHAIN_HASH_LEN + n]);
    let mut k = key;
    crate::security::zeroize::zeroize(&mut k);
    unsafe { CHAIN[slot] = h; }
}

/// Zero the chain table. Called from `audit::wipe_ring` so the
/// chain doesn't carry hashes pointing at the (now-zeroed)
/// entries. SAFETY: same single-writer assumption as the rest
/// of the chain module; never call from a concurrent record path.
pub fn reset_for_test() {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CHAIN);
        for i in 0..RING_CAP {
            (*ptr)[i] = [0u8; CHAIN_HASH_LEN];
        }
    }
}

/// Return the current chain head — the hash of the most recently
/// recorded entry. Operators should seal this externally on a
/// regular cadence (every N entries, every M seconds, etc.) so
/// tampering against the live ring becomes detectable.
pub fn chain_head() -> [u8; CHAIN_HASH_LEN] {
    let head = HEAD.load(Ordering::Relaxed);
    if head == 0 {
        return [0u8; CHAIN_HASH_LEN];
    }
    let slot = (head - 1) % RING_CAP;
    unsafe { CHAIN[slot] }
}

/// On-platform seal record. 32-byte chain head + the absolute
/// entry count at the moment of the seal. Serialized as a 40-byte
/// blob (8-byte big-endian count + 32-byte hash) into the
/// BatFS-backed "audit-chain.seal" file.
///
/// Verification: read the seal, walk the live ring from
/// `(seal.count - resident_count) .. seal.count`, recompute, and
/// assert the final hash == `seal.hash`. If the live ring is
/// shorter than expected, `truncation_at` reports how many
/// entries are missing.
pub struct ChainSeal {
    pub count: usize,
    pub hash:  [u8; CHAIN_HASH_LEN],
}

/// SP-C4.1: encoded seal length grew 40 -> 56 bytes (8B count + 48B
/// hash) when chain HMAC upgraded SHA-256 -> SHA-384.
pub const SEAL_ENCODED_LEN: usize = 8 + CHAIN_HASH_LEN;

impl ChainSeal {
    /// Encode as 8B big-endian count || 48B hash (56 bytes total).
    pub fn encode(&self) -> [u8; SEAL_ENCODED_LEN] {
        let mut out = [0u8; SEAL_ENCODED_LEN];
        out[..8].copy_from_slice(&(self.count as u64).to_be_bytes());
        out[8..SEAL_ENCODED_LEN].copy_from_slice(&self.hash);
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SEAL_ENCODED_LEN { return None; }
        let mut c = [0u8; 8];
        c.copy_from_slice(&bytes[..8]);
        let mut h = [0u8; CHAIN_HASH_LEN];
        h.copy_from_slice(&bytes[8..]);
        Some(ChainSeal { count: u64::from_be_bytes(c) as usize, hash: h })
    }
}

/// Verification result for a seal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealVerify {
    /// Seal matches the live ring's chain head at `seal.count`.
    Ok,
    /// Live ring has fewer entries than the seal recorded — the
    /// tail has been truncated by `missing` entries since the seal.
    Truncated { missing: usize },
    /// Hash mismatch even though counts match — somebody rewrote a
    /// past entry without updating CHAIN. `seal.count`-th entry's
    /// recomputed link doesn't match the seal.
    Mismatch,
    /// Seal's recorded count is BELOW any entry still resident — we
    /// can't verify it against the in-memory ring (the seal predates
    /// every entry the ring still holds).
    SealAboveRingTail,
    /// Seal's count is AHEAD of HEAD — either the seal is from a
    /// future run we never reached, or there's clock-skew between
    /// seal and ring.
    SealAheadOfHead,
}

/// Verify a seal against the live ring. Walks
/// `start .. seal.count`, recomputing each chain link, and
/// asserts the final hash equals `seal.hash`. `start` is the
/// oldest absolute index that's still resident, derived as
/// `head.saturating_sub(RING_CAP)`.
pub fn verify_seal(seal: &ChainSeal) -> SealVerify {
    let head = HEAD.load(Ordering::Relaxed);
    if seal.count > head {
        return SealVerify::SealAheadOfHead;
    }
    let ring_tail = head.saturating_sub(RING_CAP);
    if seal.count < ring_tail {
        return SealVerify::SealAboveRingTail;
    }
    if seal.count == 0 {
        // Genesis seal: hash should equal the all-zero chain
        // (no entries recorded yet).
        return if seal.hash == [0u8; CHAIN_HASH_LEN] { SealVerify::Ok }
               else { SealVerify::Mismatch };
    }
    // Recompute the chain from `ring_tail .. seal.count`. The
    // starting prev_hash is the stored chain at the slot just
    // before ring_tail (or zeros if ring_tail == 0).
    let mut prev = if ring_tail == 0 {
        [0u8; CHAIN_HASH_LEN]
    } else {
        let prev_slot = (ring_tail - 1) % RING_CAP;
        unsafe { CHAIN[prev_slot] }
    };
    for i in ring_tail..seal.count {
        let slot = i % RING_CAP;
        let entry = unsafe { &crate::security::audit::raw_ring()[slot] };
        let mut canon = [0u8; CHAIN_HASH_LEN + MSG_LEN];
        let n = canonical_bytes(entry, &mut canon);
        let mut buf = [0u8; CHAIN_HASH_LEN + CHAIN_HASH_LEN + MSG_LEN];
        buf[..CHAIN_HASH_LEN].copy_from_slice(&prev);
        buf[CHAIN_HASH_LEN..CHAIN_HASH_LEN + n].copy_from_slice(&canon[..n]);
        // SP-C4.1: HMAC-SHA-384 (matches append_chain).
        let key = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(AUDIT_HMAC_KEY)) };
        prev = sha384::hmac(&key, &buf[..CHAIN_HASH_LEN + n]);
        let mut k = key;
        crate::security::zeroize::zeroize(&mut k);
    }
    if prev == seal.hash {
        // Independent witness: if `head > seal.count`, the head we
        // just recomputed should land on the entry one before the
        // last new one. We don't fail on that; the seal is a
        // checkpoint, not the live tip.
        SealVerify::Ok
    } else if head > seal.count {
        // Tail moved forward; the recomputed prev is the hash at
        // index seal.count - 1, which IS what the seal claims.
        // If still doesn't match, somebody edited a past entry.
        SealVerify::Mismatch
    } else {
        // head == seal.count and the recomputed final hash
        // doesn't match.
        SealVerify::Mismatch
    }
}

/// Build a fresh seal capturing the current chain head + entry
/// count. Caller persists the bytes off-platform (BatFS today;
/// TPM / Apple SE in a future arc).
pub fn current_seal() -> ChainSeal {
    ChainSeal {
        count: HEAD.load(Ordering::Relaxed),
        hash:  chain_head(),
    }
}

/// Verification result for one entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    Ok,
    FirstMismatchAt(usize),
}

/// Walk the resident portion of the ring and recompute the chain
/// from the genesis. Returns `Ok` if every entry hashes to its
/// stored chain slot, otherwise `FirstMismatchAt(absolute_head_index)`
/// pointing to the first entry whose chain value didn't match.
///
/// Note: only the LIVE portion of the ring is verifiable in-place.
/// Earlier entries that rolled over are gone — the operator-side
/// seal is what extends auditability past one ring cycle.
pub fn verify_chain() -> VerifyOutcome {
    let head = HEAD.load(Ordering::Relaxed);
    if head == 0 {
        return VerifyOutcome::Ok;
    }
    let start = head.saturating_sub(RING_CAP);
    let mut prev_hash = if start == 0 {
        [0u8; CHAIN_HASH_LEN]
    } else {
        // We don't have the pre-eviction hash; assume the chain head
        // from the entry just before start is what the stored CHAIN
        // slot says it is. If an operator-side anchor exists, the
        // caller can verify continuity separately.
        let prev_slot = (start - 1) % RING_CAP;
        unsafe { CHAIN[prev_slot] }
    };

    for i in start..head {
        let slot = i % RING_CAP;
        let entry = unsafe { &crate::security::audit::raw_ring()[slot] };
        let mut canon = [0u8; CHAIN_HASH_LEN + MSG_LEN];
        let n = canonical_bytes(entry, &mut canon);
        let mut buf = [0u8; CHAIN_HASH_LEN + CHAIN_HASH_LEN + MSG_LEN];
        buf[..CHAIN_HASH_LEN].copy_from_slice(&prev_hash);
        buf[CHAIN_HASH_LEN..CHAIN_HASH_LEN + n].copy_from_slice(&canon[..n]);
        // SP-C4.1: HMAC-SHA-384 (matches append_chain).
        let key = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(AUDIT_HMAC_KEY)) };
        let expected = sha384::hmac(&key, &buf[..CHAIN_HASH_LEN + n]);
        let mut k = key;
        crate::security::zeroize::zeroize(&mut k);
        let stored = unsafe { CHAIN[slot] };
        if expected != stored {
            return VerifyOutcome::FirstMismatchAt(i);
        }
        prev_hash = expected;
    }
    VerifyOutcome::Ok
}
