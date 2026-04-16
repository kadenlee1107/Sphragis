// Bat_OS — kernel heap.
//
// V4: added so the X.509 verifier (x509-cert/der crates) has an allocator.
// Uses `linked_list_allocator` backed by a 4 MB reserved region carved
// from kernel RAM. No growth; exhaustion aborts the allocation (returns
// null from `GlobalAlloc::alloc`).
//
// Layout choice: heap region sits at KERNEL_HEAP_BASE and grows up. We
// pick 0x48000000 — 128 MB above the kernel image base 0x40000000. This
// is inside the L2_high identity-mapped range, marked kernel-data
// (NX + EL1 RW) by the page tables.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use linked_list_allocator::LockedHeap;

pub const KERNEL_HEAP_BASE: usize = 0x4800_0000;
pub const KERNEL_HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 MB

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator { inner: LockedHeap::empty() };

pub struct KernelAllocator {
    inner: LockedHeap,
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.inner.lock().allocate_first_fit(layout) {
            Ok(p) => p.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(nn) = core::ptr::NonNull::new(ptr) {
            self.inner.lock().deallocate(nn, layout);
        }
    }
}

/// Initialize the kernel heap. Call once from early boot, before any
/// `Box`/`Vec`/`String` is used.
pub fn init() {
    unsafe {
        ALLOCATOR.inner.lock().init(KERNEL_HEAP_BASE as *mut u8, KERNEL_HEAP_SIZE);
    }
}

// alloc_error_handler is nightly-only; on stable Rust 1.94 we rely on the
// allocator returning null from alloc() — Box/Vec then propagate that into
// a regular panic via the default handler. We keep the OOM path explicit
// in main.rs's panic handler which is reached on alloc failure.
