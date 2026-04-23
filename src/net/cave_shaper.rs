//! Per-cave traffic shaper (token-bucket rate limiter).
//!
//! Defense layer that fires AFTER cave_policy has already said
//! "this destination is in the allowlist". A cave whose C2 beacons
//! or exfil traffic rides over an allowlisted flow can still be
//! rate-limited here — if the attacker tries to flood the one host
//! they're allowed to reach, we cap the throughput.
//!
//! Model: classic token bucket per (cave, proto-independent). Each
//! cave has a configured `tokens_per_sec` fill rate and `burst`
//! bucket depth. Every packet debits one token; if the bucket is
//! empty the verdict is DropRate.
//!
//! Clock source: CNTVCT_EL0 (same one NAT GC uses). Refill is
//! computed lazily on each check, so we don't need a timer.
//!
//! Default policy: **no rate limit** for caves that haven't had
//! explicit rate set. Rate limiting is opt-in; cave_policy alone
//! remains the default-deny baseline. Operators who want shaping
//! call `cpol-rate <cave> <pps> <burst>`.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::cave_policy::{CaveId, cave_id_from_name};

/// Refill rate / burst for one cave.
#[derive(Clone, Copy)]
struct Bucket {
    cave: CaveId,
    tokens_per_sec: u32,
    burst: u32,
    /// Accumulated tokens scaled by TICK_SCALE to preserve fractional
    /// fill between calls. A "token" = 1 packet; we keep the running
    /// count in ticks-worth-of-tokens and divide when comparing.
    tokens_scaled: u64,
    last_refill_ticks: u64,
}

const TICK_SCALE: u64 = 1_000_000;
const MAX_BUCKETS: usize = 16;

static mut BUCKETS: [Option<Bucket>; MAX_BUCKETS] = [None; MAX_BUCKETS];

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

fn find(cave: &CaveId) -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        for i in 0..MAX_BUCKETS {
            if let Some(b) = &(*t)[i] {
                if &b.cave == cave { return Some(i); }
            }
        }
    }
    None
}

fn find_free() -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        for i in 0..MAX_BUCKETS {
            if (*t)[i].is_none() { return Some(i); }
        }
    }
    None
}

/// Install or update a cave's rate limit.  pps=0 means "remove the
/// limit" (reverts to unlimited). burst is the peak bucket depth.
pub fn set_rate(cave: CaveId, tokens_per_sec: u32, burst: u32) {
    unsafe {
        let t = core::ptr::addr_of_mut!(BUCKETS);
        if tokens_per_sec == 0 {
            // Remove any existing bucket.
            for i in 0..MAX_BUCKETS {
                if let Some(b) = &(*t)[i] {
                    if b.cave == cave { (*t)[i] = None; return; }
                }
            }
            return;
        }
        let now = now_ticks();
        // If there's already a bucket, update it and preserve tokens.
        if let Some(i) = find(&cave) {
            let b = (*t)[i].as_mut().unwrap();
            b.tokens_per_sec = tokens_per_sec;
            b.burst = burst;
            b.tokens_scaled = (burst as u64) * TICK_SCALE;
            b.last_refill_ticks = now;
            return;
        }
        // Else install in a free slot.
        if let Some(i) = find_free() {
            (*t)[i] = Some(Bucket {
                cave,
                tokens_per_sec,
                burst,
                tokens_scaled: (burst as u64) * TICK_SCALE,
                last_refill_ticks: now,
            });
        }
    }
}

pub fn set_rate_by_name(name: &str, tokens_per_sec: u32, burst: u32) {
    set_rate(cave_id_from_name(name), tokens_per_sec, burst);
}

pub fn clear_rate_by_name(name: &str) {
    set_rate(cave_id_from_name(name), 0, 0);
}

/// Outcome of a rate-limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateVerdict {
    /// No bucket configured for this cave — unlimited (opt-in).
    Unlimited,
    /// Under the bucket's budget; one token was debited.
    Ok,
    /// Bucket empty; caller should drop the packet.
    OverLimit,
}

