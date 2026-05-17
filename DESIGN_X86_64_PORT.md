# DESIGN: Sphragis x86_64 Port

**Document version:** 1.0 (SP-HW-002, 2026-05-16)
**Status:** Design lock; implementation is SP-HW-002.IMPL across multiple sub-phases.
**Companion docs:** `docs/HARDWARE_COMPATIBILITY.md`, `DESIGN_CHERI_MAPPING.md` (parallel ARM Morello track), `DESIGN_LMS_KERNEL_SIGNING.md` (bootloader trust root per platform).
**REQ:** Closes REQ-HW-002 design portion + REQ-HW-006 (QEMU x86_64 CI).

## Why x86_64

The 36-month master plan lists x86_64 as P0 because **DoD overwhelmingly deploys on x86_64**. Without an x86_64 port, the gov-procurement story is M4-only. Most current FedRAMP-authorized cloud environments + most DoD analyst workstations + most Air Force / Army endpoint deployments are x86_64.

This document specifies the architectural plan for the x86_64 target.

## Target hardware

Two reference platforms:

| Platform | Why |
|---|---|
| **Intel NUC 13 Pro** (or equivalent) | Commonly procured in fed environments. UEFI. TPM 2.0. Small form factor for deployable systems. |
| **Lenovo ThinkPad X1 Carbon Gen 11** | Standard analyst-laptop posture. UEFI. TPM 2.0. Common GSA-purchased model. |

Both: x86_64 UEFI + TPM 2.0.

Out of scope:
- BIOS-only (non-UEFI) platforms — modern gov procurement is UEFI
- ARM Windows (Surface Pro X etc.) — covered by SP-HW-001 M4 + SP-HW-003 ARM server, not x86_64
- AMD-specific platforms — not required for gov procurement specifically (Intel is more common); add later if customer demands

## Architectural decisions

### A1: Boot via UEFI + LMS-signed kernel verification

Bootloader: GRUB 2 with custom config that loads Sphragis via the UEFI Boot Services. Two variants:

- **Direct UEFI** (operator owns the firmware): Sphragis-grub2 with embedded LMS pubkey + verification before kernel load
- **Shim path** (Secure Boot enabled): standard Linux Shim with Sphragis kernel + MOK-installed LMS pubkey

LMS verification per `DESIGN_LMS_KERNEL_SIGNING.md` §"x86_64 bootloader trust root."

### A2: TPM 2.0 as attestation root

x86_64 boxes have TPM 2.0 (modern Intel/AMD both ship with discrete or firmware TPMs). Sphragis's `attest` module gains a TPM-backed attestation-key chain:

- TPM PCR0..7 captures the boot chain measurement (replaces the SHA-384(.text||.rodata) in `attest::init_kernel_measurement` for x86_64 — instead, kernel TRUSTS the PCR values + extends PCR8 with its own measurement)
- TPM holds the per-device attestation private key, only released to sign Quote bytes via TPM2_Sign with a policy that requires PCR-state-matches-known-good
- Operator-CA endorsement chain (per `DESIGN_HSM_OPERATOR_CA.md`) chains to TPM-attested per-device pubkey

This is the SP-C1.5 design (TPM 2.0 attestation) that becomes implementable once x86_64 lands.

### A3: ASID equivalent (PCID + Process Context Identifier)

Sphragis uses per-cave ASIDs in TTBR0_EL1 on aarch64 (audit-week-11 closure). x86_64's equivalent is PCID:

