# Sphragis Gov-OS Master Implementation Plan

**Date:** 2026-05-16
**Phase:** 4 of 5 (Research → Requirements → Gap → **Master plan** → Per-subproject specs)
**Horizon:** 36 months (Q3 2026 → Q2 2029)
**Team assumption:** 3 engineers + 1 founder/biz-dev Y1, scaling to 5 by Y2 per Phase-1 financial model

**Inputs consumed:**
- [Research synthesis](../research/2026-05-16-gov-os-requirements.md)
- [Requirements spec (114 reqs)](../specs/2026-05-16-sphragis-gov-os-requirements.md)
- [Gap analysis (88 missing / 21 partial / 5 have)](../research/2026-05-16-gov-os-gap-analysis.md)

**Output:** This document. Decomposes the 88 missing + 21 partial requirements into **7 parallel tracks with ~35 sub-projects**, sequenced month-by-month, with explicit dependencies and decision gates. Phase 5 will produce per-subproject specs starting with the highest-priority items.

---

## Master Sequencing Principles

Drawn from Phase 3 §"Output for Phase 4" and confirmed by the strategic answers:

1. **Apache-2.0 relicense is Week 1.** Every PR after that lands under Apache-2.0.
2. **Procurement minimums front-loaded** (incorporation, SAM.gov, BIS) — cheap, gating, can run in parallel with engineering.
3. **Verification (VER) started early** — long lead time, #1 differentiator, late start kills timeline.
4. **Attestation primitives (ATT) by Month 9** — gates the demo bundle; demo bundle gates AFRL/DIU meetings.
5. **CNSA 2.0 algo alignment by Month 9** — 2027-01-01 hard deadline for new NSS acquisitions.
6. **UX productization is a parallel track** — different skill set, no contention with security/cert.
7. **x86_64 port is a 6-month dedicated sub-project** — substantial, blocks differentiator surfaces.
8. **CHERIoT-Ibex is a separate small-team play** — embedded variant, different procurement angle.
9. **First commercial gov revenue: Month 30-36** per the financial model.

---

## The 7 Tracks

| Track | Theme | Spans | Headcount | Output |
|---|---|---|---|---|
| **A** | Foundation: license + corp + anti-feature scope | Months 0-3 | 0.5 founder + 1 eng | Clean Apache-2.0 tree, registered entity, demo-bundle skeleton |
| **B** | Crypto + build-chain hardening (CNSA 2.0 ready) | Months 1-9 | 1.5 eng | CNSA-2.0-compliant build, SLSA-L4 attestation chain |
| **C** | Attestation + formal verification + audit-hardening | Months 3-18 | 2 eng | Attestation kernel primitive + Verus IFC proofs + WORM audit |
| **D** | Productization UX (window manager, installer, settings, package manager, POSIX) | Months 4-24 | 1-2 eng | "Real OS" feel; gov-buyer can self-serve a demo |
| **E** | Multi-hardware (x86_64, ARM server, CHERIoT) | Months 6-24 | 1 eng (rotating) | Sphragis runs where DoD lives, plus embedded CHERIoT variant |
| **F** | Documentation + certification engineering | Months 4-36 | 0.5 founder + 0.5 eng | Threat model, security target, NIST 800-53 matrix, FIPS 140-3 cert, DoD STIG accepted |
| **G** | Procurement + funding + biz-dev | Months 3-36 | 1 founder | GSA MAS, SBIR I→II→III, DARPA grants, ACT 3 teaming, first commercial gov revenue |

---

## Track A — Foundation (Months 0-3)

**Goal:** Clear the procurement runway. Apache-2.0 tree. Registered legal entity. Strategic-positioning docs in place. Demo bundle skeleton ready.

### A1 — Apache-2.0 relicense + anti-feature docs (Week 1-2)

**Requirements closed:** LIC-001, LIC-002, LIC-003 (CI enforcement), STRAT-004, ANTI-001 through ANTI-007 (explicit doc)

**Tasks:**
- Update `Cargo.toml` `license` field to `Apache-2.0`
- Replace `LICENSE` file with Apache-2.0 text
- Sweep SPDX headers across `src/` (Apache-2.0)
- Add `LICENSE-MIT` if dual-licensing later considered (defer)
- Add a `CONTRIBUTING.md` with DCO sign-off requirement (lighter than CLA)
- Add `cargo-deny.toml` enforcing Apache-2.0-compatible deps; wire into CI
- Run `cargo-deny` audit; remediate any GPL-family transitive deps
- Write `ANTI_FEATURES.md` at repo root documenting REQ-ANTI-001 through 007
- Update `README.md` with new license badge + 5-differentiator strategic positioning

**Dependencies:** None. Week 1.

**Deliverable:** Clean Apache-2.0 tree, contributor sign-off process, anti-features documented.

### A2 — Drop AGENT app (Week 2-3)

**Requirements closed:** STRAT-003 (gov build profile), ANTI-002

**Tasks:**
- Remove `src/ai/` directory (5,327 LoC)
- Remove `src/ui/apps/agent.rs` (529 LoC)
- Remove agent from `src/ui/apps/mod.rs` registration
- Remove agent from lock-screen app cycle
- Update `DESIGN_AI_AGENT.md` — keep as historical doc; mark "removed for gov-pivot 2026-05" header
- Verify boot smoke + cave private selftest still pass

**Dependencies:** A1 (license clean before structural removal).

**Deliverable:** No AI in the kernel TCB. Cleaner TCB measurement.

### A3 — Incorporation + procurement on-ramp (Months 1-3)

**Requirements closed:** PRC-001, PRC-002, CRT-007

**Tasks (founder-led, can run in parallel with engineering tracks):**
- Incorporate Delaware C-Corp (Stripe Atlas or local counsel, ~$500-2K)
- Register in SAM.gov (free, ~30-60 days end-to-end)
- Register in DSIP (DoD SBIR portal)
- Apply for CAGE code (free, ~14 days)
- Apply for UEI (replaces DUNS)
- File ECCN 5D002 initial classification with BIS (`crypt@bis.doc.gov`) and NSA (`web_site@nsa.gov`)
- Open business bank account
- Engage IP counsel for trademark filing (REQ-LIC-004 separate, can defer to Month 6)

**Dependencies:** Founder bandwidth. ~$5-15K legal/admin costs.

**Deliverable:** Eligible to receive federal contract awards.

