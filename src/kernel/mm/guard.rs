//! Heap allocation guards — gov-grade hardened-malloc primitives.
//!
//! Wraps the linked-list allocator with a per-allocation canary
//! frame so heap overflows, underflows, and double-frees are
//! detected at `dealloc` time instead of silently corrupting
//! adjacent allocations.
//!
//! ## Layout per allocation
//!
//! ```text
//!   +----------------+----------------------+----------------+
//!   | front canary   |  user payload (sz)   |  back canary   |
//!   |   16 bytes     |                      |   16 bytes     |
//!   +----------------+----------------------+----------------+
//!   ^ inner_ptr      ^ user_ptr (returned)  ^ user_ptr+sz
//! ```
//!
//! The user receives a pointer to the middle. The inner heap holds
//! `2*GUARD_SIZE + sz` bytes at `inner_ptr`. On free, we recompute
//! the canary and verify both frames match before scrubbing and
//! returning the memory to the heap.
//!
//! ## Threat model
//!
//! - **Heap overflow** — write past `user_ptr + sz` corrupts the
//!   back canary. Detected on free, panic.
//! - **Heap underflow** — write before `user_ptr` corrupts the
//!   front canary. Detected on free, panic.
//! - **Double-free** — after a valid free, the front canary is
//!   overwritten with a fixed POISON pattern. A second free sees
//!   POISON instead of the expected canary, panic.
//! - **Wild-pointer write across the canary region** — the canary
//!   covers 32 bytes adjacent to the payload, so most stack-typed
//!   wild writes are caught.
//!
//! ## Canary derivation
//!
//! `canary(addr, sz) = sha256(KEY || addr_be || sz_be)[..16]`
//!
//! `KEY` is a 32-byte boot-random seed published once via
//! `guard::init(seed)` from the early-boot RNG. Reading one
//! canary therefore reveals nothing about another canary at a
//! different address. Attacker who corrupts a back canary needs
//! to forge the HMAC-style tag → infeasible without KEY.
//!
//! ## Limitations / future arcs
//!
//! - Alignment > GUARD_SIZE (16) is not supported by this layer —
//!   the global allocator falls through to the un-guarded path
//!   for those allocations. Typical kernel types are align ≤ 16,
//!   so this is rare in practice.
//! - No quarantine: a freed block can be re-allocated immediately.
//!   UAF on a still-alive object is NOT caught by this layer.
//!   A delayed-reuse ring is the natural next arc.
//! - No size-class segregation: arbitrary-size allocations all
//!   go through the same linked-list freelist. Segregation +
//!   XOR'd freelist pointers is the next arc after quarantine.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::crypto::sha256;

/// Size of each canary frame. 16 bytes gives 128 bits of forge
/// resistance per canary — well above the 64-bit floor that
/// hardened allocators target (e.g., GrapheneOS hardened_malloc).
pub const GUARD_SIZE: usize = 16;

/// Total per-allocation overhead (front + back).
pub const FRAME_OVERHEAD: usize = 2 * GUARD_SIZE;

/// Poison pattern written into the FRONT canary when a block is
/// freed. Picked to be visually distinct in memory dumps
/// (`POISON POISON`) and to fail canary verification with high
/// probability (~ 1 in 2^128 collision).
const POISON: [u8; GUARD_SIZE] = *b"POISONPOISON1107";

/// Boot-random secret for canary derivation. Set once via
/// `guard::init`. Reads as zero before init — that's a degraded
/// mode but the canary frame still detects buffer overflows
/// (front and back canaries still match each other because they
/// share the same key, just one that's known to an offline
/// attacker).
static KEY: [AtomicU64; 4] = [
    AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0),
];

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Number of guard-protected allocations made since boot. Updated
/// per `wrap_alloc` call. Cheap counter; not a memory accounting
/// primitive (use the heap's own metrics for that).
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);

/// Number of guard-verified deallocations.
static FREE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Number of canary mismatches detected. Each one panics so this
/// counter only ever reaches 1, but we keep it for the selftest
/// to confirm the detection path fires.
static CORRUPTION_COUNT: AtomicU64 = AtomicU64::new(0);

