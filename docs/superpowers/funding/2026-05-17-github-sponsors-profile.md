# GitHub Sponsors Profile — Copy

**STATUS: DRAFT v1**
**Date drafted:** 2026-05-17
**Author:** Funding Team (Mac Claude, vault-mediated)
**Target account:** `github.com/sponsors/kadenlee1107`
**Mode:** drafting only; founder pastes into the GitHub Sponsors
signup form and submits.

This file is structured as **paste-ready blocks**. Each section
maps 1:1 to a field in the GitHub Sponsors profile editor:

- §1  Short bio (the "tagline" / one-line subtitle)
- §2  Long bio ("introduction" markdown block)
- §3  Tier descriptions ($5 / $25 / $100 / $250 / $1000)
- §4  Goals + milestones
- §5  FAQ
- §6  Featured work + linked projects
- §7  "What Kaden does next"
- §8  Sources cited

Markdown in each section is GitHub-flavored (gfm); GitHub Sponsors
renders the same subset GitHub uses for issues + PRs (headings,
emphasis, lists, code, links). **No emoji** in any paste-ready
block — Sphragis's project style avoids them.

---

## §1 — Tagline (one line, ~80 chars max)

Paste exactly:

```
Security-first Rust microkernel for Apple Silicon. Apache-2.0. Boots on M4.
```

---

## §2 — Introduction (long bio, GitHub-flavored markdown)

Paste exactly:

```markdown
## What Sphragis is

Sphragis is a bare-metal Rust microkernel for Apple Silicon. It boots
today on real Apple M4 hardware (Mac16,1 / J604 / T8132 "Donan") via
an independent reverse-engineering pipeline — Asahi Linux does not
yet support M4; we got there our own way.

The design is security-first. The default is no third-party browser
in the trusted computing base; no telemetry; no upstream Linux fork.
What ships in the kernel: an AES-256-GCM-SIV encrypted filesystem
(SealFS), kernel-mediated TLS 1.3 with post-quantum hybrid key
agreement (X25519 + ML-KEM-768), per-process default-deny network
egress, and capability-isolated processes ("caves") with
kernel-enforced multi-level security labels.

## Why fund this

Open-source operating systems serve as critical security
infrastructure. Sphragis is built to be the substrate that survives
the 2027 procurement cliff — when NSA CNSA 2.0 (May 2025) and the EU
Cyber Resilience Act + NIS2 align to require post-quantum-cryptography
+ memory-safe systems languages + reproducible builds for new
high-assurance deployments. Most operating systems shipping today do
not meet that bar.

Sphragis is Apache-2.0 licensed throughout. License hygiene is
enforced by `cargo-deny` in CI — no GPL, no LGPL, no SSPL, no
copyleft contamination. This means prime contractors and commercial
integrators can adopt the substrate without dependency conflicts.

## Where the work is today

Roughly 96,000 lines of Rust across 199 source files. Bit-identical
reproducible build verified. 14 weeks of mechanical-trace security
audit history. Hybrid post-quantum TLS verified end-to-end against
`pq.cloudflareresearch.com`. Full algorithm enumeration on the
[specification page](https://github.com/kadenlee1107/Sphragis#specifications).

This is solo-maintained at present. Sponsorship directly funds my
time on the project — every dollar reduces the bus factor.
```

---

## §3 — Tiers

GitHub Sponsors supports both one-time and monthly tiers. The charter
specifies **3-tier minimum**: $5/mo (supporter), $25/mo (named in
CONTRIBUTORS.md), $100/mo (logo on README). I expand to **5 tiers** so
the long tail of higher-conviction sponsors has a place to land
without forcing a custom-tier conversation, which GitHub Sponsors
makes friction-heavy.

Each tier below has: monthly amount, short name, description
(markdown), and the concrete reward Kaden commits to.

### Tier 1 — $5/month

**Tier name (short):** `Supporter`

**Description (paste exactly):**

