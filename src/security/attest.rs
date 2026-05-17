//! Sphragis Attestation Kernel Primitive (SP-C1.1).
//!
//! Differentiator #3: every cave is an attestable identity. The kernel
//! mediates the attestation surface — caves cannot forge their own
//! identity claims, and external verifiers can prove what code and
//! configuration is running before extending trust.
//!
//! ## What this module provides today (SP-C1.1)
//!
//! - `Claims` — caller-supplied claim set, opaque bytes per RATS
//!   conventions (CBOR or JWT encoding is the caller's choice; quote
//!   signs the bytes verbatim).
//! - `Quote` — the produced attestation envelope: kernel measurement,
//!   cave identity, claims, nonce, signature.
//! - `KernelMeasurement` — 48-byte SHA-384 hash of the loaded kernel
//!   image (text + rodata). Placeholder today; SP-C1.2 wires actual
//!   measurement at boot.
//! - `CaveIdentity` — name + per-cave attestation public key + the
//!   cave's code/config measurement. SP-C1.3 wires the kernel to
//!   maintain a registry; today there's a stub registry behind
//!   `set_local_cave_identity` for testing.
//! - `quote(nonce, claims) -> Quote` — produces a signed quote.
//!   Today the signature uses an in-memory ML-DSA-87 key generated
//!   at first use. SP-C1.4 (M4) / SP-C1.5 (Caliptra) replace this
//!   with a hardware-rooted key.
//!
//! ## What's NOT here yet
//!
//! - Kernel measurement at boot (`SP-C1.2`). Today a fixed
//!   placeholder is used; real measurement requires linker-script
//!   symbols `__kernel_text_start` / `__kernel_text_end` and a
//!   hash-at-boot pass.
//! - Hardware-rooted attestation key (SP-C1.4 SEP, SP-C1.5 Caliptra,
//!   SP-C1.6 HSM-backed CA). Today: in-memory ML-DSA-87 keypair
//!   generated at first use; quotes are verifiable but the root of
//!   trust lives only in RAM.
//! - Per-cave attestable identity binding (SP-C1.3 wiring to
//!   `caves/cave.rs`). Today: stub registry of one identity.
//! - CBOR serialization of the wire format per IETF RATS RFC 9334
//!   (SP-C1.7). Today: simple length-prefixed byte concat.
//! - External verifier tool (SP-C1.8). The Quote bytes are
//!   verifiable in-process via `verify_quote_local` below; a stand-
//!   alone verifier lands later.
//!
//! ## Threat model assumptions
//!
//! - Caves cannot reach into this module's static state. Per audit
//!   ISO-006 / ISO-007 / per-cave ASIDs (week 11), kernel-mode
//!   memory is unreachable from EL0.
//! - The kernel measurement is trusted because the bootloader
//!   verified the LMS signature on the kernel image before
//!   jump-to-Rust (SP-B4 wiring). Without the bootloader chain in
//!   place, attestation reduces to "this kernel claims to be itself"
//!   — useful for development, not for production until SP-B4 lands.
//!
//! ## See also
//!
//! - REQ-ATT-001 (API surface), REQ-ATT-005 (per-cave identity),
//!   REQ-ATT-007 (RATS protocol envelope).
//! - Strategic differentiator #3 in the master plan.
//! - `docs/FIPS_140_3_MODULE_BOUNDARY.md` §7.8 (CSP table).

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;

use core::sync::atomic::{AtomicBool, Ordering};

use crate::crypto::pq_cnsa::{Dsa87Key, verify_mldsa87, MLDSA87_PK_LEN, MLDSA87_SIG_LEN};

/// Length of a SHA-384 kernel measurement.
pub const KERNEL_MEASUREMENT_LEN: usize = 48;

/// Length of a per-cave measurement (SHA-384 over code+config).
pub const CAVE_MEASUREMENT_LEN: usize = 48;

/// Length of a quote nonce (RATS recommends 32-byte freshness nonce).
pub const NONCE_LEN: usize = 32;

/// Maximum size of a Claims payload. RATS profiles typically run a
/// few hundred bytes; cap conservatively.
pub const MAX_CLAIMS_LEN: usize = 4096;

/// Caller-supplied claim set. Bytes are signed verbatim — the
/// caller chooses the encoding (CBOR per RATS RFC 9334 §7 is the
/// expected production choice; raw bytes acceptable today).
#[derive(Clone, Debug)]
pub struct Claims {
    pub bytes: Vec<u8>,
}

