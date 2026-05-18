# Sphragis — ACT 3 Prime Cold-Pitches (Draft v1)

**Status:** DRAFT v1 — Kaden reviews + sends. Outreach team only drafts.
**Date:** 2026-05-17
**Author:** Outreach team (2026-05-17 multi-team push)
**Source:** `docs/superpowers/funding/2026-05-17-act3-capability-brief-v1.md` (Capability Brief v1) §1–§7. Public anchor: `https://github.com/kadenlee1107/Sphragis` + marketing site (M4 boot evidence + specs section).

**Targets (per charter §3 Outreach):**
1. Assured Information Security (AIS) — Rome, NY (largest ACT 3 sub; collocated with AFRL Info Directorate)
2. CNF Technologies — San Antonio, TX + DC (cyber R&D; smaller / more flexible)
3. Global InfoTek (GiTec) — Reston, VA (ML-threat detection; substrate-as-AI-host angle)

**Common framing all three emails reuse (paragraph 1 template):**
*"We're a single-founder, Rust-microkernel startup positioned for the 2027-01-01 NSA CNSA 2.0 acquisition cliff. The kernel boots on real Apple M4 hardware today (independent reverse-engineering; Asahi Linux doesn't yet support M4). We're reaching out because [PRIME] is an active ACT 3 sub with a portfolio that intersects what Sphragis offers as a substrate, and we'd like to explore a teaming arrangement on an upcoming task-order response."*

Each email below: subject line, addressee, body. Tailoring lives in paragraphs 2 + 3 per charter §3 DoD. Founder signature block is omitted — Kaden fills before send (per Capability Brief §7 "Founder action — items to fill before send: contact info").

---

## Email 1 — Assured Information Security (AIS)

**To:** Business Development inbox, Assured Information Security, Inc. (Rome, NY)
**Addressee line:** "To the Assured Information Security BD / Cyber R&D leadership team,"
**Subject:** `Sphragis — Rust microkernel for ACT 3, CNSA-2.0-native, boots on M4 today`

---

To the Assured Information Security BD / Cyber R&D leadership team,

I'm Kaden Lee, founder of Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel positioned for the 2027-01-01 NSA CNSA 2.0 acquisition cliff. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm reaching out because AIS is the largest ACT 3 sub-contractor by task-order value on record and is collocated with the AFRL Information Directorate in Rome — the procurement vehicle and the customer Sphragis is being built to serve. The full capability brief is below; the short version is that Sphragis is a sub-contractable, Apache-2.0-licensed substrate that lets AIS add a "CNSA-2.0-native + memory-safe + formally-grounded kernel" check to upcoming ACT 3 task-order bids without any copyleft contamination of AIS's proprietary integration work.

Three capabilities tailored to AIS's cyber-R&D portfolio:

- **CNSA-2.0-native crypto module — live + boot-time KATs.** ML-KEM-1024 (FIPS 203), ML-DSA-87 (FIPS 204), AES-256, SHA-384, and LMS as defaults in the `gov-strict` build profile, not retrofits — direct fit for AFRL solicitations that increasingly cite the 2027-01-01 mandate.
- **Bell-LaPadula + Biba MLS labels enforced at the kernel layer.** Per-cave (per-process) classification labels with kernel-mediated cross-cave gates on every syscall — relevant to AIS's cross-domain and tactical-edge analyst workstation problem space.
- **Attestation as a first-class kernel primitive.** `attest::quote(nonce, claims) -> Quote` (CBOR + ML-DSA-87 signature) with an external verifier tool — produces evidence artifacts that map directly to AFRL TIM (Technical Interchange Meeting) deliverables.

Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, bit-identical reproducible build verified). Marketing site with M4 boot photos + full specifications: https://sphragis.com (or the equivalent public URL — Kaden to confirm). The full capability brief (§1–§7, including a concrete 6–12 month / ~$500K–$1M task-order pilot shape for a tactical-edge analyst workstation handling mixed-classification data) is ready to send on confirmation that the framing fits AIS's pipeline.

