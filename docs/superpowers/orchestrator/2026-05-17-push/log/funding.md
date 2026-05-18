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
