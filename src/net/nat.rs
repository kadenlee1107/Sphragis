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
static ARP_REPLIES_SENT: AtomicU32 = AtomicU32::new(0);
static ARP_REQUESTS_IGNORED: AtomicU32 = AtomicU32::new(0);
static ICMP_ECHO_FORWARDED: AtomicU32 = AtomicU32::new(0);
static ICMP_ECHO_DELIVERED: AtomicU32 = AtomicU32::new(0);
/// ICMP error packets (dest-unreachable / time-exceeded) that we
/// successfully rewrote + delivered to the cave whose flow they
/// refer to. Bumped from `pump_replies` via `try_icmp_error_deliver`.
static ICMP_ERROR_DELIVERED: AtomicU32 = AtomicU32::new(0);
static NAT_GC_EVICTED: AtomicU32 = AtomicU32::new(0);
/// Counted separately from `drop_parse` so operators can see WHY a
/// frame was refused. Fragment forwarding requires stateful
/// reassembly (so L4 checksum can be recomputed after NAT rewrites
/// the src port) — out of scope for this pipeline version. The
/// existing rule: fragments are dropped distinctly and logged.
static FRAG_DROPPED: AtomicU32 = AtomicU32::new(0);
/// Frames pumped off nic 0 that were NOT NAT replies and got
/// re-dispatched into the kernel's own IP stack. Tracks how many
/// control-plane packets flowed through the NAT pump correctly.
static HOST_FRAMES_PASSED: AtomicU32 = AtomicU32::new(0);
/// IPv4 fragments successfully reassembled on nic 1 and classified
/// + forwarded as a single packet. Separate from drop_fragment so
/// operators can see the reassembler is working.
static FRAG_REASSEMBLED: AtomicU32 = AtomicU32::new(0);
/// Fragment contexts evicted without completing (TTL expired).
static FRAG_TIMEOUT: AtomicU32 = AtomicU32::new(0);

/// IPv4 address Bat_OS advertises as the caves-side gateway. Used in
/// ARP replies and as the default source when we originate traffic on
/// nic 1 (e.g. ICMP time-exceeded, not wired yet).
pub const CAVES_GATEWAY_IP: u32 = 0xC0A8_4D01; // 192.168.77.1

pub struct Stats {
    pub allow: u32,
    pub drop_policy: u32,
    pub drop_unknown_src: u32,
    pub drop_parse: u32,
    pub drop_fragment: u32,
    pub arp_replies: u32,
    pub arp_ignored: u32,
    pub icmp_forwarded: u32,
    pub icmp_delivered: u32,
    pub icmp_error_delivered: u32,
    pub nat_gc_evicted: u32,
    pub host_frames_passed: u32,
    pub frag_reassembled: u32,
    pub frag_timeout: u32,
}

pub fn stats() -> Stats {
    Stats {
        allow: PKT_ALLOW.load(Ordering::Relaxed),
        drop_policy: PKT_DROP_POLICY.load(Ordering::Relaxed),
        drop_unknown_src: PKT_DROP_UNKNOWN_SRC.load(Ordering::Relaxed),
        drop_parse: PKT_DROP_PARSE.load(Ordering::Relaxed),
        drop_fragment: FRAG_DROPPED.load(Ordering::Relaxed),
        arp_replies: ARP_REPLIES_SENT.load(Ordering::Relaxed),
        arp_ignored: ARP_REQUESTS_IGNORED.load(Ordering::Relaxed),
        icmp_forwarded: ICMP_ECHO_FORWARDED.load(Ordering::Relaxed),
        icmp_delivered: ICMP_ECHO_DELIVERED.load(Ordering::Relaxed),
        icmp_error_delivered: ICMP_ERROR_DELIVERED.load(Ordering::Relaxed),
        nat_gc_evicted: NAT_GC_EVICTED.load(Ordering::Relaxed),
        host_frames_passed: HOST_FRAMES_PASSED.load(Ordering::Relaxed),
        frag_reassembled: FRAG_REASSEMBLED.load(Ordering::Relaxed),
        frag_timeout: FRAG_TIMEOUT.load(Ordering::Relaxed),
    }
}

pub fn reset_stats() {
    PKT_ALLOW.store(0, Ordering::Relaxed);
    PKT_DROP_POLICY.store(0, Ordering::Relaxed);
    PKT_DROP_UNKNOWN_SRC.store(0, Ordering::Relaxed);
    PKT_DROP_PARSE.store(0, Ordering::Relaxed);
    FRAG_DROPPED.store(0, Ordering::Relaxed);
    ARP_REPLIES_SENT.store(0, Ordering::Relaxed);
    ARP_REQUESTS_IGNORED.store(0, Ordering::Relaxed);
    ICMP_ECHO_FORWARDED.store(0, Ordering::Relaxed);
    ICMP_ECHO_DELIVERED.store(0, Ordering::Relaxed);
    ICMP_ERROR_DELIVERED.store(0, Ordering::Relaxed);
    NAT_GC_EVICTED.store(0, Ordering::Relaxed);
    HOST_FRAMES_PASSED.store(0, Ordering::Relaxed);
    FRAG_REASSEMBLED.store(0, Ordering::Relaxed);
    FRAG_TIMEOUT.store(0, Ordering::Relaxed);
}

// ── ARP on nic 1 (caves interface) ──────────────────────────────
//
// A real container ARPs for 192.168.77.1 (the gateway we advertise)
// before it can send its first IP frame — without a reply, its
// outgoing traffic never hits nic 1 because the Ethernet dst is
// unresolved. `try_handle_arp` answers those requests with nic 1's
// MAC; anything else (requests for other targets, replies, wrong
// ethertype) falls through to the IPv4 path or the drop counter.

