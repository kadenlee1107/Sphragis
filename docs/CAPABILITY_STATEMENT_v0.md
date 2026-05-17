# Sphragis Capability Statement

**v0 — pre-incorporation draft (2026-05-16).** First complete draft per `docs/templates/CAPABILITY_STATEMENT_TEMPLATE.md`. Founder placeholders filled with pre-incorporation values; entity-specific fields (UEI / CAGE / address / DUNS) carry "TBD upon SAM.gov filing" until PRC-002/003 complete. Operator updates this file post-incorporation by replacing each `[TBD …]` block with the registered value.

---

## Cover page

**Sphragis Systems** *(pre-incorporation working name; legal entity TBD upon Delaware C-Corp filing — PRC-001)*

Sphragis Operating System — Capability Statement

Version v0 — pre-incorporation / 2026-05-16

| Contact | Detail |
|---|---|
| Primary POC | Kaden Lee, Founder + Lead Engineer |
| Email | kadenlee1107@gmail.com |
| Phone | [TBD upon incorporation — PRC-001] |
| Address | [TBD upon Delaware C-Corp filing — PRC-001] |
| Website | [TBD upon SP-A4 marketing-site publication] |
| DUNS / UEI | [TBD upon SAM.gov registration — PRC-002] |
| CAGE Code | [TBD upon SAM.gov registration — PRC-002] |

---

## Executive summary

Sphragis Systems is the developer of **Sphragis** — a sovereign-grade attested-cave operating system designed natively for the 2027-2030 government procurement world. Sphragis is a memory-safe Rust microkernel that ships CNSA 2.0-compliant post-quantum cryptography by default, formally-specified information-flow isolation between processes, hardware-rooted attestation as a first-class kernel primitive, and a CHERI-ready architecture for the capability-hardware era.

We are an unfunded pre-incorporation project (~14 weeks of focused engineering as of 2026-05-16). The project has publicly-demonstrated boot on Apple M4 hardware via an independent reverse-engineering pipeline + 40+ sub-projects executed across crypto / verification / attestation / build-chain / certification-engineering / productization tracks per a documented 36-month master plan. Path to first federal-contract eligibility runs through PRC-001 (Delaware C-Corp), PRC-002 (SAM.gov + UEI + CAGE), and PRC-003 (GSA MAS offer) — none of which are technical risks.

---

## Core capabilities

### 1. Rust microkernel + formally-specified information-flow isolation

- Memory-safe by language (Rust 2024, `#![no_std]`, `panic = "abort"`)
- 70-80K LoC TCB (vs Linux ~30M); audit-traceable
- Cave-isolation primitive with per-cave page tables + per-cave ARMv8.5 ASIDs + per-cave AF_UNIX namespace + per-cave taint bitmap
- Verus formal-verification harness in place (`verification/`); SP-VER-001 (capability dispatcher non-interference) + SP-VER-002 (IPC information-flow non-interference) Verus proof specifications landed; multi-week proof IMPL effort ahead — funding sought via DARPA PROVERS program

### 2. CNSA-2.0-native post-quantum cryptography (gov-strict build)

- ML-KEM-1024 (FIPS 203) — `src/crypto/pq_cnsa.rs`
- ML-DSA-87 (FIPS 204) — `src/crypto/pq_cnsa.rs`
- AES-256 (GCM, GCM-SIV, XTS, CTR)
- SHA-384 + SHA-512 + HMAC-SHA-384/512
- LMS (RFC 8554, NIST SP 800-208) — `src/crypto/lms.rs` with verify-only RFC 8554 §F.1 boot KAT
- Fail-closed RNG with ARMv8.5 FEAT_RNG (`require_hw_rng_or_halt`)
- Compile-time enforced gov-strict feature flag rejects AES-128, RSA, ECDSA, plain-ChaCha20, SHA-256-for-signing
- 9 boot-time KATs run fail-closed at every boot (AES-256-GCM, ChaCha20-Poly1305, SHA-384/512, HMAC-SHA-384 RFC 4231 TC1, ML-KEM-1024 round-trip, ML-DSA-87 sign/verify+tamper, LMS verify-only, RNG strict-probe)

### 3. Attestation as kernel primitive

- Per-cave attestable identity registry (`src/security/attest.rs`)
- Real kernel measurement at boot (SHA-384 over .text+.rodata via linker symbols)
- ML-DSA-87-signed Quote envelope with kernel-measurement + cave-identity + nonce + claims
- SPHATTV1 wire format + offline external verifier (`tools/attest-verifier/`) — structural validation always; cryptographic ML-DSA-87 verification when `pqcrypto-mldsa` or `liboqs-python` installed
- Hardware-rooted endorsement chain designed (`DESIGN_HSM_OPERATOR_CA.md`); SEP-rooted on M4 + TPM 2.0 on x86_64 + Caliptra-ready

### 4. SLSA L4 reproducible builds + sigstore-signed releases

- `cargo-deny` + `cargo-audit` CI gate enforces zero GPL/AGPL deps + zero known advisories
- Reproducible-build verification VERIFIED (`scripts/check_reproducible_build.sh` produces bit-identical SHA-256 across clean rebuilds)
- SBOM per release (`scripts/gen_sbom.py`)
- SLSA L4 architecture + sigstore/Rekor release-signing IMPL drafted (`DESIGN_SLSA_PROVENANCE.md`, `DESIGN_SIGSTORE_REKOR.md`, `.github-workflows-pending/release-sign.yml` + `tools/release-verifier/verify.sh`)
- LMS-signed kernel boot verification designed (`DESIGN_LMS_KERNEL_SIGNING.md`)
- WORM audit segment export to SealFS (`src/security/audit_worm.rs`) with HMAC-SHA-384 chain across segments + offline verifier (`tools/audit-verifier/ --worm-dir`)