### A4 — Marketing site + capability statement skeleton (Months 1-3)

**Requirements closed:** DOC-004, DOC-009, STRAT-001 (initial publication)

**Tasks:**
- Register domain (sphragis.com / sphragis.org / similar)
- Build static-site (Hugo or Astro): home + category claim + 5 differentiators + downloads + docs + blog + contact
- Write 8-12 page capability statement: company overview, NAICS codes (541511 Custom Computer Programming Services, 541512 Computer Systems Design, 541519 Other), core capabilities, certifications-in-progress, points of contact
- Deploy via Cloudflare Pages or GitHub Pages

**Dependencies:** A3 for company name; A1 for clean licensing message.

**Deliverable:** Public-facing brand. First link to share at AFCEA WEST 2026 (Feb).

### Track A Decision Gate (End of Month 3)
- ✅ Apache-2.0 tree clean and CI-enforced
- ✅ AGENT removed; gov build profile coherent
- ✅ Company incorporated; SAM.gov + CAGE + UEI active
- ✅ BIS encryption notification filed
- ✅ Marketing site live
- ✅ Capability statement v1 published

Goto Track B/C/D/E in parallel; Track G (procurement) ramps now that entity exists.

---

## Track B — Crypto + Build-Chain Hardening (Months 1-9)

**Goal:** CNSA 2.0 compliance well ahead of the 2027-01-01 deadline. SLSA-L4 build provenance. Crypto module boundary documented.

### B1 — CNSA 2.0 algorithm alignment (Months 1-4)

**Requirements closed:** CRY-001, CRY-002, CRY-003, CRY-004, CRY-005, CRY-006 (partial), CRY-007 (initial doc)

**Tasks:**
- **B1.1** — Confirm `ml-kem` crate parameter sets; expose `ML-KEM-1024` explicitly. Verify FIPS 203 conformance.
- **B1.2** — Confirm `ml-dsa` crate (currently `0.1.0-rc.8`) parameter sets; expose `ML-DSA-87` explicitly. Track upstream to stable release.
- **B1.3** — Implement LMS module (`src/crypto/lms.rs`) per NIST SP 800-208. Or vendor an audited crate (`hash-sig` or similar). For kernel + software signing.
- **B1.4** — Implement XMSS module (`src/crypto/xmss.rs`) per same standard. Pair with LMS for transition.
- **B1.5** — Add `src/crypto/sha512.rs` (standalone SHA-512 implementation or wrap RustCrypto crate). Some FIPS contexts require SHA-512 over SHA-384.
- **B1.6** — Add `sphragis-gov` build-profile feature flag (`#[cfg(feature = "gov-strict")]`) that:
  - Rejects AES-128 at the cipher-selection layer
  - Rejects SHA-1, SHA-256-for-signatures (allow SHA-256 for HMAC/legacy verify only)
  - Rejects RSA, ECDSA, ECDH unless behind a `legacy-pki` interop flag
  - Rejects ChaCha20-Poly1305 unless behind a CNSA-grade-context flag
- **B1.7** — Extend boot-time KAT (`crypto::run_self_tests()`) to cover: ML-KEM-1024, ML-DSA-87, LMS, XMSS, SHA-384, SHA-512, HMAC-SHA-384. Test vectors from NIST CAVP.
- **B1.8** — Convert RNG to **fail-closed** when RNDR absent (closes audit FS-H3); document the fallback policy.
- **B1.9** — Document the FIPS 140-3 cryptographic-module boundary: list every public API of the crypto module, every SSP it holds, every service it provides, every role it recognizes (operator vs cryptographic-officer vs maintenance). Output: `docs/FIPS_140_3_MODULE_BOUNDARY.md`.

**Dependencies:** A1 for clean tree.

**Deliverable:** CNSA-2.0-compliant `sphragis-gov` build profile. Boot-time KATs cover all CNSA algorithms. FIPS 140-3 module boundary documented (precondition for B5).

### B2 — Constant-time discipline + CI assertions (Months 3-5)

**Requirements closed:** CRY-010

**Tasks:**
- Audit every secret-dependent code path (HMAC compare, password compare, signature verify, cave-policy check). List in a doc.
- For each: add a property test that compares timing variance against a synthetic baseline; mark "constant-time" in a tracking matrix.
- Add `cargo-criterion` benchmarks asserting bounded variance on critical paths.
- Where rust-side constant-time is hard to assert from benchmarks, fall back to manual review checklist.

**Dependencies:** B1.

**Deliverable:** Documented constant-time discipline + CI gate.

### B3 — Reproducible builds verified end-to-end (Months 2-4)

**Requirements closed:** BLD-002, BLD-004 (CI), BLD-007

**Tasks:**
- Run `scripts/check_reproducible_build.sh` on two independent machines; diff outputs.
- Identify and eliminate non-determinism sources: build timestamps, source-path embedding, parallel build ordering, dep version drift.
- Pin Rust toolchain exactly (already done via `rust-toolchain.toml`).
- Pin all dependency versions exactly (no `^` in `Cargo.lock`).
- Sort file globs to remove filesystem-order dependency.
- Wire `cargo-audit` + `cargo-deny` into CI (Track A1 added the config; this wires the check).
- Wire `scripts/gen_sbom.py` into CI to produce SBOM artifact per release.
- Re-run `check_reproducible_build.sh` on every release tag; fail build on non-reproducibility.

**Dependencies:** A1 (license-clean tree avoids late dep churn).

**Deliverable:** Bit-for-bit reproducible builds verified. SBOM generated per release.

### B4 — Sigstore + in-toto + SLSA-L4 attestation chain (Months 4-7)

**Requirements closed:** BLD-001, BLD-003 (CI integration), BLD-005, BLD-008

**Tasks:**
- Set up sigstore cosign signing in CI. Use Fulcio for keyless signing; entries land in Rekor transparency log.
- Sign release binary + SBOM + in-toto attestation envelope.
- Implement LMS-signing of kernel image (uses B1.3 LMS module). Signature embedded in the boot envelope.
- Update boot stub (`src/arch/aarch64/linux_header.s` and Apple-side equivalent) to verify LMS signature before jump-to-Rust. Future: integrate with m1n1 chain on M4.
- Wire `scripts/build_intoto_attestation.py` into CI; produce per-release attestation chain (source → build → release).
- Claim SLSA v1.1 Level 4 in the release-page metadata; provide verification instructions.

