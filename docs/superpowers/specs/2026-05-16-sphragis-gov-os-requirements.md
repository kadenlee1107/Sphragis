# Sphragis Gov-OS Requirements Spec

**Date:** 2026-05-16
**Phase:** 2 of 5 (Research → **Requirements** → Gap analysis → Master plan → Per-subproject plans)
**Source:** [docs/superpowers/research/2026-05-16-gov-os-requirements.md](../research/2026-05-16-gov-os-requirements.md)
**Strategic decisions baked in:**
- License: **Apache-2.0** (relicense from AGPL-3.0-or-later)
- AGENT app: **dropped entirely** from the codebase
- DARPA targeting: **PROVERS + INSPECTA + RSSC + future BAAs** in parallel

This document enumerates every requirement Sphragis must satisfy to be a credible "sovereign-grade attested-cave OS for the post-quantum, capability-hardware era." Phase 3 (gap analysis) will map each REQ to have / partial / missing against the Sphragis current state inventory. Phase 4 (master implementation plan) will sequence them.

**Requirement IDs:** `REQ-<AREA>-NNN`. Areas: `STRAT`, `LIC`, `CRY`, `ISO`, `ATT`, `AUD`, `BLD`, `VER`, `CHR`, `UX`, `HW`, `DOC`, `CRT`, `PRC`, `ANTI`.

**Priorities:** **P0** = must-have for "gov-grade" claim. **P1** = should-have within 18 months. **P2** = roadmap, 18-36 months.

---

## 1. Strategic Positioning (STRAT)

### REQ-STRAT-001 (P0) — Defensible category claim
Sphragis shall be marketable as "**Sovereign-grade attested-cave OS for the post-quantum, capability-hardware era**." Every customer-facing artifact (website, capability statement, demo deck, whitepaper) shall use this category framing or a derivative.

### REQ-STRAT-002 (P0) — Five-differentiator claim discipline
Every gov-facing material (slide deck, briefing, RFP response) shall foreground the **five differentiators** in this order, with at least one concrete artifact backing each:
1. Rust microkernel + information-flow proofs on capability/IPC subsystem (artifact: Verus or Kani proof script + verified-property statement)
2. CNSA-2.0-native, PQC-only kernel-mediated crypto (artifact: cipher-suite policy + boot-time crypto self-test output)
3. Attestation as first-class kernel primitive (artifact: live attestation quote from a cave, verified against a Caliptra/SEP/Pluton root)
4. Reproducible, bootstrappable, SLSA-L4 build chain (artifact: bit-for-bit-identical re-build evidence from a clean machine)
5. CHERI-ready architecture (artifact: cave-to-CHERI-compartment mapping document; CHERIoT-Ibex prototype boot)

### REQ-STRAT-003 (P0) — Two-build-profile model
The Sphragis tree shall produce **two distinct build profiles**:
- **`sphragis-community`** — full feature set minus restricted-export content; AGPL→Apache-2.0 relicensed; for OSS adoption, academic use, contributor onramp.
- **`sphragis-gov`** — gov-grade SKU; strips AGENT app entirely; ships only CNSA-2.0 cipher suites; includes the in-toto attestation chain and STIG hardening defaults.

Both profiles share the same kernel TCB.

### REQ-STRAT-004 (P1) — Explicit non-goals (anti-features)
The product shall **not** claim or pursue:
- Full functional-correctness proof of the whole kernel (cede to seL4; we claim info-flow only on critical subsystems).
- AI-in-kernel / LLM-in-scheduler / kernel-mode ML (non-deterministic, hard to certify, expands attack surface).
- QKD integration as a featured capability (treat as key-plane abstraction option, not a marketed differentiator).
- Linux binary compatibility / drop-in replacement (we keep a narrow Linux ABI shim for analyst-toolbox use, but do not promise binary compat).

---

## 2. License (LIC)

### REQ-LIC-001 (P0) — Relicense to Apache-2.0
The repository root `Cargo.toml` `license = "AGPL-3.0-or-later"` shall be changed to `license = "Apache-2.0"`. SPDX headers across the source tree shall be updated to match. A `LICENSE` file containing the full Apache-2.0 text shall replace the current LICENSE (if AGPL-text).

### REQ-LIC-002 (P0) — Contributor License Agreement
A CLA (Apache Software Foundation-style or DCO) shall be established and applied to all new external contributions. Existing contributor history shall be reviewed and contributors contacted for relicensing consent (likely just the project owner + Claude-attributed commits, which the project owner can authorize).

