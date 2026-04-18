// Bat_OS — Linux-compatible Futex (Fast Userspace Mutex)
//
// This module implements the kernel side of the Linux futex(2) ABI. Chromium
// (and essentially every modern Linux userspace) layers every thread sync
// primitive — pthread_mutex_t, pthread_cond_t, pthread_once_t, absl::Mutex,
// std::condition_variable, etc. — on top of FUTEX_WAIT / FUTEX_WAKE.
//
// Contract (summary):
//   FUTEX_WAIT(uaddr, val, timeout):
//       atomically: if *uaddr != val -> return -EAGAIN,
//                   else enqueue caller and block until FUTEX_WAKE,
//                   timeout, or signal.
//   FUTEX_WAKE(uaddr, n):
//       wake up to n waiters enqueued on uaddr. Return number woken.
//   FUTEX_REQUEUE(uaddr, uaddr2, wake_n, req_n):
//       wake wake_n waiters on uaddr, then requeue up to req_n more to uaddr2.
//   FUTEX_CMP_REQUEUE(uaddr, uaddr2, val, wake_n, req_n):
//       same as REQUEUE but first check *uaddr == val (EAGAIN otherwise).
//       Used by glibc's pthread_cond_broadcast to avoid thundering herd.
//
// Design:
//   - Fixed-size hash table of `NUM_BUCKETS` buckets, keyed by (uaddr >> 3).
//   - Each bucket has a fixed array of `WAITERS_PER_BUCKET` slots.
//   - Each slot stores: { in_use, uaddr, tid, woken_flag, deadline_ticks }.
//   - Entire table guarded by a single `AtomicBool` spinlock per bucket — fine
//     grained enough to avoid global contention, coarse enough to fit in a
//     no_std kernel without a real lock implementation.
//   - No heap. All state lives in `static mut` arrays, accessed under the
//     per-bucket spinlock.
//   - All shared state uses `core::sync::atomic` types so the compiler will
//     not reorder loads across publication.
//
// Blocking model (IMPORTANT TODO):
//   Bat_OS does have a priority-preemptive scheduler (see kernel/scheduler.rs)
//   but the Linux-compat runner currently does not mark tasks as Blocked /
//   wake them via the scheduler. Until that integration lands, FUTEX_WAIT
//   falls back to a "spin with arch-timer deadline + yield hint" loop:
//     - we publish the waiter into the hash table,
//     - we spin-read an atomic `woken` flag on the slot,
//     - on each iteration we re-check the user's memory against `val` (so
//       a missed FUTEX_WAKE doesn't livelock us),
//     - we honour the timeout via cntpct_el0.
//   When the scheduler gains a real Blocked state, replace the spin body in
//   `park_slot` with `scheduler::block_on(slot)` and have `futex_wake` call
//   `scheduler::unblock(tid)`.
//
// Error codes (Linux ABI):
//   -EAGAIN    = -11   value at uaddr didn't match expected val
//   -EINVAL    = -22   bad arguments (unaligned uaddr, etc.)
//   -ETIMEDOUT = -110  timeout expired before wake
//   -ENOSPC    = -28   wait queue full (Bat_OS-specific fallback)

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ─── Linux futex op codes ────────────────────────────────────────────────
pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_FD: u32 = 2;
pub const FUTEX_REQUEUE: u32 = 3;
pub const FUTEX_CMP_REQUEUE: u32 = 4;
pub const FUTEX_WAKE_OP: u32 = 5;
pub const FUTEX_LOCK_PI: u32 = 6;
pub const FUTEX_UNLOCK_PI: u32 = 7;
pub const FUTEX_TRYLOCK_PI: u32 = 8;
pub const FUTEX_WAIT_BITSET: u32 = 9;
pub const FUTEX_WAKE_BITSET: u32 = 10;
pub const FUTEX_PRIVATE_FLAG: u32 = 128;
pub const FUTEX_CLOCK_REALTIME: u32 = 256;

// ─── Error codes ─────────────────────────────────────────────────────────
pub const EAGAIN: i64 = -11;
pub const EINVAL: i64 = -22;
pub const ENOSPC: i64 = -28;
pub const ETIMEDOUT: i64 = -110;