**Dependencies:** B1 (LMS), B3 (reproducible builds — prerequisite for SLSA L4).

**Deliverable:** Every release has cosign signature + Rekor entry + in-toto chain + LMS-signed kernel. SLSA-L4 verifiable offline.

### B5 — FIPS 140-3 Level 1 lab pre-engagement (Months 6-9)

**Requirements closed:** CRY-008 (pre-engagement only — full cert is Track F5)

**Tasks (founder-led, with engineering input):**
- Identify accredited CMVP lab: Atsec (Austin TX), Leidos, or InfoGard.
- Initial scoping call: present FIPS 140-3 module boundary doc (B1.9), ask for ballpark $$ and timeline.
- Sign engagement letter (~$30-50K for L1 pre-engagement; full L1 cert $150-500K later in Track F).
- Begin algorithm-validation (CAVP) submissions for each cipher in scope; CAVP queue ~3-6 months.

**Dependencies:** B1.9 (module boundary doc), B2 (constant-time evidence).

**Deliverable:** Lab engaged. CAVP submissions in queue. Cost + timeline locked for Track F5.

### Track B Decision Gate (End of Month 9)
- ✅ All CNSA 2.0 algorithms shipping in `sphragis-gov` build
- ✅ Boot-time KATs cover them all
- ✅ Bit-for-bit reproducible builds; SBOM per release
- ✅ Sigstore + Rekor entries; LMS-signed kernel; SLSA-L4 claim
- ✅ FIPS 140-3 lab engaged; CAVP submissions queued

---

## Track C — Attestation + Formal Verification + Audit (Months 3-18)

