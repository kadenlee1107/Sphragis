# Sphragis — Defense Seed VC Cold-Pitches (Draft v1)

**Status:** DRAFT v1 — Kaden reviews + sends. Outreach team only drafts.
**Date:** 2026-05-17
**Author:** Outreach team (2026-05-17 multi-team push)
**Source:** `docs/superpowers/funding/2026-05-17-vc-target-list-v1.md` (target list + intro paths + master template) + `docs/superpowers/funding/2026-05-17-vc-pitch-deck-v1.md` (16-slide deck, used as memo for first meetings) + `docs/superpowers/funding/2026-05-17-financial-model-3yr-v1.md` (3-year model). Public anchor: `https://github.com/kadenlee1107/Sphragis`.

**Targets (per charter §3 Outreach):**
1. Shield Capital (DC + SF) — defense-first thesis; Anduril/Hadrian/Saronic/Skydio portfolio. (Tier 1 lead per VC target list §1.)
2. Lux Capital (NYC + SF) — defense + frontier-science thesis; Anduril/Saildrone/Shield AI portfolio. (Tier 1 second outreach per VC target list §6.)
3. a16z American Dynamism (SF + NYC) — explicit "national interest" thesis; Anduril/Hadrian/Castle/Apex portfolio. (Tier 1 third per VC target list §6.)

**Note on public-marketing-site posture:** The marketing site footer reads "Sphragis is developed by an independent team. We do not solicit investment." That language is the *public-product* posture (evaluation-access framing for buyers). The VC track is the *founder/private* track and uses the public GitHub + boot-evidence as credibility anchors — these emails should not reference the marketing site as an investment-solicitation page. (If a VC partner asks why the site says "we do not solicit investment," the honest answer is: the site is positioned for security buyers; investment solicitation is via direct founder outreach.)

**Common framing all three emails reuse (paragraph 1 template, derived from VC target list §5):**
*"I'm raising a $1.5M–$3M seed for Sphragis, a memory-safe Rust microkernel positioned to capture the 2027-01-01 NSA CNSA 2.0 acquisition cliff that invalidates every legacy gov-OS substrate (INTEGRITY-178B, VxWorks 653, LynxOS-178, RHEL) for new National Security System deployments. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4)."*

Each email below: subject line, addressee, body. Tailoring lives in paragraphs 2 + 3 per charter §3 DoD. Founder signature block omitted — Kaden fills before send.

---

## Email 1 — Shield Capital

**To:** Andrew Berenberg, Partner — Shield Capital (DC + SF). VC target list §1 names Berenberg + Phil Bilden as public-facing partners; Berenberg is the natural cold-outreach destination for a single-founder defense seed.
**Addressee line:** "Dear Andrew,"
**Subject:** `Sphragis — Rust microkernel for the 2027 CNSA-2.0 procurement cliff (seed)`

---

Dear Andrew,

I'm raising a $1.5M–$3M seed for Sphragis, a memory-safe Rust microkernel positioned to capture the 2027-01-01 NSA CNSA 2.0 acquisition cliff that invalidates every legacy gov-OS substrate (INTEGRITY-178B, VxWorks 653, LynxOS-178, RHEL) for new National Security System deployments. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm reaching out to Shield first because Shield's portfolio thesis — Anduril, Hadrian, Saronic, Skydio — is the cleanest fit for what Sphragis is: a substrate the next generation of defense-tech companies in your portfolio will eventually need underneath the application layer, and which today has no Rust-native, CNSA-2.0-native, Apache-2.0-licensed incumbent.

Why Sphragis maps to the Shield thesis specifically:

- **Two procurement cliffs hit in the same 18-month window.** FIPS 140-2 → 140-3 (2026-09-21) plus the NSA CNSA 2.0 cliff (2027-01-01) plus the CISA/NSA/ONCD memory-safety mandate plus the ARM Morello/CHERIoT-Ibex capability-hardware availability — four converging forces, no current commercial substrate that satisfies all four. The federal CDS + tactical-edge refresh market alone is ~$2B per the master plan, and the installed base (Green Hills, Wind River, Lynx, RHEL) cannot meet the 2027 bar without a multi-year retrofit. This is the timing thesis (deck slide 2 and slide 5).
- **The five-differentiator moat is artifact-backed, not narrative.** (1) Rust microkernel + Verus information-flow proof harness on cap-dispatcher + IPC; (2) CNSA-2.0-native crypto module live with boot-time KATs (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384, LMS); (3) attestation as a first-class kernel primitive — `attest::quote()` + external verifier in `tools/attest-verifier/`; (4) bit-identical reproducible build verified end-to-end; (5) CHERI-ready architecture with caves mapping 1:1 to CHERI compartments. Deck slide 4 lists the source artifact backing each one.
- **Defensible substrate vs. defensible application play.** Where Anduril is a defensible application company, Sphragis is the defensible substrate company. Apache-2.0 means defense primes can integrate without copyleft contamination; 14-week mechanical-trace security audit history + DCO sign-off on every commit is the production discipline that lets a prime sub-contract us with confidence.