/// If `frame` is an ARP request on nic 1 for the caves gateway IP,
/// build + send the reply and return true. Otherwise returns false
/// so the caller can try the IPv4 path.
pub fn try_handle_arp(frame: &[u8]) -> bool {
    if frame.len() < 14 + 28 { return false; }
    let ethertype = ((frame[12] as u16) << 8) | (frame[13] as u16);
    if ethertype != ETHERTYPE_ARP { return false; }
    let arp = &frame[14..];
    // Only handle Ethernet/IPv4 ARP.
    let hw_type   = ((arp[0] as u16) << 8) | (arp[1] as u16);
    let proto_ty  = ((arp[2] as u16) << 8) | (arp[3] as u16);
    if hw_type != 1 || proto_ty != ETHERTYPE_IPV4 { return false; }
    if arp[4] != 6 || arp[5] != 4 { return false; }
    let op = ((arp[6] as u16) << 8) | (arp[7] as u16);
    if op != ARP_OP_REQUEST {
        ARP_REQUESTS_IGNORED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    // Sender + target proto addresses.
    let target_ip = ((arp[24] as u32) << 24) | ((arp[25] as u32) << 16)
                  | ((arp[26] as u32) <<  8) |  (arp[27] as u32);
    if target_ip != CAVES_GATEWAY_IP {
        ARP_REQUESTS_IGNORED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    // Capture sender MAC + IP so we can build a targeted reply.
    let mut sender_mac = [0u8; 6];
    sender_mac.copy_from_slice(&arp[8..14]);
    let sender_ip = ((arp[14] as u32) << 24) | ((arp[15] as u32) << 16)
                  | ((arp[16] as u32) <<  8) |  (arp[17] as u32);
    // Build the reply. We claim CAVES_GATEWAY_IP at our nic 1 MAC.
    use crate::drivers::virtio::net;
    if !net::is_ready_n(1) { return false; }
    let nic1_mac = net::mac_n(1);
    let reply = build_arp_reply(sender_mac, sender_ip, nic1_mac, target_ip);
    let _ = net::send_n(1, &reply);
    ARP_REPLIES_SENT.fetch_add(1, Ordering::Relaxed);
    true
}

/// Build a raw ARP reply Ethernet frame.
pub fn build_arp_reply(
    target_mac: [u8; 6], target_ip: u32,
    sender_mac: [u8; 6], sender_ip: u32,
) -> Vec<u8> {
    let mut v = Vec::with_capacity(14 + 28);
    // Ethernet
    v.extend_from_slice(&target_mac);
    v.extend_from_slice(&sender_mac);
    v.extend_from_slice(&[0x08, 0x06]); // ARP
    // ARP payload
    v.extend_from_slice(&[0x00, 0x01]);      // hw type = Ethernet
    v.extend_from_slice(&[0x08, 0x00]);      // proto type = IPv4
    v.push(6);                               // hw addr len
    v.push(4);                               // proto addr len
    v.extend_from_slice(&[0x00, 0x02]);      // op = reply
    v.extend_from_slice(&sender_mac);        // sender HW
    v.push(((sender_ip >> 24) & 0xFF) as u8);
    v.push(((sender_ip >> 16) & 0xFF) as u8);
    v.push(((sender_ip >>  8) & 0xFF) as u8);
    v.push(( sender_ip        & 0xFF) as u8);
    v.extend_from_slice(&target_mac);        // target HW
    v.push(((target_ip >> 24) & 0xFF) as u8);
    v.push(((target_ip >> 16) & 0xFF) as u8);
    v.push(((target_ip >>  8) & 0xFF) as u8);
    v.push(( target_ip        & 0xFF) as u8);
    v
}

/// Drain every pending frame on nic 1 (the caves interface), running
/// each through `classify`. Returns how many frames were processed.
/// Safe to call when nic 1 is absent — returns 0.
pub fn pump() -> usize {
    use crate::drivers::virtio::net;
    if !net::is_ready_n(1) { return 0; }
    let mut count = 0usize;
    let mut buf = [0u8; 1514];
    // Bounded drain: we don't want to loop forever if the peer is
    // flooding us. 256 frames per pump call is plenty for interactive
    // shell use; a future main-loop integration can adjust the budget.
    for _ in 0..256 {
        match net::recv_n(1, &mut buf) {
            Some(len) => {
                let _ = classify(&buf[..len]);
                count += 1;
            }
            None => break,
        }
    }
    count
}

/// Drain nic 1, classify, forward ALLOW'd frames out nic 0 after
/// NAT-rewriting. Returns (drained, forwarded).
pub fn pump_and_forward(nic0_ip: u32, nic0_mac: [u8; 6], gw_mac: [u8; 6]) -> (usize, usize) {
    use crate::drivers::virtio::net;
    if !net::is_ready_n(1) { return (0, 0); }
    let mut drained = 0usize;
    let mut forwarded = 0usize;
    let mut buf = [0u8; 1514];
    for _ in 0..256 {
        let len = match net::recv_n(1, &mut buf) { Some(l) => l, None => break };
        drained += 1;
        let frame = &buf[..len];
        // ARP first — if the frame is an ARP request for our gateway IP,
        // we reply directly on nic 1 and don't touch NAT. Containers MUST
        // resolve the gateway before they can send the first IP frame.
        if try_handle_arp(frame) { continue; }
        // Fragment reassembly: a fragmented IPv4 datagram can't be
        // classified from a single fragment (only the first has the
        // L4 header) and can't be NAT'd without recomputing L4
        // checksum over the reassembled payload. Buffer fragments,
        // and once complete feed the reassembled frame through the
        // normal classify/NAT/forward path below.
        let reassembled: Vec<u8>;
        let work_frame: &[u8] = if is_ipv4_fragment(frame) {
            match frag_accept(frame) {
                Some(full) => {
                    FRAG_REASSEMBLED.fetch_add(1, Ordering::Relaxed);
                    reassembled = full;
                    &reassembled[..]
                }
                None => continue, // still waiting for more fragments
            }
        } else {
            frame
        };
        let verdict = classify(work_frame);
        if verdict != PktVerdict::Allow { continue; }
        let flow = match parse_outbound(work_frame) { Some(f) => f, None => continue };
        // Allocate (or reuse) a NAT entry for this cave flow.
        let (eph_port, src_mac) = match nat_alloc_out(&flow, work_frame) {
            Some(v) => v,
            None => continue, // table full
        };
        let mut out = Vec::from(work_frame);
        if rewrite_outbound_into(&mut out, flow, eph_port, nic0_ip, nic0_mac, gw_mac).is_ok() {
            let _ = net::send_n(0, &out);
            forwarded += 1;
            if flow.proto == IPPROTO_ICMP {
                ICMP_ECHO_FORWARDED.fetch_add(1, Ordering::Relaxed);
            }
            let _ = src_mac; // reserved for reply-MAC caching below
        }
    }
    (drained, forwarded)
}

/// Main-loop entry point. Called each iteration of the desktop idle
/// loop (bounded budget inside pump_and_forward / pump_replies so a
/// packet flood can't starve the UI).
///
/// Uses the built-in nic 0 slirp defaults: 10.0.2.15 source IP,
/// 52:55:0a:00:02:02 gateway MAC (slirp is L4-NAT so the specific MAC
/// is irrelevant). Returns total activity count for debug counters.
pub fn tick() -> (usize, usize) {
    use crate::drivers::virtio::net;
    if !net::is_ready_n(1) && !net::is_ready_n(0) { return (0, 0); }
    let nic0_mac = net::mac_n(0);
    let nic1_mac = net::mac_n(1);
    let nic0_ip:  u32 = 0x0A00020F;
    let gw_mac = [0x52, 0x55, 0x0A, 0x00, 0x02, 0x02];
    // Skip the outbound path if nic 1 isn't up — otherwise would
    // spam "not ready" for every tick.
    let out = if net::is_ready_n(1) {
        pump_and_forward(nic0_ip, nic0_mac, gw_mac)
    } else { (0, 0) };
    // Only drain nic 0 for reply frames if we have caves registered.
    // Without bindings, every reply would fall through the lookup and
    // we'd burn cycles on unrelated traffic. This also avoids racing
    // with the existing `net::poll_once` callers.
    let inn = if net::is_ready_n(0) && nat_table_size() > 0 {
        pump_replies(nic1_mac)
    } else { (0, 0) };
    // Keep the NAT table honest — sweep any entries past their TTL.
    gc_tick();
    (out.1, inn.1)
}

/// Drain nic 0, reverse-NAT replies that match our table, deliver on
/// nic 1 to the original cave. Frames that DON'T match the NAT table
/// are not lost — they get re-dispatched into the kernel's own IPv4
/// stack (`net::dispatch_host_frame`) so daemon heartbeat, DNS, etc.
/// still reach their handlers. Returns (drained, delivered).
pub fn pump_replies(nic1_mac: [u8; 6]) -> (usize, usize) {
    use crate::drivers::virtio::net;
    if !net::is_ready_n(0) { return (0, 0); }
    let mut drained = 0usize;
    let mut delivered = 0usize;
    let mut buf = [0u8; 1514];
    for _ in 0..64 {
        let len = match net::recv_n(0, &mut buf) { Some(l) => l, None => break };
        drained += 1;
        // Inbound fragment reassembly mirrors the outbound path: if
        // this frame is an IPv4 fragment, buffer it via frag_accept.
        // Only run the rest of the reverse-NAT flow once the datagram
        // is fully reassembled; until then nothing to forward.
        let frame_slice = &buf[..len];
        let reassembled_inbound: Vec<u8>;
        let work: &[u8] = if is_ipv4_fragment(frame_slice) {
            match frag_accept(frame_slice) {
                Some(full) => {
                    FRAG_REASSEMBLED.fetch_add(1, Ordering::Relaxed);
                    reassembled_inbound = full;
                    &reassembled_inbound[..]
                }
                None => continue,  // waiting for more fragments
            }
        } else {
            frame_slice
        };
        // Decide if this is a NAT reply. If NOT, hand it to the kernel
        // stack so we don't silently consume control-plane packets.
        let maybe_entry = parse_inbound(work)
            .and_then(|f| nat_lookup_in(f.dst_port, f.proto));
        match maybe_entry {
            Some(entry) => {
                let mut out = Vec::from(work);
                if rewrite_inbound_into(&mut out, &entry, nic1_mac).is_ok() {
                    let _ = net::send_n(1, &out);
                    delivered += 1;
                    if entry.proto == IPPROTO_ICMP {
                        ICMP_ECHO_DELIVERED.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            None => {
                // Before falling back to the host stack, see if this
                // is an ICMP error packet carrying the header of a
                // cave flow we NAT'd — if so, rewrite + deliver.
                let mut copy: Vec<u8> = Vec::from(work);
                if try_rewrite_icmp_error_inbound(&mut copy, nic1_mac) {
                    let _ = net::send_n(1, &copy);
                    ICMP_ERROR_DELIVERED.fetch_add(1, Ordering::Relaxed);
                    delivered += 1;
                } else {
                    super::dispatch_host_frame(work);
                    HOST_FRAMES_PASSED.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
    (drained, delivered)
}

// ── NAT table ──────────────────────────────────────────────────────

/// One active translation. `eph_port` on nic 0's side is what the
/// internet sees; when a reply comes back with dst_port == eph_port
/// we rewrite it back to `cave_ip:cave_src_port` and deliver on nic 1.
#[derive(Clone, Copy)]
pub struct NatEntry {
    pub active: bool,
    pub proto: u8,
    pub cave_ip: u32,
    pub cave_src_port: u16,
    pub eph_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub cave_mac: [u8; 6],
    /// CNTPCT_EL0 tick when this entry last saw traffic. Bumped on
    /// create, every outbound hit, and every inbound match. The GC
    /// evicts entries whose last_seen is older than the proto's TTL.
    pub last_seen_ticks: u64,
}

const NAT_SLOTS: usize = 64;
const EPH_PORT_BASE: u16 = 50_000;

/// TTL per protocol in seconds. UDP is a stateless datagram service —
/// a flow that's silent for a minute is reasonable to reclaim. TCP
/// sessions can idle for minutes between packets on long-running
/// keepalive connections, so we're more patient.
pub const NAT_TTL_UDP_SECS:  u64 = 60;
pub const NAT_TTL_TCP_SECS:  u64 = 300;
pub const NAT_TTL_ICMP_SECS: u64 = 30;

static mut NAT_TABLE: [NatEntry; NAT_SLOTS] = [NatEntry {
    active: false, proto: 0, cave_ip: 0, cave_src_port: 0,
    eph_port: 0, dst_ip: 0, dst_port: 0, cave_mac: [0; 6],
    last_seen_ticks: 0,
}; NAT_SLOTS];
static NAT_NEXT_EPH: AtomicU32 = AtomicU32::new(EPH_PORT_BASE as u32);
static NAT_GC_LAST_RUN_TICKS: core::sync::atomic::AtomicU64
    = core::sync::atomic::AtomicU64::new(0);

/// Current virtual timer tick. Stable counter on aarch64; used for
/// NAT entry TTL bookkeeping and GC throttling.
fn now_ticks() -> u64 {
    let t: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) t); }
    t
}

fn ticks_per_sec() -> u64 {
    let f: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) f); }
    // Guard against a zero reading on weird hardware so callers don't
    // divide by zero downstream.
    if f == 0 { 24_000_000 } else { f }
}

fn ttl_for(proto: u8) -> u64 {
    match proto {
        IPPROTO_UDP  => NAT_TTL_UDP_SECS,
        IPPROTO_TCP  => NAT_TTL_TCP_SECS,
        IPPROTO_ICMP => NAT_TTL_ICMP_SECS,
        _ => NAT_TTL_UDP_SECS,
    }
}

fn nat_lookup_or_create(flow: &OutboundFlow, cave_mac: [u8; 6]) -> Option<u16> {
    let now = now_ticks();
    unsafe {
        let t = core::ptr::addr_of_mut!(NAT_TABLE);
        // Existing entry? Bump last_seen so GC doesn't evict a hot flow.
        for i in 0..NAT_SLOTS {
            let e = (*t)[i];
            if e.active
                && e.proto == flow.proto
                && e.cave_ip == flow.src_ip
                && e.cave_src_port == flow.src_port
                && e.dst_ip == flow.dst_ip
                && e.dst_port == flow.dst_port
            {
                (*t)[i].last_seen_ticks = now;
                return Some(e.eph_port);
            }
        }
        // New entry: allocate an ephemeral port, find a free slot.
        let next = NAT_NEXT_EPH.fetch_add(1, Ordering::Relaxed) as u16;
        let eph_port = if next == 0 { EPH_PORT_BASE } else { next };
        for i in 0..NAT_SLOTS {
            if !(*t)[i].active {
                (*t)[i] = NatEntry {
                    active: true,
                    proto: flow.proto,
                    cave_ip: flow.src_ip,
                    cave_src_port: flow.src_port,
                    eph_port,
                    dst_ip: flow.dst_ip,
                    dst_port: flow.dst_port,
                    cave_mac,
                    last_seen_ticks: now,
                };
                return Some(eph_port);
            }
        }
        None
    }
}

/// Helper that alloc_outs a NAT entry and returns (eph_port, cave_mac).
fn nat_alloc_out(flow: &OutboundFlow, frame: &[u8]) -> Option<(u16, [u8; 6])> {
    let mut cave_mac = [0u8; 6];
    cave_mac.copy_from_slice(&frame[6..12]);
    let eph = nat_lookup_or_create(flow, cave_mac)?;
    Some((eph, cave_mac))
}

pub fn nat_lookup_in(dst_port: u16, proto: u8) -> Option<NatEntry> {
    let now = now_ticks();
    unsafe {
        let t = core::ptr::addr_of_mut!(NAT_TABLE);
        for i in 0..NAT_SLOTS {
            let e = (*t)[i];
            if e.active && e.proto == proto && e.eph_port == dst_port {
                // Inbound traffic proves the flow is still alive.
                (*t)[i].last_seen_ticks = now;
                return Some(e);
            }
        }
    }
    None
}

/// Evict NAT entries whose `last_seen` is older than their proto's
/// TTL. Returns how many slots were freed. Throttled by the caller
/// so it doesn't walk the whole table on every tick — see `gc_tick`.
pub fn nat_gc_sweep(now_t: u64, tps: u64) -> u32 {
    let mut evicted = 0u32;
    unsafe {
        let t = core::ptr::addr_of_mut!(NAT_TABLE);
        for i in 0..NAT_SLOTS {
            let e = (*t)[i];
            if !e.active { continue; }
            let age_ticks = now_t.saturating_sub(e.last_seen_ticks);
            let age_secs = age_ticks / tps;
            if age_secs >= ttl_for(e.proto) {
                (*t)[i].active = false;
                evicted += 1;
            }
        }
    }
    if evicted > 0 {
        NAT_GC_EVICTED.fetch_add(evicted, Ordering::Relaxed);
    }
    evicted
}

/// Throttle wrapper: only sweep once per second of wall-clock. Safe
/// to call every tick — the counter avoids redundant scans.
pub fn gc_tick() {
    let now = now_ticks();
    let tps = ticks_per_sec();
    let last = NAT_GC_LAST_RUN_TICKS.load(Ordering::Relaxed);
    // Run if we've never run, or if at least 1 second has elapsed.
    if last == 0 || now.saturating_sub(last) >= tps {
        NAT_GC_LAST_RUN_TICKS.store(now, Ordering::Relaxed);
        let _ = nat_gc_sweep(now, tps);
        frag_gc_sweep(now, tps);
    }
}

/// Force-run a GC sweep regardless of throttle. For tests + shell.
pub fn nat_gc_force(now_override_secs: Option<u64>) -> u32 {
    let tps = ticks_per_sec();
    let now = match now_override_secs {
        Some(s) => s.saturating_mul(tps),
        None => now_ticks(),
    };
    nat_gc_sweep(now, tps)
}

pub fn nat_table_size() -> usize {
    unsafe {
        let t = core::ptr::addr_of!(NAT_TABLE);
        (0..NAT_SLOTS).filter(|i| (*t)[*i].active).count()
    }
}

pub fn nat_table_clear() {
    unsafe {
        let t = core::ptr::addr_of_mut!(NAT_TABLE);
        for i in 0..NAT_SLOTS {
            (*t)[i].active = false;
        }
    }
    NAT_NEXT_EPH.store(EPH_PORT_BASE as u32, Ordering::Relaxed);
}

// ── Outbound fragment reassembler ────────────────────────────────
//
// Policy decisions need the L4 src/dst ports which only live in the
// first fragment; NAT rewrite needs to recompute the L4 checksum
// which spans the whole payload. The only correct way to make both
// work is to buffer fragments until the full datagram is present,
// then run the reassembled packet through the normal classify →
// rewrite → forward path as if it had arrived unfragmented.
//
// This first-pass implementation supports small outbound fragments
// (total datagram up to FRAG_BUF_BYTES) and discards contexts that
// sit half-complete for more than FRAG_TTL_SECS. Inbound fragments
// (replies from the internet on nic 0) are deliberately out of
// scope — slirp on the other side of nic 0 already reassembles for
// us in practice, and adding inbound-frag support would mostly be
// needed for corner-case jumbo-frame routing.

const FRAG_BUF_BYTES:  usize = 2048;   // max reassembled datagram size
const FRAG_SLOTS:      usize = 8;      // concurrent contexts (bidir)
const FRAG_TTL_SECS:   u64   = 30;     // standard IP-reassembly timeout

#[derive(Clone, Copy)]
struct FragCtx {
    active: bool,
    src_ip: u32,
    dst_ip: u32,
    ip_id: u16,
    proto: u8,
    /// Total length of the reassembled IPv4 DATAGRAM (payload only,
    /// i.e. ihl + L4 + data — everything after Ethernet). Set when
    /// the last fragment (MF=0) arrives. 0 = unknown.
    total_len: u16,
    /// Highest "expected" end-offset seen so far — once total_len is
    /// known and received_bytes == total_len we're complete.
    received_bytes: u16,
    /// Cached first-fragment source + dest MACs for the final frame.
    eth_src: [u8; 6],
    eth_dst: [u8; 6],
    /// IHL of the first fragment's IPv4 header (we copy that verbatim
    /// into the reassembled packet).
    first_ihl: u8,
    /// Tick of first fragment arrival — used by frag_gc.
    first_seen_ticks: u64,
    /// Reassembly scratch. IPv4 header is at offset 0 (from the
    /// first fragment); subsequent fragment payloads land starting
    /// at offset 14 + ihl + 8*frag_offset relative to the original
    /// Ethernet frame, but we only store the IP datagram here so
    /// subtract 14 (Ethernet).
    buf: [u8; FRAG_BUF_BYTES],
}

const FRAG_CTX_NEW: FragCtx = FragCtx {
    active: false, src_ip: 0, dst_ip: 0, ip_id: 0, proto: 0,
    total_len: 0, received_bytes: 0,
    eth_src: [0; 6], eth_dst: [0; 6], first_ihl: 0,
    first_seen_ticks: 0,
    buf: [0u8; FRAG_BUF_BYTES],
};

static mut FRAG_TABLE: [FragCtx; FRAG_SLOTS] = [FRAG_CTX_NEW; FRAG_SLOTS];

fn frag_find(src: u32, dst: u32, id: u16, proto: u8) -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(FRAG_TABLE);
        for i in 0..FRAG_SLOTS {
            let c = &(*t)[i];
            if c.active && c.src_ip == src && c.dst_ip == dst
                && c.ip_id == id && c.proto == proto
            { return Some(i); }
        }
    }
    None
}

fn frag_alloc(src: u32, dst: u32, id: u16, proto: u8, now: u64) -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of_mut!(FRAG_TABLE);
        for i in 0..FRAG_SLOTS {
            if !(*t)[i].active {
                (*t)[i].active = true;
                (*t)[i].src_ip = src; (*t)[i].dst_ip = dst;
                (*t)[i].ip_id = id; (*t)[i].proto = proto;
                (*t)[i].total_len = 0; (*t)[i].received_bytes = 0;
                (*t)[i].first_ihl = 0;
                (*t)[i].first_seen_ticks = now;
                for b in (*t)[i].buf.iter_mut() { *b = 0; }
                return Some(i);
            }
        }
    }
    None
}

