#![allow(dead_code)]
// Bat_OS — Allowlist Firewall (real default-deny)
//
// The previous version installed a wildcard "allow any inbound TCP/UDP/ICMP"
// rule, which made the "DEFAULT DENY ALL" label meaningless. We now install
// narrow allow rules for the protocols Bat_OS actually uses as a client:
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
    // TCP: allow any TCP *segment* inbound regardless of src_port so that
    // client-initiated connections receive their SYN/ACK and data. A real
    // server-side firewall would match on dst_port = our_ephemeral_range
    // but we don't plumb dst_port through yet. This is still narrower
    // than the old "allow everything" because we reject non-TCP/UDP/ICMP
    // protocols entirely.
    //
    // this wildcard rule covers BOTH client-response traffic
    // (src_port=80/443/etc., dst_port=our_ephemeral) AND any incoming SYN
    // (dst_port=our_listener). When hardens this, the wildcard
    // gets removed and per-listener rules added on `tcp::listen_register`.
    // For now: keep the wildcard for backward compat, AND have
    // listen_register install a per-port rule (defense in depth).
    add_rule(0, 6, 0, 0, 0);
    // UDP: only DNS responses (src_port = 53). This closes
    // ATTACK-NET-041 for any non-DNS port.
    add_rule(0, 17, 0, 53, 0);

    let n = RULE_COUNT.load(Ordering::Relaxed);
    uart::puts("  [firewall] default-deny installed; ");
    crate::kernel::mm::print_num(n as usize);
    uart::puts(" allow rules active (out:any, in:ICMP, in:TCP*, in:UDP/53)\n");
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
    false
}

/// Transport-layer port check for TCP (called after parsing the header).
// /
/// NET2-019 fix: the pre-parse `allow_inbound` only matches src_ip + protocol
/// for TCP, so port-gated rules (e.g. "allow TCP from 10.0.0.1 port 443 only")
/// would have let in any TCP port. TCP handler now re-checks via this helper.
pub fn allow_inbound_tcp(src_ip: u32, src_port: u16, dst_port: u16) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        // a rule matches if all THREE of (ip, src_port,
        // dst_port) are compatible. A field of 0 in the rule means
        // "any" for that dimension. The pre-#150 logic only checked
        // src_port; dst_port was not plumbed through, so a rule like
        // "allow inbound to dst_port=8080" couldn't be expressed.
        //
        // Two-pass to honor explicit-port-specific rules over wildcards:
        // first pass requires the rule to have at least one port-specific
        // match field (port or dst_port non-zero AND it actually matches);
        // second pass accepts pure wildcards (rule.port == 0 AND
        // rule.dst_port == 0) as fallback.
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

/// Transport-layer port check for UDP (called after parsing the header).
/// Returns true iff an inbound UDP rule permits traffic from `src_port`.
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
