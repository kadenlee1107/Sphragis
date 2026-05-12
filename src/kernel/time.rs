//! Bat_OS — wall-clock + monotonic time.
//!
//! The ARM generic timer (cntpct_el0 + cntfrq_el0) gives us a
//! monotonically increasing tick count from CPU reset, but no
//! relation to civil time. This module pairs that with a one-shot
//! RTC read at boot to anchor a Unix-epoch offset, then computes
//! realtime via the offset + elapsed ticks. Past boot we do not
//! touch the RTC chip again — the generic timer is more accurate
//! over short intervals and avoids re-entering the SMC/AOP every
//! call.
//!
//! Time is exposed as Unix seconds + nanoseconds (epoch
//! 1970-01-01 00:00:00 UTC). Everything is u64; we are safe past
//! the year 292,000,000,000.
//!
//! Why no NTP today: plaintext NTP is a DDoS-amp risk and not
//! authenticated. The future path is to sync against the
//! authoritative `Date:` header returned by our existing
//! kernel-mediated HTTPS path (we already pin chains + do PQ TLS),
//! so the time source inherits the same trust roots as everything
//! else. Until that lands, the RTC read is the only sync point.

#![allow(dead_code)]

use core::sync::atomic::{AtomicI32, AtomicU64, Ordering};

/// Unix seconds at the moment of boot anchor. Zero until init.
static BOOT_REALTIME_SECS: AtomicU64 = AtomicU64::new(0);
/// cntpct_el0 reading taken at the same instant.
static BOOT_TICK: AtomicU64 = AtomicU64::new(0);
/// cntfrq_el0 — generic-timer frequency in Hz. Read once at init.
static FREQ_HZ: AtomicU64 = AtomicU64::new(0);
/// Local time-zone offset in seconds east of UTC. Defaults to 0
/// (UTC). Set via `set_tz_offset_secs()`. Item 029 from the gap
/// audit reduces to this single knob once wall clock exists.
static TZ_OFFSET_SECS: AtomicI32 = AtomicI32::new(0);
/// True once `init_*` succeeded — calls before init return the
/// "epoch 0" fallback so audit timestamps don't lie about being
/// real-time.
static SYNCED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[inline]
fn read_cntpct() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("isb; mrs {}, cntpct_el0", out(reg) v); }
    v
}

#[inline]
fn read_cntfrq() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v); }
    v
}

/// Monotonic microseconds since boot (or rather, since the generic
/// timer started counting — usually CPU reset). Cheap, never blocks,
/// never goes backwards. Safe to call before `init_*`.
pub fn monotonic_us() -> u64 {
    let f = FREQ_HZ.load(Ordering::Relaxed);
    let freq = if f == 0 { read_cntfrq() } else { f };
    if freq == 0 {
        return 0;
    }
    // ticks * 1_000_000 / freq. Done in u128 to avoid overflow on
    // any realistic uptime; the result fits in u64 for >584,000 years.
    let ticks = read_cntpct() as u128;
    ((ticks * 1_000_000) / freq as u128) as u64
}

pub fn monotonic_secs() -> u64 {
    let f = FREQ_HZ.load(Ordering::Relaxed);
    let freq = if f == 0 { read_cntfrq() } else { f };
    if freq == 0 { 0 } else { read_cntpct() / freq }
}

/// Wall-clock seconds since Unix epoch. Returns 0 if init hasn't
/// run — callers can treat 0 as "no real time available." Once
/// synced, this is monotonic against the realtime clock too: there
/// is no slewing or stepping past boot.
pub fn realtime_secs() -> u64 {
    if !SYNCED.load(Ordering::Relaxed) {
        return 0;
    }
    let boot_secs = BOOT_REALTIME_SECS.load(Ordering::Relaxed);
    let boot_tick = BOOT_TICK.load(Ordering::Relaxed);
    let freq = FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return boot_secs;
    }
    let now = read_cntpct();
    let elapsed_ticks = now.saturating_sub(boot_tick);
    boot_secs + elapsed_ticks / freq
}

pub fn realtime_us() -> u64 {
    if !SYNCED.load(Ordering::Relaxed) {
        return 0;
    }
    let boot_secs = BOOT_REALTIME_SECS.load(Ordering::Relaxed);
    let boot_tick = BOOT_TICK.load(Ordering::Relaxed);
    let freq = FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return boot_secs.saturating_mul(1_000_000);
    }
    let now = read_cntpct();
    let elapsed_ticks = (now.saturating_sub(boot_tick)) as u128;
    let elapsed_us = (elapsed_ticks * 1_000_000) / freq as u128;
    (boot_secs as u128 * 1_000_000 + elapsed_us) as u64
}

pub fn is_synced() -> bool {
    SYNCED.load(Ordering::Relaxed)
}

pub fn tz_offset_secs() -> i32 {
    TZ_OFFSET_SECS.load(Ordering::Relaxed)
}

pub fn set_tz_offset_secs(offset: i32) {
    // Sanity gate: ±14h covers every real-world timezone.
    if offset.abs() <= 14 * 3600 {
        TZ_OFFSET_SECS.store(offset, Ordering::Relaxed);
    }
}

