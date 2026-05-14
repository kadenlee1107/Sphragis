#![allow(dead_code)]
pub mod cave_pool;
pub mod frame;
pub mod guard;
pub mod heap;
pub mod initrd;
pub mod mmu_el2;
pub mod page_table;

// Use the platform-dispatched serial so this module works on both
// QEMU (PL011) and Apple (dockchannel) without writing to the wrong
// MMIO. `platform::serial_*` reads `CURRENT_PLATFORM` and forwards
// to the correct driver.
use crate::platform;

unsafe extern "C" {
    pub static __kernel_end: u8;
}

// QEMU-virt / Chromium host: 1 GiB RAM base + 4 GiB = 5 GiB top.
// Apple Silicon path overrides this with boot_args-derived values.
//
// was `+ 2 * 1024 * 1024 * 1024` (= 0xC0000000), giving
// only 2 GiB of usable physical RAM (kernel + ~1.5 GB user). Chromium
// content_shell with full thread pool + V8 cage + PartitionAlloc heap
// hits 296k demand-page commits = 1.2 GB before exhausting frames and
// crashing in `[demand_page] OOM`. Bumping to 4 GiB usable matches
// the smoke script's `qemu -m 4G` and gives plenty of headroom.
//
// MUST be paired with extending the cave + primary identity map in
// `src/caves/linux/mmu.rs` (L1[3] + L1[4]) — otherwise kernel writes
// to PAs above 0xC0000000 (where alloc_frame would now hand out
// frames) would fault DATA ABORT DFSC=0x06.
// 4 GiB to give Chromium enough working set.
const QEMU_MEMORY_END: usize = 0x4000_0000 + 4 * 1024 * 1024 * 1024;

pub fn init() {
    // V6-KMEM-001: order is
    // 1. parse initrd (must run BEFORE heap so we know where blob ends)
    // 2. compute heap base = (blob_end + 1 page); init heap there
    // 3. init frame allocator over [past_heap, MEMORY_END)
    // 4. reserve heap range in frame bitmap (frame::init already does
    // this for the heap range it's told about — see below)
    initrd::init();

    let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
    let (ir_start, ir_end) = initrd::blob_phys_range();
    let ir_end_aligned = (ir_end + 0xFFFF) & !0xFFFF;
    let blob_end_aligned = match initrd::info() {
        Some(bi) => {
            let blob_end = kernel_end + 16 + bi.size + 4 + 8;
            (blob_end + 0xFFF) & !0xFFF
        }
        None => kernel_end,
    };

    // Platform-dispatched memory end. On Apple we pull phys_base +
    // mem_size from the stashed boot_args (set by kernel_main_apple).
    // On QEMU we use the old hardcoded value. Both paths place the
    // heap immediately past the end of the loaded kernel/blob.
    //
    // CHROMIUM-PHASE-B fix: take max(blob_end_aligned, ir_end_aligned).
    // With QEMU `-initrd`, the blob actually lives at 0x48000000 (far
    // past kernel_end), so using kernel_end + blob_size as heap_base
    // placed the heap INSIDE the initrd region. alloc_frame reserved
    // the initrd range but the HEAP allocator itself didn't know;
    // kernel allocations silently stomped content_shell's bytes and
    // ld-linux later crashed with NULL-deref on a half-relocated
    // data pointer. Bumping heap_base past the real initrd end
    // eliminates the overlap.
    let heap_base = {
        let a = (blob_end_aligned + 0xFFFF) & !0xFFFF;
        let b = ir_end_aligned;
        if a > b { a } else { b }
    };
    let memory_end = if crate::platform::is_apple_silicon() {
        crate::drivers::apple::boot_args::with(|b| {
            (b.phys_base().saturating_add(b.mem_size())) as usize
        }).unwrap_or(heap_base + 256 * 1024 * 1024)
    } else {
        QEMU_MEMORY_END
    };

    // Seed the heap-guard canary key BEFORE init so the very first
    // allocation gets a randomised canary. probe_hw_rng + fill_bytes
    // are pure register/stack operations and do not allocate, so
    // running them before heap::init is safe. If RNDR is unavailable
    // we still get the SHA-chain DRBG output, which is enough for a
    // per-boot canary key — see src/kernel/mm/guard.rs.
    crate::crypto::rng::probe_hw_rng();
    let mut seed = [0u8; 32];
    crate::crypto::rng::fill_bytes(&mut seed);
    guard::init(&seed);
    // Wipe the local copy of the seed before it leaves scope.
    for b in seed.iter_mut() { unsafe { core::ptr::write_volatile(b, 0); } }

    heap::init(heap_base);
    let frame_start = heap_base + heap::kernel_heap_size();
    let frame_start = (frame_start + 0xFFF) & !0xFFF;

    frame::init(frame_start, memory_end);

    // Reserve the cave-pool PA range (0xB000_0000..0xC000_0000) so
    // frame::alloc_frame / alloc_kernel_frame never hand out pages
    // inside it. Cave-private allocations go through
    // `kernel::mm::cave_pool::alloc_page()` instead. Must run
    // BEFORE setup_and_enable so no L1/L2/L3 table lands inside the
    // carve-out and gets unmapped.
    cave_pool::init();
    platform::serial_puts("  [mm] cave-pool reserved (256 MiB at 0xB000_0000)\n");

    // BUG-2026-04-23: when the initrd is delivered via QEMU `-initrd`
    // instead of appended-to-kernel, its actual phys address (e.g.
    // 0x48000000 on -m 4G virt) is INSIDE the frame pool's range.
    // Without this reservation, `alloc_frame` returns pages from the
    // initrd region and the multi-ELF loader's PT_LOAD copies smash
    // the baked-in content_shell + .so archive before we can read it.
    if ir_end > ir_start {
        frame::reserve_range(ir_start, ir_end);
        platform::serial_puts("  [mm] initrd reserved @ 0x");
        print_hex(ir_start);
        platform::serial_puts("..0x");
        print_hex(ir_end);
        platform::serial_puts("\n");
    }

    let (used, total) = frame::stats();
    platform::serial_puts("  [mm] Frame allocator initialized — ");
    print_num((total - used) * 4);
    platform::serial_puts(" KB free, heap @ 0x");
    print_hex(heap_base);
    platform::serial_puts("\n");
}

fn print_hex(n: usize) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = (n >> (i * 4)) & 0xF;
        platform::serial_putc(hex[nibble]);
    }
}

pub fn print_num(mut n: usize) {
    if n == 0 {
        platform::serial_putc(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        platform::serial_putc(buf[i]);
    }
}
