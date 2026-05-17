//! Sphragis — LMS (Leighton-Micali Signatures) per RFC 8554 / NIST SP 800-208.
//!
//! Stateful hash-based signature scheme. CNSA 2.0 mandates LMS (or
//! XMSS) for software/firmware signing where post-quantum-secure
//! signatures with quantum-conservative assumptions are required.
//! Unlike ML-DSA-87 (FIPS 204, lattice-based), LMS relies only on
//! the security of the underlying hash function — a property NSA
//! prefers for the highest-confidence software-signing roots.
//!
//! ## Statefulness — the foot-gun
//!
//! LMS is **stateful**: every signature consumes one of the tree's
//! one-time-signature (OTS) leaves. If the same OTS leaf is ever
//! used twice (e.g., a backup of the signing key was restored after
//! a sign-then-crash), the security of the entire scheme collapses
//! for THAT one-time key. Operators MUST treat the signing-key state
//! as a single-writer, fsync-or-die piece of mutable state.
//!
//! The wrapper exposes a callback the caller MUST use to persist the
//! new state bytes after every sign. The callback returns `Result<(),
//! ()>` and is expected to either commit the new state or fail-stop
//! the sign operation.
//!
//! For Sphragis the planned operator-grade usage is:
//!   * Sphragis kernel image is LMS-signed at release time on a
//!     dedicated signing host. The signing key never touches a
//!     production server.
//!   * Loadable modules + update packages are LMS-signed by the
//!     same host.
//!   * The bootloader (m1n1 on M4; future GRUB/shim on x86_64)
//!     verifies the LMS signature before jumping into Rust. See
//!     REQ-BLD-008 + SP-B4 for the boot-chain integration plan.
//!
//! ## Parameters
//!
//! Default: SHA-256/256 hash, LMS tree height 5, Winternitz w=1.
//! That gives 2^5 = 32 signatures per key — fine for KAT; real
//! release keys would use H10 (1024 sigs) or H15 (32768 sigs).
//! NIST SP 800-208 §4 enumerates the approved parameter sets.
//!
//! Backed by Fraunhofer-AISEC's `hbs-lms` crate (Apache-2.0), which
//! is binary-compatible with Cisco's reference `hash-sigs` impl
//! (the de facto interoperability target). No-std capable;
//! default-features off.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use hbs_lms::{
    HssParameter, LmotsAlgorithm, LmsAlgorithm, Seed,
    Sha256_256,
    keygen, sign, verify,
};

use crate::crypto::rng;

/// Hasher used for all LMS operations on Sphragis. SHA-256 with
/// 256-bit output — NIST SP 800-208 §4.2 LMS_SHA256_M32 parameter.
pub type Hasher = Sha256_256;

/// Default LMS parameter set: H5 / W1. Produces 32 signatures per
/// key. Fine for KAT; real release keys use H10 or H15. Override
/// via `keygen_with_params` if you need a different tree height.
pub fn default_params() -> HssParameter<Hasher> {
    HssParameter::new(LmotsAlgorithm::LmotsW1, LmsAlgorithm::LmsH5)
}

/// Generate a fresh LMS keypair using entropy from the kernel RNG.
/// Returns `(signing_key_bytes, verifying_key_bytes)` — both
/// serialized for the caller to persist. Caller MUST keep the
/// signing-key bytes single-writer + fsync-or-die: every successful
/// sign mutates them (advances the OTS-leaf counter).
///
/// `aux_data` size: H5 tree is small; 4 KiB is more than enough.
/// Larger trees (H15) require ~MB of aux. The aux is an optional
/// precomputed-tree-node cache that speeds up signing; if not
/// preserved across reboots, signing still works (just slower for
/// the next sign after restart).
pub fn keygen_default() -> Result<(Vec<u8>, Vec<u8>), &'static str> {
    let params = [default_params()];

    let mut seed_bytes = [0u8; 32];
    rng::fill_bytes(&mut seed_bytes);
    let seed: Seed<Hasher> = Seed::from(seed_bytes);

    let mut aux_data: Vec<u8> = vec![0u8; 4096];
    let aux_slice: &mut &mut [u8] = &mut &mut aux_data[..];

    let (sk, vk) = keygen::<Hasher>(&params, &seed, Some(aux_slice))
        .map_err(|_| "lms: keygen failed")?;
    Ok((sk.as_slice().to_vec(), vk.as_slice().to_vec()))
}

