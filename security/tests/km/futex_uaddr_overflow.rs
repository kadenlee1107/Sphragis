//! ATTACK-KM-021 — futex `uaddr` alignment check is not a range check.

use km_attacks::futex_sim::passes_uaddr_check;

#[test]
fn near_u64_max_addresses_pass_alignment_check() {
    // Aligned and non-zero — passes the kernel's guard.
    assert!(passes_uaddr_check(0xFFFF_FFFF_FFFF_FFFCu64));
    // Even a kernel-range address passes:
    assert!(passes_uaddr_check(0x4000_0000u64));
    // 8-byte aligned kernel text:
    assert!(passes_uaddr_check(0x4020_0000u64));
}

#[test]
fn user_range_passes_too_but_we_cant_distinguish() {
    assert!(passes_uaddr_check(0x0040_0000u64));
    // The test's point: with ONLY the alignment check, kernel and user
    // addresses are indistinguishable at the futex layer.
}

#[test]
fn zero_and_unaligned_rejected() {
    assert!(!passes_uaddr_check(0));
    assert!(!passes_uaddr_check(0x1001));
}
