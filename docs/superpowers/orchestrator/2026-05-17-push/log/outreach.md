# Outreach log — 2026-05-17 push

Mission: 9 cold-pitch email drafts in `docs/superpowers/outreach/`. See
`docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 (Outreach).

Append entries chronologically. Format per `multi-team-push.md` §5.
The very last entry on DoD-met must read `STATUS: COMPLETE` with
file paths + notes for Kaden.

---

## 2026-05-17 23:55 — outreach

Starting. Read plan §3 (Outreach charter), §4 (Operating Principles),
§5 (Coordination Protocol), §7 (Escalation Rules). Read ADR-0002
(path-corrections — no corrections affect Outreach). Read source
material: `act3-capability-brief-v1.md`, `vc-target-list-v1.md`,
`vc-pitch-deck-v1.md`, `darpa-pm-prep-v1.md`. Spot-read marketing
site for public-claim ceiling.

Note from marketing site: footer says "We do not solicit investment"
and the contact path is `sphragis-os@proton.me`. The VC pitch is a
private/founder track separate from the public marketing posture —
the public site is positioned as a security workstation product
("Request evaluation access"), not a VC fundraising page. My emails
must use the founder identity (Kaden, github.com/kadenlee1107) and
treat the public site as a credibility anchor (M4 boot evidence,
specs) rather than as an investor solicitation page.

Inbox (to-outreach) empty. Next: create `docs/superpowers/outreach/`
and draft ACT 3 cold-pitches file first (most concrete targets per
charter).

STATUS: IN_PROGRESS

## 2026-05-18 00:05 — outreach

Drafted ACT 3 cold-pitches → `docs/superpowers/outreach/2026-05-17-act3-cold-pitches.md`. 3 emails: AIS (lead — Rome NY, largest ACT 3 sub), CNF (R&D-pilot flexibility angle), Global InfoTek (ML-host / isolation framing). Each email has tailored ¶2 capability bullets + tailored ¶3 ask. Paragraph 1 follows the common template per source brief; paragraphs 2+3 fully tailored. Cited Capability Brief, marketing site, public GitHub. Founder signature blocks left for Kaden to fill (per Capability Brief §7 "Founder action").

Inbox poll (to-outreach.md): no new messages. Next: VC cold-pitches file.

STATUS: IN_PROGRESS

## 2026-05-18 00:18 — outreach

Drafted defense seed VC cold-pitches → `docs/superpowers/outreach/2026-05-17-vc-cold-pitches.md`. 3 emails: Shield Capital (Andrew Berenberg — defense-first thesis, Anduril/Hadrian/Saronic/Skydio fit), Lux Capital (Bilal Zuberi — frontier-tech + defense twin thesis), a16z American Dynamism (Katherine Boyle — "national interest" thesis). Paragraph 1 follows VC target list §5 template; paragraphs 2+3 fully tailored to each firm's portfolio + slide reference. Each email cites the public GitHub + the VC pitch deck v1 + the 3-year financial model. Noted the marketing-site "we do not solicit investment" footer and explained the VC-vs-public-product posture in the file header so Kaden knows my framing.

Inbox poll (to-outreach.md): still no new messages. Next: DARPA PM cold-pitches.

STATUS: IN_PROGRESS

## 2026-05-18 00:30 — outreach

Drafted DARPA PM cold-pitches → `docs/superpowers/outreach/2026-05-17-darpa-cold-pitches.md`. 3 emails: PROVERS (highest-fit per DARPA prep §2; framed as "application substrate where PROVERS tooling lands publication-grade evidence"), RSSC (lowest-friction entry; framed as "substrate the red team can't escape"), TRACTOR (modest-fit per DARPA prep §2; honest framing as "useful Rust target codebase, not a peer performer"). Chose TRACTOR as third instead of INSPECTA because DARPA prep §2 §INSPECTA explicitly counsels "Wait for PROVERS or RSSC engagement first" — a direct INSPECTA PM cold-pitch is contraindicated by the source.

These emails are more technical than VC/prime per charter and follow DARPA prep §8 red-flag-5 "lead with the truth about what doesn't work yet" discipline — each email has a "what does NOT work yet" paragraph (Verus proofs spec'd not complete; x86_64 designed not built; FIPS module-boundary not certified). Addressees are role-based pending Kaden verifying current PM names per DARPA prep §4.

DoD check: 3 files exist (act3, vc, darpa). 9 emails total (3 + 3 + 3). Each: subject + addressee + 3-paragraph body. ¶2 + ¶3 tailored. Public marketing site + GitHub + relevant source brief cited in each. Founder signature blocks left for Kaden per source brief instructions.

Inbox poll (to-outreach.md): still empty. Next: assess time available for stretch goals (HN / Lobsters / LinkedIn) per charter.

STATUS: IN_PROGRESS

## 2026-05-18 00:50 — outreach — DONE

DoD met. 9 cold-pitch emails delivered across 3 required files. All 3 stretch deliverables also delivered.

**DoD files (REQUIRED, 9 emails):**
- `docs/superpowers/outreach/2026-05-17-act3-cold-pitches.md` — 3 emails: AIS (Rome NY, largest ACT 3 sub), CNF Technologies (cyber R&D, smaller / flexible), Global InfoTek (ML threat detection / isolation host angle)
- `docs/superpowers/outreach/2026-05-17-vc-cold-pitches.md` — 3 emails: Shield Capital (Andrew Berenberg), Lux Capital (Bilal Zuberi), a16z American Dynamism (Katherine Boyle)
- `docs/superpowers/outreach/2026-05-17-darpa-cold-pitches.md` — 3 emails: PROVERS PM, RSSC PM, TRACTOR PM (chose TRACTOR over INSPECTA because DARPA prep §2 §INSPECTA explicitly counsels not to direct-cold INSPECTA before establishing DARPA-performer credibility elsewhere)

**Stretch files (NOT REQUIRED FOR DoD; explicitly marked STRETCH in each header):**
- `docs/superpowers/outreach/2026-05-17-hn-launch.md` — Hacker News post draft, 2 variants (Show HN vs regular submission). Recommendation in file: Variant B (regular).
- `docs/superpowers/outreach/2026-05-17-lobsters-launch.md` — Lobsters post draft with tag set + first-comment template.
- `docs/superpowers/outreach/2026-05-17-linkedin-announcement.md` — LinkedIn post draft, 2 variants (general engineering network vs defense / sovereign-tech network). Recommendation in file: post Variant A first, space Variant B 2–3 weeks later.

**Final commits this session (most recent first):**
- `bcf6b37a` outreach: LinkedIn announcement stretch draft v1
- `b588304a` outreach: Lobsters launch post stretch draft v1
- `18b0d5df` outreach: HN launch post stretch draft v1
- `f4a753af` outreach: DARPA PM cold pitches v1
- `a77ef6b0` outreach: defense seed VC cold pitches v1
- `799bf3c7` outreach: act3 cold pitches v1
- `e21776d8` outreach: starting log entry for 2026-05-17 push

**Notes for Kaden (rolled up from each file's "What Kaden does next" section):**

1. **Founder signature block.** Every email and every post leaves the founder signature block unfilled — name, email, phone, "Sphragis Inc. (Delaware C-Corp — incorporation in flight)" or equivalent. The Capability Brief §7 template is the canonical version; the VC pitch deck slide 16 is the alternate.

2. **Marketing site URL.** All drafts use `https://sphragis.com` as a placeholder. Confirm the actual public URL before any send / post.

