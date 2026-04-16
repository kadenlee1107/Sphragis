#![allow(dead_code)]
// Bat_OS — UDP Layer
// Minimal UDP for DNS queries.

use super::ip::{self, IpPacket};

const UDP_HDR_SIZE: usize = 8;

pub struct UdpPacket<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: &'a [u8],
}

pub fn handle(pkt: &IpPacket) {
    if pkt.payload.len() < UDP_HDR_SIZE { return; }

    // Validate UDP length field matches what IPv4 delivered.
    let udp_len = u16::from_be_bytes([pkt.payload[4], pkt.payload[5]]) as usize;
    if udp_len < UDP_HDR_SIZE || udp_len > pkt.payload.len() { return; }

    // ATTACK-NET hardening: if the sender populated the UDP checksum, verify
    // it. A zero checksum is legal in IPv4 ("checksum disabled"); we don't
    // force the check in that case. This defeats trivially-malformed
    // spoofed DNS responses that don't bother computing a checksum.
    let stated_cksum = u16::from_be_bytes([pkt.payload[6], pkt.payload[7]]);
    if stated_cksum != 0 && !verify_udp_checksum(pkt, udp_len) {
        return;
    }

    let src_port = u16::from_be_bytes([pkt.payload[0], pkt.payload[1]]);

    // Narrow firewall check: only UDP src_ports explicitly allowed by the
    // firewall rules get through. With the default-deny config this is
    // src_port = 53 (DNS).
    if !super::firewall::allow_inbound_udp(pkt.src, src_port) {
        return;
    }

    let payload = &pkt.payload[UDP_HDR_SIZE..udp_len];

    // Route to DNS handler if it's a DNS response (src port 53)
    if src_port == 53 {
        super::dns::handle_response(payload);
    }

    // Store in generic UDP RX buffer for syscall layer
    store_udp_response(payload);
}

/// Verify the pseudo-header + UDP checksum. Returns true on success.
fn verify_udp_checksum(pkt: &IpPacket, udp_len: usize) -> bool {
    let udp = &pkt.payload[..udp_len];
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&pkt.src.to_be_bytes());
    pseudo[4..8].copy_from_slice(&pkt.dst.to_be_bytes());
    pseudo[9] = 17;
    pseudo[10..12].copy_from_slice(&(udp_len as u16).to_be_bytes());

    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < pseudo.len() {
        sum += u16::from_be_bytes([pseudo[i], pseudo[i + 1]]) as u32;
        i += 2;
    }
    i = 0;
    while i + 1 < udp.len() {
        sum += u16::from_be_bytes([udp[i], udp[i + 1]]) as u32;
        i += 2;
    }
    if i < udp.len() {
        sum += (udp[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    // Over a valid datagram the sum is 0xFFFF (i.e. !sum == 0). A stated
    // checksum of 0xFFFF on the wire is the encoded form of "sum was 0".
    (!(sum as u16)) == 0
}

/// Store a UDP response in the syscall layer's circular queue.
fn store_udp_response(data: &[u8]) {
    unsafe {
        let head = core::ptr::read_volatile(
            core::ptr::addr_of!(crate::batcave::linux::syscall::UDP_RX_HEAD));
        let slot = head % crate::batcave::linux::syscall::UDP_RX_SLOTS;
        let dst = core::ptr::addr_of_mut!(crate::batcave::linux::syscall::UDP_RX_BUF) as usize
            + slot * 512;
        let len = data.len().min(512);
        for i in 0..len {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) dst + i, v = in(reg) data[i] as u32);
        }
        crate::batcave::linux::syscall::UDP_RX_LEN[slot] = len;
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(crate::batcave::linux::syscall::UDP_RX_HEAD), head + 1);
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(crate::batcave::linux::syscall::UDP_RX_READY), true);
    }
}

/// Send a UDP packet.
pub fn send(dst_ip: u32, src_port: u16, dst_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let total = UDP_HDR_SIZE + payload.len();
    let mut udp = [0u8; 1400];

    udp[0..2].copy_from_slice(&src_port.to_be_bytes());
    udp[2..4].copy_from_slice(&dst_port.to_be_bytes());
    udp[4..6].copy_from_slice(&(total as u16).to_be_bytes());
    udp[6..8].copy_from_slice(&[0, 0]); // checksum placeholder
    udp[UDP_HDR_SIZE..UDP_HDR_SIZE + payload.len()].copy_from_slice(payload);

    // Compute checksum so peers that *do* verify accept our datagrams.
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&ip::our_ip().to_be_bytes());
    pseudo[4..8].copy_from_slice(&dst_ip.to_be_bytes());
    pseudo[9] = 17;
    pseudo[10..12].copy_from_slice(&(total as u16).to_be_bytes());
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < pseudo.len() {
        sum += u16::from_be_bytes([pseudo[i], pseudo[i + 1]]) as u32;
        i += 2;
    }
    i = 0;
    while i + 1 < total {
        sum += u16::from_be_bytes([udp[i], udp[i + 1]]) as u32;
        i += 2;
    }
    if i < total {
        sum += (udp[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let cksum = !(sum as u16);
    // RFC 768: a computed zero is transmitted as 0xFFFF.
    let cksum = if cksum == 0 { 0xFFFF } else { cksum };
    udp[6..8].copy_from_slice(&cksum.to_be_bytes());

    ip::send(dst_ip, 17, &udp[..total])
}
