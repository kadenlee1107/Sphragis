// Bat_OS — Kernel-mode C runtime for Blink
// Provides malloc/free/printf/memcpy that work at EL1 (kernel mode).
// NO syscalls (svc) — everything runs in kernel space.

use crate::drivers::uart;

// 2MB bump allocator for Blink tokenizer
static mut BLINK_HEAP: [u8; 262144] = [0u8; 262144]; // 256KB
static mut BLINK_HEAP_POS: usize = 0;

/// Reset heap between tokenization sessions
pub fn reset_heap() {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(BLINK_HEAP_POS), 0); }
}

/// V11-state-sweep: on cave switch, additionally zero the WHOLE 256 KiB
/// heap backing store — not just the bump-position cursor. The bump
/// cursor reset alone leaves every prior HTML document's tokenized bytes
/// readable until new allocations happen to overlap. Zero-on-reset makes
/// the leak window zero.
pub fn reset_for_cave_switch() {
    unsafe {
        let p = core::ptr::addr_of_mut!(BLINK_HEAP) as *mut u8;
        for i in 0..262144 {
            core::ptr::write_volatile(p.add(i), 0);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(BLINK_HEAP_POS), 0);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe {
        let pos = core::ptr::read_volatile(core::ptr::addr_of!(BLINK_HEAP_POS));
        let aligned = (pos + 15) & !15;
        let heap_ptr = core::ptr::addr_of_mut!(BLINK_HEAP) as *mut u8;
        if aligned + size > 262144 {
            uart::puts("[blink] malloc OOM!\n");
            core::ptr::null_mut()
        } else {
            let ptr = heap_ptr.add(aligned);
            core::ptr::write_volatile(core::ptr::addr_of_mut!(BLINK_HEAP_POS), aligned + size);
            ptr
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free(_ptr: *mut u8) {
    // Bump allocator — no individual frees
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    let new_ptr = malloc(new_size);
    if !new_ptr.is_null() && !ptr.is_null() {
        unsafe { core::ptr::copy_nonoverlapping(ptr, new_ptr, new_size.min(65536)); }
    }
    new_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    // V8-ROOT-3: classic CWE-190 — count*size from attacker-driven DOM sizes
    // wraps usize and yields a tiny allocation that the caller treats as
    // huge, leading to a heap-buffer overflow. Guard with checked_mul.
    let total = match count.checked_mul(size) {
        Some(n) if n > 0 => n,
        _ => return core::ptr::null_mut(),
    };
    let ptr = malloc(total);
    if !ptr.is_null() {
        unsafe { core::ptr::write_bytes(ptr, 0, total); }
    }
    ptr
}

// abort and puts provided by blink_printf.c

// printf is provided by blink_printf.c (compiled separately)
// It's a minimal C implementation that uses uart_putc

// memcpy, memset, memmove, memcmp, strlen, bcmp — provided by compiler_builtins
// DO NOT override here — it causes infinite recursion!

// C++ runtime support
#[unsafe(no_mangle)]
pub static __dso_handle: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn __cxa_atexit(_func: *mut u8, _arg: *mut u8, _dso: *mut u8) -> i32 { 0 }

#[unsafe(no_mangle)]
pub extern "C" fn __cxa_guard_acquire(guard: *mut u64) -> i32 {
    unsafe {
        if *guard == 0 { *guard = 1; 1 } else { 0 }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __cxa_guard_release(_guard: *mut u64) {}

#[unsafe(no_mangle)]
pub extern "C" fn __cxa_guard_abort(_guard: *mut u64) {}

// C++ new/delete (forward to malloc/free)
#[unsafe(no_mangle)]
pub extern "C" fn _Znwm(size: usize) -> *mut u8 { malloc(size) }
#[unsafe(no_mangle)]
pub extern "C" fn _Znam(size: usize) -> *mut u8 { malloc(size) }
#[unsafe(no_mangle)]
pub extern "C" fn _ZdlPv(ptr: *mut u8) { free(ptr) }
#[unsafe(no_mangle)]
pub extern "C" fn _ZdaPv(ptr: *mut u8) { free(ptr) }

#[unsafe(no_mangle)]
pub extern "C" fn sincos(_x: f64, s: *mut f64, c: *mut f64) {
    // Stub — not needed for tokenization
    unsafe { *s = 0.0; *c = 1.0; }
}
