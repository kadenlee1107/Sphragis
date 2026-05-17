# Sphragis FIPS 140-3 Cryptographic Module Boundary

**Document version:** 1.0 (SP-B1.9, 2026-05-16)
**Module name:** Sphragis Cryptographic Module
**Module type:** Software Module (per ISO/IEC 19790:2012 §7.2.2)
**Target security level:** Level 1 (initial); Level 3 planned for hardware-bound key store (SP-C1 + SP-CRT-002)
**Operational environment:** Modifiable (per FIPS 140-3 §7.5)
**Approved-mode build:** `cargo build --release --target aarch64-unknown-none --features gov-strict`

This document defines the cryptographic-module boundary for Sphragis per the eleven areas of FIPS 140-3 §7. It is the starting artifact for the CMVP lab engagement planned in SP-B5.

---

## §7.1 — Cryptographic Module Specification

### 1. Logical boundary

The Sphragis Cryptographic Module is the union of:

| Source path | Role |
|---|---|
| `src/crypto/aes.rs` | AES-256 block cipher core |
| `src/crypto/aes_xts.rs` | AES-256-XTS for block-level storage encryption |
| `src/crypto/gcm_verified.rs` | AES-128/256-GCM AEAD (NIST SP 800-38D) |
| `src/crypto/chacha20poly1305.rs` | ChaCha20-Poly1305 AEAD (RFC 8439) — non-approved in gov-strict |
| `src/crypto/xchacha20poly1305.rs` | XChaCha20-Poly1305 AEAD — non-approved in gov-strict |
| `src/crypto/sha256.rs` | SHA-256 + HMAC-SHA-256 + HKDF-SHA-256 |
| `src/crypto/sha384.rs` | SHA-384 + HMAC-SHA-384 + HKDF-SHA-384 |
| `src/crypto/sha512.rs` | SHA-512 + HMAC-SHA-512 + HKDF-SHA-512 |
| `src/crypto/sha3.rs` | SHA-3 / SHAKE wrappers |
| `src/crypto/blake2s.rs` | BLAKE2s (WireGuard-spec mandate; non-approved) |
| `src/crypto/blake3.rs` | BLAKE3 (content-addressing; non-approved) |
| `src/crypto/hotp.rs` | HMAC-based one-time password (RFC 4226) |
| `src/crypto/totp.rs` | Time-based OTP (RFC 6238) |
| `src/crypto/rng.rs` | SHA-256-chained DRBG + ARMv8.5 FEAT_RNG mixing |
| `src/crypto/pq_hybrid.rs` | X25519MLKEM768 hybrid (TLS interop; ML-KEM at category 3) |
| `src/crypto/pq_hybrid_sig.rs` | Ed25519+ML-DSA-65 hybrid signature (TLS interop) |
| `src/crypto/pq_cnsa.rs` | ML-KEM-1024 + ML-DSA-87 (CNSA 2.0, category 5) |
| `src/crypto/lms.rs` | LMS hash-based signatures (RFC 8554 / NIST SP 800-208) |
| `src/crypto/sig.rs` | Ed25519 signature surface |
| `src/crypto/policy.rs` | Approved-mode policy gate (gov-strict feature) |

The boundary excludes: filesystem code (`src/fs/`), network protocols (`src/net/`), drivers (`src/drivers/`), and the kernel runtime (`src/main.rs`, `src/kernel/`). Those layers consume cryptographic services through the module's public API surface.

### 2. Physical boundary

In the **Level 1** target configuration, the physical boundary is the general-purpose computing platform on which Sphragis executes. No tamper-evidence or tamper-detection is claimed at Level 1.

