//! Shared helpers for the KM attack tests. Re-implements the minimal subset of
//! the kernel logic we want to pin down, so each test can run under plain
//! `cargo test` on the host without dragging in the `no_std` kernel tree.

pub mod frame_sim {
    //! Faithful port of `src/kernel/mm/frame.rs` semantics — bitmap, relaxed
    //! load/store — so we can observe the TOCTOU.

    use std::sync::atomic::{AtomicU64, Ordering};

    pub const PAGE_SIZE: usize = 4096;
    pub const MAX_FRAMES: usize = 524288;
    pub const BITMAP_SIZE: usize = MAX_FRAMES / 64;

    pub struct Allocator {
        pub bitmap: Vec<AtomicU64>,
        pub memory_start: usize,
        pub total_frames: usize,
    }

    impl Allocator {
        pub fn new(start: usize, end: usize) -> Self {
            let start_aligned = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            let end_aligned = end & !(PAGE_SIZE - 1);
            let total = (end_aligned - start_aligned) / PAGE_SIZE;
            let mut bitmap = Vec::with_capacity(BITMAP_SIZE);
            for _ in 0..BITMAP_SIZE {
                bitmap.push(AtomicU64::new(0));
            }
            Self {
                bitmap,
                memory_start: start_aligned,
                total_frames: total,
            }
        }

        /// Exact same pattern as kernel: load, find free bit, store back.
        /// NOT atomic — this is what the production code does today.
        pub fn alloc_frame_racy(&self) -> Option<usize> {
            for i in 0..BITMAP_SIZE {
                let val = self.bitmap[i].load(Ordering::Relaxed);
                if val == u64::MAX {
                    continue;
                }
                let bit = (!val).trailing_zeros() as usize;
                let frame_index = i * 64 + bit;
                if frame_index >= self.total_frames {
                    return None;
                }
                // === WINDOW === a second thread can race here with us.
                let new_val = val | (1u64 << bit);
                self.bitmap[i].store(new_val, Ordering::Relaxed);
                return Some(self.memory_start + frame_index * PAGE_SIZE);
            }
            None
        }

        pub fn free_frame_lax(&self, addr: usize) -> bool {
            // Mirrors kernel: no alignment check, no prior-state check.
            if addr < self.memory_start {
                return false;
            }
            let frame_index = (addr - self.memory_start) / PAGE_SIZE;
            let bitmap_index = frame_index / 64;
            let bit = frame_index % 64;
            if bitmap_index >= BITMAP_SIZE {
                return false;
            }
            let val = self.bitmap[bitmap_index].load(Ordering::Relaxed);
            self.bitmap[bitmap_index].store(val & !(1u64 << bit), Ordering::Relaxed);
            true
        }

        pub fn bit_is_set(&self, frame_index: usize) -> bool {
            let i = frame_index / 64;
            let b = frame_index % 64;
            (self.bitmap[i].load(Ordering::Relaxed) >> b) & 1 == 1
        }
    }
}

pub mod futex_sim {
    //! Port of the hash in `src/caves/linux/futex.rs::bucket_index`.
    pub const NUM_BUCKETS: usize = 64;
    pub const WAITERS_PER_BUCKET: usize = 32;

    pub fn bucket_index(uaddr: u64) -> usize {
        let mut h = uaddr >> 2;
        h ^= h >> 17;
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        (h as usize) % NUM_BUCKETS
    }

    /// True iff the exact `validate_uaddr` check in `futex_wait`
    /// (`uaddr == 0 || (uaddr & 0x3) != 0`) would accept this address.
    pub fn passes_uaddr_check(uaddr: u64) -> bool {
        !(uaddr == 0 || (uaddr & 0x3) != 0)
    }
}

pub mod heap_sim {
    //! Port of `src/kernel/mm/mod.rs::init`'s heap_start computation.
    pub struct BlobInfo {
        pub size: usize,
    }

    /// Unchecked: matches today's kernel. We use wrapping_add because `cargo
    /// test` defaults to debug builds (overflow checks on), and the kernel
    /// release build wraps silently — that wrapping IS the bug we're
    /// demonstrating.
    pub fn heap_start_unchecked(kernel_end: usize, info: Option<BlobInfo>) -> usize {
        match info {
            Some(bi) => {
                let blob_end = kernel_end
                    .wrapping_add(16)
                    .wrapping_add(bi.size)
                    .wrapping_add(4)
                    .wrapping_add(8);
                blob_end.wrapping_add(0xFFF) & !0xFFF
            }
            None => kernel_end,
        }
    }

    /// What the kernel _should_ do.
    pub fn heap_start_checked(
        kernel_end: usize,
        info: Option<BlobInfo>,
        memory_end: usize,
    ) -> Option<usize> {
        match info {
            Some(bi) => {
                let blob_end = kernel_end
                    .checked_add(16)?
                    .checked_add(bi.size)?
                    .checked_add(4)?
                    .checked_add(8)?;
                let aligned = blob_end.checked_add(0xFFF)? & !0xFFF;
                if aligned >= memory_end {
                    None
                } else {
                    Some(aligned)
                }
            }
            None => Some(kernel_end),
        }
    }
}

pub mod pt_flags {
    //! Constants from `src/caves/linux/mmu.rs`.
    pub const PTE_VALID: u64 = 1;
    pub const PTE_TABLE: u64 = 1 << 1;
    pub const PTE_AF: u64 = 1 << 10;
    pub const PTE_SH_INNER: u64 = 3 << 8;
    pub const PTE_ATTR_NORMAL: u64 = 0 << 2;
    pub const PTE_ATTR_DEVICE: u64 = 1 << 2;
    pub const PTE_AP_RW: u64 = 0 << 6;
    pub const PTE_UXN: u64 = 1 << 54;
    pub const PTE_PXN: u64 = 1 << 53;

    pub const BLOCK_NORMAL: u64 =
        PTE_VALID | PTE_AF | PTE_SH_INNER | PTE_ATTR_NORMAL | PTE_AP_RW;
}

pub mod trap_frame {
    //! Port of `src/kernel/arch/mod.rs::TrapFrame` layout.
    #[repr(C)]
    pub struct TrapFrame {
        pub x: [u64; 31],
        pub elr: u64,
        pub spsr: u64,
    }

    pub const SAVED_FRAME_BYTES: usize = 35 * 8; // static array in kernel
    pub const HANDLER_COPY_BYTES: usize = 272; // hardcoded in loop
}
