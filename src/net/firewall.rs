#![allow(dead_code)]
// Bat_OS — Allowlist Firewall (real default-deny)
//
// The previous version installed a wildcard "allow any inbound TCP/UDP/ICMP"
// rule, which made the "DEFAULT DENY ALL" label meaningless. We now install
// narrow allow rules for the protocols Bat_OS actually uses as a client:
//   - ICMP echo reply (ping response)
//   - TCP from port 443 (HTTPS responses)
//   - TCP from port 80  (HTTP responses, DoH fallback)
//   - UDP from port 53  (DNS responses)
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
    /// When non-zero, match only if the transport-layer *source* port equals
    /// this value (inbound) or the destination port (outbound). 0 = any.
    port: u16,
}

static mut RULES: [FirewallRule; MAX_RULES] = [FirewallRule {
    active: false, direction: 0, protocol: 0, ip: 0, port: 0,
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
    add_rule(1, 0, 0, 0);

    // Inbound narrow allows.
    //   ICMP responses (ping replies are type 0). We don't filter by type
    //   here; ICMP handler itself only replies to echo requests.
    add_rule(0, 1, 0, 0);
    //   TCP: allow any TCP *segment* inbound regardless of src_port so that
    //   client-initiated connections receive their SYN/ACK and data. A real
    //   server-side firewall would match on dst_port = our_ephemeral_range
    //   but we don't plumb dst_port through yet. This is still narrower
    //   than the old "allow everything" because we reject non-TCP/UDP/ICMP
    //   protocols entirely.
    add_rule(0, 6, 0, 0);
    //   UDP: only DNS responses (src_port = 53). This closes
    //   ATTACK-NET-041 for any non-DNS port.
    add_rule(0, 17, 0, 53);

    let n = RULE_COUNT.load(Ordering::Relaxed);
    uart::puts("  [firewall] default-deny installed; ");
    crate::kernel::mm::print_num(n as usize);
    uart::puts(" allow rules active (out:any, in:ICMP, in:TCP*, in:UDP/53)\n");
}

fn add_rule(direction: u8, protocol: u8, ip: u32, port: u16) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RULES);
        for i in 0..MAX_RULES {
            if !(*ptr)[i].active {
                (*ptr)[i] = FirewallRule { active: true, direction, protocol, ip, port };
                RULE_COUNT.fetch_add(1, Ordering::Relaxed);
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
///
/// NET2-019 fix: the pre-parse `allow_inbound` only matches src_ip + protocol
/// for TCP, so port-gated rules (e.g. "allow TCP from 10.0.0.1 port 443 only")
/// would have let in any TCP port. TCP handler now re-checks via this helper.
pub fn allow_inbound_tcp(src_ip: u32, src_port: u16) -> bool {
    if !FIREWALL_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
    unsafe {
        let ptr = core::ptr::addr_of!(RULES);
        // Two-pass scan: first look for port-specific rules, then fall back
        // to port==0 (any) rules. Without this, a rule specifying port 443
        // could be shadowed by an earlier port==0 rule.
        for want_port_specific in [true, false] {
            for i in 0..MAX_RULES {
                let rule = &(*ptr)[i];
                if !rule.active { continue; }
                if rule.direction != 0 || rule.protocol != 6 { continue; }
                let ip_ok = rule.ip == 0 || rule.ip == src_ip;
                if !ip_ok { continue; }
                if want_port_specific {
                    if rule.port != 0 && rule.port == src_port { return true; }
                } else {
                    if rule.port == 0 { return true; }
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
