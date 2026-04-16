// Bat_OS — Signature verification (Ed25519 + ECDSA-P256).
//
// Thin wrappers over `ed25519-compact` and `p256` that the boot/TLS paths
// can call without knowing the underlying crate.  All inputs are byte
// slices; failure is "Err with a fixed string" so the caller can log
// without leaking which check failed.
//
// FLv2-NEW-010: initrd Ed25519 signature verification.
// NEW-CRYPTO-010 / NET2-001: TLS CertificateVerify (ECDSA P-256).

#![allow(dead_code)]

/// Verify an Ed25519 signature.
///
/// `pubkey` — 32-byte little-endian Edwards25519 point.
/// `sig`    — 64-byte (R || S) signature.
/// `msg`    — message bytes (this function hashes them internally per RFC 8032).
///
/// Returns `Ok(())` on a valid signature, `Err("ed25519 verify failed")`
/// on any failure (bad encoding, low-order, equation mismatch).
pub fn ed25519_verify(pubkey: &[u8; 32], sig: &[u8; 64], msg: &[u8]) -> Result<(), &'static str> {
    use ed25519_compact::{PublicKey, Signature};

    let pk = PublicKey::from_slice(pubkey).map_err(|_| "ed25519 verify failed")?;
    let sg = Signature::from_slice(sig).map_err(|_| "ed25519 verify failed")?;
    pk.verify(msg, &sg).map_err(|_| "ed25519 verify failed")
}

/// Verify an ECDSA-P256 signature over an already-hashed message digest.
///
/// `pubkey` — 65 bytes uncompressed SEC1 (0x04 || X || Y) **or** 33 bytes
///            compressed.  We accept both for X.509 SPKI compatibility.
/// `digest` — 32-byte SHA-256 digest of the signed bytes.
/// `sig_der` — DER-encoded ECDSA signature (`SEQUENCE { r INTEGER, s INTEGER }`).
///
/// Returns `Ok(())` on valid, `Err("ecdsa verify failed")` otherwise.
pub fn ecdsa_p256_verify(pubkey: &[u8], digest: &[u8; 32], sig_der: &[u8])
    -> Result<(), &'static str>
{
    use p256::ecdsa::{VerifyingKey, Signature, signature::hazmat::PrehashVerifier};
    use p256::EncodedPoint;

    let ep = EncodedPoint::from_bytes(pubkey).map_err(|_| "ecdsa verify failed")?;
    let vk = VerifyingKey::from_encoded_point(&ep).map_err(|_| "ecdsa verify failed")?;
    let sig = Signature::from_der(sig_der).map_err(|_| "ecdsa verify failed")?;
    vk.verify_prehash(digest, &sig).map_err(|_| "ecdsa verify failed")
}

/// SHA-256 helper that returns a fixed-size digest, used by callers that
/// want to hash before calling ecdsa_p256_verify.  Backed by the `sha2`
/// crate to share with whatever else we pull in; same byte-for-byte
/// output as `crate::crypto::sha256`.
pub fn sha256_digest(msg: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut h = Sha256::new();
    h.update(msg);
    let out = h.finalize();
    let mut r = [0u8; 32];
    r.copy_from_slice(&out);
    r
}
