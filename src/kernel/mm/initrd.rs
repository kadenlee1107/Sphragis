// Bat_OS — Chromium blob locator (initrd-style appended binary).
//
// When the kernel image is built with `tools/bake_chromium.sh`, the
// content_shell ELF is concatenated onto the kernel image, wrapped
// in a simple framing format:
//
//   [kernel ELF]
//   [magic "BATCHROM" (8 bytes)]
//   [u64 size (LE)]
//   [content_shell bytes (size bytes)]
//   [u32 crc32 (LE)]
//   [magic "CHROMEND" (8 bytes)]
//
// On a plain cargo build (no bake), the bytes past `__kernel_end` are
// whatever the loader left there (usually zero or frame-allocator
// bookkeeping). `locate_chromium_blob` must therefore handle the
// "no magic" case gracefully and return `None`.

#![allow(dead_code)]

use crate::drivers::uart;

unsafe extern "C" {
    static __kernel_end: u8;
}

const MAGIC_HEAD: [u8; 8] = *b"BATCHROM";
const MAGIC_TAIL: [u8; 8] = *b"CHROMEND";

/// Upper bound we'll trust a declared blob size against.
/// 256 MB is comfortably larger than a stripped content_shell.
const MAX_BLOB_SIZE: usize = 256 * 1024 * 1024;

/// Safety cap on how far past `__kernel_end` we will search. Under QEMU
/// virt the frame allocator sits at `MEMORY_END = 2 GB`, so anything
/// past that is fiction. Blob is always at the very first byte of the
/// append area so this cap matters only defensively.
const SEARCH_CEILING: usize = 0x4000_0000 + 2 * 1024 * 1024 * 1024;

#[derive(Copy, Clone)]
pub struct BlobInfo {
    pub size: usize,
    pub crc_valid: bool,
}

// One-shot cache. We compute once at boot and reuse thereafter so the
// CRC32 over ~150 MB runs exactly once.
static mut CACHE_INITIALIZED: bool = false;
static mut CACHED: Option<BlobInfo> = None;
static mut CACHED_BASE: usize = 0;
static mut CACHED_SIZE: usize = 0;

/// Returns `true` if a Chromium blob was baked into this image.
pub fn is_present() -> bool {
    info().is_some()
}

/// Returns blob metadata if present.
pub fn info() -> Option<BlobInfo> {
    ensure_cached();
    // Safety: we only ever read CACHED through this raw-pointer path;
    // ensure_cached serializes writes to it via the one-shot init flag.
    unsafe { core::ptr::read(core::ptr::addr_of!(CACHED)) }
}

/// Returns the Chromium content_shell bytes as a static slice, or
/// `None` if no blob is present. Callers may treat this as read-only
/// input to the ELF loader.
pub fn locate_chromium_blob() -> Option<&'static [u8]> {
    ensure_cached();
    unsafe {
        let present = core::ptr::read(core::ptr::addr_of!(CACHED)).is_some();
        let size = core::ptr::read(core::ptr::addr_of!(CACHED_SIZE));
        let base = core::ptr::read(core::ptr::addr_of!(CACHED_BASE));
        if present && size > 0 {
            Some(core::slice::from_raw_parts(base as *const u8, size))
        } else {
            None
        }
    }
}

/// Called once from `kernel_main` / `init` to log what was (or wasn't)
/// found. Idempotent — safe to call multiple times.
pub fn init() {
    ensure_cached();
    match info() {
        Some(bi) => {
            uart::puts("[initrd] Chromium blob: ");
            let mb = bi.size / (1024 * 1024);
            crate::kernel::mm::print_num(mb);
            uart::puts(" MB, CRC ");
            uart::puts(if bi.crc_valid { "OK" } else { "MISMATCH" });
            uart::puts("\n");
        }
        None => {
            uart::puts("[initrd] no blob\n");
        }
    }
}

// --- internals ---------------------------------------------------------

fn ensure_cached() {
    unsafe {
        if core::ptr::read(core::ptr::addr_of!(CACHE_INITIALIZED)) {
            return;
        }
        core::ptr::write(core::ptr::addr_of_mut!(CACHE_INITIALIZED), true);
        let v = probe();
        core::ptr::write(core::ptr::addr_of_mut!(CACHED), v);
    }
}

fn kernel_end_addr() -> usize {
    core::ptr::addr_of!(__kernel_end) as usize
}

/// Probe memory past `__kernel_end` for the `BATCHROM` framing.
/// Returns the discovered BlobInfo and (via the module-level cache)
/// records the base pointer + size for later slice construction.
fn probe() -> Option<BlobInfo> {
    let base = kernel_end_addr();
    if base == 0 || base >= SEARCH_CEILING {
        return None;
    }

    // The bake script aligns the kernel image to a 4 KB boundary before
    // appending, and `__kernel_end` is already ALIGN(4096) in the link
    // script. Check a small window of candidate offsets in case the
    // bake tool inserts extra padding. In practice offset 0 hits.
    const CANDIDATES: [usize; 3] = [0, 0x1000, 0x2000];

    for &off in &CANDIDATES {
        let head = base + off;
        if head + 16 >= SEARCH_CEILING {
            break;
        }
        if !magic_matches(head, &MAGIC_HEAD) {
            continue;
        }

        let size = read_u64_le(head + 8) as usize;
        if size == 0 || size > MAX_BLOB_SIZE {
            continue;
        }

        let blob_start = head + 16;
        let crc_off = blob_start + size;
        let tail_off = crc_off + 4;

        if tail_off + 8 >= SEARCH_CEILING {
            continue;
        }

        if !magic_matches(tail_off, &MAGIC_TAIL) {
            continue;
        }

        let declared = read_u32_le(crc_off);
        let actual = crc32(blob_start, size);
        let crc_valid = declared == actual;

        unsafe {
            core::ptr::write(core::ptr::addr_of_mut!(CACHED_BASE), blob_start);
            core::ptr::write(core::ptr::addr_of_mut!(CACHED_SIZE), size);
        }
        return Some(BlobInfo { size, crc_valid });
    }

    None
}

fn magic_matches(addr: usize, expected: &[u8; 8]) -> bool {
    for i in 0..8 {
        let b = unsafe { core::ptr::read_volatile((addr + i) as *const u8) };
        if b != expected[i] {
            return false;
        }
    }
    true
}

fn read_u64_le(addr: usize) -> u64 {
    let mut v: u64 = 0;
    for i in 0..8 {
        let b = unsafe { core::ptr::read_volatile((addr + i) as *const u8) } as u64;
        v |= b << (8 * i);
    }
    v
}

fn read_u32_le(addr: usize) -> u32 {
    let mut v: u32 = 0;
    for i in 0..4 {
        let b = unsafe { core::ptr::read_volatile((addr + i) as *const u8) } as u32;
        v |= b << (8 * i);
    }
    v
}

/// Standard IEEE CRC32 (poly 0xEDB88320) over a raw byte span. Computed
/// in-place without a lookup table — ~150 MB runs in seconds on the
/// target CPU, paid once at boot. A table could cut this further if
/// it ever matters.
fn crc32(addr: usize, len: usize) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for i in 0..len {
        let byte = unsafe { core::ptr::read_volatile((addr + i) as *const u8) };
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}
