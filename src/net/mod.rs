pub mod arp;
pub mod beacon;
pub mod cave_policy;
pub mod cave_shaper;
pub mod cert_pin;
pub mod crl;
pub mod ct_logs;
pub mod dot;
pub mod cookies;
pub mod dns;
pub mod flow_shaper;
pub mod ethernet;
pub mod fetch;
pub mod firewall;
pub mod http;
pub mod https;
pub mod icmp;
pub mod ip;
pub mod nat;
pub mod tcp;
pub mod tls;
pub mod tls_hybrid;
pub mod x509;
pub mod udp;
// was `tor` (3-layer CTR with hardcoded keys, NOT real
// Tor — no directory consensus, no relay discovery) and `vpn`
// (PSK-derived AES-CTR overlay, NOT WireGuard — no Noise IK, no
// rekey). The audit caught both names as misleading. `tor` deleted
// outright (would be re-added when real Tor work starts), and `vpn`
// renamed to `psk_overlay` to honestly describe what it does.
pub mod psk_overlay;
// Gap-audit 043 phase 1 — real WireGuard. Spec-mandated Noise IK
// over X25519 + ChaCha20-Poly1305 + BLAKE2s; no UDP transport yet
// (phase 2). Self-tested end-to-end via `wg-selftest`.
pub mod wireguard;

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
    // Per DESIGN_TLS_HARDENING.md: chain-only strict TLS, hybrid PQ
    // on, no fallback paths. With an empty TRUST_STORE the kernel
    // refuses every HTTPS peer; warn loudly at boot so an operator
    // can't silently deploy without populating CA roots.
    let n_trust = crate::net::x509::TRUST_STORE.len();
    crate::drivers::uart::puts("  [tls] trust store: ");
    crate::kernel::mm::print_num(n_trust);
    crate::drivers::uart::puts(" CA roots, chain-only auth, hybrid PQ on\n");
    if n_trust == 0 {
        crate::drivers::uart::puts("  [tls] WARNING: trust store empty — HTTPS will refuse all peers\n");
    }
}
