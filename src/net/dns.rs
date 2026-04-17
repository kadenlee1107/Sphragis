#![allow(dead_code)]
#![allow(unused_assignments)]
// Bat_OS — DNS Resolver
// Supports both plaintext DNS (fallback) and DNS-over-HTTPS (secure).
// DoH sends DNS wire-format queries as HTTP POST to a DoH server over TCP.
// Secure pipeline: all DNS queries go through DoH when available.
//
// DoH flow: TCP connect → send HTTP POST with DNS query → parse HTTP response → extract DNS answer

use super::udp;
use core::sync::atomic::{AtomicU32, AtomicU16, AtomicBool, Ordering};

/// Rolling counter mixed with cntpct_el0 to produce a 16-bit TXID. This is
/// not cryptographic (we have no CSPRNG here) but it replaces the previous
/// hardcoded 0x4242 and defeats the single-packet Kaminsky-style spoof
/// described in ATTACK-NET-035.
static TXID_COUNTER: AtomicU16 = AtomicU16::new(0);

fn next_txid() -> u16 {
    let c = TXID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ticks: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) ticks); }
    // Fold high and low halves of the timer into the counter.
    let mix = (ticks as u16)
        ^ ((ticks >> 16) as u16)
        ^ ((ticks >> 32) as u16)
        ^ ((ticks >> 48) as u16);
    // XOR (not add) so the counter still contributes all 16 bits of entropy
    // and we never collide for two queries issued within one tick.
    c.wrapping_mul(0x9E37) ^ mix
}

/// TXID of the currently in-flight plaintext DNS query. `handle_response`
/// drops anything whose TXID doesn't match.
static EXPECTED_TXID: AtomicU16 = AtomicU16::new(0);
static TXID_VALID: AtomicBool = AtomicBool::new(false);

const DNS_SERVER: u32 = 0x0A000203; // 10.0.2.3 (QEMU) — plaintext fallback
const DNS_PORT: u16 = 53;
// ATTACK-NET-036: local port randomized per query. Forces an off-path
// attacker to guess both the TXID (16 bits) AND the dst_port (~14
// bits of ephemeral range), raising the per-packet spoof probability
// from 1-in-65k to ~1-in-1e9.
fn next_local_port() -> u16 {
    static LAST: AtomicU16 = AtomicU16::new(0);
    let mut v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v ^= (LAST.load(Ordering::Relaxed) as u64).wrapping_mul(0xa076_1d64_78bd_642f);
    // SplitMix finalizer — good distribution from one counter read.
    v ^= v >> 33;
    v = v.wrapping_mul(0xff51_afd7_ed55_8ccd);
    v ^= v >> 33;
    let port = (1024 + ((v as u16 as u32) % (65535 - 1024))) as u16;
    LAST.store(port, Ordering::Relaxed);
    port
}

// DoH server (Cloudflare) — used when DoH is enabled
const DOH_SERVER: u32 = 0x01010101; // 1.1.1.1
const DOH_PORT: u16 = 80; // Use HTTP (port 80) since TLS handshake needs more work
// When TLS is fully wired, switch to port 443

static DOH_ENABLED: AtomicBool = AtomicBool::new(false); // disabled until TLS handshake is complete

/// Enable or disable DNS-over-HTTPS.
pub fn set_doh(enabled: bool) {
    DOH_ENABLED.store(enabled, Ordering::Relaxed);
}

static RESOLVED_IP: AtomicU32 = AtomicU32::new(0);
static DNS_DONE: AtomicBool = AtomicBool::new(false);

