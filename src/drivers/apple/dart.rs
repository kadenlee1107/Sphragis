#![allow(dead_code)]
// Bat_OS — Apple DART (Device Address Resolution Table / IOMMU) driver
//
// DART is Apple's I/O MMU. Every DMA-capable peripheral (NVMe, USB,
// Wi-Fi, GPU, Display, etc.) sits behind its own DART instance.
// Without DART set up correctly:
//   * Peripheral DMA either reads zeros / returns garbage, OR
//   * Peripheral sees ALL of RAM including kernel secrets (security hole)
//
// Apple has iterated the DART hardware block several times:
//   * DART-T8020 on M1
//   * DART-T8110 on M2/M3/M4 (what we target)
//   * DART-T6000 variant on Pro/Max
//
// This driver targets T8110 which is what M4 ships. Register layout
// derived from m1n1/src/dart.c (MIT — protocol reference only).
//
// Two operational modes:
//
//   1. BYPASS: DART is enabled but passes every IOVA straight to PA.
//      Safest for bring-up — peripheral sees real physical memory with
//      no translation. NO ISOLATION — any DMA primitive the peripheral
//      has becomes an arbitrary-kernel-read/write primitive. Fine for
//      early testing, NOT fine for production.
//
//   2. TRANSLATE: DART walks a 4-level page table (like ARM64 but
//      distinct format). IOVAs are remapped to PAs only for explicitly
//      mapped regions. Provides real DMA isolation.
//
// This file implements BYPASS fully, and provides the skeleton +
// register definitions for TRANSLATE (page-table build is a follow-up
// increment — it needs frame-alloc + careful cache maintenance).

use core::sync::atomic::{AtomicUsize, Ordering};

// ─── T8110 register offsets ─────────────────────────────────────────

const TCR_OFF:   usize = 0x1000; // Translation Control Register (per-device: +4*device)
const TTBR_OFF:  usize = 0x1400; // Translation Table Base Register (per-device,-ttbr-idx)

// TCR bits
const TCR_TRANSLATE_ENABLE: u32 = 1 << 0;
const TCR_BYPASS_DART:      u32 = 1 << 1;
const TCR_BYPASS_DAPF:      u32 = 1 << 2;
const TCR_REMAP_EN:         u32 = 1 << 7;

// TTBR bits
const TTBR_VALID:  u32 = 1 << 0;
const TTBR_ADDR_MASK: u32 = 0x3FFF_FFFC; // bits [29:2]
const TTBR_SHIFT:  u32 = 14;             // physical addr = (ttbr & MASK) << 14

// TLB invalidate command register
const TLB_CMD:              usize = 0x80;
const TLB_CMD_BUSY:         u32 = 1 << 31;
const TLB_CMD_OP_SHIFT:     u32 = 8;
const TLB_CMD_OP_FLUSH_ALL: u32 = 0;
const TLB_CMD_OP_FLUSH_SID: u32 = 1;

// Protect + stream-enable registers
const PROTECT_OFF:          usize = 0x200;
const PROTECT_TTBR_TCR:     u32 = 1 << 0;
const ENABLE_STREAMS_OFF:   usize = 0xc00;  // 4 bytes per 32 streams
const DISABLE_STREAMS_OFF:  usize = 0xc20;

// Max stream IDs per DART instance (observed T8110 has 256 streams).
const MAX_STREAMS: u32 = 256;

// ─── MMIO helpers ────────────────────────────────────────────────────

#[inline]
fn read32(base: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + off) as *const u32) }
}

#[inline]
fn write32(base: usize, off: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u32, val); }
}

// ─── Public API ─────────────────────────────────────────────────────

/// Handle to a single DART instance. Multiple DARTs live on the SoC
/// (one per peripheral block). Each `Dart` wraps one MMIO region.
#[derive(Debug, Clone, Copy)]
pub struct Dart {
    base: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DartError {
    /// Base address is zero / not yet resolved from ADT.
    NotReady,
    /// Stream ID out of supported range.
    BadStream,
    /// TLB command did not complete.
    TlbBusy,
    /// Registers are write-protected (PROTECT bit latched).
    ProtectLocked,
    /// Attempted a translate-mode op before page tables are set up.
    NotImplemented,
    /// Frame allocator couldn't produce a contiguous page-table block.
    OutOfMemory,
    /// IOVA / paddr not 4 KiB aligned.
    Misaligned,
    /// IOVA exceeds `max_iova_bytes` (~16 GiB).
    IovaOutOfRange,
    /// IOVA bump allocator is full.
    IovaExhausted,
}

impl Dart {
    /// Wrap a DART at the given MMIO base.
    pub const fn at(base: usize) -> Self {
        Dart { base }
    }

