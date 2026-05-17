# SP-A4: Marketing Site + Capability Statement Skeleton

**Type:** Founder + light engineering work. Static site + 8-12 page PDF.

**Goal:** Public-facing brand for Sphragis. First link to share at AFCEA WEST 2026 (Feb), DARPA Forecast to Industry, and every first-meeting follow-up. Establishes credibility through positioning + technical depth before any conversations.

**Architecture:** Hugo or Astro static site, deployed to Cloudflare Pages. Single-page-plus structure: home + 5 differentiators + technical depth + downloads + docs + blog + contact. Capability statement as separate downloadable PDF (LaTeX or Pages/Word — whichever the founder prefers).

**Tech Stack:** Hugo (recommended — fastest) or Astro. Cloudflare Pages. Tailwind CSS. Vercel / Cloudflare Pages CI.

**Requirements closed:** DOC-004, DOC-009, STRAT-001 (initial publication).

**Depends on:** SP-A1 (Apache-2.0 license to mention), SP-A3 (entity name for "Copyright Sphragis Inc.").

**Estimated duration:** 1-2 weeks initial; ongoing maintenance.

---

## File Structure (new repo: `sphragis-web/`)

This is a **separate repository** (`sphragis-web/`) — keep it out of the kernel repo to avoid mixing concerns.

```
sphragis-web/
├── content/
│   ├── _index.md                  # Home
│   ├── differentiators/_index.md  # The 5 differentiators page
│   ├── technical/_index.md        # Technical overview
│   ├── docs/...                   # Hosted docs (links to GitHub repo's docs/)
│   ├── blog/...                   # Blog posts (start with launch announcement)
│   └── contact.md                 # Contact + security disclosure
├── layouts/
│   ├── _default/baseof.html
│   ├── _default/single.html
│   └── partials/
├── static/
│   ├── img/
│   ├── downloads/                 # Capability statement PDF goes here
│   └── favicon.ico
├── config.toml                    # Hugo config
└── README.md                      # Self-docs for site contributors
```

**Capability statement:** standalone PDF, separate doc source (LaTeX or Pages). Hosted at `static/downloads/sphragis-capability-statement-v1.pdf`.

---

## Pre-Work: Pick a static-site generator

- [ ] **Decide: Hugo, Astro, or Eleventy**
  - **Hugo** (recommended): single-binary install, fast build, mature theme ecosystem. Good for tech-marketing site.
  - **Astro**: component-based, modern. More flexible but steeper learning curve.
  - **Eleventy**: simple, Node-based. Fine but smaller ecosystem.
  - For a 1-person Y1 effort with security focus, **Hugo wins on simplicity and zero JS-required output**.

---

### Task 1: Set up the repo

- [ ] **Step 1: Create `sphragis-web` repo on GitHub**

Visible (public). Apache-2.0 license. Add a README.

- [ ] **Step 2: Initialize Hugo locally**

```bash
mkdir sphragis-web && cd sphragis-web
hugo new site . --force
git init
git add -A
git commit -s -m "init: empty Hugo scaffold"
```

- [ ] **Step 3: Pick a theme**

