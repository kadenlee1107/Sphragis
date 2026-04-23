//! Followup 3c-nat: kernel-side NAT + packet-layer cave_policy gate.
//!
//! This module is what turns Bat_OS's per-cave policy from a
//! connect-time check at the daemon proxy into a per-packet check in
//! the kernel. Containers on the caves segment send Ethernet frames
//! that arrive on nic 1 (virtio-net, from `-netdev vmnet-host` or
//! `-netdev socket`); Bat_OS parses each frame, looks up which cave
//! owns the source IP, consults `net::cave_policy`, and either drops
//! the frame or forwards it out nic 0 (slirp path to the host).
//!
//! ## Scope for 3c-nat (this commit)
//! - Packet parser for Ethernet + IPv4 + TCP / UDP.
//! - Source-IP → cave lookup table populated by shell command (and
//!   eventually by batcaved at container create time).
//! - cave_policy check per packet; counters for allow vs drop.
//! - Synthetic-frame self-test that exercises both paths without
//!   touching real virtio-net.
//!
//! ## Explicitly OUT of scope here
//! - Full NAT rewriting (src IP/port + checksum recompute). That
//!   lands in 3c-nat-forward.
//! - ARP plumbing on nic 1 (container MAC ↔ IP resolution).
//! - ICMP / IGMP / fragmented IP.
//!
//! ## Rule-matching semantics
//! cave_policy rules are keyed by (host, port, proto). NAT hands the
//! destination IP as the "host" string (dotted-decimal) so operators
//! can write rules like `cpol-add kali 93.184.216.34 443 tcp`. A
//! future bridge to batcaved's DNS cache will let rules specified by
//! name also match raw-IP packets.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

use super::cave_policy;

// ── Counters (observability) ─────────────────────────────────────
static PKT_ALLOW: AtomicU32 = AtomicU32::new(0);
static PKT_DROP_POLICY: AtomicU32 = AtomicU32::new(0);
static PKT_DROP_UNKNOWN_SRC: AtomicU32 = AtomicU32::new(0);
static PKT_DROP_PARSE: AtomicU32 = AtomicU32::new(0);

pub struct Stats {
    pub allow: u32,
    pub drop_policy: u32,
    pub drop_unknown_src: u32,
    pub drop_parse: u32,
}

pub fn stats() -> Stats {
    Stats {
        allow: PKT_ALLOW.load(Ordering::Relaxed),
        drop_policy: PKT_DROP_POLICY.load(Ordering::Relaxed),
        drop_unknown_src: PKT_DROP_UNKNOWN_SRC.load(Ordering::Relaxed),
        drop_parse: PKT_DROP_PARSE.load(Ordering::Relaxed),
    }
}

pub fn reset_stats() {
    PKT_ALLOW.store(0, Ordering::Relaxed);
    PKT_DROP_POLICY.store(0, Ordering::Relaxed);
    PKT_DROP_UNKNOWN_SRC.store(0, Ordering::Relaxed);
    PKT_DROP_PARSE.store(0, Ordering::Relaxed);
}

// ── Source-IP → cave-name mapping ────────────────────────────────
//
// When a container is created, its IP on the caves bridge is
// registered here so the NAT forwarder can attribute each inbound
// frame to a cave. Mirrors the daemon's CAVE_NET_IP but lives in the
// kernel for packet-time decisions.

struct IpBinding {
    ip: u32,
    cave: String,
}

static mut IP_BINDINGS: Option<Vec<IpBinding>> = None;

fn ensure_ip_init() -> &'static mut Vec<IpBinding> {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(IP_BINDINGS);
        if (*ptr).is_none() { *ptr = Some(Vec::new()); }
        (*ptr).as_mut().unwrap()
    }
}

/// Bind a caves-side IPv4 address to a cave name. Replaces any
/// previous binding for the same IP.
pub fn bind_ip(ip: u32, cave: &str) {
    let t = ensure_ip_init();
    for b in t.iter_mut() {
        if b.ip == ip { b.cave = cave.to_string(); return; }
    }
    t.push(IpBinding { ip, cave: cave.to_string() });
}

/// Remove every binding for this cave. Used on cave destroy.
pub fn unbind_cave(cave: &str) {
    let t = ensure_ip_init();
    t.retain(|b| b.cave != cave);
}

