STATUS: COMPLETE | date 2026-05-18 | author researcher-eng

# Next 1–2 weeks of engineering priorities (Days 1–14 = 2026-05-18 → 2026-06-01)

## TL;DR

- **Highest-leverage push:** **SP-TEST-001 — `[lib]` restructure so `cargo test --workspace` actually runs.** Yesterday's 3 eng teams left ~18 `#[cfg(test)]` blocks on the floor (`docs/superpowers/orchestrator/2026-05-17-push/session-report.md` §5 + §6). Until host tests run, every "TDD red+green" claim is gated on a slow QEMU smoke loop and clippy-on-aarch64. Unblocking this multiplies every future eng team's velocity.
- **Highest-leverage cleanup:** **Run the two QEMU smoke scripts Eng-2 + Eng-3 wrote but never executed** (`scripts/qemu_cap_mls_selftest.py`, `scripts/qemu_sealfs_rotation_selftest.py`). Both ship a `PASS` summary line and were the only "DoD verified" pieces left dangling per yesterday's §6.1. Ubuntu Claude on the Linux dev host can do this on Day 1; closes the "deep green" gap.
- **Tempting but should NOT be next:** **x86_64 boot bring-up** (`DESIGN_X86_64_PORT.md`). Needs ~$2K hardware procurement (§"Open user actions"), a long port across `src/arch/x86_64/`, and no team is formed for it. Don't start until founder paperwork unlocks procurement + a Verus proof beachhead has landed (so the "verified Rust microkernel" story isn't undercut by a half-built second arch).

---

## §1. Sprint plan (Day 1 = 2026-05-18; Day 14 = 2026-06-01)

### Block A — Days 1–2 (Mon–Tue 2026-05-18 → 05-19): "Close yesterday's loop"