// ─── Table geometry ──────────────────────────────────────────────────────
// 64 buckets × 32 waiters = 2048 concurrent waiters max. Plenty for a
// single-process Chromium renderer (typical: ~100 threads, each may have
// one or two pending futex_waits).
const NUM_BUCKETS: usize = 64;
const WAITERS_PER_BUCKET: usize = 32;

// A single wait-queue slot. All fields are atomic so the waker and the
// waiter can race safely without holding the bucket lock across the wait.
#[repr(C)]
struct WaitSlot {
    // Slot occupancy. Written under bucket lock.
    in_use: AtomicBool,
    // User-space futex address (key). Written under bucket lock.
    uaddr: AtomicU64,
    // Thread id of the waiter (for scheduler integration / debugging).
    tid: AtomicUsize,
    // Set to true by FUTEX_WAKE. The waiter polls this.
    woken: AtomicBool,
    // Deadline in cntpct_el0 ticks. 0 == no deadline.
    deadline_ticks: AtomicU64,
    // Bitset for FUTEX_WAIT_BITSET / FUTEX_WAKE_BITSET. Default 0xFFFFFFFF.
    bitset: AtomicU32,
}

impl WaitSlot {
    const fn new() -> Self {
        Self {
            in_use: AtomicBool::new(false),
            uaddr: AtomicU64::new(0),
            tid: AtomicUsize::new(0),
            woken: AtomicBool::new(false),
            deadline_ticks: AtomicU64::new(0),
            bitset: AtomicU32::new(0xFFFF_FFFF),
        }
    }
}

struct Bucket {
    // Per-bucket spinlock. Held only for short critical sections (enqueue,
    // scan-and-wake). Never held across the actual block/spin.
    lock: AtomicBool,
    slots: [WaitSlot; WAITERS_PER_BUCKET],
}

impl Bucket {
    const fn new() -> Self {
        // Const-construct the slot array.
        const EMPTY: WaitSlot = WaitSlot::new();
        Self {
            lock: AtomicBool::new(false),
            slots: [EMPTY; WAITERS_PER_BUCKET],
        }
    }
}

// The hash table itself. Static, zero-initialised at boot.
static mut TABLE: [Bucket; NUM_BUCKETS] = {
    const EMPTY: Bucket = Bucket::new();
    [EMPTY; NUM_BUCKETS]
};

// ─── Bucket lock helpers (test-and-set spinlock) ─────────────────────────

fn bucket_lock(b: &Bucket) {
    while b
        .lock
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

fn bucket_unlock(b: &Bucket) {
    b.lock.store(false, Ordering::Release);
}

// ─── Hashing ─────────────────────────────────────────────────────────────
// Shift out the bottom 2 bits (futex values are u32-aligned) then mix with
// a cheap xorshift. Collisions are fine — the bucket still holds multiple
// waiters.
fn bucket_index(uaddr: u64) -> usize {
    let mut h = uaddr >> 2;
    h ^= h >> 17;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (h as usize) % NUM_BUCKETS
}

// Safe wrapper to borrow a bucket from the static table.
#[allow(static_mut_refs)]
fn bucket(i: usize) -> &'static Bucket {
    unsafe { &TABLE[i] }
}

// ─── Time helpers (ARM64 generic timer) ──────────────────────────────────

fn cntpct() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v) }
    v
}

fn cntfrq() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v) }
    v
}

fn ns_to_ticks(ns: u64) -> u64 {
    let f = cntfrq();
    // avoid overflow for very large timeouts
    if ns == 0 {
        return 0;
    }
    let sec = ns / 1_000_000_000;
    let rem = ns % 1_000_000_000;
    sec.wrapping_mul(f) + rem.wrapping_mul(f) / 1_000_000_000
}