pub fn cave_for(ip: u32) -> Option<String> {
    let t = ensure_ip_init();
    for b in t.iter() {
        if b.ip == ip { return Some(b.cave.clone()); }
    }
    None
}

pub fn list_bindings() -> Vec<(u32, String)> {
    let t = ensure_ip_init();
    t.iter().map(|b| (b.ip, b.cave.clone())).collect()
}

pub fn reset_bindings() {
    let t = ensure_ip_init();
    t.clear();
}

// ── Packet parser ────────────────────────────────────────────────

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const IPPROTO_TCP: u8 = 6;
pub const IPPROTO_UDP: u8 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PktVerdict {
    /// Packet permitted — OK to forward out nic 0.
    Allow,
    /// Policy denied (cave found but rule mismatch).
    DropPolicy,
    /// Source IP not bound to any cave → default deny.
    DropUnknownSrc,
    /// Frame parse failed (too short, bad ethertype, fragmented, etc).
    DropParse,
}

/// The 5-tuple extracted from an outbound cave frame.
#[derive(Debug, Clone, Copy)]
pub struct OutboundFlow {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

/// Convert a 32-bit IPv4 (big-endian in wire sense, stored as host
/// u32 where the high byte is the first octet) into dotted-decimal.
fn ip_to_string(ip: u32) -> String {
    let mut s = String::with_capacity(16);
    let b = [
        ((ip >> 24) & 0xFF) as u8,
        ((ip >> 16) & 0xFF) as u8,
        ((ip >>  8) & 0xFF) as u8,
        ( ip        & 0xFF) as u8,
    ];
    for (i, oct) in b.iter().enumerate() {
        if i > 0 { s.push('.'); }
        let n = *oct as u32;
        if n >= 100 { s.push(((n / 100) as u8 + b'0') as char); }
        if n >=  10 { s.push((((n / 10) % 10) as u8 + b'0') as char); }
        s.push(((n % 10) as u8 + b'0') as char);
    }
    s
}

/// Parse an Ethernet+IPv4 frame and return the 5-tuple for
/// outbound-direction NAT decisions. None = drop-parse.
pub fn parse_outbound(frame: &[u8]) -> Option<OutboundFlow> {
    if frame.len() < 14 + 20 { return None; } // ETH + IPv4 min
    let ethertype = ((frame[12] as u16) << 8) | (frame[13] as u16);
    if ethertype != ETHERTYPE_IPV4 { return None; }

    // IPv4 header
    let ip = &frame[14..];
    let ver_ihl = ip[0];
    if (ver_ihl >> 4) != 4 { return None; }
    let ihl = ((ver_ihl & 0x0F) as usize) * 4;
    if ip.len() < ihl { return None; }
    let total_len = ((ip[2] as usize) << 8) | (ip[3] as usize);
    if total_len > ip.len() { return None; }
    // Reject fragments for now (MF set or non-zero offset)
    let frag = ((ip[6] as u16) << 8) | (ip[7] as u16);
    if (frag & 0x3FFF) != 0 { return None; }
    let proto = ip[9];
    let src_ip = ((ip[12] as u32) << 24) | ((ip[13] as u32) << 16)
               | ((ip[14] as u32) <<  8) |  (ip[15] as u32);
    let dst_ip = ((ip[16] as u32) << 24) | ((ip[17] as u32) << 16)
               | ((ip[18] as u32) <<  8) |  (ip[19] as u32);

    let l4 = &ip[ihl..];
    let (src_port, dst_port) = match proto {
        IPPROTO_TCP | IPPROTO_UDP => {
            if l4.len() < 4 { return None; }
            let sp = ((l4[0] as u16) << 8) | (l4[1] as u16);
            let dp = ((l4[2] as u16) << 8) | (l4[3] as u16);
            (sp, dp)
        }
        _ => return None, // other protocols out of scope
    };

    Some(OutboundFlow { src_ip, dst_ip, src_port, dst_port, proto })
}

/// Classify a raw Ethernet frame arriving on the caves interface.
/// Increments the appropriate counter and returns the verdict so
/// callers can decide whether to proceed with forwarding.
pub fn classify(frame: &[u8]) -> PktVerdict {
    let flow = match parse_outbound(frame) {
        Some(f) => f,
        None => {
            PKT_DROP_PARSE.fetch_add(1, Ordering::Relaxed);
            return PktVerdict::DropParse;
        }
    };
    let cave = match cave_for(flow.src_ip) {
        Some(c) => c,
        None => {
            PKT_DROP_UNKNOWN_SRC.fetch_add(1, Ordering::Relaxed);
            return PktVerdict::DropUnknownSrc;
        }
    };
    let dst_str = ip_to_string(flow.dst_ip);
    let v = cave_policy::check_by_name(&cave, &dst_str, flow.dst_port, flow.proto);
    match v {
        cave_policy::Verdict::Allow => {
            PKT_ALLOW.fetch_add(1, Ordering::Relaxed);
            PktVerdict::Allow
        }
        cave_policy::Verdict::Drop => {
            PKT_DROP_POLICY.fetch_add(1, Ordering::Relaxed);
            PktVerdict::DropPolicy
        }
    }
}

// ── Synthetic-frame builder (for tests / demos) ──────────────────

/// Build a minimal Ethernet + IPv4 + TCP frame for testing the
/// classifier. No checksums — the classifier doesn't verify them at
/// this layer. Ports and IPs in host order. Returns bytes suitable
/// for feeding to `classify`.
pub fn build_test_frame(
    src_mac: [u8; 6], dst_mac: [u8; 6],
    src_ip: u32, dst_ip: u32,
    src_port: u16, dst_port: u16,
    proto: u8,
) -> Vec<u8> {
    let mut v = Vec::with_capacity(14 + 20 + 20);
    // Ethernet
    v.extend_from_slice(&dst_mac);
    v.extend_from_slice(&src_mac);
    v.extend_from_slice(&[0x08, 0x00]); // IPv4
    // IPv4 (20 B, IHL=5, proto=tcp/udp, total_len filled later)
    v.push(0x45); v.push(0x00);
    let total_len_slot = v.len();
    v.extend_from_slice(&[0, 0]);       // total length placeholder
    v.extend_from_slice(&[0, 0, 0, 0]); // id + flags/frag
    v.push(64);                         // TTL
    v.push(proto);
    v.extend_from_slice(&[0, 0]);       // header checksum (skip)
    v.push(((src_ip >> 24) & 0xFF) as u8);
    v.push(((src_ip >> 16) & 0xFF) as u8);
    v.push(((src_ip >>  8) & 0xFF) as u8);
    v.push(( src_ip        & 0xFF) as u8);
    v.push(((dst_ip >> 24) & 0xFF) as u8);
    v.push(((dst_ip >> 16) & 0xFF) as u8);
    v.push(((dst_ip >>  8) & 0xFF) as u8);
    v.push(( dst_ip        & 0xFF) as u8);
    // Minimal TCP or UDP header (we just need ports for classifier)
    v.push((src_port >> 8) as u8); v.push((src_port & 0xFF) as u8);
    v.push((dst_port >> 8) as u8); v.push((dst_port & 0xFF) as u8);
    // Pad the L4 header to 20 B for TCP; UDP would be 8 B but 20 is
    // safe since we only parse the first 4 bytes.
    v.extend_from_slice(&[0; 16]);

    // Back-fill IPv4 total length = len(ipv4 onward).
    let tl = v.len() - 14;
    v[total_len_slot]     = (tl >> 8) as u8;
    v[total_len_slot + 1] = (tl & 0xFF) as u8;
    v
}

// ── Self-test ────────────────────────────────────────────────────

pub struct SelftestReport {
    pub allow: u32,
    pub drop_policy: u32,
    pub drop_unknown_src: u32,
    pub drop_parse: u32,
    pub bindings_installed: usize,
}

/// End-to-end classifier proof:
///   1. Wire 2 cave→IP bindings (kali=192.168.77.10, alpine=.11).
///   2. Install a cave_policy where kali may reach 8.8.8.8:53/udp and
///      93.184.216.34:443/tcp; alpine may reach ONLY httpbin (raw IP).
///   3. Inject six synthetic frames:
///        a) kali→8.8.8.8:53/udp          → Allow
///        b) kali→93.184.216.34:443/tcp   → Allow
///        c) kali→evil.example:443 (raw
///           IP 203.0.113.42) /tcp        → DropPolicy
///        d) alpine→93.184.216.34:443/tcp → DropPolicy (cross-cave)
///        e) unknown_src (10.0.0.77)→...  → DropUnknownSrc
///        f) garbage 8-byte frame         → DropParse
///   4. Verify counters match expectation.
pub fn selftest() -> Result<SelftestReport, &'static str> {
    reset_stats();
    reset_bindings();
    // Clean cave_policy so prior tests don't leak rules.
    cave_policy::init();