### REQ-LIC-003 (P0) — Third-party dependency license audit
Every crate in `Cargo.toml` shall have a license compatible with Apache-2.0 distribution. The existing memory entry `feedback_license_posture.md` already mandates avoiding GPL/AGPL deps; this requirement formalizes a periodic re-audit via `cargo-license` or `cargo-deny` in CI.

### REQ-LIC-004 (P1) — Trademark / branding strategy
Sphragis (the name) shall be trademarked in the US (USPTO) by the project's corporate entity once incorporated. Trademark protects the brand even when the code is permissively licensed — standard pattern for commercial-OSS hybrids.

---

## 3. Crypto (CRY) — CNSA 2.0 alignment + FIPS 140-3 readiness

### REQ-CRY-001 (P0) — ML-KEM-1024 (FIPS 203) as default key establishment
All kernel-mediated key-establishment paths (TLS, attestation, IPC sealing, SealFS key wrap) shall use ML-KEM-1024 by default. Hybrid (ML-KEM + X25519) is acceptable for transition; pure-classical is forbidden in the `sphragis-gov` build. Configure parameter set explicitly in the `ml-kem` crate invocation.

### REQ-CRY-002 (P0) — ML-DSA-87 (FIPS 204) as default signature
All kernel-mediated signature paths (X.509 verify, code signing, attestation quote signing) shall accept ML-DSA-87 by default. RSA / ECDSA shall be accepted ONLY as legacy-verification for `sphragis-community`; the `sphragis-gov` build shall reject them at the policy layer.

### REQ-CRY-003 (P0) — LMS / XMSS for software signing (NIST SP 800-208)
The kernel image, loadable modules, and update artifacts shall be signable with LMS or XMSS stateful-hash signatures. Add a new `crypto/lms.rs` or `crypto/xmss.rs` module (or vetted crate). This closes the CNSA 2.0 software-firmware-signing requirement.

### REQ-CRY-004 (P0) — AES-256 only (no AES-128) in gov build
All bulk encryption shall use AES-256 (GCM, GCM-SIV, XTS, CTR variants). AES-128 shall be rejected at the policy layer in `sphragis-gov`. Sphragis already uses AES-256 in SealFS (GCM-SIV) and TLS; this requirement formalizes the policy.

### REQ-CRY-005 (P0) — SHA-384 preferred; SHA-256 deprecated in gov build
All hash uses shall default to SHA-384 (or SHA-512 where API requires it). SHA-256 shall be accepted ONLY for legacy compat (e.g., X.509 verify of older intermediates) and shall be deprecated for new-issuance per CNSA 2.0. Add `crypto/sha512.rs` if not present.

### REQ-CRY-006 (P0) — Boot-time crypto self-tests (KAT)
At boot, the kernel shall run Known-Answer-Test (KAT) vectors for every CNSA-2.0 algorithm and **panic** on any failure. KATs shall cover: AES-256-GCM, AES-256-GCM-SIV, AES-256-XTS, SHA-384, SHA-512, ML-KEM-1024, ML-DSA-87, LMS/XMSS, HMAC-SHA-384. This satisfies FIPS 140-3 §7.9 (self-tests) and audit Crypto-F7.

### REQ-CRY-007 (P0) — FIPS 140-3 crypto-module boundary defined
The crypto subsystem shall be physically and logically isolated as a "cryptographic module" per FIPS 140-3 §7.1 (module specification). All cryptographic services routed through a single, documented API surface. SSP (Sensitive Security Parameter) management per §7.8: keys never serialized to non-cryptographic storage without wrap; key destruction via documented zeroization.

### REQ-CRY-008 (P1) — FIPS 140-3 Level 1 lab engagement
By month 12, engage Atsec, Leidos, or InfoGard for FIPS 140-3 Level 1 validation of the crypto module. Budget $150-500K all-in per Phase-1 research §1.3.

### REQ-CRY-009 (P2) — FIPS 140-3 Level 3 hardware-bound key store
By month 36, design and implement a Level-3 hardware-bound key store (HSM/SEP/Caliptra-backed) for operator CA private keys. Requires tamper-detection circuitry interaction. $500K-$1.5M.

### REQ-CRY-010 (P0) — Constant-time discipline for secret-dependent operations
Every secret-dependent code path (HMAC compare, password compare, signature verify, cave-policy check) shall be constant-time. Existing constant-time code in `crypto/hotp.rs` (week 5 fix) sets the pattern. Add `cargo-tests` to assert constant-time properties on review-critical functions.

