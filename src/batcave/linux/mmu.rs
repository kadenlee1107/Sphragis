// Bat_OS — MMU Page Table Setup for BatCave Linux Binaries
// Creates a virtual address mapping so PIE binaries see their expected addresses.
//
// ARM64 4KB granule, 4-level page tables:
//   L0: 1 entry = 512GB
//   L1: 1 entry = 1GB
//   L2: 1 entry = 2MB (block mapping)
//   L3: 1 entry = 4KB
//
// We use 2MB block mappings for simplicity.

use crate::kernel::mm::frame;
use crate::drivers::uart;

// Linker-provided boundary between kernel .text and everything-else. Defined
// in `linker.ld` / `linker_apple.ld` as `__text_end`, 2 MB-aligned so a 2 MB
// block mapping can split here. Below this VA the kernel is executable+RO;
// above it, NX+RW.
unsafe extern "C" {
    static __text_end: u8;
}

#[inline]
fn text_end_addr() -> u64 {
    // SAFETY: `__text_end` is a symbol defined by the linker; we only take its
    // address, never dereference.
    (unsafe { core::ptr::addr_of!(__text_end) }) as u64
}

const PAGE_SIZE: usize = 4096;
const ENTRIES_PER_TABLE: usize = 512;

/// Cave user-window size in 2 MB blocks. 400 MB = 200 × 2 MB, large enough
/// for a real Chromium content_shell (~280 MB today) plus headroom for
/// future growth. Each cave maps `CAVE_BLOCKS` × 2 MB starting at its
/// `virt_base` (0x10000000 for Chromium, above MMIO at indices 64..82).
/// Also consulted by `loader::load_elf_rebased` via `CAVE_BLOCKS_PUB`.
pub const CAVE_BLOCKS: usize = 200;

// Page table entry flags
const PTE_VALID: u64 = 1;
const PTE_TABLE: u64 = 1 << 1;  // Points to next-level table
#[allow(dead_code)]
const PTE_BLOCK: u64 = 0;       // 2MB block (at L2) — implied by !PTE_TABLE
const PTE_AF: u64 = 1 << 10;    // Access flag
const PTE_SH_INNER: u64 = 3 << 8; // Inner shareable
const PTE_ATTR_NORMAL: u64 = 0 << 2; // MAIR index 0: normal memory
const PTE_ATTR_DEVICE: u64 = 1 << 2; // MAIR index 1: device memory

// Access-permission encodings (AP[2:1] at bits 7:6 of the block/page desc).
// AP[2] is bit 7 (0 = RW, 1 = RO); AP[1] is bit 6 (0 = EL1 only, 1 = EL0+EL1).
const PTE_AP_EL1_RW: u64 = 0 << 6;  // kernel-only, read-write
const PTE_AP_EL1_RO: u64 = 2 << 6;  // kernel-only, read-only
const PTE_AP_EL0_RW: u64 = 1 << 6;  // EL0+EL1, read-write
const PTE_AP_EL0_RO: u64 = 3 << 6;  // EL0+EL1, read-only
// Legacy alias so we don't break anywhere still reading it (none should remain).
#[allow(dead_code)]
const PTE_AP_RW: u64 = PTE_AP_EL1_RW;

// Execute-never bits (ARMv8 VMSA block/page descriptor).
// Both default to 0 (permissive) — that was ROOT-3 of the pentest summary.
const PTE_PXN: u64 = 1 << 53; // Privileged (EL1) execute-never
const PTE_UXN: u64 = 1 << 54; // Unprivileged (EL0) execute-never

// === W^X-aware block variants ===
//
// ROOT-3 fix: the previous single `BLOCK_NORMAL` had neither UXN nor PXN and
// used AP=RW, so every kernel page was writable AND executable from both EL1
// and EL0. Any arbitrary-write bug became trivial code-injection RCE.
//
// We now split by purpose. Each caller picks the variant matching the page's
// role; the MMU enforces W^X per-block from that point on.

/// Kernel .text: executable from EL1, never from EL0, read-only everywhere.
const BLOCK_KERNEL_TEXT: u64 =
    PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_EL1_RO | PTE_UXN;

/// Kernel .data/.bss/heap: no-exec anywhere, read-write from EL1 only.
const BLOCK_KERNEL_DATA: u64 =
    PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_EL1_RW | PTE_UXN | PTE_PXN;

/// User .text: executable from EL0, never from EL1, read-only.
#[allow(dead_code)]
const BLOCK_USER_TEXT: u64 =
    PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_EL0_RO | PTE_PXN;

/// User .data/.bss/stack: no-exec anywhere, read-write from EL0.
#[allow(dead_code)]
const BLOCK_USER_DATA: u64 =
    PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_EL0_RW | PTE_UXN | PTE_PXN;

/// Transitional: user page that is writable AND executable from EL0, no-exec
/// from EL1. TODO: remove once the ELF loader parses PT_LOAD PF_X/PF_W flags
/// and maps cave .text vs .data/.bss separately. Chromium/V8 needs JIT pages
/// (W+X) to run, so until we have per-segment perms this is the only workable
/// mapping for the whole cave window. W^X inside a cave is therefore NOT
/// enforced yet; kernel W^X (via BLOCK_KERNEL_TEXT/DATA above) IS enforced.
const BLOCK_USER_RW_EXEC: u64 =
    PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_EL0_RW | PTE_PXN;

// Back-compat: anything still referring to BLOCK_NORMAL should migrate to one
// of the specific variants above. Kept only so external debug tooling that
// pattern-matches the old constant name still compiles; intentionally gated
// to avoid any new uses being introduced silently.
#[allow(dead_code)]
const BLOCK_NORMAL: u64 = BLOCK_KERNEL_DATA;

// 2MB block entry flags for device memory (MMIO). PXN+UXN so an attacker who
// reaches MMIO via a bug can't execute from it.
const BLOCK_DEVICE: u64 =
    PTE_VALID | PTE_AF | PTE_ATTR_DEVICE | PTE_AP_EL1_RW | PTE_UXN | PTE_PXN;
// Table descriptor flags
const TABLE_DESC: u64 = PTE_VALID | PTE_TABLE;

// Per-BatCave page table registry
// Each cave gets its own L1 table, mapping only its own physical memory
const MAX_CAVE_PAGETABLES: usize = 8;
static mut CAVE_L1: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES]; // L1 table phys addr per cave
static mut CAVE_PHYS_BASE: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES]; // per-cave phys base
static mut PRIMARY_L1: usize = 0; // the primary (ash) page table

// V3 — per-cave user-window bounds, tracked so `is_user_range` can refuse
// pointers outside the actively-mounted cave's window (NEW-SYS-001 +
// related cross-cave pointer abuse).  Indexed by cave_slot.
static mut CAVE_VIRT_BASE: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES];
static mut CAVE_VIRT_EXTENT: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES];

// Currently-mounted cave window (start, end). Updated on every
// switch_to_cave / switch_to_primary. Sentinel (0, 0) means "no cave
// mounted; fall back to legacy 0x1000..0x4000_0000".
use core::sync::atomic::{AtomicUsize, Ordering as Ord2};
static ACTIVE_WIN_START: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_WIN_END:   AtomicUsize = AtomicUsize::new(0);

/// Returns the active cave's user-VA window as (start, end). When no cave
/// is mounted this returns (0, 0); callers should treat that as "use the
/// legacy default window" (0x1000..0x4000_0000).
///
/// V5-TOCTOU-001 fix: reads a single packed u64 atomic so readers never
/// see an inconsistent (start, end) pair split across two stores.
pub fn active_user_window() -> (usize, usize) {
    let packed = ACTIVE_WIN_PACKED.load(Ord2::Acquire);
    let start = (packed & 0xFFFF_FFFF) as usize;
    let end   = ((packed >> 32) & 0xFFFF_FFFF) as usize;
    (start << 12, end << 12)
}

