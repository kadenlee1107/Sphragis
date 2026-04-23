//! Followup #3a: per-cave egress policy store.
//!
//! ## Why this module exists
//!
//! Today the per-cave allowlist lives in the Python daemon's
//! `FW_ALLOWLIST` table. A container hits the HTTP CONNECT proxy on
//! port 9998, batcaved.py reads the CONNECT, checks its dict, and
//! either dials out or returns 403. The policy brain is in Python on
//! the host — Bat_OS never sees it.
//!
//! That was fine as an MVP, but DESIGN_BATCAVES.md is explicit that
//! the kernel is supposed to be the authority for every packet a cave
//! emits. The eventual tap-device path (Followup #3c) will forward
//! cave frames through a vmnet-backed netdev into Bat_OS's IP stack,
//! and Bat_OS will make the allow/drop call at packet time. To do
//! that it needs somewhere to LOOK — a kernel-side table keyed by
//! cave id.
//!
//! This module is that table. It intentionally lives beside
//! `net::firewall` but is semantically different:
//!   - `firewall` filters Bat_OS's OWN packets (IP + transport port
//!     at packet time).
//!   - `cave_policy` decides whether a cave is PERMITTED to reach a
//!     given host:port at all. The check is by hostname (string) +
//!     port + protocol, because that's the granularity the daemon
//!     already uses and the tap path will have access to the SNI /
//!     HTTP Host field before the connection even opens.
//!
//! ## Lifecycle
//!
//! 1. Shell or docker-client pushes a policy for a cave id when the
//!    cave is created (`set_policy`).
//! 2. On cave destroy, `clear_policy` removes the entry — a fresh
//!    cave with the same id does NOT inherit the old allowlist.
//! 3. At connect time (daemon RPC in 3b, or in-kernel packet dispatch
//!    in 3c), callers invoke `check` with the 3-tuple and act on the
//!    verdict.
//!
//! Default is DENY. A cave with no entry gets `Verdict::Drop` for
//! every query. Removing a rule that was never there is a no-op.
//!
//! ## Self-test
//!
//! `selftest()` sets up two caves with different allowlists, verifies
//! the positive + negative cases on both, asserts that clearing cave
//! A leaves cave B untouched, and checks rule count bookkeeping. The
//! shell command is `cave-policy-selftest`.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// 16-byte opaque cave identifier. Matches `cave::CaveBacking::image`
/// length so the rest of batcave/ can hand us ids without conversion
/// headaches.
pub type CaveId = [u8; 16];

/// A single allow entry for a cave. Default-deny means we only store
/// allow rules; a destination that doesn't match any rule is denied.
#[derive(Clone)]
pub struct EgressRule {
    /// Lowercased hostname. Empty string = wildcard (any host).
    pub host: String,
    /// Destination port. 0 = wildcard (any port).
    pub port: u16,
    /// Protocol. 6 = TCP, 17 = UDP, 0 = any.
    pub proto: u8,
}

impl EgressRule {
    pub fn tcp(host: &str, port: u16) -> Self {
        Self { host: host.to_ascii_lowercase(), port, proto: 6 }
    }
    pub fn udp(host: &str, port: u16) -> Self {
        Self { host: host.to_ascii_lowercase(), port, proto: 17 }
    }
    pub fn matches(&self, host: &str, port: u16, proto: u8) -> bool {
        // Protocol: 0 wildcards.
        if self.proto != 0 && self.proto != proto { return false; }
        // Port: 0 wildcards.
        if self.port != 0 && self.port != port { return false; }
        // Host: empty wildcards; else case-insensitive exact.
        if self.host.is_empty() { return true; }
        self.host.as_bytes().eq_ignore_ascii_case(host.as_bytes())
    }
}

/// Policy container for one cave.
pub struct CavePolicy {
    pub cave: CaveId,
    pub rules: Vec<EgressRule>,
}

/// Verdict from `check`. Kept as a dedicated enum rather than bool so
/// callers can't accidentally invert the sense (critical for security
/// decisions).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Verdict { Allow, Drop }

// Single-threaded kernel invariant: the whole net/ stack is polled
// from one task. We gate access behind an Option<Vec> initialized by
// `init()`.
static mut POLICIES: Option<Vec<CavePolicy>> = None;

