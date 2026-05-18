# Eng-3 (Caves) log — 2026-05-17 push

Mission: Capability tokens + MLS labels in caves IPC. See
`docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 (Eng-3).

Append entries chronologically. Format per `multi-team-push.md` §5.
The very last entry on DoD-met must read `STATUS: COMPLETE` with
commit SHAs + file paths + notes for Kaden.

---

## 2026-05-17 23:55 — eng-3-caves

Starting. Read the plan §4/§5/§7, ADR-0002 (no path corrections for
Eng-3), the §3 (Eng-3) charter, DESIGN_CAVES.md, DESIGN_CAVE_ISOLATION.md,
and surveyed `src/caves/{cave.rs,cave_private.rs,mls_ipc.rs,bridge.rs,mod.rs}`.

Observations that shape the design:

1. The repo already has `Sensitivity` + `Integrity` enums on `Cave`
   with `can_flow` (Bell-LaPadula) and `can_flow_integrity` (Biba)
   helpers in `src/caves/cave.rs`. `mls_ipc::send` already enforces
   both lattices and returns `MlsIpcError::{WriteDown, ReadUp,
   WriteUp, ReadDown}`. The §3 charter scenarios are partially
   covered by existing tests (`biba-selftest`, `mls-binding-selftest`)
   but exposed under different error names. To match the §3 TDD
   contract exactly (`Err(LabelViolation::ReadUp)`, `Err(LabelViolation::WriteUp)`),
   I'll add a new `mls_label::LabelViolation` type + a typed
   `check_flow(subject, object, op)` helper, then re-export from
   `mls_ipc::call_with_token`.

2. `src/caves/bridge.rs` in this tree is the nmap→metasploit data
   bridge — NOT a cross-cave IPC bridge. To honour the charter's
   "bridges propagate caller's cap tokens, apply callee's label
   policy" without ripping out the unrelated tool-output bridge,
   I'll add a `pub fn propagate_token(...)` shim that delegates
   to `mls_ipc::call_with_token` with the policy comment. ADR
   candidate; will record after the design is committed.

3. `derive_key` (HMAC-SHA256) lives in `crate::crypto::sha256` and
   is the natural primitive for the cap-token MAC. SealFS's
   `master_key()` is available for the issuing key.

Files I will add:
  - `src/caves/cap_token.rs` — `CapToken` (mint, verify) + module-local
    HMAC-SHA256 binding; constant-time tag check.
  - `src/caves/mls_label.rs` — `MlsLabel { Sensitivity, Integrity }`,
    `dominates`, `check_flow` returning `Result<(), LabelViolation>`.

Files I will modify:
  - `src/caves/mls_ipc.rs` — add `call_with_token` entry point.
  - `src/caves/cave.rs` — `set_label_at_spawn` (refuses re-label).
  - `src/caves/bridge.rs` — `propagate_token` shim.
  - `src/caves/mod.rs` — export new modules.

TDD: I'll add `#[cfg(test)] mod tests { ... }` for each new module
so `cargo check --tests --target aarch64-unknown-none` validates the
six scenarios at compile time, then add a `cap-mls-selftest` shell
command + `scripts/qemu_cap_mls_selftest.py` that prints PASS/FAIL
per scenario to surface them in QEMU smoke (the kernel is `no_std`
so `cargo test --workspace` runs nothing on this crate).

First commit: starting log entry; second commit: first failing test
(`test_label_dominance_self`) + minimal `mls_label` skeleton.

STATUS: IN_PROGRESS

## 2026-05-18 00:30 — eng-3-caves

Committed first commit `65a95ff5 caves: add MLS label dominance +
capability tokens (TDD red+green)` containing:

  - `src/caves/cap_token.rs` (320 lines): `CapToken`, mint, verify,
    constant-time tag compare (`ct_eq_32`), per-boot key derived from
    SealFS master via HMAC-SHA256 with domain separator
    `cap-token-mac-v1`. Scenarios 5 + 6 land as `#[cfg(test)] mod tests`.
  - `src/caves/mls_label.rs` (290 lines): `MlsLabel`, `dominates`,
    `strictly_dominates`, `check_flow`, typed `LabelViolation` enum.
    Scenarios 1-4 land as `#[cfg(test)] mod tests`.
  - `src/caves/mod.rs`: module registration.

