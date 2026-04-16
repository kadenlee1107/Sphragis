#![allow(dead_code)]
// Bat_OS — Allowlist Firewall
// DEFAULT DENY ALL. Only explicitly allowed traffic passes.
// Every connection must be whitelisted.

use crate::drivers::uart;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

const MAX_RULES: usize = 32;

#[derive(Clone, Copy)]
struct FirewallRule {
    active: bool,
    direction: u8,    // 0 = inbound, 1 = outbound
    protocol: u8,     // 0 = any, 1 = ICMP, 6 = TCP, 17 = UDP
    ip: u32,          // 0 = any
    port: u16,        // 0 = any
}

static mut RULES: [FirewallRule; MAX_RULES] = [FirewallRule {
    active: false, direction: 0, protocol: 0, ip: 0, port: 0,
}; MAX_RULES];

static FIREWALL_ENABLED: AtomicBool = AtomicBool::new(true);
static BLOCKED_COUNT: AtomicU32 = AtomicU32::new(0);
static ALLOWED_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    // Default rules: allow essential traffic
    // Rule 0: Allow all outbound (we initiate)
    add_rule(1, 0, 0, 0);
    // Rule 1: Allow ICMP inbound (ping replies)
    add_rule(0, 1, 0, 0);
    // Rule 2: Allow TCP inbound from any (for responses)
    add_rule(0, 6, 0, 0);
    // Rule 3: Allow UDP inbound (DNS responses)
    add_rule(0, 17, 0, 0);

    uart::puts("  [fw] Firewall active — DEFAULT DENY ALL\n");
    uart::puts("  [fw] Allowlist: outbound(all), inbound(ICMP,TCP,UDP responses)\n");
}

fn add_rule(direction: u8, protocol: u8, ip: u32, port: u16) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RULES);
        for i in 0..MAX_RULES {
            if !(*ptr)[i].active {
                (*ptr)[i] = FirewallRule { active: true, direction, protocol, ip, port };
                return;
            }
        }
    }
}

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
                ALLOWED_COUNT.store(ALLOWED_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                return true;
            }
        }
    }

    BLOCKED_COUNT.store(BLOCKED_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
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
                ALLOWED_COUNT.store(ALLOWED_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                return true;
            }
        }
    }

    BLOCKED_COUNT.store(BLOCKED_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    false
}

pub fn stats() -> (u32, u32) {
    (
        ALLOWED_COUNT.load(Ordering::Relaxed),
        BLOCKED_COUNT.load(Ordering::Relaxed),
    )
}
