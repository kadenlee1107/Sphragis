// Sphragis — kernel concurrency primitives.
//
// Created for V8-ROOT-1: every prior round (V5→V7) fixed IRQ races
// site-by-site, and every round introduced new sites. The root cause
// is that the kernel has no systematic critical-section primitive, so
// every multi-step state transition is at the mercy of ad-hoc
// masking (or lack thereof).
//
// This module provides:
//
//   IrqGuard       — RAII type that masks DAIF.I on construct,
//                    restores previous DAIF on drop. Nestable.
//
//   critical_section! { ... }
//                  — macro that wraps a block in an IrqGuard scope.
//
//   FORBIDDEN inside a critical section:
//     - threads::schedule()
//     - wfi / wfe
//     - any loop that spins on external state (uart::getc, network recv)
//     - user-space copy_{from,to}_user for more than a few bytes
//       (because it can page-fault into the exception handler)
//     - calls that may themselves take a long lock (prefer to release
//       the guard, take the lock, operate, re-enter the guard if needed)
//
// Usage:
//
//   use crate::kernel::sync::critical_section;
//
//   critical_section! {
//       // Multi-step state transition. Timer IRQ / preempt cannot fire
//       // between these stores.
//       GLOBAL_A.store(new_a, Release);
//       GLOBAL_B.store(new_b, Release);
//   }
//
// Or manually:
//
//   let _g = crate::kernel::sync::IrqGuard::new();
//   // ... critical code ...
//   // _g dropped at scope end, DAIF restored.

#![allow(dead_code)]

use core::marker::PhantomData;

/// RAII guard that masks IRQ (DAIF.I) on construction and restores the
/// prior DAIF state on drop. Nestable — the saved DAIF is per-guard.
///
/// The `PhantomData<*const ()>` ensures !Send + !Sync so the guard can't
/// cross threads. (There's only one thread at a time on the kernel
/// stack, but this prevents accidental misuse once SMP lands.)
pub struct IrqGuard {
    prev_daif: u64,
    _not_send: PhantomData<*const ()>,
}

impl IrqGuard {
    /// Save DAIF and mask IRQ. Equivalent to:
    ///     prev = DAIF; DAIF.I = 1;
    #[inline(always)]
    pub fn new() -> Self {
        let prev_daif: u64;
        unsafe {
            core::arch::asm!(
                "mrs {p}, daif",
                "msr daifset, #0x2",   // mask IRQ bit
                p = out(reg) prev_daif,
                options(nostack, preserves_flags),
            );
        }
        IrqGuard { prev_daif, _not_send: PhantomData }
    }
}

impl Drop for IrqGuard {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            core::arch::asm!(
                "msr daif, {p}",
                p = in(reg) self.prev_daif,
                options(nostack, preserves_flags),
            );
        }
    }
}

/// Wrap a block in a critical section (IRQ masked from entry to exit).
///
/// Returns the block's value, so you can write:
///
///     let x = critical_section! { some_computation() };
///
/// The guard is named `__irq_guard` inside the macro; do NOT rely on
/// that name from the caller.
#[macro_export]
macro_rules! critical_section {
    ($($body:tt)*) => {{
        let __irq_guard = $crate::kernel::sync::IrqGuard::new();
        let __cs_result = { $($body)* };
        drop(__irq_guard);
        __cs_result
    }};
}
