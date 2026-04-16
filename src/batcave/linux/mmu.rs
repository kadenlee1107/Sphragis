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

const PAGE_SIZE: usize = 4096;
const ENTRIES_PER_TABLE: usize = 512;

// Page table entry flags
const PTE_VALID: u64 = 1;
const PTE_TABLE: u64 = 1 << 1;  // Points to next-level table
const PTE_BLOCK: u64 = 0;       // 2MB block (at L2)
const PTE_AF: u64 = 1 << 10;    // Access flag
const PTE_SH_INNER: u64 = 3 << 8; // Inner shareable
const PTE_ATTR_NORMAL: u64 = 0 << 2; // MAIR index 0: normal memory
const PTE_ATTR_DEVICE: u64 = 1 << 2; // MAIR index 1: device memory
const PTE_AP_RW: u64 = 0 << 6;  // Read-write

// 2MB block entry flags for normal memory
const BLOCK_NORMAL: u64 = PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_RW;
// 2MB block entry flags for device memory (MMIO)
const BLOCK_DEVICE: u64 = PTE_VALID | PTE_AF | PTE_ATTR_DEVICE | PTE_AP_RW;
// Table descriptor flags
const TABLE_DESC: u64 = PTE_VALID | PTE_TABLE;

// Per-BatCave page table registry
// Each cave gets its own L1 table, mapping only its own physical memory
const MAX_CAVE_PAGETABLES: usize = 8;
static mut CAVE_L1: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES]; // L1 table phys addr per cave
static mut CAVE_PHYS_BASE: [usize; MAX_CAVE_PAGETABLES] = [0; MAX_CAVE_PAGETABLES]; // per-cave phys base
static mut PRIMARY_L1: usize = 0; // the primary (ash) page table

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
    if cave_slot >= MAX_CAVE_PAGETABLES { return Err("too many cave page tables"); }

    let l1 = frame::alloc_frame().ok_or("oom for cave L1")?;
    let l2_low = frame::alloc_frame().ok_or("oom for cave L2_low")?;
    let l2_high = frame::alloc_frame().ok_or("oom for cave L2_high")?;

    // Zero tables
    for table in [l1, l2_low, l2_high] {
        for i in 0..(PAGE_SIZE / 8) {
            unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) table + i * 8); }
        }
    }

    // L1[0] → L2_low, L1[1] → L2_high
    write_pte(l1, 0, l2_low as u64 | TABLE_DESC);
    write_pte(l1, 1, l2_high as u64 | TABLE_DESC);

    // L2_low: map THIS cave's user-space binary (blocks 0-99 = 200 MB).
    // Widened from 20 MB to 200 MB to host Chromium content_shell (~150 MB).
    //
    // NOTE: no MMIO mapping in cave page tables. Caves must go through
    // syscalls for UART / virtio access — direct MMIO from user space
    // was a legacy shortcut that punched 5 × 2 MB holes through this
    // window at 0x08M/0x09M/0x0AM/0x0A2M/0x0A4M, corrupting any binary
    // large enough to reach those addresses.
    for block in 0..100 {
        let virt_block = block * 0x200000;
        let phys_block = (phys_base & !0x1FFFFF) + virt_block;
        write_pte(l2_low, block, phys_block as u64 | BLOCK_NORMAL);
    }

    // L2_high: identity map kernel RAM
    for block in 0..128 {
        let addr = 0x40000000u64 + (block as u64) * 0x200000;
        write_pte(l2_high, block, addr | BLOCK_NORMAL);
    }

    unsafe {
        CAVE_L1[cave_slot] = l1;
        CAVE_PHYS_BASE[cave_slot] = phys_base;
    }

    Ok(l1)
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
pub fn setup_and_enable(phys_base: usize) -> Result<(), &'static str> {
    uart::puts("[mmu] Setting up page tables...\n");

    // Allocate L0 table (1 page)
    let l0 = frame::alloc_frame().ok_or("out of memory for L0")?;
    // Allocate L1 table for first 512GB
    let l1 = frame::alloc_frame().ok_or("out of memory for L1")?;
    // Allocate L2 tables
    let l2_low = frame::alloc_frame().ok_or("out of memory for L2 low")?;   // 0x00000000-0x3FFFFFFF
    let l2_high = frame::alloc_frame().ok_or("out of memory for L2 high")?;  // 0x40000000-0x7FFFFFFF

    // Zero all tables
    for table in [l0, l1, l2_low, l2_high] {
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

    // L2_low: Map busybox virtual addresses to physical
    // Block 0: 0x00000000-0x001FFFFF → phys_base (busybox code segment 1)
    write_pte(l2_low, 0, (phys_base as u64 & !0x1FFFFF) | BLOCK_NORMAL);

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
        write_pte(l2_low, block, phys_block as u64 | BLOCK_NORMAL);
    }

    // L2_high: Identity map kernel RAM (0x40000000 - 0x4FFFFFFF)
    // 256MB = 128 × 2MB blocks
    for block in 0..128 {
        let addr = 0x40000000u64 + (block as u64) * 0x200000;
        write_pte(l2_high, block, addr | BLOCK_NORMAL);
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
        let tcr: u64 = (25 << 0)  // T0SZ
                      | (0b00 << 14) // TG0: 4KB
                      | (0b11 << 12) // SH0: inner shareable
                      | (0b01 << 10) // ORGN0
                      | (0b01 << 8); // IRGN0
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

        // Enable MMU via SCTLR_EL1 (without caches first)
        let mut sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        sctlr |= 1;  // M bit = enable MMU
        // Don't enable caches yet — keep it simple
        sctlr &= !(1 << 2);  // C bit OFF
        sctlr &= !(1 << 12); // I bit OFF
        core::arch::asm!("msr sctlr_el1, {}", in(reg) sctlr);
        core::arch::asm!("isb");
    }

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