### REQ-CRY-011 (P1) — RNG: HMAC-DRBG with RNDR seed; fail-closed
HMAC-DRBG per NIST SP 800-90A, seeded from RNDR. Fail-closed on RNDR-not-present (currently fail-soft per FS-H3 audit finding; closing this is required for gov-grade).

---

## 4. Process / Cave Isolation (ISO)

### REQ-ISO-001 (P0) — Capability-based cave model documented as separation-kernel
The cave model shall be formally documented as a separation-kernel pattern: the kernel enforces (a) Data Isolation, (b) Period Processing, (c) Information Flow, (d) Fault Isolation — the four MILS NEAT properties. Document as `DESIGN_SEPARATION_KERNEL.md`.

### REQ-ISO-002 (P0) — Per-cave ASIDs (already done — week 11)
Per-cave ASIDs in TTBR0_EL1 are landed (commit `7d86d273`). REQ formalizes this as a permanent property and adds: defense-in-depth TLBI flush shall be replaced by targeted `tlbi aside1` keyed to the outgoing cave's ASID once a multi-cave cross-read regression test is in place.

### REQ-ISO-003 (P0) — Per-cave information-flow policy
Each cave declares its information-flow class (Bell-LaPadula sensitivity + Biba integrity). The kernel rejects cross-cave reads/writes that violate the policy. Existing CIPSO/CALIPSO labels in `src/net/` provide network-side labeling; extend to IPC and shared memory.