/// Sign `msg` with `signing_key_bytes`, returning `(new_signing_key_bytes,
/// signature_bytes)`. Caller MUST persist `new_signing_key_bytes` BEFORE
/// using `signature_bytes` externally — otherwise a crash after sign
/// but before persist could allow OTS-leaf reuse on retry.
pub fn sign_default(signing_key_bytes: &[u8], msg: &[u8])
    -> Result<(Vec<u8>, Vec<u8>), &'static str>
{
    // The hbs-lms API takes an FnMut callback that is invoked with
    // the updated signing-key bytes. We capture into a Vec to return
    // alongside the signature.
    let mut new_sk: Vec<u8> = Vec::new();
    let mut update_fn = |bytes: &[u8]| -> Result<(), ()> {
        new_sk.clear();
        new_sk.extend_from_slice(bytes);
        Ok(())
    };

    let sig = sign::<Hasher>(msg, signing_key_bytes, &mut update_fn, None)
        .map_err(|_| "lms: sign failed")?;
    let sig_bytes = sig.as_ref().to_vec();
    if new_sk.is_empty() {
        return Err("lms: sign succeeded but state-update callback was not invoked");
    }
    Ok((new_sk, sig_bytes))
}

/// Verify `signature_bytes` over `msg` under `verifying_key_bytes`.
/// Stateless — the verifier never needs the signing state.
pub fn verify_default(verifying_key_bytes: &[u8], msg: &[u8], signature_bytes: &[u8])
    -> Result<(), &'static str>
{
    verify::<Hasher>(msg, signature_bytes, verifying_key_bytes)
        .map_err(|_| "lms: verify failed")
}

// ── Boot-time KAT (SP-B1.5 / SP-B1.7 partial; REQ-CRY-003) ───────

/// Round-trip + tamper-detect KAT. Generate a fresh key, sign a
/// fixed test vector, verify, then bit-flip the message and assert
/// verification fails. Called from `crypto::run_self_tests()` at
/// boot; failure halts boot per the audit-CRYPTO-F7 fail-closed
/// pattern.
///
/// Note: each KAT run consumes ONE OTS leaf from the generated
/// H5 tree (which has 32). Since the keypair is fresh per call and
/// thrown away after, this is fine — no OTS-leaf-reuse concern.
pub fn kat() -> Result<(), &'static str> {
    let (sk, vk) = keygen_default()?;
    let msg = b"sphragis LMS KAT vector 2026-05-16";
    let (new_sk, sig) = sign_default(&sk, msg)?;

    // Sanity: signing-key state must have advanced (OTS counter
    // bumped). Length-equal-but-content-different is the expected
    // invariant; we just check at least one byte changed.
    if new_sk.len() != sk.len() {
        return Err("KAT-FAIL: LMS signing-key length changed across sign");
    }
    let mut any_diff = false;
    for i in 0..sk.len() { if new_sk[i] != sk[i] { any_diff = true; break; } }
    if !any_diff {
        return Err("KAT-FAIL: LMS signing-key state did not advance after sign");
    }

    // Positive verify.
    verify_default(&vk, msg, &sig)?;

    // Tamper-detect: flip a message byte → verify MUST fail.
    let mut bad_msg = msg.to_vec();
    bad_msg[0] ^= 0x01;
    if verify_default(&vk, &bad_msg, &sig).is_ok() {
        return Err("KAT-FAIL: LMS accepted modified message");
    }

    // Tamper-detect: flip a signature byte → verify MUST fail.
    let mut bad_sig = sig.clone();
    if bad_sig.len() > 8 { bad_sig[8] ^= 0x01; }
    if verify_default(&vk, msg, &bad_sig).is_ok() {
        return Err("KAT-FAIL: LMS accepted modified signature");
    }

    Ok(())
}
