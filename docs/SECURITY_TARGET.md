# Sphragis Security Target

**Document version:** 1.0 (SP-DOC-005, 2026-05-16)
**Format:** Common Criteria-style ST per CC:2022 Rev 1 Part 1 §B
**Audience:** NIAP CCTLs, prospective NIAP-PP claim authors, AOs scoping evaluation strategy
**Companion docs:** `docs/FIPS_140_3_MODULE_BOUNDARY.md` (crypto-module boundary — narrower scope than this ST), `docs/THREAT_MODEL.md` (consolidated threat model — feeds §3 of this ST), `VERIFICATION_BOUNDARY.md` (verified-subsystem scope), `docs/NIST_800_53_INHERITANCE.md` (control inheritance).

---

## 1. ST Introduction

### 1.1 ST Reference

| Field | Value |
|---|---|
| ST Title | Sphragis Security Target |
| ST Version | 1.0 |
| Date | 2026-05-16 |
| Author | Kaden Lee and contributors |

### 1.2 TOE Reference

| Field | Value |
|---|---|
| TOE Name | Sphragis Operating System |
| TOE Version | 0.1.0 (autonomous-run-end snapshot, 2026-05-16) |
| Developer | Kaden Lee and contributors |
| Type | Operating System (with optional gov-strict cryptographic policy gate) |

### 1.3 TOE Overview

Sphragis is a security-first, bare-metal Rust microkernel for Apple Silicon (M4 today; x86_64 planned via SP-HW-002). It is designed for government, defense-contractor, and high-assurance commercial use. The TOE comprises the kernel + first-party drivers + the cave isolation runtime + the cryptographic module (per FIPS 140-3 module boundary) + the attestation primitive + the audit subsystem.

**TOE type:** General-purpose operating system (NIAP-relevant PPs: GPOSPP v4.4; MDF PP v3.3 if the fixed-app deployment posture fits the customer use case).

**Major security features:**
- Capability-based cave isolation (per-cave page tables + per-cave ASIDs + per-cave AF_UNIX namespace + per-cave taint bitmap)
- CNSA 2.0 post-quantum cryptography (ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384/512 + LMS)
- Kernel-mediated attestation primitive (every cave is an attestable identity)
- Tamper-evident HMAC-chained audit ring (planned upgrade SHA-256 → SHA-384 per SP-C4.1)
- Two-person-integrity quorum on high-consequence privileged operations
- SealFS encrypted filesystem with AES-256-GCM-SIV at-rest + per-cave + per-file keys
- Mandatory access control: Bell-LaPadula sensitivity + Biba integrity + CIPSO/CALIPSO network labels + SELinux-style type-enforcement
- Hardware-rooted exploit mitigations: PAN, BTI, per-cave ASIDs, FEAT_SB Spectre barriers, stack canaries from RNDR

### 1.4 TOE Description

#### 1.4.1 Logical scope

The TOE includes:

| Subsystem | Source path | Role |
|---|---|---|
| Kernel core | `src/main.rs`, `src/kernel/` | Boot, scheduler, memory, syscall dispatch |
| Architecture support | `src/kernel/arch/mod.rs`, `src/arch/aarch64/` | Exception handling, ASID management |
| Drivers (first-party) | `src/drivers/` | M4 Apple Silicon + virtio + QEMU support |
| Cave isolation runtime | `src/caves/cave.rs`, `src/caves/linux/` | Per-cave page tables, syscall filter, Linux ABI shim |
| Cryptographic module | `src/crypto/` | All CNSA-2.0 + legacy crypto primitives + policy gate |
| SealFS encrypted filesystem | `src/fs/` | At-rest AEAD encryption with per-cave + per-file keys |
| Network stack | `src/net/` | TLS 1.3 + WireGuard + X.509 + DNS + NAT + firewall |
| Attestation primitive | `src/security/attest.rs` | Kernel-mediated quote production |
| Audit subsystem | `src/security/audit.rs`, `audit_chain.rs` | 24-category event ring with HMAC chain |
| User interface (community build) | `src/ui/` | Lock screen + 7 in-OS apps (gov-strict build same) |

The TOE excludes:
- Bootloader (m1n1 on M4; future GRUB/shim on x86_64) — verified separately
- Hardware vendor firmware (SEP, Caliptra, Boot ROM) — attested TO, not signed BY
- Third-party Rust crates not in `src/` — depended upon, subject to supply-chain controls in `deny.toml`
- The AGENT app (removed entirely SP-A2) — not in any build

