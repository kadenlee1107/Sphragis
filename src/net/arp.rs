// Bat_OS — ARP (Address Resolution Protocol)
// Resolves IPv4 addresses to MAC addresses on the local network.

use crate::drivers::virtio::net as netdev;
use super::ethernet;

const ARP_HW_ETHERNET: u16 = 1;
const ARP_OP_REQUEST: u16 = 1;
const ARP_OP_REPLY: u16 = 2;

// ARP cache
const ARP_CACHE_SIZE: usize = 16;
static mut ARP_CACHE: [(u32, [u8; 6], bool); ARP_CACHE_SIZE] = [(0, [0; 6], false); ARP_CACHE_SIZE];

pub fn handle_arp(data: &[u8]) {
    if data.len() < 28 { return; }

    let op = u16::from_be_bytes([data[6], data[7]]);
    let sender_mac = &data[8..14];
    let sender_ip = u32::from_be_bytes([data[14], data[15], data[16], data[17]]);
    let target_ip = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);

    // Cache sender's MAC
    let mut mac = [0u8; 6];
    mac.copy_from_slice(sender_mac);
    cache_put(sender_ip, mac);

    let our_ip = super::ip::our_ip();

    if op == ARP_OP_REQUEST && target_ip == our_ip {
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
        let len = ethernet::EthFrame::build(&mac, &our_mac, ethernet::ETHERTYPE_ARP, &reply, &mut frame);
        let _ = netdev::send(&frame[..len]);
    }
}

pub fn resolve(ip: u32) -> Option<[u8; 6]> {
    // Check cache first
    if let Some(mac) = cache_get(ip) {
        return Some(mac);
    }

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

    None
}

fn cache_put(ip: u32, mac: [u8; 6]) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(ARP_CACHE);
        for i in 0..ARP_CACHE_SIZE {
            if !(*ptr)[i].2 || (*ptr)[i].0 == ip {
                (*ptr)[i] = (ip, mac, true);
                return;
            }
        }
        (*ptr)[0] = (ip, mac, true);
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
