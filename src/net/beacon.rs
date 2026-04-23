//! Low-and-slow beacon detector.
//!
//! C2 malware typically phones home on a regular cadence — every
//! 30s, every 5min, jittered by only a few percent. That stays
//! comfortably under any rate shaper we'd tolerate on legit traffic,
//! so packet-count and byte-count limits don't catch it. What DOES
//! give a beacon away is the regularity itself — a human-driven
//! flow varies enormously in inter-packet gaps; a C2 beacon's
//! intervals cluster tightly around a single value.
//!
//! We record per-flow inter-packet intervals in a small ring, and
//! flag the flow when the coefficient-of-variation (stddev / mean)
//! drops below a threshold for a sustained window. This is
//! deliberately DETECTION-ONLY — false positives (NTP, legitimate
//! heartbeats, keepalives) make blocking too risky without operator
//! confirmation. Flagged flows surface via the `nat-beacons` shell
//! command so the operator can decide whether to add an explicit
//! policy rule.
//!
//! Integer math only: we avoid libm by comparing variance * 100 vs
//! mean², which is equivalent to CoV² < 0.01 without ever taking
//! a square root.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

use super::cave_policy::CaveId;

const MAX_FLOWS:    usize = 32;
const RING_LEN:     usize = 8;
const MIN_SAMPLES:  usize = 5;
/// 10% CoV threshold. CoV² < 0.01 → variance * 100 < mean².
const COV2_SCALE:   u64 = 100;

/// Minimum mean interval (ticks) to consider a flow a beacon.
/// Prevents flagging chat-like bursts where packets are microseconds
/// apart — these aren't beacons. We want 1-second-or-slower cadences.
/// CNTFRQ_EL0 is ~24 MHz on macOS QEMU, so 1s = 24_000_000 ticks.
fn min_mean_ticks() -> u64 {
    let f: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) f); }
    if f == 0 { 24_000_000 } else { f }   // 1 sec
}

fn now_ticks() -> u64 {
    let t: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) t); }
    t
}

#[derive(Clone, Copy)]
struct Flow {
    active: bool,
    cave: CaveId,
    dst_ip: u32,
    dst_port: u16,
    last_tick: u64,
    intervals: [u64; RING_LEN],
    ring_len: u8,
    ring_idx: u8,
    flagged: bool,
}

const EMPTY_FLOW: Flow = Flow {
    active: false, cave: [0; 16], dst_ip: 0, dst_port: 0,
    last_tick: 0,
    intervals: [0; RING_LEN],
    ring_len: 0, ring_idx: 0,
    flagged: false,
};

static mut FLOWS: [Flow; MAX_FLOWS] = [EMPTY_FLOW; MAX_FLOWS];
static TOTAL_FLAGS: AtomicU32 = AtomicU32::new(0);
static TOTAL_SAMPLES: AtomicU32 = AtomicU32::new(0);

fn find(cave: &CaveId, dst_ip: u32, dst_port: u16) -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(FLOWS);
        for i in 0..MAX_FLOWS {
            let f = &(*t)[i];
            if f.active && &f.cave == cave
                && f.dst_ip == dst_ip && f.dst_port == dst_port
            { return Some(i); }
        }
    }
    None
}

fn find_free() -> Option<usize> {
    unsafe {
        let t = core::ptr::addr_of!(FLOWS);
        for i in 0..MAX_FLOWS {
            if !(*t)[i].active { return Some(i); }
        }
    }
    None
}

/// Integer periodicity test. Returns Some(mean) if the ring is
/// periodic (CoV² < 0.01 AND mean ≥ min_mean).
fn is_periodic(intervals: &[u64]) -> Option<u64> {
    let n = intervals.len();
    if n < MIN_SAMPLES { return None; }
    let sum: u64 = intervals.iter().copied().fold(0u64, |a, b| a.saturating_add(b));
    let mean = sum / n as u64;
    if mean == 0 { return None; }
    if mean < min_mean_ticks() { return None; }
    let mut var_sum: u64 = 0;
    for &x in intervals {
        let d = if x > mean { x - mean } else { mean - x };
        var_sum = var_sum.saturating_add(d.saturating_mul(d));
    }
    let var = var_sum / n as u64;
    // CoV² < 1/COV2_SCALE ⇒ var * COV2_SCALE < mean²
    if var.saturating_mul(COV2_SCALE) < mean.saturating_mul(mean) {
        Some(mean)
    } else {
        None
    }
}