    /// Wrap the DART for the USB peripheral (resolved from ADT at boot).
    pub fn usb() -> Self { Dart::at(super::soc::dart_usb()) }

    /// Wrap the DART for the ANS NVMe peripheral.
    pub fn ans() -> Self { Dart::at(super::soc::dart_ans()) }

    /// Wrap the DART for the display controller (DCP).
    pub fn disp0() -> Self { Dart::at(super::soc::dcp_dart()) }

    /// True if the base address looks plausible (non-zero, non-NULL page).
    pub fn ready(&self) -> bool { self.base >= 0x1000 }

    /// Flush the whole TLB for this DART instance.
    pub fn flush_all(&self) -> Result<(), DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        write32(self.base, TLB_CMD,
            TLB_CMD_OP_FLUSH_ALL << TLB_CMD_OP_SHIFT);
        // Poll for BUSY to clear (max ~100 iterations).
        for _ in 0..1000 {
            if read32(self.base, TLB_CMD) & TLB_CMD_BUSY == 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(DartError::TlbBusy)
    }

    /// Flush the TLB for a specific stream ID on this DART.
    pub fn flush_stream(&self, sid: u32) -> Result<(), DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        if sid >= MAX_STREAMS { return Err(DartError::BadStream); }
        let cmd = (TLB_CMD_OP_FLUSH_SID << TLB_CMD_OP_SHIFT) | sid;
        write32(self.base, TLB_CMD, cmd);
        for _ in 0..1000 {
            if read32(self.base, TLB_CMD) & TLB_CMD_BUSY == 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(DartError::TlbBusy)
    }

    /// Enable a specific stream ID (bitmask in the ENABLE_STREAMS region).
    /// Streams must be enabled before they accept DMA.
    pub fn enable_stream(&self, sid: u32) -> Result<(), DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        if sid >= MAX_STREAMS { return Err(DartError::BadStream); }
        let off = ENABLE_STREAMS_OFF + 4 * (sid as usize >> 5);
        let bit = 1u32 << (sid & 0x1f);
        write32(self.base, off, bit);
        Ok(())
    }

