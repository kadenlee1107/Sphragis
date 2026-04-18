// Bat_OS — ARP (Address Resolution Protocol)
// Resolves IPv4 addresses to MAC addresses on the local network.
//
// ATTACK-NET-001 hardening: we only cache ARP replies whose sender_ip is in
// our pending-request queue. Gratuitous / unsolicited replies are rejected.
// Cache writes are rate-limited so a flood of legitimate-looking replies
// cannot evict the gateway entry in a tight loop.

use crate::drivers::virtio::net as netdev;
use super::ethernet;
use core::sync::atomic::{AtomicU64, Ordering};

const ARP_HW_ETHERNET: u16 = 1;
const ARP_OP_REQUEST: u16 = 1;
const ARP_OP_REPLY: u16 = 2;

// ARP cache
const ARP_CACHE_SIZE: usize = 16;
static mut ARP_CACHE: [(u32, [u8; 6], bool); ARP_CACHE_SIZE] = [(0, [0; 6], false); ARP_CACHE_SIZE];

// Pending-request queue: IPs we have sent an ARP request for and expect a
// reply to. Entries are cleared when a matching reply is cached or when the
// slot is overwritten by a newer request. Single-core, no lock needed.
const PENDING_SIZE: usize = 8;
static mut PENDING: [u32; PENDING_SIZE] = [0; PENDING_SIZE];

// Rate-limit successive cache updates. `cntpct_el0` ticks; cache accepts at
// most one update per entry per ~10ms wall-clock (≈cntfrq/100). We store the
// last-update tick per cache slot so noisy peers cannot churn the table.
static mut LAST_UPDATE_TICK: [u64; ARP_CACHE_SIZE] = [0; ARP_CACHE_SIZE];
static MIN_UPDATE_GAP: AtomicU64 = AtomicU64::new(0); // computed lazily

#[inline]
fn now_ticks() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

#[inline]
fn min_gap_ticks() -> u64 {
    let g = MIN_UPDATE_GAP.load(Ordering::Relaxed);
    if g != 0 { return g; }
    let freq: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq); }
    let g = freq / 100; // 10ms
    MIN_UPDATE_GAP.store(g, Ordering::Relaxed);
    g
}

fn pending_push(ip: u32) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(PENDING);
        // Dedupe: if already present, nothing to do.
        for i in 0..PENDING_SIZE {
            if (*ptr)[i] == ip { return; }
        }
        // Insert into first empty slot, else overwrite slot 0.
        for i in 0..PENDING_SIZE {
            if (*ptr)[i] == 0 {
                (*ptr)[i] = ip;
                return;
            }
        }
        (*ptr)[0] = ip;
    }
}

fn pending_take(ip: u32) -> bool {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(PENDING);
        for i in 0..PENDING_SIZE {
            if (*ptr)[i] == ip {
                (*ptr)[i] = 0;
                return true;
            }
        }
    }
    false
}

pub fn handle_arp(data: &[u8]) {
    if data.len() < 28 { return; }

    let op = u16::from_be_bytes([data[6], data[7]]);
    let sender_mac = &data[8..14];
    let sender_ip = u32::from_be_bytes([data[14], data[15], data[16], data[17]]);
    let target_ip = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);

    let mut mac = [0u8; 6];
    mac.copy_from_slice(sender_mac);

    let our_ip = super::ip::our_ip();

    match op {
        ARP_OP_REQUEST => {
            // A REQUEST with sender_ip=our_ip would be an announcement/collision
            // probe — not a reply — so it's OK to *consider* caching the
            // sender's MAC only when the request is directly addressed to us
            // (implying bi-directional L2 reachability). We still require the
            // target to match our_ip before we reply or cache.
            if target_ip == our_ip {
                // Cache the sender so the reply path can find them. This is
                // safe because a REQUEST aimed at us is not the spoofing
                // vector — the exploit is a gratuitous REPLY claiming to be
                // the gateway. Rate-limited.
                cache_put_rl(sender_ip, mac);

                // Reply
                let our_mac = netdev::mac();
                let mut reply = [0u8; 28];
                reply[0..2].copy_from_slice(&ARP_HW_ETHERNET.to_be_bytes());
                reply[2..4].copy_from_slice(&ethernet::ETHERTYPE_IPV4.to_be_bytes());
                reply[4] = 6; // HW addr len
                reply[5] = 4; // Protocol addr len
                reply[6..8].copy_from_slice(&ARP_OP_REPLY.to_be_bytes());
                reply[8..14].copy_from_slice(&our_mac);
                reply[14..18].copy_from_slice(&our_ip.to_be_bytes());
                reply[18..24].copy_from_slice(&mac);
                reply[24..28].copy_from_slice(&sender_ip.to_be_bytes());

                let mut frame = [0u8; 42];
                let len = ethernet::EthFrame::build(
                    &mac, &our_mac, ethernet::ETHERTYPE_ARP, &reply, &mut frame);
                let _ = netdev::send(&frame[..len]);
            }
        }

        ARP_OP_REPLY => {
            // ATTACK-NET-001: only cache REPLY packets for IPs we actually
            // asked about. An unsolicited reply (gratuitous ARP claiming to
            // be the gateway) is silently dropped. Also require the target
            // IP to match our_ip to filter replies intended for other hosts.
            if target_ip != our_ip {
                return;
            }
            if !pending_take(sender_ip) {
                // Unsolicited — log once, drop. We don't spam uart.
                return;
            }
            cache_put_rl(sender_ip, mac);
        }

        _ => {}
    }
}