// V5-TOCTOU-001: pack start/end (each page-aligned, so fits in 20 bits
// within a 4 GB address space) into a single 64-bit atomic. Shift-by-12
// drops the 12 low zero bits so we get ~32 bits of address range each.
// ACTIVE_WIN_START / ACTIVE_WIN_END kept for backwards-compat callers.
static ACTIVE_WIN_PACKED: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

/// V5-WEIRD-008: primary-path flag. Set by the primary busybox/ash
/// runner so is_user_range can use the legacy [0x1000, 0x4000_0000)
/// window. When no cave and no primary flag are set, is_user_range
/// fails closed.
static ACTIVE_IS_PRIMARY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

pub fn set_active_primary(v: bool) {
    ACTIVE_IS_PRIMARY.store(v, core::sync::atomic::Ordering::Release);
}
pub fn active_is_primary() -> bool {
    ACTIVE_IS_PRIMARY.load(core::sync::atomic::Ordering::Acquire)
}

// Cave-slot bitmap. Bit set = slot in use. CAS-free (we're single-core).
use core::sync::atomic::{AtomicU8, Ordering as SlotOrder};
static CAVE_SLOT_USED: AtomicU8 = AtomicU8::new(0);

/// Allocate a free cave page-table slot. Returns Some(index) or None if
/// all MAX_CAVE_PAGETABLES are in use.
///
/// V5-TOCTOU-011 fix: CAS-per-bit instead of load-then-fetch_or. The
/// old path could race: thread A loads used, sees bit i is 0; thread B
/// also loads used and sees bit i is 0; both fetch_or(bit i) and both
/// get Some(i), so two threads think they own cave slot i.
pub fn alloc_cave_slot() -> Option<usize> {
    for i in 0..MAX_CAVE_PAGETABLES {
        let bit = 1u8 << i;
        loop {
            let cur = CAVE_SLOT_USED.load(SlotOrder::Acquire);
            if cur & bit != 0 { break; } // slot taken, try next
            let new = cur | bit;
            if CAVE_SLOT_USED
                .compare_exchange(cur, new, SlotOrder::AcqRel, SlotOrder::Acquire)
                .is_ok()
            {
                return Some(i);
            }
            // CAS lost the race — re-read and retry on same slot.
        }
    }
    None
}

/// Release a cave slot. Also clears CAVE_L1[slot] so subsequent
/// `get_cave_l1(slot)` returns the primary table.
///
/// FLv2-NEW-018 fix: also releases the cave's L1 + L2_low + L2_high
/// kernel-pool frames back to `frame::alloc_kernel_frame`. Walks the L1
/// to find the L2_low / L2_high pointers before clearing CAVE_L1[slot].
/// Without this, every cave create→destroy cycle leaked 12 KB of
/// kernel-pool memory and the 512-frame reserved pool would exhaust
/// after ~40 caves.
pub fn free_cave_slot(slot: usize) {
    if slot >= MAX_CAVE_PAGETABLES { return; }
    // V5-CHAIN-002 fix: zero the per-slot state BEFORE releasing the
    // slot-used bit. Previously we cleared the used bit first, which
    // let a racing alloc_cave_slot grab the slot while CAVE_L1[slot]
    // still held the old tenant's L1. When this function then
    // zero-and-freed the L1 pages it clobbered the new tenant's page
    // tables, collapsing the sandbox to PRIMARY_L1.
    unsafe {
        let l1 = CAVE_L1[slot];
        if l1 != 0 {
            // L1[0] -> L2_low ; L1[1] -> L2_high (see setup_cave_pagetable_at).
            // Mask off the descriptor flags to recover the table address.
            let table_mask: u64 = !0xFFFu64; // page-aligned address
            let l2_low_pte: u64;
            let l2_high_pte: u64;
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) l1, v = out(reg) l2_low_pte);
            core::arch::asm!("ldr {v}, [{a}]", a = in(reg) l1 + 8, v = out(reg) l2_high_pte);
            let l2_low = (l2_low_pte & table_mask) as usize;
            let l2_high = (l2_high_pte & table_mask) as usize;
            // Zero the page-table memory before returning to the pool.
            for p in [l1, l2_low, l2_high] {
                if p != 0 {
                    for i in 0..(PAGE_SIZE / 8) {
                        core::arch::asm!("str xzr, [{a}]",
                            a = in(reg) p + i * 8,
                            options(nostack, preserves_flags));
                    }
                    crate::kernel::mm::frame::free_frame(p);
                }
            }
        }
        CAVE_L1[slot] = 0;
        CAVE_PHYS_BASE[slot] = 0;
        CAVE_VIRT_BASE[slot] = 0;
        CAVE_VIRT_EXTENT[slot] = 0;
    }
    // Release the slot-used bit LAST so a concurrent alloc_cave_slot
    // cannot claim this slot while the per-slot state is still being
    // reset. This is the V5-CHAIN-002 ordering fix.
    CAVE_SLOT_USED.fetch_and(!(1u8 << slot), SlotOrder::AcqRel);
}

/// Get the L1 table address for a specific cave.
pub fn get_cave_l1(cave_slot: usize) -> usize {
    unsafe { if cave_slot < MAX_CAVE_PAGETABLES { CAVE_L1[cave_slot] } else { PRIMARY_L1 } }
}

/// Set up a separate page table for a new BatCave.
/// The cave's page table maps ONLY:
///   - Its own busybox code/data (different phys_base from primary)
///   - MMIO (needed for I/O)
///   - Kernel RAM (identity, needed for syscall handling)
/// It does NOT map the primary busybox or other caves.
pub fn setup_cave_pagetable(cave_slot: usize, phys_base: usize) -> Result<usize, &'static str> {
    setup_cave_pagetable_at(cave_slot, phys_base, 0)
}

