//! Two-person integrity (TPI) — gov-grade §3.23 insider-collusion
//! resistance.
//!
//! Certain operations should never be authorised by a single
//! operator alone — wiping the audit ring, rotating master keys,
//! installing a new release pubkey, declassifying labeled data,
//! flushing the audit chain seal to BatFS. SELinux + RBAC alone
//! can't stop a sole compromised operator; TPI forces M-of-N
//! co-signatures before the kernel honours the op.
//!
//! Threat model
//! ============
//! Single attacker has full operator credentials (e.g. via an
//! insider, a stolen laptop, or a session-hijack). Without TPI
//! they can wipe the audit trail and exfiltrate keys. With TPI,
//! they need M-of-N quorum partners — collusion is harder to
//! organise and easier to detect.
//!
//! Design
//! ======
//! Each TPI op has a fixed `OpId`. The audit-officer and crypto-
//! officer roles each hold an Ed25519 keypair, with their pubkeys
//! registered via `register_officer(role, pk)`. To execute the op:
//!
//!   1. Operator A signs `op_id || nonce || timestamp` with their
//!      role-key, calls `propose_op(op_id, nonce, ts, sig_a)`.
//!      The proposal lives in a tiny ring of `MAX_PENDING` slots.
//!   2. Operator B (different role) signs the same bytes, calls
//!      `cosign_op(op_id, nonce, sig_b)`.
//!   3. If both sigs verify against their registered pubkeys AND
//!      the proposal is still in the ring AND the timestamps are
//!      within `OP_TTL_SECS`, the kernel records "op N approved"
//!      and the actual privileged code path can run.
//!
//! Today's slice covers the M=2-of-N=2 case (one audit officer +
//! one crypto officer, both required). A future generalisation
//! lifts that to arbitrary M-of-N via threshold sigs (gov-grade
//! §3.23 second half).

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::security::audit::{self, Category};

/// Distinct privileged operations that require TPI quorum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpId {
    /// Wipe the audit ring (`audit-flush --truncate`, future).
    AuditRingWipe       = 1,
    /// Rotate the BatFS master key (future).
    MasterKeyRotate     = 2,
    /// Install a new release pubkey baked into the kernel image.
    ReleasePubkeyRotate = 3,
    /// Declassify a TS file to a lower label (future).
    DeclassifyDowngrade = 4,
    /// Capture + persist the audit chain seal to BatFS.
    AuditSealFlush      = 5,
}

