#![allow(dead_code)]
// Sphragis — Apple Silicon EL2 MMU bring-up.
//
// On M4 (and every Apple Silicon chip so far), m1n1 hands off at EL2
// with the MMU DISABLED. That leaves every physical address mapped as
// Device-nGnRnE, which has three painful consequences we've been
// working around all session:
//
//  1. LDXR/STXR never succeed on Device memory, so every atomic RMW
//     on AArch64 without `+lse` lowers to an infinite LDXR/STXR loop.
//     Fixed point-by-point by rewriting every boot-path RMW as a
//     plain load + store under an IrqGuard.
//
//  2. Writes to RAM go through the write buffer without caching, so
//     every `write_volatile` is a bus transaction. This is why
//     `fill_screen` takes visible time and why we had to add
//     `dsb sy` at the end of big wipes.
//
//  3. No address-space isolation between whatever would be "kernel"
//     and whatever would be "user" — so BatCave processes would share
//     the kernel's flat PA space. Blocks real process isolation.
//
// This module builds a minimal identity-mapped page-table tree and
// enables the EL2 MMU. After that:
//   * RAM is Normal-Inner-Shareable-Cacheable (write-back + cache
//     coherent), so LDXR/STXR work correctly, fill_screen runs at
//     memory-bandwidth speed, and CPU caches are usable.
//   * MMIO regions (UART at 0x3_8812_8000, AIC at 0x3_8100_0000, FB
//     at 0x1_03e0_0500_0, ...) stay Device-nGnRE so we don't
//     accidentally reorder writes to control registers.
//   * Every virtual address == physical (identity map), so existing
//     code that uses PAs directly keeps working after `enable()`.
//
// This is a pure-EL2 scaffold — the existing `kernel::mm::page_table`
// module targets EL1 (writes TTBR0_EL1). We don't share types with
// it because the EL2 TCR/MAIR field layouts differ subtly (no EL1-0
// access, no ASID support in EL2_regime translation regime).
//
// Status: SCAFFOLD. Not wired into kernel_main_apple yet. Next
// session's job: call `build_and_enable()` from the Apple path
// after `discover_from_adt` finishes (so we know the MMIO base
// addresses to map as Device), before `kernel::mm::init` so the
// heap and frame allocator start with caches working.

use super::frame::{self, PAGE_SIZE};

// ─── Page-table entry bits (T8110-family, 4KiB granule, 3-level) ───

const PTE_VALID: u64 = 1 << 0;
const PTE_TABLE: u64 = 1 << 1; // intermediate levels: next level is a table
const PTE_PAGE:  u64 = 1 << 1; // level 3 leaf: this is a 4KiB page
const PTE_BLOCK: u64 = 0 << 1; // level 1/2 leaf: this is a 1GiB/2MiB block

// Low-attr indexes into MAIR_EL2 (see `MAIR_EL2_VALUE`).
const ATTR_IDX_DEVICE_NGNRNE: u64 = 0 << 2;
const ATTR_IDX_DEVICE_NGNRE:  u64 = 1 << 2;
const ATTR_IDX_NORMAL:        u64 = 2 << 2;

// Shareability (bits [9:8]).
const SH_NONE:    u64 = 0 << 8;
const SH_OUTER:   u64 = 2 << 8;
const SH_INNER:   u64 = 3 << 8;

// Access permissions at EL2 translation regime: the "AP" field is a
// single bit (AP[2], bit 7) — 0 = R/W, 1 = R/O. No EL0 access concept.
const AP_RW: u64 = 0 << 7;
const AP_RO: u64 = 1 << 7;

const AF:   u64 = 1 << 10; // Access Flag — we set unconditionally so the
                           //   CPU never takes an access-flag fault.
const XN:   u64 = 1 << 54; // Execute-never (at EL2 this is UXN+XN in one bit).

const ADDR_MASK: u64 = 0x0000_FFFF_FFFF_F000;

const ENTRIES_PER_TABLE: usize = 512;
const TABLE_SIZE_BYTES: usize = ENTRIES_PER_TABLE * 8; // 4 KiB

