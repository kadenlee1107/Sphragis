// Sphragis — Production ELF Loader
// Handles real-world static ARM64 Linux binaries (like busybox).
//
// File-level `allow(dead_code)`: this module carries two parallel
// loader paths: a single-binary path (`load_elf` + `execute_with_args`
// + `load_hello_elf`, all wired) and a multi-binary / dynamic-linking
// path (`load_archive_multi` + `LoadedLib` + `resolve_cross_module` +
// `build_init_trampoline` + `apply_relocs_cross`, currently dormant).
// The multi-binary path is staged for when caves need to load
// dependencies; deleting it would mean re-implementing dynamic linking
// from scratch when that comes due. Kept intentionally.

#![allow(dead_code)]

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
// bumped from 4 (16 KB) to 64 (256 KB).
//
// Plan-agent audit of the 13 Chromium ELFs found total static TLS
// requirements would exceed 16 KB once libc's pthread + libstdc++
// thread_local globals + V8 isolate-thread state are summed. ld-linux
// fails its `_dl_resize_dtv` / `_dl_allocate_tls_init` path silently
// when the static TLS template is too small, ending up in a tight
// loop or a SEGV before main runs.
//
// 256 KB is comfortable headroom for content_shell --version. Real
// Chromium browser-side workloads with full V8/Blink state may need
// more; tune via `audit_chromium_initmap.sh` summary if so.
pub const LOADED_TLS_PAGES: usize = 64;

/// One page for the init_array trampoline. Holds a tiny assembly stub
/// that walks a terminated list of init function VAs, calls each, and
/// finally branches to the main exe's original entry point. Used by
/// `load_archive_multi` when any loaded lib carries a DT_INIT_ARRAY.
pub const LOADED_TRAMPOLINE_PAGES: usize = 1;

/// VA of the init_array trampoline. Set by `load_archive_multi` when
/// one is built; read by `execute_with_args` to override `virt_entry`
/// so eret jumps to the trampoline first instead of the main exe's
/// _start. Zero means "no trampoline, eret directly to main".
static LOADED_TRAMPOLINE_VA: AtomicUsize = AtomicUsize::new(0);

/// Fields populated by `load_archive_multi` so `execute_with_args`
/// can build a glibc-compatible auxv when we eret to ld-linux
/// rather than directly to the main exe's _start.
// /
/// Non-zero `LOADED_INTERP_ENTRY` means "eret here, not to main"; it
/// should be the cave VA of ld-linux's _start. The rest are the
/// values auxv advertises so ld-linux can find content_shell
/// (AT_PHDR / AT_PHENT / AT_PHNUM / AT_ENTRY / AT_BASE).
static LOADED_INTERP_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_INTERP_BASE:  AtomicUsize = AtomicUsize::new(0);
static LOADED_MAIN_PHOFF:   AtomicUsize = AtomicUsize::new(0);
static LOADED_MAIN_PHNUM:   AtomicUsize = AtomicUsize::new(0);
static LOADED_MAIN_PHENT:   AtomicUsize = AtomicUsize::new(0);
pub static SAVED_RETURN_ADDR: AtomicUsize = AtomicUsize::new(0);
static SAVED_KERNEL_SP: AtomicUsize = AtomicUsize::new(0);

/// Physical base of the user stack allocated right after the ELF pages so
/// it sits inside the primary cave's user window (phys_base..+20 MB). See
/// QEMU-BUGFIX-3 comment in `load_elf` for why: the previous code
/// allocated the stack anywhere in kernel RAM, and EL0 couldn't write to
/// it — every Cave-runner ELF faulted at its first `stp` to `sp`.
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
// /
/// V5-PARSER-002 fix: match `load_elf_rebased` bounds discipline
/// exactly. Previously this function lacked:
/// * ELF magic check
/// * vaddr + memsz overflow guard (→ write_bytes(ptr, 0, ~0) wiped RAM)
/// * phys_addr >= phys_base guard (crafted vaddr<min_addr caused
/// underflow, writing somewhere below phys_base)
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
// /
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
    /// PT_TLS segment: vaddr, file/mem sizes, alignment. Zero if none.
    pub tls_vaddr: u64,
    pub tls_filesz: usize,
    pub tls_memsz: usize,
    pub tls_align: usize,
    /// Offset from tpidr_el0 where THIS lib's TLS block starts. Set
    /// AFTER all libs are loaded; used to compute TLS_TPREL64 values.
    pub tls_tp_offset: usize,
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
            tls_vaddr: 0, tls_filesz: 0, tls_memsz: 0, tls_align: 0,
            tls_tp_offset: 0,
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
// /
/// `frame::free_contig` zeroes each page on free, so the residue from
/// the previous tenant is wiped before the frames re-enter the pool.
pub fn free_loaded_elf(info: &LoadedElfInfo) {
    if info.phys_base == 0 || info.total_size == 0 { return; }
    let pages = (info.total_size + 4095) / 4096;
    crate::kernel::mm::frame::free_contig(info.phys_base, pages);
}

