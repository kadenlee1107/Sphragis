# Sphragis — Defense Seed VC Pitch Deck v1

**Audience:** Defense-focused seed VCs (Shield Capital, Lux Capital, a16z American Dynamism, 8VC, Razor's Edge Ventures, Booz Allen Ventures, Lockheed Martin Ventures). Strategic / IC capital (In-Q-Tel) as secondary audience after Phase I award.
**Stage:** Seed / Series Seed-Extension. Typical check $1-3M.
**Use:** Source material for slide deck (Keynote/Pixelplus); also valid as a written investor memo for pre-meeting send-ahead.
**Version:** v1, 2026-05-17

> **Founder action — items to fill before send:**
> - Slide 1 founder photo + name + email + phone
> - Slide 11 cap table (founder/angel split if any)
> - Slide 15 "Ask" line — adjust based on round size you want to raise
> - Slide 16 contact info

---

## Slide 1 — Title

**SPHRAGIS**

*Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era.*

[Founder Name] · Founder & CEO
[email] · [phone]
2026-05-17

> **Speaker note:** Open with the boot-evidence photo (`docs/photos/2026-04-17_first_m4_boot/`). "This is Sphragis booting on an Apple M4 in April 2026. We built our own reverse-engineering pipeline because Asahi Linux doesn't support M4 yet. That's where the technical credibility starts."

---

## Slide 2 — The market window

**Two hard procurement cliffs hit U.S. government in 18 months.**

| Date | Cliff | Impact |
|---|---|---|
| **2026-09-21** | FIPS 140-2 → 140-3 | All FIPS 140-2 certs become "Historical." New federal crypto acquisitions must use FIPS 140-3. |
| **2027-01-01** | NSA CNSA 2.0 hard cliff | All new National Security System acquisitions must use ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384. RSA, ECDSA, and SHA-256 forbidden for new deployments. |

**Today's installed base** — INTEGRITY-178B, VxWorks 653, LynxOS-178, RHEL — **does not meet this bar.** Vendors are scrambling to retrofit; some can't.

**Sphragis is CNSA-2.0-native from day one.** Not retrofitted — designed for the 2027-2030 procurement window.

> **Speaker note:** This is the single most important slide. Investors who understand this slide understand the timing thesis. Investors who don't will fixate on "why another OS." Don't move past this slide until they nod.

---

## Slide 3 — The category

Sphragis defines a new category:

**"Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era."**

| What it isn't | What it is |
|---|---|
| Not Linux-with-hardening | A 96K-line memory-safe Rust microkernel, ~300× smaller TCB than Linux |
| Not seL4 (formally-verified C) | Formally-grounded but Rust-throughout; cedes whole-kernel proofs to seL4, claims info-flow non-interference on critical subsystems |
| Not closed (Green Hills / Wind River) | Apache-2.0, every line of kernel + drivers source-available |
| Not a research project | Boots on real Apple M4 hardware today; 14-week mechanical-trace security audit history |

We compete in a category that **doesn't have a current incumbent.** The closest commercial product — INTEGRITY-178B from Green Hills — was certified in 2008 on PowerPC and is frozen-config closed-source. The closest open-source comparable — seL4 — is a kernel substrate, not a deployable OS.

---

## Slide 4 — The 5 differentiators

| # | Claim | Artifact backing it |
|---|---|---|
| **1** | **Rust microkernel + information-flow proofs** on capability dispatcher + IPC | Verus harness + 2 written proof specs (`verification/`) |
| **2** | **CNSA-2.0-native, PQC-only crypto** — ML-KEM-1024, ML-DSA-87, AES-256, SHA-384 default in gov build | `src/crypto/pq_cnsa.rs` + boot-time KATs + `gov-strict` build profile |
| **3** | **Attestation as a first-class kernel primitive** — every cave is an attestable identity | `src/security/attest.rs` (598 LoC) + `attest::quote()` API + external verifier in `tools/attest-verifier/` |
| **4** | **Reproducible, bootstrappable, SLSA-L4 build chain** | Bit-identical SHA-256 `f4b12add...e5ad03` verified from two independent build passes |
| **5** | **CHERI-ready architecture** — caves map 1:1 to CHERI compartments | `DESIGN_CHERI_MAPPING.md` + CHERIoT-Ibex porting roadmap |

No incumbent claims all five simultaneously. **The category is defensible.**

---

## Slide 5 — Why now? Three converging forces

**1. Algorithm cliff (CNSA 2.0).** Forces every federal crypto deployment refresh by 2027.

**2. Memory safety cliff.** CISA + NSA + ONCD pushed memory-safe-languages mandate in 2024-2025. Rust is the policy-approved path. Linux is C; rewriting Linux to Rust is a 10-year project that's barely started. **Greenfield Rust OS is the only realistic option for new gov procurement.**

**3. Capability hardware cliff.** ARM Morello (CHERI) pure-cap roadmap is March-September 2026. CHERIoT-Ibex (Microsoft + lowRISC + SCI Semiconductor) shipping silicon 2026. The hardware exists; no production OS targets it yet. First-mover advantage available.

**These three forces converge in the same 18-month window.** Sphragis is positioned for all three.

---

## Slide 6 — Strategic gap proof

NIAP **stopped accepting new General-Purpose OS Common Criteria evaluations** in the early 2020s (per Oracle Solaris blog + NIAP guidance). The Separation Kernel Protection Profile was sunset in 2011.

**That means:** there is currently **no commercial procurement vehicle** (GSA MAS, ACT 3, SEWP) that lists a CNSA-2.0-native, FIPS-140-3-validated, formally-grounded, memory-safe OS substrate. The Air Force operates 250+ Cross-Domain Solution endpoints that need refresh by 2027; the procurement vehicle for that refresh doesn't exist yet.

**Sphragis is being built to be the substrate that fills that procurement vacuum.**

---

## Slide 7 — Current status (proof of progress)

**24-hour productization push (2026-05-16 → 2026-05-17):** 143 commits, 47 P0 requirements moved from MISSING to HAVE or PARTIAL. Demo bundle assembled.

| Metric | Day-1 start | Today |
|---|---|---|
| P0 requirements HAVE | 5 / 75 | **28 / 75** |
| CNSA-2.0 crypto module | None | **Live** (ML-KEM-1024 + ML-DSA-87 + boot KATs) |
| Attestation primitive | None | **Live** (`attest::quote()` + external verifier) |
| Audit chain | HMAC-SHA-256 | **HMAC-SHA-384** + WORM export + offline verifier |
| Reproducible build | Not verified | **Bit-identical SHA-256 verified** |
| Threat model + Security Target docs | None | **Both complete** |
| Demo bundle ready for gov-buyer | No | **Yes** |

This was achieved by one founder + autonomous-agent execution per a documented 36-month master plan. The velocity is sustainable because the plan is bounded — every remaining requirement either has a sub-project plan or is explicit founder paperwork.

> **Speaker note:** Pull up the actual GitHub commit log on the laptop. Velocity is more credible than slides.

---

## Slide 8 — Architecture (one slide)

```
┌─────────────────────────────────────────────────────────────────┐
│ User-mode apps (in caves)                                       │
│   CAVES  FILES  NET  SECURITY  SHELL  EDITOR  COMMS             │
├─────────────────────────────────────────────────────────────────┤
│ Kernel TCB (~96K LoC Rust, gov-strict build profile)            │
│                                                                 │
│   ┌─── Cave isolation ────┐   ┌─── Crypto (CNSA 2.0) ────┐      │
│   │ per-cave ASIDs        │   │ ML-KEM-1024 (FIPS 203)    │     │
│   │ MLS BLP+Biba labels   │   │ ML-DSA-87 (FIPS 204)      │     │
│   │ per-cave IPC ns       │   │ AES-256-GCM-SIV (SealFS)  │     │
│   │ cave-policy syscall   │   │ SHA-384 (audit HMAC)      │     │
│   │   gate (every syscall)│   │ LMS (kernel signing)      │     │
│   └───────────────────────┘   └───────────────────────────┘     │
│                                                                 │
│   ┌─── Attestation ───────┐   ┌─── Audit ─────────────────┐     │
│   │ attest::quote(nonce)  │   │ HMAC-SHA-384 chain        │     │
│   │ CaveIdentity registry │   │ WORM export to SealFS     │     │
│   │ Caliptra/SEP rooted   │   │ offline Python verifier   │     │
│   │ HSM-backed operator CA│   │ cap-aware reads           │     │
│   └───────────────────────┘   └───────────────────────────┘     │
│                                                                 │
│   ┌─── Network (TLS 1.3 + PQ hybrid + WireGuard) ───────────┐   │
│   │ X25519MLKEM768 hybrid for TLS interop                   │   │
│   │ CALIPSO + CIPSO MLS-labeled IPv6 / IPv4                 │   │
│   └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ Hardware abstraction                                            │
│   Apple Silicon M4 (verified boot)  |  QEMU virt aarch64        │
│   x86_64 (designed)  |  CHERIoT-Ibex (designed)                 │
│                                                                 │
│   ARMv8.5+ FEAT_RNG | PAN | BTI | per-cave ASID | RNDR canaries │
└─────────────────────────────────────────────────────────────────┘
```

**Live on Apple M4 today.** **Designed for x86_64 + CHERIoT next.**

---

## Slide 9 — Markets (TAM by vehicle)

| Market | Sales cycle | Deal size | Approximate TAM |
|---|---|---|---|
| **US gov direct (DoD/IC)** | 3-5 yr | $5M-$50M/program | $2B+ (replacement of installed CDS + tactical-edge base) |
| **Allied gov (Five Eyes + EU)** | 2-4 yr | $1M-$10M/program | $500M-$1B |
| **Defense prime OEM licensing** | 1-2 yr | $1M-$10M perpetual + royalty | $500M (Lockheed/Northrop/RTX/BAE addressable embedded share) |
| **Confidential AI inference** (Anthropic/OpenAI/Meta) | 6-9 mo | $5M-$20M ARR | **$3B+ projected by 2028** (Anjuna + Edgeless + Fortanix combined indicative) |
| **Automotive Tier 1** (Bosch/Continental/Aptiv) | 1-2 yr | $1M-$5M + per-device royalty | $2B (ISO 26262 ASIL-D market; seL4/SkyOS proves demand exists) |
| **HSM + crypto custody + HFT** | 6-12 mo | $500K-$5M | $300M |

**Total addressable revenue: $8B+ over 5 years.** Realistic capture in Y3-5 at $20-50M ARR per the master plan's financial model.

> **Speaker note:** Investors will probe market sizing. The honest answer is that the federal CDS / tactical-edge market alone is ~$2B and almost-entirely served by legacy products that fail the 2027 CNSA cliff. We don't need to win all 4 commercial markets — we need to win one and the federal track in parallel.

---

## Slide 10 — Go-to-market

**Two parallel motions:**

**Federal (4-tier funnel):**
1. SBIR Phase I ($75K, 6mo) — DoD SBIR 26.1 + AFWERX Open + DARPA SBIR. **Ready to file pending entity incorporation.**
2. Phase II ($1.25M, 21mo) — converts Phase I.
3. DARPA PROVERS / RSSC engagement ($3-15M per performer, 3-4yr).
4. Phase III sole-source OR ACT 3 task-order subcontract (no upper limit).

**Commercial (Plan B if federal stalls):**
1. Confidential AI inference design partner (3-9mo sales cycle, no cert gating).
2. Defense prime OEM teaming via ACT 3 IDIQ subs (AIS, CNF, Global InfoTek, Invictus, Radiance).
3. Automotive Tier 1 follow-on once first commercial deal lands.

**The two motions reinforce.** Each Phase I dollar produces evidence usable in commercial sales; each commercial dollar produces revenue that funds patient federal sales.

---

## Slide 11 — Team

> **[FOUNDER TO FILL]**
>
> **Founder & CEO: [Kaden Lee]**
> - 14 weeks shipping mechanical-trace security remediation on Sphragis
> - Independently reverse-engineered Apple M4 boot path (Asahi doesn't support M4 yet)
> - Boots Sphragis on real M4 hardware April 2026
> - [Education, prior roles, prior projects]
>
> **Open Y1 hires** (post Phase I award):
> - Engineer 1: Verus / formal verification specialist
> - Engineer 2: x86_64 + CHERIoT hardware port + driver work
> - Engineer 3: UX productization (window manager, installer)
>
> **Advisors** (sought):
> - 1× Federal contracting / SBIR strategy
> - 1× FIPS 140-3 / NIAP / CCTL relationship
> - 1× Defense prime OEM relationship (ex-Lockheed / Northrop / BAE BD)

---

## Slide 12 — Competition

| Competitor | Tech | Cert | Source | Memory safe | CNSA 2.0 | CHERI ready | Status |
|---|---|---|---|---|---|---|---|
| **INTEGRITY-178B** (Green Hills) | C, separation kernel | CC EAL6+ (2008, frozen) | Closed | No | No | No | Installed base; can't retrofit easily |
| **seL4** | C, formal proof of whole kernel | Not CC certified | Open (BSD) | No | No | No | Substrate only, not deployable OS |
| **PikeOS** (SYSGO) | C++, partitioning | CC EAL5+ (v5.1.3 2022) | Closed | No | No | No | Avionics-focused |
| **VxWorks 653** (Wind River) | C, ARINC 653 | DO-178C DAL A | Closed | No | No | No | Aging |
| **LynxOS-178** | C | DO-178C DAL A | Closed | No | No | No | Aging |
| **RHEL + SELinux** | C, hardened Linux | CC EAL4+ via OSPP | Open + commercial | No (C) | Partial (OpenSSL retrofit) | No | TCB too large for proofs |
| **Qubes OS** | Xen + VMs | Not certified | Open | No | No | No | Analyst workstation; no gov sale |
| **CheriBSD** (Morello) | FreeBSD on CHERI | Not certified | Open | Partial | No | **Yes** | Research, not gov product |
| **Sphragis** | **Rust microkernel** | **In flight** | **Apache-2.0** | **Yes** | **Yes (native)** | **Yes (designed)** | Demo-ready today |

**Sphragis is the only OS with all four columns simultaneously: memory-safe + CNSA-2.0-native + CHERI-ready + Apache-2.0.**

---

## Slide 13 — Unit economics + milestones

**Burn structure (per master-plan financial model):**

| Year | Headcount | Salary + loaded | Other | Total burn |
|---|---|---|---|---|
| Y1 (M0-12) | 1 founder + 3 eng | $750K | $250K (cert work, hardware, conf, legal) | **$1.0M** |
| Y2 (M13-24) | +1 eng (4 total) | $900K | $400K (FIPS lab in earnest, more hardware) | **$1.3M** |
| Y3 (M25-36) | +1 eng (5 total) | $1.1M | $300K | **$1.4M** |
| **3-year total** | | | | **$3.7M** |

**Capital efficiency:**
- 47 P0 reqs out-of-MISSING in 24 hours of founder + autonomous-agent work
- Founder burned ~0 capital pre-incorporation
- Demo bundle assembled with no external funding

**Revenue ramp (master plan):**

| Year | Source | Revenue |
|---|---|---|
| Y1 | SBIR Phase I | $75K |
| Y2 | SBIR Phase II + small commercial | $1.5M |
| Y3 | Phase III + ACT 3 sub + first commercial | $5-15M |

**Cash-out / cash-in gap:** ~$1-2M from outside capital beyond SBIR + Phase III to bridge.

**Investor ask: $1-3M seed → 18 months runway → SBIR Phase II + first commercial pilot.**

---

## Slide 14 — Key risks + mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| SBIR Phase I rejection (80-90% per-submission rate) | High per-submission | Submit to 3 programs in parallel (DoD/AFWERX/DARPA SBIR); ≥70% odds of ≥1 award |
| FIPS 140-3 CMVP queue >24 months | High | Submit as early as possible; non-critical-path for first revenue |
| INSPECTA / PROVERS doesn't fund us | Medium | RSSC is the open-BAA path; ACT 3 sub is parallel revenue not dependent on DARPA |
| Apple changes M4 firmware breaking our boot | Low-Med | Maintain Asahi-community relationship; x86_64 port reduces single-hw dependency |
| Confidential AI market goes to Anjuna / Edgeless first | Medium | Differentiator is TCB size (we're 96K, they wrap 30M-line Linux); time-to-market is bounded |
| Competing Rust gov-OS startup emerges 2027-2028 | Medium | Moat is 14-week audit history + M4 boot + Verus proof artifact + Apache-2.0 + SLSA-L4 chain — hard to replicate quickly |
| Founder solo-bandwidth limit | High Y1 | Y1 hires post-Phase-I award; agent-augmented execution proven (47 P0 reqs / 24 hours) |

---

## Slide 15 — The ask

> **[FOUNDER TO FILL based on round size]**
>
> **Seed round target: $1.5M - $3M.**
>
> Use of funds:
> - **40%** engineer hires (Verus specialist + x86_64/CHERIoT hw + UX productization)
> - **25%** FIPS 140-3 Level 1 certification (lab + CMVP queue)
> - **15%** conference attendance + gov-buyer outreach + travel
> - **10%** legal + IP + trademark + insurance
> - **10%** runway buffer
>
> **Milestones in exchange for capital:**
> - **M6:** SBIR Phase I awarded (or 3 rejections + commercial design partner LOI)
> - **M12:** Phase II started; x86_64 port live; first AFRL / DIU / DARPA PM relationship
> - **M18:** Pre-FIPS-cert pilot deployment; first commercial design partner contract
> - **M24:** FIPS 140-3 Level 1 cert in hand; STIG submitted to DISA
> - **M30-36:** First commercial gov revenue ($500K-$5M ARR)
>
> **Co-investor preferences:** defense-focused (Shield, Lux, a16z American Dynamism, Razor's Edge, Booz Allen Ventures) + strategic (Lockheed Martin Ventures, In-Q-Tel post-Phase-II).

---

## Slide 16 — Contact + next steps

> **[FOUNDER TO FILL]**
>
> **[Founder Name], Founder & CEO**
> [email]@sphragis.com
> [phone]
>
> **Public evidence chain:** https://github.com/kadenlee1107/Sphragis
> **Apache-2.0 licensed | DCO sign-off on every commit | Reproducible build verified**
>
> **What we'd like from this meeting:**
> 1. Feedback on Phase I framing (Sphragis-CDX as software-defined CDS substrate)
> 2. Intro to defense-relevant LPs in your network if the thesis resonates
> 3. Indicative timing on a follow-on diligence conversation
>
> **We will follow up within 48 hours with:** complete demo-bundle artifacts (capability statement PDF, security target, threat model, demo deck, Phase I proposal draft) on request.

---

## Appendix A — Investor objection FAQ

**Q: Why another OS? RHEL works.**
A: RHEL fails the 2027 CNSA 2.0 mandate without a multi-year OpenSSL retrofit. RHEL's 30M-line C TCB can't be formally verified for non-interference. RHEL is not memory-safe. The mandate forces refresh; the refresh has no current vendor that meets the bar.

**Q: Why not just use seL4?**
A: seL4 is a kernel substrate, not a deployable OS. We use seL4's existence as a moat — they cede whole-kernel proofs to us in exchange for we cede the substrate market to them. seL4 + Sphragis are complementary, not competitive. Sphragis is what an end-user operator deploys; seL4 is what a defense integrator embeds.

**Q: How do you compete with Green Hills / Wind River when they have 30 years of relationships?**
A: We don't compete on relationships. We compete on the CNSA 2.0 cliff and the memory-safety policy mandate. Their products **cannot** meet the bar without a multi-year rewrite. Ours **does**, today.

**Q: Defense sales cycles are 3-5 years. Where's the cash flow?**
A: Two answers. (1) SBIR Phase I → II → III converts in 24-36 months, not 3-5 years, and has revenue per phase. (2) Commercial Plan B (confidential AI inference) has 3-9 month sales cycles and doesn't gate on cert work.

**Q: Apple M4 is unusual for gov work. Why not x86?**
A: M4 is the demo target; x86_64 port is a Phase II deliverable. M4 is strategic — it proves we can independently RE a chip the Asahi community can't, which is technical credibility no other Rust-OS team has. x86_64 is where DoD actually runs production today.

**Q: Why Apache-2.0 not commercial-only?**
A: Defense primes won't embed copyleft code. Apache-2.0 maximizes commercial OEM channels. We monetize via professional services, support contracts, and direct gov contracts — not license fees.

**Q: One founder is a risk.**
A: True. Post-Phase-I award we hire 3 engineers (verus specialist + hardware port + UX). The current velocity (47 P0 reqs / 24 hours) is agent-augmented; this scales with seed capital, not blocks on it.

**Q: What if In-Q-Tel passes?**
A: They typically don't fund pure OS plays — we expect a pass for that reason and don't model IQT into the financial plan. SBIR Phase II + Phase III + STRATFI bridge fund without them.

---

## Appendix B — One-paragraph elevator

Sphragis is a memory-safe Rust microkernel built for the 2027 NSA CNSA 2.0 cliff that invalidates every legacy gov-OS for new acquisitions. It's the only OS that simultaneously ships CNSA-2.0-native crypto, attestation as a kernel primitive, reproducible bit-identical builds, CHERI-ready architecture, and Apache-2.0 licensing — proven by a 14-week mechanical-trace audit history and a verified boot on Apple M4 hardware. We're raising $1.5M-$3M seed to hire 3 engineers, close FIPS 140-3 Level 1 certification, and convert SBIR Phase I into Phase II ($1.25M) and Phase III (no cap). First commercial gov revenue at Month 30-36; commercial dual-use Plan B in confidential AI inference at 3-9 month sales cycles.
