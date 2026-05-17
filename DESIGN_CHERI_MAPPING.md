# DESIGN: Sphragis Cave Model → CHERI Compartment Mapping

**Document version:** 1.0 (SP-CHR-001, 2026-05-16)
**Status:** Design lock for the architecture; implementation is SP-CHR-002 (Morello target) / SP-CHR-003 (CHERIoT-Ibex variant).
**Strategic differentiator:** #5 — CHERI-ready architecture; caves map 1:1 to CHERI compartments.

## Why this document exists

CHERI (Capability Hardware Enhanced RISC Instructions) and its derivatives — ARM Morello, CHERIoT-Ibex, lowRISC CHERIoT — implement *capability-based memory protection in hardware*. Where Sphragis today enforces cave isolation via MMU page tables (per-cave L1 page tables, per-cave ASIDs landed week 11), CHERI lets the same isolation be enforced by hardware-checked capabilities on every pointer load/store.

The strategic claim from the master plan is "**CHERI-ready architecture** — caves map 1:1 to CHERI compartments." This document spells out what that mapping looks like in practice, so:

1. The SP-CHR-002 / SP-CHR-003 implementation has a written architectural plan to land against, rather than re-deriving it during a code change.
2. A gov buyer reviewing differentiator #5 can read what's planned without waiting for the implementation.
3. Any architectural decision in the cave-isolation code today can be cross-checked against CHERI-compatibility (do we encode an assumption that breaks under capabilities?).

## Quick CHERI primer

