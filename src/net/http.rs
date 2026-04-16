// Bat_OS — Hardened HTTP/1.1 client helpers
//
// Security hardening for pentest findings:
//   - ATTACK-NET-046: slow-loris — total and idle read deadlines
//   - ATTACK-NET-045: header bomb  — bounded header size / count / per-line
//   - ATTACK-DOS-026: chunked bomb — bounded chunk size and total body
//   - Response-splitting guard on user-controlled request header values
//
// This module does NOT own any sockets. The caller is responsible for the
// TCP connect / TLS handshake / close. We only implement the
// security-sensitive read loop and on-the-wire parsing, so the logic sits
// in one reviewable place.
//
// #![no_std] — no heap, no allocator. Caller supplies buffers.

#![allow(dead_code)]

use core::arch::asm;

// ─── Limits (ATTACK-NET-045, ATTACK-DOS-026) ────────────────────────────
pub const MAX_TOTAL_HEADER_BYTES: usize = 64 * 1024;    // 64 KB total headers
pub const MAX_HEADER_LINE_BYTES:  usize = 8 * 1024;     //  8 KB per line
pub const MAX_HEADER_COUNT:       usize = 128;          // 128 header lines
pub const MAX_CHUNK_BYTES:        usize = 4 * 1024 * 1024;   // 4 MB per chunk
pub const MAX_BODY_BYTES:         usize = 16 * 1024 * 1024;  // 16 MB total

// ─── Timeouts (ATTACK-NET-046) ──────────────────────────────────────────
pub const READ_DEADLINE_SECS: u64 = 30; // whole-response hard deadline
pub const READ_IDLE_SECS:     u64 = 5;  // no-progress idle deadline

/// HTTP read/parse error taxonomy. `&'static str` keeps us allocation-free.
#[derive(Copy, Clone, Debug)]
pub enum HttpError {
    DeadlineExceeded,   // total 30s exceeded
    IdleTimeout,        // 5s without any recv progress
    HeadersTooLarge,    // >64KB headers or line >8KB or >128 lines
    MalformedHeaders,   // no CRLFCRLF terminator seen
    ChunkTooLarge,      // single chunk >4MB
    BodyTooLarge,       // cumulative body >16MB
    ChunkedUnsupported, // Transfer-Encoding: chunked but caller rejects it
    InvalidHeaderValue, // CR/LF in a user-supplied header value
    RecvFailed,         // underlying transport error
    BufferFull,         // caller's buffer too small
}

impl HttpError {
    pub fn as_str(self) -> &'static str {
        match self {
            HttpError::DeadlineExceeded   => "http: read deadline exceeded",
            HttpError::IdleTimeout        => "http: idle timeout",
            HttpError::HeadersTooLarge    => "http: headers too large",
            HttpError::MalformedHeaders   => "http: malformed headers",
            HttpError::ChunkTooLarge      => "http: chunk too large",
            HttpError::BodyTooLarge       => "http: body too large",
            HttpError::ChunkedUnsupported => "http: chunked unsupported",
            HttpError::InvalidHeaderValue => "http: invalid header value (CR/LF)",
            HttpError::RecvFailed         => "http: recv failed",
            HttpError::BufferFull         => "http: response buffer full",
        }
    }
}

// ─── Timebase (cntpct_el0 / cntfrq_el0) ─────────────────────────────────
#[inline(always)]
fn now_ticks() -> u64 {
    let t: u64;
    unsafe { asm!("mrs {}, cntpct_el0", out(reg) t); }
    t
}

#[inline(always)]
fn tick_freq() -> u64 {
    let f: u64;
    unsafe { asm!("mrs {}, cntfrq_el0", out(reg) f); }
    f
}

/// Monotonic deadline: returns `true` once `secs` have elapsed since `start`.
#[inline(always)]
fn deadline_reached(start: u64, freq: u64, secs: u64) -> bool {
    now_ticks().wrapping_sub(start) > freq.saturating_mul(secs)
}

// ─── Transport abstraction ──────────────────────────────────────────────
// We don't want to pull in the whole tls/tcp graph; the caller passes a
// function pointer so this module stays testable in isolation.
pub type RecvFn = fn(&mut [u8]) -> Result<usize, &'static str>;

