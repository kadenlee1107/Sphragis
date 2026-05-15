#![allow(dead_code)]
// Sphragis — Allowlist Firewall (real default-deny)
//
// The previous version installed a wildcard "allow any inbound TCP/UDP/ICMP"
// rule, which made the "DEFAULT DENY ALL" label meaningless. We now install
// narrow allow rules for the protocols Sphragis actually uses as a client:
// ICMP echo reply (ping response)
// TCP from port 443 (HTTPS responses)
// TCP from port 80 (HTTP responses, DoH fallback)
// UDP from port 53 (DNS responses)
//
// Rules match on direction + protocol + src_port (where relevant). Inbound
// packets on any other 4-tuple are dropped and counted as blocked.

use crate::drivers::uart;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

const MAX_RULES: usize = 32;

#[derive(Clone, Copy)]
struct FirewallRule {
    active: bool,
    direction: u8,    // 0 = inbound, 1 = outbound
    protocol: u8,     // 0 = any, 1 = ICMP, 6 = TCP, 17 = UDP
    ip: u32,          // 0 = any
    /// Source port match.
    /// Inbound rules: matches segment's src_port (the remote end).
    /// Outbound rules: matches segment's dst_port (the remote end).
    /// 0 = any.
    port: u16,
    /// destination-port match for inbound rules.
    /// Used by per-listener rules to allow inbound traffic only to
    /// ports a listener has been registered on. 0 = any (legacy
    /// wildcard behavior; use only for explicit defaults).
    /// Outbound rules ignore this field today.
    dst_port: u16,
}

static mut RULES: [FirewallRule; MAX_RULES] = [FirewallRule {
    active: false, direction: 0, protocol: 0, ip: 0, port: 0, dst_port: 0,
}; MAX_RULES];

static FIREWALL_ENABLED: AtomicBool = AtomicBool::new(true);
static BLOCKED_COUNT: AtomicU32 = AtomicU32::new(0);
static ALLOWED_COUNT: AtomicU32 = AtomicU32::new(0);
static RULE_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    // Clear any stale rules (idempotent init).
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RULES);
        for i in 0..MAX_RULES {
            (*ptr)[i].active = false;
        }
    }
    RULE_COUNT.store(0, Ordering::Relaxed);

    // Outbound: we initiate, so allow all outbound.
    add_rule(1, 0, 0, 0, 0);

    // Inbound narrow allows.
    // ICMP responses (ping replies are type 0). We don't filter by type
    // here; ICMP handler itself only replies to echo requests.
    add_rule(0, 1, 0, 0, 0);
    // gap-audit 045 hardening pass: the old wildcard
    //   `add_rule(0, 6, 0, 0, 0)`
    // (any inbound TCP) is GONE. Inbound TCP now passes only if
    // ONE of these is true (see `allow_inbound_tcp`):
    //   (a) `conntrack::lookup_inbound` finds an outbound-recorded
    //       flow matching the 4-tuple — reply traffic to a
    //       Sphragis-initiated connection.
    //   (b) `tcp::listener_lookup_by_port(dst_port)` says a
    //       server is listening on that port — server-side
    //       handshake / cookie-recovery path.
    //   (c) An explicit per-port rule installed via
    //       `allow_inbound_tcp_dst_port` (defense in depth — still
    //       used by `listen_register`).
    // Unsolicited SYNs to random ephemeral ports get dropped at
    // the firewall before tcp::handle sees them.
    //
    // UDP: only DNS responses (src_port = 53). This closes
    // ATTACK-NET-041 for any non-DNS port.
    add_rule(0, 17, 0, 53, 0);

    let n = RULE_COUNT.load(Ordering::Relaxed);
    uart::puts("  [firewall] default-deny installed; ");
    crate::kernel::mm::print_num(n as usize);
    uart::puts(" allow rules active (out:any, in:ICMP, in:UDP/53; in:TCP gated on conntrack+listener)\n");
}

fn add_rule(direction: u8, protocol: u8, ip: u32, port: u16, dst_port: u16) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RULES);
        for i in 0..MAX_RULES {
            if !(*ptr)[i].active {
                (*ptr)[i] = FirewallRule { active: true, direction, protocol, ip, port, dst_port };
                RULE_COUNT.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
    }
}

