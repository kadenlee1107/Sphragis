// Sphragis — User-space access helpers (ROOT-2).
//
// These wrap the raw ldrb/strb inline asm that syscall.rs previously
// scattered across every pointer-bearing syscall. By funneling reads
// and writes through a single validation + access primitive we:
//
// 1. Centralize the bounds check (today: coarse 0x1000..0x40000000
// via is_user_ptr_range; will tighten to exact walk of the cave's
// L2_low once runtime-probing lands).
//
// 2. Make it structurally impossible for a new syscall to forget the
// guard — `copy_from_user<T>` does the check, returning EFAULT on
// failure.
//
// 3. Provide `T`-aware copy helpers so callers can read/write structs
// without reinventing byte-by-byte asm loops.
//
// These are *not* a full Linux-style copy_from_user with page-fault
// recovery — our Cave has no page-fault handler yet. They're range
// validators that crash safely rather than wandering into kernel data.

#![allow(dead_code)]

use core::mem::{size_of, MaybeUninit};

const USER_MIN: usize = 0x1000;           // no NULL, no zero page
const USER_MAX: usize = 0x4000_0000;      // below kernel RAM identity map

/// iter 25: page-table fallback for is_user_range. Walks
/// the active cave's L1→L2→L3 for the START and END pages of the
/// requested range. Accepts if both are mapped with AP=EL0 R/W.
// /
/// Used as a last resort when the static-zone checks (cave window,
/// scratch zone, demand_page reservations) all reject. This covers
/// dynamically-allocated user pages — currently the brk-extended
/// region above 0x800000+0x40000 — which sys_brk installs with
/// EL0_RW perms in iter 18 but doesn't register anywhere static.
// /
/// Cost: ~6 memory loads in the worst case (L1+L2+L3 for start and
/// end). Cheap enough to call on every is_user_range fallback.
fn pages_are_user_accessible(start: usize, end: usize) -> bool {
    if start >= end { return false; }
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;
    if l1_phys == 0 { return false; }

    // Walk for both the first and last page (everything in between
    // would also need to be mapped, but checking endpoints catches
    // the common cases — single-page buffers or ranges crossing one
    // boundary). For multi-page checks we'd iterate; the caller
    // typically passes small buffers (<4KB) for syscall args.
    let first_page = start & !0xFFFusize;
    let last_page = (end - 1) & !0xFFFusize;
    if !page_is_el0_rw(l1_phys, first_page) { return false; }
    if last_page != first_page && !page_is_el0_rw(l1_phys, last_page) { return false; }

    // Walk every page in between for ranges spanning >2 pages. Cheap
    // for typical syscall buffers; only kicks in for large ones.
    let mut p = first_page + 0x1000;
    while p < last_page {
        if !page_is_el0_rw(l1_phys, p) { return false; }
        p += 0x1000;
    }
    true
}

#[inline]
fn page_is_el0_rw(l1_phys: u64, va: usize) -> bool {
    let l1_idx = ((va as u64) >> 30) & 0x1FF;
    let l1_ent_addr = l1_phys + l1_idx * 8;
    let l1e: u64 = unsafe {
        core::ptr::read_volatile(l1_ent_addr as *const u64)
    };
    if (l1e & 0b11) != 0b11 { return false; }
    let l2_phys = l1e & 0x0000_FFFF_FFFF_F000;
    let l2_idx = ((va as u64) >> 21) & 0x1FF;
    let l2_ent_addr = l2_phys + l2_idx * 8;
    let l2e: u64 = unsafe {
        core::ptr::read_volatile(l2_ent_addr as *const u64)
    };
    // L2 BLOCK (0b01) — a 2 MB block, common for the cave's main
    // user window. Check AP bits at this level.
    if (l2e & 0b11) == 0b01 {
        let ap = (l2e >> 6) & 0b11;
        return ap == 0b01; // EL0 R/W
    }
    if (l2e & 0b11) != 0b11 { return false; }
    let l3_phys = l2e & 0x0000_FFFF_FFFF_F000;
    let l3_idx = ((va as u64) >> 12) & 0x1FF;
    let l3_ent_addr = l3_phys + l3_idx * 8;
    let l3e: u64 = unsafe {
        core::ptr::read_volatile(l3_ent_addr as *const u64)
    };
    if (l3e & 0b11) != 0b11 { return false; }
    let ap = (l3e >> 6) & 0b11;
    ap == 0b01 // EL0 R/W
}

