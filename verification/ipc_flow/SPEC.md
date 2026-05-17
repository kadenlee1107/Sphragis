# Sphragis IPC Information-Flow Non-Interference — Verus Proof Specification

**Document version:** 1.0 (SP-VER-002, 2026-05-16)
**Status:** Proof specification; the actual Verus proof is SP-VER-002.IMPL (multi-week effort).
**Companion docs:** `VERIFICATION_BOUNDARY.md` (overall verified subsystem scope), `verification/README.md` (Verus harness install), `DESIGN_CAVE_ISOLATION.md` (the underlying isolation model).
**REQ:** Closes REQ-VER-002 design portion.

## Goal

Prove, by deductive reasoning checked by Verus + Z3, the following property:

> **IPC Non-Interference (P3).** Let A and B be distinct caves whose information-flow rules in `cave_policy::RULES` permit no flow from A to B. Then for every IPC channel C in {pipe, AF_UNIX socket, shm region}, every kernel state σ, and every sequence of operations σ_A by cave A on C: the observable state of B after σ_A is independent of the *content* of σ_A's writes to C. (B may observe metadata — that C exists, that A holds it — but never the bytes A wrote.)

In English: cave A cannot leak data to cave B through ANY IPC primitive, unless the operator explicitly allowed flow A→B via cave-policy.

## Why this property is load-bearing

The cave-isolation discipline already enforces non-interference at the MMU layer (per-cave page tables + per-cave ASIDs). MMU-side isolation is hardware-checked, so a kernel bug in the page-table-setup code is detectable at boot.

But IPC primitives DELIBERATELY cross cave boundaries: pipes, sockets, shm — that's their purpose. So they're the place where non-interference is at risk:
- A pipe writer in cave A sends bytes; a pipe reader in cave B receives them. That's the intended use case ONLY when policy allows.
- A shm region attached by both caves shares memory. Same.
- AF_UNIX socket between A and B: connect+accept exchanges bytes.

A bug in the cave-policy check, a missing check on a new IPC op, a TOCTOU race during cap-check + cap-use — any of these could expose bytes A→B without operator intent. The Verus proof rules out the class.

## What's being proven (mathematically)

Pick a state representation:
```
σ = (caves: [Cave; MAX_CAVES], pipes: [Pipe; PIPE_CAP], sockets: [Socket; SOCKET_CAP], shms: [Shm; SHM_CAP], policy: PolicyTable)
```

For two distinct caves a, b ∈ Cave, let `obs_b(σ)` denote the part of σ observable to cave b — concretely:
- b's own page-table state (already enforced not-to-include-a's-pages by MMU)
- the pipe/socket/shm regions b has open
- the metadata b can query via syscalls (`stat`-like operations)

Theorem we prove:

```
forall a, b: Cave, ops_a: Sequence<IpcOp>, σ: State :
    a ≠ b ∧
    not policy_permits_flow(σ.policy, a, b) ⟹
    obs_b(execute(σ, ops_a as a)) ≡ obs_b(σ)
```

Where ≡ means "byte-equal in every observable field" and `execute(σ, ops_a as a)` applies the sequence of IPC ops as if cave a is the actor.

## Proof strategy

### Step 1: Define the type-state invariants

In Verus syntax (illustrative):

```rust
verus! {

/// Invariant: every IPC channel is owned by exactly one cave.
/// Cross-cave reads/writes go through kernel-mediated transfer functions
/// that consult cave_policy.
spec fn ipc_well_formed(σ: State) -> bool {
    forall|i: int| 0 <= i < σ.pipes.len() ==>
        σ.pipes[i].owner_cave_id < MAX_CAVES &&
        σ.pipes[i].peer_cave_id < MAX_CAVES
    && forall|i: int| 0 <= i < σ.sockets.len() ==>
        σ.sockets[i].owner_cave_id < MAX_CAVES
    && forall|i: int| 0 <= i < σ.shms.len() ==>
        σ.shms[i].owner_cave_id < MAX_CAVES
}

/// Invariant: cave_policy lookup is deterministic + total.
spec fn policy_total(p: PolicyTable, a: Cave, b: Cave) -> bool {
    exists|allowed: bool|
        policy_permits_flow(p, a, b) == allowed
}

}
```

### Step 2: Annotate each IPC op with a refinement spec

For `pipe::write(pipe_id, bytes)`:

```rust
verus! {
fn pipe_write(σ: &mut State, calling_cave: Cave, pipe_id: usize, bytes: Seq<u8>)
    requires
        ipc_well_formed(*old(σ)),
        calling_cave < MAX_CAVES,
        pipe_id < σ.pipes.len(),
    ensures
        // The write only succeeds if the calling cave OWNS the pipe.
        result is Ok ⟹ σ.pipes[pipe_id].owner_cave_id == calling_cave,
        // And only if cave-policy permits flow to the pipe's peer.
        result is Ok ⟹ policy_permits_flow(σ.policy, calling_cave, σ.pipes[pipe_id].peer_cave_id),
        // Failure case leaves σ unchanged.
        result is Err ⟹ *σ == *old(σ),
{
    // Implementation calls cave_policy::check before mutating pipe state...
}

}
```

### Step 3: Prove the top-level theorem by case analysis on IpcOp variants