**Goal:** Attestation kernel primitive (differentiator #3) + formal-verification harness (differentiator #1) + audit-trail upgraded.

### C1 — Attestation API + Caliptra/SEP integration (Months 3-9)

**Requirements closed:** ATT-001, ATT-002, ATT-003, ATT-005, ATT-006

**Tasks:**
- **C1.1** — Design `attest` module: `pub fn quote(nonce: &[u8], claims: &Claims) -> Quote`. Quote contains: kernel measurement, cave identity, claims, nonce, signature. Document in `DESIGN_ATTESTATION.md`.
- **C1.2** — Implement kernel-measurement: hash the loaded kernel image (text + rodata) + cargo metadata at boot. Store in a kernel-private attestation TCB key region.
- **C1.3** — Define `CaveIdentity` type: name + public-key + measurement (hash of the cave's loaded code + config). Persisted in SealFS, sealed under the kernel attestation key.
- **C1.4** — Implement attestation-key chain on M4: SEP-rooted. Boot Monitor → sepOS → m1n1 (measurement consumed) → Sphragis kernel measurement. Sphragis derives a kernel-attestation-key sealed to the boot measurement.
- **C1.5** — Caliptra integration design (for x86_64 future): API surface for Caliptra-rooted measurement consumption. Implementation deferred to E1 (x86_64 port).
- **C1.6** — Implement HSM-backed operator-CA pattern: PKCS#11 client in kernel (or via syscall surface) that talks to an external HSM. Operator-CA cert pre-installed; kernel attests against it. Operator-CA private key NEVER in Sphragis.
- **C1.7** — Quote serialization: use CBOR encoding. Quote includes ML-DSA-87 signature.
- **C1.8** — Verifier tool (user-mode): given a Quote, a CA bundle, and an expected-measurement allowlist, verify integrity + identity + freshness. Ships in `sphragis-tools/attestation-verifier/`.

**Dependencies:** B1 (ML-DSA-87 confirmed), B4 (LMS-signed kernel for boot measurement).

**Deliverable:** A cave can produce a Quote that an external verifier accepts. End-to-end attestation working on M4. Demo-bundle ready (PRC-010 unblocked).

### C2 — Formal verification harness + capability dispatcher proof (Months 3-12)

**Requirements closed:** VER-001, VER-005

**Tasks:**
- **C2.1** — Set up Verus toolchain in the repo. Add a `verification/` directory with Verus configuration and a single proven property as a smoke test.
- **C2.2** — Identify the 5-10K LoC of "verified subsystem": **capability dispatcher** in `src/caves/cave.rs` (specifically the `enter` / `exit` / `set_active` paths) + `src/kernel/syscall.rs` dispatch table.
- **C2.3** — Refactor the verified subsystem into a single Cargo crate with documented inputs/outputs. Preserves API surface to the rest of the kernel.
- **C2.4** — Write the Verus specification: "non-interference between caves" — given two caves A and B with no explicit IFC permission, the kernel state visible to A is unaffected by any syscall sequence from B.
- **C2.5** — Prove the specification holds. Iterate on any unprovable code paths; refactor for verifiability.
- **C2.6** — Document in `VERIFICATION.md`: what's proven, what's not, what assumptions the proof rests on, how to re-run the proof in CI.
- **C2.7** — Wire Verus check into CI; PRs that break the proof fail.

**Dependencies:** A1, C2 needs no other tracks; can run in parallel.

**Deliverable:** Verus proof of cave-capability non-interference. Differentiator #1 claim becomes truthful.

### C3 — IPC information-flow proof (Months 9-15)

**Requirements closed:** VER-002

**Tasks:**
- Extend C2 to AF_UNIX (`src/kernel/unix_sock.rs`), pipes (`src/kernel/pipe.rs`), shm (`src/kernel/shm.rs`).
- Specification: bytes written by cave A in namespace X are unobservable by cave B in namespace Y when no policy rule permits.
- Verus proof per IPC type.
- Update VERIFICATION.md.

**Dependencies:** C2.

**Deliverable:** IPC info-flow proof. Strengthens differentiator #1.

### C4 — Audit upgrade (Months 4-7)

**Requirements closed:** AUD-001 (SHA-384 upgrade), AUD-002 (WORM export), AUD-003 (missing categories), AUD-004 (offline verifier), AUD-006 (cave-scoped reads), ISO-009

**Tasks:**
- **C4.1** — Upgrade audit-chain HMAC from SHA-256 to SHA-384 (CNSA 2.0 alignment). Migration path: dual-chain during a transition window, then SHA-256 retired.
- **C4.2** — Add missing audit categories per NIAP `FAU_GEN.1`: Authentication, PrivilegeEscalation, KernelModuleLoad, UpdateApply, FileAccessConfig.
- **C4.3** — Implement WORM export: audit-ring writes a sealed (HMAC + LMS-signed) append-only file to a dedicated SealFS volume. Closes audit FS-H7.
- **C4.4** — Implement offline verifier tool: given the audit volume + the seal-key cert chain, verify continuity of the hash chain + all per-entry HMACs. Ships in `sphragis-tools/audit-verifier/`.
- **C4.5** — Cave-scoped audit reads: a cave reading the audit ring sees only its own entries unless it holds `audit:read-all` capability.
- **C4.6** — Update audit-related QMP self-tests.

**Dependencies:** B1 (SHA-384, LMS).

**Deliverable:** Audit chain HMAC-SHA-384, WORM export, offline verifier. NIAP `FAU_GEN.1` mapping complete.

### C5 — Isolation finishing (Months 4-6)

**Requirements closed:** ISO-005 (Cave-H2 native seccomp), ISO-008 (SOCK_DGRAM), ISO-003 (IPC/shm labeling)

**Tasks:**
- **C5.1** — Close audit Cave-H2: per-cave seccomp on native SVC≠0 path. Mirror the week-1 Linux-ABI fix. Add `caves/syscall_filter.rs` native-path entry.
- **C5.2** — Implement AF_UNIX SOCK_DGRAM with the same per-cave namespace scoping as SOCK_STREAM (week 12 pattern).
- **C5.3** — Extend CIPSO/CALIPSO labeling to IPC + shm: each IPC channel + shm region carries a sensitivity + integrity label; cross-cave access checks labels.
- **C5.4** — Add CI lint that any new syscall handler must declare its cave-policy gate (closes the structural pattern from REQ-ISO-004).

**Dependencies:** None.

**Deliverable:** Isolation completeness across all syscall paths + IPC types.

### Track C Decision Gate (End of Month 18)
- ✅ Attestation API live; M4 SEP-rooted Quote production + external verifier
- ✅ Verus proof: cave capability non-interference + IPC info-flow
- ✅ Audit HMAC-SHA-384 + WORM + offline verifier + cave-scoped reads
- ✅ Cave-H2 closed; SOCK_DGRAM landed; IPC labeled
- ✅ Demo bundle complete; ready for AFRL/DIU/DARPA meetings

---

## Track D — Productization UX (Months 4-24)

**Goal:** Sphragis feels like a "real OS" to a gov buyer. Multi-app concurrent UI, installer, settings, user accounts, package manager, POSIX toolbox.

### D1 — Multi-app concurrent UI + window manager (Months 4-9)

**Requirements closed:** UX-001, UX-007 (initial multi-monitor support)

**Tasks:**
- **D1.1** — Design: tiling window manager (Sway/i3 model). One cave per window. Status bar with cave indicator, audit-event count, attestation status.
- **D1.2** — Implement compositor in `src/ui/compositor.rs`. Damage tracking + region invalidation. Hardware-accelerated via M4 GPU (`src/drivers/apple/agx.rs`) where available; software-rendered fallback.
- **D1.3** — Refactor existing 8 apps to render into windows instead of full-screen.
- **D1.4** — Input routing: keyboard + virtio-tablet input dispatched to focused window.
- **D1.5** — Window-switching keybindings (Mod+Tab, Mod+1..9). Move/resize keybindings (vim-style hjkl).
- **D1.6** — Multi-monitor initial support via M4 `dcp.rs`: per-monitor workspaces.

**Dependencies:** A1.

**Deliverable:** Multi-app concurrent UI. "Looks like a real OS."

### D2 — Installer / boot ISO (Months 4-9)

**Requirements closed:** UX-002

**Tasks:**
- **D2.1** — Design installer flow: language → hardware probe → operator-CA selection or generate → unlock-passphrase setup → initial cave creation → first-boot tutorial.
- **D2.2** — Build UEFI-bootable ISO for x86_64 (depends on E1) — placeholder during E1 development.
- **D2.3** — Build M4-bootable package (m1n1 chainload installer + initial SealFS image). Ships as a `.dmg` for macOS install-helper, plus raw img for advanced users.
- **D2.4** — Implement first-boot wizard inside Sphragis: runs once, then is removed from the boot sequence.
- **D2.5** — Document installation procedure (`docs/INSTALLATION.md`).

**Dependencies:** A1; partially E1 (x86_64 ISO requires the port).

**Deliverable:** Self-serve installation. Gov buyer can try Sphragis without Sphragis devs at the keyboard.

### D3 — Settings + user accounts (Months 7-12)

**Requirements closed:** UX-003, UX-004

**Tasks:**
- **D3.1** — Implement multi-user model: each user has an operator-CA-attested identity, a per-user keyring, a per-user capability set (which caves they can enter).
- **D3.2** — Settings app: networking, audit-log review (links to UX-009 console), cave management (links to UX-010), attestation status, update apply, user accounts, time/date, locale, keyboard layout.
- **D3.3** — Login screen variant of lock screen: select user, enter passphrase. Operator-only mode for embedded variants stays available as a build-time choice.
- **D3.4** — Per-user audit logging: every authentication, login session, settings change recorded.

**Dependencies:** C1 (attestation surface for user identity), C4 (audit categories for auth events), D1 (windowed UI).

**Deliverable:** Settings app + multi-user accounts. NIAP `FIA` and `FMT` SFRs partially covered.

### D4 — Package manager (Months 9-15)

**Requirements closed:** UX-005

**Tasks:**
- **D4.1** — Design: TUF (The Update Framework) over HTTPS with CNSA-2.0 cipher suites only. Packages are LMS-signed; verification gates installation.
- **D4.2** — Package format: tarball with manifest (name, version, dependencies, capabilities-required, signature). Installed into a per-package read-only mount inside a target cave.
- **D4.3** — Repository protocol: signed metadata + delta updates. Mirror infrastructure on Cloudflare R2 or similar.
- **D4.4** — CLI tool (inside a cave) + UI integration in Settings.
- **D4.5** — Built-in repository: maintain a curated set of Sphragis-blessed packages (initial set is the POSIX toolbox from D5).

**Dependencies:** B4 (LMS-signed packages), B1 (CNSA-2.0 cipher suites), C1 (attestation of package source).

**Deliverable:** Update mechanism. POSIX toolbox installs via this path.

### D5 — POSIX analyst toolbox (Months 9-18)

**Requirements closed:** UX-006

**Tasks:**
- **D5.1** — Identify minimum viable toolbox: `vim`, `git`, `python3` (CPython), `ssh`/`scp` (OpenSSH), `tmux`, `curl`, `jq`, `grep`/`sed`/`awk`/`find` (BusyBox or coreutils), `make`, `bash` (or `dash`).
- **D5.2** — Cross-compile each for `aarch64-unknown-none` (or via a small static-musl toolchain). Validate each runs inside a cave under the Linux-ABI shim.
- **D5.3** — Extend Linux-ABI shim where individual tools require unimplemented syscalls. Bound expansion: do not extend the shim to "run everything." Document accepted shim surface.
- **D5.4** — Package each tool via D4.
- **D5.5** — Ship default-installed in `sphragis-gov` build (analyst SKU); optional in `sphragis-community`.

**Dependencies:** D4 (package management), existing Linux-ABI shim.

**Deliverable:** Analyst toolbox shipping. Real workflows possible.

### D6 — UX polish (Months 12-18)

**Requirements closed:** UX-007 (full multi-monitor), UX-008 (WiFi UX), UX-009 (audit-review console), UX-010 (cave-mgr extensions)

**Tasks:**
- **D6.1** — Multi-monitor: workspaces per monitor, drag-between-monitors, configurable arrangement.
- **D6.2** — WiFi networking config UI in Settings (uses `bcm_wifi.rs` driver).
- **D6.3** — Audit-review console: filter by category, severity, time range; offline-verify chain; export to WORM (calls C4.3 path).
- **D6.4** — Cave-management console extensions: per-cave attestation status display, IFC policy editor, resource quotas display + edit, "freeze cave" for forensic capture.

**Dependencies:** D1, D3, C1, C4.

**Deliverable:** UX polish complete. Sphragis competitive with Qubes / a hardened Linux desktop in feel.

### Track D Decision Gate (End of Month 24)
- ✅ Multi-app concurrent UI with WM
- ✅ Installer for both M4 + x86_64
- ✅ Settings + multi-user
- ✅ Package management + POSIX toolbox
- ✅ Multi-monitor + WiFi + audit/cave consoles polished
- → Sphragis demos as a "real OS" to a gov buyer

---

## Track E — Multi-Hardware (Months 6-24)

**Goal:** Sphragis runs where DoD lives (x86_64) + an embedded CHERIoT variant for the embedded-attestation gov niche.

### E1 — x86_64 port (Months 6-18)

**Requirements closed:** HW-002, HW-006, ATT-004 (TPM 2.0 on x86_64), ATT-002 (Caliptra-ready)

**Tasks:**
- **E1.1** — Pick reference hardware: Intel NUC 13 (commonly procured in fed) + ThinkPad X1 Carbon Gen 11 (analyst-laptop posture). Both UEFI.
- **E1.2** — Add `x86_64-unknown-none` build target. Boot stub in `src/arch/x86_64/`.
- **E1.3** — Port boot sequence: UEFI entry, ACPI parse, PCIe enumeration, APIC init.
- **E1.4** — Port drivers: serial (UART or 16550), virtio-net (already exists, may need x86 PCI scan integration), USB (xHCI), keyboard/mouse (i8042 + USB HID), GPU (basic VESA framebuffer; modern GPU drivers deferred).
- **E1.5** — MMU port: x86_64 paging structures, ASIDs via PCID, IBRS/STIBP/IBPB enablement, eIBRS, BHB clearing.
- **E1.6** — TPM 2.0 attestation: PCR extension during boot, attestation key derived from TPM.
- **E1.7** — Caliptra-compatible attestation API surface (for future Caliptra-equipped server SKUs).
- **E1.8** — QEMU x86_64 CI: port test scripts to QEMU x86_64; add to weekly CI matrix.
- **E1.9** — Bring-up testing on the NUC + ThinkPad hardware.

**Dependencies:** A1, C1 (attestation API surface for TPM integration).

**Deliverable:** Sphragis boots on Intel NUC + ThinkPad with TPM-rooted attestation. Where DoD lives.

### E2 — ARM server reference (Months 18-24, optional)

**Requirements closed:** HW-003

**Tasks:**
- Target Ampere Altra Max or AWS Graviton (Bare-Metal EC2 ARM instances are accessible without hardware purchase).
- Port boot stub, drivers (mostly already aarch64 but need server-class PCIe + NIC drivers).
- Validate attestation against ARM CCA (Confidential Compute Architecture) where available.

**Dependencies:** E1 (proves the multi-arch pattern works).

**Deliverable:** ARM server reference. Story for gov ARM adoption.

### E3 — CHERIoT-Ibex embedded variant (Months 12-24)

**Requirements closed:** HW-004, CHR-001, CHR-002, CHR-003

**Tasks:**
- **E3.1** — Acquire SCI Semiconductor ICENI dev kit OR lowRISC CHERIoT-Ibex FPGA dev kit (TBD pricing/availability).
- **E3.2** — Add `riscv32-cheriot-unknown-none` build target. Boot stub in `src/arch/cheriot/`.
- **E3.3** — Write `DESIGN_CHERI_MAPPING.md`: cave-to-CHERI-compartment mapping. Each cave's base+bound becomes a CHERI capability; cross-cave IPC becomes capability-mediated.
- **E3.4** — Port minimum kernel: scheduler, MMU-less memory protection via capabilities, IPC, syscall dispatch. **NO MMU** — CHERI provides isolation.
- **E3.5** — Ship `sphragis-embedded` SKU. Different procurement angle (embedded gov contracts).

**Dependencies:** A1; nothing else (separate-team play).

**Deliverable:** Sphragis-embedded boots on CHERIoT-Ibex. Differentiator #5 has a working artifact.

### Track E Decision Gate (End of Month 24)
- ✅ x86_64 reference platform supported with TPM attestation
- ✅ QEMU x86_64 in CI
- ✅ ARM server reference (optional, may slip to Y3)
- ✅ CHERIoT-Ibex embedded variant boots
- → Hardware story addresses "where can I actually run this?"

---

## Track F — Documentation + Certification Engineering (Months 4-36)

**Goal:** Documentation suitable for gov AOs. FIPS 140-3 cert. STIG accepted. CSfC component listed (if applicable).

### F1 — Threat model + security target + architecture doc (Months 4-9)

**Requirements closed:** DOC-002, DOC-003, DOC-005

**Tasks:**
- **F1.1** — Consolidate the existing `DESIGN_*.md` files into a single formal **Threat Model** document (`docs/THREAT_MODEL.md`). Attacker capabilities, assets, attack surfaces, mitigations, residual risk.
- **F1.2** — Write **Architecture Document for AOs** (`docs/ARCHITECTURE_FOR_AOS.md`). 30-50 pp. Audience: an Authorizing Official with NIST 800-53 knowledge, not necessarily a developer. System overview → TCB boundary → cave model → capability semantics → audit-chain integrity → attestation flow → crypto-module boundary → build-provenance chain.
- **F1.3** — Write **Security Target** (`docs/SECURITY_TARGET.md`) per CC Part 1 structure: TOE description, security problem definition (assumptions, threats, OSPs), security objectives, security requirements (SFRs, SARs), TOE summary specification.

**Dependencies:** C1 (attestation), C4 (audit), B1 (crypto boundary).

**Deliverable:** Documentation suitable for AO review.

### F2 — NIST SP 800-53 Rev 5.2.0 control-inheritance matrix (Months 6-12)

**Requirements closed:** DOC-006

**Tasks:**
- For each of 1,196 controls in SP 800-53 Rev 5.2.0: mark `fully satisfy` / `partially satisfy` / `customer-implemented`. Cite Sphragis feature or doc per row.
- Focus on the OS-relevant families first: AC, AU, CM, IA, SC, SI, MP, SA, SR, PT.
- Publish as `docs/NIST_800_53_INHERITANCE.md`.

**Dependencies:** F1 (architecture + threat model provide content).

**Deliverable:** Customer-inheritance matrix. FedRAMP-ready.

### F3 — Operator runbook (Months 9-15)

**Requirements closed:** DOC-001

**Tasks:**
- ~50-100 pp deployment + ops guide. Audience: sysadmin / SecOps engineer.
- Sections: deployment, hardening to STIG baseline, operator-CA integration, WORM audit export, cave-policy configuration, incident response, updates, troubleshooting.

**Dependencies:** D2 (installer exists), D3 (settings), F1 (architecture).

**Deliverable:** A sysadmin can deploy Sphragis from this doc.

### F4 — USENIX/NDSS whitepaper (Months 6-12)

**Requirements closed:** DOC-008

**Tasks:**
- Write 12-20 pp peer-review-quality paper: architecture, threat model, formal-verification results, benchmark data.
- Target: USENIX Security 2027 (submission due ~Feb 2027) OR NDSS 2027 (submission ~Sep 2026 — even earlier).
- Co-author with an academic if a relationship can be developed (helps placement).

**Dependencies:** C2/C3 (verification results), F1 (architecture).

**Deliverable:** Academic credibility marker. FFRDC introductions.

### F5 — FIPS 140-3 Level 1 cert (Months 12-30)

**Requirements closed:** CRT-001 (full cert; B5 was the pre-engagement)

**Tasks:**
- Lab testing (12-30 months wall-clock; mostly lab + NIST CMVP queue).
- Documentation: FIPS module boundary doc (B1.9) + key management policy + SSP management plan + operator/CO role separation + self-test policy.
- CAVP individual algorithm validations (started in B5).
- Submit to CMVP. Queue ~6-18 months.

**Dependencies:** B1.9, B5.

**Deliverable:** **FIPS 140-3 Level 1 certificate** issued. Required for federal crypto deployment.

### F6 — DoD STIG (Months 18-30)

**Requirements closed:** DOC-007, CRT-004

**Tasks:**
- Draft STIG against GP OS SRG. ~200-400 individual requirements. Map each to a Sphragis configuration or capability.
- Author in XCCDF/SCAP format.
- Submit to `disa.stig_spt@mail.mil`.
- Iterate on DISA review feedback. Validation + risk acceptance + RME signing.

**Dependencies:** F1, F2, F3 (architecture + controls + runbook).

**Deliverable:** **DoD STIG accepted by DISA.** Unlocks DoDIN deployment.

### F7 — NSA CSfC Components List submission (Months 24-36)

**Requirements closed:** CRT-009

**Tasks:**
- Identify the most-relevant CSfC capability package: likely **Mobile Access** or **Data-at-Rest**. Sphragis-gov's per-cave isolation + SealFS encryption is a natural fit.
- Engage with NSA CSfC program office. Submit Sphragis (gov build) for component listing review.
- Iterate on technical questions; provide attestation evidence + audit-chain evidence + threat model + ST.

**Dependencies:** F1, F5 (FIPS module helps).

**Deliverable:** Sphragis listed on **NSA CSfC Components List**. Major procurement unlock.

### F8 — NIAP PCL (Months 24-36, conditional)

**Requirements closed:** CRT-003

**Tasks:**
- If a NIAP Protection Profile fits (MDF PP v3.3 most likely, given Sphragis's fixed-app posture; GPCP v1.0 also plausible at the platform layer): engage CCTL for evaluation.
- Conditional on: (a) PP feasibility analysis, (b) sponsoring customer demanding it.
- Cost $150K floor → $500K-$2.5M.

**Deliverable:** NIAP PCL listing **IF** a feasible path exists.

### F9 — FedRAMP Moderate (Months 24-36, conditional)

**Requirements closed:** CRT-005

**Tasks:**
- Only pursue if a sponsoring agency emerges and a cloud-deployment use case exists.
- Use FedRAMP 20x path (faster, cheaper than traditional).
- 3PAO assessment + agency sponsor.

**Deliverable:** FedRAMP authorization **IF** sponsored.

### Track F Decision Gate (End of Month 36)
- ✅ Threat model + security target + architecture doc + operator runbook + NIST 800-53 matrix all published
- ✅ FIPS 140-3 Level 1 cert in hand
- ✅ DoD STIG accepted
- ✅ NSA CSfC submission in flight (acceptance often slips into Y4)
- ✅ USENIX/NDSS paper published
- → Sphragis is a procurable gov OS

---

## Track G — Procurement / Funding / Biz-Dev (Months 3-36)

**Goal:** First commercial gov revenue by Month 30-36. Sustainable funding via SBIR + DARPA + selective VC.

### G1 — GSA MAS IT-category offer (Months 9-18)

**Requirements closed:** PRC-003

**Tasks:**
- Submit MAS offer for SINs 511210 + 54151S.
- Two years past-performance requirement: achieved via first SBIR Phase I (Track G2) + 1-2 small commercial contracts.
- 6-12 month timeline from clean offer to award.

**Dependencies:** A3 (incorporation + SAM.gov), G2 (Phase I as past performance).

**Deliverable:** Sphragis on GSA MAS Schedule. Federal agencies can buy directly.

### G2 — SBIR Phase I submissions (Months 3-9)

**Requirements closed:** PRC-006

**Tasks:**
- Submit to 3 programs in parallel: DoD SBIR 26.1, AFWERX open topic, DARPA SBIR.
- Topics chosen to align with Sphragis differentiators.
- 80-90% rejection rate per submission → 3 submissions ≈ 70%+ chance of ≥1 award.
- Target: 1 Phase I award ($75K, 6mo) by month 9.

**Dependencies:** A3 (incorporated and SAM-registered).

**Deliverable:** First federal contract revenue.

### G3 — DARPA pitches (Months 6-12)

**Requirements closed:** PRC-007

**Tasks:**
- Attend DARPA Forecast to Industry (annual fall, DC). Request PM meetings.
- Pitch PROVERS, INSPECTA, RSSC PMs separately.
- Submit BAA responses to each as opportunities open.

**Dependencies:** Track A (incorporation + capability statement), Track C (verification work to point at).

**Deliverable:** DARPA awareness; potential follow-on funding ($3-15M per performer).

### G4 — ACT 3 / prime teaming (Months 6-15)

**Requirements closed:** PRC-004

**Tasks:**
- Identify prime teaming partners: AIS, CNF Technologies, Global InfoTek, Invictus, Radiance (all ACT 3 awardees).
- Pitch teaming arrangement: Sphragis as a niche capability they can layer into AFRL task orders.
- Sign teaming agreement + identify task-order opportunities.

**Dependencies:** Track A.

**Deliverable:** Subcontract revenue + prime relationships.

### G5 — IWRP / C5 consortium membership (Months 3-6)

**Requirements closed:** PRC-005

**Tasks:**
- Join one of: Information Warfare Research Project (IWRP), C5 consortium. $10-25K.
- Subscribe to RFS notices.
- Bid on relevant prototype work via OTAs.

**Dependencies:** A3.

**Deliverable:** OTA-award eligibility.

### G6 — Conference attendance (Months 3-ongoing)

**Requirements closed:** PRC-011

**Tasks (per quarter):**
- AFCEA WEST (Feb 2026 — first opportunity), TechNet Cyber (Jun), TechNet Indo-Pacific (Oct)
- AUSA Annual (Oct)
- Sea-Air-Space (Apr)
- DEF CON (Aug)
- DARPA Forecast to Industry (fall)
- RSA Gov Track (May)
- NSWC Crane "Connect to Crane"
- National Cyber Summit (Huntsville)
- USENIX Security / NDSS paper presentation (per F4)

**Dependencies:** Track A.

**Deliverable:** Pipeline of gov-buyer conversations.

### G7 — SBIR Phase II → III conversion (Months 15-36)

**Requirements closed:** Revenue path

**Tasks:**
- Convert Phase I results into Phase II proposal ($1.25M, 21mo).
- Mid-Phase II: pitch In-Q-Tel.
- Late Phase II: convert to Phase III sole-source contract OR ACT 3 task order.
- STRATFI bridge for Phase II→III gap if needed ($3-15M with matching).

**Dependencies:** G2 (Phase I).

**Deliverable:** **First commercial gov revenue at Month 30-36.**

### G8 — In-Q-Tel pitch (Months 18-24)

**Requirements closed:** PRC-008

**Tasks:**
- After Phase II validates: pitch In-Q-Tel as "secure compute substrate for IC mission systems."
- Average check $500K-$3M + intro letter to IC tech customers.

**Dependencies:** G7 (Phase II in progress).

**Deliverable:** Strategic capital + IC intro letter.

### G9 — Demo bundle assembly + maintenance (Months 9-ongoing)

**Requirements closed:** PRC-010

**Tasks:**
- Assemble at Month 9: M4 boot + attestation quote (C1) + audit log walk + threat model (F1) + cap statement (A4) + demo deck.
- Maintain over time: each new feature lands a 2-min addition to the demo deck.
- Use at every gov meeting from Month 9 onward.

**Dependencies:** C1 done; A4 done.

**Deliverable:** "Ready to demo on 24 hours notice" posture.

### Track G Decision Gates

- **Month 9:** 1+ SBIR Phase I award; first AFRL/DIU/DARPA meeting completed
- **Month 15:** SBIR Phase II started; In-Q-Tel pitch scheduled; ACT 3 subcontract identified
- **Month 24:** Phase II mid-point; FedRAMP customer (if any) identified; CSfC submission in flight
- **Month 30-36:** **First commercial gov revenue ($500K-$5M+).** Phase III sole-source OR ACT 3 task order OR direct GSA MAS sale.
- **Month 48 (Year 4 — beyond this plan):** $2-10M ARR plausible if Phase III converts and second program lands.

---

## Cross-Track Dependency Graph

```
A1 (relicense) → A2 (drop AGENT) → B/C/D/E/F all unblocked
A3 (incorporation) → G2/G3/G4/G5 (all procurement)
A4 (marketing site) → G3/G4/G6 (conference + pitch materials)

B1 (CNSA crypto) → B4 (LMS-signed kernel) → B5 (FIPS lab)
B3 (reproducible) → B4 (sigstore needs determinism) → F5 (FIPS cert)
B1 + C1 → C4 (audit upgrade uses SHA-384 + LMS)
B1 → D4 (package manager LMS verification)

C1 (attestation) → C4 (sealed audit), D3 (user identity), E1 (TPM on x86)
C2 (Verus harness) → C3 (IPC proof)
C5 (isolation finishing) — independent

D1 (window manager) → D2 (installer UX), D3 (settings UI), D6 (multi-monitor)
D4 (package manager) → D5 (POSIX toolbox shipping)

E1 (x86_64 port) → D2 x86_64 ISO; E2 (ARM server)

F1 (threat model + ST) → F2 (controls matrix), F3 (runbook)
F1 + B1.9 → F5 (FIPS cert)
F1 + F2 + F3 → F6 (STIG)
F5 → F7 (CSfC submission)

G2 (Phase I) → G7 (Phase II → III) → G8 (In-Q-Tel)
G3 (DARPA) → potentially fuels C2/C3 verification work
```

**Critical-path items** (delays here delay everything downstream):
1. **A1 (Apache-2.0 relicense)** — Week 1
2. **A3 (incorporation)** — Months 1-3
3. **B1 (CNSA crypto)** — Months 1-4
4. **C1 (attestation)** — Months 3-9
5. **B5 + F5 (FIPS engagement → cert)** — Months 6-30
6. **G2 (SBIR Phase I)** — Months 3-9

---

## Resource / Staffing Model

| Year | Headcount | Tracks owned |
|---|---|---|
| Y1 (M0-12) | 1 founder + 3 engineers (Eng1: crypto/verification; Eng2: kernel/isolation; Eng3: UX) | Founder: A3/A4/F1-4/G all. Eng1: B1-5/C2-3. Eng2: C1/C4-5. Eng3: D1-3. |
| Y2 (M13-24) | + Eng4 (x86_64 port + cert engineering) | Eng4: E1/F5/F6. Existing team continues Y1 tracks. |
| Y3 (M25-36) | + Eng5 (CHERIoT + ARM server + customer engineering) | Eng5: E2/E3 + customer ATO support. |

**Burn**: ~$700-850K/yr salaries + $25-50K hardware/yr + $30-100K legal/yr + $200-400K certifications Y2-3 + $60K conferences = **~$1.0-1.2M/yr Y1, $1.2-1.5M/yr Y2-3**. **3-year total ~$3.5M** per Phase 1 financial model.

---

## Decision Gate Summary (the major checkpoints)

| Month | Gate | Pass criteria | Pivot if fail |
|---|---|---|---|
| 3 | Foundation | A1-A4 all green; entity registered | Delay G2 by 1 month; otherwise no pivot |
| 9 | Demo-ready | A1-A4, B1-3, C1, demo bundle (PRC-010) all green; 1 SBIR Phase I award | If no Phase I award: re-submit; if no demo bundle: delay G3/G6 outreach |
| 15 | Phase II + product | SBIR Phase II started; D1+D2 + multi-app UI shipping in community build | If no Phase II: bridge via friends-family round; if D1/D2 not shipping: delay D3-D6 |
| 24 | Certification + revenue path | F5 lab testing complete, F6 STIG submitted, E1 x86_64 done, D6 UX polish complete, G7 mid-Phase-II | If F5 slips: extend timeline (no avoiding the CMVP queue); if G7 fails: pursue ACT 3 sub or direct GSA path |
| 30-36 | First commercial gov revenue | $500K-$5M+ revenue from Phase III / ACT 3 / GSA / FedRAMP / CSfC-driven sale | If no revenue: extend runway via second-round VC + continued SBIR submissions |

---

## What Phase 5 Will Produce

Phase 5 = per-subproject specs. Sequenced by start-date:

**Immediate (Month 0-1) priority:**
- SP-A1 spec — Apache-2.0 relicense + anti-feature docs
- SP-A2 spec — Drop AGENT
- SP-A3 spec — Incorporation + procurement on-ramp
- SP-A4 spec — Marketing site + capability statement

**Month 1-3 priority:**
- SP-B1 spec — CNSA 2.0 algorithm alignment
- SP-G2 spec — SBIR Phase I submissions

**Month 3-9 priority:**
- SP-C1 spec — Attestation API + Caliptra/SEP
- SP-C2 spec — Verus harness + capability dispatcher proof
- SP-B4 spec — Sigstore + in-toto + SLSA-L4

Each spec follows the brainstorming-skill format: clear scope, design, success criteria, sequenced tasks, tests, commit boundary. Phase 5 writes them in order.

---

## Risks + Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| SBIR Phase I all 3 rejected | 30-40% (per Agent D 80-90% per-submission rejection) | Submit to next cycle (DoD 26.2); pursue AFWERX TACFI bridge; pursue commercial SBIR-equivalents (NSF, DOE) |
| FIPS 140-3 CMVP queue >24 months | High | Submit early (Month 12 not Month 18); use the longest-queue items first |
| Verus tooling regression on the kernel-scale proof | Medium | Identify a smaller subsystem if dispatch is too large; AWS Kani as fallback (more mature for Rust) |
| No DARPA program fit | Medium | Stay informed via DARPA Forecast; keep submission tempo |
| Apple changes M4 firmware in a way that breaks our boot path | Low-Medium | Maintain ability to revert to a known-good macOS Recovery firmware; engage Asahi Linux community for early warning |
| AGPL→Apache-2.0 has a contributor we can't reach | Low | All commits are Kaden + claude-flow co-author; both can authorize relicense |
| Prime channel rejects us because we're too small | Medium | SBA Mentor-Protégé Program; lean on ACT 3 sub-channel where we're explicitly a sub |
| In-Q-Tel pass | Medium (they don't fund pure OS plays) | Frame as "secure compute substrate for IC mission" not "an OS"; not blocking — Phase II + Phase III + STRATFI fund without IQT |
| First commercial gov revenue slips past Month 36 | Medium | Extend runway by 6-12 months; pursue STRATFI; consider strategic-investor round at Month 24 |
| Competing Rust gov-OS startup emerges in 2027-2028 | Medium-High over time | Moat is the 14-week audit history + M4 boot + Verus proof artifact + AGPL→Apache → SLSA-L4 chain — all hard to replicate quickly |

---

## What Happens Next

**Immediately (this conversation):** Phase 5 begins. We write the first 3-4 sub-project specs (Track A items, since they're Week 1):

1. **SP-A1**: Apache-2.0 relicense + anti-feature docs
2. **SP-A2**: Drop AGENT
3. **SP-A3**: Incorporation + procurement on-ramp (founder-led; spec is more checklist than code-task list)
4. **SP-A4**: Marketing site + capability statement skeleton

Each spec follows the writing-plans skill format: file structure, task list with bite-sized steps, test plan, commit boundary.

**Then we execute SP-A1, SP-A2 immediately in this session** (they're tractable in a single sitting), and queue SP-A3/A4 for founder-time follow-up.

**Beyond this session:** the user runs the master plan with this doc as the source of truth. Re-visit at decision gates (Months 3, 9, 15, 24, 30-36).