impl Claims {
    /// Construct from raw bytes. Returns Err if longer than MAX_CLAIMS_LEN.
    pub fn from_bytes(b: &[u8]) -> Result<Self, &'static str> {
        if b.len() > MAX_CLAIMS_LEN {
            return Err("attest: claims exceed MAX_CLAIMS_LEN");
        }
        Ok(Self { bytes: b.to_vec() })
    }
}

/// SHA-384 hash of the loaded kernel image. Trusted because the
/// bootloader verified the LMS signature before jump-to-Rust
/// (SP-B4). Until SP-B4 lands, this is a fixed placeholder so the
/// API can be exercised end-to-end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelMeasurement(pub [u8; KERNEL_MEASUREMENT_LEN]);

impl KernelMeasurement {
    /// Return the current kernel measurement. Read from `MEASUREMENT`
    /// after `init_kernel_measurement()` runs at boot. Before init,
    /// returns the placeholder bytes (all-zero).
    pub fn current() -> Self {
        unsafe {
            let ptr = core::ptr::addr_of!(MEASUREMENT);
            Self(*ptr)
        }
    }
}

unsafe extern "C" {
    /// SP-C1.2: linker-script-provided boundary of the .text section.
    /// Defined in `linker.ld` / `linker_apple.ld`. These are symbols,
    /// not values — take their addresses with `core::ptr::addr_of`.
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
}

/// Slot for the kernel measurement, populated once at boot by
/// `init_kernel_measurement()`. Read by `KernelMeasurement::current()`.
static mut MEASUREMENT: [u8; KERNEL_MEASUREMENT_LEN] = [0u8; KERNEL_MEASUREMENT_LEN];
static MEASUREMENT_INIT: AtomicBool = AtomicBool::new(false);

/// Compute and cache the SHA-384 hash of the loaded kernel image
/// (text section + rodata). Must be called once at boot, BEFORE any
/// caller invokes `KernelMeasurement::current()` for a real claim.
///
/// Safe to call multiple times — only the first call computes; later
/// calls return early.
///
/// SAFETY: reads linker-provided memory ranges `__text_start..__text_end`
/// and `__rodata_start..__rodata_end`. Those ranges are mapped read-
/// executable / read-only respectively in the kernel page tables; the
/// read is always safe under EL1.
pub fn init_kernel_measurement() {
    if MEASUREMENT_INIT.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe {
        let text_start = core::ptr::addr_of!(__text_start) as usize;
        let text_end = core::ptr::addr_of!(__text_end) as usize;
        let rodata_start = core::ptr::addr_of!(__rodata_start) as usize;
        let rodata_end = core::ptr::addr_of!(__rodata_end) as usize;

        let text_len = text_end.saturating_sub(text_start);
        let rodata_len = rodata_end.saturating_sub(rodata_start);

        let text_slice = core::slice::from_raw_parts(text_start as *const u8, text_len);
        let rodata_slice = core::slice::from_raw_parts(rodata_start as *const u8, rodata_len);

        // SHA-384 streaming over (text || rodata).
        use sha2::{Sha384 as Sha384Hasher, Digest};
        let mut hasher = Sha384Hasher::new();
        hasher.update(text_slice);
        hasher.update(rodata_slice);
        let out = hasher.finalize();
        let ptr = core::ptr::addr_of_mut!(MEASUREMENT);
        (*ptr).copy_from_slice(&out);

        crate::drivers::uart::puts("  [attest] kernel measurement computed (SHA-384 of text+rodata)\n");
    }
}

/// Per-cave identity. Caves cannot create or modify their own
/// CaveIdentity — the kernel binds it at cave-create time and the
/// caller (e.g., the cave loader in `caves::cave`) supplies the
/// measurement of the cave's code+config.
#[derive(Clone, Debug)]
pub struct CaveIdentity {
    /// Human-readable cave name (≤ 64 bytes).
    pub name: Vec<u8>,
    /// SHA-384 hash of the cave's loaded code+config.
    pub measurement: [u8; CAVE_MEASUREMENT_LEN],
}

