# Sphragis → Government OS: Research Synthesis

**Date:** 2026-05-16
**Purpose:** Phase 1 of the productization plan. Compiles 4 parallel research streams (certification standards, existing certified OSes, paradigm-shifting capabilities, procurement reality) plus a Sphragis current-state inventory into a single foundation document.
**Next:** Phase 2 (requirements spec) consumes this; Phase 3 (gap analysis) compares against the Sphragis inventory in §5; Phase 4 (master implementation plan) sequences the work.

---

## 1. The Certification Landscape (Reality, Not the Brochure)

### 1.1 Common Criteria — what's actually live in 2026

- **Standard**: CC:2022 Revision 1 / ISO/IEC 15408:2022 (5 parts). Methodology is CEM / ISO/IEC 18045:2022.
- **EAL tiers**: cost & timeline by tier:
  - EAL2 typical (~$100-300K, 6-12mo)
  - EAL4+ typical for commercial OS (~$300K-$2.5M, 12-24mo) — historically RHEL, SLES, Windows
  - EAL5+ — smartcards, secure elements (~$1-3M, 18-36mo)
  - EAL6+ — high-grade smartcards, separation kernels in historical configs (~$2-5M, 2-4yr)
  - EAL7 — formally verified (only seL4 has produced the proof artifacts)
- **CCRA mutual recognition tops at EAL2** since the 2014 rewrite. Anything above EAL2 is national-only.
- **EUCC (Feb 2026)** replaces raw EAL labels in Europe with AVA_VAN levels: "Substantial" = AVA_VAN.1-2, "High" = AVA_VAN.3+.

### 1.2 The contradiction we have to resolve: GPOSPP "exists" but isn't being run

Two of the research agents disagreed:

- **Agent A**: NIAP Protection Profile for General Purpose Operating Systems (**GPOSPP v4.4, published 2024-09-06**) exists with ~30 functional requirements across audit, crypto, access control, X.509, ASLR, W^X, stack canaries, trusted update.
- **Agent D**: NIAP has effectively **stopped accepting new GPOS CC evaluations** (Oracle Solaris blog; NIAP guidance) — they don't believe the resulting assurance is worth the spend. The **Separation Kernel Protection Profile (SKPP) was sunset in 2011**.

**Reconciliation**: The PP technically exists; existing certified products (RHEL 8/9, etc.) can re-cert against it; **but for a NEW vendor entering today, the realistic procurement path is NOT a GPOS CC eval.** It is:
1. **FIPS 140-3 validation** of the crypto module (mandatory for any federal crypto deployment)
2. **DoD RMF authorization** (ATO under NIST SP 800-53) tied to a sponsoring agency / program
3. **STIG authored against the GP OS SRG** and accepted by DISA
4. **NSA CSfC capability-package inclusion** if targeting classified-data deployments
5. **NIAP PCL listing** against a more current PP (e.g., MDF PP v3.3 or component PPs) if applicable

### 1.3 FIPS 140-3 (the unavoidable gate)

- **Standard**: FIPS 140-3 incorporates ISO/IEC 19790:2012 and ISO/IEC 24759:2017. NIST SP 800-140 series defines implementation requirements.
- **2026-09-21 cliff**: all FIPS 140-2 certificates move to *Historical* status. After that date, only 140-3 modules acceptable for new federal acquisition.
- **Security Levels**:
  - L1: production-grade components only; no auth required. Software-only crypto.
  - L2: role-based auth, tamper-evident seals.
  - **L3**: identity-based auth, tamper-detection circuitry that zeroises CSPs, EFP/EFT. **Typical gov requirement.**
  - L4: complete envelope protection, environmental attack resistance.
- **Cost (vendor-reported)**:
  - L1 software module: $50-150K, 6-12mo
  - L2: $150-400K, 12-18mo
  - **L3 hardware: $500K-$1.5M+, 18-30mo** — the dominant variable is CMVP queue time.
- **NIST CMVP cost-recovery (2025)**: $16-19K per submission scenario + ALG/OEUP at $2,500 each. Lab fees on top: $75-300K typical. **Total realistic: $150-500K.**
- **Lab options**: Atsec (Austin TX), Leidos, InfoGard/Intertek, Acumen, Lightship.

### 1.4 CNSA 2.0 — the algorithms we MUST ship by 2027-01-01

NSA Commercial National Security Algorithm Suite 2.0, May 2025 issuance:

| Purpose | Algorithm | Parameter | Standard |
|---|---|---|---|
| Symmetric encryption | AES | 256-bit keys | FIPS 197 |
| Hashing | SHA-384 (preferred) or SHA-512 | — | FIPS 180-4 |
| **PQ key establishment** | **ML-KEM** | **ML-KEM-1024** | FIPS 203 |
| **PQ signatures (general)** | **ML-DSA** | **ML-DSA-87** | FIPS 204 |
| **Software/firmware signing** | **LMS** and **XMSS** | per use-case | NIST SP 800-208 |

