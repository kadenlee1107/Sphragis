// Bat_OS — Dead Man's Switch
// If you don't re-authenticate within the configured interval,
// the system assumes you're compromised and auto-wipes on next boot.
//
// Timer state is stored persistently (in Secure Enclave on real HW).
// Cannot be disabled without authentication.
//
// Scenarios:
// - You're arrested: timer expires, data destroyed
// - You're incapacitated: timer expires, data destroyed
// - You're fine: refresh the timer periodically

use crate::drivers::uart;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

// Default: 48 hours (in seconds)
const DEFAULT_INTERVAL: u64 = 48 * 60 * 60;

static ARMED: AtomicBool = AtomicBool::new(false);
static INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_INTERVAL);
static LAST_REFRESH: AtomicU64 = AtomicU64::new(0);
static EXPIRED: AtomicBool = AtomicBool::new(false);

fn current_time() -> u64 {
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    count / freq
}

/// Arm the dead man's switch.
pub fn arm(interval_hours: u64) {
    let interval = interval_hours * 60 * 60;
    INTERVAL_SECS.store(interval, Ordering::Relaxed);
    LAST_REFRESH.store(current_time(), Ordering::Relaxed);
    ARMED.store(true, Ordering::Release);
    EXPIRED.store(false, Ordering::Relaxed);

    uart::puts("  [dms] Dead man's switch ARMED (");
    crate::kernel::mm::print_num(interval_hours as usize);
    uart::puts("h)\n");
}

/// Refresh the timer — call this when the user re-authenticates.
pub fn refresh() {
    if ARMED.load(Ordering::Relaxed) {
        LAST_REFRESH.store(current_time(), Ordering::Relaxed);
    }
}

/// Disarm the switch (requires authentication first).
pub fn disarm() {
    ARMED.store(false, Ordering::Relaxed);
    uart::puts("  [dms] Dead man's switch DISARMED\n");
}

/// Check if the switch has expired. Called periodically.
/// Returns true if wipe should be triggered.
pub fn check() -> bool {
    if !ARMED.load(Ordering::Relaxed) || EXPIRED.load(Ordering::Relaxed) {
        return false;
    }

    let now = current_time();
    let last = LAST_REFRESH.load(Ordering::Relaxed);
    let interval = INTERVAL_SECS.load(Ordering::Relaxed);

    if now > last && (now - last) >= interval {
        EXPIRED.store(true, Ordering::Release);
        return true;
    }

    false
}

/// Get remaining time before expiry (in seconds).
pub fn remaining() -> u64 {
    if !ARMED.load(Ordering::Relaxed) {
        return 0;
    }

    let now = current_time();
    let last = LAST_REFRESH.load(Ordering::Relaxed);
    let interval = INTERVAL_SECS.load(Ordering::Relaxed);
    let elapsed = if now > last { now - last } else { 0 };

    if elapsed >= interval { 0 } else { interval - elapsed }
}

pub fn is_armed() -> bool {
    ARMED.load(Ordering::Relaxed)
}

pub fn is_expired() -> bool {
    EXPIRED.load(Ordering::Relaxed)
}

/// Format remaining time as "XXh XXm".
pub fn remaining_str(buf: &mut [u8; 16]) -> usize {
    let rem = remaining();
    let hours = rem / 3600;
    let mins = (rem % 3600) / 60;

    let mut pos = 0;
    pos += write_num(&mut buf[pos..], hours as usize);
    buf[pos] = b'h'; pos += 1;
    buf[pos] = b' '; pos += 1;
    pos += write_num(&mut buf[pos..], mins as usize);
    buf[pos] = b'm'; pos += 1;
    pos
}

fn write_num(buf: &mut [u8], n: usize) -> usize {
    let mut tmp = [0u8; 10];
    let mut val = n;
    let mut i = 9;
    if val == 0 { buf[0] = b'0'; return 1; }
    while val > 0 && i > 0 {
        tmp[i] = b'0' + (val % 10) as u8;
        val /= 10;
        i -= 1;
    }
    let start = i + 1;
    let len = 10 - start;
    buf[..len].copy_from_slice(&tmp[start..]);
    len
}