For each IPC op (PipeWrite, PipeRead, SocketSend, SocketRecv, ShmRead, ShmWrite, etc.), show that:
- If the op is `as a` (cave a is the actor), the only state mutations are to a's own observable regions OR to channels both a and the destination cave own (the latter is the case where policy permits).
- If policy does NOT permit a→b flow, then b's observables are NOT in the mutation set.

The proof reduces to: "every cross-cave mutation goes through `cave_policy::check`, and if `check` returns false, no mutation occurs." Static type-state analysis discharges this.

### Step 4: Compose into the top-level non-interference theorem

```rust
verus! {
proof fn ipc_non_interference(
    σ: State,
    a: Cave,
    b: Cave,
    ops_a: Seq<IpcOp>,
)
    requires
        ipc_well_formed(σ),
        a < MAX_CAVES,
        b < MAX_CAVES,
        a != b,
        !policy_permits_flow(σ.policy, a, b),
    ensures
        obs_b(execute_ops(σ, a, ops_a)) == obs_b(σ),
{
    // By induction on ops_a.len():
    //   Base case (empty): execute is identity; obs_b unchanged.
    //   Inductive step: assume holds for ops_a[..k]; show holds for
    //   ops_a[..k+1].
    //   - Run op k. By Step 3 (per-op spec), the mutation set is
    //     constrained to a's observables + (a-and-X-shared-channels-where-policy-permits).
    //   - Since policy doesn't permit a -> b, b is not in any
    //     permitted-shared-channel set.
    //   - Therefore obs_b is unchanged by op k.
}

}
```

## Out-of-scope (deliberate gaps)

The proof does NOT cover:
- **Timing side channels**: cave A's behavior might affect cache state observable to cave B. Mitigated by SP-VER-005 §"Threat-model assumptions" — covered by FEAT_SB barriers + constant-time crypto, not by this proof.
- **MMU bypass**: the proof assumes per-cave page tables enforce data isolation. If a kernel bug breaks page-table setup, the assumption fails. Covered separately by SP-VER-001 (capability dispatcher proof).
- **Operator misconfiguration**: if the operator sets `policy_permits_flow(σ.policy, a, b) = true`, the proof is trivially satisfied (it's a permitted flow). Policy correctness is operator-side.
- **Physical attacks** (A6 in the threat model): out of scope at FIPS L1.

## Proof execution

```bash
cd verification/ipc_flow/
$VERUS SPEC.rs   # the actual Rust+Verus file landing in SP-VER-002.IMPL
# Expected output:
#   verification results:: <N> verified, 0 errors
```

The proof MUST be re-run in CI on every PR that touches:
- `src/caves/cave.rs`
- `src/caves/linux/*`
- `src/kernel/pipe.rs`, `src/kernel/unix_sock.rs`, `src/kernel/shm.rs`
- The cave-policy table

A PR that breaks the proof FAILS CI; the PR author must either fix the implementation or update the spec (the latter is a major decision; require maintainer approval).

## Implementation phasing

**SP-VER-002.IMPL.A** (~2 weeks): Type-state annotations for pipe ops. Prove the pipe-only sub-theorem. Wire into verification/ipc_flow/ as a runnable proof.

**SP-VER-002.IMPL.B** (~2 weeks): Same for AF_UNIX socket ops.

**SP-VER-002.IMPL.C** (~2 weeks): Same for shm ops.

**SP-VER-002.IMPL.D** (~1 week): Compose into the top-level non-interference theorem.

**SP-VER-002.IMPL.E** (~1 week): CI integration — verus-runner.yml workflow.

Total: ~8 weeks of focused proof-engineering work. Could be funded by DARPA PROVERS / INSPECTA / RSSC programs.

## Why this matters strategically

Strategic differentiator #1 in the master plan is **"Rust microkernel + information-flow proofs."** Today the proof side is scaffolded (SP-C2.1 Verus harness landed) but no load-bearing proof has landed. SP-VER-002 produces the load-bearing artifact — a machine-checked proof of cave non-interference for the IPC subsystem. That's the artifact a DARPA PM or NIAP CCTL points at when validating the differentiator #1 claim.

Combined with SP-VER-001 (capability dispatcher proof — also planned) + SP-VER-005 (verified subsystem boundary doc — landed), this gives Sphragis a formal-verification posture that:
- Beats every Rust competitor (Hubris, RedoxOS, Asterinas have no info-flow proofs)
- Doesn't compete with seL4 on raw scope (seL4 has full functional correctness; we have non-interference on critical subsystems — different + complementary claim)
- Is exactly the deliverable DARPA PROVERS / INSPECTA wanted (per the autonomous run's Phase 1 research)

## REQ traceability

Closes REQ-VER-002 (design portion). The IMPL closes the rest across 5 sub-phases.

## References

- Verus tutorial: https://verus-lang.github.io/verus/guide/
- Non-interference (Goguen & Meseguer 1982): the canonical definition of the property we're proving
- seL4 functional correctness proof: https://sel4.systems/Verification/ — different shape of proof; reference for "what verified looks like at the production-scale OS level"
- Sphragis VERIFICATION_BOUNDARY.md — Properties P1-P4 (this is P3)
- Sphragis verification/README.md — Verus install + how-to-run
