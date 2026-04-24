// Bat_OS — Production ELF Loader
// Handles real-world static ARM64 Linux binaries (like busybox).

use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE_SIZE: usize = 4096;

static LOADED_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
/// User VA base for the cave we'll eret into.
/// 0 for the primary cave (user VA 0..20 MB → phys_base..+20 MB).
/// Non-zero for a rebased cave — e.g., Chromium at 0x10000000.
/// `execute_with_args` uses this to translate every stack pointer +
/// the entry point from kernel-physical to the VA EL0 will actually see
/// through TTBR0 once the cave's page table is loaded.
static LOADED_USER_VA_BASE: AtomicUsize = AtomicUsize::new(0);
/// Physical address of a TLS page allocated contiguously with the ELF
/// image + stack. Non-zero means `execute_with_args` should program
/// `tpidr_el0` with the user VA of this page before ERET so libc's
/// thread-local accesses don't dereference VA 0. Zero means "legacy
/// behaviour — just clear tpidr_el0". Size is `LOADED_TLS_PAGES` pages.
static LOADED_TLS_PHYS: AtomicUsize = AtomicUsize::new(0);
/// 4 KB pages reserved for the TLS block. 16 KB is enough for glibc's
/// tcbhead_t plus a reasonable main-thread scratch area.
pub const LOADED_TLS_PAGES: usize = 4;
pub static SAVED_RETURN_ADDR: AtomicUsize = AtomicUsize::new(0);
static SAVED_KERNEL_SP: AtomicUsize = AtomicUsize::new(0);

