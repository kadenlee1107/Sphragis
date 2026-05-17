# SBIR Phase I Proposal — Volume 2: Technical Proposal

**Solicitation:** AFWERX Open Topic (DoD SBIR)
**Proposer:** Sphragis Inc. (Delaware C-Corp — incorporation in flight)
**Proposal title:** Sphragis-CDX: Capability-Isolated Cross-Domain Compute Substrate for the Post-Quantum, Attested-Hardware Era
**Proposed performance period:** 6 months
**Requested funding:** $75,000
**Topic alignment:** Critical Infrastructure / Cyber / Information Operations
**Version:** v0 draft, 2026-05-17

> **Founder action — fields to fill before submission:**
> - Cover sheet info (CAGE/UEI/DUNS once issued, banking info)
> - §3.7 Key Personnel (founder CV, education, prior projects)
> - §3.8 Foreign Citizens (declare citizenship of every team member who'll work on the contract)
> - §3.9 Facilities (workspace address; "home office" is acceptable for a 1-2 person SBIR)
> - §3.11 Prior, Current, or Pending Support (other federal funding the same work is being submitted for — currently NONE if this is first SBIR)
> - Volume 3 Cost Volume (separate workbook; numbers below feed into it)
> - Volume 4 Company Commercialization Report (SBA-provided template)

---

## 3.1 Identification and Significance of the Problem

The Department of Defense operates across a tiered classification system that demands strict separation between information at different sensitivity levels — Unclassified, Confidential, Secret, Top Secret, and SCI/SAR caveats. Today's cross-domain solutions (CDS) that bridge these tiers are dominated by legacy products built on dedicated hardware: TENIX Data Diodes, Forcepoint Trusted Thin Client, ManTech HoloDeck, Owl Cyber Defense. These products were architected when:

- Crypto was classical (RSA-2048, ECDSA, SHA-256) — the entire NSA Commercial National Security Algorithm Suite 2.0 cliff at 2027-01-01 invalidates these for new acquisitions
- Attestation was a TPM-1.2 afterthought rather than a kernel-mediated primitive
- Memory safety was assumed away by C++ codebases that produce >70% of modern CVEs
- Formal verification was a research curiosity, not a procurement deliverable
- Capability-safe hardware (CHERI, CHERIoT) did not exist

The result: DoD's installed CDS base is **expensive** (six-figure per-unit hardware costs), **closed** (proprietary firmware blocks third-party assurance), **outdated** (algorithms NSA has explicitly deprecated), and **inflexible** (fixed hardware can't be repurposed for new mission contexts like AI-inference compartments or coalition data sharing).

The 2027-01-01 NSA CNSA 2.0 mandate combined with the FIPS 140-2 → 140-3 cliff (also 2026-09-21) creates an acquisition forcing function: every new National Security System cryptographic deployment must use ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384, validated through FIPS 140-3. The installed CDS base does not meet this bar. There is **no current commercial replacement** that is simultaneously: memory-safe (Rust or formally-verified), CNSA-2.0-native, attestation-mediated, reproducibly built, and CHERI-ready for the 2027-2030 capability-hardware deployment window.

**Sphragis is the substrate that fills that gap.**

Sphragis is a security-first bare-metal Rust microkernel that organizes user workloads into capability-isolated "caves" with Bell-LaPadula sensitivity labels and Biba integrity labels enforced by the kernel at every cross-cave syscall. It boots today on Apple M4 hardware (verified — boot evidence in our public repository) and on QEMU virt aarch64. The audit history spans 14 weeks of mechanical-trace remediation closing 32 critical/high findings with traceable git commits. The current TCB is approximately 96,000 lines of Rust (versus Linux's ~30M lines and seL4's ~10K lines of C plus proof corpus), placing Sphragis in a defensible position between Linux (too large to certify) and seL4 (too narrow to deploy as a complete OS).

The proposed Phase I effort scopes a 6-month feasibility study demonstrating that Sphragis can serve as the foundation substrate for a next-generation, software-defined Cross-Domain Solution targeted at tactical edge endpoints — replacing six-figure dedicated CDS hardware with a $5K commodity-class Apple-Silicon or x86-64 laptop running a CNSA-2.0-compliant, attestable, formally-grounded OS.

## 3.2 Phase I Technical Objectives

The Phase I scope is bounded to a feasibility demonstration. Specifically, the work will:

