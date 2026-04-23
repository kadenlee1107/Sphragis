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

use crate::platform;

unsafe extern "C" {
    static __kernel_end: u8;
}

const MAGIC_HEAD: [u8; 8] = *b"BATCHROM";
const MAGIC_TAIL: [u8; 8] = *b"CHROMEND";

/// Upper bound we'll trust a declared blob size against.
/// A non-stripped ARM64 content_shell weighs ~280 MB (measured
/// 2026-04-23 from the Chromium 132 checkout). Keep the ceiling a
/// comfortable step above that so future blob growth doesn't silently
/// re-trip the probe with "size implausible → skip candidate" — the
/// user sees "no blob" and thinks the bake failed.
const MAX_BLOB_SIZE: usize = 512 * 1024 * 1024;

/// Safety cap on how far past `__kernel_end` we will search. Under QEMU
/// virt the frame allocator sits at `MEMORY_END = 2 GB`, so anything
/// past that is fiction. Blob is always at the very first byte of the
/// append area so this cap matters only defensively.
const SEARCH_CEILING: usize = 0x4000_0000 + 2 * 1024 * 1024 * 1024;

#[derive(Copy, Clone)]
pub struct BlobInfo {
    pub size: usize,
    pub crc_valid: bool,
    /// FLv2-NEW-010: Ed25519 signature verification result. The blob now
    /// carries a 64-byte signature appended after CHROMEND, and the kernel
    /// embeds the signing public key in `.rodata` (INITRD_PUBKEY).
    /// `false` either means the signature didn't verify or the blob was
    /// built without the signature trailer (legacy / dev images).
    pub sig_valid: bool,
}

/// Ed25519 public key the kernel trusts for signed initrd blobs.
///
/// **Dev placeholder** — currently all-zero so unsigned dev builds get
/// `sig_valid = false` cleanly without panicking. Replace with the real
/// production signing key (32 bytes, generated offline) before shipping.
/// Update flow: `tools/bake_chromium.sh` should be extended to sign the
/// blob with the matching private key and append `INITRD_SIG_HEAD ||
/// 64-byte signature || INITRD_SIG_TAIL` after the CHROMEND marker.
pub const INITRD_PUBKEY: [u8; 32] = [0u8; 32];

/// Optional signature trailer markers. The locator looks for
///   [CHROMEND][INITRD_SIG_HEAD][64 sig bytes][INITRD_SIG_TAIL]
/// and verifies sig over (size_le || blob_bytes). Absence ⇒ sig_valid=false.
const INITRD_SIG_HEAD: [u8; 8] = *b"BATSIGv1";
const INITRD_SIG_TAIL: [u8; 8] = *b"ENDSIGv1";


// One-shot cache. We compute once at boot and reuse thereafter so the
// CRC32 over ~150 MB runs exactly once.
static mut CACHE_INITIALIZED: bool = false;
static mut CACHED: Option<BlobInfo> = None;
static mut CACHED_BASE: usize = 0;
static mut CACHED_SIZE: usize = 0;
/// Override range set by the boot code when QEMU's `-initrd` path
/// delivered the blob instead of the `tools/bake_chromium.sh`
/// append-to-kernel path. Zero → probe falls back to `__kernel_end`.
static mut OVERRIDE_BASE: usize = 0;
static mut OVERRIDE_END:  usize = 0;

/// Called by the boot code after the DTB is parsed. If `start != 0`
/// the probe will look at `[start, end)` (typically the region QEMU
/// loaded via `-initrd`) instead of scanning past `__kernel_end`.
/// Safe to call multiple times as long as the cache hasn't been
/// populated yet — after that we ignore further calls so the blob
/// pointer stays stable for the rest of the kernel's life.
pub fn set_range(start: usize, end: usize) {
    unsafe {
        if core::ptr::read(core::ptr::addr_of!(CACHE_INITIALIZED)) { return; }
        core::ptr::write(core::ptr::addr_of_mut!(OVERRIDE_BASE), start);
        core::ptr::write(core::ptr::addr_of_mut!(OVERRIDE_END),  end);
    }
}

/// Returns `true` if a Chromium blob was baked into this image.
pub fn is_present() -> bool {
    info().is_some()
}

