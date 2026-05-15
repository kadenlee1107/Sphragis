//! 24-hour rolling counters for MLS / TE deny events. Read by the
//! Wave-4 SECURITY app's INTEGRITY panel.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

static BLP_DENIES_24H:  AtomicU32 = AtomicU32::new(0);
static BIBA_DENIES_24H: AtomicU32 = AtomicU32::new(0);
static TE_DENIES_24H:   AtomicU32 = AtomicU32::new(0);
static LAST_BIBA_TS:    AtomicU64 = AtomicU64::new(0);
static LAST_BLP_TS:     AtomicU64 = AtomicU64::new(0);
static LAST_TE_TS:      AtomicU64 = AtomicU64::new(0);
static EPOCH_START:     AtomicU64 = AtomicU64::new(0);

const WINDOW_SECS: u64 = 24 * 3600;

fn maybe_roll_window() {
    let now = crate::kernel::time::monotonic_secs();
    let start = EPOCH_START.load(Ordering::Relaxed);
    if start == 0 || now.saturating_sub(start) >= WINDOW_SECS {
        EPOCH_START.store(now, Ordering::Relaxed);
        BLP_DENIES_24H.store(0, Ordering::Relaxed);
        BIBA_DENIES_24H.store(0, Ordering::Relaxed);
        TE_DENIES_24H.store(0, Ordering::Relaxed);
    }
}

pub fn record_blp_deny() {
    maybe_roll_window();
    BLP_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_BLP_TS.store(crate::kernel::time::monotonic_secs(), Ordering::Relaxed);
}
pub fn record_biba_deny() {
    maybe_roll_window();
    BIBA_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_BIBA_TS.store(crate::kernel::time::monotonic_secs(), Ordering::Relaxed);
}
pub fn record_te_deny() {
    maybe_roll_window();
    TE_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_TE_TS.store(crate::kernel::time::monotonic_secs(), Ordering::Relaxed);
}

pub fn blp_denies()      -> u32 { maybe_roll_window(); BLP_DENIES_24H.load(Ordering::Relaxed) }
pub fn biba_denies()     -> u32 { maybe_roll_window(); BIBA_DENIES_24H.load(Ordering::Relaxed) }
pub fn te_denies()       -> u32 { maybe_roll_window(); TE_DENIES_24H.load(Ordering::Relaxed) }

pub fn last_biba_ts()    -> u64 { LAST_BIBA_TS.load(Ordering::Relaxed) }
pub fn last_blp_ts()     -> u64 { LAST_BLP_TS.load(Ordering::Relaxed) }
pub fn last_te_ts()      -> u64 { LAST_TE_TS.load(Ordering::Relaxed) }