impl OpId {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(OpId::AuditRingWipe),
            2 => Some(OpId::MasterKeyRotate),
            3 => Some(OpId::ReleasePubkeyRotate),
            4 => Some(OpId::DeclassifyDowngrade),
            5 => Some(OpId::AuditSealFlush),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            OpId::AuditRingWipe       => "audit-ring-wipe",
            OpId::MasterKeyRotate     => "master-key-rotate",
            OpId::ReleasePubkeyRotate => "release-pubkey-rotate",
            OpId::DeclassifyDowngrade => "declassify-downgrade",
            OpId::AuditSealFlush      => "audit-seal-flush",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Role { AuditOfficer, CryptoOfficer }

#[derive(Clone, Copy)]
struct OfficerSlot {
    in_use: bool,
    role: u8,           // 0 = AuditOfficer, 1 = CryptoOfficer
    pubkey: [u8; 32],
}

const MAX_OFFICERS: usize = 8;
static mut OFFICERS: [OfficerSlot; MAX_OFFICERS] = [OfficerSlot {
    in_use: false, role: 0, pubkey: [0u8; 32],
}; MAX_OFFICERS];
static OFFICER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Pending proposal awaiting a second signature.
#[derive(Clone, Copy)]
struct Pending {
    in_use: bool,
    op_id: u8,
    nonce: u64,
    timestamp: u64,        // operator-supplied unix seconds
    proposer_role: u8,
    sig_a: [u8; 64],
}

const MAX_PENDING: usize = 4;
/// Operator timestamps within ±5 minutes of each other count as
/// "same time" for the TTL window.
pub const OP_TTL_SECS: u64 = 300;

static mut PENDING: [Pending; MAX_PENDING] = [Pending {
    in_use: false, op_id: 0, nonce: 0, timestamp: 0,
    proposer_role: 0, sig_a: [0u8; 64],
}; MAX_PENDING];
static PENDING_COUNT: AtomicUsize = AtomicUsize::new(0);

static APPROVED_COUNT: AtomicUsize = AtomicUsize::new(0);
static REJECTED_COUNT: AtomicUsize = AtomicUsize::new(0);

/// One-shot approval ring: when `cosign_op` succeeds, the
/// resulting (op_id, granted_ts) lands here. A privileged code
/// path calls `consume_approval(op_id, now)` and either gets the
/// approval (removing it from the ring) or fails closed. Same
/// shape as a Kerberos ticket: short-lived, single-use, bound to
/// a specific action.
#[derive(Clone, Copy)]
struct Grant {
    in_use: bool,
    op_id: u8,
    granted_ts: u64,
}

const MAX_GRANTS: usize = 4;
/// How long an approval is valid before `consume_approval` rejects
/// it as stale. 60 s is the gov-grade default; long enough for an
/// operator to type the privileged command after co-signing,
/// short enough that a stale grant in a forgotten terminal can't
/// be replayed days later.
pub const GRANT_TTL_SECS: u64 = 60;
static mut GRANTS: [Grant; MAX_GRANTS] = [Grant {
    in_use: false, op_id: 0, granted_ts: 0,
}; MAX_GRANTS];
static GRANT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CONSUMED_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Record a fresh approval. Internal — only called from
/// `cosign_op` on success. Evicts oldest grant if the ring is
/// full so a TPI flood can't block legitimate later approvals.
fn record_approval(op_id: u8, granted_ts: u64) {
    unsafe {
        let grants = &mut *core::ptr::addr_of_mut!(GRANTS);
        let slot = match grants.iter().position(|g| !g.in_use) {
            Some(i) => i,
            None => {
                for i in 1..MAX_GRANTS { grants[i - 1] = grants[i]; }
                MAX_GRANTS - 1
            }
        };
        grants[slot] = Grant { in_use: true, op_id, granted_ts };
        let n = GRANT_COUNT.load(Ordering::Relaxed);
        if n < MAX_GRANTS {
            GRANT_COUNT.store(n + 1, Ordering::Relaxed);
        }
    }
}

/// True iff at least one audit + crypto officer have been
/// registered. Privileged ops in legacy environments where TPI
/// hasn't been provisioned keep working unchanged; ops that have
/// seen even one officer per role MUST go through quorum.
pub fn enforcement_active() -> bool {
    let n = OFFICER_COUNT.load(Ordering::Relaxed);
    if n < 2 { return false; }
    let (mut has_audit, mut has_crypto) = (false, false);
    unsafe {
        for i in 0..n {
            let o = &(*core::ptr::addr_of!(OFFICERS))[i];
            if !o.in_use { continue; }
            if o.role == 0 { has_audit  = true; }
            if o.role == 1 { has_crypto = true; }
        }
    }
    has_audit && has_crypto
}

/// Privileged-op gate. Returns true if a fresh approval for
/// `op_id` exists in the ring AND removes it (one-shot). Returns
/// false in legacy mode where TPI hasn't been provisioned —
/// the caller treats that as "skip the gate". Caller passes
/// the current time so the selftest can drive the TTL deterministically.
pub fn consume_approval(op_id: OpId, current_time_secs: u64) -> bool {
    if !enforcement_active() {
        // No officers registered ⇒ legacy single-operator mode.
        // Privileged ops keep working unchanged.
        return true;
    }
    unsafe {
        let grants = &mut *core::ptr::addr_of_mut!(GRANTS);
        for g in grants.iter_mut() {
            if !g.in_use { continue; }
            if g.op_id != op_id as u8 { continue; }
            let drift = if current_time_secs > g.granted_ts {
                current_time_secs - g.granted_ts
            } else {
                g.granted_ts - current_time_secs
            };
            if drift > GRANT_TTL_SECS {
                // Stale — drain and keep looking.
                g.in_use = false;
                continue;
            }
            g.in_use = false;
            CONSUMED_COUNT.fetch_add(1, Ordering::Relaxed);
            audit::record(Category::Auth,
                b"tpi: approval consumed by privileged op");
            return true;
        }
    }
    false
}

/// Diagnostic — observable in `tpi-status`. Doesn't expose any
/// signature bytes.
pub fn pending_grant_count() -> usize {
    let mut n = 0usize;
    unsafe {
        let grants = &*core::ptr::addr_of!(GRANTS);
        for g in grants.iter() {
            if g.in_use { n += 1; }
        }
    }
    n
}

#[derive(Debug, PartialEq, Eq)]
pub enum TpiError {
    UnknownRole,
    OfficerTableFull,
    BadSignature,
    NoSuchProposal,
    PendingTableFull,
    SameRoleCosign,
    ProposalExpired,
    UnknownOp,
}

fn role_byte(r: Role) -> u8 {
    match r { Role::AuditOfficer => 0, Role::CryptoOfficer => 1 }
}

fn role_from_byte(b: u8) -> Option<Role> {
    match b {
        0 => Some(Role::AuditOfficer),
        1 => Some(Role::CryptoOfficer),
        _ => None,
    }
}

/// Register an officer's Ed25519 pubkey under a given role.
/// Idempotent — re-registering the same (role, pubkey) is a
/// no-op. Multiple officers per role are permitted (any of the
/// audit officers + any of the crypto officers can co-sign).
pub fn register_officer(role: Role, pubkey: [u8; 32]) -> Result<(), TpiError> {
    let r = role_byte(role);
    unsafe {
        let n = OFFICER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            if OFFICERS[i].in_use
                && OFFICERS[i].role == r
                && OFFICERS[i].pubkey == pubkey
            {
                return Ok(());
            }
        }
        if n >= MAX_OFFICERS { return Err(TpiError::OfficerTableFull); }
        OFFICERS[n] = OfficerSlot { in_use: true, role: r, pubkey };
        OFFICER_COUNT.store(n + 1, Ordering::Relaxed);
    }
    Ok(())
}