/// Physical (kernel identity) address where the blob actually lives, and
/// the end of its BATCHROM framing. Used by `mm::init` to reserve this
/// range in the frame bitmap — without that reservation, `alloc_frame`
/// hands out the same pages that hold the baked content_shell + libs and
/// our first big copy corrupts the archive. Returns `(0, 0)` if no
/// blob is present.
pub fn blob_phys_range() -> (usize, usize) {
    ensure_cached();
    unsafe {
        let size = core::ptr::read(core::ptr::addr_of!(CACHED_SIZE));
        let base = core::ptr::read(core::ptr::addr_of!(CACHED_BASE));
        if size == 0 || base == 0 {
            return (0, 0);
        }
        // `base` points to the blob bytes (right after BATCHROM+size
        // header). The framing head/tail we need to keep intact too.
        let head = base.saturating_sub(16); // BATCHROM(8) + size(8)
        let tail = base.saturating_add(size).saturating_add(4 + 8); // crc32 + CHROMEND
        (head, tail)
    }
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

/// Multi-file archive magic: if the blob starts with `BATARCH\0` instead
/// of a plain ELF header, it's the new tools/bake_chromium_archive.sh
/// layout — `[BATARCH\0][n_files:u32][reserved:u32][files_table: 128 B * n]
/// then concatenated file bytes`. Each table entry is
///   name: [u8; 64]      null-padded POSIX path
///   file_size: u64
///   file_offset: u64    relative to the start of the archive
///   reserved: [u8; 48]
///
/// Returns `true` if the BATCHROM blob actually contains an archive.
pub fn is_archive() -> bool {
    match locate_chromium_blob() {
        Some(b) if b.len() >= 8 => &b[0..8] == b"BATARCH\0",
        _ => false,
    }
}

/// Look up a file by name inside the archive (e.g. `"bin/content_shell"`,
/// `"lib/libc.so.6"`). Returns `None` if the blob is not an archive or
/// the name isn't present. Does bounds-checked arithmetic throughout —
/// a crafted initrd can't coerce us into reading past the blob.
pub fn archive_file(name: &str) -> Option<&'static [u8]> {
    let blob = locate_chromium_blob()?;
    if blob.len() < 16 || &blob[0..8] != b"BATARCH\0" {
        return None;
    }
    let n_files = u32_le(blob, 8)? as usize;
    if n_files > 4096 {
        // Sanity cap — our bake never produces anywhere near this.
        return None;
    }
    let table_start = 16usize;
    let entry_size = 128usize;
    let table_end = table_start.checked_add(entry_size.checked_mul(n_files)?)?;
    if table_end > blob.len() { return None; }
    let needle = name.as_bytes();
    if needle.len() >= 64 { return None; } // name cap from bake script
    for i in 0..n_files {
        let off = table_start + i * entry_size;
        // The name is null-padded to 64 bytes.
        let name_slice = &blob[off .. off + 64];
        let real_len = name_slice.iter().position(|&b| b == 0).unwrap_or(64);
        if real_len == needle.len() && &name_slice[..real_len] == needle {
            let file_size = u64_le(blob, off + 64)? as usize;
            let file_off  = u64_le(blob, off + 72)? as usize;
            let file_end = file_off.checked_add(file_size)?;
            if file_end > blob.len() { return None; }
            // SAFETY: blob is a static slice backed by the initrd memory
            // region. The returned sub-slice inherits its lifetime.
            return Some(&blob[file_off .. file_end]);
        }
    }
    None
}

/// Enumerate (name, len) pairs for every file in the archive. Useful for
/// the shell's `chromium --list-archive` diagnostic. Returns 0..N file
/// entries via a caller-supplied callback; lets the caller decide how
/// many to print without pulling alloc into the kernel initrd path.
pub fn archive_for_each<F: FnMut(&str, usize)>(mut f: F) {
    let blob = match locate_chromium_blob() { Some(b) => b, None => return };
    if blob.len() < 16 || &blob[0..8] != b"BATARCH\0" { return; }
    let n_files = match u32_le(blob, 8) { Some(v) => v as usize, None => return };
    if n_files > 4096 { return; }
    let entry_size = 128usize;
    let table_end = match 16usize.checked_add(entry_size.checked_mul(n_files).unwrap_or(usize::MAX)) {
        Some(v) if v <= blob.len() => v,
        _ => return,
    };
    let _ = table_end;
    for i in 0..n_files {
        let off = 16 + i * entry_size;
        let name_slice = &blob[off .. off + 64];
        let real_len = name_slice.iter().position(|&b| b == 0).unwrap_or(64);
        let sz = match u64_le(blob, off + 64) { Some(v) => v as usize, None => return };
        // SAFETY: name_slice bytes are bounded + valid UTF-8 in practice
        // (ASCII POSIX paths); we use from_utf8 lossily via a temporary
        // str::from_utf8 and fall back to empty on non-ASCII junk.
        let name = match core::str::from_utf8(&name_slice[..real_len]) {
            Ok(s) => s,
            Err(_) => continue,
        };
        f(name, sz);
    }
}