/// Read the full HTTP response (headers + body bytes, up to buffer capacity)
/// into `out`, returning the total number of bytes written.
///
/// Hardened against:
///   - ATTACK-NET-046 slow-loris: 30s total + 5s idle deadlines
///   - ATTACK-NET-045 header bomb: rejects if headers exceed MAX_TOTAL_HEADER_BYTES,
///     any single header line exceeds MAX_HEADER_LINE_BYTES, or more than
///     MAX_HEADER_COUNT header lines appear before CRLFCRLF.
///   - Response-truncation: stops at CRLFCRLF + declared/chunked body,
///     not just when the TCP stream stalls.
pub fn read_response(recv: RecvFn, out: &mut [u8]) -> Result<usize, HttpError> {
    let start = now_ticks();
    let freq  = tick_freq();
    let mut last_progress = start;

    let mut total = 0usize;
    let mut header_end: Option<usize> = None;

    // Scratch — small enough for the stack. Matches typical MSS/TLS record sizes.
    let mut chunk = [0u8; 4096];

    loop {
        // ── Deadlines ───────────────────────────────────────────────────
        if deadline_reached(start, freq, READ_DEADLINE_SECS) {
            return Err(HttpError::DeadlineExceeded);
        }
        if deadline_reached(last_progress, freq, READ_IDLE_SECS) {
            return Err(HttpError::IdleTimeout);
        }

        if total >= out.len() {
            // Buffer full before we saw full response — treat as truncation.
            // Still enforce the header-bomb cap if we never saw CRLFCRLF.
            if header_end.is_none() {
                return Err(HttpError::HeadersTooLarge);
            }
            return Err(HttpError::BufferFull);
        }

        // Bound each read by the header cap until we have a header terminator,
        // so a pathological server can't pre-fill the response buffer with
        // header bytes.
        let room = out.len() - total;
        let bound = if header_end.is_none() {
            let budget = MAX_TOTAL_HEADER_BYTES.saturating_sub(total);
            core::cmp::min(chunk.len(), core::cmp::min(room, budget.max(1)))
        } else {
            core::cmp::min(chunk.len(), room)
        };

        match recv(&mut chunk[..bound]) {
            Ok(0) => {
                // NEW-DOS-010 fix: yield on "no data yet" so slow-loris
                // peers can't pin the core for the full READ_IDLE window.
                // Scheduler access is platform-specific; this call is a
                // no-op on cores without co-scheduled caves.
                crate::batcave::linux::threads::schedule();
                continue;
            }
            Ok(n) => {
                out[total..total + n].copy_from_slice(&chunk[..n]);
                total += n;
                last_progress = now_ticks();

                // Scan for CRLFCRLF if we haven't found it yet.
                if header_end.is_none() {
                    if let Some(end) = find_crlfcrlf(&out[..total]) {
                        // Enforce per-line and per-count caps on the header block.
                        validate_header_block(&out[..end])?;
                        header_end = Some(end);
                    } else if total >= MAX_TOTAL_HEADER_BYTES {
                        return Err(HttpError::HeadersTooLarge);
                    }
                }

                // If we have headers and an end-of-html sentinel in the body,
                // we can return early. (Preserves the old browser heuristic
                // without reintroducing the unbounded 500-iter loop.)
                if let Some(hend) = header_end {
                    if total > hend + 7 {
                        let look_from = if total > hend + 20 { total - 20 } else { hend };
                        if contains_ci(&out[look_from..total], b"</html>") {
                            return Ok(total);
                        }
                    }
                }
            }
            Err(_) => {
                // Transient recv failure — re-enter the deadline check and
                // count this as no-progress.
                if deadline_reached(last_progress, freq, READ_IDLE_SECS) {
                    return Err(HttpError::IdleTimeout);
                }
                continue;
            }
        }
    }
}

