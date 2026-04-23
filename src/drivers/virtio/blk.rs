//! Bat_OS — VirtIO Block Driver
//!
//! DESIGN.md Phase 3 lists virtio-blk alongside virtio-net / -input /
//! -gpu. Shipping it finally closes that gap. We keep the driver
//! mirror-image to virtio-net's structure so anyone reading one can
//! follow the other: init → probe → queue setup → request submit →
//! poll for completion.
//!
//! Protocol (VirtIO Spec v1.2 §5.2):
//!   Each request is a 3-segment descriptor chain:
//!     1. 16 B  header  — `virtio_blk_outhdr` (type, reserved, sector).
//!                        Readable-from-device.
//!     2. N  B  data    — 512 B sector data. Writable (READ) or
//!                        readable (WRITE) depending on op direction.
//!     3.  1 B  status  — device writes 0=OK / 1=IOERR / 2=UNSUPP.
//!                        Writable.
//!
//! We read config[0..8] at init to pick up the device's sector-count
//! (capacity). That lets callers know how big the disk is before they
//! issue a read past the end.

#![allow(dead_code)]

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, AtomicU64, Ordering};

pub const BLK_SECTOR_SIZE: usize = 512;

// ── Request types (virtio_blk_req.type) ────────────────────────────
pub const VIRTIO_BLK_T_IN:    u32 = 0;   // read device → host
pub const VIRTIO_BLK_T_OUT:   u32 = 1;   // write host → device
pub const VIRTIO_BLK_T_FLUSH: u32 = 4;

// ── Request status (last byte of each request) ────────────────────
pub const VIRTIO_BLK_S_OK:     u8 = 0;
pub const VIRTIO_BLK_S_IOERR:  u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
struct BlkOutHdr {
    ty: u32,
    reserved: u32,
    sector: u64,
}

// ── Driver state ──────────────────────────────────────────────────

static NET_BASE: AtomicUsize = AtomicUsize::new(0);
static QUEUE_ADDR: AtomicUsize = AtomicUsize::new(0);
static READY: AtomicBool = AtomicBool::new(false);
static CAPACITY_SECTORS: AtomicU64 = AtomicU64::new(0);
// Per-request scratch buffers. Single-request-at-a-time protocol is
// enough for Phase 3 use — higher QD needs a pool.
static HDR_BUF: AtomicUsize = AtomicUsize::new(0);
static DATA_BUF: AtomicUsize = AtomicUsize::new(0);
static STATUS_BUF: AtomicUsize = AtomicUsize::new(0);

pub fn init() -> Option<()> {
    let devices = mmio::probe(mmio::DEVICE_BLK);
    let base = devices[0]?;

    uart::puts("  [blk] Found virtio-blk, initializing...\n");
    let device = VirtioMmio::new(base);
    device.init_device().ok()?;

    // Config space layout (offset 0x100):
    //   u64 capacity (in 512-B sectors)
    //   …other fields, only capacity is essential for us.
    let cap_lo = unsafe { core::ptr::read_volatile((base + 0x100) as *const u32) };
    let cap_hi = unsafe { core::ptr::read_volatile((base + 0x104) as *const u32) };
    let capacity = ((cap_hi as u64) << 32) | (cap_lo as u64);
    CAPACITY_SECTORS.store(capacity, Ordering::Relaxed);
    uart::puts("  [blk] capacity: ");
    crate::kernel::mm::print_num(capacity as usize);
    uart::puts(" sectors (");
    crate::kernel::mm::print_num((capacity as usize) / 2);
    uart::puts(" KiB)\n");

    // One virtqueue for blk requests (queue 0).
    let q_mem = frame::alloc_frame()?;
    let q_ptr = q_mem as *mut Virtqueue;
    let q = Virtqueue::new()?;
    unsafe { core::ptr::write(q_ptr, q); }
    QUEUE_ADDR.store(q_mem, Ordering::Relaxed);
    NET_BASE.store(base, Ordering::Relaxed);
    let vq = get_queue();
    device.setup_queue(0, vq);
    device.driver_ok();

    // Scratch buffers (one page each; virtio-blk requests are tiny).
    let hdr = frame::alloc_frame()?;
    let data = frame::alloc_frame()?;
    let status = frame::alloc_frame()?;
    HDR_BUF.store(hdr, Ordering::Relaxed);
    DATA_BUF.store(data, Ordering::Relaxed);
    STATUS_BUF.store(status, Ordering::Relaxed);

    READY.store(true, Ordering::Relaxed);
    uart::puts("  [blk] Block device ready\n");
    Some(())
}

fn get_queue() -> &'static mut Virtqueue {
    unsafe { &mut *(QUEUE_ADDR.load(Ordering::Relaxed) as *mut Virtqueue) }
}

pub fn is_ready() -> bool { READY.load(Ordering::Relaxed) }
pub fn capacity_sectors() -> u64 { CAPACITY_SECTORS.load(Ordering::Relaxed) }

// ── Read / write primitives ───────────────────────────────────────