/// True if `[p, p+size)` is entirely inside the user-space range and
/// doesn't wrap.
// /
/// V3 / NEW-SYS-001: when a cave is actively mounted (mmu published a
/// non-zero window), we tighten the check to the cave's actual virtual
/// window instead of the coarse legacy range. This makes cross-cave
/// pointer abuse impossible from the syscall layer — a cave with
/// virt_base=0x10000000 cannot pass pointer 0x05000000 (legacy window)
/// because it falls outside its own L2_low mapping.
// /
/// Caves with no published window (primary/ash path) fall back to the
/// legacy [0x1000, 0x4000_0000) window; they have no per-cave isolation
/// to enforce.
#[inline]
pub fn is_user_range(p: usize, size: usize) -> bool {
    // V5-WEIRD-008 hardening: zero-sized must still require a non-null
    // pointer (callers rely on "valid user ptr" for non-zero-byte uses).
    if size == 0 { return p >= USER_MIN && p < USER_MAX; }
    let end = match p.checked_add(size) {
        Some(e) => e,
        None => return false,
    };
    let (active_start, active_end) =
        crate::caves::linux::mmu::active_user_window();
    if active_start != 0 && active_end != 0 {
        // Active cave window — the tight check wins.
        if p >= active_start && end <= active_end {
            return true;
        }
        // iter 15: glibc-scratch zone at 0x800000–0x840000.
        // signal::install_trampoline pre-maps 64 RW pages here in every
        // cave because mimalloc/glibc init writes a zero byte to
        // 0x800000 during startup. Once mimalloc claims the region as
        // its first arena, glibc reads (e.g. /etc/nsswitch.conf) land
        // here too — the buffer is a legit user pointer that just
        // happens to live OUTSIDE the cave's L2 window. Without this
        // case, sys_read returns EFAULT and glibc's NSS subsystem
        // hangs/aborts.
        const SCRATCH_LO: usize = 0x0080_0000;
        const SCRATCH_HI: usize = 0x0084_0000;
        if p >= SCRATCH_LO && end <= SCRATCH_HI {
            return true;
        }
        // CHROMIUM-PHASE-B: a syscall buffer can also legitimately
        // live inside a huge mmap reservation (V8 pointer compression
        // reserves 32 GB in the 0x28_00xx_xxxx range and lazily
        // commits via mprotect / demand-page). Those reservations are
        // OUTSIDE the cave's L2 window so the range check above
        // rejects them, but they ARE user-owned memory once a page
        // fault commits a frame. Accept them here; the demand-page
        // handler commits on first physical access.
        if crate::caves::linux::demand_page::is_in_active_reservation(p, size) {
            return true;
        }
        // iter 25: brk-extended region (above the 0x800000
        // scratch zone) and any other dynamically-allocated user
        // pages aren't in the cave window OR the scratch zone OR a
        // registered reservation. But they ARE valid user-RW pages —
        // sys_brk allocs frames + install_l3_mappings them with
        // EL0_RW perms in iter 18. Without this fallback,
        // getdents64/read/etc. on a buffer in the brk region returns
        // EFAULT (caught fontconfig scanning /usr/share/fonts).
        //
        // Walk the page table for the start + end pages and accept
        // if both are mapped EL0-RW. This is more expensive than the
        // static-zone checks above but guarantees correctness for
        // any dynamically-allocated user region.
        if pages_are_user_accessible(p, end) {
            return true;
        }
        return false;
    }
    // V5-WEIRD-008: no active cave means we're either in kernel context
    // (early boot, IRQ handler) or in the primary/ash path. In kernel
    // context there are no user pointers — safest is to reject. But
    // sys_write + ash path legitimately pass primary-side pointers in
    // [0x0, 0x14000000). We distinguish via a new ACTIVE_PRIMARY flag
    // that the primary runner sets; otherwise fail closed.
    if crate::caves::linux::mmu::active_is_primary() {
        return p >= USER_MIN && end <= USER_MAX;
    }
    false
}

