// Bat_OS — DNS Resolver
// Minimal DNS client for A record lookups.
// Uses QEMU's built-in DNS at 10.0.2.3.

use super::udp;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

const DNS_SERVER: u32 = 0x0A000203; // 10.0.2.3 (QEMU)
const DNS_PORT: u16 = 53;
const LOCAL_PORT: u16 = 12345;

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
pub fn resolve(hostname: &str) -> Result<u32, &'static str> {
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