/// Same as `setup_cave_pagetable` but maps the cave's user window starting
/// at a configurable virtual address `virt_base`. ROOT-1 Option B: pass
/// `virt_base = 0x10000000` to place cave user code ABOVE MMIO so a
/// Chromium-sized (150 MB) binary doesn't collide with UART/virtio at
/// physical/virtual blocks 64/72/80/81/82.
///
/// Legacy callers pass `virt_base = 0` — MMIO overlap remains their
/// problem, but test binaries that never grow past 128 MB of VA don't
/// actually hit the hole.
pub fn setup_cave_pagetable_at(
    cave_slot: usize,
    phys_base: usize,
    virt_base: u64,
) -> Result<usize, &'static str> {
    if cave_slot >= MAX_CAVE_PAGETABLES { return Err("too many cave page tables"); }
    // virt_base must be 2 MB-aligned and below 1 GB (L2_low covers 0..0x3FFFFFFF).
    if virt_base & 0x1FFFFF != 0 { return Err("virt_base not 2MB aligned"); }
    if virt_base >= 0x40000000 { return Err("virt_base outside L2_low"); }

    // V2-001/V2-040 fix: cave L1/L2 frames come from the kernel-only
    // reserved pool. The old shared allocator could return a frame whose
    // address fell inside the cave's user window, letting the cave
    // self-remap to its own page table.
    let l1 = frame::alloc_kernel_frame().ok_or("oom for cave L1 (kernel pool)")?;
    let l2_low = frame::alloc_kernel_frame().ok_or("oom for cave L2_low (kernel pool)")?;
    let l2_high = frame::alloc_kernel_frame().ok_or("oom for cave L2_high (kernel pool)")?;
    // INITRD-FIX 2: same 2 GB identity widen as `setup_and_enable` — the
    // cave needs to reach its own phys_base (which for a 280 MB content_shell
    // can land past 0x50000000) via the kernel identity map while syscalls
    // run under the cave's TTBR0.
    let l2_xhi = frame::alloc_kernel_frame().ok_or("oom for cave L2_xhi (kernel pool)")?;
    // 🎯 STUMP #7 FIX: extend identity map to cover the full 4 GiB
    // physical RAM (paired with QEMU_MEMORY_END bump in mm/mod.rs).
    // L2_xxhi covers [0xC0000000, 0x100000000); L2_xxxhi covers
    // [0x100000000, 0x140000000). Without these, alloc_frame would
    // hand out frames in the new range but kernel writes to them
    // (demand_page::install, file copies, etc.) would DATA ABORT.
    let l2_xxhi = frame::alloc_kernel_frame().ok_or("oom for cave L2_xxhi (kernel pool)")?;
    let l2_xxxhi = frame::alloc_kernel_frame().ok_or("oom for cave L2_xxxhi (kernel pool)")?;

    // Zero tables
    for table in [l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
        for i in 0..(PAGE_SIZE / 8) {
            unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) table + i * 8); }
        }
    }

    // L1[0] → L2_low, L1[1] → L2_high, L1[2] → L2_xhi
    // L1[3] → L2_xxhi (kernel identity 0xC0000000-0x100000000)
    // L1[4] → L2_xxxhi (kernel identity 0x100000000-0x140000000)
    write_pte(l1, 0, l2_low as u64 | TABLE_DESC);
    write_pte(l1, 1, l2_high as u64 | TABLE_DESC);
    write_pte(l1, 2, l2_xhi as u64 | TABLE_DESC);
    write_pte(l1, 3, l2_xxhi as u64 | TABLE_DESC);
    write_pte(l1, 4, l2_xxxhi as u64 | TABLE_DESC);

    // L2_low: map THIS cave's user-space binary (400 MB window starting
    // at `virt_base`).  With `virt_base = 0x10000000` the user blocks
    // live at L2 indices 128..327 — above MMIO (indices 64/72/80/81/82),
    // so real content_shell (280 MB today, may grow) fits with headroom.
    //
    // W^X CAVEAT (ROOT-3): we still use BLOCK_USER_RW_EXEC because
    // Chromium/V8 JIT needs W+X pages. Per-PT_LOAD PF_X/PF_W permissions
    // are future work; kernel-side W^X is unaffected.
    let virt_base_block = (virt_base / 0x200000) as usize;
    for i in 0..CAVE_BLOCKS {
        let block_idx = virt_base_block + i;
        if block_idx >= 512 { break; }
        let phys_block = (phys_base & !0x1FFFFF) + i * 0x200000;
        write_pte(l2_low, block_idx, phys_block as u64 | BLOCK_USER_RW_EXEC);
    }

    // 🎯 STUMP #3 ROOT CAUSE FIX (PartitionAlloc CorruptionDetected):
    //
    // The cave's user-VA window above maps phys_base..phys_base+400 MB
    // into virt_base..virt_base+400 MB via L2 BLOCK descriptors. But the
    // loader only RESERVES (in the frame bitmap) the actual loaded ELF +
    // stacks portion (~188 MB for content_shell). The remaining ~212 MB
    // of physical frames were:
    //   1. Mapped into the cave's user VA window (writable from EL0 +
    //      kernel via the L2 BLOCK PTEs we just wrote).
    //   2. Marked FREE in the frame bitmap.
    //
    // Any later alloc_frame() (sys_brk worker, demand_page::try_handle,
    // sys_mmap anon contig, anything else) would scan the bitmap from
    // low → high, hand out a frame inside that aliased range, and
    // install it at a SECOND virtual address (PartitionAlloc super-
    // page region 0x2c..., thread stack region 0x70..., etc.).
    //
    // Two VAs → same physical frame. Content_shell writes its own
    // .data/.bss via the low VA (0x1xxxxxxx, inside the cave window);
    // PartitionAlloc reads its metadata via the high VA → sees
    // content_shell's bytes (often NULL pointers in BSS) instead of
    // the bucket pointer it wrote → CorruptionDetected() BRK.
    //
    // The fix: reserve every page in the cave's mapped 400-MB window
    // BEFORE alloc_frame can scan the bitmap. Idempotent w.r.t. the
    // loader's own reservations of the loaded portion (reserve_range
    // sets bits unconditionally).
    frame::reserve_range(phys_base, phys_base + CAVE_BLOCKS * 0x200000);

    // MMIO identity mappings — EL1 can read/write MMIO during syscall
    // handling while the cave's TTBR0 is active; EL0 gets a fault (AP
    // forbids EL0 on BLOCK_DEVICE).  Required so the exception handler
    // can still reach UART / virtio while a cave is mounted.
    //
    // If the cave's user window and MMIO addresses overlap (legacy
    // virt_base=0 case with user blocks 0..99 and MMIO at 64/72/80/
    // 81/82) the MMIO mapping wins — a binary larger than 128 MB
    // would see device memory where code was expected. Callers with
    // large binaries must pass virt_base=0x10000000 (block 128+).
    for mmio in [0x08000000, 0x09000000, 0x0A000000, 0x0A200000, 0x0A400000] {
        write_pte(l2_low, mmio / 0x200000, mmio as u64 | BLOCK_DEVICE);
    }

    // L2_high + L2_xhi + L2_xxhi + L2_xxxhi: identity map the full
    // 4 GiB kernel RAM (mirror `setup_and_enable`). Transitional widen
    // past __text_end stays in place until .text.cold.* actually lands
    // inside the .text segment. The two HIGHER tables (L2_xxhi for
    // [0xC0000000, 0x100000000), L2_xxxhi for [0x100000000, 0x140000000))
    // never contain text, so they always get the EL1-RW + UXN flags.
    let text_end = text_end_addr();
    let kblk = |addr: u64| -> u64 {
        if addr + 0x200000 <= text_end {
            BLOCK_KERNEL_TEXT
        } else {
            PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL
                | PTE_AP_EL1_RW | PTE_UXN
        }
    };
    for block in 0..512 {
        let addr = 0x40000000u64 + (block as u64) * 0x200000;
        write_pte(l2_high, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0x80000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xhi, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0xC0000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxhi, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0x100000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxxhi, block, addr | kblk(addr));
    }

    // 🎯 STUMP #7: clean every PT page we just wrote to PoC so that
    // when the walker activates (after switch_to_cave) it sees the
    // entries instead of stale zeros. Without this, accesses through
    // any of the new tables fault L2-translation despite the entries
    // being correctly written. (Mirrors the cache-flush we do in
    // setup_and_enable.)
    unsafe {
        for pt in [l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
            let base = pt as u64;
            let mut line = base;
            while line < base + PAGE_SIZE as u64 {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
        }
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");
    }

    unsafe {
        CAVE_L1[cave_slot] = l1;
        CAVE_PHYS_BASE[cave_slot] = phys_base;
        // V3 — record per-cave window so is_user_range can refuse pointers
        // outside the cave that's actually mounted (vs. the legacy coarse
        // 0x1000..0x4000_0000 window).
        CAVE_VIRT_BASE[cave_slot] = virt_base as usize;
        // Match the CAVE_BLOCKS loop above (400 MB by default).
        CAVE_VIRT_EXTENT[cave_slot] = CAVE_BLOCKS * 0x200000;
    }

    Ok(l1)
}

