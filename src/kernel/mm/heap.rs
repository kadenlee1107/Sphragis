// Sphragis — kernel heap.
//
// V6-KMEM-001 fix: heap base is now DYNAMIC, set by mm::init AFTER the
// initrd blob is parsed. The previous fixed base 0x48000000 (128 MB
// above kernel image) collided with baked Chromium blobs larger than
// ~120 MB — alloc_frame would silently hand out heap pages from the
// blob bytes, then any cave mmap could read/write the heap.
//
// Layout choice: caller passes `base = (initrd_blob_end + page) & !page`
// so the heap sits IMMEDIATELY past the blob, with the heap range still
// reserved in the frame bitmap via frame::reserve_range.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use linked_list_allocator::Heap;

// Bumped from 4 MB → 32 MB to give Argon2id (DESIGN_CRYPTO.md #1) and
// the upcoming PQ primitives (ML-KEM key exchange, ML-DSA signatures)
// room to work. Argon2 alone scales with its memory_cost parameter
// (per-auth cost + attacker's per-guess cost); we set it to 8 MiB in
// security::auth, leaving ~24 MB for general kernel heap. Measured
// peak utilisation under the 40/40 test suite was ~2 MB so the slack
// is deliberate.
pub const KERNEL_HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 MB

/// Resolved at init() time. Reads as 0 before init().
static HEAP_BASE: AtomicUsize = AtomicUsize::new(0);

/// Public accessor for callers (frame allocator) that need to know
/// which range to reserve in the bitmap.
pub fn kernel_heap_base() -> usize { HEAP_BASE.load(Ordering::Acquire) }
pub fn kernel_heap_size() -> usize { KERNEL_HEAP_SIZE }

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator {
    inner: UnsafeCell::new(Heap::empty()),
};

/// Single-CPU allocator — we hold IRQs off during the allocation
/// critical section instead of using a mutex.
///
/// Previously this was `spin::Mutex<Heap>` (via `LockedHeap`). On
/// Apple Silicon with the MMU disabled (which is how m1n1 hands off
/// on M4), every memory access is Device-nGnRnE, and LDXR/STXR on
/// Device memory have unpredictable behavior — specifically, STXR
/// always fails. `spin::Mutex::lock()` uses `AtomicBool::
/// compare_exchange_weak`, so the lock spin never makes progress and
/// `heap::init` hangs forever. Since Sphragis is single-CPU during
/// bring-up (we chainload with `-S`), the mutex was just a nicety;
/// masking IRQs is enough mutual exclusion.
pub struct KernelAllocator {
    inner: UnsafeCell<Heap>,
}

/// SAFETY: single-CPU, and every allocation masks IRQs before touching
/// the inner heap.
unsafe impl Sync for KernelAllocator {}

#[inline(always)]
unsafe fn irq_save() -> u64 {
    let prev: u64;
    unsafe {
        core::arch::asm!(
            "mrs {p}, daif",
            "msr daifset, #0x2",   // mask IRQ (DAIF.I)
            p = out(reg) prev,
            options(nostack, preserves_flags),
        );
    }
    prev
}

#[inline(always)]
unsafe fn irq_restore(prev: u64) {
    unsafe {
        core::arch::asm!("msr daif, {p}", p = in(reg) prev,
            options(nostack, preserves_flags));
    }
}

/// Inner layout for guarded allocations: payload + 2 * GUARD_SIZE,
/// aligned to at least GUARD_SIZE so the canary frames sit at 16-
/// byte boundaries inside the block. Falls back to `None` when
/// the caller's alignment exceeds GUARD_SIZE — those allocations
/// skip the guard wrapper (rare in this kernel).
fn guarded_layout(layout: Layout) -> Option<Layout> {
    use crate::kernel::mm::guard::{FRAME_OVERHEAD, GUARD_SIZE};
    if layout.align() > GUARD_SIZE { return None; }
    let new_size = layout.size().checked_add(FRAME_OVERHEAD)?;
    Layout::from_size_align(new_size, GUARD_SIZE).ok()
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let saved = unsafe { irq_save() };
        let heap = unsafe { &mut *self.inner.get() };
        let result = if let Some(inner) = guarded_layout(layout) {
            match heap.allocate_first_fit(inner) {
                Ok(p) => {
                    // SAFETY: heap just gave us a block of `inner.size()`
                    // bytes (= layout.size() + 2*GUARD_SIZE). wrap_alloc
                    // writes both canaries and returns inner_ptr+GUARD_SIZE.
                    unsafe { crate::kernel::mm::guard::wrap_alloc(p.as_ptr(), layout.size()) }
                }
                Err(_) => ptr::null_mut(),
            }
        } else {
            // High-alignment path: skip canaries entirely.
            match heap.allocate_first_fit(layout) {
                Ok(p) => p.as_ptr(),
                Err(_) => ptr::null_mut(),
            }
        };
        unsafe { irq_restore(saved); }
        result
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        if p.is_null() { return; }
        // V5-KMEM-002: scrub before returning memory to the free list.
        for i in 0..layout.size() {
            unsafe { core::ptr::write_volatile(p.add(i), 0); }
        }
        let saved = unsafe { irq_save() };
        if let Some(inner) = guarded_layout(layout) {
            // SAFETY: paired with the wrap_alloc above — same payload size,
            // same user_ptr that came back from alloc. verify_and_unwrap
            // checks both canaries and poisons the front canary so a
            // second free of this address fires the corruption detector.
            match unsafe { crate::kernel::mm::guard::verify_and_unwrap(p, layout.size()) } {
                Ok(inner_ptr) => {
                    if let Some(nn) = core::ptr::NonNull::new(inner_ptr) {
                        let heap = unsafe { &mut *self.inner.get() };
                        unsafe { heap.deallocate(nn, inner); }
                    }
                }
                Err(fault) => {
                    // Re-enable IRQs before panicking so the panic
                    // handler doesn't deadlock waiting on a sub-system
                    // that wants IRQs.
                    unsafe { irq_restore(saved); }
                    crate::kernel::mm::guard::panic_on_fault(fault, p as usize);
                }
            }
        } else if let Some(nn) = core::ptr::NonNull::new(p) {
            let heap = unsafe { &mut *self.inner.get() };
            unsafe { heap.deallocate(nn, layout); }
        }
        unsafe { irq_restore(saved); }
    }
}

/// Initialize the kernel heap. Call once from early boot, before any
/// `Box`/`Vec`/`String` is used. `base` MUST be page-aligned.
pub fn init(base: usize) {
    HEAP_BASE.store(base, Ordering::Release);
    unsafe {
        let heap = &mut *ALLOCATOR.inner.get();
        heap.init(base as *mut u8, KERNEL_HEAP_SIZE);
    }
}

// alloc_error_handler is nightly-only; on stable Rust 1.94 we rely on the
// allocator returning null from alloc() — Box/Vec then propagate that into
// a regular panic via the default handler. We keep the OOM path explicit
// in main.rs's panic handler which is reached on alloc failure.
