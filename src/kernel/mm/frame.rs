// Bat_OS — Page Frame Allocator
// Manages physical memory in 4KB pages using a bitmap.
// Simple, predictable, no external dependencies.

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = 524288; // 2GB / 4KB = 524288 frames
const BITMAP_SIZE: usize = MAX_FRAMES / 64; // 8192 u64s = 8192 bitmap entries

static BITMAP: [AtomicU64; BITMAP_SIZE] = {
    const INIT: AtomicU64 = AtomicU64::new(0);
    [INIT; BITMAP_SIZE]
};

static MEMORY_START: AtomicUsize = AtomicUsize::new(0);
static MEMORY_END_ADDR: AtomicUsize = AtomicUsize::new(0);
static TOTAL_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn init(start: usize, end: usize) {
    let start_aligned = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let end_aligned = end & !(PAGE_SIZE - 1);

    MEMORY_START.store(start_aligned, Ordering::Relaxed);
    MEMORY_END_ADDR.store(end_aligned, Ordering::Relaxed);
    TOTAL_FRAMES.store((end_aligned - start_aligned) / PAGE_SIZE, Ordering::Relaxed);
}

pub fn alloc_frame() -> Option<usize> {
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);

    // V2-001/V2-040: reserve the top of memory for kernel-only allocations
    // (specifically cave page-table frames). Regular alloc_frame stops
    // before that pool so cave user-window mappings can never alias into
    // a cave's own L1/L2 table.
    let user_cap = total.saturating_sub(KERNEL_RESERVED_FRAMES);

    for i in 0..BITMAP_SIZE {
        let val = BITMAP[i].load(Ordering::Relaxed);
        if val == u64::MAX {
            continue; // All 64 bits used
        }

        // Find first free bit
        let bit = (!val).trailing_zeros() as usize;
        let frame_index = i * 64 + bit;

        if frame_index >= user_cap {
            return None;
        }

        // Claim this frame (single-core, no CAS needed)
        let new_val = val | (1u64 << bit);
        BITMAP[i].store(new_val, Ordering::Relaxed);
        {
            let addr = start + frame_index * PAGE_SIZE;
            // Zero the page using inline asm str (HVF-safe)
            unsafe {
                let ptr = addr;
                for i in 0..(PAGE_SIZE / 8) {
                    let target = ptr + i * 8;
                    core::arch::asm!(
                        "str xzr, [{addr}]",
                        addr = in(reg) target,
                    );
                }
            }
            return Some(addr);
        }
    }

    None
}

/// Kernel-reserved frames at the top of the memory range. Never returned
/// by `alloc_frame`, so cave user-window mappings cannot alias into them.
/// Sized for ~64 caves × 4 tables each (256) + slack.
pub const KERNEL_RESERVED_FRAMES: usize = 512;

/// V2-001/V2-040 fix: allocate a frame from the kernel-reserved pool.
/// Used by `setup_cave_pagetable` / `setup_cave_pagetable_at` so a cave's
/// own L1 / L2 tables can never be remapped into the cave's user window.
///
/// Returns None if the kernel pool is exhausted; callers should surface
/// this as "too many caves" rather than falling back to alloc_frame.
pub fn alloc_kernel_frame() -> Option<usize> {
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);
    if total < 1 { return None; }
    let lower_bound = total.saturating_sub(KERNEL_RESERVED_FRAMES);

    // Search top-down over the reserved range.
    for rev in 0..KERNEL_RESERVED_FRAMES {
        let frame_index = total.saturating_sub(1).saturating_sub(rev);
        if frame_index < lower_bound { break; }
        let bitmap_index = frame_index / 64;
        let bit = frame_index % 64;
        if bitmap_index >= BITMAP_SIZE { continue; }
        let val = BITMAP[bitmap_index].load(Ordering::Relaxed);
        if val & (1u64 << bit) != 0 { continue; } // in use

        BITMAP[bitmap_index].store(val | (1u64 << bit), Ordering::Relaxed);
        let addr = start + frame_index * PAGE_SIZE;
        unsafe {
            for i in 0..(PAGE_SIZE / 8) {
                core::arch::asm!("str xzr, [{a}]",
                    a = in(reg) addr + i * 8,
                    options(nostack, preserves_flags));
            }
        }
        return Some(addr);
    }
    None
}

pub fn free_frame(addr: usize) {
    let start = MEMORY_START.load(Ordering::Relaxed);
    if addr < start {
        return;
    }

    // Defense-in-depth: wipe the page contents before returning it to the
    // free pool. alloc_frame already zeroes on allocation, so this is
    // belt-and-suspenders against the window between free and the next
    // alloc where the page might be read by a DMA-capable peripheral or
    // a snapshot-based introspection tool. Uses str xzr to mirror the
    // existing HVF-safe pattern in alloc_frame rather than write_bytes.
    unsafe {
        for i in 0..(PAGE_SIZE / 8) {
            core::arch::asm!("str xzr, [{a}]",
                a = in(reg) addr + i * 8,
                options(nostack, preserves_flags));
        }
    }

    let frame_index = (addr - start) / PAGE_SIZE;
    let bitmap_index = frame_index / 64;
    let bit = frame_index % 64;

    if bitmap_index < BITMAP_SIZE {
        // Simple store instead of fetch_and (no exclusive monitors — HVF safe)
        let val = BITMAP[bitmap_index].load(Ordering::Relaxed);
        BITMAP[bitmap_index].store(val & !(1u64 << bit), Ordering::Relaxed);
    }
}

/// Free a run of `count` contiguous physical pages starting at `base`.
/// Convenience wrapper over `free_frame`, used by the loader and munmap
/// paths that allocated large contiguous regions (e.g. 38k pages for a
/// Chromium-sized ELF). Silently ignores unaligned or out-of-range bases
/// so callers can blindly free "whatever I got from alloc".
pub fn free_contig(base: usize, count: usize) {
    if count == 0 { return; }
    let base = base & !(PAGE_SIZE - 1);
    for i in 0..count {
        free_frame(base + i * PAGE_SIZE);
    }
}

pub fn stats() -> (usize, usize) {
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let mut used = 0usize;

    for i in 0..BITMAP_SIZE {
        let mut val = BITMAP[i].load(Ordering::Relaxed);
        while val != 0 {
            used += 1;
            val &= val - 1; // Clear lowest set bit (Kernighan's trick)
        }
    }

    (used, total)
}
