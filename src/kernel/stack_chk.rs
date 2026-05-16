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

/// Called from `kernel_main` early in boot to seed the canary from
/// the hardware RNG so it isn't a predictable constant. Safe to call
/// exactly once before any function that touches it.
///
/// AUDIT-MEM-H2 (2026-05-15): wired into kernel_main right after
/// `crypto::rng::probe_hw_rng()`. Prior to this commit the function
/// existed but had zero call sites; the canary was the literal
/// `0xdead_beef_cafe_babe`, making the mitigation structurally
/// disabled.
pub unsafe fn seed_from_rng() {
    // Use the raw RNDR encoding (s3_3_c2_c4_0) — symbolic `RNDR`
    // requires the v8.5 target-feature flag which isn't enabled in
    // .cargo/config. The same encoding is used by crypto::rng.
    let mut v: u64 = 0;
    let ok: u64;
    unsafe {
        core::arch::asm!(
            "mrs {v}, s3_3_c2_c4_0",    // RNDR (ARMv8.5)
            "cset {ok}, ne",             // NZCV.Z clear ⇒ success
            v = out(reg) v,
            ok = out(reg) ok,
            options(nostack, preserves_flags),
        );
    }
    if ok == 0 {
        // RNDR transiently unavailable; fall back to cntpct mix.
        let t: u64;
        unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) t,
                                  options(nostack, preserves_flags)); }
        v = t ^ 0xc6bc_279e_a6a8_8d57;
    }
    // Avoid 0 — some libc impls treat 0 as "uninitialized."
    if v == 0 { v = 0xa5a5_a5a5_5a5a_5a5a; }
    unsafe { __stack_chk_guard = v; }
}