```markdown
Funds about 4 minutes of focused engineering per month. Helps cover
infrastructure (domains, CI minutes, hosting). Receives a thank-you
note + access to the sponsors-only release-note pre-publish drop the
day before each tagged release.
```

**Reward Kaden commits to:**
- Sponsor receives a thank-you email when they start the
  subscription.
- Sponsor gets a 24-hour pre-publish look at every tagged-release
  note via a private GitHub Discussion thread tagged
  `Sponsors / Release-Preview`.

---

### Tier 2 — $25/month

**Tier name (short):** `Named contributor`

**Description (paste exactly):**

```markdown
Everything in Supporter, plus your name (or handle) added to
`CONTRIBUTORS.md` in the public repository under the "Sustaining
sponsors" section. Helps cover ~20 minutes of focused engineering per
month — meaningfully extends the runway on a 0-revenue project.
```

**Reward Kaden commits to:**
- Everything in Tier 1.
- Sponsor's display name (or chosen handle) added to
  `CONTRIBUTORS.md` under a `Sustaining sponsors` section, in the
  order their subscription began. Removed automatically if the
  subscription ends (with a 90-day grace period).

---

### Tier 3 — $100/month

**Tier name (short):** `README sponsor`

**Description (paste exactly):**

```markdown
Everything in Named contributor, plus an organization logo or
personal avatar (~64×64 px) displayed in the `## Sponsors` section
of the repository README.md. Funded sponsors at this tier directly
support the Verus formal-methods work, the FIPS 140-3 module
boundary documentation, and the SLSA Level 4 supply-chain provenance
pipeline.
```

**Reward Kaden commits to:**
- Everything in Tier 2.
- Logo (organization) or avatar (individual) of up to 64×64 px,
  linked to the sponsor's chosen URL, in the README `## Sponsors`
  section. Logo refreshed monthly; sponsor can update the asset by
  emailing `sponsors@sphragis.com`.
- Sponsor listed by name in release notes for every minor and major
  tag during the sponsorship window.

---

### Tier 4 — $250/month

**Tier name (short):** `Quarterly call`

**Description (paste exactly):**

```markdown
Everything in README sponsor, plus a quarterly 30-minute video call
with the maintainer to discuss roadmap, threat model, or specific
integration questions. Includes pre-publish review access to any
public-facing capability brief or whitepaper before release.
```

**Reward Kaden commits to:**
- Everything in Tier 3.
- One scheduled 30-minute video call per quarter (90 days). Sponsor
  books a slot via the founder's calendar link.
- Pre-publish review window (5 business days) on any capability
  brief, whitepaper, or major announcement before public release.

---

### Tier 5 — $1,000/month

**Tier name (short):** `Strategic sponsor`

**Description (paste exactly):**

```markdown
Everything in Quarterly call, plus monthly video sync with the
maintainer, named acknowledgment in any peer-reviewed publication
arising from the work (e.g. the planned USENIX Security 2027 /
IEEE S&P 2027 methodology paper on Verus non-interference proofs),
and priority access to evaluation builds. Strategic sponsors are
listed at the top of the README `## Sponsors` section.
```

**Reward Kaden commits to:**
- Everything in Tier 4.
- Monthly 30-minute video sync with the maintainer.
- Named acknowledgment in the Acknowledgments section of any
  peer-reviewed publication produced during the sponsorship.
- Top-of-list placement in README `## Sponsors`.
- First-look access to evaluation build artifacts (SHA-256
  fingerprint + signed bundle) on the same schedule as
  Sphragis Inc.'s own evaluation customers.

---

## §4 — Goals + milestones

GitHub Sponsors lets the maintainer set public funding goals. Use
two: one short-term, one long-term. Both are paste-ready below.

### Goal 1 — Short-term: $500/month sustaining

**Goal title (paste exactly):**

```
$500/month — covers infrastructure + 10% maintainer time
```

**Goal description (paste exactly):**