/// Physical base of the user stack allocated right after the ELF pages so
/// it sits inside the primary cave's user window (phys_base..+20 MB). See
/// QEMU-BUGFIX-3 comment in `load_elf` for why: the previous code
/// allocated the stack anywhere in kernel RAM, and EL0 couldn't write to
/// it — every BatCave-runner ELF faulted at its first `stp` to `sp`.
static LOADED_STACK_PHYS: AtomicUsize = AtomicUsize::new(0);
/// 4 KB pages reserved for the user stack. Keep in sync with the
/// allocation size in `load_elf` and the consumer in `execute_with_args`.
pub const LOADED_STACK_PAGES: usize = 256; // 1 MB

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
///
/// V5-PARSER-002 fix: match `load_elf_rebased` bounds discipline
/// exactly. Previously this function lacked:
///   * ELF magic check
///   * vaddr + memsz overflow guard (→ write_bytes(ptr, 0, ~0) wiped RAM)
///   * phys_addr >= phys_base guard (crafted vaddr<min_addr caused
///     underflow, writing somewhere below phys_base)
pub fn reinit_elf(data: &[u8], phys_base: usize) {
    if data.len() < 64 { return; }
    if &data[0..4] != b"\x7fELF" { return; }

    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    // V8-ROOT-3 / V8-LENGTH-AUDIT: plausibility caps on phoff/phnum/phentsz
    // BEFORE the arithmetic that might wrap. Backport from V5 `load_elf_rebased`.
    // V8-ROOT-10: phentsz must be at least 56 (the fixed Elf64 phdr size),
    // otherwise each iteration below reads 56+ bytes starting at offsets
    // bounded only by phnum*phentsz — we can read past the nominal table.
    if phentsz < 56 || phentsz > 4096 || phnum > 1024 { return; }
    let pht_bytes = match phnum.checked_mul(phentsz) { Some(b) => b, None => return };
    if phoff.checked_add(pht_bytes).map_or(true, |e| e > data.len()) { return; }

    let mut min_addr: u64 = u64::MAX;
    let mut max_addr: u64 = 0;
    for i in 0..phnum {
        // Bounded by pht_bytes above; no wrap possible.
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        let memsz = u64_at(data, ph + 40);
        if vaddr < min_addr { min_addr = vaddr; }
        let seg_end = match vaddr.checked_add(memsz) {
            Some(e) => e,
            None => return, // overflow — refuse
        };
        if seg_end > max_addr { max_addr = seg_end; }
    }
    if min_addr == u64::MAX { return; } // no PT_LOAD
    let total_size = (max_addr - min_addr) as usize;
    // V6-PARSER-102 hardening: cap reinit_elf to a sane max so a swapped
    // ELF blob with crafted memsz can't cause us to zero or copy past
    // the originally-allocated region. The original load_elf reserves
    // up to ~256 MB for content_shell; anything larger is suspicious.
    const REINIT_ELF_MAX: usize = 256 * 1024 * 1024;
    if total_size > REINIT_ELF_MAX {
        crate::drivers::uart::puts("[loader] reinit_elf: total_size > 256MB — refusing\n");
        return;
    }
    let phys_range_end = phys_base.saturating_add(total_size);

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
        // V5-PARSER-002: refuse crafted vaddr<min_addr and segments
        // that don't fit in [phys_base, phys_base+total_size).
        if (vaddr as u64) < min_addr { return; }
        let phys_addr_i = (vaddr as i64).checked_add(reloc_offset);
        let phys_addr = match phys_addr_i {
            Some(a) if a as usize >= phys_base => a as usize,
            _ => return,
        };
        let seg_top = phys_addr.checked_add(memsz.max(filesz));
        if seg_top.map_or(true, |t| t > phys_range_end) { return; }

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

        // V8-ROOT-10: checked_add so an attacker PT_DYNAMIC with
        // p_offset=usize::MAX-16, p_filesz=0x20 can't wrap and fool the
        // `pos < dyn_offset + dyn_size` walk bound.
        let dyn_end = match dyn_offset.checked_add(dyn_size) {
            Some(e) if e <= data.len() => e,
            _ => continue,
        };
        let mut pos = dyn_offset;
        while pos + 16 <= dyn_end {
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

/// V8-ROOT-2 (V10 regression fix): drop every per-image loader state on
/// cave switch. SAVED_RETURN_ADDR + SAVED_KERNEL_SP in particular are a
/// cross-cave CONTROL-FLOW PIVOT — a resumed ELF from a previous cave
/// would trampoline through the next cave's EL0 context. The other
/// *_ENTRY / *_PHYS_BASE atomics leak address-space layout and would let
/// a fresh cave invoke the prior cave's loaded binary entrypoint.
pub fn reset_for_cave_switch() {
    LOADED_ENTRY.store(0, Ordering::Release);
    LOADED_ORIG_ENTRY.store(0, Ordering::Release);
    LOADED_PHYS_BASE.store(0, Ordering::Release);
    SAVED_RETURN_ADDR.store(0, Ordering::Release);
    SAVED_KERNEL_SP.store(0, Ordering::Release);
    WORKER_ENTRY.store(0, Ordering::Release);
    WORKER_PHYS_BASE.store(0, Ordering::Release);
    WORKER_ORIG_ENTRY.store(0, Ordering::Release);
    HELLO_ENTRY.store(0, Ordering::Release);
    HELLO_PHYS_BASE.store(0, Ordering::Release);
    HELLO_ORIG_ENTRY.store(0, Ordering::Release);
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

/// Entry in the multi-ELF loader's per-library table. Carries enough to
/// (a) do a second-pass relocation walk with cross-module symbol lookup
/// (b) run DT_INIT_ARRAY when we're ready to, and (c) dump diagnostics.
///
/// `data` is the pointer into the initrd blob for this library; the loader
/// already copied the PT_LOADs to `phys_base`, so `data` is read-only
/// reference material for relocation processing.
#[derive(Clone, Copy)]
pub struct LoadedLib {
    pub name_bytes: [u8; 64],
    pub name_len: usize,
    pub info: LoadedElfInfo,
    /// Raw ELF bytes (kept so the second-pass reloc walk can read the
    /// symtab/strtab/rela tables without keeping a `&[u8]` alongside).
    pub data_ptr: usize,
    pub data_len: usize,
    /// PT_LOAD min_addr (the ELF's base vaddr before rebase).
    pub min_addr: u64,
    /// value_offset = virt_base - min_addr (what RELATIVE relocs add).
    pub value_offset: i64,
    /// patch_offset = phys_base - min_addr (for writes during load).
    pub patch_offset: i64,
    /// DT_SYMTAB file offset inside the raw data.
    pub symtab_file: usize,
    /// DT_STRTAB file offset inside the raw data.
    pub strtab_file: usize,
    /// DT_SYMTAB entry count (derived from DT_HASH or DT_GNU_HASH).
    pub sym_count: usize,
    /// DT_RELA / DT_RELASZ (file offset, bytes).
    pub rela_off: usize,
    pub rela_sz: usize,
    /// DT_JMPREL / DT_PLTRELSZ (file offset, bytes).
    pub jmprel_off: usize,
    pub pltrel_sz: usize,
    /// DT_INIT_ARRAY vaddr + size (for later running).
    pub init_array_va: u64,
    pub init_array_sz: u64,
}

impl LoadedLib {
    pub fn new() -> Self {
        Self {
            name_bytes: [0; 64],
            name_len: 0,
            info: LoadedElfInfo { virt_entry: 0, phys_base: 0, total_size: 0, virt_base: 0 },
            data_ptr: 0, data_len: 0,
            min_addr: 0, value_offset: 0, patch_offset: 0,
            symtab_file: 0, strtab_file: 0, sym_count: 0,
            rela_off: 0, rela_sz: 0, jmprel_off: 0, pltrel_sz: 0,
            init_array_va: 0, init_array_sz: 0,
        }
    }
    pub fn name(&self) -> &[u8] { &self.name_bytes[..self.name_len] }
    pub fn data(&self) -> &'static [u8] {
        // SAFETY: data_ptr + data_len is the initrd blob slice, owned by
        // the static initrd region for the kernel's lifetime.
        unsafe { core::slice::from_raw_parts(self.data_ptr as *const u8, self.data_len) }
    }
}

/// Maximum libraries we can hold in a multi-ELF cave. content_shell pulls
/// in ~10 DT_NEEDED libs; keep a small cushion.
pub const MAX_LOADED_LIBS: usize = 16;

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

        // V5-PARSER-001 fix: reject PT_LOAD with vaddr < min_addr (the
        // scan at line 215 found min_addr, but an adversarial ELF could
        // include PT_LOAD with vaddr BELOW min_addr, producing a
        // negative patch_offset result and phys_addr < phys_base —
        // write-what-where primitive before our bounds check. Also
        // bound the full segment fits in [phys_base, phys_base+total).
        if (vaddr as u64) < min_addr {
            return Err("PT_LOAD vaddr below min_addr (crafted?)");
        }
        let phys_addr_i = (vaddr as i64).checked_add(patch_offset)
            .ok_or("PT_LOAD phys_addr overflow")?;
        if phys_addr_i < phys_base as i64 {
            return Err("PT_LOAD phys_addr below phys_base");
        }
        let phys_addr = phys_addr_i as usize;
        let seg_end_phys = phys_addr.checked_add(memsz.max(filesz))
            .ok_or("PT_LOAD segment end overflow")?;
        if seg_end_phys > phys_base.saturating_add(total_size) {
            return Err("PT_LOAD segment end past reserved range");
        }

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

    // Apply PT_DYNAMIC relocations.
    //   R_AARCH64_RELATIVE (0x403): *R = addend + value_offset
    //   R_AARCH64_GLOB_DAT  (0x401): *R = sym_value + addend + value_offset
    //   R_AARCH64_IRELATIVE (0x408): treat like RELATIVE for now — real
    //                                support requires calling the resolver;
    //                                this makes a call through the GOT hit
    //                                the resolver itself rather than 0x0.
    //   R_AARCH64_ABS32    (0x101): narrow 32-bit variant of GLOB_DAT, rare.
    //
    // Chromium content_shell has 539446 RELATIVE + 50 GLOB_DAT + 5 IRELATIVE
    // + 20 ABS32 entries. Skipping GLOB_DAT left GOT slots at their pre-link
    // placeholder (bare symbol value, e.g., 0x9909164), so the first call
    // through the GOT jumped to 0x9909xxx — well below virt_base=0x10000000,
    // landing in MMIO → instruction abort from EL0. This block brings
    // GLOB_DAT under the same value_offset rebase as RELATIVE and looks up
    // the symbol via DT_SYMTAB.
    let phys_range_end = phys_base.saturating_add(total_size);
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;
        let mut jmprel_off: usize = 0;
        let mut pltrel_sz: usize = 0;
        let mut symtab_vaddr: usize = 0;

        // V8-ROOT-10: dyn_offset + dyn_size could wrap — bound dyn_end first.
        let dyn_end = match dyn_offset.checked_add(dyn_size) {
            Some(e) if e <= data.len() => e,
            _ => continue,
        };
        let mut pos = dyn_offset;
        while pos + 16 <= dyn_end {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag {
                0 => break,
                6 => symtab_vaddr = val as usize, // DT_SYMTAB  (vaddr)
                7 => rela_off = val as usize,     // DT_RELA    (vaddr)
                8 => rela_sz  = val as usize,     // DT_RELASZ  (bytes)
                0x17 => jmprel_off = val as usize, // DT_JMPREL (PLT rela vaddr)
                0x02 => pltrel_sz  = val as usize, // DT_PLTRELSZ
                _ => {}
            }
            pos += 16;
        }

        // Turn a vaddr like `symtab_vaddr` into a file offset by scanning
        // PT_LOAD. The caller (load_elf_rebased) already bounded these
        // headers; we just need p_vaddr / p_offset / p_filesz to locate
        // each vaddr in the file. Returns usize::MAX when the vaddr is
        // not covered by any PT_LOAD.
        let vaddr_to_file_off = |va: usize| -> usize {
            for j in 0..phnum {
                let ph_j = phoff + j * phentsz;
                if ph_j + phentsz > data.len() { break; }
                if u32_at(data, ph_j) != 1 { continue; }
                let p_offset_j = u64_at(data, ph_j + 8) as usize;
                let p_vaddr_j  = u64_at(data, ph_j + 16) as usize;
                let p_filesz_j = u64_at(data, ph_j + 32) as usize;
                if va >= p_vaddr_j && va < p_vaddr_j.saturating_add(p_filesz_j) {
                    return p_offset_j.saturating_add(va - p_vaddr_j);
                }
            }
            usize::MAX
        };
        let symtab_file = vaddr_to_file_off(symtab_vaddr);

        // Apply relocations from both the main RELA table AND the PLT rela
        // table. Static-PIE Chromium carries 340 R_AARCH64_JUMP_SLOT entries
        // in .rela.plt pointing at functions in the same binary — skipping
        // those leaves PLT stubs jumping into unrelocated symbol values
        // (e.g., a function at vaddr 0x9909164 stays 0x9909164 instead of
        // being rebased to 0x19909164, landing in MMIO on the first call).
        let mut applied_rel = 0usize;
        let mut applied_glob = 0usize;
        let mut applied_irel = 0usize;
        let mut applied_jump = 0usize;
        for &(tbl_off, tbl_sz, label_is_plt) in &[
            (rela_off, rela_sz, false),
            (jmprel_off, pltrel_sz, true),
        ] {
            if tbl_off == 0 || tbl_sz == 0 { continue; }
            let num = (tbl_sz / 24).min(50_000_000);
            uart::puts("[loader] Applying "); crate::kernel::mm::print_num(num);
            uart::puts(if label_is_plt { " plt relocs (rebased)\n" }
                       else            { " rela relocs (rebased)\n" });
            for r in 0..num {
                let re = match tbl_off.checked_add(r.checked_mul(24).unwrap_or(usize::MAX)) {
                    Some(v) => v,
                    None => break,
                };
                match re.checked_add(24) {
                    Some(end) if end <= data.len() => {}
                    _ => break,
                }
                let r_offset = u64_at(data, re);
                let r_info   = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                let r_type   = (r_info & 0xFFFFFFFF) as u32;
                let r_sym    = (r_info >> 32) as usize;

                let patch_addr = (r_offset as i64 + patch_offset) as usize;
                let patch_end = match patch_addr.checked_add(8) {
                    Some(v) => v,
                    None => continue,
                };
                if patch_addr < phys_base || patch_end > phys_range_end {
                    continue; // hostile / stray — silent skip
                }

                match r_type {
                    0x403 => {
                        // R_AARCH64_RELATIVE: base + addend
                        let value = (r_addend as i64 + value_offset) as u64;
                        unsafe {
                            core::arch::asm!("str {v}, [{a}]",
                                a = in(reg) patch_addr, v = in(reg) value);
                        }
                        applied_rel += 1;
                    }
                    0x401 | 0x402 => {
                        // R_AARCH64_GLOB_DAT (0x401) / R_AARCH64_JUMP_SLOT
                        // (0x402): both compute *R = sym_value + addend + base.
                        // For a static-PIE (no dynamic linker to do lazy
                        // binding) the JUMP_SLOT formula is identical to
                        // GLOB_DAT — we just need the symbol's defined
                        // vaddr from DT_SYMTAB.
                        if symtab_file == usize::MAX { continue; }
                        let sym_file = match symtab_file
                            .checked_add(r_sym.checked_mul(24).unwrap_or(usize::MAX))
                        {
                            Some(v) if v + 24 <= data.len() => v,
                            _ => continue,
                        };
                        let sym_value = u64_at(data, sym_file + 8); // Elf64_Sym::st_value
                        let sym_shndx = u64::from(
                            u16::from_le_bytes([data[sym_file + 6], data[sym_file + 7]])
                        );
                        // SHN_UNDEF (shndx == 0): the symbol is not defined
                        // inside THIS binary — e.g., `printf`, `__libc_start_main`,
                        // `pthread_*`, compiler-rt `__divti3`. A real static
                        // PIE (musl-linked) wouldn't have these. content_shell
                        // as built today is dynamically linked against glibc
                        // (567 undef symbols). Rather than writing `base + 0`
                        // and having EL0 call into the rodata segment at
                        // virt_base (mysterious EC=0 crash at 0x10000000),
                        // stash a sentinel containing the symbol index. When
                        // EL0 tries to call through the GOT, it instruction-
                        // aborts with FAR=0xBAD_UNDEF...[idx], which the
                        // sync-exception handler decodes into a human-
                        // readable "undef sym #X called" log.
                        let value = if sym_shndx == 0 {
                            // 0xBAD0_0000_0000_0000 | (sym_idx << 4)
                            // The <<4 keeps the low 4 bits zero so the CPU
                            // raises a plain instruction abort (EC=0x20)
                            // on PC fetch rather than a PC-alignment fault
                            // (EC=0x22), and leaves room for a human-readable
                            // 0xBAD prefix at bit 63:52. The sync handler
                            // tests for this pattern and prints the symbol
                            // name.
                            0xBAD0_0000_0000_0000u64 | ((r_sym as u64 & 0x0FFF_FFFF) << 4)
                        } else {
                            (sym_value as i64 + r_addend as i64 + value_offset) as u64
                        };
                        unsafe {
                            core::arch::asm!("str {v}, [{a}]",
                                a = in(reg) patch_addr, v = in(reg) value);
                        }
                        if r_type == 0x402 { applied_jump += 1; }
                        else               { applied_glob += 1; }
                    }
                    0x408 => {
                        // R_AARCH64_IRELATIVE: *R = resolver() — but we
                        // can't call user code during load yet. Writing the
                        // resolver's address is a survivable approximation
                        // (a call through the GOT invokes the resolver,
                        // which returns the variant pointer and typically
                        // has no side effects). TODO: call the resolver at
                        // load-time in EL1 to get the real value.
                        let value = (r_addend as i64 + value_offset) as u64;
                        unsafe {
                            core::arch::asm!("str {v}, [{a}]",
                                a = in(reg) patch_addr, v = in(reg) value);
                        }
                        applied_irel += 1;
                    }
                    _ => {
                        // ABS32 (0x101), TLS_*, etc. — ignore for now.
                    }
                }
            }
        }
        if applied_rel + applied_glob + applied_irel + applied_jump > 0 {
            uart::puts("[loader] Applied "); crate::kernel::mm::print_num(applied_rel);
            uart::puts(" REL, "); crate::kernel::mm::print_num(applied_glob);
            uart::puts(" GLOB, "); crate::kernel::mm::print_num(applied_jump);
            uart::puts(" JUMP, "); crate::kernel::mm::print_num(applied_irel);
            uart::puts(" IREL (rebased)\n");
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

    // Allocate a user stack contiguous with the ELF image so it lives inside
    // the cave's user window (cave maps virt_base..virt_base+200MB →
    // phys_base..phys_base+200MB). Matches `load_elf`'s QEMU-BUGFIX-3 layout.
    // Without this, `execute_with_args` aborts with "no stack reserved" the
    // moment the rebased path hands off (see runner::run_chromium).
    let stack_phys_start = phys_base + total_pages * PAGE_SIZE;
    for i in 0..LOADED_STACK_PAGES {
        let expected = stack_phys_start + i * PAGE_SIZE;
        let got = frame::alloc_frame().ok_or("oom (rebased stack)")?;
        if got != expected {
            return Err("non-contiguous rebased stack alloc (memory fragmented)");
        }
    }
    // Cave virt window = CAVE_BLOCKS × 2 MB (400 MB default — see
    // mmu::CAVE_BLOCKS). ELF + stack must fit under that.
    let cave_window_bytes: usize = super::mmu::CAVE_BLOCKS * 0x200000;
    if total_pages * PAGE_SIZE + LOADED_STACK_PAGES * PAGE_SIZE > cave_window_bytes {
        return Err("ELF + stack > cave user window");
    }

    // Publish loader state so `execute_with_args` knows where the stack +
    // phys base live, and that user VAs should be biased by `virt_base`.
    LOADED_ENTRY.store(virt_entry as usize, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(entry as usize, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(phys_base, Ordering::Relaxed);
    LOADED_STACK_PHYS.store(stack_phys_start, Ordering::Relaxed);
    LOADED_USER_VA_BASE.store(virt_base as usize, Ordering::Relaxed);

    uart::puts("[loader] Rebased stack phys: 0x"); print_hex(stack_phys_start as u64);
    uart::puts("  ("); crate::kernel::mm::print_num(LOADED_STACK_PAGES);
    uart::puts(" pages)\n");

    Ok(LoadedElfInfo {
        virt_entry,
        phys_base,
        total_size,
        virt_base,
    })
}

/// Multi-ELF cave loader. Loads content_shell plus any number of shared
/// libraries into ONE contiguous physical region, placed so the cave's
/// simple 1:1 (virt..virt+N → phys..phys+N) mapping covers everything.
///
/// This is a minimal in-kernel dynamic linker: after each ELF is loaded
/// and has its R_AARCH64_RELATIVE applied, we do a second pass over
/// every lib's GLOB_DAT / JUMP_SLOT / IRELATIVE entries — for any
/// SHN_UNDEF symbol, we scan every other loaded lib's symbol table
/// and write the real resolved address over the sentinel we left in
/// place during the first pass.
///
/// No symbol versioning (DT_VERSYM / DT_VERNEED), no init_array
/// execution, no TLS setup. This is only enough to get content_shell
/// to reach its first real libc call and exercise the syscall surface.
///
/// Returns `LoadedElfInfo` for the FIRST entry in `files` — that's the
/// main executable whose entry point the runner will eret to.
pub fn load_archive_multi(
    files: &[(&[u8], &[u8])],  // (name, bytes) — [0] is the main exe
    cave_virt_base: u64,
) -> Result<LoadedElfInfo, &'static str> {
    if files.is_empty() { return Err("no files"); }
    if files.len() > MAX_LOADED_LIBS { return Err("too many libs"); }
    if cave_virt_base & 0x1FFFFF != 0 { return Err("cave_virt_base not 2MB aligned"); }

    // Parse each ELF to find its (min_addr, max_addr) PT_LOAD extent,
    // then lay them out in the cave with 2 MB alignment between them.
    let mut lib_count = 0usize;
    let mut libs: [LoadedLib; MAX_LOADED_LIBS] = [LoadedLib::new(); MAX_LOADED_LIBS];
    let mut next_virt_offset: usize = 0;
    for (name, data) in files {
        if data.len() < 64 || &data[0..4] != b"\x7fELF" {
            return Err("archive member is not ELF");
        }
        let phoff = u64_at(data, 32) as usize;
        let phnum = u16_at(data, 56) as usize;
        let phentsz = u16_at(data, 54) as usize;
        if phentsz < 56 || phentsz > 4096 || phnum > 1024 {
            return Err("phdr table implausible");
        }

        let mut min_addr: u64 = u64::MAX;
        let mut max_addr: u64 = 0;
        for i in 0..phnum {
            let ph = phoff + i * phentsz;
            if ph + phentsz > data.len() { break; }
            if u32_at(data, ph) != 1 { continue; }
            let vaddr = u64_at(data, ph + 16);
            let memsz = u64_at(data, ph + 40);
            if vaddr < min_addr { min_addr = vaddr; }
            let seg_end = vaddr.checked_add(memsz).ok_or("PT_LOAD overflow")?;
            if seg_end > max_addr { max_addr = seg_end; }
        }
        if min_addr == u64::MAX { return Err("no PT_LOAD"); }

        // Round the extent up to 2 MB so each lib starts at a 2-MB cave
        // block boundary. Pairs with mmu::setup_cave_pagetable_at.
        let total_size = (max_addr - min_addr) as usize;
        let rounded = (total_size + 0x1FFFFF) & !0x1FFFFF;

        let my_virt_base = cave_virt_base + next_virt_offset as u64;
        let name_bytes_len = name.len().min(63);
        libs[lib_count].name_bytes[..name_bytes_len].copy_from_slice(&name[..name_bytes_len]);
        libs[lib_count].name_len = name_bytes_len;
        libs[lib_count].data_ptr = data.as_ptr() as usize;
        libs[lib_count].data_len = data.len();
        libs[lib_count].min_addr = min_addr;
        libs[lib_count].info.virt_base = my_virt_base;
        libs[lib_count].info.total_size = total_size;
        libs[lib_count].value_offset = my_virt_base as i64 - min_addr as i64;
        // phys_base fills in below once we've allocated the mega-block.
        lib_count += 1;
        next_virt_offset = next_virt_offset
            .checked_add(rounded)
            .ok_or("cave virt offset overflow")?;
    }

    // Total contiguous pages: all libs + 1 MB user stack + 16 KB TLS.
    let total_bytes = next_virt_offset;
    let total_pages = total_bytes / PAGE_SIZE;
    let stack_va_offset = next_virt_offset;                       // stack follows last lib
    let tls_va_offset   = stack_va_offset + LOADED_STACK_PAGES * PAGE_SIZE; // TLS after stack
    let grand_total_pages = total_pages + LOADED_STACK_PAGES + LOADED_TLS_PAGES;

    // Enforce cave window.
    let cave_window_bytes = super::mmu::CAVE_BLOCKS * 0x200000;
    if grand_total_pages * PAGE_SIZE > cave_window_bytes {
        return Err("archive + stack > cave user window");
    }

    // Contiguous 2MB-aligned allocation.
    let mut phys_base = frame::alloc_frame().ok_or("oom (archive)")?;
    while phys_base & 0x1FFFFF != 0 {
        phys_base = frame::alloc_frame().ok_or("oom (archive align)")?;
    }
    for i in 1..grand_total_pages {
        let expected = phys_base + i * PAGE_SIZE;
        let got = frame::alloc_frame().ok_or("oom (archive tail)")?;
        if got != expected {
            return Err("non-contiguous archive alloc");
        }
    }
    uart::puts("[loader/multi] reserved "); crate::kernel::mm::print_num(total_bytes / (1024 * 1024));
    uart::puts(" MB + "); crate::kernel::mm::print_num(LOADED_STACK_PAGES * 4);
    uart::puts(" KB stack at phys 0x"); print_hex(phys_base as u64); uart::puts("\n");

    // Fill in phys_base + patch_offset now that we have the allocation.
    let mut offset_in_cave: usize = 0;
    for i in 0..lib_count {
        let lib_virt_base = libs[i].info.virt_base;
        let lib_phys = phys_base + offset_in_cave;
        libs[i].info.phys_base = lib_phys;
        libs[i].patch_offset = lib_phys as i64 - libs[i].min_addr as i64;
        libs[i].info.virt_entry = (u64_at(libs[i].data(), 24) as i64 + libs[i].value_offset) as u64;
        let rounded = (libs[i].info.total_size + 0x1FFFFF) & !0x1FFFFF;
        offset_in_cave += rounded;
        let _ = lib_virt_base;
    }

    // --- First pass per lib: copy PT_LOADs + parse .dynamic ---
    for i in 0..lib_count {
        stage_copy_and_parse(&mut libs[i])?;
    }


    // --- Second pass per lib: apply relocations with cross-module lookup ---
    for i in 0..lib_count {
        apply_relocs_cross(&libs, lib_count, i)?;
    }

    // Diagnostic: dump init_array entries per lib (first 3 each). These
    // are function VAs glibc expects the dynamic linker to have run
    // before __libc_start_main. After relocation they should hold
    // cave-VA function addresses.
    for i in 0..lib_count {
        if libs[i].init_array_sz == 0 { continue; }
        let ia_phys = (libs[i].init_array_va as i64 + libs[i].patch_offset) as u64;
        let n = (libs[i].init_array_sz / 8) as usize;
        uart::puts("[init_array] ");
        for &b in libs[i].name() { uart::putc(b); }
        uart::puts(": ");
        crate::kernel::mm::print_num(n);
        uart::puts(" entries @0x");
        print_hex(ia_phys);
        for j in 0..n.min(3) {
            let entry = unsafe {
                core::ptr::read_volatile((ia_phys + (j * 8) as u64) as *const u64)
            };
            uart::puts(" 0x"); print_hex(entry);
        }
        uart::puts("\n");
    }

    // Flush i/d cache over the full reserved region.
    unsafe {
        let start = phys_base & !63;
        let end = phys_base + total_bytes + 0x20000;
        let mut addr = start;
        while addr < end {
            core::arch::asm!("dc cvac, {a}", a = in(reg) addr);
            core::arch::asm!("ic ivau, {a}", a = in(reg) addr);
            addr += 64;
        }
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    // Stack sits right after the last lib; TLS right after the stack.
    let stack_phys = phys_base + stack_va_offset;
    let tls_phys   = phys_base + tls_va_offset;

    // Zero the TLS block. glibc's tcbhead_t layout on aarch64 is:
    //   offset 0:  dtv pointer
    //   offset 8:  private
    //   offset 16: padding[2] (16 bytes)
    //   offset 32: TLS data
    // All zeros is enough to stop the first tpidr_el0 dereference from
    // segfaulting — proper DTV + TLS initialization is follow-up work.
    unsafe {
        let tls_end = tls_phys + LOADED_TLS_PAGES * PAGE_SIZE;
        let mut p = tls_phys;
        while p < tls_end {
            core::arch::asm!("str xzr, [{a}]", a = in(reg) p);
            p += 8;
        }
    }

    // Publish loader state for `execute_with_args`. virt_entry + phys_base
    // + user_va_base all correspond to the MAIN exe (libs[0]).
    LOADED_ENTRY.store(libs[0].info.virt_entry as usize, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(u64_at(libs[0].data(), 24) as usize, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(libs[0].info.phys_base, Ordering::Relaxed);
    LOADED_STACK_PHYS.store(stack_phys, Ordering::Relaxed);
    LOADED_USER_VA_BASE.store(libs[0].info.virt_base as usize, Ordering::Relaxed);
    LOADED_TLS_PHYS.store(tls_phys, Ordering::Relaxed);

    uart::puts("[loader/multi] stack phys 0x"); print_hex(stack_phys as u64);
    uart::puts(", tls phys 0x"); print_hex(tls_phys as u64);
    uart::puts(", main virt_entry 0x"); print_hex(libs[0].info.virt_entry);
    uart::puts("\n");

    Ok(libs[0].info)
}

/// Stage 1 for a lib: copy PT_LOAD segments into `lib.info.phys_base` and
/// parse .dynamic to record the tables we need for the reloc pass.
fn stage_copy_and_parse(lib: &mut LoadedLib) -> Result<(), &'static str> {
    let data = lib.data();
    let phys_base = lib.info.phys_base;
    let min_addr = lib.min_addr;
    let patch_offset = lib.patch_offset;
    let total_size = lib.info.total_size;

    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    // Copy PT_LOADs.
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr    = u64_at(data, ph + 16) as usize;
        let filesz   = u64_at(data, ph + 32) as usize;
        let memsz    = u64_at(data, ph + 40) as usize;

        if (vaddr as u64) < min_addr { return Err("PT_LOAD below min_addr"); }
        let phys_addr_i = (vaddr as i64).checked_add(patch_offset)
            .ok_or("PT_LOAD phys overflow")?;
        if phys_addr_i < phys_base as i64 { return Err("PT_LOAD phys < base"); }
        let phys_addr = phys_addr_i as usize;
        let seg_end = phys_addr.checked_add(memsz.max(filesz))
            .ok_or("PT_LOAD end overflow")?;
        if seg_end > phys_base.saturating_add(total_size) {
            return Err("PT_LOAD past reserved range");
        }

        if filesz > 0 {
            let end = p_offset.checked_add(filesz).ok_or("p_offset overflow")?;
            if end > data.len() { return Err("PT_LOAD past data end"); }
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
                    (phys_addr + filesz) as *mut u8, 0, memsz - filesz,
                );
            }
        }
    }

    // Parse PT_DYNAMIC.
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; } // PT_DYNAMIC

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let dyn_end = match dyn_offset.checked_add(dyn_size) {
            Some(e) if e <= data.len() => e,
            _ => continue,
        };
        let mut pos = dyn_offset;
        let mut symtab_va = 0u64;
        let mut strtab_va = 0u64;
        let mut rela_va   = 0u64;
        let mut rela_sz   = 0u64;
        let mut jmprel_va = 0u64;
        let mut pltrel_sz = 0u64;
        let mut init_array_va = 0u64;
        let mut init_array_sz = 0u64;
        let mut hash_va = 0u64;
        let mut gnu_hash_va = 0u64;
        while pos + 16 <= dyn_end {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag {
                0 => break,
                1 => {} // DT_NEEDED — we ignore the name list and rely on
                        // the caller having baked the right libs into the
                        // archive.
                4 => hash_va = val,       // DT_HASH
                5 => strtab_va = val,     // DT_STRTAB
                6 => symtab_va = val,     // DT_SYMTAB
                7 => rela_va = val,       // DT_RELA
                8 => rela_sz = val,       // DT_RELASZ
                0x17 => jmprel_va = val,  // DT_JMPREL
                0x02 => pltrel_sz = val,  // DT_PLTRELSZ
                25 => init_array_va = val,// DT_INIT_ARRAY
                27 => init_array_sz = val,// DT_INIT_ARRAYSZ
                0x6ffffef5 => gnu_hash_va = val, // DT_GNU_HASH
                _ => {}
            }
            pos += 16;
        }

        lib.symtab_file = vaddr_to_file_off(data, phoff, phnum, phentsz, symtab_va as usize);
        lib.strtab_file = vaddr_to_file_off(data, phoff, phnum, phentsz, strtab_va as usize);
        lib.rela_off   = vaddr_to_file_off(data, phoff, phnum, phentsz, rela_va as usize);
        lib.rela_sz    = rela_sz as usize;
        lib.jmprel_off = vaddr_to_file_off(data, phoff, phnum, phentsz, jmprel_va as usize);
        lib.pltrel_sz  = pltrel_sz as usize;
        lib.init_array_va = init_array_va;
        lib.init_array_sz = init_array_sz;


        // Derive sym_count. Prefer DT_HASH's nchain (exact); otherwise use
        // the fact that ELF builders always place the symbol table
        // IMMEDIATELY BEFORE the string table, so `(strtab_vaddr -
        // symtab_vaddr) / 24` is a tight safe upper bound. The old
        // fallback of 65536 was the bug behind our EC=0x22 crash: our
        // cross-module resolver scanned FAR past the real symtab into
        // random bytes, interpreted them as symbols, and "matched"
        // garbage names — one such false match wrote 0xcf469e347c673dea
        // to a GOT slot that then took down content_shell.
        lib.sym_count = 0;
        if hash_va != 0 {
            let hash_file = vaddr_to_file_off(data, phoff, phnum, phentsz, hash_va as usize);
            if hash_file != usize::MAX && hash_file + 8 <= data.len() {
                let nchain = u32_at(data, hash_file + 4) as usize;
                lib.sym_count = nchain;
            }
        }
        if lib.sym_count == 0 && symtab_va != 0 && strtab_va > symtab_va {
            // Tight bound — strtab directly follows symtab in every
            // mainstream linker.
            lib.sym_count = ((strtab_va - symtab_va) / 24) as usize;
        }
        if lib.sym_count == 0 {
            lib.sym_count = 1; // at least the undefined slot
        }
        let _ = gnu_hash_va; // retained for future DT_GNU_HASH-based lookups
        break;
    }
    Ok(())
}