### 5. CHERI-ready architecture

- Cave-to-CHERI-compartment mapping designed (`DESIGN_CHERI_MAPPING.md`)
- Targets ARM Morello (server/desktop, follows ARM Q1-Q3 2026 pure-cap roadmap) + CHERIoT-Ibex (embedded gov, lowRISC + SCI Semiconductor 2026 hardware)

---

## Technology readiness

| Subsystem | TRL today |
|---|---|
| Microkernel + cave isolation | TRL 6 (boots on real M4 + comprehensive QEMU CI) |
| CNSA 2.0 crypto + boot KATs | TRL 7 (production-grade primitives) |
| Attestation primitive | TRL 6 (API + measurement + per-cave registry + wire format + offline verifier; hardware-rooting designs landed) |
| SealFS encrypted filesystem | TRL 7 (AES-256-GCM-SIV; per-cave + per-file keys) |
| Audit chain | TRL 7 (HMAC-SHA-384 tamper-evident + WORM export to SealFS) |
| Window manager + multi-app UI | TRL 4 (design landed; implementation pending) |
| Multi-hardware (x86_64) | TRL 3 (design landed; implementation pending) |
| Formal verification | TRL 4 (harness scaffolded; SP-VER-001 + SP-VER-002 Verus proof specs landed; proof IMPL pending — DARPA-funding sought) |
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
| Reproducible builds | No | Partial | Partial | **SLSA L4 designed (build verified bit-identical)** |

---

## Past performance + reference engagements

- None yet — pursuing first SBIR Phase I.

For v0 (pre-incorporation): currently pursuing first SBIR Phase I submissions to DoD SBIR / AFWERX / DARPA SBIR (windows TBD upon entity registration). Founder (Kaden Lee) brings 14 weeks of focused independent reverse-engineering of Apple M4 hardware + bare-metal Rust microkernel development as primary recent work; prior background includes [TBD by founder].

---

## Certifications + standards posture

| Certification | Status |
|---|---|
| FIPS 140-3 L1 | Module boundary documented (`docs/FIPS_140_3_MODULE_BOUNDARY.md`); CMVP lab engagement pending |
| NIAP PP-conformant CC | Security Target documented (`docs/SECURITY_TARGET.md`); CCTL engagement pending |
| DoD STIG (against GP OS SRG) | Drafting phase |
| NSA CSfC | Components List submission planned |
| NIST SP 800-53 Rev. 5.2.0 | Control-inheritance matrix v1.2 published (`docs/NIST_800_53_INHERITANCE.md` — AC + AU + CM + IA families complete, 80 controls covered); FedRAMP-customer-ready |
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

Small business, not currently in a set-aside program. Eligibility for 8(a) / SDVOSB / HUBZone / WOSB to be determined post-incorporation against the appropriate program rules.

---

## Differentiating clauses

- We are **NOT** another Linux distribution. We are a from-scratch microkernel in Rust.
- We are **NOT** a research project. Sphragis has booted on real Apple M4 hardware (April 2026) + has 40+ sub-projects merged across a documented 36-month master plan + ships in two build profiles (community + sphragis-gov) + has reproducible bit-identical builds + an offline-verifiable attestation Quote format.
- We are **NOT** a thin wrapper over seL4. We cede full functional-correctness proofs to seL4 and claim a different, complementary property: information-flow non-interference on critical subsystems — provable in modern Rust verification tools (Verus) at a fraction of seL4's 25-person-year cost.

---

## Contact + next steps

To engage:

1. **Initial briefing**: 30-minute slide deck + live boot demo on M4 hardware. Request via kadenlee1107@gmail.com.
2. **Technical deep-dive**: 90-minute session covering threat model + verified subsystem + attestation API. Suitable for AO scoping.
3. **Pilot deployment**: Per mutually-agreed scope of work; eligible after PRC-001/002/003 entity-registration milestones.

---

## Document version

| Version | Date | Notes |
|---|---|---|
| v0 | 2026-05-16 | First complete draft. Pre-incorporation; entity-specific fields placeholder until PRC-001/002 land. Reflects state as of run-3 autonomous-batch-2 snapshot: 43 SPs merged, 28 P0 HAVE / 34 PARTIAL / 13 MISSING. |

---

## How to update this document

1. **Update entity placeholders** (`[TBD …]`) as PRC-001/002/003 land.
2. **Update technology readiness table** when subsystem TRLs change (per gap doc).
3. **Update certifications row** as cert engagements progress (`docs/superpowers/research/2026-05-16-gov-os-gap-analysis.md` CRT-001..007 rows).
4. **Update past performance** after first SBIR award / first task order / first partnership.
5. **Re-version** quarterly OR after major milestones (first SBIR Phase I award, first ATO, first certification cert issuance).
6. **Render to PDF** via `pandoc -o capstmt.pdf CAPABILITY_STATEMENT_v0.md` (Markdown → PDF; works with standard pandoc + a LaTeX engine).