**No RSA, no ECDH, no ECDSA, no SHA-256 for new NSS deployments.** Migration timetable for OSes: support+prefer by **2027**, exclusive use by **2033**. Hard cliff: all new NSS acquisitions must be CNSA 2.0 compliant by **2027-01-01**.

### 1.5 DoD STIGs — the practical deployment gate

- DISA publishes high-level **SRGs** (Security Requirements Guides). Vendors author product-specific **STIGs** mapping SRG requirements to concrete settings.
- For a new OS: applicable SRG is the **General Purpose Operating System SRG** (current V1R6).
- **Process**: map each SRG requirement to a config/capability on Sphragis → draft STIG in XCCDF/SCAP format → submit to `disa.stig_spt@mail.mil` → validation, risk acceptance, RME digital signing, posting on cyber.mil. **Can take years.**
- **No DoD entity can deploy your OS on a DoDIN until a STIG (or interim MOU) is in place** — regardless of CC or FIPS posture.

### 1.6 NIST SP 800-53 Rev 5 (5.2.0)

- 1,196 controls across 20 families. OS-relevant families: AC, AU, CM, IA, SC, SI, MP, PE, SA, SR, PT.
- FedRAMP **High** baseline = 370 controls; **Moderate** = ~325.
- Sphragis-as-product needs a **control inheritance matrix** stating which controls we fully/partially satisfy so downstream FedRAMP-authorized deployments can inherit them.

### 1.7 TL;DR — what an OS must claim to call itself "gov-grade" in 2026

1. **Crypto module FIPS 140-3 validated at minimum L1** (L3 for hardware-bound key store).
2. **CNSA 2.0 algorithm support shipped by 2027-01-01**, exclusive use by 2033.
3. **NIST SP 800-53 Rev 5.2.0 control-inheritance matrix** published.
4. **DoD STIG authored against the GP OS SRG**, submitted to DISA.
5. **Software supply chain controls**: SBOM, signed builds with CNSA-2.0 algorithms, reproducible builds.
6. **NIAP PCL listing** if a relevant PP applies (MDF, GPCP, component PPs).
7. **EU posture (optional but increasingly required)**: EUCC at "High" assurance starting Feb 2026.
8. **Differentiation tier (not required, but separates us)**: formal-methods assurance equivalent to EAL6/EAL7 — the seL4 model of machine-checked correctness proofs of the security-critical kernel subsystems.

---

## 2. The Competitive Landscape — Who Owns What Slot Today

### 2.1 Separation kernels / high-assurance microkernels

| Product | Certification | TCB | Deployments | Maintainer |
|---|---|---|---|---|
| **INTEGRITY-178B (Green Hills)** | CC EAL6+ "High Robustness" / SKPP (2008, on PowerPC 750CXe — frozen config) | "low tens of thousands of lines" (academic estimates) | F-35, F-22, B-1B, B-52, F-16, C-130J, C-17 | Green Hills |
| **LynxOS-178 (Lynx Software)** | DO-178B/C DAL A; only FAA-approved RSC OS | not disclosed | Rockwell Collins Pro Line Fusion, military avionics | Lynx Software Technologies |
| **PikeOS (SYSGO)** | CC EAL5+ (v5.1.3, 2022); DO-178C DAL-A | not disclosed | Airbus A350 XWB IMA computers, European rail (CENELEC EN 50128), automotive | SYSGO GmbH (Germany) |
| **VxWorks 653 / Helix (Wind River)** | ARINC 653; DO-178C/ED-12C DAL A | not disclosed | Boeing 787, P-8A Poseidon, A330 MRTT, A400M, UH-60V | Wind River (Aptiv) |
| **seL4** | Formally verified end-to-end (functional correctness + info-flow + binary correctness); NOT CC certified | ~8,830 LoC of C; proof corpus ~20× that | NIO SkyOS (mass production cars), HENSOLDT TRENTOS, NASA cFS, DARPA HACMS | seL4 Foundation (Linux Foundation) |

### 2.2 MLS / hardened general-purpose OSes

| Product | Status |
|---|---|
| **Solaris 11.4 + Trusted Extensions** | Oracle extended paid support to **2037**. Mature MAC + labels + RBAC + Zones-as-compartments. Deepest MLS Unix track record. |
| **RHEL + SELinux** | RHEL 7.1 CC EAL4+ against OSPP with SELinux MLS+RBAC; RHEL 8 has separate cert. NIAP-listed, CSfC-eligible. ~43% enterprise Linux server market 2025. |
| **Astra Linux Special Edition** | Russian FSTEC + FSB + MoD certified. PARSEC MAC subsystem. Russian armed forces, FSB, fed civil. |
| **Kylin OS V11** | Chinese gov OS. Linux 6.6 LTS-based with AI subsystem. >16M instances. Document 79 mandates 2027 full domestic substitution. |
| **Qubes OS** | Xen-based per-app VM isolation. Used by Snowden, SecureDrop. No formal cert. |
| **HardenedBSD / OpenBSD** | No US-gov cert of note. OpenSSH provenance. |
| **Tails OS** | Live USB amnesic OS over Tor. Merged with Tor Project ops 2024. |

### 2.3 Emerging / research that matters