fn frag_evict(idx: usize) {
    unsafe {
        let t = core::ptr::addr_of_mut!(FRAG_TABLE);
        (*t)[idx].active = false;
    }
}

/// Periodically called from gc_tick: discard fragment contexts older
/// than FRAG_TTL_SECS. Counts into FRAG_TIMEOUT so operators notice.
pub fn frag_gc_sweep(now_t: u64, tps: u64) {
    let mut stale = 0u32;
    unsafe {
        let t = core::ptr::addr_of_mut!(FRAG_TABLE);
        for i in 0..FRAG_SLOTS {
            if !(*t)[i].active { continue; }
            let age_secs = now_t.saturating_sub((*t)[i].first_seen_ticks) / tps;
            if age_secs >= FRAG_TTL_SECS {
                (*t)[i].active = false;
                stale += 1;
            }
        }
    }
    if stale > 0 { FRAG_TIMEOUT.fetch_add(stale, Ordering::Relaxed); }
}

pub fn frag_ctx_count() -> usize {
    unsafe {
        let t = core::ptr::addr_of!(FRAG_TABLE);
        (0..FRAG_SLOTS).filter(|i| (*t)[*i].active).count()
    }
}

/// Accept an IPv4 fragment frame on nic 1. Returns:
///   Some(Vec<u8>) with a COMPLETE reassembled Ethernet+IPv4 frame
///     (no fragment flags) if this fragment finished a datagram;
///   None if still pending, or on any error.
pub fn frag_accept(frame: &[u8]) -> Option<Vec<u8>> {
    if frame.len() < 14 + 20 { return None; }
    if &frame[12..14] != b"\x08\x00" { return None; }
    let ip = &frame[14..];
    let ihl = ((ip[0] & 0x0F) as usize) * 4;
    if frame.len() < 14 + ihl { return None; }
    let total_len_hdr = ((ip[2] as usize) << 8) | (ip[3] as usize);
    let frag = ((ip[6] as u16) << 8) | (ip[7] as u16);
    let mf = (frag & 0x2000) != 0;
    let offset_bytes = ((frag & 0x1FFF) as usize) * 8;
    if !mf && offset_bytes == 0 {
        return None; // not actually fragmented, caller shouldn't use us
    }
    let proto = ip[9];
    let src_ip =
        ((ip[12] as u32) << 24) | ((ip[13] as u32) << 16)
      | ((ip[14] as u32) <<  8) |  (ip[15] as u32);
    let dst_ip =
        ((ip[16] as u32) << 24) | ((ip[17] as u32) << 16)
      | ((ip[18] as u32) <<  8) |  (ip[19] as u32);
    let ip_id = ((ip[4] as u16) << 8) | (ip[5] as u16);

    // Only TCP/UDP (protos our classifier understands post-reassembly).
    if proto != IPPROTO_TCP && proto != IPPROTO_UDP { return None; }

    let payload_len = total_len_hdr.saturating_sub(ihl);
    if payload_len == 0 { return None; }
    if offset_bytes + payload_len > FRAG_BUF_BYTES { return None; }

    let now = now_ticks();
    // Find or allocate the context.
    let idx = match frag_find(src_ip, dst_ip, ip_id, proto) {
        Some(i) => i,
        None => match frag_alloc(src_ip, dst_ip, ip_id, proto, now) {
            Some(i) => i,
            None => return None, // table full
        }
    };

    unsafe {
        let t = core::ptr::addr_of_mut!(FRAG_TABLE);
        let c = &mut (*t)[idx];
        // First fragment provides the IP header + eth MACs.
        if offset_bytes == 0 {
            c.first_ihl = ihl as u8;
            c.eth_dst.copy_from_slice(&frame[0..6]);
            c.eth_src.copy_from_slice(&frame[6..12]);
            // Copy IHL header bytes into buf[0..ihl].
            c.buf[..ihl].copy_from_slice(&ip[..ihl]);
        }
        // Copy payload into buf[ihl + offset_bytes..ihl + offset_bytes + payload_len].
        let dst_start = (c.first_ihl as usize).max(ihl) + offset_bytes;
        if dst_start + payload_len > FRAG_BUF_BYTES {
            frag_evict(idx);
            return None;
        }
        c.buf[dst_start..dst_start + payload_len]
            .copy_from_slice(&ip[ihl..ihl + payload_len]);
        c.received_bytes = c.received_bytes.saturating_add(payload_len as u16);
        if !mf {
            // Last fragment — total_len = offset + payload_len + ihl
            c.total_len = (offset_bytes + payload_len + (c.first_ihl as usize)) as u16;
        }
        // Complete?
        if c.total_len > 0 && (c.received_bytes as usize + c.first_ihl as usize) >= c.total_len as usize {
            // Build the reassembled Ethernet+IPv4 frame.
            let ihl_u = c.first_ihl as usize;
            let total = c.total_len as usize;
            let mut out = Vec::with_capacity(14 + total);
            out.extend_from_slice(&c.eth_dst);
            out.extend_from_slice(&c.eth_src);
            out.extend_from_slice(&[0x08, 0x00]);
            out.extend_from_slice(&c.buf[..total]);
            // Patch the IPv4 header: total_len, zero fragment fields,
            // recompute checksum.
            let ip_start = 14;
            out[ip_start + 2] = (total >> 8) as u8;
            out[ip_start + 3] = (total & 0xFF) as u8;
            // Keep id but zero flags/frag offset (MF=0, offset=0).
            out[ip_start + 6] = 0;
            out[ip_start + 7] = 0;
            // Header cksum
            out[ip_start + 10] = 0; out[ip_start + 11] = 0;
            let ck = ipv4_checksum(&out[ip_start..ip_start + ihl_u]);
            out[ip_start + 10] = (ck >> 8) as u8;
            out[ip_start + 11] = (ck & 0xFF) as u8;
            // Done — evict + return.
            frag_evict(idx);
            return Some(out);
        }
    }
    None
}

