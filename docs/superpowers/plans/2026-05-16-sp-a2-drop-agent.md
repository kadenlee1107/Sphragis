# SP-A2: Drop AGENT App from the Codebase

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the AGENT app (`src/ai/` + `src/ui/apps/agent.rs`) from the Sphragis tree entirely. AI-in-the-kernel-TCB is anti-feature per REQ-ANTI-002 for gov-grade positioning. Reduces TCB by ~5,856 LoC and removes a future attack surface that one bad commit could activate.

**Architecture:** Subtractive change. Remove the AI subsystem, the agent UI app, all references to both. Verify the lock-screen app cycle skips index 8 cleanly and the kernel still boots through every remaining surface.

**Tech Stack:** Rust, Bash.

**Requirements closed:** STRAT-003 (gov build profile consequence), ANTI-002.

**Depends on:** SP-A1 (Apache-2.0 relicense; clean tree).

---

## File Structure

- **Delete:** `src/ai/` directory (entire subtree, 5,327 LoC)
- **Delete:** `src/ui/apps/agent.rs` (529 LoC)
- **Modify:** `src/ui/apps/mod.rs` (remove agent registration)
- **Modify:** `src/main.rs` (remove `mod ai;` if present)
- **Modify:** `src/ui/shell.rs` or wherever the lock-screen app cycle iterates 1..=8 (downgrade to 1..=7 or skip 8)
- **Modify:** `DESIGN_AI_AGENT.md` (add historical-removal banner; keep doc for context)
- **Sweep:** `Cargo.toml` (remove AI-specific feature flags / deps if any)
- **Sweep:** `scripts/qemu_*.py` (remove agent-specific test invocations)

---

### Task 1: Locate all `src/ai/` and agent references

- [ ] **Step 1: Inventory the AI module**

Run: `ls -la src/ai/`
Note: should see `mod.rs`, `client.rs`, plus other files (5,327 LoC total).

- [ ] **Step 2: Find every reference to `crate::ai`, `src/ai`, `mod ai`, AGENT app**

Run: `grep -rn "crate::ai\|mod ai\|use crate::ai" src/ scripts/ docs/ 2>/dev/null | head -30`
Save the output as a checklist. Each reference needs handling.

- [ ] **Step 3: Find lock-screen app-cycle code**

Run: `grep -rn "agent\|AGENT" src/ui/shell.rs src/ui/apps/mod.rs scripts/qmp_test.py 2>/dev/null | head -20`

- [ ] **Step 4: Find scripts that exercise AGENT specifically**

Run: `grep -l "agent\|AGENT" scripts/qemu_*.py 2>/dev/null`

---

### Task 2: Delete `src/ai/` directory

- [ ] **Step 1: Remove the directory**

Run: `git rm -r src/ai/`
Expected: git stages deletion of all files under `src/ai/`.

- [ ] **Step 2: Verify removal**

Run: `ls src/ai/ 2>&1 | head -3`
Expected: "No such file or directory".

---

### Task 3: Delete `src/ui/apps/agent.rs`

- [ ] **Step 1: Remove the file**

Run: `git rm src/ui/apps/agent.rs`

- [ ] **Step 2: Verify**

Run: `ls src/ui/apps/agent.rs 2>&1`
Expected: "No such file or directory".

---

### Task 4: Remove agent from `src/ui/apps/mod.rs`

- [ ] **Step 1: Read the mod file**

Read `src/ui/apps/mod.rs`. Locate the `pub mod agent;` line and any reference within an apps-registration enum/struct.

- [ ] **Step 2: Remove `pub mod agent;`**

Use Edit tool to delete that exact line.

- [ ] **Step 3: Remove agent from any apps-enum/registration**

E.g., if there's an `enum App { Caves, Files, ..., Agent }`, remove the `Agent` variant and any match arm that handles it.

- [ ] **Step 4: Verify build hint**

Run: `grep -n "agent" src/ui/apps/mod.rs`
Expected: no remaining references (or only comments mentioning historical removal).

---

### Task 5: Remove `mod ai;` from `src/main.rs` (if present)

- [ ] **Step 1: Check**

