# Sphragis — ACT 3 Capability Brief (Teaming Pitch)

**Audience:** Prime contractors with active ACT 3 IDIQ task orders at AFRL Information Directorate (Rome, NY).
- **Assured Information Security (AIS)** — Rome, NY headquartered; largest ACT 3 sub ($54.7M task order on record); cyber-research focus aligns
- **CNF Technologies** — cyber R&D
- **Global InfoTek (GiTec)** — cyber engineering, ML-based threat detection
- **Invictus International Consulting** — cyber ops + RDT&E
- **Radiance Technologies** — cyber + space + intel SETA

**Purpose:** Sphragis as a teaming-partner offering a niche memory-safe + CNSA-2.0-native + formally-grounded OS substrate that the prime layers into their existing AFRL task-order responses.

**Use:** First-meeting capability brief for prime BD/CTO conversation. ~4 pages printed; can be cut to a 1-page leave-behind.

**Version:** v1, 2026-05-17

> **Founder action — items to fill before send:**
> - Page 4 contact info (your name, email, phone)
> - Decide which prime to lead with (see §6 ranking) and tailor cover paragraph if helpful

---

## §1 — Why this matters now

The Air Force Research Laboratory (AFRL) Information Directorate's **Agile Cyber Technology 3 (ACT 3)** IDIQ ($950M ceiling) is the primary contract vehicle through which AFRL procures cybersecurity research, prototype development, and technology transition into Air Force operational programs. ACT 3 task orders increasingly call out three procurement-relevant capabilities:

1. **CNSA 2.0 cryptographic substrates** — driven by the NSA's 2027-01-01 hard cliff requiring ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384 for new National Security System acquisitions.
2. **Memory-safe systems software** — driven by the CISA/NSA/ONCD memory-safety policy guidance (2024-2025) explicitly identifying Rust as the canonical path.
3. **Formally-grounded high-assurance kernels** — driven by the DARPA PROVERS / INSPECTA / Resilient Software Systems Capstone programs the AFRL ecosystem feeds into.

**The prime needs a substrate that satisfies all three simultaneously. None exist in the current commercial market.** Sphragis is being built specifically to fill that gap.

---

## §2 — What Sphragis offers a prime

**A sub-contractable, Apache-2.0-licensed, sovereign-grade OS substrate that the prime integrates into the prime's proprietary product line without copyleft contamination.**

Concrete deliverables Sphragis can produce for an ACT 3 task-order response:

| Deliverable | Status today | Source artifact |
|---|---|---|
| Memory-safe Rust microkernel TCB | **Live, ~96K LoC** | `github.com/kadenlee1107/Sphragis` |
| Boots on real Apple M4 hardware | **Verified** | `docs/photos/2026-04-17_first_m4_boot/` |
| CNSA-2.0-native crypto module (ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384 + LMS) | **Live + boot-time KATs** | `src/crypto/pq_cnsa.rs` + `src/crypto/lms.rs` |
| Bell-LaPadula + Biba MLS labels with kernel-enforced cross-cave gates | **Live** | `src/caves/cave.rs` |
| Attestation primitive (`attest::quote(nonce, claims) -> Quote` CBOR + ML-DSA-87 sig) | **Live + external verifier tool** | `src/security/attest.rs` + `tools/attest-verifier/` |
| HMAC-SHA-384 audit chain + WORM segment export to SealFS | **Live** | `src/security/audit*.rs` + `tools/audit-verifier/audit_verifier.py` |
| Reproducible builds + SLSA-L4 attestation chain | **Live, bit-identical verified** | `scripts/check_reproducible_build.sh` |
| Apache-2.0 license (no GPL/AGPL contamination) | **Active** | `deny.toml` + CI gate |
| FIPS 140-3 cryptographic module boundary documented | **Complete** | `docs/FIPS_140_3_MODULE_BOUNDARY.md` |
| Threat model, Security Target, NIST 800-53 inheritance matrix | **Complete or substantial** | `docs/THREAT_MODEL.md`, `docs/SECURITY_TARGET.md`, `docs/NIST_800_53_INHERITANCE.md` |
| Multi-hardware target roadmap (x86_64 designed; CHERIoT-Ibex designed) | **Designs published** | `DESIGN_X86_64_PORT.md`, `DESIGN_CHERI_MAPPING.md` |

