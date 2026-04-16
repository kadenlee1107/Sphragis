// Bat_OS — Production ELF Loader
// Handles real-world static ARM64 Linux binaries (like busybox).

use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE_SIZE: usize = 4096;

static LOADED_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static SAVED_RETURN_ADDR: AtomicUsize = AtomicUsize::new(0);
static SAVED_KERNEL_SP: AtomicUsize = AtomicUsize::new(0);

// Worker busybox instance (second copy for child applet execution)
pub static WORKER_ENTRY: AtomicUsize = AtomicUsize::new(0);
pub static WORKER_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static WORKER_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);

// Hello binary instance (standalone ELF, loaded on demand)
pub static HELLO_ENTRY: AtomicUsize = AtomicUsize::new(0);
pub static HELLO_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static HELLO_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);

/// Re-initialize a previously loaded ELF at the given phys_base.
/// Re-copies all PT_LOAD segments and re-applies relocations.
/// Does NOT allocate new pages — reuses existing allocation.
pub fn reinit_elf(data: &[u8], phys_base: usize) {
    if data.len() < 64 { return; }

    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    let mut min_addr: u64 = u64::MAX;
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        if vaddr < min_addr { min_addr = vaddr; }
    }

    let reloc_offset = phys_base as i64 - min_addr as i64;

    // Re-copy PT_LOAD segments (data + zero BSS)
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        let phys_addr = (vaddr as i64 + reloc_offset) as usize;

        // FL-001: guard p_offset + filesz against usize wrap before the copy.
        let copy_ok = filesz > 0 && match p_offset.checked_add(filesz) {
            Some(end) => end <= data.len(),
            None => false,
        };
        if copy_ok {
            // Bulk copy — a byte-by-byte strb loop would take minutes on
            // a 150 MB Chromium binary (≈157 M inline-asm round-trips).
            // copy_nonoverlapping lowers to a tuned memcpy that coalesces
            // stores and runs 30×+ faster.
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(p_offset),
                    phys_addr as *mut u8,
                    filesz,
                );
            }
        }
        if memsz > filesz {
            unsafe {
                core::ptr::write_bytes(
                    (phys_addr + filesz) as *mut u8,
                    0,
                    memsz - filesz,
                );
            }
        }
    }

    // Re-apply relocations
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;

        let mut pos = dyn_offset;
        while pos + 16 <= data.len() && pos < dyn_offset + dyn_size {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag { 0 => break, 7 => rela_off = val as usize, 8 => rela_sz = val as usize, _ => {} }
            pos += 16;
        }

        if rela_off > 0 && rela_sz > 0 {
            // FL-003: cap num_relas at 50M — anything more is hostile.
            let num = (rela_sz / 24).min(50_000_000);
            // reinit path reuses the original allocation; recompute the
            // authoritative range from phys_base + total loaded size.
            // Since we don't track total_size here, derive from phdrs.
            let mut lo: u64 = u64::MAX;
            let mut hi: u64 = 0;
            for j in 0..phnum {
                let ph2 = phoff + j * phentsz;
                if ph2 + phentsz > data.len() { break; }
                if u32_at(data, ph2) != 1 { continue; }
                let va = u64_at(data, ph2 + 16);
                let ms = u64_at(data, ph2 + 40);
                if va < lo { lo = va; }
                if let Some(end) = va.checked_add(ms) {
                    if end > hi { hi = end; }
                }
            }
            let phys_range_size = (hi.saturating_sub(lo)) as usize;
            let phys_range_end = phys_base.saturating_add(phys_range_size);
            for r in 0..num {
                let re = match rela_off.checked_add(r.checked_mul(24).unwrap_or(usize::MAX)) {
                    Some(v) => v,
                    None => break,
                };
                match re.checked_add(24) {
                    Some(end) if end <= data.len() => {}
                    _ => break,
                }
                let r_offset = u64_at(data, re);
                let r_info = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                if (r_info & 0xFFFFFFFF) as u32 == 0x403 {
                    let patch_addr = (r_offset as i64 + reloc_offset) as usize;
                    let value = (r_addend as i64 + reloc_offset) as u64;
                    // FL-004: reject writes outside the ELF's allocated range.
                    let patch_end = match patch_addr.checked_add(8) {
                        Some(v) => v,
                        None => continue,
                    };
                    if patch_addr < phys_base || patch_end > phys_range_end {
                        continue;
                    }
                    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) patch_addr, v = in(reg) value); }
                }
            }
        }
    }
}