- CR3 holds a PCID in bits 11:0 when CR4.PCIDE=1
- 12-bit PCID = 4096 process contexts (vs aarch64's 16-bit / 65536); plenty for MAX_CAVES=32
- INVPCID instruction flushes per-PCID entries (analog to TLBI ASIDE1)
- Per-cave PCID mapping: cave_id → unique PCID at cave-create time

### A4: SMEP + SMAP + UMIP for kernel-mode protection

aarch64 has PAN (audit-week-3-4 closure) which Sphragis uses for kernel-mode no-read-user. x86_64 equivalents:

- **SMEP** (CR4.SMEP=1): supervisor cannot execute user pages — blocks the "user-mapped-shellcode-jumped-to-from-kernel" attack
- **SMAP** (CR4.SMAP=1): supervisor cannot read/write user pages without explicit STAC/CLAC — the PAN analog
- **UMIP** (CR4.UMIP=1): user-mode can't execute SLDT/SIDT/STR/SMSW — blocks fingerprinting

All three enabled at boot in SP-HW-002.IMPL.A.

### A5: CET (Control-flow Enforcement Technology) for ROP/JOP mitigation

aarch64's BTI (audit-week-9 elite-tier closure) prevents indirect-branch redirection. x86_64 equivalent is Intel CET:

- **Shadow Stack** (CET-SS): hardware-enforced return-address checks. Recent CPUs (Tiger Lake+) support.
- **Indirect Branch Tracking** (CET-IBT): every indirect call/jmp target must be an ENDBRANCH instruction — blocks JOP.

Enable via CR4.CET=1 + IA32_S_CET MSR. Compiler emits ENDBRANCH automatically with `-Z cf-protection=full`.

Where CET isn't supported (older CPUs), Sphragis falls back to software CFI (Clang's `-fsanitize=cfi`).

### A6: Spectre v2 + RSB stuffing

aarch64 uses FEAT_SB `sb` barriers at cross-domain transitions. x86_64 uses:

- **IBRS** (Indirect Branch Restricted Speculation): set IA32_SPEC_CTRL.IBRS at every kernel entry
- **STIBP** (Single-Thread Indirect Branch Predictor): isolate predictor state between SMT siblings
- **IBPB** (Indirect Branch Predictor Barrier): emit at cave-switch boundary
- **eIBRS** (Enhanced IBRS): set-once at boot on supported CPUs (Coffee Lake+)
- **RSB stuffing**: emit 32+ RET-like patterns at every cross-domain transition

CPU-dispatch logic: detect CPU family, choose the strongest available barrier.

### A7: Drivers

| Subsystem | x86_64 driver |
|---|---|
| Serial (early UART) | 16550 UART at 0x3F8 (legacy) OR PCH-AHCI mapped serial |
| Interrupt controller | APIC (LAPIC + IOAPIC); x86_64 doesn't have GIC |
| Timer | HPET (event-based) + TSC (read-only for cntpct equivalent) |
| Storage | NVMe (mainline; standard PCIe Express MMIO interface) |
| Network | Intel I210/I225/I226 (mainline); virtio-net (QEMU CI) |
| USB | xHCI (same as M4 DWC3 — different PCI mapping) |
| GPU | UEFI GOP framebuffer (no GPU driver until SP-UX-007 wires modesetting) |
| Keyboard / mouse | USB HID + (early-boot) i8042 PS/2 fallback |
| TPM | TIS (Trusted Computing Group ISA) MMIO interface; standard across vendors |

### A8: SealFS on NVMe

SealFS today uses an in-kernel storage abstraction. x86_64 wires the same abstraction to NVMe (instead of M4 ANS). Per-file AEAD + per-cave keys unchanged.

### A9: Confidential VM compatibility

The x86_64 port should ALSO run inside AMD SEV-SNP, Intel TDX, ARM CCA guests. Sphragis-as-confidential-guest is a high-value FedRAMP scenario (run gov workloads on hyperscaler infrastructure with cryptographic isolation from the cloud provider). This unlocks SP-ATT-008 (CVM attestation).

The x86_64 port needs to:
- Detect CVM environment at boot (CPUID leaf for SEV-SNP / TDX)
- Use CVM-attested kernel measurement instead of TPM where running guest
- Expose the inner-measurement to the outer CVM verifier

## Sphragis-side scope (what changes)

