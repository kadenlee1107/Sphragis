# SP-A1: Apache-2.0 Relicense + Anti-Feature Docs

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Relicense Sphragis from AGPL-3.0-or-later to Apache-2.0 so the prime channel (Lockheed, Northrop, Booz Allen, etc.) becomes accessible. Add CI enforcement of license compatibility. Publish explicit anti-feature documentation.

**Architecture:** Single-PR change. License metadata + LICENSE file + SPDX sweep + new CI config + 2 new docs. No code logic changes. Boot smoke + cave private selftests must continue to pass.

**Tech Stack:** Cargo, cargo-deny, GitHub Actions (CI), Markdown.

**Requirements closed:** LIC-001, LIC-002, LIC-003, STRAT-004, ANTI-001 through ANTI-007 (explicit doc).

---

## File Structure

- **Modify:** `Cargo.toml` (license field)
- **Replace:** `LICENSE` (Apache-2.0 text, was AGPL text)
- **Create:** `CONTRIBUTING.md` (DCO sign-off requirement)
- **Create:** `cargo-deny.toml` (license + advisory enforcement)
- **Create:** `.github/workflows/license-check.yml` (CI wire-up)
- **Create:** `ANTI_FEATURES.md` (explicit non-goals doc)
- **Modify:** `README.md` (license badge + 5-differentiator framing)
- **Sweep:** all `src/**/*.rs` for SPDX header updates

---

### Task 1: Update `Cargo.toml` license field

**Files:** Modify `Cargo.toml:5`

- [ ] **Step 1: Edit Cargo.toml**

Change line 5 from:
```toml
license = "AGPL-3.0-or-later"
```
to:
```toml
license = "Apache-2.0"
```

- [ ] **Step 2: Verify build**

Run: `cargo build --target aarch64-unknown-none --release`
Expected: clean build (no license-related warnings).

- [ ] **Step 3: Stage**

Run: `git add Cargo.toml`

---

### Task 2: Replace `LICENSE` file with Apache-2.0 text

**Files:** Replace `LICENSE`

- [ ] **Step 1: Download canonical Apache-2.0 text**

Run: `curl -sL https://www.apache.org/licenses/LICENSE-2.0.txt -o LICENSE.new`

- [ ] **Step 2: Verify content**

Run: `head -5 LICENSE.new`
Expected: starts with "Apache License" / "Version 2.0, January 2004".

- [ ] **Step 3: Replace LICENSE**

Run: `mv LICENSE.new LICENSE`

- [ ] **Step 4: Add appendix block with copyright**

Append to `LICENSE`:
```
Copyright 2026 Kaden Lee and contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

- [ ] **Step 5: Stage**

Run: `git add LICENSE`

---

### Task 3: SPDX header sweep across `src/`

**Files:** All `src/**/*.rs`

Sphragis currently does not have universal SPDX headers (project-wide pattern is to omit them per `feedback_decisive_defaults`). We will NOT add headers retroactively; instead we'll document the project-wide license in a top-level `NOTICE` and rely on the `Cargo.toml` `license` field.

- [ ] **Step 1: Create NOTICE file**

Create `NOTICE` at repo root with:
```
Sphragis Operating System
Copyright 2026 Kaden Lee and contributors

This product is licensed under the Apache License, Version 2.0.
See the LICENSE file for the full license text.

This product includes software developed by the Sphragis project
and third-party Rust crates listed in Cargo.toml. See SBOM artifacts
in releases for the complete dependency graph and per-dependency licenses.
```

- [ ] **Step 2: Stage**

Run: `git add NOTICE`

- [ ] **Step 3: Check for any existing AGPL/GPL SPDX headers in source**

Run: `grep -rn "SPDX-License-Identifier: AGPL\|SPDX-License-Identifier: GPL" src/ || echo "none found"`

If any found: update each to `SPDX-License-Identifier: Apache-2.0`.
If none: proceed.

---

### Task 4: Create `CONTRIBUTING.md` with DCO sign-off

**Files:** Create `CONTRIBUTING.md`

- [ ] **Step 1: Write CONTRIBUTING.md**

Create `CONTRIBUTING.md` with:

```markdown
# Contributing to Sphragis