- **CHERI capability hardware** — CheriBSD on Morello: pure-capability kernel **March 2026**, pure-capability userspace through Q3 2026, FreeBSD 16.0 mainstream CHERI support targeted Dec 2027. **CHERIoT (Microsoft + lowRISC) shipping in 2026**: SCI Semiconductor ICENI MCU, Wyvern WARP RISC-V chipset.
- **Genode + Sculpt OS** — most credible attempt at productizing seL4-based desktop. Sculpt 25.10 shipped Oct 2025. 2025 work added seL4 on 64-bit Arm + matured sDDF (seL4 device driver framework).
- **Rust OS landscape (2025-2026)**: Rust-for-Linux declared permanent at 2025 Maintainers' Summit. Google reports zero memory-safety bugs from production Android Rust drivers. RedoxOS, Hubris (Oxide, ~2000 LoC), Asterinas (framekernel, TCB only 14% vs Tock 43%), Hermit unikernel. **No gov-grade Rust OS announced.**
- **DARPA programs LITERALLY funding this space (2025-26)**:
  - **PROVERS** (Pipelined Reasoning of Verifiers Enabling Robust Systems) — execution phase
  - **INSPECTA** (Collins Aerospace + CMU + UNSW + U Kansas — formal methods around seL4) — performer awards $3-15M over 3-4yr
  - **Resilient Software Systems Capstone** — Dec 2025 start, March 2026 first red team
  - **TRACTOR** ($5M Jul 2025) — Translate All C to Rust
  - **V-SPELLS** — formal methods on legacy software
  - **DARPA Software Systems Accelerator** — seed-funding for formal-methods tool developers

### 2.4 White spaces (the 4 slots Sphragis can occupy simultaneously)

1. **No Rust-based, formally-verified, separation-kernel-class OS exists.**
2. **No CHERI-native production OS with security certification.**
3. **No modern open-architecture replacement for the SKPP regime** (structural gap since 2011).
4. **The analyst-workstation high-assurance niche** is occupied only by Qubes (no cert) and Sculpt/Genode (research-grade).

**A Rust + bare-metal + microkernel + Apple Silicon target hits all four simultaneously.** That is unusual.

---

## 3. Paradigm-Shifting Differentiators (How to Be "Not in Their Field")

### 3.1 The five claims Sphragis should make

Drawn from the strategic-positioning analysis. Position **C** (recommended) is: **"The first OS designed natively for the 2027-2030 procurement world."** Avoid Position A ("compete with seL4 on proofs" — they have 25 person-years; we don't) and Position B ("Linux but Rust" — what gov buyers don't want).

The five differentiators:

| # | Claim | Why credible | Why nobody else has it |
|---|---|---|---|
| 1 | **Rust microkernel + information-flow proofs on capability/IPC subsystem** | Verus/Kani are real enough in 2026 to verify 5-10K LoC of critical subsystems; gives the non-interference property needed for CDS/MLS arguments | seL4 has functional-correctness proofs but is C; Rust microkernels (Redox, Hubris, Asterinas) don't claim proofs |
| 2 | **CNSA-2.0-native, PQC-only kernel-mediated crypto** | NIST FIPS 203/204/205 ratified; ml-kem and ml-dsa Rust crates exist; we already use them | Linux distros are bolting PQC onto OpenSSL; no OS treats PQC-only as the default with no classical fallback |
| 3 | **Attestation as first-class kernel primitive** — every cave is an attestable identity, Caliptra/SEP/Pluton-rooted | Caliptra 2.1 is open silicon RoT; M4 SEP is documented; HSM-backed operator CAs are standard | Every existing OS treats attestation as bolted-on TPM library, not kernel primitive |
| 4 | **Reproducible, bootstrappable, SLSA-L4 build chain** | Guix proves it's possible (full-source bootstrap on multiple arches, MSR 2026 award); NixOS independently rebuilt minimal ISO | Green Hills/Lynx/Wind River can produce build artifacts but cannot show source-reproducibility |
| 5 | **CHERI-ready architecture + CHERIoT-shipping for embedded variant** | CHERIoT-Ibex production-ready 2026; ARM Morello pure-cap roadmap Mar-Sep 2026; cave isolation model already capability-shaped | Nobody else maps a microkernel cave-isolation model to CHERI compartments |

### 3.2 The defensible category name

**"Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era."**

No incumbent claims that phrase, and none can retrofit it without breaking ABI commitments.

### 3.3 Explicitly NOT to claim

- **Full functional-correctness proofs** — cede to seL4. Their 25-person-year lead is unmatchable.
- **AI-in-kernel** — anti-feature for gov. Non-deterministic, hard to certify, huge new attack surface.
- **QKD integration** — niche. Treat as future option behind a key-plane abstraction; don't lead with it.
- **Linux drop-in compatibility** — commits us to a TCB shape we don't want. Keep the narrow Linux-ABI shim for analyst toolbox compatibility, but don't promise binary compat.

---

## 4. Procurement Reality — How Money Actually Flows

### 4.1 The path is RMF/ATO, not CC

Since NIAP isn't running GPOS CC evaluations for new vendors, the real procurement gate is:

1. **DoD RMF process (NIST SP 800-37 / 800-53)** — sponsored ATO for a specific deployment
2. **FIPS 140-3 validation** of the crypto module
3. **STIG authored and accepted by DISA**
4. **Reference deployments on a sponsored research program** (DARPA, AFRL, ONR, Army Research Lab) — produces evidence used by AOs for ATO
5. **NSA CSfC capability-package inclusion** for classified deployments

### 4.2 The on-ramp: SBIR/STTR

| Phase | Award | Timeline | Notes |
|---|---|---|---|
| Phase I | $75K (SBIR) / $110K (STTR) | 3-12 mo performance; 30-50 day avg award; 80-90% rejection rate | Most accessible federal funding for a small Rust-OS vendor |
| Phase II / Direct-to-Phase-II | up to $1.25M (SBIR) / $1.8M (STTR) | up to 21 mo | Bridge funding |
| Phase III | **No upper limit. Sole-source authority.** | Variable | Where the real money lives; converts Phase II results |
| STRATFI (Air Force) | $3-15M | Bridge | Requires matching private+gov funds |
| TACFI (Air Force) | $375K-$1.7M | Bridge | Lower threshold than STRATFI |

**SBIR/STTR reauthorized through September 30, 2031** (April 2026 reauthorization), so no near-term sunset risk.

### 4.3 DARPA programs we should be pitching RIGHT NOW

- **PROVERS** — formal methods for software assurance. Direct fit.
- **INSPECTA** (Collins Aerospace + CMU + UNSW + U Kansas, on seL4) — we are the Rust alternative they don't have.
- **Resilient Software Systems Capstone** — Dec 2025 start, March 2026 first red team.
- **TRACTOR** — translation tooling we could contribute to / consume.
- **DARPA Software Systems Accelerator** — seed-funding for formal-methods tool developers partnering with primes.

Talk to PMs at the **DARPA Forecast to Industry** (annual fall, DC).

### 4.4 Conferences where vendors meet gov in 2026

| Event | Date | Who's there |
|---|---|---|
| **AFCEA WEST 2026** | Feb 10-12, San Diego | Sea Services + tri-service IT |
| **AFCEA TechNet Cyber** | Jun 2-4, Baltimore | DISA's marquee event |
| **AFCEA TechNet Indo-Pacific** | Oct 27-29, Honolulu | Indo-Pacific Command |
| **AUSA Annual** | Early Oct, DC | Army PEOs |
| **Sea-Air-Space** | Apr, National Harbor | Navy League |
| **DEF CON** | Aug, Las Vegas | NSA/IC recruiters, operators (not contracts) |
| **USENIX Security / NDSS / IEEE S&P** | Various | FFRDC relationships (MITRE, MIT LL, JHU/APL, GTRI) |
| **RSA Conference Government Track** | May, San Francisco | Broader gov audience |
| **DARPA Forecast to Industry** | Annual fall, DC | Best room for DARPA PMs |
| **National Cyber Summit** | Huntsville, AL | Army/AMC, IC |
| **NSWC Crane "Connect to Crane"** | TBD | ~10-min speed-networking with program reps |

### 4.5 Who takes meetings

- **AFRL Information Directorate (Rome, NY)** — runs ACT 3 IDIQ ($950M, awarded to AIS, CNF, Global InfoTek, Invictus, Radiance). Approachable via BAA cycles + TAP-Lab.
- **NSWC Crane** — small-business outreach.
- **DIU (Mountain View / DC / Austin / Boston / Chicago)** — `diu.mil/work-with-us`. OTA awards.
- **DARPA I2O / ITO** — for formal-methods/OS work.
- **In-Q-Tel** — CIA strategic VC. $500K-$3M typical, plus intro letter. **They rarely fund pure OS plays** — frame as "secure compute substrate for IC mission systems."
- **NSA "Adopt-a-Cell"** — opaque; usually accessed through cleared primes or TCG/NCCoE.

### 4.6 Realistic costs (2025-2026)

| Cert | Cost | Time |
|---|---|---|
| Common Criteria at NIAP CCTL | $150K floor → $500K-$2.5M typical | 9-36 mo |
| FIPS 140-3 CMVP | $150-500K all-in | 12-30 mo (CMVP queue is the bottleneck) |
| FedRAMP Moderate (traditional) | $800K-$2M total ($350-650K 3PAO alone) | 12-18 mo |
| FedRAMP Moderate (new "20x" path) | $500K-$1.5M | 3-6 mo (first pilot landed 119 days, Dec 2025) |
| DoD IL4-6 overlay | +$200-500K, +6-12 mo | On top of FedRAMP |
| DO-178C (if targeting avionics) | $500K-$2M per DAL-A line | 18-36 mo |

**Total "all the certs" budget: $3-7M and 24-36 months wall-clock.** Realistic 18-month claims are marketing.

### 4.7 The 3-year financial roadmap (Agent D's bottom line)

Assumptions: 3-5 person US-based team, $500K founder/angel capital at month 0, audit-clean kernel today.

