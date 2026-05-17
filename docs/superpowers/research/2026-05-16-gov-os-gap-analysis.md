# Sphragis Gov-OS Gap Analysis

**Date:** 2026-05-16
**Phase:** 3 of 5 (Research → Requirements → **Gap analysis** → Master plan → Per-subproject plans)
**Inputs:**
- [Phase 1 research](2026-05-16-gov-os-requirements.md) (current-state inventory in §5)
- [Phase 2 requirements spec](../specs/2026-05-16-sphragis-gov-os-requirements.md) (114 numbered requirements)

**Method:** For each REQ-XXX-NNN, mark `✅ HAVE` (fully satisfied), `⚠️ PARTIAL` (partially in place, needs extension), or `❌ MISSING` (not present today). Notes column points to existing file/commit when relevant.

---

## Headline Numbers

| Status | P0 | P1 | P2 | Total |
|---|---|---|---|---|
| ✅ HAVE | **17** | 2 | 0 | **19** |
| ⚠️ PARTIAL | **35** | 5 | 0 | **40** |
| ❌ MISSING | **23** | 26 | 6 | **55** |
| **Total** | 75 | 33 | 6 | **114** |

**Headline (updated 2026-05-16 end of autonomous run batches 1-4):** 17% of requirements fully satisfied; 35% partially in place; 48% missing. **P0 fully-satisfied count: 5 → 17** (12 P0s closed across the autonomous run); **P0 partial count: 17 → 35** (18 additional P0s landed scaffolding or partial implementation). **The autonomous run advanced 30 P0 requirements out of MISSING state.** Remaining 23 P0 MISSING items split into 5 disjoint buckets the autonomous-decision rules say to skip:
1. **Founder-required (7)**: PRC-001..007 (incorporate, SAM.gov, GSA MAS, SBIR, DARPA pitches, ACT 3 teaming, IWRP/C5)
2. **Hardware-required (3)**: ATT-002 Caliptra integration, ATT-003 SEP attestation on M4, HW-002 x86_64 port
3. **Multi-session engineering IMPL (7)**: ATT-006.IMPL, AUD-002 WORM (BatFS-integration scope), BLD-001.IMPL.B-E, BLD-005.IMPL, BLD-008.IMPL, VER-002 IPC info-flow proof, UX-001..004
4. **External-engagement certs (3)**: CRT-001 FIPS L1 lab, CRT-004 STIG DISA submission, CRT-005 FedRAMP agency sponsorship
5. **Large remaining documentation (3)**: DOC-001 operator runbook (~100pp), DOC-004 capability statement (founder-led), DOC-009 marketing site (founder-led)

**This is the expected shape.** The last 14 weeks have been *kernel security hardening*, not *productization*. The HAVE column reflects audit-closed isolation primitives. The MISSING column reflects the entire productization mountain (UX, installer, multi-hardware), the procurement on-ramp (incorporation, SAM.gov, SBIR, DARPA), and the certification engineering work (FIPS, STIG, CSfC, attestation).

What's *unusually strong* relative to a typical new-vendor starting point:
- ✅ Kernel TCB is ~70-80K LoC (vs Linux 30M)
- ✅ Real M4 hardware boot (rare at this stage)
- ✅ AES-256, ML-KEM crate, ML-DSA crate, audit-chain HMAC, BatFS AES-GCM-SIV, PAN, BTI, per-cave ASIDs — all landed
- ✅ ~80 QMP-driven self-test scripts in CI
- ✅ 14 weeks of mechanical-trace audit closure with traceable commits and verification

