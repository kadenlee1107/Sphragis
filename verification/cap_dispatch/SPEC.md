# Sphragis Capability Dispatcher Non-Interference — Verus Proof Specification

**Document version:** 1.0 (SP-VER-001, 2026-05-16)
**Status:** Proof specification; the actual Verus proof is SP-VER-001.IMPL (multi-week effort).
**Companion docs:** `VERIFICATION_BOUNDARY.md` (overall verified subsystem scope), `verification/README.md` (Verus harness install), `verification/ipc_flow/SPEC.md` (sister proof for IPC channels), `DESIGN_CAVE_ISOLATION.md` (the underlying isolation model).
**REQ:** Closes REQ-VER-001 design portion. Together with `ipc_flow/SPEC.md` these two specs cover the differentiator-#1 information-flow non-interference claim end-to-end (cap-dispatcher proof says "the gate is sound"; IPC proof says "the gated paths preserve isolation").

## Goal

Prove, by deductive reasoning checked by Verus + Z3, the following property:

> **Capability Dispatcher Non-Interference (P2).** Let A and B be distinct caves with disjoint capability sets in their respective `Cave.caps` arrays. Then for every kernel-internal capability-mediated operation `op`, every kernel state σ, every cave context `as_cave` switching, and every sequence of operations σ_A invoked by cave A: the post-state observable to cave B is independent of A's capability-mediated operations on caps it holds but B does not.

In English: cave A's use of a capability X that B does not hold cannot affect any state observable to B unless the operator explicitly granted B a paired capability and the policy permits the cross-cave effect.

## Why this property is load-bearing

Sphragis's security model rests on a single architectural assumption: **the only way for one cave's actions to affect another cave's observable state is through capabilities held by both, mediated by the kernel-side dispatcher**. The cave-isolation property (per-cave page tables + per-cave ASIDs, week-11 audit closure) hardware-checks the memory side. The IPC info-flow proof (sister SPEC) covers the IPC-channel side. This cap-dispatcher proof covers everything else — every syscall, every kernel-side mutable resource that's gated by `has_cap()`.

Specifically the proof rules out:
- A buggy `has_cap()` that returns true for a cap the cave doesn't hold (false positive)
- A TOCTOU race between cap check and cap use that lets a non-holder act
- A missing cap check on a new syscall handler (linted at compile-time by SP-ISO-004 IMPL but additionally enforced by the proof)
- A confused-deputy where a kernel routine acts on behalf of A using B's caps without operator intent
- A capability-bit overflow that aliases two distinct caps to the same value

The MMU is the floor; IPC info-flow is one ceiling; the cap dispatcher is the OTHER ceiling. Together they bound the set of cave-to-cave influences.

## What's being proven (mathematically)

Pick a state representation:

```
State = (
    caves: [Cave; MAX_CAVES],          // each Cave has caps: [CaveCap; MAX_CAPS]
    cap_policy: CapPolicyTable,        // operator-defined cap→cave grants
    kernel_resources: KernelResources, // mutable kernel state gated by caps
)
```

For two distinct caves a, b ∈ {0..MAX_CAVES}, define `obs_b(σ)` as the part of σ observable to cave b — concretely:
- The set of caps b holds: `σ.caves[b].caps`
- The kernel resources b can read via syscalls (file inodes b owns, mappings b made, etc.)
- The metadata b can query (own cave id, own scheduling state, etc.)

For a single capability-mediated operation:

```
op := (cap: CapId, acting_cave: CaveId, payload: Bytes)
```

`dispatch(σ, op)` either:
- (Accepts) returns `Ok(σ')` if `σ.caves[op.acting_cave].caps` contains `op.cap`
- (Rejects) returns `Err(EPERM)` and σ' = σ otherwise (the rejection is the entire effect — no logging side channel; that's a separate threat covered by AUD-002 WORM observability)

Theorem we prove:

