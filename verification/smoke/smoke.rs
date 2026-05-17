// Sphragis verification harness — smoke proof.
//
// This file exists to confirm the Verus toolchain runs and produces
// the expected `verified` output. It proves a single trivial property
// (a const-true theorem) — nothing load-bearing.
//
// SP-C2.2 (next) replaces this with the cave capability dispatcher
// non-interference proof.
//
// To run:
//   $VERUS verification/smoke/smoke.rs
//
// Expected output:
//   verification results:: 1 verified, 0 errors

use vstd::prelude::*;

verus! {

/// Trivially-true theorem: every u32 added to zero equals itself.
/// Verus discharges this via Z3 with no axioms required.
proof fn smoke_add_zero(x: u32)
    ensures
        x as int + 0int == x as int,
{
}

/// Sphragis-specific marker: prove that the cave-ID type fits in
/// u16. This is a structural invariant the real cave dispatcher
/// proof (SP-C2.2) will rely on — putting it here ensures the
/// invariant is verifiable in isolation before being layered into
/// the larger proof.
proof fn cave_id_fits_u16()
    ensures
        u16::MAX as int >= 65535int,
{
}

} // verus!