Run: `grep -n "^mod ai\|^pub mod ai" src/main.rs`

- [ ] **Step 2: If found, remove with Edit tool**

Use Edit tool to delete the line(s).

---

### Task 6: Downgrade lock-screen app cycle

- [ ] **Step 1: Locate the cycle code**

The lock screen iterates apps 1-8 (`scripts/qmp_test.py` shows it). Find the source-side code:

Run: `grep -rn "\.\.=8\|for i in 1\.\.9\|apps\[.*8\]\|MAX_APPS\|NUM_APPS" src/ui/ 2>/dev/null | head -10`

- [ ] **Step 2: Identify the iteration bound**

Likely a constant or a 1..=8 range. Determine whether agent was app #8.

- [ ] **Step 3: Update the bound**

Two options:
- (a) Renumber: pull index 8 out; cycle becomes 1..=7.
- (b) Skip: leave 8 slots but make slot 8 unreachable / display "(reserved)".

Option (a) is cleaner. Update the bound to 1..=7 (or `NUM_APPS = 7`).

- [ ] **Step 4: Update any per-app dispatch match**

If there's a `match app_idx { 1 => caves, 2 => files, ..., 8 => agent }` pattern, remove the `8 => agent` arm.

---

### Task 7: Update `DESIGN_AI_AGENT.md` as historical doc

- [ ] **Step 1: Add removal banner**

Edit `DESIGN_AI_AGENT.md`. Prepend at the top:

```markdown
> **HISTORICAL — REMOVED 2026-05-16**
>
> The AGENT app was removed from the Sphragis tree as part of SP-A2
> in the gov-OS productization plan. AI-in-the-kernel-TCB is an
> anti-feature for gov-grade positioning (see [ANTI_FEATURES.md](ANTI_FEATURES.md)
> §ANTI-002). This document is preserved for historical context and
> as design reference if a user-mode (out-of-TCB) variant is built
> in the future for the `sphragis-community` profile.

---
```

- [ ] **Step 2: Stage**

Run: `git add DESIGN_AI_AGENT.md`

---

### Task 8: Sweep `Cargo.toml` for AI-specific deps / features

- [ ] **Step 1: Check current Cargo.toml**

Run: `grep -i "ai\|agent\|llm\|inference\|onnx\|tflite" Cargo.toml`

- [ ] **Step 2: Remove any AI-specific dep / feature flag**

Edit `Cargo.toml`. Remove anything specific to the agent app.

- [ ] **Step 3: Verify Cargo.lock regenerates clean**

Run: `cargo build --target aarch64-unknown-none --release 2>&1 | head -20`

If unused-deps appear in lockfile, run `cargo update` after editing.

---

### Task 9: Sweep `scripts/qemu_*.py` for agent test invocations

- [ ] **Step 1: Locate agent-related tests**

Run: `grep -l "AGENT\|agent_focused\|app.*8.*agent\|cmd_ai" scripts/qemu_*.py 2>/dev/null`

- [ ] **Step 2: Update `scripts/qmp_test.py`**

Read `scripts/qmp_test.py` (we already know it cycles apps 1-8 with labels). Remove:
- `"agent"` from the labels list (line ~137)
- AGENT-specific test steps (lines ~152-160 — `07-agent-focused`, `08-agent-typing`, `09-agent-enter`, `11-back-to-agent`)

Replace the labels list `["caves", "files", "net", "security", "shell", "editor", "comms", "agent"]` with `["caves", "files", "net", "security", "shell", "editor", "comms"]`.

- [ ] **Step 3: Update any other test that types into agent**

Run: `grep -rn "07-agent\|08-agent\|09-agent\|11-back-to-agent\|cmd_ai" scripts/ 2>/dev/null`

For each hit: remove that step.

---

### Task 10: Verify build + tests

- [ ] **Step 1: Build**

Run: `cargo build --target aarch64-unknown-none --release 2>&1 | tail -20`
Expected: clean build. If there are unresolved references to `crate::ai::*` or `agent::*`, sweep and remove.

- [ ] **Step 2: Clippy**

