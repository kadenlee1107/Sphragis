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