/// STUMP #145: build a per-cave L1 for a NATIVE BatCave (kernel-only,
/// no user window, no EL0 code).
///
/// `setup_cave_pagetable_at` above is built for the Linux ELF runner —
/// it maps a 400 MB EL0-RW user window in L2_low for the loaded ELF.
/// Native caves don't load ELFs; they're tagged kernel-side workloads.
/// We still want each native cave to have its own L1 so:
///
///   1. The audit's "memory isolation is fiction for native caves"
///      verdict closes — every native cave gets a distinct TTBR0.
///   2. TLB entries don't bleed between caves (every cave-switch
///      flushes via `tlbi vmalle1`).
///   3. When future work loads cave-specific code/data into native
///      caves, the L1 is already plumbed.
///
/// Layout (kernel-only):
///
///   L1[0] → L2_low  (MMIO entries only, no user blocks)
///   L1[1] → L2_high (kernel identity 0x40000000..0x80000000)
///   L1[2] → L2_xhi  (kernel identity 0x80000000..0xC0000000)
///   L1[3] → L2_xxhi (kernel identity 0xC0000000..0x100000000)
///   L1[4] → L2_xxxhi(kernel identity 0x100000000..0x140000000)
///
/// MMIO must live in L2_low because UART (0x08000000) + virtio
/// (0x0A000000..) are below 1 GB. Without this, a kernel UART write
/// while a cave is active translation-faults. Existing
/// setup_cave_pagetable_at handles MMIO the same way.
///
/// The user window (0..0x40000000 except MMIO) stays unmapped, so
/// any EL0 access from this cave faults. That's the desired posture
/// for a native cave: there's no EL0 code today, but if anything
/// tries to drop to EL0 by mistake, the MMU stops it.
pub fn setup_native_cave_l1(cave_slot: usize) -> Result<usize, &'static str> {
    if cave_slot >= MAX_CAVE_PAGETABLES {
        return Err("too many cave page tables");
    }

    // V2-001/V2-040: page-table frames come from the kernel-only pool
    // so a future user mapping can never alias them.
    let l1     = frame::alloc_kernel_frame().ok_or("oom for native cave L1")?;
    let l2_low = frame::alloc_kernel_frame().ok_or("oom for native cave L2_low")?;
    let l2_high  = frame::alloc_kernel_frame().ok_or("oom for native cave L2_high")?;
    let l2_xhi   = frame::alloc_kernel_frame().ok_or("oom for native cave L2_xhi")?;
    let l2_xxhi  = frame::alloc_kernel_frame().ok_or("oom for native cave L2_xxhi")?;
    let l2_xxxhi = frame::alloc_kernel_frame().ok_or("oom for native cave L2_xxxhi")?;

    // Zero every table.
    for table in [l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
        for i in 0..(PAGE_SIZE / 8) {
            unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) table + i * 8); }
        }
    }

    // L1 entries.
    write_pte(l1, 0, l2_low   as u64 | TABLE_DESC);
    write_pte(l1, 1, l2_high  as u64 | TABLE_DESC);
    write_pte(l1, 2, l2_xhi   as u64 | TABLE_DESC);
    write_pte(l1, 3, l2_xxhi  as u64 | TABLE_DESC);
    write_pte(l1, 4, l2_xxxhi as u64 | TABLE_DESC);

    // MMIO into L2_low (matches setup_cave_pagetable_at's MMIO list).
    // EL1 can read/write; EL0 faults (BLOCK_DEVICE has no EL0 AP bit).
    for mmio in [0x08000000, 0x09000000, 0x0A000000, 0x0A200000, 0x0A400000] {
        write_pte(l2_low, mmio / 0x200000, mmio as u64 | BLOCK_DEVICE);
    }

    // Kernel identity into L2_high..L2_xxxhi (mirror of setup_cave_pagetable_at).
    let text_end = text_end_addr();
    let kblk = |addr: u64| -> u64 {
        if addr + 0x200000 <= text_end {
            BLOCK_KERNEL_TEXT
        } else {
            PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL
                | PTE_AP_EL1_RW | PTE_UXN
        }
    };
    for block in 0..512 {
        let addr = 0x40000000u64 + (block as u64) * 0x200000;
        write_pte(l2_high, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0x80000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xhi, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0xC0000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxhi, block, addr | kblk(addr));
    }
    for block in 0..512 {
        let addr = 0x100000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxxhi, block, addr | kblk(addr));
    }

    // STUMP #7 cache-flush — ensure the MMU walker sees what we wrote.
    unsafe {
        for pt in [l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
            let base = pt as u64;
            let mut line = base;
            while line < base + PAGE_SIZE as u64 {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
        }
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");
    }

    unsafe {
        CAVE_L1[cave_slot] = l1;
        CAVE_PHYS_BASE[cave_slot] = 0;        // no user phys window
        CAVE_VIRT_BASE[cave_slot] = 0;        // no user virt window
        CAVE_VIRT_EXTENT[cave_slot] = 0;
    }

    Ok(l1)
}

/// STUMP #145: try to find a free CAVE_L1 slot for a native cave.
/// Returns None if all MAX_CAVE_PAGETABLES are in use. Pairs with
/// `setup_native_cave_l1` — caller allocates the slot, then calls
/// the setup helper with the slot number.
pub fn alloc_native_cave_slot() -> Option<usize> {
    unsafe {
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Returns the L1 phys of the FIRST registered cave (slot 0) —
/// what the runner set up at cave creation time. Used by the
/// arch exit handler to tell "this is the main process exiting,
/// tear down the whole cave and go to desktop" from "this is a
/// forked child exiting, just unwind the thread".
pub fn host_cave_l1() -> usize {
    unsafe {
        if MAX_CAVE_PAGETABLES == 0 { return 0; }
        CAVE_L1[0]
    }
}

/// Find the cave slot whose L1 matches the active TTBR0. Used by
/// per-cave structures (FD table, signal handlers, …) so they can
/// look up the right per-process state without every caller
/// having to plumb cave_id through.
///
/// Returns 0 (the host cave) if no cave matches — that's the
/// pre-fork single-cave path and the safe default for any thread
/// that started before the fork machinery exists.
pub fn current_cave_slot() -> usize {
    let active_l1: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) active_l1); }
    let active_l1 = (active_l1 & !1u64) as usize;
    unsafe {
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == active_l1 && active_l1 != 0 {
                return i;
            }
        }
    }
    0
}

/// Maximum number of cave page tables we support. Exposed so
/// per-cave bookkeeping (fd tables, etc.) can size their arrays
/// to match.
pub const NUM_CAVES: usize = MAX_CAVE_PAGETABLES;

/// Look up the cave slot for a given L1 phys address. Like
/// `current_cave_slot()` but for an arbitrary L1 (used by wait4
/// to figure out which cave to free when reaping a forked child).
pub fn cave_slot_for_l1(l1_phys: u64) -> Option<usize> {
    if l1_phys == 0 { return None; }
    unsafe {
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == l1_phys as usize {
                return Some(i);
            }
        }
    }
    None
}

/// Look up the user-window bounds (virt_base, virt_extent) for
/// the given L1 phys address. Returns None if no cave is
/// registered with that L1.
pub fn cave_bounds_for_l1(l1_phys: u64) -> Option<(u64, u64)> {
    unsafe {
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == l1_phys as usize && l1_phys != 0 {
                return Some((
                    CAVE_VIRT_BASE[i] as u64,
                    CAVE_VIRT_EXTENT[i] as u64,
                ));
            }
        }
    }
    None
}