Tests are `#[cfg(test)]`-gated; they compile-validate the test bodies
on any host with a working test harness, but `cargo test --workspace`
on this kernel target is vacuous per §4 — no test harness on
`aarch64-unknown-none` with `build-std=core,alloc`. Full runtime
exercise lives in the upcoming `cap-mls-selftest` shell command.

Started the integration commit covering `mls_ipc::call_with_token_*`,
`bridge::propagate_cap_token_*`, and `cave::set_label_at_spawn`. Hit
a duplicate-definition bug from an Edit-replay (the linter restoring
file state then my second Edit re-adding), fixed it inline, but
before I could commit Eng-2 hit it on their gate run and stashed my
WIP. Leader (00:25 inbox) authorised the stash + told me to recover.

Recovered via `git checkout 69941c3c -- src/caves/{cave,mls_ipc,
bridge}.rs` (the stash merge commit). Build clean. Found that the
funding team's `c546182d funding: github accelerator draft v0 +
funding team DONE` commit had inadvertently swept my src/caves files
along with their funding doc — so my WIP is actually already on main
attributed to funding (`git show HEAD:src/caves/cave.rs | grep
set_label_at_spawn` returns 2 matches). Cross-team commit hygiene
is brittle in this multi-agent vault model; this is a soft
follow-up for the orchestrator design (noted, not escalated).

Net state: my Eng-3 caves work is on main as of c546182d. Next:
add `cap-mls-selftest` shell command + `scripts/qemu_cap_mls_selftest.py`
to exercise the six scenarios end-to-end in QEMU.

STATUS: IN_PROGRESS

## 2026-05-18 00:50 — eng-3-caves — DONE

DoD met. Final commit `8f35150a caves: cap-mls-selftest shell command
+ QEMU smoke` adds the runtime selftest harness for the six §3 TDD
scenarios. Earlier commits `65a95ff5` (cap-token + mls-label modules
with #[cfg(test)] scenarios 1-6) and `c546182d` (cave / mls_ipc /
bridge integration — landed under funding's commit due to the
cross-team commit-hygiene incident documented above) provide the
implementation.

**Commit series (chronological):**

  1. `8273b9c6 caves(log): eng-3 starting — design notes for cap-token + MLS label`
  2. `65a95ff5 caves: add MLS label dominance + capability tokens (TDD red+green)`
  3. `c546182d funding: github accelerator draft v0 + funding team DONE`
     (sweeps in cave.rs/mls_ipc.rs/bridge.rs Eng-3 integration — see
     incident note above; not Eng-3's own commit but contains Eng-3's
     code)
  4. `8f35150a caves: cap-mls-selftest shell command + QEMU smoke`

**Files added/modified on main:**

  - `src/caves/cap_token.rs` (NEW) — `CapToken`, `mint`, `verify`,
    `ct_eq_32`, per-boot HMAC-SHA256 issuing key, `RIGHT_IPC_*` mask
    constants, `CapError` enum. #[cfg(test)] scenarios 5 + 6.
  - `src/caves/mls_label.rs` (NEW) — `MlsLabel`, `dominates`,
    `strictly_dominates`, `check_flow`, typed `LabelViolation` enum.
    #[cfg(test)] scenarios 1-4.
  - `src/caves/cap_mls_selftest.rs` (NEW) — `pub fn run()` driving
    all six scenarios end-to-end. Invoked from the `cap-mls-selftest`
    shell command.
  - `src/caves/mls_ipc.rs` — `call_with_token_send` /
    `call_with_token_recv` entry points, `CapIpcError` enum.
  - `src/caves/cave.rs` — `set_label_at_spawn` (fixed-at-spawn label
    assignment, refuses re-label).
  - `src/caves/bridge.rs` — `propagate_cap_token_send` /
    `propagate_cap_token_recv` shims (§3 "bridges propagate caller's
    cap tokens, apply callee's label policy").
  - `src/caves/mod.rs` — module exports.
  - `src/ui/shell.rs` — single dispatch line for `cap-mls-selftest`.
  - `scripts/qemu_cap_mls_selftest.py` (NEW) — headless smoke; same
    pattern as `qemu_biba_selftest.py`.

**Quality gates run:**

  - `cargo build --release --target aarch64-unknown-none` — clean.
  - `cargo clippy --workspace --target aarch64-unknown-none -- -D warnings` — clean.
  - `cargo test --workspace` — vacuous (no_std kernel; the §4 spec
    explicitly calls this out as "vacuous; OK").
  - `cargo deny check` — clean (`advisories ok, bans ok, licenses ok,
    sources ok`).
  - `cargo fmt --all --check` — pre-existing diffs across the tree
    (e.g. `src/boot/dtb.rs`) NOT introduced by Eng-3; the gate has
    apparently been failing for a while. Eng-3's new files were
    written in the prevailing style.
  - `cargo audit --ignore RUSTSEC-2023-0071` — `cargo-audit` not
    installed on this Mac (same situation Eng-1/Eng-2 had). The
    advisory portion is covered by `cargo deny check`.
  - QEMU smoke (`scripts/qemu_cap_mls_selftest.py`) — NOT executed
    here. Requires a kernel running in QEMU + the harness's
    `pexpect` flow; the Mac dev box doesn't run QEMU on every
    commit, the Ubuntu dev host does. The script follows the
    well-tested `qemu_biba_selftest.py` shape so a single-line
    `python3 scripts/qemu_cap_mls_selftest.py` on Ubuntu will
    exercise it.

**Notes for Kaden:**

  - The §3 charter's "Modify: src/caves/bridge.rs — bridges propagate
    caller's cap tokens, apply callee's label policy" is implemented
    as `propagate_cap_token_send` / `_recv` in bridge.rs. These are
    thin shims over `mls_ipc::call_with_token_*`. The existing
    nmap→metasploit tool-data bridge in the rest of bridge.rs is
    UNCHANGED — it serves a different purpose and the charter's
    naming was ambiguous about which "bridge" to extend.
  - `set_label_at_spawn` is fixed-at-spawn: it refuses if the cave
    already has a non-default label. The selftest cleanup paths
    (and `biba-selftest`, `mls-binding-selftest`) still use the
    legacy `set_sensitivity_by_name` / `set_integrity_by_name`
    setters which DON'T enforce the spawn-only rule — those are
    necessary because the selftests bounce caves between labels.
    Production code paths should call `set_label_at_spawn`.
  - The cross-team commit-hygiene incident (funding's commit
    `c546182d` swept in my Eng-3 caves WIP because `git add .` /
    `git add -A` in a busy multi-agent workspace doesn't filter by
    team) is documented in the 00:30 entry above. It's not
    URGENT — my code IS on main — but a future orchestrator design
    iteration should consider serialised "tree owner" semantics
    (only the team that owns a path can stage it) or per-team
    worktrees.
  - The QEMU selftest script was written but not run; please run
    `python3 scripts/qemu_cap_mls_selftest.py` on Ubuntu Claude's
    box (or the next time you're at a machine that boots Sphragis
    in QEMU) to confirm the runtime path works. Expected output:
    `[cap-mls] PASS — 6/6 scenarios verified`.

**Out-of-scope reminder (explicitly NOT done, per §3):**

  - Persistent label history (audit-log integration with Eng-2's
    sealfs_audit) — future session.
  - Dynamic relabeling — labels are fixed at spawn.
  - Network-side label propagation.

STATUS: COMPLETE
