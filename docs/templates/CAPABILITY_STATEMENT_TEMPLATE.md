# Sphragis Capability Statement — TEMPLATE

**SP-DOC-004 TEMPLATE (2026-05-16).** Replace every `{PLACEHOLDER}` with the founder-supplied value. Render to PDF (LaTeX or Pages) for distribution. The template is structured per the gov-procurement-standard 8-12 page capability-statement format used for GSA MAS offers + ACT 3 task-order responses + DARPA Forecast-to-Industry meeting handoffs.

---

## Cover page

**{COMPANY_LEGAL_NAME}**

Sphragis Operating System — Capability Statement

Version {VERSION} / {DATE}

| Contact | Detail |
|---|---|
| Primary POC | {FOUNDER_NAME}, {FOUNDER_TITLE} |
| Email | {FOUNDER_EMAIL} |
| Phone | {FOUNDER_PHONE} |
| Address | {COMPANY_ADDRESS} |
| Website | {COMPANY_DOMAIN} |
| DUNS / UEI | {UEI_NUMBER} |
| CAGE Code | {CAGE_CODE} |

---

## Executive summary

{COMPANY_LEGAL_NAME} is the developer of **Sphragis** — a sovereign-grade attested-cave operating system designed natively for the 2027-2030 government procurement world. Sphragis is a memory-safe Rust microkernel that ships CNSA 2.0-compliant post-quantum cryptography by default, formally-specified information-flow isolation between processes, hardware-rooted attestation as a first-class kernel primitive, and a CHERI-ready architecture for the capability-hardware era.

We are an unfunded {COMPANY_AGE_MONTHS}-month-old US-incorporated Delaware C-Corp. We have publicly-demonstrated boot on Apple M4 hardware via an independent reverse-engineering pipeline + 24 sub-projects executed across crypto / verification / attestation / build-chain / certification-engineering / productization tracks per a 36-month master plan.

---

## Core capabilities

### 1. Rust microkernel + formally-specified information-flow isolation

- Memory-safe by language (Rust 2024, `#![no_std]`, `panic = "abort"`)
- 70-80K LoC TCB (vs Linux ~30M); audit-traceable
- Cave-isolation primitive with per-cave page tables + per-cave ARMv8.5 ASIDs + per-cave AF_UNIX namespace + per-cave taint bitmap
- Verus formal-verification harness in place (`verification/`); SP-VER-002 spec for IPC non-interference proof landed (multi-week proof effort ahead — funding sought)

### 2. CNSA-2.0-native post-quantum cryptography (gov-strict build)

- ML-KEM-1024 (FIPS 203) — `src/crypto/pq_cnsa.rs`
- ML-DSA-87 (FIPS 204) — `src/crypto/pq_cnsa.rs`
- AES-256 (GCM, GCM-SIV, XTS, CTR)
- SHA-384 + SHA-512 + HMAC-SHA-384/512
- LMS (RFC 8554, NIST SP 800-208) — `src/crypto/lms.rs`
- Fail-closed RNG with ARMv8.5 FEAT_RNG (`require_hw_rng_or_halt`)
- Compile-time enforced gov-strict feature flag rejects AES-128, RSA, ECDSA, plain-ChaCha20, SHA-256-for-signing
- 8 boot-time KATs run fail-closed at every boot

### 3. Attestation as kernel primitive

- Per-cave attestable identity registry (`src/security/attest.rs`)
- Real kernel measurement at boot (SHA-384 over .text+.rodata via linker symbols)
- ML-DSA-87-signed Quote envelope with kernel-measurement + cave-identity + nonce + claims
- Hardware-rooted endorsement chain designed (`DESIGN_HSM_OPERATOR_CA.md`); SEP-rooted on M4 + TPM 2.0 on x86_64 + Caliptra-ready

### 4. SLSA L4 reproducible builds + sigstore-signed releases

- `cargo-deny` + `cargo-audit` CI gate enforces zero GPL/AGPL deps + zero known advisories
- Reproducible-build verification (`scripts/check_reproducible_build.sh`)
- SBOM per release
- SLSA L4 architecture + sigstore/Rekor release-signing designs landed (`DESIGN_SLSA_PROVENANCE.md`, `DESIGN_SIGSTORE_REKOR.md`)
- LMS-signed kernel boot verification designed (`DESIGN_LMS_KERNEL_SIGNING.md`)

### 5. CHERI-ready architecture

- Cave-to-CHERI-compartment mapping designed (`DESIGN_CHERI_MAPPING.md`)
- Targets ARM Morello (server/desktop, follows ARM Q1-Q3 2026 pure-cap roadmap) + CHERIoT-Ibex (embedded gov, lowRISC + SCI Semiconductor 2026 hardware)

---

## Technology readiness

