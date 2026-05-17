# Sphragis Operator Runbook

**Document version:** 1.0 — STARTER (SP-DOC-001, 2026-05-16)
**Scope:** Operator-facing how-to + day-2 operations for the parts of Sphragis that ship today. SP-UX-005 (package manager), SP-BLD-008.IMPL (LMS-signed kernel pipeline), SP-UX-001 (window manager), SP-UX-004 (multi-user accounts) — each adds an operations chapter to this doc once landed.
**Audience:** sysadmins / SecOps engineers deploying Sphragis. Assumes familiarity with bare-metal OS concepts + cryptographic primitives at the buyer level (not implementer).
**Companion docs:** `docs/THREAT_MODEL.md`, `docs/SECURITY_TARGET.md`, `docs/NIST_800_53_INHERITANCE.md`, `docs/HARDWARE_COMPATIBILITY.md`, `DESIGN_CHERI_MAPPING.md`, `DESIGN_HSM_OPERATOR_CA.md`, `DESIGN_LMS_KERNEL_SIGNING.md`, `DESIGN_SLSA_PROVENANCE.md`, `DESIGN_SIGSTORE_REKOR.md`, `ANTI_FEATURES.md`.

---

## 1. Choosing a build profile

Sphragis ships two build profiles. Choose at deploy time; cannot be switched at runtime.

| Profile | Build command | When to use |
|---|---|---|
| `community` (default) | `cargo build --release --target aarch64-unknown-none --features gicv3` | Development, research, demo, non-gov deployment. All crypto algorithms available; fail-soft on RNG. |
| `sphragis-gov` | `cargo build --release --target aarch64-unknown-none --features gicv3,gov-strict` | Gov / high-assurance deployment. CNSA 2.0 enforced at the policy gate (rejects AES-128, SHA-256-for-signing, RSA, ECDSA, plain ChaCha20-Poly1305). RNG fail-closed at boot if ARMv8.5 FEAT_RNG absent. |

**The gov-strict build's TLS handshakes today fail-closed** because the AES-256-GCM-SHA384 cipher path isn't fully plumbed end-to-end (SP-B1.6.1 follow-up). Until that lands, gov-strict is suitable for offline + air-gapped use, not for direct TLS-based connectivity. Plan a phased deployment: dev on community build, then graduate to gov-strict once your specific TLS-peer or BatFS-only-no-TLS scenario is validated.

## 2. Hardware target selection

See `docs/HARDWARE_COMPATIBILITY.md` for tier + driver coverage.

- **M4 (tier 2)**: real hardware boot via m1n1 chainload. Boot photos in `docs/photos/2026-04-17_first_m4_boot/`. Recommended for the demo + early gov-buyer engagements.
- **QEMU virt aarch64 (tier 1, CI)**: development + CI target. Boot: `qemu-system-aarch64 -machine virt -cpu max -m 2G -kernel target/aarch64-unknown-none/release/sphragis`.
- **x86_64 / ARM server / CHERIoT-Ibex**: not yet supported. See SP-HW-002/003/004 in the master plan.

## 3. First-boot ceremony

When Sphragis boots for the first time on a device, the lock screen presents a passphrase prompt. Until SP-UX-002 (installer) lands, first-boot passphrase initialization happens via a boot-time hook (operator builds a kernel with the initial passphrase hash baked in OR uses the dev-mode default per-build dummy passphrase + immediately rotates).

**Production-grade flow:**

1. Operator builds Sphragis with their initial passphrase hash provisioned (`security::auth::init`) — the build host has Argon2id available.
2. Operator builds m1n1 with the Sphragis kernel embedded + LMS pubkey embedded (once SP-BLD-008.IMPL lands — today the LMS chain is unwired).
3. Operator boots the device + enters the initial passphrase + immediately runs the passphrase-rotation flow (currently shell command — UI surface lands in SP-UX-003).

**Demo flow (M4 + dev only):**