/// Install the boot-random key. Call once from `mm::init` after
/// the RNG is seeded but before the first `Box`/`Vec` use.
/// Idempotent: a second call is a no-op (the key is sticky for
/// the lifetime of the boot).
pub fn init(seed: &[u8; 32]) {
    if INITIALIZED.swap(true, Ordering::AcqRel) {
        return;
    }
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&seed[i * 8..(i + 1) * 8]);
        KEY[i].store(u64::from_be_bytes(buf), Ordering::Release);
    }
}

/// Returns true if the guard subsystem has been initialised. Used
/// by `heap.rs` to fall back to the un-guarded path during early
/// boot before `init` runs.
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Compute the canary value for a block at `addr` of payload size
/// `sz`. `addr` is the INNER pointer (the start of the front
/// canary frame). Both front and back canaries get the same
/// value — a checker that finds front == back == expected on
/// dealloc confirms the block boundaries are intact.
fn canary(addr: usize, sz: usize) -> [u8; GUARD_SIZE] {
    let mut buf = [0u8; 32 + 8 + 8];
    for i in 0..4 {
        let k = KEY[i].load(Ordering::Acquire).to_be_bytes();
        buf[i * 8..(i + 1) * 8].copy_from_slice(&k);
    }
    buf[32..40].copy_from_slice(&(addr as u64).to_be_bytes());
    buf[40..48].copy_from_slice(&(sz as u64).to_be_bytes());
    let h = sha256::hash(&buf);
    let mut out = [0u8; GUARD_SIZE];
    out.copy_from_slice(&h[..GUARD_SIZE]);
    out
}

/// Wrap a just-allocated inner block with canary frames and
/// return the user pointer.
///
/// SAFETY: caller must guarantee `inner_ptr` is a valid block of
/// at least `FRAME_OVERHEAD + payload_size` bytes that the
/// underlying allocator just handed us.
pub unsafe fn wrap_alloc(inner_ptr: *mut u8, payload_size: usize) -> *mut u8 {
    let cnry = canary(inner_ptr as usize, payload_size);
    unsafe {
        // Front canary at inner_ptr[..GUARD_SIZE]
        core::ptr::copy_nonoverlapping(cnry.as_ptr(), inner_ptr, GUARD_SIZE);
        // Back canary at inner_ptr[GUARD_SIZE + payload_size ..]
        let back = inner_ptr.add(GUARD_SIZE + payload_size);
        core::ptr::copy_nonoverlapping(cnry.as_ptr(), back, GUARD_SIZE);
    }
    ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    unsafe { inner_ptr.add(GUARD_SIZE) }
}

/// Reason a canary verification failed. Stored only for the
/// selftest; the production path panics on any non-Ok outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyFault {
    /// Front canary matched POISON — caller is freeing a block
    /// that's already on the freelist.
    DoubleFree,
    /// Front canary doesn't match expected — heap underflow, or
    /// freeing a pointer the guard layer never wrapped.
    UnderflowOrAlien,
    /// Back canary doesn't match expected — heap overflow.
    Overflow,
}

/// Unwrap a guarded allocation back to its inner pointer, verifying
/// both canaries and poisoning the front canary for double-free
/// detection. Returns the inner pointer suitable for the underlying
/// allocator's `deallocate`, or `Err(VerifyFault)` on any mismatch.
///
/// SAFETY: caller passes the user_ptr originally returned from
/// `wrap_alloc` and the SAME payload size that was passed to
/// `wrap_alloc`.
pub unsafe fn verify_and_unwrap(
    user_ptr: *mut u8,
    payload_size: usize,
) -> Result<*mut u8, VerifyFault> {
    let inner_ptr = unsafe { user_ptr.sub(GUARD_SIZE) };
    let expected = canary(inner_ptr as usize, payload_size);
    let mut front = [0u8; GUARD_SIZE];
    let mut back = [0u8; GUARD_SIZE];
    unsafe {
        core::ptr::copy_nonoverlapping(inner_ptr, front.as_mut_ptr(), GUARD_SIZE);
        let back_ptr = inner_ptr.add(GUARD_SIZE + payload_size);
        core::ptr::copy_nonoverlapping(back_ptr, back.as_mut_ptr(), GUARD_SIZE);
    }
    if front == POISON {
        CORRUPTION_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(VerifyFault::DoubleFree);
    }
    if front != expected {
        CORRUPTION_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(VerifyFault::UnderflowOrAlien);
    }
    if back != expected {
        CORRUPTION_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(VerifyFault::Overflow);
    }
    // Poison the front canary so a subsequent double-free is
    // detected even after the heap recycles the block (the
    // poison value won't match the new block-address's canary
    // either, so we'd hit UnderflowOrAlien on a recycled-then-
    // freed-again block — still detected).
    unsafe {
        core::ptr::copy_nonoverlapping(POISON.as_ptr(), inner_ptr, GUARD_SIZE);
    }
    FREE_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(inner_ptr)
}