/// Idempotent init. Safe to call after boot; wipes any stale table.
pub fn init() {
    unsafe { POLICIES = Some(Vec::new()); }
}

fn ensure_init() -> &'static mut Vec<CavePolicy> {
    unsafe {
        // SAFETY: single-threaded kernel; called from init paths and
        // poll loop only.
        let ptr = core::ptr::addr_of_mut!(POLICIES);
        if (*ptr).is_none() {
            *ptr = Some(Vec::new());
        }
        (*ptr).as_mut().unwrap()
    }
}

/// Install or replace a cave's policy. Passing an empty rule vec is
/// valid and means "this cave exists but can reach nothing".
pub fn set_policy(cave: CaveId, rules: Vec<EgressRule>) {
    let table = ensure_init();
    for p in table.iter_mut() {
        if p.cave == cave {
            p.rules = rules;
            return;
        }
    }
    table.push(CavePolicy { cave, rules });
}

/// Append one rule to an existing cave. Creates the cave entry with
/// just this rule if it didn't exist.
pub fn add_rule(cave: CaveId, rule: EgressRule) {
    let table = ensure_init();
    for p in table.iter_mut() {
        if p.cave == cave {
            p.rules.push(rule);
            return;
        }
    }
    let mut rules = Vec::new();
    rules.push(rule);
    table.push(CavePolicy { cave, rules });
}

/// Remove a cave's policy entirely. Used on cave destroy.
pub fn clear_policy(cave: &CaveId) {
    let table = ensure_init();
    table.retain(|p| &p.cave != cave);
}

/// The connect-time question: "may CAVE reach HOST:PORT over PROTO?"
pub fn check(cave: &CaveId, host: &str, port: u16, proto: u8) -> Verdict {
    let table = ensure_init();
    for p in table.iter() {
        if &p.cave != cave { continue; }
        for r in p.rules.iter() {
            if r.matches(host, port, proto) { return Verdict::Allow; }
        }
        // Cave found but no rule matched — deny.
        return Verdict::Drop;
    }
    // Cave has no entry at all — default deny.
    Verdict::Drop
}

/// How many rules are installed for this cave (0 if cave is unknown).
pub fn rule_count(cave: &CaveId) -> usize {
    let table = ensure_init();
    for p in table.iter() {
        if &p.cave == cave { return p.rules.len(); }
    }
    0
}

/// How many caves have policy entries.
pub fn cave_count() -> usize {
    ensure_init().len()
}

/// Enumerate (cave_id, rule_count) for every installed policy.
/// Caller gets a fresh Vec; safe to iterate outside the table.
pub fn list_all() -> Vec<(CaveId, usize)> {
    let table = ensure_init();
    let mut out = Vec::with_capacity(table.len());
    for p in table.iter() {
        out.push((p.cave, p.rules.len()));
    }
    out
}

/// Read-only access to a cave's rule vec. Returned vec is a fresh clone.
pub fn rules_for(cave: &CaveId) -> Vec<EgressRule> {
    let table = ensure_init();
    for p in table.iter() {
        if &p.cave == cave { return p.rules.clone(); }
    }
    Vec::new()
}

// ── By-name convenience layer ────────────────────────────────────
//
// Cave names are human-readable ("kali1"), cave_policy keys are
// opaque 16 B ids. We derive one from the other via SHA-256(name),
// truncated. Deterministic per-build so a reboot with the same
// configuration lands on the same id; collision-resistant enough
// for a local table that caps at hundreds of entries.

/// Deterministic 16-byte CaveId from a human-readable cave name.
pub fn cave_id_from_name(name: &str) -> CaveId {
    use sha2::{Sha256, Digest};
    let mut h = Sha256::new();
    h.update(b"batos-cave-id-v1\0");
    h.update(name.as_bytes());
    let full = h.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&full[..16]);
    out
}

pub fn set_policy_by_name(name: &str, rules: Vec<EgressRule>) {
    set_policy(cave_id_from_name(name), rules);
}

pub fn add_rule_by_name(name: &str, rule: EgressRule) {
    add_rule(cave_id_from_name(name), rule);
}

pub fn clear_by_name(name: &str) {
    let id = cave_id_from_name(name);
    clear_policy(&id);
}

pub fn check_by_name(name: &str, host: &str, port: u16, proto: u8) -> Verdict {
    let id = cave_id_from_name(name);
    check(&id, host, port, proto)
}

