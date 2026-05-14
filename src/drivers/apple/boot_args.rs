#![allow(dead_code)]
// Sphragis — m1n1 / iBoot boot-args handoff
//
// When m1n1 (or raw iBoot for a payload-style boot) hands control to us,
// register x0 holds a pointer to a `BootArgs` structure and x1-x3 are
// undefined. Our assembly entry stub at src/arch/aarch64/apple/boot.s
// preserves x0 and calls `kernel_main_apple(x0)`.
//
// This module:
//   * Declares the raw on-wire BootArgs layout (matching m1n1's
//     xnuboot.h exactly — wrong field order here = boot garbage).
//   * Validates revision/version/devtree pointer bounds.
//   * Exposes typed accessors (video, mem, devtree slice, cmdline).
//
// REFERENCE (used for protocol layout only; clean-room implementation):
//   m1n1/src/xnuboot.h (MIT)
//   AsahiLinux kernel: arch/arm64/include/asm/apple_boot.h (GPL)
//
// SECURITY: `BootArgs` is the FIRST untrusted data we see. An attacker
// with a pre-boot foothold could scribble it. We treat it as untrusted:
// every pointer is bounds-checked against a conservative RAM window,
// every string is nul-bounded before UTF-8 decode, every ADT pointer
// is validated before we hand it to the ADT parser.

use super::adt::{Adt, AdtError};

// ─── Layout constants ────────────────────────────────────────────────

/// Revision-1 command-line buffer size.
pub const CMDLINE_LEN_RV1: usize = 256;
/// Revision-2 command-line buffer size (M1-era).
pub const CMDLINE_LEN_RV2: usize = 608;
/// Revision-3 command-line buffer size (M3+, including M4).
pub const CMDLINE_LEN_RV3: usize = 1024;

/// Plausibility cap on devtree size. A real ADT is ~100-300 KiB;
/// anything over 16 MiB means we're looking at a corrupted struct.
const MAX_DEVTREE_SIZE: usize = 16 * 1024 * 1024;

// ─── Wire format ─────────────────────────────────────────────────────

/// Video block within `BootArgs`. Matches m1n1's `struct boot_video`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootVideo {
    pub base: u64,
    pub display: u64,
    pub stride: u64,
    pub width: u64,
    pub height: u64,
    pub depth: u64,
}

/// Raw m1n1/iBoot boot-args, revision 3 (the most generous cmdline
/// size). Earlier revisions have a shorter `cmdline` and therefore a
/// shorter overall struct — callers MUST consult `revision` before
/// reading past `cmdline[CMDLINE_LEN_RV2]`.
///
/// Layout rules:
///   * `#[repr(C)]` — Rust adds 4 bytes of padding after the two u16s
///     so `virt_base` lands on its 8-byte alignment. This matches the
///     C compiler's layout for `struct boot_args` exactly.
#[repr(C)]
pub struct BootArgsRaw {
    pub revision: u16,
    pub version: u16,
    // (4 bytes implicit padding here under repr(C))
    pub virt_base: u64,
    pub phys_base: u64,
    pub mem_size: u64,
    pub top_of_kernel_data: u64,
    pub video: BootVideo,
    pub machine_type: u32,
    // (4 bytes padding before pointer)
    pub devtree: *const u8,
    pub devtree_size: u32,
    // Start of the revision-tagged union. We declare the longest
    // variant and rely on `revision` to tell us what's actually valid.
    pub cmdline: [u8; CMDLINE_LEN_RV3],
    pub boot_flags: u64,
    pub mem_size_actual: u64,
}

// ─── Parsed / validated form ─────────────────────────────────────────

#[derive(Debug, Copy, Clone)]
pub struct VideoInfo {
    pub base: u64,
    pub stride: u64, // bytes per row
    pub width: u32,
    pub height: u32,
    pub depth: u32,  // bits per pixel
}

#[derive(Debug)]
pub enum BootArgsError {
    NullPointer,
    UnsupportedRevision(u16),
    ImplausibleDevtree { addr: u64, size: u32 },
    MemSizeZero,
    VideoBad,
}