pub fn get_phys_base() -> usize { LOADED_PHYS_BASE.load(Ordering::Relaxed) }
pub fn get_orig_entry() -> usize { LOADED_ORIG_ENTRY.load(Ordering::Relaxed) }
pub fn get_entry() -> usize { LOADED_ENTRY.load(Ordering::Relaxed) }
pub fn set_phys_base(v: usize) { LOADED_PHYS_BASE.store(v, Ordering::Relaxed); }
pub fn set_orig_entry(v: usize) { LOADED_ORIG_ENTRY.store(v, Ordering::Relaxed); }
pub fn set_entry(v: usize) { LOADED_ENTRY.store(v, Ordering::Relaxed); }

/// ELF loaded into a rebased virtual window. Returned by `load_elf_rebased`
/// so the runner can call `setup_cave_pagetable_at(slot, phys_base, virt_base)`
/// and then `switch_to_cave` / `execute_with_args(virt_entry, ...)`.
#[derive(Clone, Copy, Debug)]
pub struct LoadedElfInfo {
    /// Virtual entry point (virt_base + entry - min_addr).
    pub virt_entry: u64,
    /// Physical base the frames were allocated at.
    pub phys_base: usize,
    /// Total bytes reserved (max_addr - min_addr, rounded up).
    pub total_size: usize,
    /// Virtual base passed by the caller (for reference).
    pub virt_base: u64,
}

/// FLv2-NEW-017/018 fix: free every page that `load_elf_rebased` allocated.
/// Without this, ~150 MB of Chromium pages leaked permanently per cave
/// teardown — caves got destroyed, the `Cave` struct cleared, but the
/// underlying frames were never returned to the bitmap.
///
/// `frame::free_contig` zeroes each page on free, so the residue from
/// the previous tenant is wiped before the frames re-enter the pool.
pub fn free_loaded_elf(info: &LoadedElfInfo) {
    if info.phys_base == 0 || info.total_size == 0 { return; }
    let pages = (info.total_size + 4095) / 4096;
    crate::kernel::mm::frame::free_contig(info.phys_base, pages);
}

