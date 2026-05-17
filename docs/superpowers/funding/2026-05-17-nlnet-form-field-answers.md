# NLnet Form — Per-Field Answer Cheat Sheet

**Form:** https://nlnet.nl/propose (live as of 2026-05-17)
**Submission portal:** their web form (linked from the propose page)

Copy-paste each field below into the live form. Every answer respects the form's character limits (verified by approximate count). Stay concise — NLnet explicitly says **"focus primarily on the what and how, not so much on the why."**

---

## §1 — Please select a call

**Thematic call:** `NGI Zero Commons Fund`

(Most-recommended choice for security infrastructure with broad public benefit. The other NGI funds are more narrowly scoped: TALER = payments, Fediversity = federated social, RH Tech = academic-focused. Commons Fund is the catch-all for general open-source security/sovereignty work, which is us.)

---

## §2 — Contact information

> **[FOUNDER TO FILL — your actual personal info]**

| Field | Value |
|---|---|
| **Your name** | Kaden Lee |
| **Email address** | sphragis-os@proton.me |
| **Phone number** | +1 [your phone — required field per the `+` placeholder] |
| **Organisation** | *(leave blank — incorporation in flight; NLnet accepts individual applicants)* |
| **Country** | United States of America |

---

## §3 — General project information

| Field | Value |
|---|---|
| **Proposal name** | `Sphragis: Verus Non-Interference Proofs for a Memory-Safe Rust Microkernel` |
| **Website / wiki** | `https://sphragis.netlify.app/` |

---

## §4 — Abstract (1200 characters max)

Copy this directly:

> Sphragis is an open-source (Apache-2.0), memory-safe Rust microkernel that boots on real Apple M4 hardware today. It organizes user workloads into capability-isolated "caves" with kernel-enforced multi-level security labels. The project targets the 2027-2030 procurement-cliff window when NSA CNSA 2.0 + EU NIS2/CRA mandates render most existing OS substrates ineligible for high-assurance deployments.
>
> This proposal requests €50,000 over 6 months to complete two formal non-interference proofs using the Verus tool: (a) the cave capability dispatcher — proving that no system-call sequence from one cave can affect another's observable state, and (b) the IPC subsystem (AF_UNIX, pipes, shared memory). Funds support the principal investigator (~500h) and an external Verus consultant (~200h), preferably European.
>
> Outcomes: completed Verus proofs in production code; reusable proof patterns published for adoption by other Rust microkernels (Hubris, Asterinas, Theseus, RedoxOS); a methodology paper submitted to USENIX Security 2027 or IEEE S&P 2027.

*(~1110 characters — within 1200 limit ✓)*

---

## §5 — Have you been involved with projects or organisations relevant to this project before? (2500 chars optional)

Copy this directly:

> Yes — I am the sole founder and maintainer of Sphragis, which I have built from scratch over 14 weeks of full-time work. The relevant prior work consists of:
>
> Independent reverse-engineering of Apple M4 boot: Asahi Linux does not yet support M4. I built a custom reverse-engineering pipeline (Apple Device Tree parsing, PMGR clock-gate discovery, ATC-PHY tunable replay) from scratch. The kernel boots on real M4 hardware today. Boot evidence: docs/photos/2026-04-17_first_m4_boot/ in the repository.
>
> Sphragis kernel itself: ~96,000 lines of memory-safe Rust including capability-isolated process model ("caves"), AES-256-GCM-SIV encrypted filesystem (SealFS), TLS 1.3 + X25519MLKEM768 hybrid post-quantum key exchange, HMAC-SHA-384 chained audit log with WORM segment export, attestation primitive (ML-DSA-87 signed quotes), and 85 QMP-driven self-test scripts. License: Apache-2.0 throughout, enforced by cargo-deny CI gate.
>
> Strategic and threat-modelling documentation: ~6,000 lines spanning requirements specification, gap analysis, master implementation plan, threat model, Security Target (CC:2022 Part 1 §B-conformant), NIST SP 800-53 control inheritance matrix, FIPS 140-3 cryptographic module boundary documentation, and operator runbook.
>
> Formal-methods scaffolding directly relevant to this proposal: the verification/ directory contains a Verus toolchain installation, smoke proofs, and two complete non-interference proof specifications (verification/cap_dispatch/SPEC.md and verification/ipc_flow/SPEC.md) — the exact items this grant funds completion of.
>
> Public artifact: https://github.com/kadenlee1107/Sphragis (Apache-2.0, ~199 Rust source files, ~143 commits in the most recent productization push alone).

*(~1750 characters — within 2500 limit ✓)*

---

## §6 — Requested support

### Requested Amount

> **50000**