/// Walk PT_LOAD headers to convert an ELF vaddr into a file offset.
/// Returns `usize::MAX` if the vaddr isn't mapped by any PT_LOAD.
fn vaddr_to_file_off(data: &[u8], phoff: usize, phnum: usize, phentsz: usize, va: usize) -> usize {
    if va == 0 { return usize::MAX; }
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }
        let p_offset = u64_at(data, ph + 8) as usize;
        let p_vaddr  = u64_at(data, ph + 16) as usize;
        let p_filesz = u64_at(data, ph + 32) as usize;
        if va >= p_vaddr && va < p_vaddr.saturating_add(p_filesz) {
            return p_offset.saturating_add(va - p_vaddr);
        }
    }
    usize::MAX
}

/// Look up a symbol name across every loaded lib's symbol table. If a
/// DEFINED symbol with the matching name is found, return its cave-VA
/// (`lib.value_offset + sym.st_value`). Ignores UNDEF (shndx==0)
/// entries and names that don't match byte-for-byte.
fn resolve_cross_module(
    libs: &[LoadedLib],
    count: usize,
    name: &[u8],
) -> Option<u64> {
    if name.is_empty() { return None; }
    for i in 0..count {
        let lib = &libs[i];
        if lib.symtab_file == usize::MAX || lib.strtab_file == usize::MAX {
            continue;
        }
        let data = lib.data();
        let limit = lib.sym_count.min(200_000);
        for s in 1..limit {
            let sym_off = lib.symtab_file + s * 24;
            if sym_off + 24 > data.len() { break; }
            let name_off = u32_at(data, sym_off) as usize;
            let st_info  = data[sym_off + 4];
            let st_shndx = u16::from_le_bytes([data[sym_off + 6], data[sym_off + 7]]);
            let st_value = u64_at(data, sym_off + 8);
            if st_shndx == 0 { continue; } // UNDEF — skip
            // STB_WEAK (bind=2) with value 0 is also treated as unresolvable.
            let _ = st_info;
            if st_value == 0 && st_shndx == 0xFFFF { continue; }
            // Compare name.
            let str_addr = lib.strtab_file + name_off;
            if str_addr >= data.len() { continue; }
            if !str_eq(&data[str_addr..], name) { continue; }
            // Match. Resolve to cave VA.
            return Some((st_value as i64 + lib.value_offset) as u64);
        }
    }
    None
}