    // IP bindings (host-order u32)
    const KALI_IP:    u32 = 0xC0A8_4D0A; // 192.168.77.10
    const ALPINE_IP:  u32 = 0xC0A8_4D0B; // 192.168.77.11
    bind_ip(KALI_IP,   "kali");
    bind_ip(ALPINE_IP, "alpine");

    // Policies — use raw IP strings since the packet layer sees IPs.
    use cave_policy::EgressRule;
    cave_policy::add_rule_by_name("kali", EgressRule::udp("8.8.8.8",        53));
    cave_policy::add_rule_by_name("kali", EgressRule::tcp("93.184.216.34",  443));
    cave_policy::add_rule_by_name("alpine", EgressRule::tcp("104.26.11.228", 443));

    let kali_mac   = [0x02, 0xAA, 0, 0, 0, 0x10];
    let alpine_mac = [0x02, 0xAA, 0, 0, 0, 0x11];
    let gw_mac     = [0x02, 0xBB, 0, 0, 0, 0x01];

    // a) kali → 8.8.8.8:53/udp
    let f = build_test_frame(
        kali_mac, gw_mac, KALI_IP, 0x0808_0808, 40000, 53, IPPROTO_UDP,
    );
    if classify(&f) != PktVerdict::Allow { return Err("a) kali udp 53 should Allow"); }

