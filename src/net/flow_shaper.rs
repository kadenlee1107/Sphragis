//! Per-flow rate limiter (second-tier shaper after cave_shaper).
//!
//! cave_shaper imposes an aggregate ceiling: a cave can't exceed
//! N packets/sec TOTAL across all destinations. That doesn't stop
//! an attacker who SPREADS a DDoS across many victims — each
//! flow stays under the aggregate but the aggregate gets fully
//! utilised. This shaper adds a second bucket keyed on the
//! (cave, dst_ip, dst_port) tuple, so per-destination behaviour is
//! bounded independently.
//!
//! Config model: each cave has one pair of numbers (pps, burst)
//! that describes the DEFAULT per-flow bucket. When a cave reaches
//! a new destination, a bucket is lazy-allocated from that default.
//! If the cave has no default configured, per-flow check passes
//! through (Unlimited).
//!
//! Table is a fixed LRU array of 64 flow buckets; the oldest gets
//! evicted when a new flow can't find a slot. That's fine because
//! legitimate caves rarely fan out to 64+ simultaneous destinations,
//! and an attacker who does just recreates the bucket at default
//! tokens — which costs them more than it saves.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, Ordering};

use super::cave_policy::{CaveId, cave_id_from_name};
use super::cave_shaper::RateVerdict;

const MAX_DEFAULTS:    usize = 16;
const MAX_FLOW_BUCKETS: usize = 64;
const TICK_SCALE:      u64   = 1_000_000;

fn now_ticks() -> u64 {
    let t: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) t); }
    t
}
fn ticks_per_sec() -> u64 {
    let f: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) f); }
    if f == 0 { 24_000_000 } else { f }
}

#[derive(Clone, Copy)]
struct Default {
    active: bool,
    cave: CaveId,
    pps: u32,
    burst: u32,
}
const EMPTY_DEFAULT: Default = Default {
    active: false, cave: [0; 16], pps: 0, burst: 0,
};

#[derive(Clone, Copy)]
struct Bucket {
    active: bool,
    cave: CaveId,
    dst_ip: u32,
    dst_port: u16,
    pps: u32,
    burst: u32,
    tokens_scaled: u64,
    last_refill_ticks: u64,
    /// Bumped on every check_and_debit; oldest LRU gets evicted.
    last_access_ticks: u64,
}
const EMPTY_BUCKET: Bucket = Bucket {
    active: false, cave: [0; 16], dst_ip: 0, dst_port: 0,
    pps: 0, burst: 0, tokens_scaled: 0,
    last_refill_ticks: 0, last_access_ticks: 0,
};

static mut DEFAULTS: [Default; MAX_DEFAULTS] = [EMPTY_DEFAULT; MAX_DEFAULTS];
static mut BUCKETS:  [Bucket; MAX_FLOW_BUCKETS] = [EMPTY_BUCKET; MAX_FLOW_BUCKETS];
static EVICTIONS: AtomicU32 = AtomicU32::new(0);

fn find_default(cave: &CaveId) -> Option<(u32, u32)> {
    unsafe {
        let t = core::ptr::addr_of!(DEFAULTS);
        for i in 0..MAX_DEFAULTS {
            let d = &(*t)[i];
            if d.active && &d.cave == cave {
                return Some((d.pps, d.burst));
            }
        }
    }
    None
}

pub fn set_default_by_name(name: &str, pps: u32, burst: u32) {
    let cave = cave_id_from_name(name);
    unsafe {
        let t = core::ptr::addr_of_mut!(DEFAULTS);
        // pps=0 → remove.
        if pps == 0 {
            for i in 0..MAX_DEFAULTS {
                if (*t)[i].active && (*t)[i].cave == cave {
                    (*t)[i].active = false;
                    return;
                }
            }
            return;
        }
        // Update in-place if exists.
        for i in 0..MAX_DEFAULTS {
            if (*t)[i].active && (*t)[i].cave == cave {
                (*t)[i].pps = pps;
                (*t)[i].burst = burst;
                return;
            }
        }
        // Free slot.
        for i in 0..MAX_DEFAULTS {
            if !(*t)[i].active {
                (*t)[i] = Default { active: true, cave, pps, burst };
                return;
            }
        }
    }
}

pub fn clear_by_name(name: &str) {
    set_default_by_name(name, 0, 0);
}

fn find_bucket(cave: &CaveId, dst_ip: u32, dst_port: u16) -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        for i in 0..MAX_FLOW_BUCKETS {
            let b = &(*t)[i];
            if b.active && &b.cave == cave
                && b.dst_ip == dst_ip && b.dst_port == dst_port
            { return Some(i); }
        }
    }
    None
}