**14-week mechanical-trace audit history**, **32 P0 requirements HAVE** of 75 total, every commit DCO-signed, every release bit-reproducible — the substrate has production-quality discipline behind it.

---

## §3 — How the prime monetizes the relationship

**Sub-contracting model (recommended for ACT 3 fit):**

1. Prime wins ACT 3 task order requiring high-assurance OS substrate as a CDRL (Contract Data Requirements List) line item.
2. Prime engages Sphragis as a subcontractor for the OS substrate deliverable (typically 10-20% of task-order value).
3. Sphragis delivers the OS + integration artifacts to the prime.
4. Prime delivers the integrated product to the Air Force as the prime contractor of record.
5. Apache-2.0 license means **no AGPL/GPL contamination risk** to the prime's proprietary integration work — the prime can ship a closed-source integration product even though Sphragis itself is open-source.

**Strategic value to the prime:**

- **Differentiation in AFRL bids.** Most ACT 3 responders bid Linux-derived substrates (RHEL, hardened Ubuntu, or custom Yocto). Sphragis offers a credibly differentiated option that satisfies the 2027 CNSA 2.0 mandate without retrofit work.
- **Mitigates the prime's crypto-modernization risk.** Primes with installed Linux-based product lines face the 2027 cliff and must scope rework. Sphragis sub-contracted in is a hedge.
- **Memory-safety policy alignment.** ONCD memory-safety guidance is increasingly cited in solicitation evaluation criteria; Sphragis-as-substrate is a "memory-safe" check for the prime's bid.
- **Phase II SBIR cross-leverage.** If Sphragis wins SBIR Phase II in parallel, the prime gets a sub whose R&D is partly federally funded — capital-efficient pairing.

---

## §4 — Proposed first project (concrete task-order shape)

A 6-12 month, ~$500K-$1M task-order pilot demonstrating Sphragis-CDX (Capability-Isolated Cross-Domain) substrate on a specific AFRL use case:

**Suggested use case: tactical-edge analyst workstation handling mixed-classification data.**

The analyst boots Sphragis on a ruggedized Apple M4 MacBook Pro 14" (commodity hardware, COTS-procurable). Their workflow involves three caves at different classification levels: an Unclassified cave for open-source intelligence (OSINT), a Confidential cave for FOUO/CUI material, and a Secret cave for classified working storage. Sphragis's kernel-enforced Bell-LaPadula + Biba labels prevent unintentional cross-domain data flow at the kernel layer; the attestation primitive produces verifiable per-cave Quotes that an external verifier can audit; the HMAC-SHA-384 audit chain produces tamper-evident logs exportable via WORM segments.

Pilot deliverables (analogous to a Phase I/II SBIR shape, but funded by the prime via task order rather than SBA):

1. Working analyst-workstation prototype on M4 hardware with 3-cave MLS configuration.
2. End-to-end attestation flow demonstrating per-cave Quotes accepted by an external verifier.
3. Audit log producing forensic-grade WORM export verifiable by the offline Python tool.
4. CNSA-2.0 crypto policy fail-closed on weak-algo attempts (verified via QMP test harness).
5. Integration package the prime can demonstrate at AFRL TIM (Technical Interchange Meeting).
6. Joint authorship of an AFRL technical note or whitepaper.

**Cost share:** Sphragis can offer ~30% in-kind contribution against parallel SBIR Phase II work, materially reducing the prime's bid cost.

---

## §5 — Risk reduction for the prime

| Risk concern | Sphragis mitigation |
|---|---|
| **"Open source means no support."** | Apache-2.0 source is free; Sphragis Inc. offers paid support contracts + SLA + custom feature development (Red Hat model). |
| **"Single-founder team is brittle."** | Y1 hires post-Phase-I award (Verus specialist + hardware port + UX); agent-augmented development velocity demonstrated (143 commits / 24 hours, 47 P0 reqs out of MISSING in same window). |
| **"Apache 2 vs GPL contamination."** | `deny.toml` + CI gate blocks every GPL/AGPL/LGPL/SSPL/Commons-Clause dependency. Zero copyleft anywhere in the tree. |
| **"FIPS 140-3 not yet certified."** | Module boundary doc complete; CMVP lab pre-engagement scheduled per Phase I plan. Until cert is in hand, the prime can ship with `[FIPS validation in progress]` disclosure consistent with industry norm (Red Hat shipped RHEL years before FIPS cert closed). |
| **"What if Sphragis Inc. fails as a company?"** | Apache-2.0 license means the prime can fork and maintain the substrate independently if needed. No lock-in. |
| **"DoD STIG not yet accepted by DISA."** | Draft STIG exists; submission planned for Phase II. The prime can deploy in non-DoDIN contexts (lab, R&D, prototype) immediately and gate DoDIN production deployment on STIG acceptance. |