| Phase | Months | Spend | Milestone | Revenue |
|---|---|---|---|---|
| 0. Foundation | 0-3 | $80K | Delaware C-Corp, SAM.gov + DSIP, CAGE + UEI, BIS encryption classification, threat-model docs | — |
| 1. First gov ear | 3-9 | $200K | AFCEA WEST, AFRL briefing, DARPA Forecast. Submit 2-3 SBIR Phase I. Join IWRP/C5 consortium. | Target 1 Phase I award by month 9 |
| 2. Phase I + paid pilot | 9-15 | $250K | Execute Phase I ($75K). USENIX/NDSS paper. CCTL pre-engagement. FIPS 140-3 lab pre-engagement. Prime teaming (Lockheed/Booz Allen/AIS via ACT 3) | $75K Phase I |
| 3. SBIR Phase II + STRATFI | 15-27 | $1.3M | Win Phase II ($1.25M, 21mo). Hire 2 engineers. FIPS 140-3 lab testing in earnest ($150-250K). RMF-aligned ATO package for AFRL test range. Pitch In-Q-Tel. STRATFI bridge prep. | $1.25M Phase II |
| 4. Phase III / first commercial | 27-36 | $1.5M | Phase III sole-source (no upper limit) OR subcontract under ACT 3 OR OTA via IWRP/C5. FIPS 140-3 cert issued mid-period. **First non-research revenue.** | $500K-$5M+ |

**3-year cash: ~$3.5M out, $2-6M in (counting SBIR + Phase III + optional IQT/VC). Net additional capital needed: $1-2M** (friends/family + small angel or defense seed VC).

**First real gov sale: Month 30-36. $2-10M ARR plausible by month 48.**

### 4.8 License posture — critical strategic issue

**Sphragis is currently licensed AGPL-3.0-or-later.** Per the procurement research:
- "AGPL is effectively a no-go for prime integration."
- Primes (Lockheed, Northrop, etc.) will not embed AGPL code into their proprietary product lines.
- Classified deployments where source disclosure is sensitive also have friction with GPL family.

**Gov-friendly licenses**: MIT, Apache 2.0, BSD-2/3-Clause, ISC, MPL-2.0 (with caveats).

**Strategic options for Phase 2 to consider**:
1. Relicense Sphragis to Apache-2.0 or MIT (kills the AGPL friction but loses copyleft protection)
2. Dual-license model: AGPL community + commercial license for primes (RedHat / MongoDB / Sentry pattern)
3. Stay AGPL and accept that the prime channel is closed — only sell direct to gov or via small-business contracts

The user's existing memory says "avoid GPL/AGPL deps in Bat_OS; preserve proprietary-distribution option" — that posture is **contradicted by the current AGPL-3.0-or-later license**. This needs explicit resolution in Phase 2.

### 4.9 Export controls

- US-developed OS governed by **EAR** (not ITAR, unless we build features specifically for USML defense articles).
- Encryption triggers **ECCN 5D002**. License Exception ENC covers most commercial encryption with initial classification notification to BIS.
- **Open-source crypto exempt as "publicly available"** IF we publish + email BIS (`crypt@bis.doc.gov`) and NSA (`web_site@nsa.gov`). **Failing to send the notification is the most common compliance miss.** Should be done at incorporation.

---

## 5. Sphragis Current State (Inventory for Gap Analysis)

### 5.1 Codebase size

**Total: ~99,380 lines of Rust** across `src/`. Breakdown by subsystem:

| Subsystem | LOC | Notes |
|---|---|---|
| `src/caves/` | 28,956 | Largest — process isolation, Linux ABI compat, syscall dispatch, MMU |
| `src/ui/` | 21,826 | 8 apps + shell (11.5K LoC just in shell.rs) |
| `src/net/` | 15,137 | TCP (2.3K), NAT (2.2K), TLS (1.7K), X.509, WireGuard, DNS, ARP, ICMP, cookies, firewall |
| `src/kernel/` | 9,493 | Includes `arch/mod.rs` (3.9K — exception handling, syscall plumbing) |
| `src/drivers/` | 7,856 | Apple Silicon (M4) + virtio |
| `src/ai/` | 5,327 | AGENT app backing — needs careful consideration for gov pivot |
| `src/security/` | 3,235 | Audit chain, capability system |
| `src/crypto/` | 3,005 | Primitives — see §5.4 |
| `src/fs/` | 1,848 | BatFS encrypted filesystem |
| `src/main.rs` | 2,220 | Kernel entry |