/// Look up the physical base of the given L1's cave.
pub fn cave_phys_base_for_l1(l1_phys: u64) -> Option<usize> {
    unsafe {
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == l1_phys as usize && l1_phys != 0 {
                return Some(CAVE_PHYS_BASE[i]);
            }
        }
    }
    None
}

/// Real-fork (eager-copy) page table duplication.
///
/// Allocates a fresh L1 + L2 set for the child cave and copies the
/// parent's user-window mappings into it page-by-page. Kernel
/// mappings (L2_high, L2_xhi) and MMIO entries are shared verbatim
/// — they're identical for every cave anyway. User pages get fresh
/// physical frames + memcpy of the parent's data so post-fork
/// writes from one process don't bleed into the other.
///
/// This is the one-shot "real fork" primitive that gives us proper
/// process semantics (separate address spaces). It costs ~1 frame
/// allocation + page copy per touched user page, and a 2 MB
/// contiguous alloc + memcpy per pre-mapped block. Slow at fork
/// time (~50-200 ms for a 100 MB Chromium cave) but correct, and
/// happens at most a handful of times per browser launch.
///
/// Caller must register the returned L1 + per-cave bookkeeping
/// via `record_forked_cave` (below) so `switch_to_cave(child_l1)`
/// finds the right user-window bounds when the child is scheduled.
pub fn fork_cave_pagetable(
    parent_l1: usize,
    virt_base: u64,
    virt_extent: u64,
) -> Result<usize, &'static str> {
    use crate::drivers::uart;
    use crate::kernel::mm::frame;

    // 1. Allocate child page tables.
    let child_l1      = frame::alloc_kernel_frame().ok_or("fork: oom L1")?;
    let child_l2_low  = frame::alloc_kernel_frame().ok_or("fork: oom L2_low")?;
    let child_l2_high = frame::alloc_kernel_frame().ok_or("fork: oom L2_high")?;
    let child_l2_xhi  = frame::alloc_kernel_frame().ok_or("fork: oom L2_xhi")?;
    // 🎯 STUMP #7: kernel identity-map extended into L1[3]+L1[4].
    let child_l2_xxhi  = frame::alloc_kernel_frame().ok_or("fork: oom L2_xxhi")?;
    let child_l2_xxxhi = frame::alloc_kernel_frame().ok_or("fork: oom L2_xxxhi")?;
    for t in [child_l1, child_l2_low, child_l2_high, child_l2_xhi, child_l2_xxhi, child_l2_xxxhi] {
        unsafe {
            for i in 0..(PAGE_SIZE / 8) {
                core::arch::asm!("str xzr, [{a}]",
                    a = in(reg) t + i * 8);
            }
        }
    }

    // 2. L1 entries: child's L2s for the standard 5 kernel-identity slots.
    write_pte(child_l1, 0, child_l2_low   as u64 | TABLE_DESC);
    write_pte(child_l1, 1, child_l2_high  as u64 | TABLE_DESC);
    write_pte(child_l1, 2, child_l2_xhi   as u64 | TABLE_DESC);
    write_pte(child_l1, 3, child_l2_xxhi  as u64 | TABLE_DESC);
    write_pte(child_l1, 4, child_l2_xxxhi as u64 | TABLE_DESC);

    // 3. L2_high + L2_xhi + L2_xxhi + L2_xxxhi: copy verbatim
    //    (kernel identity mappings, same for every cave; never
    //    written to from EL0).
    let parent_l2_high  = unsafe {
        core::ptr::read_volatile((parent_l1 + 8)  as *const u64)
    } & 0x0000_FFFF_FFFF_F000;
    let parent_l2_xhi   = unsafe {
        core::ptr::read_volatile((parent_l1 + 16) as *const u64)
    } & 0x0000_FFFF_FFFF_F000;
    let parent_l2_xxhi  = unsafe {
        core::ptr::read_volatile((parent_l1 + 24) as *const u64)
    } & 0x0000_FFFF_FFFF_F000;
    let parent_l2_xxxhi = unsafe {
        core::ptr::read_volatile((parent_l1 + 32) as *const u64)
    } & 0x0000_FFFF_FFFF_F000;
    for i in 0..512 {
        let pte = unsafe {
            core::ptr::read_volatile((parent_l2_high + (i * 8) as u64) as *const u64)
        };
        write_pte(child_l2_high, i, pte);
    }
    for i in 0..512 {
        let pte = unsafe {
            core::ptr::read_volatile((parent_l2_xhi + (i * 8) as u64) as *const u64)
        };
        write_pte(child_l2_xhi, i, pte);
    }
    for i in 0..512 {
        let pte = unsafe {
            core::ptr::read_volatile((parent_l2_xxhi + (i * 8) as u64) as *const u64)
        };
        write_pte(child_l2_xxhi, i, pte);
    }
    for i in 0..512 {
        let pte = unsafe {
            core::ptr::read_volatile((parent_l2_xxxhi + (i * 8) as u64) as *const u64)
        };
        write_pte(child_l2_xxxhi, i, pte);
    }

    // 4. L1[5..512]: V8 cage and other high-VA user mappings live
    //    here (e.g. L1 idx 0x60 for VA 0x18_0000_0000). Demand
    //    paging populates them lazily as the parent touches new
    //    addresses. Walk each L1 entry; if it points to a user
    //    L2, deep-copy that L2's L3 pages (cage allocations are
    //    page-granular L3 entries, never 2 MB blocks).
    //
    //    🎯 STUMP #7: changed lower bound from 3 to 5 because L1[3]+L1[4]
    //    are now kernel identity mappings (handled verbatim above).
    for l1_idx in 5..512usize {
        let parent_l1_pte = unsafe {
            core::ptr::read_volatile(
                (parent_l1 + (l1_idx * 8)) as *const u64
            )
        };
        if parent_l1_pte & PTE_VALID == 0 { continue; }
        // Allocate fresh L2 for the child at this slot.
        let child_l2 = frame::alloc_kernel_frame()
            .ok_or("fork: oom L2 (high)")? as u64;
        unsafe {
            for k in 0..(PAGE_SIZE / 8) {
                core::arch::asm!("str xzr, [{a}]",
                    a = in(reg) child_l2 as usize + k * 8);
            }
        }
        let parent_l2 = parent_l1_pte & 0x0000_FFFF_FFFF_F000;
        // Walk parent's L2 entries.
        for l2_idx in 0..512usize {
            let l2_pte = unsafe {
                core::ptr::read_volatile(
                    (parent_l2 + (l2_idx * 8) as u64) as *const u64
                )
            };
            if l2_pte & PTE_VALID == 0 { continue; }
            // Should always be TABLE descriptor (cage uses L3 entries),
            // but handle BLOCK defensively by copying it as a block-
            // split (same as L2_low BLOCK path).
            if (l2_pte & 0b11) != TABLE_DESC { continue; }
            let parent_l3 = l2_pte & 0x0000_FFFF_FFFF_F000;
            let child_l3 = frame::alloc_kernel_frame()
                .ok_or("fork: oom L3 (high)")? as u64;
            unsafe {
                for k in 0..(PAGE_SIZE / 8) {
                    core::arch::asm!("str xzr, [{a}]",
                        a = in(reg) child_l3 as usize + k * 8);
                }
            }
            for l3_idx in 0..512usize {
                let l3_pte = unsafe {
                    core::ptr::read_volatile(
                        (parent_l3 + (l3_idx * 8) as u64) as *const u64
                    )
                };
                if l3_pte & PTE_VALID == 0 { continue; }
                let parent_page = l3_pte & 0x0000_FFFF_FFFF_F000;
                let child_page = match frame::alloc_frame() {
                    Some(p) => p as u64,
                    None => return Err("fork: oom L3 leaf (high)"),
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        parent_page as *const u8,
                        child_page as *mut u8,
                        PAGE_SIZE,
                    );
                }
                let flag_mask = l3_pte & !0x0000_FFFF_FFFF_F000;
                write_pte(child_l3 as usize, l3_idx, child_page | flag_mask);
            }
            write_pte(child_l2 as usize, l2_idx, child_l3 | TABLE_DESC);
        }
        write_pte(child_l1, l1_idx, child_l2 | TABLE_DESC);
    }

    // 5. L2_low: walk and fork.
    let parent_l2_low = unsafe {
        core::ptr::read_volatile(parent_l1 as *const u64)
    } & 0x0000_FFFF_FFFF_F000;
    let virt_base_block = (virt_base / 0x200000) as usize;
    let virt_end_block  = ((virt_base + virt_extent + 0x1FFFFF) / 0x200000) as usize;

    let mut blocks_copied: usize = 0;
    let mut l3_pages_copied: usize = 0;

    for i in 0..512usize {
        let parent_pte = unsafe {
            core::ptr::read_volatile(
                (parent_l2_low + (i * 8) as u64) as *const u64
            )
        };
        if parent_pte & PTE_VALID == 0 { continue; }

        let in_user = i >= virt_base_block && i < virt_end_block;
        if !in_user {
            // MMIO or unused — share verbatim. (MMIO is device memory
            // that both processes can/should access identically.)
            write_pte(child_l2_low, i, parent_pte);
            continue;
        }

        let is_table = (parent_pte & 0b11) == TABLE_DESC;
        if !is_table {
            // 2 MB BLOCK descriptor in parent. We CANNOT do the
            // same in the child because `frame::alloc_contig(512)`
            // returns 4 KB-aligned (not 2 MB-aligned) memory, and
            // writing a non-2MB-aligned address into a BLOCK
            // descriptor truncates the low bits — child reads
            // wrong data, fetches garbage instructions, faults.
            //
            // Instead, break the 2 MB block into 512 individual
            // 4 KB L3 entries on the child side. Each L3 leaf is
            // a fresh 4 KB frame with parent's data memcpy'd in.
            // The PTE permission/attribute bits are preserved
            // from the parent block.
            let parent_phys = parent_pte & 0x0000_FFFF_FFE0_0000;
            let flag_mask = parent_pte & !0x0000_FFFF_FFE0_0000;
            // L3 entries on AArch64 *require* PTE_TABLE bit set
            // (= 0b11 in low bits) to mark them as PAGE descriptors
            // even though they're leaves. The block desc didn't
            // have PTE_TABLE; OR it in for the L3 leaves.
            let l3_leaf_flags = flag_mask | PTE_TABLE;
            // Allocate child's L3 table.
            let child_l3 = frame::alloc_kernel_frame()
                .ok_or("fork: oom L3 (block-split)")? as u64;
            unsafe {
                for k in 0..(PAGE_SIZE / 8) {
                    core::arch::asm!("str xzr, [{a}]",
                        a = in(reg) child_l3 as usize + k * 8);
                }
            }
            // 512 4KB frames + memcpy each.
            for j in 0..512usize {
                let src_phys = parent_phys + (j * PAGE_SIZE) as u64;
                let dst_phys = match frame::alloc_frame() {
                    Some(p) => p as u64,
                    None => {
                        uart::puts("[fork] oom on L3 leaf during block split\n");
                        return Err("fork: oom L3 leaf");
                    }
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        src_phys as *const u8,
                        dst_phys as *mut u8,
                        PAGE_SIZE,
                    );
                }
                write_pte(child_l3 as usize, j, dst_phys | l3_leaf_flags);
                l3_pages_copied += 1;
            }
            // Install L3 table in child's L2_low.
            write_pte(child_l2_low, i, child_l3 | TABLE_DESC);
            blocks_copied += 1;
        } else {
            // TABLE descriptor → walk L3, copy each leaf page.
            let parent_l3 = parent_pte & 0x0000_FFFF_FFFF_F000;
            let child_l3 = frame::alloc_kernel_frame().ok_or("fork: oom L3")? as u64;
            unsafe {
                for k in 0..(PAGE_SIZE / 8) {
                    core::arch::asm!("str xzr, [{a}]",
                        a = in(reg) child_l3 as usize + k * 8);
                }
            }
            for j in 0..512usize {
                let l3_pte = unsafe {
                    core::ptr::read_volatile(
                        (parent_l3 + (j * 8) as u64) as *const u64
                    )
                };
                if l3_pte & PTE_VALID == 0 { continue; }
                let parent_page = l3_pte & 0x0000_FFFF_FFFF_F000;
                let child_page = match frame::alloc_frame() {
                    Some(p) => p as u64,
                    None => {
                        uart::puts("[fork] oom on L3 leaf page, aborting\n");
                        return Err("fork: oom L3 page");
                    }
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        parent_page as *const u8,
                        child_page as *mut u8,
                        PAGE_SIZE,
                    );
                }
                let flag_mask = l3_pte & !0x0000_FFFF_FFFF_F000;
                write_pte(child_l3 as usize, j, child_page | flag_mask);
                l3_pages_copied += 1;
            }
            write_pte(child_l2_low, i, child_l3 | TABLE_DESC);
        }
    }

    uart::puts("[fork] pagetable cloned: ");
    crate::kernel::mm::print_num(blocks_copied);
    uart::puts(" blocks (2MB) + ");
    crate::kernel::mm::print_num(l3_pages_copied);
    uart::puts(" L3 pages (4KB)\n");

    Ok(child_l1)
}

