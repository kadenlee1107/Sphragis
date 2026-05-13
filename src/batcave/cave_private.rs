//! Per-cave private memory regions.
//!
//! Allocates a kernel-pool frame and maps it at a per-cave VA in
//! ONLY the owning cave's L1 page table. The same VA is unmapped in
//! `PRIMARY_L1` (kernel-ns) and in every other cave's L1, so an
//! access to that VA from the wrong context faults at the MMU walker
//! — turning "accidentally readable" into a hardware fault.
//!
//! VA layout. Each cave's private region lives at a single fixed VA
//! that is OUTSIDE the kernel-identity range
//! `[0x40000000, 0x140000000)`. That keeps the VA from colliding
//! with kernel data the regular identity map already covers, AND
//! makes it trivial for the selftest to look up: the VA is constant
//! per cave_id, computed as `CAVE_PRIVATE_VA_BASE + cave_id * 0x1000`.
//!
//! Currently one 4 KiB page per cave. The keypair (32 bytes) + a
//! small peer table fit comfortably; a future arc can extend the
//! allocator to multi-page regions by adding more L3 entries.
//!
//! Slice 1 (this commit): allocator + selftest that proves the
//! cross-cave isolation property (VA is mapped in sys-wg's L1,
//! unmapped in PRIMARY_L1). No existing sys-wg state moves yet.
//!
//! Slice 2 (future): relocate sys-wg's static keypair and peer
//! table backing into a cave-private region.

#![allow(dead_code)]

use crate::batcave::{cave, linux::mmu};
use crate::kernel::mm::cave_pool;

/// Base VA for cave-private regions. Sits one full L1 entry (1 GiB)
/// above the top of the kernel identity range, so it never collides
/// with anything the standard kernel L1 maps. Each cave gets a 4 KiB
/// page at `CAVE_PRIVATE_VA_BASE + cave_id * 0x1000`.
pub const CAVE_PRIVATE_VA_BASE: usize = 0x1_4000_0000;

/// PTE flags for a cave-private 4 KiB page:
///   - PTE_VALID, PTE_AF (access flag pre-set so HW doesn't update it)
///   - PTE_SH_INNER (inner-shareable, matches the rest of our maps)
///   - PTE_ATTR_NORMAL (write-back cacheable via MAIR index 0)
///   - PTE_AP_EL1_RW (kernel-mode read/write; EL0 denied)
///   - PTE_UXN | PTE_PXN (no execute at either level — pure data)
const CAVE_PRIVATE_PTE_FLAGS: u64 = 0
    | (1 << 0)              // PTE_VALID
    | (1 << 10)             // PTE_AF
    | (3 << 8)              // SH_INNER
    | (0 << 2)              // MAIR idx 0 = normal WB cacheable
    | (0 << 6)              // AP[1:0] = 0,0 = EL1 RW, EL0 none
    | (1 << 53)             // PXN: no EL1 exec
    | (1 << 54);            // UXN: no EL0 exec

/// Per-cave allocation tracking. `Some(pa)` once `ensure_page` has
/// allocated + mapped the cave's private 4 KiB region. The PA is
/// retained so future reuse paths (e.g. teardown) can release it.
const MAX_CAVES_TRACKED: usize = 32;
static mut PRIVATE_PA: [Option<usize>; MAX_CAVES_TRACKED] = [None; MAX_CAVES_TRACKED];

/// Idempotent. On first call for a given `cave_id`: allocate a
/// kernel-pool frame, map it at `CAVE_PRIVATE_VA_BASE + cave_id *
/// 0x1000` in the cave's L1, record the PA in `PRIVATE_PA[cave_id]`.
/// Subsequent calls for the same `cave_id` return the same VA
/// without re-allocating.
///
/// Returns the VA on success; `None` if the cave has no L1, the
/// cave_id is out of range, or any allocation step fails.
pub fn ensure_page(cave_id: u16) -> Option<usize> {
    let idx = cave_id as usize;
    if idx >= MAX_CAVES_TRACKED { return None; }

    // Already mapped?
    let already = unsafe { (*core::ptr::addr_of!(PRIVATE_PA))[idx] };
    if already.is_some() {
        return Some(cave_private_va(cave_id));
    }

    let l1_phys = cave::get_cave_l1_phys(cave_id)?;

    // gap-audit 030 expansion: charge the cave's memory quota
    // BEFORE the cave_pool allocation. Cave-private is the only
    // remaining cave-attributable allocator that wasn't quota-
    // metered (shm / pipe / batfs already charge). Quota 0 means
    // unlimited, so default-quota caves are unaffected. On any
    // downstream failure below we release back.
    if cave::charge_pages_for(cave_id, 1).is_err() {
        return None;
    }

    // Allocate from the carved-out cave-pool — these PAs are NOT
    // covered by any cave L1's kernel-identity map, so even an
    // attacker who learns the PA can't reach the bytes through
    // PRIMARY_L1 or any other cave's L1. cave_pool::alloc_page
    // intentionally does NOT zero the page — kernel-ns can't reach
    // the PA, so any zero-store via the identity VA would
    // data-abort and the demand-pager would shadow the carve-out.
    // We zero through the cave-private VA below instead.
    let page_pa = match cave_pool::alloc_page() {
        Some(pa) => pa,
        None => {
            cave::release_pages_for(cave_id, 1);
            return None;
        }
    };

    let va = cave_private_va(cave_id);
    if mmu::map_4k_in_l1(l1_phys, va, page_pa, CAVE_PRIVATE_PTE_FLAGS).is_err() {
        // Couldn't install the mapping — leak the frame for now
        // (rare error path; cave_pool has no free entry point yet),
        // but DO release the quota charge so the cave isn't
        // penalised for an installer failure.
        cave::release_pages_for(cave_id, 1);
        return None;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(PRIVATE_PA))[idx] = Some(page_pa);
    }

    // Invalidate any stale TLB entry for this VA, then zero the page
    // through the just-installed cave-private mapping. Must run
    // under `with_cave_active(cave_id)` so the VA translates.
    unsafe {
        core::arch::asm!(
            "tlbi vae1, {a}",
            "dsb sy",
            "isb",
            a = in(reg) (va >> 12) as u64,
        );
    }
    cave::with_cave_active(cave_id, || {
        cave_pool::zero_via_cave_va(va);
    });

    Some(va)
}

/// VA computation. Public so callers can determine the address
/// without going through `ensure_page` (e.g. to walk page tables
/// in the selftest).
pub fn cave_private_va(cave_id: u16) -> usize {
    CAVE_PRIVATE_VA_BASE + (cave_id as usize) * 0x1000
}

/// True if `cave_id` has had its private page allocated. Lets the
/// selftest assert state without exposing the PA itself.
pub fn has_page(cave_id: u16) -> bool {
    let idx = cave_id as usize;
    if idx >= MAX_CAVES_TRACKED { return false; }
    unsafe { (*core::ptr::addr_of!(PRIVATE_PA))[idx].is_some() }
}
