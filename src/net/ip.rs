// Bat_OS — IPv4 Layer
// Handles IP packet construction and parsing.
// QEMU user-mode networking: gateway 10.0.2.2, our IP 10.0.2.15

use crate::drivers::virtio::net as netdev;
use super::{ethernet, arp};
use core::sync::atomic::{AtomicU32, AtomicU16, Ordering};

const IP_HDR_SIZE: usize = 20;
const PROTO_ICMP: u8 = 1;
const PROTO_TCP: u8 = 6;
const PROTO_UDP: u8 = 17;

// QEMU user-mode networking defaults
static OUR_IP: AtomicU32 = AtomicU32::new(0x0A00020F);    // 10.0.2.15
static GATEWAY: AtomicU32 = AtomicU32::new(0x0A000202);    // 10.0.2.2
static SUBNET_MASK: AtomicU32 = AtomicU32::new(0xFFFFFF00); // 255.255.255.0
static IP_ID: AtomicU16 = AtomicU16::new(1);

pub fn our_ip() -> u32 {
    OUR_IP.load(Ordering::Relaxed)
}

pub fn gateway() -> u32 {
    GATEWAY.load(Ordering::Relaxed)
}

pub struct IpPacket<'a> {
    pub src: u32,
    pub dst: u32,
    pub protocol: u8,
    pub payload: &'a [u8],
    pub ttl: u8,
}

impl<'a> IpPacket<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < IP_HDR_SIZE { return None; }

        let version = data[0] >> 4;
        if version != 4 { return None; }

        let ihl = (data[0] & 0x0F) as usize * 4;
        let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        let protocol = data[9];
        let src = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
        let dst = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);

        // ATTACK-NET-004/005/006: validate IHL against the buffer and total_len
        // *before* indexing. A crafted packet with ihl < 20 feeds a garbage
        // header, and ihl > total_len causes a slice panic (remote kernel DoS).
        if ihl < IP_HDR_SIZE { return None; }
        if total_len < ihl { return None; }
        if total_len > data.len() { return None; }

        // ATTACK-NET-008: verify the IPv4 header checksum. Over a valid header
        // the one's-complement sum is 0xFFFF (!=0 after the final complement).
        if checksum(&data[..ihl]) != 0 { return None; }

        Some(Self {
            src, dst, protocol,
            payload: &data[ihl..total_len],
            ttl: data[8],
        })
    }
}

/// Send an IP packet.
pub fn send(dst_ip: u32, protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    // Secure pipeline: if VPN or Tor is active, wrap the payload
    let mut secured = [0u8; 1400];
    let (final_payload, final_len) = if super::tor::is_ready() {
        // Full pipeline: Tor(VPN(payload))
        let mut vpn_buf = [0u8; 1400];
        let vpn_len = super::vpn::encrypt_packet(payload, &mut vpn_buf);
        let tor_len = super::tor::onion_encrypt(&vpn_buf[..vpn_len], &mut secured);
        (&secured[..], tor_len)
    } else if super::vpn::is_active() {
        // VPN only
        let vpn_len = super::vpn::encrypt_packet(payload, &mut secured);
        (&secured[..], vpn_len)
    } else {
        // Direct — no encryption (or TLS handles it at TCP level)
        (payload, payload.len())
    };
    let payload = &final_payload[..final_len];

    let src_ip = our_ip();
    let id = IP_ID.load(Ordering::Relaxed); IP_ID.store(id.wrapping_add(1), Ordering::Relaxed);

    // Build IP header
    let total_len = (IP_HDR_SIZE + payload.len()) as u16;
    let mut ip_pkt = [0u8; 1500];

    ip_pkt[0] = 0x45; // Version 4, IHL 5
    ip_pkt[1] = 0;    // DSCP/ECN
    ip_pkt[2..4].copy_from_slice(&total_len.to_be_bytes());
    ip_pkt[4..6].copy_from_slice(&id.to_be_bytes());
    ip_pkt[6] = 0x40; // Don't fragment
    ip_pkt[7] = 0;
    ip_pkt[8] = 64;   // TTL
    ip_pkt[9] = protocol;
    // Checksum at [10..12] — computed below
    ip_pkt[12..16].copy_from_slice(&src_ip.to_be_bytes());
    ip_pkt[16..20].copy_from_slice(&dst_ip.to_be_bytes());

    // Copy payload
    ip_pkt[IP_HDR_SIZE..IP_HDR_SIZE + payload.len()].copy_from_slice(payload);

    // Compute header checksum
    let cksum = checksum(&ip_pkt[..IP_HDR_SIZE]);
    ip_pkt[10..12].copy_from_slice(&cksum.to_be_bytes());

    // Determine next-hop MAC
    let mask = SUBNET_MASK.load(Ordering::Relaxed);
    let next_hop = if (dst_ip & mask) == (src_ip & mask) {
        dst_ip // Same subnet, send directly
    } else {
        gateway() // Different subnet, send to gateway
    };

    let dst_mac = arp::resolve(next_hop).ok_or("ARP resolve failed")?;
    let src_mac = netdev::mac();

    // Build Ethernet frame
    let mut frame = [0u8; 1514];
    let frame_len = ethernet::EthFrame::build(
        &dst_mac, &src_mac, ethernet::ETHERTYPE_IPV4,
        &ip_pkt[..total_len as usize], &mut frame,
    );

    netdev::send(&frame[..frame_len])
}