/// install an inbound TCP allow rule for a specific
/// destination port. Called by `tcp::listen_register` when a listener
/// comes up so traffic to that port is explicitly allowed.
// /
/// Currently redundant with the wildcard inbound TCP rule above (which
/// allows everything), but in defense-in-depth mode this rule survives
/// when the wildcard is removed in a future hardening pass.
// /
/// Idempotent: if a rule for (proto=TCP, dst_port=N) already exists,
/// no-op. Returns false if the rule table is full.
pub fn allow_inbound_tcp_dst_port(port: u16) -> bool {
    if port == 0 { return false; }
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        // Idempotent — skip if already present.
        for i in 0..MAX_RULES {
            let r = &(*ptr)[i];
            if r.active && r.direction == 0 && r.protocol == 6
                && r.ip == 0 && r.port == 0 && r.dst_port == port {
                return true;
            }
        }
    }
    add_rule(0, 6, 0, 0, port);
    true
}

/// remove an inbound TCP allow rule installed by
/// `allow_inbound_tcp_dst_port`. Called by `tcp::listen_close`.
/// Idempotent — silent no-op if no matching rule exists.
pub fn revoke_inbound_tcp_dst_port(port: u16) {
    if port == 0 { return; }
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RULES);
        for i in 0..MAX_RULES {
            let r = &mut (*ptr)[i];
            if r.active && r.direction == 0 && r.protocol == 6
                && r.ip == 0 && r.port == 0 && r.dst_port == port {
                r.active = false;
                RULE_COUNT.fetch_sub(1, Ordering::Relaxed);
                return;
            }
        }
    }
}

/// Check inbound policy. The IP layer calls this before the transport
/// header has been parsed, so it currently matches on src_ip and protocol
/// only; UDP-port matching is enforced in `allow_inbound_udp`.
pub fn allow_inbound(src_ip: u32, _dst_ip: u32, protocol: u8) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }

    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        for i in 0..MAX_RULES {
            let rule = &(*ptr)[i];
            if !rule.active { continue; }
            if rule.direction != 0 { continue; }

            let proto_match = rule.protocol == 0 || rule.protocol == protocol;
            let ip_match = rule.ip == 0 || rule.ip == src_ip;

            if proto_match && ip_match {
                // For UDP, defer the port check to allow_inbound_udp —
                // otherwise any UDP would pass here.
                if protocol == 17 && rule.port != 0 {
                    // This rule has a port; we can't verify it here. Allow
                    // for now and rely on the UDP handler to re-check.
                    ALLOWED_COUNT.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
                ALLOWED_COUNT.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
    }

    BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
    crate::net::activity::push(
        crate::net::activity::ActivityKind::FwDrop,
        "fw drop",
    );
    false
}

/// Transport-layer port check for TCP (called after parsing the header).
///
/// Stateful policy (gap-audit 045 hardening pass). The wildcard
/// inbound TCP rule from `init` is gone; this function now permits
/// a segment iff ONE of these holds:
///
///   (a) `conntrack::lookup_inbound` finds a flow matching the
///       4-tuple — i.e. Sphragis already initiated this connection
///       and the inbound packet is reply traffic.
///   (b) `tcp::listener_lookup_by_port(dst_port)` reports a
///       registered listener — i.e. we're a server expecting
///       inbound SYNs / cookie-recovery ACKs on this port.
///   (c) An explicit per-IP/per-port rule was installed (e.g.
///       `allow_inbound_tcp_dst_port` from `listen_register`).
///       Defense in depth: the rule scan still happens last so
///       admin overrides keep working.
///
/// Unsolicited SYNs to ephemeral ports — the class of packet the
/// old wildcard let through — get dropped here.
pub fn allow_inbound_tcp(src_ip: u32, src_port: u16, dst_port: u16) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }

    // (a) Outbound-initiated flow — connect_start registered it in
    // conntrack as State::New, connect_blocking_pcb upgraded it to
    // Established on SYN-ACK. Either state is good enough to let
    // reply traffic through.
    if crate::net::conntrack::lookup_inbound(6, src_ip, src_port, dst_port).is_some() {
        return true;
    }

    // (b) Server-side: a listener is registered on this dst_port,
    // so inbound segments (SYN, ACK-with-cookie) are part of an
    // anticipated handshake. tcp::handle decides if the segment
    // is well-formed; firewall just opens the door.
    if crate::net::tcp::listener_lookup_by_port(dst_port).is_some() {
        return true;
    }

    // (c) Fall-through: explicit rules. Two-pass so port-specific
    // rules win over pure wildcards. With the inbound-TCP
    // wildcard gone from `init` this set is small, but
    // `allow_inbound_tcp_dst_port` still installs per-port
    // rules at listen_register time (and admin can add more).
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        for want_specific in [true, false] {
            for i in 0..MAX_RULES {
                let rule = &(*ptr)[i];
                if !rule.active { continue; }
                if rule.direction != 0 || rule.protocol != 6 { continue; }
                let ip_ok = rule.ip == 0 || rule.ip == src_ip;
                if !ip_ok { continue; }
                let src_port_ok = rule.port == 0 || rule.port == src_port;
                let dst_port_ok = rule.dst_port == 0 || rule.dst_port == dst_port;
                let any_specific = rule.port != 0 || rule.dst_port != 0;
                if !src_port_ok || !dst_port_ok { continue; }
                if want_specific {
                    if any_specific { return true; }
                } else {
                    if !any_specific { return true; }
                }
            }
        }
    }
    false
}