| Subsystem | TRL today |
|---|---|
| Microkernel + cave isolation | TRL 6 (boots on real M4 + comprehensive QEMU CI) |
| CNSA 2.0 crypto + boot KATs | TRL 7 (production-grade primitives) |
| Attestation primitive | TRL 5 (API + measurement + per-cave registry; hardware-rooting designs landed) |
| SealFS encrypted filesystem | TRL 7 (AES-256-GCM-SIV; per-cave + per-file keys) |
| Audit chain | TRL 7 (HMAC-SHA-256 + tamper-evident; SHA-384 upgrade designed) |
| Window manager + multi-app UI | TRL 4 (design landed; implementation pending) |
| Multi-hardware (x86_64) | TRL 3 (design landed; implementation pending) |
| Formal verification | TRL 4 (harness scaffolded; IPC non-interference spec landed; proof pending — DARPA-funding sought) |
| FIPS 140-3 L1 cert | TRL 5 (module boundary doc landed; lab engagement pending) |

---

## Differentiators vs incumbents

| | INTEGRITY-178B | seL4 | RHEL | Sphragis |
|---|---|---|---|---|
| Memory-safe language | C (no) | C (no) | C (no) | **Rust (yes)** |
| Post-quantum crypto | No | No | Partial | **CNSA 2.0 native** |
| Attestation primitive | Bolt-on | Bolt-on | Bolt-on | **Kernel-mediated** |
| TCB size | ~10K LoC | ~10K LoC C | ~30M LoC | **~70K LoC** |
| Open source | No | Yes | Yes | **Yes (Apache-2.0)** |
| Formal verification | Cert-only | Full functional correctness | None | **Non-interference on critical subsystems** |
| CHERI compatibility | Static | Research | None | **Designed-in** |
| Reproducible builds | No | Partial | Partial | **SLSA L4 designed** |

---

## Past performance + reference engagements

- {LIST_OF_AWARDS_TO_DATE_OR_"None yet — pursuing SBIR Phase I"}
- {LIST_OF_TASK_ORDERS_OR_"None yet"}
- {LIST_OF_PARTNERSHIPS_OR_"None yet"}

For new entrants: this section may say "Currently pursuing first SBIR Phase I (DoD SBIR 26.1 / AFWERX / DARPA SBIR — submission window {DATE}). Founder previously {FOUNDER_PRIOR_RELEVANT_EXPERIENCE}."

---

## Certifications + standards posture

| Certification | Status |
|---|---|
| FIPS 140-3 L1 | Module boundary documented (`docs/FIPS_140_3_MODULE_BOUNDARY.md`); CMVP lab engagement pending |
| NIAP PP-conformant CC | Security Target documented (`docs/SECURITY_TARGET.md`); CCTL engagement pending |
| DoD STIG (against GP OS SRG) | Drafting phase |
| NSA CSfC | Components List submission planned |
| NIST SP 800-53 | Control-inheritance matrix STARTER published; FedRAMP-customer-ready |
| EUCC (EU) | Planned for European-allied procurement |

---

## NAICS codes

| Code | Description |
|---|---|
| 541511 | Custom Computer Programming Services |
| 541512 | Computer Systems Design Services |
| 541519 | Other Computer Related Services |

---

## Small-business status

{IF_APPLICABLE_LIST_8(a)_SDVOSB_HUBZONE_WOSB_ETC_OR_"Small business, not currently in a set-aside program"}

---

## Differentiating clauses

- We are **NOT** another Linux distribution. We are a from-scratch microkernel in Rust.
- We are **NOT** a research project. Sphragis has booted on real Apple M4 hardware (April 2026) + has 24 sub-projects merged across a documented 36-month master plan + ships in two build profiles (community + sphragis-gov).
- We are **NOT** a thin wrapper over seL4. We cede full functional-correctness proofs to seL4 and claim a different, complementary property: information-flow non-interference on critical subsystems — provable in modern Rust verification tools (Verus) at a fraction of seL4's 25-person-year cost.

---

## Contact + next steps

To engage:

1. **Initial briefing**: 30-minute slide deck + live boot demo on M4 hardware. Request via {FOUNDER_EMAIL}.
2. **Technical deep-dive**: 90-minute session covering threat model + verified subsystem + attestation API. Suitable for AO scoping.
3. **Pilot deployment**: {PILOT_TERMS_OR_"Per mutually-agreed scope of work"}.

---

## Document version

| Version | Date | Notes |
|---|---|---|
| {VERSION} | {DATE} | Initial. Reflects state as of the autonomous-run-end snapshot (2026-05-16 + N days of founder paperwork). |

---

## How to use this template

1. **Copy** to a working doc (`CAPABILITY_STATEMENT_v1.md` in a private repo).
2. **Replace** every `{PLACEHOLDER}` with the founder-supplied value.
3. **Update** the "Past performance" and "Small-business status" sections as those evolve.
4. **Render to PDF** via:
   - LaTeX (preferred for typesetting consistency): `pandoc -o capstmt.pdf CAPABILITY_STATEMENT_v1.md`
   - Pages / Word: Export → PDF
5. **Distribute** via {COMPANY_DOMAIN}/capability-statement.pdf + email handoff after meetings.
6. **Version** quarterly OR after major milestones (first SBIR Phase I award, first ATO, first certification cert issuance).
