// Sphragis — ICMP (Ping)
// Handles ICMP echo requests (replies to pings) and sends pings.

use super::ip::{self, IpPacket};
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicUsize, Ordering};

const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_ECHO_REQUEST: u8 = 8;

static PING_RECEIVED: AtomicBool = AtomicBool::new(false);
static PING_SEQ: AtomicU16 = AtomicU16::new(0);

// AUDIT-NET-F4 (2026-05-15): ICMP echo-reply rate limit. Token-bucket
// state: ECHO_REPLY_TOKENS holds the current number of replies we may
// emit this window; ECHO_REPLY_WINDOW_START is the monotonic-tick
// stamp at which the bucket was last refilled. We allow up to
// MAX_ECHO_REPLIES_PER_SEC bucket-full of replies per second. Bucket
// refills lazily on each receive.
const MAX_ECHO_REPLIES_PER_SEC: u32 = 100;
static ECHO_REPLY_TOKENS:       AtomicU32 = AtomicU32::new(MAX_ECHO_REPLIES_PER_SEC);
static ECHO_REPLY_WINDOW_START: AtomicU32 = AtomicU32::new(0);

/// Raw-socket delivery buffer. Populated whenever an ICMP packet arrives
/// so a SOCK_RAW reader (e.g. busybox ping) can pull back the bytes in a
/// Linux-compatible "IP header + ICMP payload" shape via `take_raw_reply`.
/// The synthesized IP header lets the user's parser (which expects a full
/// IPv4 datagram) walk the bytes without modification.
const RAW_BUF_CAP: usize = 1500;
static mut RAW_REPLY_BUF: [u8; RAW_BUF_CAP] = [0; RAW_BUF_CAP];
static RAW_REPLY_LEN: AtomicUsize = AtomicUsize::new(0);
static RAW_REPLY_SRC: AtomicU32 = AtomicU32::new(0);

pub fn handle(pkt: &IpPacket) {
    if pkt.payload.len() < 8 { return; }

    let icmp_type = pkt.payload[0];

    match icmp_type {
        ICMP_ECHO_REQUEST => {
            // AUDIT-NET-F4 (2026-05-15): source-address sanity.
            // Reject echo requests from non-unicast sources (zero,
            // broadcast, multicast, or our own IP). Combined with
            // Net-F3's dst-IP filter, this closes the smurf-amplifier
            // surface: an attacker can no longer trick us into
            // reflecting echoes to broadcast/multicast destinations.
            let is_unicast = pkt.src != 0
                && pkt.src != 0xFFFF_FFFF
                && (pkt.src & 0xF000_0000) != 0xE000_0000   // not 224/4
                && pkt.src != crate::net::ip::our_ip();
            if !is_unicast { return; }

            // AUDIT-NET-F4: token-bucket rate limit. At
            // MAX_ECHO_REPLIES_PER_SEC system-wide, a sustained flood
            // can amplify at most that rate — well below any single
            // attacker's outbound capacity. Refill the bucket once per
            // second (monotonic_secs is cheap; granularity is fine for
            // the use case).
            let now = crate::kernel::time::monotonic_secs() as u32;
            let win = ECHO_REPLY_WINDOW_START.load(Ordering::Relaxed);
            if now.wrapping_sub(win) >= 1 {
                ECHO_REPLY_TOKENS.store(MAX_ECHO_REPLIES_PER_SEC, Ordering::Relaxed);
                ECHO_REPLY_WINDOW_START.store(now, Ordering::Relaxed);
            }
            let tokens = ECHO_REPLY_TOKENS.load(Ordering::Relaxed);
            if tokens == 0 { return; }
            ECHO_REPLY_TOKENS.store(tokens - 1, Ordering::Relaxed);

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

            // Also publish a Linux-compatible raw-socket view (IP header
            // + ICMP payload) so a SOCK_RAW/IPPROTO_ICMP reader can pull
            // it out via `take_raw_reply`. busybox ping parses an IPv4
            // datagram and won't accept headerless ICMP.
            publish_raw_reply(pkt, ICMP_ECHO_REPLY);
        }
        _ => {
            // Still publish so raw sockets see non-echo traffic (TTL
            // exceeded, unreachable, etc.) a future busybox traceroute
            // variant could listen for.
            publish_raw_reply(pkt, icmp_type);
        }
    }
}

fn publish_raw_reply(pkt: &IpPacket, _icmp_type: u8) {
    unsafe {
        let icmp_len = pkt.payload.len().min(RAW_BUF_CAP - 20);
        let total_len = 20 + icmp_len;
        // IPv4 header (20 bytes, no options)
        RAW_REPLY_BUF[0] = 0x45;                           // version 4, IHL 5
        RAW_REPLY_BUF[1] = 0;                              // TOS
        let tot = (total_len as u16).to_be_bytes();
        RAW_REPLY_BUF[2] = tot[0]; RAW_REPLY_BUF[3] = tot[1];
        RAW_REPLY_BUF[4] = 0; RAW_REPLY_BUF[5] = 0;        // ID
        RAW_REPLY_BUF[6] = 0; RAW_REPLY_BUF[7] = 0;        // flags / frag
        RAW_REPLY_BUF[8] = pkt.ttl.max(1);                 // TTL (carry through)
        RAW_REPLY_BUF[9] = pkt.protocol;                   // protocol (1 = ICMP)
        RAW_REPLY_BUF[10] = 0; RAW_REPLY_BUF[11] = 0;      // checksum placeholder
        RAW_REPLY_BUF[12..16].copy_from_slice(&pkt.src.to_be_bytes());
        RAW_REPLY_BUF[16..20].copy_from_slice(&pkt.dst.to_be_bytes());
        // Fill in correct IP checksum so parsers that validate don't reject.
        let cksum = ip::checksum(&RAW_REPLY_BUF[..20]);
        RAW_REPLY_BUF[10..12].copy_from_slice(&cksum.to_be_bytes());
        // ICMP body as-is
        RAW_REPLY_BUF[20..20 + icmp_len].copy_from_slice(&pkt.payload[..icmp_len]);
        RAW_REPLY_SRC.store(pkt.src, Ordering::Relaxed);
        RAW_REPLY_LEN.store(total_len, Ordering::Release);
    }
}

/// Consume the last raw ICMP delivery into `out`. Returns Some((n, src_ip))
/// on success, None if no reply is pending. Used by `sys_recvfrom` on
/// SOCK_RAW+IPPROTO_ICMP sockets.
pub fn take_raw_reply(out: &mut [u8]) -> Option<(usize, u32)> {
    let len = RAW_REPLY_LEN.swap(0, Ordering::Acquire);
    if len == 0 { return None; }
    let n = len.min(out.len());
    unsafe {
        out[..n].copy_from_slice(&RAW_REPLY_BUF[..n]);
    }
    let src = RAW_REPLY_SRC.load(Ordering::Relaxed);
    Some((n, src))
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

    // Payload: "SPHRAGIS_PING"
    let msg = b"SPHRAGIS_PING";
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