Thank you for your interest in contributing. Sphragis is a security-first
bare-metal Rust microkernel targeting government and high-assurance use.
We use a lightweight contribution process.

## Developer Certificate of Origin (DCO)

By contributing, you certify the statements in the [Developer Certificate
of Origin v1.1](https://developercertificate.org). Every commit must be
signed off using `git commit -s` (which adds a `Signed-off-by:` trailer).

The DCO is preferred over a CLA because it does not assign copyright;
it certifies you have the right to contribute the code under our
license (Apache-2.0). Apache-2.0 + DCO is the same model used by the
Linux kernel and most modern open-source infrastructure projects.

## License

All contributions are licensed under Apache-2.0. See [LICENSE](LICENSE).

## Process

1. Open an issue describing the change you intend to make. Brief is fine.
2. Fork and create a feature branch (`fix/<scope>-<short-desc>` or `feat/<scope>-<short-desc>`).
3. Make your changes. Run `cargo build --target aarch64-unknown-none --release` and `cargo clippy --target aarch64-unknown-none --release`; both must pass clean.
4. Run `python3 scripts/qemu_boot_smoke.py` and `python3 scripts/qemu_cave_private_selftest.py`; both must PASS.
5. Open a PR. Include your DCO sign-off (`git commit -s`).
6. Address review feedback. Maintainers will merge.

## Security disclosures

For security issues, please email security@sphragis.dev (or open a
GitHub security advisory if the repo has them enabled). Do not file
public issues for unpatched vulnerabilities.

## Code of conduct

Be respectful. Sphragis is a small project and we want it to stay welcoming.
```

- [ ] **Step 2: Stage**

Run: `git add CONTRIBUTING.md`

---

### Task 5: Create `cargo-deny.toml` license + advisory config

**Files:** Create `cargo-deny.toml`

- [ ] **Step 1: Write cargo-deny.toml**

Create `cargo-deny.toml` with:

```toml
# Sphragis dependency policy. Enforced in CI by cargo-deny.
# Apache-2.0 license + zero GPL/AGPL/SSPL/Commons-Clause deps.

[graph]
all-features = false
no-default-features = false

[licenses]
unlicensed = "deny"
allow = [
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "CC0-1.0",
    "Zlib",
]
deny = [
    "GPL-2.0",
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "AGPL-3.0",
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "LGPL-2.1",
    "LGPL-2.1-only",
    "LGPL-2.1-or-later",
    "LGPL-3.0",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
    "SSPL-1.0",
    "Commons-Clause",
    "BUSL-1.1",
]
confidence-threshold = 0.92

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

- [ ] **Step 2: Install cargo-deny locally + run it**

Run: `cargo install cargo-deny --locked` (skip if already installed)
Run: `cargo deny check`
Expected: should pass; if it flags a current dep, investigate immediately.

- [ ] **Step 3: Stage**

Run: `git add cargo-deny.toml`

---

### Task 6: Wire cargo-deny + cargo-audit into CI

**Files:** Create `.github/workflows/license-check.yml`

- [ ] **Step 1: Check whether `.github/workflows/` exists**

Run: `ls -la .github/workflows/ 2>&1 | head -5`
If directory doesn't exist: run `mkdir -p .github/workflows`.

- [ ] **Step 2: Write workflow**

Create `.github/workflows/license-check.yml`:

```yaml
name: License + Advisory Check

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  cargo-deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-deny
        run: cargo install cargo-deny --locked
      - name: Run cargo-deny
        run: cargo deny check

  cargo-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit --locked
      - name: Run cargo-audit
        run: cargo audit
```

- [ ] **Step 3: Stage**

Run: `git add .github/workflows/license-check.yml`

---

### Task 7: Create `ANTI_FEATURES.md` at repo root

**Files:** Create `ANTI_FEATURES.md`

- [ ] **Step 1: Write ANTI_FEATURES.md**

Create `ANTI_FEATURES.md` with the seven anti-features from REQ-ANTI-001 through ANTI-007:

```markdown
# Sphragis Anti-Features (Explicit Non-Goals)

Sphragis is a security-first bare-metal Rust microkernel for gov / high-assurance use. This document lists things Sphragis explicitly will NOT do. Knowing what we won't build is as important as knowing what we will.

## ANTI-001: No full functional-correctness proof of the whole kernel
**What we don't do:** Attempt to prove every line of the kernel correct end-to-end (the seL4 model).
**Why:** seL4 has a 15-year, ~25-person-year head start. We cede that lane. We claim **information-flow non-interference on critical subsystems** (capability dispatcher, IPC, scheduler invariants) via Verus or Kani — a more tractable proof effort that still produces a defensible "verified" claim for gov procurement.

## ANTI-002: No AI/LLM/ML in the kernel critical path
**What we don't do:** Run language models, neural nets, or reinforcement-learning policies inside the kernel TCB.
**Why:** Non-deterministic, hard to certify, expands the attack surface, and inference latency makes scheduler integration counterproductive. AI features can ship as user-mode caves in the community build, but the `sphragis-gov` build excludes them entirely.

## ANTI-003: No QKD integration as a featured capability
**What we don't do:** Market quantum key distribution as a Sphragis differentiator.
**Why:** NSA's stated preference is post-quantum cryptography (PQC), not QKD. Sphragis maintains a key-plane abstraction that *could* swap in a QKD-derived link key for a specific tactical comms scenario, but we don't lead with it.

## ANTI-004: No Linux binary compatibility promise
**What we don't do:** Promise to run arbitrary Linux binaries.
**Why:** A full Linux ABI compatibility surface drags in the same TCB shape we're trying to avoid. Sphragis ships a narrow Linux ABI shim (`src/caves/linux/`) sufficient to host an analyst-toolbox (vim, git, python, ssh, tmux) under heavy capability restrictions, but binary compat is not a goal.

## ANTI-005: No weak cryptography in the gov build
**What we don't do:** Allow AES-128, SHA-1, MD5, RSA-2048, ECDSA-256, plain ChaCha20-Poly1305 (without CNSA-grade context), DH-2048, or other below-CNSA-2.0 algorithms in the `sphragis-gov` build profile.
**Why:** CNSA 2.0 deadlines (2027-01-01 for new NSS acquisitions; 2033 for exclusive use) require modern algorithms. Weak algorithms are accepted in `sphragis-community` only for legacy-interop scenarios; the gov SKU rejects them at the policy layer.

## ANTI-006: No closed-source kernel components
**What we don't do:** Ship binary blobs in the Sphragis kernel or first-party drivers.
**Why:** Auditable from sand to syscall. Apache-2.0 source for every line we own. Hardware-vendor firmware (M4 SEP firmware is Apple-signed) exists at the boundary; we attest TO it but don't sign on its behalf.

## ANTI-007: No GPL/AGPL/copyleft dependencies
**What we don't do:** Accept GPL-2.0, GPL-3.0, AGPL, LGPL, SSPL, BUSL, or Commons-Clause dependencies.
**Why:** Apache-2.0 license requires compatibility. Prime-integration friction is real — primes (Lockheed, Northrop, etc.) will not embed copyleft code into their proprietary product lines. Enforced in CI via `cargo-deny.toml`.

---

These anti-features are part of Sphragis's strategic identity, not arbitrary limits. Changing one of them is a category-redefinition and requires explicit project-leadership approval.
```

- [ ] **Step 2: Stage**

Run: `git add ANTI_FEATURES.md`

---

### Task 8: Update `README.md` with license badge + 5-differentiator framing

**Files:** Modify `README.md`

- [ ] **Step 1: Read current README**

Read `README.md` to understand current structure.

- [ ] **Step 2: Update the license badge / line**

Find the existing license line/badge (likely `[![License: AGPL...]]` or text "License: AGPL-3.0"). Replace with Apache-2.0 badge:

```markdown
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
```

- [ ] **Step 3: Add strategic-positioning header**

Add (near the top, after the project name + tagline):

```markdown
## What Sphragis Is

Sphragis is a **sovereign-grade attested-cave OS for the post-quantum, capability-hardware era** — a security-first bare-metal Rust microkernel for government and high-assurance use.

**Five differentiators:**

1. **Rust microkernel + information-flow proofs** on the capability and IPC subsystems (via Verus or Kani — non-interference, not full functional correctness)
2. **CNSA-2.0-native, PQC-only crypto** — ML-KEM-1024, ML-DSA-87, AES-256, SHA-384 by default in the gov build; no classical fallback
3. **Attestation as a first-class kernel primitive** — every cave is an attestable identity, rooted in Caliptra / Apple SEP / TPM 2.0 hardware roots
4. **Reproducible, bootstrappable, SLSA-L4 build chain** — bit-for-bit reproducible from source, sigstore-signed, in-toto-attested
5. **CHERI-ready architecture** — caves map 1:1 to CHERI compartments; CHERIoT-Ibex embedded variant ships in 2026-27

See [ANTI_FEATURES.md](ANTI_FEATURES.md) for explicit non-goals.
```

- [ ] **Step 4: Stage**

Run: `git add README.md`

---

### Task 9: Verify everything builds + selftests pass

- [ ] **Step 1: Build**

Run: `cargo build --target aarch64-unknown-none --release`
Expected: clean build.

- [ ] **Step 2: Clippy**

Run: `cargo clippy --target aarch64-unknown-none --release`
Expected: clean.

- [ ] **Step 3: Boot smoke test**

Run: `python3 scripts/qemu_boot_smoke.py`
Expected: `[smoke] PASS — kernel boots, all required subsystems init`.

- [ ] **Step 4: Cave private selftest**

Run: `python3 scripts/qemu_cave_private_selftest.py`
Expected: `[cave-private] PASS — per-cave L1 isolation property verified`.

- [ ] **Step 5: Local cargo-deny check**

Run: `cargo deny check`
Expected: pass with no violations.

If any of the above fail: stop, diagnose, fix before proceeding.

---

### Task 10: Commit + push + merge

- [ ] **Step 1: Create branch**

Run: `git checkout -b feat/apache-2-0-relicense`

- [ ] **Step 2: Commit (with DCO sign-off)**

Run:
```bash
git commit -s -m "$(cat <<'EOF'
feat: relicense Sphragis from AGPL-3.0-or-later to Apache-2.0

Per the gov-OS productization plan (SP-A1). The AGPL license blocks
prime integration (Lockheed, Northrop, Booz Allen, etc. will not
embed copyleft code into their proprietary product lines), which
closes the most realistic gov-revenue channel for a small Rust-OS
vendor.

Changes:
  - Cargo.toml: license field -> Apache-2.0
  - LICENSE: replaced with canonical Apache-2.0 text + copyright block
  - NOTICE: created (Apache-2.0 standard)
  - CONTRIBUTING.md: created, DCO sign-off required
  - cargo-deny.toml: license + advisory enforcement; allows
    Apache-2.0 / MIT / BSD / ISC / Unicode / CC0 / Zlib;
    denies GPL family / AGPL / LGPL / SSPL / BUSL / Commons-Clause
  - .github/workflows/license-check.yml: cargo-deny + cargo-audit on
    every push + PR
  - ANTI_FEATURES.md: explicit project non-goals (7 anti-features)
  - README.md: license badge + 5-differentiator strategic-positioning header

No code logic changes. Boot smoke + cave private selftest both PASS.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

- [ ] **Step 3: Push**

Run: `git push -u origin feat/apache-2-0-relicense`

- [ ] **Step 4: Merge to main**

Run:
```bash
git checkout main && \
  git merge --no-ff feat/apache-2-0-relicense \
    -m "Merge feat/apache-2-0-relicense — SP-A1 done (relicense AGPL→Apache-2.0)" && \
  git push origin main && \
  git branch -d feat/apache-2-0-relicense
```

---

## Test Plan

- Build clean: `cargo build --target aarch64-unknown-none --release`
- Clippy clean: `cargo clippy --target aarch64-unknown-none --release`
- `cargo deny check` passes (no GPL-family deps, no advisories)
- `python3 scripts/qemu_boot_smoke.py` → PASS
- `python3 scripts/qemu_cave_private_selftest.py` → PASS
- License badge in README renders correctly
- LICENSE file content matches `https://www.apache.org/licenses/LICENSE-2.0.txt`
- CI workflow runs on the first post-merge push and passes

## Commit Boundary

Single commit on a feature branch, merged into main via `--no-ff`. Per project convention.

## Estimated Duration

~30-45 minutes end-to-end including verification.