// ─── Memory-attribute encoding table (MAIR_EL2) ───
//
// Layout: 8 attr bytes packed into a u64. Our layout:
//   attr0 = 0x00  Device-nGnRnE (strict order, no gathering, no
//                                 reordering, no early ack)
//   attr1 = 0x04  Device-nGnRE  (allow early ack; same ordering)
//   attr2 = 0xff  Normal memory, Inner/Outer WB non-transient R+W
//                 cache-allocate — the ARM "everyday RAM" attr.
//   attr3..7: 0
const MAIR_EL2_VALUE: u64 = 0x00ff0400;

// ─── TCR_EL2 encoding for Apple Silicon identity map (4K granule) ───
//
// Fields:
//   T0SZ  = 16   → 48-bit input address (2^(64-16) = 2^48)
//   IRGN0 = 0b01 → Inner WB-WA cacheable
//   ORGN0 = 0b01 → Outer WB-WA cacheable
//   SH0   = 0b11 → Inner Shareable
//   TG0   = 0b00 → 4 KiB granule
//   IPS   = 0b101 → 48-bit output (64 TiB); Apple M4 supports up to
//                   Intermediate Phys Addr size ID_AA64MMFR0_EL1.PARange
//                   which is ≥48 bits on Apple cores.
//   TBI0  = 0    → Top-byte-ignore off (we don't use AArch64 tagged
//                  pointers in the kernel).
//
// Bit layout: the EL2 regime only has one TTBR (TTBR0_EL2), so
// TCR_EL2 uses the "simple" layout (bits [5:0] T0SZ, [9:8] IRGN0,
// [11:10] ORGN0, [13:12] SH0, [15:14] TG0, [18:16] PS, [20] TBI).
const TCR_EL2_VALUE: u64 =
      16u64                    // T0SZ
    | (0b01u64 << 8)           // IRGN0 = WB-WA
    | (0b01u64 << 10)          // ORGN0 = WB-WA
    | (0b11u64 << 12)          // SH0 = Inner Shareable
    | (0b00u64 << 14)          // TG0 = 4K
    | (0b101u64 << 16)         // PS  = 48-bit
    ;

/// High-level region descriptor passed to `build_identity_tables`.
#[derive(Clone, Copy)]
pub struct Region {
    pub phys_start: usize,
    pub size: usize,
    pub attr: RegionAttr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegionAttr {
    /// Normal RAM — Inner-Shareable, cacheable, R/W, XN.
    Ram,
    /// Kernel text — Normal, cacheable, R/O, executable.
    KernelText,
    /// MMIO — Device-nGnRE (allow early ack, strict order).
    Mmio,
}

impl RegionAttr {
    fn to_pte_low(self) -> u64 {
        match self {
            RegionAttr::Ram        => ATTR_IDX_NORMAL | SH_INNER | AP_RW | AF | XN,
            RegionAttr::KernelText => ATTR_IDX_NORMAL | SH_INNER | AP_RO | AF, // exec allowed
            RegionAttr::Mmio       => ATTR_IDX_DEVICE_NGNRE | AP_RW | AF | XN,
        }
    }
}

/// Walk of an EL2 4K/3-level page-table tree. Allocated via the
/// `frame` allocator (one 4 KiB frame per level).
pub struct El2TablesBuilder {
    /// Level-0 table root (the address we'll drop into TTBR0_EL2).
    l0: usize,
}

impl El2TablesBuilder {
    pub fn new() -> Option<Self> {
        let l0 = frame::alloc_frame()?;
        zero_table(l0);
        Some(Self { l0 })
    }

    pub fn root_phys(&self) -> usize { self.l0 }

    /// Populate the table tree so every 4 KiB page in each `Region`
    /// is identity-mapped with its attribute. Returns the first
    /// error if any frame allocation fails.
    pub fn map_region(&self, r: Region) -> Result<(), &'static str> {
        let mut page = r.phys_start & !(PAGE_SIZE - 1);
        let end = (r.phys_start + r.size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        while page < end {
            self.map_page(page, r.attr)?;
            page += PAGE_SIZE;
        }
        Ok(())
    }

    fn map_page(&self, pa: usize, attr: RegionAttr) -> Result<(), &'static str> {
        let va = pa; // identity
        let l0_idx = (va >> 39) & 0x1FF;
        let l1_idx = (va >> 30) & 0x1FF;
        let l2_idx = (va >> 21) & 0x1FF;
        let l3_idx = (va >> 12) & 0x1FF;

        let l1 = get_or_create_child(self.l0, l0_idx)?;
        let l2 = get_or_create_child(l1, l1_idx)?;
        let l3 = get_or_create_child(l2, l2_idx)?;

