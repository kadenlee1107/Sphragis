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
