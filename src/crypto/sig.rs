// Sphragis — Signature verification (Ed25519 + ECDSA-P256).
//
// Thin wrappers over `ed25519-compact` and `p256` that the boot/TLS paths
// can call without knowing the underlying crate. All inputs are byte
// slices; failure is "Err with a fixed string" so the caller can log
// without leaking which check failed.
//
// FLv2-NEW-010: initrd Ed25519 signature verification.
// NEW-CRYPTO-010 / NET2-001: TLS CertificateVerify (ECDSA P-256).

#![allow(dead_code)]

/// Verify an Ed25519 signature.
// /
/// `pubkey` — 32-byte little-endian Edwards25519 point.
/// `sig` — 64-byte (R || S) signature.
/// `msg` — message bytes (this function hashes them internally per RFC 8032).
// /
/// Returns `Ok(())` on a valid signature, `Err("ed25519 verify failed")`
/// on any failure (bad encoding, low-order, equation mismatch).
pub fn ed25519_verify(pubkey: &[u8; 32], sig: &[u8; 64], msg: &[u8]) -> Result<(), &'static str> {
    use ed25519_compact::{PublicKey, Signature};

    let pk = PublicKey::from_slice(pubkey).map_err(|_| "ed25519 verify failed")?;
    let sg = Signature::from_slice(sig).map_err(|_| "ed25519 verify failed")?;
    pk.verify(msg, &sg).map_err(|_| "ed25519 verify failed")
}

/// Verify an ECDSA-P256 signature over an already-hashed message digest.
// /
/// `pubkey` — 65 bytes uncompressed SEC1 (0x04 || X || Y) **or** 33 bytes
/// compressed. We accept both for X.509 SPKI compatibility.
/// `digest` — 32-byte SHA-256 digest of the signed bytes.
/// `sig_der` — DER-encoded ECDSA signature (`SEQUENCE { r INTEGER, s INTEGER }`).
// /
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
/// want to hash before calling ecdsa_p256_verify. Backed by the `sha2`
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

/// SHA-384 digest. ECDSA-P384 + RSA-PSS-SHA384 paths use this.
pub fn sha384_digest(msg: &[u8]) -> [u8; 48] {
    use sha2::{Sha384, Digest};
    let mut h = Sha384::new();
    h.update(msg);
    let out = h.finalize();
    let mut r = [0u8; 48];
    r.copy_from_slice(&out);
    r
}

/// SHA-512 digest. RSA-PSS-SHA512 path uses this.
pub fn sha512_digest(msg: &[u8]) -> [u8; 64] {
    use sha2::{Sha512, Digest};
    let mut h = Sha512::new();
    h.update(msg);
    let out = h.finalize();
    let mut r = [0u8; 64];
    r.copy_from_slice(&out);
    r
}

/// Verify an ECDSA-P384 signature over an already-hashed message digest.
/// anchors under the ISRG Root X2 ECDSA P-384 root + TLS-1.3
/// ecdsa_secp384r1_sha384 (sig scheme 0x0503).
// /
/// `pubkey` — uncompressed SEC1 (0x04 || X || Y, 97 bytes) or compressed.
/// `digest` — 48-byte SHA-384 digest of the signed bytes.
/// `sig_der` — DER-encoded ECDSA signature.
pub fn ecdsa_p384_verify(pubkey: &[u8], digest: &[u8; 48], sig_der: &[u8])
    -> Result<(), &'static str>
{
    use p384::ecdsa::{VerifyingKey, Signature, signature::hazmat::PrehashVerifier};
    use p384::EncodedPoint;

    let ep = EncodedPoint::from_bytes(pubkey).map_err(|_| "ecdsa-p384 verify failed")?;
    let vk = VerifyingKey::from_encoded_point(&ep).map_err(|_| "ecdsa-p384 verify failed")?;
    let sig = Signature::from_der(sig_der).map_err(|_| "ecdsa-p384 verify failed")?;
    vk.verify_prehash(digest, &sig).map_err(|_| "ecdsa-p384 verify failed")
}