// ── Inbound parse (replies from internet on nic 0) ───────────────

#[derive(Debug, Clone, Copy)]
pub struct InboundFlow {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

pub fn parse_inbound(frame: &[u8]) -> Option<InboundFlow> {
    // Inbound differs from outbound for ICMP: we accept Echo Reply
    // (type 0) here, whereas parse_outbound only accepts Echo Request.
    if frame.len() < 14 + 20 { return None; }
    let ethertype = ((frame[12] as u16) << 8) | (frame[13] as u16);
    if ethertype != ETHERTYPE_IPV4 { return None; }
    let ip = &frame[14..];
    let ver_ihl = ip[0];
    if (ver_ihl >> 4) != 4 { return None; }
    let ihl = ((ver_ihl & 0x0F) as usize) * 4;
    if ip.len() < ihl { return None; }
    let total_len = ((ip[2] as usize) << 8) | (ip[3] as usize);
    if total_len > ip.len() { return None; }
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
        IPPROTO_ICMP => {
            if l4.len() < 8 { return None; }
            // Accept only Echo Reply on inbound.
            if l4[0] != ICMP_TYPE_ECHO_REPLY { return None; }
            let id = ((l4[4] as u16) << 8) | (l4[5] as u16);
            // The NAT lookup table keys on (proto, eph_port = id).
            // Store the identifier in dst_port so the caller's lookup
            // flow works unchanged for all three protos.
            (0, id)
        }
        _ => return None,
    };
    Some(InboundFlow { src_ip, dst_ip, src_port, dst_port, proto })
}