/// Transport-layer port check for UDP. Same stateful posture as
/// the TCP path (gap-audit 045 hardening): the firewall permits an
/// inbound UDP datagram iff
///
///   (a) `conntrack::lookup_inbound` finds a matching outbound flow
///       — we recently sent UDP to (src_ip, src_port) from
///       `dst_port`, so this is its reply.
///   (b) An explicit per-src_port rule matches (currently only
///       the DNS allow `src_port = 53` installed by `init`).
///
/// The caller (the UDP handler) supplies `dst_port` so the
/// conntrack lookup can match on the full 4-tuple. Pre-NET-045
/// callers that only knew `src_ip + src_port` get a backward-
/// compat shim below.
pub fn allow_inbound_udp_full(src_ip: u32, src_port: u16, dst_port: u16) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }

    // (a) Outbound-initiated flow — udp::send registered it.
    if crate::net::conntrack::lookup_inbound(17, src_ip, src_port, dst_port).is_some() {
        return true;
    }

    // (b) Explicit rules (init installs DNS allow at src_port=53).
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        for i in 0..MAX_RULES {
            let rule = &(*ptr)[i];
            if !rule.active { continue; }
            if rule.direction != 0 || rule.protocol != 17 { continue; }
            let ip_ok = rule.ip == 0 || rule.ip == src_ip;
            let port_ok = rule.port == 0 || rule.port == src_port;
            if ip_ok && port_ok { return true; }
        }
    }
    false
}

/// Backward-compat wrapper for the original `(src_ip, src_port)`
/// shape. Conntrack lookup is skipped (no dst_port available) so
/// only the explicit-rule path can match. The caller usually has
/// dst_port available — prefer `allow_inbound_udp_full`.
pub fn allow_inbound_udp(src_ip: u32, src_port: u16) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        for i in 0..MAX_RULES {
            let rule = &(*ptr)[i];
            if !rule.active { continue; }
            if rule.direction != 0 || rule.protocol != 17 { continue; }
            let ip_ok = rule.ip == 0 || rule.ip == src_ip;
            let port_ok = rule.port == 0 || rule.port == src_port;
            if ip_ok && port_ok { return true; }
        }
    }
    false
}

pub fn allow_outbound(_dst_ip: u32, protocol: u8) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }

    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        for i in 0..MAX_RULES {
            let rule = &(*ptr)[i];
            if !rule.active { continue; }
            if rule.direction != 1 { continue; }

            let proto_match = rule.protocol == 0 || rule.protocol == protocol;
            if proto_match {
                ALLOWED_COUNT.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
    }

    BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
    false
}

pub fn stats() -> (u32, u32) {
    (
        ALLOWED_COUNT.load(Ordering::Relaxed),
        BLOCKED_COUNT.load(Ordering::Relaxed),
    )
}