**Objective 1 — Demonstrate end-to-end information-flow control across MLS-labeled caves on Apple M4 hardware.**
Output: a working prototype where two caves at different sensitivity levels (e.g., U-cave reading a Secret-labeled file is rejected; S-cave writing to a U-labeled BatFS namespace is rejected) exercises the Bell-LaPadula + Biba dual-lattice enforcement in `src/caves/cave.rs` and `src/fs/sealfs.rs`. Demonstrated via QMP-driven test harness (already 85 self-test scripts exist; add 4-6 specifically exercising MLS bypass-attempt paths). Deliverable: PASS/FAIL log + screenshot capture for each scenario.

**Objective 2 — Produce CNSA-2.0-compliant attestation Quotes from each cave with external verification.**
The kernel's existing `attest::quote(nonce, claims) -> Quote` API produces ML-DSA-87-signed CBOR quotes today; Phase I will harden the Quote contents to bind cave-measurement + audit-log digest + attestation root cert chain, and produce a reference verifier tool that an external auditor can run independently. Output: working flow where an external Python verifier (extending the existing `tools/audit-verifier/`) accepts a captured Quote bundle and reports trust/no-trust binding to a Caliptra-equivalent root key. Verifier ships as Apache-2.0 alongside the proposal repository.

**Objective 3 — Produce a Verus specification of cave non-interference with a partial proof.**
A complete machine-checked proof is multi-session work outside Phase I budget. Phase I scopes: (a) a complete Verus specification at `verification/cap_dispatch/SPEC.md` declaring the non-interference property formally; (b) Verus proof attempts on the simplest non-trivial subset (one IPC primitive or one syscall dispatch arm); (c) a written technical report identifying which parts of the dispatcher resist proof and why. Output: spec doc + partial proof + analysis of remaining work. This sets up Phase II to complete the proof.

**Objective 4 — Document FIPS 140-3 cryptographic-module boundary and engage a CMVP lab for pre-engagement scoping.**
The boundary doc exists (`docs/FIPS_140_3_MODULE_BOUNDARY.md`). Phase I work refines the doc per CMVP review feedback and produces a written engagement letter from at least one accredited lab (Atsec, Leidos, or InfoGard) scoping cost and timeline for a full Level 1 validation. Output: lab engagement letter + revised boundary doc + Phase II budget line.

**Objective 5 — Reproducible build pipeline with SLSA-L4 attestation chain.**
The build is already bit-for-bit reproducible (SHA-256 `f4b12add37d44d4ae031a0bc5db83739a15c2d54d7d8096e1fcb667ca7e5ad03` verified). Phase I extends this to a complete SLSA-L4 chain: sigstore-signed release artifacts with Rekor transparency log entries + in-toto attestation envelopes + LMS-signed kernel images. Output: working release process producing an artifact bundle that an external auditor can verify offline using only Apache-2.0 tools.

These five objectives are mutually independent and can be parallelized across the 6-month period. Each produces a discrete, evaluable deliverable.

## 3.3 Phase I Statement of Work

**Month 1 — Baseline + scope refinement.**
Confirm Phase I deliverables with AFRL / AFWERX TPOC. Refresh the FIPS 140-3 module boundary doc against latest CMVP guidance. Begin Verus toolchain hardening at `verification/`. Reproducible build verification re-run on three independent build hosts to confirm determinism survives toolchain drift.

**Month 2 — MLS enforcement test bench.**
Author 4-6 new QMP-driven test scripts exercising MLS bypass-attempt scenarios. Wire results into the existing QEMU CI harness. Capture PASS/FAIL evidence as proposal deliverable.

**Month 3 — Attestation flow.**
Harden `attest::quote()` to bind audit-log digest + cave-measurement + attestation root cert chain. Implement external Python verifier extending `tools/audit-verifier/`. Demonstrate end-to-end flow: cave produces Quote, external verifier accepts/rejects against allowlist of trusted root keys.

**Month 4 — Verus specification + partial proof.**
Author complete non-interference specification at `verification/cap_dispatch/SPEC.md`. Begin proof attempts on simplest dispatcher subset. Document which paths resist proof and propose Phase II strategy.

**Month 5 — CMVP lab engagement.**
Contact Atsec, Leidos, and InfoGard. Sign engagement letter with one for Phase II pre-engagement scoping. Refine FIPS 140-3 boundary doc per lab feedback.