#### 1.4.2 Physical scope

The TOE is software. Distributed via:
- GitHub Releases (signed via sigstore + Rekor; SP-BLD-005 design)
- LMS-signed kernel image (SP-BLD-008 design; .IMPL pending)
- SBOM accompanies each release (`scripts/gen_sbom.py`)

#### 1.4.3 Configuration scope

Two build profiles:
- **Community build** (`cargo build --release`): all features available; policy gates permissive; suitable for development, research, non-gov deployment.
- **`sphragis-gov` build** (`cargo build --release --features gov-strict`): CNSA-2.0-only policy enforced at the policy gate (SP-B1.6); RNG fail-closed at boot if RNDR absent (SP-B1.8 via require_hw_rng_or_halt wired in SP-B1.6).

This ST primarily addresses the `sphragis-gov` build profile.

---

## 2. Conformance Claims

### 2.1 CC Conformance Claims

| Claim | Status |
|---|---|
| CC:2022 Rev 1 Part 2 conformance | Extended (Sphragis-specific SFRs added for cave isolation, attestation, audit-chain HMAC) |
| CC:2022 Rev 1 Part 3 conformance | EAL N/A under NIAP's current "no new GPOS CC evaluations" stance; Protection Profile conformance per §2.3 |

### 2.2 Protection Profile Claims

Sphragis claims NO Protection Profile conformance today. Once a NIAP CCTL engagement begins (SP-CRT-003 pending), the most plausible PPs to claim against:

| Candidate PP | Status |
|---|---|
| NIAP GPOSPP v4.4 (General Purpose Operating Systems) | Probable fit for the broad capability set |
| NIAP MDF PP v3.3 (Mobile Device Fundamentals) | Alternative if the fixed-app posture aligns |
| NIAP GPCP v1.0 (General-Purpose Computing Platforms) | Platform-layer claim alongside one of the above |

Final PP selection depends on customer demand + a feasibility analysis by the CCTL.

### 2.3 Package Claims

None. Functional and assurance requirements are spelled out directly per §6.

---

## 3. Security Problem Definition

Mirrors `docs/THREAT_MODEL.md` in compressed form for CC consumption.

### 3.1 Assumptions (about the operational environment)

| ID | Assumption |
|---|---|
| A.PHYSICAL | The device's powered-state hardware (RAM, in-use peripherals) is protected by the operator's physical-security policy. CC Level 1 evaluation does not claim physical-attacker resistance. |
| A.PEER_AUTH | TLS peers, WireGuard peers, and operator-CA endorsement-chain endpoints have their own authentication infrastructure that the operator manages. |
| A.OPERATOR_TRAINING | Operators are trained on TPI quorum operation, lock-screen passphrase strength, and the differences between community and gov-strict builds. |
| A.BOOT_CHAIN | The bootloader (m1n1 / GRUB / shim per platform) is provisioned and signed per `DESIGN_LMS_KERNEL_SIGNING.md`. Until SP-BLD-008.IMPL lands, the boot chain is trusted-by-policy. |
| A.HSM_OPERATOR | The operator-CA HSM is physically secured per FIPS 140-3 Level 3 (or higher). Sphragis attests TO it, doesn't manage it. |

### 3.2 Threats (mapped to adversaries from `docs/THREAT_MODEL.md` §2)

| ID | Threat | Adversary |
|---|---|---|
| T.NETWORK_INJECTION | Adversary modifies in-flight TLS / WireGuard traffic | A1 |
| T.CAVE_ESCAPE | Adversary in one cave reads/writes another cave's state | A2 |
| T.OFFLINE_DATA_DISCLOSURE | Adversary extracts data from powered-off storage media | A3 |
| T.SUPPLY_CHAIN | Adversary substitutes malicious code via build pipeline | A4 |
| T.SIDE_CHANNEL | Adversary extracts secrets via cache/timing/microarch | A5 |
| T.QUANTUM_HARVEST | Adversary records ciphertext today to decrypt with future CRQC | A7 |
| T.INSIDER_ABUSE | Privileged operator misuses TPI or operator-CA capability | A8 |

### 3.3 Organisational Security Policies (OSPs)

