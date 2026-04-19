#![allow(dead_code)]
// Bat_OS — Apple Watchdog driver.
//
// Simple port of external/m1n1/src/wdt.c. Apple SoCs expose the
// system watchdog via three 32-bit registers at the WDT block:
//
//   0x10  WDT_COUNT  — current count (writes reset it)
//   0x14  WDT_ALARM  — target count that triggers a reset
//   0x1c  WDT_CTL    — control: writing 0 disables, 4 enables for reset
//
// On M4 the block lives at 0x3_882b_0000 (observed by m1n1 at
// `WDT registers @ 0x3882b0000`). m1n1 disables it before chainload,
// but something re-enables the iBoot-side watchdog every ~30-60 s
// and kicks the Mac back into iBoot. Calling `disable()` from our
// own bring-up is defensive: at worst a no-op, at best prevents the
// spontaneous resets we've been chasing.

use super::soc;

const WDT_COUNT: usize = 0x10;
const WDT_ALARM: usize = 0x14;
const WDT_CTL:   usize = 0x1c;

#[inline(always)]
fn write32(off: usize, val: u32) {
    unsafe {
        core::ptr::write_volatile(
            (soc::wdt_base() + off) as *mut u32,
            val,
        );
    }
}

/// Disable the Apple watchdog. Safe to call unconditionally — the
/// fallback WDT base is hard-coded to the M4-observed value, so even
/// if the ADT discovery didn't populate `/arm-io/wdt` we still hit
/// the right register block.
pub fn disable() {
    write32(WDT_CTL, 0);
    unsafe { core::arch::asm!("dsb sy"); }
}