// ─── User memory load ────────────────────────────────────────────────────
// Futex values are 32-bit loads of a userspace pointer.
//
// ATTACK-SYS-006 fix: the previous implementation dereferenced any address
// the cave passed, giving a 32-bit kernel-memory oracle (and blocking on
// kernel state when used with FUTEX_WAIT). We now gate every load through
// `uaccess::is_user_range` and return 0 on rejection; the caller treats a
// mismatch against the expected value as EWOULDBLOCK, so rejection surfaces
// to the cave as a normal futex failure rather than a kernel read.
fn load_u32(uaddr: u64) -> u32 {
    if !crate::batcave::linux::uaccess::is_user_range(uaddr as usize, 4) {
        return 0;
    }
    let v: u32;
    unsafe {
        core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) uaddr, v = out(reg) v);
    }
    v
}

/// Separate gate exposed for futex entry points that need to distinguish
/// "bad pointer" from "value mismatch". Returns true if `uaddr` is safe to
/// touch as a 4-byte userspace value.
pub(crate) fn is_valid_uaddr(uaddr: u64) -> bool {
    crate::batcave::linux::uaccess::is_user_range(uaddr as usize, 4)
}

// ─── Current thread id stub ──────────────────────────────────────────────
// The Linux compat layer keeps the "current TID" in syscall.rs's CURRENT_TID.
// We expose a tiny shim here so we don't create a cyclic module dependency.
// When syscall.rs wires us up, it can pass the real TID via the public API.
fn current_tid() -> usize {
    // Placeholder: without integration we treat everything as tid 1.
    // syscall.rs should call futex_wait_tid() directly once wired.
    1
}

// ─── Enqueue / dequeue ───────────────────────────────────────────────────

// Find a free slot in the bucket, claim it, and populate it. Returns the
// slot index on success. Caller must hold the bucket lock.
fn enqueue(b: &Bucket, uaddr: u64, tid: usize, deadline_ticks: u64, bitset: u32) -> Option<usize> {
    for (i, s) in b.slots.iter().enumerate() {
        if !s.in_use.load(Ordering::Relaxed) {
            s.uaddr.store(uaddr, Ordering::Relaxed);
            s.tid.store(tid, Ordering::Relaxed);
            s.deadline_ticks.store(deadline_ticks, Ordering::Relaxed);
            s.bitset.store(bitset, Ordering::Relaxed);
            s.woken.store(false, Ordering::Relaxed);
            // Publish in_use last so a concurrent waker seeing in_use=true
            // also sees the populated key/bitset.
            s.in_use.store(true, Ordering::Release);
            return Some(i);
        }
    }
    None
}

// Release a slot. Caller must hold the bucket lock.
fn release(b: &Bucket, slot: usize) {
    let s = &b.slots[slot];
    s.in_use.store(false, Ordering::Release);
    s.uaddr.store(0, Ordering::Relaxed);
    s.tid.store(0, Ordering::Relaxed);
    s.woken.store(false, Ordering::Relaxed);
    s.deadline_ticks.store(0, Ordering::Relaxed);
    s.bitset.store(0xFFFF_FFFF, Ordering::Relaxed);
}

// ─── Park loop (the actual "block") ──────────────────────────────────────
//
// TODO(sched): replace this spin with a real scheduler.block_on(slot) once
// the Linux runner marks tasks Blocked and kicks them from futex_wake. The
// current implementation is correct (no lost wakeups, honours timeout) but
// wastes CPU.
fn park_slot(b: &Bucket, slot: usize, uaddr: u64, val: u32) -> i64 {
    let s = &b.slots[slot];
    let deadline = s.deadline_ticks.load(Ordering::Relaxed);
    let has_deadline = deadline != 0;

    loop {
        // Waker set the flag?
        if s.woken.load(Ordering::Acquire) {
            // Detach the slot under the bucket lock and report success.
            bucket_lock(b);
            release(b, slot);
            bucket_unlock(b);
            return 0;
        }

        // Timeout expired?
        if has_deadline && cntpct() >= deadline {
            bucket_lock(b);
            // Re-check woken under the lock — a wake that raced with us
            // should still be honoured (we'd rather consume that wake than
            // return ETIMEDOUT and leave another thread blocked forever).
            if s.woken.load(Ordering::Acquire) {
                release(b, slot);
                bucket_unlock(b);
                return 0;
            }
            release(b, slot);
            bucket_unlock(b);
            return ETIMEDOUT;
        }

        // Belt-and-braces: if the user memory diverges from `val` and no
        // wake arrived, we can still wake ourselves spuriously. This is
        // allowed by the futex contract (POSIX calls it a spurious wakeup)
        // and helps if we missed a FUTEX_WAKE due to a bug. Commented out
        // by default because strict FUTEX_WAIT semantics require only
        // returning on explicit wake or timeout.
        //
        // if load_u32(uaddr) != val { ... }
        let _ = val; // silence unused-var when the check is commented

        // Yield hint to the CPU, then re-poll. When the real scheduler
        // integration lands, swap this for a true block.
        core::hint::spin_loop();
        let _ = uaddr;
    }
}