1. Boot to the lock screen.
2. Enter the dev-mode default passphrase (configurable per build).
3. Use any of the 7 in-OS apps via the keyboard cycle (1-7 keys).

## 4. Lock-screen operations

| Operation | How |
|---|---|
| Unlock | Type the passphrase at the lock screen. 5 attempts allowed before lockout. |
| Lock | Press Ctrl+L (or any lock-bound shortcut configured at build). Session is terminated; re-auth required. |
| Emergency wipe | Press Ctrl+W to trigger `wipe::execute(WipeReason::Panic, false)`. On real M4 this halts the SoC; under QEMU it returns normally. Documented as the panic-wipe hotkey per `src/security/mod.rs::check_panic_hotkey`. |
| Duress code | Type the duress code at the lock screen. Treated identically to a normal unlock from a UX perspective but triggers documented countermeasures (audit-log notation; future SP adds richer duress-mode behaviors). |

Audit-trail: every unlock attempt (success, fail, lockout, duress) is recorded under audit category `AuthSession` (post-SP-AUD-003.1-wave-2) or `Auth` (login attempt classification).

## 5. Two-Person Integrity (TPI) ceremony

High-consequence operations require Ed25519 quorum from two pre-registered officers.

**Operations requiring TPI:**
- `audit-wipe` — purge audit ring
- `audit-seal` — establish off-platform anchor of audit chain head
- `mls-declassify` — downgrade a file's MLS sensitivity
- Master-key rotation
- Any future operation that calls `tpi::consume_approval(...)` before proceeding

**Setup (per officer, one-time):**
1. Officer generates an Ed25519 keypair on their personal trusted device.
2. Operator registers the officer's public key via `tpi::register_officer(role, pubkey)`. Role is `AuditOfficer` or `CryptoOfficer`.
3. Officer stores their private key on a hardware key (YubiKey, etc.).

**Ceremony for a privileged operation:**
1. Operator initiates the op (e.g., types `audit-wipe`).
2. Sphragis emits a TPI challenge: a 17-byte canonical-bytes blob containing `(op_id, nonce, timestamp)`.
3. **Both** officers (in any order) sign the challenge with their Ed25519 private key + return the signature.
4. Operator submits the two signatures via `tpi::propose_op` + `tpi::cosign_op`.
5. Sphragis verifies both signatures + the timestamp window (default GRANT_TTL_SECS — operator-configured at build).
6. If both verify within TTL, the op proceeds. Audit log shows both officer IDs.

**Failure modes:**
- One officer offline: the op times out at GRANT_TTL_SECS and is rejected. Operator re-initiates.
- Officer signature replayed (same nonce): rejected by `tpi::consume_approval`.
- Officer key rotated mid-ceremony: ceremony fails; operator re-registers + re-initiates.

Audit-trail: every TPI step (propose, cosign, consume, reject) is recorded under `Auth` + `TpiOp` + `PrivEsc` categories.

## 6. Audit-log operations

### 6.1 Inspect recent events

`audit` shell command — shows the last N events with category, cave_id, timestamp, message. Default N=20.

### 6.2 Flush ring to BatFS

`audit-flush` — persists the resident ring to `/audit.log` in BatFS. Idempotent overwrite. Run before any operation that might evict events (long-running test, kernel panic, planned reboot).

### 6.3 Verify the chain

`audit-chain` shell command — recomputes the HMAC chain from the resident-ring genesis and detects any mismatch. Returns `Ok` if every entry hashes to its stored chain slot, otherwise `FirstMismatchAt(absolute_index)`.

### 6.4 Offline verification (forensic context)

After `audit-flush` exports `/audit.log` to BatFS, copy it out to a forensic workstation and run:

```bash
python3 tools/audit-verifier/audit_verifier.py --summary /path/to/audit.log
```

Structural verification mode is fully working today. Cryptographic mode awaits SP-AUD-004.1 binary-format export.

### 6.5 Seal off-platform