impl CaveIdentity {
    pub fn new(name: &[u8], measurement: [u8; CAVE_MEASUREMENT_LEN]) -> Result<Self, &'static str> {
        if name.len() > 64 {
            return Err("attest: cave name > 64 bytes");
        }
        Ok(Self { name: name.to_vec(), measurement })
    }
}

// ── Per-cave identity registry (SP-C1.3) ─────────────────────────
//
// Storage shape: one slot per cave (MAX_CAVES = 32). Each slot
// holds a fixed-size StoredCaveIdentity to keep the registry as a
// no-heap static. Public API surface continues to use CaveIdentity
// with its Vec<u8> name; the conversion happens at register-time.

const STORED_NAME_MAX: usize = 64;

#[derive(Copy, Clone)]
struct StoredCaveIdentity {
    present: bool,
    name_len: u8,                            // 0..=64
    name: [u8; STORED_NAME_MAX],
    measurement: [u8; CAVE_MEASUREMENT_LEN],
}

const EMPTY_STORED: StoredCaveIdentity = StoredCaveIdentity {
    present: false,
    name_len: 0,
    name: [0u8; STORED_NAME_MAX],
    measurement: [0u8; CAVE_MEASUREMENT_LEN],
};

/// One slot per cave. Index by cave_id.
static mut CAVE_REGISTRY: [StoredCaveIdentity; crate::caves::cave::MAX_CAVES] =
    [EMPTY_STORED; crate::caves::cave::MAX_CAVES];

/// Register an identity for a specific cave. Intended caller:
/// `caves::cave` at cave-create or cave-enter time. Overwrites any
/// existing entry (caves can re-register if their measurement changes,
/// e.g., a config-reload). Returns Err on cave_id out-of-range or
/// name-too-long.
///
/// SP-C1.3.1 (future) tightens this so only privileged cave-
/// management code can call it. Today: trust the caller; the public
/// surface from EL0 routes through cave-policy gates before reaching
/// here.
///
/// SAFETY: single-writer assumption. Multiple concurrent registers
/// for the SAME cave_id race; concurrent registers for DIFFERENT
/// cave_ids are independent. The cave-create path holds the cave
/// lock so collisions don't happen in practice.
pub fn register_cave_identity(
    cave_id: usize,
    name: &[u8],
    measurement: [u8; CAVE_MEASUREMENT_LEN],
) -> Result<(), &'static str> {
    if cave_id >= crate::caves::cave::MAX_CAVES {
        return Err("attest: cave_id out of range");
    }
    if name.len() > STORED_NAME_MAX {
        return Err("attest: cave name > 64 bytes");
    }
    let mut stored = StoredCaveIdentity {
        present: true,
        name_len: name.len() as u8,
        name: [0u8; STORED_NAME_MAX],
        measurement,
    };
    stored.name[..name.len()].copy_from_slice(name);
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CAVE_REGISTRY);
        (*ptr)[cave_id] = stored;
    }
    Ok(())
}

/// Clear a cave's identity (cave-teardown hook).
pub fn unregister_cave_identity(cave_id: usize) {
    if cave_id >= crate::caves::cave::MAX_CAVES { return; }
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CAVE_REGISTRY);
        (*ptr)[cave_id] = EMPTY_STORED;
    }
}

/// Look up the identity for the currently-active cave. Returns None
/// if no cave is active OR the active cave has no registered identity.
fn current_cave_identity() -> Option<CaveIdentity> {
    let cave_id = crate::caves::cave::get_active();
    if cave_id == usize::MAX || cave_id >= crate::caves::cave::MAX_CAVES {
        return None;
    }
    unsafe {
        let ptr = core::ptr::addr_of!(CAVE_REGISTRY);
        let stored = &(*ptr)[cave_id];
        if !stored.present { return None; }
        Some(CaveIdentity {
            name: stored.name[..stored.name_len as usize].to_vec(),
            measurement: stored.measurement,
        })
    }
}

/// Test-only: register identity for cave 0 directly. Equivalent to
/// the old single-slot stub. Kept for `smoke()` and any caller that
/// wants the SP-C1.1-era behaviour. Real callers use
/// `register_cave_identity` with the proper cave_id.
///
/// SP-C1.3 compat shim — remove when no caller depends on it.
pub fn set_local_cave_identity(id: CaveIdentity) {
    // Use cave 0 as the implicit slot. Tolerates name-length up to
    // STORED_NAME_MAX; truncates silently above that (this is a
    // back-compat helper, not a load-bearing API).
    let name = if id.name.len() > STORED_NAME_MAX {
        &id.name[..STORED_NAME_MAX]
    } else {
        &id.name[..]
    };
    let _ = register_cave_identity(0, name, id.measurement);
}