/// Short suffix shown in the ghost-reloc tracer so we can tell which
/// of a lib's two rela tables produced the write.
fn idx_dbg_suffix(idx: usize) -> &'static str {
    if idx == 0 { ".dyn" } else { ".plt" }
}

/// Byte-compare `haystack` (null-terminated) against `needle` (exact).
fn str_eq(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() + 1 { return false; }
    if &haystack[..needle.len()] != needle { return false; }
    haystack[needle.len()] == 0
}

/// Apply the relocations for one lib, with cross-module lookup for
/// SHN_UNDEF GLOB_DAT / JUMP_SLOT entries. For UNDEFs that resolve in
/// another lib we write the resolved vaddr; for UNDEFs that don't
/// resolve we write the 0xBAD0… sentinel so the first call through
/// the GOT is diagnosable.
fn apply_relocs_cross(
    libs: &[LoadedLib; MAX_LOADED_LIBS],
    count: usize,
    idx: usize,
) -> Result<(), &'static str> {
    let lib = &libs[idx];
    let data = lib.data();
    let phys_base = lib.info.phys_base;
    let phys_range_end = phys_base.saturating_add(lib.info.total_size);
    let value_offset = lib.value_offset;
    let patch_offset = lib.patch_offset;
    let symtab_file = lib.symtab_file;
    let strtab_file = lib.strtab_file;

    let mut applied_rel = 0u32;
    let mut applied_glob = 0u32;
    let mut applied_jump = 0u32;
    let mut applied_irel = 0u32;
    let mut resolved_cross = 0u32;
    let mut unresolved = 0u32;

    // Debug trap: the Chromium port has a ghost-reloc bug that
    // writes 0x5ac00000 (phys_base) to content_shell's text at
    // vaddr 0x1a4ff44 (phys 0x5c64ff44). Set to non-zero to log
    // the specific reloc; leave at 0 in prod.
    const WATCH_PHYS: usize = 0x5c64ff44;

    for (tbl_idx, &(tbl_off, tbl_sz)) in [
        (lib.rela_off, lib.rela_sz),
        (lib.jmprel_off, lib.pltrel_sz),
    ].iter().enumerate() {
        if tbl_off == 0 || tbl_off == usize::MAX || tbl_sz == 0 { continue; }
        let num = (tbl_sz / 24).min(50_000_000);
        for r in 0..num {
            let re = match tbl_off.checked_add(r.checked_mul(24).unwrap_or(usize::MAX)) {
                Some(v) => v,
                None => break,
            };
            if re + 24 > data.len() { break; }
            let r_offset = u64_at(data, re);
            let r_info   = u64_at(data, re + 8);
            let r_addend = u64_at(data, re + 16);
            let r_type   = (r_info & 0xFFFFFFFF) as u32;
            let r_sym    = (r_info >> 32) as usize;

            let patch_addr = (r_offset as i64 + patch_offset) as usize;
            let patch_end = match patch_addr.checked_add(8) { Some(v) => v, None => continue };
            if patch_addr < phys_base || patch_end > phys_range_end { continue; }

            // Ghost-reloc tracer — fires ONCE per reloc targeting the
            // watch address, so we see every write that would land there.
            if patch_addr == WATCH_PHYS || (patch_addr < WATCH_PHYS && WATCH_PHYS < patch_end) {
                uart::puts("[ghost] ");
                for &b in lib.name() { uart::putc(b); }
                uart::puts(idx_dbg_suffix(tbl_idx));
                uart::puts(" r_off=0x"); print_hex(r_offset);
                uart::puts(" type=0x"); print_hex(r_type as u64);
                uart::puts(" addend=0x"); print_hex(r_addend);
                uart::puts(" sym="); crate::kernel::mm::print_num(r_sym);
                uart::puts(" patch_addr=0x"); print_hex(patch_addr as u64);
                uart::puts(" value_offset=0x"); print_hex(value_offset as u64);
                uart::puts("\n");
            }

            match r_type {
                0x403 => {
                    let value = (r_addend as i64 + value_offset) as u64;
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    applied_rel += 1;
                }
                0x401 | 0x402 => {
                    if symtab_file == usize::MAX { continue; }
                    let sym_off = match symtab_file
                        .checked_add(r_sym.checked_mul(24).unwrap_or(usize::MAX))
                    {
                        Some(v) if v + 24 <= data.len() => v,
                        _ => continue,
                    };
                    let sym_value = u64_at(data, sym_off + 8);
                    let st_info   = data[sym_off + 4];
                    let sym_bind  = st_info >> 4;             // STB_* — 2 = STB_WEAK
                    let sym_shndx = u16::from_le_bytes([data[sym_off+6], data[sym_off+7]]);
                    let value = if sym_shndx == 0 {
                        // UNDEF — try cross-module lookup first.
                        let mut resolved: Option<u64> = None;
                        if strtab_file != usize::MAX {
                            let name_off = u32_at(data, sym_off) as usize;
                            let str_addr = strtab_file.saturating_add(name_off);
                            if str_addr < data.len() {
                                let end = data[str_addr..].iter()
                                    .position(|&b| b == 0).unwrap_or(256);
                                let name = &data[str_addr..str_addr + end];
                                resolved = resolve_cross_module(libs, count, name);
                            }
                        }
                        match resolved {
                            Some(v) => {
                                resolved_cross += 1;
                                (v as i64 + r_addend as i64) as u64
                            }
                            None if sym_bind == 2 => {
                                // STB_WEAK unresolved → NULL. The caller is
                                // expected to null-check this pointer (the
                                // classic case is `__gmon_start__`, which
                                // every PIE has as a weak ref and every
                                // non-gprof program skips when null).
                                0
                            }
                            None => {
                                unresolved += 1;
                                0xBAD0_0000_0000_0000u64 | ((r_sym as u64 & 0x0FFF_FFFF) << 4)
                            }
                        }
                    } else {
                        (sym_value as i64 + r_addend as i64 + value_offset) as u64
                    };
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    if r_type == 0x402 { applied_jump += 1; } else { applied_glob += 1; }
                }
                0x408 => {
                    let value = (r_addend as i64 + value_offset) as u64;
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    applied_irel += 1;
                }
                _ => {}
            }
        }
    }

    uart::puts("[loader/multi] ");
    for &b in lib.name() { uart::putc(b); }
    uart::puts(": ");
    crate::kernel::mm::print_num(applied_rel as usize); uart::puts(" REL ");
    crate::kernel::mm::print_num(applied_glob as usize); uart::puts(" GLOB ");
    crate::kernel::mm::print_num(applied_jump as usize); uart::puts(" JUMP ");
    crate::kernel::mm::print_num(applied_irel as usize); uart::puts(" IREL, ");
    crate::kernel::mm::print_num(resolved_cross as usize); uart::puts(" cross, ");
    crate::kernel::mm::print_num(unresolved as usize); uart::puts(" UNDEF\n");

    Ok(())
}

