//! ATTACK-KM-016 / ATTACK-KM-017 — kernel PTEs lack UXN/PXN.

use km_attacks::pt_flags::{BLOCK_NORMAL, PTE_PXN, PTE_UXN};

#[test]
fn block_normal_has_no_uxn() {
    assert_eq!(
        BLOCK_NORMAL & PTE_UXN,
        0,
        "ATTACK-KM-016 confirmed: BLOCK_NORMAL does not set UXN — EL0 could execute kernel RAM"
    );
}

#[test]
fn block_normal_has_no_pxn() {
    assert_eq!(
        BLOCK_NORMAL & PTE_PXN,
        0,
        "ATTACK-KM-017 confirmed: BLOCK_NORMAL does not set PXN — EL1 could execute data pages"
    );
}