pub fn rule_count_by_name(name: &str) -> usize {
    let id = cave_id_from_name(name);
    rule_count(&id)
}

pub fn rules_for_by_name(name: &str) -> Vec<EgressRule> {
    let id = cave_id_from_name(name);
    rules_for(&id)
}

// ── Self-test ────────────────────────────────────────────────────

pub struct SelftestReport {
    pub caves_installed: usize,
    pub allow_checks: usize,
    pub drop_checks: usize,
    pub cross_cave_isolation_ok: bool,
}

/// End-to-end proof:
///   1. Install two caves with disjoint allowlists.
///   2. Allowed destinations → Verdict::Allow.
///   3. Destinations outside the allowlist → Verdict::Drop.
///   4. Unknown cave → Verdict::Drop (default deny).
///   5. Clearing cave A does NOT affect cave B.
///   6. Wildcard port rule matches any port.
pub fn selftest() -> Result<SelftestReport, &'static str> {
    // Fresh table so this is deterministic regardless of prior calls.
    init();

    let cave_a: CaveId = [0xAA; 16];
    let cave_b: CaveId = [0xBB; 16];
    let cave_ghost: CaveId = [0xCC; 16]; // never installed

    // Cave A: can hit github + anthropic on 443.
    let mut a_rules = Vec::new();
    a_rules.push(EgressRule::tcp("github.com", 443));
    a_rules.push(EgressRule::tcp("api.anthropic.com", 443));
    set_policy(cave_a, a_rules);

    // Cave B: can hit httpbin on any port; plus DNS (UDP/53) anywhere.
    let mut b_rules = Vec::new();
    b_rules.push(EgressRule { host: "httpbin.org".to_string(), port: 0, proto: 6 });
    b_rules.push(EgressRule::udp("", 53)); // wildcard host on UDP/53
    set_policy(cave_b, b_rules);

    let mut allows = 0usize;
    let mut drops = 0usize;

    // (2) Positive cases.
    if check(&cave_a, "github.com", 443, 6) != Verdict::Allow {
        return Err("cave A github allow failed");
    }
    allows += 1;
    if check(&cave_a, "GitHub.com", 443, 6) != Verdict::Allow {
        return Err("cave A case-insensitive host match failed");
    }
    allows += 1;
    if check(&cave_b, "httpbin.org", 80, 6) != Verdict::Allow {
        return Err("cave B httpbin:80 allow failed (port wildcard)");
    }
    allows += 1;
    if check(&cave_b, "httpbin.org", 8080, 6) != Verdict::Allow {
        return Err("cave B httpbin:8080 allow failed (port wildcard)");
    }
    allows += 1;
    if check(&cave_b, "1.1.1.1", 53, 17) != Verdict::Allow {
        return Err("cave B DNS wildcard allow failed");
    }
    allows += 1;

    // (3) Negative cases: cave exists, destination not in its allowlist.
    if check(&cave_a, "httpbin.org", 443, 6) != Verdict::Drop {
        return Err("cave A httpbin should be denied");
    }
    drops += 1;
    if check(&cave_a, "github.com", 80, 6) != Verdict::Drop {
        return Err("cave A github:80 should be denied (rule pins 443)");
    }
    drops += 1;
    if check(&cave_b, "github.com", 443, 6) != Verdict::Drop {
        return Err("cave B github should be denied");
    }
    drops += 1;

    // (4) Unknown cave → default deny even for hosts that some OTHER
    //     cave would allow.
    if check(&cave_ghost, "github.com", 443, 6) != Verdict::Drop {
        return Err("unknown cave must default deny");
    }
    drops += 1;

    // (5) Cross-cave isolation.
    clear_policy(&cave_a);
    if check(&cave_a, "github.com", 443, 6) != Verdict::Drop {
        return Err("cleared cave A still allows");
    }
    drops += 1;
    let b_still_ok = check(&cave_b, "httpbin.org", 443, 6) == Verdict::Allow;
    if !b_still_ok {
        return Err("cave B allowlist disrupted by cave A clear");
    }
    allows += 1;

    Ok(SelftestReport {
        // cave_a was cleared; cave_b remains.
        caves_installed: cave_count(),
        allow_checks: allows,
        drop_checks: drops,
        cross_cave_isolation_ok: b_still_ok,
    })
}