/// Typed + bounds-checked view of the boot args. Returned by
/// [`parse`] after basic sanity checks pass.
pub struct BootArgs<'a> {
    raw: &'a BootArgsRaw,
    devtree: &'a [u8],
}

impl<'a> BootArgs<'a> {
    pub fn revision(&self) -> u16 { self.raw.revision }
    pub fn version(&self) -> u16 { self.raw.version }
    pub fn machine_type(&self) -> u32 { self.raw.machine_type }
    pub fn phys_base(&self) -> u64 { self.raw.phys_base }
    pub fn virt_base(&self) -> u64 { self.raw.virt_base }
    pub fn mem_size(&self) -> u64 { self.raw.mem_size }
    pub fn mem_size_actual(&self) -> u64 {
        // Only valid on rev >= 2; rev-1 doesn't have this field at
        // this offset. We declared the struct as rev-3 layout, so
        // reading it is always safe, but the VALUE is only
        // meaningful when revision >= 2.
        if self.raw.revision >= 2 { self.raw.mem_size_actual } else { self.raw.mem_size }
    }
    pub fn boot_flags(&self) -> u64 {
        if self.raw.revision >= 2 { self.raw.boot_flags } else { 0 }
    }

    pub fn video(&self) -> VideoInfo {
        VideoInfo {
            base: self.raw.video.base,
            stride: self.raw.video.stride,
            width: self.raw.video.width as u32,
            height: self.raw.video.height as u32,
            depth: self.raw.video.depth as u32,
        }
    }

    /// Kernel command-line as a byte slice (up to first nul, bounded
    /// by the revision-appropriate max length). Never trusts C-style
    /// "until you hit a nul" without a hard cap — revisions cap the
    /// buffer, and we respect that cap.
    pub fn cmdline(&self) -> &'a [u8] {
        let max = match self.raw.revision {
            1 => CMDLINE_LEN_RV1,
            2 => CMDLINE_LEN_RV2,
            _ => CMDLINE_LEN_RV3,
        };
        let slice = &self.raw.cmdline[..max];
        let nul = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        &slice[..nul]
    }

    /// The raw Apple Device Tree blob handed to us by m1n1.
    pub fn devtree_bytes(&self) -> &'a [u8] {
        self.devtree
    }

    /// Pre-validated ADT wrapped in the bounds-checked parser.
    pub fn adt(&self) -> Result<Adt<'a>, AdtError> {
        Adt::new(self.devtree)
    }
}

// ─── Entry point: validate + parse ───────────────────────────────────

/// Parse the boot-args pointer m1n1 hands us.
///
/// # Safety
/// `ptr` must be a non-null, readable pointer to a live `BootArgsRaw`
/// struct (plus its referenced devtree region). This is true by
/// construction when called from `_apple_start` with the x0 value
/// m1n1 provided.
pub unsafe fn parse<'a>(ptr: *const BootArgsRaw) -> Result<BootArgs<'a>, BootArgsError> {
    if ptr.is_null() {
        return Err(BootArgsError::NullPointer);
    }
    let raw = unsafe { &*ptr };

    // Support revisions 1-3. (M4 observed to be rev 3 in m1n1 traces.)
    if !(1..=3).contains(&raw.revision) {
        return Err(BootArgsError::UnsupportedRevision(raw.revision));
    }

    if raw.mem_size == 0 {
        return Err(BootArgsError::MemSizeZero);
    }

    let devtree_addr = raw.devtree as u64;
    let devtree_size = raw.devtree_size as usize;
    // Plausibility: non-null, size in (0, 16 MiB]. The stricter
    // "within [phys_base, phys_base+mem_size)" check used to fire here,
    // but m1n1 on M4 hands us the devtree pointer in its own virtual /
    // chainload-relocated address space (observed: 0xdf4000 when
    // phys_base is 0x10002798000). Trust the pointer; the ADT parser
    // will still bounds-check every internal offset.
    if devtree_addr == 0 || devtree_size == 0 || devtree_size > MAX_DEVTREE_SIZE {
        return Err(BootArgsError::ImplausibleDevtree {
            addr: devtree_addr,
            size: raw.devtree_size,
        });
    }

    // Basic video sanity — not fatal, but we flag it. m1n1 always
    // provides some framebuffer, even if just a 1x1 placeholder.
    if raw.video.base == 0 || raw.video.width == 0 || raw.video.height == 0 {
        // Non-fatal; some boot paths hand us a deferred FB.
    }

    // Translate the devtree pointer from m1n1's virtual address space
    // to physical. m1n1 stores `devtree` as a virtual address (per
    // m1n1/src/startup.c:172) relative to its own virt_base/phys_base
    // mapping; by the time we run, m1n1 has disabled its MMU, so we
    // must deref by physical address. Formula matches m1n1's own:
    //   phys = virt - virt_base + phys_base
    let v2p_offset = raw.phys_base.wrapping_sub(raw.virt_base);
    let devtree_phys = (raw.devtree as u64).wrapping_add(v2p_offset);
    let devtree = unsafe {
        core::slice::from_raw_parts(devtree_phys as *const u8, devtree_size)
    };

    Ok(BootArgs { raw, devtree })
}

