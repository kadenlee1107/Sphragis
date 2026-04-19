#![allow(dead_code)]
pub mod frame;
pub mod heap;
pub mod initrd;
pub mod page_table;

// Use the platform-dispatched serial so this module works on both
// QEMU (PL011) and Apple (dockchannel) without writing to the wrong
// MMIO. `platform::serial_*` reads `CURRENT_PLATFORM` and forwards
// to the correct driver.
use crate::platform;

unsafe extern "C" {
    pub static __kernel_end: u8;
}

// QEMU-virt / Chromium host: 1 GiB RAM base + 2 GiB = 3 GiB top.
// Apple Silicon path overrides this with boot_args-derived values.
const QEMU_MEMORY_END: usize = 0x4000_0000 + 2 * 1024 * 1024 * 1024;

pub fn init() {
    // V6-KMEM-001: order is
    //   1. parse initrd (must run BEFORE heap so we know where blob ends)
    //   2. compute heap base = (blob_end + 1 page); init heap there
    //   3. init frame allocator over [past_heap, MEMORY_END)
    //   4. reserve heap range in frame bitmap (frame::init already does
    //      this for the heap range it's told about — see below)
    initrd::init();

    let kernel_end = core::ptr::addr_of!(__kernel_end) as usize;
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
    let heap_base = (blob_end_aligned + 0xFFFF) & !0xFFFF;
    let memory_end = if crate::platform::is_apple_silicon() {
        crate::drivers::apple::boot_args::with(|b| {
            (b.phys_base().saturating_add(b.mem_size())) as usize
        }).unwrap_or(heap_base + 256 * 1024 * 1024)
    } else {
        QEMU_MEMORY_END
    };

    heap::init(heap_base);
    let frame_start = heap_base + heap::kernel_heap_size();
    let frame_start = (frame_start + 0xFFF) & !0xFFF;

    frame::init(frame_start, memory_end);

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
