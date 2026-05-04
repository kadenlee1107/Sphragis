pub mod arp;
pub mod beacon;
pub mod cave_policy;
pub mod cave_shaper;
pub mod cookies;
pub mod dns;
pub mod flow_shaper;
pub mod ethernet;
pub mod fetch;
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
pub mod udp;
// STUMP #141: was `tor` (3-layer CTR with hardcoded keys, NOT real
// Tor — no directory consensus, no relay discovery) and `vpn`
// (PSK-derived AES-CTR overlay, NOT WireGuard — no Noise IK, no
// rekey). The audit caught both names as misleading. `tor` deleted
// outright (would be re-added when real Tor work starts), and `vpn`
// renamed to `psk_overlay` to honestly describe what it does.
pub mod psk_overlay;

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
    // STUMP #111 (audit C002 / C010): the TLS stack ships with an
    // empty TRUST_STORE and an empty PINS list. With both empty the
    // verification chain reduces to "accept anything" in Research
    // mode — encrypted-but-not-authenticated. Warn loudly at boot so
    // an operator deploying production-class workloads can't silently
    // miss this. Populate src/net/x509.rs::TRUST_STORE with CA roots
    // OR src/net/tls_pinning.rs::PINS with per-host SHA-256 pins
    // before relying on TLS for authentication.
    let n_trust = crate::net::x509::TRUST_STORE.len();
    let n_pins = crate::net::tls_pinning::PINS.len();
    if n_trust == 0 && n_pins == 0 {
        crate::drivers::uart::puts("\n");
        crate::drivers::uart::puts("================================================================\n");
        crate::drivers::uart::puts("  WARNING: TRUST_STORE and PINS are both empty.\n");
        crate::drivers::uart::puts("  TLS connections in Research mode (renderer's default) accept\n");
        crate::drivers::uart::puts("  ANY certificate from ANY MITM. This is encrypted-but-not-\n");
        crate::drivers::uart::puts("  authenticated. Do NOT use this for credential exchange.\n");
        crate::drivers::uart::puts("  Populate src/net/x509.rs::TRUST_STORE or src/net/tls_pinning\n");
        crate::drivers::uart::puts("  ::PINS before deploying.\n");
        crate::drivers::uart::puts("================================================================\n\n");
    } else {
        crate::drivers::uart::puts("  [tls] trust store: ");
        crate::kernel::mm::print_num(n_trust);
        crate::drivers::uart::puts(" CA roots, ");
        crate::kernel::mm::print_num(n_pins);
        crate::drivers::uart::puts(" host pins\n");
    }
}
