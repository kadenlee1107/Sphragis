#![allow(dead_code)]
// Sphragis — Apple ANS NVMe driver (Phase 3.3c)
//
// ANS = "Apple NVMe Storage". Apple took the standard NVMe spec and:
//   1. Replaced the PCIe transport with MMIO behind a DART.
//   2. Added their own "linear queue" mode (registers at 0x24000+)
//      because they dislike the standard submission/completion doorbell
//      model. The standard NVMe CAP/CC/CSTS/AQA/ASQ/ACQ/doorbell
//      registers are still honored — but the ANS firmware also accepts
//      commands through the linear-SQ path which lets it batch better.
//   3. Wrapped firmware loading + boot through an ASC/RTKit mailbox.
//      On AP boot, the AP:
//        a) Loads firmware into ANS memory via the RTKit protocol
//        b) Starts the ANS coprocessor (ASC CPU_CONTROL.START)
//        c) Polls NVME_BOOT_STATUS for 0xDE71CE55 (ANS boot OK magic)
//        d) Issues standard NVMe Identify Controller + Identify Namespace
//        e) Creates I/O submission + completion queues
//        f) I/O commands use standard NVMe opcodes 0x01 (Write) / 0x02 (Read)
//
// Apple additionally uses an "NVMMU" (at offset 0x28100+) which provides
// a TCB (Transfer Control Block) table for DMA addressing — a
// hardware-side DMA descriptor cache that sits between the standard
// NVMe PRP list and the DART IOMMU.
//
// This module implements the LOW-LEVEL NVMe/ANS register layout plus
// command/completion struct layouts. The actual queue allocation + I/O
// issuance paths are stubbed with `NotReady` returns until we have:
//   (a) DART IOVA allocator with real mapping (Phase 3.2)
//   (b) A DMA-coherent frame allocator (Phase 3.3d)
// Those two dependencies are what gate actual disk I/O on real hardware.
//
// Reference: m1n1/src/nvme.c (MIT — register + protocol reference only).
// Reference: NVM Express Base Specification 1.4 (public, not copyrighted
//            layout).

use super::asc::{Asc, AscError};
use super::dart::Dart;
use super::rtkit::{Rtkit, RtkitError};

// ─── Standard NVMe register offsets ────────────────────────────────

pub const REG_CAP:   usize = 0x00; // Controller Capabilities (u64)
pub const REG_VS:    usize = 0x08; // Version
pub const REG_CC:    usize = 0x14; // Controller Configuration
pub const REG_CSTS:  usize = 0x1c; // Controller Status
pub const REG_AQA:   usize = 0x24; // Admin Queue Attributes
pub const REG_ASQ:   usize = 0x28; // Admin SQ Base (u64)
pub const REG_ACQ:   usize = 0x30; // Admin CQ Base (u64)

pub const CC_EN:     u32 = 1 << 0;
pub const CC_SHN_NORMAL: u32 = 1 << 14;
pub const CC_SHN_ABRUPT: u32 = 2 << 14;

pub const CSTS_RDY:           u32 = 1 << 0;
pub const CSTS_SHST_NORMAL:   u32 = 0;
pub const CSTS_SHST_BUSY:     u32 = 1 << 2;
pub const CSTS_SHST_DONE:     u32 = 2 << 2;

// Standard doorbells (we only use these in fallback; Apple prefers linear).
pub const REG_DB_ACQ:    usize = 0x1004;
pub const REG_DB_IOCQ:   usize = 0x100c;

// ─── Apple-specific ANS registers ──────────────────────────────────

/// `BOOT_STATUS` reads 0xDE71CE55 once the ANS coproc firmware is up.
pub const ANS_BOOT_STATUS:    usize = 0x1300;
pub const ANS_BOOT_STATUS_OK: u32   = 0xDE71_CE55;

// "Linear SQ" — Apple's preferred queue mode.
pub const ANS_LINEAR_SQ_CTRL:    usize = 0x24908;
pub const ANS_LINEAR_SQ_CTRL_EN: u32   = 1 << 0;

pub const ANS_UNKNOWN_CTRL:                usize = 0x24008;
pub const ANS_UNKNOWN_CTRL_PRP_NULL_CHECK: u32   = 1 << 11;

pub const ANS_MAX_PEND_CMDS_CTRL: usize = 0x1210;
pub const ANS_DB_LINEAR_ASQ:      usize = 0x2490c;
pub const ANS_DB_LINEAR_IOSQ:     usize = 0x24910;

