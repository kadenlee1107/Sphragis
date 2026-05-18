# Sphragis Multi-Team Push — 2026-05-17

> **YOU ARE THE LEADER/ORCHESTRATOR.** This file is your operating
> manual. Read it end-to-end before doing anything else, then execute
> the pre-flight checklist (§9), then run the coordination loop
> (§5–§7) until all team DoDs (§3) are met or you are interrupted.
>
> **No improvisation on the non-negotiables (§4) or escalation rules
> (§7).** Everything else, use your judgment grounded in the docs
> listed in §1.
>
> **Design rationale:** `docs/superpowers/specs/2026-05-17-multi-team-orchestrator-design.md`

---

## §1. Context Briefing — what Sphragis is

You are the leader of a 5-team parallel push on **Sphragis**, a
bare-metal Rust microkernel for Apple Silicon. The target machine is
an Apple M4 MacBook Pro 14" (Mac16,1 / J604 / T8132 "Donan"). We have
booted Sphragis on real M4 hardware (verified, not aspirational).

The project is mid-flight: security-first microkernel with isolated
user "caves" (processes), no in-tree browser, TLS-enforced networking
with kernel-mediated HTTPS for caves, SealFS encrypted filesystem
(AES-256-GCM-SIV), CNSA 2.0 post-quantum crypto (ML-KEM-1024 +
ML-DSA-87), fail-closed RNG, Apache-2.0 licensed, ~96K LoC.

### Required reading (read all of these before you spawn any team)

In this exact order:

1. **`CLAUDE.md`** (repo root) — onboarding, two-repo split, "which
   Claude are you" guidance, key facts you must not forget. You are
   Mac Claude for the purposes of file paths but conceptually you are
   the *orchestrator*.
2. **`~/sphragis-internal/docs/SESSION_JOURNAL.md`** — the last 3-5
   entries tell you what just happened and what's next.
3. **`~/sphragis-internal/docs/M4_GROUND_TRUTH.md`** — authoritative
   M4 reverse-engineering reference. Only relevant if a team touches
   hardware-adjacent code; you should still skim it.
4. **`~/sphragis-internal/docs/DISCLOSURE_POSTURE.md`** — Tier 1/2/3
   rules. Critical: nothing from `~/sphragis-internal/` ever gets
   read or written by a team. Only you (leader) ever read it, and
   only the public-track repo gets written.
5. **`docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`**
   — the 7-track, 35-sub-project master plan. Context for what each
   engineering team is contributing toward.
6. **`docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`**
   — end-of-day-1 sweep. Context for what the funding + outreach
   teams are contributing toward.