fn u32_le(s: &[u8], off: usize) -> Option<u32> {
    if off.checked_add(4)? > s.len() { return None; }
    Some(u32::from_le_bytes([s[off], s[off+1], s[off+2], s[off+3]]))
}
fn u64_le(s: &[u8], off: usize) -> Option<u64> {
    if off.checked_add(8)? > s.len() { return None; }
    Some(u64::from_le_bytes([
        s[off], s[off+1], s[off+2], s[off+3],
        s[off+4], s[off+5], s[off+6], s[off+7],
    ]))
}

/// Called once from `kernel_main` / `init` to log what was (or wasn't)
/// found. Idempotent — safe to call multiple times.
pub fn init() {
    ensure_cached();
    match info() {
        Some(bi) => {
            platform::serial_puts("[initrd] Chromium blob: ");
            let mb = bi.size / (1024 * 1024);
            crate::kernel::mm::print_num(mb);
            platform::serial_puts(" MB, CRC ");
            platform::serial_puts(if bi.crc_valid { "OK" } else { "MISMATCH" });
            platform::serial_puts("\n");
        }
        None => {
            platform::serial_puts("[initrd] no blob\n");
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
    // Prefer the DTB-supplied initrd range (populated by
    // `set_range`). Fall back to scanning past `__kernel_end` for
    // the legacy bake-to-kernel-image path.
    let (base, ceiling) = unsafe {
        let ob = core::ptr::read(core::ptr::addr_of!(OVERRIDE_BASE));
        let oe = core::ptr::read(core::ptr::addr_of!(OVERRIDE_END));
        if ob != 0 && oe > ob {
            (ob, oe)
        } else {
            (kernel_end_addr(), SEARCH_CEILING)
        }
    };
    if base == 0 || base >= ceiling {
        return None;
    }

    // The bake script aligns the kernel image to a 4 KB boundary before
    // appending, and `__kernel_end` is already ALIGN(4096) in the link
    // script. Check a small window of candidate offsets in case the
    // bake tool inserts extra padding. In practice offset 0 hits.
    const CANDIDATES: [usize; 3] = [0, 0x1000, 0x2000];

    for &off in &CANDIDATES {
        let head = base + off;
        // Need to read [head, head+16) — 8 B magic + 8 B size.
        // Range is valid iff head+16 <= ceiling.
        if head + 16 > ceiling {
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

        // Need to read [tail_off, tail_off+8) — 8 B CHROMEND marker.
        if tail_off + 8 > ceiling {
            continue;
        }

        if !magic_matches(tail_off, &MAGIC_TAIL) {
            continue;
        }

        let declared = read_u32_le(crc_off);
        let actual = crc32(blob_start, size);
        let crc_valid = declared == actual;

        // FLv2-NEW-010: optional Ed25519 signature trailer immediately
        // after the CHROMEND marker:
        //   [INITRD_SIG_HEAD 8B][64-byte sig][INITRD_SIG_TAIL 8B]
        // We collect the 64 bytes if both markers match, then verify
        // against the kernel-embedded INITRD_PUBKEY over the blob bytes.
        let sig_valid = verify_initrd_signature(blob_start, size, tail_off + 8);

        unsafe {
            core::ptr::write(core::ptr::addr_of_mut!(CACHED_BASE), blob_start);
            core::ptr::write(core::ptr::addr_of_mut!(CACHED_SIZE), size);
        }
        return Some(BlobInfo { size, crc_valid, sig_valid });
    }

    None
}

/// Look for the Ed25519 signature trailer right after CHROMEND and verify
/// it. Returns false on any failure (no trailer, bad markers, signature
/// mismatch, all-zero pubkey i.e. dev image).
fn verify_initrd_signature(blob_start: usize, size: usize, after_chromend: usize) -> bool {
    // Pubkey all-zero ⇒ dev image with no production trust anchor.
    let pk_all_zero = INITRD_PUBKEY.iter().all(|&b| b == 0);
    if pk_all_zero { return false; }

    // Bounds: head(8) + sig(64) + tail(8) = 80 bytes.
    if after_chromend + 80 > SEARCH_CEILING { return false; }
    if !magic_matches(after_chromend, &INITRD_SIG_HEAD) { return false; }
    if !magic_matches(after_chromend + 8 + 64, &INITRD_SIG_TAIL) { return false; }

    let mut sig = [0u8; 64];
    for i in 0..64 {
        sig[i] = unsafe {
            core::ptr::read_volatile((after_chromend + 8 + i) as *const u8)
        };
    }
    // Build the signed message: blob bytes (size). For boot speed we
    // sign the blob bytes directly. Production may switch to signing
    // the SHA-256 digest if blob copies become a bottleneck.
    let blob = unsafe { core::slice::from_raw_parts(blob_start as *const u8, size) };
    crate::crypto::sig::ed25519_verify(&INITRD_PUBKEY, &sig, blob).is_ok()
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