// NVMMU: Apple's hardware DMA descriptor cache in front of DART.
pub const NVMMU_NUM:        usize = 0x28100;
pub const NVMMU_ASQ_BASE:   usize = 0x28108; // u64
pub const NVMMU_IOSQ_BASE:  usize = 0x28110; // u64
pub const NVMMU_TCB_INVAL:  usize = 0x28118;
pub const NVMMU_TCB_STAT:   usize = 0x29120;

// ─── NVMe admin / I/O opcodes ──────────────────────────────────────

pub const ADMIN_DELETE_SQ: u8 = 0x00;
pub const ADMIN_CREATE_SQ: u8 = 0x01;
pub const ADMIN_DELETE_CQ: u8 = 0x04;
pub const ADMIN_CREATE_CQ: u8 = 0x05;
pub const ADMIN_IDENTIFY:  u8 = 0x06;
pub const ADMIN_SET_FEAT:  u8 = 0x09;

pub const IO_WRITE: u8 = 0x01;
pub const IO_READ:  u8 = 0x02;

/// CQE bits: `NVME_QUEUE_CONTIGUOUS` (CREATE_SQ/CQ must be contiguous).
pub const QUEUE_CONTIGUOUS: u32 = 1 << 0;

// ─── Wire structures ────────────────────────────────────────────────

/// 64-byte NVMe Submission Queue Entry (standard NVMe layout).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Sqe {
    pub opcode: u8,
    pub flags: u8,
    pub cid: u16,         // Command Identifier
    pub nsid: u32,        // Namespace ID (0 for admin, ≥1 for I/O)
    pub _rsvd0: [u32; 2],
    pub mptr: u64,        // Metadata Pointer (unused)
    pub prp1: u64,        // First Physical Region Page
    pub prp2: u64,        // Second PRP (or list pointer)
    pub cdw10: u32,       // Command-specific DWORDs 10-15
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}
const _SQE_SIZE_CHECK: () = assert!(core::mem::size_of::<Sqe>() == 64);

impl Sqe {
    pub const fn zero() -> Self {
        Sqe {
            opcode: 0, flags: 0, cid: 0, nsid: 0,
            _rsvd0: [0; 2], mptr: 0, prp1: 0, prp2: 0,
            cdw10: 0, cdw11: 0, cdw12: 0, cdw13: 0, cdw14: 0, cdw15: 0,
        }
    }

    /// Build an Identify Controller (CNS=1) admin command.
    pub fn identify_controller(cid: u16, prp1: u64) -> Self {
        let mut s = Self::zero();
        s.opcode = ADMIN_IDENTIFY;
        s.cid = cid;
        s.prp1 = prp1;
        s.cdw10 = 1; // CNS = Identify Controller
        s
    }

    /// Build an Identify Namespace (CNS=0, NSID=n) admin command.
    pub fn identify_namespace(cid: u16, nsid: u32, prp1: u64) -> Self {
        let mut s = Self::zero();
        s.opcode = ADMIN_IDENTIFY;
        s.cid = cid;
        s.nsid = nsid;
        s.prp1 = prp1;
        s.cdw10 = 0;
        s
    }

    /// Build an I/O Read command. `slba` is the starting LBA (512-byte
    /// sectors), `nlb` is count-1 (NVMe convention: 0 means 1 block).
    pub fn io_read(cid: u16, nsid: u32, prp1: u64, slba: u64, nlb: u16) -> Self {
        let mut s = Self::zero();
        s.opcode = IO_READ;
        s.cid = cid;
        s.nsid = nsid;
        s.prp1 = prp1;
        s.cdw10 = slba as u32;
        s.cdw11 = (slba >> 32) as u32;
        s.cdw12 = nlb as u32;
        s
    }

    /// Build an I/O Write command. See `io_read` for LBA conventions.
    pub fn io_write(cid: u16, nsid: u32, prp1: u64, slba: u64, nlb: u16) -> Self {
        let mut s = Self::zero();
        s.opcode = IO_WRITE;
        s.cid = cid;
        s.nsid = nsid;
        s.prp1 = prp1;
        s.cdw10 = slba as u32;
        s.cdw11 = (slba >> 32) as u32;
        s.cdw12 = nlb as u32;
        s
    }
}

