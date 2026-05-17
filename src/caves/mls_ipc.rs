//! MLS-labeled inter-cave mailbox — gov-grade §3.2 (labeled IPC slice).
//!
//! Today's `ipc_session` and `sys_wg_ipc` are point-to-point channels
//! that don't carry sensitivity labels. This module adds a tiny
//! per-cave mailbox that stamps every message with the sender's
//! `cave::Sensitivity` at enqueue time and rejects the send if it
//! would violate Bell-LaPadula:
//!
//!   * `send(sender_id, receiver_id, body)` enforces the *-property
//!     (no write-down): `sender.level <= receiver.level`. Data may
//!     flow UP the lattice freely; flowing DOWN requires an
//!     explicit downgrade by the admin (a future arc).
//!   * `recv(receiver_id)` enforces the simple security property
//!     (no read-up): `receiver.level >= message.level`. Same level
//!     or below succeeds; above is denied.
//!
//! Storage: a fixed `MAX_BOXES`-slot ring per cave (admin caps via
//! `MAX_PER_CAVE`). Old messages get overwritten round-robin once
//! the box is full — same shape as the audit ring's overflow path.
//!
//! Crypto-binding (2026-05-13): every queued message is
//! ChaCha20-Poly1305 sealed with AAD = sender_id || sensitivity ||
//! integrity, keyed by a per-boot MAC key derived from BatFS's
//! master key. A memory-corrupting attacker who flips any of those
//! label bytes — even keeping the body the same — invalidates the
//! Poly1305 tag, so `recv` returns `MlsIpcError::AeadFail` instead
//! of delivering the body under a downgraded label. Same TOCTOU
//! property as the AEAD-bound BatFS file labels we already ship.
//!
//! Not yet:
//!   - No declassification path (deliberate downgrade by a trusted
//!     subject). The lattice rejects all write-down attempts today.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    ChaCha20Poly1305, Tag,
};

use crate::caves::cave::{self, Integrity, MlsOp, Sensitivity};
use crate::crypto::sha256;

/// Per-cave mailbox depth. 16 messages × N caves = bounded memory.
pub const MAX_PER_CAVE: usize = 16;
/// Payload cap per message. Sized so a full mailbox doesn't blow the
/// kernel BSS — 16 × 4 KiB max per cave, 64 KiB worst case.
pub const MAX_PAYLOAD: usize = 256;

#[derive(Clone, Copy)]
pub struct LabeledMsg {
    pub in_use: bool,
    pub sender_id: u16,
    pub sensitivity: u8,
    pub integrity:   u8,
    pub len: u16,
    /// Per-message 12-byte nonce. Monotonic from `NONCE_COUNTER`
    /// so reuse across messages can't happen.
    pub nonce: [u8; 12],
    /// 16-byte Poly1305 tag covering (AAD = sender_id || sens ||
    /// integ) and the ciphertext below. Verified at recv time;
    /// tamper on either AAD field OR the body invalidates this.
    pub tag: [u8; 16],
    /// Ciphertext (same length as the original plaintext). Stays
    /// encrypted while sitting in kernel RAM; `recv` decrypts in
    /// place into the caller's output buffer after the tag is
    /// verified.
    pub body: [u8; MAX_PAYLOAD],
}

impl LabeledMsg {
    pub const fn empty() -> Self {
        Self {
            in_use: false, sender_id: 0, sensitivity: 0, integrity: 0,
            len: 0, nonce: [0u8; 12], tag: [0u8; 16],
            body: [0u8; MAX_PAYLOAD],
        }
    }
    pub fn body_slice(&self) -> &[u8] {
        &self.body[..self.len as usize]
    }
}

/// Monotonic nonce counter for the mls_ipc AEAD. Same discipline as
/// `batfs::next_nonce` — increment on each send so no two messages
/// ever share a nonce.
static NONCE_COUNTER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(1);

fn next_nonce() -> [u8; 12] {
    let n = NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut out = [0u8; 12];
    out[..8].copy_from_slice(&n.to_le_bytes());
    out
}

/// Per-boot MAC key for mls_ipc. Derived from BatFS's master key
/// the first time `mls_ipc_key()` is called, then cached. Volatile
/// read of the master so the compiler can't dead-store-eliminate
/// it in a future refactor.
static mut MLS_IPC_KEY: [u8; 32] = [0u8; 32];
static MLS_IPC_KEY_READY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

