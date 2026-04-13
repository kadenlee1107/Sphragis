// Bat_OS — ICMP (Ping)
// Handles ICMP echo requests (replies to pings) and sends pings.

use super::ip::{self, IpPacket};
use crate::drivers::uart;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_ECHO_REQUEST: u8 = 8;

static PING_RECEIVED: AtomicBool = AtomicBool::new(false);
static PING_SEQ: AtomicU16 = AtomicU16::new(0);

pub fn handle(pkt: &IpPacket) {
    if pkt.payload.len() < 8 { return; }

    let icmp_type = pkt.payload[0];

    match icmp_type {
        ICMP_ECHO_REQUEST => {
            // Reply to ping
            let mut reply = [0u8; 1400];
            let len = pkt.payload.len().min(1400);
            reply[..len].copy_from_slice(&pkt.payload[..len]);
            reply[0] = ICMP_ECHO_REPLY; // Change type to reply
            reply[2] = 0; reply[3] = 0; // Clear checksum
            let cksum = ip::checksum(&reply[..len]);
            reply[2..4].copy_from_slice(&cksum.to_be_bytes());
            let _ = ip::send(pkt.src, 1, &reply[..len]);
        }
        ICMP_ECHO_REPLY => {
            let seq = u16::from_be_bytes([pkt.payload[6], pkt.payload[7]]);
            PING_SEQ.store(seq, Ordering::Relaxed);
            PING_RECEIVED.store(true, Ordering::Release);
            crate::drivers::uart::puts("[icmp] ping reply received!\n");
        }
        _ => {}
    }
}

/// Send a ping and wait for reply. Returns round-trip info.
pub fn ping(dst_ip: u32) -> Result<u16, &'static str> {
    PING_RECEIVED.store(false, Ordering::Relaxed);

    let seq = PING_SEQ.load(Ordering::Relaxed).wrapping_add(1);
    let mut icmp = [0u8; 64];
    icmp[0] = ICMP_ECHO_REQUEST;
    icmp[1] = 0; // Code
    // Checksum at [2..4]
    icmp[4..6].copy_from_slice(&0x4241u16.to_be_bytes()); // ID = "BA"
    icmp[6..8].copy_from_slice(&seq.to_be_bytes());

    // Payload: "BAT_OS_PING"
    let msg = b"BAT_OS_PING";
    icmp[8..8 + msg.len()].copy_from_slice(msg);

    let total = 8 + msg.len();
    let cksum = ip::checksum(&icmp[..total]);
    icmp[2..4].copy_from_slice(&cksum.to_be_bytes());

    ip::send(dst_ip, 1, &icmp[..total])?;

    // Wait for reply
    for _ in 0..20_000_000 {
        super::poll_once();
        if PING_RECEIVED.load(Ordering::Relaxed) {
            return Ok(PING_SEQ.load(Ordering::Relaxed));
        }
    }

    Err("ping timeout")
}
