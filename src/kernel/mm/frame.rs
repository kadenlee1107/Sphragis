// Bat_OS — Page Frame Allocator
// Manages physical memory in 4KB pages using a bitmap.
// Simple, predictable, no external dependencies.

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = 32768; // 128MB / 4KB = 32768 frames
const BITMAP_SIZE: usize = MAX_FRAMES / 64; // 512 u64s = 512 bitmap entries

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

    for i in 0..BITMAP_SIZE {
        let val = BITMAP[i].load(Ordering::Relaxed);
        if val == u64::MAX {
            continue; // All 64 bits used
        }

        // Find first free bit
        let bit = (!val).trailing_zeros() as usize;
        let frame_index = i * 64 + bit;

        if frame_index >= total {
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

pub fn free_frame(addr: usize) {
    let start = MEMORY_START.load(Ordering::Relaxed);
    if addr < start {
        return;
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