/// Handle an incoming IP packet.
pub fn handle(data: &[u8]) {
    if let Some(pkt) = IpPacket::parse(data) {
        // Check firewall
        if !super::firewall::allow_inbound(pkt.src, pkt.dst, pkt.protocol) {
            // Debug: show blocked packets during TCP connect
            if pkt.protocol == 6 {
                crate::drivers::uart::puts("[fw] BLOCKED TCP from ");
                crate::kernel::mm::print_num(((pkt.src >> 24) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num(((pkt.src >> 16) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num(((pkt.src >> 8) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num((pkt.src & 0xFF) as usize);
                crate::drivers::uart::puts("\n");
            }
            return;
        }

        // Secure pipeline: decrypt inbound if VPN/Tor active
        let mut decrypted_payload = [0u8; 1400];
        let decrypted_pkt = if super::tor::is_ready() {
            // Peel Tor, then VPN
            let mut tor_out = [0u8; 1400];
            let tor_len = super::tor::onion_decrypt(pkt.payload, &mut tor_out);
            let vpn_len = super::vpn::decrypt_packet(&tor_out[..tor_len], &mut decrypted_payload);
            Some(IpPacket {
                src: pkt.src, dst: pkt.dst, protocol: pkt.protocol, ttl: pkt.ttl,
                payload: &decrypted_payload[..vpn_len],
            })
        } else if super::vpn::is_active() {
            let vpn_len = super::vpn::decrypt_packet(pkt.payload, &mut decrypted_payload);
            Some(IpPacket {
                src: pkt.src, dst: pkt.dst, protocol: pkt.protocol, ttl: pkt.ttl,
                payload: &decrypted_payload[..vpn_len],
            })
        } else {
            None
        };
        let pkt_ref = decrypted_pkt.as_ref().unwrap_or(&pkt);

        match pkt_ref.protocol {
            PROTO_ICMP => super::icmp::handle(pkt_ref),
            PROTO_UDP => super::udp::handle(pkt_ref),
            PROTO_TCP => super::tcp::handle_incoming(pkt_ref),
            _ => {}
        }
    }
}

pub fn checksum(data: &[u8]) -> u16 {
    // V8-ROOT-3: one's-complement summation tolerates wrap-around by
    // construction (the carry is folded back below). Use wrapping_add so
    // overflow-checks=true does not panic on long buffers.
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum = sum.wrapping_add(u16::from_be_bytes([data[i], data[i + 1]]) as u32);
        i += 2;
    }
    if i < data.len() {
        sum = sum.wrapping_add((data[i] as u32) << 8);
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

pub fn ip_to_str(ip: u32, buf: &mut [u8; 16]) -> usize {
    let octets = ip.to_be_bytes();
    let mut pos = 0;
    for (i, &oct) in octets.iter().enumerate() {
        let mut n = oct;
        if n >= 100 { buf[pos] = b'0' + n / 100; pos += 1; n %= 100; }
        if n >= 10 || oct >= 100 { buf[pos] = b'0' + n / 10; pos += 1; n %= 10; }
        buf[pos] = b'0' + n; pos += 1;
        if i < 3 { buf[pos] = b'.'; pos += 1; }
    }
    pos
}