/// Verify an RSA PKCS#1 v1.5 signature with SHA-256.
/// cert chains where the parent's signature uses the
/// classic sha256WithRSAEncryption OID (1.2.840.113549.1.1.11) — most
/// public CA self-signatures.
// /
/// `pubkey_der` — DER-encoded RSAPublicKey (`SEQUENCE { n, e }`) — i.e.
/// the bare RSA pubkey, not the SubjectPublicKeyInfo
/// wrapper. Caller is responsible for stripping SPKI.
/// `msg` — bytes that were signed (this fn hashes them).
/// `sig` — raw signature bytes.
pub fn rsa_pkcs1v15_sha256_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs1v15::Pkcs1v15Sign,
              traits::SignatureScheme};
    use sha2::{Sha256, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa pkcs1v15 verify: bad pubkey")?;
    let mut h = Sha256::new();
    h.update(msg);
    let digest = h.finalize();
    Pkcs1v15Sign::new::<Sha256>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa pkcs1v15 sha256 verify failed")
}

/// RSA PKCS#1 v1.5 with SHA-384. .
pub fn rsa_pkcs1v15_sha384_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs1v15::Pkcs1v15Sign,
              traits::SignatureScheme};
    use sha2::{Sha384, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa pkcs1v15 verify: bad pubkey")?;
    let mut h = Sha384::new();
    h.update(msg);
    let digest = h.finalize();
    Pkcs1v15Sign::new::<Sha384>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa pkcs1v15 sha384 verify failed")
}

/// RSA PKCS#1 v1.5 with SHA-512. .
pub fn rsa_pkcs1v15_sha512_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs1v15::Pkcs1v15Sign,
              traits::SignatureScheme};
    use sha2::{Sha512, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa pkcs1v15 verify: bad pubkey")?;
    let mut h = Sha512::new();
    h.update(msg);
    let digest = h.finalize();
    Pkcs1v15Sign::new::<Sha512>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa pkcs1v15 sha512 verify failed")
}

/// Verify an RSA-PSS signature with SHA-256 / 384 / 512. /// TLS-1.3 CertificateVerify uses these (rsa_pss_rsae_sha256 = 0x0804,
/// rsa_pss_rsae_sha384 = 0x0805, rsa_pss_rsae_sha512 = 0x0806). Cert
/// sigs that use OID 1.2.840.113549.1.1.10 (rsassa-pss) also land here.
// /
/// PSS salt length defaults to "match hash output size" per RFC 8446,
/// which is the modern interpretation that all real implementations
/// produce.
pub fn rsa_pss_sha256_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pss::Pss,
              traits::SignatureScheme};
    use sha2::{Sha256, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa-pss verify: bad pubkey")?;
    let mut h = Sha256::new();
    h.update(msg);
    let digest = h.finalize();
    Pss::new::<Sha256>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa-pss sha256 verify failed")
}

pub fn rsa_pss_sha384_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pss::Pss,
              traits::SignatureScheme};
    use sha2::{Sha384, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa-pss verify: bad pubkey")?;
    let mut h = Sha384::new();
    h.update(msg);
    let digest = h.finalize();
    Pss::new::<Sha384>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa-pss sha384 verify failed")
}

pub fn rsa_pss_sha512_verify(pubkey_der: &[u8], msg: &[u8], sig: &[u8])
    -> Result<(), &'static str>
{
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pss::Pss,
              traits::SignatureScheme};
    use sha2::{Sha512, Digest};

    let pk = RsaPublicKey::from_pkcs1_der(pubkey_der)
        .map_err(|_| "rsa-pss verify: bad pubkey")?;
    let mut h = Sha512::new();
    h.update(msg);
    let digest = h.finalize();
    Pss::new::<Sha512>()
        .verify(&pk, digest.as_slice(), sig)
        .map_err(|_| "rsa-pss sha512 verify failed")
}