        let entry = (pa as u64 & ADDR_MASK)
            | PTE_VALID | PTE_PAGE
            | attr.to_pte_low();
        unsafe {
            let slot = (l3 + l3_idx * 8) as *mut u64;
            core::ptr::write_volatile(slot, entry);
        }
        Ok(())
    }
}

fn zero_table(phys: usize) {
    unsafe {
        for i in 0..ENTRIES_PER_TABLE {
            core::ptr::write_volatile((phys + i * 8) as *mut u64, 0);
        }
    }
}

fn get_or_create_child(parent: usize, idx: usize) -> Result<usize, &'static str> {
    let slot = (parent + idx * 8) as *mut u64;
    let cur = unsafe { core::ptr::read_volatile(slot) };
    if cur & PTE_VALID != 0 {
        if cur & PTE_TABLE == 0 {
            // There's a block entry here — can't drill down; caller
            // tried to map a finer region over a coarser one.
            return Err("block entry blocks sub-mapping");
        }
        return Ok((cur & ADDR_MASK) as usize);
    }
    let child = frame::alloc_frame().ok_or("out of memory (page tables)")?;
    zero_table(child);
    let entry = (child as u64 & ADDR_MASK) | PTE_VALID | PTE_TABLE;
    unsafe { core::ptr::write_volatile(slot, entry); }
    Ok(child)
}

/// Enable the MMU at EL2 using `ttbr0` as the translation root.
///
/// Caller invariants:
///  1. `ttbr0` points at a level-0 table populated with a valid
///     identity map covering every code/data/stack region the kernel
///     currently executes from. Any missing page → instant translation
///     fault the moment MMU turns on.
///  2. Called at EL2 (m1n1 hands off at EL2 on M4). Writing SCTLR_EL2
///     while at EL1 does nothing useful.
///  3. Interrupts are masked (caller's responsibility). TLB
///     invalidation between populating the tables and enabling the
///     MMU is done here.
///
/// After return, every memory access goes through translation, and
/// LDXR/STXR (and native atomics) start working on Normal regions.
///
/// # Safety
/// Violating any of the invariants above bricks the kernel.
pub unsafe fn enable(ttbr0: usize) {
    unsafe {
        // Configure attribute indirection + translation control.
        core::arch::asm!(
            "msr mair_el2, {mair}",
            "msr tcr_el2, {tcr}",
            "msr ttbr0_el2, {ttbr}",
            "isb",
            mair = in(reg) MAIR_EL2_VALUE,
            tcr  = in(reg) TCR_EL2_VALUE,
            ttbr = in(reg) ttbr0 as u64,
        );

        // Drop any stale TLB entries from the disabled-MMU period.
        core::arch::asm!(
            "tlbi alle2",
            "dsb sy",
            "isb",
        );

        // Read-modify-write SCTLR_EL2 to set M (bit 0) + C (bit 2) +
        // I (bit 12). Leave the rest of the bits as m1n1 set them.
        let mut sctlr: u64;
        core::arch::asm!("mrs {0}, sctlr_el2", out(reg) sctlr);
        sctlr |= 1 << 0;   // M: enable stage-1 translation
        sctlr |= 1 << 2;   // C: enable data cacheability
        sctlr |= 1 << 12;  // I: enable instruction cacheability
        core::arch::asm!(
            "msr sctlr_el2, {0}",
            "isb",
            in(reg) sctlr,
        );
    }
}

/// Convenience: build identity tables for all the regions the Apple
/// bring-up currently touches and enable MMU. Caller must guarantee
/// the bring-up path stops painting the FB, writing MMIO, and
/// allocating frames during the call (the enable sequence itself
/// takes only microseconds but tables are built beforehand).
///
/// # Safety
/// See `enable`. This is the one-call wrapper; gate behind a feature
/// flag until we've tested it end to end on M4.
pub unsafe fn build_and_enable(regions: &[Region]) -> Result<(), &'static str> {
    let builder = El2TablesBuilder::new().ok_or("no frames for page table root")?;
    for r in regions {
        builder.map_region(*r)?;
    }
    // Publish before enabling — we need every PTE write visible to
    // the MMU walker before MMU turns on.
    unsafe { core::arch::asm!("dsb sy"); }
    unsafe { enable(builder.root_phys()); }
    Ok(())
}
