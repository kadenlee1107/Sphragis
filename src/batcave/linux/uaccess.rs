// Bat_OS — User-space access helpers (ROOT-2).
//
// These wrap the raw ldrb/strb inline asm that syscall.rs previously
// scattered across every pointer-bearing syscall. By funneling reads
// and writes through a single validation + access primitive we:
//
//   1. Centralize the bounds check (today: coarse 0x1000..0x40000000
//      via is_user_ptr_range; will tighten to exact walk of the cave's
//      L2_low once runtime-probing lands).
//
//   2. Make it structurally impossible for a new syscall to forget the
//      guard — `copy_from_user<T>` does the check, returning EFAULT on
//      failure.
//
//   3. Provide `T`-aware copy helpers so callers can read/write structs
//      without reinventing byte-by-byte asm loops.
//
// These are *not* a full Linux-style copy_from_user with page-fault
// recovery — our BatCave has no page-fault handler yet. They're range
// validators that crash safely rather than wandering into kernel data.

#![allow(dead_code)]

use core::mem::{size_of, MaybeUninit};

const USER_MIN: usize = 0x1000;           // no NULL, no zero page
const USER_MAX: usize = 0x4000_0000;      // below kernel RAM identity map

/// True if `[p, p+size)` is entirely inside the user-space range and
/// doesn't wrap. Coarse — see module doc.
#[inline]
pub fn is_user_range(p: usize, size: usize) -> bool {
    if size == 0 { return true; } // zero-sized access is trivially valid
    let end = match p.checked_add(size) {
        Some(e) => e,
        None => return false,
    };
    p >= USER_MIN && end <= USER_MAX
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
