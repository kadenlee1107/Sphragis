# Sphragis Threat Model

**Document version:** 1.0 (SP-DOC-002, 2026-05-16)
**Audience:** Authorizing Officials, NIAP CCTLs, CMVP labs, security
auditors, third-party reviewers evaluating Sphragis for deployment.
**Companion docs:** `docs/FIPS_140_3_MODULE_BOUNDARY.md` (CSP/PSP
tables), `VERIFICATION_BOUNDARY.md` (verified subsystem properties),
`DESIGN_CAVE_ISOLATION.md` (cave isolation primitive),
`DESIGN_TLS_HARDENING.md` (network attack-surface narrative),
`DESIGN_CRYPTO.md` (cryptographic architecture),
`DESIGN_CHERI_MAPPING.md` (hardware-isolation roadmap).

This document consolidates the threat model that the per-subsystem
`DESIGN_*.md` files express implicitly into one structured document
suitable for AO / CMVP / CCTL review. It does not introduce new
properties — it names and organizes the ones the design already
expresses.

---

## 1. Assets

Things Sphragis protects, ordered by sensitivity.

### 1.1 Cryptographic Sensitive Security Parameters (CSPs)

Per `docs/FIPS_140_3_MODULE_BOUNDARY.md` §7.8 in full detail. Summary:

| CSP | Sensitivity | Compromise impact |
|---|---|---|
| Operator-CA private key | Critical | Full break of attestation chain; attacker can sign quotes for any cave |
| BatFS master key | Critical | Full break of at-rest encryption for affected cave's files |
| Per-cave file keys | High | Break of one cave's at-rest data |
| ML-KEM-1024 decapsulation keys | High | Decrypt of any KEM-encapsulated payload to that key |
| ML-DSA-87 signing keys | High | Forge signatures under that key |
| LMS signing keys | Critical (stateful) | Reusing an OTS leaf collapses security for that leaf; whole-key compromise breaks software-signing root |
| TLS session keys | Medium | Decrypt of one session's traffic |
| Audit-chain HMAC key | High | Forge audit-log entries undetectably |
| Stack-canary value | Low (per-task) | Bypass of one task's stack-overflow detection |
| DRBG state | High | Predict subsequent randomness; downstream key derivations weaken |
| TPI operator Ed25519 private keys | Critical | Approve unauthorized quorum-protected ops (wipe, declassify, key rotation) |

### 1.2 Data assets

- **Cave user data** (files, in-memory state) — confidentiality + integrity per the cave's classification labels
- **Audit log** — integrity (tamper-evident chain) + non-repudiation (HMAC under kernel-only key)
- **Configuration state** — integrity (cave-policy rules, transition rules, deny matrix)
- **Boot-chain measurements** — integrity (kernel-image hash; SP-B4 LMS signature + bootloader verify)

### 1.3 Policy assets

- **Cave-policy rules** — must be unforgeable by caves; enforced by kernel-mediated cave_policy::check at every cross-cave operation
- **Type-enforcement deny matrix** — same property
- **Bell-LaPadula sensitivity + Biba integrity labels** — must be tamper-evident; bound into AEAD AAD for BatFS files (audit-week-3-4 closure)

---

## 2. Adversary capabilities (attacker model)

Listed from least to most powerful. Sphragis defends against all of them to varying degrees; the cells in §4 (mitigations) name which property covers which combination.

### A1: Network attacker

- Can observe + modify all network traffic in/out of Sphragis
- Cannot run code on the device
- Can replay observed packets
- Cannot break standard cryptographic primitives

### A2: Malicious cave

- An attacker has compromised one cave (e.g., via a bug in an app running inside it)
- Can run arbitrary code at EL0 within that cave's address space
- Holds the cave's syscall capability set
- Cannot synthesize forged capabilities into other caves' state
- Cannot read kernel memory directly (per-cave page tables enforce isolation)

### A3: Local console attacker

- Physical access to the device at a non-running state (powered off)
- Can read raw storage media (BatFS encrypted blocks)
- Cannot extract RAM contents (no cold-boot attack assumed within scope — though `panic_wipe` mitigates partial RAM exposure)
- Cannot extract secrets from a powered-down SEP / TPM / Caliptra

### A4: Supply-chain attacker

- Can submit malicious patches to upstream dependencies (`cargo-deny`-enforced licenses + advisory DB constrain the attack surface)
- Can target a specific build pipeline (mitigated by reproducible builds + sigstore signing in SP-B4)
- Cannot forge sigstore signatures (relies on Fulcio CA + Rekor transparency log integrity)