### REQ-ISO-004 (P0) — Cave-policy gate on EVERY cross-cave syscall
The audit identified Cave-H6 (sys_connect doesn't consult cave_policy) — closed in week 3-4. REQ formalizes: every syscall that crosses the cave boundary (network, file, IPC, shm, ptrace, signal) shall consult `cave_policy::check` before proceeding. CI lints any new syscall handler that lacks the check.

### REQ-ISO-005 (P0) — Native-path syscall-source-EL hardening (Cave-H2 from audit)
Per-cave seccomp on the native SVC≠0 path is currently unwired (audit Cave-H2). Mirror the week-1 Linux-ABI fix to close the parallel structural gap.

### REQ-ISO-006 (P1) — `cave::set_active` access control (already done — week 13)
Done in week 13. Formalize: ACTIVE_CAVE_ID mutation is via `cave::enter` / `cave::exit` only; no external module shall hold a mutator path.

### REQ-ISO-007 (P0) — AF_UNIX namespace per cave (already done — week 12)
Done in week 12. Formalize: every IPC namespace (AF_UNIX, shm, futex, eventfd, signalfd) shall scope by `owner_cave`. CI lints any new IPC type that lacks per-cave scoping.

### REQ-ISO-008 (P1) — AF_UNIX SOCK_DGRAM support with per-cave scoping
SOCK_STREAM exists; add SOCK_DGRAM with the same per-cave namespace policy.

### REQ-ISO-009 (P1) — Cave-scoped audit-ring reads
Audit-ring access control is open. Restrict cave's reads to its own entries unless the cave holds an explicit `audit:read-all` capability.

---

## 5. Attestation as Kernel Primitive (ATT)

### REQ-ATT-001 (P0) — Attestation API surface
The kernel shall expose a stable attestation API: `attest::quote(nonce, claims) -> Quote`. Returns a signed quote attesting to: kernel measurement, cave identity, claim set, nonce. Signed with an attestation key derived from a hardware RoT.

### REQ-ATT-002 (P0) — Caliptra-rooted attestation chain
On platforms with Caliptra (open silicon RoT), the attestation key chain shall be: Caliptra ECC/PQ identity → kernel measurement signing key (LMS-derived per CNSA 2.0 software-signing rules) → per-cave attestation key. Caliptra spec: ChipsAlliance/Caliptra. Target Caliptra 2.x.

### REQ-ATT-003 (P0) — Apple SEP attestation (M4 path)
On M4 hardware, attestation roots in the Apple Secure Enclave. Document the boot-chain measurement walk: SEP Boot ROM → Boot Monitor → sepOS → m1n1 → Sphragis kernel measurement. Per Apple Platform Security March 2026 architecture.

### REQ-ATT-004 (P1) — TPM 2.0 attestation (x86_64 path, when added)
On x86_64 hardware (REQ-HW-002), TPM 2.0 with DICE shall be the attestation root. Use stateful-hash signing (LMS) for transition compatibility.

### REQ-ATT-005 (P0) — Per-cave attestable identity
Each cave shall have an attestable identity: a name + a public attestation key + a measurement of the cave's loaded code/config. The kernel signs the cave's identity statement at first-attestation, binds it to the boot-measurement of the kernel.

### REQ-ATT-006 (P0) — HSM-backed operator CA flow
Operator-facing attestation verification shall route through an HSM-backed operator CA (PKCS#11 or REST API to a YubiHSM / AWS CloudHSM / Azure Dedicated HSM). The kernel does not hold the operator-CA private key; only the public root.

### REQ-ATT-007 (P1) — Remote attestation protocol
Define a JSON-LD or CBOR attestation envelope per the Remote Attestation Procedures (RATS) working group (IETF, RFC 9334). Quote includes: claim set, nonce, evidence (kernel measurement + cave measurement), endorsements (Caliptra/SEP/TPM cert chain), signature.

### REQ-ATT-008 (P2) — Confidential VM attestation
On Confidential VM platforms (AMD SEV-SNP, Intel TDX, ARM CCA), the kernel shall participate in CVM attestation — exposing the inner-measurement to the outer CVM verifier. Closes the "OS running as confidential VM" use case.

---

## 6. Audit (AUD)

### REQ-AUD-001 (P0) — HMAC-keyed chained audit log (already done)
HMAC-SHA384 (upgrade from current SHA-256 to align with CNSA 2.0) chained audit log with RNDR-seeded kernel-only HMAC key. Existing implementation closes audit Cave-M1/M2/M3 (week 3-4). REQ adds: upgrade hash to SHA-384.

### REQ-AUD-002 (P0) — WORM audit export to SealFS
Audit ring shall periodically export to a WORM-sealed SealFS volume. Audit Phase-2 FS-H7 (deferred from week 3-4) — reopen and close. Pairs with REQ-CRT-004 below.

### REQ-AUD-003 (P0) — Audit categories cover NIAP GPOSPP FAU_GEN.1
Audit categories shall include at minimum: Authentication, Privilege Escalation, File Access (configurable), Kernel Module Load, Cave Create/Destroy, Crypto Key Use, Network Connect, Update Apply. Maps to NIAP `FAU_GEN.1`.

### REQ-AUD-004 (P0) — Audit integrity verifier
A user-mode tool shall verify the HMAC chain offline given the seal key. Output: which range of entries verify, which (if any) tampering points are detected. Required for AO review during ATO.

### REQ-AUD-005 (P1) — SIGMA-bitmap-style anomaly detection
Existing `ui/sigma_bitmap.rs` (589 LoC) does anomaly detection. Formalize as documented requirement: real-time anomaly scoring on audit events with configurable thresholds.

### REQ-AUD-006 (P0) — Audit-ring access control (Cave-H finding from §4 of inventory)
Cave-scoped reads from the audit ring. Open audit item. Formalize.

---

## 7. Build Chain / Provenance (BLD)

### REQ-BLD-001 (P0) — SLSA Level 4 build provenance
Every binary in the release shall ship with SLSA v1.1 Level 4 provenance: hermetic, reproducible, parameterless build with two-party review of the build configuration. Use GitHub-built-in artifact attestation as the substrate.

### REQ-BLD-002 (P0) — Bit-for-bit reproducible builds
`scripts/check_reproducible_build.sh` shall pass on every release: two independent checkouts on different hosts produce byte-identical artifacts. Pin all toolchain versions; eliminate timestamps from build outputs; sort all globs.

### REQ-BLD-003 (P0) — In-toto attestation chain
`scripts/build_intoto_attestation.py` shall produce a complete attestation chain: source → build → release. Verify chain offline.

### REQ-BLD-004 (P0) — SBOM (CycloneDX or SPDX) per release
Every release ships with an SBOM at SPDX 2.3 or CycloneDX 1.5 minimum. `scripts/gen_sbom.py` + `scripts/generate_sbom.py` already exist; formalize CI integration.

### REQ-BLD-005 (P0) — Sigstore / Rekor transparency-log signing
Every release binary and SBOM shall be signed via sigstore cosign and recorded in the public Rekor transparency log. Use ML-DSA-87 signature where sigstore supports it; otherwise dual-sign with classical + ML-DSA in the in-toto envelope.

### REQ-BLD-006 (P1) — Full-source bootstrap from a documented seed
Following the Guix pattern: Sphragis shall be buildable from a documented bootstrap seed (~tens of MB of cross-compiler binary + audited rebuild script). Independent reproducibility monitoring (à la Lila / NixOS) is a stretch goal.

### REQ-BLD-007 (P0) — `cargo-audit` + `cargo-deny` in CI
Every PR shall run `cargo-audit` (RustSec advisory) and `cargo-deny` (license + ban list). Build fails on advisory or non-Apache-compatible license.

### REQ-BLD-008 (P0) — Signed kernel + signed loadable modules
Kernel image and any loadable module shall be LMS-signed. Bootloader (m1n1 chain on M4; future GRUB / systemd-boot / shim on x86_64) verifies signature before execution. Signature key chain rooted in the operator CA (REQ-ATT-006).

---

## 8. Formal Verification (VER)

### REQ-VER-001 (P0) — Verus or Kani harness for capability dispatcher
Set up Verus (or Kani if Verus is unsuitable for kernel code) verification harness for the cave capability dispatcher. Target: prove **non-interference** — given two caves A and B with no explicit information-flow permission, no syscall from A can cause B's state to differ.

### REQ-VER-002 (P0) — Information-flow proof on IPC subsystem
Prove the analogous non-interference property for AF_UNIX, pipes, and shm: bytes written by cave A in namespace X are unobservable by cave B in namespace Y when no policy rule permits.

### REQ-VER-003 (P1) — Scheduler invariants
Prove: no cave can preempt another in violation of its priority class; no cave can monopolize CPU beyond its quota; no kernel critical section exceeds its bounded-time budget.

### REQ-VER-004 (P1) — Memory-safety regression tests via Kani
Use Kani to model-check critical pointer arithmetic in `caves/linux/mmu.rs`, `kernel/mm/frame.rs`, and SealFS path resolution. Catch the class of bugs that drove audit findings BatCave-F9 (symlink TOCTOU).

### REQ-VER-005 (P0) — Verified subsystem boundary documented
The 5-10K LoC of "verified subsystem" shall be physically isolated in a single Cargo crate/module with documented inputs and outputs. This is the artifact we point to when claiming "verified IFC on critical subsystems."

### REQ-VER-006 (P2) — CompCert-class verified C → translation to Rust verification chain
If sufficient Verus tooling matures, extend the verified boundary to include the bootstrap path. Aspirational; tracks DARPA TRACTOR / V-SPELLS output.

---

## 9. CHERI Readiness (CHR)

### REQ-CHR-001 (P0) — Cave-to-CHERI-compartment mapping document
A design document shall describe how each cave maps to a CHERI compartment on Morello / CHERIoT-capable hardware. Caves' base+bound become CHERI capabilities; cross-cave IPC becomes capability-mediated.

### REQ-CHR-002 (P1) — CHERI-aware build target (`--target morello-unknown-cheribsd`)
Add a build profile for Morello with `--target` set appropriately. Even if it doesn't pass at first, the harness exists.

### REQ-CHR-003 (P1) — CHERIoT-Ibex prototype boot
On a SCI Semiconductor ICENI or lowRISC CHERIoT-Ibex dev kit, boot a minimal Sphragis variant. Functional cave isolation via CHERI capabilities rather than software-enforced MMU isolation.

### REQ-CHR-004 (P2) — FreeBSD 16.0 / Morello pure-capability cave runtime
Track FreeBSD 16.0 (Dec 2027 mainstream CHERI). Port Sphragis cave runtime to pure-capability mode.

---

## 10. UX / "Real OS" Features (UX)

### REQ-UX-001 (P0) — Multi-app concurrent UI with window manager
Today: 8 apps with one-at-a-time switching. Required: concurrent multi-app UI with a window manager (tiling preferred for gov-workflow ergonomics; floating optional). Each app runs in its own cave.

### REQ-UX-002 (P0) — Installer / boot ISO
A bootable installation image (UEFI-bootable ISO; Apple-Silicon variant via m1n1 chainload package). First-boot flow: hardware probe → operator-CA selection → initial-cave creation → unlock-passphrase setup.

### REQ-UX-003 (P0) — Settings / system management app
A unified settings app: networking (interface, firewall, DNS), audit log review, cave management, attestation status, update apply, user accounts.

### REQ-UX-004 (P0) — User accounts beyond lock-screen passphrase
Multi-user model. Each user has an operator-CA-attested identity. Per-user capability set drives which caves they can enter. Single-user "operator" mode for embedded variants stays available.

### REQ-UX-005 (P1) — Package manager
A package-management subsystem: install/update/remove of in-OS apps. Packages are LMS-signed; signature verified by kernel pre-load (REQ-BLD-008). Repository protocol: TUF (The Update Framework) over HTTPS with CNSA-2.0 cipher suites only.

### REQ-UX-006 (P1) — Analyst POSIX toolbox
Port (or vendor pre-built) the analyst toolbox into the `sphragis-gov` build: `vim` / `nano`, `git`, `python3`, `ssh`, `tmux`, `curl`, `jq`, GNU coreutils equivalents. Run in dedicated caves with restricted capabilities by default. Likely via cross-compiled BusyBox + a select set of full binaries.

### REQ-UX-007 (P1) — External display / multi-monitor
HDMI / DisplayPort output on M4 (driver exists per `src/drivers/apple/dcp.rs`). UX surface: window manager spans multiple displays.

### REQ-UX-008 (P2) — Bluetooth / WiFi userspace
M4 has `bcm_wifi.rs` driver-level. Add networking-config UX flow. Bluetooth is P2 — gov SCIF deployments typically disable BT.

### REQ-UX-009 (P1) — Audit-review console
A dedicated app surfacing the audit ring: filter by category, severity, time range; offline-verify chain integrity (REQ-AUD-004 UI).

### REQ-UX-010 (P1) — Cave-management console (extend existing)
`caves_mgr` app exists. Extend with: per-cave attestation status, per-cave information-flow policy editor, per-cave resource quotas, "freeze cave" for forensic capture.

---

## 11. Hardware Targets (HW)

### REQ-HW-001 (P0) — Apple M4 (Mac16,1 / J604 / T8132)
Already verified. Lock in as the demo/development target.

### REQ-HW-002 (P0) — x86_64 reference platform
DoD overwhelmingly deploys on x86_64. Pick a reference: a specific Intel NUC, a Lenovo ThinkPad, or a Dell Latitude commonly used in fed environments. Build a port. Boot via UEFI.

### REQ-HW-003 (P1) — ARM server reference platform
Target one of: Ampere Altra Max, AWS Graviton (via Bare-Metal EC2), or NVIDIA Grace. Gov is increasingly ARM-server-curious; having a story matters.

### REQ-HW-004 (P1) — CHERIoT-Ibex embedded variant
Per REQ-CHR-003. Sphragis-embedded SKU on a CHERIoT-capable RISC-V dev board.

### REQ-HW-005 (P0) — QEMU virt aarch64 (already supported)
Lock in as the CI test target.

### REQ-HW-006 (P1) — QEMU x86_64 for CI parity
Add QEMU x86_64 once REQ-HW-002 lands.

### REQ-HW-007 (P0) — Hardware compatibility list (HCL) publication
Publish a versioned HCL document listing supported hardware, with for each: driver coverage, attestation root availability, certification status.

---

## 12. Documentation (DOC)

### REQ-DOC-001 (P0) — Operator runbook (gov hardening guide)
Document for sysadmins: how to deploy Sphragis in a gov environment, lock down to STIG baseline, integrate with operator CA, enable WORM audit export, configure cave policy. ~50-100 pages target.

### REQ-DOC-002 (P0) — Threat model document
Formal threat model covering: attacker capabilities (network attacker, malicious cave, supply-chain attacker, physical-access attacker, side-channel attacker, microarchitectural attacker), assets (kernel TCB, cave data, audit log, attestation keys), attack surfaces, mitigations. Pair with REQ-DOC-005 (security target).

### REQ-DOC-003 (P0) — Architecture document for gov AOs
30-50 pp architecture document: system overview, TCB boundary, cave model, capability semantics, audit-chain integrity, attestation flow, crypto-module boundary, build-provenance chain. Written for an authorizing-official (AO) audience, not a developer audience.

### REQ-DOC-004 (P0) — Capability statement (8-12 pp)
Gov-procurement-standard capability statement: company overview, core capabilities, NAICS codes, certifications, past performance, points of contact. Updated quarterly.

### REQ-DOC-005 (P0) — Security target (ST) document
Per CC / NIAP convention even if we don't immediately seek CC eval: formal Security Target describing the TOE (Target of Evaluation), TSF (TOE Security Functionality), assumptions, threats, OSPs, security objectives, SFRs, SARs. Standard structure per CC Part 1.

### REQ-DOC-006 (P0) — NIST SP 800-53 Rev 5.2.0 control-inheritance matrix
For each of the 1,196 controls: which we fully satisfy, partially satisfy, do not cover, or require customer-side implementation. Required for FedRAMP-customer adoption.

### REQ-DOC-007 (P1) — DoD STIG (draft) against GP OS SRG
Draft STIG in XCCDF/SCAP format, mapped to each SRG-OS-NNNNNN requirement. Submit to DISA STIG support at month 18-24.

### REQ-DOC-008 (P1) — Whitepaper: "Sphragis Microkernel Architecture" (USENIX-Security-quality)
A peer-review-quality whitepaper describing the architecture, threat model, formal-verification results, and benchmark data. Aim for USENIX Security or NDSS submission as Year-1 credibility marker.

### REQ-DOC-009 (P0) — Public marketing site (sphragis.org or similar)
Single-page-plus marketing site: category claim, 5 differentiators, demo video, downloads, docs, blog, contact.

### REQ-DOC-010 (P1) — Demo deck (gov-buyer audience)
20-slide deck for AFCEA / AFRL / DARPA Forecast meetings. Cover: who we are, the strategic gap we fill, live demo (M4 boot + attestation quote), roadmap, ask.

---

## 13. Certification Deliverables (CRT)

### REQ-CRT-001 (P0) — FIPS 140-3 Level 1 certificate
Per REQ-CRY-008. Target: certificate issued by month 30.

### REQ-CRT-002 (P1) — FIPS 140-3 Level 3 hardware-bound module
Per REQ-CRY-009. Target: certificate issued by month 48-60.

### REQ-CRT-003 (P1) — NIAP PCL listing (against MDF PP v3.3 or GPCP v1.0)
If a feasible PP fit emerges, pursue NIAP evaluation. PP choice TBD pending detailed feature mapping (open Phase-1 question #8).

### REQ-CRT-004 (P0) — DoD STIG submission to DISA
Per REQ-DOC-007. Target: submission by month 24. Acceptance by month 30+.

### REQ-CRT-005 (P0) — FedRAMP Moderate authorization (20x path)
If a sponsoring agency emerges and a cloud-deployment use case exists, pursue FedRAMP Moderate via the 20x path. $500K-$1.5M, 3-6 months once sponsored. Target month 30 if path opens.

### REQ-CRT-006 (P1) — Common Criteria evaluation at NIAP CCTL
Engage CCTL (Atsec / Leidos / Booz Allen) at month 18 for scoping. Pursue eval only if a customer demands it AND a PP fits cleanly. Otherwise defer (the procurement reality is that CC isn't the gate for this product class anymore).

### REQ-CRT-007 (P0) — BIS encryption classification notification filed
Within 90 days of incorporation: file ECCN 5D002 initial classification with BIS (`crypt@bis.doc.gov`) and NSA (`web_site@nsa.gov`). Required for legal distribution of crypto-bearing software.

### REQ-CRT-008 (P2) — EUCC certificate at "High" assurance
For European-allied procurement. Pursue only if EU customer emerges. AVA_VAN.3+.

### REQ-CRT-009 (P1) — NSA CSfC Components List submission
For classified-data deployment relevance. Submit Sphragis (gov build) for CSfC Components List evaluation under the most relevant capability package (Mobile Access, Data-at-Rest, or Multi-Site Connectivity).

---

## 14. Procurement Readiness (PRC)

### REQ-PRC-001 (P0) — Incorporate as US Delaware C-Corp
Standard structure for gov-vendor onramp. Founder + early-employee equity structure. Month 0-3.

### REQ-PRC-002 (P0) — SAM.gov + DSIP registration; CAGE + UEI
Required to receive federal contract awards. ~60 days end-to-end if no issues.

### REQ-PRC-003 (P0) — GSA MAS IT-category offer submission
SINs 511210 (Software Licenses) and 54151S (IT Professional Services). Targets month 9-12 (need 2 years past-performance; achieve via first SBIR Phase I + small commercial contracts in months 0-9).

### REQ-PRC-004 (P0) — Subcontract under ACT 3 IDIQ via teaming with AIS, CNF, Global InfoTek, Invictus, or Radiance
ACT 3 is the most relevant active AFRL cybersecurity IDIQ. Teaming partner gets us onto sub-task-orders without our own prime relationship. Months 6-15.

### REQ-PRC-005 (P0) — IWRP / C5 consortium membership
$10-25K. Gets us access to DoD OTA RFS notices. Required for OTA awards.

### REQ-PRC-006 (P0) — SBIR Phase I parallel-submission strategy
Submit to **all three target programs**: DoD SBIR 26.1, AFWERX open topic, DARPA SBIR. 80-90% rejection rate per submission — three submissions diversify the odds. Months 3-9.

### REQ-PRC-007 (P0) — DARPA program pitches (PROVERS, INSPECTA, RSSC)
Per the user's directive: pitch all three. Attend DARPA Forecast to Industry (annual fall, DC) and request PM meetings.

### REQ-PRC-008 (P1) — In-Q-Tel pitch (after Phase II)
After SBIR Phase II validates technology, pitch In-Q-Tel as "secure compute substrate for IC mission systems" framing. Average check $500K-$3M + intro letter to IC customers.

### REQ-PRC-009 (P1) — Small-business set-aside positioning
Evaluate eligibility for 8(a), WOSB, SDVOSB, HUBZone. Stack where possible. If not eligible, find a certified teaming partner.

### REQ-PRC-010 (P0) — First gov-meeting demo bundle
By month 9: assembled demo bundle for first AFRL / DIU / DARPA meeting:
- Live boot on M4 hardware
- Attestation quote verified against a Caliptra/SEP root
- Audit log walk showing security-relevant event capture
- Threat model + capability statement docs
- 20-slide deck (REQ-DOC-010)

### REQ-PRC-011 (P1) — Conference attendance plan
AFCEA WEST (Feb 2026), AFCEA TechNet Cyber (Jun 2026), AUSA Annual (Oct 2026), DARPA Forecast to Industry (annual fall), DEF CON (Aug), USENIX Security / NDSS submission attempt.

---

## 15. Anti-Features (ANTI) — what we will NOT build

### REQ-ANTI-001 — No full functional-correctness proof of the whole kernel
Cede to seL4. We claim information-flow non-interference on critical subsystems only.

### REQ-ANTI-002 — No AI/LLM/ML in the kernel critical path
Anti-feature for gov. Drop AGENT app entirely from both builds. AI features can ship as caves in the community build, but not in `sphragis-gov`.

### REQ-ANTI-003 — No QKD integration as a featured capability
Maintain a key-plane abstraction that *could* swap in a QKD-derived link key, but don't market it.

### REQ-ANTI-004 — No Linux binary compatibility promise
The narrow Linux ABI shim stays for analyst-toolbox use, but we don't promise to run arbitrary Linux binaries.

### REQ-ANTI-005 — No support for AES-128, SHA-1, MD5, RSA-2048, ECDSA-256, plain ChaCha20-Poly1305 (without CNSA-grade context), DH-2048, etc., in the `sphragis-gov` build
Rejected at policy layer. Available in `sphragis-community` only for legacy interop.

### REQ-ANTI-006 — No closed-source kernel components
Every line of Sphragis kernel + drivers is Apache-2.0 source. Hardware-vendor blobs only at the firmware boundary (M4 SEP firmware is Apple-signed, not our problem; we attest TO it, not on its behalf).

### REQ-ANTI-007 — No GPL/AGPL dependencies
Per existing memory. CI enforces via `cargo-deny`. Apache-2.0 license requires compatibility.

---

## 16. Open Items Carried Forward to Phase 3 / 4

These don't have a clean resolution yet and will be addressed downstream:

1. **CSfC vs CC strategy priority** — Phase 4 sequencing decision once a sponsoring agency emerges.
2. **NIAP PP choice** (MDF v3.3 vs GPCP v1.0 vs no NIAP eval) — depends on feature mapping in Phase 3.
3. **Verified-subsystem scope** (which 5-10K LoC gets Verus proofs first — capability dispatcher vs IPC vs scheduler) — Phase 4 sequencing.
4. **Founding company timing** — coordinate with first SBIR submission month.
5. **Cap-table structure** — beyond scope of tech plan; product of founder discussion + legal counsel.

---

## 17. Counts

| Area | P0 | P1 | P2 | Total |
|---|---|---|---|---|
| Strategic positioning (STRAT) | 3 | 1 | 0 | 4 |
| License (LIC) | 3 | 1 | 0 | 4 |
| Crypto (CRY) | 8 | 2 | 1 | 11 |
| Isolation (ISO) | 6 | 3 | 0 | 9 |
| Attestation (ATT) | 5 | 2 | 1 | 8 |
| Audit (AUD) | 5 | 1 | 0 | 6 |
| Build chain (BLD) | 7 | 1 | 0 | 8 |
| Formal verification (VER) | 3 | 2 | 1 | 6 |
| CHERI (CHR) | 1 | 2 | 1 | 4 |
| UX (UX) | 4 | 5 | 1 | 10 |
| Hardware (HW) | 4 | 3 | 0 | 7 |
| Documentation (DOC) | 7 | 3 | 0 | 10 |
| Certification (CRT) | 4 | 4 | 1 | 9 |
| Procurement (PRC) | 8 | 3 | 0 | 11 |
| Anti-features (ANTI) | 7 | 0 | 0 | 7 |
| **TOTAL** | **75** | **33** | **6** | **114** |

**75 P0 requirements** is a lot. Phase 4 will sequence them across 24-36 months. Phase 3 (gap analysis) will mark each as **have**, **partial**, or **missing** to identify where the existing 14 weeks of work already covers ground.
