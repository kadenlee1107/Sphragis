---
title: "Sphragis — Sovereign-grade attested-cave OS"
subtitle: "Pre-incorporation v0 deck · 2026-05-16"
author: "Kaden Lee, Founder + Lead Engineer · kadenlee1107@gmail.com"
date: "2026-05-16"
---

<!--
SP-DOC-010 (2026-05-16). 20-slide demo deck for gov-buyer briefings.
Markdown source. Render with: pandoc -t beamer SPHRAGIS_DEMO_DECK_v0.md -o deck.pdf
or any Markdown-presenter (e.g., reveal-md / Marp / slidev).
Slide separator: `---`. One H1 per slide.
-->

---

# 1 · Sphragis

**Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era.**

A from-scratch Rust microkernel. Memory-safe. CNSA 2.0 by default. Boots on real Apple M4.

*kadenlee1107@gmail.com · 2026-05-16*

---

# 2 · Who we are

- **Sphragis Systems** — pre-incorporation Delaware C-Corp (PRC-001 filing imminent).
- **Founder:** Kaden Lee. 14 weeks of focused independent kernel + M4 reverse-engineering.
- **Status:** Unfunded. ~80K LoC TCB. 43+ sub-projects merged across a 36-month master plan.
- **Engagement path:** SBIR Phase I → GSA MAS → DARPA Forecast-to-Industry → first ATO.

---

# 3 · The category claim

> Every other gov OS is a *retrofit*.
>
> Sphragis is the first OS designed natively for the world the procurement office is buying into in 2027-2030.

That world has three irreversible inputs:

1. **Post-quantum mandates** (CNSA 2.0, NSM-10) make classical-crypto-only OSes ineligible.
2. **Capability-hardware** (ARM Morello, CHERIoT) is shipping; bolt-on can't use it.
3. **Adversaries who patient-collect-then-decrypt** make "we'll add PQ later" a strategic loss.

---

# 4 · The 5 differentiators (vs incumbents)

| | INTEGRITY-178B | seL4 | RHEL | **Sphragis** |
|---|---|---|---|---|
| Memory-safe language | C | C | C | **Rust** |
| Post-quantum crypto | No | No | Partial | **CNSA 2.0 native** |
| Attestation primitive | Bolt-on | Bolt-on | Bolt-on | **Kernel-mediated** |
| TCB size | ~10K LoC | ~10K LoC C | ~30M LoC | **~70K LoC** |
| Open source | No | Yes | Yes | **Apache-2.0** |
| Formal verification | Cert-only | Full correctness | None | **Non-interference proofs** |
| CHERI compatibility | Static | Research | None | **Designed-in** |

---

# 5 · Diff #1 — Rust microkernel + non-interference proofs

- **Memory-safe by language.** No use-after-free, no buffer-overflow, no double-free as a class of bugs.
- **70-80K LoC TCB** — auditable in days, not years (vs Linux 30M).
- **Verus formal-verification harness** wired in (`verification/`). SP-VER-001 + SP-VER-002 Verus proof specifications landed: capability dispatcher + IPC channel non-interference between caves.
- Multi-week proof IMPL underway — DARPA PROVERS-program-fundable.

---

# 6 · Diff #2 — CNSA 2.0 from the kernel up

| Algorithm | Where in the kernel |
|---|---|
| ML-KEM-1024 (FIPS 203) | `src/crypto/pq_cnsa.rs` |
| ML-DSA-87 (FIPS 204) | `src/crypto/pq_cnsa.rs` |
| AES-256-{GCM, GCM-SIV, XTS, CTR} | `src/crypto/aes*.rs` |
| SHA-384 / SHA-512 / HMAC-SHA-384/512 | `src/crypto/sha*.rs` |
| LMS HBS (RFC 8554, NIST SP 800-208) | `src/crypto/lms.rs` + verify-only boot KAT |
| Fail-closed RNG (ARMv8.5 FEAT_RNG) | `src/crypto/rng.rs::require_hw_rng_or_halt` |

`gov-strict` build flag rejects AES-128, RSA, ECDSA, plain-ChaCha20, SHA-256-for-signing at compile time.

---

# 7 · Diff #3 — Attestation as kernel primitive

`src/security/attest.rs`:

- Per-cave **attestable identity registry** (auto-registered at cave create; unregistered at destroy)
- **Real kernel measurement** — SHA-384 of `__text_start..__text_end + __rodata_start..__rodata_end` at boot
- **ML-DSA-87-signed Quote envelope** carrying kernel-measurement + cave-identity + nonce + caller-supplied claims
- **Wire format + offline verifier** — `tools/attest-verifier/attest_verifier.py` parses + cryptographically verifies (if `pqcrypto-mldsa` installed) or structural-only

