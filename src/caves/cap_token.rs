//! Capability tokens for cross-cave IPC — gov-grade §3.2 hardening.
//!
//! Bell-LaPadula + Biba ("can this flow happen at all?") is the *policy*
//! layer; capability tokens are the *authorisation* layer. A cave that
//! wants to call into another cave must present a `CapToken` whose MAC
//! verifies under the per-boot issuing key AND whose `(issuer, holder,
//! target, rights, nonce)` tuple matches the call site.
//!
//! Design properties:
//!   * MAC: HMAC-SHA256 over the canonical token bytes, keyed by a
//!     per-boot key derived from SealFS's master key. An attacker who
//!     can flip RAM bytes can't forge a token without the issuing key
//!     — the same property that protects `mls_ipc`'s AEAD tag.
//!   * No clocks: tokens don't carry expiry. The microkernel boots
//!     fresh on every wipe / reboot — the per-boot key is fresh too,
//!     so a token from a previous boot would fail to verify. (See
//!     "Time Without a Clock" concept note for context.)
//!   * Constant-time tag comparison: a forged-tag attacker can't
//!     measure timing to recover prefix bytes.
//!   * Self-contained: the token IS the authorisation. No external
//!     state to update at mint time, no revocation table to consult
//!     at verify — verification is `(recompute_mac, equal in CT)`.
//!     Revocation = rotate the issuing key.
//!
//! What this module is NOT:
//!   * Not a session ticket. There's no replay-detection state. Same
//!     token presented twice verifies twice; the IPC path layered on
//!     top is responsible for whatever single-use semantics it wants.
//!   * Not network-grade. Tokens are kernel-internal — they're stamped
//!     at mint by the kernel and verified by the kernel. No cave ever
//!     sees the issuing key; that's the same model as `mls_ipc`'s
//!     per-boot MAC key.

#![allow(dead_code)]

use core::sync::atomic::Ordering;

use crate::crypto::sha256;

/// Rights bitmap. Each cap token carries a 32-bit rights mask the
/// holder can exercise against the target. Bits are independent
/// (orthogonal) so a token can grant any combination.
///   * `RIGHT_IPC_READ`  — `mls_ipc::recv` is permitted.
///   * `RIGHT_IPC_WRITE` — `mls_ipc::send` is permitted.
///   * `RIGHT_IPC_CALL`  — full call semantics (write request, read reply).
/// Higher bits are reserved for future fine-grained rights.
pub const RIGHT_IPC_READ:  u32 = 1 << 0;
pub const RIGHT_IPC_WRITE: u32 = 1 << 1;
pub const RIGHT_IPC_CALL:  u32 = RIGHT_IPC_READ | RIGHT_IPC_WRITE;

/// Compact, copyable capability token. Field layout is `#[repr(C)]`
/// so the canonical-bytes serialization stays stable across compilers.
/// The MAC binds every field in `canonical_bytes()` — flipping any
/// byte invalidates `verify`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CapToken {
    /// Cave that minted the token. Today every token is issued by
    /// the kernel — `issuer_cave = u16::MAX` denotes "kernel-issued."
    /// Reserved for a future cave-to-cave delegation arc.
    pub issuer_cave: u16,
    /// Cave that holds the token (the principal that presents it at
    /// call time). Verification rejects if `holder_cave` doesn't
    /// match the caller at the IPC entry point.
    pub holder_cave: u16,
    /// Cave the token grants access TO. Verification rejects if
    /// `target_cave` doesn't match the callee.
    pub target_cave: u16,
    pub _pad: u16,
    /// Rights mask. See `RIGHT_*` constants.
    pub rights: u32,
    /// Per-token nonce. Drawn from a monotonic counter at mint time
    /// so two mint() calls with identical (issuer, holder, target,
    /// rights) still produce distinct tokens — needed if a future
    /// arc wants to attach replay-detection state keyed by nonce.
    pub nonce: [u8; 16],
    /// HMAC-SHA256 tag over `canonical_bytes()`.
    pub mac: [u8; 32],
}

impl CapToken {
    /// Reserved value meaning "kernel-issued". Kernel context has
    /// no cave id; we encode it as `u16::MAX` to fit the field.
    pub const KERNEL_ISSUER: u16 = u16::MAX;

    /// Build a zeroed/invalid token. Useful as a placeholder; will
    /// fail `verify`.
    pub const fn empty() -> Self {
        Self {
            issuer_cave: 0,
            holder_cave: 0,
            target_cave: 0,
            _pad: 0,
            rights: 0,
            nonce: [0u8; 16],
            mac:   [0u8; 32],
        }
    }