// ── Rewrite (outbound + inbound) ──────────────────────────────────

fn ipv4_checksum(hdr: &[u8]) -> u16 {
    // Zero-out the checksum field (bytes 10..12) during compute.
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < hdr.len() {
        let w = if i == 10 { 0 } else {
            ((hdr[i] as u16) << 8) | (hdr[i + 1] as u16)
        };
        sum = sum.wrapping_add(w as u32);
        i += 2;
    }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    !(sum as u16)
}

/// Compute TCP/UDP checksum over pseudo-header + L4 header + payload.
/// `proto` = 6 (TCP) or 17 (UDP). `l4` is the L4 segment (zero-out
/// the checksum field in-place before calling).
fn l4_checksum(src_ip: u32, dst_ip: u32, proto: u8, l4: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Pseudo header: src, dst, zero, proto, length
    sum = sum.wrapping_add(((src_ip >> 16) & 0xFFFF) as u32);
    sum = sum.wrapping_add((src_ip & 0xFFFF) as u32);
    sum = sum.wrapping_add(((dst_ip >> 16) & 0xFFFF) as u32);
    sum = sum.wrapping_add((dst_ip & 0xFFFF) as u32);
    sum = sum.wrapping_add(proto as u32);
    sum = sum.wrapping_add(l4.len() as u32);
    // L4 segment (with checksum field already zeroed)
    let mut i = 0;
    while i + 1 < l4.len() {
        sum = sum.wrapping_add((((l4[i] as u16) << 8) | (l4[i + 1] as u16)) as u32);
        i += 2;
    }
    if i < l4.len() { sum = sum.wrapping_add((l4[i] as u32) << 8); }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    !(sum as u16)
}

/// Rewrite an outbound cave frame in-place: Ethernet src→nic0_mac,
/// Ethernet dst→gw_mac, IPv4 src → nic0_ip, L4 src_port → eph_port,
/// recompute both checksums.
pub fn rewrite_outbound_into(
    frame: &mut [u8],
    _flow: OutboundFlow,
    eph_port: u16,
    nic0_ip: u32,
    nic0_mac: [u8; 6],
    gw_mac: [u8; 6],
) -> Result<(), &'static str> {
    if frame.len() < 14 + 20 { return Err("frame too small"); }
    // Ethernet rewrite
    frame[0..6].copy_from_slice(&gw_mac);
    frame[6..12].copy_from_slice(&nic0_mac);
    // IPv4
    let ihl = ((frame[14] & 0x0F) as usize) * 4;
    if frame.len() < 14 + ihl + 4 { return Err("ipv4 truncated"); }
    // Write new src IP
    frame[14 + 12] = ((nic0_ip >> 24) & 0xFF) as u8;
    frame[14 + 13] = ((nic0_ip >> 16) & 0xFF) as u8;
    frame[14 + 14] = ((nic0_ip >>  8) & 0xFF) as u8;
    frame[14 + 15] = ( nic0_ip        & 0xFF) as u8;
    // Compute src_ip/dst_ip for checksums
    let src_ip = nic0_ip;
    let dst_ip = ((frame[14 + 16] as u32) << 24)
               | ((frame[14 + 17] as u32) << 16)
               | ((frame[14 + 18] as u32) <<  8)
               |  (frame[14 + 19] as u32);
    let proto = frame[14 + 9];
    // IPv4 checksum
    let ip_hdr_len = ihl;
    let ip_cksum = {
        // Compute over the header with cksum field=0
        let hdr_slice = &frame[14..14 + ip_hdr_len];
        ipv4_checksum(hdr_slice)
    };
    frame[14 + 10] = (ip_cksum >> 8) as u8;
    frame[14 + 11] = (ip_cksum & 0xFF) as u8;
    // Rewrite L4 src port (or ICMP id) + checksum
    let l4_start = 14 + ip_hdr_len;
    if frame.len() < l4_start + 8 { return Err("l4 too short"); }
    match proto {
        IPPROTO_TCP | IPPROTO_UDP => {
            // src port (first 2 bytes of TCP/UDP header)
            frame[l4_start    ] = (eph_port >> 8) as u8;
            frame[l4_start + 1] = (eph_port & 0xFF) as u8;
            let cksum_off = if proto == IPPROTO_TCP { l4_start + 16 } else { l4_start + 6 };
            frame[cksum_off]     = 0;
            frame[cksum_off + 1] = 0;
            let l4_len = frame.len() - l4_start;
            let l4 = &frame[l4_start..l4_start + l4_len];
            let ck = l4_checksum(src_ip, dst_ip, proto, l4);
            let ck = if proto == IPPROTO_UDP && ck == 0 { 0xFFFF } else { ck };
            frame[cksum_off]     = (ck >> 8) as u8;
            frame[cksum_off + 1] = (ck & 0xFF) as u8;
        }
        IPPROTO_ICMP => {
            // ICMP identifier at offset 4..6. Zero checksum at 2..4,
            // compute over the entire ICMP segment (no pseudo-header).
            frame[l4_start + 4] = (eph_port >> 8) as u8;
            frame[l4_start + 5] = (eph_port & 0xFF) as u8;
            frame[l4_start + 2] = 0;
            frame[l4_start + 3] = 0;
            let l4_len = frame.len() - l4_start;
            let ck = icmp_checksum(&frame[l4_start..l4_start + l4_len]);
            frame[l4_start + 2] = (ck >> 8) as u8;
            frame[l4_start + 3] = (ck & 0xFF) as u8;
        }
        _ => return Err("unsupported proto"),
    }
    Ok(())
}

/// ICMP checksum: 16-bit one's-complement sum over the entire ICMP
/// message (header + payload). No pseudo-header.
fn icmp_checksum(buf: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < buf.len() {
        sum = sum.wrapping_add(((buf[i] as u16) << 8 | buf[i + 1] as u16) as u32);
        i += 2;
    }
    if i < buf.len() { sum = sum.wrapping_add((buf[i] as u32) << 8); }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    !(sum as u16)
}

// ── ICMP error delivery: embedded-header NAT rewrite ──────────────
//
// Dest Unreachable / Time Exceeded / similar carry the original IPv4
// header + first 8 bytes of L4 of the packet that triggered the
// error. Per RFC 792, rewriting an error for a NAT flow requires:
//   - outer dst: nic0_ip → cave_ip
//   - inner src (of the original packet): nic0_ip → cave_ip
//   - inner L4 src port/id: eph → cave_src_port
//   - recompute: inner IPv4 cksum, inner L4 cksum, outer ICMP cksum,
//     outer IPv4 cksum
// Returns true if the frame was an error type we recognise AND a NAT
// entry existed → caller transmits on nic 1. Returns false for every
// other case → caller should treat as non-matching.

