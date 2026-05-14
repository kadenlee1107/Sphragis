---
type: concept-note
topic: crypto
---

# Cryptography stack

> Every primitive in Sphragis is from RustCrypto, audited, and used at the syscall boundary or below. This note is the catalog: what each primitive is for, where it lives, and what's deliberate about the choice.

## The full set

| Primitive             | Used for                                         | Where                       |
| --------------------- | ------------------------------------------------ | --------------------------- |
| **ChaCha20-Poly1305** | BatFS block AEAD; audit ring AEAD                | [[_generated/src/batfs]], [[_generated/src/security/audit]] |
| **AES-GCM**           | TLS 1.3 record-layer encryption                  | [[_generated/src/net/tls.rs]]    |
| **Argon2id**          | passphrase → master key derivation               | [[_generated/src/auth]]    |
| **SHA-256 / SHA-384** | TLS transcript hash, BatFS Merkle nodes          | [[_generated/src/net/tls.rs]], [[_generated/src/batfs]] |
| **HMAC**              | TLS HKDF construction                            | [[_generated/src/net/tls.rs]]    |
| **X25519**            | TLS classical key agreement                      | [[_generated/src/net/tls.rs]], [[_generated/src/net/tls_hybrid.rs]] |
| **ML-KEM-768**        | TLS post-quantum key agreement (hybrid w/ X25519)| [[_generated/src/net/tls_hybrid.rs]] |
| **ECDSA P-256 / P-384** | TLS certificate signature verification         | [[_generated/src/net/x509.rs]]   |
| **RSA PKCS#1v1.5 / RSA-PSS** | TLS certificate signature verification    | [[_generated/src/net/x509.rs]]   |
| **Ed25519**           | Reserved (planned for cave script signing)       | not yet wired               |
| **ML-DSA-65**         | Reserved (post-quantum signature, planned)       | not yet wired               |

## Choices worth understanding

### Why ChaCha20-Poly1305 for BatFS, not AES-GCM

Chosen because:
1. ChaCha20 is constant-time on every aarch64 implementation we ship; AES-GCM is constant-time only with hardware AES instructions, and we don't want to fork the security guarantee on whether the M4 advertises AES.
2. ChaCha20 has no nonce-misuse-resistance footgun the way AES-GCM does at high traffic volumes — for BatFS where blocks are independent, this matters.
3. The RustCrypto `chacha20poly1305` crate is small and audited.

### Why Argon2id at 8 MiB / 3 passes

The audit's V11 numbers. 8 MiB memory cost / 3 iterations / single thread on the M4 takes ~250ms. That's on the edge of what's acceptable for unlock UX while being **six orders of magnitude harder** to brute-force than legacy SHA-based KDFs on commodity hardware.

The KDF parameters are visible to the user via the lock-screen field meta (`argon2id · 64 MB · t=3` shown on the marketing site corresponds to the design-doc default; the in-kernel default is currently 8 MiB / 3p — these have drifted; reconcile before the next eval build).

### Why hybrid X25519 + ML-KEM-768

Per `draft-ietf-tls-ecdhe-mlkem-04`. The wire format is ML-KEM-768 ciphertext concatenated with X25519 ephemeral public key, derived shared secret used as raw concatenation per the standard. See [[Concepts/TLS Hardening Journey#Hybrid PQ wire format]].

The hybrid construction means: even if ML-KEM-768 is later broken, X25519's classical security is still intact. Even if a quantum computer breaks X25519, ML-KEM-768's lattice security is still intact. **Both** would have to fall.

### Six trust anchors, no system trust store

[[_generated/src/net/x509.rs]] embeds DER bytes for exactly six CA roots:

- ISRG Root X1 (RSA 4096) — anchors most Let's Encrypt-issued chains
- ISRG Root X2 (ECDSA P-384) — Let's Encrypt's ECDSA root
- Amazon Root CA 1 (RSA 2048)
- DigiCert Global Root CA (RSA 2048)
- DigiCert Global Root G2 (RSA 2048)
- GTS Root R4 (ECDSA P-384) — required for `pq.cloudflareresearch.com`

There is no system trust store. Adding a CA requires recompiling the kernel image. This is a deliberate tradeoff: a smaller list is easier to audit, harder to subvert, and removes the question of "whose CA store version are we on."

### Constant-cost abort discipline

Every TLS abort in the kernel takes the same amount of work regardless of *which* check failed. A timing observer measuring abort time learns nothing about which specific check tripped — was it hostname? signature? trust anchor? validity period? Constant cost: every check runs to completion, flags accumulate, the error code is selected at the end.

This came out of V6-SIDE-002 and has been preserved in every change since. Adding a check is *adding a flag*, not adding an early return.

## Where it does NOT live

- No crypto runs in userspace caves. Caves never see TLS bytes; they hand the kernel a hostname and get back a plaintext fd. See [[_generated/DESIGN_HTTPS_SYSCALL.md]].
- No crypto agility ("negotiate the cipher suite at runtime"). The set above is the set; alternatives are cryptographic policy decisions, not runtime configuration.
- No bring-your-own-CA. The trust store is hard-coded.

## Audit cadence

Every primitive in this stack has been read end-to-end at least once. New crates require the same. The discipline matters more than any specific primitive — a fresh CVE in a vendored crate is a whole-image rebuild, and we want to *want* to do that.

## Related

- [[Concepts/TLS Hardening Journey]] — the timeline of this stack getting hardened
- [[Concepts/Audit Ring Contract]] — same AEAD as BatFS, by design
- [[_generated/DESIGN_CRYPTO.md]] — design doc for the full stack