fn mls_ipc_key() -> [u8; 32] {
    unsafe {
        if !MLS_IPC_KEY_READY.load(Ordering::Acquire) {
            let master = crate::fs::batfs::master_key();
            let derived = sha256::derive_key(&master, b"mls-ipc-aead-v1");
            MLS_IPC_KEY = derived;
            MLS_IPC_KEY_READY.store(true, Ordering::Release);
        }
        core::ptr::read_volatile(core::ptr::addr_of!(MLS_IPC_KEY))
    }
}

/// Canonical AAD layout: sender_id (BE u16) || sensitivity || integrity.
/// Any tamper on the on-disk fields makes the recomputed AAD
/// differ from what was sealed under, invalidating the Poly1305
/// tag at recv time.
fn aad_for(sender_id: u16, sensitivity: u8, integrity: u8) -> [u8; 4] {
    [
        (sender_id >> 8) as u8,
        (sender_id & 0xff) as u8,
        sensitivity,
        integrity,
    ]
}

/// One inbox per cave. `MAX_CAVES` is small (32) so we statically
/// allocate the full grid — 32 × 16 × ~260 B ≈ 128 KiB total.
const MAX_BOXES: usize = crate::caves::cave::MAX_CAVES;
static mut INBOX: [[LabeledMsg; MAX_PER_CAVE]; MAX_BOXES] =
    [[LabeledMsg::empty(); MAX_PER_CAVE]; MAX_BOXES];
static SEND_COUNT:   AtomicUsize = AtomicUsize::new(0);
static RECV_COUNT:   AtomicUsize = AtomicUsize::new(0);
static REJECT_WRITE_DOWN: AtomicUsize = AtomicUsize::new(0);
static REJECT_READ_UP:    AtomicUsize = AtomicUsize::new(0);
static REJECT_WRITE_UP:   AtomicUsize = AtomicUsize::new(0);
static REJECT_READ_DOWN:  AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq, Eq)]
pub enum MlsIpcError {
    /// Bell-LaPadula *-property violation (sender's sensitivity is
    /// strictly above the receiver's).
    WriteDown,
    /// Bell-LaPadula simple-security violation (the queued message
    /// is sensitivity-labeled above the receiver — covers runtime
    /// demotion between send and recv).
    ReadUp,
    /// Biba *-integrity violation (sender's integrity is strictly
    /// BELOW the receiver's — would taint a higher-integrity
    /// destination with low-integrity data).
    WriteUp,
    /// Biba simple-integrity violation (queued message is from a
    /// lower-integrity source than the receiver — covers runtime
    /// elevation between send and recv).
    ReadDown,
    /// Receiver mailbox is empty.
    Empty,
    /// Cave id out of range.
    BadId,
    /// Body too large.
    TooLong,
    /// AEAD verification failed — either the body or the
    /// (sender_id, sensitivity, integrity) AAD tuple was tampered
    /// with at rest. Returned by `recv` instead of delivering a
    /// possibly-downgraded message.
    AeadFail,
    /// SP-B1.6.2 (2026-05-16): gov-strict policy rejected the AEAD
    /// primitive used by MLS IPC (plain ChaCha20-Poly1305 — not on
    /// the CNSA 2.0 allowlist). Future SP-B1.6.3 may add an
    /// AES-256-GCM-SIV-backed MLS IPC variant; until then, MLS IPC
    /// is community-build-only.
    PolicyRejected,
}