/// Attestation quote — the produced envelope. Signed payload is:
///   `kernel_meas (48) || cave_meas (48) || nonce (32) ||
///    cave_name_len_be (2) || cave_name || claims_len_be (4) || claims`
/// The signature covers exactly those bytes (no transcript hashing).
/// Wire encoding today is the raw concat + 4627-byte ML-DSA-87
/// signature appended; SP-C1.7 swaps to CBOR per RATS.
#[derive(Clone, Debug)]
pub struct Quote {
    pub kernel_measurement: KernelMeasurement,
    pub cave_identity: CaveIdentity,
    pub nonce: [u8; NONCE_LEN],
    pub claims: Claims,
    pub signature: Vec<u8>,  // ML-DSA-87, MLDSA87_SIG_LEN bytes
    pub verifying_key: Vec<u8>, // ML-DSA-87 pub, MLDSA87_PK_LEN bytes; lands inline for SP-C1.1 testing — SP-C1.4 moves this to an out-of-band endorsement chain
}

/// Bytes that the signature covers. Pure function — same inputs always
/// produce the same byte sequence (for verifier reproducibility).
pub fn signed_payload(
    kernel_meas: &KernelMeasurement,
    cave: &CaveIdentity,
    nonce: &[u8; NONCE_LEN],
    claims: &Claims,
) -> Vec<u8> {
    let name_len = cave.name.len() as u16;
    let claims_len = claims.bytes.len() as u32;
    let mut out = vec![];
    out.extend_from_slice(&kernel_meas.0);
    out.extend_from_slice(&cave.measurement);
    out.extend_from_slice(nonce);
    out.extend_from_slice(&name_len.to_be_bytes());
    out.extend_from_slice(&cave.name);
    out.extend_from_slice(&claims_len.to_be_bytes());
    out.extend_from_slice(&claims.bytes);
    out
}

// ── In-memory attestation key (SP-C1.1 placeholder) ──────────────
//
// SP-C1.4 (M4 SEP) and SP-C1.5 (Caliptra) replace this with a
// hardware-rooted key. Today the key is generated at first use and
// lives in kernel-private heap; quotes verify against the
// `verifying_key` embedded in each Quote (out-of-band endorsement
// chain for the verifier).

static ATTEST_KEY_INIT: AtomicBool = AtomicBool::new(false);
static mut ATTEST_KEY: Option<Dsa87Key> = None;

fn ensure_attest_key() -> Result<(), &'static str> {
    if ATTEST_KEY_INIT.load(Ordering::Acquire) {
        return Ok(());
    }
    // First-call init. Race-safe because we never replace once set;
    // first writer wins, others see the same key on next read.
    if !ATTEST_KEY_INIT.swap(true, Ordering::AcqRel) {
        let kp = Dsa87Key::generate();
        unsafe {
            let ptr = core::ptr::addr_of_mut!(ATTEST_KEY);
            (*ptr) = Some(kp);
        }
    }
    Ok(())
}

/// Look up a registered cave identity by ID (None if unregistered or
/// out of range). Public so callers (test harnesses, the cave-mgmt UI,
/// the future external verifier) can inspect what's registered.
pub fn cave_identity(cave_id: usize) -> Option<CaveIdentity> {
    if cave_id >= crate::caves::cave::MAX_CAVES { return None; }
    unsafe {
        let ptr = core::ptr::addr_of!(CAVE_REGISTRY);
        let stored = &(*ptr)[cave_id];
        if !stored.present { return None; }
        Some(CaveIdentity {
            name: stored.name[..stored.name_len as usize].to_vec(),
            measurement: stored.measurement,
        })
    }
}

