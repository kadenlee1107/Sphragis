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

    // V6-KMEM-001: heap now lives BELOW `start` (mm::init places it
    // immediately past the initrd blob and starts the frame range past
    // the heap), so we don't need a frame-bitmap reservation. Defense
    // in depth: if the heap is somehow inside our range (caller bug),
    // reserve_range still marks it as in-use.
    let heap_start = super::heap::kernel_heap_base();
    let heap_end = heap_start + super::heap::kernel_heap_size();
    if heap_start != 0 && heap_start >= start_aligned && heap_end <= end_aligned {
        reserve_range(heap_start, heap_end);
    }
}

/// Mark every frame in [start, end) as in-use so alloc_frame skips them.
/// Used to carve out fixed kernel-owned regions (kernel heap, MMIO, etc.)
/// from the general page pool.
pub fn reserve_range(start: usize, end: usize) {
    let mem_start = MEMORY_START.load(Ordering::Relaxed);
    let mem_end = MEMORY_END_ADDR.load(Ordering::Relaxed);
    // Only reserve frames that actually fall in the pool.
    let s = start.max(mem_start) & !(PAGE_SIZE - 1);
    let e = end.min(mem_end) & !(PAGE_SIZE - 1);
    if e <= s { return; }
    let mut addr = s;
    while addr < e {
        let frame_index = (addr - mem_start) / PAGE_SIZE;
        let bitmap_index = frame_index / 64;
        let bit = frame_index % 64;
        if bitmap_index < BITMAP_SIZE {
            let val = BITMAP[bitmap_index].load(Ordering::Relaxed);
            BITMAP[bitmap_index].store(val | (1u64 << bit), Ordering::Relaxed);
        }
        addr += PAGE_SIZE;
    }
}

pub fn alloc_frame() -> Option<usize> {
    // V8-ROOT-1 + V8-KMEM-ROOT-1: the entire scan-find-claim sequence is
    // one critical section. Previously the load-then-store on BITMAP[i]
    // could race with a timer IRQ that itself allocates (via log → heap →
    // alloc_frame), yielding the same bit to two callers. Heap got this
    // fix in V6-TOCTOU-007; frame allocator did not.
    let _g = crate::kernel::sync::IrqGuard::new();

    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);
    let user_cap = total.saturating_sub(KERNEL_RESERVED_FRAMES);

    for i in 0..BITMAP_SIZE {
        // M4 / MMU-off: `compare_exchange*` lowers to LDXR/STXR which
        // never succeeds on Device-nGnRnE memory. We already hold
        // `IrqGuard` above and we're single-CPU during bring-up, so
        // plain load + check + store is exclusive. When SMP lands the
        // frame allocator will need a proper lock (or `+lse`) anyway.
        let val = BITMAP[i].load(Ordering::Acquire);
        if val == u64::MAX { continue; } // full word, try next

        let bit = (!val).trailing_zeros() as usize;
        let frame_index = i * 64 + bit;

        if frame_index >= user_cap {
            return None;
        }

        BITMAP[i].store(val | (1u64 << bit), Ordering::Release);

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
    // V8-ROOT-1: same CS discipline as alloc_frame.
    let _g = crate::kernel::sync::IrqGuard::new();

    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);
    if total < 1 { return None; }
    let lower_bound = total.saturating_sub(KERNEL_RESERVED_FRAMES);

    for rev in 0..KERNEL_RESERVED_FRAMES {
        let frame_index = total.saturating_sub(1).saturating_sub(rev);
        if frame_index < lower_bound { break; }
        let bitmap_index = frame_index / 64;
        let bit = frame_index % 64;
        if bitmap_index >= BITMAP_SIZE { continue; }
        // M4 / MMU-off: plain load + store under the outer IrqGuard.
        // See `alloc_frame` for why CAS doesn't work here.
        let val = BITMAP[bitmap_index].load(Ordering::Acquire);
        if val & (1u64 << bit) != 0 { continue; }
        BITMAP[bitmap_index].store(val | (1u64 << bit), Ordering::Release);

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

/// Allocate `n_pages` physically-contiguous, zero-filled 4 KiB frames.
/// Returns the base address of the first frame, or None if no such
/// run exists.
///
/// V-ASAHI-3.2: needed by DART TRANSLATE to allocate 16 KiB (4-page)
/// page-table root blocks that must be contiguous in physical RAM.
pub fn alloc_contig(n_pages: usize) -> Option<usize> {
    if n_pages == 0 { return None; }
    if n_pages == 1 { return alloc_frame(); }

    let _g = crate::kernel::sync::IrqGuard::new();
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);
    let user_cap = total.saturating_sub(KERNEL_RESERVED_FRAMES);
    if user_cap < n_pages { return None; }

    // Scan bitmap for `n_pages` consecutive zero bits.
    let mut frame_idx: usize = 0;
    'outer: while frame_idx + n_pages <= user_cap {
        // Check each of the next n_pages bits — all must be clear.
        for j in 0..n_pages {
            let fi = frame_idx + j;
            let wi = fi / 64;
            let bit = fi % 64;
            if wi >= BITMAP_SIZE { return None; }
            if BITMAP[wi].load(Ordering::Acquire) & (1u64 << bit) != 0 {
                frame_idx = fi + 1;
                continue 'outer;
            }
        }
        // M4 / MMU-off: plain load + store under IrqGuard. `fetch_and`
        // / `fetch_or` lower to LDXR/STXR on Device memory and hang.
        // The rollback path is preserved (dead on UP, correct on SMP
        // once we add a real lock).
        for j in 0..n_pages {
            let fi = frame_idx + j;
            let wi = fi / 64;
            let bit = fi % 64;
            let cur = BITMAP[wi].load(Ordering::Acquire);
            if cur & (1u64 << bit) != 0 {
                // Someone raced us; roll back the bits we claimed.
                for k in 0..j {
                    let fk = frame_idx + k;
                    let wk = fk / 64;
                    let bk = fk % 64;
                    let v = BITMAP[wk].load(Ordering::Acquire);
                    BITMAP[wk].store(v & !(1u64 << bk), Ordering::Release);
                }
                frame_idx += 1;
                continue 'outer;
            }
            BITMAP[wi].store(cur | (1u64 << bit), Ordering::Release);
        }
        // Zero all frames.
        let base = start + frame_idx * PAGE_SIZE;
        unsafe {
            for i in 0..(n_pages * PAGE_SIZE / 8) {
                core::arch::asm!("str xzr, [{a}]",
                    a = in(reg) base + i * 8,
                    options(nostack, preserves_flags));
            }
        }
        return Some(base);
    }
    None
}