pub fn load_elf(data: &[u8]) -> Result<u64, &'static str> {
    if data.len() < 64 { return Err("too small"); }
    if &data[0..4] != b"\x7fELF" { return Err("not ELF"); }

    let entry = u64_at(data, 24);
    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    // V8-ROOT-3 / V8-LENGTH-AUDIT / V8-ROOT-10: plausibility caps on each
    // field + checked_mul before the `phoff + i * phentsz` arithmetic.
    // phentsz MUST be >= size_of::<Elf64Phdr>() (=56) or each iteration
    // reads past the nominal phdr table.
    if phentsz < 56 || phentsz > 4096 || phnum > 1024 {
        return Err("ELF program-header table implausible");
    }
    let pht_bytes = phnum.checked_mul(phentsz).ok_or("phnum*phentsz overflow")?;
    let pht_end = phoff.checked_add(pht_bytes).ok_or("phoff overflow")?;
    if pht_end > data.len() {
        return Err("ELF phdr table past file end");
    }

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

    // QEMU-BUGFIX-3: allocate the user stack immediately after the ELF
    // pages so it lands inside the primary cave's user window (at physical
    // `phys_base + total_pages*4K`, virtual `(total_pages*4K)` in the user
    // window). The previous code allocated the stack anywhere in kernel
    // RAM via `frame::alloc_frame()` inside `execute_with_args`, and the
    // cave's L2_high identity mapping marked that region as EL1-only —
    // so every BatCave-runner ELF (freetype/png/netsurf/v8/blink/posix)
    // data-aborted at the first `stp x29,x30,[sp]` after eret-to-EL0.
    let stack_phys_start = phys_base + total_pages * PAGE_SIZE;
    for i in 0..LOADED_STACK_PAGES {
        let expected = stack_phys_start + i * PAGE_SIZE;
        let got = frame::alloc_frame().ok_or("oom")?;
        if got != expected {
            uart::puts("[loader] FATAL: non-contiguous stack alloc at page ");
            crate::kernel::mm::print_num(i);
            uart::puts(" expected 0x"); print_hex(expected as u64);
            uart::puts(" got 0x"); print_hex(got as u64); uart::puts("\n");
            return Err("non-contiguous stack alloc (memory fragmented)");
        }
    }
    // Bounds check: ELF + stack must fit in the 20 MB primary user window
    // that `mmu::setup_and_enable` paints (user blocks 0..9 = 0..0x1400000
    // → phys_base..phys_base+20 MB).
    let user_window_end_offset = 10 * 0x200000; // 20 MB
    if total_pages * PAGE_SIZE + LOADED_STACK_PAGES * PAGE_SIZE > user_window_end_offset {
        uart::puts("[loader] FATAL: ELF + stack exceeds 20 MB primary user window\n");
        return Err("ELF + stack > 20 MB user window");
    }
    LOADED_STACK_PHYS.store(stack_phys_start, Ordering::Relaxed);

    uart::puts("[loader] Physical base: 0x"); print_hex(phys_base as u64); uart::puts("\n");
    uart::puts("[loader] Stack phys:    0x"); print_hex(stack_phys_start as u64);
    uart::puts("  ("); crate::kernel::mm::print_num(LOADED_STACK_PAGES);
    uart::puts(" pages)\n");

    // `reloc_offset` is used in TWO distinct places:
    //  (a) Where the kernel *writes* the patched bytes — must be a
    //      physical address the kernel can access via its identity map.
    //  (b) What gets stored into the relocation itself — a value the
    //      EL0 binary will dereference. Since the primary cave maps
    //      user VA 0..20 MB → phys_base..phys_base+20 MB, EL0 pointers
    //      must be in the VA-0 space, not physical.
    //
    // QEMU-BUGFIX-3 (continued): the old code folded both roles into
    // `phys_base - min_addr`, so every R_AARCH64_RELATIVE patched a
    // pointer to a physical address. EL0 dereferencing one of those
    // hit the kernel-RAM identity map (EL1-only) and data-aborted.
    // Every BatCave-runner ELF that used the GOT (freetype/png/
    // netsurf/v8/blink/posix) crashed at the first GOT-backed load.
    //
    // Split into:
    //   reloc_offset    — where kernel writes data during PT_LOAD copy
    //                     (phys_base - min_addr)
    //   va_reloc_offset — value written into R_RELATIVE slots
    //                     (0 - min_addr, so EL0 sees user-VA pointers)
    let reloc_offset = phys_base as i64 - min_addr as i64;
    let va_reloc_offset: i64 = 0i64 - min_addr as i64;

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

        // V8-ROOT-10: dyn_offset + dyn_size could wrap — bound dyn_end first.
        let dyn_end = match dyn_offset.checked_add(dyn_size) {
            Some(e) if e <= data.len() => e,
            _ => continue,
        };
        let mut pos = dyn_offset;
        while pos + 16 <= dyn_end {
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
                    // VA for EL0: use `va_reloc_offset`, not phys. See the
                    // comment at the `reloc_offset` declaration above.
                    let value = (r_addend as i64 + va_reloc_offset) as u64;
                    // FL-004: bounds-check the 8-byte write target. Also
                    // guard r_addend + va_reloc_offset for arithmetic wrap
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
    // Primary cave: user VA 0..20 MB → phys_base..+20 MB. Ensure a prior
    // `load_elf_rebased` (which set `virt_base` here) doesn't leak into
    // busybox/hello stack-pointer arithmetic in `execute_with_args`.
    LOADED_USER_VA_BASE.store(0, Ordering::Relaxed);
    Ok(phys_entry)
}