| Owner | Task | Depends on | Definition of Done |
|---|---|---|---|
| Ubuntu Claude | `cargo build --release --target aarch64-unknown-none` then `python3 scripts/qemu_cap_mls_selftest.py` | M4 dev host has `qemu-system-aarch64` | Script prints `[cap-mls] PASS — 6/6 scenarios verified`; log under `logs/qemu-tests/cap-mls-selftest-*.log` committed to journal entry. |
| Ubuntu Claude | `cargo build --release --features selftest-on-boot --target aarch64-unknown-none` then `python3 scripts/qemu_sealfs_rotation_selftest.py` | Same | Script prints `[sealfs-rotation] PASS — 6/6` (per `scripts/qemu_sealfs_rotation_selftest.py:21-30` pass criteria); log committed. |
| Ubuntu Claude | `git stash list` audit + drop the 2 stale stashes flagged in `session-report.md:81-85` | Stash audit only | Stash list is empty OR each remaining stash documented in session journal. |
| Mac Claude | Open `git worktree add` for each team-track + draft `WORKFLOW_TEAMS.md` (workspace-hygiene rules per `session-report.md:179` recommendation #1+#2) | None | Doc landed; `scripts/install_hooks.sh` updated. |

### Block B — Days 2–4 (Tue–Thu 2026-05-19 → 05-21): **SP-TEST-001 `[lib]` restructure**

| Owner | Task | Depends on | DoD |
|---|---|---|---|
| Mac Claude | Split workspace: extract host-buildable pure-Rust modules into `kernel-core/` crate exposing `[lib]`. Keep `src/main.rs` + arch-bound modules in `[bin]` for `aarch64-unknown-none`. Re-export the `#[cfg(test)]` blocks from `src/net/x509.rs`, `src/fs/sealfs*.rs`, `src/caves/cap_token.rs`, `src/caves/mls_label.rs`, `src/caves/cap_mls_selftest.rs` through the lib. | Block A worktree hygiene (so the restructure doesn't churn on other teams) | `cargo test -p kernel-core --target $(rustc -vV \| awk '/host/{print $2}')` runs ≥18 scenarios, all green. `cargo build --release --target aarch64-unknown-none` still produces a bootable kernel. `cargo deny check` clean. QEMU `qemu_boot_smoke.py` still PASS. |
| Mac Claude | Update `.github-workflows-pending/*.yml` references to `cargo test --workspace` so the SP-BLD-005 family of workflows can light up. | Restructure landed | YAML edits compile via `actionlint` if installed; gated behind workflows-pending until OAuth opens. |

### Block C — Days 4–7 (Thu–Sun 2026-05-21 → 05-24): **Verus cap-dispatch Phase A**

| Owner | Task | Depends on | DoD |
|---|---|---|---|
| Mac Claude | Per `verification/cap_dispatch/SPEC.md` Phase A (~1 week): write `verification/cap_dispatch/state.rs` (~200 LoC abstract `State`/`Cave`/`Op`/`KernelResources`) + `verification/cap_dispatch/invariants.rs`. Just the type-state invariants — NO non-interference yet, per SPEC.md:152-154. | Verus toolchain installed locally (`verification/README.md` §"Installation"); Kaden must confirm `$VERUS` env var is set OR procure an Ubuntu install slot. | `$VERUS verification/cap_dispatch/state.rs` outputs `verification results:: N verified, 0 errors` (N ≥ the count of invariant lemmas landed). `verification/cap_dispatch/refinement.md` draft maps abstract field names to `src/caves/cave.rs` field names. |
| Mac Claude | Update `verification/README.md` directory layout table — Phase A landed. | Above | Doc reflects state. |

### Block D — Days 7–10 (Sun–Wed 2026-05-24 → 05-27): **HSM operator-CA endorsement loader (Sphragis side, SP-C1.6.IMPL.A subset)**

| Owner | Task | Depends on | DoD |
|---|---|---|---|
| Mac Claude | Per `DESIGN_HSM_OPERATOR_CA.md` §"Implementation scope" item 1+2: add `endorsement: Vec<u8>` to `Quote` struct in `src/security/attest.rs`, bump Quote wire-format version constant, load `/attest/endorsement.cbor` from SealFS at boot, cache parsed bytes. If file absent → `endorsement` is empty vec (no regression). | None (no HSM hardware needed for IMPL.A — operator-side host script is item 3, deferred). | Kernel boots in QEMU with + without the endorsement file. New shell command `attest-endorsement-status` prints `present`/`absent` + validity-window if present. `#[cfg(test)]` parse-roundtrip test under `kernel-core` (depends on Block B). |
| Mac Claude | Test fixture: a software-stand-in ML-DSA-87-signed endorsement cert (`scripts/gen_endorsement_test_fixture.py` mirroring `scripts/gen_x509_test_chains.py` pattern). NOT a real HSM yet — DESIGN doc says HSM/operator-CA host is `IMPL.A item 3`, deferred. | Above | Fixture committed under `src/security/attest_fixtures/`. |

### Block E — Days 10–14 (Wed–Sun 2026-05-27 → 06-01): **Verus cap-dispatch Phase B start (FileOpen + FileWrite)**

| Owner | Task | Depends on | DoD |
|---|---|---|---|
| Mac Claude | Per SPEC.md Phase B (~2 weeks; we get the start of it): write `verification/cap_dispatch/dispatch.rs` (~150 LoC pure-modeled `dispatch` function) + first two per-op lemmas (`op_only_affects_actors_caps` for `Op::FileOpen` and `Op::FileWrite` per SPEC.md:81-92). | Block C Phase A complete. | `$VERUS verification/cap_dispatch/lemmas.rs` reports both lemmas verified. NO top-level theorem yet (that's Phase D, ~Day 28). |
| Hardware-bound | Kaden: skim Phase A's `refinement.md` for any wrong-shaped abstraction; respond inline in `docs/superpowers/research/2026-05-18-eng-next-2-weeks-feedback.md` or via inbox. | Phase A landed | Phase B is unblocked by Kaden's read OR by 48h timeout. |

### Out of these 14 days, untouched on purpose (see §3 for why)

x86_64 IMPL.A, CHERIoT-Ibex bring-up, Caliptra integration, IPC info-flow Phase A, Sigstore CI activation, real OCSP/CRL.

---

## §2. Top-5 picks — detail

### Pick 1: SP-TEST-001 — Cargo `[lib]` restructure (Block B, Days 2–4)

**Scope.** Today the workspace is single-crate `[bin]` for `aarch64-unknown-none` with `-Z build-std=core,alloc`. `cargo test --workspace` is vacuous (`session-report.md:200-203`). Solve by extracting pure-Rust modules — anything that doesn't directly touch `aarch64` registers — into a sibling `kernel-core/` crate that builds against the host triple. Wire `src/main.rs` and the arch-bound modules to depend on `kernel-core` via path dep. The `#[cfg(test)] mod tests` blocks Eng-1 + Eng-2 + Eng-3 already wrote (the 18 scenarios from yesterday's push) now run on the host.

**Candidate first commit.** `Cargo.toml`, `kernel-core/Cargo.toml`, `kernel-core/src/lib.rs` (re-exports `net::x509`, `fs::sealfs_rotation`, `fs::sealfs_journal`, `fs::sealfs_audit`, `caves::cap_token`, `caves::mls_label`).
Message: `build: extract kernel-core lib crate so cargo test runs on host (SP-TEST-001)`.

**Success criteria.** Three:
1. `cargo test -p kernel-core` runs ≥18 scenarios green on Mac host.
2. `cargo build --release --target aarch64-unknown-none` still produces a bootable kernel ELF.
3. `python3 scripts/qemu_boot_smoke.py` PASS unchanged.

**Risk.** Cyclic dependencies — `attest.rs` imports `fs::sealfs`, but `fs::sealfs` imports `crypto::rng` which transitively pulls aarch64 register reads. Mitigation: gate aarch64-only entry points behind `#[cfg(target_arch="aarch64")]` and provide host stubs (`#[cfg(test)] fn rng_fill_bytes(...) { ... }`).

**Fallback if blocked.** Land kernel-core only for the leaf modules that DON'T pull arch code (x509 chain validator is purely byte-pushing; sealfs_rotation/journal/audit similarly). Defer mls_ipc + cap_token + cap_mls_selftest to a follow-on. Even partial restructure runs ~10 of the 18 scenarios on host.

---

### Pick 2: QEMU smoke runs (Block A, Days 1–2)

**Scope.** Two scripts, both shipped in yesterday's push but not executed: `scripts/qemu_cap_mls_selftest.py` (shell-driven; passes the boot passphrase prompt then types `cap-mls-selftest`) and `scripts/qemu_sealfs_rotation_selftest.py` (requires `--features selftest-on-boot`). Per yesterday's §6.1 these are the only "DoD verified" claims left dangling.

**Candidate first commit.** `docs/superpowers/research/2026-05-18-qemu-smoke-evidence.md` summarizing both runs + log paths under `logs/qemu-tests/`.
Message: `research: log QEMU evidence for cap-mls + sealfs-rotation selftests`.

**Success criteria.** Both scripts exit 0. Pass lines `[cap-mls] PASS — 6/6 scenarios verified` and `[sealfs-rotation] ... all 6 ... PASS` captured. Logs committed.

**Risk.** Cap-mls script `c.expect(rb"Enter passphrase", timeout=60); c.sendline("")` assumes the empty-passphrase QEMU default; if Kaden has set a non-empty default for local builds the script wedges. Mitigation: Ubuntu Claude runs against a fresh `cargo build` from `main` HEAD (no `.env`-style override).

**Fallback if blocked.** Mac-Claude can run them in `qemu-system-aarch64` via Homebrew if Ubuntu host is offline (same ARG list per `scripts/qemu_cap_mls_selftest.py:28-33`).

---

### Pick 3: Verus cap-dispatch Phase A (Block C, Days 4–7)

**Scope.** Per `verification/cap_dispatch/SPEC.md:152-154` Phase A only: land the abstract `State` / `Cave` / `Op` / `KernelResources` model + the invariant lemmas (Step 1 of the proof strategy). No non-interference theorem yet (that's Phase B–D, ~Days 21-35). This buys: a concrete artifact for differentiator-#1 funding pitches that says "Verus proof IS landing," NOT just "we wrote a SPEC.md."

**Candidate first commit.** `verification/cap_dispatch/state.rs` (Verus syntax skeleton with `spec fn inv(σ: State) -> bool` + the type definitions).
Message: `verification(cap-dispatch): Phase A — state.rs + invariant lemmas (SP-VER-001.IMPL.A)`.

**Success criteria.** `$VERUS verification/cap_dispatch/state.rs` outputs `verification results:: N verified, 0 errors`. `refinement.md` draft maps the abstract state fields to `src/caves/cave.rs` fields (per SPEC.md:147-151).

**Risk.** Verus toolchain install is operator-local (`verification/README.md:27-46`); not in CI. If Verus pinned-toolchain breaks under current Rust nightly upstream, Phase A stalls. Mitigation: Kaden documents the actual Verus commit pinned by `verification/smoke/smoke.rs` and we pin Phase A to that revision.

**Fallback if blocked.** Skip Verus; land the proof OUTLINE as `verification/cap_dispatch/PHASE_A_OUTLINE.md` — a checkable proof sketch in pseudocode — so the SPEC.md is no longer the most-recent artifact. Better than nothing; concedes the differentiator claim to "designed not built" until next sprint.

---

### Pick 4: HSM operator-CA endorsement loader (Sphragis side; Block D, Days 7–10)

**Scope.** `DESIGN_HSM_OPERATOR_CA.md` §"Implementation scope" items 1+2 only (NOT 3 — the operator-CA host script needs a real HSM, deferred). Add `Quote.endorsement: Vec<u8>`, bump wire-format version, load `/attest/endorsement.cbor` from SealFS at boot, cache parsed structure, surface via `attest-endorsement-status` shell command. Test fixture is a software-stand-in ML-DSA-87 endorsement, NOT real HSM-issued.

**Candidate first commit.** `src/security/attest.rs` (+ `src/security/attest_fixtures/`).
Message: `attest: add endorsement field + SealFS loader (SP-C1.6.IMPL.A)`.

**Success criteria.** Kernel boots both with + without `/attest/endorsement.cbor`. New shell command prints `[attest-endorsement] present  device-id=...  expires=...` OR `[attest-endorsement] absent` cleanly. Round-trip parse test passes under `kernel-core` (depends on Pick 1). No regression in existing `attest-smoke`.

**Risk.** Wire-format version bump is a one-way door for any externally-stored quotes. Mitigation: version 2 reads version 1 quotes (no `endorsement` field) and emits version 2 only when one is loaded. Documented in `src/security/attest.rs` migration-history comment block.

**Fallback if blocked.** Land just the `Quote` field bump + a `None` cache; defer the SealFS loader to a follow-on. Half the value, none of the risk.

---

### Pick 5: Per-team git-worktree + commit-hygiene hook (Block A.4, Days 1–4 background)

**Scope.** Yesterday's 4 hygiene incidents (`session-report.md` §4) all trace to `git add .` / `git add -A` in a single shared tree. Two pieces: (a) add `WORKFLOW_TEAMS.md` documenting `git worktree add ../sphragis-worktree-<team> -b <team>/<purpose> main` per team; (b) ship `scripts/check_owned_paths.sh` pre-commit hook that errors on `git add .` invocation and warns when a commit touches paths outside a team's declared `owned-paths.txt` under `docs/superpowers/orchestrator/<push>/teams/<team>/`.

**Candidate first commit.** `WORKFLOW_TEAMS.md` + `scripts/check_owned_paths.sh` + hook wiring in `scripts/install_hooks.sh`.
Message: `chore(workflow): adopt git worktree per team + commit-hygiene hook`.

**Success criteria.** Doc landed. Hook fires (warn only, not block) on a test commit that touches paths outside a stub `owned-paths.txt`. Next orchestrator session's plan can reference `WORKFLOW_TEAMS.md` instead of re-deriving the rule.

**Risk.** Worktree adoption requires the orchestrator plan template to RELAX `§4 "no feature branches"`. Mitigation: explicitly call this out in `WORKFLOW_TEAMS.md` and link the orchestrator plan-template diff.

**Fallback if blocked.** Ship only the hook (no worktree adoption); single-tree discipline improves even without worktrees.

---

## §3. Deliberately NOT on the list

| Candidate | Why deferred |
|---|---|
| **Real OCSP / CRL fetching** | Eng-1 §3 explicitly out-of-scope (`session-report.md:217`). The stub returns `Ok` and is covered by scenario 6. Real fetching needs a network stack policy decision (which caves get to make outbound requests? what's the timeout model? offline mode?) — that's a design SP, not a 2-week impl. |
| **New CA certs in the trust bundle** | Eng-1 out-of-scope, same line. Trust bundle policy is gov-buyer-specific (operator picks roots at install time per `DESIGN_PRODUCTIZATION_UX.md` §UX-002 first-boot flow). No engineering value in shipping arbitrary roots now. |
| **Disk-format migration tooling for pre-2026-05-17 SealFS images** | Eng-2 out-of-scope (`session-report.md:219`); pre-rename images fail magic check by design (CLAUDE.md "Pre-2026-05-17 disk images will fail magic check by design"). No installed-base disks exist yet. |
| **Networked audit-log replication** | Eng-2 out-of-scope (`session-report.md:220`). Needs the SP-AUD-004 design that doesn't exist yet. |
| **Persistent label history + dynamic relabeling + network-side label propagation** | Eng-3 out-of-scope (`session-report.md:222-224`). Labels are fixed-at-spawn for v1; persistent history blocked on Eng-2's `sealfs_audit` integration, also out-of-scope. |
| **x86_64 IMPL.A boot bring-up** | `DESIGN_X86_64_PORT.md` §"Open user actions" requires NUC 13 Pro + ThinkPad X1 procurement (~$2K) AND a long port across `src/arch/x86_64/`. Procurement is gated on founder paperwork (separate task #3). Starting QEMU-x86_64-only IMPL.A would burn 2 weeks AND leave you with a half-built second arch competing for attention with Verus proof landing. Postpone until Verus Phase A+B is done and Kaden has signed off on hardware procurement. |
| **CHERIoT-Ibex target build** | `DESIGN_CHERI_MAPPING.md` §"CHERIoT-Ibex (SP-CHR-003)" lists "available 2026" + needs SCI Semiconductor ICENI or lowRISC dev kit. No hardware in hand; `cheriot-llvm` fork install is multi-day. Differentiator #5 is "CHERI-READY architecture (mapping doc landed)" — that's already ✅ in `DESIGN_CHERI_MAPPING.md`. The IMPL is a market-entry play (embedded gov / automotive) that doesn't compete with the 14-day window. |
| **Caliptra integration** | Needs FPGA dev board. Same procurement-and-hardware story as x86_64. SP-C1.5 (TPM 2.0 attestation) lands FIRST per `DESIGN_X86_64_PORT.md` §A2, and TPM 2.0 needs the x86_64 port. Two-step prerequisite. |
| **Sigstore CI workflow activation (`.github-workflows-pending/release-sign.yml`)** | OAuth-blocked from autonomous activation (note in `.github-workflows-pending/release-sign.yml` style header + CLAUDE.md's "no `--no-verify`"). Needs Kaden to flip the workflow on after one Verus proof has actually landed in CI (so the sigstore signatures are signing something whose verification posture matches the master plan's claims). Tabling until Block C-E gets a green build. |
| **IPC info-flow Phase A in parallel with cap-dispatch** | `verification/ipc_flow/SPEC.md:172-184` says ~8 weeks for IMPL.A-E sequentially; starting IMPL.A in week 1 would put us at 50% on TWO proofs at Day 14 instead of 100% on cap-dispatch Phase A. Sequence them. IPC info-flow Phase A starts Week 3 (the follow-on sprint), once cap-dispatch Phase A is locked + reviewed. |
| **UX track (window manager, installer, package mgr — `DESIGN_PRODUCTIZATION_UX.md`)** | Entire 10-chapter spec needs a UX team that doesn't exist. SP-UX-001 alone (window manager) is multi-week. Differentiator-#1 (verified microkernel) lands faster + matters more for the immediate DARPA / VC pitch than "feels like a real OS" polish. Schedule UX once funding closes a hire. |

---

## Cross-cutting watch items (not blocks, but worth noting)

- **Cargo.lock holder discipline.** Yesterday's session ran with 0 Cargo.lock changes; if any of Picks 1, 4 add a dep (CBOR parser for endorsement?), the orchestrator owns the lock + must broadcast hold/release per yesterday's protocol.
- **DCO sign-off.** Per task #1 researcher prompt — every commit `Signed-off-by: Kaden Lee <kadenlee1107@gmail.com>` (DCO). Conventional `<scope>: ...` prefix per existing log.
- **Audit-ring impact.** Pick 4 (endorsement loader) writes to `Category::Attest`; pre-flight check `audit::recent_for_cave` doesn't regress. Pick 1 restructure must not change audit-ring HMAC key derivation (SealFS rename already burned us on that — see CLAUDE.md SealFS rename notes).