fn alloc_bucket_slot(now: u64) -> usize {
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        // Prefer a free slot.
        for i in 0..MAX_FLOW_BUCKETS {
            if !(*t)[i].active { return i; }
        }
        // LRU eviction: find oldest last_access.
        let mut oldest_idx = 0;
        let mut oldest_ticks = (*t)[0].last_access_ticks;
        for i in 1..MAX_FLOW_BUCKETS {
            if (*t)[i].last_access_ticks < oldest_ticks {
                oldest_ticks = (*t)[i].last_access_ticks;
                oldest_idx = i;
            }
        }
        let _ = now;
        EVICTIONS.fetch_add(1, Ordering::Relaxed);
        oldest_idx
    }
}

pub fn check_and_debit(cave: &CaveId, dst_ip: u32, dst_port: u16) -> RateVerdict {
    let (pps, burst) = match find_default(cave) {
        Some(v) => v,
        None => return RateVerdict::Unlimited,
    };
    let now = now_ticks();
    let tps = ticks_per_sec();
    unsafe {
        let t = core::ptr::addr_of_mut!(BUCKETS);

        // Find or create.
        let idx = match find_bucket(cave, dst_ip, dst_port) {
            Some(i) => i,
            None => {
                let i = alloc_bucket_slot(now);
                (*t)[i] = Bucket {
                    active: true,
                    cave: *cave, dst_ip, dst_port,
                    pps, burst,
                    tokens_scaled: (burst as u64) * TICK_SCALE,
                    last_refill_ticks: now,
                    last_access_ticks: now,
                };
                i
            }
        };

        let b = &mut (*t)[idx];
        // Refill.
        let elapsed = now.saturating_sub(b.last_refill_ticks);
        if elapsed > 0 && b.pps > 0 {
            let add = (elapsed.saturating_mul(b.pps as u64))
                        .saturating_mul(TICK_SCALE) / tps;
            b.tokens_scaled = core::cmp::min(
                b.tokens_scaled.saturating_add(add),
                (b.burst as u64) * TICK_SCALE,
            );
            b.last_refill_ticks = now;
        }
        b.last_access_ticks = now;

        if b.tokens_scaled >= TICK_SCALE {
            b.tokens_scaled -= TICK_SCALE;
            RateVerdict::Ok
        } else {
            RateVerdict::OverLimit
        }
    }
}

pub fn check_and_debit_by_name(name: &str, dst_ip: u32, dst_port: u16) -> RateVerdict {
    check_and_debit(&cave_id_from_name(name), dst_ip, dst_port)
}

pub fn evictions() -> u32 { EVICTIONS.load(Ordering::Relaxed) }

pub fn reset() {
    unsafe {
        let d = core::ptr::addr_of_mut!(DEFAULTS);
        let b = core::ptr::addr_of_mut!(BUCKETS);
        for i in 0..MAX_DEFAULTS { (*d)[i].active = false; }
        for i in 0..MAX_FLOW_BUCKETS { (*b)[i].active = false; }
    }
    EVICTIONS.store(0, Ordering::Relaxed);
}

// ── Self-test ────────────────────────────────────────────────────

pub struct FlowShaperReport {
    pub flow_a_allowed: u32,
    pub flow_b_allowed: u32,
    pub both_independently_capped: bool,
}

/// Two flows from the same cave should each get their own budget.
/// Send 20 checks to flow A, 20 checks to flow B; with pps=100 burst=5
/// each gets 5 Ok and 15 OverLimit, INDEPENDENTLY.
pub fn selftest() -> Result<FlowShaperReport, &'static str> {
    reset();
    let cave = cave_id_from_name("kali");
    set_default_by_name("kali", 100, 5);
    let dst_a = 0x5DB8_D822u32;
    let dst_b = 0x0808_0808u32;
    let mut a_ok = 0u32;
    let mut b_ok = 0u32;
    for _ in 0..20 {
        if matches!(check_and_debit(&cave, dst_a, 443), RateVerdict::Ok) { a_ok += 1; }
    }
    for _ in 0..20 {
        if matches!(check_and_debit(&cave, dst_b, 53),  RateVerdict::Ok) { b_ok += 1; }
    }
    if a_ok != 5 || b_ok != 5 {
        return Err("each flow should get its own 5-token burst");
    }
    Ok(FlowShaperReport {
        flow_a_allowed: a_ok,
        flow_b_allowed: b_ok,
        both_independently_capped: a_ok == 5 && b_ok == 5,
    })
}