3. **Verify named recipients.** VC partner rosters change; the Shield / Lux / a16z names (Berenberg / Zuberi / Boyle) are from VC target list v1 §1 as of 2026-05-17. DARPA PMs rotate (3–5 year terms); the program-page PM names must be verified per DARPA prep §4 (some are pseudonymous for OPSEC and the right addressee is the COR / TPOC on the most recent SAM.gov BAA listing). ACT 3 prime emails are addressed to roles because no specific BD contact is named in the source brief; warm intros, if Kaden has any, should replace the role-line.

4. **Sequencing — DO NOT fire everything at once.**
   - ACT 3 (per Capability Brief §6): AIS Week 1, then CNF + GiTec Week 3 if AIS slow.
   - VC (per VC target list §6): Shield + Lux Week 1, then a16z American Dynamism + others Week 2+ if no progress.
   - DARPA (per DARPA prep §7): cold-outreach to PROVERS + RSSC PMs M2–M3 (i.e. now), TRACTOR opportunistic 2–4 weeks behind. Don't fire all DARPA emails same day — PMs sometimes compare notes.
   - Launch posts (per HN draft §5 + Lobsters draft §1 + LinkedIn draft §1): space them out. HN Variant B first, Lobsters 24–48h later, LinkedIn Variant A in a third window. Don't post within 24h of any cold outreach (looks choreographed).

5. **Marketing-site posture mismatch.** The site footer says "We do not solicit investment." VC emails address this — if asked, the honest answer is the public site is positioned for security-eval buyers; investment solicitation is direct founder outreach. If this becomes a recurring friction point, the marketing-site copy can be updated later (out of scope for Outreach).

6. **DARPA cold-pitches use honesty discipline.** Per DARPA prep §8 red-flag #5 ("PM 1-on-1s reward technical honesty more than salesmanship"), each DARPA email has a paragraph that explicitly names gaps — Verus proofs spec'd not complete, x86_64 designed not built, FIPS module-boundary not certified. This is intentional; do not edit it out before send.

7. **TRACTOR email is intentionally framed as low-fit-but-honest** ("I'd value 30 minutes even if the outcome is a clean 'no fit'") — the goal is either a useful narrow conversation or a redirect referral to a higher-fit PM. Either is a win.

8. **No emails sent. No posts posted.** Strict per charter §3 Outreach "Out of scope: Send any email; post on HN/Lobsters/LinkedIn" and per CLAUDE.md leader prompt. All artifacts are drafts only.

**Quality gate check (§4 funding/outreach version):**
- Per-commit working tree: each of my 7 commits left a clean tree on the Outreach-touched files (unrelated WIP from eng teams existed at certain points but was not introduced or modified by Outreach commits).
- obsidian-sync hook: reported `done — N note(s) changed, 0 orphan(s) pruned` after every commit. No hook failures.

STATUS: COMPLETE