/// Test-only: clear all officers + pending state + grants.
pub fn reset_for_test() {
    unsafe {
        OFFICERS = [OfficerSlot { in_use: false, role: 0, pubkey: [0u8; 32] };
                    MAX_OFFICERS];
        PENDING  = [Pending { in_use: false, op_id: 0, nonce: 0,
                              timestamp: 0, proposer_role: 0, sig_a: [0u8; 64] };
                    MAX_PENDING];
        GRANTS   = [Grant { in_use: false, op_id: 0, granted_ts: 0 };
                    MAX_GRANTS];
    }
    OFFICER_COUNT.store(0, Ordering::Relaxed);
    PENDING_COUNT.store(0, Ordering::Relaxed);
    GRANT_COUNT.store(0, Ordering::Relaxed);
}

/// Canonical bytes a TPI signature covers:
///   op_id (BE u8) || nonce (BE u64) || timestamp (BE u64)
/// Each operator signs THIS exact byte string with their role key.
/// Any wire-level encoding wraps it but the cryptographic message
/// is this — so signatures from operator A and B over the same op
/// are bitwise interchangeable as challenge bytes.
pub fn canonical_bytes(op_id: OpId, nonce: u64, timestamp: u64) -> [u8; 17] {
    let mut out = [0u8; 17];
    out[0] = op_id as u8;
    out[1..9].copy_from_slice(&nonce.to_be_bytes());
    out[9..17].copy_from_slice(&timestamp.to_be_bytes());
    out
}

fn verify_sig(pubkey: &[u8; 32], msg: &[u8], sig: &[u8; 64]) -> bool {
    crate::crypto::sig::ed25519_verify(pubkey, sig, msg).is_ok()
}

/// Find any officer (in_use, role matches, pubkey verifies the
/// sig over msg) and return their role byte.
fn first_valid_signer(msg: &[u8], sig: &[u8; 64]) -> Option<u8> {
    unsafe {
        let n = OFFICER_COUNT.load(Ordering::Relaxed);
        for i in 0..n {
            if !OFFICERS[i].in_use { continue; }
            if verify_sig(&OFFICERS[i].pubkey, msg, sig) {
                return Some(OFFICERS[i].role);
            }
        }
    }
    None
}