/// Register a forked cave's page table + user-window bounds in
/// the per-slot tables so `switch_to_cave(child_l1)` finds the
/// right `is_user_range` bounds when scheduling the child. Uses
/// the same CAVE_L1 / CAVE_PHYS_BASE / CAVE_VIRT_BASE arrays as
/// `setup_cave_pagetable_at` so one set of bookkeeping covers
/// both the initial cave creation and post-fork children.
pub fn record_forked_cave(
    cave_slot: usize,
    l1_phys: u64,
    parent_phys_base: usize,
    virt_base: u64,
    virt_extent: u64,
) -> Result<(), &'static str> {
    if cave_slot >= MAX_CAVE_PAGETABLES {
        return Err("record_forked_cave: slot oob");
    }
    unsafe {
        CAVE_L1[cave_slot] = l1_phys as usize;
        CAVE_PHYS_BASE[cave_slot] = parent_phys_base;
        CAVE_VIRT_BASE[cave_slot] = virt_base as usize;
        CAVE_VIRT_EXTENT[cave_slot] = virt_extent as usize;
    }
    Ok(())
}

/// Switch MMU to a cave's page table.
pub fn switch_to_cave(l1_addr: usize) {
    unsafe {
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");
        core::arch::asm!("msr ttbr0_el1, {}", in(reg) l1_addr as u64);
        core::arch::asm!("isb");
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");

        // V3: publish active user window for is_user_range. Look up by L1.
        let mut start = 0usize;
        let mut extent = 0usize;
        for i in 0..MAX_CAVE_PAGETABLES {
            if CAVE_L1[i] == l1_addr && l1_addr != 0 {
                start = CAVE_VIRT_BASE[i];
                extent = CAVE_VIRT_EXTENT[i];
                break;
            }
        }
        if start != 0 && extent != 0 {
            // V5-TOCTOU-001: single-store of the packed (start, end).
            let end = start.saturating_add(extent);
            let packed = ((end as u64 >> 12) << 32) | ((start as u64 >> 12) & 0xFFFF_FFFF);
            ACTIVE_WIN_PACKED.store(packed, Ord2::Release);
            // Legacy split atomics kept so any stray reader also sees the new value.
            ACTIVE_WIN_START.store(start, Ord2::Release);
            ACTIVE_WIN_END.store(end, Ord2::Release);
            ACTIVE_IS_PRIMARY.store(false, Ord2::Release);
        } else {
            ACTIVE_WIN_PACKED.store(0, Ord2::Release);
            ACTIVE_WIN_START.store(0, Ord2::Release);
            ACTIVE_WIN_END.store(0, Ord2::Release);
            // This switch targeted the primary L1 (no cave slot match).
            // Let is_user_range use the legacy window in that case.
            ACTIVE_IS_PRIMARY.store(l1_addr == PRIMARY_L1 && PRIMARY_L1 != 0, Ord2::Release);
        }
    }
}

