// Bat_OS — DNS Resolver
// Supports both plaintext DNS (fallback) and DNS-over-HTTPS (secure).
// DoH sends DNS wire-format queries as HTTP POST to a DoH server over TCP.
// Secure pipeline: all DNS queries go through DoH when available.
//
// DoH flow: TCP connect → send HTTP POST with DNS query → parse HTTP response → extract DNS answer

use super::udp;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

const DNS_SERVER: u32 = 0x0A000203; // 10.0.2.3 (QEMU) — plaintext fallback
const DNS_PORT: u16 = 53;
const LOCAL_PORT: u16 = 12345;

// DoH server (Cloudflare) — used when DoH is enabled
const DOH_SERVER: u32 = 0x01010101; // 1.1.1.1
const DOH_PORT: u16 = 80; // Use HTTP (port 80) since TLS handshake needs more work
// When TLS is fully wired, switch to port 443

static DOH_ENABLED: AtomicBool = AtomicBool::new(true);

/// Enable or disable DNS-over-HTTPS.
pub fn set_doh(enabled: bool) {
    DOH_ENABLED.store(enabled, Ordering::Relaxed);
}

static RESOLVED_IP: AtomicU32 = AtomicU32::new(0);
static DNS_DONE: AtomicBool = AtomicBool::new(false);

/// Handle a DNS response.
pub fn handle_response(data: &[u8]) {
    if data.len() < 12 { return; }

    let answers = u16::from_be_bytes([data[6], data[7]]);
    if answers == 0 { return; }

    // Skip header (12 bytes) and question section
    let mut offset = 12;

    // Skip question name
    while offset < data.len() {
        let len = data[offset] as usize;
        if len == 0 { offset += 1; break; }
        offset += len + 1;
    }
    offset += 4; // Skip QTYPE + QCLASS

    // Parse first answer
    if offset + 12 > data.len() { return; }

    // Skip name (might be compressed pointer)
    if data[offset] & 0xC0 == 0xC0 {
        offset += 2; // Compressed pointer
    } else {
        while offset < data.len() && data[offset] != 0 {
            offset += data[offset] as usize + 1;
        }
        offset += 1;
    }

    if offset + 10 > data.len() { return; }

    let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let rdlen = u16::from_be_bytes([data[offset + 8], data[offset + 9]]) as usize;
    offset += 10;

    if rtype == 1 && rdlen == 4 && offset + 4 <= data.len() {
        // A record
        let ip = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        RESOLVED_IP.store(ip, Ordering::Relaxed);
        DNS_DONE.store(true, Ordering::Release);
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
    query[0..2].copy_from_slice(&0x4242u16.to_be_bytes()); // Transaction ID
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

    // Send query
    udp::send(DNS_SERVER, LOCAL_PORT, DNS_PORT, &query[..offset])?;

    // Wait for response — retry multiple times
    for attempt in 0..3 {
        if attempt > 0 {
            // Re-send query
            let _ = udp::send(DNS_SERVER, LOCAL_PORT, DNS_PORT, &query[..offset]);
        }
        for _ in 0..10_000_000 {
            super::poll_once();
            if DNS_DONE.load(Ordering::Acquire) {
                let ip = RESOLVED_IP.load(Ordering::Relaxed);
                if ip != 0 {
                    return Ok(ip);
                }
            }
            core::hint::spin_loop();
        }
    }

    Err("DNS timeout")
}

/// Resolve via DNS-over-HTTPS (sends DNS wire format over HTTP POST).
/// Falls back to plaintext if DoH fails.
fn resolve_doh(hostname: &str) -> Result<u32, &'static str> {
    // Build the DNS wire-format query (same as plaintext)
    let mut query = [0u8; 512];
    query[0..2].copy_from_slice(&0x4243u16.to_be_bytes()); // Transaction ID
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
                    if ip != 0 { return Ok(ip); }
                }
            }
            Err("DoH: no answer in response")
        }
        Err(_) => {
            super::tcp::close();
            Err("DoH recv failed")
        }
    }
}
