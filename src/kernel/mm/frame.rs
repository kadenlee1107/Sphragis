// Bat_OS — Page Frame Allocator
// Manages physical memory in 4KB pages using a bitmap.
// Simple, predictable, no external dependencies.

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub const PAGE_SIZE: usize = 4096;
// 🎯 bitmap now
// covers 4 GiB of frames so alloc_kernel_frame can find frames at the
// top of physical RAM. Was 524288 (= 2 GiB) — when QEMU_MEMORY_END
// got bumped to 4 GiB, the kernel-pool scan from total-1 downward
// always hit `bitmap_index >= BITMAP_SIZE` (skipped) → OOM on every
// alloc_kernel_frame call.
//
// 1 MiB frames = 4 GiB physical. Bitmap is 16384 u64s = 128 KiB —
// negligible static overhead.
const MAX_FRAMES: usize = 1024 * 1024;            // 4 GiB / 4 KiB
const BITMAP_SIZE: usize = MAX_FRAMES / 64;        // 16384 u64s = 128 KiB

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

    // skip the kernel-pool window so user pool can't poach
    // the kernel-reachable PAs that `alloc_kernel_frame` needs for L1/L2/
    // L3 page tables. Without this, deep Chromium runs (~50K demand-page
    // commits) would fill user-pool frames from index 0 upward, eventually
    // crossing `kern_cap_index` (~405K) and leaving zero free frames
    // below 0xC0000000 for kernel-pool to grab → `oom for L3 table`
    // crashes ~50K commits in.
    //
    // The kernel pool reserves the TOP `KERNEL_RESERVED_FRAMES` frames
    // of the [start, 0xC0000000) range. We compute that bound here and
    // make user_pool / alloc_contig skip frames in [kern_pool_lo,
    // kern_cap_index). They remain reachable by `alloc_kernel_frame`
    // via my top-down scan above.
    const KERNEL_FRAME_PA_CAP_LOCAL: usize = 0xC000_0000;
    let kern_cap_index = if start < KERNEL_FRAME_PA_CAP_LOCAL {
        ((KERNEL_FRAME_PA_CAP_LOCAL - start) / PAGE_SIZE).min(total)
    } else { 0 };
    let kern_pool_lo = kern_cap_index.saturating_sub(KERNEL_RESERVED_FRAMES);

    for i in 0..BITMAP_SIZE {
        // M4 / MMU-off: `compare_exchange*` lowers to LDXR/STXR which
        // never succeeds on Device-nGnRnE memory. We already hold
        // `IrqGuard` above and we're single-CPU during bring-up, so
        // plain load + check + store is exclusive. When SMP lands the
        // frame allocator will need a proper lock (or `+lse`) anyway.
        let val = BITMAP[i].load(Ordering::Acquire);
        if val == u64::MAX { continue; } // full word, try next

        // Find the lowest clear bit whose frame_index falls outside the
        // kernel-pool window [kern_pool_lo, kern_cap_index).
        let mut bit_opt: Option<usize> = None;
        for b in 0..64usize {
            if val & (1u64 << b) != 0 { continue; }
            let fi = i * 64 + b;
            if fi >= kern_pool_lo && fi < kern_cap_index { continue; }
            bit_opt = Some(b);
            break;
        }
        let bit = match bit_opt { Some(b) => b, None => continue };
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
            // dc civac to PoC. The caller may map this
            // frame into a different cave VA; any stale dirty cache
            // lines (from a previous EL1-side use of this PA) would
            // shadow our fresh zeros.
            let mut line = addr as u64;
            let end_line = line + PAGE_SIZE as u64;
            while line < end_line {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
            core::arch::asm!("dsb sy");
        }
        return Some(addr);
    }

    None
}

/// Kernel-reserved frames at the top of the memory range. Never returned
/// by `alloc_frame`, so cave user-window mappings cannot alias into them.
// /
/// HISTORY: was 512 (= 2 MB) sized for cave L1/L2 tables only. Once
/// demand_page::install_l3_mapping started lazily allocating L2/L3 tables
/// (Stump #3 + Stump #4 fixes), and Chromium spread its 30+ thread stacks
/// + many small_mmap regions across hundreds of distinct 2-MB pages,
/// 512 frames blew through ~2300 demand-page commits and threw
/// `oom for L3 table`.
// /
/// 4096 frames = 16 MB on a 4 GB system (0.4%). Plenty for any
/// reasonable cave + small_mmap workload, including 32 caves of
/// content_shell-class binaries with their full thread/cage layouts.
// bumped 4096 → 16384 → 32768 (16 MB → 64 MB → 128 MB)
// Even at 64 MB we OOM at ~22K-line runs ("install_l3 failed
// va=0x14e200000 reason: demand_page: oom for L3 table"). The
// deep runs map many L3 tables for V8 cages + content_shell text +
// scratch + Mojo IPC pages. 128 MB gives 32K L3 tables = 64 GB
// VA range, well over what V8 needs. We have 3.6 GB total RAM.
pub const KERNEL_RESERVED_FRAMES: usize = 32768;