/// 16-byte NVMe Completion Queue Entry.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cqe {
    pub result: u32,     // Command-specific
    pub _rsvd: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub cid: u16,
    pub status: u16,     // Phase bit + status code
}
const _CQE_SIZE_CHECK: () = assert!(core::mem::size_of::<Cqe>() == 16);

impl Cqe {
    /// Phase tag bit — used to detect "new" completion entries in a
    /// ring buffer (NVMe alternates phase on wrap).
    #[inline]
    pub fn phase(&self) -> bool { self.status & 1 != 0 }

    /// Status code field (bits 1..15 of `status`). 0 = success.
    #[inline]
    pub fn status_code(&self) -> u16 { self.status >> 1 }
}

// ─── Queue geometry ─────────────────────────────────────────────────

/// Standard NVMe queue depth for both admin + I/O queues.
pub const QUEUE_DEPTH: usize = 64;
pub const SQ_BYTES: usize = QUEUE_DEPTH * 64;  // 64 × 64 = 4096
pub const CQ_BYTES: usize = QUEUE_DEPTH * 16;  // 64 × 16 = 1024 (page-round up)
pub const IDENTIFY_BYTES: usize = 4096;

/// DMA buffer: a single 4 KiB physical frame that's been DART-mapped
/// into the coprocessor's IOVA space. `phys` is the kernel-side
/// virtual address (identity mapped), `iova` is what the coproc sees.
#[derive(Debug, Clone, Copy)]
pub struct DmaBuf {
    pub phys: usize,
    pub iova: usize,
    pub bytes: usize,
}

/// The set of DMA buffers that make up a live NVMe controller: admin
/// SQ, admin CQ, I/O SQ, I/O CQ, plus an identify response scratch.
#[derive(Debug, Clone, Copy)]
pub struct QueueSet {
    pub admin_sq: DmaBuf,
    pub admin_cq: DmaBuf,
    pub io_sq: DmaBuf,
    pub io_cq: DmaBuf,
    pub identify: DmaBuf,

    /// Next SQ slot to write (tail). Caller increments after submit.
    pub admin_sq_tail: u16,
    pub io_sq_tail: u16,

    /// Next CQ slot to read (head). Increments after completion.
    pub admin_cq_head: u16,
    pub io_cq_head: u16,

    /// Current phase bit to look for in CQEs. Flips on wrap.
    pub admin_cq_phase: bool,
    pub io_cq_phase: bool,

    /// Next CID we'll use (wraps at 16 bits).
    pub next_cid: u16,
}

impl QueueSet {
    pub fn alloc_cid(&mut self) -> u16 {
        let cid = self.next_cid;
        self.next_cid = self.next_cid.wrapping_add(1);
        cid
    }
}

// ─── Controller handle ─────────────────────────────────────────────

#[derive(Debug, Copy, Clone)]
pub struct AnsNvme {
    /// ANS MMIO base (from soc::ans_base()).
    pub base: usize,
    /// ASC mailbox for firmware/power-state control.
    pub asc: Asc,
    /// DART in front of the ANS for DMA.
    pub dart: Dart,
    /// RTKit session on the ASC mailbox.
    pub rtkit: Rtkit,
    pub state: NvmeState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvmeState {
    Idle,
    FirmwareLoaded,
    Identified,
    Ready,
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NvmeError {
    NotReady,
    Timeout,
    BadMagic(u32),
    AscErr(AscError),
    RtkitErr(RtkitError),
    DartErr(super::dart::DartError),
    /// Requires DMA support we don't have yet (DART IOVA + coherent alloc).
    NotImplemented,
}
impl From<AscError> for NvmeError { fn from(e: AscError) -> Self { NvmeError::AscErr(e) } }
impl From<RtkitError> for NvmeError { fn from(e: RtkitError) -> Self { NvmeError::RtkitErr(e) } }
impl From<super::dart::DartError> for NvmeError { fn from(e: super::dart::DartError) -> Self { NvmeError::DartErr(e) } }

// ─── MMIO helpers ──────────────────────────────────────────────────

#[inline]
fn read32(base: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + off) as *const u32) }
}
#[inline]
fn write32(base: usize, off: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u32, val); }
}
#[inline]
fn write_u64(base: usize, off: usize, val: u64) {
    // ASQ/ACQ are 64-bit registers. Write as two 32-bit halves to
    // avoid generating an STR Xn that some NVMe controllers reject.
    write32(base, off, val as u32);
    write32(base, off + 4, (val >> 32) as u32);
}
#[inline]
const fn page_round_up(n: usize) -> usize {
    (n + 4095) & !4095
}