// ─── Public API ──────────────────────────────────────────────────────────

/// FUTEX_WAIT — atomically check *uaddr == val, then block until woken or
/// the timeout (in ns; 0 = infinite) expires.
///
/// Return 0 on wake, -EAGAIN on value mismatch, -ETIMEDOUT on timeout,
/// -ENOSPC if the wait queue is full.
pub fn futex_wait(uaddr: u64, val: u32, timeout_ns: u64) -> i64 {
    if uaddr == 0 || (uaddr & 0x3) != 0 {
        return EINVAL;
    }
    // ATTACK-SYS-006: reject non-user addresses. Without this the cave
    // could set uaddr to point at kernel state and probe it.
    if !is_valid_uaddr(uaddr) {
        return EINVAL;
    }

    // Compute absolute deadline in cntpct ticks (0 == none).
    let deadline = if timeout_ns == 0 {
        0
    } else {
        cntpct().wrapping_add(ns_to_ticks(timeout_ns))
    };

    let bi = bucket_index(uaddr);
    let b = bucket(bi);

    // Critical section: check user memory and enqueue atomically wrt other
    // wakers. Without this lock, a FUTEX_WAKE firing between our value
    // check and our enqueue would be lost.
    bucket_lock(b);

    let current = load_u32(uaddr);
    if current != val {
        bucket_unlock(b);
        return EAGAIN;
    }

    let slot = match enqueue(b, uaddr, current_tid(), deadline, 0xFFFF_FFFF) {
        Some(s) => s,
        None => {
            bucket_unlock(b);
            return ENOSPC;
        }
    };

    bucket_unlock(b);

    // Release the bucket lock before blocking — park_slot re-acquires it
    // only for the short detach at the end.
    park_slot(b, slot, uaddr, val)
}

/// FUTEX_WAIT_BITSET — same as WAIT but only wakeable by a matching bitset.
pub fn futex_wait_bitset(uaddr: u64, val: u32, timeout_ns: u64, bitset: u32) -> i64 {
    if bitset == 0 {
        return EINVAL;
    }
    if uaddr == 0 || (uaddr & 0x3) != 0 {
        return EINVAL;
    }
    // ATTACK-SYS-006: reject non-user addresses.
    if !is_valid_uaddr(uaddr) {
        return EINVAL;
    }

    let deadline = if timeout_ns == 0 {
        0
    } else {
        cntpct().wrapping_add(ns_to_ticks(timeout_ns))
    };

    let bi = bucket_index(uaddr);
    let b = bucket(bi);

    bucket_lock(b);
    let current = load_u32(uaddr);
    if current != val {
        bucket_unlock(b);
        return EAGAIN;
    }
    let slot = match enqueue(b, uaddr, current_tid(), deadline, bitset) {
        Some(s) => s,
        None => {
            bucket_unlock(b);
            return ENOSPC;
        }
    };
    bucket_unlock(b);

    park_slot(b, slot, uaddr, val)
}