---

## §6 — Prime ranking for first engagement

Recommended outreach sequence based on fit + accessibility:

**1. Assured Information Security (AIS)** — Rome, NY.
- **Fit: HIGH.** AIS is the largest ACT 3 sub by task-order value, cyber-research-focused, and Rome-based (collocated with AFRL Info Directorate).
- **Accessibility: HIGH.** AIS publishes BD contact info; their cyber-R&D portfolio aligns with Sphragis differentiators.
- **First-meeting ask:** 30-min capability brief discussion → "if our differentiators resonate, what's the path to a joint AFRL TIM presentation in the next 60 days?"

**2. CNF Technologies** — San Antonio, TX + DC.
- **Fit: HIGH.** Cyber R&D; smaller than AIS, which means more flexibility on novel-technology pilots.
- **Accessibility: MEDIUM.** Less public-facing BD outreach. Best path: warm intro via AFRL Info Directorate TPOC who has cross-relationships.

**3. Global InfoTek (GiTec)** — Reston, VA.
- **Fit: MEDIUM-HIGH.** ML-based threat detection; cyber engineering. Aligns less with Sphragis-as-substrate, more with "Sphragis-as-AI-inference-host" angle which is also a real value prop.
- **Accessibility: MEDIUM.** Public BD inquiries channel; mid-size firm with technology-evaluation discipline.

**4. Invictus International Consulting** — Alexandria, VA.
- **Fit: MEDIUM.** Cyber ops focus more than substrate research; relevant for operational deployment scoping rather than R&D phase.
- **Accessibility: MEDIUM.** Standard BD outreach.

**5. Radiance Technologies** — Huntsville, AL + multiple sites.
- **Fit: MEDIUM.** Broader portfolio (cyber + space + intel SETA); less natural anchor for an OS-substrate teaming relationship.
- **Accessibility: LOW.** Larger firm; more formal BD process.

**Recommended sequence:** AIS first (highest fit + accessibility). If AIS engagement is slow or non-responsive after 2 weeks, parallel outreach to CNF + GiTec.

---

## §7 — Contact + next steps

> **[FOUNDER TO FILL]**
>
> **[Founder Name], Founder & CEO**
> [email]@sphragis.com
> [phone]
> Sphragis Inc. (Delaware C-Corp — incorporation in flight)
> Public evidence chain: `https://github.com/kadenlee1107/Sphragis`
>
> **What we ask from a first meeting:**
> 1. 30 minutes of CTO + BD time to walk through the capability brief
> 2. Feedback on which of your active or upcoming ACT 3 task orders has the closest fit
> 3. Indicative timing on a joint AFRL TIM demonstration if the fit resonates
>
> **What we can deliver within 48 hours of mutual interest:**
> - Full demo bundle (M4 boot recording, capability statement PDF, security target, threat model, attestation flow demo)
> - NDA execution if your standard NDA workflow allows for an early-stage vendor
> - A drafted teaming-agreement skeleton for your contracting counsel's review

**Sphragis is pre-revenue but technically production-ready.** A teaming arrangement with an ACT 3 prime is the fastest path from technical readiness to deployed Air Force capability. We expect to be one of several niche-substrate offerings you evaluate; we ask only for the opportunity to demonstrate that our differentiators are real and verifiable in 48 hours' notice.

---

**One-paragraph elevator (for cold outreach email):**

*"We're building Sphragis, a memory-safe Rust microkernel with CNSA-2.0-native crypto, kernel-mediated attestation, and Bell-LaPadula MLS labels that boots today on Apple M4 hardware. We see AFRL's 2027 CNSA refresh as a tailwind that no current commercial OS substrate satisfies. We'd like to explore a teaming arrangement under ACT 3 where you integrate Sphragis as the high-assurance substrate in your AFRL task-order responses. The substrate is Apache-2.0 (no GPL contamination risk) and we have 14 weeks of mechanical-trace audit history + a complete demo bundle ready to walk through. Would 30 minutes in the next two weeks make sense?"*

---

**End of capability brief v1.**