```
forall a, b: CaveId, ops_a: Sequence<Op>, σ: State :
    a ≠ b ∧
    disjoint(σ.caves[a].caps, σ.caves[b].caps) ⟹
    obs_b(execute(σ, ops_a as a)) ≡ obs_b(σ)
```

Where:
- `disjoint(c_a, c_b)` is true iff for every (cap, granted) in c_a where granted=true, c_b does not contain (cap, granted=true). This is the operator's policy expression.
- `execute(σ, ops_a as a)` applies each op in sequence, treating `acting_cave = a` for every op.
- `≡` is byte-equality on every observable field.

## Proof strategy

Step 1 — **Type-state invariants.** Define a `CapDispatch` type-state that encodes the cap-check pre-condition for every kernel resource access. The invariant: `kernel_resources[r].last_modified_by == c ⟹ σ.caves[c].caps contains required_cap(r)`. Lemma:

```rust
proof fn dispatch_preserves_invariant(σ: State, op: Op)
    requires inv(σ),
    ensures inv(dispatch(σ, op).0),
{ ... }
```

Step 2 — **Per-op refinement specs.** For every concrete `Op` variant (file_open, file_write, ipc_send, schedule_yield, audit_record_lookup, ...), state and prove:

```rust
proof fn op_only_affects_actors_caps(σ: State, op: Op, b: CaveId)
    requires
        inv(σ),
        op.acting_cave != b,
        disjoint(σ.caves[op.acting_cave].caps, σ.caves[b].caps),
    ensures
        obs_b(dispatch(σ, op).0) ≡ obs_b(σ),
{ ... }
```

This is the per-op heart of the proof. Each variant gets its own lemma.

Step 3 — **Case analysis on Op variants.** Compose the per-op lemmas into:

```rust
proof fn single_op_no_cross_cave_effect(σ: State, op: Op, a: CaveId, b: CaveId)
    requires
        inv(σ), a ≠ b, op.acting_cave == a,
        disjoint(σ.caves[a].caps, σ.caves[b].caps),
    ensures
        obs_b(dispatch(σ, op).0) ≡ obs_b(σ),
{
    match op {
        Op::FileOpen(_) => op_only_affects_actors_caps(σ, op, b),
        Op::FileWrite(_) => op_only_affects_actors_caps(σ, op, b),
        ... // one arm per Op variant
    }
}
```

Step 4 — **Top-level theorem composition.** Induct over the operation sequence:

```rust
proof fn cap_dispatch_non_interference(σ: State, ops_a: Seq<Op>, a: CaveId, b: CaveId)
    requires
        a ≠ b,
        disjoint(σ.caves[a].caps, σ.caves[b].caps),
        forall|op: Op| #[trigger] ops_a.contains(op) ⟹ op.acting_cave == a,
    ensures
        obs_b(execute(σ, ops_a)) ≡ obs_b(σ),
    decreases ops_a.len(),
{
    if ops_a.len() == 0 { /* trivial */ }
    else {
        let σ1 = dispatch(σ, ops_a[0]).0;
        single_op_no_cross_cave_effect(σ, ops_a[0], a, b);
        cap_dispatch_non_interference(σ1, ops_a.subrange(1, ops_a.len()), a, b);
    }
}
```

## Out-of-scope (caveats — documented adversary capabilities NOT covered)

