// Sphragis — Volatile zeroization helper.
//
// Standard defensive cryptography hygiene: when a struct holding key
// material or plaintext secrets goes out of scope, overwrite the bytes
// with zeroes so they don't survive in RAM (cold-boot attack, DMA
// snapshot, HVF suspend image, crash-dump inspection).
//
// Why `core::ptr::write_volatile` and not `.fill(0)`:
// LLVM is permitted to remove dead stores to soon-dropped stack slots.
// A plain `buf.fill(0)` inside `Drop` is typically optimized away
// because the buffer is about to be freed. `write_volatile` is treated
// as a side effect the compiler must preserve.
//
// Paired with a `compiler_fence(SeqCst)` to prevent reorder-elision
// across the wipe boundary.

#![allow(dead_code)]

use core::sync::atomic::{compiler_fence, Ordering};

/// Zeroize a byte slice. Guaranteed-observable writes.
#[inline]
pub fn zeroize(slice: &mut [u8]) {
    let p = slice.as_mut_ptr();
    let n = slice.len();
    for i in 0..n {
        // SAFETY: in-bounds write to a &mut [u8] we own.
        unsafe { core::ptr::write_volatile(p.add(i), 0u8); }
    }
    compiler_fence(Ordering::SeqCst);
}

/// Zeroize a `&mut [u32]` — used by AES round keys.
#[inline]
pub fn zeroize_u32_slice(slice: &mut [u32]) {
    let p = slice.as_mut_ptr();
    let n = slice.len();
    for i in 0..n {
        unsafe { core::ptr::write_volatile(p.add(i), 0u32); }
    }
    compiler_fence(Ordering::SeqCst);
}

/// Zeroize a `&mut [u64]` — used by SHA state + GHASH tables.
#[inline]
pub fn zeroize_u64_slice(slice: &mut [u64]) {
    let p = slice.as_mut_ptr();
    let n = slice.len();
    for i in 0..n {
        unsafe { core::ptr::write_volatile(p.add(i), 0u64); }
    }
    compiler_fence(Ordering::SeqCst);
}

/// Zeroize a single `T` via byte-level volatile writes. Works for any
/// `Copy` struct; callers use it by passing `&mut instance`.
#[inline]
pub fn zeroize_bytes_of<T>(val: &mut T) {
    let p = val as *mut T as *mut u8;
    let n = core::mem::size_of::<T>();
    for i in 0..n {
        unsafe { core::ptr::write_volatile(p.add(i), 0u8); }
    }
    compiler_fence(Ordering::SeqCst);
}