| Module | Changes |
|---|---|
| `src/arch/x86_64/` (NEW) | Boot stub (multiboot2 entry), early-UART, APIC init, paging setup |
| `src/kernel/arch/mod.rs` | Add `#[cfg(target_arch="x86_64")]` branches for exception handling, syscall dispatch, ASID = PCID |
| `src/caves/linux/mmu.rs` | Add x86_64 page-table layout (4-level vs aarch64's 4-level different format); PCID management |
| `src/drivers/x86_64/` (NEW) | UART, APIC, HPET, NVMe, virtio-net, xHCI, TIS-TPM |
| `linker_x86_64.ld` (NEW) | UEFI-loadable layout starting at 0x100000 (1MB); same __text_start/end/__rodata_start/end symbols as aarch64 for SP-C1.2 |
| `src/security/attest.rs` | TPM-backed kernel measurement (replaces SHA-384(.text||.rodata) on x86_64) |
| `src/crypto/rng.rs` | RDRAND/RDSEED-backed RNG path (replaces ARMv8.5 RNDR) |
| `Cargo.toml` | Add `[target.x86_64-unknown-none]` section |
| `scripts/qemu_x86_64_*.py` (NEW) | Mirror the existing aarch64 QEMU CI scripts |

## Implementation phasing

**SP-HW-002.IMPL.A — minimum viable boot (target: kernel reaches shell prompt under QEMU x86_64)**
- Boot stub + paging + early UART
- Stub-out aarch64-only paths with #[cfg]
- Get to `auth::prompt_passphrase` running

**SP-HW-002.IMPL.B — driver layer**
- APIC + HPET + NVMe + virtio-net
- xHCI + USB HID keyboard
- TIS-TPM detection (no use yet)

**SP-HW-002.IMPL.C — security primitives**
- PCID per-cave (replaces aarch64 ASID code paths)
- SMEP + SMAP + UMIP + CET
- IBRS/STIBP/IBPB/RSB-stuffing on cross-domain transitions
- RDSEED-backed RNG

**SP-HW-002.IMPL.D — TPM attestation**
- TPM 2.0 TIS interface
- Kernel measurement via PCR8 extension
- Per-device attestation key via TPM2_Create + TPM2_Sign
- Integrates with `attest::init_kernel_measurement` path

**SP-HW-002.IMPL.E — physical hardware bring-up**
- Intel NUC 13 Pro test
- Lenovo ThinkPad X1 Carbon Gen 11 test
- HCL (docs/HARDWARE_COMPATIBILITY.md) updates per result

**SP-HW-002.IMPL.F — confidential VM scenarios**
- SEV-SNP guest detection + measurement
- TDX guest detection + measurement
- CVM-attested Quote production

## Sphragis-side risks

- **MMU layout divergence**: x86_64 4-level pages have different bit layout than aarch64's. The cave-isolation discipline transfers conceptually but the mmu.rs code is mostly distinct. Mitigation: extract a trait `PageTableArch` that aarch64 and x86_64 implement; cave-isolation code calls the trait.
- **Exception-handling models**: aarch64's exception-vector-table vs x86_64's IDT are very different. Mitigation: `src/kernel/arch/mod.rs` is split into arch-specific files.
- **TPM API complexity**: TPM 2.0 has a large command surface. Mitigation: use only the narrow subset Sphragis needs (Create, Sign, PCR_Extend, Read_Public).
- **CET availability**: not all x86_64 hardware has CET. Mitigation: SW-CFI fallback per A5.

## Open user actions

- **Hardware procurement** for IMPL.E: 1× NUC 13 Pro + 1× ThinkPad X1 Carbon Gen 11. ~$2000.
- **QEMU x86_64 CI runner** for IMPL.A-D: GitHub Actions runner with KVM-enabled instance, or self-hosted runner.
- **CVM testing**: AWS Bare-Metal EC2 i3.metal instance with SEV-SNP enabled; OR Azure DCadsv5 series with TDX.

## REQ traceability

Closes REQ-HW-002 (x86_64 reference platform) design portion. REQ-HW-006 (QEMU x86_64 CI) implemented as part of IMPL.A. Unblocks REQ-ATT-004 (TPM 2.0 attestation) once IMPL.D lands; SP-ATT-008 (CVM attestation) once IMPL.F lands.

## References

- Intel SDM Vol 3 §2.5 (PCID), §4.5 (CET), §11 (APIC)
- Intel SDM Vol 4 §2 (Memory ordering, IBRS/STIBP/IBPB)
- TPM 2.0 spec: https://trustedcomputinggroup.org/resource/tpm-library-specification/
- AMD SEV-SNP: https://www.amd.com/system/files/TechDocs/SEV-SNP-strengthening-vm-isolation-with-integrity-protection-and-more.pdf
- Intel TDX: https://www.intel.com/content/www/us/en/developer/articles/technical/intel-trust-domain-extensions.html
- ARM CCA (companion track, parallel to x86_64): https://developer.arm.com/documentation/den0125/0301/Confidential-Compute-Architecture/Realms
- Sphragis HCL: `docs/HARDWARE_COMPATIBILITY.md`
