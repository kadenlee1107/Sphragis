//! Sphragis crypto policy gate — SP-B1.6.
//!
//! The community build accepts every algorithm in the tree. The
//! gov-strict build (`cargo build --features gov-strict`) enforces
//! a CNSA-2.0-only policy at the policy layer, refusing weak
//! algorithms BEFORE they are negotiated or used.
//!
//! ## What gov-strict rejects
//!
//! | Category | Rejected | Replacement |
//! |---|---|---|
//! | Symmetric encryption | AES-128 (all modes) | AES-256 (GCM, GCM-SIV, XTS, CTR) |
//! | Hashing for signatures | SHA-1, SHA-256 | SHA-384, SHA-512 |
//! | Public-key signing | RSA, ECDSA | ML-DSA-87, LMS, XMSS |
//! | Public-key key-exchange | RSA, classical ECDH-only | ML-KEM-1024 (or hybrid with X25519 for TLS interop) |
//! | AEAD outside CNSA | plain ChaCha20-Poly1305 | AES-256-GCM-SIV (BatFS), AES-256-GCM (TLS) |
//! | RNG | fail-soft on RNDR absent | fail-closed: kernel halts at boot |
//!
//! The policy gate is a small set of compile-time-evaluated boolean
//! constants + a `reject_or_continue` helper that callers invoke
//! when they're about to use a weak algorithm. The reject path is
//! a fail-stop: under gov-strict, an attempted negotiation of a
//! rejected algorithm halts the calling cave's operation with a
//! documented error code.
//!
//! ## What gov-strict does NOT do (yet)
//!
//! - Re-validate every existing call site. The cipher-suite tables
//!   in `src/net/tls.rs` and the X.509 verify paths still include
//!   RSA / ECDSA for community-build interop. SP-B1.6 wires the
//!   policy gate; SP-B1.6.1 (follow-up) sweeps every call site to
//!   route through it. Until that sweep, gov-strict callers MUST
//!   call `policy::ensure_cnsa_algo()` before invoking algorithm-
//!   selecting code.
//! - Replace the X.509 trust-anchor set. Sphragis ships 6 anchors,
//!   most of which are RSA. Replacing them is a separate sub-project
//!   (SP-CRT-002 territory).

#![allow(dead_code)]

/// Compile-time boolean — is gov-strict mode enabled in this build?
/// Use in `if` guards or `match` arms to fork policy decisions.
#[cfg(feature = "gov-strict")]
pub const GOV_STRICT: bool = true;
#[cfg(not(feature = "gov-strict"))]
pub const GOV_STRICT: bool = false;

/// Enumerated algorithm categories the policy gate knows about.
/// Add new variants as new algorithms are wired through Sphragis.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Algo {
    // Symmetric ciphers
    Aes128Gcm,
    Aes128GcmSiv,
    Aes128Xts,
    Aes128Ctr,
    Aes256Gcm,
    Aes256GcmSiv,
    Aes256Xts,
    Aes256Ctr,
    ChaCha20Poly1305,         // plain — rejected outside CNSA context
    XChaCha20Poly1305,        // extended-nonce — rejected outside CNSA context
    // Hashes
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Blake2s,
    Blake3,
    // Public-key signatures
    RsaPkcs1Sha256,
    RsaPkcs1Sha384,
    RsaPssSha256,
    RsaPssSha384,
    EcdsaP256Sha256,
    EcdsaP384Sha384,
    Ed25519,
    MlDsa65,
    MlDsa87,
    Lms,
    Xmss,
    // Public-key KEM / key-exchange
    Rsa2048Encrypt,
    X25519,
    EcdhP256,
    EcdhP384,
    MlKem768,
    MlKem1024,
    X25519MlKem768,           // TLS hybrid; allowed for TLS interop
    // MACs
    HmacSha256,
    HmacSha384,
    HmacSha512,
}

/// Returns true iff this algorithm is permitted in the current
/// build profile.
///
/// Under gov-strict, every category-3-or-lower algorithm and every
/// non-CNSA-listed primitive is rejected. The community build is
/// permissive: every variant returns true.
pub const fn is_permitted(algo: Algo) -> bool {
    if !GOV_STRICT { return true; }

    // Gov-strict allowlist. Anything not explicitly listed is denied.
    matches!(algo,
        // Symmetric: AES-256 only.
        Algo::Aes256Gcm | Algo::Aes256GcmSiv | Algo::Aes256Xts | Algo::Aes256Ctr
        // Hashing: SHA-384/512 + SHA-3 wide variants only.
        | Algo::Sha384 | Algo::Sha512
        | Algo::Sha3_384 | Algo::Sha3_512
        // Signatures: ML-DSA-87 + LMS + XMSS only.
        | Algo::MlDsa87 | Algo::Lms | Algo::Xmss
        // KEM: ML-KEM-1024 only (TLS hybrid acceptable for interop).
        | Algo::MlKem1024 | Algo::X25519MlKem768
        // MACs: HMAC-SHA-384/512 only.
        | Algo::HmacSha384 | Algo::HmacSha512
    )
}