/// Record one packet event for (cave, dst_ip, dst_port). Called from
/// the classifier on every Allow-verdict packet.
pub fn record(cave: &CaveId, dst_ip: u32, dst_port: u16) {
    let now = now_ticks();
    let idx = match find(cave, dst_ip, dst_port) {
        Some(i) => i,
        None => match find_free() {
            Some(i) => {
                unsafe {
                    let t = core::ptr::addr_of_mut!(FLOWS);
                    let f = &mut (*t)[i];
                    f.active = true;
                    f.cave = *cave;
                    f.dst_ip = dst_ip;
                    f.dst_port = dst_port;
                    f.last_tick = now;
                    f.ring_len = 0;
                    f.ring_idx = 0;
                    f.flagged = false;
                }
                return;  // No interval yet.
            }
            None => return,  // Table full; flow not tracked.
        },
    };

    unsafe {
        let t = core::ptr::addr_of_mut!(FLOWS);
        let f = &mut (*t)[idx];
        let interval = now.saturating_sub(f.last_tick);
        f.last_tick = now;
        let pos = f.ring_idx as usize;
        f.intervals[pos] = interval;
        f.ring_idx = ((pos + 1) % RING_LEN) as u8;
        if (f.ring_len as usize) < RING_LEN { f.ring_len += 1; }
        TOTAL_SAMPLES.fetch_add(1, Ordering::Relaxed);

        if f.ring_len as usize >= MIN_SAMPLES {
            let slice = &f.intervals[..f.ring_len as usize];
            if is_periodic(slice).is_some() {
                if !f.flagged {
                    f.flagged = true;
                    TOTAL_FLAGS.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                // If the flow has deviated (e.g. legit human traffic)
                // we un-flag so we don't carry a stale verdict forever.
                f.flagged = false;
            }
        }
    }
}

pub fn is_flagged(cave: &CaveId, dst_ip: u32, dst_port: u16) -> bool {
    match find(cave, dst_ip, dst_port) {
        Some(i) => unsafe {
            let t = core::ptr::addr_of!(FLOWS);
            (*t)[i].flagged
        },
        None => false,
    }
}

pub fn total_flags() -> u32 { TOTAL_FLAGS.load(Ordering::Relaxed) }
pub fn total_samples() -> u32 { TOTAL_SAMPLES.load(Ordering::Relaxed) }

/// List every currently-flagged flow: (cave_id, dst_ip, dst_port,
/// mean_ticks, samples). Fresh allocation on each call.
pub fn list_flagged() -> Vec<(CaveId, u32, u16, u64, u8)> {
    let mut out = Vec::new();
    unsafe {
        let t = core::ptr::addr_of!(FLOWS);
        for i in 0..MAX_FLOWS {
            let f = &(*t)[i];
            if !f.active || !f.flagged { continue; }
            let slice = &f.intervals[..f.ring_len as usize];
            let mean = if !slice.is_empty() {
                slice.iter().copied().fold(0u64, |a, b| a.saturating_add(b)) / slice.len() as u64
            } else { 0 };
            out.push((f.cave, f.dst_ip, f.dst_port, mean, f.ring_len));
        }
    }
    out
}

pub fn reset() {
    unsafe {
        let t = core::ptr::addr_of_mut!(FLOWS);
        for i in 0..MAX_FLOWS { (*t)[i].active = false; (*t)[i].flagged = false; }
    }
    TOTAL_FLAGS.store(0, Ordering::Relaxed);
    TOTAL_SAMPLES.store(0, Ordering::Relaxed);
}

// ── Self-test ────────────────────────────────────────────────────

pub struct BeaconReport {
    pub beacon_flagged: bool,
    pub jitter_flagged: bool,
    pub total_flags: u32,
}

/// Pure-logic test of `is_periodic`. We don't need real timestamps;
/// we synthesize two interval series and check the classifier.
pub fn selftest() -> Result<BeaconReport, &'static str> {
    reset();
    // Beacon: 8 intervals of exactly 1.2 seconds (CoV = 0).
    let tps = min_mean_ticks();
    let beacon = [tps + tps/5; 8];
    let bp = is_periodic(&beacon);
    if bp.is_none() {
        return Err("regular beacon intervals should flag");
    }
    // Jittery human traffic: random-ish, CoV > 50%.
    let jitter: [u64; 8] = [
        tps, 2*tps, tps/2, 5*tps, tps, 3*tps, tps*4, tps/3,
    ];
    let jp = is_periodic(&jitter);
    if jp.is_some() {
        return Err("jittery human flow should NOT flag");
    }
    Ok(BeaconReport {
        beacon_flagged: bp.is_some(),
        jitter_flagged: jp.is_some(),
        total_flags: total_flags(),
    })
}