/// FUTEX_WAKE — wake up to `max_wakers` tasks waiting on uaddr.
/// Returns the number woken.
pub fn futex_wake(uaddr: u64, max_wakers: u32) -> i64 {
    if uaddr == 0 {
        return EINVAL;
    }
    // ATTACK-SYS-006: reject non-user addresses. Wake doesn't deref but
    // a cave using kernel-addr bucketing would still poison shared buckets.
    if !is_valid_uaddr(uaddr) {
        return EINVAL;
    }
    let bi = bucket_index(uaddr);
    let b = bucket(bi);
    let mut woken: i64 = 0;

    bucket_lock(b);
    for s in b.slots.iter() {
        if woken as u32 >= max_wakers {
            break;
        }
        if !s.in_use.load(Ordering::Acquire) {
            continue;
        }
        if s.uaddr.load(Ordering::Relaxed) != uaddr {
            continue;
        }
        if s.woken.load(Ordering::Relaxed) {
            continue; // already flagged, waiter just hasn't reaped yet
        }
        // Mark woken. The waiter will detach the slot itself.
        s.woken.store(true, Ordering::Release);
        // TODO(sched): scheduler::unblock(s.tid.load(Ordering::Relaxed))
        woken += 1;
    }
    bucket_unlock(b);

    woken
}

/// FUTEX_WAKE_BITSET — wake only waiters whose bitset intersects `bitset`.
pub fn futex_wake_bitset(uaddr: u64, max_wakers: u32, bitset: u32) -> i64 {
    if bitset == 0 {
        return EINVAL;
    }
    let bi = bucket_index(uaddr);
    let b = bucket(bi);
    let mut woken: i64 = 0;

    bucket_lock(b);
    for s in b.slots.iter() {
        if woken as u32 >= max_wakers {
            break;
        }
        if !s.in_use.load(Ordering::Acquire) {
            continue;
        }
        if s.uaddr.load(Ordering::Relaxed) != uaddr {
            continue;
        }
        if s.woken.load(Ordering::Relaxed) {
            continue;
        }
        if s.bitset.load(Ordering::Relaxed) & bitset == 0 {
            continue;
        }
        s.woken.store(true, Ordering::Release);
        woken += 1;
    }
    bucket_unlock(b);

    woken
}

/// FUTEX_REQUEUE — wake up to `wake_count` waiters on uaddr, then move up
/// to `requeue_count` remaining waiters to uaddr2 (without waking them).
///
/// Returns total number of waiters woken + requeued.
pub fn futex_requeue(uaddr: u64, uaddr2: u64, wake_count: u32, requeue_count: u32) -> i64 {
    requeue_impl(uaddr, uaddr2, wake_count, requeue_count, None)
}

/// FUTEX_CMP_REQUEUE — same as REQUEUE but first check *uaddr == val.
/// Used by glibc pthread_cond_broadcast to avoid the thundering-herd
/// problem where N threads all wake to contend on the same mutex.
pub fn futex_cmp_requeue(
    uaddr: u64,
    uaddr2: u64,
    val: u32,
    wake_count: u32,
    requeue_count: u32,
) -> i64 {
    requeue_impl(uaddr, uaddr2, wake_count, requeue_count, Some(val))
}