impl AnsNvme {
    /// Construct a handle. No MMIO touched here.
    pub fn new(base: usize, asc: Asc, dart: Dart) -> Self {
        AnsNvme {
            base, asc, dart,
            rtkit: Rtkit::new(asc),
            state: NvmeState::Idle,
        }
    }

    pub fn ready(&self) -> bool {
        self.base >= 0x1000 && self.asc.ready() && self.dart.ready()
    }

    /// Read the ANS-specific boot-status magic. Returns `BadMagic(x)` if
    /// the value isn't 0xDE71CE55. Used to confirm ANS firmware is up
    /// after the RTKit boot handshake.
    pub fn check_boot_status(&self) -> Result<(), NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let v = read32(self.base, ANS_BOOT_STATUS);
        if v != ANS_BOOT_STATUS_OK {
            return Err(NvmeError::BadMagic(v));
        }
        Ok(())
    }

    /// Read standard NVMe CSTS.RDY bit. Controller is ready when this is 1
    /// AFTER CC.EN was set.
    pub fn csts_ready(&self) -> bool {
        if !self.ready() { return false; }
        read32(self.base, REG_CSTS) & CSTS_RDY != 0
    }

    /// Poll CSTS.RDY for up to `iters` iterations.
    pub fn wait_ready(&self, iters: u32) -> Result<(), NvmeError> {
        for _ in 0..iters {
            if self.csts_ready() { return Ok(()); }
            core::hint::spin_loop();
        }
        Err(NvmeError::Timeout)
    }

    /// Run the full ANS bring-up sequence:
    ///   1. DART bypass on the ANS stream (stream 0)
    ///   2. RTKit boot handshake (HELLO/EPMAP/power-on)
    ///   3. Poll ANS_BOOT_STATUS for the 0xDE71CE55 magic
    ///   4. Identify Controller + Namespace 1
    ///   5. Create I/O submission + completion queues
    ///   6. Mark state = Ready
    ///
    /// Steps 4-5 are currently stubbed with `NotImplemented` — they
    /// require a DART IOVA allocator + DMA-coherent frame allocator
    /// that will land in Phase 3.2 + 3.3d.
    pub fn bring_up(&mut self) -> Result<(), NvmeError> {
        if !self.ready() {
            self.state = NvmeState::Failed;
            return Err(NvmeError::NotReady);
        }

        // Step 1: DART in bypass so firmware-load DMA works.
        self.dart.set_bypass(0)?;

        // Step 2: RTKit handshake. Note: ASC CPU start is assumed to be
        // already done by m1n1 (it runs ANS for its own storage probe).
        // If not, we'd need `self.asc.start()?;` here first.
        if self.rtkit.state == super::rtkit::State::Idle {
            self.rtkit.boot()?;
        }

        // Step 3: Firmware is up — confirm via the ANS-specific magic.
        // Give the coproc a bit of grace — firmware sets this register
        // shortly AFTER the RTKit IOP_PWR_STATE ACK.
        let mut ok = false;
        for _ in 0..5_000_000 {
            if read32(self.base, ANS_BOOT_STATUS) == ANS_BOOT_STATUS_OK {
                ok = true;
                break;
            }
            core::hint::spin_loop();
        }
        if !ok {
            self.state = NvmeState::Failed;
            return Err(NvmeError::BadMagic(read32(self.base, ANS_BOOT_STATUS)));
        }
        self.state = NvmeState::FirmwareLoaded;

        // Steps 4-6 require DMA-capable buffers. Stub until DART+IOVA
        // alloc lands. Caller can check `state` to see how far we got.
        Err(NvmeError::NotImplemented)
    }

    // ── Low-level queue primitives (used once DMA lands) ─────────

    /// Enable the NVMe controller: set CC.EN, then wait for CSTS.RDY=1.
    pub fn enable(&self) -> Result<(), NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let cc = read32(self.base, REG_CC);
        write32(self.base, REG_CC, cc | CC_EN);
        self.wait_ready(5_000_000)
    }

    // ── Queue setup + doorbells ─────────────────────────────────

    /// Allocate all DMA buffers, DART-map each one, and populate a
    /// `QueueSet`. Call this after `bring_up()` reaches FirmwareLoaded
    /// state. The returned QueueSet is then passed to every subsequent
    /// admin/IO operation.
    pub fn alloc_queues(
        &self,
        translate: &mut super::dart::TranslateHandle,
    ) -> Result<QueueSet, NvmeError> {
        let asq = self.alloc_dma(translate, SQ_BYTES)?;
        let acq = self.alloc_dma(translate, page_round_up(CQ_BYTES))?;
        let iosq = self.alloc_dma(translate, SQ_BYTES)?;
        let iocq = self.alloc_dma(translate, page_round_up(CQ_BYTES))?;
        let ident = self.alloc_dma(translate, IDENTIFY_BYTES)?;
        Ok(QueueSet {
            admin_sq: asq, admin_cq: acq,
            io_sq: iosq, io_cq: iocq,
            identify: ident,
            admin_sq_tail: 0, io_sq_tail: 0,
            admin_cq_head: 0, io_cq_head: 0,
            admin_cq_phase: true, io_cq_phase: true, // NVMe starts phase=1
            next_cid: 1,
        })
    }

    fn alloc_dma(
        &self,
        translate: &mut super::dart::TranslateHandle,
        bytes: usize,
    ) -> Result<DmaBuf, NvmeError> {
        let pages = (bytes + 4095) / 4096;
        let phys = crate::kernel::mm::frame::alloc_contig(pages)
            .ok_or(NvmeError::NotReady)?;
        let iova = translate.map_region(phys as u64, bytes)
            .map_err(NvmeError::DartErr)?;
        Ok(DmaBuf { phys, iova, bytes })
    }

    /// Write the admin queue base addresses + attributes into the NVMe
    /// registers and enable the controller (CC.EN). Must be called
    /// AFTER `alloc_queues`.
    pub fn configure_admin(&self, qs: &QueueSet) -> Result<(), NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        // AQA: ACQS (bits 27:16) and ASQS (bits 11:0), both depth-1.
        let aqa = (((QUEUE_DEPTH - 1) as u32) << 16) | ((QUEUE_DEPTH - 1) as u32);
        write32(self.base, REG_AQA, aqa);
        // ASQ / ACQ are 64-bit registers — write low then high.
        write_u64(self.base, REG_ASQ, qs.admin_sq.iova as u64);
        write_u64(self.base, REG_ACQ, qs.admin_cq.iova as u64);

        // Enable + wait for ready.
        self.enable()?;
        Ok(())
    }

    /// Write an SQE into the admin submission queue at `qs.admin_sq_tail`
    /// and bump the doorbell. Returns the CID used.
    pub fn submit_admin(&self, qs: &mut QueueSet, mut sqe: Sqe) -> Result<u16, NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let cid = qs.alloc_cid();
        sqe.cid = cid;
        let slot = qs.admin_sq_tail as usize;
        unsafe {
            let base = qs.admin_sq.phys as *mut Sqe;
            core::ptr::write_volatile(base.add(slot), sqe);
        }
        // Make the SQE visible to the coproc BEFORE we ring the doorbell.
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
        qs.admin_sq_tail = ((slot + 1) % QUEUE_DEPTH) as u16;
        // Ring admin-SQ doorbell (standard NVMe offset 0x1000).
        write32(self.base, 0x1000, qs.admin_sq_tail as u32);
        Ok(cid)
    }

    /// Poll the admin completion queue for a CQE with matching CID.
    /// Returns the CQE or Timeout.
    pub fn wait_admin_cqe(&self, qs: &mut QueueSet, cid: u16, iters: u32) -> Result<Cqe, NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        for _ in 0..iters {
            let slot = qs.admin_cq_head as usize;
            let cqe = unsafe {
                let base = qs.admin_cq.phys as *const Cqe;
                core::ptr::read_volatile(base.add(slot))
            };
            // Has the coproc posted a new entry here? Check phase bit.
            if cqe.phase() == qs.admin_cq_phase {
                // Consume this slot.
                qs.admin_cq_head = ((slot + 1) % QUEUE_DEPTH) as u16;
                if qs.admin_cq_head == 0 {
                    qs.admin_cq_phase = !qs.admin_cq_phase;
                }
                // Ring admin-CQ doorbell (standard NVMe offset 0x1004).
                write32(self.base, REG_DB_ACQ, qs.admin_cq_head as u32);
                if cqe.cid == cid {
                    return Ok(cqe);
                }
                // Wrong CID — probably a completion from a previous
                // concurrent admin command. Keep polling.
                continue;
            }
            core::hint::spin_loop();
        }
        Err(NvmeError::Timeout)
    }

    /// Issue Identify Controller (CNS=1) and block for completion.
    /// Response lands in `qs.identify.phys` — 4 KiB of NVMe Identify
    /// Controller data structure.
    pub fn identify_controller(&self, qs: &mut QueueSet) -> Result<(), NvmeError> {
        // Zero the identify buffer so we can see fresh data.
        unsafe {
            core::ptr::write_bytes(qs.identify.phys as *mut u8, 0, qs.identify.bytes);
        }
        let sqe = Sqe::identify_controller(0, qs.identify.iova as u64);
        let cid = self.submit_admin(qs, sqe)?;
        let cqe = self.wait_admin_cqe(qs, cid, 5_000_000)?;
        if cqe.status_code() != 0 {
            return Err(NvmeError::Timeout);
        }
        Ok(())
    }

    /// Issue Identify Namespace (CNS=0, NSID=1).
    pub fn identify_namespace(&self, qs: &mut QueueSet, nsid: u32) -> Result<(), NvmeError> {
        unsafe {
            core::ptr::write_bytes(qs.identify.phys as *mut u8, 0, qs.identify.bytes);
        }
        let sqe = Sqe::identify_namespace(0, nsid, qs.identify.iova as u64);
        let cid = self.submit_admin(qs, sqe)?;
        let cqe = self.wait_admin_cqe(qs, cid, 5_000_000)?;
        if cqe.status_code() != 0 {
            return Err(NvmeError::Timeout);
        }
        Ok(())
    }

    /// Create the I/O completion queue via admin CREATE_CQ (opcode 0x05).
    /// CQ must be created BEFORE the matching SQ.
    pub fn create_io_cq(&self, qs: &mut QueueSet, qid: u16) -> Result<(), NvmeError> {
        let mut sqe = Sqe::zero();
        sqe.opcode = ADMIN_CREATE_CQ;
        sqe.prp1 = qs.io_cq.iova as u64;
        // CDW10: [31:16] = queue-size-1, [15:0] = QID.
        sqe.cdw10 = (((QUEUE_DEPTH - 1) as u32) << 16) | (qid as u32);
        // CDW11: [0] = physically contiguous, interrupts disabled (we poll).
        sqe.cdw11 = QUEUE_CONTIGUOUS;
        let cid = self.submit_admin(qs, sqe)?;
        let cqe = self.wait_admin_cqe(qs, cid, 5_000_000)?;
        if cqe.status_code() != 0 { return Err(NvmeError::Timeout); }
        Ok(())
    }

    /// Create the I/O submission queue via admin CREATE_SQ (opcode 0x01).
    pub fn create_io_sq(&self, qs: &mut QueueSet, sqid: u16, cqid: u16) -> Result<(), NvmeError> {
        let mut sqe = Sqe::zero();
        sqe.opcode = ADMIN_CREATE_SQ;
        sqe.prp1 = qs.io_sq.iova as u64;
        sqe.cdw10 = (((QUEUE_DEPTH - 1) as u32) << 16) | (sqid as u32);
        // CDW11: [31:16] = CQID, [0] = physically contiguous.
        sqe.cdw11 = ((cqid as u32) << 16) | QUEUE_CONTIGUOUS;
        let cid = self.submit_admin(qs, sqe)?;
        let cqe = self.wait_admin_cqe(qs, cid, 5_000_000)?;
        if cqe.status_code() != 0 { return Err(NvmeError::Timeout); }
        Ok(())
    }

    // ── I/O path ──────────────────────────────────────────────

    /// Submit an I/O read. `prp1` is the DART-mapped IOVA of a data
    /// buffer; `slba` is the starting 512-byte LBA; `nlb` is count-1
    /// (hardware uses N-1 encoding).
    ///
    /// Caller must `wait_io_cqe` for completion before touching the
    /// data buffer — DMA hasn't landed yet when submit returns.
    pub fn submit_io_read(
        &self,
        qs: &mut QueueSet,
        prp1: u64,
        slba: u64,
        nlb: u16,
        nsid: u32,
    ) -> Result<u16, NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let cid = qs.alloc_cid();
        let sqe = Sqe::io_read(cid, nsid, prp1, slba, nlb);
        let slot = qs.io_sq_tail as usize;
        unsafe {
            let base = qs.io_sq.phys as *mut Sqe;
            core::ptr::write_volatile(base.add(slot), sqe);
        }
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
        qs.io_sq_tail = ((slot + 1) % QUEUE_DEPTH) as u16;
        // I/O SQ doorbell at 0x1008 (standard: 0x1000 + (1 << DB_STRIDE) × 2qid).
        write32(self.base, 0x1008, qs.io_sq_tail as u32);
        Ok(cid)
    }

    /// Symmetric to `submit_io_read` but for writes.
    pub fn submit_io_write(
        &self,
        qs: &mut QueueSet,
        prp1: u64,
        slba: u64,
        nlb: u16,
        nsid: u32,
    ) -> Result<u16, NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let cid = qs.alloc_cid();
        let sqe = Sqe::io_write(cid, nsid, prp1, slba, nlb);
        let slot = qs.io_sq_tail as usize;
        unsafe {
            let base = qs.io_sq.phys as *mut Sqe;
            core::ptr::write_volatile(base.add(slot), sqe);
        }
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
        qs.io_sq_tail = ((slot + 1) % QUEUE_DEPTH) as u16;
        write32(self.base, 0x1008, qs.io_sq_tail as u32);
        Ok(cid)
    }

    /// Poll the I/O completion queue.
    pub fn wait_io_cqe(&self, qs: &mut QueueSet, cid: u16, iters: u32) -> Result<Cqe, NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        for _ in 0..iters {
            let slot = qs.io_cq_head as usize;
            let cqe = unsafe {
                let base = qs.io_cq.phys as *const Cqe;
                core::ptr::read_volatile(base.add(slot))
            };
            if cqe.phase() == qs.io_cq_phase {
                qs.io_cq_head = ((slot + 1) % QUEUE_DEPTH) as u16;
                if qs.io_cq_head == 0 { qs.io_cq_phase = !qs.io_cq_phase; }
                // I/O CQ doorbell at REG_DB_IOCQ.
                write32(self.base, REG_DB_IOCQ, qs.io_cq_head as u32);
                if cqe.cid == cid { return Ok(cqe); }
                continue;
            }
            core::hint::spin_loop();
        }
        Err(NvmeError::Timeout)
    }

    /// Request normal shutdown: set CC.SHN=NORMAL, wait for SHST=DONE.
    pub fn shutdown(&self) -> Result<(), NvmeError> {
        if !self.ready() { return Err(NvmeError::NotReady); }
        let cc = read32(self.base, REG_CC);
        write32(self.base, REG_CC, cc | CC_SHN_NORMAL);
        for _ in 0..5_000_000 {
            let csts = read32(self.base, REG_CSTS);
            if (csts & 0xc) == CSTS_SHST_DONE {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(NvmeError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqe_is_64_bytes() {
        assert_eq!(core::mem::size_of::<Sqe>(), 64);
    }
    #[test]
    fn cqe_is_16_bytes() {
        assert_eq!(core::mem::size_of::<Cqe>(), 16);
    }
    #[test]
    fn identify_ctrl_sets_cns_1() {
        let s = Sqe::identify_controller(0x42, 0x1000);
        assert_eq!(s.opcode, ADMIN_IDENTIFY);
        assert_eq!(s.cid, 0x42);
        assert_eq!(s.prp1, 0x1000);
        assert_eq!(s.cdw10, 1); // CNS = Identify Controller
    }
    #[test]
    fn io_read_packs_slba_across_cdw10_11() {
        let s = Sqe::io_read(0x1, 0x1, 0x2000, 0x1_0000_0000, 7);
        assert_eq!(s.opcode, IO_READ);
        assert_eq!(s.cdw10, 0);            // low 32 of slba
        assert_eq!(s.cdw11, 1);            // high 32 of slba
        assert_eq!(s.cdw12, 7);            // nlb (count-1 = 7 → 8 blocks)
    }
    #[test]
    fn cqe_phase_and_status_decode() {
        let c = Cqe { result: 0, _rsvd: 0, sq_head: 0, sq_id: 0, cid: 0, status: 0x0003 };
        assert!(c.phase());             // bit 0 set
        assert_eq!(c.status_code(), 1); // bits [15:1] = 1
    }
}
