#![allow(dead_code)]
pub mod frame;
pub mod page_table;

use crate::drivers::uart;

unsafe extern "C" {
    static __kernel_end: u8;
}

const MEMORY_END: usize = 0x4000_0000 + 128 * 1024 * 1024; // RAM base + 128MB

pub fn init() {
    let heap_start = core::ptr::addr_of!(__kernel_end) as usize;
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
