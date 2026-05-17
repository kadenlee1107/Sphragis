# DESIGN: LMS-Signed Kernel Boot Chain

**Document version:** 1.0 (SP-BLD-008, 2026-05-16)
**Status:** Design lock; implementation is SP-BLD-008.IMPL.
**Companion docs:** `src/crypto/lms.rs` (SP-B1.3 — the LMS primitive this design uses), `docs/FIPS_140_3_MODULE_BOUNDARY.md` (§7.4 software/firmware-security area).
**REQ:** Closes REQ-BLD-008 design portion.

## Why this document exists

CNSA 2.0 mandates LMS (or XMSS) for software/firmware signing where the signer wants quantum-conservative signatures relying only on hash-function security. Sphragis ships LMS support today (SP-B1.3 — `src/crypto/lms.rs` via the `hbs-lms` crate). What's missing is the END-TO-END chain that uses it:

1. Kernel image is signed AT RELEASE TIME on a dedicated offline signing host
2. Signature accompanies the kernel image in the release artifact
3. Bootloader VERIFIES the signature BEFORE jumping into Rust
4. Sphragis-as-running-kernel attests to its measurement (already done — SP-C1.2)

Today, the bootloader (m1n1 on M4, future GRUB/shim on x86_64) treats the kernel image as trusted-by-policy. An attacker who can write the storage media between cold boots can substitute their own kernel. This design closes that gap.

## The release-time signing flow

```
Sphragis kernel binary          Offline signing host             LMS signing key
(produced by cargo build)            (air-gapped)                  (state-tracked)
        │                                  │                              │
        │ ─── 1. Operator copies ──────▶ │                              │
        │     kernel binary to            │                              │
        │     signing host via USB        │                              │
        │                                 │                              │
        │                                 │  2. Operator runs:           │
        │                                 │     tools/lms-signer/        │
        │                                 │       sphragis-lms-sign \    │
        │                                 │       --in kernel \          │
        │                                 │       --out kernel.sig \     │
        │                                 │       --key keystore         │
        │                                 │                              │
        │                                 │ ─── 3. Sign call ──────────▶ │
        │                                 │      (consumes one OTS leaf) │
        │                                 │ ◀─── 4. Signature ────────── │
        │                                 │      + updated state         │
        │                                 │                              │
        │ ◀── 5. Operator copies ──────  │                              │
        │     kernel + kernel.sig         │                              │
        │     back, prepares release      │                              │
```

## The boot-time verification flow

```
Bootloader (m1n1 / GRUB / shim)              Kernel image
        │                                            │
        │  1. Load kernel image + kernel.sig          │
        │     from boot media into RAM                │
        │                                            │
        │  2. Compute SHA-256 of kernel image bytes   │
        │     (LMS internal hash, not the SHA-384     │
        │     of SP-C1.2 — that's a separate          │
        │     measurement)                            │
        │                                            │
        │  3. Verify the LMS signature using the      │
        │     pinned LMS public key (embedded in      │
        │     the bootloader at provision time)       │
        │                                            │
        │  4a. Verify FAILS → halt; print error;      │
        │      DO NOT jump to kernel                  │
        │                                            │
        │  4b. Verify PASSES → jump to kernel ─────▶  │
        │                                            5. Kernel boots,
        │                                               runs init_kernel_
        │                                               measurement (SP-C1.2)
        │                                               which computes the
        │                                               SHA-384 of text+rodata
        │                                               for ATTESTATION
        │                                               purposes
```

## Two distinct hashes

| Hash | Algorithm | Purpose | Where computed |
|---|---|---|---|
| **Boot verification hash** | SHA-256 (per LMS_SHA256_M32 parameter set) | LMS signature pre-image; verified by bootloader before jump-to-Rust | Bootloader, every boot |
| **Attestation measurement** | SHA-384 (per SP-C1.2) | Quoted to external verifiers via `attest::quote(...)` | Kernel itself, at `init_kernel_measurement()` |

These cover DIFFERENT TRUST OPERATIONS:

- Boot verification answers: "did the operator sign this exact kernel image?"
- Attestation answers: "what is this running kernel's measurement?"

A bootloader that fails verification doesn't get to attestation at all. A bootloader that passes verification then runs the kernel, which computes the attestation measurement, which feeds into any future Quote.

## Bootloader trust root

The LMS public key must be embedded in the bootloader at provision time. Distribution options:

| Bootloader | LMS pubkey storage | Provisioning ceremony |
|---|---|---|
| **m1n1 (M4)** | Embedded in m1n1 binary at compile time | Operator builds custom m1n1 with their LMS pubkey embedded; signs the m1n1 binary itself via macOS-side `kmutil`. The macOS LocalPolicy chain attests to which m1n1 is approved. |
| **GRUB (x86_64, future)** | Embedded in `grub.cfg` or a separate file in EFI System Partition; or in TPM-sealed storage | Operator uses `grub-install` with a custom config; or stores in TPM and binds via PCR policy. |
| **shim (x86_64 Secure Boot)** | Embedded in the MOK (Machine Owner Key) database | Operator uses MokManager to install the LMS verification pubkey at first-boot. |
| **CHERIoT-Ibex (future)** | Embedded in immutable boot ROM | Manufacturer / operator provisions the boot ROM at silicon programming time. |

