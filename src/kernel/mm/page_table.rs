// Bat_OS — ARM64 Page Table Manager
// Sets up 4-level page tables (4KB granule) for virtual address spaces.
// Each process gets its own page table tree → hardware-enforced isolation.

use super::frame::{self, PAGE_SIZE};

// ARM64 page table entry flags
const PT_VALID: u64 = 1 << 0;
const PT_TABLE: u64 = 1 << 1; // Next level is a table (not a block)
const PT_PAGE: u64 = 1 << 1; // Level 3: this is a page (same bit, different level)
const PT_USER: u64 = 1 << 6; // EL0 accessible
const PT_READ_ONLY: u64 = 1 << 7; // Read-only
const PT_ACCESSED: u64 = 1 << 10;
const PT_INNER_SHAREABLE: u64 = 0b11 << 8;
const PT_AF: u64 = 1 << 10; // Access Flag

// Memory attribute indices (configured in MAIR_EL1)
const PT_ATTR_NORMAL: u64 = 0 << 2; // Index 0: normal memory
const PT_ATTR_DEVICE: u64 = 1 << 2; // Index 1: device memory (MMIO)

// UXN/PXN: execute-never bits
const PT_UXN: u64 = 1 << 54; // Unprivileged execute-never
const PT_PXN: u64 = 1 << 53; // Privileged execute-never

const ENTRIES_PER_TABLE: usize = 512;
const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

#[derive(Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub user: bool,
    pub executable: bool,
    pub device: bool,
}

impl PageFlags {
    pub fn kernel_code() -> Self {
        Self { writable: false, user: false, executable: true, device: false }
    }

    pub fn kernel_data() -> Self {
        Self { writable: true, user: false, executable: false, device: false }
    }

    pub fn user_code() -> Self {
        Self { writable: false, user: true, executable: true, device: false }
    }

    pub fn user_data() -> Self {
        Self { writable: true, user: true, executable: false, device: false }
    }

    pub fn device() -> Self {
        Self { writable: true, user: false, executable: false, device: true }
    }

    fn to_pte_flags(&self) -> u64 {
        let mut flags = PT_VALID | PT_PAGE | PT_AF | PT_INNER_SHAREABLE;

        if self.device {
            flags |= PT_ATTR_DEVICE;
        } else {
            flags |= PT_ATTR_NORMAL;
        }

        if self.user {
            flags |= PT_USER;
        }

        if !self.writable {
            flags |= PT_READ_ONLY;
        }

        if !self.executable {
            if self.user {
                flags |= PT_UXN;
            } else {
                flags |= PT_PXN;
            }
        }

        flags
    }
}

/// A page table root. Each process has one.
pub struct AddressSpace {
    root_table: usize, // Physical address of L0 page table
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let root = frame::alloc_frame()?;
        Some(Self { root_table: root })
    }

    pub fn root_phys(&self) -> usize {
        self.root_table
    }

    /// Map a virtual address to a physical address with given flags.
    pub fn map(&self, virt: usize, phys: usize, flags: PageFlags) -> Result<(), &'static str> {
        let l0_idx = (virt >> 39) & 0x1FF;
        let l1_idx = (virt >> 30) & 0x1FF;
        let l2_idx = (virt >> 21) & 0x1FF;
        let l3_idx = (virt >> 12) & 0x1FF;

        let l1_table = self.get_or_create_table(self.root_table, l0_idx)?;
        let l2_table = self.get_or_create_table(l1_table, l1_idx)?;
        let l3_table = self.get_or_create_table(l2_table, l2_idx)?;

        let entry_ptr = (l3_table + l3_idx * 8) as *mut u64;
        let entry = (phys as u64 & ADDR_MASK) | flags.to_pte_flags();
        unsafe {
            core::ptr::write_volatile(entry_ptr, entry);
        }

        Ok(())
    }

    /// Map a range of pages (virtual to physical, 1:1 offset).
    pub fn map_range(
        &self,
        virt_start: usize,
        phys_start: usize,
        size: usize,
        flags: PageFlags,
    ) -> Result<(), &'static str> {
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        for i in 0..pages {
            let offset = i * PAGE_SIZE;
            self.map(virt_start + offset, phys_start + offset, flags)?;
        }
        Ok(())
    }

    fn get_or_create_table(&self, table: usize, index: usize) -> Result<usize, &'static str> {
        let entry_ptr = (table + index * 8) as *mut u64;
        let entry = unsafe { core::ptr::read_volatile(entry_ptr) };

        if entry & PT_VALID != 0 {
            // Table already exists
            Ok((entry & ADDR_MASK) as usize)
        } else {
            // Allocate new table
            let new_table = frame::alloc_frame().ok_or("out of memory")?;
            let new_entry = (new_table as u64 & ADDR_MASK) | PT_VALID | PT_TABLE;
            unsafe {
                core::ptr::write_volatile(entry_ptr, new_entry);
            }
            Ok(new_table)
        }
    }
}

/// Activate an address space by writing to TTBR0_EL1.
pub fn activate(space: &AddressSpace) {
    unsafe {
        let ttbr = space.root_phys() as u64;
        core::arch::asm!(
            "msr ttbr0_el1, {0}",
            "isb",
            in(reg) ttbr,
        );
    }
}