*(between 5000 and 500000 EUR per their dropdown range — €50K hits NLnet's sweet spot for focused-scope security infrastructure projects)*

### Explain what the requested budget will be used for? (2500 chars max)

Copy this directly:

> Total request: €50,000 over 6 months.
>
> Breakdown:
>
> Principal investigator labor: €35,000 = ~500 hours at €70/hr loaded rate = ~6 months at 50% time on the formal-methods work. The €70/hr rate represents approximately 70% of fair-market senior systems-engineering compensation; the discount is an in-kind cost-share contribution.
>
> External Verus consultant: €12,000 = ~200 hours at €60/hr = ~25% allocation across months 3-6. Preferred consultant pool: European formal-methods research groups (IMDEA Software Institute, MPI-SWS, INRIA, University of Cambridge, KTH). Selecting a European consultant keeps NLnet funds within the European research ecosystem.
>
> Paper publication (open-access fee): €1,000. USENIX Security or IEEE Symposium on Security and Privacy open-access fee for the methodology paper.
>
> Hosting / cloud / domain (6 months): €800. CI runners, project website, domain renewals.
>
> Conference travel (FOSDEM 2027, Brussels, February): €1,200. Present the grant-funded results to the European open-source community.
>
> Other funding sources, past and present:
>
> - No external funding received to date. Personal savings have funded the work so far.
>
> - Parallel applications in flight or planned: Sovereign Tech Fund Germany (€120K, broader scope covering crypto module + supply-chain provenance + this Verus work); OpenSSF Alpha-Omega ($10-50K, different scope on security infrastructure broadly); US Federal SBIR Phase I ($75K, entirely separate US-federal scope).
>
> No duplicate-funding risk relative to this NLnet proposal: the €50K Verus-proof scope is narrowly defined. If STF also awards, the NLnet portion specifically funds the Verus work; STF would fund the parallel crypto + supply-chain work. Transparent coordination would be reported per standard grant terms.

*(~1900 characters — within 2500 limit ✓)*

### Compare your own project with existing or historical efforts (4000 chars max)

Copy this directly:

> Closest comparable in formal-methods scope: seL4 (NICTA / Data61, BSD-2). seL4 has full functional-correctness proofs over ~10,000 lines of C kernel code with proof corpus 20× that. Deployments include NIO SkyOS (mass-production cars), HENSOLDT TRENTOS (defense), NASA cFS, DARPA HACMS. Sphragis is COMPLEMENTARY to seL4: we cede whole-kernel proofs and claim narrower information-flow non-interference proofs on critical subsystems via Verus (a Rust-native formal-methods tool). The seL4 team has spent ~25 person-years on their proofs; we target a more tractable scope using a more modern toolchain in a memory-safe language.
>
> Closest Rust microkernel comparables (none with formal proofs today):
>
> - Hubris (Oxide Computer, MPL-2.0) — ~2,000-line kernel, no formal verification claims.
>
> - Asterinas (Ant Group, MPL-2.0) — framekernel architecture, no proofs.
>
> - Theseus (Rice University, MIT) — single-address-space design, some type-system safety analysis, no kernel proofs.
>
> - RedoxOS (community, MIT) — full microkernel + userspace, no formal verification claims.
>
> Closest formal-methods tools for Rust:
>
> - Verus (Microsoft Research + CMU, MIT) — what we use; well-suited for kernel-scale proofs but documented production patterns are scarce.
>
> - Kani (AWS, Apache-2.0) — bounded model-checking, complementary; we use as fallback for memory-safety regression tests.
>
> - Creusot (INRIA, LGPL) — more mature SMT-based but LGPL license is incompatible with our Apache-2.0-only dependency policy.
>
> - Aeneas (INRIA, Apache-2.0) — also of interest, evaluated as alternative if Verus regresses.
>
> What makes this work novel:
>
> - First production-Rust-kernel non-interference proof using Verus.
>
> - Pattern publication enables Hubris, Asterinas, Theseus, and RedoxOS to adopt similar proofs without re-deriving from scratch.
>
> - Targets the post-CNSA-2.0 procurement window when EU governments need formally-grounded substrates that are not US-vendor-locked.
>
> - Apache-2.0 license maximizes downstream adoption (no copyleft contamination concerns for prime contractors or commercial integrators).
>
> Closest historical NLnet-funded analogues for inspiration:
>
> - Sequoia PGP — Rust security infrastructure (analogous in being a memory-safe Rust security project).
>
> - rust-tls — Rust TLS stack.
>
> - Various NGI Assure-funded formal-methods and verification tooling projects.
>
> This proposal extends the trajectory of those projects to a new domain: capability-isolated microkernels with formally-proven non-interference properties.

*(~2700 characters — within 4000 limit ✓)*

### Significant technical challenges (5000 chars max, optional but recommended)

Copy this directly:

> Six anticipated challenges and planned mitigations:
>
> 1. Verus toolchain maturity. Verus is actively-developed research-grade software at Microsoft Research and CMU. Toolchain regressions are possible mid-engagement. Mitigation: parallel Kani fallback for memory-safety properties; reduce proof scope to single-function granularity if dispatcher-wide proofs prove intractable. Either case still produces a publishable methodology paper documenting what worked and what did not.
>
> 2. Non-interference at production-kernel scale is genuinely uncertain. No published Verus non-interference proof of a production Rust microkernel exists today. We are prepared for the possibility that the full dispatcher proof will require multiple iterations of refactoring the dispatcher itself to be more proof-friendly. The 6-month timeline accommodates this.
>
> 3. Solo-founder bandwidth. Sphragis is currently single-maintainer with ~96,000 LoC. Six months focused on formal methods means less attention to other subsystems. Mitigation: existing 85-script QMP-driven self-test harness catches regressions; cargo-deny + cargo-audit CI gates run autonomously; agent-assisted maintenance (Anthropic Claude as a paired engineering agent) handles routine code review and audit tracking. The grant funds a Verus consultant to supplement principal-investigator bandwidth specifically on the formal-methods axis.
>
> 4. Verus consultant identification and onboarding. The pool of researchers with Verus expertise AND availability for a 200-hour engagement is narrow. Mitigation: identification within Month 1; multiple candidate institutions identified (IMDEA Software Institute, MPI-SWS, INRIA, Cambridge, KTH). European consultant preferred for NLnet alignment.
>
> 5. Specification correctness. Non-interference proofs depend on a correctly-stated specification. The verification/cap_dispatch/SPEC.md and verification/ipc_flow/SPEC.md documents exist but may need iteration as we engage with the actual proof. Mitigation: the consultant brings methodology depth for specification validation; the open-source nature means the security research community can review specifications even before proofs complete.
>
> 6. Pattern reusability validation. The "reusable proof patterns" deliverable is only valuable if other projects can actually adopt them. Mitigation: outreach to Hubris, Asterinas, Theseus, and RedoxOS maintainers during Months 5-6 to validate the patterns against their architectures and document the adaptation paths.
>
> The proposal scope is deliberately conservative: even if multiple challenges materialise, we deliver a partial proof + analysis + methodology paper, which is independently publishable and useful to the broader open-source security community.

*(~2900 characters — within 5000 limit ✓)*

### Describe the ecosystem of the project, and how you will engage with relevant actors and promote the outcomes? (2500 chars max)

Copy this directly:

> Direct ecosystem actors involved:
>
> - Verus development team (Microsoft Research / CMU) — proofs validate Verus at production scale, feeding back into tool improvement and Verus community case studies.
>
> - European formal-methods research groups (IMDEA, MPI-SWS, INRIA, Cambridge, KTH) — consultant candidate pool, paper co-authorship opportunity, ecosystem visibility for the broader European open-source security community.
>
> - Rust microkernel community (Hubris by Oxide Computer, Asterinas by Ant Group, Theseus by Rice University, RedoxOS community) — pattern adoption targets; direct outreach during Months 5-6.
>
> Downstream beneficiaries (potential, not yet active):
>
> - European public-sector cybersecurity agencies (BSI Germany, ANSSI France, NCSC UK, ENISA) seeking sovereign-deployable formally-grounded substrates that are not US-vendor-locked.
>
> - Open-source security infrastructure ecosystem broadly (curl, OpenSSL, Sequoia PGP, NTRU/PQ projects — all adjacent NLnet alumni whose security stories benefit from a memory-safe Rust microkernel option).
>
> - Academic kernel-verification research community (USENIX Security, IEEE S&P, EuroSys).
>
> Engagement plan:
>
> - Monthly progress reports on the project blog (forthcoming at sphragis.netlify.app).
>
> - Direct outreach to Hubris / Asterinas / Theseus / RedoxOS maintainers during Months 5-6 to validate proof-pattern transferability.
>
> - Conference presentation at FOSDEM 2027 (Brussels, February) presenting grant-funded results to the European open-source community.
>
> - Methodology paper submitted to USENIX Security 2027 or IEEE Symposium on Security and Privacy 2027.
>
> - NLnet acknowledgment in repository README, release notes, and paper acknowledgments section.
>
> Sphragis is pre-1.0 with no established downstream dependents — we are explicit about this rather than overstating traction. The grant-funded work aims to make Sphragis more credible to potential adopters by producing the formal-methods backing that procurement reviewers increasingly require.

*(~1900 characters — within 2500 limit ✓)*

---

## §7 — Attachments

NLnet explicitly says: **"Don't waste too much time on this. Really."**

**Skip attachments** unless you want to attach a small supporting file. The proposal text above is self-contained.

If you do want to attach something for credibility, the most useful would be:
1. The Phase 1 research synthesis (`docs/superpowers/research/2026-05-16-gov-os-requirements.md`) — exported to PDF, ~25 pages, gives reviewers the procurement-landscape context
2. The capability statement PDF (if rendered)
3. The Security Target (`docs/SECURITY_TARGET.md`) — exported to PDF

None of these are required. Reviewers can browse the public GitHub repository directly.

---

## §8 — Generative AI question

**They link their policy:** https://nlnet.nl/foundation/policies/generativeAI/

The honest disclosure: **YES, this application was drafted with extensive AI assistance via Anthropic Claude, with full human review, editing, and authorship of all strategic claims and final content.**

Whatever option in the dropdown most closely matches that honest answer is the right pick. Likely options the dropdown shows:

- "No, I did not use generative AI" → **do not pick this** (would be dishonest)
- "Yes, for editing / proofreading only" → marginal
- "Yes, for drafting" → **most accurate**
- "Yes, extensively / for the bulk of the content" → also honest
- "Yes, with disclosure" → if available

**Pick whichever option most accurately reflects: Claude drafted, you authored and reviewed.** NLnet's GenAI policy (per the link) accepts AI-assisted work with transparent disclosure. They reject only undisclosed AI-only work without human review.

If the dropdown asks for a written explanation, use:

> Yes. This application was drafted with substantial assistance from Anthropic Claude (a large language model) acting as a paired writing and editing agent. The principal investigator authored all strategic claims, technical specifications, and final content. Claude assisted with drafting, structural editing, and compression to fit form character limits. All factual claims about Sphragis are personally verified by the principal investigator against the project repository. The AI-augmented development pattern is itself a documented feature of the Sphragis project's workflow.

---

## §9 — How may we handle your information

**Check:** `[ ] I have read and understood NLnet's Privacy Statement.` → **CHECK IT** (required to submit)

**Check:** `[X] Send me a copy of this application.` → **already pre-checked, leave as-is** (recommended; you'll get the full submission emailed to you)

**PGP pub-key field:** Leave blank unless you have a PGP key handy.

---

## §10 — Final check before clicking "Send request"

- [ ] Call selected: **NGI Zero Commons Fund**
- [ ] All contact-info fields filled (name, email, phone, country)
- [ ] Proposal name: `Sphragis: Verus Non-Interference Proofs for a Memory-Safe Rust Microkernel`
- [ ] Website: `https://sphragis.netlify.app/`
- [ ] Abstract: pasted from §4 above (1200 char limit verified)
- [ ] Prior projects: pasted from §5 above (2500 char limit verified)
- [ ] Requested amount: `50000`
- [ ] Budget explanation: pasted from §6 budget block (2500 char limit verified)
- [ ] Compare with existing: pasted from §6 comparison block (4000 char limit verified)
- [ ] Technical challenges: pasted from §6 challenges block (5000 char limit verified, optional)
- [ ] Ecosystem: pasted from §6 ecosystem block (2500 char limit verified)
- [ ] GenAI disclosure: honest, per §8
- [ ] Privacy Statement checkbox: checked
- [ ] "Send me a copy" checkbox: checked (default)

**Then click "Send request."**

---

## §11 — After submission

- **Confirmation copy** arrives at `sphragis-os@proton.me` (because you checked "Send me a copy"). Verify it arrived.
- **Decision time:** 4-12 weeks. NLnet sends a personal response either accepting or declining.
- **Declined applications receive constructive feedback** — useful even if not awarded.
- **Awards are usually €5K-€50K direct;** larger asks are reviewed against tighter criteria.
- **Calendar reminder for week 6** to start following-up rhythm if no decision yet.

---

## §12 — Strategic note

NLnet's mission statement: **"We fund people with bright ideas — and the technical know-how to make them happen."**

Sphragis fits this exactly. The 14-week build history + working M4 boot + Verus toolchain already scaffolded = "technical know-how" demonstrated. The €50K ask for focused Verus-proof completion = "bright idea" with concrete deliverables.

**This is the highest-confidence shot in the parallel-grants portfolio.** STF was a stretch (foundational existing infrastructure framing). NLnet is a direct fit (early-stage bright ideas).

If awarded, this funds 6 months of formal-methods work AND provides the credibility marker that opens doors to OpenSSF Alpha-Omega, GitHub Accelerator, and the federal SBIR/DARPA tracks downstream.