- **Recommended**: PaperMod (https://github.com/adityatelange/hugo-PaperMod) — clean, fast, dev-blog-friendly.
- Alternatives: Anatole, Doks (more docs-heavy), Hello-Friend.

Install:
```bash
git submodule add https://github.com/adityatelange/hugo-PaperMod themes/PaperMod
```

- [ ] **Step 4: Configure `config.toml`**

```toml
baseURL = "https://sphragis.com"
languageCode = "en-us"
title = "Sphragis"
theme = "PaperMod"

[params]
description = "Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era"
keywords = ["microkernel", "Rust", "security", "post-quantum", "CHERI", "attestation", "government"]

[params.author]
name = "Sphragis"
email = "info@sphragis.com"

[params.homeInfoParams]
Title = "Sphragis"
Content = "A sovereign-grade attested-cave OS for the post-quantum, capability-hardware era."

[menu]
  [[menu.main]]
    name = "Differentiators"
    url = "/differentiators/"
    weight = 1
  [[menu.main]]
    name = "Technical"
    url = "/technical/"
    weight = 2
  [[menu.main]]
    name = "Docs"
    url = "https://github.com/kadenlee1107/Sphragis/tree/main/docs"
    weight = 3
  [[menu.main]]
    name = "Blog"
    url = "/blog/"
    weight = 4
  [[menu.main]]
    name = "Contact"
    url = "/contact/"
    weight = 5
```

---

### Task 2: Write home page (`content/_index.md`)

- [ ] **Step 1: Draft content**

```markdown
---
title: "Sphragis"
---

Sphragis is a **sovereign-grade attested-cave OS for the post-quantum, capability-hardware era**.

A security-first bare-metal Rust microkernel for government and high-assurance use. Built to be procurable in the 2027-2030 procurement world — not retrofitted to it.

## Five differentiators

1. **Rust microkernel + information-flow proofs.** Memory-safe by language. Non-interference proofs on the capability and IPC subsystems via Verus/Kani.
2. **CNSA-2.0-native, PQC-only crypto.** ML-KEM-1024, ML-DSA-87, AES-256, SHA-384 by default in the gov build. No classical fallback.
3. **Attestation as a first-class kernel primitive.** Every cave is an attestable identity. Caliptra / Apple SEP / TPM 2.0 rooted.
4. **Reproducible, bootstrappable, SLSA-L4 build chain.** Bit-for-bit reproducible. Sigstore-signed. In-toto attested.
5. **CHERI-ready architecture.** Caves map 1:1 to CHERI compartments. CHERIoT-Ibex embedded variant ships in 2026-27.

[See the technical overview →](/technical/)

## What's it for?

Sphragis is for environments where you can name the threat model:
- Confidential computing in the cloud
- Defense contractor mission-system OS
- Government analyst workstation
- Embedded high-assurance (avionics, automotive, industrial control)
- Cross-domain solutions hosting

## Status (2026-05)

Active development. Booting on Apple M4 hardware + QEMU. 14 weeks of mechanical-trace security audit closure (14 criticals + 17 highs + 23 mediums + 3 elite-tier wins). Roadmap toward FIPS 140-3 + DoD STIG + NSA CSfC component listing in flight.

[Source on GitHub](https://github.com/kadenlee1107/Sphragis) · [Latest release](https://github.com/kadenlee1107/Sphragis/releases)
```

---

### Task 3: Write differentiators page (`content/differentiators/_index.md`)

- [ ] **Step 1: Draft long-form content for each of the 5**

For each differentiator: 200-400 words explaining what it is, why it matters for gov, what's the artifact backing it, what competitors claim or don't claim.

Skeleton:
```markdown
---
title: "Five Differentiators"
---

## #1 — Rust microkernel + information-flow proofs

[200-400 words. Lead with: what we claim. Then: why it matters
for gov procurement. Then: the artifact (Verus proof script). Then:
contrast with seL4 (we don't try to outproof them — different claim).]

## #2 — CNSA-2.0-native, PQC-only crypto

[Lead with the CNSA 2.0 mandate dates (2027-01-01 for new NSS, 2033
exclusive). Show the algorithms. Show how the gov-build profile enforces.]

## #3 — Attestation as a first-class kernel primitive

[Lead with: every cave is an attestable identity. Show: Caliptra
2.x / SEP / TPM rooted. Show: HSM-backed operator CA pattern. Contrast
with: every other OS treats attestation as a bolted-on TPM library.]

## #4 — Reproducible, bootstrappable, SLSA-L4 build chain

[Lead with: complete trust chain from silicon RoT to syscall. Show:
sigstore + Rekor entries + in-toto attestations. Contrast with:
Green Hills / Lynx / Wind River can produce build artifacts but
cannot show source-reproducibility.]

## #5 — CHERI-ready architecture

[Lead with: capability-safe in hardware where available. Show: cave-
to-CHERI-compartment mapping. Show: CHERIoT-Ibex prototype roadmap.]
```

---

### Task 4: Write technical overview (`content/technical/_index.md`)

- [ ] **Step 1: Draft**

Pull from the existing `DESIGN.md`, `DESIGN_CAVES.md`, `DESIGN_CAVE_ISOLATION.md`, `DESIGN_CRYPTO.md`, `DESIGN_TLS_HARDENING.md`. Synthesize a 1,000-2,000 word public technical overview suitable for an engineer or AO doing first-pass diligence.

Structure:
- TCB measurement (~70-80K LoC Rust)
- Cave model + per-cave ASIDs
- Crypto stack (CNSA 2.0 alignment)
- SealFS encrypted filesystem
- HMAC audit chain
- Attestation primitive
- Build chain
- Verified subsystem boundary (when VER lands)
- Hardware targets

---

### Task 5: Contact + security disclosure page (`content/contact.md`)

- [ ] **Step 1: Write**

```markdown
---
title: "Contact"
---

## Business inquiries

`info@sphragis.com`

## Security disclosures

`security@sphragis.com`

For sensitive disclosures, use our PGP key (fingerprint posted on this page once issued).
For US Government disclosures, please use our entry in the GitHub Security Advisories
(https://github.com/kadenlee1107/Sphragis/security/advisories).

## Press / media

`press@sphragis.com`

## Company

Sphragis Inc. (Delaware) — incorporated 2026.
NAICS: 541511, 541512, 541519.
CAGE / UEI: [to be added once issued]
SAM.gov: [link to entity record once active]
```

---

### Task 6: Write capability statement PDF (separate doc source)

- [ ] **Step 1: Use LaTeX or Pages — whichever the founder prefers**

Recommended structure for a 8-12 page gov capability statement:

1. **Cover page** — company name, logo, "Capability Statement", date
2. **Executive summary** (1 page) — who we are, what we do, one-sentence value prop
3. **Core capabilities** (2 pages) — the 5 differentiators with concrete artifacts
4. **Technology overview** (2 pages) — architecture, TCB, certification posture
5. **Differentiators vs. incumbents** (1 page) — small comparison matrix (INTEGRITY-178B / seL4 / RHEL / Sphragis)
6. **Past performance** (1 page) — note that we're a new entrant; cite the M4 boot + 14-week audit work as proof points
7. **NAICS codes + certifications-in-progress** (1 page) — list:
   - NAICS 541511, 541512, 541519
   - SAM.gov + CAGE + UEI
   - License Exception ENC filed
   - SBIR Phase I (in flight / awarded)
   - DARPA pitches (PROVERS / INSPECTA / RSSC) (in flight)
   - FIPS 140-3 Level 1 (lab engagement)
   - DoD STIG (drafting)
8. **Differentiating clauses** (1 page) — "We are NOT another Linux distribution. We are NOT a research project. We are NOT a thin wrapper over seL4."
9. **Contact + company info** (final page) — POC name + email, address, banking, GSA-purchase status (once on Schedule)

- [ ] **Step 2: Render to PDF**

If LaTeX: `pdflatex sphragis-capability-statement.tex`
If Pages: File → Export → PDF.

- [ ] **Step 3: Place in site**

Copy to `static/downloads/sphragis-capability-statement-v1.pdf`.

Link from home page footer and contact page.

---

### Task 7: Deploy to Cloudflare Pages

- [ ] **Step 1: Push the site repo to GitHub**

```bash
git remote add origin git@github.com:<kadenlee1107>/sphragis-web.git
git push -u origin main
```

- [ ] **Step 2: Connect to Cloudflare Pages**

Dashboard: https://dash.cloudflare.com → Workers & Pages → Create application → Pages → Connect to Git → Select repo.

Build settings:
- Framework: Hugo
- Build command: `hugo --minify`
- Build output directory: `public`
- Root directory: `/`

- [ ] **Step 3: Configure custom domain**

Cloudflare Pages → Custom domains → `sphragis.com` (after DNS migration).

- [ ] **Step 4: Verify deployment**

Visit `https://sphragis.com`. Verify:
- Home page loads
- All five differentiator sections present
- Capability statement PDF download works
- Contact links work
- No broken images / 404s

---

### Task 8: First blog post (launch announcement)

- [ ] **Step 1: Write `content/blog/sphragis-public-launch.md`**

500-800 word post:
- Why we're building Sphragis
- The strategic gap we fill (4 white spaces)
- The 5 differentiators
- Current status (14-week audit, M4 boot)
- 36-month roadmap (link to master plan doc)
- Call to action: contact for first-meeting demo

- [ ] **Step 2: Cross-post**

Post on HackerNews, /r/rust, /r/cybersecurity, /r/government. Caveat: gov-procurement isn't the HN audience; expect more interest in the Rust microkernel + formal verification + CHERI angle than in the gov-positioning angle.

---

## Test Plan

- All pages render correctly (smartphone + desktop)
- Capability statement PDF downloads + renders correctly
- All internal links work; no 404s
- Site loads in <1s on Cloudflare edge
- Security headers configured (CSP, HSTS, X-Content-Type-Options) via Cloudflare Workers if needed
- Lighthouse score >90 on all axes (performance, accessibility, SEO, best-practices)

## Decision Gates

- **End of Week 2:** Site live at `sphragis.com`. Capability statement PDF v1 downloadable.
- **Ongoing:** Each blog post adds technical depth (weekly cadence is aspirational; monthly is realistic).
- **End of Month 6:** Update capability statement to v2 with: incorporated entity name, SAM/CAGE/UEI numbers, first SBIR Phase I award (if any), DARPA engagement status.

## Output

- ✅ Public marketing site live
- ✅ First share-link for AFCEA WEST 2026
- ✅ Capability statement PDF for every first-meeting follow-up
- ✅ Blog as a forcing function for technical writing (forces synthesis)

## Risks

| Risk | Mitigation |
|---|---|
| Site goes stale | Set quarterly review reminder; update with new milestones |
| Capability statement gets out of date | Version it. Hold to quarterly updates. |
| Marketing copy oversells unverified claims | Every claim cites an artifact. "Verified by Verus" only after VER lands. |
| Domain squatting if we delay | Buy domains in Week 1 of SP-A3 (Operational Setup section). |
