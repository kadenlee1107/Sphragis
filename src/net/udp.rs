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

    let src_port = u16::from_be_bytes([pkt.payload[0], pkt.payload[1]]);
    let payload = &pkt.payload[UDP_HDR_SIZE..];

    // Route to DNS handler if it's a DNS response (src port 53)
    if src_port == 53 {
        super::dns::handle_response(payload);
    }

    // Store in generic UDP RX buffer for syscall layer
    store_udp_response(payload);
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
    udp[6..8].copy_from_slice(&[0, 0]); // Checksum (optional for IPv4 UDP)
    udp[UDP_HDR_SIZE..UDP_HDR_SIZE + payload.len()].copy_from_slice(payload);

    ip::send(dst_ip, 17, &udp[..total])
}
