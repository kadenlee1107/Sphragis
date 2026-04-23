pub mod arp;
pub mod cave_policy;
pub mod cave_shaper;
pub mod dns;
pub mod ethernet;
pub mod firewall;
pub mod http;
pub mod icmp;
pub mod ip;
pub mod nat;
pub mod tcp;
pub mod tls;
pub mod tls_hybrid;
pub mod tls_pinning;
pub mod x509;
pub mod tor;
pub mod udp;
pub mod vpn;

use crate::drivers::virtio::net as netdev;

/// Poll the network device for one incoming packet and dispatch it.
pub fn poll_once() {
    let mut buf = [0u8; 1514];
    if let Some(len) = netdev::recv(&mut buf) {
        dispatch_host_frame(&buf[..len]);
    }
}

/// Dispatch an Ethernet frame into the kernel's own IPv4 stack.
/// Exposed so `net::nat::pump_replies` can fall back here when a frame
/// on nic 0 isn't a NAT reply — otherwise the NAT pump would silently
/// consume kernel control-plane traffic (daemon heartbeat, DNS, etc).
pub fn dispatch_host_frame(buf: &[u8]) {
    if let Some(frame) = ethernet::EthFrame::parse(buf) {
        match frame.ethertype {
            ethernet::ETHERTYPE_ARP  => arp::handle_arp(frame.payload),
            ethernet::ETHERTYPE_IPV4 => ip::handle(frame.payload),
            _ => {}
        }
    }
}

pub fn init() {
    firewall::init();
    cave_policy::init();
}