/// Copy a single `T` from user space, byte-wise via volatile reads.
/// Returns -EFAULT on range failure.
pub fn copy_from_user<T: Copy>(user_ptr: usize) -> Result<T, i64> {
    let n = size_of::<T>();
    if !is_user_range(user_ptr, n) { return Err(-14); } // EFAULT

    let mut out: MaybeUninit<T> = MaybeUninit::uninit();
    let dst = out.as_mut_ptr() as *mut u8;
    for i in 0..n {
        unsafe {
            let b: u8 = core::ptr::read_volatile((user_ptr + i) as *const u8);
            *dst.add(i) = b;
        }
    }
    Ok(unsafe { out.assume_init() })
}

/// Copy a single `T` to user space. Returns Err(-14) on range failure.
pub fn copy_to_user<T: Copy>(user_ptr: usize, val: &T) -> Result<(), i64> {
    let n = size_of::<T>();
    if !is_user_range(user_ptr, n) { return Err(-14); }
    let src = val as *const T as *const u8;
    for i in 0..n {
        unsafe {
            let b = *src.add(i);
            core::ptr::write_volatile((user_ptr + i) as *mut u8, b);
        }
    }
    Ok(())
}

/// Copy a byte slice from user space into a kernel buffer. Returns
/// Err(-14) on range failure, otherwise the number of bytes copied
/// (== min(kern.len(), len)).
pub fn copy_from_user_slice(user_ptr: usize, kern: &mut [u8]) -> Result<usize, i64> {
    let n = kern.len();
    if !is_user_range(user_ptr, n) { return Err(-14); }
    for i in 0..n {
        unsafe {
            kern[i] = core::ptr::read_volatile((user_ptr + i) as *const u8);
        }
    }
    Ok(n)
}

/// Copy a byte slice from a kernel buffer into user space. Returns
/// Err(-14) on range failure, otherwise the number of bytes written.
pub fn copy_to_user_slice(user_ptr: usize, kern: &[u8]) -> Result<usize, i64> {
    let n = kern.len();
    if !is_user_range(user_ptr, n) { return Err(-14); }
    for i in 0..n {
        unsafe {
            core::ptr::write_volatile((user_ptr + i) as *mut u8, kern[i]);
        }
    }
    Ok(n)
}

/// Read a NUL-terminated C string from user space into a kernel buffer,
/// capped at `max_len - 1` bytes (always NUL-terminates the output).
/// Returns the length not counting the terminator, or Err(-14) / Err(-2)
/// on range or no-NUL-found failure.
pub fn copy_cstr_from_user(user_ptr: usize, kern: &mut [u8]) -> Result<usize, i64> {
    if kern.is_empty() { return Err(-22); } // EINVAL — no room even for terminator
    let max = kern.len() - 1;
    if !is_user_range(user_ptr, max + 1) { return Err(-14); }
    for i in 0..max {
        let b = unsafe { core::ptr::read_volatile((user_ptr + i) as *const u8) };
        kern[i] = b;
        if b == 0 {
            return Ok(i);
        }
    }
    kern[max] = 0;
    // Didn't find NUL within the cap — treat as name-too-long.
    Err(-36) // ENAMETOOLONG
}

/// A null-able user pointer: Ok(None) if the pointer is 0,
/// Ok(Some(val)) if it's a valid user pointer to a T, Err on bad range.
pub fn copy_from_user_opt<T: Copy>(user_ptr: usize) -> Result<Option<T>, i64> {
    if user_ptr == 0 { return Ok(None); }
    copy_from_user::<T>(user_ptr).map(Some)
}