/// Handle a DNS response.
pub fn handle_response(data: &[u8]) {
    if data.len() < 12 { return; }
    if data.len() > 4096 { return; } // RFC 1035 soft cap + sanity

    // ATTACK-NET-035: verify the transaction ID matches the query we sent.
    // An off-path attacker now has to guess a 16-bit TXID plus land within
    // the query window; still weak by modern standards but defeats the
    // trivial single-packet spoof.
    if TXID_VALID.load(Ordering::Acquire) {
        let txid = u16::from_be_bytes([data[0], data[1]]);
        if txid != EXPECTED_TXID.load(Ordering::Relaxed) {
            return;
        }
    } else {
        // No outstanding query — drop unsolicited response.
        return;
    }

    // Response bounds sanity (hardens against spoofed giant payloads
    // that would otherwise drag us through thousands of RR iterations).
    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let answers = u16::from_be_bytes([data[6], data[7]]);
    if qdcount != 1 { return; }   // we always send exactly one question
    if answers == 0 { return; }
    if answers > 32 { return; }   // RFC-valid but suspicious

    // V8-ROOT-3 / V8-ARITH-A4 / V8-PARSER-1: every offset += external_len
    // arithmetic uses checked_add and aborts the parse on overflow OR
    // out-of-bounds. The previous code used plain `+=` and `+` against
    // unchecked attacker-supplied lengths.
    //
    // Skip header (12 bytes) and question section
    let mut offset: usize = 12;

    // Skip question name with bounded label-length cap (255 per RFC 1035).
    let mut name_steps = 0usize;
    while offset < data.len() {
        let len = data[offset] as usize;
        if len == 0 {
            offset = match offset.checked_add(1) { Some(o) => o, None => return };
            break;
        }
        if len > 63 { return; } // RFC 1035 §2.3.4: label max 63 octets
        offset = match offset.checked_add(len + 1) { Some(o) => o, None => return };
        name_steps += 1;
        if name_steps > 128 { return; } // walk-too-long guard
    }
    offset = match offset.checked_add(4) { Some(o) => o, None => return };

    let max_answers = (answers as usize).min(16);
    for _ in 0..max_answers {
        if offset.checked_add(2).map_or(true, |e| e > data.len()) { return; }

        if data[offset] & 0xC0 == 0xC0 {
            offset = match offset.checked_add(2) { Some(o) => o, None => return };
        } else {
            let mut label_steps = 0usize;
            while offset < data.len() && data[offset] != 0 {
                let label_len = data[offset] as usize;
                if label_len > 63 { return; }
                offset = match offset.checked_add(label_len + 1) { Some(o) => o, None => return };
                label_steps += 1;
                if label_steps > 128 { return; }
            }
            if offset < data.len() {
                offset = match offset.checked_add(1) { Some(o) => o, None => return };
            }
        }

        if offset.checked_add(10).map_or(true, |e| e > data.len()) { return; }

        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let rdlen = u16::from_be_bytes([data[offset + 8], data[offset + 9]]) as usize;
        if rdlen > 512 { return; }
        offset = match offset.checked_add(10) { Some(o) => o, None => return };

        if rtype == 1 && rdlen == 4
            && offset.checked_add(4).map_or(false, |e| e <= data.len())
        {
            let ip = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            RESOLVED_IP.store(ip, Ordering::Relaxed);
            DNS_DONE.store(true, Ordering::Release);
            return;
        }

        if offset.checked_add(rdlen).map_or(true, |e| e > data.len()) { return; }
        offset = match offset.checked_add(rdlen) { Some(o) => o, None => return };
    }
}

/// Resolve a hostname to an IPv4 address.
/// Tries DNS-over-HTTPS first, falls back to plaintext UDP.
pub fn resolve(hostname: &str) -> Result<u32, &'static str> {
    // Try DoH first if enabled
    if DOH_ENABLED.load(Ordering::Relaxed) {
        DNS_DONE.store(false, Ordering::Relaxed);
        RESOLVED_IP.store(0, Ordering::Relaxed);
        if let Ok(ip) = resolve_doh(hostname) {
            return Ok(ip);
        }
        // DoH failed — fall through to plaintext
    }

    // Plaintext DNS fallback
    DNS_DONE.store(false, Ordering::Relaxed);
    RESOLVED_IP.store(0, Ordering::Relaxed);

    // Build DNS query
    let mut query = [0u8; 512];
    let mut offset = 0;

    // Header
    let txid = next_txid();
    EXPECTED_TXID.store(txid, Ordering::Relaxed);
    TXID_VALID.store(true, Ordering::Release);
    query[0..2].copy_from_slice(&txid.to_be_bytes());      // Transaction ID (randomized)
    query[2..4].copy_from_slice(&0x0100u16.to_be_bytes()); // Standard query, recursion desired
    query[4..6].copy_from_slice(&1u16.to_be_bytes());      // 1 question
    offset = 12;

    // Question: encode hostname
    for part in hostname.as_bytes().split(|&b| b == b'.') {
        query[offset] = part.len() as u8;
        offset += 1;
        query[offset..offset + part.len()].copy_from_slice(part);
        offset += part.len();
    }
    query[offset] = 0; // Null terminator
    offset += 1;

    query[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes()); // Type A
    offset += 2;
    query[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes()); // Class IN
    offset += 2;

    // Randomize source port per query (NET-036).
    let src_port = next_local_port();

    // Send query
    udp::send(DNS_SERVER, src_port, DNS_PORT, &query[..offset])?;

    // Wait for response — retry with increasing timeouts
    for attempt in 0..5 {
        if attempt > 0 {
            // Re-send query on each retry (same src_port for this attempt).
            let _ = udp::send(DNS_SERVER, src_port, DNS_PORT, &query[..offset]);
        }
        // Poll for response (50M iterations ≈ several seconds on fast CPUs)
        for _ in 0..50_000_000u64 {
            super::poll_once();
            if DNS_DONE.load(Ordering::Acquire) {
                let ip = RESOLVED_IP.load(Ordering::Relaxed);
                if ip != 0 {
                    TXID_VALID.store(false, Ordering::Release);
                    return Ok(ip);
                }
            }
            core::hint::spin_loop();
        }
    }

    TXID_VALID.store(false, Ordering::Release);
    Err("DNS timeout")
}