/// Load a PIE ELF with virtual addresses rebased to `virt_base`.
// /
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
    // patch_offset — where we WRITE during load (physical, identity-mapped)
    // value_offset — what VALUE we write in relocations (the VA the binary
    // will see at runtime, once switch_to_cave installs the
    // cave's TTBR0 mapping virt_base..virt_base+total_size
    // to phys_base..phys_base+total_size)
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
    // R_AARCH64_RELATIVE (0x403): *R = addend + value_offset
    // R_AARCH64_GLOB_DAT (0x401): *R = sym_value + addend + value_offset
    // R_AARCH64_IRELATIVE (0x408): treat like RELATIVE for now — real
    // support requires calling the resolver;
    // this makes a call through the GOT hit
    // the resolver itself rather than 0x0.
    // R_AARCH64_ABS32 (0x101): narrow 32-bit variant of GLOB_DAT, rare.
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

                // V8-RELOC-LEAK: catch any relocation that produces a
                // KERNEL VA (0x40000000..0x80000000). User-mode code can't
                // legitimately call/load through such a value — calling
                // would EL0-instruction-abort, loading a kernel data
                // pointer is meaningless. If we ever see one, log it
                // loudly: tells us which (sym, addend, value_offset)
                // produced it and where in the user binary the slot lives.
                let value_check = |val: u64, kind: &str| {
                    if val >= 0x40000000 && val < 0x80000000 {
                        uart::puts("[reloc] KERNEL-VA value 0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((val >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" written to user GOT slot uva=0x");
                        let uva = (r_offset as i64 + value_offset) as u64;
                        for sh in (0..16).rev() {
                            uart::putc(hex[((uva >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" via "); uart::puts(kind);
                        uart::puts(" addend=0x");
                        for sh in (0..16).rev() {
                            uart::putc(hex[((r_addend >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts("\n");
                    }
                };
                match r_type {
                    0x403 => {
                        // R_AARCH64_RELATIVE: base + addend
                        let value = (r_addend as i64 + value_offset) as u64;
                        value_check(value, "RELATIVE");
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
                        value_check(value, if r_type == 0x402 { "JUMP_SLOT" } else { "GLOB_DAT" });
                        unsafe {
                            core::arch::asm!("str {v}, [{a}]",
                                a = in(reg) patch_addr, v = in(reg) value);
                        }
                        if r_type == 0x402 { applied_jump += 1; }
                        else               { applied_glob += 1; }
                    }
                    0x408 => {
                        // ROOT: R_AARCH64_IRELATIVE — actually
                        // call the resolver at EL1 to get the resolved impl
                        // address. See the loader/multi handler above for
                        // the full root-cause writeup. Without this fix
                        // PA::Free's call through RemaskPointer IFUNC reads
                        // the wrong address and BRKs on a CHECK.
                        let resolver_pa = (r_addend as i64 + patch_offset) as u64;
                        let result_pa: u64;
                        unsafe {
                            core::arch::asm!(
                                "blr {f}",
                                f = in(reg) resolver_pa,
                                inout("x0") 0u64 => result_pa,
                                in("x1") 0u64,
                                clobber_abi("C"),
                            );
                        }
                        let value = (result_pa as i64
                                     + value_offset
                                     - patch_offset) as u64;
                        value_check(value, "IRELATIVE");
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
// /
/// This is a minimal in-kernel dynamic linker: after each ELF is loaded
/// and has its R_AARCH64_RELATIVE applied, we do a second pass over
/// every lib's GLOB_DAT / JUMP_SLOT / IRELATIVE entries — for any
/// SHN_UNDEF symbol, we scan every other loaded lib's symbol table
/// and write the real resolved address over the sentinel we left in
/// place during the first pass.
// /
/// No symbol versioning (DT_VERSYM / DT_VERNEED), no init_array
/// execution, no TLS setup. This is only enough to get content_shell
/// to reach its first real libc call and exercise the syscall surface.
// /
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
    let tramp_va_offset = tls_va_offset   + LOADED_TLS_PAGES   * PAGE_SIZE; // trampoline after TLS
    let grand_total_pages = total_pages + LOADED_STACK_PAGES
        + LOADED_TLS_PAGES + LOADED_TRAMPOLINE_PAGES;

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

    // First pass per lib: copy PT_LOADs + parse .dynamic ---
    for i in 0..lib_count {
        stage_copy_and_parse(&mut libs[i])?;
    }

    // CHROMIUM-PHASE-B: detect the interpreter BEFORE we apply
    // relocations. apply_relocs_cross reads LOADED_INTERP_ENTRY to
    // decide whether to skip JUMP_SLOT/GLOB_DAT for the main exe —
    // see the `skip_cross_for_main` comment inside that function.
    // Without this the statics were only populated AFTER the reloc
    // loop, so the skip-check never fired.
    {
        const INTERP_NAME: &[u8] = b"lib/ld-linux-aarch64.so.1";
        for i in 1..lib_count {
            if libs[i].name() == INTERP_NAME {
                LOADED_INTERP_ENTRY.store(
                    libs[i].info.virt_entry as usize, Ordering::Relaxed);
                LOADED_INTERP_BASE.store(
                    libs[i].info.virt_base as usize, Ordering::Relaxed);
                uart::puts("[loader/multi] interp early-detect: ");
                for &b in libs[i].name() { uart::putc(b); }
                uart::puts(" base=0x"); print_hex(libs[i].info.virt_base);
                uart::puts(" entry=0x"); print_hex(libs[i].info.virt_entry);
                uart::puts("\n");
                break;
            }
        }
    }

    // Second pass per lib: apply relocations with cross-module lookup ---
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
    let tramp_phys = phys_base + tramp_va_offset;

    // CHROMIUM-PHASE-B: zero the stack + TLS region before handing
    // control to EL0. Without this, alloc_contig() may hand back
    // pages that still contain bytes from a previous tenant. Content
    // shell's startup code reads its own uninitialised stack in a
    // few places (glibc _start reserves a scratch area; ICU's TZ
    // path loads 8 bytes at a time via strlen magic); stale pointers
    // there caused an immediate non-canonical-VA data abort with
    // FAR=0x70797400... (ASCII "pyt\0" bleeding into a pointer).
    unsafe {
        let zero_start = stack_phys;
        let zero_end = tls_phys + LOADED_TLS_PAGES * PAGE_SIZE;
        let mut p = zero_start;
        while p < zero_end {
            core::ptr::write_volatile(p as *mut u64, 0);
            p += 8;
        }
    }
    let tramp_va   = cave_virt_base as usize + tramp_va_offset;

    // Build the init_array trampoline. Emits a tiny aarch64 stub that
    // walks a null-terminated list of init function VAs, calls each
    // via BLR, then BRs to the main exe's real entry. The main exe
    // gets the stack glibc's _start expects (argc/argv/envp/auxv
    // already placed there by `execute_with_args`), and init
    // constructors run BEFORE _start — exactly what ld-linux does.
    //
    // Layout in the trampoline page:
    // 0x00..0x20: code (8 instructions)
    // 0x20..0x28: main_entry VA
    // 0x28..: init function VAs, terminated by 0
    // SPHRAGIS_DISABLE_INIT_TRAMPOLINE=1 at build time disables the
    // init_array trampoline. Useful while we iterate on TLS / rtld
    // setup — without the trampoline content_shell gets to V8 heap
    // setup before wedging; with it we crash earlier in libc init
    // because its constructors dereference uninitialized _rtld_global
    // state.
    const DISABLE_TRAMPOLINE: bool =
        option_env!("SPHRAGIS_DISABLE_INIT_TRAMPOLINE").is_some();
    if !DISABLE_TRAMPOLINE {
        let tramp_entry_va = build_init_trampoline(
            tramp_phys,
            tramp_va as u64,
            &libs,
            lib_count,
            libs[0].info.virt_entry,  // real main entry
        )?;
        LOADED_TRAMPOLINE_VA.store(tramp_entry_va as usize, Ordering::Relaxed);
    } else {
        uart::puts("[loader/multi] init-trampoline DISABLED via env\n");
        let _ = build_init_trampoline; // silence dead_code
    }

    // Additional flush for the trampoline page so the icache sees it
    // once the CPU eret's there.
    unsafe {
        let tramp_end = tramp_phys + LOADED_TRAMPOLINE_PAGES * PAGE_SIZE;
        let mut a = tramp_phys;
        while a < tramp_end {
            core::arch::asm!("dc cvau, {a}", a = in(reg) a);
            core::arch::asm!("ic ivau, {a}", a = in(reg) a);
            a += 64;
        }
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    // Initialize the TLS block per the aarch64 Variant I ABI.
    //
    // Layout (16 KB total, LOADED_TLS_PAGES pages):
    // [tp - 0]..[tp + 16): tcbhead_t (dtv ptr, self, pad, pad)
    // [tp + 16)..: per-module TLS data, packed in the order
    // libs[0], libs[1], ... with each module's
    // memsz rounded up to p_align. The file
    // portion is copied verbatim from each
    // lib's PT_TLS; bss portion is zero.
    //
    // `lib.tls_tp_offset` is set to the offset-from-tp where that
    // module's data starts — this is the value TLS_TPREL64 relocs add
    // to `sym.st_value` when computing the final offset in the GOT.
    //
    // TCB_SIZE on aarch64 (for glibc) is 16 bytes. First module's data
    // starts at tp + 16.
    const TCB_SIZE: usize = 16;
    let tls_user_va = user_va_base_from_libs(&libs, 0) + tls_va_offset as u64;
    let tls_total_bytes = LOADED_TLS_PAGES * PAGE_SIZE;

    // First, zero the whole block. tcbhead_t fields stay zero except
    // `self` (offset 8) which we fill below with tls_user_va so any
    // code that reads `[tp + 8]` (glibc does for robust-list / etc.)
    // gets a non-null pointer back.
    unsafe {
        let mut p = tls_phys;
        while p < tls_phys + tls_total_bytes {
            core::arch::asm!("str xzr, [{a}]", a = in(reg) p);
            p += 8;
        }
        // `self` pointer at tp + 8 (points to the tcbhead_t itself).
        core::ptr::write_volatile((tls_phys + 8) as *mut u64, tls_user_va);
    }

    // Assign each lib a TLS offset and copy its template in.
    let mut cur_tp_offset: usize = TCB_SIZE;
    for i in 0..lib_count {
        if libs[i].tls_memsz == 0 { continue; }
        let align = libs[i].tls_align.max(1);
        cur_tp_offset = (cur_tp_offset + align - 1) & !(align - 1);
        libs[i].tls_tp_offset = cur_tp_offset;
        // Copy the template from the lib's data (file-offset derived
        // from tls_vaddr via vaddr_to_file_off).
        let data_ref = libs[i].data();
        let phoff_i = u64_at(data_ref, 32) as usize;
        let phnum_i = u16_at(data_ref, 56) as usize;
        let phentsz_i = u16_at(data_ref, 54) as usize;
        let src_off = vaddr_to_file_off(
            data_ref, phoff_i, phnum_i, phentsz_i,
            libs[i].tls_vaddr as usize,
        );
        let dst_phys = tls_phys + cur_tp_offset;
        if src_off != usize::MAX
            && src_off + libs[i].tls_filesz <= data_ref.len()
            && cur_tp_offset + libs[i].tls_memsz <= tls_total_bytes
        {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data_ref.as_ptr().add(src_off),
                    dst_phys as *mut u8,
                    libs[i].tls_filesz,
                );
                // Zero the BSS portion (memsz - filesz).
                if libs[i].tls_memsz > libs[i].tls_filesz {
                    core::ptr::write_bytes(
                        (dst_phys + libs[i].tls_filesz) as *mut u8,
                        0,
                        libs[i].tls_memsz - libs[i].tls_filesz,
                    );
                }
            }
        }
        uart::puts("[tls] ");
        for &b in libs[i].name() { uart::putc(b); }
        uart::puts(" tp+0x"); print_hex(cur_tp_offset as u64);
        uart::puts(" filesz="); crate::kernel::mm::print_num(libs[i].tls_filesz);
        uart::puts(" memsz="); crate::kernel::mm::print_num(libs[i].tls_memsz);
        uart::puts("\n");
        cur_tp_offset += libs[i].tls_memsz;
    }
    if cur_tp_offset > tls_total_bytes {
        uart::puts("[tls] WARN: overflow — combined TLS > ");
        crate::kernel::mm::print_num(tls_total_bytes);
        uart::puts("\n");
    }

    // Publish loader state for `execute_with_args`. virt_entry + phys_base
    // + user_va_base all correspond to the MAIN exe (libs[0]).
    LOADED_ENTRY.store(libs[0].info.virt_entry as usize, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(u64_at(libs[0].data(), 24) as usize, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(libs[0].info.phys_base, Ordering::Relaxed);
    LOADED_STACK_PHYS.store(stack_phys, Ordering::Relaxed);
    LOADED_USER_VA_BASE.store(libs[0].info.virt_base as usize, Ordering::Relaxed);
    LOADED_TLS_PHYS.store(tls_phys, Ordering::Relaxed);

    // Publish main-exe ELF-header fields auxv needs, plus find the ELF
    // interpreter (ld-linux) in the loaded lib list and publish its
    // entry VA + base VA. When both interp entry and base are non-zero,
    // `execute_with_args` eret's to ld-linux and hands it a full auxv
    // with AT_PHDR / AT_PHENT / AT_PHNUM / AT_ENTRY / AT_BASE so
    // ld-linux can find the main exe and do its job.
    let main_data = libs[0].data();
    let main_phoff_raw = u64_at(main_data, 32) as usize;
    let main_phnum_raw = u16_at(main_data, 56) as usize;
    let main_phent_raw = u16_at(main_data, 54) as usize;
    LOADED_MAIN_PHOFF.store(main_phoff_raw, Ordering::Relaxed);
    LOADED_MAIN_PHNUM.store(main_phnum_raw, Ordering::Relaxed);
    LOADED_MAIN_PHENT.store(main_phent_raw, Ordering::Relaxed);

    // Interpreter detection: match by filename. The bake script packs
    // ld-linux under `lib/ld-linux-aarch64.so.1` (see
    // tools/bake_chromium_archive.sh).
    const INTERP_NAME: &[u8] = b"lib/ld-linux-aarch64.so.1";
    for i in 1..lib_count {
        if libs[i].name() == INTERP_NAME {
            LOADED_INTERP_ENTRY.store(libs[i].info.virt_entry as usize, Ordering::Relaxed);
            LOADED_INTERP_BASE.store(libs[i].info.virt_base as usize, Ordering::Relaxed);
            uart::puts("[loader/multi] interpreter found: ");
            for &b in libs[i].name() { uart::putc(b); }
            uart::puts(" base=0x"); print_hex(libs[i].info.virt_base);
            uart::puts(" entry=0x"); print_hex(libs[i].info.virt_entry);
            uart::puts("\n");
            break;
        }
    }

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

    // Parse PT_TLS (p_type == 7). Captures the TLS template's
    // location + size so the loader can lay out a combined TLS block
    // across all libs and copy template data in at the right offset.
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 7 { continue; }
        let p_vaddr = u64_at(data, ph + 16);
        let p_filesz = u64_at(data, ph + 32) as usize;
        let p_memsz  = u64_at(data, ph + 40) as usize;
        let p_align  = u64_at(data, ph + 48) as usize;
        lib.tls_vaddr  = p_vaddr;
        lib.tls_filesz = p_filesz;
        lib.tls_memsz  = p_memsz;
        lib.tls_align  = p_align.max(1);
        break; // at most one PT_TLS per ELF
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

/// Helper — the user VA of libs[0] (the main exe). Just a shorthand
/// since the expression shows up in a couple of spots.
fn user_va_base_from_libs(libs: &[LoadedLib; MAX_LOADED_LIBS], idx: usize) -> u64 {
    libs[idx].info.virt_base
}

/// Build the init_array trampoline in a cave-resident page. Writes
/// ~32 bytes of aarch64 code followed by a main-entry slot and a
/// null-terminated array of init function VAs (gathered from every
/// loaded lib's DT_INIT_ARRAY).
// /
/// Layout in the page:
/// offset 0x00: adr x19, init_list ; offset +0x28
/// 0x04: adr x20, main_slot ; offset +0x1c
/// 0x08: loop: ldr x21, [x19], #8 ; post-indexed
/// 0x0c: cbz x21, done ; +3 words
/// 0x10: blr x21
/// 0x14: b loop ; -3 words
/// 0x18: done: ldr x20, [x20] ; deref slot
/// 0x1c: br x20
/// 0x20: main_slot (u64)
/// 0x28: init_list:
/// .quad init_fn_0
/// .quad init_fn_1
/// ...
/// .quad 0
// /
/// Initialization order: we walk `libs[0..lib_count]` in archive
/// order (matches the DT_NEEDED dependency pass: the main exe is
/// files[0], then libs). ld-linux normally runs in reverse-dependency
/// order (leaf libs first) — close enough for the common case where
/// inits are idempotent and don't depend on each other's internal
/// state. If that assumption breaks for specific apps, we'll
/// iterate.
// /
/// Returns the VA of the first instruction (the stub entry).
fn build_init_trampoline(
    tramp_phys: usize,
    tramp_va: u64,
    libs: &[LoadedLib; MAX_LOADED_LIBS],
    lib_count: usize,
    main_entry: u64,
) -> Result<u64, &'static str> {
    // Aarch64 instruction encoders (just enough for this stub).
    fn enc_adr(rd: u32, imm: i32) -> u32 {
        let imm = imm as u32;
        let immlo = (imm & 0x3) << 29;
        let immhi = ((imm >> 2) & 0x7FFFF) << 5;
        0x1000_0000 | immlo | immhi | (rd & 0x1F)
    }
    // Post-indexed LDR of 64-bit Xt from [Xn], #imm8
    fn enc_ldr_post(rt: u32, rn: u32, imm9: i32) -> u32 {
        let imm = (imm9 as u32) & 0x1FF;
        0xF840_0400 | (imm << 12) | ((rn & 0x1F) << 5) | (rt & 0x1F)
    }
    // CBZ Xt, #imm (signed 19-bit in words)
    fn enc_cbz(rt: u32, imm_words: i32) -> u32 {
        let imm = (imm_words as u32) & 0x7FFFF;
        0xB400_0000 | (imm << 5) | (rt & 0x1F)
    }
    fn enc_b(imm_words: i32) -> u32 {
        let imm = (imm_words as u32) & 0x03FFFFFF;
        0x1400_0000 | imm
    }
    fn enc_ldr_unsigned(rt: u32, rn: u32) -> u32 {
        0xF940_0000 | ((rn & 0x1F) << 5) | (rt & 0x1F)
    }
    fn enc_br(rn: u32) -> u32 {
        0xD61F_0000 | ((rn & 0x1F) << 5)
    }
    fn enc_blr(rn: u32) -> u32 {
        // Not used here — kept for reference. BLR Xn.
        0xD63F_0000 | ((rn & 0x1F) << 5)
    }
    let _ = enc_blr;

    // Collect init function VAs into the page. Main slot at 0x20,
    // init list from 0x28 onward.
    let code: [u32; 8] = [
        enc_adr(19, 0x28),          // adr x19, init_list (+40 from PC=0x00)
        enc_adr(20, 0x1c),          // adr x20, main_slot (+28 from PC=0x04)
        enc_ldr_post(21, 19, 8),    // loop: ldr x21, [x19], #8
        enc_cbz(21, 3),             // cbz x21, done (+3 words from PC=0x0c)
        0xD63F_02A0,                // blr x21
        enc_b(-3),                  // b loop (-3 words from PC=0x14)
        enc_ldr_unsigned(20, 20),   // done: ldr x20, [x20]
        enc_br(20),                 // br x20
    ];

    // Write code.
    for (i, &insn) in code.iter().enumerate() {
        unsafe {
            core::ptr::write_volatile(
                (tramp_phys + i * 4) as *mut u32,
                insn,
            );
        }
    }

    // Write main_slot at offset 0x20.
    unsafe {
        core::ptr::write_volatile(
            (tramp_phys + 0x20) as *mut u64,
            main_entry,
        );
    }

    // Walk each lib's init_array and write entries starting at 0x28.
    let init_list_base = tramp_phys + 0x28;
    let mut slot = 0usize;
    let max_entries = (PAGE_SIZE - 0x28) / 8 - 1; // leave room for terminator
    for i in 0..lib_count {
        let lib = &libs[i];
        if lib.init_array_sz == 0 { continue; }
        let ia_phys = (lib.init_array_va as i64 + lib.patch_offset) as u64;
        let n = (lib.init_array_sz / 8) as usize;
        for j in 0..n {
            if slot >= max_entries {
                return Err("too many init_array entries for one-page trampoline");
            }
            let entry = unsafe {
                core::ptr::read_volatile((ia_phys + (j * 8) as u64) as *const u64)
            };
            if entry == 0 || entry == !0 { continue; } // skip placeholder slots
            unsafe {
                core::ptr::write_volatile(
                    (init_list_base + slot * 8) as *mut u64,
                    entry,
                );
            }
            slot += 1;
        }
    }
    // Null terminator.
    unsafe {
        core::ptr::write_volatile(
            (init_list_base + slot * 8) as *mut u64,
            0,
        );
    }

    uart::puts("[trampoline] built at va=0x"); print_hex(tramp_va);
    uart::puts(" phys=0x"); print_hex(tramp_phys as u64);
    uart::puts(" inits="); crate::kernel::mm::print_num(slot);
    uart::puts(" main=0x"); print_hex(main_entry);
    uart::puts("\n");

    Ok(tramp_va)
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

    // CHROMIUM-PHASE-B: when we're going to hand off to real ld-linux,
    // SKIP cross-module JUMP_SLOT/GLOB_DAT/TPREL relocations for the
    // MAIN EXECUTABLE (idx == 0). Our loader was resolving those to
    // the kernel-side copies of libc etc. — addresses like 0x20427780
    // that ld-linux's freshly-mmap'd libc at 0x11b10000 doesn't
    // populate. Leaving the main exe's .got / .got.plt at their
    // file-default values lets ld-linux's own relocator write the
    // correct addresses during its symbol-resolution pass.
    //
    // We still apply RELATIVE (0x403) and IRELATIVE (0x408) for the
    // main exe — those are base-rebase operations that don't depend
    // on external symbols, so ld-linux would write the same result.
    let running_interp = LOADED_INTERP_ENTRY.load(Ordering::Relaxed) != 0;
    let skip_cross_for_main = running_interp && idx == 0;
    if skip_cross_for_main {
        uart::puts("[loader/multi] ");
        for &b in lib.name() { uart::putc(b); }
        uart::puts(": interp present, skipping JUMP_SLOT/GLOB_DAT/TPREL (ld-linux will resolve)\n");
    }

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

            // V8-RELOC-LEAK (loader/multi): same kernel-VA detector as
            // the single-binary path. Logs any reloc that produces a
            // value in 0x40000000..0x80000000 — those are kernel
            // addresses and shouldn't ever be written to user GOT/data.
            let value_check_multi = |val: u64, kind: &str| {
                if val >= 0x40000000 && val < 0x80000000 {
                    uart::puts("[reloc/multi] KERNEL-VA value 0x");
                    let hex = b"0123456789abcdef";
                    for sh in (0..16).rev() {
                        uart::putc(hex[((val >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts(" written via "); uart::puts(kind);
                    uart::puts(" addend=0x");
                    for sh in (0..16).rev() {
                        uart::putc(hex[((r_addend >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts(" patch_addr=0x");
                    for sh in (0..16).rev() {
                        uart::putc(hex[((patch_addr as u64 >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts(" value_offset=0x");
                    for sh in (0..16).rev() {
                        uart::putc(hex[((value_offset as u64 >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts("\n");
                }
            };
            match r_type {
                0x403 => {
                    let value = (r_addend as i64 + value_offset) as u64;
                    value_check_multi(value, "RELATIVE");
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    applied_rel += 1;
                }
                0x401 | 0x402 => {
                    if skip_cross_for_main { continue; }
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
                    value_check_multi(value, if r_type == 0x402 { "JUMP_SLOT" } else { "GLOB_DAT" });
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    if r_type == 0x402 { applied_jump += 1; } else { applied_glob += 1; }
                }
                0x408 => {
                    // ROOT: R_AARCH64_IRELATIVE — the GOT slot
                    // must hold the RESOLVED implementation address, not
                    // the resolver itself. Previously we wrote the resolver
                    // into the slot; the IPLT trampoline branched to the
                    // resolver and treated its return value as the operation's
                    // result, when in fact that return value IS a function
                    // pointer the caller expected to dereference indirectly.
                    //
                    // For PA's RemaskPointer IFUNC, this caused the IPLT
                    // call to return the no-MTE remask function address
                    // (0x1a4ff44 = `bti c; ret`) instead of the slot pointer
                    // PA passed in. PA then `ldar w8, [x24]` read the bytes
                    // at 0x1a4ff44 (a `bti c` opcode) instead of the
                    // InSlotMetadata refcount, eventually triggering
                    // `DoubleFreeOrCorruptionDetected` BRK.
                    //
                    // Fix: actually call the resolver from EL1. It runs at
                    // its physical address (PC-relative `adrp` gives PA-
                    // based results), so we must convert the PA returned to
                    // a runtime VA before storing.
                    let resolver_pa = (r_addend as i64 + patch_offset) as u64;
                    let result_pa: u64;
                    unsafe {
                        core::arch::asm!(
                            "blr {f}",
                            f = in(reg) resolver_pa,
                            inout("x0") 0u64 => result_pa,
                            in("x1") 0u64,
                            clobber_abi("C"),
                        );
                    }
                    let value = (result_pa as i64
                                 + value_offset
                                 - patch_offset) as u64;
                    value_check_multi(value, "IRELATIVE");
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    applied_irel += 1;
                }
                0x406 => {
                    // R_AARCH64_TLS_TPREL64: *R = sym.st_value + addend +
                    // lib_tls_offset. glibc's initial-exec-model code
                    // reads this GOT slot and adds `tpidr_el0` to get
                    // the absolute address of the TLS variable. Without
                    // this, every TLS access through GLOB_DAT produces
                    // zero and glibc NULL-derefs (libc.so.6+0x27824
                    // observed).
                    if skip_cross_for_main { continue; }
                    if symtab_file == usize::MAX { continue; }
                    let sym_off = match symtab_file
                        .checked_add(r_sym.checked_mul(24).unwrap_or(usize::MAX))
                    {
                        Some(v) if v + 24 <= data.len() => v,
                        _ => continue,
                    };
                    let sym_value = u64_at(data, sym_off + 8);
                    let tls_off = lib.tls_tp_offset as u64;
                    let value = sym_value
                        .wrapping_add(r_addend as u64)
                        .wrapping_add(tls_off);
                    unsafe {
                        core::arch::asm!("str {v}, [{a}]",
                            a = in(reg) patch_addr, v = in(reg) value);
                    }
                    // Counted in applied_irel to avoid another counter
                    // for this rare type; logs will show IREL counts
                    // slightly inflated when TLS relocs are present.
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

    // Allocate 2MB-aligned for MMU block mapping. Spin-for-alignment
    // leaks frames (up to 511 per load after pool fragmentation) —
    // a real `frame::alloc_aligned(pages, 2MB)` helper is the proper
    // fix; until then we verify each subsequent alloc is contiguous
    // so we crash loudly rather than corrupt random memory.
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
    // so every Cave-runner ELF (freetype/png/netsurf/v8/blink/posix)
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
    // (a) Where the kernel *writes* the patched bytes — must be a
    // physical address the kernel can access via its identity map.
    // (b) What gets stored into the relocation itself — a value the
    // EL0 binary will dereference. Since the primary cave maps
    // user VA 0..20 MB → phys_base..phys_base+20 MB, EL0 pointers
    // must be in the VA-0 space, not physical.
    //
    // QEMU-BUGFIX-3 (continued): the old code folded both roles into
    // `phys_base - min_addr`, so every R_AARCH64_RELATIVE patched a
    // pointer to a physical address. EL0 dereferencing one of those
    // hit the kernel-RAM identity map (EL1-only) and data-aborted.
    // Every Cave-runner ELF that used the GOT (freetype/png/
    // netsurf/v8/blink/posix) crashed at the first GOT-backed load.
    //
    // Split into:
    // reloc_offset — where kernel writes data during PT_LOAD copy
    // (phys_base - min_addr)
    // va_reloc_offset — value written into R_RELATIVE slots
    // (0 - min_addr, so EL0 sees user-VA pointers)
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
    // Ensure an ambient Cave is active before any syscall handler runs.
    // Without this, every cap-gated syscall (write/mmap/socket/...) hits
    // EACCES because `get_active()` returns `usize::MAX` on a fresh boot.
    crate::caves::cave::ensure_host_cave_active();

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

    // AT_RANDOM — 16 bytes glibc reads to seed the stack canary
    // (bytes 8-15) and the pointer-mangling cookie (bytes 0-7).
    //
    // iter 13: a tight cntpct_el0 loop only spans a few
    // ticks of the 24 MHz counter, so 16 successive reads share most
    // of their high bits. Worse, the XOR-with-i pattern can produce
    // zero bytes in the canary slots. glibc's stack canary set from
    // such a low-entropy seed lands close to zero or the same value
    // as some uninitialized stack slot — when a function epilogue
    // compares saved vs current canary, the values match in fragile
    // ways that get clobbered by mid-init writes. Result: a
    // late-init `*** stack smashing detected ***`.
    //
    // Use the ARMv8.5 RNDR system register if available; fall back
    // to mixing several cntpct samples + boot-image hash so each
    // byte has independent entropy. We also force the LOW byte of
    // the canary slot (byte 8) to be NON-ZERO to satisfy glibc's
    // "canary first byte is 0 to stop strcpy()" convention while
    // still having strong entropy.
    sp -= 16;
    let random_uva = to_uva(sp);
    {
        let mut rnd = [0u8; 16];
        // SplitMix-style entropy stretch on multiple cntpct samples.
        // Each iteration pulls a fresh cntpct (HVF advances it on
        // every host-trap) and mixes via xorshift constants. The
        // RNDR register is unavailable under HVF (raises SIGILL); we
        // depend on cntpct-mixing alone, which is fine for canary
        // seeding because we only need 16 hard-to-predict bytes,
        // not crypto-grade randomness.
        let mut acc: u64 = 0x9E37_79B9_7F4A_7C15;
        for i in 0..16usize {
            let mut t: u64;
            unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) t); }
            // Xorshift the counter to spread its low entropy across
            // all 64 bits before mixing into the accumulator.
            t ^= t >> 17;
            t = t.wrapping_mul(0xff51afd7ed558ccd);
            t ^= t >> 33;
            t = t.wrapping_mul(0xc4ceb9fe1a85ec53);
            t ^= t >> 33;
            acc = acc.wrapping_add(t).rotate_left((i * 7 + 13) as u32);
            rnd[i] = (acc & 0xff) as u8;
        }
        // glibc convention: AT_RANDOM[8] (canary low byte) is 0 to
        // stop strcpy() from copying through the canary. Force it.
        rnd[8] = 0;
        // Avoid an all-zero canary in the unlikely event entropy
        // collapsed.
        if rnd[9] | rnd[10] | rnd[11] | rnd[12] | rnd[13] | rnd[14] | rnd[15] == 0 {
            rnd[15] = 0xa5;
        }
        for i in 0..16usize {
            unsafe { core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) sp + i, v = in(reg) rnd[i] as u32); }
        }
    }

    // Write argv strings
    // Chromium content_shell passes 20-40 flags (--no-sandbox, --disable-gpu,
    // user-data-dir=..., --remote-debugging-port=..., etc.); bump from 16.
    let mut arg_uvas = [0u64; 64];
    let argc = argv.len().min(32);
    for i in 0..argc {
        let arg = argv[i].as_bytes();
        sp -= arg.len() + 1;
        arg_uvas[i] = to_uva(sp);
        for (j, &b) in arg.iter().enumerate() {
            unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
        }
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + arg.len()); }
    }

    // envp — push each entry string to the stack. glibc and ICU
    // read several of these during startup:
    // PATH=/bin (basic $PATH)
    // TZ=UTC — lets ICU skip the filesystem-based
    // timezone lookup that was crashing
    // in getAliasTargetAsResourceBundle.
    // HOME=/root (stops glibc from looking it up via getpw)
    // LANG=C.UTF-8 (UTF-8 locale, no collation lookup)
    sp -= 10;
    let env0_uva = to_uva(sp);
    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }
    sp -= 7;
    let env1_uva = to_uva(sp);
    for (j, &b) in b"TZ=UTC\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }
    sp -= 11;
    let env2_uva = to_uva(sp);
    for (j, &b) in b"HOME=/root\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }
    sp -= 12;
    let env3_uva = to_uva(sp);
    for (j, &b) in b"LANG=C.UTF-8\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }

    sp = (sp - 256) & !0xF; // leave headroom for extended auxv

    // ensure FINAL SP (process-entry SP) lands 16-byte
    // aligned regardless of argc. After this padding sp is 16-aligned.
    // Subsequent pushes are: auxv (multiples of 16), envp ptrs (40
    // bytes = 8 % 16), argv ptrs ((argc+1)*8 bytes), argc word (8).
    // Total non-aligned tail: 40 + (argc+1)*8 + 8 = 48 + (argc+1)*8.
    // For final to be 16-aligned: (48 + (argc+1)*8) % 16 == 0
    // → (argc+1)*8 % 16 == 0 (since 48 % 16 == 0)
    // → (argc+1) must be even → argc must be ODD
    // If argc is EVEN, push 8 extra bytes here to swap parity.
    //
    // Without this fix, adding any extra command-line flag (going from
    // argc=11 to argc=12) misaligned process-entry SP by 8 bytes,
    // which corrupted ICU CharString::buffer somewhere downstream
    // ("typeMap/timezone" stack-data bytes leaking into a pointer
    // field, with the high 4 bytes spelling "pyt\0").
    if argc % 2 == 0 {
        sp -= 8;
        unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    }

    // auxv. When ld-linux is the interpreter we're eret'ing to, give it
    // enough to find the main exe:
    // AT_PHDR (3) = main_virt_base + main_phoff
    // AT_PHENT (4) = sizeof(Elf64_Phdr) = 56
    // AT_PHNUM (5) = main's phnum
    // AT_ENTRY (9) = main exe's rebased entry VA
    // AT_BASE (7) = ld-linux's load base (virt_base)
    // AT_PAGESZ(6), AT_RANDOM(25), AT_NULL(0)
    // When NOT running ld-linux (legacy single-ELF / old multi-ELF
    // path without an interpreter), only the original 3 entries are
    // emitted so existing binaries stay binary-compatible.
    let interp_entry = LOADED_INTERP_ENTRY.load(Ordering::Relaxed) as u64;
    let interp_base  = LOADED_INTERP_BASE.load(Ordering::Relaxed) as u64;
    let main_phoff   = LOADED_MAIN_PHOFF.load(Ordering::Relaxed) as u64;
    let main_phnum   = LOADED_MAIN_PHNUM.load(Ordering::Relaxed) as u64;
    let main_phent   = LOADED_MAIN_PHENT.load(Ordering::Relaxed) as u64;
    let running_interp = interp_entry != 0 && interp_base != 0;
    let main_entry_va = user_va_base + (LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64);
    let main_phdr_va  = user_va_base + main_phoff;

    // b: AT_HWCAP advertises CPU features. Without it,
    // glibc's `init_have_lse_atomics` calls `getauxval(AT_HWCAP)`,
    // sees 0, sets `__aarch64_have_lse_atomics = 0`, and ALL outline
    // atomics (used by PartitionAlloc's __aarch64_swp4_rel /
    // __aarch64_ldclr4_rel / __aarch64_ldadd4_rel etc.) fall back to
    // LDXR/STLXR loops. The LDXR/STLXR fallback is correct in theory
    // but has a tighter exclusive-monitor window and may miss
    // concurrent updates under cooperative scheduling, producing
    // spurious "old refcount didn't have bit 0 set" failures.
    //
    // QEMU `-cpu max` exposes LSE atomics; advertise them. Keep the
    // value MINIMAL: just the bits PA's outline atomics actually
    // check, plus FP/ASIMD which glibc ld-linux assumes everywhere.
    // HWCAP_FP (1<<0) = 0x001
    // HWCAP_ASIMD (1<<1) = 0x002
    // HWCAP_ATOMICS (1<<8) = 0x100
    // = 0x103
    //
    // Earlier broader values (0x1DFFB inc. SHA, AES, FPHP, FCMA,
    // LRCPC, DCPOP) made ld-linux NULL-deref at 0x1a4157d8 — likely
    // a code path that's only reached when those bits are set and
    // depends on additional auxv entries (AT_HWCAP2, AT_PLATFORM)
    // we don't supply. Keep it minimal.
    const AT_HWCAP_VALUE: u64 = 0x0000_0103;

    // extend auxv with the minimum-safe set glibc
    // reads during startup. The entries marked NEW are required
    // for `chromium --version` to print:
    //
    // AT_CLKTCK (17): clock ticks/sec. glibc reads this for
    // `sysconf(_SC_CLK_TCK)`. 100 = standard.
    // AT_UID (11) / AT_EUID (12) / AT_GID (13) / AT_EGID (14):
    // process credentials. glibc reads these
    // in `__libc_start_main`. 0 = root, but with
    // AT_SECURE=0 below this isn't a sec issue.
    // AT_SECURE (23): "is this a setuid program" flag. 0 = no.
    // Without this, glibc may take the
    // secure-binary path and disable LD_*
    // environment-var processing AND require
    // non-zero UIDs to match.
    //
    // Deliberately STILL NOT INCLUDED (per the legacy comment
    // above — these caused ld-linux NULL-deref at 0x1a4157d8 in
    // an earlier attempt):
    // AT_HWCAP2 (26) — wider feature bits
    // AT_PLATFORM (15) — string ptr to "aarch64"
    //
    // If `--version` still doesn't reach main, those are the next
    // candidates, and the deref was likely because a feature bit
    // we set in HWCAP needed PLATFORM to be valid first. Keep
    // HWCAP minimal AND add PLATFORM together if revisiting.
    let auxv: &[(u64, u64)] = if running_interp {
        &[
            (0, 0),                      // AT_NULL (last — written first, grows up)
            (3, main_phdr_va),           // AT_PHDR
            (4, main_phent),             // AT_PHENT
            (5, main_phnum),             // AT_PHNUM
            (9, main_entry_va),          // AT_ENTRY
            (7, interp_base),            // AT_BASE
            (6, 4096),                   // AT_PAGESZ
            (25, random_uva),            // AT_RANDOM
            (16, AT_HWCAP_VALUE),        // AT_HWCAP
            (17, 100),                   // AT_CLKTCK            (NEW #152)
            (23, 0),                     // AT_SECURE = 0        (NEW #152)
            (11, 0),                     // AT_UID = 0           (NEW #152)
            (12, 0),                     // AT_EUID = 0          (NEW #152)
            (13, 0),                     // AT_GID = 0           (NEW #152)
            (14, 0),                     // AT_EGID = 0          (NEW #152)
        ]
    } else {
        &[
            (0, 0), (25, random_uva), (6, 4096), (16, AT_HWCAP_VALUE),
            (17, 100), (23, 0), (11, 0), (12, 0), (13, 0), (14, 0),
        ]
    };
    for &(k, v) in auxv {
        sp -= 16;
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v);
        }
    }

    // envp NULL + pointers (user VAs), most-recent first per SysV ABI
    // (envp[] is: env0, env1, env2, env3, NULL — NULL is pushed first,
    // then env3, env2, env1, env0, so env0 sits at the lowest addr).
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env3_uva); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env2_uva); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env1_uva); }
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
    // INSIDE the cave window. Use its user VA — the cave's L2 maps
    // it, EL0 can both read and write it, and no kernel address ever
    // hits tpidr_el0. glibc's tcbhead_t goes at offset 0 of this
    // page; everything is zero-initialized.
    // 2. Legacy single-ELF path has no reserved TLS page. Keep the
    // pre-existing behaviour: zero tpidr_el0 and let the binary
    // fault if it touches TLS (busybox / hello / the small test
    // ELFs don't).
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
    let virt_entry = if user_va_base == 0 {
        orig_entry
    } else {
        // Priority order for the multi-ELF cave:
        // 1. Interp entry (ld-linux) — ld-linux does init/reloc itself.
        // 2. Init trampoline — if we're running things ourselves
        // (no interp), chain through init_array constructors.
        // 3. Plain `entry` — single-ELF-like fallback.
        let interp = LOADED_INTERP_ENTRY.load(Ordering::Relaxed) as u64;
        if interp != 0 { interp }
        else {
            let tva = LOADED_TRAMPOLINE_VA.load(Ordering::Relaxed) as u64;
            if tva != 0 { tva } else { entry }
        }
    };
    uart::puts("[loader] --- executing ---\n");

    // Save kernel SP into the kernel-BSS scratch slot shared with the
    // EL0-exit restore path in `src/kernel/arch/mod.rs`. See
    // `crate::kernel::arch::KERNEL_SP_SAVE` for the full story.
    //
    // QEMU-BUGFIX: the save+restore used to hardcode 0x40000100 and
    // 0x40001000 respectively (mismatched!), both in the Linux Image
    // header region which the MMU maps R-X on QEMU → DATA ABORT
    // DFSC=0x0e the moment we stored. Every Cave-runner ELF
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