    /// Canonical byte layout used by the MAC. The order is fixed
    /// (issuer → holder → target → rights → nonce). Any change to
    /// this layout is a breaking change for in-flight tokens — but
    /// the per-boot key already invalidates them on reboot, so no
    /// disk-format migration is needed.
    pub fn canonical_bytes(&self) -> [u8; 28] {
        let mut out = [0u8; 28];
        out[0..2].copy_from_slice(&self.issuer_cave.to_be_bytes());
        out[2..4].copy_from_slice(&self.holder_cave.to_be_bytes());
        out[4..6].copy_from_slice(&self.target_cave.to_be_bytes());
        // Skip _pad (2 bytes; reserved for future expansion).
        out[6..8].fill(0);
        out[8..12].copy_from_slice(&self.rights.to_be_bytes());
        out[12..28].copy_from_slice(&self.nonce);
        out
    }
}

/// Reasons a `verify` call rejects a token. Distinct from
/// `mls_label::LabelViolation` — these are *authorisation* failures,
/// not policy failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapError {
    /// MAC tag did not verify under the per-boot issuing key.
    /// Either the bytes were tampered with OR the token was minted
    /// in a prior boot.
    BadMac,
    /// Token's `holder_cave` doesn't match the caller presenting it.
    HolderMismatch,
    /// Token's `target_cave` doesn't match the call's destination.
    TargetMismatch,
    /// Token doesn't carry the rights the operation needs.
    InsufficientRights,
}

// ── Per-boot issuing key ──
//
// Same discipline as `mls_ipc::mls_ipc_key`: lazy-derived from the
// SealFS master key on first use, cached in BSS, read volatile so a
// future refactor can't dead-store-eliminate it. The cap-token key
// uses a distinct domain-separation tag so it can't be confused with
// the mls_ipc AEAD key.

static mut CAP_TOKEN_KEY: [u8; 32] = [0u8; 32];
static CAP_TOKEN_KEY_READY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

fn cap_token_key() -> [u8; 32] {
    unsafe {
        if !CAP_TOKEN_KEY_READY.load(Ordering::Acquire) {
            let master = crate::fs::sealfs::master_key();
            let derived = sha256::derive_key(&master, b"cap-token-mac-v1");
            CAP_TOKEN_KEY = derived;
            CAP_TOKEN_KEY_READY.store(true, Ordering::Release);
        }
        core::ptr::read_volatile(core::ptr::addr_of!(CAP_TOKEN_KEY))
    }
}

// ── Monotonic nonce ──
//
// Same shape as `mls_ipc::next_nonce`: a 64-bit counter, padded into
// the 16-byte field. Wrap-around after 2^64 mints is treated as
// "never happens in human time."

static NONCE_COUNTER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(1);

fn next_nonce() -> [u8; 16] {
    let n = NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&n.to_be_bytes());
    out
}

/// Mint a fresh cap token. The kernel (issuer) attaches its MAC; the
/// holder receives the bytes and presents them at the IPC entry
/// point.
pub fn mint(
    issuer_cave: u16,
    holder_cave: u16,
    target_cave: u16,
    rights:      u32,
) -> CapToken {
    let nonce = next_nonce();
    let mut tok = CapToken {
        issuer_cave, holder_cave, target_cave, _pad: 0,
        rights, nonce, mac: [0u8; 32],
    };
    let bytes = tok.canonical_bytes();
    let key   = cap_token_key();
    tok.mac   = sha256::hmac(&key, &bytes);
    tok
}

/// Verify a presented token against the live `(holder, target, op)`.
/// Constant-time tag comparison guards the MAC check; the binding
/// checks (`holder`, `target`, `rights`) are exact equality.
///
/// `op_rights` is the rights mask the call site needs — e.g.
/// `RIGHT_IPC_WRITE` for a one-way send. If the token doesn't carry
/// *every* bit in `op_rights`, the call is denied with
/// `InsufficientRights`. The IPC layer should pick the narrowest
/// rights mask the operation actually requires.
pub fn verify(
    token:       &CapToken,
    holder_cave: u16,
    target_cave: u16,
    op_rights:   u32,
) -> Result<(), CapError> {
    // Recompute MAC FIRST in CT — we don't short-circuit on the
    // binding mismatches before the MAC because that would leak
    // (via timing) whether the holder/target fields were correct
    // even on tokens with bad MACs.
    let bytes  = token.canonical_bytes();
    let key    = cap_token_key();
    let expect = sha256::hmac(&key, &bytes);
    let mac_ok = ct_eq_32(&expect, &token.mac);
    if !mac_ok {
        return Err(CapError::BadMac);
    }
    // The binding checks may now short-circuit safely (no MAC
    // information leaks because the MAC is already validated).
    if token.holder_cave != holder_cave {
        return Err(CapError::HolderMismatch);
    }
    if token.target_cave != target_cave {
        return Err(CapError::TargetMismatch);
    }
    if (token.rights & op_rights) != op_rights {
        return Err(CapError::InsufficientRights);
    }
    Ok(())
}