/// Switch back to the primary page table.
pub fn switch_to_primary() {
    unsafe {
        if PRIMARY_L1 != 0 {
            switch_to_cave(PRIMARY_L1);
        }
    }
}

/// Set up page tables and enable MMU.
/// Maps:
///   0x00000000 - 0x001FFFFF → phys_base (busybox code, 2MB block)
///   0x08000000 - 0x0BFFFFFF → identity (MMIO: UART, virtio)
///   0x40000000 - 0x4FFFFFFF → identity (kernel RAM, 256MB)
///
/// **Idempotent** (V2-NEW-026 fix): if the MMU is already enabled (SCTLR.M==1),
/// this function is a no-op. The previous behavior re-allocated all tables and
/// reset `TTBR0_EL1 = PRIMARY_L1` unconditionally; when called from the cave
/// path (after `switch_to_cave` had already loaded the cave's L1), it silently
/// reverted TTBR0 to the primary table, dissolving the sandbox mid-startup.
pub fn setup_and_enable(phys_base: usize) -> Result<(), &'static str> {
    // V2-NEW-026: bail out if MMU is already enabled. The cave path calls
    // `switch_to_cave(cave_l1)` before `execute_with_args`, which used to
    // re-invoke setup_and_enable and clobber TTBR0 back to PRIMARY_L1.
    unsafe {
        let sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        if sctlr & 1 != 0 {
            uart::puts("[mmu] already enabled, skipping setup (preserving TTBR0)\n");
            return Ok(());
        }
    }

    uart::puts("[mmu] Setting up page tables...\n");

    // V2-001: primary page-table frames also come from the kernel pool so
    // no user-mappable allocation can alias into them.
    let l0 = frame::alloc_kernel_frame().ok_or("oom for primary L0")?;
    let l1 = frame::alloc_kernel_frame().ok_or("oom for primary L1")?;
    let l2_low = frame::alloc_kernel_frame().ok_or("oom for primary L2 low")?;
    let l2_high = frame::alloc_kernel_frame().ok_or("oom for primary L2 high")?;
    // INITRD-FIX 2: kernel-pool frames sit near the top of RAM (observed
    // 0xbffff000 on `-m 4G virt`), and a real content_shell baked into
    // initrd spans ~280 MB starting around 0x48000000 — both past the old
    // 256 MB (0x40000000..0x50000000) identity window. We now map the full
    // 2 GB frame pool with two more L2 tables:
    //   L1[1] → L2_high  covers 0x40000000..0x7FFFFFFF (1 GB)
    //   L1[2] → L2_xhi   covers 0x80000000..0xBFFFFFFF (1 GB)
    // Without L2_xhi every write to a kernel frame above 0x80000000 —
    // including ELF loader copies to the rebased cave's phys_base+ELF
    // image — faults DATA ABORT DFSC=0x06.
    let l2_xhi = frame::alloc_kernel_frame().ok_or("oom for primary L2 xhi")?;
    // 🎯 STUMP #7: extend identity map to cover the full 4 GiB physical
    // RAM (paired with QEMU_MEMORY_END = 0x140000000). Without these,
    // alloc_frame would hand out frames in [0xC0000000, 0x140000000)
    // but kernel writes there would DATA ABORT.
    let l2_xxhi  = frame::alloc_kernel_frame().ok_or("oom for primary L2 xxhi")?;
    let l2_xxxhi = frame::alloc_kernel_frame().ok_or("oom for primary L2 xxxhi")?;

    // Zero all tables
    for table in [l0, l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
        for i in 0..(PAGE_SIZE / 8) {
            let addr = table + i * 8;
            unsafe {
                core::arch::asm!("str xzr, [{a}]", a = in(reg) addr);
            }
        }
    }

    // L0[0] → L1 table (covers 0x0000000000 - 0x7FFFFFFFFF)
    write_pte(l0, 0, l1 as u64 | TABLE_DESC);

    // L1[0] → L2_low (covers 0x00000000 - 0x3FFFFFFF)
    write_pte(l1, 0, l2_low as u64 | TABLE_DESC);
    // L1[1] → L2_high (covers 0x40000000 - 0x7FFFFFFF)
    write_pte(l1, 1, l2_high as u64 | TABLE_DESC);
    // L1[2] → L2_xhi  (covers 0x80000000 - 0xBFFFFFFF)
    write_pte(l1, 2, l2_xhi as u64 | TABLE_DESC);
    // L1[3] → L2_xxhi (covers 0xC0000000 - 0xFFFFFFFF)
    write_pte(l1, 3, l2_xxhi as u64 | TABLE_DESC);
    // L1[4] → L2_xxxhi (covers 0x100000000 - 0x13FFFFFFF)
    write_pte(l1, 4, l2_xxxhi as u64 | TABLE_DESC);

    // L2_low: Map busybox virtual addresses to physical.
    // Block 0: 0x00000000-0x001FFFFF → phys_base (busybox code segment 1).
    // BLOCK_USER_RW_EXEC per the ROOT-3 transitional note in
    // setup_cave_pagetable — until the ELF loader emits per-segment perms,
    // we give the primary cave a single EL0-RW+X window. EL1 cannot
    // execute here (PXN is set).
    write_pte(l2_low, 0, (phys_base as u64 & !0x1FFFFF) | BLOCK_USER_RW_EXEC);

    // Map MMIO regions (identity mapped, 2MB blocks)
    // UART at 0x09000000 → index 0x09000000/0x200000 = 72
    // virtio at 0x0A000000 → index 0x0A000000/0x200000 = 80
    for mmio_block in [0x08000000, 0x09000000, 0x0A000000, 0x0A200000, 0x0A400000] {
        write_pte(l2_low, mmio_block / 0x200000, mmio_block as u64 | BLOCK_DEVICE);
    }

    // Primary page table: map 20 MB of user-space for legacy in-kernel
    // tests (hello, busybox-running-on-primary). Caves get their own
    // 200 MB window via setup_cave_pagetable. The primary path is NOT
    // widened to 200 MB because it shares this L2 with the MMIO mapping
    // above (blocks 64/72/80/81/82) — a big user window here would
    // overlap MMIO. Chromium runs in a cave, so it doesn't need this.
    for block in 1..10 {
        let virt_block = block * 0x200000;
        let phys_block = (phys_base & !0x1FFFFF) + virt_block;
        write_pte(l2_low, block, phys_block as u64 | BLOCK_USER_RW_EXEC);
    }

    // L2_high + L2_xhi: Identity map the full 2 GB frame pool
    // (0x40000000 - 0xBFFFFFFF) with the same W^X split at __text_end.
    //
    // INITRD-FIX notes (both folded in here):
    //  1. Rust scatters executable code past __text_end into the rodata
    //     PT_LOAD, so the post-text blocks must stay EL1-exec. Transitional
    //     widen to EL1-RW + EL1-exec + UXN (loses EL1 W^X). TODO V9: revert
    //     once `.text.cold.*` lands inside the text segment reliably.
    //  2. Real content_shell initrd lives at 0x48000000..~0x60000000 and
    //     kernel-pool frames sit near 0xBFFFX000 — both past the old 256 MB
    //     window. Extending to 2 GB via L2_xhi (L1[2]) keeps every kernel
    //     write reachable through TTBR0 after MMU-enable.
    let text_end = text_end_addr();
    let kernel_blk_flags = |addr: u64| -> u64 {
        if addr + 0x200000 <= text_end {
            BLOCK_KERNEL_TEXT  // RO + EL1-exec (real .text only)
        } else {
            // EL1-RW + EL1-exec + UXN. Loses EL1 W^X; keeps MMU-enable
            // fetch + large-initrd writes working.
            PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL
                | PTE_AP_EL1_RW | PTE_UXN
        }
    };
    for block in 0..512 {
        let addr = 0x40000000u64 + (block as u64) * 0x200000;
        write_pte(l2_high, block, addr | kernel_blk_flags(addr));
    }
    for block in 0..512 {
        let addr = 0x80000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xhi, block, addr | kernel_blk_flags(addr));
    }
    // 🎯 STUMP #7: identity-map [0xC0000000, 0x140000000). No kernel
    // text lives here, so always EL1-RW + UXN + PXN-implicit (the
    // kernel_blk_flags closure will return non-text flags for these
    // addresses since they're past text_end_addr()).
    for block in 0..512 {
        let addr = 0xC0000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxhi, block, addr | kernel_blk_flags(addr));
    }
    for block in 0..512 {
        let addr = 0x100000000u64 + (block as u64) * 0x200000;
        write_pte(l2_xxxhi, block, addr | kernel_blk_flags(addr));
    }

    uart::puts("[mmu] Page tables built\n");

    uart::puts("[mmu] Configuring registers...\n");

    // Configure MMU registers
    unsafe {
        // MAIR_EL1: Memory Attribute Indirection Register
        // Attr0 = 0xFF (Normal, Write-Back, Cacheable)
        // Attr1 = 0x00 (Device-nGnRnE)
        let mair: u64 = 0x00000000000000FF;
        core::arch::asm!("msr mair_el1, {}", in(reg) mair);

        // TCR_EL1: Translation Control Register
        // T0SZ = 25 (39-bit VA → 512GB address space)
        // TG0 = 0b00 (4KB granule)
        // SH0 = 0b11 (Inner shareable)
        // ORGN0 = 0b01 (Write-back, write-allocate)
        // IRGN0 = 0b01 (Write-back, write-allocate)
        // TBI0 = 1 (Top Byte Ignore for EL0/TTBR0 accesses)
        //
        // CHROMIUM-PHASE-B: Chromium's PartitionAlloc uses tagged
        // pointers with a sentinel byte in the top 8 bits. Without
        // TBI0=1 the hardware faults with FAR=0x70797400... during
        // string-copy-ish code deep in content_shell startup. With
        // TBI0 enabled the CPU strips the top byte for translation,
        // matching Linux's default ARM64 config.
        // 🎯 STUMP #7: TCR.IPS = 0b010 (40-bit IPA, 1 TB) so PAs above
        // 4 GiB (= 0x100000000) translate. Default IPS=0 = 32-bit max
        // = 4 GiB, and any walker output >= 0x100000000 is silently
        // invalidated → DFSC=0x02 (L2 translation fault) on the first
        // kernel access through L2_xxxhi. Sized for headroom.
        let tcr: u64 = (25 << 0)  // T0SZ
                      | (0b00 << 14) // TG0: 4KB
                      | (0b11 << 12) // SH0: inner shareable
                      | (0b01 << 10) // ORGN0
                      | (0b01 << 8)  // IRGN0
                      | (0b010u64 << 32) // IPS: 40-bit IPA (1 TB)
                      | (1u64 << 37); // TBI0: top byte ignore for TTBR0
        core::arch::asm!("msr tcr_el1, {}", in(reg) tcr);

        // TTBR0_EL1: Point to L1 table (T0SZ=25 → 39-bit VA → starts at L1)
        PRIMARY_L1 = l1;
        core::arch::asm!("msr ttbr0_el1, {}", in(reg) l1 as u64);

        uart::puts("[mmu] MAIR+TCR+TTBR set\n");

        // Flush TLB
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");

        uart::puts("[mmu] TLB flushed, enabling MMU...\n");

        // Enable MMU via SCTLR_EL1.
        // 🎯 STUMP #59: caches must be ON. With C=0, atomic exclusive
        // accesses (LDXR/STXR/CAS) targeting cacheable memory have
        // UNPREDICTABLE behavior per ARM ARM C5.2.4 — observed under
        // HVF/M4 as fetch_add/cmpxchg returning the old value but
        // NOT performing the RMW, breaking all bump-allocator
        // atomics in syscall.rs. TCG's software path masks the bug
        // (each instruction is interpreted), so this only surfaces
        // on hardware-accelerated guests.
        let mut sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        sctlr |= 1;          // M bit = enable MMU
        sctlr |= 1 << 2;     // C bit  = enable D-cache (REQUIRED for atomics on M4/HVF)
        sctlr |= 1 << 12;    // I bit  = enable I-cache

        // 🎯 STUMP #7: clean every page-table page we just wrote to PoC.
        // The walker reads PT entries with TCR attributes (inner-
        // shareable, write-back); our pre-MMU writes need to be
        // visible after turn-on. Without this, the walker hits stale
        // (zero) cache lines for the new high-PA tables and the next
        // instruction fetch silently faults.
        for pt in [l0, l1, l2_low, l2_high, l2_xhi, l2_xxhi, l2_xxxhi] {
            let base = pt as u64;
            let mut line = base;
            while line < base + PAGE_SIZE as u64 {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
        }
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");

        core::arch::asm!("msr sctlr_el1, {}", in(reg) sctlr);
        core::arch::asm!("isb");
    }

    // QEMU-BUGFIX-7: flag that we're on the primary page table so
    // `uaccess::is_user_range` accepts pointers in the legacy
    // [0x1000, 0x4000_0000) window. Without this, every user-space
    // pointer from a primary-cave ELF (busybox uname, hello, etc.) is
    // rejected with EFAULT by the strict post-V5 path-check, and
    // sys_writev silently drops all stdout traffic from the ELF —
    // process runs to completion but produces no visible output.
    set_active_primary(true);

    uart::puts("[mmu] MMU enabled!\n");
    Ok(())
}

/// Disable MMU (return to flat physical addressing).
pub fn disable() {
    unsafe {
        // Flush TLB first
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");

        // Disable MMU
        let mut sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        sctlr &= !1;       // M bit off
        sctlr &= !(1 << 2);  // C bit off
        sctlr &= !(1 << 12); // I bit off
        core::arch::asm!("msr sctlr_el1, {}", in(reg) sctlr);
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");

        // Flush TLB again after disable
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb");
    }
}

fn write_pte(table: usize, index: usize, value: u64) {
    let addr = table + index * 8;
    unsafe {
        core::arch::asm!("str {v}, [{a}]", a = in(reg) addr, v = in(reg) value);
    }
}