/// Resolve via DNS-over-HTTPS (sends DNS wire format over HTTP POST).
/// Falls back to plaintext if DoH fails.
fn resolve_doh(hostname: &str) -> Result<u32, &'static str> {
    // Build the DNS wire-format query (same as plaintext).
    // DoH runs over TCP so TXID spoofing is not the same threat, but we
    // still randomize it (no reason not to) and verify the response.
    let mut query = [0u8; 512];
    let txid = next_txid();
    EXPECTED_TXID.store(txid, Ordering::Relaxed);
    TXID_VALID.store(true, Ordering::Release);
    query[0..2].copy_from_slice(&txid.to_be_bytes());      // Transaction ID (randomized)
    query[2..4].copy_from_slice(&0x0100u16.to_be_bytes()); // Recursion desired
    query[4..6].copy_from_slice(&1u16.to_be_bytes());      // 1 question
    let mut qlen = 12;
    for part in hostname.as_bytes().split(|&b| b == b'.') {
        query[qlen] = part.len() as u8;
        qlen += 1;
        query[qlen..qlen + part.len()].copy_from_slice(part);
        qlen += part.len();
    }
    query[qlen] = 0; qlen += 1;
    query[qlen..qlen + 2].copy_from_slice(&1u16.to_be_bytes()); qlen += 2; // Type A
    query[qlen..qlen + 2].copy_from_slice(&1u16.to_be_bytes()); qlen += 2; // Class IN

    // TCP connect to DoH server
    if super::tcp::connect(DOH_SERVER, DOH_PORT).is_err() {
        return Err("DoH connect failed");
    }

    // Build HTTP POST request
    // POST /dns-query HTTP/1.1\r\nHost: 1.1.1.1\r\nContent-Type: application/dns-message\r\nContent-Length: NN\r\n\r\n<binary>
    let mut http = [0u8; 512];
    let header = b"POST /dns-query HTTP/1.1\r\nHost: 1.1.1.1\r\nContent-Type: application/dns-message\r\nAccept: application/dns-message\r\nContent-Length: ";
    let mut hlen = header.len();
    http[..hlen].copy_from_slice(header);

    // Content-Length as ASCII digits
    let mut digits = [0u8; 4];
    let mut dlen = 0;
    let mut v = qlen;
    if v == 0 { digits[0] = b'0'; dlen = 1; }
    else {
        while v > 0 { digits[dlen] = b'0' + (v % 10) as u8; dlen += 1; v /= 10; }
        // Reverse
        for i in 0..dlen / 2 { digits.swap(i, dlen - 1 - i); }
    }
    http[hlen..hlen + dlen].copy_from_slice(&digits[..dlen]);
    hlen += dlen;
    http[hlen..hlen + 4].copy_from_slice(b"\r\n\r\n");
    hlen += 4;

    // Append DNS query body
    http[hlen..hlen + qlen].copy_from_slice(&query[..qlen]);
    hlen += qlen;

    // Send HTTP request
    if super::tcp::send_data(&http[..hlen]).is_err() {
        super::tcp::close();
        return Err("DoH send failed");
    }

    // Receive HTTP response
    let mut resp = [0u8; 1024];
    match super::tcp::recv_data(&mut resp) {
        Ok(n) => {
            super::tcp::close();

            // Find the DNS response body after HTTP headers (\r\n\r\n)
            let mut body_start = 0;
            for i in 0..n.saturating_sub(3) {
                if resp[i] == b'\r' && resp[i+1] == b'\n' && resp[i+2] == b'\r' && resp[i+3] == b'\n' {
                    body_start = i + 4;
                    break;
                }
            }

            if body_start > 0 && body_start < n {
                // Parse DNS response from the body
                handle_response(&resp[body_start..n]);
                if DNS_DONE.load(Ordering::Acquire) {
                    let ip = RESOLVED_IP.load(Ordering::Relaxed);
                    if ip != 0 {
                        TXID_VALID.store(false, Ordering::Release);
                        return Ok(ip);
                    }
                }
            }
            TXID_VALID.store(false, Ordering::Release);
            Err("DoH: no answer in response")
        }
        Err(_) => {
            super::tcp::close();
            TXID_VALID.store(false, Ordering::Release);
            Err("DoH recv failed")
        }
    }
}