/// V2-001/V2-040 fix: allocate a frame from the kernel-reserved pool.
/// Used by `setup_cave_pagetable` / `setup_cave_pagetable_at` so a cave's
/// own L1 / L2 tables can never be remapped into the cave's user window.
// /
/// Returns None if the kernel pool is exhausted; callers should surface
/// this as "too many caves" rather than falling back to alloc_frame.
pub fn alloc_kernel_frame() -> Option<usize> {
    // V8-ROOT-1: same CS discipline as alloc_frame.
    let _g = crate::kernel::sync::IrqGuard::new();

    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = MEMORY_START.load(Ordering::Relaxed);
    if total < 1 { return None; }

    // cap kernel frames at PA < 0xC0000000. The cave's
    // identity map only covers L2_high+L2_xhi (0x40000000–0xC0000000)
    // for kernel access; tables placed at higher PAs are direct-PA-
    // writable but the MMU walker can't reach them after MMU-enable
    // (observed: kernel hangs immediately after SCTLR.M=1 even though
    // a sentinel write+read at the same PA succeeds). Until we fix
    // the actual walker reachability problem, restrict the kernel pool
    // to PAs the walker can definitely reach. User pool stays at 4 GiB.
    const KERNEL_FRAME_PA_CAP: usize = 0xC000_0000;
    let cap_index = if start < KERNEL_FRAME_PA_CAP {
        ((KERNEL_FRAME_PA_CAP - start) / PAGE_SIZE).min(total)
    } else { 0 };
    if cap_index == 0 { return None; }
    let scan_top = cap_index;

    // scan the ENTIRE range below KERNEL_FRAME_PA_CAP
    // top-down rather than capping at `KERNEL_RESERVED_FRAMES` iterations.
    //
    // Symptom this fixes: deep Chromium runs (~20K lines) hit
    // "oom for L3 table" from `install_l3_mapping` even though we only
    // demand-paged ~107 L3 tables. The 32K-iteration cap meant any
    // fragmentation in the top 128 MB kernel-pool window (e.g. fork()'s
    // bulk-cloned L2/L3 tables, repeated cave setup_cave_pagetable
    // hitting the same upper frames) made `alloc_kernel_frame` return
    // None even though plenty of kernel-reachable frames remained free
    // below the window.
    //
    // The reserved-pool ROLE (don't let user pool starve kernel) is
    // still preserved by `user_cap = total - KERNEL_RESERVED_FRAMES`
    // in `alloc_frame`; we just relax kernel pool's *scan range* so it
    // can find any kernel-reachable free frame top-down.
    //
    // Word-level fast-skip on `u64::MAX` keeps the worst-case cost
    // bounded: 64 frames per non-MAX bitmap probe, one cheap read per
    // fully-saturated word.
    // Top word index (inclusive) — the word that contains frame
    // `scan_top - 1`. May be a partial word: any bit `bit` such that
    // `wi*64 + bit >= scan_top` is skipped.
    let top_word = scan_top.saturating_sub(1) / 64;
    let mut wi = top_word as isize;
    while wi >= 0 {
        let wu = wi as usize;
        if wu < BITMAP_SIZE {
            let val = BITMAP[wu].load(Ordering::Acquire);
            if val != u64::MAX {
                for bit in (0..64usize).rev() {
                    let frame_index = wu * 64 + bit;
                    if frame_index >= scan_top { continue; }
                    if val & (1u64 << bit) != 0 { continue; }
                    BITMAP[wu].store(val | (1u64 << bit), Ordering::Release);

                    let addr = start + frame_index * PAGE_SIZE;
                    unsafe {
                        for i in 0..(PAGE_SIZE / 8) {
                            core::arch::asm!("str xzr, [{a}]",
                                a = in(reg) addr + i * 8,
                                options(nostack, preserves_flags));
                        }
                        let mut line = addr as u64;
                        let end_line = line + PAGE_SIZE as u64;
                        while line < end_line {
                            core::arch::asm!("dc civac, {a}", a = in(reg) line);
                            line += 64;
                        }
                        core::arch::asm!("dsb sy");
                    }
                    return Some(addr);
                }
            }
        }
        wi -= 1;
    }
    None
}

/// Allocate `n_pages` physically-contiguous, zero-filled 4 KiB frames.
/// Returns the base address of the first frame, or None if no such
/// run exists.
// /
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

    // skip the kernel-pool window — see the comment in
    // `alloc_frame`. If a contiguous run would straddle the window,
    // bump `frame_idx` past the window and resume.
    const KERNEL_FRAME_PA_CAP_LOCAL: usize = 0xC000_0000;
    let kern_cap_index = if start < KERNEL_FRAME_PA_CAP_LOCAL {
        ((KERNEL_FRAME_PA_CAP_LOCAL - start) / PAGE_SIZE).min(total)
    } else { 0 };
    let kern_pool_lo = kern_cap_index.saturating_sub(KERNEL_RESERVED_FRAMES);

    // Scan bitmap for `n_pages` consecutive zero bits.
    let mut frame_idx: usize = 0;
    'outer: while frame_idx + n_pages <= user_cap {
        // If the proposed run [frame_idx, frame_idx + n_pages) overlaps
        // the kernel-pool window, jump frame_idx past the window.
        if frame_idx < kern_cap_index && frame_idx + n_pages > kern_pool_lo {
            frame_idx = kern_cap_index;
            continue 'outer;
        }
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
            // dc civac to PoC (see alloc_frame).
            let mut line = base as u64;
            let end_line = line + (n_pages * PAGE_SIZE) as u64;
            while line < end_line {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
            core::arch::asm!("dsb sy");
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
        // dc civac to PoC after zeroing on free. Same
        // root cause family as c demand_page fix: when this
        // frame is later reallocated to a NEW user VA via demand-page,
        // the user might read via that new VA and see stale (cached
        // dirty) data instead of the zeros we just wrote. PartitionAlloc
        // CHECK on InSlotMetadata fails → CorruptionDetected BRK.
        let mut line = addr as u64;
        let end_line = line + PAGE_SIZE as u64;
        while line < end_line {
            core::arch::asm!("dc civac, {a}", a = in(reg) line);
            line += 64;
        }
        core::arch::asm!("dsb sy");
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
// /
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
