# Sphragis Verified-Subsystem Boundary

**Document version:** 1.0 (SP-VER-005, 2026-05-16)
**Status:** Boundary defined; proof artifacts land in SP-VER-001 (capability dispatcher) and SP-VER-002 (IPC info-flow).
**Strategic differentiator:** #1 — Rust microkernel + information-flow proofs.

## Why this document exists

The master-plan claim is "**information-flow non-interference on critical subsystems** (capability dispatcher, IPC, scheduler invariants) via Verus or Kani." For that claim to be defensible:

1. The set of *verified* code must be explicitly enumerated and physically isolated. A gov verifier or a CMVP lab cannot accept "the IPC subsystem is verified" without a precise list of source files + functions + their stated security properties.
2. Code OUTSIDE the verified boundary cannot break the proofs INSIDE it. The boundary's API surface is the contract.
3. The properties being proven must be named + cited so they map cleanly to the requirements they satisfy.

This document is the boundary. SP-VER-001 / SP-VER-002 are the actual proofs.

## The verified subsystem

| Module | Source path | Verified property | Status |
|---|---|---|---|
| Capability dispatcher | `src/caves/cave.rs` — `enter` / `exit` / `set_active` paths + the cave-policy gate in `cave_policy::check` | **Non-interference**: given two caves A and B with no explicit information-flow permission, the kernel state visible to A is unchanged by any syscall sequence from B | Pending — SP-C2.2 (proof) |
| Syscall dispatch | `src/kernel/syscall.rs` dispatch table + EL0-origin SVC#N!=0 rejection in `src/kernel/arch/mod.rs:1308-1329` | **Source-EL discipline**: every EL0-origin syscall routes through the Linux-ABI path with per-cave seccomp; native SVC#N!=0 from EL0 is refused with EPERM | Pending — SP-C2.2 sub-claim |
| IPC subsystem | `src/kernel/unix_sock.rs`, `src/kernel/pipe.rs`, `src/kernel/shm.rs` | **IPC non-interference**: bytes written by cave A in namespace X are unobservable by cave B in namespace Y when no policy rule permits | Pending — SP-VER-002 |
| Crypto module boundary | `src/crypto/policy.rs` `ensure_permitted` + the per-CSP zeroization paths | **Approved-mode algo allowlist** matches the FIPS 140-3 module boundary doc (`docs/FIPS_140_3_MODULE_BOUNDARY.md`) | Compile-time const-eval assertions in policy.rs land the matrix invariant today (SP-B1.6); deductive proof is future SP |

## The boundary contract

### What's INSIDE the verified boundary

- Functions enumerated in the table above
- The data-types they manipulate (e.g., `cave::Cave`, `cave_policy::Rule`)
- The static state they read/write (`cave::CAVES[]`, `cave::ACTIVE_CAVE_ID`, `cave::CAVE_TAINT[]`)

### What's OUTSIDE the verified boundary

- All driver code (`src/drivers/*`) — too peripheral, too hardware-specific
- All UI code (`src/ui/*`) — caller-side, no verified property
- All network code (`src/net/*`) — separate verification effort (planned `SP-NET-VER` if customer demand)
- The cave-management UI (`caves_mgr`) — caller-side
- All test code (`scripts/qemu_*.py`, `verification/smoke/`)

### Why the split

The verified boundary corresponds to the *TCB-critical* code: the parts where a bug breaks the security model, vs. parts where a bug "merely" causes a denial-of-service or UI glitch. The non-interference property does not care about driver bugs (a driver bug crashes the driver, not the cave model); it cares specifically about cross-cave information leakage. Picking the boundary correctly = focusing the verification effort where it produces gov-grade defensible claims.

## Properties INSIDE the boundary, in plain language

### P1: Cave dispatcher non-interference

Two caves A and B exist. There is no `cave_policy::Rule` permitting A→B or B→A information flow. Then:

  For every initial kernel state s, every syscall sequence σ_A from cave A, and every syscall sequence σ_B from cave B:
    execute(s, σ_A then σ_B) restricted to cave-A-visible state ==
    execute(s, σ_A) restricted to cave-A-visible state

In English: cave B's actions never leak into cave A's observable state. The proof discharges by showing the kernel's syscall handlers project to cave-A-visible state in a B-independent way.

### P2: Source-EL discipline

For every EL0-origin SVC#N!=0 trap that reaches the exception handler:
  The handler returns to EL0 with x0 == -EPERM AND records an audit-log entry of category `Cave` with message "EL0 svc #N!=0 refused".

This is a property of `src/kernel/arch/mod.rs:1308-1329`. Already statically true by inspection (closed in audit-week-1 via commit `5dbba7fd`); SP-VER-001 lifts it to a machine-checked guarantee.

### P3: IPC non-interference

For every IPC primitive (pipe, AF_UNIX socket, shm region):
  Bytes written by cave A in namespace X are unobservable by cave B in namespace Y, unless there exists a `cave_policy::Rule` permitting flow X→Y.

Discharges via: every cross-cave read/write goes through a kernel-mediated function that consults the policy table before completing the transfer.

### P4: Crypto policy matrix consistency

For every `Algo` variant in `policy::is_permitted`:
  Under `gov-strict`, `is_permitted(algo) == true` IFF the algorithm is on the CNSA 2.0 allowlist (the table in `FIPS_140_3_MODULE_BOUNDARY.md` §7.2 marked "✅").

Discharged today by compile-time `const _: () = { assert!(is_permitted(...)); ... };` blocks. SP-VER-005-EXT could lift this to a deductive proof.

## Threat-model assumptions (taken as axioms by the proofs)

- The Rust compiler correctly implements the language semantics on which the proofs rely. (Outside scope of Verus; mitigated by `cargo build` reproducibility + CompCert-class follow-on if customer mandates.)
- The kernel measurement (`docs/FIPS_140_3_MODULE_BOUNDARY.md` §7.4 + SP-C1.2) is computed correctly and the bootloader verified the kernel's signature before jump-to-Rust. This is the trust root the proofs sit on top of.
- The per-cave ASIDs in `TTBR0_EL1` (audit-week-11) provide the hardware enforcement that backs the software-side cave-state separation. Verus reasons about Rust semantics; the hardware-isolation invariant comes from the MMU spec.

## REQ traceability

Closes REQ-VER-005 (verified-subsystem boundary documented) to ✅ HAVE.
Unblocks REQ-VER-001 (capability dispatcher proof) + REQ-VER-002 (IPC info-flow proof) + REQ-VER-003 (scheduler invariants) as the boundary they implement against.

## What this document does NOT do

- Land any proof. Pure boundary definition.
- Commit to one prover. Verus is the leading candidate (per SP-C2.1); Kani available for memory-safety sub-claims (per SP-VER-004).
- Promise a timeline for SP-VER-001/002/003. Those land when the proof effort is funded (likely a DARPA program — PROVERS / INSPECTA / RSSC fit).

## References

- `docs/FIPS_140_3_MODULE_BOUNDARY.md` (SP-B1.9) — the FIPS-side companion boundary
- `DESIGN_CHERI_MAPPING.md` (SP-CHR-001) — the hardware-side companion that strengthens these proofs into hardware guarantees on CHERI targets
- `verification/README.md` (SP-C2.1) — operator install + how-to-run for the Verus harness
- Strategic differentiator #1 in the master plan: `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
