//! ATTACK-KM-002 — free_frame has no double-free / alignment check.
//!
//! Proves that:
//!   (a) An unaligned address that lies inside a frame clears the neighbour's bit.
//!   (b) Calling free twice on the same frame is silently accepted.
//!   (c) Freeing an address below `MEMORY_START` is silently ignored
//!       (not the exploit — just the silent-failure surface that leads to
//!       resource leaks).

use km_attacks::frame_sim::{Allocator, PAGE_SIZE};

#[test]
fn free_of_unaligned_address_clears_wrong_bit() {
    let base = 0x1000_0000;
    let top = base + PAGE_SIZE * 64;
    let alloc = Allocator::new(base, top);

    // Manually mark frames 0 and 1 as allocated.
    let _ = alloc.alloc_frame_racy(); // frame 0
    let _ = alloc.alloc_frame_racy(); // frame 1
    assert!(alloc.bit_is_set(0));
    assert!(alloc.bit_is_set(1));

    // Free at base + 1 — inside frame 0, NOT aligned. Integer division
    // still maps to frame 0.
    assert!(alloc.free_frame_lax(base + 1));
    assert!(
        !alloc.bit_is_set(0),
        "free of unaligned (base+1) cleared frame-0's bit — no alignment check"
    );
    // And the lie is: user thought they freed nothing.
    // Meanwhile, they've just leaked the alloc.
}

#[test]
fn double_free_is_silent() {
    let base = 0x1000_0000;
    let top = base + PAGE_SIZE * 64;
    let alloc = Allocator::new(base, top);

    let pa = alloc.alloc_frame_racy().unwrap();
    assert!(alloc.free_frame_lax(pa));
    // bit was cleared; allocator doesn't know it was already cleared.
    let second = alloc.free_frame_lax(pa);
    assert!(
        second,
        "double free accepted — no sentinel / no audit trail"
    );
    // A real kernel would panic or log; this one doesn't.
}

#[test]
fn free_below_start_silently_dropped() {
    let base = 0x1000_0000;
    let top = base + PAGE_SIZE * 64;
    let alloc = Allocator::new(base, top);

    let pa = alloc.alloc_frame_racy().unwrap();
    let bogus = base.wrapping_sub(PAGE_SIZE);
    assert!(
        !alloc.free_frame_lax(bogus),
        "free below start returned true — unexpected"
    );
    // frame still marked used.
    assert!(alloc.bit_is_set((pa - base) / PAGE_SIZE));
}
