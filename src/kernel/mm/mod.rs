#![allow(dead_code)]
pub mod frame;
pub mod heap;
pub mod initrd;
pub mod page_table;

use crate::drivers::uart;

unsafe extern "C" {
    pub static __kernel_end: u8;
}

const MEMORY_END: usize = 0x4000_0000 + 2 * 1024 * 1024 * 1024; // RAM base + 2GB (Chromium host)

pub fn init() {
    // V6-KMEM-001 fix: order is now
    //   1. parse initrd (must run BEFORE heap so we know where blob ends)
    //   2. compute heap base = (blob_end + 1 page); init heap there
    //   3. init frame allocator over [past_heap, MEMORY_END)
    //   4. reserve heap range in frame bitmap (frame::init already does
    //      this for the heap range it's told about — see below)
    //
    // The previous order had the heap at fixed 0x48000000 — for blobs
    // larger than ~120 MB, blob_end > 0x48000000 and the heap pages
    // overlapped the blob bytes.
    initrd::init();

    let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
    let blob_end_aligned = match initrd::info() {
        Some(bi) => {
            let blob_end = kernel_end + 16 + bi.size + 4 + 8;
            (blob_end + 0xFFF) & !0xFFF
        }
        None => kernel_end,
    };

    // Place heap immediately past the blob. Round to 64 KB for safety
    // margin. Then frame allocator starts past heap.
    let heap_base = (blob_end_aligned + 0xFFFF) & !0xFFFF;
    heap::init(heap_base);
    let frame_start = heap_base + heap::kernel_heap_size();
    let frame_start = (frame_start + 0xFFF) & !0xFFF;

    frame::init(frame_start, MEMORY_END);

    let (used, total) = frame::stats();
    uart::puts("  [mm] Frame allocator initialized — ");
    print_num((total - used) * 4);
    uart::puts(" KB free, heap @ 0x");
    print_hex(heap_base);
    uart::puts("\n");
}

fn print_hex(n: usize) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = (n >> (i * 4)) & 0xF;
        uart::putc(hex[nibble]);
    }
}

pub fn print_num(mut n: usize) {
    if n == 0 {
        uart::putc(b'0');
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
        uart::putc(buf[i]);
    }
}