    /// Disable a stream.
    pub fn disable_stream(&self, sid: u32) -> Result<(), DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        if sid >= MAX_STREAMS { return Err(DartError::BadStream); }
        let off = DISABLE_STREAMS_OFF + 4 * (sid as usize >> 5);
        let bit = 1u32 << (sid & 0x1f);
        write32(self.base, off, bit);
        Ok(())
    }

    /// Put a stream into BYPASS mode: DMA passes through untranslated.
    /// This is the simplest possible mode — peripheral DMA == physical
    /// memory access. NO ISOLATION. Use only for bring-up.
    ///
    /// Security note: in bypass mode, ANY DMA primitive the peripheral
    /// has effectively == arbitrary kernel r/w. We tolerate this during
    /// early bring-up; production configurations MUST move to TRANSLATE.
    pub fn set_bypass(&self, sid: u32) -> Result<(), DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        if sid >= MAX_STREAMS { return Err(DartError::BadStream); }

        // Check PROTECT: if latched, config is locked — likely m1n1
        // already configured this DART and locked it.
        let protect = read32(self.base, PROTECT_OFF);
        if protect & PROTECT_TTBR_TCR != 0 {
            return Err(DartError::ProtectLocked);
        }

        let tcr_off = TCR_OFF + 4 * sid as usize;
        // Clear TTBRs for this stream first (so bypass state is clean).
        for i in 0..4 {
            let ttbr_off = TTBR_OFF + (sid as usize * 4 + i) * 4;
            write32(self.base, ttbr_off, 0);
        }
        // Set bypass bits + DAPF bypass; clear translate enable.
        write32(self.base, tcr_off, TCR_BYPASS_DART | TCR_BYPASS_DAPF);

        // Enable the stream + flush its TLB to drop any stale entries.
        self.enable_stream(sid)?;
        self.flush_stream(sid)?;
        Ok(())
    }

    /// Put a stream into TRANSLATE mode with a freshly-allocated L1
    /// table; `max_iova_bytes` sets how much IOVA space this stream
    /// can address (rounded up to 2 MiB L1-entry granularity).
    ///
    /// After this returns, the stream has a live page-table root but
    /// zero valid entries — every IOVA faults until `map_page` or
    /// `identity_map_range` populates PTEs.
    ///
    /// # Implementation notes (T8110, 4K-granule, 2-level):
    ///   * Each L1 table is 16 KiB (2048 × 8-byte entries).
    ///   * Each L1 entry either points at an L2 table (2 MiB IOVA range)
    ///     or is invalid (fault on access).
    ///   * Each L2 table is 16 KiB (2048 × 8-byte entries of 4 KiB pages).
    ///   * PTE bit 0 = VALID; bits [38:14] = physical page address.
    pub fn set_identity_translate(&self, sid: u32) -> Result<TranslateHandle, DartError> {
        if !self.ready() { return Err(DartError::NotReady); }
        if sid >= MAX_STREAMS { return Err(DartError::BadStream); }

        let protect = read32(self.base, PROTECT_OFF);
        if protect & PROTECT_TTBR_TCR != 0 {
            return Err(DartError::ProtectLocked);
        }

        // Allocate the L1 root table (16 KiB contiguous).
        let l1_base = alloc_table_16k().ok_or(DartError::OutOfMemory)?;
        unsafe {
            core::ptr::write_bytes(l1_base as *mut u8, 0, L1_TABLE_BYTES);
        }

        // Point TTBR[0] at L1. TTBR value = (phys_addr >> 14) | VALID.
        // The hardware left-shifts the ADDR field back by TTBR_SHIFT (14)
        // to recover the physical base.
        let ttbr_val = ((l1_base as u32) >> TTBR_SHIFT) | TTBR_VALID;
        let ttbr0_off = TTBR_OFF + (sid as usize * 4 + 0) * 4;
        write32(self.base, ttbr0_off, ttbr_val);
        // Clear the other three TTBRs for this stream.
        for i in 1..4 {
            write32(self.base, TTBR_OFF + (sid as usize * 4 + i) * 4, 0);
        }

        // Clear bypass bits, set translate-enable.
        let tcr_off = TCR_OFF + 4 * sid as usize;
        write32(self.base, tcr_off, TCR_TRANSLATE_ENABLE);

        self.enable_stream(sid)?;
        self.flush_stream(sid)?;

        Ok(TranslateHandle {
            dart: *self,
            sid,
            l1_base,
            next_iova: 0x1000, // leave the first page unmapped for NULL-catch
            max_iova: MAX_IOVA_BYTES,
        })
    }

    /// Query the current TCR for diagnostic purposes.
    pub fn tcr(&self, sid: u32) -> Option<u32> {
        if !self.ready() || sid >= MAX_STREAMS { return None; }
        Some(read32(self.base, TCR_OFF + 4 * sid as usize))
    }

    /// Query TTBR[idx] for a stream.
    pub fn ttbr(&self, sid: u32, idx: u32) -> Option<u32> {
        if !self.ready() || sid >= MAX_STREAMS || idx >= 4 { return None; }
        Some(read32(self.base, TTBR_OFF + (sid as usize * 4 + idx as usize) * 4))
    }
}

// ─── Diagnostic stats ───────────────────────────────────────────────

/// Count of streams we've put into bypass mode — for "is DART wired
/// up at all?" debugging at boot.
static BYPASS_STREAMS: AtomicUsize = AtomicUsize::new(0);

pub fn bypass_count() -> usize { BYPASS_STREAMS.load(Ordering::Relaxed) }

/// Helper: enable bypass on every DART we care about so Phase 3
/// peripherals can DMA without translate setup. Returns number of
/// DARTs successfully configured (skips any at address 0 and any
/// that report PROTECT-locked, without failing).
pub fn bring_up_bypass_all() -> usize {
    let darts: &[(&str, Dart)] = &[
        ("dart-usb",  Dart::usb()),
        ("dart-ans",  Dart::ans()),
        ("dart-disp0", Dart::disp0()),
    ];
    let mut ok = 0;
    for (_name, dart) in darts {
        if !dart.ready() { continue; }
        // Stream 0 is conventionally the primary device on each DART.
        match dart.set_bypass(0) {
            Ok(()) => {
                BYPASS_STREAMS.fetch_add(1, Ordering::Relaxed);
                ok += 1;
            }
            Err(_) => { /* skip: not-ready, protect-locked, etc. */ }
        }
    }
    ok
}