7. **`docs/superpowers/funding/2026-05-17-founder-action-checklist.md`**
   — paperwork roadmap. Tells you what the funding team should
   *draft* (vs. what stays on Kaden's plate to *file*).

### Useful background (read selectively if a team's work needs them)

- `DESIGN.md`, `DESIGN_CAVES.md`, `DESIGN_CAVE_ISOLATION.md`,
  `DESIGN_TLS_HARDENING.md`, `DESIGN_HTTPS_SYSCALL.md`,
  `DESIGN_CRYPTO.md` — subsystem design docs.
- `docs/superpowers/specs/2026-05-16-sphragis-gov-os-requirements.md`
  — 114 numbered requirements. The engineering teams should hit some
  of these incidentally; not every requirement maps to this session.

### Things you must not forget

- **`main` is the default branch** (renamed 2026-05-08 from
  `feat/js-engine-browser-posix` after the no-browser pivot).
- **The Mac is the target.** This session runs on the Mac. Ubuntu
  Claude is a different agent on a different machine; you do not
  coordinate with Ubuntu directly. If a team needs Ubuntu's help,
  escalate to Kaden.
- **Do NOT touch `~/sphragis-internal/`** from any team. Only the
  leader (you) reads it for context. Nothing gets written.
- **Pre-2026-05-17 disk images of SealFS will fail magic check by
  design.** The Eng-2 team needs to know this — it's not a bug.
- **`Cargo.lock` is shared across the workspace** — three eng teams
  will all want to update it. Serialize per §5.

---

## §2. Mission

Push Sphragis forward across engineering, funding, and outreach in
**parallel** by spawning 5 sub-teams that work independently and
coordinate exclusively through committed files in the repo (which
auto-sync to `~/SPHRAGIS_VAULT/` via the obsidian-sync hook).

### What "success" looks like at end-of-session

- 3 engineering subsystems advanced with code committed to `main`,
  each behind a coherent commit (or commit series) that passes all
  quality gates (§4).
- 4 funding/grants drafts created in `docs/superpowers/funding/`,
  ready for Kaden to review and submit.
- 9 cold-pitch emails (3 ACT 3 primes + 3 defense VCs + 3 DARPA PMs)
  drafted in `docs/superpowers/outreach/`.
- Complete vault audit trail in
  `docs/superpowers/orchestrator/2026-05-17-push/` (logs, inboxes,
  decisions, session report) mirrored to `~/SPHRAGIS_VAULT/_generated/`.
- Zero broken pushes on `main` (every commit passes §4 gates).
- Zero leaks of `~/sphragis-internal/` content into public repo.

### What "partial success" looks like

If any team can't reach its full DoD within the session, that's
acceptable as long as: (a) the team's work is committed cleanly so
future sessions can pick up, (b) the gap is documented in the team's
log AND in the final session report, (c) no broken state lands on
`main`.

---

## §3. Team Roster — 5 charters

You will spawn 5 teams. **Spawn them in this order**, in this order
of priority (so that if `TeamCreate` enforces a tighter ceiling than
expected, the highest-value teams come up first):

1. Eng-1 (TLS)
2. Eng-2 (SealFS)
3. Eng-3 (Caves)
4. Funding
5. Outreach

If `TeamCreate` fails after 4 spawns (i.e. parallelism ceiling is 4
spawned + 1 leader = 5 total), **merge Funding + Outreach into one
"Bizdev" team**. Do not drop an engineering team to keep both
business teams separate — engineering is the priority.

### Eng-1 — TLS X.509 chain validation

**Mission:** Add real X.509 certificate chain validation to
`src/net/tls/`. Currently the TLS stack does pinning but does not
validate the chain or check revocation. This was item #1 on the
post-no-browser roadmap.

**Reading list (read these before writing any code):**
- `DESIGN_TLS_HARDENING.md`
- `src/net/tls/` (entire directory — note `cert_pin.rs` already
  exists; you are extending alongside it, not replacing it)
- `src/net/tls/ca_certs/` (existing CA bundle, if any)

**Files you will likely create or modify:**
- New: `src/net/tls/x509.rs` — chain parsing + path validation
- New: `src/net/tls/x509_test.rs` (or `#[cfg(test)] mod tests` inside
  `x509.rs`) — TDD test scenarios
- Modify: `src/net/tls/mod.rs` to expose the new module
- Modify: TLS handshake code to call chain validation after pinning
  check (do not remove pinning — it's defense in depth)

**TDD test scenarios (add these BEFORE writing implementation):**
- `test_valid_chain_3_levels` — leaf → intermediate → root, all in
  bundle, all dates valid, signatures correct → returns Ok
- `test_chain_signature_mismatch` — leaf signed by a different key
  than intermediate's pubkey → returns specific error variant
- `test_chain_expired_intermediate` — intermediate `not_after` in the
  past → returns specific error variant
- `test_chain_unknown_root` — root not in trust store → returns
  specific error variant
- `test_chain_basic_constraints_violated` — leaf has CA:TRUE or
  intermediate has CA:FALSE → returns specific error variant
- `test_revocation_stub_returns_ok` — revocation check is stubbed to
  always return Ok for this milestone (real OCSP/CRL is a future
  session); document this in a code comment

**DoD (Definition of Done):**
- All 6 test scenarios above pass.
- Chain validation is called from the TLS handshake path (verified by
  a `#[test]` that asserts the call happens, or by tracing).
- `cargo test --workspace` green, all §4 quality gates green.
- A single commit (or coherent series) is pushed to `main` with
  message format `net/tls: add X.509 chain validation` (and any
  helper commits like `net/tls: scaffold x509 module`).
- A handoff note in `eng-1-tls.md` log marked `STATUS: COMPLETE` with
  pointers to commit SHAs and the file paths added/changed.

**Out of scope (do NOT do these in this session):**
- Real OCSP or CRL fetching (network-side revocation).
- Adding new CA certs to the trust bundle.
- Changing the pinning logic.
- Touching `Cargo.lock` to add new TLS crates — if you need a new
  crate, request lock via §5 first.

### Eng-2 — SealFS rotation + recovery + audit

**Mission:** Add three production-hardening capabilities to SealFS:
key rotation, journal recovery on mount, and a per-mount audit log.

**Reading list:**
- `src/fs/sealfs.rs`
- `src/fs/sealfs_disk.rs` (the disk format — note SB_MAGIC was
  bumped on 2026-05-17 to `*b"SEALFS\0\0"`, SB_VERSION=2)
- `DESIGN_AUDIT_WORM.md` (audit-log design pattern)
- `DESIGN_CRYPTO.md` (key-management discipline)

**Files you will likely create or modify:**
- New: `src/fs/sealfs_rotation.rs` — rotation primitives
- New: `src/fs/sealfs_journal.rs` — journal recovery
- New: `src/fs/sealfs_audit.rs` — per-mount audit log
- Modify: `src/fs/sealfs.rs` — wire the three new modules into mount/
  unmount/sync paths
- Modify: `src/fs/sealfs_disk.rs` — add `audit_seq` and
  `journal_state` fields to a new on-disk structure if needed
  (carefully — preserve SB_MAGIC compatibility; if you bump format,
  document the migration like the existing comments do)

**TDD test scenarios:**
- `test_rotation_old_data_still_decryptable` — write block under key
  A, rotate to key B, read block back, expect plaintext (old key was
  retained in a key-history slot).
- `test_rotation_new_data_uses_new_key` — after rotation, write a
  new block, confirm it was encrypted under key B (by attempting to
  decrypt with key A and expecting failure).
- `test_journal_recovery_after_partial_write` — simulate a torn
  write (write the first half of a journal entry, then the rest is
  zeroed), mount the volume, expect the partial entry to be rolled
  back and the FS to be in a consistent pre-write state.
- `test_audit_log_records_mount` — mount the volume, check the
  audit log, expect a `MountEvent` entry with timestamp and key
  generation.
- `test_audit_log_records_rotation` — rotate, expect a
  `RotationEvent` with old + new key generation IDs.
- `test_audit_log_append_only` — attempt to overwrite a past entry,
  expect a hard error.

**DoD:**
- All 6 test scenarios above pass.
- `cargo test --workspace` green, all §4 quality gates green.
- A coherent commit series on `main` with messages like
  `fs/sealfs: add key rotation primitive`,
  `fs/sealfs: add journal recovery on mount`,
  `fs/sealfs: add per-mount audit log`.
- A handoff note in `eng-2-sealfs.md` log marked `STATUS: COMPLETE`
  with commit SHAs.

**Out of scope:**
- Disk-format migration tooling for older images (pre-2026-05-17
  images fail magic check by design — that's not a bug).
- Networked audit-log replication.
- Multi-volume key management.

### Eng-3 — Caves capability tokens + MLS labels

**Mission:** Add capability tokens and MLS (multi-level security)
label enforcement to the caves IPC path. Enforces Bell-LaPadula
("no read up") and Biba ("no write up") semantics on cross-cave
calls.

**Reading list:**
- `DESIGN_CAVES.md`
- `DESIGN_CAVE_ISOLATION.md`
- `src/caves/` (entire directory — especially `cave.rs`,
  `cave_private.rs`, `mls_ipc.rs`, `bridge.rs`)

**Files you will likely create or modify:**
- New: `src/caves/cap_token.rs` — capability-token type, mint, verify
- New: `src/caves/mls_label.rs` — MLS label type, dominance check
- Modify: `src/caves/mls_ipc.rs` — enforce label dominance + cap
  tokens on every cross-cave call
- Modify: `src/caves/cave.rs` — each cave has a label at spawn time;
  cannot be changed afterward
- Modify: `src/caves/bridge.rs` — bridges propagate caller's cap
  tokens but apply the *callee's* label policy

**TDD test scenarios:**
- `test_label_dominance_self` — a label always dominates itself.
- `test_label_dominance_strict` — UNCLASSIFIED is dominated by
  CONFIDENTIAL is dominated by SECRET is dominated by TOPSECRET
  (with appropriate compartments).
- `test_bell_lapadula_read_up_denied` — cave at CONFIDENTIAL tries
  to read from SECRET cave → IPC call returns
  `Err(LabelViolation::ReadUp)`.
- `test_biba_write_up_denied` — cave at CONFIDENTIAL tries to write
  to UNCLASSIFIED cave → IPC call returns
  `Err(LabelViolation::WriteUp)` (Biba protects integrity, not
  confidentiality; rule is opposite of Bell-LaPadula's read).
- `test_cap_token_forge_attempt` — attempt to construct a cap token
  with a wrong MAC → `verify` returns `Err`.
- `test_cap_token_valid_call_passes` — mint a valid cap token, make
  a call across caves with dominated labels, expect Ok.

**DoD:**
- All 6 test scenarios above pass.
- `cargo test --workspace` green, all §4 quality gates green.
- A coherent commit series on `main`.
- A handoff note in `eng-3-caves.md` log marked `STATUS: COMPLETE`
  with commit SHAs.

**Out of scope:**
- Persistent label history (audit-log integration with Eng-2's audit
  log is a future session).
- Dynamic relabeling (labels are fixed at spawn).
- Network-side label propagation.

### Funding — drafts for grants + paperwork

**Mission:** Produce four submission-ready drafts in
`docs/superpowers/funding/`. Kaden submits; you only draft.

**Reading list:**
- `docs/superpowers/funding/2026-05-17-day1-sweep-and-funding-readiness.md`
- `docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
- `docs/superpowers/funding/2026-05-17-stf-form-field-answers.md`
  (model for tone/length/grounding)
- `docs/superpowers/funding/2026-05-17-nlnet-form-field-answers.md`
  (same)
- `docs/superpowers/funding/2026-05-17-sbir-phase-1-afwerx-open-v0.md`
  (technical narrative reuse)
- `marketing-site/index.html` (public claims — your drafts must not
  exceed what's said here publicly without good reason)

**Drafts to produce (4 files):**

1. **OpenSSF Alpha-Omega application draft** →
   `docs/superpowers/funding/2026-05-17-openssf-alpha-omega-v0.md`
   - Look up Alpha-Omega's current application format
     (WebSearch/WebFetch is allowed for this).
   - Use the SBIR/STF/NLnet drafts as the technical narrative source.
   - Frame Sphragis as foundational open-source security
     infrastructure (CNSA 2.0, fail-closed RNG, SealFS, caves).
   - Ask for the typical grant amount Alpha-Omega awards (research
     this); justify the ask with what you'd ship in 6-12 months.

2. **GitHub Accelerator application draft** →
   `docs/superpowers/funding/2026-05-17-github-accelerator-v0.md`
   - Look up the current GitHub Accelerator cohort window. If a
     cohort is open, draft the application. If no cohort is open,
     produce a "ready when next cohort opens" draft with a header
     noting that.

3. **BIS encryption notification template** →
   `docs/superpowers/funding/2026-05-17-bis-notification-template.md`
   - For ECCN 5D002 publicly available open-source crypto, BIS
     requires a one-time email notification to crypt@bis.doc.gov +
     enc@nsa.gov per 15 CFR 740.17(b)(2).
   - Draft the email template (subject line, body, attachments to
     reference). Include Sphragis's URL, license, brief description,
     and the assertion that it's publicly available under
     Apache-2.0.
   - Do NOT send the email yourself — only Kaden does that. Mark the
     draft `STATUS: DRAFT — KADEN TO SEND`.

4. **GitHub Sponsors profile copy** →
   `docs/superpowers/funding/2026-05-17-github-sponsors-profile.md`
   - Profile bio, tier descriptions, goals/milestones.
   - 3-tier minimum: $5/mo (supporter), $25/mo (named in
     CONTRIBUTORS.md), $100/mo (logo on README).
   - Markdown ready to paste into GitHub Sponsors signup.

**DoD:**
- All 4 drafts exist, each with clear status header (`STATUS: DRAFT
  v1`), date, author (`Funding Team`), and a "what Kaden does next"
  section at the bottom.
- Each draft references existing primary sources (the SBIR/STF/NLnet
  drafts, the marketing site, the master plan) — do not invent
  numbers or claims.
- A handoff note in `funding.md` log marked `STATUS: COMPLETE` with
  pointers to all 4 file paths.

**Out of scope:**
- Submitting any application. Kaden submits.
- Sending the BIS email. Kaden sends.
- Touching `~/sphragis-internal/`.

### Outreach — 9 cold-pitch emails

**Mission:** Produce 9 personalized cold-pitch emails (3 ACT 3
primes, 3 defense VCs, 3 DARPA PMs) ready for Kaden to send.

**Reading list:**
- `docs/superpowers/funding/2026-05-17-act3-capability-brief-v1.md`
- `docs/superpowers/funding/2026-05-17-vc-target-list-v1.md`
- `docs/superpowers/funding/2026-05-17-vc-pitch-deck-v1.md`
- `docs/superpowers/funding/2026-05-17-darpa-pm-prep-v1.md`
- `marketing-site/index.html`

**Emails to produce:**

Create `docs/superpowers/outreach/` directory if missing.

1. **ACT 3 prime cold-pitches** →
   `docs/superpowers/outreach/2026-05-17-act3-cold-pitches.md`
   - One email each to: AIS (Adaptive Intelligence Solutions), CNF
     (Charles Newcomb / CNF Industries), Global InfoTek.
   - Each email: subject line, addressee (role-based if a specific
     name isn't in the brief), 3-paragraph body: (1) why we're
     reaching out, (2) Sphragis 1-line summary + 3 capability bullets
     tailored to that prime's portfolio, (3) ask (15-min intro call).
   - Link to public marketing site and the master plan if
     appropriate.

2. **Defense seed VC cold-pitches** →
   `docs/superpowers/outreach/2026-05-17-vc-cold-pitches.md`
   - One email each to: Shield Capital, Lux Capital, a16z American
     Dynamism.
   - Each email: subject line, addressee (partner from
     `vc-target-list-v1.md` if listed), 3-paragraph body framed for
     the firm's specific thesis. Reference the VC pitch deck and
     financial model.

3. **DARPA PM cold-pitches** →
   `docs/superpowers/outreach/2026-05-17-darpa-cold-pitches.md`
   - One email each to: PROVERS PM, RSSC PM (and one more if a third
     is identified in `darpa-pm-prep-v1.md`).
   - Each email: subject line, addressee, 3-paragraph body framed
     around the specific program's interests. These should be more
     technical than the VC/prime emails.

**DoD:**
- All 3 files exist, totaling 9 emails, each with subject + addressee
  + body, personalized to its target (no copy-paste boilerplate
  across emails of different categories — paragraph 1 may be similar
  but paragraphs 2 and 3 must be tailored).
- Each email cites the public marketing site and existing brief/deck
  appropriately.
- A handoff note in `outreach.md` log marked `STATUS: COMPLETE` with
  pointers to all 3 file paths.

**Stretch (if time permits, NOT required for DoD):**
- HN launch post draft → `docs/superpowers/outreach/2026-05-17-hn-launch.md`
- Lobsters launch post draft → `docs/superpowers/outreach/2026-05-17-lobsters-launch.md`
- LinkedIn announcement draft →
  `docs/superpowers/outreach/2026-05-17-linkedin-announcement.md`

If the team chooses to do any stretch goals, they must be marked
explicitly as STRETCH in their handoff note so Kaden can review
separately.

**Out of scope:**
- Sending any email. Kaden sends.
- Posting on HN/Lobsters/LinkedIn. Kaden posts.
- Inventing claims not supported by the marketing site or existing
  drafts.

---

## §4. Operating Principles — non-negotiable

**Every team operates under these rules. Quality gates are HARD —
do not push if any gate fails. Fix and re-test.**

### Branch + commit discipline

- All work on `main`. No feature branches in this session.
- Every commit must:
  - Pass all quality gates (below)
  - Have a DCO sign-off (`Signed-off-by: <name> <email>`)
  - Use conventional commit format: `scope: subject` (e.g.
    `net/tls: add chain parser`, `funding: openssf draft v0`)
  - Be a NEW commit. **No `--amend`. No `--force` push. No history
    rewrites.**
- After each commit, the `post-commit` hook auto-runs
  `scripts/sync_obsidian.py`. If the hook fails, halt the offending
  team and escalate to Kaden (§7).

### Quality gates (must all pass before push)

For engineering teams (Eng-1, Eng-2, Eng-3) on every Rust-touching
commit:

1. `cargo test --workspace` — all tests green
2. `cargo deny check` — no new policy violations
3. `cargo audit --ignore RUSTSEC-2023-0071` — no vulns (the
   `RUSTSEC-2023-0071` ignore is pre-existing; do not add others
   without writing an ADR in `decisions/`)
4. `cargo clippy --workspace --all-targets -- -D warnings` — lint
   clean
5. `cargo fmt --all --check` — formatted

For all teams (engineering + funding + outreach) on every commit:

6. `git status` reports a clean tree post-commit (no untracked or
   modified files left behind unintentionally)
7. The obsidian-sync `post-commit` hook reports
   `[obsidian-sync] done — N note(s) changed, 0 orphan(s) pruned`

If any gate fails on a push attempt:

- **Engineering team**: stop, write the failure to the team's log
  with the exact gate name and error output, fix the failure, re-run
  the full gate set, and try again.
- **All teams**: if the same gate fails 3 times on the same commit
  attempt, write `URGENT: gate-failure escalation` to
  `inbox/to-leader.md` and halt that team.

### Commit early and often

Every meaningful step is a commit. Specifically:

- Every TDD red→green cycle is one commit (the test + the
  implementation that makes it pass).
- Every status update, log entry, inbox message, or ADR is committed
  before moving on.
- Do not accumulate uncommitted changes. Uncommitted work is
  invisible to the vault and therefore invisible to the audit trail
  and to Kaden.

### TDD is mandatory for engineering

- Write the failing test FIRST.
- Run the test, confirm it fails for the *expected* reason (not a
  compile error from a missing module).
- Write the minimal implementation that makes the test pass.
- Run the test, confirm it passes.
- Run the full `cargo test --workspace`, confirm everything is still
  green.
- Commit.

Skipping TDD is a §7 violation.

### Things you must NEVER do

- Run any command with `--no-verify` (skips hooks).
- Run `git push --force` or `git push --force-with-lease` to `main`.
- Run `git reset --hard` to discard work that hasn't been
  acknowledged in Kaden's inbox.
- Run `git rebase -i` or any interactive git command.
- Modify `.git/config`.
- Read or write anything inside `~/sphragis-internal/`.
- Read or write any user credentials, API tokens, or env files.
- Submit any application, send any email, or post to any external
  service. You only *draft*.
- Add a new RUSTSEC ignore without an ADR.
- Touch `Cargo.lock` without first acquiring the lock per §5.

---

## §5. Coordination Protocol — vault-mediated, no SendMessage

**All inter-team communication flows through committed files in
`docs/superpowers/orchestrator/2026-05-17-push/`.** No direct
`SendMessage` between teams or from the leader to a team.

The leader briefs each team via the Agent tool's `prompt` parameter
at spawn time (one-shot briefing). Everything after that is
vault-mediated.

### Team execution model — long-running preferred, subagent-per-task fallback

The design assumes each team is a **long-running agent** inside a
`TeamCreate`-created team. The agent stays alive, does TDD work, polls
its inbox at natural break points, and writes to its log + the
leader's inbox as needed. The agent only exits when DoD is met (it
writes `STATUS: COMPLETE` to its log as its last act).

If `TeamCreate` is unavailable or its agents are one-shot (return
after a single response rather than staying alive), **fall back to
the subagent-per-task pattern**:

- Leader spawns a fresh subagent via the Agent tool with the team's
  full charter + a directive: "do one TDD cycle (test, implement,
  commit) and then exit, leaving a status entry in your log file
  describing what you did and what's next."
- After the subagent exits, leader reads the team's log to see what
  was done, decides if more work is needed, and spawns the NEXT
  subagent for that team with an updated prompt that includes the
  log state.
- Coordination is unchanged: still vault-mediated, still no
  SendMessage. The difference is the leader's loop becomes
  spawn-monitor-respawn rather than spawn-once-monitor-forever.

The leader chooses the execution model during pre-flight (§9 step 6)
based on what `TeamCreate` actually does. The choice is recorded in
ADR `0001-team-execution-model.md`.

### Workspace layout (you create this in pre-flight, §9)

```
docs/superpowers/orchestrator/2026-05-17-push/
├── status.md                # leader-maintained dashboard
├── log/                     # append-only per-team logs
│   ├── leader.md
│   ├── eng-1-tls.md
│   ├── eng-2-sealfs.md
│   ├── eng-3-caves.md
│   ├── funding.md
│   └── outreach.md
├── inbox/                   # vault-mediated messages
│   ├── to-leader.md         # teams append, leader polls
│   ├── to-eng-1.md          # leader writes, eng-1 polls
│   ├── to-eng-2.md
│   ├── to-eng-3.md
│   ├── to-funding.md
│   └── to-outreach.md
└── decisions/               # ADR-style records
    └── (created as needed)
```

### Log file format

Each team appends to its log on every meaningful step. Format:

```
## YYYY-MM-DD HH:MM — <team-name>

<one short paragraph describing what just happened, what file(s) were
touched, what commit SHA was produced (if any), what's next>

STATUS: IN_PROGRESS | BLOCKED | COMPLETE
```

The very last entry in a team's log when DoD is met must be:

```
## YYYY-MM-DD HH:MM — <team-name> — DONE

DoD met. Final commit(s): <SHA list>. Files: <path list>. Notes for
Kaden: <anything he should know>.

STATUS: COMPLETE
```

### Inbox file format

`inbox/to-leader.md` is append-only; teams add at the bottom. Each
message:

```
## YYYY-MM-DD HH:MM — from <team-name>

URGENT | NORMAL: <brief subject>

<body — what you need, why, what you'll do if you don't hear back
within 10 min>
```

`inbox/to-<team>.md` is written by the leader. Each message:

```
## YYYY-MM-DD HH:MM — leader

<subject>

<body — directive or answer to a question>
```

### Status.md format

Leader updates this every 10-15 min OR after any significant event.
Single source of truth dashboard. Format:

```markdown
# 2026-05-17 Push Status — last update HH:MM

## Leader
Current focus: <one line>

## Eng-1 (TLS)
Last update: HH:MM
Status: IN_PROGRESS | BLOCKED | COMPLETE
Current task: <one line>
Commits this session: N
Blocker (if any): <one line>

## Eng-2 (SealFS)
[same shape]

## Eng-3 (Caves)
[same shape]

## Funding
[same shape]

## Outreach
[same shape]

## Open inboxes-to-leader: <count of unread messages>
## Cargo.lock holder: <team name or "none">
```

### Polling cadence

Teams check `inbox/to-<team>.md` at natural break points:
- After completing a TDD cycle (eng teams)
- After committing a draft section (funding/outreach teams)
- Before starting a new sub-task
- After every commit (immediately)

**No timer-based polling.** Teams should not waste tokens reading the
file every N minutes if nothing changed. The natural break points
are frequent enough.

Leader polls all `inbox/to-leader.md` messages at the same cadence:
after every status.md update.

### Cargo.lock lock-grant protocol

`Cargo.lock` is shared workspace state. Three eng teams might
collide. To prevent races:

**To acquire the lock**, an eng team writes to `inbox/to-leader.md`:

```
## HH:MM — from eng-N

NORMAL: cargo.lock acquire request

Need to update Cargo.lock to add <crate-name> for <reason>. Will
release within 10 min. If no grant within 10 min I will escalate
URGENT and fall back to a non-Cargo task.
```

**Leader grants** by writing to `inbox/to-eng-N.md`:

```
## HH:MM — leader

cargo.lock GRANTED to eng-N. Release by writing "released" to
inbox/to-leader.md when done. No other team will be granted the lock
until you release.
```

Leader updates `status.md` → `Cargo.lock holder: eng-N`.

**Team releases** by writing to `inbox/to-leader.md`:

```
## HH:MM — from eng-N

NORMAL: cargo.lock release

Done. New Cargo.lock committed at <SHA>.
```

Leader updates `status.md` → `Cargo.lock holder: none`. Next pending
request (if any) gets granted.

**Only one lock granted at a time.** Pending requests are queued
FIFO in `status.md` under a `## Cargo.lock queue` section.

### Decision records (ADRs)

Any decision that future-Kaden or future-Claude would benefit from
seeing gets an ADR in `decisions/`. Format:

```markdown
# ADR-NNNN: <title>

Date: 2026-05-17 HH:MM
Decider: leader | <team-name>
Status: accepted

## Context
<a paragraph>

## Decision
<what we decided, in one line>

## Consequences
<what this enables, what it precludes>
```

Number them sequentially: `0001-<slug>.md`, `0002-<slug>.md`, etc.

Examples of decisions that warrant an ADR:
- A team chose one of two valid approaches and the choice
  constrains future work.
- A team discovered a `Cargo.toml` config that affects all teams.
- The leader merged Funding + Outreach due to the 5-agent ceiling.
- A team had to add a new RUSTSEC ignore.

---

## §6. Stop Conditions

### Per-team DoD met

A team's DoD is met when:
- All test scenarios listed in §3 for that team pass (eng teams), OR
- All draft files listed in §3 for that team exist (funding/outreach)
- AND a `STATUS: COMPLETE` entry has been written to that team's log

When a team is COMPLETE, the leader:
1. Updates `status.md` to mark the team done.
2. Decides whether to reallocate the slot. Default: leave the slot
   idle (don't risk introducing collisions late in the session).
   Only reallocate if there's a clear, scoped task ready to go.

### Full session complete

When all 5 teams are COMPLETE:
1. Leader reads every team's log.
2. Leader writes `docs/superpowers/orchestrator/2026-05-17-push/session-report.md`
   summarizing what each team did, commit SHAs, drafts produced, any
   gaps, any open ADRs.
3. Leader commits the session report.
4. Leader writes a final `STATUS: SESSION_COMPLETE` entry to
   `log/leader.md`.
5. `/goal` exits naturally.

### Interrupted by Kaden

If Kaden interrupts in tmux at any time:
1. Leader writes a `STATUS: INTERRUPTED` entry to `log/leader.md`
   with timestamp.
2. Leader does not attempt to "wrap up" — Kaden's interrupt means
   stop now. The vault audit trail will preserve what was done so
   far.

---

## §7. Escalation Rules

Escalate to Kaden by writing to `inbox/to-leader.md` from the team's
log AND including the prefix `URGENT:` in the message subject AND
writing a corresponding entry in `log/leader.md` so it appears in
the vault sync.

If a `PushNotification` tool is available, the leader sends a brief
notification to Kaden's phone after any URGENT escalation. The
notification should reference the inbox path so he can read context.

### Hard escalations (halt the team, notify Kaden)

- A team attempts to read or write `~/sphragis-internal/`.
- A team attempts to run a prohibited command (`--no-verify`,
  `--force`, etc.).
- A team's commit lands a quality-gate failure on `main` despite
  the gates being run (i.e. a bug in the gate itself).
- A team has had 3 quality-gate failures on the same commit attempt.
- A team is BLOCKED for more than 30 min on a single issue.
- A team's reading list points at a file that doesn't exist.
- Anything related to user credentials, API tokens, secrets.
- A team encounters a situation the docs don't cover and that you,
  the leader, also can't decide from the docs.

### Soft escalations (note but don't halt)

- Two teams want `Cargo.lock` at the same time (queue per §5).
- A team's TDD test fails for an *unexpected* reason (compile error,
  panic in unrelated code). Team must investigate before continuing.
- A team writes a "Question for Kaden" entry in its log — these
  don't halt the team; they get rolled up in the session report.

---

## §8. Deliverables

At end of session, the following must exist on `main`:

### Code

- N commits across `src/net/tls/`, `src/fs/sealfs*`, `src/caves/`
  implementing the per-team DoDs in §3.
- All commits pass §4 quality gates.

### Drafts

In `docs/superpowers/funding/`:
- `2026-05-17-openssf-alpha-omega-v0.md`
- `2026-05-17-github-accelerator-v0.md`
- `2026-05-17-bis-notification-template.md`
- `2026-05-17-github-sponsors-profile.md`

In `docs/superpowers/outreach/`:
- `2026-05-17-act3-cold-pitches.md`
- `2026-05-17-vc-cold-pitches.md`
- `2026-05-17-darpa-cold-pitches.md`
- (stretch) HN, Lobsters, LinkedIn drafts if Outreach team had time

### Audit trail

In `docs/superpowers/orchestrator/2026-05-17-push/`:
- `status.md` with final state
- 6 log files (`log/leader.md`, `log/eng-1-tls.md`, etc.) each
  ending in `STATUS: COMPLETE` (or `STATUS: INTERRUPTED` if Kaden
  killed the session)
- All inbox files preserving the conversation history
- ADRs in `decisions/` for any significant choices
- `session-report.md` written by the leader at session end

All of the above auto-syncs to `~/SPHRAGIS_VAULT/_generated/` via the
obsidian-sync post-commit hook. The vault is the authoritative audit
trail.

---

## §9. Pre-flight Checklist (do these first, in order)

Run these steps BEFORE spawning any team. If any step fails, halt
and escalate to Kaden.

1. **Read the required-reading list in §1 in full.** Do not skim.
   Specifically:
   - `cat CLAUDE.md`
   - `cat ~/sphragis-internal/docs/SESSION_JOURNAL.md` (read the last
     few entries closely; older entries are context)
   - `cat ~/sphragis-internal/docs/M4_GROUND_TRUTH.md` (skim — only
     relevant if an eng team hits hardware-adjacent code)
   - `cat ~/sphragis-internal/docs/DISCLOSURE_POSTURE.md` (read in
     full — you cannot violate this)
   - `cat docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
     (skim — context for what eng teams contribute toward)
   - `cat docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
     (read in full — context for funding + outreach teams)
   - `cat docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
     (read in full — context for funding team's "draft vs send"
     boundary)

2. **Verify clean working state.**
   - `git status` — must be clean (no untracked, no modified).
   - `git branch --show-current` — must report `main`.
   - `git log --oneline -10` — eyeball recent commits to confirm we
     start from a known state.

3. **Verify obsidian-sync hook is installed.**
   - `ls .git/hooks/post-commit` must exist.
   - If missing: `sh scripts/install_hooks.sh` to install. Re-check
     existence. If still missing, halt and escalate.

4. **Probe vault sync end-to-end.**
   - Note the current vault index timestamp:
     `ls -l ~/SPHRAGIS_VAULT/_generated/_index.md` → record mtime.
   - Make a trivial test commit (e.g. create
     `docs/superpowers/orchestrator/2026-05-17-push/.gitkeep` and
     commit with message
     `chore(orchestrator): scaffold push workspace`).
   - Re-check the vault index timestamp — must have advanced.
   - If not advanced, the hook ran but didn't propagate; halt and
     escalate.

5. **Scaffold the workspace.**
   - Create directories:
     - `docs/superpowers/orchestrator/2026-05-17-push/`
     - `docs/superpowers/orchestrator/2026-05-17-push/log/`
     - `docs/superpowers/orchestrator/2026-05-17-push/inbox/`
     - `docs/superpowers/orchestrator/2026-05-17-push/decisions/`
   - Create empty files (with a one-line header so they're not
     literal-empty):
     - `status.md` (initial dashboard skeleton — see §5)
     - `log/leader.md` (header + one entry: "Pre-flight starting")
     - `log/eng-1-tls.md`, `log/eng-2-sealfs.md`, `log/eng-3-caves.md`,
       `log/funding.md`, `log/outreach.md` (each with a header noting
       the team will write here)
     - `inbox/to-leader.md` (empty header)
     - `inbox/to-eng-1.md`, `inbox/to-eng-2.md`, `inbox/to-eng-3.md`,
       `inbox/to-funding.md`, `inbox/to-outreach.md` (each empty
       header)
   - Commit all of this with message
     `chore(orchestrator): scaffold 2026-05-17 push workspace`.

6. **Probe TeamCreate ceiling + execution model.**
   - `TeamCreate` is a deferred tool in this environment. Load it
     first via `ToolSearch` with query `select:TeamCreate,TeamDelete`
     (and load `Agent`'s `team_name` parameter docs too if it's also
     deferred).
   - Attempt to create a test team. If the call errors immediately,
     record the error and fall back to the subagent-per-task model
     described in §5 — record this choice in ADR `0001-team-execution-model.md`
     and treat each "team" hereafter as a series of subagent spawns
     rather than one long-running agent.
   - If `TeamCreate` works, do one minimal end-to-end test: create a
     test team, spawn a trivial agent in it via the Agent tool with
     `team_name` set, observe whether the agent stays alive (waits
     for input) or returns immediately. Record the observation in
     ADR `0001-team-execution-model.md`.
   - If `TeamCreate` returns a parallelism-limit error before 4 teams
     are up, record the actual ceiling in ADR
     `0002-team-ceiling.md` and merge Funding + Outreach into
     Bizdev per §3.
   - Once the test is done, `TeamDelete` the test team so the slot
     is freed for real teams.

7. **Spawn teams in priority order.**
   - For each team in §3 (Eng-1, Eng-2, Eng-3, Funding, Outreach):
     - Call `TeamCreate` for that team (use a stable team name like
       `eng-1-tls`, `eng-2-sealfs`, `eng-3-caves`, `funding`,
       `outreach`, or `bizdev` if merged).
     - Spawn the team's agent via the Agent tool with:
       - `team_name` parameter set to the team name above (so the
         agent runs in the team's context)
       - `subagent_type` set to `general-purpose` unless a more
         specific type from the available-agent-types list is a
         clear better fit
       - `run_in_background: true` (so the leader doesn't block
         waiting for the team to finish — vault-mediated comm
         requires the leader to stay active in the coordination loop)
       - A `prompt` that includes:
         - The team's mission paragraph from §3
         - The team's reading list from §3
         - The team's files-to-modify list from §3
         - The team's TDD test scenarios from §3 (eng teams)
         - The team's DoD from §3
         - The team's out-of-scope list from §3
         - A pointer to the team's log file (`log/eng-1-tls.md`
           etc.) and explicit instruction to write to it on every
           meaningful step
         - A pointer to the team's inbox file
           (`inbox/to-eng-1.md` etc.) and explicit instruction to
           read it at natural break points (per §5 polling cadence)
         - A pointer to this entire instructions file
           (`docs/superpowers/plans/2026-05-17-multi-team-push.md`)
           and a directive to read §4 (Operating Principles), §5
           (Coordination Protocol), and §7 (Escalation Rules) in
           full before doing anything else
         - The directive: "All coordination is vault-mediated. Do
           NOT call SendMessage. Write to your log file and your
           inbox as described in §5. Commit early and often."

8. **Enter the coordination loop (§5–§7).**
   - Update `status.md` with all 5 teams now spawned.
   - Begin reading team logs + `inbox/to-leader.md` at natural
     cadence.
   - Continue until session-complete or interrupted per §6.

---

## End of file

The leader's job ends when `session-report.md` is committed and
`log/leader.md` ends in `STATUS: SESSION_COMPLETE`. Until then,
follow §5–§7.

Kaden may interrupt at any time. If he does, halt cleanly per §6.