/// Non-destructive canary inspection — returns the same `VerifyFault`
/// codes as `verify_and_unwrap` but does NOT poison the front
/// canary or change any counters. Used by the heap selftest to
/// exercise the detection path without entering the panic path.
///
/// SAFETY: same precondition as `verify_and_unwrap`.
pub unsafe fn inspect_user_ptr(
    user_ptr: *mut u8,
    payload_size: usize,
) -> Result<(), VerifyFault> {
    let inner_ptr = unsafe { user_ptr.sub(GUARD_SIZE) };
    let expected = canary(inner_ptr as usize, payload_size);
    let mut front = [0u8; GUARD_SIZE];
    let mut back = [0u8; GUARD_SIZE];
    unsafe {
        core::ptr::copy_nonoverlapping(inner_ptr, front.as_mut_ptr(), GUARD_SIZE);
        let back_ptr = inner_ptr.add(GUARD_SIZE + payload_size);
        core::ptr::copy_nonoverlapping(back_ptr, back.as_mut_ptr(), GUARD_SIZE);
    }
    if front == POISON { return Err(VerifyFault::DoubleFree); }
    if front != expected { return Err(VerifyFault::UnderflowOrAlien); }
    if back != expected { return Err(VerifyFault::Overflow); }
    Ok(())
}

/// Recompute and rewrite both canaries for a block. Used by the
/// heap selftest to restore a deliberately-corrupted canary
/// before the block is freed (otherwise the production dealloc
/// path would panic on the still-corrupt canary, killing the
/// selftest mid-run).
///
/// SAFETY: caller holds the only pointer to this block, and
/// `payload_size` matches the size originally passed to
/// `wrap_alloc`.
pub unsafe fn repair_for_test(user_ptr: *mut u8, payload_size: usize) {
    let inner_ptr = unsafe { user_ptr.sub(GUARD_SIZE) };
    let cnry = canary(inner_ptr as usize, payload_size);
    unsafe {
        core::ptr::copy_nonoverlapping(cnry.as_ptr(), inner_ptr, GUARD_SIZE);
        let back = inner_ptr.add(GUARD_SIZE + payload_size);
        core::ptr::copy_nonoverlapping(cnry.as_ptr(), back, GUARD_SIZE);
    }
}

/// Snapshot of guard counters. Cheap, reflects state at the time
/// of the call. Exposed for the `heap-stats` shell command.
pub fn stats() -> (u64, u64, u64) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        FREE_COUNT.load(Ordering::Relaxed),
        CORRUPTION_COUNT.load(Ordering::Relaxed),
    )
}

/// Increment the corruption counter (no panic). For the selftest
/// only — production callers panic on corruption via
/// `panic_on_fault`.
#[cfg(test)]
pub fn _bump_corruption_for_test() {
    CORRUPTION_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Panic with a descriptive message for a corruption fault.
pub fn panic_on_fault(fault: VerifyFault, addr: usize) -> ! {
    let msg = match fault {
        VerifyFault::DoubleFree         => "heap double-free detected",
        VerifyFault::UnderflowOrAlien   => "heap underflow or alien pointer",
        VerifyFault::Overflow           => "heap overflow detected (back canary corrupted)",
    };
    let _ = addr;
    panic!("{msg}");
}
