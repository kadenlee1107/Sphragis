pub mod arp;
pub mod dns;
pub mod ethernet;
pub mod firewall;
pub mod http;
pub mod icmp;
pub mod ip;
pub mod tcp;
pub mod tls;
pub mod tls_pinning;
pub mod tor;
pub mod udp;
pub mod vpn;

use crate::drivers::virtio::net as netdev;

/// Poll the network device for one incoming packet and dispatch it.
pub fn poll_once() {
    let mut buf = [0u8; 1514];
    if let Some(len) = netdev::recv(&mut buf) {
        if let Some(frame) = ethernet::EthFrame::parse(&buf[..len]) {
            match frame.ethertype {
                ethernet::ETHERTYPE_ARP => arp::handle_arp(frame.payload),
                ethernet::ETHERTYPE_IPV4 => ip::handle(frame.payload),
                _ => {}
            }
        }
    }
}

pub fn init() {
    firewall::init();
}
