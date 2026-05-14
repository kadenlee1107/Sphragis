# Internal documentation

Some of this project's documentation lives in a **separate private
repository** rather than in this one. That's deliberate: the
material in question is Tier 2 or Tier 3 under the project's
disclosure-posture rules — trade-secret content, internal
strategy, internal coordination, or raw debugging artifacts whose
value depends on not being publicly accessible.

## Where the internal docs live

[`kadenlee1107/sphragis-internal`](https://github.com/kadenlee1107/sphragis-internal)
(private — request access by emailing the project contact in the
[`README`](../README.md)).

| Category | What's there |
|---|---|
| **Trade-secret reverse-engineering** | `docs/M4_GROUND_TRUTH.md` — every verified hex address, PMGR sequence, ATC PHY tunable, AIC2 base, dockchannel UART address, compatible string on Apple M4 silicon. |
| **HV trace evidence** | 31 dated `.txt` files capturing hypervisor traces from M4 bring-up: register dumps, boot-log captures, timing measurements. |
| **AOP bring-up evidence** | `docs/aop_bringup_evidence/` — 31 logs from Apple Always-On-Processor reverse-engineering. |
| **UI screenshots** | `docs/screens/` — internal screenshots from M4 boot sessions. |
| **Pentest writeups** | `security/PENTEST_*.md` — internal security analyses of past builds across nine pentest cycles. Some content is general mitigations (Tier 2); much is specific exploit chains, symbol dumps, and attack-surface analyses (Tier 3). |
| **Internal planning + audit docs** | `docs/PLAN_*`, `docs/OS_FEATURE_GAP_AUDIT.md`, `docs/CAPTURES_AUDIT.md`, `docs/M4_CHICKEN_HUNT.md`, `docs/HV_TRACE_HANDOFF.md`, `docs/STUMP_153_TTBR1_PLAN.md` — roadmaps, gap analyses, work-in-progress thinking. |
| **Internal coordination** | `docs/ARCHITECTURE.md`, `docs/DEBUGGING_RUNBOOK.md`, `docs/INFRA.md`, `docs/L_ITEM_EVAL.md`, `docs/UBUNTU_SETUP.md` — operational docs covering the dev environment and shared brain across machines. |
| **Strategy** | `docs/SESSION_JOURNAL.md` (chronological dev log), `docs/DISCLOSURE_POSTURE.md` (the Tier 1/2/3 rules), `docs/LICENSING.md` (AGPL+commercial strategy). |

## Why this split exists

Sphragis is open-source (AGPL-3.0-or-later) and benefits from
public visibility, third-party citation, and grant eligibility.
But several categories of internal artifact represent trade
secrets or working-paper material whose competitive value
evaporates the moment they're public — chief among them the M4
hardware reverse-engineering and the historical pentest writeups
that map our attack surface.

Splitting them into a private companion repo preserves both
properties: **the OS itself is open; the hard-won
reverse-engineering and the internal security analysis aren't
volunteered for free.**