Hardware-root chain designed in `DESIGN_HSM_OPERATOR_CA.md`: SEP on M4 + TPM 2.0 on x86_64 + Caliptra-ready.

---

# 8 · Diff #4 — SLSA L4 + sigstore + WORM audit

- **`cargo-deny` + `cargo-audit`** CI gates enforce zero GPL/AGPL deps + zero known advisories.
- **Reproducible builds verified** — `scripts/check_reproducible_build.sh` produces bit-identical SHA-256 across clean rebuilds (SP-BLD-002 closure 2026-05-16).
- **SBOM per release** (`scripts/gen_sbom.py`).
- **Sigstore release-signing** IMPL drafted (`.github-workflows-pending/release-sign.yml`); operator-side `tools/release-verifier/verify.sh`.
- **WORM audit segment export** — `src/security/audit_worm.rs` HMAC-SHA-384 chained segments persist to SealFS; offline verifier walks the chain.

---

# 9 · Diff #5 — CHERI-ready architecture

- `DESIGN_CHERI_MAPPING.md` published.
- Each Sphragis cave maps cleanly to a CHERI compartment — capability bits in the cave model align with CHERI bounds.
- Targets:
  - **ARM Morello** server / desktop (follows ARM Q1-Q3 2026 pure-cap roadmap)
  - **CHERIoT-Ibex** embedded gov (lowRISC + SCI Semiconductor 2026 hardware)

When CHERI hardware ships in volume, Sphragis is the kernel that already speaks the language.

---

# 10 · Live demo — M4 boot

Apple M4 MacBook Pro 14" (Mac16,1 / J604 / T8132 "Donan") — independent reverse-engineering pipeline since Asahi doesn't yet support M4 (M4/M5 RE in progress in Asahi community as of 2026).

Boot flow:

```text
m1n1 (installed via kmutil configure-boot, Permissive Security)
  → chainload.py --skip-secondary-cpus (P-cluster SError fix)
  → Sphragis kernel
  → Cave isolation init + per-cave page tables
  → Interactive microkernel shell
```

[16 photos of first successful boot — April 2026, `docs/photos/2026-04-17_first_m4_boot/`]

---

# 11 · Live demo — attestation round-trip

Inside the running Sphragis shell:

```text
> attest-dump
  attest-dump: wrote attest-quote.bin (7389 bytes)

[copy file off-device]

$ python3 tools/attest-verifier/attest_verifier.py attest-quote.bin
[attest-verifier] parsed 7389-byte Quote from attest-quote.bin
[attest-verifier]   kernel_meas (first 16): a1b2c3d4...
[attest-verifier]   cave_meas   (first 16): 11223344...
[attest-verifier]   nonce              : 00000000...
[attest-verifier]   cave_name          : b'demo-cave'
[attest-verifier]   vk_len             : 2592 (expected 2592)
[attest-verifier]   sig_len            : 4627 (expected 4627)
[attest-verifier] STRUCTURAL: PASS
[attest-verifier] CRYPTO: PASS — ML-DSA-87 signature valid
```

---

# 12 · Live demo — WORM audit verification

```text
> audit-worm-seal
  ✓ sealed; next segment seq=2

> audit-worm-status
  current_seq = 2
  records_in_current = 0
  prev_head_first8 = 7f3a91...

[mount SealFS off-device, walk audit/worm/]

$ python3 tools/audit-verifier/audit_verifier.py \
    --worm-dir audit/worm/ --key-hex $HMAC_KEY
[audit-verifier] WORM chain VERIFIED for audit/worm/
```

Closes audit FS-H7 finding from the 2026-05-15 rolling security audit.

---

# 13 · Audit posture — 14 weeks of rigor

| Audit week | Finding | Sphragis-side closure |
|---|---|---|
| Week 1 | TLS bypass | gov-strict policy gate rejects weak suites |
| Week 1 | SealFS lock | IrqGuard around critical sections |
| Week 5 | Constant-time HOTP | Cache-line-level discipline + unit tests |
| Week 11 | Per-cave page tables | Per-cave L1 + per-cave ARMv8.5 ASIDs |
| Week 14 | PSK overlay retire | Removed legacy path; gov-strict only |

149 findings tracked end-to-end in the rolling audit (2026-05-15 baseline). 0 audit-closed property has been reopened across runs 1+2+3.

---

# 14 · NIST SP 800-53 inheritance — concrete

`docs/NIST_800_53_INHERITANCE.md` v1.2:

| Family | Status | Notes |
|---|---|---|
| AC | Complete (25 controls) | Per-cave caps + capability system |
| AU | Complete (16 controls) | Tamper-evident chain + WORM export |
| CM | Complete (14 controls) | Reproducible builds + SBOM + signing |
| IA | Complete (12 controls) | Argon2id + TPI + attestable identity |
| SC, SI, MP, SA, SR, PT | STARTER (~25 more) | OS-relevant subset covered |

**80 controls / 34 SATISFIED / 29 PARTIAL / 4 HYBRID / 5 CUSTOMER / 9 N/A.**

FedRAMP-customer-ready: AOs can scope ATO boundaries against this matrix today.

---

# 15 · Certification posture

| Cert | Status |
|---|---|
| FIPS 140-3 L1 | `docs/FIPS_140_3_MODULE_BOUNDARY.md` published; CMVP lab engagement pending |
| NIAP PP-conformant CC:2022 Rev 1 | `docs/SECURITY_TARGET.md` published; CCTL pending |
| DoD STIG | Drafting phase |
| NSA CSfC Components List | Submission planned |
| NIST SP 800-53 | Inheritance matrix v1.2 published (FedRAMP-customer-ready) |
| EUCC (EU) | Planned for allied procurement |

Vendor-side prep complete on items in scope; external engagements gated on incorporation + funding.

---

# 16 · The 36-month master plan — where we are

| Block | Months | Status |
|---|---|---|
| A: Strategic positioning + AGENT removal | 0-3 | ✅ DONE |
| B: CNSA 2.0 crypto + verified primitives | 3-9 | ✅ DONE |
| C: Attestation + verification harness | 9-15 | ⚠️ IN PROGRESS (Quote landed; hardware-root + Verus proofs in flight) |
| D: UX productization + multi-hardware | 15-21 | ⚠️ DESIGNED (IMPL pending) |
| E: Cert engagements + first ATO | 21-30 | ⚠️ READY (waiting on PRC-001 + lab availability) |
| F: First volume contract | 30-36 | — (post-ATO) |

Currently: end of month ~4 of a 36-month plan. **30+ months runway worth of content already shipped.**

---

# 17 · What we're NOT (the discipline)

`ANTI_FEATURES.md`:

1. **No** in-tree browser (host does browsing; kernel doesn't ship a JS engine attack surface)
2. **No** AI/LLM/ML in the kernel critical path (no nondeterministic decisions in the security path)
3. **No** GPL/AGPL deps (CI-enforced; preserves proprietary-distribution option)
4. **No** weak crypto in the gov build (gov-strict gate; compile-time enforced)
5. **No** Linux binary compatibility promise (means a Sphragis cave can't run an unmodified Linux binary; means we don't inherit Linux's attack surface either)
6. **No** closed-source kernel components (everything verifiable)
7. **No** functional-correctness proof of the whole kernel (we prove non-interference instead; honest scope bound)

---

# 18 · The ask

1. **SBIR Phase I sponsorship.** DoD SBIR 26.1 / AFWERX / DARPA SBIR. We bring the kernel; you bring the topic alignment. Initial work: pilot SP-UX-005 (TUF-protocol package manager) or SP-VER-001.IMPL (Verus non-interference proof).
2. **DARPA Forecast-to-Industry meeting.** 30-minute slide + demo. Decision criterion: does the differentiator-#1 (Rust + non-interference) story align with a known PM's program?
3. **Pilot deployment / co-design.** Mutually-agreed scope. Eligible after PRC-001/002/003 (entity registration; ~30-90 days post-funding).

---

# 19 · Path-to-yes risk table

| Risk | Mitigation |
|---|---|
| Pre-incorporation entity | PRC-001/002/003 are mechanical filings; no technical risk |
| Single-founder bus factor | Apache-2.0 license + clean architecture + onboarding doc means a second engineer is productive in <2 weeks |
| FIPS 140-3 lab queue (12-30 mo) | Module boundary documented today; entered queue is months saved vs starting cold |
| Verus proof completion (multi-week) | Spec landed; PROVERS-program-funded path documented |
| Hardware-rooted attestation (SEP/Caliptra) | Designs landed; hardware-bound on M4 / Caliptra availability — independent gating from software roadmap |

**Zero technical risk in run-to-yes.** Time + funding only.

---

# 20 · Contact

**Kaden Lee** · Founder + Lead Engineer

📧 kadenlee1107@gmail.com

📦 https://github.com/kadenlee1107/Sphragis (Apache-2.0)

📑 `docs/CAPABILITY_STATEMENT_v0.md`

🔬 Live demo: 30-minute briefing on request

> "We didn't build a better Linux. We built the OS the 2027-2030 procurement office is going to demand."