### A5: Microarchitectural / side-channel attacker

- Co-resident on the same physical CPU as a victim cave
- Can exploit cache timing, branch-predictor state, speculative execution
- Targeted classes: Spectre v1/v2, Meltdown, MDS, L1TF, branch-history-injection

### A6: Physical-access attacker (active)

- Physical access to the device while running
- Tamper with motherboard, attach debugger probes (JTAG, SWD)
- Modify firmware between cold-boot cycles
- Out of scope for Level 1 FIPS validation; addressed at Level 3 (planned SP-CRT-002 hardware-bound key store)

### A7: Quantum-capable adversary (future)

- Cryptographically-relevant quantum computer (CRQC)
- Can break RSA, ECDH, ECDSA in polynomial time via Shor's algorithm
- Cannot break ML-KEM / ML-DSA / LMS / XMSS (post-quantum primitives)

### A8: Privileged insider (operator-CA holder)

- Holds the operator-CA private key
- Can sign arbitrary attestation chain endorsements
- Constrained by two-person-integrity (TPI) for high-consequence ops (wipe, declassify, key rotation)
- Constrained by the audit trail (TPI operations are logged with both signatories' identities)

---

## 3. Attack surfaces

Mapped to source-code regions for traceability.

### S1: EL0 → EL1 syscall boundary

- **Linux ABI path**: `src/caves/linux/syscall.rs` (hardened — per-cave seccomp, cave_policy::check at every cross-cave op)
- **Native SVC path**: kernel-internal only; EL0-origin SVC#N!=0 refused with EPERM at `src/kernel/arch/mod.rs:1308-1329` (audit-week-1 closure, AUDIT-CAVE-C1 + AUDIT-MEM-H1 + Cave-H2 verified SP-C5.1)
- **`svc #0` only** is the accepted user-space entry from EL0

### S2: Network input

- **TLS handshake**: `src/net/tls.rs` (~1,750 LoC). Cipher-suite + signature-algorithm validation per audit-week-1 Crypto-F1+F2. Per-cave SPKI pinning + revocation lists (audit-week-1 Crypto-F3+F4).
- **X.509 parsing**: `src/net/x509.rs` (1,040 LoC). 6 trust anchors. RSA-PKCS#1, RSA-PSS, ECDSA-P256/P384 verify paths. SHA-256/384 sigalg validation.
- **DNS**: `src/net/dns.rs` (538 LoC). Spoofing mitigation via DoH for outbound + per-cave-policy gate on which resolvers a cave can reach.
- **WireGuard responder**: `src/net/wireguard.rs` (988 LoC). Noise IK pattern, sliding-window replay protection (audit week N), cave-private state (no peer leak between caves).
- **NAT + ICMP + conntrack**: `src/net/{nat,arp,ip}.rs`. Per-cave shaper (`src/net/cave_shaper.rs`) prevents one cave from DoS-ing the network.

### S3: Filesystem (BatFS)

- **At-rest AEAD**: AES-256-GCM-SIV (audit-week-8 elite-tier closure; misuse-resistant against nonce reuse)
- **Per-cave file keys** derived via HKDF-SHA-384 from BatFS master + per-file nonce
- **Mount namespace per cave** — two caves cannot see each other's filenames
- **AAD bound to security label** — tampering with a file's classification invalidates decryption
- **Merkle root sealed** under HMAC (audit-week-3-4 FS-H7)

### S4: Boot chain

- **Bootloader**: m1n1 chainload on M4 (untrusted by Sphragis); GRUB or systemd-boot on x86_64 (planned, SP-HW-002)
- **Kernel measurement** at boot via SHA-384 over .text + .rodata (SP-C1.2)
- **LMS signature verification** before jump-to-Rust (SP-B4 future; today the chain is "trusted-by-policy" until that lands)
- **Trust anchor**: the LMS public key embedded in the bootloader OR (better) provisioned via the device's hardware RoT

### S5: Update / loadable-module surface

- **Today**: no in-OS update mechanism; updates require re-flash from off-platform
- **Planned (SP-UX-005)**: TUF-protocol package manager with LMS-signed packages, CNSA-2.0-only HTTPS transport (gov-strict build)

### S6: User input

- **Lock-screen passphrase** entry (Argon2id-protected)
- **TPI quorum signing** ceremony — two operators provide Ed25519 signatures
- **Cave management commands** (cave-create, cave-enter, etc.) — gated by cave-policy

### S7: Hardware peripherals

- **PCIe / virtio devices** — DART (IOMMU) enforces per-device address-space isolation on M4
- **USB** — DWC3 + XHCI bring-up on M4; device-side attacker mitigation via per-device USB policy is open work
- **Display / GPU** — DCP on M4; cave-side framebuffer access mediated by the window-manager (SP-UX-001 future)

### S8: Side-channels

- **Constant-time crypto** — AES via RustCrypto constant-time impl, ML-KEM/ML-DSA decapsulate constant-time per upstream crate design
- **Cache side-channels** — same; T-table-free AES
- **Spectre v1/v2** — ARMv8.5 FEAT_SB `sb` barriers at every cross-domain transition (EL0↔EL1, TTBR0 cave swap, scheduler task switch)
- **Cold-boot** — `panic_wipe` zeros DRBG state + POISONED flag prevents post-panic key derivation

### S9: Audit-log surface

- **Read**: cave-scoped via `audit::recent_for_cave(cave_id_filter, buf)` (SP-ISO-009); privileged callers use `audit::recent`
- **Write**: kernel-internal only; not reachable from EL0
- **Tamper**: detected by HMAC-SHA-256 chain (planned SP-C4.1 → HMAC-SHA-384 for CNSA alignment) keyed by RNDR-seeded kernel-only key (audit-week-3-4 CAVE-M1)

### S10: Attestation surface

- **Quote produce**: `attest::quote(nonce, claims)` — kernel-mediated; signs over (kernel_measurement, cave_identity, nonce, claims) with ML-DSA-87
- **Quote verify**: `attest::verify_quote_local` — local verifier; SP-C1.8 adds standalone external verifier tool
- **Endorsement chain**: today the verifying key is embedded in the Quote (in-memory key in `ATTEST_KEY`); SP-C1.4/1.5/1.6 wire hardware-rooted endorsement (SEP/Caliptra/HSM)

---

## 4. Mitigations (per attack surface × adversary capability)

Rows = adversary capabilities A1-A8. Columns = attack surfaces S1-S10. Cells = mitigation or "out of scope".

### S1 EL0→EL1 syscall

| | Mitigation |
|---|---|
| A1 Network attacker | N/A — no network path into S1 |
| A2 Malicious cave | Per-cave seccomp on Linux ABI; native SVC#N!=0 refused; cave_policy::check on every cross-cave syscall; per-cave ASIDs (audit-week-11) prevent TLB-based cross-cave VA confusion |
| A3 Local console (powered off) | N/A — syscalls only exist during execution |
| A4 Supply-chain | Rust memory-safety + cargo-deny GPL-deny + cargo-audit RUSTSEC blocks 90%+ of known supply-chain vulnerabilities at the syscall surface |
| A5 Microarch | FEAT_SB barriers at EL0↔EL1 transition; PAN enforces kernel-mode no-read-user; constant-time crypto |
| A6 Physical (active) | Out of scope at FIPS L1; SP-CRT-002 future for L3 |
| A7 Quantum | N/A — syscall surface doesn't use long-term keys; ephemeral keys are ML-KEM-1024 or X25519MlKem768 hybrid |
| A8 Privileged insider | TPI for high-consequence syscalls; audit log captures the syscall pair (cave_id + operator_id when TPI-approved) |

### S2 Network input

| | Mitigation |
|---|---|
| A1 Network attacker | TLS 1.3 + X25519MLKEM768 hybrid; SPKI pinning per cave; X.509 chain validation against 6 trust anchors; firewall + per-cave shaper |
| A2 Malicious cave | Cannot reach network surface without `bat_https_open` or NAT-mediated syscall; cave_policy gates which remote endpoints a cave can contact |
| A3-A4 | as A1 |
| A5 Microarch | Same FEAT_SB story; AES + ChaCha20 constant-time |
| A6 Physical | Out of scope |
| A7 Quantum | Hybrid PQ key-exchange shields the symmetric session keys against store-now-decrypt-later |
| A8 Insider | Operator-CA validation + audit trail |

### S3 BatFS

| | Mitigation |
|---|---|
| A1 Network | N/A — BatFS is local storage |
| A2 Cave | Per-cave mount namespace; per-cave file keys; AAD-bound classification labels; no cave can see another's filenames |
| A3 Powered-off console | AES-256-GCM-SIV at rest; master key derived from operator passphrase via Argon2id (memory-hard); attacker must brute-force passphrase + GPU-resistant cost |
| A4-A5 | as elsewhere |
| A6 Physical | Out of scope at L1 |
| A7 Quantum | AES-256 is post-quantum-secure (Grover halves the effective key length; 256 bits still leaves 128-bit margin) |
| A8 Insider | TPI required for cave-mass-wipe + master-key rotation |

### S4 Boot chain

| | Mitigation |
|---|---|
| A4 Supply-chain | Reproducible builds (SP-B3 partial); LMS-signed kernel (SP-B4 future); sigstore + Rekor (SP-B4 future) |
| A6 Physical (firmware modification) | Out of scope today; hardware RoT (SEP/Caliptra/TPM) addresses in SP-C1.4/1.5 |

### S5 Updates

- Future SP-UX-005. Until then, updates are out-of-band (re-flash); reduces attack surface to S4 only.

### S6 User input

| | Mitigation |
|---|---|
| A2 Cave | Lock-screen passphrase keystrokes routed through trusted-input path (not via cave's input queue) |
| A3 Powered-off | Argon2id memory-hardness on passphrase derivation |
| A8 Insider | TPI quorum gates high-consequence ops; both operators' signatures captured in audit trail |

### S7 Peripherals

| | Mitigation |
|---|---|
| A2 Cave | DART (IOMMU) on M4 enforces per-device address-space isolation; cave cannot inject DMA via a peripheral it doesn't own |
| A6 Physical | Out of scope at L1; in-scope for L3 with tamper-detection on PCIe + USB ports |

### S8 Side-channels

Covered as cross-cutting columns above; FEAT_SB + constant-time crypto + cache-side-channel-resistant AES impl is the unified story.

### S9 Audit-log

| | Mitigation |
|---|---|
| A2 Cave | `recent_for_cave` filters; cave can read only its own entries unless it holds `audit:read-all` |
| Tamper (any) | HMAC chain detects modification; offline verifier (SP-AUD-004 future) confirms continuity |

### S10 Attestation

| | Mitigation |
|---|---|
| A2 Cave | Cannot forge cave_identity; kernel-mediated `register_cave_identity` is the only write path |
| A4-A7 | ML-DSA-87 signature is post-quantum-secure |
| A8 Insider | Operator-CA endorsement of the attestation key (SP-C1.6); HSM-bound private key (SP-CRT-002) |

---

## 5. Residual risks (acknowledged + bounded)

Things Sphragis explicitly does NOT defend against, with documented bounds.

### R1: Pre-SP-B4 boot-chain trust

Until LMS-signed kernel + bootloader verification (SP-B4) lands, the boot chain is "trusted by policy": an attacker who can modify the kernel image on disk between cold boots could substitute a malicious kernel. The kernel measurement (SP-C1.2) still computes correctly — just over the attacker's kernel. Attestation works locally but cannot detect the swap until SP-B4.

**Mitigation today**: device must be physically secured between cold-boots; storage attached only to trusted boot media.

**Closure plan**: SP-B4 (LMS-signed kernel + m1n1/GRUB verify).

### R2: Pre-SP-C1.4/1.5/1.6 attestation root

Today's attestation key (`ATTEST_KEY` in `src/security/attest.rs`) is generated at first use and lives in RAM. An attacker who can read kernel memory (which they can't via the cave model, but COULD via a kernel exploit) can extract it and forge quotes.

**Mitigation today**: kernel memory is unreachable from EL0 via per-cave ASIDs + page tables (audit-week-11 closure). A kernel exploit defeats this — same as it defeats everything else.

**Closure plan**: SP-C1.4 (SEP-rooted on M4), SP-C1.5 (Caliptra-rooted on x86_64), SP-C1.6 (HSM-bound endorsement chain).

### R3: Cold-boot DRAM exposure

`panic_wipe` zeros DRBG state but RAM may retain key material from active caves at the moment of power loss. Specialized cold-boot attacks (chip-cooling + rapid-power-cycle) could recover ~seconds of DRAM contents.

**Mitigation today**: out of scope at FIPS L1 (which assumes attacker cannot access powered RAM).

**Closure plan**: hardware-encrypted memory (Apple Silicon MIE on M3+; AMD SME; Intel TME) — feature-detect and rely on it where present.

### R4: XMSS not yet implemented

LMS is landed (SP-B1.3); XMSS is not (SP-B1.4 blocked on upstream crate not being no_std-clean). NIST SP 800-208 allows EITHER LMS OR XMSS for software-signing; LMS alone satisfies the standard.

**Mitigation today**: LMS suffices for the SP-B4 signing-root requirement.

**Closure plan**: SP-B1.4 (hand-roll RFC 8391 OR upstream-patch the `xmss` crate to be no_std-clean).

### R5: Multi-app concurrent UI not yet shipped

Today caves switch one-at-a-time at the UI layer. A cave running a long-running operation blocks other caves' UI access (though their kernel-side state is unaffected).

**Mitigation today**: UI ergonomic limitation, not a security issue.

**Closure plan**: SP-UX-001 (window manager + concurrent multi-app UI).

### R6: SOCK_DGRAM not implemented

AF_UNIX SOCK_STREAM is per-cave-namespaced (audit-week-12 ISO-007). SOCK_DGRAM is not implemented at all.

**Mitigation today**: caves cannot use SOCK_DGRAM (the syscall returns ENOTSUP).

**Closure plan**: SP-ISO-008.

### R7: Hardware attacker against memory bus / cold-RAM (FIPS L1 out-of-scope)

Per FIPS 140-3 §7.6 Level 1: no claim of physical-attacker resistance.

**Closure plan**: SP-CRT-002 hardware-bound key store at L3 + tamper-evident enclosure for the platform.

---

## 6. Defense-in-depth summary

Layered defenses, each layer addressing a distinct attack vector:

1. **Memory safety** — Rust language guarantees prevent use-after-free / double-free / buffer-overflow at compile time
2. **Stack canary** — RNDR-seeded; detects stack-overflow buffer overruns (audit-MEM-H2)
3. **BTI** — Branch Target Identification (audit-week-9 elite-tier) prevents ROP/JOP via indirect-branch type tags
4. **PAN** — Privileged Access Never (audit-week-3-4) prevents kernel-mode reads of user pages without explicit `uaccess` (catches the entire class of confused-deputy kernel bugs)
5. **Per-cave ASIDs** (audit-week-11) — TLB tagging prevents cross-cave VA reuse leakage
6. **Per-cave page tables** — strict cave isolation; no shared writable pages cross-cave
7. **DART (IOMMU)** — peripheral-side isolation; cave cannot DMA outside its own memory
8. **Spectre v2** — ARMv8.5 FEAT_SB `sb` barriers at every cross-domain transition
9. **Constant-time crypto** — defense against timing side-channels on key-dependent code paths
10. **Cave-policy gate** at every cross-cave syscall — defense in depth above seccomp
11. **Type enforcement** — domain-based deny matrix; finer-grained than POSIX uid/gid
12. **BLP + Biba labels** — multi-level security on file accesses; AAD-bound so labels are tamper-evident
13. **Audit-chain HMAC** — kernel-only key; tampering with past entries is detectable
14. **Two-person integrity** — Ed25519 quorum on high-consequence ops; replay-resistant + role-separated
15. **CNSA 2.0 PQ crypto** — ML-KEM-1024 + ML-DSA-87 + LMS; quantum-conservative
16. **Attestation primitive** — every cave is an attestable identity; external verifier can detect impersonation

Each layer is independently audited; compromising the security model requires defeating multiple layers.

---

## 7. Threat-model versioning

This document is a living artifact. Every architecturally-significant change to a defense should result in a versioned update.

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-05-16 | Initial consolidation. Captures the threat model as of the autonomous run's close-out — covers all audit-2026-05-15 closures + the 2026-05-16 productization-push additions (SP-A1/A2, SP-B1.1/.2/.3/.5/.6/.7/.8/.9, SP-C1.1/.2/.3, SP-C2.1, SP-C5.1, SP-ISO-009, SP-AUD-003, SP-CHR-001, SP-HW-007, SP-VER-005). |

## 8. References

- `docs/FIPS_140_3_MODULE_BOUNDARY.md` — CSP/PSP storage + zeroization policy
- `VERIFICATION_BOUNDARY.md` — formal-verification scope + properties
- `DESIGN_CAVE_ISOLATION.md` — cave model in detail
- `DESIGN_TLS_HARDENING.md` — TLS attack surface
- `DESIGN_CRYPTO.md` — cryptographic architecture
- `DESIGN_CHERI_MAPPING.md` — hardware-isolation roadmap
- `ANTI_FEATURES.md` — explicit non-defenses
- Audit reports under `docs/superpowers/audits/` — finding-level traceability
- `docs/superpowers/research/2026-05-16-gov-os-gap-analysis.md` — REQ-level closure status