/// Produce a signed Quote naming `cave_id` as the asserting cave.
/// Used by `quote()` (which calls with `cave::get_active()`) and by
/// `smoke()` (which calls with a fixed cave_id, since the smoke
/// harness runs without an active cave context).
pub fn quote_for_cave(
    cave_id: usize,
    nonce: &[u8; NONCE_LEN],
    claims: Claims,
) -> Result<Quote, &'static str> {
    ensure_attest_key()?;
    let cave = cave_identity(cave_id).ok_or("attest: cave identity not registered for cave_id")?;
    let kernel_meas = KernelMeasurement::current();
    let payload = signed_payload(&kernel_meas, &cave, nonce, &claims);

    let (sig, vk) = unsafe {
        let ptr = core::ptr::addr_of!(ATTEST_KEY);
        let kp_ref = (*ptr).as_ref().ok_or("attest: key not initialized")?;
        let sig = kp_ref.sign(&payload)?;
        let vk = kp_ref.verifying_bytes();
        (sig, vk)
    };

    Ok(Quote {
        kernel_measurement: kernel_meas,
        cave_identity: cave,
        nonce: *nonce,
        claims,
        signature: sig,
        verifying_key: vk,
    })
}

/// Public API: produce a signed Quote attesting to:
///   - The current kernel measurement
///   - The currently-active cave's identity (per `cave::get_active`)
///   - The caller-supplied claims
///   - A freshness nonce supplied by the verifier
///
/// Returns an error if no cave is active OR the active cave has no
/// registered identity OR signing fails.
pub fn quote(nonce: &[u8; NONCE_LEN], claims: Claims) -> Result<Quote, &'static str> {
    let cave_id = crate::caves::cave::get_active();
    if cave_id == usize::MAX {
        return Err("attest: no active cave (call quote_for_cave with explicit cave_id instead)");
    }
    quote_for_cave(cave_id, nonce, claims)
}

/// Verify a Quote produced by `quote()` above. Returns Ok iff the
/// signature is valid over the canonical signed payload AND the
/// verifying key length matches MLDSA87_PK_LEN AND the signature
/// length matches MLDSA87_SIG_LEN.
///
/// This is a local-process verifier — the caller still needs to
/// validate that `q.verifying_key` chains to a trusted endorsement
/// (operator-CA-attested). SP-C1.6 wires the endorsement-chain
/// validator.
pub fn verify_quote_local(q: &Quote) -> Result<(), &'static str> {
    if q.verifying_key.len() != MLDSA87_PK_LEN {
        return Err("attest: bad verifying key length");
    }
    if q.signature.len() != MLDSA87_SIG_LEN {
        return Err("attest: bad signature length");
    }
    let payload = signed_payload(&q.kernel_measurement, &q.cave_identity, &q.nonce, &q.claims);
    verify_mldsa87(&q.verifying_key, &payload, &q.signature)
}

// ── Boot-time smoke (NOT a KAT — runs on demand only) ────────────
//
// The full attestation round-trip exercises ML-DSA-87 keygen + sign +
// verify, which (per SP-B1.3 LMS experience) takes seconds under QEMU
// emulation. We do NOT wire this into run_self_tests for the same
// reason LMS isn't there: boot-smoke timeout. Exposed for shell-
// command testing instead (SP-C1.8 follow-up could add a dedicated
// `attest-smoke` shell command).

/// Round-trip self-test: register a fake cave identity for cave 0,
/// produce a quote naming cave 0 explicitly (since smoke runs without
/// an active cave context), verify it locally, tamper-check. Useful
/// for SP-C1.x regression checking, not wired into boot KAT (ML-DSA-87
/// keygen too slow under QEMU emulation).
pub fn smoke() -> Result<(), &'static str> {
    let fake_meas = [0xa5u8; CAVE_MEASUREMENT_LEN];
    register_cave_identity(0, b"test-cave", fake_meas)?;

    let nonce = [0x42u8; NONCE_LEN];
    let claims = Claims::from_bytes(b"smoke-claim:hello")?;
    let q = quote_for_cave(0, &nonce, claims)?;

    // Positive verify.
    verify_quote_local(&q)?;

    // Tamper-detect: flip a claims byte by reconstructing the Quote
    // with a different claims payload + the same signature → must fail.
    let mut tampered = q.clone();
    tampered.claims.bytes[0] ^= 0x01;
    if verify_quote_local(&tampered).is_ok() {
        return Err("attest smoke: verify accepted tampered claims");
    }

    // Tamper-detect: bit-flip the signature → must fail.
    let mut bad_sig = q.clone();
    bad_sig.signature[0] ^= 0x01;
    if verify_quote_local(&bad_sig).is_ok() {
        return Err("attest smoke: verify accepted tampered signature");
    }
    Ok(())
}