I'd value 30 minutes with your CTO + BD lead in the next two weeks to walk through a live M4 boot + Quote() demo and discuss whether any of AIS's active or upcoming ACT 3 task orders has a fit for Sphragis-as-substrate. I can travel to Rome on short notice; I can also do video. Within 48 hours of mutual interest I can send the full demo bundle (capability statement PDF, security target, threat model, attestation flow demo) and a drafted teaming-agreement skeleton for your contracting counsel's review. Many thanks for your time and consideration.

[Founder signature block — Kaden to fill: name, email, phone, "Sphragis Inc. (Delaware C-Corp — incorporation in flight)"]

---

## Email 2 — CNF Technologies

**To:** Business Development inbox, CNF Technologies (San Antonio, TX + DC)
**Addressee line:** "To the CNF Technologies cyber R&D leadership team,"
**Subject:** `Sphragis — memory-safe Rust kernel substrate, ACT 3 teaming inquiry`

---

To the CNF Technologies cyber R&D leadership team,

I'm Kaden Lee, founder of Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel positioned for the 2027-01-01 NSA CNSA 2.0 acquisition cliff. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm reaching out to CNF specifically because your cyber-R&D footprint — smaller than the largest ACT 3 subs but with a reputation for flexibility on novel-technology pilots — looks like a high-fit first home for a substrate-stage teaming relationship. Sphragis is Apache-2.0, so a CNF integration product can ship closed-source without copyleft contamination; the substrate is sub-contractable into your existing AFRL task-order pipeline rather than displacing it.

Three capabilities tailored to CNF's R&D-pilot orientation:

- **Greenfield Rust, ~96K-LoC kernel TCB — ~300× smaller than Linux.** Memory-safety policy alignment (CISA/NSA/ONCD guidance citing Rust as the canonical path) is increasingly cited in solicitation evaluation criteria; CNF gets a "memory-safe substrate" check that no Linux-derived bid can match without a 10-year rewrite.
- **Formally-grounded subsystems.** Verus harness on the capability dispatcher + IPC paths with two written non-interference proof specifications (`verification/`); the proof artifact is publishable evidence that CNF can cite in BAA responses for PROVERS / INSPECTA-adjacent task orders.
- **14-week mechanical-trace security audit history + bit-identical reproducible build verified.** 32 P0 requirements HAVE of 75 total (full requirements register at `docs/superpowers/specs/2026-05-16-sphragis-gov-os-requirements.md`); every commit DCO-signed; SLSA-L4 chain demonstrable end-to-end. This is the production-quality discipline a CNF pilot evaluation needs to clear.

Public evidence chain: https://github.com/kadenlee1107/Sphragis. Boot evidence on real M4 hardware is published with photos and per-frame captions: https://sphragis.com (or the equivalent public URL — Kaden to confirm). The full capability brief (§1–§7) includes a 6–12 month / ~$500K–$1M task-order pilot proposal and a risk-mitigation table tailored to common prime objections ("open source means no support" / "Apache-2 vs GPL contamination" / "FIPS 140-3 not yet certified" — all addressed).

I'd value 30 minutes with your CTO or technical-BD lead in the next two-to-four weeks. The fastest format is a video walk-through of a live M4 boot + the attestation Quote flow + the audit-chain WORM export, which I can demonstrate over Zoom share-screen in five minutes. If the fit resonates I can send the full demo bundle within 48 hours and we can scope a joint AFRL TIM presentation in the following 60 days. Many thanks for your time.

[Founder signature block — Kaden to fill]

---

## Email 3 — Global InfoTek (GiTec)

**To:** Business Development inbox, Global InfoTek, Inc. (Reston, VA)
**Addressee line:** "To the Global InfoTek BD / Engineering leadership team,"
**Subject:** `Sphragis — secure substrate for ML threat-detection workloads (ACT 3 teaming)`