**Month 6 — SLSA-L4 chain + final report.**
Extend reproducible build to full SLSA-L4: sigstore signing, Rekor entries, in-toto envelopes, LMS-signed kernel. Document the chain in a verifier runbook. Compile Phase I final report: technical feasibility, evidence chain (boot logs, screenshots, signed binaries, Quote bundle, partial proof, lab letter, FIPS boundary doc). Submit Phase II proposal if invited.

**Risk mitigation:**
- If Verus toolchain matures slower than expected, fall back to a smaller proof scope (single function rather than dispatcher subsystem).
- If CMVP lab engagement letters take >2 months, move to AWS / Azure HSM partnership for the attestation hardware story (parallel commercial path).
- If reproducible build breaks under a Rust toolchain upgrade mid-period, pin to a stable version for the duration and document the workaround in the Phase II proposal.

## 3.4 Related Work

**seL4** (https://sel4.systems) — the gold-standard formally-verified microkernel. seL4 has machine-checked functional-correctness proofs of ~8,830 lines of C plus binary correctness for AArch64 / x86_64. Deployments include NIO SkyOS (mass-production cars), HENSOLDT TRENTOS (defense systems), NASA cFS, DARPA HACMS. seL4 is **complementary, not competitive** to Sphragis: seL4 is a kernel substrate with no built-in filesystem, network stack, or user-space tooling, while Sphragis is a complete OS targeting end-user deployment. Sphragis cedes "full functional-correctness proof" to seL4 (see our `ANTI_FEATURES.md` §ANTI-001) and claims information-flow non-interference on critical subsystems via Verus, which is a tractable proof scope for our staffing model.

**Green Hills INTEGRITY-178B** — the only OS ever certified against the now-sunset Separation Kernel Protection Profile at CC EAL6+ "High Robustness" in 2008 on PowerPC 750CXe. Deployments: F-35, F-22, B-1B, B-52, F-16, C-130J, C-17. **Closed-source, frozen-config, classical-crypto-only.** Does not meet CNSA 2.0. No published roadmap to ML-KEM/ML-DSA.

**LynxOS-178** (Lynx Software Technologies) — DO-178C DAL A; the only FAA-approved RSC OS. Avionics-focused.

**PikeOS** (SYSGO GmbH) — CC EAL5+ (v5.1.3, 2022); DO-178C DAL-A. Airbus A350 XWB IMA computers, European rail (CENELEC EN 50128), automotive. Closed source.

**VxWorks 653 / Helix** (Wind River) — ARINC 653; DO-178C/ED-12C DAL A. Boeing 787, P-8A Poseidon, A330 MRTT, A400M, UH-60V. Closed source.

**Red Hat Enterprise Linux** — dominant enterprise Linux (~43.1% market share 2025); CC EAL4+ against OSPP via SELinux MLS+RBAC. **TCB too large** (~30M lines) for formal proofs of non-interference; not memory-safe (C); CNSA 2.0 retrofit via OpenSSL is incomplete. Production gov deployments via NIAP PCL listing.

**Qubes OS** — Xen-based per-VM isolation. Used by Snowden, SecureDrop. **Not formally certified**, no FIPS, no CNSA, no path to NIAP. Strong analyst-workstation precedent but not a sellable gov product.

**Confidential computing TEEs (AWS Nitro, Azure Confidential VMs, Anjuna, Edgeless Systems, Fortanix)** — provide encrypted-memory isolation but rely on the underlying Linux guest's security posture. They do **not** offer the small TCB + capability isolation + formal-proof differentiator Sphragis provides.

**The strategic gap:** no incumbent ships a memory-safe, formally-grounded, CNSA-2.0-native, reproducibly-built OS substrate with capability-based MLS enforcement. Sphragis fills that gap simultaneously across all five dimensions.

## 3.5 Relationship with Future Research and Development

**Phase II ($1.25M / 21 months)** scope:

1. Complete the Verus non-interference proof on the capability dispatcher (Phase I produces specification + partial proof; Phase II completes the proof and extends to IPC and shm subsystems).
2. Achieve FIPS 140-3 Level 1 cryptographic module certification (Phase I produces lab engagement; Phase II runs the validation through CMVP).
3. Port to a second hardware reference (Intel NUC x86_64 baseline) demonstrating multi-platform.
4. Author + submit DoD STIG against the GP OS SRG (draft exists; Phase II refines to DISA acceptance).
5. Author + submit NSA CSfC Components List package for Mobile Access or Data-at-Rest capability.
6. Window-manager + installer UX so the OS is operator-deployable without Sphragis engineering staff at the keyboard.

**Phase III** (sole-source via Phase III authority, no upper limit):

Direct deployment of Sphragis-CDX into AFRL pilot programs (likely ACT 3 task-order via teaming with AIS, CNF, Global InfoTek, Invictus, or Radiance), NSWC Crane embedded contexts, AFLCMC Battle Management secure-comm endpoints, or DIU-sponsored tactical edge programs. Parallel commercial Phase III tracks: confidential AI inference (Anthropic, OpenAI), defense prime OEM licensing (Lockheed, Northrop, BAE), automotive Tier 1 suppliers (Bosch, Continental — NIO precedent on seL4 indicates ISO 26262 ASIL-D market is real and currently underserved by memory-safe options).

**Long-term R&D pipeline:**

- CHERI-ready architecture (CHERIoT-Ibex prototype boot, then ARM Morello pure-cap port) leverages the upcoming 2026-2028 capability-hardware market window.
- Caliptra 2.x silicon root-of-trust integration for x86_64 server contexts.
- EU EUCC certificate at "High" assurance (AVA_VAN.3+) for allied procurement.
- USENIX Security 2027 / NDSS 2027 peer-reviewed publication on the Verus non-interference proof results — academic credibility marker for FFRDC (MITRE, MIT LL, JHU/APL, GTRI) relationships.

## 3.6 Commercialization Strategy

**Dual-use is genuine, not boilerplate.** Sphragis's technical attributes — small TCB, attestation-as-primitive, formal verification on critical subsystems, reproducible builds — are equally valuable in **four distinct commercial markets**:

**Market 1 — Confidential AI inference for hyperscalers** ($5-20M ARR/customer plausible).
Anthropic, OpenAI, Meta AI, and emerging providers need to prove to enterprise customers that on-device or in-cloud AI inference cannot be observed by the operator. Today, providers wrap Linux inside a confidential VM (SEV-SNP/TDX/CCA) — that gives memory encryption but the Linux TCB is still ~30M lines. Sphragis offers a 96K-line TCB alternative deployable as a confidential VM guest OS. Sales cycle: 3-9 months; no cert gating; engineering scope is bounded.

**Market 2 — Defense prime OEM licensing** (Lockheed, Northrop, BAE, General Dynamics, RTX, Booz Allen).
Apache-2.0 license + verifiable supply chain + DoD-relevant differentiators position Sphragis as an embeddable security substrate inside primes' proprietary product lines. Sales cycle: 1-2 years; deal sizes $1-10M perpetual + per-device royalty. The DARPA INSPECTA / PROVERS / RSSC program ecosystem provides natural intro paths.

**Market 3 — Automotive Tier 1 suppliers** (Bosch, Continental, Aptiv/Wind River; NIO already ships seL4-based SkyOS in mass-production cars).
ISO 26262 ASIL-D + ISO/SAE 21434 cybersecurity require memory-safe + formally-verified substrates. Sphragis as a Rust + Verus alternative to seL4 hits an unfilled lane. Sales cycle: 12-24 months; volume play (millions of vehicles per design win).

**Market 4 — High-assurance enterprise** (HSM vendors Thales/Utimaco/Entrust, institutional crypto custody Anchorage/Fireblocks/BitGo, high-frequency trading firms Citadel/Jane Street).
Tight TCB + attestation primitive + AEAD-protected storage maps directly onto institutional key management. Sales cycle: 6-12 months; deal sizes $500K-$5M.

**Federal commercialization strategy:**

- **GSA MAS Schedule** (Software SIN 511210; IT Services SIN 54151S) — target award by Month 12 of Phase II using Phase I as past performance.
- **NSWC Crane "Connect to Crane"** program for tactical edge demos.
- **DIU OTA** consortium participation (IWRP or C5; $10-25K consortium membership).
- **ACT 3 IDIQ subcontract** via teaming agreement with one of the five primes (AIS, CNF, Global InfoTek, Invictus, Radiance).
- **In-Q-Tel pitch** after Phase II proof points (strategic capital, not check-size primary).

**Capital strategy:**

3-year financial model assumes Phase I ($75K, M9) + Phase II ($1.25M, M27) + one defense-focused seed VC round ($1-3M, M18) + STRATFI bridge if needed ($3-15M with matching). Founder commits to bootstrapping the gap with personal/angel capital while SBIR is in flight. The 36-month plan converts to first commercial gov revenue at Month 30-36 ($500K-$5M annual run rate plausible).

## 3.7 Key Personnel

> **[FOUNDER TO FILL]**
>
> **Principal Investigator:** [Kaden Lee, Founder & CEO, Sphragis Inc.]
> **Education:** [degree, institution, year]
> **Prior projects:** [Bat_OS → Sphragis 14-week security audit history is the strongest proof point; include link to public GitHub repository, M4 boot evidence photo set, and the docs/superpowers/ planning corpus]
> **Time commitment Phase I:** [estimate hours/week; 75% time on Phase I work is typical for principal investigator]
> **Co-investigators or technical staff:** [if any; ok to be single-PI for a small Phase I]

**Reviewer note:** A 1-2 person SBIR Phase I is well-precedented at AFWERX and is supported by the proposal scope above (5 objectives, each producing a discrete deliverable in 6 months at ~1.5 person-month total budget).

## 3.8 Foreign Citizens

> **[FOUNDER TO FILL]**
>
> List every person who will perform work on this contract and their citizenship. The default expectation is U.S. citizens or U.S. permanent residents only. If any foreign nationals will work on the contract, declare their citizenship + visa status here; AFWERX will assess whether the topic permits foreign-national participation.

## 3.9 Facilities and Equipment

**Computing infrastructure:**

- 1 × Apple M4 MacBook Pro 14" (Mac16,1 / J604) — already in use for Sphragis development; serves as the verified hardware boot target.
- 1 × Linux dev host (Ubuntu) — already in use for m1n1 chainload pipeline, USB serial proxy, QEMU CI runs.
- QEMU virt aarch64 with `-cpu max` configuration — all 85 self-test scripts already exercise this target.
- Cloudflare Workers / Pages for marketing site + capability statement hosting.
- GitHub Actions runners (free tier sufficient for current build matrix).

**Required new acquisitions (within Phase I budget):**

- 1 × Intel NUC reference platform for x86_64 port baseline ($500). Note: x86_64 port is a Phase II deliverable; Phase I uses the existing M4 + QEMU targets and acquires the NUC for Phase II scoping work in Month 6.
- CMVP lab engagement fees ($30-50K) — covered under Phase I budget for lab pre-engagement scoping; full validation runs in Phase II.

**Facilities:**

> **[FOUNDER TO FILL — workspace address; home office is acceptable for SBIR Phase I]**

**No specialized facilities required.** Sphragis development requires only commodity computing hardware. No clean room, no chemical handling, no controlled environment.

## 3.10 Subcontractors and Consultants

**No subcontractors planned for Phase I.** All Phase I work is performed by Sphragis Inc. personnel using publicly available open-source tooling (Rust toolchain, Verus, QEMU, cargo-deny, sigstore, in-toto).

**Phase I budget reserves $5K for consultant engagement** specifically for:
- CMVP lab engagement letter negotiation (Atsec / Leidos / InfoGard initial-scoping calls; consultants such as Corsec Security or Acumen can structure these efficiently)
- Federal contracting compliance counsel (Smith Pachter McWhorter, Holland & Knight, or PilieroMazza for small-business set-aside scoping)

**Phase II will scope larger consultant engagement** for: full CMVP validation, NIAP CCTL relationship if PP fits, USPTO trademark counsel, and SBIR Phase III commercialization counsel.

## 3.11 Prior, Current, or Pending Support

> **[FOUNDER TO FILL]**
>
> List any federal funding the proposer has received or has currently applied for that is related to the proposed work. If this is the proposer's first federal funding application, write: **"No prior, current, or pending federal support for this or related work."**
>
> Required even if the answer is "none" — leaving this section blank is a common SBIR rejection reason.

---

## Supporting evidence (Volume 5 — separate 5-page supplement)

The proposer maintains a complete public evidence chain at https://github.com/kadenlee1107/Sphragis (Apache-2.0 licensed). Reviewers may verify all technical claims directly:

- **M4 hardware boot evidence:** `docs/photos/2026-04-17_first_m4_boot/` — independent reverse-engineering pipeline (Asahi Linux does not yet support M4).
- **14-week mechanical-trace security audit history:** git commit log + commit messages traceable from project root.
- **Threat model:** `docs/THREAT_MODEL.md` (380 lines, attacker classes + assets + mitigations).
- **Security Target:** `docs/SECURITY_TARGET.md` (CC:2022 Part 1 §B-conformant).
- **NIST SP 800-53 Rev. 5.2.0 inheritance matrix:** `docs/NIST_800_53_INHERITANCE.md` (AC + AU + CM + IA families complete).
- **FIPS 140-3 cryptographic module boundary:** `docs/FIPS_140_3_MODULE_BOUNDARY.md`.
- **Operator runbook starter:** `docs/OPERATOR_RUNBOOK.md`.
- **Hardware Compatibility List:** `docs/HARDWARE_COMPATIBILITY.md`.
- **Reproducible build:** SHA-256 `f4b12add37d44d4ae031a0bc5db83739a15c2d54d7d8096e1fcb667ca7e5ad03` from `scripts/check_reproducible_build.sh`.
- **CI gate:** `.github/workflows/license-check.yml` enforcing Apache-2.0 + RustSec advisory scanning on every push.

---

## Volume 3 — Cost Volume (companion workbook, not Volume 2)

> **[FOUNDER TO FILL via the SBIR cost-volume template]**
>
> Indicative Phase I budget allocation to assist Volume 3 preparation:
>
> | Category | Amount | Notes |
> |---|---|---|
> | Principal investigator labor (6 months @ ~50% time) | $40,000 | Loaded cost; founder salary |
> | Consultant fees (CMVP scoping, contracting counsel) | $5,000 | Per §3.10 |
> | CMVP lab pre-engagement letter | $15,000 | Per §3.2 Objective 4 |
> | Intel NUC reference platform | $500 | Per §3.9 |
> | Cloud compute / storage / domain hosting | $1,500 | AWS/Cloudflare incidental |
> | Conference attendance + travel | $5,000 | AFCEA WEST, DARPA Forecast to Industry |
> | Indirect (G&A, fringe, overhead) | $8,000 | ~12% per typical SBIR cost-share |
> | **Total** | **$75,000** | |
>
> Actual figures must be reconciled to SBA-published indirect-rate ceilings for first-time SBIR awardees. Solicitations sometimes specify the maximum cost-share percentage explicitly.

---

## Appendix: Why Sphragis-CDX matters in the 2027-2030 procurement window

The Air Force operates over 250 Cross-Domain Solution endpoints today across PACAF, USAFE, AFCENT, AFAFRICA, AFSOUTH, AFNORTH, and the AOC modernization initiatives. Most are aging hardware from the 2008-2015 procurement era. The 2027-01-01 CNSA 2.0 cliff forces refresh; the 2026-09-21 FIPS 140-2 → 140-3 cliff forces parallel refresh. There is no current commercial procurement vehicle (GSA MAS, ACT 3, SEWP) that lists a CNSA-2.0-native, FIPS-140-3-validated, formally-grounded, memory-safe CDS substrate. Sphragis-CDX targets that vacuum directly.

If awarded, Phase I deliverables position Sphragis to bid into Phase II ($1.25M) at Month 9, secure FIPS 140-3 certification by Month 30, and become eligible for direct ACT 3 task-order subcontract or Phase III sole-source by Month 36. The dual-use commercialization paths (confidential AI inference, automotive Tier 1, HSM vendor licensing) ensure the company survives the typical Phase II-to-Phase III "valley of death" without sole reliance on federal contracts.

The proposer asks for the minimum tier of federal investment ($75K) to demonstrate feasibility on a technology that is otherwise fully self-funded and approaching production readiness. The Air Force gets first look at a substrate that — if Phase I succeeds — will be commercialized regardless of follow-on award, but where AFWERX participation establishes the procurement relationship and the dual-use commitment from the proposer's earliest stage.

---

**End of Volume 2 — Technical Proposal (v0 draft)**

> Page-count target: this draft runs ~12 pages at 11pt single-spaced. AFWERX Volume 2 page limit is 15 pages; the founder has ~3 pages of headroom for figures, key-personnel CV expansion, or additional related-work citations.