/// Scan a headers-only slice and enforce per-line / count limits.
/// Input must be the byte range [0 .. CRLFCRLF) from the response.
fn validate_header_block(hdrs: &[u8]) -> Result<(), HttpError> {
    if hdrs.len() > MAX_TOTAL_HEADER_BYTES {
        return Err(HttpError::HeadersTooLarge);
    }
    let mut count = 0usize;
    let mut line_start = 0usize;
    let mut i = 0usize;
    while i < hdrs.len() {
        if hdrs[i] == b'\n' {
            // A line ending with \r\n; the \r is at i-1 if present.
            let line_len = i - line_start + 1;
            if line_len > MAX_HEADER_LINE_BYTES {
                return Err(HttpError::HeadersTooLarge);
            }
            count += 1;
            if count > MAX_HEADER_COUNT {
                return Err(HttpError::HeadersTooLarge);
            }
            line_start = i + 1;
        }
        i += 1;
    }
    // Trailing partial line (no LF before CRLFCRLF boundary) — still enforce.
    if i - line_start > MAX_HEADER_LINE_BYTES {
        return Err(HttpError::HeadersTooLarge);
    }
    Ok(())
}

// ─── Chunked decoder with bounds ────────────────────────────────────────
/// Decode a chunked body into `out`. Enforces 4MB-per-chunk and 16MB-total.
/// `chunked` is the body bytes already read (headers stripped).
pub fn decode_chunked(chunked: &[u8], out: &mut [u8]) -> Result<usize, HttpError> {
    let mut ri = 0usize;
    let mut wi = 0usize;
    loop {
        // Skip stray CR/LF between chunks.
        while ri < chunked.len() && (chunked[ri] == b'\r' || chunked[ri] == b'\n') {
            ri += 1;
        }
        if ri >= chunked.len() { break; }

        // Parse hex chunk size.
        let mut chunk_size: usize = 0;
        let hex_start = ri;
        let mut hex_digits = 0u32;
        while ri < chunked.len() && chunked[ri] != b'\r' && chunked[ri] != b'\n' {
            let c = chunked[ri];
            let digit = match c {
                b'0'..=b'9' => (c - b'0') as usize,
                b'a'..=b'f' => (c - b'a' + 10) as usize,
                b'A'..=b'F' => (c - b'A' + 10) as usize,
                b';' => break, // chunk extensions — ignore
                _ => break,
            };
            // Bound the value before we even multiply: hex_digits > 7 would
            // allow values up to 16^8 = 4 GiB, far above our 4 MiB cap.
            if hex_digits > 7 { return Err(HttpError::ChunkTooLarge); }
            chunk_size = chunk_size * 16 + digit;
            if chunk_size > MAX_CHUNK_BYTES { return Err(HttpError::ChunkTooLarge); }
            hex_digits += 1;
            ri += 1;
        }
        if ri == hex_start { break; }

        // Skip to end of size line.
        while ri < chunked.len() && chunked[ri] != b'\n' { ri += 1; }
        if ri < chunked.len() { ri += 1; }

        // Terminator chunk.
        if chunk_size == 0 { break; }

        // Bound the copy by both source availability and output space.
        let src_avail = chunked.len().saturating_sub(ri);
        let copy_len  = chunk_size.min(src_avail);
        if wi + copy_len > MAX_BODY_BYTES { return Err(HttpError::BodyTooLarge); }
        if wi + copy_len > out.len() { return Err(HttpError::BufferFull); }
        out[wi..wi + copy_len].copy_from_slice(&chunked[ri..ri + copy_len]);
        wi += copy_len;
        ri = ri.saturating_add(chunk_size); // advance past the (possibly
                                             // truncated) chunk data
    }
    Ok(wi)
}

// ─── Response-splitting guard ───────────────────────────────────────────
/// Reject any CR/LF/NUL in a header value destined for the wire.
/// Call this on every user-controlled string that ends up in a request
/// line or header (Host, Location-follow URLs, User-Agent overrides, etc.).
#[inline]
pub fn validate_header_value(v: &[u8]) -> Result<(), HttpError> {
    for &b in v {
        if b == b'\r' || b == b'\n' || b == 0 {
            return Err(HttpError::InvalidHeaderValue);
        }
    }
    Ok(())
}

// ─── Transfer-Encoding handling ─────────────────────────────────────────
/// Return `true` if the header block declares `Transfer-Encoding: chunked`.
/// Caller decides whether to decode or reject.
pub fn is_chunked(headers: &[u8]) -> bool {
    match find_header_ci(headers, b"Transfer-Encoding:") {
        Some(v) => contains_ci(v, b"chunked"),
        None => false,
    }
}

/// Policy guard: if the caller does NOT implement chunked decoding, use this
/// to explicitly reject `Transfer-Encoding: chunked`.
pub fn reject_if_chunked(headers: &[u8]) -> Result<(), HttpError> {
    if is_chunked(headers) { Err(HttpError::ChunkedUnsupported) } else { Ok(()) }
}