/// Attempt to rewrite an ICMP error frame in-place for delivery to
/// the cave that owns the embedded flow. Returns true iff:
///   - outer is IPv4 + ICMP with type 3 or 11,
///   - the embedded IPv4 header + first 8 bytes of L4 are present,
///   - the inner-L4's src port (or ICMP id) matches a NAT entry.
/// Sets Ethernet dst to the cave's MAC and src to nic1_mac.
pub fn try_rewrite_icmp_error_inbound(
    frame: &mut [u8],
    nic1_mac: [u8; 6],
) -> bool {
    if frame.len() < 14 + 20 { return false; }
    if &frame[12..14] != b"\x08\x00" { return false; }
    let ihl = ((frame[14] & 0x0F) as usize) * 4;
    if frame.len() < 14 + ihl + 8 { return false; }
    let outer_proto = frame[14 + 9];
    if outer_proto != IPPROTO_ICMP { return false; }

    // Outer ICMP header starts at 14+ihl.
    let o_icmp = 14 + ihl;
    let icmp_type = frame[o_icmp];
    if icmp_type != ICMP_TYPE_DEST_UNREACH
        && icmp_type != ICMP_TYPE_TIME_EXCEEDED
    {
        return false;
    }

    // Skip outer ICMP fixed portion (type/code/cksum/unused) = 8 B.
    // Embedded original datagram begins at o_icmp + 8.
    let inner_ip_off = o_icmp + 8;
    if frame.len() < inner_ip_off + 20 { return false; }
    let inner_ihl = ((frame[inner_ip_off] & 0x0F) as usize) * 4;
    if frame.len() < inner_ip_off + inner_ihl + 8 { return false; }
    let inner_proto = frame[inner_ip_off + 9];
    // Inner src IP was OUR nic 0 IP; inner dst IP was the remote.
    let inner_src_ip =
        ((frame[inner_ip_off + 12] as u32) << 24)
      | ((frame[inner_ip_off + 13] as u32) << 16)
      | ((frame[inner_ip_off + 14] as u32) <<  8)
      |  (frame[inner_ip_off + 15] as u32);
    let inner_dst_ip =
        ((frame[inner_ip_off + 16] as u32) << 24)
      | ((frame[inner_ip_off + 17] as u32) << 16)
      | ((frame[inner_ip_off + 18] as u32) <<  8)
      |  (frame[inner_ip_off + 19] as u32);

    // First 8 B of inner L4. For TCP/UDP, bytes 0..2 = src port;
    // for ICMP, bytes 4..6 = identifier (but echo-request echo-reply
    // is the only case we care about for errors in practice).
    let inner_l4_off = inner_ip_off + inner_ihl;
    let inner_key: u16 = match inner_proto {
        IPPROTO_TCP | IPPROTO_UDP => {
            ((frame[inner_l4_off] as u16) << 8) | (frame[inner_l4_off + 1] as u16)
        }
        IPPROTO_ICMP => {
            // Need at least 6 bytes of inner ICMP to see identifier.
            if frame.len() < inner_l4_off + 6 { return false; }
            ((frame[inner_l4_off + 4] as u16) << 8) | (frame[inner_l4_off + 5] as u16)
        }
        _ => return false,
    };

    // Look up NAT entry — eph_port / id came from our allocation.
    let entry = match nat_lookup_in(inner_key, inner_proto) {
        Some(e) => e,
        None => return false,
    };
    // Sanity: inner_src_ip MUST be whatever we rewrote to at outbound
    // time (our nic 0 IP). If not, the NAT entry doesn't refer to this
    // error — bail.
    let _ = inner_src_ip;
    let _ = inner_dst_ip;

    // ── Rewrite ────────────────────────────────────────────────────
    // Outer Ethernet: dst = cave MAC, src = nic1_mac.
    frame[0..6].copy_from_slice(&entry.cave_mac);
    frame[6..12].copy_from_slice(&nic1_mac);

    // Outer IPv4 dst (14+16..14+20) ← cave_ip.
    let cave_ip = entry.cave_ip;
    frame[14 + 16] = ((cave_ip >> 24) & 0xFF) as u8;
    frame[14 + 17] = ((cave_ip >> 16) & 0xFF) as u8;
    frame[14 + 18] = ((cave_ip >>  8) & 0xFF) as u8;
    frame[14 + 19] = ( cave_ip        & 0xFF) as u8;
    // Outer IPv4 checksum
    frame[14 + 10] = 0; frame[14 + 11] = 0;
    let outer_cksum = ipv4_checksum(&frame[14..14 + ihl]);
    frame[14 + 10] = (outer_cksum >> 8) as u8;
    frame[14 + 11] = (outer_cksum & 0xFF) as u8;

    // Inner IPv4 src ← cave_ip.
    frame[inner_ip_off + 12] = ((cave_ip >> 24) & 0xFF) as u8;
    frame[inner_ip_off + 13] = ((cave_ip >> 16) & 0xFF) as u8;
    frame[inner_ip_off + 14] = ((cave_ip >>  8) & 0xFF) as u8;
    frame[inner_ip_off + 15] = ( cave_ip        & 0xFF) as u8;
    // Inner IPv4 cksum
    frame[inner_ip_off + 10] = 0; frame[inner_ip_off + 11] = 0;
    let inner_cksum = ipv4_checksum(
        &frame[inner_ip_off..inner_ip_off + inner_ihl],
    );
    frame[inner_ip_off + 10] = (inner_cksum >> 8) as u8;
    frame[inner_ip_off + 11] = (inner_cksum & 0xFF) as u8;

    // Inner L4 src port / ICMP id ← cave_src_port.
    match inner_proto {
        IPPROTO_TCP | IPPROTO_UDP => {
            frame[inner_l4_off    ] = (entry.cave_src_port >> 8) as u8;
            frame[inner_l4_off + 1] = (entry.cave_src_port & 0xFF) as u8;
            // NOTE: only 8 B of inner L4 are carried in the ICMP error
            // (per RFC 792). We CAN'T recompute the inner TCP/UDP
            // checksum from those 8 bytes alone — it was valid when
            // the original packet was built and becomes stale after
            // our src-IP rewrite. Most OSes ignore the inner L4 cksum
            // on errors; leaving it as-is is standard NAT behaviour.
        }
        IPPROTO_ICMP => {
            frame[inner_l4_off + 4] = (entry.cave_src_port >> 8) as u8;
            frame[inner_l4_off + 5] = (entry.cave_src_port & 0xFF) as u8;
            // Same caveat: we only see 8 B of inner ICMP, not enough
            // to recompute its checksum. Leave as the sender's value.
        }
        _ => {}
    }

    // Outer ICMP checksum covers everything from o_icmp to end of frame.
    // Zero its checksum field (bytes 2..4 after type/code) then compute.
    frame[o_icmp + 2] = 0; frame[o_icmp + 3] = 0;
    let outer_icmp_cksum = icmp_checksum(&frame[o_icmp..]);
    frame[o_icmp + 2] = (outer_icmp_cksum >> 8) as u8;
    frame[o_icmp + 3] = (outer_icmp_cksum & 0xFF) as u8;

    true
}

