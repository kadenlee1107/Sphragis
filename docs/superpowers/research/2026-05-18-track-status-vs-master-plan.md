STATUS: COMPLETE | date 2026-05-18 | author researcher-tracks

# Track-by-Track Status vs 2026-05-16 Master Plan

## TL;DR

Two calendar days into a 36-month plan, **engineering is 6-9 months ahead and paperwork is ~3 months behind**. Tracks **B, C4, F1-F2** delivered Month-3 / Month-9 / Month-12 deliverables in the first 24-hour push. Tracks **D, E** have not started but are still pre-window (plan §A2's plan starts D at Month 4, E at Month 6). Track **G** has full draft kits for SBIR Phase I + NLnet + STF + 2 VC + 3 ACT 3 + 3 DARPA + 3 launch posts, **all blocked on the one thing that hasn't moved: incorporation (A3)**. The single biggest insight: the Month-9 demo-ready gate is **artifact-met today** (M4 boot + Quote() + audit walk + threat model + cap statement + 20-slide deck) — what's actually gating Month 9 is the SBIR Phase I award, which requires SAM.gov, which requires the Delaware C-Corp filing nobody has filed yet.

---

## §1 — Per-track status

Symbol legend: ✅ ahead | 🟡 on-pace | ❌ behind (relative to the plan's month-by-month sequencing in [`docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`](../plans/2026-05-16-sphragis-gov-os-master-plan.md)).

| Track | Status | 1-line summary | Source |
|---|---|---|---|
| **A — Foundation (M0-3)** | 🟡 on-pace | A1 (license) + A2 (drop AGENT) complete; A4 (site + cap-stmt) scaffolded; **A3 (incorporation) not started** — pure founder bandwidth, gates G. | [`research/2026-05-17-day1-sweep…md` §1 + §2](2026-05-17-day1-sweep-and-funding-readiness.md) — "Apache-2.0 licensed ✅ NEW", "AGENT removed ✅ NEW"; §2 "Delaware C-Corp incorporation ❌", "SAM.gov ❌"; `marketing-site-scaffold/` exists; commit `95b54d52 site: ship marketing-site/`. |
| **B — Crypto + build-chain (M1-9)** | ✅ ahead | CNSA 2.0 algos shipping (ML-KEM-1024, ML-DSA-87, LMS verify-only, SHA-384/512, gov-strict profile, fail-closed RNG, boot KATs) at Day 2 — plan budgets Months 1-4 for B1. B3 reproducible builds **verified** (bit-identical SHA-256). FIPS-140-3 module boundary doc landed. **Misses: XMSS (B1.4), CMVP lab (B5).** | [`research/2026-05-17-day1-sweep…md` §1 "Crypto suite — CNSA 2.0 ready"](2026-05-17-day1-sweep-and-funding-readiness.md) + §2 "XMSS module ❌"; commits `aeedd534` (repro-build VERIFIED), `97131f58` (CRY-001/002 PARTIAL→HAVE), `541e0073` (CRY-003 PARTIAL→HAVE for LMS). |
| **C — Attestation + verification + audit (M3-18)** | ✅ ahead on artifact, 🟡 on-pace on proof | **C1 (attest)** live skeletal — Quote() + CaveIdentity registry + external verifier tool (commit `ae1bba63 SP-ATT-001 wire format`); SEP/Caliptra root chain still designed-only. **C4 (audit)** complete: HMAC-SHA-384, WORM (commit `c43e1a9f`), offline verifier, missing categories (commit `764e1ea5`), cave-scoped reads (commit `b7b849b2`). **C5** Cave-H2 closed + 2026-05-17 push added cap-token + MLS-label IPC (commit `617ea8f4`). **C2/C3** Verus harness scaffolded, SPEC.md written (`10203bd8`), proofs **not finished** — multi-session engineering. | [`research/2026-05-17-day1-sweep…md` §1 "Audit subsystem", "Attestation primitive"](2026-05-17-day1-sweep-and-funding-readiness.md); [`orchestrator/2026-05-17-push/session-report.md` §2 Eng-3](../orchestrator/2026-05-17-push/session-report.md). |
| **D — Productization UX (M4-24)** | 🟡 on-pace | Nothing built yet; plan starts D at Month 4. Multi-app WM, installer, settings/multi-user, package manager, POSIX toolbox all designed (`DESIGN_PRODUCTIZATION_UX.md`) — Day 2 is pre-window. | [`research/2026-05-17-day1-sweep…md` §2 "Productization (UX) gaps"](2026-05-17-day1-sweep-and-funding-readiness.md) — all 6 items ❌ / 🟡; plan §Track D budgets M4-24. |
| **E — Multi-hardware (M6-24)** | 🟡 on-pace | x86_64 port designed (`DESIGN_X86_64_PORT.md`), not built. CHERIoT-Ibex mapping designed (`DESIGN_CHERI_MAPPING.md`), not built. ARM server not started. Plan opens E at Month 6 — pre-window. | [`research/2026-05-17-day1-sweep…md` §2 "Hardware gaps"](2026-05-17-day1-sweep-and-funding-readiness.md). |
| **F — Docs + cert engineering (M4-36)** | ✅ ahead | **F1 done early**: Threat model (380 lines), Security Target (CC:2022 Part 1 conformant), Architecture-for-AOs all landed at Day 1-2 — plan budgets Months 4-9. **F2 ~40%**: NIST 800-53 inheritance matrix has AC+AU+CM+IA families complete (commit `a6941084`); needs SC+SI+MP+SA+SR+PT. **F3 STARTER** operator runbook. **F4 USENIX/NDSS paper** not started — see §4 deltas. **F5-F9** all pre-window. | [`research/2026-05-17-day1-sweep…md` §1 "Documentation surface"](2026-05-17-day1-sweep-and-funding-readiness.md); plan §F1 budgets M4-9. |
| **G — Procurement / funding (M3-36)** | ✅ ahead on drafts, ❌ behind on filings | **Drafts in hand for every Month-3-to-Month-15 G item**: SBIR Phase I (commit `fab21456`), 3 DARPA cold-pitches (`f4a753af`), 3 ACT 3 cold-pitches (`799bf3c7`), 3 VC cold-pitches (`a77ef6b0`), 20-slide demo deck (`8ffaf440`), cap-stmt v0 (`dbcb16bc`), VC pitch deck (`f65722ce`), 3-yr financial model (`44fabe49`). **Filings: zero.** G2 SBIR submit, G3 DARPA meetings, G4 ACT 3 teaming, G5 IWRP membership, G6 conferences — every one blocks on A3 incorporation. G9 (demo-bundle) is **already met** months early. | [`research/2026-05-17-day1-sweep…md` §6 "SBIR Phase I…READY TO FILE once incorporated"](2026-05-17-day1-sweep-and-funding-readiness.md); [`orchestrator/2026-05-17-push/session-report.md` §2 Funding + Outreach](../orchestrator/2026-05-17-push/session-report.md). |

---

## §2 — Unplanned work that landed since 2026-05-16

The master plan was written 2026-05-16 envisioning Track A items as Week-1 work. **173 commits later**, several items outside the plan have landed:

### Two new top-level deliverables (not in plan §Track A-G)

1. **SealFS rename (BatFS → SealFS)** — completed 2026-05-17 in two phases:
   - `351fe46b refactor(fs): rename BatFS → SealFS across the tree` (identifier sweep, byte constants preserved for compat)
   - `f9b03d48 refactor(fs): clean break — rename BATFS byte constants to SEALFS` (clean break, disk magic now `SEALFS\0\0`, SB_VERSION=2, pre-2026-05-17 disks fail magic check by design)
   - Plan never mentioned this rebrand — happened ad-hoc but is now permanent. Plan refs to "BatFS" in §C4 / §D4 / §E need updating.

2. **2026-05-17 multi-team push** — orchestrator-driven 95-minute, 5-parallel-team session producing 36 commits on `main` ([`orchestrator/2026-05-17-push/session-report.md`](../orchestrator/2026-05-17-push/session-report.md)). Specifically:
   - **Eng-1 TLS**: X.509 6-scenario regression suite (`de63c8b4`, `0653f6f3`, `24997b11`, `5b326610`, `3f4e2239`) — closes a gap implicit in plan §B but never explicitly tracked.
   - **Eng-2 SealFS**: key rotation + journal recovery + per-mount audit log (work landed in `e74803e8`, canonical log `1037281f`, status `31e2c2c0`) — three production-hardening capabilities that don't map to a plan subproject; they're "ahead of B/C plan items that didn't exist yet".
   - **Eng-3 Caves**: capability tokens + MLS-label enforcement on cross-cave IPC (`65a95ff5`, `8f35150a`, `617ea8f4`) — partially fulfills plan §C5.3 (CIPSO/CALIPSO labeling extension to IPC).
   - **3 ADRs** at `decisions/0001-0003.md`: team execution model, path corrections, **TeamCreate ceiling is 1 (not 4)** — operational learning that affects next session's orchestrator design.

### Funding drafts beyond the plan's named vehicles

Plan §Track G enumerates: SBIR Phase I, DARPA, GSA MAS, ACT 3, IWRP, In-Q-Tel, conferences. The push added drafts for:
- **NLnet NGI0 Commons Fund** €50K / 6mo (commit `666365aa`, cheat sheet `c4aaaeec`) — EU public-interest funding, not in the plan.
- **Sovereign Tech Fund** €120K / 9mo (commit `71055b13`, cheat sheet `341efc9a`) — German federal sovereign-tech funding.
- **GitHub Sponsors** profile v0 (`70fafecb`) — passive funding stream.
- **OpenSSF Alpha-Omega** $150K / 9mo with 3 work packages (`f6fa47e3`) — security-focused OSS grant.
- **GitHub Secure Open Source Fund** $10K + $10-150K Azure credits (`c546182d`) — replaces the plan's GitHub Accelerator slot (Accelerator 2024 was last AI-only cohort, poor fit per ANTI-002).

Per session-report §2 Funding, **worst-case all-five-award scenario: ~$340K + €170K over 6-9 months**. Parallel-funding overlap flagged for transparent intake-call disclosure: A-O WP3 (FIPS 140-3) ⇄ STF WP1 (CNSA 2.0); A-O WP2 (SLSA-L4) ⇄ Secure-OSS Fund Week 2 (supply-chain attestation).

### Outreach drafts (stretch goals)

Beyond the plan: 3 launch-post stretch drafts ready to fire — HN (`18b0d5df`, 2 variants), Lobsters (`b588304a`), LinkedIn (`bcf6b37a`, 2 variants). Sequencing-load-bearing per session-report §2 Outreach notes.

---

## §3 — What's slipping, and why

Working against the plan's decision-gate criteria in [`plan §Decision Gate Summary`](../plans/2026-05-16-sphragis-gov-os-master-plan.md):

### Month-3 gate (pass criteria: A1-A4 green; entity registered)

| Gate item | Status | Why |
|---|---|---|
| A1 Apache-2.0 tree CI-enforced | ✅ | Done 2026-05-16. `deny.toml` + `cargo-audit` in CI ([day-1 sweep §1 "Build chain"](2026-05-17-day1-sweep-and-funding-readiness.md)). |
| A2 AGENT removed; gov build coherent | ✅ | −5,856 LoC; `gov-strict` profile shipping ([day-1 sweep §1](2026-05-17-day1-sweep-and-funding-readiness.md)). |
| A3 Company incorporated; SAM/CAGE/UEI active | ❌ | **NOT STARTED.** Pure founder paperwork. Stripe Atlas ~3-7 days, SAM.gov 30-60 days, CAGE ~14 days. Plan §A3 budgets M1-3, so on-paper on-pace; but **gates every G item**. |
| A3 BIS encryption notification filed | 🟡 | Corrected template ready (`2026-05-17-bis-notification-template.md`, fixes CFR citation 15 CFR §742.15(b) and NSA address enc@nsa.gov per session-report §2 Funding). Awaiting Kaden to send. |
| A4 Marketing site live | 🟡 | Scaffold exists (`marketing-site-scaffold/` Hugo + 20-slide demo deck). Not yet deployed at sphragis.com. |
| A4 Capability statement v1 published | 🟡 | v0 pre-incorporation draft committed (`dbcb16bc`), references "Sphragis Inc. — in formation as of 2026-05-17". `sed`-friendly replace once Atlas issues Certificate (session-report §2 Funding). |

**Slip diagnosis:** A3 is the only true ❌ at this gate, and the plan **explicitly budgets Months 1-3 for it**, so we're technically on-pace at Day 2. The risk is the Month-9 demo-ready gate, which requires "1 SBIR Phase I award" — SBIR submission requires SAM.gov, which requires 30-60 days post-Atlas. If Atlas filing slips past ~M2, SBIR Phase I award (G2) plausibly slips past Month 9.

### Decision-gate criteria in detail

Plan §B Decision Gate (M9): "FIPS 140-3 lab engaged; CAVP submissions queued" — **not started**, but plan §B5 budgets M6-9, so we're 6+ months before the deadline.

Plan §C Decision Gate (M18): "Verus proof: cave capability non-interference + IPC info-flow" — **specs written, proofs not finished**. Plan §C2 budgets M3-12 for the dispatcher proof; we're at M0+2d. The day-1 sweep §2 explicitly flags "Completed Verus proof of capability dispatcher non-interference ❌" and "Completed Verus proof of IPC info-flow non-interference ❌". This is identified as ~2-4 weeks engineering work in [day-1 sweep §7](2026-05-17-day1-sweep-and-funding-readiness.md).

### Session-report §6 "Required for true deep green"

From [`orchestrator/2026-05-17-push/session-report.md` §6](../orchestrator/2026-05-17-push/session-report.md):
1. **QEMU smoke scripts not all executed.** Eng-2's `qemu_sealfs_rotation_selftest.py` and Eng-3's `qemu_cap_mls_selftest.py` need a session with `qemu-system-aarch64` to actually run (Mac Claude can't, Ubuntu Claude can).
2. **`cargo test --workspace` is vacuous** in this `no_std` repo. SP-TEST-001 (Cargo `[lib]` restructuring) is a precondition for `#[cfg(test)]` blocks to execute on host — plan doesn't name this.
3. **2 stale stashes** in `git stash list` from cross-team hygiene incidents (session-report §2 Eng-2 notes).

---

## §4 — Recommended plan deltas

Specific edits to fold into the next plan revision:

1. **Promote A3 incorporation to "Week 1 critical path"** alongside A1. The plan currently treats A3 as a 3-month founder track running in parallel; in practice it gates G2/G3/G4/G5/G6 absolutely. Add a Month-1 sub-gate: "Atlas filed by Day 30, SAM.gov submitted by Day 45, CAGE applied by Day 60." Without this the Month-9 demo gate carries SBIR risk.

2. **Add a Track G entry "G0 — Public-interest grant track"** covering NLnet NGI0 (€50K), STF (€120K), OpenSSF Alpha-Omega ($150K), GitHub Sponsors, GitHub Secure OSS Fund. These don't fit the plan's gov/VC framing but the drafts are ready. Note parallel-funding overlap rules (session-report §2 Funding) so intake calls have a coherent story.

3. **Tighten F4 USENIX/NDSS timeline.** Plan §F4 says "M6-12" but NDSS 2027 submits ~Sep 2026 — that's M4 from now. Either start drafting in M2 (using Verus specs that exist today even before the proof is finished) or accept slipping to USENIX Security 2027 (Feb 2027 submission, M9). Mark explicitly which target the plan is aiming for.

4. **Add C2.6.5 "Cargo test target restructuring (SP-TEST-001)"** as a precondition for `cargo test --workspace` to be non-vacuous. Currently Verus proof-CI and Kani regression both compile-test only on the aarch64-unknown-none target. Without a host `[lib]` target the §C2.7 "wire Verus check into CI" step can't run host-side unit tests. ~1-2 day refactor, blocks several downstream verification items.

5. **Update §C4 / §D4 / §E references "BatFS" → "SealFS"** throughout the plan. Disk magic `SEALFS\0\0`, SB_VERSION=2, HMAC domain `sealfs-integrity-v2`, Argon2id salt `sphragis-sealfs-v4`. Migration history is documented in `src/fs/sealfs_disk.rs` / `src/fs/sealfs.rs` / `src/main.rs`. The 2026-05-17 clean-break commits (`351fe46b`, `f9b03d48`) mean pre-rename disks fail magic check by design — pre-production, no shipped disks to break.

6. **Codify ADR-0003's "parallel `Agent{run_in_background=true}` subagents" model** in the master plan's resource section. The plan §Resource/Staffing Model assumes 3-5 engineers; the actual day-1 throughput was 5 parallel agents under 1 orchestrator. Worth a paragraph distinguishing "human engineers" from "agent subteams" so the Month-9/15/24 gates have realistic per-period output budgets.

7. **Per-team worktree discipline.** Session-report §4 documented 4 commit-hygiene incidents in 95 minutes, all from single-tree + `git add -A`. Plan §Resource/Staffing should mandate `git worktree add` per parallel agent OR explicit-paths-only `git add`. Cheap to add, prevents wrong-scope sweeps like `e74803e8` (which swept Eng-2's full sealfs impl under an Eng-3-status commit).

8. **Reframe Month-9 gate.** Plan §Decision Gate Summary M9 requires "demo bundle (PRC-010) all green; 1 SBIR Phase I award". The demo bundle is **already met at Day 2** (M4 boot + Quote() + audit walk + 20-slide deck + cap-stmt + threat model + security target). The real M9 gate is the SBIR Phase I award, which is a function of submission timing + reviewer luck (80-90% per-submission rejection). Plan should explicitly note "demo bundle ✅ already met; G2 award is the actual M9 critical-path".

9. **Add a Track-D pull-forward question.** Engineering tracks B/C/F shipped 6-9 months of plan output in 24-48 hours. Track D (UX) sits at "designed not built". If next session aims to maintain push tempo, **D1 (multi-app concurrent UI / window manager) is the highest-impact pull-forward candidate** because (a) plan §D1 budgets 5 engineer-months, (b) demo-bundle gravitas multiplies with a real WM, (c) no other track blocks on D1. Worth a "feasibility-of-pull-forward" sub-gate at M3 instead of waiting for M4 start.

10. **Verify CMVP timeline reality.** Plan §F5 budgets 12-30 months for FIPS 140-3 L1 cert. CMVP queue is currently ~12-18 months wall-clock and growing. If a sponsoring customer or grant (Alpha-Omega WP3) compresses the budget, plan should call out "submit by M9 to land cert by M30" rather than "M12-30" — the deadline is the queue, not the engineering.

---

*End of report. researcher-tracks, 2026-05-18.*