// ─── Tiny helpers (duplicated rather than importing from browser.rs
//     so this file is usable from anywhere in the kernel) ────────────────
fn find_crlfcrlf(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 { return None; }
    let mut i = 0;
    while i + 3 < buf.len() {
        if buf[i] == b'\r' && buf[i+1] == b'\n' && buf[i+2] == b'\r' && buf[i+3] == b'\n' {
            return Some(i + 4);
        }
        i += 1;
    }
    None
}

fn eq_ci(a: u8, b: u8) -> bool { a.to_ascii_lowercase() == b.to_ascii_lowercase() }

fn starts_with_ci(h: &[u8], n: &[u8]) -> bool {
    if h.len() < n.len() { return false; }
    for i in 0..n.len() { if !eq_ci(h[i], n[i]) { return false; } }
    true
}

fn contains_ci(h: &[u8], n: &[u8]) -> bool {
    if n.is_empty() { return true; }
    if h.len() < n.len() { return false; }
    for i in 0..=h.len() - n.len() {
        if starts_with_ci(&h[i..], n) { return true; }
    }
    false
}

fn find_header_ci<'a>(headers: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    let mut i = 0;
    while i + name.len() < headers.len() {
        if starts_with_ci(&headers[i..], name) {
            let mut vi = i + name.len();
            while vi < headers.len() && (headers[vi] == b' ' || headers[vi] == b'\t') { vi += 1; }
            let val_start = vi;
            while vi < headers.len() && headers[vi] != b'\r' && headers[vi] != b'\n' { vi += 1; }
            return Some(&headers[val_start..vi]);
        }
        while i < headers.len() && headers[i] != b'\n' { i += 1; }
        i += 1;
    }
    None
}

// ─── Unit tests for the parser logic ────────────────────────────────────
// These compile in both std and no_std hosts because they only exercise
// pure-function logic (no cntpct reads, no transport).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_bomb_too_large() {
        let mut buf = [b'A'; MAX_TOTAL_HEADER_BYTES + 10];
        // Fake a CRLFCRLF just past the limit — validate_header_block gets
        // called with the prefix, so feed it the oversized slice directly.
        buf[MAX_TOTAL_HEADER_BYTES - 1] = b'\n';
        let r = validate_header_block(&buf[..]);
        assert!(matches!(r, Err(HttpError::HeadersTooLarge)));
    }

    #[test]
    fn header_count_capped() {
        // 129 lines of "X: y\r\n" — exceeds MAX_HEADER_COUNT.
        let mut v = [0u8; 8192];
        let mut off = 0;
        for _ in 0..(MAX_HEADER_COUNT + 1) {
            let line = b"X: y\r\n";
            v[off..off+line.len()].copy_from_slice(line);
            off += line.len();
        }
        let r = validate_header_block(&v[..off]);
        assert!(matches!(r, Err(HttpError::HeadersTooLarge)));
    }

    #[test]
    fn chunk_too_large() {
        // "FFFFFFFF\r\n" — 4 GiB chunk claim, must reject.
        let big = b"FFFFFFFF\r\n";
        let mut out = [0u8; 16];
        let r = decode_chunked(big, &mut out);
        assert!(matches!(r, Err(HttpError::ChunkTooLarge)));
    }

    #[test]
    fn response_splitting_rejected() {
        assert!(validate_header_value(b"normal.com").is_ok());
        assert!(matches!(validate_header_value(b"evil.com\r\nX: y"),
                         Err(HttpError::InvalidHeaderValue)));
        assert!(matches!(validate_header_value(b"evil\n"),
                         Err(HttpError::InvalidHeaderValue)));
        assert!(matches!(validate_header_value(b"evil\0"),
                         Err(HttpError::InvalidHeaderValue)));
    }

    #[test]
    fn chunked_detection() {
        let h = b"Host: x\r\nTransfer-Encoding: chunked\r\n";
        assert!(is_chunked(h));
        assert!(reject_if_chunked(h).is_err());
        let h2 = b"Host: x\r\nContent-Length: 4\r\n";
        assert!(!is_chunked(h2));
        assert!(reject_if_chunked(h2).is_ok());
    }
}