| ID | OSP |
|---|---|
| P.CNSA_2_0 | Cryptographic primitives in the gov-strict build must comply with NSA CNSA 2.0 algorithm suite. |
| P.AUDIT_TAMPER_EVIDENT | All security-relevant events must be logged in a tamper-evident manner. |
| P.LEAST_PRIVILEGE | No ambient authority; high-consequence operations require multi-party authorization. |
| P.ATTEST_TO_ROOT | Every cave's identity must be attestable to an operator-CA root. |
| P.NO_AGENT_AI | No AI/LLM/ML in the kernel critical path. |
| P.PERMISSIVE_DEPS | No GPL/AGPL/copyleft dependencies. |

---

## 4. Security Objectives

### 4.1 Security objectives for the TOE

| ID | Objective | Addresses |
|---|---|---|
| O.CAVE_ISOLATION | Each cave's state is unreachable to other caves except through documented IPC primitives gated by cave-policy | T.CAVE_ESCAPE |
| O.NETWORK_INTEGRITY | All network communications use authenticated encryption with PQ-hybrid key exchange | T.NETWORK_INJECTION, T.QUANTUM_HARVEST |
| O.AT_REST_ENCRYPTION | All persistent data is encrypted with misuse-resistant AEAD | T.OFFLINE_DATA_DISCLOSURE |
| O.AUDIT_TAMPER_EVIDENT | Audit ring uses HMAC chain detectable upon tampering | P.AUDIT_TAMPER_EVIDENT |
| O.ATTESTABLE | Every cave is attestable to a kernel-mediated identity registry | P.ATTEST_TO_ROOT |
| O.QUORUM_PRIVILEGE | High-consequence operations require TPI quorum, captured in audit log | P.LEAST_PRIVILEGE, T.INSIDER_ABUSE |
| O.CONSTANT_TIME | Cryptographic operations are constant-time on key-dependent code paths | T.SIDE_CHANNEL |
| O.CNSA_ENFORCEMENT | Gov-strict build rejects non-CNSA-2.0 algorithms at the policy gate | P.CNSA_2_0 |
| O.SUPPLY_CHAIN | Build pipeline produces SLSA-L4 provenance; dependencies license-checked + advisory-scanned in CI | T.SUPPLY_CHAIN, P.PERMISSIVE_DEPS |
| O.NO_AI_IN_KERNEL | AGENT app removed entirely; no ML/LLM in kernel critical path | P.NO_AGENT_AI |

### 4.2 Security objectives for the operational environment

| ID | Objective |
|---|---|
| OE.PHYSICAL | Operator provides physical security per their deployment-tier policy |
| OE.HSM | Operator-CA HSM operates per FIPS 140-3 L3 (or higher) policy |
| OE.OPERATOR | Operators are trained per A.OPERATOR_TRAINING |
| OE.PEER | Peer-authentication infrastructure managed per A.PEER_AUTH |
| OE.BOOT_CHAIN | Bootloader provisioning + signing per A.BOOT_CHAIN |

---

## 5. Extended Components Definition

Components beyond CC Part 2 catalogue that Sphragis introduces:

### FCS_QKD.1 (Defined extended) — Post-Quantum KEM

**Definition:** TOE shall implement ML-KEM-1024 per FIPS 203 for key encapsulation.
**Justification:** CC Part 2 catalogue's FCS_COP family doesn't pre-define PQ-KEM operations.
**Implementation:** `src/crypto/pq_cnsa.rs` — closed by SP-B1.1.

### FCS_PQS.1 (Defined extended) — Post-Quantum Signature

**Definition:** TOE shall implement ML-DSA-87 per FIPS 204 for digital signatures.
**Justification:** Same as FCS_QKD.1.
**Implementation:** `src/crypto/pq_cnsa.rs` — closed by SP-B1.2.

### FCS_SHB.1 (Defined extended) — Stateful Hash-Based Signature

**Definition:** TOE shall implement LMS per NIST SP 800-208 for software/firmware signing.
**Justification:** CC Part 2 catalogue doesn't pre-define stateful-hash-based signatures.
**Implementation:** `src/crypto/lms.rs` — closed by SP-B1.3.

### FDP_CAV.1 (Defined extended) — Cave Isolation

**Definition:** TOE shall enforce per-cave address-space isolation via per-cave page tables AND per-cave ASIDs.
**Justification:** CC FDP doesn't pre-define a "cave" abstraction that combines MMU + ASID + namespace primitives.
**Implementation:** `src/caves/cave.rs` + `src/caves/linux/mmu.rs` — closed by audit-week-11 elite-tier.

### FIA_ATTEST.1 (Defined extended) — Kernel-Mediated Attestation