/// V8-ROOT-2: clear the ARP cache, pending list, and rate-limit ticks on
/// cave switch. Without this, a new cave inherits the previous cave's
/// MAC-IP mapping cache — a cross-cave network-topology leak.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let cache = &mut *core::ptr::addr_of_mut!(ARP_CACHE);
        for slot in cache.iter_mut() { *slot = (0, [0; 6], false); }
        let pending = &mut *core::ptr::addr_of_mut!(PENDING);
        for slot in pending.iter_mut() { *slot = 0; }
        let ticks = &mut *core::ptr::addr_of_mut!(LAST_UPDATE_TICK);
        for slot in ticks.iter_mut() { *slot = 0; }
    }
}

pub fn resolve(ip: u32) -> Option<[u8; 6]> {
    // Check cache first
    if let Some(mac) = cache_get(ip) {
        return Some(mac);
    }

    // Register the outstanding request so the reply handler accepts the
    // eventual response.
    pending_push(ip);

    // Send ARP request
    let our_mac = netdev::mac();
    let our_ip = super::ip::our_ip();

    let mut arp = [0u8; 28];
    arp[0..2].copy_from_slice(&ARP_HW_ETHERNET.to_be_bytes());
    arp[2..4].copy_from_slice(&ethernet::ETHERTYPE_IPV4.to_be_bytes());
    arp[4] = 6;
    arp[5] = 4;
    arp[6..8].copy_from_slice(&ARP_OP_REQUEST.to_be_bytes());
    arp[8..14].copy_from_slice(&our_mac);
    arp[14..18].copy_from_slice(&our_ip.to_be_bytes());
    arp[18..24].copy_from_slice(&[0; 6]); // Target MAC unknown
    arp[24..28].copy_from_slice(&ip.to_be_bytes());

    let mut frame = [0u8; 42];
    let len = ethernet::EthFrame::build(&ethernet::BROADCAST, &our_mac, ethernet::ETHERTYPE_ARP, &arp, &mut frame);
    let _ = netdev::send(&frame[..len]);

    // Wait for reply — send multiple requests and poll aggressively
    for attempt in 0..5 {
        // Re-send ARP request each attempt
        if attempt > 0 {
            let mut frame2 = [0u8; 42];
            let len2 = ethernet::EthFrame::build(&ethernet::BROADCAST, &our_mac, ethernet::ETHERTYPE_ARP, &arp, &mut frame2);
            let _ = netdev::send(&frame2[..len2]);
        }

        for _ in 0..5_000_000 {
            super::poll_once();
            if let Some(mac) = cache_get(ip) {
                return Some(mac);
            }
            core::hint::spin_loop();
        }
    }

    // Timed out — drop the pending entry so a stray late reply cannot
    // poison the cache.
    pending_take(ip);
    None
}

fn cache_put_rl(ip: u32, mac: [u8; 6]) {
    let now = now_ticks();
    let gap = min_gap_ticks();
    unsafe {
        let ptr = core::ptr::addr_of_mut!(ARP_CACHE);
        let tick_ptr = core::ptr::addr_of_mut!(LAST_UPDATE_TICK);
        // Prefer updating the existing entry, otherwise first empty slot.
        for i in 0..ARP_CACHE_SIZE {
            if (*ptr)[i].2 && (*ptr)[i].0 == ip {
                let last = (*tick_ptr)[i];
                if now.wrapping_sub(last) < gap {
                    return; // rate-limited
                }
                (*ptr)[i] = (ip, mac, true);
                (*tick_ptr)[i] = now;
                return;
            }
        }
        for i in 0..ARP_CACHE_SIZE {
            if !(*ptr)[i].2 {
                (*ptr)[i] = (ip, mac, true);
                (*tick_ptr)[i] = now;
                return;
            }
        }
        // Cache full — reuse slot 0. Still rate-limit to avoid thrash.
        let last = (*tick_ptr)[0];
        if now.wrapping_sub(last) < gap {
            return;
        }
        (*ptr)[0] = (ip, mac, true);
        (*tick_ptr)[0] = now;
    }
}

fn cache_get(ip: u32) -> Option<[u8; 6]> {
    unsafe {
        let ptr = core::ptr::addr_of!(ARP_CACHE);
        for i in 0..ARP_CACHE_SIZE {
            if (*ptr)[i].2 && (*ptr)[i].0 == ip {
                return Some((*ptr)[i].1);
            }
        }
    }
    None
}