Run: `cargo clippy --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: clean.

- [ ] **Step 3: Boot smoke**

Run: `python3 scripts/qemu_boot_smoke.py`
Expected: PASS.

- [ ] **Step 4: Cave private selftest**

Run: `python3 scripts/qemu_cave_private_selftest.py`
Expected: PASS.

- [ ] **Step 5: Sanity-check kernel binary shrank**

Run: `ls -la target/aarch64-unknown-none/release/sphragis`
Expected: smaller than the pre-removal size (was 7,840,544 bytes). Should drop by ~500K-1MB given the LOC delta.

- [ ] **Step 6: cargo-deny still passes**

Run: `cargo deny check`
Expected: PASS.

---

### Task 11: Commit + push + merge

- [ ] **Step 1: Branch**

Run: `git checkout -b feat/drop-agent-app`

- [ ] **Step 2: Stage everything**

Run: `git add -A && git status --short`
Verify deletions (`D`) of `src/ai/*` and `src/ui/apps/agent.rs` plus modifications (`M`) of `src/ui/apps/mod.rs`, `src/main.rs` (if applicable), `src/ui/shell.rs` (or wherever cycle lives), `DESIGN_AI_AGENT.md`, `scripts/qmp_test.py`, `Cargo.toml`.

- [ ] **Step 3: Commit (DCO sign-off)**

Run:
```bash
git commit -s -m "$(cat <<'EOF'
feat: drop AGENT app from Sphragis tree (SP-A2)

Per the gov-OS productization plan. AI-in-the-kernel-TCB is anti-feature
ANTI-002 for gov-grade positioning. Removing it now (rather than feature-flagging
it out of the gov build) keeps the TCB shape we want and eliminates a
future-attack-surface that one bad commit could activate.

Deletions:
  - src/ai/             — entire subsystem (5,327 LoC)
  - src/ui/apps/agent.rs — UI app (529 LoC)
  - Net: ~5,856 LoC removed

Modifications:
  - src/ui/apps/mod.rs   — remove agent registration
  - src/main.rs          — remove mod ai (if present)
  - src/ui/shell.rs      — lock-screen app cycle: 1..=8 -> 1..=7
  - DESIGN_AI_AGENT.md   — historical-removal banner; doc preserved
  - scripts/qmp_test.py  — remove agent test steps
  - Cargo.toml           — remove AI-specific deps/features

A future user-mode (out-of-TCB) variant could ship in sphragis-community
if desired; sphragis-gov has no AI surface.

Verified: build clean, clippy clean, qemu_boot_smoke PASS, qemu_cave_private_selftest PASS, cargo deny check PASS.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

- [ ] **Step 4: Push + merge to main**

Run:
```bash
git push -u origin feat/drop-agent-app && \
  git checkout main && \
  git merge --no-ff feat/drop-agent-app \
    -m "Merge feat/drop-agent-app — SP-A2 done (drop AGENT app, -5856 LoC)" && \
  git push origin main && \
  git branch -d feat/drop-agent-app
```

---

## Test Plan

- Build clean: `cargo build --target aarch64-unknown-none --release`
- Clippy clean: `cargo clippy --target aarch64-unknown-none --release`
- Boot smoke: PASS
- Cave private selftest: PASS
- Kernel binary size dropped by ~500K-1MB
- Lock-screen app cycle 1-7 walks every remaining app without panic (manually verified via QMP if `qmp_test.py` runs)
- `cargo deny check` still passes
- No `grep -r "crate::ai\|mod ai\|use crate::ai" src/` matches

## Commit Boundary

Single commit on a feature branch, merged into main via `--no-ff`.

## Estimated Duration

~45-60 minutes end-to-end including verification + sweeping for stragglers.

## Risk: AI-related test scripts may have other dependencies

If `scripts/qemu_*.py` has agent-specific harnesses (e.g., `qemu_chat_with_v3.py` per the `scripts/` inventory), those become dead scripts. Inventory and decide: delete or mark `# REMOVED: depended on AGENT app`. The `scripts/chat_with_v3.py` and `scripts/deploy_chatterbox.sh` are user-side dev tools, not in-tree kernel tests — leave them but flag.