/// Anchor the wall clock using a freshly-read Unix-seconds value.
/// Called once at boot from whichever RTC backend is appropriate
/// for the platform. Captures the cntpct snapshot in the same
/// breath so the offset is consistent.
pub fn set_realtime_anchor(unix_secs: u64) {
    // Snapshot in a fixed order: freq first, then tick, then
    // commit secs + flip SYNCED. Concurrent readers either see
    // SYNCED=false (return 0) or SYNCED=true with all three values
    // already populated — no torn anchor.
    let freq = read_cntfrq();
    if freq == 0 {
        // Catastrophic: timer not running. Leave SYNCED off so
        // callers know not to trust realtime_*.
        return;
    }
    FREQ_HZ.store(freq, Ordering::Relaxed);
    BOOT_TICK.store(read_cntpct(), Ordering::Relaxed);
    BOOT_REALTIME_SECS.store(unix_secs, Ordering::Relaxed);
    SYNCED.store(true, Ordering::Release);
}

/// Seed from the QEMU virt PL031 RTC.
pub fn init_from_pl031() {
    if let Some(secs) = crate::drivers::rtc::read_pl031() {
        set_realtime_anchor(secs);
        log_sync("pl031", secs);
    } else {
        log_failed("pl031");
    }
}

/// Seed from the Apple-side RTC (stub today). Logs that no real
/// source was available; monotonic still works.
pub fn init_from_apple() {
    if let Some(secs) = crate::drivers::rtc::read_apple() {
        set_realtime_anchor(secs);
        log_sync("apple", secs);
    } else {
        log_failed("apple");
    }
}

fn log_sync(src: &str, secs: u64) {
    use crate::drivers::uart;
    uart::puts("  [time] anchored from ");
    uart::puts(src);
    uart::puts(": ");
    print_unix_human(secs, 0);
    uart::puts(" UTC\n");
}

fn log_failed(src: &str) {
    use crate::drivers::uart;
    uart::puts("  [time] ");
    uart::puts(src);
    uart::puts(" returned no time — monotonic only, realtime calls return 0\n");
}

// ---------- Unix-epoch → civil-time conversion ----------
//
// Howard Hinnant's days-from-civil algorithm, in u64. Public domain.
// Used by `format_unix_*` and the `date` shell command. No tzdata —
// just the static TZ_OFFSET_SECS knob.

const SECS_PER_DAY: u64 = 86_400;

/// Convert Unix seconds (UTC) to (year, month, day, hour, min, sec).
/// Years are CE; valid for years 1970..262143 give or take.
pub fn split_unix(unix_secs: u64) -> (u32, u8, u8, u8, u8, u8) {
    let days = (unix_secs / SECS_PER_DAY) as i64;
    let secs_of_day = (unix_secs % SECS_PER_DAY) as u32;
    let (y, m, d) = civil_from_days(days);
    let h = (secs_of_day / 3600) as u8;
    let mm = ((secs_of_day / 60) % 60) as u8;
    let s = (secs_of_day % 60) as u8;
    (y as u32, m, d, h, mm, s)
}

/// Days-since-1970 → (year, month, day). Howard Hinnant's algorithm,
/// faithful to the published version.
fn civil_from_days(days: i64) -> (i64, u8, u8) {
    let z = days + 719_468;
    let era = if z >= 0 { z / 146_097 } else { (z - 146_096) / 146_097 };
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Print a Unix-secs timestamp as `YYYY-MM-DD HH:MM:SS` over UART.
/// `tz_offset` is added (in seconds) before splitting — pass 0 for
/// UTC, `tz_offset_secs()` for local.
pub fn print_unix_human(unix_secs: u64, tz_offset: i32) {
    use crate::drivers::uart;
    let adjusted = if tz_offset >= 0 {
        unix_secs.saturating_add(tz_offset as u64)
    } else {
        unix_secs.saturating_sub((-tz_offset) as u64)
    };
    let (y, m, d, h, mm, s) = split_unix(adjusted);
    let mut buf = [0u8; 20];
    let n = format_human(&mut buf, y, m, d, h, mm, s);
    for &b in &buf[..n] {
        uart::putc(b);
    }
}

/// Format `YYYY-MM-DD HH:MM:SS` into `buf`. Returns bytes written
/// (always 19 for valid years). buf must be at least 19 bytes.
pub fn format_human(buf: &mut [u8], y: u32, m: u8, d: u8, h: u8, mm: u8, s: u8) -> usize {
    if buf.len() < 19 { return 0; }
    let y = y.min(9999);
    buf[0] = b'0' + ((y / 1000) % 10) as u8;
    buf[1] = b'0' + ((y / 100) % 10) as u8;
    buf[2] = b'0' + ((y / 10) % 10) as u8;
    buf[3] = b'0' + (y % 10) as u8;
    buf[4] = b'-';
    buf[5] = b'0' + (m / 10);
    buf[6] = b'0' + (m % 10);
    buf[7] = b'-';
    buf[8] = b'0' + (d / 10);
    buf[9] = b'0' + (d % 10);
    buf[10] = b' ';
    buf[11] = b'0' + (h / 10);
    buf[12] = b'0' + (h % 10);
    buf[13] = b':';
    buf[14] = b'0' + (mm / 10);
    buf[15] = b'0' + (mm % 10);
    buf[16] = b':';
    buf[17] = b'0' + (s / 10);
    buf[18] = b'0' + (s % 10);
    19
}