pub fn free_frame(addr: usize) {
    // V8-ROOT-1: zero-then-clear-bit must be atomic w.r.t. IRQ. Previously
    // a racing alloc_frame could see the bit still set, not return this
    // page, but an OTHER reader sees freshly-zeroed memory (UAF holders
    // see zeros mid-loop). Nestable — heap dealloc already holds this.
    let _g = crate::kernel::sync::IrqGuard::new();

    let start = MEMORY_START.load(Ordering::Relaxed);
    let end   = MEMORY_END_ADDR.load(Ordering::Relaxed);
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);

    // V6-WEIRD-007 defense-in-depth: validate the frame index FIRST,
    // BEFORE zeroing. The previous order (zero → bitmap-index check)
    // let any caller that supplied an address in the kernel-RAM range
    // wipe 4 KB of kernel memory (heap, BSS, bitmap itself). Now we
    // refuse to touch the page unless its frame index is in-range.
    if addr < start || addr >= end || (addr & (PAGE_SIZE - 1)) != 0 {
        return;
    }
    let frame_index = (addr - start) / PAGE_SIZE;
    if frame_index >= total {
        return;
    }
    let bitmap_index = frame_index / 64;
    if bitmap_index >= BITMAP_SIZE {
        return;
    }

    // Now safe to wipe the page contents.
    unsafe {
        for i in 0..(PAGE_SIZE / 8) {
            core::arch::asm!("str xzr, [{a}]",
                a = in(reg) addr + i * 8,
                options(nostack, preserves_flags));
        }
    }

    let bit = frame_index % 64;
    // Simple store instead of fetch_and (no exclusive monitors — HVF safe)
    let val = BITMAP[bitmap_index].load(Ordering::Relaxed);
    BITMAP[bitmap_index].store(val & !(1u64 << bit), Ordering::Relaxed);
}

/// V6-XLAYER-003 fix: free_contig now RETURNS the number of pages it
/// actually freed (those that were in-use in the bitmap). Callers
/// (sys_munmap) refund quota based on this real count instead of the
/// user-supplied length. Without this, a cave could munmap a tiny
/// real region with a huge `length` and saturating-sub its memory
/// quota to zero.
///
/// Free a run of `count` contiguous physical pages starting at `base`.
/// Convenience wrapper over `free_frame`, used by the loader and munmap
/// paths that allocated large contiguous regions (e.g. 38k pages for a
/// Chromium-sized ELF). Silently ignores unaligned or out-of-range bases
/// so callers can blindly free "whatever I got from alloc".
pub fn free_contig(base: usize, count: usize) -> usize {
    if count == 0 { return 0; }
    let base = base & !(PAGE_SIZE - 1);
    let start = MEMORY_START.load(Ordering::Relaxed);
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let mut actually_freed = 0usize;
    for i in 0..count {
        let addr = base + i * PAGE_SIZE;
        // Was this frame actually in-use? Only count it if so, so the
        // caller's quota refund matches reality (V6-XLAYER-003).
        if addr >= start {
            let frame_index = (addr - start) / PAGE_SIZE;
            if frame_index < total {
                let bi = frame_index / 64;
                let bit = frame_index % 64;
                if bi < BITMAP_SIZE {
                    let val = BITMAP[bi].load(Ordering::Relaxed);
                    if val & (1u64 << bit) != 0 {
                        actually_freed += 1;
                    }
                }
            }
        }
        free_frame(addr);
    }
    actually_freed
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
