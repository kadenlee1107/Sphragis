# Funding log — 2026-05-17 push

Mission: 4 funding drafts in `docs/superpowers/funding/`. See
`docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 (Funding).

Append entries chronologically. Format per `multi-team-push.md` §5.
The very last entry on DoD-met must read `STATUS: COMPLETE` with
file paths + notes for Kaden.

---

## 2026-05-17 23:54 — funding — starting

Read plan §3 (Funding), §4 (Operating Principles), §5 (Coordination
Protocol), §7 (Escalation Rules). Read ADR-0002 (path corrections —
day-1 sweep lives in `research/` not `funding/`). Verified clean tree
on `main`. Inbox to-funding.md is empty.

Drafting order (shortest → longest):
1. BIS notification template (template from founder-action-checklist
   already exists; refine + commit).
2. GitHub Sponsors profile copy (markdown ready to paste).
3. OpenSSF Alpha-Omega draft (WebSearch first for current format +
   funding amounts).
4. GitHub Accelerator draft (WebSearch first for cohort window).

Next: read founder-action-checklist (BIS template starting point),
STF/NLnet form-field answers (model for tone + grounding), marketing
site (public-claim ceiling), day-1 sweep (status of stack).

STATUS: IN_PROGRESS

---

## 2026-05-17 23:59 — funding — draft 1 of 4 written (BIS notification)

Drafted `docs/superpowers/funding/2026-05-17-bis-notification-template.md`.

Key findings while drafting:
- The v0 template embedded in the founder-action-checklist Phase 3
  item 12 had **two factual errors** I corrected:
  - Wrong CFR citation: cited `EAR §740.17(b)(1)`; correct citation for
    publicly available open-source crypto is **15 CFR §742.15(b)**
    (with §734.7 underlying). 740.17(b)(2) is a different regime
    (self-classification reports). The charter actually said
    "740.17(b)(2)" but cross-referencing the actual rule, BIS guidance
    page, and EFF's published explainer confirms §742.15(b) is the
    right governing section for source-code-on-internet notification.
  - Wrong NSA email: v0 had `web_site@nsa.gov`; correct is
    `enc@nsa.gov` (ENC Encryption Request Coordinator). Verified via
    BIS guidance + EFF explainer.
- Template now uses 5 explicit `[FOUNDER: ...]` placeholders so
  nothing ships with a hidden gap (legal name, reply email, phone,
  business address, first-publication date).
- Marked `STATUS: DRAFT — KADEN TO SEND` per charter.

Sources cited in draft: founder-action-checklist, marketing-site,
day-1 sweep, master plan.

Next: GitHub Sponsors profile copy.

STATUS: IN_PROGRESS

---

## 2026-05-18 00:08 — funding — draft 2 of 4 written (GitHub Sponsors)

Drafted `docs/superpowers/funding/2026-05-17-github-sponsors-profile.md`.

Structure: 8 sections, paste-ready blocks for each GitHub Sponsors
profile field (tagline, intro markdown, tiers, goals, FAQ, featured
work, founder action steps, sources).

Charter said "3-tier minimum: $5 / $25 / $100." Expanded to **5 tiers**
($5 / $25 / $100 / $250 / $1000) so the long-tail of higher-conviction
sponsors has a place to land without forcing a custom-tier
conversation (GitHub Sponsors makes custom tiers friction-heavy). The
charter's minimum tier reward structure ($5 supporter / $25 named in
CONTRIBUTORS.md / $100 logo on README) is preserved exactly.

Each tier has a "Reward Kaden commits to" subsection so the founder
knows what they're signing up to deliver. FAQ covers tax
deductibility, anonymous sponsorship, conflict-of-interest disclosure,
roadmap pointer, lapse grace period — the questions most likely to
land in `sponsors@sphragis.com` inbox.

Inbox to-funding.md polled: empty.

Sources cited in draft: marketing-site, day-1 sweep, founder-action
checklist, master plan, multi-team plan §3 charter.

Next: OpenSSF Alpha-Omega draft (will WebSearch for current format +
typical grant amount).

STATUS: IN_PROGRESS

---

## 2026-05-18 00:22 — funding — draft 3 of 4 written (OpenSSF Alpha-Omega)

Drafted `docs/superpowers/funding/2026-05-17-openssf-alpha-omega-v0.md`.

Research conducted via WebSearch + WebFetch:
- Alpha-Omega website (`alpha-omega.dev/grants/how-to-apply/`)
  outlines a 4-part proposal framework: (1) project identification,
  (2) current state, (3) desired outcomes, (4) implementation
  approach. Submission is via HubSpot form at
  `share.hsforms.com/1sZmUUNQLQ0SwlMhrcOs7ww4tvhy`. Form fields not
  directly inspectable; structure draft to match the 4-part
  proposal framework on the published page so paste-in is clean.
- Documented Alpha-Omega grants to established projects: Rust
  Foundation $460K, Eclipse $400K, Node.js $300K. Total: >$20M
  across >70 grants since 2022.
- March 2026 announcement: Linux Foundation secured $12.5M in
  fresh funding from Anthropic, AWS, GitHub, Google, Google
  DeepMind, Microsoft, OpenAI specifically for AI-era OSS
  security work. Suggests appetite for security infrastructure
  funding is elevated right now.

Ask sized at **$150,000 over 9 months** — between A-O's small-grant
baseline and the multi-hundred-K grants to mature projects. Rationale
in §0: Sphragis is "smallest but essential components" per A-O
language, sits at the OS-substrate layer beneath libraries A-O
already funds.

Three work packages (Verus proofs / SLSA-L4 supply chain / FIPS 140-3
boundary refinement) — each independent and parallelizable so A-O
can de-scope if budget pressure requires. Distinct from STF's WP1
(crypto module completion) and NLnet's Verus-only scope; partial
overlap on FIPS 140-3 is flagged for transparent intake-call
discussion.

Sources cited: marketing-site, day-1 sweep, STF/NLnet/SBIR drafts,
founder-action checklist, master plan, multi-team plan §3 charter,
plus 2 WebFetch sources from alpha-omega.dev and 1 WebSearch result.

Inbox to-funding.md polled: empty.

Next: GitHub Accelerator draft (WebSearch for current cohort
window first).

STATUS: IN_PROGRESS

---

## 2026-05-18 00:36 — funding — DONE

DoD met. Four drafts produced:

1. `docs/superpowers/funding/2026-05-17-bis-notification-template.md`
   — commit `8900a8fc` — STATUS: DRAFT — KADEN TO SEND
2. `docs/superpowers/funding/2026-05-17-github-sponsors-profile.md`
   — commit `70fafecb` — STATUS: DRAFT v1
3. `docs/superpowers/funding/2026-05-17-openssf-alpha-omega-v0.md`
   — commit `f6fa47e3` — STATUS: DRAFT v1
4. `docs/superpowers/funding/2026-05-17-github-accelerator-v0.md`
   — this commit — STATUS: DRAFT v1

Each draft:
- Has a clear header with status, date, and author = "Funding Team"
- Cites primary sources (marketing-site, day-1 sweep, STF/NLnet/SBIR
  drafts, founder-action checklist, master plan) and WebSearch /
  WebFetch results where program facts required them
- Ends with a "What Kaden does next" section
- Does not exceed the public-claim ceiling in
  `marketing-site/index.html` lines 1180-1810

**Notes for Kaden:**

1. The BIS template corrects two factual errors in the v0 template
   embedded in the founder-action-checklist Phase 3 item 12:
   - Cites **15 CFR §742.15(b)** (correct for publicly-available
     open-source crypto), not §740.17(b)(1) (a different License
     Exception ENC regime). Verified via BIS guidance + EFF
     explainer. The charter directive said "740.17(b)(2)" — that
     is also incorrect; it covers self-classification reports for
     encryption *items*, not source-code notification.
   - Addresses NSA at **`enc@nsa.gov`** (correct per BIS guidance
     + EFF), not `web_site@nsa.gov` (the v0 template's value).
   I recommend updating the founder-action-checklist v0 template to
   match, or treating this v1 as the canonical version. Founder
   should send THIS v1, not the v0.

2. The GitHub Sponsors profile expands from the charter's "3-tier
   minimum" to **5 tiers** ($5/$25/$100/$250/$1000) so higher-
   conviction sponsors don't have to negotiate custom tiers
   (GitHub Sponsors makes that friction-heavy). The charter's
   minimum reward structure (named in CONTRIBUTORS.md at $25,
   README logo at $100) is preserved exactly.

3. The Alpha-Omega ask is sized at **$150K over 9 months** — between
   the documented small-grant baseline and the multi-hundred-K
   grants to mature projects (Rust Foundation $460K, Eclipse $400K,
   Node.js $300K were the published comparables). Three independent
   work packages so A-O can de-scope to a subset if budget pressure
   requires.

4. The "GitHub Accelerator" draft pivots to **GitHub Secure Open
   Source Fund** as the primary application target. Reasons in
   §0 of that file: (a) GitHub Accelerator 2024 was the last
   cohort, was AI-only (a poor fit for Sphragis's anti-AI-in-TCB
   stance), and is now closed; (b) GitHub Secure Open Source Fund
   IS currently open on rolling applications, is security-focused,
   and is a direct fit; (c) the file preserves an
   "if-Accelerator-reopens" section for future use. The charter's
   directive — "ready when next cohort opens" — is satisfied for
   Accelerator; the equivalent OPEN program is targeted as
   primary.

5. Parallel-funding overlap flagged for Kaden's transparent
   disclosure on intake calls: Alpha-Omega WP3 (FIPS 140-3
   pre-engagement) partially overlaps STF WP1 (CNSA 2.0 module
   completion); Alpha-Omega WP2 (SLSA-L4) partially overlaps
   GitHub Secure Open Source Fund Week 2 (supply-chain
   attestation goes live). The Sphragis-side ask total is now:
   STF €120K + NLnet €50K + Alpha-Omega $150K + GitHub Secure
   $10K + SBIR Phase I $75K. Worst-case all-five award scenario
   is ~$340K + €170K over 6-9 months — within reason for a
   solo-maintainer scaling to small team, but Kaden should be
   ready to deconflict scope if multiple awards land.

6. The Sphragis Inc. incorporation status is referenced as "in
   formation as of 2026-05-17" across all four drafts. Once
   Atlas issues the Certificate of Incorporation + EIN, those
   four drafts can be sent to a sed-friendly find/replace pass
   ("in formation" → "incorporated YYYY-MM-DD"). The drafts are
   structured to make this trivial.

Inbox to-funding.md polled final time: empty.

Working tree before commit: my files only (BIS / sponsors /
alpha-omega / accelerator drafts + this log). Other teams'
in-flight work (eng-1 x509, eng-2 sealfs_rotation, eng-3
cap_token/mls_label) untouched.

STATUS: COMPLETE