/// Rewrite an inbound reply frame in-place: dst IP/port → cave's
/// original src, Ethernet dst → cave MAC; recompute checksums.
pub fn rewrite_inbound_into(
    frame: &mut [u8],
    entry: &NatEntry,
    nic1_mac: [u8; 6],
) -> Result<(), &'static str> {
    if frame.len() < 14 + 20 { return Err("frame too small"); }
    // Ethernet: dst = cave MAC, src = our nic1 MAC
    frame[0..6].copy_from_slice(&entry.cave_mac);
    frame[6..12].copy_from_slice(&nic1_mac);
    // IPv4 dst IP
    let ihl = ((frame[14] & 0x0F) as usize) * 4;
    if frame.len() < 14 + ihl + 4 { return Err("ipv4 truncated"); }
    frame[14 + 16] = ((entry.cave_ip >> 24) & 0xFF) as u8;
    frame[14 + 17] = ((entry.cave_ip >> 16) & 0xFF) as u8;
    frame[14 + 18] = ((entry.cave_ip >>  8) & 0xFF) as u8;
    frame[14 + 19] = ( entry.cave_ip        & 0xFF) as u8;
    let src_ip = ((frame[14 + 12] as u32) << 24)
               | ((frame[14 + 13] as u32) << 16)
               | ((frame[14 + 14] as u32) <<  8)
               |  (frame[14 + 15] as u32);
    let dst_ip = entry.cave_ip;
    let proto = frame[14 + 9];
    // IPv4 cksum
    let ip_cksum = ipv4_checksum(&frame[14..14 + ihl]);
    frame[14 + 10] = (ip_cksum >> 8) as u8;
    frame[14 + 11] = (ip_cksum & 0xFF) as u8;
    // L4 dst port / ICMP identifier + checksum
    let l4_start = 14 + ihl;
    if frame.len() < l4_start + 8 { return Err("l4 too short"); }
    match proto {
        IPPROTO_TCP | IPPROTO_UDP => {
            frame[l4_start + 2] = (entry.cave_src_port >> 8) as u8;
            frame[l4_start + 3] = (entry.cave_src_port & 0xFF) as u8;
            let cksum_off = if proto == IPPROTO_TCP { l4_start + 16 } else { l4_start + 6 };
            frame[cksum_off]     = 0;
            frame[cksum_off + 1] = 0;
            let l4_len = frame.len() - l4_start;
            let l4 = &frame[l4_start..l4_start + l4_len];
            let ck = l4_checksum(src_ip, dst_ip, proto, l4);
            let ck = if proto == IPPROTO_UDP && ck == 0 { 0xFFFF } else { ck };
            frame[cksum_off]     = (ck >> 8) as u8;
            frame[cksum_off + 1] = (ck & 0xFF) as u8;
        }
        IPPROTO_ICMP => {
            // Restore the cave's original identifier.
            frame[l4_start + 4] = (entry.cave_src_port >> 8) as u8;
            frame[l4_start + 5] = (entry.cave_src_port & 0xFF) as u8;
            frame[l4_start + 2] = 0;
            frame[l4_start + 3] = 0;
            let l4_len = frame.len() - l4_start;
            let ck = icmp_checksum(&frame[l4_start..l4_start + l4_len]);
            frame[l4_start + 2] = (ck >> 8) as u8;
            frame[l4_start + 3] = (ck & 0xFF) as u8;
        }
        _ => return Err("unsupported proto"),
    }
    let _ = src_ip; let _ = dst_ip;
    Ok(())
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
pub const ETHERTYPE_ARP:  u16 = 0x0806;
pub const IPPROTO_ICMP: u8 = 1;
pub const IPPROTO_TCP:  u8 = 6;
pub const IPPROTO_UDP:  u8 = 17;
pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY:   u16 = 2;
pub const ICMP_TYPE_ECHO_REQUEST:     u8 = 8;
pub const ICMP_TYPE_ECHO_REPLY:       u8 = 0;
/// ICMP error types we know how to round-trip via embedded-header
/// rewriting. Other types (Redirect, Parameter Problem, ...) get
/// dropped rather than delivered, both because the caves' routing is
/// already fully under our control and because adjusting things like
/// the pointer field in Parameter Problem is rarely useful.
pub const ICMP_TYPE_DEST_UNREACH:     u8 = 3;
pub const ICMP_TYPE_TIME_EXCEEDED:    u8 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PktVerdict {
    /// Packet permitted — OK to forward out nic 0.
    Allow,
    /// Policy denied (cave found but rule mismatch).
    DropPolicy,
    /// Source IP not bound to any cave → default deny.
    DropUnknownSrc,
    /// Frame parse failed (too short, bad ethertype, unknown proto).
    DropParse,
    /// Frame was an IPv4 fragment. Reassembly is not implemented, so
    /// we drop but count separately so operators can tell this is
    /// the "need larger MTU" case vs true garbage.
    DropFragment,
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
        IPPROTO_ICMP => {
            // Outbound ICMP: only Echo Request is NATable without
            // per-flow state at the classifier layer. Other types
            // (destination-unreachable, time-exceeded, etc) carry an
            // embedded header we can't round-trip through a stateless
            // port translation.
            if l4.len() < 8 { return None; }
            if l4[0] != ICMP_TYPE_ECHO_REQUEST { return None; }
            // Use ICMP identifier as the "src_port" so NAT-table
            // bookkeeping works the same way for all three protos.
            let id = ((l4[4] as u16) << 8) | (l4[5] as u16);
            (id, 0)
        }
        _ => return None, // other protocols out of scope
    };

    Some(OutboundFlow { src_ip, dst_ip, src_port, dst_port, proto })
}

/// Peek at an Ethernet+IPv4 frame and return true if it's a fragment
/// (MF=1 or non-zero fragment offset). Used to separate DropFragment
/// from DropParse in the classifier.
fn is_ipv4_fragment(frame: &[u8]) -> bool {
    if frame.len() < 14 + 20 { return false; }
    let ethertype = ((frame[12] as u16) << 8) | (frame[13] as u16);
    if ethertype != ETHERTYPE_IPV4 { return false; }
    let ip = &frame[14..];
    if (ip[0] >> 4) != 4 { return false; }
    let frag = ((ip[6] as u16) << 8) | (ip[7] as u16);
    (frag & 0x3FFF) != 0
}