I'd value 30 minutes in the next two-to-four weeks to walk through the full memo and live-demo an M4 boot + Quote() flow over Zoom share-screen — the live boot is the single highest-impact thing I can show a defense seed VC, and most have never watched a Rust microkernel boot on verified Apple M4 silicon with attestation Quote production. I can travel to DC or SF, or video at your convenience. Within 48 hours of mutual interest I can send the full diligence bundle (16-slide pitch deck, 6–10 page investor memo, capability statement, security target, threat model, 3-year financial model, master implementation plan). Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, reproducible build verified). Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## Email 2 — Lux Capital

**To:** Bilal Zuberi, Partner — Lux Capital (NYC + SF). VC target list §1 names Josh Wolfe + Bilal Zuberi as public-facing; Zuberi's frontier-tech focus is the closer match for a deep-tech systems-software seed.
**Addressee line:** "Dear Bilal,"
**Subject:** `Sphragis — memory-safe Rust microkernel for the post-quantum, capability-hardware era (seed)`

---

Dear Bilal,

I'm raising a $1.5M–$3M seed for Sphragis, a memory-safe Rust microkernel positioned to capture the 2027-01-01 NSA CNSA 2.0 acquisition cliff that invalidates every legacy gov-OS substrate (INTEGRITY-178B, VxWorks 653, LynxOS-178, RHEL) for new National Security System deployments. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm reaching out because Lux's twin thesis — defense (Anduril, Shield AI, Saildrone) and frontier-science deep-tech — is exactly the band Sphragis lives in: it's both a defense substrate and a category-defining systems-software bet on the convergence of post-quantum crypto, memory-safe languages, and capability hardware in the same 18-month window.

Why Sphragis maps to the Lux thesis specifically:

- **A new category, not a new entrant in an old one.** Sphragis is defining "sovereign-grade attested-cave OS for the post-quantum, capability-hardware era" — not Linux-with-hardening (96K-line Rust kernel, ~300× smaller TCB than Linux), not seL4 (we cede whole-kernel proofs to seL4 and claim info-flow non-interference on critical subsystems), not Green Hills / Wind River (closed-source, retrofit-only). The closest commercial product (INTEGRITY-178B) was certified in 2008 on PowerPC and is frozen-config closed-source. The category has no current incumbent (deck slide 3).
- **24-hour productization push as proof of velocity.** 143 commits / 24 hours; 47 P0 requirements moved from MISSING to HAVE/PARTIAL; demo bundle assembled with no external capital. Pre-seed founder + autonomous-agent execution against a documented 36-month master plan. This is the capital-efficiency story Lux cares about — agent-augmented development against a bounded plan, not unbounded R&D burn. Deck slide 7 shows the day-1-start vs today metric table.
- **Four-market TAM with two parallel motions.** (a) US gov direct (DoD/IC, $2B+ TAM) + (b) Confidential AI inference for Anthropic/OpenAI/Meta-class workloads (3–9 month sales cycles, no cert gating, $3B+ projected TAM by 2028 — Anjuna + Edgeless + Fortanix indicative). The federal track is patient capital, the commercial track is near-term cash flow, and each dollar in one produces evidence usable in the other. Deck slides 9–10 and the 3-year model at `docs/superpowers/funding/2026-05-17-financial-model-3yr-v1.md`.

I'd value 30 minutes in the next two-to-four weeks. The single highest-impact thing I can show is a live M4 boot + attestation Quote flow over Zoom share-screen — five minutes of substance most defense-tech investors have never seen. I can travel to NYC or SF, or video. Within 48 hours of mutual interest I can send the full diligence bundle (16-slide deck, investor memo, capability statement, security target, threat model, 3-year financial model, master implementation plan). Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, bit-identical reproducible build verified). Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## Email 3 — a16z American Dynamism

**To:** Katherine Boyle, General Partner — a16z American Dynamism (SF + NYC). VC target list §1 names David Ulevitch + Katherine Boyle as public-facing American Dynamism partners; Boyle is the most active public voice on the thesis and the natural cold destination.
**Addressee line:** "Dear Katherine,"
**Subject:** `Sphragis — Rust microkernel for sovereign-compute infrastructure (seed)`

---

Dear Katherine,