A CHERI capability is a 128-bit (or 64-bit on CHERIoT) value that contains:
- A 64-bit base address
- A 64-bit length (or base + top)
- A permissions field (read / write / execute / sealed / etc.)
- A validity bit (tagged in hardware; capabilities can't be forged by pure arithmetic on integer types)
- An object type for sealed capabilities (used for cross-compartment calls)

The CPU enforces that every memory access (load, store, indirect-branch target) goes through a capability whose bounds + permissions cover the access. Out-of-bounds = trap. Wrong permission = trap. Forged (tag-stripped) capability = trap.

A **compartment** is a unit of code that holds capabilities to its own data + a narrow API to the rest of the system. Cross-compartment calls are mediated by sealed capabilities (caller can't forge a callee-domain capability and use it directly; they have to invoke an entry-point routine the callee installed).

## Sphragis cave model recap

Each Sphragis cave owns:
- A per-cave L1 page table (`cave::CAVE_L1[]`, audit-week-1 / ISO-002 closure)
- A per-cave ASID in `TTBR0_EL1` bits 63:48 (audit-week-11, ISO-002 closure)
- A per-cave audit-trail subset (filterable via `audit::recent_for_cave`, SP-ISO-009)
- A per-cave AF_UNIX namespace (audit-week-12, ISO-007 closure)
- A per-cave taint bitmap (`cave::CAVE_TAINT[]`)
- A per-cave information-flow class (sensitivity + integrity labels)
- An attestable identity (SP-C1.3 `StoredCaveIdentity` per-cave registry slot)
- A capability/policy entry indexed by `cave_id`

Caves are slot-allocated: `MAX_CAVES = 32`. Cross-cave operations route through documented APIs (`cave::can_transition`, `cave_policy::check`) that consult per-cave policy at the kernel level.

## The mapping

### 1. Cave ↔ CHERI compartment

Each cave_id (0..MAX_CAVES) corresponds to one CHERI compartment. The compartment's identity is the cave_id.

| Sphragis primitive | CHERI realization |
|---|---|
| Cave's L1 page table | Set of capabilities the compartment holds (one per writable / executable / read-only region) |
| Per-cave ASID | Compartment identity (sealed-capability object type) |
| `cave::enter(target)` | Sealed-capability invocation: caller holds a sealed-call capability to the target compartment's entry point; CPU does the seal-check, unseals temporarily for the call, restores on return |
| `cave_policy::check(src, dst, op)` | Compile- / load-time check that the source compartment actually has a sealed-call capability for the destination's API. If not, no capability exists; the call can't even be expressed |
| Cave isolation property | Hardware-enforced: no compartment can synthesize a capability into another's data without going through the sealed-call mediator |

### 2. Sphragis kernel ↔ CHERI hypercompartment

The Sphragis kernel runs in a privileged compartment that holds *all* capabilities at boot time. It distributes narrow capabilities to caves at cave-create time. Capability monotonicity (CHERI's invariant that you can only *narrow* a capability you hold, never widen) gives us a hardware guarantee that caves can't escalate to kernel privilege.

In CHERIoT terminology this is the "compartment with the universal capability" — equivalent to the EL1 kernel in the MMU-based design.

### 3. Per-cave attestable identity ↔ Sealed compartment-identity capability

SP-C1.3's `StoredCaveIdentity` registry maps cave_id → identity. On CHERI, the cave's compartment-identity capability *is* the identity — sealed with the kernel's identity-sealing key, only the kernel can re-seal or invalidate. The compartment can present its sealed identity capability to the attestation API; the attestation module unwraps and signs over the embedded identity. No forgery possible (CHERI tag check fails on synthesized capabilities).

### 4. IPC and shared memory ↔ Sealed buffer capabilities

| IPC primitive today | CHERI realization |
|---|---|
| `kernel::pipe::write/read` | The pipe is a sealed pair of capabilities (one read-only into the buffer for the reader, one write-only for the writer). Owner-cave-id check at kernel-mediated open; once open, the pipe operates entirely within the compartments via capability invocation |
| `kernel::shm::create/attach` | The shm region is a sealed buffer capability the kernel issues to each attaching compartment with the appropriate (RW or RO) permissions |
| `kernel::unix_sock::*` (AF_UNIX) | The socket is an entry-point sealed capability the listener installs; connect() returns a paired send/recv capability |

This is strictly stronger than the MMU-based isolation today: in the MMU model, an attacker who escapes their cave (via a kernel exploit) gains access to all kernel memory. Under CHERI, even an escaped cave still only holds the capabilities it was issued — kernel memory remains unreachable unless the attacker also forges a kernel-grade capability, which the hardware tag check prevents.

### 5. CNSA-2.0 crypto + attestation under CHERI

The CSP storage from FIPS 140-3 §7.8 (`docs/FIPS_140_3_MODULE_BOUNDARY.md`) maps cleanly:

| CSP storage today | CHERI |
|---|---|
| Kernel-private static (audit-chain HMAC key, DRBG state) | Capability held only by the kernel compartment; not in any cave's set |
| Per-cave heap (ML-KEM keys, ML-DSA keys) | Capability sealed-in to the owning compartment; cave-teardown drops the capability and the key memory is unreachable |
| Operator-CA private key (future, SP-C1.6) | Held by an HSM outside the CHERI boundary; only handled across the sealed-call boundary into a hardened key-handler compartment |

## Target hardware

### ARM Morello (SP-CHR-002)

- **Available:** Morello development boards (university research kits) since 2021. CheriBSD is the reference OS.
- **Sphragis target:** the `morello-unknown-cheribsd` triple. Rust 2024 has experimental tier-3 support via the `riscv64imac-unknown-none-elf`-style infrastructure but the toolchain story is still maturing.
- **Plan:** add a `--target morello-unknown-cheribsd` build profile to Cargo. Initial goal is *compiles*, not boots. Boot brings up the same boot stub structure as the aarch64-unknown-none target but consults capabilities from the firmware boot envelope.
- **Timing:** ARM Morello pure-cap roadmap projects Q1-Q3 2026 for mainstream capability-mode support in FreeBSD 16.0. Sphragis SP-CHR-002 follows that timeline.

### CHERIoT-Ibex (SP-CHR-003)

- **Available 2026:** SCI Semiconductor ICENI MCU + Wyvern WARP-V dev kits; lowRISC CHERIoT-Ibex FPGA bitstreams.
- **Sphragis target:** a stripped-down kernel that fits in CHERIoT's memory budget (~hundreds of KB to single-digit MB). Aimed at embedded gov use (industrial control, defense-comms, automotive cybersecurity gateway).
- **Plan:** add a `riscv32imac-unknown-none-elf` build profile (or equivalent CHERIoT target). Boot stub from CHERIoT lowRISC + Sphragis cave model adapted for the smaller capability format. **No MMU** — CHERI provides isolation; this is structurally simpler than the aarch64 boot path.
- **Timing:** CHERIoT-Ibex hardware available 2026. SP-CHR-003 plan is a *separate-team* play (different procurement angle: embedded gov + automotive Tier 1, not desktop / server).

## What we already do that's CHERI-friendly

- **Cave isolation is the central kernel primitive.** The whole architecture is compartment-shaped already; CHERI is a hardware enforcer for what we already enforce in software.
- **No address arithmetic on integer types in the cross-cave boundary** (no `usize::add` to derive cross-cave pointers). Cross-cave references already route through documented APIs (`cave_policy::check` + the syscall layer).
- **Per-cave allocators** — each cave has its own heap region; no cross-cave heap pointer sharing today.
- **No `transmute` to forge pointer values** — the static-mut hardening (audit `static_mut_refs` lints) keeps the type system honest.

## What we'd have to fix for CHERI

- **Linker scripts** — currently express text/rodata/data/bss as MMU-page-aligned ranges. CHERI wants ranges expressed as capability-bounded segments. Linker rewrites land in SP-CHR-002.
- **`addr_of`/`as_mut_ptr`** call sites currently produce raw `*mut T`. Under CHERI those become "pointer to capability"; the type system handles the bounds-check, but we may have to thread the capability rather than the bare address. SP-CHR-002 sweeps these.
- **Embedded asm that loads MMIO at fixed addresses** — needs a hardware-issued capability to that MMIO range. Driver layer in `src/drivers/` is the touch-point.

## What this document does NOT do

- Wire any code. Pure design. SP-CHR-002 / SP-CHR-003 implement.
- Choose between Morello + CHERIoT-Ibex. Both targets are pursued; they serve different markets (Morello = server / desktop / gov-workstation; CHERIoT = embedded / industrial / automotive).
- Pin a specific Rust toolchain version. Toolchain follows upstream as it stabilizes.

## REQ traceability

Closes REQ-CHR-001 (cave-to-CHERI-compartment mapping doc) to ✅ HAVE.
Unblocks REQ-CHR-002 (CHERI build target), REQ-CHR-003 (CHERIoT-Ibex prototype), REQ-CHR-004 (Morello pure-cap cave runtime) as the design they implement against.

## References

- CHERI ISA v9: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/cheri-isa-v9.html
- CheriBSD: https://www.cheribsd.org/
- CHERIoT (Microsoft + lowRISC): https://cheriot.org/
- ARM Morello: https://www.arm.com/architecture/cpu/morello
- SCI Semiconductor ICENI: https://www.scisemi.com/
- Strategic differentiator #5 in the Sphragis master plan: `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