/// Caller-side enforcement helper. Returns `Ok(())` if the algorithm
/// is permitted, `Err("...")` with a human-readable reason otherwise.
/// Wire this into negotiation entry points (TLS cipher-suite select,
/// X.509 signature-algorithm check, BatFS key-wrap entry, etc.).
pub fn ensure_permitted(algo: Algo) -> Result<(), &'static str> {
    if is_permitted(algo) {
        Ok(())
    } else {
        Err(reject_reason(algo))
    }
}

/// Static error message for each rejected algorithm under gov-strict.
/// Returned by `ensure_permitted` so callers can surface a precise
/// reason to the audit log.
const fn reject_reason(algo: Algo) -> &'static str {
    match algo {
        Algo::Aes128Gcm | Algo::Aes128GcmSiv | Algo::Aes128Xts | Algo::Aes128Ctr
            => "gov-strict: AES-128 rejected; CNSA 2.0 requires AES-256",
        Algo::ChaCha20Poly1305 | Algo::XChaCha20Poly1305
            => "gov-strict: ChaCha20-Poly1305 rejected outside CNSA context",
        Algo::Sha1 => "gov-strict: SHA-1 rejected (broken)",
        Algo::Sha256 => "gov-strict: SHA-256 rejected for new signing; SHA-384/512 only",
        Algo::Sha3_256 | Algo::Blake2s | Algo::Blake3
            => "gov-strict: narrow hash rejected; SHA-384/512 only for signing",
        Algo::RsaPkcs1Sha256 | Algo::RsaPkcs1Sha384 | Algo::RsaPssSha256 | Algo::RsaPssSha384
            => "gov-strict: RSA signing rejected; ML-DSA-87 / LMS / XMSS only",
        Algo::EcdsaP256Sha256 | Algo::EcdsaP384Sha384
            => "gov-strict: ECDSA rejected; ML-DSA-87 / LMS / XMSS only",
        Algo::Ed25519
            => "gov-strict: Ed25519 rejected for new signing; ML-DSA-87 / LMS / XMSS only",
        Algo::MlDsa65
            => "gov-strict: ML-DSA-65 (category 3) rejected; ML-DSA-87 (category 5) only",
        Algo::Rsa2048Encrypt
            => "gov-strict: RSA encryption rejected; ML-KEM-1024 only",
        Algo::X25519 | Algo::EcdhP256 | Algo::EcdhP384
            => "gov-strict: classical-only key-exchange rejected; ML-KEM-1024 or X25519MLKEM hybrid required",
        Algo::MlKem768
            => "gov-strict: ML-KEM-768 (category 3) rejected; ML-KEM-1024 (category 5) only",
        Algo::HmacSha256
            => "gov-strict: HMAC-SHA-256 rejected; HMAC-SHA-384 / HMAC-SHA-512 only",
        _ => "gov-strict: algorithm not on the CNSA 2.0 allowlist",
    }
}

/// Boot-time gov-strict assertion. Must be called early in the boot
/// path, BEFORE any cryptographic operation that could consume weak
/// entropy. Under gov-strict, halts the kernel via WFE-loop if the
/// platform doesn't have ARMv8.5 FEAT_RNG. Under the community build,
/// no-op.
///
/// This is the wiring required by REQ-CRY-011 (fail-closed RNG in
/// gov build) per the SP-B1.6 plan.
pub fn enforce_boot_policy() {
    if GOV_STRICT {
        crate::drivers::uart::puts("[policy] gov-strict build — enforcing CNSA 2.0 boot policy\n");
        crate::crypto::rng::require_hw_rng_or_halt();
    }
}

// ── Compile-time unit-test-style assertions ──────────────────────
//
// These exist as `const _` so they're evaluated at compile time
// and produce a build-time error if the policy matrix drifts.
// Add new assertions as new Algo variants are added.

#[cfg(feature = "gov-strict")]
const _: () = {
    assert!(is_permitted(Algo::Aes256Gcm));
    assert!(is_permitted(Algo::MlKem1024));
    assert!(is_permitted(Algo::MlDsa87));
    assert!(is_permitted(Algo::Sha384));
    assert!(is_permitted(Algo::Lms));
    assert!(!is_permitted(Algo::Aes128Gcm));
    assert!(!is_permitted(Algo::Sha1));
    assert!(!is_permitted(Algo::RsaPkcs1Sha256));
    assert!(!is_permitted(Algo::MlKem768));
    assert!(!is_permitted(Algo::MlDsa65));
    assert!(!is_permitted(Algo::Ed25519));
};

#[cfg(not(feature = "gov-strict"))]
const _: () = {
    // Community build: everything permitted.
    assert!(is_permitted(Algo::Aes128Gcm));
    assert!(is_permitted(Algo::Sha256));
    assert!(is_permitted(Algo::RsaPkcs1Sha256));
    assert!(is_permitted(Algo::MlKem1024));
};