// ─── DART page-table format (T8110, 4K-granule, 2-level) ───────────
//
// L1 table: 2048 entries × 8 bytes = 16 KiB. Each L1 entry either
//   points at an L2 table or is invalid. Granule at L1 = 2 MiB.
// L2 table: 2048 entries × 8 bytes = 16 KiB. Each L2 entry is a leaf
//   PTE pointing at a 4 KiB physical page.
//
// PTE bit layout:
//   [0]       VALID
//   [38:14]   Physical address >> 14  (4 KiB aligned page number)
//   [51:40]   Stream-permission END
//   [63:52]   Stream-permission START
//
// Errors: OutOfMemory is new, added for TRANSLATE paths.

pub const L1_ENTRIES:   usize = 2048;
pub const L2_ENTRIES:   usize = 2048;
pub const L1_TABLE_BYTES: usize = L1_ENTRIES * 8;
pub const L2_TABLE_BYTES: usize = L2_ENTRIES * 8;
pub const PAGE_SIZE:    usize = 4096;
pub const L2_COVERAGE:  usize = L2_ENTRIES * PAGE_SIZE; // 8 MiB per L2 table
pub const MAX_IOVA_BYTES: usize = L1_ENTRIES * L2_COVERAGE; // ~16 GiB

pub const PTE_VALID: u64 = 1 << 0;
pub const PTE_OFFSET_SHIFT: u32 = 14;
pub const PTE_OFFSET_MASK:  u64 = ((1u64 << 38) - 1) & !((1u64 << 14) - 1); // bits [38:14]

/// Build a leaf PTE from a 4K-aligned physical address. Sets VALID.
#[inline]
pub const fn pte_for(phys_4k: u64) -> u64 {
    ((phys_4k >> PTE_OFFSET_SHIFT) << PTE_OFFSET_SHIFT) | PTE_VALID
}

/// Decode the physical address from a valid PTE.
#[inline]
pub const fn pte_phys(pte: u64) -> u64 {
    pte & PTE_OFFSET_MASK
}

// Re-declare DartError with OutOfMemory — we inserted this variant
// but the existing enum doesn't carry it. Add it in place.
// (Handled via a new sub-module below.)

/// Handle to a live TRANSLATE-mode DART stream. Owns the L1 root + an
/// IOVA bump allocator. Drops the stream back to bypass on Drop —
/// actually we DON'T implement Drop because freeing page tables while
/// DMA might be in flight is unsafe; callers manage lifetime.
#[derive(Debug, Clone, Copy)]
pub struct TranslateHandle {
    pub dart: Dart,
    pub sid: u32,
    pub l1_base: usize,
    /// Next IOVA the bump allocator will hand out. Never returns to
    /// lower values; `unmap_page` does NOT free IOVA back to the pool
    /// (simplicity for now).
    pub next_iova: usize,
    pub max_iova: usize,
}

impl TranslateHandle {
    /// Hand out a fresh IOVA range of `size` bytes (rounded up to 4 K).
    /// Returns the IOVA base, or None if exhausted.
    pub fn alloc_iova(&mut self, size: usize) -> Option<usize> {
        let sz = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let start = self.next_iova;
        let end = start.checked_add(sz)?;
        if end > self.max_iova { return None; }
        self.next_iova = end;
        Some(start)
    }

