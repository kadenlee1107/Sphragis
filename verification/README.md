# Sphragis Formal Verification Harness

**Status:** Bootstrap (SP-C2.1, 2026-05-16). The directory + smoke proof
exist; the actual Verus toolchain is installed locally by the operator
(not in CI yet).

**Goal:** establish a verification harness so future sub-projects
(SP-C2 capability dispatcher proof, SP-C3 IPC info-flow proof) can land
without re-litigating tool choice or setup. The smoke proof below is
literally a const-true theorem — it exists to confirm the tool runs and
produces the expected "verified" output, not to prove anything load-bearing.

## Why Verus

Picked Verus over Kani for the master-plan-mandated *information-flow
non-interference* differentiator (#1). Verus is built specifically for
deductive verification of full functional + safety properties on
SMT-backable Rust programs; Kani is a bounded model checker that's a
better fit for memory-safety properties on small bounded slices. SP-C2.2
(IPC info-flow proof) is an existential-over-program-paths property
that Verus handles more naturally.

Kani stays available as a follow-on for memory-safety regression
tests (SP-VER-004 in the requirements spec) — different tool, different
job.

## Installation (operator-local; not in CI yet)

Verus ships its own pinned Rust toolchain because it depends on
unstable compiler internals. Install per the upstream guide:

```bash
# 1. Clone Verus
git clone https://github.com/verus-lang/verus ~/verus
cd ~/verus

# 2. Set up the project (downloads pinned toolchain + builds verifier)
./tools/get-z3.sh                     # pulls Z3 SMT solver
cd source && rustup show              # confirms pinned toolchain
cargo build --release                 # builds the verifier
```

Once built, set:

```bash
export VERUS=~/verus/source/target-verus/release/verus
```

## Running the smoke proof

From the Sphragis repo root:

```bash
$VERUS verification/smoke/smoke.rs
```

Expected output: `verification results:: 1 verified, 0 errors`.

## Directory layout

| Path | Purpose |
|---|---|
| `verification/README.md` | This file — overview + install + how-to-run |
| `verification/smoke/smoke.rs` | Single `const-true`-style proof that verifies the toolchain works |
| `verification/cap_dispatch/SPEC.md` | Capability dispatcher non-interference Verus proof specification (SP-VER-001 design). The proof itself is SP-VER-001.IMPL (~7 weeks per phasing). |
| `verification/ipc_flow/SPEC.md` | IPC information-flow non-interference Verus proof specification (SP-VER-002 design). The proof itself is SP-VER-002.IMPL (~8 weeks per phasing). |

## CI integration (future, SP-C2.2)

Once a future SP wires Verus into the GitHub Actions workflow, CI will
fail any PR that breaks an existing proof. Until then, verification is
operator-driven: run `$VERUS verification/smoke/smoke.rs` before merging
work that touches a verified subsystem.

The CI integration is intentionally deferred to a separate SP because:
1. Verus's toolchain is ~hundreds of MB; CI image needs caching strategy
2. The smoke proof has no load-bearing claim, so a CI failure here is
   tool-regression theater rather than security signal
3. Real verification value lands when SP-C2.2 produces the
   non-interference proof of the cave capability dispatcher

## References

- Verus: https://github.com/verus-lang/verus
- Verus tutorial: https://verus-lang.github.io/verus/guide/
- Kani (sibling tool, SP-VER-004): https://github.com/model-checking/kani
- Strategic differentiator #1 in the master plan: Rust microkernel +
  info-flow proofs on capability/IPC subsystem