`audit-seal` shell command — emits the current ChainSeal (40 bytes = 8B count || 32B hash). Operator copies this off-platform (paper QR code; TPM PCR; offline log) so future verification can detect head-truncation attacks. TPI-protected.

## 7. Attestation operations

### 7.1 Smoke-test the attestation primitive

`attest-smoke` shell command — generates a fresh ML-DSA-87 keypair, signs a test Quote, verifies locally, runs tamper-detection. Prints PASS or FAIL. Takes ~few seconds under QEMU emulation.

### 7.2 Produce an attestation Quote

(Programmatic — no shell command today; lands in SP-UX-009 audit-review console + SP-UX-010 cave-management console UI.)

The kernel API: `crate::security::attest::quote(nonce, claims) -> Result<Quote, &'static str>`. Caller must have called `register_cave_identity` for the active cave first (today the cave-create path doesn't auto-register — SP-C1.3.1 follow-up).

## 8. Build profile checklist (gov-strict deployment)

Before deploying a gov-strict build to a real device, verify:

- [ ] `cargo build --release --features gicv3,gov-strict` produces a clean build
- [ ] `cargo clippy --release --features gicv3,gov-strict` produces a clean lint
- [ ] `cargo deny check` shows no advisories/bans/license/source violations
- [ ] Hardware target has ARMv8.5 FEAT_RNG (rule out QEMU without `-cpu max` + older silicon)
- [ ] LMS keystore provisioned on an air-gapped signing host (SP-BLD-008.IMPL)
- [ ] m1n1 customized + signed with operator-controlled LMS pubkey embedded (SP-BLD-008.IMPL)
- [ ] Audit-chain HMAC key has been confirmed to seed from RNDR (boot log: `[rng] ARMv8.5 RNDR available — mixing HW entropy`)
- [ ] Initial passphrase provisioned + immediately rotated post-boot
- [ ] TPI officer Ed25519 keypairs generated + public keys registered
- [ ] Operator-CA HSM provisioned per `DESIGN_HSM_OPERATOR_CA.md` (SP-C1.6.IMPL)
- [ ] Endorsement cert issued + copied to device's BatFS at `/attest/endorsement.cbor` (SP-C1.6.IMPL)

Until the SP-X.IMPL items land, the gov-strict deployment is partial — operator documents the gaps + accepts the bounded risk in their ATO package.

## 9. Day-2 operations

### 9.1 Update (rough flow — full version lands SP-UX-005 + SP-BLD-008.IMPL)

Today there is no in-OS update mechanism. Updates require re-flash:

1. Operator builds the new Sphragis kernel + signs it (SP-BLD-008.IMPL flow once landed).
2. Operator distributes the new kernel via their normal channel.
3. On the target device, operator boots into Recovery (M4: hold power, pick macOS Recovery, run `kmutil configure-boot` with the new m1n1 + Sphragis bundle).
4. Reboot.

The Update-applied event is logged under audit category `UpdateApply` once SP-UX-005 lands.

### 9.2 Cave management

`caves` shell command opens the caves_mgr app. Operator can:
- Create cave (within `cave_policy::can_transition` rules)
- Destroy cave (TPI-protected for caves holding sensitive labels)
- Re-enter cave
- Inspect cave's labels (BLP sensitivity + Biba integrity + taint bitmap)
- View per-cave audit subset (post-SP-ISO-009: `audit::recent_for_cave`)

### 9.3 Network configuration

Today CLI-only via shell commands. SP-UX-003 settings app lands the UI surface.

Relevant commands:
- `firewall` — view active per-cave firewall rules
- `tcp-listen <port>` / `tcp-list` — TCP listener management
- `origin` / `origin-allow` / `origin-mode` — per-cave HTTPS origin allowlist
- `pq-interop-test` (if built with `--features pq-interop-test`) — TLS PQ-hybrid handshake against pq.cloudflareresearch.com

### 9.4 Crypto self-test on demand

- `lms-kat` shell command — runs LMS keygen → sign → verify + tamper-detect (~30-60s under QEMU emulation)
- `attest-smoke` shell command — round-trip + tamper-detect for the attestation primitive
- Boot-time KATs run automatically; check the UART log for `[crypto] self-tests PASS`

## 10. Incident response

### 10.1 Suspected audit-log tampering

1. `audit-chain` to detect: returns `FirstMismatchAt(idx)` if any entry's chain hash doesn't match.
2. `audit-flush` to persist the (possibly-tampered) ring to BatFS for forensic capture.
3. Copy `/audit.log` off-platform to a forensic workstation.
4. Run `python3 tools/audit-verifier/audit_verifier.py --summary` to enumerate.
5. Cross-reference against the operator's off-platform seal (audit-seal output) to bound the tampering window.

### 10.2 Crypto KAT failure at boot

A KAT failure halts boot via `panic!("crypto self-test failed: {reason}")`. The UART log shows the specific algorithm + reason. Action:
- DO NOT continue to boot (already prevented by the panic-halt)
- Capture the UART log
- File a security incident: this is either (a) compiler / toolchain regression OR (b) hardware corruption OR (c) supply-chain compromise — all serious

### 10.3 TPI quorum compromised

If a TPI officer's private key is suspected compromised:
1. Immediately rotate the officer's keypair on a clean device.
2. Re-register the new pubkey via `tpi::register_officer`. Old key is no longer valid.
3. Audit log captures both events; the operator's incident-review trail is preserved.
4. Until SP-X future: TPI ops in flight under the compromised key time out at GRANT_TTL_SECS.

### 10.4 Emergency wipe (lost device)

`Ctrl+W` from the lock screen triggers `wipe::execute(WipeReason::Panic, false)`. On real M4 this halts the SoC; cold-boot would re-encrypt the BatFS storage. The wipe path:
- Zeros DRBG state (`rng::panic_wipe`)
- Sets the POISONED flag so post-panic `fill_bytes` halts
- Issues a final UART message
- WFE-loops forever

For "device lost while powered off," the BatFS at-rest encryption (AES-256-GCM-SIV with Argon2id-protected master key) defends against offline data extraction.

## 11. Quick reference card

| Want to ... | Command |
|---|---|
| Unlock | passphrase at lock screen |
| Lock immediately | `Ctrl+L` |
| Emergency wipe | `Ctrl+W` |
| Show audit log | `audit` |
| Persist audit log | `audit-flush` |
| Verify audit chain | `audit-chain` |
| Seal audit chain head | `audit-seal` (TPI-protected) |
| LMS self-test | `lms-kat` |
| Attestation smoke | `attest-smoke` |
| Crypto status | `sec-status` |
| Cave management | `caves` |
| Network config | `firewall`, `tcp-listen`, `origin`, `cookies` |
| Cycle apps | keys `1` through `7` |

## 12. Pending sections (deferred to SP-DOC-001.FULL)

Once the dependent SPs land, these chapters land here:

- **Update workflow** (depends on SP-UX-005 package manager + SP-BLD-008.IMPL signing chain)
- **Multi-user account management** (depends on SP-UX-004)
- **Window-manager UX** (depends on SP-UX-001 multi-app concurrent UI)
- **Settings app walkthrough** (depends on SP-UX-003)
- **HSM endorsement-cert refresh** (depends on SP-C1.6.IMPL)
- **WORM audit-export operations** (depends on SP-AUD-002)
- **Hardware-attestation provisioning** (depends on SP-C1.4 SEP / SP-C1.5 Caliptra)
- **FedRAMP control-mapping deployment guide** (depends on SP-DOC-006.FULL)

## REQ traceability

Closes REQ-DOC-001 STARTER. SP-DOC-001.FULL incorporates the deferred chapters above once their dependent SPs land.

## References

See companion-docs list at the top of this document.