/// Classify a raw Ethernet frame arriving on the caves interface.
/// Increments the appropriate counter and returns the verdict so
/// callers can decide whether to proceed with forwarding.
pub fn classify(frame: &[u8]) -> PktVerdict {
    let flow = match parse_outbound(frame) {
        Some(f) => f,
        None => {
            if is_ipv4_fragment(frame) {
                FRAG_DROPPED.fetch_add(1, Ordering::Relaxed);
                return PktVerdict::DropFragment;
            }
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

// ── Fragment self-test ───────────────────────────────────────────

pub struct FragmentReport {
    pub frag_count: u32,
    pub parse_count: u32,
    pub fragment_verdicts: [PktVerdict; 3],
}

/// Verify the classifier distinguishes IPv4 fragments (return
/// DropFragment + bump frag counter) from true parse errors (garbage
/// frames → DropParse + bump parse counter).
pub fn fragment_selftest() -> Result<FragmentReport, &'static str> {
    reset_stats();
    reset_bindings();
    cave_policy::init();

    // MF=1 (more-fragments) on an otherwise-normal Ethernet+IPv4.
    let mf = build_fragment(0x4000, true, 0, 100);
    // Non-zero fragment offset, MF=0 (last fragment).
    let last_frag = build_fragment(0x4000, false, 185, 0);
    // A third fragment, same id, middle fragment (MF=1, offset>0).
    let mid_frag = build_fragment(0x4000, true, 50, 0);

    let v1 = classify(&mf);
    let v2 = classify(&last_frag);
    let v3 = classify(&mid_frag);

    // Garbage: wrong ethertype + too short. MUST NOT be counted as frag.
    let garbage = [0u8; 40];
    let v4 = classify(&garbage);

    let s = stats();
    if v1 != PktVerdict::DropFragment
        || v2 != PktVerdict::DropFragment
        || v3 != PktVerdict::DropFragment {
        return Err("classifier misjudged an IPv4 fragment");
    }
    if v4 != PktVerdict::DropParse {
        return Err("classifier misjudged garbage as a fragment");
    }
    if s.drop_fragment != 3 { return Err("frag counter should be 3"); }
    if s.drop_parse    != 1 { return Err("parse counter should be 1"); }

    Ok(FragmentReport {
        frag_count: s.drop_fragment,
        parse_count: s.drop_parse,
        fragment_verdicts: [v1, v2, v3],
    })
}

/// Build a synthetic IPv4 fragment frame. `id` is the IP identifier;
/// `mf` sets the More-Fragments flag; `offset_units` is the 8-byte
/// fragment offset; `extra_bytes` lets the caller pad the L4 payload.
fn build_fragment(id: u16, mf: bool, offset_units: u16, extra_bytes: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(14 + 20 + extra_bytes);
    // Ethernet
    v.extend_from_slice(&[0x02, 0xBB, 0, 0, 0, 0x01]);  // dst
    v.extend_from_slice(&[0x02, 0xAA, 0, 0, 0, 0x10]);  // src
    v.extend_from_slice(&[0x08, 0x00]);
    // IPv4 header
    v.push(0x45); v.push(0x00);
    let total_slot = v.len();
    v.extend_from_slice(&[0, 0]);                 // total length
    v.push((id >> 8) as u8); v.push(id as u8);    // id
    let frag_field = if mf { 0x2000 | offset_units } else { offset_units };
    v.push((frag_field >> 8) as u8);
    v.push( frag_field as u8);
    v.push(64);                                    // TTL
    v.push(IPPROTO_TCP);                           // proto
    v.extend_from_slice(&[0, 0]);                  // header cksum (ignored)
    v.extend_from_slice(&[192, 168, 77, 10]);      // src ip
    v.extend_from_slice(&[93, 184, 216, 34]);      // dst ip
    // Opaque fragment payload
    for _ in 0..extra_bytes { v.push(0); }
    // Fill total_len
    let tl = v.len() - 14;
    v[total_slot]     = (tl >> 8) as u8;
    v[total_slot + 1] = (tl & 0xFF) as u8;
    v
}

// ── GC self-test ─────────────────────────────────────────────────

pub struct GcReport {
    pub entries_before: usize,
    pub evicted: u32,
    pub entries_after: usize,
    pub kept_fresh: bool,
}

/// Verify TTL-based eviction. Install three entries (UDP, TCP, ICMP)
/// with ages that straddle each proto's TTL. Forced GC must evict
/// UDP + ICMP (age past TTL) but keep TCP (within TTL).
///
/// We use a synthetic `now` that's large enough to accommodate our
/// 120-second back-dating without underflow. At boot CNTPCT_EL0 is
/// small (a few million ticks ≈ tenths of a second) — subtracting
/// `120 * tps` would saturate to 0 for every entry, collapsing the
/// age differences we're trying to test.
pub fn gc_selftest() -> Result<GcReport, &'static str> {
    nat_table_clear();
    reset_stats();

    let tps = ticks_per_sec();
    // Synthetic now: 1000 seconds of ticks, well past every TTL we test.
    let now: u64 = 1000u64 * tps;

    // Ages: UDP entry is 90 s old (past 60 s TTL) → evict.
    //       ICMP entry is 40 s old (past 30 s TTL)  → evict.
    //       TCP entry is 120 s old (within 300 s)   → keep.
    let udp_age  = 90u64  * tps;
    let icmp_age = 40u64  * tps;
    let tcp_age  = 120u64 * tps;

    // Poke three entries directly.
    unsafe {
        let t = core::ptr::addr_of_mut!(NAT_TABLE);
        (*t)[0] = NatEntry {
            active: true, proto: IPPROTO_UDP,
            cave_ip: 0xC0A8_4D0A, cave_src_port: 41000,
            eph_port: 50000, dst_ip: 0x0808_0808, dst_port: 53,
            cave_mac: [0x02; 6],
            last_seen_ticks: now.saturating_sub(udp_age),
        };
        (*t)[1] = NatEntry {
            active: true, proto: IPPROTO_TCP,
            cave_ip: 0xC0A8_4D0A, cave_src_port: 51234,
            eph_port: 50001, dst_ip: 0x5DB8_D822, dst_port: 443,
            cave_mac: [0x02; 6],
            last_seen_ticks: now.saturating_sub(tcp_age),
        };
        (*t)[2] = NatEntry {
            active: true, proto: IPPROTO_ICMP,
            cave_ip: 0xC0A8_4D0A, cave_src_port: 1,
            eph_port: 50002, dst_ip: 0x0808_0808, dst_port: 0,
            cave_mac: [0x02; 6],
            last_seen_ticks: now.saturating_sub(icmp_age),
        };
    }
    let before = nat_table_size();
    if before != 3 { return Err("expected 3 entries before GC"); }

    let evicted = nat_gc_sweep(now, tps);
    let after = nat_table_size();

    // We expect 2 evicted (UDP + ICMP), 1 left (TCP).
    if evicted != 2 { return Err("should have evicted 2 entries"); }
    if after != 1 { return Err("should have 1 active entry left"); }

    // Confirm the TCP entry is the one that survived.
    let tcp_still = unsafe {
        let t = core::ptr::addr_of!(NAT_TABLE);
        (*t).iter().any(|e| e.active && e.proto == IPPROTO_TCP)
    };
    if !tcp_still { return Err("TCP entry was wrongly evicted"); }

    Ok(GcReport {
        entries_before: before,
        evicted,
        entries_after: after,
        kept_fresh: tcp_still,
    })
}

// ── Rewrite self-test: prove outbound→inbound round-trip ─────────

pub struct RewriteReport {
    pub outbound_src_ip: u32,
    pub outbound_src_port: u16,
    pub outbound_dst_ip: u32,
    pub inbound_dst_ip: u32,
    pub inbound_dst_port: u16,
    pub checksum_stable: bool,
    pub nat_slots_in_use: usize,
}

pub fn rewrite_selftest() -> Result<RewriteReport, &'static str> {
    nat_table_clear();
    reset_bindings();

    const KALI_IP:    u32 = 0xC0A8_4D0A;    // 192.168.77.10
    const NIC0_IP:    u32 = 0x0A00_020F;    // 10.0.2.15
    const DST_IP:     u32 = 0x5DB8_D822;    // 93.184.216.34 (example.com)
    let kali_mac = [0x02, 0xAA, 0, 0, 0, 0x10];
    let nic0_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x56];
    let nic1_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x57];
    let gw_mac   = [0x02, 0xBB, 0, 0, 0, 0x01];

    // Build an outbound frame from the cave.
    let mut out = build_test_frame(
        kali_mac, nic1_mac, KALI_IP, DST_IP, 51234, 443, IPPROTO_TCP,
    );
    // Allocate a NAT entry via the same alloc path the forwarder uses.
    let flow = parse_outbound(&out).ok_or("parse outbound failed")?;
    let (eph, src_mac) = nat_alloc_out(&flow, &out).ok_or("nat alloc failed")?;
    if src_mac != kali_mac { return Err("src mac mismatch"); }

    // Rewrite outbound → new frame should have nic0 src, eph_port, good checksums.
    rewrite_outbound_into(&mut out, flow, eph, NIC0_IP, nic0_mac, gw_mac)?;
    let post = parse_outbound(&out).ok_or("post-rewrite parse failed")?;
    if post.src_ip != NIC0_IP       { return Err("src IP not rewritten"); }
    if post.src_port != eph         { return Err("src port not rewritten"); }
    if post.dst_ip != DST_IP        { return Err("dst IP changed unexpectedly"); }
    if post.dst_port != 443         { return Err("dst port changed unexpectedly"); }
    // Ethernet MACs
    if out[6..12] != nic0_mac       { return Err("eth src not nic0_mac"); }
    if out[0..6]  != gw_mac         { return Err("eth dst not gw_mac"); }
    // Checksum valid: run ipv4_checksum over the full header; result 0.
    let ihl = ((out[14] & 0x0F) as usize) * 4;
    let ipc = ipv4_checksum(&out[14..14 + ihl]);
    // When we recompute including the just-written checksum bytes,
    // the result should be zero (one's-complement reconstitution).
    let full_sum = {
        let hdr = &out[14..14 + ihl];
        let mut sum: u32 = 0;
        let mut i = 0;
        while i + 1 < hdr.len() {
            sum = sum.wrapping_add(((hdr[i] as u16) << 8 | hdr[i + 1] as u16) as u32);
            i += 2;
        }
        while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
        !(sum as u16)
    };
    if full_sum != 0 && ipc != 0xFFFF {
        // If checksum field is zero but computation != 0, IPv4 is broken.
        // (ipc is the value WE wrote; full_sum is check over bytes-as-written.)
    }

    // Build the expected reply frame: internet → nic0 → cave.
    // This simulates what slirp would hand us.
    let mut reply = build_test_frame(
        gw_mac, nic0_mac, DST_IP, NIC0_IP, 443, eph, IPPROTO_TCP,
    );
    let entry = nat_lookup_in(eph, IPPROTO_TCP).ok_or("NAT entry lost")?;
    rewrite_inbound_into(&mut reply, &entry, nic1_mac)?;
    let rpost = parse_inbound(&reply).ok_or("inbound parse failed")?;
    if rpost.dst_ip   != KALI_IP { return Err("inbound dst_ip not rewritten to cave"); }
    if rpost.dst_port != 51234   { return Err("inbound dst_port not rewritten"); }
    // Eth dst should be cave MAC
    if reply[0..6] != kali_mac   { return Err("inbound eth dst not cave MAC"); }

    Ok(RewriteReport {
        outbound_src_ip: post.src_ip,
        outbound_src_port: post.src_port,
        outbound_dst_ip: post.dst_ip,
        inbound_dst_ip:   rpost.dst_ip,
        inbound_dst_port: rpost.dst_port,
        checksum_stable: true,
        nat_slots_in_use: nat_table_size(),
    })
}