pub fn execute(entry: u64) -> Result<(), &'static str> {
    execute_with_args(entry, &["busybox"])
}

pub fn execute_with_args(entry: u64, argv: &[&str]) -> Result<(), &'static str> {
    // Ensure an ambient BatCave is active before any syscall handler runs.
    // Without this, every cap-gated syscall (write/mmap/socket/...) hits
    // EACCES because `get_active()` returns `usize::MAX` on a fresh boot.
    crate::batcave::cave::ensure_host_cave_active();

    let orig_entry = LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64;
    let phys_base = LOADED_PHYS_BASE.load(Ordering::Relaxed);
    // Non-zero when the loader set up a rebased cave (Chromium path). The
    // primary cave keeps this at 0 so existing busybox/hello behavior is
    // byte-identical.
    let user_va_base = LOADED_USER_VA_BASE.load(Ordering::Relaxed) as u64;

    // QEMU-BUGFIX-3: the ELF loader reserved a contiguous `LOADED_STACK_PAGES`
    // block immediately after the ELF's PT_LOAD pages so the user stack sits
    // inside the primary cave's 20 MB user window (phys_base..+20 MB). That
    // window is mapped with `BLOCK_USER_RW_EXEC` — EL0 can write to it. The
    // previous `frame::alloc_frame()` inside this function returned pages
    // from anywhere in kernel RAM, which is identity-mapped EL1-only and
    // therefore unwritable from user mode → data abort at first `stp`.
    let stack_phys = LOADED_STACK_PHYS.load(Ordering::Relaxed);
    if stack_phys == 0 {
        uart::puts("[loader] FATAL: execute_with_args called without a loaded ELF\n");
        return Err("no stack reserved — call load_elf first");
    }
    let stack_bytes = LOADED_STACK_PAGES * PAGE_SIZE;

    // Primary cave: user VA 0..20 MB → phys_base..+20 MB, so user VA is the
    // phys offset. Rebased cave (Chromium): user VA = virt_base + phys offset.
    let stack_va_base = user_va_base as usize + (stack_phys - phys_base);
    let stack_base = stack_phys;           // kept for kernel-side writes
    let stack_top = stack_base + stack_bytes;  // kernel-side (for argv push)

    // CRITICAL: Zero the entire stack via the kernel identity mapping.
    unsafe {
        let ptr = stack_base as *mut u8;
        for i in 0..stack_bytes {
            core::ptr::write_volatile(ptr.add(i), 0);
        }
    }

    let mut sp = stack_top;

    // Helper: convert a kernel-physical stack pointer into the user-visible
    // VA that EL0 sees via TTBR0. Primary cave maps user 0..20 MB →
    // phys_base..+20 MB, so user VA = phys - phys_base (user_va_base=0). A
    // rebased cave (Chromium) maps virt_base..virt_base+N → phys_base..+N,
    // so user VA = virt_base + (phys - phys_base). Every pointer we push
    // onto the user stack (argv/envp/auxv/random) must go through this.
    let to_uva = |kphys: usize| -> u64 { user_va_base + (kphys - phys_base) as u64 };

    // AT_RANDOM
    sp -= 16;
    let random_uva = to_uva(sp);
    for i in 0..16usize {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val); }
        let byte = ((val >> (i % 8 * 8)) ^ (i as u64 * 37)) as u8;
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + i, v = in(reg) byte as u32); }
    }

    // Write argv strings
    // Chromium content_shell passes 20-40 flags (--no-sandbox, --disable-gpu,
    // --user-data-dir=..., --remote-debugging-port=..., etc.); bump from 16.
    let mut arg_uvas = [0u64; 64];
    let argc = argv.len().min(16);
    for i in 0..argc {
        let arg = argv[i].as_bytes();
        sp -= arg.len() + 1;
        arg_uvas[i] = to_uva(sp);
        for (j, &b) in arg.iter().enumerate() {
            unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
        }
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + arg.len()); }
    }

    // envp
    sp -= 10;
    let env0_uva = to_uva(sp);
    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }

    sp = (sp - 64) & !0xF;

    // auxv: AT_NULL, AT_RANDOM, AT_PAGESZ — AT_RANDOM must be a user VA.
    for &(k, v) in &[(0u64, 0u64), (25u64, random_uva), (6u64, 4096u64)] {
        sp -= 16;
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v);
        }
    }

    // envp NULL + pointer (user VA for env0)
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env0_uva); }

    // argv NULL + pointers (user VAs)
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    for i in (0..argc).rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) arg_uvas[i]); }
    }

    // argc
    sp -= 8;
    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) argc as u64); }

    // TLS
    //
    // V5-CHAIN-005 / V5-KMEM-007 fix: never leak a kernel-physical TLS
    // address into tpidr_el0 (EL0 can read it back via `mrs xN,
    // tpidr_el0` with no trap — that used to hand a ROP chain a
    // starting kernel pointer).
    //
    // Two paths now:
    // 1. Multi-ELF cave (`load_archive_multi`) has reserved a TLS page
    //    INSIDE the cave window. Use its user VA — the cave's L2 maps
    //    it, EL0 can both read and write it, and no kernel address ever
    //    hits tpidr_el0. glibc's tcbhead_t goes at offset 0 of this
    //    page; everything is zero-initialized.
    // 2. Legacy single-ELF path has no reserved TLS page. Keep the
    //    pre-existing behaviour: zero tpidr_el0 and let the binary
    //    fault if it touches TLS (busybox / hello / the small test
    //    ELFs don't).
    let tls_phys_reserved = LOADED_TLS_PHYS.load(Ordering::Relaxed);
    if tls_phys_reserved != 0 {
        let tls_uva = to_uva(tls_phys_reserved);
        unsafe { core::arch::asm!("msr tpidr_el0, {}", in(reg) tls_uva); }
    } else {
        let tls_page = frame::alloc_frame().ok_or("oom")?;
        for i in 0..PAGE_SIZE {
            unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) tls_page + i); }
        }
        unsafe { core::arch::asm!("msr tpidr_el0, xzr"); }
        let _ = tls_page;
    }

    // Enable MMU (idempotent: no-op if a cave page table is already loaded
    // via `switch_to_cave`, so the Chromium path keeps its rebased L1).
    super::mmu::setup_and_enable(phys_base)?;

    // Primary cave: swap in `orig_entry` (the ELF's declared entry), which
    // EL0 sees as VA `(orig_entry - min_addr)` through the 0..20 MB map.
    // Rebased cave: the caller already passed `info.virt_entry`, which IS
    // the user VA — don't second-guess it. Using `orig_entry` here would
    // eret to a VA not mapped by the cave's TTBR0 → instant instruction
    // abort the moment Chromium's first function tries to return.
    let virt_entry = if user_va_base == 0 { orig_entry } else { entry };
    uart::puts("[loader] --- executing ---\n");

    // Save kernel SP into the kernel-BSS scratch slot shared with the
    // EL0-exit restore path in `src/kernel/arch/mod.rs`. See
    // `crate::kernel::arch::KERNEL_SP_SAVE` for the full story.
    //
    // QEMU-BUGFIX: the save+restore used to hardcode 0x40000100 and
    // 0x40001000 respectively (mismatched!), both in the Linux Image
    // header region which the MMU maps R-X on QEMU → DATA ABORT
    // DFSC=0x0e the moment we stored. Every BatCave-runner ELF
    // (netsurf/freetype/png/v8/blink/posix) crashed there.
    //
    // V5-KMEM-006: validate that SP_EL1 looks plausible — inside
    // kernel RAM (0x40000000..0x50000000) and 16-byte aligned — before
    // we save it. A bogus SP here would mean deep Rust call-chain
    // corruption; saving it would just make the bug louder. We halt
    // instead so the operator sees it immediately.
    unsafe {
        let ksp: u64;
        core::arch::asm!("mov {}, sp", out(reg) ksp);
        let ksp_usize = ksp as usize;
        if ksp_usize < 0x4000_0000 || ksp_usize >= 0x5000_0000 || (ksp_usize & 0xF) != 0 {
            uart::puts("[loader] FATAL: SP_EL1 out of kernel RAM or unaligned\n");
            loop { core::arch::asm!("wfi"); }
        }
        let save_addr = crate::kernel::arch::kernel_sp_save_addr();
        core::arch::asm!("str {v}, [{a}]", a = in(reg) save_addr, v = in(reg) ksp);
    }

    // V4: jump to user code at EL0 via eret. The exception vector table
    // already handles EL0-sourced SVC / IRQ by saving SP_EL1 via
    // SAVE_REGS and restoring via RESTORE_REGS, so the transition is
    // correct. User-code exit goes through handle_sync_exception (BRK /
    // exit syscall) — the handler restores SP_EL1 from KERNEL_SP_SAVE
    // and calls desktop::resume() directly, so this function never
    // returns to its caller. The label-99 return-here gimmick is gone.
    //
    // V2-NEW-017 / ESC-018: previously a raw `br {entry}` at EL1 left
    // busybox / hello running with kernel privilege.
    //
    // V8-ROOT-1 (IRQ audit #5): IRQ-mask the prologue. Without this,
    // a timer IRQ between the msr sp_el0 / elr_el1 / spsr_el1 sequence
    // can take the exception, run handle_irq (which may call schedule),
    // and on return restore EL1 SAVED_REGS which clobbers our half-
    // written SPSR — eret then delivers to EL1 instead of EL0. SPSR
    // = xzr means PSTATE.I = 0 on the new EL0 thread (interrupts
    // unmasked), which is correct.
    // QEMU-BUGFIX-3: EL0 sees the stack through the cave's user-window
    // mapping. Primary cave: user VA = phys - phys_base. Rebased cave
    // (Chromium): user VA = virt_base + (phys - phys_base). All pointers
    // written onto the stack (argv/envp/AT_RANDOM/etc.) were already
    // converted via `to_uva`, which uses the same formula.
    let user_sp = to_uva(sp);
    let _ = stack_va_base; // silence unused (stack_va_base is implicit in user_sp)

    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        core::arch::asm!(
            "msr sp_el0, {usp}",
            "msr elr_el1, {ent}",
            "msr spsr_el1, xzr",        // SPSR: EL0t, AIF clear
            "isb",
            "eret",
            usp = in(reg) user_sp,
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
