#![allow(dead_code)]
pub mod frame;
pub mod initrd;
pub mod page_table;

use crate::drivers::uart;

unsafe extern "C" {
    pub static __kernel_end: u8;
}

const MEMORY_END: usize = 0x4000_0000 + 2 * 1024 * 1024 * 1024; // RAM base + 2GB (Chromium host)

pub fn init() {
    // Detect and record any baked Chromium blob BEFORE we hand the
    // region past `__kernel_end` to the frame allocator, so the
    // allocator can skip over the blob's footprint.
    initrd::init();

    let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
    let heap_start = match initrd::info() {
        Some(bi) => {
            // Blob layout past __kernel_end is:
            //   [BATCHROM 8][size 8][bytes N][crc 4][CHROMEND 8]
            // Round up one page past the trailer so the frame
            // allocator has aligned start.
            let blob_end = kernel_end + 16 + bi.size + 4 + 8;
            (blob_end + 0xFFF) & !0xFFF
        }
        None => kernel_end,
    };
    frame::init(heap_start, MEMORY_END);

    let (used, total) = frame::stats();
    uart::puts("  [mm] Frame allocator initialized — ");
    print_num((total - used) * 4);
    uart::puts(" KB free\n");
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
