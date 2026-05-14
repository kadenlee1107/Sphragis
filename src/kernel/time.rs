//! Sphragis — wall-clock + monotonic time.
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

/// Sync wall clock against the authoritative `Date:` header of a
/// pinned HTTPS server. Skips NTP entirely — the entire trust path
/// rides on our existing PQ-TLS chain validation, which is what we
/// already trust for cert checks. No DDoS-amp vector (TCP only,
/// authenticated handshake), no need for a separate time-server
/// keyset, and the resulting clock has the same provenance as
/// everything else our HTTPS stack does.
///
/// Returns the new Unix-seconds anchor on success.
pub fn sync_from_https(host: &str) -> Result<u64, &'static str> {
    use crate::net::https;
    use crate::drivers::uart;

    uart::puts("  [time] sync-from-https: opening ");
    uart::puts(host);
    uart::puts(":443 ...\n");

    let pcb = https::open_kernel(host, 443)?;

    // Build a minimal HEAD request. We only need the response
    // headers; the body is irrelevant.
    let mut req = [0u8; 256];
    let mut p = 0;
    let head = b"HEAD / HTTP/1.1\r\nHost: ";
    req[p..p + head.len()].copy_from_slice(head); p += head.len();
    let hb = host.as_bytes();
    let hn = hb.len().min(req.len() - p - 32);
    req[p..p + hn].copy_from_slice(&hb[..hn]); p += hn;
    let tail = b"\r\nConnection: close\r\nUser-Agent: sphragis/0.5\r\n\r\n";
    req[p..p + tail.len()].copy_from_slice(tail); p += tail.len();

    if let Err(e) = https::write(pcb, &req[..p]) {
        https::close_pcb(pcb);
        return Err(e);
    }

    // Read a small slice of the response — Date: lands in the
    // first few hundred bytes for every server I've tested. 2 KiB
    // is generous and bounds memory use.
    let mut resp = [0u8; 2048];
    let n = match https::read(pcb, &mut resp) {
        Ok(n) => n,
        Err(e) => { https::close_pcb(pcb); return Err(e); }
    };
    https::close_pcb(pcb);

    let secs = scan_date_header(&resp[..n])
        .ok_or("time: no usable Date: header in response")?;

    set_realtime_anchor(secs);
    uart::puts("  [time] sync OK from https: ");
    print_unix_human(secs, 0);
    uart::puts(" UTC\n");
    Ok(secs)
}

/// Scan an HTTP response buffer for a case-insensitive `Date:`
/// header and parse its IMF-fixdate value.
fn scan_date_header(buf: &[u8]) -> Option<u64> {
    let mut i = 0;
    while i + 6 < buf.len() {
        // Match start-of-line "Date:" case-insensitively. Header
        // lines are separated by CRLF; the very first line is the
        // status line, then headers follow.
        let lower_match = (buf[i] == b'd' || buf[i] == b'D')
            && (buf[i + 1] == b'a' || buf[i + 1] == b'A')
            && (buf[i + 2] == b't' || buf[i + 2] == b'T')
            && (buf[i + 3] == b'e' || buf[i + 3] == b'E')
            && buf[i + 4] == b':';
        let at_line_start = i == 0
            || (i >= 2 && buf[i - 2] == b'\r' && buf[i - 1] == b'\n');

        if lower_match && at_line_start {
            // Walk forward over optional leading whitespace.
            let mut j = i + 5;
            while j < buf.len() && (buf[j] == b' ' || buf[j] == b'\t') { j += 1; }
            // Find CRLF terminating the header value.
            let mut k = j;
            while k + 1 < buf.len() && !(buf[k] == b'\r' && buf[k + 1] == b'\n') {
                k += 1;
            }
            return parse_imf_fixdate(&buf[j..k]);
        }
        i += 1;
    }
    None
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

/// Inverse of `civil_from_days` — Howard Hinnant's `days_from_civil`.
/// Returns days-since-1970 for the given Gregorian (year, month, day).
fn days_from_civil(y: i64, m: u8, d: u8) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64; // [0, 399]
    let mp = if m > 2 { (m as u64) - 3 } else { (m as u64) + 9 };
    let doy = (153 * mp + 2) / 5 + (d as u64) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe as i64 - 719_468
}

/// Parse an HTTP IMF-fixdate header value (RFC 7231 §7.1.1.1):
/// "Sun, 06 Nov 1994 08:49:37 GMT". Returns Unix seconds, or None
/// on any parse failure. Strict — RFC 850 / asctime variants are
/// rejected (they're SHOULD-NOT in HTTP/1.1 and modern servers
/// don't emit them).
pub fn parse_imf_fixdate(s: &[u8]) -> Option<u64> {
    // Expect length 29: "Sun, 06 Nov 1994 08:49:37 GMT"
    //                    0    5  8   12   17 20 23
    if s.len() < 29 { return None; }
    let day  = parse_u2(&s[5..7])? as u8;
    let mon  = parse_month(&s[8..11])?;
    let year = parse_u4(&s[12..16])? as i64;
    if s[16] != b' ' { return None; }
    let hh = parse_u2(&s[17..19])? as u32;
    if s[19] != b':' { return None; }
    let mm = parse_u2(&s[20..22])? as u32;
    if s[22] != b':' { return None; }
    let ss = parse_u2(&s[23..25])? as u32;
    if &s[25..29] != b" GMT" { return None; }

    if mon == 0 || mon > 12 || day == 0 || day > 31 { return None; }
    if hh >= 24 || mm >= 60 || ss >= 60 { return None; }

    let days = days_from_civil(year, mon, day);
    if days < 0 { return None; } // pre-1970
    let secs = (days as u64) * SECS_PER_DAY
        + (hh as u64) * 3600
        + (mm as u64) * 60
        + (ss as u64);
    Some(secs)
}

fn parse_u2(s: &[u8]) -> Option<u32> {
    if s.len() != 2 { return None; }
    let d0 = if (b'0'..=b'9').contains(&s[0]) { (s[0] - b'0') as u32 } else { return None; };
    let d1 = if (b'0'..=b'9').contains(&s[1]) { (s[1] - b'0') as u32 } else { return None; };
    Some(d0 * 10 + d1)
}

fn parse_u4(s: &[u8]) -> Option<u32> {
    if s.len() != 4 { return None; }
    let mut v = 0u32;
    for &b in s {
        if !(b'0'..=b'9').contains(&b) { return None; }
        v = v * 10 + (b - b'0') as u32;
    }
    Some(v)
}

fn parse_month(s: &[u8]) -> Option<u8> {
    Some(match s {
        b"Jan" => 1,  b"Feb" => 2,  b"Mar" => 3,  b"Apr" => 4,
        b"May" => 5,  b"Jun" => 6,  b"Jul" => 7,  b"Aug" => 8,
        b"Sep" => 9,  b"Oct" => 10, b"Nov" => 11, b"Dec" => 12,
        _ => return None,
    })
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