For the **Level 3** future configuration (SP-CRT-002), the physical boundary is the hardware-bound key store: an HSM (PKCS#11) or platform secure element (Apple SEP, Caliptra) holding the operator-CA private key. Sphragis attests to it but never holds the private material.

### 3. Excluded components

- Hardware vendor firmware (Apple SEP firmware, M4 boot ROM, dwc3 USB controller firmware) — Sphragis attests TO these but is not authoritative for them.
- Third-party Rust crates not in the table above — these are used by the wider OS but are NOT part of the cryptographic module.

### 4. Modes of operation

Two modes:

| Mode | Build | Restrictions |
|---|---|---|
| **Approved (gov-strict)** | `--features gov-strict` | Algorithms restricted to the CNSA 2.0 allowlist (see §7.3). Fail-closed RNG. `policy::ensure_permitted` rejects non-approved algorithms at every entry point. |
| **Non-approved (community)** | default | All algorithms in the table above usable. RNG is fail-soft on RNDR absent. For development, research, and non-gov deployment. |

The mode is selected at build time via the `gov-strict` Cargo feature. Runtime mode-switching is NOT supported (avoids cross-mode contamination of SSPs).

---

## §7.2 — Cryptographic Module Interfaces

The module exposes four logical interface types per FIPS 140-3 §7.2:

| Interface | Direction | Realization in Sphragis |
|---|---|---|
| **Data Input** | In | Function arguments to public API (plaintext, ciphertext, keys, nonces, messages, signatures, etc.) |
| **Data Output** | Out | Function return values (ciphertext, plaintext, MAC, signature, KEM shared secret) |
| **Control Input** | In | Function calls + the `policy::Algo` enum that selects algorithm + the `gov-strict` build flag |
| **Status Output** | Out | `Result<T, &'static str>` error returns + `[crypto] …` UART log lines + audit ring entries |

Interface separation is enforced by Rust's type system. Data and Control inputs cannot cross because every public function has a typed signature. Status Output to UART is logically isolated from Data Output (UART goes to operator console, Data Output goes to in-process buffers).

### Public API surface (gov-strict)

| Function (qualified path) | Service | Approved? |
|---|---|---|
| `crypto::gcm_verified::encrypt(key256, nonce, aad, pt)` | AES-256-GCM encrypt | ✅ |
| `crypto::gcm_verified::decrypt(key256, nonce, aad, ct)` | AES-256-GCM decrypt | ✅ |
| `crypto::aes_xts::encrypt_sector` | AES-256-XTS encrypt | ✅ |
| `crypto::aes_xts::decrypt_sector` | AES-256-XTS decrypt | ✅ |
| `crypto::sha384::hash` | SHA-384 | ✅ |
| `crypto::sha384::hmac` | HMAC-SHA-384 | ✅ |
| `crypto::sha384::hkdf_extract` / `hkdf_expand` / `hkdf_expand_label` | HKDF-SHA-384 | ✅ |
| `crypto::sha512::hash` / `hmac` / `hkdf_*` | SHA-512 + HMAC + HKDF | ✅ |
| `crypto::pq_cnsa::Kem1024Key::generate` | ML-KEM-1024 keygen | ✅ |
| `crypto::pq_cnsa::encapsulate_1024` / `decapsulate_1024` | ML-KEM-1024 encap/decap | ✅ |
| `crypto::pq_cnsa::Dsa87Key::generate` / `sign` | ML-DSA-87 keygen + sign | ✅ |
| `crypto::pq_cnsa::verify_mldsa87` | ML-DSA-87 verify | ✅ |
| `crypto::lms::keygen_default` / `sign_default` / `verify_default` | LMS keygen/sign/verify | ✅ |
| `crypto::rng::fill_bytes` | DRBG output (fail-soft mode) | ⚠️ approved only when RNDR present |
| `crypto::rng::fill_bytes_strict` | DRBG output (fail-closed) | ✅ |
| `crypto::policy::ensure_permitted` | Policy gate | ✅ (always) |

Non-approved surfaces (rejected by `policy::ensure_permitted` under gov-strict):

| Function | Reason |
|---|---|
| `crypto::sha256::*` for signature use | SHA-256 deprecated for new signing per CNSA 2.0 |
| Any AES-128 path | Replaced by AES-256 |
| `crypto::chacha20poly1305::*` | Outside CNSA-allowed AEAD set |
| `crypto::pq_hybrid::*` | Category-3 (ML-KEM-768); TLS-interop only |
| `crypto::pq_hybrid_sig::*` | Category-3 (ML-DSA-65); TLS-interop only |
| `crypto::sig::*` (Ed25519) | Pre-PQ signing; non-approved in gov-strict |
| `crypto::blake2s::*`, `crypto::blake3::*` | Outside FIPS-approved hash set |

These remain compilable for the community build so existing TLS / WireGuard / BatFS code keeps working when the gov-strict flag is off. SP-B1.6.1 follow-up sweeps every call site to route through `policy::ensure_permitted`.

---

## §7.3 — Roles, Services, and Authentication

### Roles

At **Level 1**, FIPS 140-3 mandates two distinct logical roles. Sphragis recognizes:

| Role | Purpose | Authentication |
|---|---|---|
| **Crypto Officer (CO)** | Key generation, key destruction, policy configuration, audit-key rotation | Lock-screen passphrase + M-of-2 Ed25519 quorum (Two-Person Integrity, audit `TPI-*` series) |
| **User** | Encrypt/decrypt, sign/verify, hash, MAC, HKDF, KEM encap/decap | Cave-level authentication (passphrase + label-policy check) |

At Level 1 only role-based authentication is required (FIPS 140-3 §7.3.4 Level 1: "no authentication required"); Sphragis already provides role-based via the lock-screen + TPI flow.

For **Level 3**, identity-based authentication is required. Sphragis's per-cave identity (REQ-ATT-005) + operator-CA-attested user accounts (REQ-UX-004) will provide this.

### Services

The module provides the following services per FIPS 140-3 §7.3.3:

| Service | Inputs | Outputs | Role required |
|---|---|---|---|
| Show Status | (none) | Module version, approved mode flag, RNG status, KAT state | User or CO |
| Run Self-Tests on Demand | (none) | KAT pass/fail per algorithm | User or CO |
| Generate Symmetric Key | Algorithm selector | Key bytes | User |
| Encrypt | Key, plaintext, mode params | Ciphertext | User |
| Decrypt | Key, ciphertext, mode params | Plaintext or error | User |
| Hash | Algorithm selector, message | Digest | User |
| MAC | Key, message, algorithm selector | MAC | User |
| Generate Asymmetric Keypair | Algorithm selector | Public key, private key handle | CO (for operator CA), User (for ephemeral keys) |
| Sign | Private key handle, message | Signature | User (or CO for operator CA — Level 3 future) |
| Verify | Public key, message, signature | Pass/fail | User |
| KEM Encapsulate | Recipient public key | Shared secret, ciphertext | User |
| KEM Decapsulate | Private key handle, ciphertext | Shared secret | User |
| Zeroize | Key handle | Confirmation; key material destroyed in volatile + persistent memory | CO |

---

## §7.4 — Software / Firmware Security

- **Integrity**: kernel image is signed with LMS at release time (SP-B4); bootloader verifies signature before jump-to-Rust.
- **Software supply chain**: every dependency is permissively licensed (Apache-2.0 / MIT / BSD / ISC / Zlib / Unicode / CC0), enforced by `cargo-deny check` in CI. GPL/AGPL/LGPL/SSPL/BUSL are denied. No-yanked, no-unmaintained policy on `cargo audit`.
- **Code-signing root**: operator-CA-attested. The operator CA holds the LMS signing key; Sphragis attests TO it but never holds it.
- **Approved mode build is reproducible**: `scripts/check_reproducible_build.sh` produces bit-identical artifacts on independent machines (SP-B3 verified).

---

## §7.5 — Operating Environment

The module operates in a **modifiable operating environment** (FIPS 140-3 §7.5.3). The host OS is Sphragis itself — a bare-metal microkernel with no underlying operating system. The module runs at EL1 (kernel privilege) on aarch64.

Process isolation between cryptographic operations from different callers is enforced by the Sphragis cave model (per-cave page tables with per-cave ASIDs; see `DESIGN_CAVE_ISOLATION.md` and audit ISO-002 closure).

---

## §7.6 — Physical Security

At Level 1: **not applicable** (software-only module on production-grade components per FIPS 140-3 §7.6.2).

At Level 3 (future, SP-CRT-002): tamper-evidence + tamper-detection circuitry on the HSM that holds the operator-CA private key. EFP/EFT (Environmental Failure Protection/Testing) per §7.6.4 Level 3.

---

## §7.7 — Non-Invasive Security

Sphragis mitigates the following non-invasive attack classes:

- **Timing side channels**: secret-dependent code paths follow the constant-time discipline (`crypto::hotp::ct_eq`, signature-verify branchless compares, AES T-table avoidance via `aes-gcm` constant-time impl, ML-KEM constant-time decapsulate per `ml-kem` crate). Audit Crypto-F1/F2 closed; SP-B2 adds CI benchmarks asserting bounded variance.
- **Cache side channels**: AES uses RustCrypto's constant-time-by-construction impl (no T-tables). ML-KEM/ML-DSA decapsulate is constant-time per the upstream crate's design.
- **Microarchitectural side channels**: Spectre-v2 mitigations via ARMv8.5 FEAT_SB (`sb` instruction) at every cross-domain transition (EL1↔EL0, TTBR0 cave swap, scheduler task switch). See `src/kernel/arch/mod.rs`.

---

## §7.8 — Sensitive Security Parameter (SSP) Management

### Critical Security Parameters (CSPs) — secret material

| CSP | Generation | Storage | Zeroization |
|---|---|---|---|
| Audit-chain HMAC key | RNDR seed at boot (`security::audit_chain::init_audit_key`) | Kernel-private static memory | `panic_wipe` → volatile zero + POISON flag |
| BatFS master key | Argon2id over operator passphrase | RAM only (never persisted unencrypted); wrapped under user passphrase + per-cave label | Zeroized on cave teardown + on unmount |
| Per-cave file keys | HKDF-SHA-384 derived from master key + per-file nonce | RAM only | Zeroized on cave teardown |
| ML-KEM-1024 decapsulation keys | `Kem1024Key::generate` via RNG | Cave-private heap; explicit `Drop` zeroize | `Drop` impl |
| ML-DSA-87 signing keys | `Dsa87Key::generate` via RNG | Cave-private heap | `Drop` impl |
| LMS signing keys | `lms::keygen_default` via RNG | Caller-managed bytes; cave-private | Caller zeroize on retire |
| TLS session keys | HKDF-Expand-Label derived from (EC)DHE + handshake transcript | Per-session heap | Zeroized on connection close |
| Stack-canary value | RNDR at boot | Per-task TLS slot | Reseeded on each task spawn |
| DRBG state (`STATE_LO`/`HI`/`CTR`) | RNDR seed at boot | Kernel-private static | `panic_wipe` |
| TPI operator Ed25519 private keys | Operator-supplied (off-platform generation expected) | Caller-supplied bytes; never persisted | Zeroized after each quorum-consume |

### Public Security Parameters (PSPs) — public material

| PSP | Storage |
|---|---|
| X.509 trust anchor certificates (6) | Hard-coded in `src/net/x509.rs` |
| TPI operator Ed25519 public keys | Static configuration |
| Per-host SPKI pins (cert_pin) | Heap, cave-policy-managed |
| Operator-CA public keys | Hard-coded or operator-installed; never private material |

### Zeroization policy

All CSPs that live in heap-allocated `Vec<u8>` are wrapped in types with `ZeroizeOnDrop` (via the `zeroize` crate, already a transitive dep). All CSPs in static memory have an explicit `zeroize` path callable by `panic_wipe` or by the cave-teardown hook.

The `panic_wipe` function in `src/crypto/rng.rs` zeros DRBG state + sets the POISONED flag so any post-panic `fill_bytes` halts the kernel rather than emit weak-entropy output.

---

## §7.9 — Self-Tests

Sphragis runs the following at boot, before any user-space code executes (`crypto::run_self_tests` in `src/crypto/mod.rs`):

| Test | Vector source | Status |
|---|---|---|
| SHA-256 KAT | RFC 6234 §8.5 "abc" | ✅ Wired |
| SHA-384 KAT | FIPS 180-4 §F.4 "abc" | ✅ Wired (SP-B1.7) |
| SHA-512 KAT | FIPS 180-4 §F.3 "abc" | ✅ Wired (SP-B1.5) |
| HMAC-SHA-384 KAT | RFC 4231 §4.2 TC1 | ✅ Wired (SP-B1.7) |
| HMAC-SHA-512 determinism | Self-consistency | ✅ Wired (SP-B1.5) |
| AES-128/256-GCM KAT | NIST SP 800-38D via `gcm_verified::selftest` | ✅ Wired |
| ChaCha20-Poly1305 round-trip + tamper | RFC 8439 + fixed vector | ✅ Wired |
| ML-KEM-1024 round-trip | Keygen → encap → decap → SS equality | ✅ Wired (SP-B1.1) |
| ML-DSA-87 sign-verify + tamper | Keygen → sign → verify → bit-flip-reject | ✅ Wired (SP-B1.2) |
| RNG strict-mode probe | RNDR consistency check | ✅ Wired (SP-B1.8) |
| LMS keygen/sign/verify + tamper | Self-test of round-trip + tamper | ⚠️ Shell command only (`lms-kat`); too slow for boot smoke under QEMU |
| XMSS | RFC 8391 / SP 800-208 | ❌ Deferred (SP-B1.4 blocked on upstream xmss crate not no_std-clean) |

Any failure of a wired KAT halts boot via `panic!` (`crypto self-test failed: …` UART message + kernel halt). No silent fallthrough to broken crypto possible — audit CRYPTO-F7 fail-closed pattern.

### Conditional self-tests

- **Pairwise consistency test** for asymmetric keygen: implicit in the round-trip KATs (every keygen is exercised at boot via a sign-then-verify).
- **Continuous DRBG entropy-source test**: `rng::fill_bytes_strict` rejects RNDR stalls (`rndr_stall_count` exposed for trending).
- **Software integrity test**: kernel image is LMS-signed; bootloader verifies (SP-B4 wiring planned for M4 m1n1 chain).

---

## §7.10 — Life-Cycle Assurance

- **Configuration management**: Git, `main` branch protected by feature-branch + `--no-ff` merge convention. Every commit DCO-signed (`git commit -s` per `CONTRIBUTING.md`).
- **Delivery**: GitHub releases signed via sigstore cosign + in-toto attestation chain (SP-B4 wiring).
- **Operator guidance**: `docs/superpowers/plans/2026-05-16-sp-a3-incorporation.md` (operator on-ramp), `DESIGN_CRYPTO.md` (architecture), `ANTI_FEATURES.md` (non-goals), this document.
- **Vendor testing**: `scripts/qemu_*.py` (~80 QMP-driven selftest scripts), `cargo build` / `cargo clippy` / `cargo deny check` / `cargo audit` in CI on every PR.

---

## §7.11 — Mitigation of Other Attacks

Sphragis mitigates the following attack classes by design:

- **Memory safety**: Rust ownership system prevents use-after-free, double-free, buffer overflow at compile time. `panic = "abort"` halts on detected runtime overflow rather than silent corruption.
- **Stack-overflow buffer overrun**: stack canaries seeded from RNDR at boot (audit MEM-H2 closure).
- **Return-oriented programming (ROP/JOP)**: ARMv8.5 BTI (Branch Target Identification) enforced via SCTLR_EL1.BT0/BT1 (audit week-9 elite-tier).
- **EL0 ↔ EL1 boundary attacks**: PAN (Privileged Access Never) auto-enabled via SCTLR_EL1.SPAN clear (audit week-3-4).
- **Per-cave address-space confusion**: per-cave ASIDs in TTBR0_EL1 (audit week-11 elite-tier).
- **TLB-based cross-cave leakage**: defense-in-depth TLBI flush on cave switch (in addition to ASID tagging).
- **Spectre v2 / Branchpredictor attacks**: ARMv8.5 FEAT_SB barriers at every cross-domain transition.
- **Microarchitectural side channels via shared cache**: cave isolation places cross-cave evictions outside the attacker's privilege boundary.
- **Cold-boot DRBG state recovery**: `panic_wipe` zeroes DRBG state + POISONED-flag prevents post-panic key derivation.
- **Audit-log tampering**: HMAC-SHA-256 chain (upgrading to HMAC-SHA-384 in SP-C4.1) detects modification; offline verifier (SP-C4.4) confirms continuity.

---

## Module version + change history

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-05-16 | Initial document. Boundary defined for SP-B5 CMVP engagement. Covers SP-B1.1 / B1.2 / B1.5 / B1.6 / B1.7 / B1.8 + the pre-existing crypto surface. |

## Open items for lab engagement (SP-B5)

1. **Algorithm validation (CAVP)**: submit individual algorithm impls for CAVP testing — AES-256, SHA-384/512, HMAC-SHA-384, ML-KEM-1024, ML-DSA-87. CAVP queue ~3-6 months.
2. **Source-code review prep**: the lab will request annotated source for every CSP touchpoint. The CSP table in §7.8 is the index they'll use.
3. **Threat-model document** (`docs/THREAT_MODEL.md`, REQ-DOC-002) — adjacent SP, lab will request as part of the ST.
4. **Operator guidance document** (REQ-DOC-001, SP-F3) — distinct from this boundary doc; describes how a sysadmin deploys and operates the module.
5. **Roles + service mapping ↔ AppSec audit**: confirm with the cave-policy audit that no User-role surface invokes a CO-only service path.

## References

- FIPS PUB 140-3 (NIST, 2019-03-22)
- ISO/IEC 19790:2012 (Cryptographic Module Security Requirements)
- ISO/IEC 24759:2017 (Test requirements)
- NIST SP 800-140 + 800-140A–F (Implementation guidance)
- NIST SP 800-90A (DRBG)
- NIST SP 800-38D (GCM)
- NIST SP 800-208 (Stateful hash-based signatures: LMS, XMSS)
- FIPS 203 (ML-KEM)
- FIPS 204 (ML-DSA)
- NSA CNSA 2.0 Algorithm Suite (May 2025)