**TCB estimate**: most of `src/` runs at EL1 today (microkernel-shaped but not pure microkernel — fs, net, drivers, audit all kernel-mode). **Realistic TCB ~70-80K LoC.** This is **far smaller than Linux (~30M)** but **far larger than seL4 (~10K C + proof corpus)**. For formal verification of a critical subsystem (#1 differentiator), we'd target 5-10K LoC.

### 5.2 Hardware boot targets

- **QEMU virt aarch64** — primary dev target. ~80 self-test scripts exercise it.
- **Apple Silicon M4 MacBook Pro 14" (Mac16,1 / J604 / T8132 "Donan")** — verified boot at hardware level, photos in `docs/photos/2026-04-17_first_m4_boot`. Boot chain: m1n1 chainload → kernel → ADT discovery → PMGR clock-gates → ATC PHY tunable → display + interactive shell.
- **Asahi Linux installer doesn't yet support M4** (community RE in progress) — Sphragis has independent RE pipeline.

M4 driver coverage in `src/drivers/apple/`:
adt, agx (GPU), aic (interrupt controller), ane (Neural Engine), ans/ans_nvme (storage), asc, bcm_wifi, boot_args, dart (IOMMU), dcp (display), dwc3 (USB3), fb_console, rtkit, sio, smc, soc, spi, uart, wdt.

### 5.3 Test infrastructure

**~80 QMP-driven self-test scripts in `scripts/`** covering: audit chain, audit seal, BatFS quotas, beacon, BIBA (integrity model), block devices, boot smoke, busybox baseline, byte-rate E2E, CALIPSO labels, cave policy, cave private isolation, conntrack, CPOL (cave policy), Docker caves, exec translation, flow rate, firewall hardening, GCM, heap guard, HTTPS smoke, kali (red-team), MLS binding, MLS IPC, mount namespaces, multi-NIC, NAT (many variants), NetProbe, OCSP, OTP, PQ demo, PQ interop, redirect, seal, secmark, selftests aggregator, SNI, sys-caves, sys-wg, syscall filter, taint, te (type enforcement), TPI, unified cave demo, vmnet/scapy E2E, wg dispatch/endpoint/handshake/initiator/peer/replay/wire.

Plus boot smoke and cave private selftest run as part of every audit-remediation week.

### 5.4 Crypto inventory (vs CNSA 2.0 requirement)

| CNSA 2.0 Requirement | Sphragis Status |
|---|---|
| AES-256 (FIPS 197) | ✅ `aes.rs` (481 LoC), `aes_xts.rs`, plus RustCrypto `aes` crate |
| AES-GCM-SIV | ✅ via `aes-gcm-siv` crate (BatFS at-rest, week 8 elite-tier) |
| AES-GCM | ✅ `gcm_verified.rs` (390 LoC) |
| AES-XTS | ✅ `aes_xts.rs` + `xts-mode` crate |
| SHA-384 | ✅ `sha384.rs` (121 LoC) |
| SHA-512 | ❌ Not present as standalone file (have SHA-256, 384, 3) |
| ML-KEM-1024 | ⚠️ `ml-kem` crate present; need to confirm parameter set (FIPS 203) |
| ML-DSA-87 | ⚠️ `ml-dsa = 0.1.0-rc.8` crate present; need to confirm parameter set |
| LMS / XMSS | ❌ Not present — needed for CNSA 2.0 software signing |

Additional algorithms present (defense-in-depth, not CNSA-mandated):
SHA-3, BLAKE2s, BLAKE3, ChaCha20-Poly1305, XChaCha20-Poly1305, HOTP, TOTP, X25519 (via dalek per memory), PQ-hybrid (X25519 + ML-KEM), RSA (via `rsa` crate — non-CNSA, for X.509 backward compat).

### 5.5 Existing exploit-mitigation posture (audit-closed elite-tier wins)

From the 14-week audit-remediation:
- **Stack canaries** seeded from RNDR at boot (week 3-4, Mem-H2)
- **PAN (Privileged Access Never)** enabled via SCTLR_EL1.SPAN (week 3-4)
- **BTI enforcement** via SCTLR_EL1.BT0/BT1 (week 9 elite-tier)
- **Per-cave ASIDs** in TTBR0_EL1 via TCR.AS=1 (week 11 elite-tier)
- **HMAC-keyed audit chain** with RNDR-seeded kernel-only key (week 3-4)
- **BatFS AES-256-GCM-SIV** at-rest (week 8 elite-tier; misuse-resistant AEAD)
- **X25519 via dalek** (week 6, removed 267 LoC hand-rolled)
- **TLS handshake order validation** (week 1, Crypto-F1+F2)
- **Per-cave SPKI pinning** + revocation (week 1, Crypto-F3+F4)
- **Per-cave AF_UNIX namespace** (week 12, Cave-H5)
- **set_active access control** (week 13, Cave-H4)
- **psk_overlay retired** (week 14 — eliminated replay-protection gap)

**Cumulative**: 14C + 17H + 23M + 5L + 3 elite-tier closed across 14 audit weeks.

### 5.6 Documentation surface

- Root: `DESIGN.md`, `DESIGN_AI_AGENT.md`, `DESIGN_CAVES.md`, `DESIGN_CAVE_ISOLATION.md`, `DESIGN_CRYPTO.md`, `DESIGN_HTTPS_SYSCALL.md`, `DESIGN_NO_BROWSER.md`, `DESIGN_PACKET_PIPELINE.md`, `DESIGN_SCHEDULER_BLOCK_ON.md`, `DESIGN_SYS_CAVES.md`, `DESIGN_TLS_HARDENING.md`
- `docs/`: `INTERNAL.md`, `RECEIPTS.md`, `WHY.md` + photo evidence
- `docs/superpowers/`: `audits/`, `plans/`, `research/` (this doc), `specs/`
- Private repo (`sphragis-internal`): `M4_GROUND_TRUTH.md`, `SESSION_JOURNAL.md`, `DISCLOSURE_POSTURE.md`, `LICENSING.md`, `ARCHITECTURE.md`, `DEBUGGING_RUNBOOK.md`

### 5.7 Supply chain scripts (provenance / SBOM / reproducibility)

Present in `scripts/`:
- `gen_sbom.py` + `generate_sbom.py` — SBOM generation
- `build_intoto_attestation.py` — in-toto attestation
- `check_reproducible_build.sh` — reproducibility check
- `audit_canaries.sh` — verify exploit-mitigation flags in built binary

**Unclear what's actually running in CI**. The check-script existence ≠ verified-reproducible binaries. Phase 3 gap analysis will probe this.

### 5.8 Build artifact

```
target/aarch64-unknown-none/release/sphragis
7,840,544 bytes, ELF 64-bit LSB executable, ARM aarch64, statically linked, stripped
```

**No installer. No boot ISO. No deployment story for end users beyond `qemu-system-aarch64 -kernel <file>` or m1n1 chainload on M4.**

### 5.9 What's NOT here (the "real OS" gap)

- ❌ **Window manager / multi-app concurrent UI** — 8 apps exist but only one runs at a time with status bar
- ❌ **User accounts** beyond lock-screen passphrase
- ❌ **Settings/preferences app**
- ❌ **Networking config UI** (CLI only)
- ❌ **External display / multi-monitor**
- ❌ **Installer** / boot ISO / first-boot setup
- ❌ **Package management** / update mechanism
- ❌ **POSIX userspace toolbox** — Linux ABI shim exists (`src/caves/linux/`) but narrow; no `vim`, `git`, `python`, `ssh`, `tmux`
- ❌ **Bluetooth/WiFi UX layer** (M4 has `bcm_wifi.rs` driver-level but no userspace networking config)
- ❌ **Multi-hardware support** beyond M4 + QEMU virt aarch64 (no x86_64, no other ARM SoCs, no RISC-V)
- ❌ **Attestation primitives** as kernel API (we have BatFS HMAC seal but no TPM/SEP/Caliptra hookup)
- ❌ **Formal verification harness** (no Verus, Kani, or Coq/Isabelle setup)
- ❌ **CHERI-readiness** (no capability-hardware abstraction)
- ❌ **Reproducible build verified end-to-end** (check script exists, but unclear it passes)
- ❌ **CNSA-mandated LMS/XMSS** for software signing

### 5.10 Sphragis-in-one-paragraph (for competitive comparison)

Sphragis is a security-first, ~100K-LoC bare-metal Rust OS for Apple Silicon, with a microkernel-shaped architecture organized around capability-isolated processes ("caves"). It boots on real M4 hardware via independent reverse engineering (Asahi doesn't support M4 yet), runs an encrypted filesystem (BatFS, AES-256-GCM-SIV at rest), an HMAC-chained audit ring, a TLS+X.509 stack with PQ-hybrid key exchange, and 8 in-OS apps under a lock-screen-gated single-app UX. 14 weeks of mechanical-trace audit remediation have closed 14 critical and 17 high-severity findings plus three elite-tier hardening items (BatFS GCM-SIV, BTI enforcement, per-cave ASIDs). The codebase is currently AGPL-3.0-or-later, has no installer, no formal-verification harness, no kernel-mediated attestation, no CHERI plumbing, no LMS/XMSS, and no multi-hardware target beyond M4 and QEMU.

---

## 6. Reconciling Research → Strategic Position

Putting the four research streams together produces a coherent strategic picture:

- **§1 says**: the formal gov-cert path (CC) is not the modern entry point. FIPS 140-3 + STIG + ATO via RMF, with CNSA 2.0 by 2027, is the actual path.
- **§2 says**: there are 4 white spaces simultaneously occupiable by a Rust microkernel on capability-friendly hardware. SKPP is dead since 2011 leaving a structural gap. seL4 is the only verified competitor and uses C.
- **§3 says**: position as "Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era." Pick 5 differentiators (Rust microkernel with info-flow proofs / CNSA-2.0-native / attestation-as-primitive / SLSA-L4 builds / CHERI-ready). Explicitly cede full functional-correctness proofs to seL4 and avoid AI-in-kernel.
- **§4 says**: realistic entry is SBIR Phase I → II → III, leveraging DARPA PROVERS/INSPECTA/RSSC funding streams. 3-year run costs ~$3.5M and yields first commercial revenue at month 30-36. License must move off AGPL for prime-channel viability.
- **§5 says**: Sphragis has 99K LoC, real M4 boot, strong crypto + audit foundation, but lacks installer/window-manager/attestation-primitive/formal-verification-harness/multi-hardware/LMS-XMSS/non-AGPL-license.

The phase-2 requirements spec consumes this directly. Each gov requirement (from §1) and each strategic differentiator (from §3) becomes a numbered requirement; the gap analysis (Phase 3) maps each to "have / partial / missing" against §5.

---

## 7. Open questions surfaced by the research (resolve in Phase 2)

1. **License resolution**: AGPL → Apache-2.0 / MIT / dual-license? (Affects prime-channel viability — §4.8)
2. **Hardware target strategy**: stay M4-only as the demo? Add x86_64 (where DoD lives)? CHERIoT-Ibex for embedded variant?
3. **Verification scope**: which 5-10K LoC subsystem gets the Verus/Kani proof effort? (Capability dispatcher? IPC? Scheduler invariants?)
4. **AGENT app**: is the AI/chat app shippable in a gov product, or is it stripped from the gov build? (Position C explicitly anti-AI-in-kernel.)
5. **First DARPA pitch target**: PROVERS, INSPECTA, RSSC, or a future open BAA? Each implies a different framing.
6. **Founding company structure**: when to incorporate (gov-vendor onramp), where (Delaware C-Corp is standard), what cap-table shape (founder + angel vs SBIR-only vs defense-VC).
7. **CSfC vs CC strategy**: focus on CSfC capability-package inclusion (modern path) or invest in CC against an existing PP for credibility (slow, expensive)?
8. **NIAP MDF PP vs GPOSPP vs new component PP**: which PP gives the best fit for Sphragis's small-TCB, capability-isolated, fixed-app posture?

---

## Sources (curated; see individual research reports for full citations)

**Standards:**
- [NIAP Protection Profiles](https://www.niap-ccevs.org/protectionprofiles)
- [GPOSPP v4.4 source](https://github.com/commoncriteria/operatingsystem)
- [FIPS 140-3 PUB](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.140-3.pdf)
- [NIST CMVP](https://csrc.nist.gov/projects/cryptographic-module-validation-program)
- [CNSA 2.0 algorithms (NSA, May 2025)](https://media.defense.gov/2025/May/30/2003728741/-1/-1/0/CSA_CNSA_2.0_ALGORITHMS.PDF)
- [NIST SP 800-53 Rev 5](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final)
- [DISA STIGs](https://www.cyber.mil/stigs/)

**Existing OSes:**
- [INTEGRITY-178 Security Target](https://www.commoncriteriaportal.org/files/epfiles/st_vid10362-st.pdf)
- [seL4 Verification](https://sel4.systems/Verification/)
- [seL4 deployments](https://sel4.systems/use.html) (NIO SkyOS, HENSOLDT TRENTOS, NASA cFS)
- [PikeOS EAL5+ (2022)](https://www.eejournal.com/industry_news/sysgo-pikeos-achieves-common-criteria-cc-level-eal5-security-certification/)

**Paradigm-shift:**
- [CHERIoT-Ibex (Microsoft + lowRISC)](https://techcommunity.microsoft.com/blog/azureinfrastructureblog/cheriot-ibex-closing-the-door-on-memory-safety-vulnerabilities-with-hardware-enf/4517904)
- [CheriBSD Morello roadmap](https://www.cheribsd.org/)
- [Caliptra 2.1](https://techcommunity.microsoft.com/blog/azureinfrastructureblog/caliptra-2-1-an-open-source-silicon-root-of-trust-with-enhanced-protection-of-da/4460758)
- [Rust for Linux permanent](https://www.phoronix.com/news/Rust-To-Stay-Linux-Kernel)
- [Verus](https://github.com/verus-lang/verus)
- [Kani](https://aws.amazon.com/blogs/opensource/verify-the-safety-of-the-rust-standard-library/)
- [SLSA v1.1](https://slsa.dev/spec/v1.1/faq)
- [Reproducible Builds project](https://reproducible-builds.org/)

**Procurement:**
- [DARPA PROVERS / INSPECTA](https://idstch.com/cyber/darpa-provers-advancing-formal-methods-for-software-assurance-in-critical-systems/)
- [DARPA Resilient Software Systems Capstone](https://www.darpa.mil/research/programs/resilient-software-systems-capstone)
- [GSA MAS IT category](https://www.gsa.gov/technology/it-contract-vehicles-and-purchasing-programs/multiple-award-schedule-it)
- [NSA CSfC](https://www.nsa.gov/Resources/Commercial-Solutions-for-Classified-Program/)
- [AFWERX SBIR 2025](https://www.akelaconsultants.com/post/afwerx-sbir-2025)
- [NITAAC cancels CIO-SP4 (Nextgov, Feb 2026)](https://www.nextgov.com/modernization/2026/02/nitaac-finally-pulls-plug-cio-sp4/411120/)
- [NIAP recommends no further OS CC evaluations (Oracle blog)](https://blogs.oracle.com/solaris/niap-recommends-no-further-common-criteria-evaluation-of-operating-systems-dbms-v2)
- [FedRAMP 20x Phase One](https://www.fedramp.gov/20x/phase-one/)
- [In-Q-Tel 2026 Investor Profile](https://tracxn.com/d/venture-capital/inqtel/__ncyOatblZk9suUG1SKwkii9UoqRqgFQIbGfAjkGRY-M)