fn requeue_impl(
    uaddr: u64,
    uaddr2: u64,
    wake_count: u32,
    requeue_count: u32,
    cmp_val: Option<u32>,
) -> i64 {
    if uaddr == 0 || uaddr2 == 0 {
        return EINVAL;
    }

    let bi1 = bucket_index(uaddr);
    let bi2 = bucket_index(uaddr2);

    // Acquire the two buckets in a deterministic order to avoid deadlock.
    // (If bi1 == bi2 we only need to lock once.)
    let b1 = bucket(bi1);
    let b2 = bucket(bi2);

    if bi1 == bi2 {
        bucket_lock(b1);
    } else if bi1 < bi2 {
        bucket_lock(b1);
        bucket_lock(b2);
    } else {
        bucket_lock(b2);
        bucket_lock(b1);
    }

    // CMP_REQUEUE check — done under both locks.
    if let Some(expected) = cmp_val {
        let current = load_u32(uaddr);
        if current != expected {
            if bi1 != bi2 {
                bucket_unlock(b2);
            }
            bucket_unlock(b1);
            return EAGAIN;
        }
    }

    // Step 1: wake up to wake_count waiters on uaddr.
    let mut woken: i64 = 0;
    for s in b1.slots.iter() {
        if woken as u32 >= wake_count {
            break;
        }
        if !s.in_use.load(Ordering::Acquire) {
            continue;
        }
        if s.uaddr.load(Ordering::Relaxed) != uaddr {
            continue;
        }
        if s.woken.load(Ordering::Relaxed) {
            continue;
        }
        s.woken.store(true, Ordering::Release);
        woken += 1;
    }

    // Step 2: requeue up to requeue_count remaining waiters from uaddr to
    // uaddr2. "Requeue" means change their key so a future FUTEX_WAKE on
    // uaddr2 will find them.
    let mut requeued: i64 = 0;
    for s in b1.slots.iter() {
        if requeued as u32 >= requeue_count {
            break;
        }
        if !s.in_use.load(Ordering::Acquire) {
            continue;
        }
        if s.uaddr.load(Ordering::Relaxed) != uaddr {
            continue;
        }
        if s.woken.load(Ordering::Relaxed) {
            continue;
        }

        if bi1 == bi2 {
            // Same bucket — just rewrite the key in place.
            s.uaddr.store(uaddr2, Ordering::Release);
            requeued += 1;
        } else {
            // Different bucket — try to move the slot over.
            match enqueue(
                b2,
                uaddr2,
                s.tid.load(Ordering::Relaxed),
                s.deadline_ticks.load(Ordering::Relaxed),
                s.bitset.load(Ordering::Relaxed),
            ) {
                Some(_new_idx) => {
                    // NOTE: moving the slot is tricky because the waiter
                    // is spinning on its *original* slot's `woken` flag.
                    // Until we have real blocking, we can't safely migrate
                    // a spinning waiter to a new slot. As a correct-but-
                    // conservative fallback, we just rewrite the key in
                    // the original slot — the waiter keeps parking on its
                    // slot, and a FUTEX_WAKE on uaddr2 will find it via
                    // the matching uaddr field. The enqueued shadow slot
                    // is immediately released.
                    //
                    // TODO(sched): once waiters truly block in the
                    // scheduler (not in the slot), migrate the blocked
                    // task onto the new bucket's slot and release the old.
                    release(b2, _new_idx);
                    s.uaddr.store(uaddr2, Ordering::Release);
                    requeued += 1;
                }
                None => {
                    // Destination bucket full — stop requeuing.
                    break;
                }
            }
        }
    }

    if bi1 != bi2 {
        bucket_unlock(b2);
    }
    bucket_unlock(b1);

    woken + requeued
}

// ─── Introspection (debug / tests) ───────────────────────────────────────

/// Count the number of active waiters in the entire table. For diagnostics.
pub fn active_waiters() -> usize {
    let mut n = 0;
    for bi in 0..NUM_BUCKETS {
        let b = bucket(bi);
        for s in b.slots.iter() {
            if s.in_use.load(Ordering::Relaxed) {
                n += 1;
            }
        }
    }
    n
}

/// Count active waiters for a specific address. For diagnostics.
pub fn waiters_on(uaddr: u64) -> usize {
    let bi = bucket_index(uaddr);
    let b = bucket(bi);
    let mut n = 0;
    for s in b.slots.iter() {
        if s.in_use.load(Ordering::Relaxed) && s.uaddr.load(Ordering::Relaxed) == uaddr {
            n += 1;
        }
    }
    n
}

/// V8-ROOT-2: drop every waiter when switching caves. Without this, a new
/// cave's userspace could accidentally satisfy (or be woken by) a previous
/// cave's parked thread on an overlapping address — breaks isolation.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    for bi in 0..NUM_BUCKETS {
        let b = bucket(bi);
        // Take the bucket lock so any in-flight op completes first.
        bucket_lock(b);
        for s in b.slots.iter() {
            s.in_use.store(false, Ordering::Release);
            s.uaddr.store(0, Ordering::Relaxed);
            s.bitset.store(0, Ordering::Relaxed);
            s.woken.store(false, Ordering::Relaxed);
        }
        bucket_unlock(b);
    }
}