/// Load a PIE ELF with virtual addresses rebased to `virt_base`.
///
/// Unlike `load_elf` (which treats phys = virt, identity-mapped), this
/// function applies relocations so the binary's internal addresses
/// reference the cave's VA window rather than raw physical addresses.
/// Pair with `setup_cave_pagetable_at(slot, info.phys_base, virt_base)`
/// and `switch_to_cave(l1)` to actually mount the cave before
/// `execute_with_args(info.virt_entry, argv)`.
pub fn load_elf_rebased(data: &[u8], virt_base: u64) -> Result<LoadedElfInfo, &'static str> {
    if data.len() < 64 { return Err("too small"); }
    if &data[0..4] != b"\x7fELF" { return Err("not ELF"); }
    if virt_base & 0x1FFFFF != 0 { return Err("virt_base not 2MB aligned"); }

    let entry = u64_at(data, 24);
    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    uart::puts("[loader] Rebased Entry: 0x"); print_hex(entry);
    uart::puts(" virt_base: 0x"); print_hex(virt_base); uart::puts("\n");

    // Scan program headers for min/max vaddr.
    let mut min_addr: u64 = u64::MAX;
    let mut max_addr: u64 = 0;
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        let memsz = u64_at(data, ph + 40);
        if vaddr < min_addr { min_addr = vaddr; }
        let seg_end = vaddr.checked_add(memsz).ok_or("PT_LOAD vaddr+memsz overflow")?;
        if seg_end > max_addr { max_addr = seg_end; }
    }
    if min_addr == u64::MAX { return Err("no PT_LOAD segments"); }

    let total_size = (max_addr - min_addr) as usize;
    let total_pages = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;

    // Same 2MB-aligned contiguous-alloc pattern as load_elf.
    let mut phys_base = frame::alloc_frame().ok_or("oom")?;
    while phys_base & 0x1FFFFF != 0 {
        phys_base = frame::alloc_frame().ok_or("oom")?;
    }
    for i in 1..total_pages {
        let expected = phys_base + i * PAGE_SIZE;
        let got = frame::alloc_frame().ok_or("oom")?;
        if got != expected {
            return Err("non-contiguous alloc (memory fragmented)");
        }
    }

    uart::puts("[loader] Rebased phys_base: 0x"); print_hex(phys_base as u64); uart::puts("\n");

    // Two offsets:
    //   patch_offset — where we WRITE during load (physical, identity-mapped)
    //   value_offset — what VALUE we write in relocations (the VA the binary
    //                  will see at runtime, once switch_to_cave installs the
    //                  cave's TTBR0 mapping virt_base..virt_base+total_size
    //                  to phys_base..phys_base+total_size)
    let patch_offset: i64 = phys_base as i64 - min_addr as i64;
    let value_offset: i64 = virt_base as i64 - min_addr as i64;

    // Copy PT_LOAD segments to physical memory (using patch_offset).
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        let phys_addr = (vaddr as i64 + patch_offset) as usize;

        if filesz > 0 {
            match p_offset.checked_add(filesz) {
                Some(end) if end <= data.len() => {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            data.as_ptr().add(p_offset),
                            phys_addr as *mut u8,
                            filesz,
                        );
                    }
                }
                _ => return Err("PT_LOAD past data end or overflow"),
            }
        }
        if memsz > filesz {
            unsafe {
                core::ptr::write_bytes(
                    (phys_addr + filesz) as *mut u8,
                    0,
                    memsz - filesz,
                );
            }
        }
    }

    // Apply PT_DYNAMIC R_AARCH64_RELATIVE relocations.
    // Patches go to physical (patch_offset); values use virt_base (value_offset).
    let phys_range_end = phys_base.saturating_add(total_size);
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;

        let mut pos = dyn_offset;
        while pos + 16 <= data.len() && pos < dyn_offset + dyn_size {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag { 0 => break, 7 => rela_off = val as usize, 8 => rela_sz = val as usize, _ => {} }
            pos += 16;
        }

        if rela_off > 0 && rela_sz > 0 {
            let num = (rela_sz / 24).min(50_000_000);
            uart::puts("[loader] Applying "); crate::kernel::mm::print_num(num);
            uart::puts(" relocations (rebased)\n");
            let mut applied = 0usize;
            for r in 0..num {
                let re = match rela_off.checked_add(r.checked_mul(24).unwrap_or(usize::MAX)) {
                    Some(v) => v,
                    None => break,
                };
                match re.checked_add(24) {
                    Some(end) if end <= data.len() => {}
                    _ => break,
                }
                let r_offset = u64_at(data, re);
                let r_info = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                if (r_info & 0xFFFFFFFF) as u32 == 0x403 {
                    let patch_addr = (r_offset as i64 + patch_offset) as usize;
                    let value      = (r_addend as i64 + value_offset) as u64;
                    let patch_end = match patch_addr.checked_add(8) {
                        Some(v) => v,
                        None => continue,
                    };
                    if patch_addr < phys_base || patch_end > phys_range_end {
                        continue; // hostile / stray — silent skip
                    }
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    applied += 1;
                }
            }
            uart::puts("[loader] Applied "); crate::kernel::mm::print_num(applied);
            uart::puts(" R_RELATIVE (rebased)\n");
        }
    }

    // Flush dcache + icache.
    unsafe {
        let start = phys_base & !63;
        let end = phys_base + total_size + 0x20000;
        let mut addr = start;
        while addr < end {
            core::arch::asm!("dc cvac, {a}", a = in(reg) addr);
            core::arch::asm!("ic ivau, {a}", a = in(reg) addr);
            addr += 64;
        }
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    let virt_entry = (entry as i64 + value_offset) as u64;

    Ok(LoadedElfInfo {
        virt_entry,
        phys_base,
        total_size,
        virt_base,
    })
}