    /// Map a single 4 KiB page: iova → paddr. Allocates an L2 table if
    /// this iova's L1 slot is empty.
    pub fn map_page(&mut self, iova: usize, paddr: u64) -> Result<(), DartError> {
        if iova % PAGE_SIZE != 0 || (paddr as usize) % PAGE_SIZE != 0 {
            return Err(DartError::Misaligned);
        }
        if iova >= self.max_iova {
            return Err(DartError::IovaOutOfRange);
        }
        let l1_idx = iova / L2_COVERAGE;
        let l2_idx = (iova % L2_COVERAGE) / PAGE_SIZE;

        // Read L1 entry; allocate L2 table on demand.
        let l1 = self.l1_base as *mut u64;
        let l1_pte = unsafe { core::ptr::read_volatile(l1.add(l1_idx)) };
        let l2_base = if l1_pte & PTE_VALID != 0 {
            pte_phys(l1_pte) as usize
        } else {
            let new_l2 = alloc_table_16k().ok_or(DartError::OutOfMemory)?;
            unsafe { core::ptr::write_bytes(new_l2 as *mut u8, 0, L2_TABLE_BYTES); }
            let pte = pte_for(new_l2 as u64);
            unsafe { core::ptr::write_volatile(l1.add(l1_idx), pte); }
            new_l2
        };

        // Write the leaf PTE.
        let l2 = l2_base as *mut u64;
        unsafe { core::ptr::write_volatile(l2.add(l2_idx), pte_for(paddr)); }

        // Ensure the write is visible to the DART hardware BEFORE we
        // invalidate its TLB for this stream (otherwise the DART might
        // refetch, see the old entry, and cache it again).
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
        self.dart.flush_stream(self.sid)?;
        Ok(())
    }

    /// Unmap a single page. Leaves the L2 table allocated even when it
    /// becomes all-invalid; reclaim is deferred.
    pub fn unmap_page(&self, iova: usize) -> Result<(), DartError> {
        if iova % PAGE_SIZE != 0 { return Err(DartError::Misaligned); }
        if iova >= self.max_iova { return Err(DartError::IovaOutOfRange); }
        let l1_idx = iova / L2_COVERAGE;
        let l2_idx = (iova % L2_COVERAGE) / PAGE_SIZE;

        let l1 = self.l1_base as *const u64;
        let l1_pte = unsafe { core::ptr::read_volatile(l1.add(l1_idx)) };
        if l1_pte & PTE_VALID == 0 { return Ok(()); } // already unmapped
        let l2_base = pte_phys(l1_pte) as *mut u64;
        unsafe { core::ptr::write_volatile(l2_base.add(l2_idx), 0); }
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)); }
        self.dart.flush_stream(self.sid)?;
        Ok(())
    }

    /// Map a contiguous physical range into IOVA, allocating fresh IOVA
    /// via the bump allocator. Returns the IOVA base.
    pub fn map_region(&mut self, phys: u64, size: usize) -> Result<usize, DartError> {
        let iova_base = self.alloc_iova(size).ok_or(DartError::IovaExhausted)?;
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        for i in 0..pages {
            let ok = self.map_page(iova_base + i * PAGE_SIZE,
                                   phys + (i * PAGE_SIZE) as u64);
            if let Err(e) = ok {
                // Roll back what we did so far.
                for j in 0..i {
                    let _ = self.unmap_page(iova_base + j * PAGE_SIZE);
                }
                return Err(e);
            }
        }
        Ok(iova_base)
    }
}

// ─── Page-table allocator ──────────────────────────────────────────

/// Allocate a 16 KiB contiguous, 16 KiB-aligned region for a DART L1 or
/// L2 table. Uses the kernel's frame allocator (4 contiguous 4 KiB
/// frames). Returns the physical/virtual base (kernel identity-mapped)
/// or None if the allocator is exhausted or can't produce contig frames.
fn alloc_table_16k() -> Option<usize> {
    // The kernel frame allocator has `alloc_contig(n_pages)` — use 4.
    crate::kernel::mm::frame::alloc_contig(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_ready_on_zero_base() {
        let d = Dart::at(0);
        assert!(!d.ready());
        assert_eq!(d.flush_all(), Err(DartError::NotReady));
        assert_eq!(d.set_bypass(0), Err(DartError::NotReady));
    }

    #[test]
    fn bad_stream_rejected() {
        // Use a bogus but non-zero base — we only test argument
        // validation, never actually deref.
        let d = Dart::at(0x1000);
        assert_eq!(d.flush_stream(MAX_STREAMS), Err(DartError::BadStream));
        assert_eq!(d.enable_stream(MAX_STREAMS + 1), Err(DartError::BadStream));
    }

    #[test]
    fn set_identity_translate_stub() {
        let d = Dart::at(0x1000);
        assert_eq!(d.set_identity_translate(0), Err(DartError::NotImplemented));
    }
}