I'm raising a $1.5M–$3M seed for Sphragis, a memory-safe Rust microkernel positioned to capture the 2027-01-01 NSA CNSA 2.0 acquisition cliff that invalidates every legacy gov-OS substrate (INTEGRITY-178B, VxWorks 653, LynxOS-178, RHEL) for new National Security System deployments. The kernel boots on real Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux doesn't yet support M4). I'm writing because American Dynamism's "companies that further the national interest" thesis is the cleanest public statement of why Sphragis exists — the sovereign-compute substrate gap is downstream of the same set of policy moves (CISA/NSA/ONCD memory-safety mandate, NSA CNSA 2.0 cliff, NIAP General-Purpose OS evaluation sunset) that motivate the rest of your portfolio.

Why Sphragis maps to the American Dynamism thesis specifically:

- **Strategic-gap proof, not founder optimism.** NIAP stopped accepting new General-Purpose OS Common Criteria evaluations in the early 2020s; the Separation Kernel Protection Profile was sunset in 2011. The Air Force operates 250+ Cross-Domain Solution endpoints that need refresh by 2027, and the procurement vehicle for that refresh doesn't currently exist. Sphragis is being built to be the substrate that fills that procurement vacuum — a deck-slide-6 fact that holds independent of any execution-quality argument the founder makes.
- **Apache-2.0 + 14-week mechanical-trace audit history = prime-integration-ready.** Defense primes (Lockheed, Northrop, RTX, BAE, plus the ACT 3 IDIQ subs: AIS, CNF, Global InfoTek, Invictus, Radiance) won't embed copyleft code. Apache-2.0 maximizes commercial OEM channels; we monetize via professional services + support contracts + direct gov contracts, not license fees. This is the channel-strategy alignment American Dynamism portfolio companies (Anduril, Hadrian, Castle, Apex) all depend on for the substrate underneath their products.
- **18-month-runway plan against a 24-month-or-less convertible-evidence horizon.** $1.5M–$3M seed → 18 months → SBIR Phase II ($1.25M) + first commercial design partner pilot. Use of funds: 40% engineering / 25% FIPS 140-3 cert / 15% conf + travel + GTM / 10% legal + IP / 10% buffer. Milestones: M6 SBIR Phase I awarded (or 3 rejections + commercial LOI), M12 Phase II + x86_64 port live + first AFRL/DIU/DARPA PM relationship, M18 pre-FIPS pilot deployment + first commercial contract, M24 FIPS Level 1 cert in hand. Deck slide 15 + 3-year model at `docs/superpowers/funding/2026-05-17-financial-model-3yr-v1.md`.

I'd value 30 minutes in the next two-to-four weeks. The highest-leverage thing I can show is a live M4 boot + attestation Quote flow over Zoom share-screen — five minutes that defense-tech investors have rarely seen. I can travel to SF or NYC, or video. Within 48 hours of mutual interest I can send the full diligence bundle (16-slide deck, investor memo, capability statement, security target, threat model, 3-year financial model, master implementation plan). Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, reproducible build verified). Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## What Kaden does next

1. Fill the founder signature block on each email (see VC pitch deck slide 16 for the contact-block template — name, email, phone, "Founder & CEO, Sphragis Inc.", public evidence chain URL).
2. Verify partner names are current — VC firm partner rosters change. The names used here (Andrew Berenberg / Bilal Zuberi / Katherine Boyle) come from VC target list v1 §1 as of 2026-05-17. If any of those three has moved firms, swap to the next named public-facing partner per the same section.
3. Per VC target list §6 sequencing: **Week 1** send Shield + Lux. Wait one week for response or polite pass. **Week 2** if no progress, add a16z American Dynamism + (separately) 8VC + Founders Fund. **Don't fire all three at once** — coordinated outreach signals desperation per §6.
4. Diligence-readiness check (VC target list §7): before any first meeting confirm pitch deck PDF, investor memo PDF, capability statement, security target, threat model, master plan PDF, demo deck, 3-year financial model, cap table, founder bio, and — most importantly — live-demo capability (M4 boot + Quote() over Zoom share-screen in 5 minutes). Per §7, the live demo is the asymmetric impression-maker.
5. If a partner reply asks "your marketing site says you don't solicit investment, what's going on?" — the honest answer is the public product page is positioned for security-eval buyers (evaluation-access framing); investor solicitation is direct founder outreach. Marketing-site language can be updated later if it becomes a recurring friction point. (Outside Outreach team scope to change.)
6. Track each send in a CRM / spreadsheet (founder action — out of scope for Outreach).

**Do NOT send these emails until Kaden has reviewed personally.** Outreach team only drafts.