**Definition:** TOE shall produce signed Quotes binding (kernel-measurement, cave-identity, claims, nonce) for external verification.
**Justification:** CC FIA covers user authentication, not kernel-mediated cave attestation.
**Implementation:** `src/security/attest.rs` — closed by SP-C1.1/1.2/1.3.

---

## 6. Security Functional Requirements

Selected SFRs from CC Part 2 catalogue plus the extended components above. Iterated/refined per CC convention.

### 6.1 Cryptographic Support (FCS)

| SFR | Sphragis fulfilment |
|---|---|
| FCS_CKM.1 (Key Generation) | ML-KEM-1024 keygen (`Kem1024Key::generate`); ML-DSA-87 keygen (`Dsa87Key::generate`); LMS keygen (`lms::keygen_default`); AES-256 keys via DRBG |
| FCS_CKM.2 (Key Distribution) | ML-KEM-1024 encapsulation; X25519MLKEM768 hybrid for TLS interop |
| FCS_CKM.4 (Key Destruction) | `zeroize` crate ZeroizeOnDrop on per-cave heap CSPs; explicit `panic_wipe` on DRBG state |
| FCS_COP.1/AES (AES Operation) | AES-256-GCM, AES-256-GCM-SIV, AES-256-XTS, AES-256-CTR |
| FCS_COP.1/HASH (Hashing) | SHA-256, SHA-384, SHA-512, SHA-3 family |
| FCS_COP.1/HMAC (Keyed Hash) | HMAC-SHA-256/384/512 |
| FCS_COP.1/SIGN (Digital Signature) | ML-DSA-87, LMS, Ed25519 (community-only); ECDSA + RSA (community-only) |
| FCS_QKD.1 (PQ KEM, EXTENDED) | ML-KEM-1024 per `src/crypto/pq_cnsa.rs` |
| FCS_PQS.1 (PQ Signature, EXTENDED) | ML-DSA-87 per `src/crypto/pq_cnsa.rs` |
| FCS_SHB.1 (Stateful Hash-Based Sig, EXTENDED) | LMS per `src/crypto/lms.rs` |
| FCS_RBG_EXT.1 (Random Bit Generation) | SHA-256-chained DRBG seeded from RNDR; fail-closed strict variant in gov-strict build |
| FCS_STO_EXT.1 (Stored Sensitive Data) | SealFS AES-256-GCM-SIV with per-cave + per-file keys |

### 6.2 User Data Protection (FDP)

| SFR | Sphragis fulfilment |
|---|---|
| FDP_ACC.1 (Access Control Scope) | Cave-policy gate enforces every cross-cave access |
| FDP_ACF.1 (Access Control Rules) | Per-cave page tables + cave-policy table + type-enforcement deny matrix |
| FDP_IFC.1 (Information Flow Control Scope) | Bell-LaPadula sensitivity + Biba integrity labels |
| FDP_IFF.1 (Information Flow Control Rules) | CIPSO/CALIPSO IPv4/IPv6 packet labels; SealFS AAD-bound classification |
| FDP_CAV.1 (Cave Isolation, EXTENDED) | Per-cave L1 + per-cave ASIDs + per-cave AF_UNIX namespace |

### 6.3 Identification and Authentication (FIA)

| SFR | Sphragis fulfilment |
|---|---|
| FIA_AFL.1 (Authentication Failures) | 5 attempts → lockout (`src/security/auth.rs`) |
| FIA_UAU.1 (Authentication) | Argon2id-protected passphrase |
| FIA_UAU.5 (Multiple Mechanisms) | Passphrase + TPI quorum for high-consequence ops |
| FIA_X509_EXT.1 (X.509 Validation) | `src/net/x509.rs` against 6 trust anchors with SPKI pinning |
| FIA_X509_EXT.3 (X.509 Request Generation) | TLS ClientHello + certificate request paths |
| FIA_ATTEST.1 (Attestation, EXTENDED) | `src/security/attest.rs::quote` |

### 6.4 Security Management (FMT)

| SFR | Sphragis fulfilment |
|---|---|
| FMT_MOF_EXT.1 (Management Functions Privilege) | TPI quorum required to alter security-relevant configuration |
| FMT_SMF_EXT.1 (Enumerated Management Functions) | Cave-create, cave-destroy, audit-flush, key-rotation, master-key-wipe (TPI-gated); passphrase change |

### 6.5 Protection of the TSF (FPT)