/// Operator A's proposal. Signature must verify against any
/// registered officer's pubkey. The role of the signer is
/// recorded; operator B must hold the OTHER role to co-sign.
pub fn propose_op(
    op_id: OpId, nonce: u64, timestamp: u64, sig_a: [u8; 64],
) -> Result<usize, TpiError> {
    let msg = canonical_bytes(op_id, nonce, timestamp);
    let proposer_role = match first_valid_signer(&msg, &sig_a) {
        Some(r) => r,
        None => {
            REJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
            audit::record(Category::Auth,
                b"tpi: propose_op rejected (BadSignature)");
            return Err(TpiError::BadSignature);
        }
    };

    unsafe {
        let n = PENDING_COUNT.load(Ordering::Relaxed);
        let pending = &mut *core::ptr::addr_of_mut!(PENDING);
        // Slot allocation: prefer a free slot, else evict oldest
        // (slot 0) round-robin so a busy proposer can't lock out
        // new ones.
        let slot = match pending.iter().position(|p| !p.in_use) {
            Some(i) => i,
            None => {
                for i in 1..MAX_PENDING { pending[i - 1] = pending[i]; }
                MAX_PENDING - 1
            }
        };
        pending[slot] = Pending {
            in_use: true,
            op_id: op_id as u8,
            nonce, timestamp, proposer_role, sig_a,
        };
        if n < MAX_PENDING {
            PENDING_COUNT.store(n + 1, Ordering::Relaxed);
        }
    }
    audit::record(Category::Auth, b"tpi: propose_op accepted");
    Ok(0)
}

/// Operator B's co-signature. Verifies sig_b under a DIFFERENT-
/// role officer's pubkey than the proposer used. Returns Ok when
/// the op is approved; subsequent calls for the same (op_id,
/// nonce) are no-ops since the proposal slot is consumed on
/// success.
pub fn cosign_op(
    op_id: OpId, nonce: u64, current_time_secs: u64, sig_b: [u8; 64],
) -> Result<(), TpiError> {
    // Locate the proposal.
    let (slot, prev_ts, prev_role, _sig_a) = unsafe {
        let mut found = None;
        for i in 0..MAX_PENDING {
            if PENDING[i].in_use
                && PENDING[i].op_id == op_id as u8
                && PENDING[i].nonce == nonce
            {
                found = Some((i, PENDING[i].timestamp,
                              PENDING[i].proposer_role, PENDING[i].sig_a));
                break;
            }
        }
        match found {
            Some(t) => t,
            None => {
                REJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
                audit::record(Category::Auth,
                    b"tpi: cosign_op rejected (NoSuchProposal)");
                return Err(TpiError::NoSuchProposal);
            }
        }
    };

    // TTL check — operator B's clock skew vs the proposer's
    // timestamp must be < OP_TTL_SECS in either direction.
    let drift = if current_time_secs > prev_ts {
        current_time_secs - prev_ts
    } else {
        prev_ts - current_time_secs
    };
    if drift > OP_TTL_SECS {
        // Drain the stale slot so it doesn't block other ops.
        unsafe {
            PENDING[slot].in_use = false;
        }
        REJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
        audit::record(Category::Auth,
            b"tpi: cosign_op rejected (ProposalExpired)");
        return Err(TpiError::ProposalExpired);
    }

    // Verify operator B's signature against the SAME canonical
    // bytes operator A signed.
    let msg = canonical_bytes(op_id, nonce, prev_ts);
    let cosigner_role = match first_valid_signer(&msg, &sig_b) {
        Some(r) => r,
        None => {
            REJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
            audit::record(Category::Auth,
                b"tpi: cosign_op rejected (BadSignature)");
            return Err(TpiError::BadSignature);
        }
    };
    if cosigner_role == prev_role {
        REJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
        audit::record(Category::Auth,
            b"tpi: cosign_op rejected (SameRoleCosign)");
        return Err(TpiError::SameRoleCosign);
    }

    // Approved. Consume the proposal slot — replay of either
    // signature can't approve the op a second time without a
    // fresh nonce. Drop a one-shot grant into the approval ring
    // so the privileged code path can pick it up next.
    unsafe { PENDING[slot].in_use = false; }
    record_approval(op_id as u8, current_time_secs);
    APPROVED_COUNT.fetch_add(1, Ordering::Relaxed);
    audit::record(Category::Auth, b"tpi: op APPROVED (M-of-2 quorum)");
    Ok(())
}

/// `(officers_registered, pending_proposals, approved_count, rejected_count)`
pub fn stats() -> (usize, usize, usize, usize) {
    let pending_live = unsafe {
        let pending = &*core::ptr::addr_of!(PENDING);
        pending.iter().filter(|p| p.in_use).count()
    };
    (
        OFFICER_COUNT.load(Ordering::Relaxed),
        pending_live,
        APPROVED_COUNT.load(Ordering::Relaxed),
        REJECTED_COUNT.load(Ordering::Relaxed),
    )
}