For M4 (today's primary target), the LMS pubkey embedding into m1n1 is the simplest path: m1n1 is just a Rust+ASM payload that we already build from source.

## Signing-key state management (the LMS foot-gun)

LMS is stateful — each signature consumes one OTS leaf. The `tools/lms-signer/` operator-side tool must:

1. **Single-writer**: only one signing operation at a time. Lock file in the keystore dir.
2. **Persist-before-use**: write the updated state to disk + fsync BEFORE delivering the signature to the operator. If the operator restarts mid-sign, the state on disk reflects what was used; if the new state hasn't been persisted, the signature isn't delivered.
3. **Backup discipline**: backups must be of a CONSISTENT state. Restoring a stale backup would allow OTS-leaf reuse, which destroys security. Operator runbook documents: "never restore the LMS signing key from backup unless every signature issued since the backup is being re-issued."
4. **State counter monitoring**: keystore tracks how many leaves used. Alert at 50% / 75% / 90% so operator can roll to a new key before exhaustion.

Default parameter set: LMS_SHA256_M32_H10 (1024 signatures per key). Sized for ~yearly release cadence with ~weekly hotfix headroom; operator can pick H15 (32K signatures) at provision time for higher throughput at the cost of larger keys.

## Signature size + boot overhead

| Param | Public key | Signature | Verify time (estimated on M4 @ 4 GHz) |
|---|---|---|---|
| LMS_SHA256_M32_H10 | 60 bytes | ~1.6 KB | ~5 ms |
| LMS_SHA256_M32_H15 | 60 bytes | ~2.3 KB | ~7 ms |
| HSS over LMS (hierarchical, multi-level) | 60 bytes | proportionally larger | proportionally slower |

Boot-time verify is well under 10 ms on real silicon — negligible overhead.

## Module / package signing (SP-BLD-008.MODULE)

Future SP-UX-005 introduces a package manager. Each package = a separate signing operation = consumes a separate OTS leaf. With H10 (1024 signatures) the operator can sign ~3 packages/day for a year before key rotation. For higher throughput, use H15 (32K signatures = years of headroom) or HSS-hierarchical (effectively unlimited within tree-depth budget).

## Threat-model coverage

| Threat | Mitigation |
|---|---|
| Attacker modifies kernel image on boot media | Bootloader LMS-verify fails; kernel never starts. Loud halt with operator-visible UART message. |
| Attacker steals LMS private key from offline signing host | Operator rotates key + revokes old; previously-signed kernels remain valid (the LMS signature is over a hash, not a key-fingerprint). New signing key gets a fresh embedded pubkey in m1n1 — operator re-flashes m1n1 + updates all deployed devices. |
| Attacker reuses an old (valid) signed kernel image to downgrade | Mitigated by rollback-protection: bootloader stores the LAST-VERIFIED version number in TPM/SEP-sealed storage, refuses to load anything older. SP-BLD-008.ROLLBACK adds this. |
| Attacker swaps the LMS pubkey in m1n1 | m1n1 itself is signed by Apple via `kmutil configure-boot`; substitution requires an operator with Apple ID + Recovery-mode access. On x86_64 with Secure Boot, the bootloader chain protects MOK. |
| Quantum attacker | LMS relies only on the underlying hash (SHA-256). Grover's algorithm halves the effective hash security to 128 bits — well above policy requirements. |

## Implementation scope (SP-BLD-008.IMPL)

What SP-BLD-008.IMPL must land:

1. **Operator-side `tools/lms-signer/` Python or Rust tool**: takes a kernel binary + keystore path, produces `kernel.sig`. Wraps `hbs-lms` (Rust) or `python-hash-sigs` (Python; less mature). Enforces single-writer + persist-before-deliver.
2. **m1n1 modification**: embed pinned LMS pubkey; add verification routine; wire into the chainload boot path BEFORE the entry-jump.
3. **Release pipeline integration**: GitHub release workflow runs the signer + uploads `kernel.sig` alongside the kernel binary.
4. **Operator runbook section** in `docs/OPERATOR_RUNBOOK.md` (future SP-DOC-001): provisioning, signing ceremony, key rotation, state-backup discipline, alert response.

What's deliberately NOT in SP-BLD-008.IMPL:

- HSS hierarchical signing — pure LMS is sufficient for kernel + initial package signing volume
- Module / package signing wiring — separate SP-UX-005 (package manager)
- Rollback-protection sealed counter — separate SP-BLD-008.ROLLBACK once TPM/SEP integration lands
- x86_64 GRUB / shim integration — depends on SP-HW-002 (x86_64 port) landing first

## Open user actions

- **Key generation ceremony**: operator must perform LMS keygen on an air-gapped host using `hbs-lms`. Cannot be automated by Sphragis — requires an operator with physical access + cryptographically-safe entropy source.
- **m1n1 customization**: operator builds custom m1n1 with embedded pubkey. Sphragis ships `scripts/m1n1-customize.sh` to streamline (SP-BLD-008.IMPL adds this script).
- **Distribution**: signed kernel + signature delivered to deployed devices via operator's normal update channel (USB stick, TUF-protocol package manager once SP-UX-005 lands, etc.).

## REQ traceability

Closes REQ-BLD-008 (design portion). The IMPL closes the rest.

## References

- RFC 8554 (LMS): https://www.rfc-editor.org/rfc/rfc8554
- NIST SP 800-208: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-208.pdf
- Cisco hash-sigs reference: https://github.com/cisco/hash-sigs
- Fraunhofer hbs-lms (Apache-2.0): https://github.com/Fraunhofer-AISEC/hbs-lms-rust
- Apple `kmutil` boot-policy: https://developer.apple.com/documentation/security/apple-platform-security-guide
