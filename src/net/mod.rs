pub mod activity;
pub mod arp;
pub mod beacon;
pub mod cave_policy;
pub mod cave_shaper;
pub mod calipso;
pub mod cert_pin;
pub mod conntrack;
pub mod crl;
pub mod ct_logs;
pub mod ocsp;
pub mod ocsp_fixtures;
pub mod x509_fixtures;
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
// outright (would be re-added when real Tor work starts), `vpn`
// renamed to `psk_overlay`, and `psk_overlay` retired (2026-05-16,
// Week 14): no caller ever invoked `configure()`, so `is_active()`
// was permanently false; the AES-CTR path had no replay window so
// any future caller who flipped it on would have shipped a known
// replay weakness. Real overlay encryption now lives only in
// `wireguard` below.
// Gap-audit 043 phase 1 — real WireGuard. Spec-mandated Noise IK
// over X25519 + ChaCha20-Poly1305 + BLAKE2s; no UDP transport yet
// (phase 2). Self-tested end-to-end via `wg-selftest`.
pub mod wireguard;
pub mod wg_dispatch;

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

// ── isolation ─────────────────────────────────────────────────────
static NET_ISOLATED: AtomicBool = AtomicBool::new(true);

/// Returns whether the network is in isolated mode.
pub fn is_isolated() -> bool {
    NET_ISOLATED.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn set_isolation(isolated: bool) {
    NET_ISOLATED.store(isolated, Ordering::Relaxed);
}

// ── counters ──────────────────────────────────────────────────────
static RX_BYTES_TOTAL:    AtomicU64 = AtomicU64::new(0);
static TX_BYTES_TOTAL:    AtomicU64 = AtomicU64::new(0);
static PEAK_BYTES:        AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
static BOOT_SECS:         AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
static LAST_RX_BYTES:     AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
static LAST_TX_BYTES:     AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
static LAST_SAMPLE_SECS:  AtomicU64 = AtomicU64::new(0);

/// Called by virtio-net::recv on every received packet.
pub fn account_rx(len: usize) {
    let n = len as u64;
    RX_BYTES_TOTAL.fetch_add(n, Ordering::Relaxed);
    PEAK_BYTES.fetch_max(n, Ordering::Relaxed);
}

/// Called by virtio-net::send on every transmitted packet.
pub fn account_tx(len: usize) {
    let n = len as u64;
    TX_BYTES_TOTAL.fetch_add(n, Ordering::Relaxed);
    PEAK_BYTES.fetch_max(n, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn rx_rate() -> u32 { rate_delta(&RX_BYTES_TOTAL, &LAST_RX_BYTES) }
#[allow(dead_code)]
pub fn tx_rate() -> u32 { rate_delta(&TX_BYTES_TOTAL, &LAST_TX_BYTES) }

#[allow(dead_code)]
fn rate_delta(total: &AtomicU64, last: &AtomicU64) -> u32 {
    let now_total = total.load(Ordering::Relaxed);
    let last_total = last.swap(now_total, Ordering::Relaxed);
    let now_secs = crate::kernel::time::monotonic_secs();
    let last_secs = LAST_SAMPLE_SECS.swap(now_secs, Ordering::Relaxed);
    let elapsed = now_secs.saturating_sub(last_secs).max(1);
    let delta = now_total.saturating_sub(last_total);
    (delta / elapsed) as u32
}

#[allow(dead_code)]
pub fn peak_bytes() -> u64 {
    PEAK_BYTES.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn uptime_secs() -> u64 {
    let now = crate::kernel::time::monotonic_secs();
    let boot = BOOT_SECS.load(Ordering::Relaxed);
    if boot == 0 {
        BOOT_SECS.store(now, Ordering::Relaxed);
        return 0;
    }
    now.saturating_sub(boot)
}

#[allow(dead_code)]
pub fn clear_counters() {
    RX_BYTES_TOTAL.store(0, Ordering::Relaxed);
    TX_BYTES_TOTAL.store(0, Ordering::Relaxed);
    PEAK_BYTES.store(0, Ordering::Relaxed);
    LAST_RX_BYTES.store(0, Ordering::Relaxed);
    LAST_TX_BYTES.store(0, Ordering::Relaxed);
    LAST_SAMPLE_SECS.store(crate::kernel::time::monotonic_secs(), Ordering::Relaxed);
    activity::clear();
    activity::push(activity::ActivityKind::CountersCleared, "counters cleared");
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