    // b) kali → 93.184.216.34:443/tcp
    let f = build_test_frame(
        kali_mac, gw_mac, KALI_IP, 0x5DB8_D822, 52000, 443, IPPROTO_TCP,
    );
    if classify(&f) != PktVerdict::Allow { return Err("b) kali tcp 443 example.com should Allow"); }

    // c) kali → 203.0.113.42:443/tcp  (not in its allowlist)
    let f = build_test_frame(
        kali_mac, gw_mac, KALI_IP, 0xCB00_712A, 52001, 443, IPPROTO_TCP,
    );
    if classify(&f) != PktVerdict::DropPolicy {
        return Err("c) kali to 203.0.113.42 should DropPolicy");
    }

    // d) alpine → 93.184.216.34:443/tcp  (kali's list, not alpine's)
    let f = build_test_frame(
        alpine_mac, gw_mac, ALPINE_IP, 0x5DB8_D822, 39000, 443, IPPROTO_TCP,
    );
    if classify(&f) != PktVerdict::DropPolicy {
        return Err("d) cross-cave: alpine must not use kali's allowlist");
    }

    // e) unknown src 10.0.0.77 → anywhere
    let f = build_test_frame(
        [0x02, 0xCC, 0, 0, 0, 1], gw_mac,
        0x0A00_004D, 0x0808_0808, 40000, 53, IPPROTO_UDP,
    );
    if classify(&f) != PktVerdict::DropUnknownSrc {
        return Err("e) unknown src must DropUnknownSrc");
    }

    // f) garbage short frame
    let garbage = [0u8; 8];
    if classify(&garbage) != PktVerdict::DropParse {
        return Err("f) garbage frame must DropParse");
    }

    let s = stats();
    // Expected: 2 allow, 2 policy drops, 1 unknown, 1 parse
    if s.allow != 2 || s.drop_policy != 2 ||
       s.drop_unknown_src != 1 || s.drop_parse != 1 {
        return Err("counter totals wrong");
    }

    Ok(SelftestReport {
        allow: s.allow,
        drop_policy: s.drop_policy,
        drop_unknown_src: s.drop_unknown_src,
        drop_parse: s.drop_parse,
        bindings_installed: list_bindings().len(),
    })
}