/// Send `body` from `sender_id` to `receiver_id`. The message is
/// stamped with the SENDER's current `cave::sensitivity_of`. Returns
/// `Err(WriteDown)` if the policy rejects the flow before any state
/// is mutated.
pub fn send(sender_id: u16, receiver_id: u16, body: &[u8]) -> Result<usize, MlsIpcError> {
    let s_idx = sender_id as usize;
    let r_idx = receiver_id as usize;
    if s_idx >= MAX_BOXES || r_idx >= MAX_BOXES {
        return Err(MlsIpcError::BadId);
    }
    if body.len() > MAX_PAYLOAD {
        return Err(MlsIpcError::TooLong);
    }
    let s_sens  = cave::sensitivity_of(sender_id);
    let r_sens  = cave::sensitivity_of(receiver_id);
    if !cave::can_flow(s_sens, r_sens, MlsOp::Write) {
        REJECT_WRITE_DOWN.fetch_add(1, Ordering::Relaxed);
        return Err(MlsIpcError::WriteDown);
    }
    let s_integ = cave::integrity_of(sender_id);
    let r_integ = cave::integrity_of(receiver_id);
    if !cave::can_flow_integrity(s_integ, r_integ, MlsOp::Write) {
        REJECT_WRITE_UP.fetch_add(1, Ordering::Relaxed);
        return Err(MlsIpcError::WriteUp);
    }

    // SP-B1.6.2 (2026-05-16): policy gate. Under gov-strict, plain
    // ChaCha20-Poly1305 is rejected outside CNSA-grade context. MLS
    // IPC fails-closed under gov-strict until SP-B1.6.3 adds a
    // CNSA-eligible AES-256-GCM-SIV variant.
    if crate::crypto::policy::ensure_permitted(
        crate::crypto::policy::Algo::ChaCha20Poly1305,
    ).is_err() {
        return Err(MlsIpcError::PolicyRejected);
    }

    // Seal the body with ChaCha20-Poly1305: AAD = sender || sens ||
    // integ. Tamper on any AAD field invalidates the tag at recv.
    let nonce = next_nonce();
    let aad = aad_for(sender_id, s_sens as u8, s_integ as u8);
    let key = mls_ipc_key();
    let cipher = ChaCha20Poly1305::new(&key.into());
    let mut buf = [0u8; MAX_PAYLOAD];
    buf[..body.len()].copy_from_slice(body);
    let tag = match cipher.encrypt_in_place_detached(
        (&nonce).into(), &aad, &mut buf[..body.len()],
    ) {
        Ok(t) => t,
        Err(_) => return Err(MlsIpcError::AeadFail),
    };

    // Find a free slot in receiver's inbox; if all are in_use,
    // round-robin evict the oldest (slot 0) and shift up.
    unsafe {
        let inbox = &mut (*core::ptr::addr_of_mut!(INBOX))[r_idx];
        let slot = match inbox.iter().position(|m| !m.in_use) {
            Some(i) => i,
            None => {
                // Shift left, drop slot 0, write into slot MAX-1.
                for i in 1..MAX_PER_CAVE {
                    inbox[i - 1] = inbox[i];
                }
                MAX_PER_CAVE - 1
            }
        };
        let m = &mut inbox[slot];
        m.in_use = true;
        m.sender_id = sender_id;
        m.sensitivity = s_sens as u8;
        m.integrity   = s_integ as u8;
        m.len = body.len() as u16;
        m.nonce = nonce;
        m.tag.copy_from_slice(&tag);
        m.body[..body.len()].copy_from_slice(&buf[..body.len()]);
    }
    SEND_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(body.len())
}

