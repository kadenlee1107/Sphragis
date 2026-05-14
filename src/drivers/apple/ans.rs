#![allow(dead_code)]
// Sphragis — Apple ANS (Apple NVMe Storage) Driver
// Custom NVMe controller for Apple Silicon internal SSDs.
// Uses standard NVMe protocol with Apple-specific initialization.
// Reference: Asahi Linux drivers/nvme/host/apple.c
//
// Apple ANS differences from standard NVMe:
// - Wrapped in Apple's ANS block with extra registers
// - Requires DART (IOMMU) setup for DMA
// - Uses Apple-specific admin commands for initialization
// - SART (System Address Range Table) for memory protection

use super::soc;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// ANS Wrapper Registers (Apple-specific, around the NVMe core)
const ANS_LINEAR_SQ_CTRL: usize = 0x24908;
const ANS_LINEAR_ASQ_DB: usize = 0x2490C;
const ANS_LINEAR_IOSQ_DB: usize = 0x24910;
const ANS_MAX_PEND_CMDS: usize = 0x24914;
const ANS_BOOT_STATUS: usize = 0x1300;
const ANS_UNKNOWN_CTRL: usize = 0x24008;
const ANS_CPU_CTRL: usize = 0x44;
const ANS_MODESEL: usize = 0x01304;

// Standard NVMe registers (inside the ANS wrapper)
const NVME_CAP: usize = 0x0;
const NVME_VS: usize = 0x8;
const NVME_INTMS: usize = 0xC;
const NVME_INTMC: usize = 0x10;
const NVME_CC: usize = 0x14;
const NVME_CSTS: usize = 0x1C;
const NVME_AQA: usize = 0x24;
const NVME_ASQ: usize = 0x28;
const NVME_ACQ: usize = 0x30;

// NVMe Command Opcodes
const NVME_CMD_IDENTIFY: u8 = 0x06;
const NVME_CMD_READ: u8 = 0x02;
const NVME_CMD_WRITE: u8 = 0x01;

// NVMe Submission Queue Entry (64 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
struct NvmeSqe {
    opcode: u8,
    flags: u8,
    cid: u16,
    nsid: u32,
    rsvd: [u32; 2],
    mptr: u64,
    prp1: u64,
    prp2: u64,
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

// NVMe Completion Queue Entry (16 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
struct NvmeCqe {
    result: u32,
    rsvd: u32,
    sq_head: u16,
    sq_id: u16,
    cid: u16,
    status: u16,
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static NVME_BASE: AtomicUsize = AtomicUsize::new(0);

fn read32(base: usize, offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) }
}

fn write32(base: usize, offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + offset) as *mut u32, val) }
}

fn write64(base: usize, offset: usize, val: u64) {
    unsafe { core::ptr::write_volatile((base + offset) as *mut u64, val) }
}

/// Initialize the Apple ANS NVMe controller.
/// This requires DART setup (IOMMU) before it can do DMA.
pub fn init() -> Result<(), &'static str> {
    let base = soc::ans_base();
    NVME_BASE.store(base, Ordering::Relaxed);

    // Check ANS boot status
    let status = read32(base, ANS_BOOT_STATUS);
    if status == 0 {
        return Err("ANS not booted by firmware");
    }

    // Read NVMe capability
    let _cap_lo = read32(base, NVME_CAP);
    let cap_hi = read32(base, NVME_CAP + 4);
    let _mqes = (_cap_lo & 0xFFFF) as u16; // Max Queue Entries
    let _dstrd = (cap_hi & 0xF) as u32; // Doorbell Stride (in high 32 bits of CAP)

    // Apple ANS-specific: configure linear submission queue mode
    write32(base, ANS_LINEAR_SQ_CTRL, 0x1);
    write32(base, ANS_MAX_PEND_CMDS, 0x40);

    // We would need to:
    // 1. Set up DART (IOMMU) for ANS DMA
    // 2. Allocate admin submission/completion queues
    // 3. Configure NVMe (CC register)
    // 4. Wait for CSTS.RDY
    // 5. Send Identify commands
    // 6. Create IO queues
    //
    // For now, mark as initialized — full NVMe bring-up
    // requires DART which is the next piece.

    INITIALIZED.store(true, Ordering::Relaxed);
    Ok(())
}

/// Read sectors from the SSD.
/// sector: LBA (512-byte sectors)
/// count: number of sectors
/// buf: destination buffer
pub fn read_sectors(_sector: u64, _count: u32, _buf: &mut [u8]) -> Result<(), &'static str> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return Err("ANS not initialized");
    }
    // Full implementation would:
    // 1. Build NVMe Read command SQE
    // 2. Set PRP1 to buf physical address (via DART mapping)
    // 3. Submit to IO submission queue
    // 4. Ring doorbell
    // 5. Poll completion queue
    // 6. Check status
    Err("NVMe I/O not yet implemented (needs DART)")
}

/// Write sectors to the SSD.
pub fn write_sectors(_sector: u64, _count: u32, _buf: &[u8]) -> Result<(), &'static str> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return Err("ANS not initialized");
    }
    Err("NVMe I/O not yet implemented (needs DART)")
}

pub fn is_ready() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}