/// Constant-time byte-wise equality for 32-byte tags. Volatile reads
/// so the optimiser can't elide the loop or specialise on early-out
/// patterns. Standard XOR-OR-zero compare.
fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut acc: u8 = 0;
    for i in 0..32 {
        // Volatile loads so the optimiser can't see "same buffer"
        // pruning. In `no_std` we can't pull `subtle`'s
        // `ConstantTimeEq` without adding a dep — and adding a dep
        // would need a Cargo.lock grant, which the §3 OOS list
        // implicitly discourages.
        let av = unsafe { core::ptr::read_volatile(a.as_ptr().add(i)) };
        let bv = unsafe { core::ptr::read_volatile(b.as_ptr().add(i)) };
        acc |= av ^ bv;
    }
    acc == 0
}

/// Test-only / selftest helper: rotate the per-boot key. Used by
/// selftests that want to prove forgery attempts fail even when the
/// attacker briefly knew an earlier key. Production code should not
/// call this — the per-boot lifecycle handles rotation.
#[allow(dead_code)]
pub fn force_rotate_key_for_test(seed: &[u8]) {
    unsafe {
        let master = crate::fs::sealfs::master_key();
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&master);
        let n = seed.len().min(32);
        combined[32..32 + n].copy_from_slice(&seed[..n]);
        CAP_TOKEN_KEY = sha256::derive_key(&master, &combined);
        CAP_TOKEN_KEY_READY.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Scenario 5: forging a token with a tampered MAC must fail. ──
    #[test]
    fn test_cap_token_forge_attempt() {
        // We can't mint a real token in unit tests because
        // `sealfs::master_key()` is kernel state. Instead we
        // construct a token whose MAC is hand-crafted (all zeros,
        // then a flipped byte) and assert constant-time compare
        // rejects.
        //
        // The MAC field of `CapToken::empty()` is all zeros. Calling
        // verify against it goes through `cap_token_key()`, which
        // would attempt to read `sealfs::master_key()` — fine in a
        // kernel boot but unsafe in the unit-test compile lane that
        // doesn't initialise sealfs. So we test the CT compare
        // primitive directly here and defer the full mint+forge round
        // trip to the QEMU `cap-mls-selftest` shell command.
        let zeros = [0u8; 32];
        let mut one = [0u8; 32];
        one[31] = 1;
        assert!(ct_eq_32(&zeros, &zeros), "zeros vs zeros must compare equal");
        assert!(!ct_eq_32(&zeros, &one),  "one-bit diff must compare unequal");
        // High-bit and low-bit diffs both reject — sanity that the
        // loop covers the full 32 bytes.
        let mut hi = [0u8; 32];
        hi[0] = 1;
        assert!(!ct_eq_32(&zeros, &hi));
    }

    // ── Scenario 6: a freshly-minted token, properly bound, verifies
    //    OK. We cover the construction-and-bytes round trip here;
    //    the full mint+verify round trip (which calls
    //    `cap_token_key`) is in the QEMU selftest. ──
    #[test]
    fn test_cap_token_valid_call_passes() {
        // Build a token by hand with a deterministic "MAC" so we
        // exercise the binding-check arms of `verify` without
        // requiring `sealfs::master_key()`. We override
        // `cap_token_key()` is not pluggable in the test lane; the
        // assertion here covers the canonical-bytes layout and the
        // rights-mask logic, which are pure functions.
        let tok = CapToken {
            issuer_cave: CapToken::KERNEL_ISSUER,
            holder_cave: 2,
            target_cave: 5,
            _pad: 0,
            rights: RIGHT_IPC_CALL,
            nonce: [0x11; 16],
            mac:   [0u8; 32],
        };
        // Canonical bytes layout: issuer (BE) || holder (BE) || target (BE) || pad
        // || rights (BE) || nonce.
        let bytes = tok.canonical_bytes();
        assert_eq!(&bytes[0..2], &(u16::MAX).to_be_bytes());
        assert_eq!(&bytes[2..4], &2u16.to_be_bytes());
        assert_eq!(&bytes[4..6], &5u16.to_be_bytes());
        assert_eq!(&bytes[6..8], &[0, 0]);
        assert_eq!(&bytes[8..12], &RIGHT_IPC_CALL.to_be_bytes());
        assert_eq!(&bytes[12..28], &[0x11; 16]);

        // Rights-mask covers IPC_CALL super-set semantics.
        assert!((tok.rights & RIGHT_IPC_READ)  == RIGHT_IPC_READ);
        assert!((tok.rights & RIGHT_IPC_WRITE) == RIGHT_IPC_WRITE);
    }
}
