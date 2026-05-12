//! Dedicated PA pool for cave-private frames, outside the kernel
//! identity range. Frames from this pool are reachable ONLY via
//! per-cave L1 mappings — PRIMARY_L1 and every other cave's L1
//! have their kernel identity windows carved so no entry covers
//! this PA range.
//!
//! ### Layout (QEMU virt and Apple M4 alike)
//!
//! `CAVE_POOL_BASE..CAVE_POOL_END` carved from the top of the
//! kernel pool (0xB000_0000..0xC000_0000 = 256 MiB). Sitting at
//! the top of the kernel-PA-reachable window means:
//!   - It's always in physical RAM (configurations down to -m 2G).
//!   - It's inside the kernel pool's reachable region
//!     (`KERNEL_FRAME_PA_CAP = 0xC000_0000`) so the page-table
//!     walker can still reach it once a cave's L1 includes a
//!     mapping for these PAs.
//!   - The general `frame::alloc_frame` / `alloc_kernel_frame`
//!     paths skip these PAs via `frame::reserve_range` at boot.
//!
//! ### Security claim
//!
//! Even if an attacker pins down the PA backing a cave-private
//! page (e.g. by reading the leaf PTE from the cave's L1), they
//! cannot reach the bytes through PRIMARY_L1's kernel identity
//! window because the carve-out leaves that L1 region invalid.
//! The ONLY mapping covering this PA range lives inside the
//! owning cave's L1 — and only at a VA the cave-private allocator
//! installs.
//!
//! ### Allocator
//!
//! Simple bitmap, one bit per 4 KiB page. 256 MiB / 4 KiB =
//! 65,536 pages = 1024 `u64` bitmap words. No global lock
//! needed at present (single CPU, cooperative scheduling); the
//! per-word load/store happens under `IrqGuard` to match the
//! existing frame allocator's invariant.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU64, Ordering};
use super::frame::PAGE_SIZE;

pub const CAVE_POOL_BASE: usize = 0xB000_0000;
pub const CAVE_POOL_END:  usize = 0xC000_0000;
pub const CAVE_POOL_PAGES: usize = (CAVE_POOL_END - CAVE_POOL_BASE) / PAGE_SIZE;
const BITMAP_WORDS: usize = (CAVE_POOL_PAGES + 63) / 64;

static BITMAP: [AtomicU64; BITMAP_WORDS] =
    [const { AtomicU64::new(0) }; BITMAP_WORDS];

/// Idempotent. Called from `kernel::mm::init` after the frame
/// allocator is up — reserves the cave-pool PA range from the
/// general frame allocator so `alloc_frame` / `alloc_kernel_frame`
/// can never hand out a page inside our window.
pub fn init() {
    // The general bitmap is zero-initialised by the linker. We
    // don't need to clear anything here — what matters is that
    // the general frame allocator marks these PAs as in-use so
    // no concurrent allocator path hands them out.
    super::frame::reserve_range(CAVE_POOL_BASE, CAVE_POOL_END);
}

/// Allocate one 4 KiB page from the cave pool. Returns the PA on
/// success, `None` if the pool is exhausted.
///
/// The page is NOT zeroed here. The carve-out makes the PA's
/// identity VA unmapped in PRIMARY_L1, so a kernel-ns store would
/// data-abort. The caller (`cave_private::ensure_page`) installs
/// the page at a cave-private VA in the owning cave's L1 first,
/// then zeroes through that VA under `with_cave_active` — see the
/// cave-private allocator for the discipline.
///
/// Initial pool contents: zero, because the `.bss` cave-pool
/// allocator doesn't reclaim used frames before init() runs, and
/// boot-time RAM is zero-initialised for kernel allocations.
pub fn alloc_page() -> Option<usize> {
    let _g = crate::kernel::sync::IrqGuard::new();
    for w in 0..BITMAP_WORDS {
        let val = BITMAP[w].load(Ordering::Acquire);
        if val == u64::MAX { continue; }
        for bit in 0..64usize {
            let frame_index = w * 64 + bit;
            if frame_index >= CAVE_POOL_PAGES { break; }
            if val & (1u64 << bit) != 0 { continue; }
            BITMAP[w].store(val | (1u64 << bit), Ordering::Release);
            let pa = CAVE_POOL_BASE + frame_index * PAGE_SIZE;
            return Some(pa);
        }
    }
    None
}

/// Release a page back to the pool. No-op if the PA falls outside
/// the pool (we'd never have allocated such a page; safe to ignore).
pub fn free_page(pa: usize) {
    if pa < CAVE_POOL_BASE || pa >= CAVE_POOL_END { return; }
    let frame_index = (pa - CAVE_POOL_BASE) / PAGE_SIZE;
    if frame_index >= CAVE_POOL_PAGES { return; }
    let _g = crate::kernel::sync::IrqGuard::new();
    let w = frame_index / 64;
    let bit = frame_index % 64;
    let val = BITMAP[w].load(Ordering::Acquire);
    BITMAP[w].store(val & !(1u64 << bit), Ordering::Release);
}

/// Count of currently-allocated pages — for selftest assertions.
pub fn used_pages() -> usize {
    let mut n = 0;
    for w in 0..BITMAP_WORDS {
        n += BITMAP[w].load(Ordering::Relaxed).count_ones() as usize;
    }
    n.min(CAVE_POOL_PAGES)
}

/// Zero a 4 KiB cave-pool page through a VA the caller has already
/// mapped in the owning cave's L1. `va` must translate to the cave-
/// pool PA the caller got from `alloc_page`. Called by
/// `cave_private::ensure_page` after the leaf PTE is installed and
/// the cave's L1 is active.
pub fn zero_via_cave_va(va: usize) {
    unsafe {
        for i in 0..(PAGE_SIZE / 8) {
            core::arch::asm!("str xzr, [{a}]",
                a = in(reg) va + i * 8,
                options(nostack, preserves_flags));
        }
        let mut line = va as u64;
        let end = line + PAGE_SIZE as u64;
        while line < end {
            core::arch::asm!("dc civac, {a}", a = in(reg) line);
            line += 64;
        }
        core::arch::asm!("dsb sy");
    }
}
