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
//! Not yet:
//!   - No cryptographic binding between message and label. A future
//!     pass keys the AEAD nonce on (sender, receiver, label) so a
//!     race attacker can't smuggle one message's body under another's
//!     label.
//!   - No declassification path (deliberate downgrade by a trusted
//!     subject). The lattice rejects all write-down attempts today.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::batcave::cave::{self, Integrity, MlsOp, Sensitivity};

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
    pub body: [u8; MAX_PAYLOAD],
}

impl LabeledMsg {
    pub const fn empty() -> Self {
        Self {
            in_use: false, sender_id: 0, sensitivity: 0, integrity: 0,
            len: 0, body: [0u8; MAX_PAYLOAD],
        }
    }
    pub fn body_slice(&self) -> &[u8] {
        &self.body[..self.len as usize]
    }
}

/// One inbox per cave. `MAX_CAVES` is small (32) so we statically
/// allocate the full grid — 32 × 16 × ~260 B ≈ 128 KiB total.
const MAX_BOXES: usize = crate::batcave::cave::MAX_CAVES;
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
        m.body[..body.len()].copy_from_slice(body);
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
        // Drain the slot.
        inbox[slot] = LabeledMsg::empty();
        let copy = (m.len as usize).min(out.len());
        out[..copy].copy_from_slice(&m.body[..copy]);
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
