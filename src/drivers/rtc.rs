//! Bat_OS — RTC driver.
//!
//! Real-time clock readout. Two backends:
//!   - QEMU virt: PL031 at 0x0901_0000 (standard ARM RTC IP block).
//!   - Apple M4 : RTC lives behind the SMC / AOP / SEP and is not
//!                directly MMIO-addressable from EL1. Returns `None`
//!                today; the time module falls back to the
//!                build-stamped epoch so kernel logic that needs a
//!                monotonic-vs-real distinction still works.
//!
//! No interrupt wiring. We read on demand at boot to seed the time
//! offset; subsequent `time::realtime_*` calls compute from the
//! generic ARM monotonic counter (cntpct_el0), so we are
//! self-correcting against tick drift over short intervals without
//! needing the RTC chip past boot.

#![allow(dead_code)]

const PL031_BASE: usize = 0x0901_0000;
const PL031_DR:   usize = 0x00; // current count (seconds since 1970)
const PL031_CR:   usize = 0x0C; // control: bit 0 = enable

/// QEMU PL031 read. RTC ticks at 1 Hz; the data register is u32
/// seconds since 1970-01-01 UTC. Returns Some(secs) if the chip is
/// enabled and the read looks sane (post-2020).
pub fn read_pl031() -> Option<u64> {
    unsafe {
        let cr = core::ptr::read_volatile((PL031_BASE + PL031_CR) as *const u32);
        if cr & 1 == 0 {
            // Not enabled. Try enabling and re-read once.
            core::ptr::write_volatile((PL031_BASE + PL031_CR) as *mut u32, 1);
        }
        let secs = core::ptr::read_volatile((PL031_BASE + PL031_DR) as *const u32) as u64;
        // Sanity gate: any read before 2020-01-01 (1_577_836_800) means
        // the chip isn't really there — we read open-bus zeros, or a
        // garbage residue. Bail.
        if secs < 1_577_836_800 || secs > 4_102_444_800 {
            return None;
        }
        Some(secs)
    }
}

/// Apple-side RTC. Not yet wired — the M4 RTC sits on the SMC and
/// requires SMC keypath access. Returns None so the time module
/// falls back to the build-time epoch.
pub fn read_apple() -> Option<u64> {
    None
}

// No autodispatch: blindly poking 0x0901_0000 on a real M4 would
// SError on the unmapped window. Each boot path calls its backend
// directly (QEMU → read_pl031, Apple → read_apple).
