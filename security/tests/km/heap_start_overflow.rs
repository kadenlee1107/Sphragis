//! ATTACK-KM-004 — integer overflow in `heap_start`.

use km_attacks::heap_sim::{heap_start_checked, heap_start_unchecked, BlobInfo};

#[test]
fn crafted_blob_size_wraps_heap_start_into_kernel_text() {
    let kernel_end = 0x4020_0000usize;
    let memory_end = 0x4000_0000usize + (2 * 1024 * 1024 * 1024);

    // Attacker-controlled (via crafted baked blob) size field.
    let info = Some(BlobInfo {
        size: usize::MAX - 100,
    });

    let got = heap_start_unchecked(kernel_end, info);
    // On overflow, `got` wraps to an address that, without the overflow,
    // would have been astronomical (kernel_end + ~usize::MAX). Post-wrap,
    // it lands near 0 (or at kernel_end itself after alignment). Either
    // case is catastrophically wrong vs the `memory_end` the allocator
    // believes it has.
    assert!(
        got < memory_end,
        "heap_start={:#x} is inside the declared memory window — frame allocator would now hand out kernel-adjacent memory as if it were heap",
        got
    );
    eprintln!(
        "ATTACK-KM-004 confirmed: heap_start wrapped to {:#x} (kernel_end was {:#x}, memory_end was {:#x})",
        got, kernel_end, memory_end
    );

    // The "what it should do" path refuses.
    let safe = heap_start_checked(
        kernel_end,
        Some(BlobInfo {
            size: usize::MAX - 100,
        }),
        memory_end,
    );
    assert!(safe.is_none(), "checked version should reject");
}

#[test]
fn normal_sized_blob_is_handled_fine_in_either_version() {
    let kernel_end = 0x4020_0000usize;
    let memory_end = 0x4000_0000usize + (2 * 1024 * 1024 * 1024);
    let info = BlobInfo { size: 10 * 1024 * 1024 };
    let a = heap_start_unchecked(kernel_end, Some(BlobInfo { size: info.size }));
    let b = heap_start_checked(kernel_end, Some(info), memory_end).unwrap();
    assert_eq!(a, b);
    assert!(a > kernel_end);
    assert!(a < memory_end);
}