---

To the Global InfoTek BD / Engineering leadership team,

I'm Kaden Lee, founder of Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel positioned for the 2027-01-01 NSA CNSA 2.0 acquisition cliff. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm reaching out to Global InfoTek because your portfolio centers on ML-based threat detection and cyber engineering — which positions Sphragis less as a substrate-replacement and more as an *isolation host* for sensitive ML inference and analyst workflows, where the kernel-enforced classification labels and audit chain are the value rather than the kernel itself. The substrate is Apache-2.0; GiTec can integrate it under closed-source product packaging without copyleft contamination.

Three capabilities tailored to GiTec's ML threat-detection + cyber-engineering portfolio:

- **Per-process default-deny network egress with per-cave hostname/SNI policy.** An ML inference job, an OSINT collector, and an analyst workstation cave can run on the same machine and the kernel will refuse cross-cave network reach by default — every denial is audit-logged. This is the "confidential ML on a shared host" property that GiTec analyst products would otherwise build on Linux + namespaces (which is not memory-safe and not CNSA-2.0-native).
- **HMAC-SHA-384 audit chain with WORM export and an offline Python verifier.** Tamper-evident logging that produces forensic-grade evidence artifacts — directly usable in GiTec's threat-detection products as a tamper-evident decision trail, and inheritable as a CDRL deliverable on AFRL task orders.
- **Kernel-mediated TLS 1.3 with hybrid post-quantum key agreement (X25519MLKEM768) and chain-only strict validation, verified end-to-end against Cloudflare's public PQ research endpoint.** No process can ship broken TLS, skip cert validation, or downgrade to HTTP — the substrate guarantee, not a userspace convention.

Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, reproducible build verified). Marketing site with full technical specifications: https://sphragis.com (or the equivalent public URL — Kaden to confirm). The full capability brief — including a concrete 6–12 month pilot shape, prime-risk mitigation table, and a §6 ranking that places Global InfoTek third on our recommended outreach sequence specifically because the ML-host angle is a real value prop — is ready to send on confirmation that this framing maps to GiTec's near-term pipeline.

I'd value 30 minutes with your BD lead and a relevant technical principal in the next two-to-four weeks. Format that works best: a video walk-through of a live M4 boot + Quote()-based attestation + audit-chain WORM export, all of which I can demonstrate in under 10 minutes over Zoom. Within 48 hours of mutual interest I can send the full demo bundle and propose a 6-month proof-of-concept scoped to one of your existing AFRL or DIU pipeline opportunities. Many thanks for your time.

[Founder signature block — Kaden to fill]

---

## What Kaden does next

1. Fill the founder signature block on each email (name, email, phone, company line — see Capability Brief §7 for the template).
2. Confirm the correct marketing-site URL (drafts use `https://sphragis.com` as a placeholder; replace with the actual public URL).
3. Decide attachment vs. send-on-request: the capability brief PDF can either be attached on first send or held until the recipient confirms interest. Per the Capability Brief §6 sequence ("AIS first; if AIS is slow or non-responsive after 2 weeks, parallel to CNF + GiTec"), recommend send AIS Week 1, then CNF + GiTec on Week 3 if AIS hasn't engaged.
4. Identify BD-inbox email addresses for each prime — these emails are addressed to roles rather than specific names because no specific BD contact is identified in the source brief. If Kaden has warm intros into any of the three primes, the addressee line should be updated to that named contact and the "I'm reaching out because…" paragraph adjusted accordingly.
5. Per Capability Brief §6: "If AIS engagement is slow or non-responsive after 2 weeks, parallel outreach to CNF + GiTec." Pace accordingly.
6. Bcc / log every send in a tracking sheet (founder action — outside Outreach team's scope).

**Do NOT send these emails until Kaden has reviewed personally.** Outreach team only drafts.
