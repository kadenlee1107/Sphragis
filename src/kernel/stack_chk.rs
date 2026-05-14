//! Stack-canary failure handler — required by `-Z stack-protector=all`.
//!
//! The compiler emits a prologue + epilogue on every function that
//! reads `__stack_chk_guard` into the frame, then checks it on
//! return; on mismatch it calls `__stack_chk_fail()`. We define both
//! here.
//!
//! Today's `__stack_chk_guard` is a build-time constant — better
//! than nothing, but predictable. A future commit randomizes it at
//! boot via RNDR before any user code can run.
//!
//! `__stack_chk_fail` halts immediately. We intentionally do NOT
//! try to clean up — a smashed canary means memory corruption has
//! already happened, so any further code paths (audit log, wipe, etc.)
//! are unsafe to invoke. We just print and halt.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::drivers::uart;

/// 64-bit canary the compiler reads at function entry. The default
/// value is what the codegen expects to be a non-zero, non-trivial
/// pattern. Boot-time randomization will overwrite this.
#[unsafe(no_mangle)]
pub static mut __stack_chk_guard: u64 = 0xdead_beef_cafe_babe;

/// Called by the prologue/epilogue check when the canary doesn't
/// match. By the time we get here, the stack frame is already
/// corrupt — we can't trust local state. Print + halt.
#[unsafe(no_mangle)]
pub extern "C" fn __stack_chk_fail() -> ! {
    uart::puts("\n!! STACK CANARY FAILURE !!\n");
    uart::puts("kernel halted to prevent exploitation\n");
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}

/// Optionally called from `kernel_main` early in boot to seed the
/// canary from the hardware RNG so it isn't a predictable constant.
/// Safe to call exactly once before any function that touches it.
#[allow(dead_code)]
pub unsafe fn seed_from_rng() {
    let mut v: u64 = 0;
    // ARMv8.5 RNDR. Returns 0 on failure; we mix in a fallback
    // counter so we still have entropy if RNDR isn't usable.
    let nzcv: u64;
    unsafe {
        core::arch::asm!(
            "mrs {0}, RNDR",
            "mrs {1}, NZCV",
            out(reg) v,
            out(reg) nzcv,
        );
    }
    if (nzcv >> 30) & 1 == 1 {
        // RNDR returned 0; fall back to cntpct mix.
        let t: u64;
        unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) t); }
        v = t ^ 0xc6bc_279e_a6a8_8d57;
    }
    // Avoid 0 — some libc impls treat 0 as "uninitialized."
    if v == 0 { v = 0xa5a5_a5a5_5a5a_5a5a; }
    unsafe { __stack_chk_guard = v; }
}