/// Receive the oldest message in `receiver_id`'s inbox. Enforces
/// the simple security property — should normally always succeed
/// because `send` already rejected write-downs, but a runtime
/// elevation of the receiver between send and recv could leave a
/// stale message that's now above the receiver's level.
/// Belt-and-suspenders.
pub fn recv(receiver_id: u16, out: &mut [u8]) -> Result<(u16, u8, usize), MlsIpcError> {
    let r_idx = receiver_id as usize;
    if r_idx >= MAX_BOXES { return Err(MlsIpcError::BadId); }
    let r_sens  = cave::sensitivity_of(receiver_id);
    let r_integ = cave::integrity_of(receiver_id);

    unsafe {
        let inbox = &mut (*core::ptr::addr_of_mut!(INBOX))[r_idx];
        // Find the oldest in-use slot.
        let slot = match inbox.iter().position(|m| m.in_use) {
            Some(i) => i,
            None => return Err(MlsIpcError::Empty),
        };
        let m_sens  = Sensitivity::from_u8(inbox[slot].sensitivity);
        let m_integ = Integrity::from_u8(inbox[slot].integrity);
        if !cave::can_flow(r_sens, m_sens, MlsOp::Read) {
            REJECT_READ_UP.fetch_add(1, Ordering::Relaxed);
            return Err(MlsIpcError::ReadUp);
        }
        if !cave::can_flow_integrity(r_integ, m_integ, MlsOp::Read) {
            REJECT_READ_DOWN.fetch_add(1, Ordering::Relaxed);
            return Err(MlsIpcError::ReadDown);
        }
        let m = inbox[slot];
        // AEAD-verify the (sender, sens, integ) AAD against the
        // stored tag. Any byte-flip of those fields, or the body,
        // makes Poly1305 reject — we refuse delivery rather than
        // honour the tampered labels. Decrypts into a local
        // buffer first so a tampered ciphertext doesn't leak
        // half-plaintext to the caller.
        // SP-B1.6.2: mirror the send-side gate. Under gov-strict,
        // never decrypt with a rejected primitive.
        if crate::crypto::policy::ensure_permitted(
            crate::crypto::policy::Algo::ChaCha20Poly1305,
        ).is_err() {
            return Err(MlsIpcError::PolicyRejected);
        }
        let aad = aad_for(m.sender_id, m.sensitivity, m.integrity);
        let key = mls_ipc_key();
        let cipher = ChaCha20Poly1305::new(&key.into());
        let mut buf = [0u8; MAX_PAYLOAD];
        buf[..m.len as usize].copy_from_slice(&m.body[..m.len as usize]);
        let tag_obj: &Tag = (&m.tag).into();
        if cipher.decrypt_in_place_detached(
            (&m.nonce).into(), &aad, &mut buf[..m.len as usize], tag_obj,
        ).is_err() {
            // Don't drain the slot on AEAD fail — a follow-up
            // diagnostic call may want to inspect it. Tamper is
            // observable.
            return Err(MlsIpcError::AeadFail);
        }
        // Drain the slot after successful verification.
        inbox[slot] = LabeledMsg::empty();
        let copy = (m.len as usize).min(out.len());
        out[..copy].copy_from_slice(&buf[..copy]);
        RECV_COUNT.fetch_add(1, Ordering::Relaxed);
        Ok((m.sender_id, m.sensitivity, copy))
    }
}

/// `(sends, recvs, rej_write_down, rej_read_up, rej_write_up, rej_read_down)`
pub fn stats() -> (usize, usize, usize, usize, usize, usize) {
    (
        SEND_COUNT.load(Ordering::Relaxed),
        RECV_COUNT.load(Ordering::Relaxed),
        REJECT_WRITE_DOWN.load(Ordering::Relaxed),
        REJECT_READ_UP.load(Ordering::Relaxed),
        REJECT_WRITE_UP.load(Ordering::Relaxed),
        REJECT_READ_DOWN.load(Ordering::Relaxed),
    )
}

/// Test-only: flip one byte of the most-recent queued message's
/// sensitivity label. Used by `mls-ipc-binding-selftest` to prove
/// AEAD tag rejects label tampers. Returns true on a hit.
#[allow(dead_code)]
pub unsafe fn tamper_test_flip_sensitivity(receiver_id: u16, new_sens: u8) -> bool {
    let r_idx = receiver_id as usize;
    if r_idx >= MAX_BOXES { return false; }
    unsafe {
        let inbox = &mut (*core::ptr::addr_of_mut!(INBOX))[r_idx];
        for m in inbox.iter_mut() {
            if m.in_use {
                m.sensitivity = new_sens;
                return true;
            }
        }
    }
    false
}

/// Test-only: flip one byte of the queued message's body. Same
/// purpose as the sensitivity flipper.
#[allow(dead_code)]
pub unsafe fn tamper_test_flip_body(receiver_id: u16, byte_offset: usize) -> bool {
    let r_idx = receiver_id as usize;
    if r_idx >= MAX_BOXES { return false; }
    unsafe {
        let inbox = &mut (*core::ptr::addr_of_mut!(INBOX))[r_idx];
        for m in inbox.iter_mut() {
            if m.in_use && byte_offset < m.len as usize {
                m.body[byte_offset] ^= 0xFF;
                return true;
            }
        }
    }
    false
}

/// Drain a specific cave's mailbox — test cleanup hook.
pub fn drain(cave_id: u16) -> usize {
    let r_idx = cave_id as usize;
    if r_idx >= MAX_BOXES { return 0; }
    let mut n = 0;
    unsafe {
        let inbox = &mut (*core::ptr::addr_of_mut!(INBOX))[r_idx];
        for m in inbox.iter_mut() {
            if m.in_use { n += 1; }
            *m = LabeledMsg::empty();
        }
    }
    n
}