| SFR | Sphragis fulfilment |
|---|---|
| FPT_ACF_EXT.1 (TSF Data Protection) | TSF data in kernel-mode memory (unreachable from EL0 per cave isolation) |
| FPT_ASLR_EXT.1 (Address Space Layout Randomisation) | Not implemented today (planned SP-MEM-001) |
| FPT_SBOP_EXT.1 (Stack Buffer Overflow Protection) | Stack canaries from RNDR (audit-MEM-H2 closure); BTI (audit-week-9) |
| FPT_SRP_EXT.1 (Software Restriction) | Cave-policy gate; gov-strict policy denies weak algorithms |
| FPT_TST_EXT.1 (Self-Test) | Boot KATs for every CNSA primitive; fail-closed on any failure (audit Crypto-F7) |
| FPT_TUD_EXT.1 (Trusted Update) | LMS-signed kernel image (SP-BLD-008 design landed; .IMPL pending) |
| FPT_W_X_EXT.1 (Write-XOR-Execute) | Linker enforces W^X boundary at __text_end + page-table mappings |

### 6.6 TOE Access (FTA)

| SFR | Sphragis fulfilment |
|---|---|
| FTA_SSL.4 (Session Termination) | `auth::lock` + cave-teardown paths |

### 6.7 Trusted Path / Channels (FTP)

| SFR | Sphragis fulfilment |
|---|---|
| FTP_ITC_EXT.1 (Trusted Inter-Component Channels) | TLS 1.3 + WireGuard + per-cave encrypted IPC (planned SP-ISO-003 extension) |
| FTP_TRP.1 (Trusted Remote Management Path) | WireGuard responder for operator remote-admin; TPI quorum for privileged ops over the channel |

### 6.8 Security Audit (FAU)

| SFR | Sphragis fulfilment |
|---|---|
| FAU_GEN.1 (Audit Data Generation) | 24 categories per `src/security/audit.rs::Category` (SP-AUD-003 added the 6 NIAP-mandated ones) |
| FAU_GEN.2 (User Identity Association) | Cave_id captured per record (audit-CAVE-M3) |
| FAU_SAR.1 (Audit Review) | `audit::recent` + `audit::recent_for_cave` (SP-ISO-009 cave-scoped); offline verifier (SP-AUD-004) |
| FAU_STG.1 (Audit Storage Protection) | HMAC-chained ring (audit-week-3-4); planned SP-AUD-002 WORM export |
| FAU_STG.2 (Guarantees of Audit Data Availability) | Single-writer ring; flush-to-SealFS for durability |
| FAU_STG.3 (Action in Case of Possible Audit Data Loss) | EVICTED counter + UART warning on first ring rollover |

---

## 7. Security Assurance Requirements

Sphragis claims NO specific EAL today (NIAP's "no new GPOS CC evaluations" stance). For a future PP-conformance claim, the relevant SARs:

| SAR family | Sphragis posture |
|---|---|
| ASE (Security Target Evaluation) | This document; SP-DOC-005 |
| ADV (Development) | Architecture documented in `DESIGN_*.md`; functional spec in source + per-subsystem design docs |
| AGD (Guidance Documents) | Operator runbook SP-DOC-001 (pending); deployment notes per `docs/HARDWARE_COMPATIBILITY.md` |
| ALC (Life-Cycle Support) | Git + DCO + branch-protection (planned SP-BLD-001.IMPL.D); reproducible builds + sigstore + Rekor |
| ATE (Tests) | ~80 QMP-driven self-test scripts; boot-time KATs; offline audit verifier |
| AVA (Vulnerability Assessment) | 14-week comprehensive security audit (2026-05-15 baseline + remediation); cargo-audit + cargo-deny CI gates |

---

## 8. TOE Summary Specification

Maps each SFR to the source-code implementing it. Maintained inline in §6 (Sphragis-fulfilment column).

---

## 9. Document version + change log

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-05-16 | Initial ST. Covers TOE at autonomous-run-end snapshot. Lock document for CCTL engagement (when SP-CRT-003 begins). |

## 10. References

- CC:2022 Rev 1: https://www.commoncriteriaportal.org/cc/
- NIAP Protection Profiles: https://www.niap-ccevs.org/protectionprofiles
- NIAP GPOSPP v4.4 source: https://github.com/commoncriteria/operatingsystem
- FIPS 203 (ML-KEM): https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf
- FIPS 204 (ML-DSA): https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf
- NIST SP 800-208 (LMS / XMSS): https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-208.pdf
- NSA CNSA 2.0: https://media.defense.gov/2025/May/30/2003728741/-1/-1/0/CSA_CNSA_2.0_ALGORITHMS.PDF