```markdown
At $500/month sustaining, GitHub Sponsors covers Sphragis's
infrastructure bills (domains, CI runners, hosting, project website)
and frees roughly 10% of the maintainer's time from
adjacent-revenue work, directly extending runway on the formal-
methods + FIPS 140-3 module boundary work. This is the
"keep-the-lights-on" tier.

**Concrete progress unlocked at this goal:** monthly public progress
reports continue uninterrupted; CI gates (cargo-deny, cargo-audit,
clippy, cargo-fmt, reproducible-build verifier) continue running on
every commit without degradation; the project website at
sphragis.com stays up.
```

### Goal 2 — Long-term: $5,000/month

**Goal title (paste exactly):**

```
$5,000/month — 50% maintainer time on formal verification + FIPS 140-3
```

**Goal description (paste exactly):**

```markdown
At $5,000/month sustaining, GitHub Sponsors funds roughly 50% of the
maintainer's time, allowing direct progress on the highest-leverage
work that is otherwise blocked by funding-gap delays:

1. **Verus non-interference proofs** of the capability dispatcher
   and IPC subsystem (specs at `verification/cap_dispatch/SPEC.md`
   and `verification/ipc_flow/SPEC.md`).
2. **FIPS 140-3 cryptographic module boundary** refinement to
   CMVP-pre-engagement quality, paving the path to FIPS Level 1
   validation.
3. **SLSA Level 4 supply-chain provenance** — sigstore-signed
   release artifacts with Rekor transparency log entries,
   LMS-signed kernel images, in-toto attestation envelopes.

These three workstreams together close the largest gaps between
"promising prototype" and "production-deployable open-source
critical infrastructure." Sustained funding here directly multiplies
the public benefit of the project.
```

---

## §5 — FAQ (optional but recommended)

GitHub Sponsors profiles can include a Q&A block. Paste this as a
single markdown blob into the profile's "Additional information"
field, or stage it in the repo at `.github/SPONSORS_FAQ.md` and link
from the profile.

```markdown
## Sponsors FAQ

**Q: What does my sponsorship fund?**
A: Direct maintainer time. Sphragis is solo-maintained today. Every
sponsorship dollar reduces the bus factor and extends runway on
formally-grounded systems-software work that is otherwise difficult
to fund through traditional venture channels.

**Q: Is sponsorship tax-deductible?**
A: Currently no — Sphragis Inc. is a for-profit Delaware C-Corp
(incorporation in flight as of 2026-05-17). If you need
tax-deductible giving against U.S. tax law, consider sponsoring
parallel open-source security infrastructure projects via the Open
Source Initiative or the OpenSSF, or look for our future fiscal
sponsorship arrangement (if one is established later).

**Q: Can my organization sponsor anonymously?**
A: Yes. Pick the tier that fits and choose "Private" visibility in
your GitHub Sponsors settings. The maintainer will receive your
sponsorship amount but not display your name in CONTRIBUTORS.md or
the README. Private sponsors still receive the same reward content
(pre-publish notes, release-call access, etc.) via email.

**Q: I want to fund a specific work package — can I do that?**
A: For sums above the $1,000/month Strategic tier, yes — email
`sponsors@sphragis.com` and we can structure a directed sponsorship
(e.g. "underwrite the Verus consultant engagement"). Below that
threshold, sponsorships fund the project broadly per the public
roadmap.

**Q: Where is the roadmap?**
A: `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
in the public repository. The "what we ship next quarter" view is in
release notes; the "where we are headed over 36 months" view is in
the master plan.

**Q: How do you handle conflict of interest with prime contractors
or government customers?**
A: Sponsorship is publicly disclosed (sponsors above the Anonymous
threshold appear in CONTRIBUTORS.md and/or the README). Any sponsor
relationship that creates a procurement conflict of interest with a
specific contract opportunity will be disclosed to the contracting
party per standard FAR + DFARS conflict-of-interest disclosure
rules. Sphragis Inc. will not accept sponsorship that requires
non-disclosure of the sponsor's identity.

