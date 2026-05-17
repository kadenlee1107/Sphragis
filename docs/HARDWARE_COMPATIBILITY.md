# Sphragis Hardware Compatibility List

**Document version:** 1.0 (SP-HW-007, 2026-05-16)
**Scope:** Hardware platforms on which Sphragis is known to boot, with driver-coverage, attestation-root, and certification-status notes.

This document is the operator-facing answer to "where can I actually run this?" and a precondition for any FedRAMP / DoD ATO scoping decision.

## Versioning

Each platform row is versioned independently. A row goes from `tier-3` to `tier-2` to `tier-1` as drivers + attestation + certification land. Removing a row requires a project-leadership decision (we don't drop supported platforms silently).

| Tier | Meaning |
|---|---|
| 1 | Production-quality: full driver set, attestation rooted in platform RoT, CI runs against the platform |
| 2 | Functional: boots + core subsystems work; some drivers missing; no platform-rooted attestation yet |
| 3 | Reference: boots reach a known state (UART / framebuffer / shell), substantial driver work remaining |
| n/a | Unsupported / explicitly not pursued |

## Supported platforms (as of 2026-05-16)

### Apple Silicon — Apple M4 MacBook Pro 14" (Mac16,1 / J604 / T8132 "Donan")

- **Tier:** 2
- **Boot path:** m1n1 chainload (Asahi-style) — `chainload.py` with `--skip-secondary-cpus` per private M4_GROUND_TRUTH §1.
- **Verified:** real-hardware boot 2026-04-17 (photos in `docs/photos/2026-04-17_first_m4_boot/`); m1n1-installed-via-kmutil + boot picker pattern.
- **Driver coverage:** ADT (Apple Device Tree) parse, AIC (interrupt controller, M4 base address per M4_GROUND_TRUTH §3), AGX (GPU — discovery + display brought up via DCP), ANE (Neural Engine — discovery only), ANS NVMe (storage — discovery + identify), BCM WiFi (PCIe enum, no firmware load yet), DART (IOMMU), DCP (display coprocessor), DWC3 USB-3 (XHCI bring-up confirmed on real silicon), framebuffer console, RTKit (firmware comm channel), SIO (serial I/O), SMC (system mgmt controller — discovery), SoC tree, SPI, UART (dockchannel), watchdog.
- **Attestation root:** Apple Secure Enclave (SEP). SP-C1.4 wires SEP-rooted attestation key chain — not landed yet; today, attestation runs in `attest::ATTEST_KEY` in-memory mode.
- **MMU policy:** per-cave L1 + per-cave ASIDs (audit-week-11, ISO-002 closure).
- **Limits:** M4/M5 RE is in progress in the wider Asahi community as of 2026; Sphragis got to M4 boot via an independent RE pipeline. Some peripherals (BCM WiFi firmware, BT, biometric Touch ID) are not yet implemented. Display works via DCP path; HDMI-out not yet tested.
- **Certification status:** none. Apple Platform Security is the platform RoT; FIPS 140-3 + DoD STIG land on the Sphragis kernel side, not Apple's.

### QEMU virt aarch64

- **Tier:** 1 (CI target)
- **Boot path:** `qemu-system-aarch64 -machine virt -cpu max -m 2G -kernel target/aarch64-unknown-none/release/sphragis`
- **Verified:** all ~80 self-test scripts in `scripts/qemu_*.py` run against this target on every PR cycle.
- **Driver coverage:** virtio (block, net, keyboard, tablet, GPU, console), UART, PL011 timer, GICv2 + GICv3 (via `gicv3` cargo feature).
- **Attestation root:** none. RNG sourced from `-cpu max` FEAT_RNG (gov-strict build halts if absent — verified via SP-B1.6 + SP-B1.8 boot path).
- **MMU policy:** identical to M4 (same kernel binary).
- **Certification status:** development target only.

## Pursued (in flight)

### x86_64 reference platforms (SP-HW-002, sub-project not yet started)

- **Planned references:** Intel NUC 13 (commonly procured in fed); Lenovo ThinkPad X1 Carbon Gen 11 (analyst laptop posture). Both UEFI.
- **Status:** Tier-n/a today. Boot stub at `src/arch/x86_64/` does not exist. Plan in `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md` §E1.
- **Attestation root:** TPM 2.0 with DICE (SP-ATT-004); Caliptra-compatible API surface for future Caliptra-equipped servers (SP-C1.5).
- **Why pursued:** DoD overwhelmingly deploys on x86_64. Without this port, the gov-procurement story is M4-only.

### ARM server reference (SP-HW-003, deferred)

- **Planned:** Ampere Altra Max OR AWS Graviton (Bare-Metal EC2 ARM — no hardware procurement needed for the second).
- **Status:** Tier-n/a today.
- **Attestation root:** ARM CCA (Confidential Compute Architecture) where available.
- **Why pursued:** gov is increasingly ARM-server-curious; having a story matters for cloud-native deployments.

### CHERIoT-Ibex embedded variant (SP-HW-004, deferred to separate team)

- **Planned:** SCI Semiconductor ICENI MCU OR lowRISC CHERIoT-Ibex FPGA dev kit.
- **Status:** Tier-n/a today. See `DESIGN_CHERI_MAPPING.md` (SP-CHR-001) for the architectural plan.
- **Attestation root:** native CHERIoT identity capability (no separate TPM required — the capability hardware IS the RoT).
- **Why pursued:** embedded gov niche (industrial control, defense-comms, automotive cybersecurity gateway). Different team, different procurement angle (embedded primes + automotive Tier 1, not desktop/server).

## Explicitly not pursued

| Platform | Why |
|---|---|
| Windows host as proxy / dev environment | m1n1's composite USB device doesn't enumerate on Windows without a vendor INF that Apple/Asahi don't publish. Use Linux/macOS instead. |
| RISC-V (non-CHERIoT) server | Not in the master-plan procurement targets. Reconsider if a customer asks. |
| 32-bit ARM | aarch64-only kernel. The cave model assumes 64-bit addresses + 16-bit ASIDs. |

## How to add a new platform

1. Write a port-plan SP doc (see Track E in the master plan for the template).
2. Add `--target <triple>` build profile to `Cargo.toml`.
3. Stand up a boot stub in `src/arch/<triple>/`.
4. Port drivers (or vendor existing ones from `src/drivers/`).
5. Update this document with the platform's tier, driver coverage, attestation root.
6. Add a QEMU-or-physical-hardware CI runner if applicable.
7. Move tier from 3 → 2 → 1 as the implementation matures.

## REQ traceability

Closes REQ-HW-007 (HCL document) to ✅ HAVE.