fn submit_request(ty: u32, sector: u64, data_ptr: usize, data_len: usize,
                  data_writable: bool) -> Result<(), &'static str>
{
    if !is_ready() { return Err("blk not ready"); }
    if data_len == 0 || data_len % BLK_SECTOR_SIZE != 0 {
        return Err("data_len must be a non-zero multiple of 512");
    }
    let hdr_buf = HDR_BUF.load(Ordering::Relaxed);
    let status_buf = STATUS_BUF.load(Ordering::Relaxed);
    if hdr_buf == 0 || status_buf == 0 { return Err("blk buffers not allocated"); }

    // Build header.
    let hdr = BlkOutHdr { ty, reserved: 0, sector };
    unsafe {
        core::ptr::write_volatile(hdr_buf as *mut BlkOutHdr, hdr);
        // Pre-clear status so we can tell OK from "never written".
        core::ptr::write_volatile(status_buf as *mut u8, 0xFF);
    }

    let vq = get_queue();
    let base = NET_BASE.load(Ordering::Relaxed);
    let device = VirtioMmio::new(base);
    let submitted = vq.add_chain3(
        hdr_buf as *const u8, core::mem::size_of::<BlkOutHdr>() as u32,
        data_ptr as *mut u8, data_len as u32, data_writable,
        status_buf as *mut u8, 1, true,
    );
    if submitted.is_none() { return Err("virtqueue submit failed"); }
    device.notify(0);

    // Poll for completion.
    let mut spins: u32 = 0;
    while vq.poll_used().is_none() {
        spins = spins.saturating_add(1);
        if spins > 5_000_000 { return Err("blk request timeout"); }
        core::hint::spin_loop();
    }

    let status = unsafe { core::ptr::read_volatile(status_buf as *const u8) };
    match status {
        VIRTIO_BLK_S_OK     => Ok(()),
        VIRTIO_BLK_S_IOERR  => Err("blk IO error"),
        VIRTIO_BLK_S_UNSUPP => Err("blk unsupported op"),
        _ => Err("blk: unexpected status"),
    }
}

/// Read `buf.len()` bytes starting at logical-sector `sector`.
/// buf.len() must be a multiple of 512. Bounds are enforced with
/// checked_add so a malicious `sector` near u64::MAX can't wrap
/// through the capacity check.
pub fn read_sectors(sector: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    if buf.len() > 4096 { return Err("blk read too large (buffer cap)"); }
    let end_sector = sector.checked_add((buf.len() / BLK_SECTOR_SIZE) as u64)
        .ok_or("blk read sector overflow")?;
    if end_sector > capacity_sectors() {
        return Err("blk read past end of device");
    }
    let data_buf = DATA_BUF.load(Ordering::Relaxed);
    if data_buf == 0 { return Err("blk data buffer not allocated"); }
    submit_request(VIRTIO_BLK_T_IN, sector, data_buf, buf.len(), true)?;
    // Copy from scratch into the caller's buffer.
    for i in 0..buf.len() {
        buf[i] = unsafe { core::ptr::read_volatile((data_buf + i) as *const u8) };
    }
    Ok(())
}

/// Write `buf.len()` bytes starting at logical-sector `sector`.
pub fn write_sectors(sector: u64, buf: &[u8]) -> Result<(), &'static str> {
    if buf.len() > 4096 { return Err("blk write too large (buffer cap)"); }
    let end_sector = sector.checked_add((buf.len() / BLK_SECTOR_SIZE) as u64)
        .ok_or("blk write sector overflow")?;
    if end_sector > capacity_sectors() {
        return Err("blk write past end of device");
    }
    let data_buf = DATA_BUF.load(Ordering::Relaxed);
    if data_buf == 0 { return Err("blk data buffer not allocated"); }
    // Copy into scratch.
    for i in 0..buf.len() {
        unsafe { core::ptr::write_volatile((data_buf + i) as *mut u8, buf[i]); }
    }
    submit_request(VIRTIO_BLK_T_OUT, sector, data_buf, buf.len(), false)
}

pub fn flush() -> Result<(), &'static str> {
    let data_buf = DATA_BUF.load(Ordering::Relaxed);
    // Spec allows a dummy data segment for FLUSH (many devices don't
    // use it). We submit one sector of scratch as readable.
    submit_request(VIRTIO_BLK_T_FLUSH, 0, data_buf, BLK_SECTOR_SIZE, false)
}

// ── Self-test ─────────────────────────────────────────────────────

pub struct BlkReport {
    pub ready: bool,
    pub capacity_sectors: u64,
    pub write_ok: bool,
    pub readback_ok: bool,
    pub first_byte: u8,
}

/// Round-trip one sector: write a pattern, read it back, verify.
/// Touches sector 42 (arbitrary, well into the device) so we don't
/// collide with partition tables or other metadata the host might
/// have put in sector 0.
pub fn selftest() -> Result<BlkReport, &'static str> {
    if !is_ready() {
        return Ok(BlkReport {
            ready: false, capacity_sectors: 0,
            write_ok: false, readback_ok: false, first_byte: 0,
        });
    }
    let cap = capacity_sectors();
    if cap < 64 { return Err("blk device too small for selftest"); }

    let mut pattern = [0u8; BLK_SECTOR_SIZE];
    for i in 0..BLK_SECTOR_SIZE {
        pattern[i] = ((i as u32 * 37 + 7) & 0xFF) as u8;
    }
    write_sectors(42, &pattern)?;
    let mut read_back = [0u8; BLK_SECTOR_SIZE];
    read_sectors(42, &mut read_back)?;
    let ok = read_back == pattern;
    Ok(BlkReport {
        ready: true,
        capacity_sectors: cap,
        write_ok: true,
        readback_ok: ok,
        first_byte: read_back[0],
    })
}