**Q: Is corporate sponsorship welcome?**
A: Yes. The README + CONTRIBUTORS.md recognition tiers are designed
for corporate logos as much as individual contributors. Strategic
sponsors get pre-publish review access on whitepapers and
capability briefs — useful for integration teams aligning their
roadmap to ours.

**Q: What happens if my subscription lapses?**
A: 90-day grace period during which all recognition stays in place.
After 90 days, the name/logo is moved from "Sustaining sponsors" to
"Alumni sponsors" in CONTRIBUTORS.md and removed from the README
`## Sponsors` panel. You can resume at any time.
```

---

## §6 — Featured work + linked projects

GitHub Sponsors lets the maintainer feature up to 6 repositories on
their profile. Paste these in order:

1. `kadenlee1107/Sphragis` — the kernel itself
2. *(future)* `kadenlee1107/sphragis-marketing-site` — if/when the
   marketing site repo is split out
3. *(future)* `kadenlee1107/sphragis-verification` — if/when the
   verification harness is split out

For now (2026-05-17), only Sphragis itself is public. Feature only
that one until additional public repos exist.

---

## §7 — What Kaden does next

1. **Verify the GitHub Sponsors waiting list / availability** for
   the `kadenlee1107` account. Sponsors signup may require
   tax-form pre-clearance (W-9 for US persons + bank info for ACH
   payouts). Allow 1-3 business days for GitHub to review.
2. **Open the GitHub Sponsors profile editor** at
   `github.com/sponsors/kadenlee1107/dashboard` (URL appears after
   waitlist acceptance).
3. **Paste §1 (tagline) into the "Introductory bio" / tagline field.**
4. **Paste §2 (introduction) into the "Introduction" markdown
   editor.**
5. **Create 5 tiers per §3.** GitHub Sponsors UI: "Tiers" tab →
   "Create new tier" for each. Paste the short name, monthly
   amount, and the markdown description from §3 verbatim.
6. **Set 2 goals per §4.** GitHub Sponsors UI: "Goals" tab → "Add
   goal." Paste title + description.
7. **Paste §5 (FAQ) into the "Additional information" field** OR
   commit it as `.github/SPONSORS_FAQ.md` in the repo and link from
   the profile.
8. **Feature `kadenlee1107/Sphragis`** as the primary repository
   in the "Featured work" section.
9. **Decide the visibility setting** (Public / Private) for your
   own account's sponsorships received — keep as Public by default
   for transparency.
10. **Activate the profile.** GitHub Sponsors charges Stripe
    processing only; GitHub itself takes 0% of sponsorship.
11. **Add the "Sponsor" button to the public repository:** create
    `.github/FUNDING.yml` containing `github: [kadenlee1107]` and
    commit. This adds the Sponsor button on every public repository
    page.
12. **Stage the README `## Sponsors` section** as a placeholder
    block (`<!-- sponsors-readme-section-managed -->`) so future
    sponsor logos drop in cleanly without a layout rework.
13. **Reach the $500/month short-term goal** by mentioning
    sponsorship in the next public update (project blog, social
    post, or release note) without over-rotating the project on
    fundraising.

**Estimated total founder time:** ~45 minutes (most of which is
the GitHub-side waitlist + bank verification).

---

## §8 — Primary sources cited

- `marketing-site/index.html` lines 1180-1810 — public-claim
  ceiling, specifically: kernel claim, M4 boot, Apache-2.0,
  AES-256-GCM-SIV SealFS, X25519+ML-KEM-768 hybrid TLS, ~199 Rust
  files, ~96K LoC, contact email `sphragis-os@proton.me`
- `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  §1 (current capabilities), §6 (funding readiness verdicts —
  context for tier framing)
- `docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
  Phase 4 item 16 (sponsorship-related email infrastructure plan)
- `docs/superpowers/plans/2026-05-17-multi-team-push.md` §3
  (Funding, draft #4) — charter directive (3-tier minimum: $5 /
  $25 / $100 — this draft expands to 5 tiers per the "minimum"
  semantics)
- `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
  referenced in §5 FAQ
