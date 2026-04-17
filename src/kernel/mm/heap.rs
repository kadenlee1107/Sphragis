// Bat_OS — kernel heap.
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
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use linked_list_allocator::LockedHeap;

pub const KERNEL_HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Resolved at init() time. Reads as 0 before init().
static HEAP_BASE: AtomicUsize = AtomicUsize::new(0);

/// Public accessor for callers (frame allocator) that need to know
/// which range to reserve in the bitmap.
pub fn kernel_heap_base() -> usize { HEAP_BASE.load(Ordering::Acquire) }
pub fn kernel_heap_size() -> usize { KERNEL_HEAP_SIZE }

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator { inner: LockedHeap::empty() };

pub struct KernelAllocator {
    inner: LockedHeap,
}

/// V6-TOCTOU-007 fix: mask IRQs while holding the allocator spinlock.
/// `linked_list_allocator::LockedHeap` uses `spin::Mutex` which is NOT
/// reentrant. If a timer IRQ fires while EL1 holds the lock and the
/// IRQ handler itself tries to allocate (e.g. logs that touch heap-
/// backed strings), we spin forever waiting for ourselves. Mask DAIF.I
/// during the critical section to prevent IRQ re-entry.
#[inline(always)]
unsafe fn irq_save() -> u64 {
    let prev: u64;
    core::arch::asm!(
        "mrs {p}, daif",
        "msr daifset, #0x2",   // mask IRQ (DAIF.I)
        p = out(reg) prev,
        options(nostack, preserves_flags),
    );
    prev
}

#[inline(always)]
unsafe fn irq_restore(prev: u64) {
    core::arch::asm!("msr daif, {p}", p = in(reg) prev,
        options(nostack, preserves_flags));
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let saved = irq_save();
        let result = match self.inner.lock().allocate_first_fit(layout) {
            Ok(p) => p.as_ptr(),
            Err(_) => ptr::null_mut(),
        };
        irq_restore(saved);
        result
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // V5-KMEM-002 fix: zero the chunk before returning it to the
        // free list so residue (X.509 cert DER, SPKI bytes, TLS
        // transcript hashes) doesn't linger for the next allocator
        // client to read. Uses write_volatile so LLVM can't elide.
        if !ptr.is_null() {
            for i in 0..layout.size() {
                core::ptr::write_volatile(ptr.add(i), 0);
            }
        }
        let saved = irq_save();
        if let Some(nn) = core::ptr::NonNull::new(ptr) {
            self.inner.lock().deallocate(nn, layout);
        }
        irq_restore(saved);
    }
}

/// Initialize the kernel heap. Call once from early boot, before any
/// `Box`/`Vec`/`String` is used. `base` MUST be page-aligned and live
/// inside kernel-RAM (L2_high mapping) but NOT inside the baked
/// content_shell blob — caller (mm::init) computes blob_end + 1 page.
pub fn init(base: usize) {
    HEAP_BASE.store(base, Ordering::Release);
    unsafe {
        ALLOCATOR.inner.lock().init(base as *mut u8, KERNEL_HEAP_SIZE);
    }
}

// alloc_error_handler is nightly-only; on stable Rust 1.94 we rely on the
// allocator returning null from alloc() — Box/Vec then propagate that into
// a regular panic via the default handler. We keep the OOM path explicit
// in main.rs's panic handler which is reached on alloc failure.