- **Timing side channels.** Cave A's cap-use timing (cycles spent in `dispatch`) is observable to cave B via a shared clock. SP-VER-005 (constant-time discipline) addresses by separate proof + CI lint; the cap-dispatch proof intentionally abstracts time.
- **MMU bypass.** The proof assumes per-cave page tables hold (the week-11 ASID audit closure verified this hardware-side). If the MMU setup is buggy, all bets are off — that's why the per-cave page table init is the floor, not a target of THIS proof.
- **Operator misconfiguration.** The proof says "disjoint caps ⟹ no influence". If the operator grants both caves overlapping caps, the property doesn't apply. Operator-side cap grants are logged via SP-AUD-003 categories.
- **Kernel-side caller deception.** The proof assumes `op.acting_cave` is the TRUE acting cave per the scheduler context — not a value an attacker can spoof from user-mode. The syscall entry stub (`src/kernel/syscall.rs`) is the trusted boundary that establishes `op.acting_cave` from the active scheduler context; that boundary is verified by inspection (it's <50 LOC).
- **Capability forgery via heap corruption.** If an attacker can corrupt the `Cave.caps` array via a kernel heap overflow, the proof's input invariant no longer holds. Kernel-heap integrity is week-3-4 BatFS-C1 audit closure (IrqGuard around critical sections) + Rust borrow-checker for in-tree code.

## Implementation phasing (SP-VER-001.IMPL)

What .IMPL must land:

1. **`verification/cap_dispatch/state.rs`** (~200 LoC): Verus model types (`State`, `Cave`, `Op`, `KernelResources`) — a *simplified* abstract model that matches the real cave/cap structures in `src/caves/cave.rs` at the field-name and invariant level, but elides unmodeled details (display state, scheduling queues, etc.).
2. **`verification/cap_dispatch/dispatch.rs`** (~150 LoC): the `dispatch` function pure-modeled per Step 1.
3. **`verification/cap_dispatch/lemmas.rs`** (~300 LoC): per-op refinement lemmas per Step 2.
4. **`verification/cap_dispatch/theorem.rs`** (~100 LoC): the top-level non-interference theorem per Step 4.
5. **`verification/cap_dispatch/refinement.md`** (doc): mapping from the abstract model fields to the concrete `src/caves/cave.rs` fields. Reviewer-visible so the gap between proven and implemented is auditable.

Phased rollout:
- Phase A (~1 week): state.rs + smoke proof verifies (just the invariants — no actual non-interference yet).
- Phase B (~2 weeks): dispatch.rs + lemmas.rs for FileOpen + FileWrite ops only. Single-op theorem verifies for those two.
- Phase C (~2 weeks): extend lemmas to all Op variants in the abstract model.
- Phase D (~1 week): composition theorem (induction over sequence). Top-level theorem verifies.
- Phase E (~1 week): refinement.md documents the gap. Reviewer feedback loop.

Total ~7 weeks. Could be PROVERS-funded under DARPA's verification program.

## Out-of-scope for .IMPL (also caveats)

- Mechanized refinement proof between the abstract model and `src/caves/cave.rs` itself. We're proving a property of the *model*; refinement is doc-level today (`refinement.md`). Full mechanized refinement is SP-VER-001.REFINE (multi-week follow-on; the bar most projects don't clear and we're not pretending to).
- Proofs for the new `Op` variants added after this SP lands. The discipline going forward: any new syscall handler that's gated by `has_cap()` must come with a new arm in the abstract `Op` enum + new per-op lemma. CI-enforced by SP-ISO-004 IMPL (workflow YAML — OAuth-blocked from autonomous, in `.github-workflows-pending/`).

## REQ traceability

Closes REQ-VER-001 (design portion). The IMPL closes the rest.

Together with `verification/ipc_flow/SPEC.md`, these two specs are the formal-verification deliverable backing strategic-differentiator #1 ("Rust microkernel + info-flow proofs on capability/IPC subsystem").

## References

- `verification/README.md` — Verus harness install + smoke proof
- `verification/ipc_flow/SPEC.md` — sister proof for IPC channels
- `src/caves/cave.rs` — concrete `has_cap()`, `CaveCap`, `active_has_cap()` implementations
- `DESIGN_CAVE_ISOLATION.md` — design-side rationale for isolation model
- Verus: https://github.com/verus-lang/verus
- Verus tutorial: https://verus-lang.github.io/verus/guide/
- Information-flow non-interference (Goguen + Meseguer 1982): https://www.csl.sri.com/~rushby/papers/oakland82.pdf