/// Check whether the cave may send one more packet now. Refills the
/// bucket lazily (tokens_per_sec × elapsed_seconds) before checking.
/// Debits one token on Ok.
pub fn check_and_debit(cave: &CaveId) -> RateVerdict {
    let idx = match find(cave) { Some(i) => i, None => return RateVerdict::Unlimited };
    let now = now_ticks();
    let tps = ticks_per_sec();
    unsafe {
        let t = core::ptr::addr_of_mut!(BUCKETS);
        let b = (*t)[idx].as_mut().unwrap();
        // Refill.
        let elapsed = now.saturating_sub(b.last_refill_ticks);
        if elapsed > 0 {
            // tokens_to_add_scaled = elapsed_ticks * tokens_per_sec * TICK_SCALE / tps
            let add = (elapsed.saturating_mul(b.tokens_per_sec as u64))
                        .saturating_mul(TICK_SCALE) / tps;
            b.tokens_scaled = core::cmp::min(
                b.tokens_scaled.saturating_add(add),
                (b.burst as u64) * TICK_SCALE,
            );
            b.last_refill_ticks = now;
        }
        if b.tokens_scaled >= TICK_SCALE {
            b.tokens_scaled -= TICK_SCALE;
            RateVerdict::Ok
        } else {
            RateVerdict::OverLimit
        }
    }
}

pub fn check_and_debit_by_name(name: &str) -> RateVerdict {
    check_and_debit(&cave_id_from_name(name))
}

/// Observability: list every configured (cave_id, pps, burst, tokens_now).
pub fn list() -> Vec<(CaveId, u32, u32, u32)> {
    let mut out = Vec::new();
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        for i in 0..MAX_BUCKETS {
            if let Some(b) = &(*t)[i] {
                let tok_now = (b.tokens_scaled / TICK_SCALE) as u32;
                out.push((b.cave, b.tokens_per_sec, b.burst, tok_now));
            }
        }
    }
    out
}

/// Pretty-name lookup: return the configured rate (pps, burst) for a
/// cave name. Used by shell for `cpol-rate-show`.
pub fn rate_for(name: &str) -> Option<(u32, u32)> {
    let id = cave_id_from_name(name);
    let idx = find(&id)?;
    unsafe {
        let t = core::ptr::addr_of!(BUCKETS);
        let b = (*t)[idx].as_ref()?;
        Some((b.tokens_per_sec, b.burst))
    }
}

pub fn clear_all() {
    unsafe {
        let t = core::ptr::addr_of_mut!(BUCKETS);
        for i in 0..MAX_BUCKETS {
            (*t)[i] = None;
        }
    }
}

// ── Self-test ────────────────────────────────────────────────────

pub struct ShaperReport {
    pub allowed_in_burst: u32,
    pub denied_immediately: u32,
    pub cross_cave_unaffected: bool,
}

/// Prove the token-bucket algorithm:
///   1. Set rate(kali) = 10 pps, burst 5.
///   2. Fire 20 checks back-to-back (well under 1 sec of refill).
///      Expected: 5 Ok, 15 OverLimit.
///   3. `other` cave (no rate set) stays Unlimited: 20 checks → 20 OK
///      (actually Unlimited). No cross-talk.
pub fn selftest() -> Result<ShaperReport, &'static str> {
    clear_all();
    let kali_id  = cave_id_from_name("kali");
    let other_id = cave_id_from_name("no-rate-cave");

    set_rate(kali_id, 10, 5);

    let mut allowed = 0u32;
    let mut denied  = 0u32;
    for _ in 0..20 {
        match check_and_debit(&kali_id) {
            RateVerdict::Ok        => allowed += 1,
            RateVerdict::OverLimit => denied  += 1,
            RateVerdict::Unlimited => return Err("kali should not be unlimited"),
        }
    }
    if allowed != 5 { return Err("expected burst=5 tokens"); }
    if denied  != 15 { return Err("expected 15 OverLimit"); }

    // Other cave: no bucket, no limit.
    let mut other_unlimited = true;
    for _ in 0..20 {
        if check_and_debit(&other_id) != RateVerdict::Unlimited {
            other_unlimited = false; break;
        }
    }

    // String helpers exist to avoid moving values owned elsewhere.
    let _ = String::new();
    let _: Vec<&str> = Vec::new();
    let _ = "kali".to_string();

    Ok(ShaperReport {
        allowed_in_burst: allowed,
        denied_immediately: denied,
        cross_cave_unaffected: other_unlimited,
    })
}