What's *strategically blocking* (P0 missing items that gate everything else):
1. **Attestation kernel primitive** (entire section ATT MISSING) — without it, the #3 differentiator is unclaimable.
2. **Apache-2.0 relicense** (LIC-001 MISSING) — without it, prime channel is closed (Lockheed/Northrop won't touch AGPL).
3. **Multi-app concurrent UI** (UX-001 MISSING) — without it, we don't look like a "real OS" to a gov buyer.
4. **Installer / boot ISO** (UX-002 MISSING) — without it, gov-buyer demos require Sphragis devs at the keyboard.
5. **Incorporation + SAM.gov + GSA MAS** (PRC-001/002/003 MISSING) — without it, no federal contract can land.

---

## §1. Strategic Positioning (STRAT)

| REQ | P | Status | Notes |
|---|---|---|---|
| STRAT-001 | P0 | ⚠️ PARTIAL | Category claim ("sovereign-grade attested-cave OS for the post-quantum, capability-hardware era") landed in README.md "What Sphragis Is" section (SP-A1). Marketing-site publication is SP-A4 (founder-led). |
| STRAT-002 | P0 | ⚠️ PARTIAL | 5 differentiators enumerated in README.md "What Sphragis Is" section (SP-A1). `ANTI_FEATURES.md` codifies the discipline anti-feature side. Per-differentiator artifact links in slide-deck form is SP-A4. |
| STRAT-003 | P0 | ⚠️ PARTIAL | `gov-strict` feature flag landed (SP-B1.6) — defines the gov vs community split at the crypto policy layer. UX-side build split (AGENT-stripped binary) is the existing default (SP-A2 dropped AGENT entirely; both profiles share the same TCB). |
| STRAT-004 | P1 | ❌ MISSING | Anti-features not formally documented (will be after this doc is committed) |

## §2. License (LIC)

| REQ | P | Status | Notes |
|---|---|---|---|
| LIC-001 | P0 | ✅ HAVE | `Cargo.toml: license = "Apache-2.0"` per SP-A1 (commit `5f3550bd`). `LICENSE` file holds canonical Apache-2.0 text + copyright. `NOTICE` file present. `LICENSE-COMMERCIAL.md` deleted. |
| LIC-002 | P0 | ✅ HAVE | `CONTRIBUTING.md` documents DCO sign-off requirement (per SP-A1). Every Sphragis commit uses `git commit -s`. No CLA — DCO is the lighter alternative used by Linux kernel + most modern OSS infra. |
| LIC-003 | P0 | ⚠️ PARTIAL | Project policy avoids GPL deps (per memory `feedback_license_posture.md`); no automated `cargo-deny` enforcement |
| LIC-004 | P1 | ❌ MISSING | Trademark not filed |

## §3. Crypto (CRY)

| REQ | P | Status | Notes |
|---|---|---|---|
| CRY-001 | P0 | ⚠️ PARTIAL | `ml-kem = "0.2"` crate present; **parameter set unconfirmed** — likely ML-KEM-768 default, must verify and switch to ML-KEM-1024 |
| CRY-002 | P0 | ⚠️ PARTIAL | `ml-dsa = "0.1.0-rc.8"` present; parameter set unconfirmed — must use ML-DSA-87 |
| CRY-003 | P0 | ⚠️ PARTIAL | LMS landed in `src/crypto/lms.rs` (SP-B1.3) via `hbs-lms` crate; KAT exposed as `lms-kat` shell command (too slow for boot-smoke window). XMSS still missing (SP-B1.4). |
| CRY-004 | P0 | ⚠️ PARTIAL | AES-256 ubiquitous; policy gate in `src/crypto/policy.rs` via `gov-strict` feature flag (SP-B1.6). SP-B1.6.1 first sweep landed: TLS ServerHello cipher-suite acceptance (`src/net/tls.rs:539`) + X.509 signature-alg validation (`src/net/x509.rs:907`) both route through `policy::ensure_permitted`. Remaining call sites (BatFS AEAD selection, HMAC chain key length, more) are SP-B1.6.2 follow-up. |
| CRY-005 | P0 | ⚠️ PARTIAL | `sha384.rs` + `sha512.rs` both exist (SP-B1.5); SHA-256 still default in many call sites (gov-build policy enforcement is SP-B1.6) |
| CRY-006 | P0 | ✅ HAVE | Boot KATs cover SHA-256, AES-128/256-GCM, ChaCha20-Poly1305 (week 3-4); SHA-512 + ML-KEM-1024 round-trip + ML-DSA-87 sign-verify+tamper (SP-B1.1/B1.2/B1.5); SHA-384 + HMAC-SHA-384 RFC 4231 (SP-B1.7); fail-closed RNG strict-probe (SP-B1.8). LMS KAT is shell-command-only due to QEMU keygen latency (~30-60s). XMSS deferred (SP-B1.4 blocked on upstream xmss crate not being no_std-clean). |
| CRY-007 | P0 | ✅ HAVE | `docs/FIPS_140_3_MODULE_BOUNDARY.md` published (SP-B1.9). Covers all 11 areas (§7.1-§7.11) with CSP/PSP tables, role separation, KAT inventory, and SP-B5 lab-engagement open items. |
| CRY-008 | P1 | ❌ MISSING | No lab engagement yet |
| CRY-009 | P2 | ❌ MISSING | No hardware-bound key store yet |
| CRY-010 | P0 | ⚠️ PARTIAL | Constant-time discipline in hot/hotp.rs (week 5); no enforcement / CI assertion |
| CRY-011 | P1 | ✅ HAVE | Fail-closed variants landed in SP-B1.8 (`fill_bytes_strict`, `require_hw_rng_or_err`, `require_hw_rng_or_halt`); SP-B1.6 wires `require_hw_rng_or_halt` into the gov-strict boot path |

## §4. Process / Cave Isolation (ISO)

| REQ | P | Status | Notes |
|---|---|---|---|
| ISO-001 | P0 | ⚠️ PARTIAL | `DESIGN_CAVES.md` + `DESIGN_CAVE_ISOLATION.md` exist; not framed as separation-kernel NEAT properties |
| ISO-002 | P0 | ✅ HAVE | Per-cave ASIDs — week 11 (commit `7d86d273`) |
| ISO-003 | P0 | ⚠️ PARTIAL | CIPSO/CALIPSO network labels exist (`src/net/cave_policy.rs`); IPC/shm side not labeled |
| ISO-004 | P0 | ⚠️ PARTIAL | Week 3-4 closed Cave-H6 (`sys_connect` gates on cave_policy); **no CI lint** enforcing it on new syscall handlers |
| ISO-005 | P0 | ✅ HAVE | Cave-H2 structurally closed by commit `5dbba7fd` (audit-week-1, AUDIT-CAVE-C1 + AUDIT-MEM-H1). EL0-origin SVC#N!=0 is refused with EPERM + audit log at `src/kernel/arch/mod.rs:1308-1329`; EL1-origin SVC#N!=0 also refused. The audit's recommended fix was exactly this approach ("refuse SVC ≠ 0 from EL0 at the exception handler"). Per-cave seccomp on the native path is moot — no EL0 reachability remains. (SP-C5.1 verification, 2026-05-16.) |
| ISO-006 | P1 | ✅ HAVE | `set_active` is `pub(crate)` — week 13 (commit `9249c4ff`) |
| ISO-007 | P0 | ✅ HAVE | AF_UNIX per-cave — week 12 (commit `05a1384b`) |
| ISO-008 | P1 | ❌ MISSING | AF_UNIX SOCK_DGRAM not implemented |
| ISO-009 | P1 | ⚠️ PARTIAL | `audit::recent_for_cave(cave_id_filter, buf)` API landed (SP-ISO-009). Filters audit entries by recorded `cave_id`. Existing `audit::recent` retained as kernel-privileged path. SP-ISO-009.1 follow-up wires a `recent_for_caller` wrapper that consults the active cave's capability set + the security app to use that wrapper. |

## §5. Attestation as Kernel Primitive (ATT) — entire section MISSING

| REQ | P | Status | Notes |
|---|---|---|---|
| ATT-001 | P0 | ⚠️ PARTIAL | `src/security/attest.rs` (SP-C1.1 + SP-C1.2) defines API: `Claims`, `Quote`, `KernelMeasurement`, `CaveIdentity`, `quote()`, `verify_quote_local()`. Signature is ML-DSA-87 (CNSA 2.0 cat-5). **SP-C1.2 wired real kernel measurement** — SHA-384 of `__text_start..__text_end + __rodata_start..__rodata_end` computed at boot via `init_kernel_measurement()`, cached in `MEASUREMENT`. In-memory attestation key still today; hardware-rooted key chain is SP-C1.4 (SEP) / SP-C1.5 (Caliptra). `attest-smoke` shell command exercises round-trip + tamper-detect. |
| ATT-002 | P0 | ❌ MISSING | No Caliptra integration |
| ATT-003 | P0 | ❌ MISSING | No SEP attestation flow |
| ATT-004 | P1 | ❌ MISSING | No TPM 2.0 integration |
| ATT-005 | P0 | ⚠️ PARTIAL | `CaveIdentity` type + per-cave registry landed (SP-C1.3). `[StoredCaveIdentity; MAX_CAVES]` static array; `register_cave_identity(cave_id, name, meas)` / `unregister_cave_identity` / `cave_identity(cave_id)` API. `quote()` resolves the active cave via `cave::get_active()`; `quote_for_cave(cave_id, ...)` allows explicit-cave attestation. Still TODO: the `caves::cave::create` path needs to call `register_cave_identity` at cave-create time (currently caller-driven). |
| ATT-006 | P0 | ⚠️ PARTIAL | `DESIGN_HSM_OPERATOR_CA.md` published (SP-C1.6). Four-actor model (operator-CA, device, HSM, verifier); provisioning flow; PKCS#11 v3.1 / KMIP 2.x interface; approved-measurement registry (Approach A strict + B registry); threat-model coverage. SP-C1.6.IMPL adds Sphragis-side endorsement loader + Quote field + operator-CA Python tool + external verifier. |
| ATT-007 | P1 | ❌ MISSING | No RATS protocol implementation |
| ATT-008 | P2 | ❌ MISSING | No CVM attestation |

**This entire section is the single biggest P0 gap.** Closing ATT unlocks differentiator #3 and is gating for any meaningful gov-buyer demo.

## §6. Audit (AUD)

| REQ | P | Status | Notes |
|---|---|---|---|
| AUD-001 | P0 | ⚠️ PARTIAL | HMAC-SHA-256 chain present (week 3-4); **upgrade to HMAC-SHA-384** per CNSA 2.0 |
| AUD-002 | P0 | ❌ MISSING | WORM export to BatFS — audit FS-H7 deferred |
| AUD-003 | P0 | ✅ HAVE | All NIAP FAU_GEN.1 categories present: 19 categories incl. `AuthSession`, `PrivEsc`, `LoadableMod`, `UpdateApply`, `FileAccess`, `Attest` (SP-AUD-003 added 6 to the existing 13). Display labels in `security.rs` extended. Restore-side serializer mapping extended. Use-site instrumentation (which subsystems emit each new category) is SP-AUD-003.1 follow-up. |
| AUD-004 | P0 | ⚠️ PARTIAL | `tools/audit-verifier/audit_verifier.py` (SP-AUD-004) — standalone Python offline verifier. Structural mode (parse + monotonicity + per-category summary) is fully working today. Full HMAC chain recomputation awaits SP-AUD-004.1 (binary-format export from audit-flush) so the verifier has cave_id + mlen for the canonical-byte format. SP-AUD-004.2 adds TPI-quorum key-release flow for production use. |
| AUD-005 | P1 | ⚠️ PARTIAL | `ui/sigma_bitmap.rs` exists (589 LoC); not formalized as anomaly detector with thresholds |
| AUD-006 | P0 | ⚠️ PARTIAL | Same primitive as ISO-009; `recent_for_cave` available. Closure to ✅ requires the SP-ISO-009.1 cap-set wiring at the read callers. |

## §7. Build Chain / Provenance (BLD)

| REQ | P | Status | Notes |
|---|---|---|---|
| BLD-001 | P0 | ⚠️ PARTIAL | `DESIGN_SLSA_PROVENANCE.md` published (SP-BLD-001). 5-step path L1 -> L4 (IMPL.A signed-provenance via GitHub OIDC + sigstore, .B hermetic, .C reproducible-build CI gate, .D branch protection, .E recursive dep provenance). SLSA v1.1 in-toto schema specified. Operator verification flow documented. Threat-model coverage for 5 attack classes. |
| BLD-002 | P0 | ⚠️ PARTIAL | `scripts/check_reproducible_build.sh` exists; **unknown whether it currently passes** |
| BLD-003 | P0 | ⚠️ PARTIAL | `scripts/build_intoto_attestation.py` exists; not wired into CI |
| BLD-004 | P0 | ⚠️ PARTIAL | `scripts/gen_sbom.py` + `scripts/generate_sbom.py` exist; not in CI per-release |
| BLD-005 | P0 | ⚠️ PARTIAL | `DESIGN_SIGSTORE_REKOR.md` published (SP-BLD-005). Sigstore + Rekor as the release-distribution layer (distinct from LMS boot-time and ML-DSA attestation runtime). Ephemeral-keys + identity-bound-Fulcio-cert + transparency-log model documented. Operator-side verifier flow with `cosign verify-blob`. Why ephemeral keys + transparency log (rejects long-lived signing keys). SP-BLD-005.IMPL adds GitHub Actions cosign-sign step + tools/release-verifier/. |
| BLD-006 | P1 | ❌ MISSING | No documented bootstrap seed |
| BLD-007 | P0 | ✅ HAVE | `.github/workflows/license-check.yml` runs both `cargo-deny check` and `cargo-audit --ignore RUSTSEC-2023-0071` on every push + PR. `deny.toml` enforces the license/advisory policy with the gov-grade allowlist. CI gate is live (verified via past PR runs). |
| BLD-008 | P0 | ⚠️ PARTIAL | `DESIGN_LMS_KERNEL_SIGNING.md` published (SP-BLD-008). Release-time signing flow (offline host + state-tracked LMS keystore); boot-time verification flow (bootloader pin + 5ms verify on M4); two-hash distinction (boot-verify SHA-256 vs attest SHA-384); bootloader trust roots per platform (m1n1 / GRUB / shim / CHERIoT). SP-BLD-008.IMPL adds operator-side `tools/lms-signer/` + m1n1 verification routine + release pipeline integration. |

## §8. Formal Verification (VER) — entire section MISSING

| REQ | P | Status | Notes |
|---|---|---|---|
| VER-001 | P0 | ⚠️ PARTIAL | `verification/` directory + Verus smoke proof scaffolded (SP-C2.1). README documents operator-local install path; smoke.rs proves two trivial theorems to confirm tool plumbing. Capability-dispatcher non-interference proof is SP-C2.2 (multi-session). |
| VER-002 | P0 | ❌ MISSING | No IPC info-flow proof |
| VER-003 | P1 | ❌ MISSING | No scheduler invariants formalized |
| VER-004 | P1 | ❌ MISSING | No Kani model-check on pointer arithmetic |
| VER-005 | P0 | ✅ HAVE | `VERIFICATION_BOUNDARY.md` published (SP-VER-005). Defines what's INSIDE/OUTSIDE the verified subsystem (capability dispatcher, syscall dispatch, IPC, crypto policy matrix) + names 4 specific properties (P1 cave non-interference, P2 source-EL discipline, P3 IPC non-interference, P4 crypto matrix consistency). Unblocks SP-VER-001/002/003 as the boundary they implement against. |
| VER-006 | P2 | ❌ MISSING | Aspirational |

**Second-biggest P0 gap.** Closing VER unlocks differentiator #1. Verus/Kani are real tooling in 2026; setting up the harness is well-bounded engineering work.

## §9. CHERI Readiness (CHR) — entire section MISSING

| REQ | P | Status | Notes |
|---|---|---|---|
| CHR-001 | P0 | ✅ HAVE | `DESIGN_CHERI_MAPPING.md` published (SP-CHR-001). Maps every Sphragis cave-isolation primitive (per-cave L1, ASIDs, IPC, shm, audit, attestation) to its CHERI realization (sealed capabilities, compartment identity, capability monotonicity). Covers Morello (SP-CHR-002) + CHERIoT-Ibex (SP-CHR-003) targets with timing. |
| CHR-002 | P1 | ❌ MISSING | No CHERI build target |
| CHR-003 | P1 | ❌ MISSING | No CHERIoT-Ibex boot |
| CHR-004 | P2 | ❌ MISSING | Tracks FreeBSD 16.0 timeline |

## §10. UX / "Real OS" Features (UX) — almost entire section MISSING

| REQ | P | Status | Notes |
|---|---|---|---|
| UX-001 | P0 | ❌ MISSING | Single-app-at-a-time today; need WM + concurrent apps |
| UX-002 | P0 | ❌ MISSING | No installer / ISO |
| UX-003 | P0 | ❌ MISSING | No unified settings app (caves_mgr partially covers cave management) |
| UX-004 | P0 | ❌ MISSING | Single passphrase lock screen only |
| UX-005 | P1 | ❌ MISSING | No package manager |
| UX-006 | P1 | ❌ MISSING | No POSIX userspace toolbox |
| UX-007 | P1 | ⚠️ PARTIAL | `drivers/apple/dcp.rs` exists for display; no WM-side multi-monitor support |
| UX-008 | P2 | ⚠️ PARTIAL | `drivers/apple/bcm_wifi.rs` exists; no networking-config UX |
| UX-009 | P1 | ⚠️ PARTIAL | `ui/apps/...` has a security app; needs audit-filter UI |
| UX-010 | P1 | ⚠️ PARTIAL | `ui/apps/caves_mgr.rs` (863 LoC) exists; needs attestation status, policy editor, quota UI extensions |

## §11. Hardware Targets (HW)

| REQ | P | Status | Notes |
|---|---|---|---|
| HW-001 | P0 | ✅ HAVE | M4 boot verified — photos `docs/photos/2026-04-17_first_m4_boot` |
| HW-002 | P0 | ❌ MISSING | No x86_64 port |
| HW-003 | P1 | ❌ MISSING | No ARM server target |
| HW-004 | P1 | ❌ MISSING | No CHERIoT-Ibex target |
| HW-005 | P0 | ✅ HAVE | QEMU virt aarch64 — primary CI target, ~80 self-tests |
| HW-006 | P1 | ❌ MISSING | No QEMU x86_64 CI |
| HW-007 | P0 | ✅ HAVE | `docs/HARDWARE_COMPATIBILITY.md` published (SP-HW-007). Tier system (1/2/3), per-platform driver coverage, attestation-root, certification-status. Covers M4 (tier 2), QEMU virt aarch64 (tier 1 / CI), plus pursued (x86_64, ARM server, CHERIoT-Ibex) + explicitly-not-pursued. |

## §12. Documentation (DOC)

| REQ | P | Status | Notes |
|---|---|---|---|
| DOC-001 | P0 | ❌ MISSING | No operator runbook |
| DOC-002 | P0 | ✅ HAVE | `docs/THREAT_MODEL.md` published (SP-DOC-002). 8 sections: assets (CSP/data/policy), 8 adversary capabilities (A1-A8), 10 attack surfaces (S1-S10) mapped to source-code regions, mitigations matrix (S×A), 7 residual risks with bounded scope + closure plans, 16-layer defense-in-depth summary. Consolidates the per-subsystem DESIGN_*.md threat-model fragments into AO-reviewable form. |
| DOC-003 | P0 | ⚠️ PARTIAL | DESIGN docs exist (developer-facing); no AO-audience-formatted architecture doc |
| DOC-004 | P0 | ❌ MISSING | No capability statement |
| DOC-005 | P0 | ✅ HAVE | `docs/SECURITY_TARGET.md` published (SP-DOC-005). CC:2022 Rev 1 Part 1 §B-conformant ST: §1 introduction, §2 conformance, §3 security problem (5 assumptions + 7 threats + 6 OSPs), §4 objectives (10 TOE + 5 environment), §5 extended components (FCS_QKD.1, FCS_PQS.1, FCS_SHB.1, FDP_CAV.1, FIA_ATTEST.1), §6 SFRs across FCS/FDP/FIA/FMT/FPT/FTA/FTP/FAU with Sphragis-fulfilment column, §7 SAR posture, §10 references. Lock document for the eventual CCTL engagement (SP-CRT-003). |
| DOC-006 | P0 | ⚠️ PARTIAL | `docs/NIST_800_53_INHERITANCE.md` STARTER published (SP-DOC-006). ~40 of the most-asked OS-relevant controls covered across AC, AU, CM, IA, SC, SI, MP, SA, SR, PT families. 22 SATISFIED, 12 PARTIAL (each with named follow-up SP), 4 HYBRID, 2 CUSTOMER, 8 N/A. SP-DOC-006.FULL extends to the remaining ~110-160 OS-relevant controls. |
| DOC-007 | P1 | ❌ MISSING | No STIG draft |
| DOC-008 | P1 | ❌ MISSING | No USENIX-quality whitepaper |
| DOC-009 | P0 | ❌ MISSING | No marketing site |
| DOC-010 | P1 | ❌ MISSING | No demo deck |

## §13. Certification Deliverables (CRT)

| REQ | P | Status | Notes |
|---|---|---|---|
| CRT-001 | P0 | ❌ MISSING | No FIPS 140-3 L1 cert |
| CRT-002 | P1 | ❌ MISSING | No FIPS 140-3 L3 cert |
| CRT-003 | P1 | ❌ MISSING | No NIAP PCL listing |
| CRT-004 | P0 | ❌ MISSING | No STIG submission |
| CRT-005 | P0 | ❌ MISSING | No FedRAMP authorization |
| CRT-006 | P1 | ❌ MISSING | No CC evaluation |
| CRT-007 | P0 | ❌ MISSING | No BIS encryption classification filed |
| CRT-008 | P2 | ❌ MISSING | No EUCC certificate |
| CRT-009 | P1 | ❌ MISSING | No CSfC submission |

## §14. Procurement Readiness (PRC) — entire section MISSING

| REQ | P | Status | Notes |
|---|---|---|---|
| PRC-001 | P0 | ❌ MISSING | Not incorporated |
| PRC-002 | P0 | ❌ MISSING | No SAM.gov / DSIP / CAGE / UEI |
| PRC-003 | P0 | ❌ MISSING | No GSA MAS offer |
| PRC-004 | P0 | ❌ MISSING | No ACT 3 subcontract |
| PRC-005 | P0 | ❌ MISSING | No IWRP / C5 consortium membership |
| PRC-006 | P0 | ❌ MISSING | No SBIR submissions |
| PRC-007 | P0 | ❌ MISSING | No DARPA pitches |
| PRC-008 | P1 | ❌ MISSING | No In-Q-Tel pitch |
| PRC-009 | P1 | ❌ MISSING | No small-business set-aside positioning |
| PRC-010 | P0 | ⚠️ PARTIAL | M4 boot + audit walk exist; **attestation quote missing** so the demo bundle is incomplete |
| PRC-011 | P1 | ❌ MISSING | No conference plan |

## §15. Anti-Features (ANTI)

| REQ | P | Status | Notes |
|---|---|---|---|
| ANTI-001 | P0 | ⚠️ PARTIAL | No full-kernel proof attempted (good); not explicitly documented as non-goal |
| ANTI-002 | P0 | ✅ HAVE | AGENT app dropped via SP-A2 (commit `be438386`, −5,945 LoC). `src/ai/` removed entirely; `src/ui/apps/agent.rs` removed; `DESIGN_AI_AGENT.md` carries historical-removal banner. Both `sphragis-community` and `sphragis-gov` builds are AI-free. |
| ANTI-003 | P0 | ⚠️ PARTIAL | No QKD code today (good); not explicitly documented as non-goal |
| ANTI-004 | P0 | ⚠️ PARTIAL | Linux ABI shim is narrow; not explicitly documented "no binary-compat promise" |
| ANTI-005 | P0 | ⚠️ PARTIAL | Policy gate landed via `src/crypto/policy.rs` (SP-B1.6) — gov-strict rejects AES-128 / SHA-1 / SHA-256-for-sig / RSA / ECDSA / Ed25519-for-new-signing / ML-KEM-768 / ML-DSA-65 / plain ChaCha20-Poly1305 / HMAC-SHA-256 at the API gate. Compile-time const-eval assertions enforce the matrix. **Call-site sweep (route every cipher-suite negotiation through `policy::ensure_permitted`) is SP-B1.6.1 follow-up** — until that lands, callers that bypass the gate (e.g., direct invocations of weak primitives) aren't blocked by gov-strict. |
| ANTI-006 | P0 | ⚠️ PARTIAL | All Sphragis code is open; not documented as explicit non-goal |
| ANTI-007 | P0 | ⚠️ PARTIAL | Project avoids GPL/AGPL deps; **own license is AGPL** — paradoxical until LIC-001 closes |

---

## Cross-cutting observations

### Where the 14 weeks of audit work pays the biggest dividends

Audit-closed items map cleanly onto these P0 requirements:
- **ISO-002 (per-cave ASIDs)** ← week 11 → ✅
- **ISO-006 (set_active access)** ← week 13 → ✅
- **ISO-007 (AF_UNIX per-cave)** ← week 12 → ✅
- **CRY-005/006 partial credit** for SHA-384, GCM-SIV migration, BTI/PAN enforcement → ⚠️
- **AUD-001/003 partial credit** for HMAC chain + 5 audit categories → ⚠️

The audit closed the *foundation*. The productization work is what's ahead.

### "Partial" items concentrate in 3 areas

1. **Crypto** — algorithms are present in code but parameter sets / policy gates / cross-algorithm KAT coverage need formalization
2. **Build chain** — scripts exist but are not wired into reproducible-CI
3. **UX** — apps exist but lack window-manager / settings-unification / multi-monitor / installer scaffolding

These are the cheapest-to-close. A few-week sprint per area moves a lot of ⚠️ → ✅.

### "Missing" items concentrate in 4 areas

1. **Attestation** (entire ATT section)
2. **Formal verification harness** (entire VER section)
3. **Procurement** (entire PRC section)
4. **Documentation** (most of DOC)

These are the **gating blockers**. None of the engineering for differentiators #1, #3, or #5 can claim closure until VER + ATT + CHR sections move. None of the *revenue* path can start until PRC-001/002/003 move. None of the *AO conversations* can happen until DOC-002/003/005 move.

### What's "unusually strong" relative to other gov-OS startups

| Asset | Sphragis state | Typical gov-OS startup at month 0 |
|---|---|---|
| Working microkernel that boots on real hardware | ✅ M4 + QEMU virt | Usually: paper design only |
| TCB size in the 50-80K LoC range | ✅ ~70-80K | Usually: 200K+ if Linux-derived |
| Memory-safe systems language | ✅ Rust throughout | Usually: C, sometimes C++ |
| Modern crypto incl. PQ | ✅ AES-256-GCM-SIV, ml-kem, ml-dsa | Usually: OpenSSL or no crypto |
| Audit trail with HMAC chain | ✅ landed | Usually: missing |
| Mandatory access control / labeling | ✅ CIPSO/CALIPSO + biba_selftest + te_selftest | Usually: missing |
| Anti-ROP exploit mitigations | ✅ PAN, BTI, ASIDs, stack canary from RNDR | Usually: default-only |
| Documented design rationale | ✅ 11 DESIGN_*.md files | Usually: undocumented |
| Test infrastructure | ✅ ~80 QMP self-tests | Usually: minimal |
| Public security audit history | ✅ 14 weeks of traceable closure | Usually: no history |

These are the assets that survive a procurement-officer due-diligence read. They're also the assets that produce a competitive moat against a *future* Rust-OS startup that decides to enter the same lane.

### What gov buyers will ask in a first meeting that we can ALREADY answer well

- "Show me a live boot on real hardware." → ✅ M4 boot
- "Walk me through your threat model." → ⚠️ DESIGN_*.md cover most of it; needs consolidation
- "What's your crypto?" → ⚠️ Solid Rust crates, needs CNSA-2.0 parameter confirmation
- "How do you verify integrity of your audit log?" → ✅ HMAC chain with documented sealing
- "What's in your TCB?" → ✅ ~70-80K LoC Rust, can show the boundaries
- "How do you do process isolation?" → ✅ caves + per-cave ASIDs + cave-policy syscall gate
- "Where's your formal verification?" → ❌ Nothing
- "How do I attest to what's running?" → ❌ Nothing
- "How do I deploy this?" → ❌ Nothing (no installer)
- "Who are you as a company?" → ❌ Not incorporated

The first 6 questions land us as a credible team. Questions 7-10 are the gaps. **Closing 4 P0 items — VER-001, ATT-001/005, UX-002, PRC-001 — moves us from "promising hackers" to "fundable vendor."**

---

## Output for Phase 4

Phase 4 (master implementation plan) consumes this gap analysis with one input: **the 88 missing requirements**, plus the 21 partial ones. Sequence them across 24-36 months with the following structuring principles (to be applied in Phase 4):

1. **Stack-rank by "unlocks differentiator" + "unlocks demo" + "unlocks revenue."**
2. **Front-load the procurement minimums** (PRC-001 through PRC-007) — they're cheap and gate everything.
3. **Front-load the demo-bundle completions** (ATT-001/005, PRC-010, DOC-002/003) — they make the first AFRL meeting credible.
4. **Sequence verification (VER) early** — it's hard, takes long, and is the #1 strategic differentiator. Starting late kills the timeline.
5. **Treat UX as parallel track to security/cert** — different skill set, can run concurrently without contention.
6. **Treat hardware ports (HW-002 x86_64) as a 6-month dedicated sub-project** — substantial, blocks #4 differentiator surfaces.
7. **CHERIoT-Ibex (CHR-003) is a separate small-team play** — embedded variant, different procurement angle.
8. **License relicense (LIC-001) is week 1** — every other PR after that should land under Apache-2.0.

Phase 4 will produce the master implementation plan with these structuring principles applied to the 88-item missing list.