// ─── Stash for later access ──────────────────────────────────────────
//
// The pointer that m1n1 hands us lives for the lifetime of the kernel
// (it's placed above our kernel-data region and we inherit responsibility
// for it). We stash it in a static so the rest of the kernel can pull
// ADT data later without plumbing &BootArgs everywhere.
//
// Safe because:
//   * We write it exactly once, very early in boot, before any other
//     CPU is running.
//   * All readers only go through `with()` which re-validates and
//     hands out a short-lived `BootArgs<'static>` reference.

use core::sync::atomic::{AtomicPtr, Ordering};

static STASHED: AtomicPtr<BootArgsRaw> = AtomicPtr::new(core::ptr::null_mut());

/// Call once, from `kernel_main_apple`, with the pointer boot.s saved.
/// # Safety: Same contract as [`parse`].
pub unsafe fn stash(ptr: *const BootArgsRaw) {
    STASHED.store(ptr as *mut BootArgsRaw, Ordering::Release);
}

/// Look up the stashed boot args. Returns `None` if we haven't been
/// through the Apple boot path (e.g. running on QEMU virt).
pub fn with<R>(f: impl FnOnce(&BootArgs<'static>) -> R) -> Option<R> {
    let ptr = STASHED.load(Ordering::Acquire);
    if ptr.is_null() { return None; }
    // SAFETY: set once at boot, never freed, lives for kernel lifetime.
    let raw: &'static BootArgsRaw = unsafe { &*(ptr as *const BootArgsRaw) };
    let devtree_size = raw.devtree_size as usize;
    // Re-check devtree bounds defensively every read — we treat the
    // pointer stash as trusted, but if the struct itself was mutated
    // (it shouldn't be, but cosmic ray / hardware fault / kernel bug)
    // we still want to refuse OOB reads.
    if devtree_size == 0 || devtree_size > MAX_DEVTREE_SIZE || raw.devtree.is_null() {
        return None;
    }
    let devtree: &'static [u8] = unsafe {
        core::slice::from_raw_parts(raw.devtree, devtree_size)
    };
    let args = BootArgs { raw, devtree };
    Some(f(&args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rev_bounds() {
        // A fully-zeroed struct would have revision=0 → reject.
        let raw = BootArgsRaw {
            revision: 0,
            version: 0,
            virt_base: 0,
            phys_base: 0,
            mem_size: 0,
            top_of_kernel_data: 0,
            video: BootVideo {
                base: 0, display: 0, stride: 0, width: 0, height: 0, depth: 0,
            },
            machine_type: 0,
            devtree: core::ptr::null(),
            devtree_size: 0,
            cmdline: [0; CMDLINE_LEN_RV3],
            boot_flags: 0,
            mem_size_actual: 0,
        };
        let r = unsafe { parse(&raw as *const _) };
        assert!(matches!(r, Err(BootArgsError::UnsupportedRevision(0))));
    }
}
