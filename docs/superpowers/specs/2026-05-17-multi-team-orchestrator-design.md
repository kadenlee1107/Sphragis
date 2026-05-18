# Multi-Team Orchestrator Design — 2026-05-17 Push

**Status:** Approved (brainstorm 2026-05-17)
**Author:** Mac Claude + Kaden
**Plan:** `docs/superpowers/plans/2026-05-17-multi-team-push.md`

## Purpose

Capture the design decisions for a `/goal`-driven multi-team push that
runs in tmux on the user's terminal. The plan file (linked above) is
the operational instructions the leader/orchestrator executes; this
spec records *why* the plan looks the way it does.

## Mission

Push Sphragis forward across engineering, funding, and outreach in
**parallel**, leveraging Claude Code's Agent Teams feature. One
orchestrator session spawns up to 5 sub-teams, each tackling an
independent project, all coordinating through committed files in the
repo so that the obsidian-sync hook propagates everything to
`~/SPHRAGIS_VAULT/` automatically (single source of truth, full audit
trail).

## Decisions Locked In

### D1. Mission focus: everything in parallel

Engineering + funding + outreach run simultaneously. Higher
coordination cost, biggest blast radius. Justified because today's
end-of-day sweep showed all three areas have well-defined unblocked
work.

### D2. Team count: 5 (config ceiling)

`~/.claude/settings.json` → `claudeFlow.agentTeams.maxAgents: 5`. The
leader is the 6th agent. If `TeamCreate` enforces 5 strictly, the
leader merges Funding + Outreach into one Bizdev team; engineering
stays at 3 teams. Trade-off accepted.

### D3. Engineering allocation: TLS + SealFS + Caves

Three independent subsystems with zero file collisions:

- **Eng-1: TLS X.509 chain validation** — `src/net/tls/` — was #1 on
  post-no-browser roadmap. Headline networking unlock.
- **Eng-2: SealFS rotation + recovery + audit** — `src/fs/sealfs*.rs`
  — mature subsystem ready for hardening.
- **Eng-3: Caves capability tokens + MLS labels** — `src/caves/` —
  Bell-LaPadula/Biba label enforcement.

Rejected alternatives:
- *TLS + Secure boot + Scheduler* — secure boot too risky for one session.
- *TLS + SealFS + Verus harness* — Verus scaffolding overlaps with
  ongoing crypto work; defer to dedicated push.

### D4. Commit policy: fully autonomous

All 5 teams commit + push to `main` directly. Quality gates are
non-negotiable:

- TDD: test first, run-fail, implement, run-pass, commit
- `cargo test --workspace` green
- `cargo deny check` green
- `cargo audit --ignore RUSTSEC-2023-0071` green
- `cargo clippy --workspace -- -D warnings` green
- `cargo fmt --all --check` green
- post-commit `sync_obsidian.py` hook runs clean
- DCO sign-off on every commit
- Conventional commit format: `scope: subject`

No `--no-verify`, no `--force` push, no history rewrites.

Justified: trust in orchestrator + speed > caution of branch-only.
Quality gates compensate for direct-to-main risk.

### D5. Stop condition: per-team DoD, no wall-clock cap

`/goal` terminates naturally when the plan's work is complete. Per-team
DoDs are the stop signals. No 6-hour wall-clock cap (Kaden corrected
this during brainstorm). No token-budget signal — `/goal` manages that
natively.

### D6. Coordination protocol: vault-only, no SendMessage

All inter-team communication flows through committed files in
`docs/superpowers/orchestrator/2026-05-17-push/`. No direct
`SendMessage` between teams. Latency cost (commit + hook + poll ≈ 2-5
min per round-trip) accepted in exchange for full audit trail.

Teams poll their inbox file every ~5 min. Leader is the only entity
that writes to a team's inbox; teams write only to their own log and
to `inbox/to-leader.md`.

Decided over hybrid (SendMessage-as-doorbell) because audit
completeness wins over coordination speed. Sphragis is a security
project — auditability is the product.

### D7. Cargo.lock serialization

Three eng teams writing Rust will all touch `Cargo.lock` eventually.
Race condition risk on `main`. Leader serializes via an explicit
write-lock protocol: a team that needs to update Cargo.lock posts to
`inbox/to-leader.md`, waits for `inbox/to-eng-N.md` to say "lock
granted," does the work, commits, then posts "lock released" to
`inbox/to-leader.md`. Leader only grants one lock at a time.

## Open Risks

### R1. TeamCreate parallelism may be tighter than 5

Research confirmed `maxAgents: 5` in config but didn't confirm whether
leader counts toward the limit. Mitigation: leader detects via
`TeamCreate` failure and falls back to 4 spawned (merging
Funding+Outreach into Bizdev). Documented in plan §3.

### R2. Polling overhead on token budget

Each team polls its inbox every ~5 min. Even no-op polls burn tokens
(read-file + reasoning). Over a multi-hour session this could be
significant. Mitigation: teams check inbox only at natural break
points (after each TDD cycle, on commit, before starting new
sub-task) rather than on a strict timer. Documented in plan §5.

### R3. Three eng teams may starve each other on review/feedback

If Kaden steps away from tmux, no human is available to break
deadlocks. Mitigation: leader has explicit decision authority for
anything that can be decided from existing docs (CLAUDE.md, DESIGN_*,
M4_GROUND_TRUTH.md). Only ambiguities that can't be resolved from docs
escalate to Kaden. Documented in plan §7.

### R4. Obsidian-sync hook is critical infrastructure

If the hook fails during a commit, the vault falls out of sync and the
audit trail breaks. Mitigation: pre-flight verifies hook is installed;
leader monitors hook output on every commit; if hook fails 2x in a
row, halt the offending team and escalate.

### R5. Cargo.lock lock-grant protocol could deadlock

If a team posts a lock request and the leader doesn't poll its inbox
in time, the team waits forever. Mitigation: explicit 10-min timeout
on lock requests — if no grant arrives in 10 min, the team escalates
to `inbox/to-leader.md` with `URGENT:` prefix and falls back to
working on a non-Cargo task.

### R6. "Fully autonomous" + direct-to-main = bad commit lands publicly

A quality-gate slip means a broken `main`. Mitigation: gates are
exhaustive (test + deny + audit + clippy + fmt + hook). Belt-and-
suspenders. If a bad commit does land, recovery is `git revert` + a
new commit (per CLAUDE.md "create new commits, don't amend").

### R7. Private-repo leakage

Anything touching `~/sphragis-internal/` (M4_GROUND_TRUTH,
SESSION_JOURNAL, DISCLOSURE_POSTURE) without explicit instruction is a
disclosure-posture violation. Mitigation: plan §7 hard-stops any team
that tries to read or write `~/sphragis-internal/`; escalation only.

## Out of Scope

- Secure boot chain (Track C) — too big for one session, deferred.
- Verus formal-verification harness — overlaps with crypto work, deferred.
- Scheduler `block_on()` impl (post-no-browser roadmap #2) — small
  enough that a future session can drop it into one team.
- Captures cleanup (post-no-browser roadmap #3) — same.
- Marketing/community surfaces (HN/Lobsters posts, ONBOARDING.md
  improvements) — folded into Outreach team's charter as
  time-permitting "stretch goals," not core DoD.
- Anything that requires user credentials (BIS notification submission,
  Stripe Atlas, SAM.gov) — those stay on Kaden's plate; teams only
  *draft* materials.