pub fn load_elf(data: &[u8]) -> Result<u64, &'static str> {
    if data.len() < 64 { return Err("too small"); }
    if &data[0..4] != b"\x7fELF" { return Err("not ELF"); }

    let entry = u64_at(data, 24);
    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    uart::puts("[loader] Entry: 0x"); print_hex(entry); uart::puts("\n");

    let mut min_addr: u64 = u64::MAX;
    let mut max_addr: u64 = 0;

    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        let ptype = u32_at(data, ph);
        if ptype != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        let memsz = u64_at(data, ph + 40);
        if vaddr < min_addr { min_addr = vaddr; }
        // FL-002: reject PT_LOAD entries whose vaddr + memsz wraps.
        let seg_end = match vaddr.checked_add(memsz) {
            Some(v) => v,
            None => return Err("PT_LOAD vaddr+memsz overflow"),
        };
        if seg_end > max_addr { max_addr = seg_end; }
    }

    let total_size = (max_addr - min_addr) as usize;
    let total_pages = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;

    // Allocate 2MB-aligned for MMU block mapping.
    // TODO: spin-for-alignment leaks frames (up to 511 per load after pool
    // fragmentation). Replace with a real `frame::alloc_aligned(pages, 2MB)`
    // helper once frame.rs grows one. For now, verify each subsequent alloc
    // is contiguous so we crash loudly instead of corrupting random memory.
    let mut phys_base = frame::alloc_frame().ok_or("oom")?;
    while phys_base & 0x1FFFFF != 0 {
        phys_base = frame::alloc_frame().ok_or("oom")?;
    }
    for i in 1..total_pages {
        let expected = phys_base + i * PAGE_SIZE;
        let got = frame::alloc_frame().ok_or("oom")?;
        if got != expected {
            uart::puts("[loader] FATAL: non-contiguous alloc at page ");
            crate::kernel::mm::print_num(i);
            uart::puts(" expected 0x"); print_hex(expected as u64);
            uart::puts(" got 0x"); print_hex(got as u64); uart::puts("\n");
            return Err("non-contiguous alloc (memory fragmented)");
        }
    }

    uart::puts("[loader] Physical base: 0x"); print_hex(phys_base as u64); uart::puts("\n");

    let reloc_offset = phys_base as i64 - min_addr as i64;

    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        let ptype = u32_at(data, ph);
        if ptype != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        let phys_addr = (vaddr as i64 + reloc_offset) as usize;

        // FL-001: reject PT_LOAD where p_offset + filesz overflows or
        // extends past data end. A naked `p_offset + filesz <= data.len()`
        // silently wraps for near-SIZE_MAX p_offset and passes bogusly.
        if filesz > 0 {
            match p_offset.checked_add(filesz) {
                Some(end) if end <= data.len() => {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            data.as_ptr().add(p_offset),
                            phys_addr as *mut u8,
                            filesz,
                        );
                    }
                }
                _ => return Err("PT_LOAD past data end or overflow"),
            }
        }
        if memsz > filesz {
            unsafe {
                core::ptr::write_bytes(
                    (phys_addr + filesz) as *mut u8,
                    0,
                    memsz - filesz,
                );
            }
        }
    }

    let phys_entry = (entry as i64 + reloc_offset) as u64;

    // Apply relocations (PT_DYNAMIC)
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;

        let mut pos = dyn_offset;
        while pos + 16 <= data.len() && pos < dyn_offset + dyn_size {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag { 0 => break, 7 => rela_off = val as usize, 8 => rela_sz = val as usize, _ => {} }
            pos += 16;
        }

        if rela_off > 0 && rela_sz > 0 {
            // FL-003: cap rela count at 50M. A crafted DT_RELASZ of
            // 0xFFFF…F yields ~7.6e17 iterations; later u64_at() reads
            // panic on out-of-bounds. 50M covers even monster binaries.
            let num = (rela_sz / 24).min(50_000_000);
            uart::puts("[loader] Applying "); crate::kernel::mm::print_num(num); uart::puts(" relocations\n");
            uart::puts("[loader] reloc_offset=0x"); print_hex(reloc_offset as u64); uart::puts("\n");
            // FL-004: precompute the ELF's allocated physical range. Any
            // relocation whose target lies outside [phys_base, phys_base+total_size)
            // is hostile (write-what-where primitive) — skip it.
            let phys_range_end = phys_base.saturating_add(total_size);
            let mut applied = 0usize;
            for r in 0..num {
                // FL-003 (tail): use checked arithmetic so a hostile
                // rela_off/num combo can't wrap past data.len().
                let re = match rela_off.checked_add(r.checked_mul(24).unwrap_or(usize::MAX)) {
                    Some(v) => v,
                    None => break,
                };
                match re.checked_add(24) {
                    Some(end) if end <= data.len() => {}
                    _ => break,
                }
                let r_offset = u64_at(data, re);
                let r_info = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                if (r_info & 0xFFFFFFFF) as u32 == 0x403 {
                    let patch_addr = (r_offset as i64 + reloc_offset) as usize;
                    let value = (r_addend as i64 + reloc_offset) as u64;
                    // FL-004: bounds-check the 8-byte write target. Also
                    // guard r_addend + reloc_offset for arithmetic wrap
                    // (`value` is already computed via `as i64` cast —
                    // but check patch_addr + 8 doesn't overflow).
                    let patch_end = match patch_addr.checked_add(8) {
                        Some(v) => v,
                        None => continue,
                    };
                    if patch_addr < phys_base || patch_end > phys_range_end {
                        // Silent skip: hostile or stray reloc. Don't
                        // abort the whole load — BSS-only relocations
                        // past max_addr would otherwise kill legit ELFs.
                        continue;
                    }
                    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) patch_addr, v = in(reg) value); }
                    applied += 1;
                    // Log only the first two relocations — anything more
                    // would spam millions of lines on a Chromium-sized ELF.
                    if applied <= 2 {
                        uart::puts("[reloc] vaddr=0x"); print_hex(r_offset);
                        uart::puts(" → phys=0x"); print_hex(patch_addr as u64);
                        uart::puts(" val=0x"); print_hex(value);
                        uart::puts("\n");
                    }
                }
            }
            uart::puts("[loader] Applied "); crate::kernel::mm::print_num(applied); uart::puts(" R_RELATIVE\n");
        }
    }

    // Flush ALL loaded pages INCLUDING relocated data: clean data cache + invalidate icache
    // Must cover the FULL memory range (max_addr - min_addr) not just file data,
    // because relocations patch addresses in BSS and data sections
    unsafe {
        let start = phys_base & !63;
        let end = phys_base + total_size + 0x20000; // extra 128KB to cover BSS + relocations
        let mut addr = start;
        while addr < end {
            core::arch::asm!("dc cvac, {a}", a = in(reg) addr);
            core::arch::asm!("ic ivau, {a}", a = in(reg) addr);
            addr += 64;
        }
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    LOADED_ENTRY.store(phys_entry as usize, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(entry as usize, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(phys_base, Ordering::Relaxed);
    Ok(phys_entry)
}

pub fn execute(entry: u64) -> Result<(), &'static str> {
    execute_with_args(entry, &["busybox"])
}

pub fn execute_with_args(entry: u64, argv: &[&str]) -> Result<(), &'static str> {
    let orig_entry = LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64;
    let phys_base = LOADED_PHYS_BASE.load(Ordering::Relaxed);

    let stack_base = frame::alloc_frame().ok_or("oom")?;
    for _ in 0..255 { frame::alloc_frame(); } // 256 pages = 1MB stack
    let stack_top = stack_base + 256 * PAGE_SIZE;

    // CRITICAL: Zero the entire stack
    unsafe {
        let ptr = stack_base as *mut u8;
        for i in 0..(256 * PAGE_SIZE) {
            core::ptr::write_volatile(ptr.add(i), 0);
        }
    }

    let mut sp = stack_top;

    // AT_RANDOM
    sp -= 16;
    let random_addr = sp;
    for i in 0..16usize {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val); }
        let byte = ((val >> (i % 8 * 8)) ^ (i as u64 * 37)) as u8;
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + i, v = in(reg) byte as u32); }
    }

    // Write argv strings
    // Chromium content_shell passes 20-40 flags (--no-sandbox, --disable-gpu,
    // --user-data-dir=..., --remote-debugging-port=..., etc.); bump from 16.
    let mut arg_addrs = [0usize; 64];
    let argc = argv.len().min(16);
    for i in 0..argc {
        let arg = argv[i].as_bytes();
        sp -= arg.len() + 1;
        arg_addrs[i] = sp;
        for (j, &b) in arg.iter().enumerate() {
            unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
        }
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + arg.len()); }
    }

    // envp
    sp -= 10;
    let env0 = sp;
    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }

    sp = (sp - 64) & !0xF;

    // auxv: AT_NULL, AT_RANDOM, AT_PAGESZ
    for &(k, v) in &[(0u64, 0u64), (25u64, random_addr as u64), (6u64, 4096u64)] {
        sp -= 16;
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v);
        }
    }

    // envp NULL + pointer
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env0 as u64); }

    // argv NULL + pointers
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    for i in (0..argc).rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) arg_addrs[i] as u64); }
    }

    // argc
    sp -= 8;
    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) argc as u64); }

    // TLS
    let tls_page = frame::alloc_frame().ok_or("oom")?;
    for i in 0..PAGE_SIZE {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) tls_page + i); }
    }
    unsafe { core::arch::asm!("msr tpidr_el0, {}", in(reg) (tls_page + PAGE_SIZE - 256) as u64); }

    // Enable MMU
    super::mmu::setup_and_enable(phys_base)?;

    let virt_entry = orig_entry;
    uart::puts("[loader] --- executing ---\n");

    // Save kernel SP to a FIXED memory location (0x40000100)
    // This address is in the first page of RAM, always accessible
    const SP_SAVE_ADDR: usize = 0x40000100;
    unsafe {
        let ksp: u64;
        core::arch::asm!("mov {}, sp", out(reg) ksp);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) SP_SAVE_ADDR, v = in(reg) ksp);
    }

    // V4: jump to user code at EL0 via eret. The exception vector table
    // already handles EL0-sourced SVC / IRQ by saving SP_EL1 via
    // SAVE_REGS and restoring via RESTORE_REGS, so the transition is
    // correct. User-code exit goes through handle_sync_exception (BRK /
    // exit syscall) — the handler restores SP_EL1 from 0x40000100 and
    // calls desktop::resume() directly, so this function never returns
    // to its caller. The label-99 return-here gimmick is gone.
    //
    // V2-NEW-017 / ESC-018: previously a raw `br {entry}` at EL1 left
    // busybox / hello running with kernel privilege.
    unsafe {
        core::arch::asm!(
            "msr sp_el0, {usp}",
            "msr elr_el1, {ent}",
            "msr spsr_el1, xzr",        // SPSR: EL0t, AIF clear
            "isb",
            "eret",
            usp = in(reg) sp as u64,
            ent = in(reg) virt_entry,
            options(noreturn),
        );
    }
}

/// Load a standalone (non-busybox) ELF binary and store its metadata
/// in the HELLO_* atomics. Returns (phys_entry, phys_base, orig_entry).
pub fn load_hello_elf(data: &[u8]) -> Result<(u64, usize, u64), &'static str> {
    // Save current loader state (so we don't clobber busybox metadata)
    let saved_entry = LOADED_ENTRY.load(Ordering::Relaxed);
    let saved_phys = LOADED_PHYS_BASE.load(Ordering::Relaxed);
    let saved_orig = LOADED_ORIG_ENTRY.load(Ordering::Relaxed);

    let phys_entry = load_elf(data)?;
    let phys_base = LOADED_PHYS_BASE.load(Ordering::Relaxed);
    let orig_entry = LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64;

    // Store in HELLO_* atomics
    HELLO_ENTRY.store(phys_entry as usize, Ordering::Relaxed);
    HELLO_PHYS_BASE.store(phys_base, Ordering::Relaxed);
    HELLO_ORIG_ENTRY.store(orig_entry as usize, Ordering::Relaxed);

    // Restore busybox loader state
    LOADED_ENTRY.store(saved_entry, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(saved_phys, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(saved_orig, Ordering::Relaxed);

    Ok((phys_entry, phys_base, orig_entry))
}

fn u64_at(data: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3],
        data[off+4], data[off+5], data[off+6], data[off+7]])
}
fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]])
}
fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off+1]])
}
fn print_hex(val: u64) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() { uart::putc(hex[((val >> (i * 4)) & 0xF) as usize]); }
}
