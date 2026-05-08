# DESIGN: Futex Deadline Unification

**Status:** Active proposal as of 2026-05-08.
**Follows:** `DESIGN_SCHEDULER_BLOCK_ON.md` (decision #5 explicitly out-of-scoped this; this thread closes that gap).
**Touches:** `src/batcave/linux/futex.rs`, `src/batcave/linux/threads.rs`, `src/ui/shell.rs`, `scripts/qemu_selftests_smoke.py`.

## Goal

Move the futex per-waiter deadline from `futex.rs::WaitSlot::deadline_ticks` (an `AtomicU64` field on each WaitSlot) into `BlockReason::FutexWait`. Single source of truth for all blocker deadlines. The existing `wake_expired_deadlines()` tick pass — which today only handles `EpollWait` and `Nanosleep` — gains a third arm that covers futex timeouts the same way.

## Why now

Cleanup, not a real bug. The scheduler thread's spec said this would be a future thread because of:
- Single source of truth: with `deadline_ticks` on `BlockReason`, all blocker deadlines parametrize via the same mechanism.
- Better timeout semantics: today, futex timeouts rely on `force-wake-on-deadlock` (STUMP #63) plus per-resume re-checks. After unification, deadline expiry triggers a deterministic Runnable transition every timer tick, regardless of other thread state.

## What changes

### Type-level

```rust
// src/batcave/linux/threads.rs
// Before
FutexWait { uaddr: u64, val: u32 },
// After
FutexWait { uaddr: u64, val: u32, deadline_ticks: u64 },  // 0 = no deadline
```

`futex.rs::WaitSlot::deadline_ticks` field removed. `enqueue()` signature drops the `deadline` parameter (deadline lives on the calling thread's BlockReason now).

### `wake_expired_deadlines` gains a FutexWait arm

```rust
ThreadState::Blocked(BlockReason::FutexWait { deadline_ticks, .. })
    if deadline_ticks != 0 && now >= deadline_ticks => true,
```

Symmetric with the existing EpollWait + Nanosleep arms.

### `park_slot` adopts the park_current loop shape

Same bucket-lock dance (still needed for the woken+blocked race), plus:
- `wfi` added between schedule resumes so single-thread caves don't busy-loop.
- `mark_current_runnable()` after schedule removed (defensive marker; conflicts with the "blocker marks blocked, waker marks runnable" rule from the scheduler thread).
- Reads `deadline_ticks` as a function parameter instead of from WaitSlot.

```text
park_slot(b, slot, uaddr, val, deadline_ticks):
    loop:
        bucket_lock + IrqGuard
        if s.woken: release; return 0
        if deadline_ticks != 0 && cntpct() >= deadline_ticks: release; return ETIMEDOUT
        mark_current_blocked(FutexWait { uaddr, val, deadline_ticks })
        bucket_unlock + drop guard
        schedule()
        if !current_thread_blocked(): continue        (waker fired)
        wfi                                            (still blocked; sleep until IRQ)
        // implicit loop continue → re-check
```

### Diagnostic site

`futex.rs::futex_dump` (~line 708) currently reads `s.deadline_ticks` for output. Updated to read deadline from the threads table via the WaitSlot's `tid → BlockReason::FutexWait::deadline_ticks` lookup. If the slot's tid isn't in `Blocked(FutexWait{..})` state (rare race), prints `0`.

### Diagnostic encoding (`threads.rs:~1290`) — preserve existing semantics

The `(a1, a2)` SKIP-DEADLOCK log pack stays at `(uaddr, val as u64)` for FutexWait. Adding `deadline_ticks` would change `a1` semantics for tools that already parse the log; not worth the break since `wake_expired_deadlines` doesn't read this field anyway. EpollWait and Nanosleep already encode `deadline_ticks` as `a1`; FutexWait stays `a1=uaddr` for backwards compatibility. Asymmetric but documented.

## Selftest

Add a fourth scheduler sub-test: **`futex-deadline-fires`**. Pattern matches `nanosleep-deadline-fires`:

```text
let now = cntpct_el0();
let past = now.saturating_sub(1);
slot = test_install_blocked(BlockReason::FutexWait { uaddr: 0, val: 0, deadline_ticks: past })
wake_expired_deadlines()
assert test_inspect_state(slot) == Some(Runnable)
test_release_slot(slot)
```

`scripts/qemu_selftests_smoke.py` count threshold updates from "≥3 scheduler sub-tests" to "≥4."

## Approximate diff

- `src/batcave/linux/threads.rs`: 1 enum field added, 1 arm added to wake_expired_deadlines, ~5 lines.
- `src/batcave/linux/futex.rs`: WaitSlot::deadline_ticks removed (~5 sites), enqueue() signature change (~3 call sites updated), park_slot rewrite (~50 LOC), futex_dump deadline lookup (~10 LOC).
- `src/ui/shell.rs`: 4th sub-test in cmd_scheduler_selftest (~25 LOC).
- `scripts/qemu_selftests_smoke.py`: count threshold 3 → 4 (~3 LOC).

Net: ~100 lines changed across 4 files. Single PR.

## Testing & verification

**Layer 1 — build:** `cargo check --release` and `cargo check --release --features selftest-on-boot`. Both clean.

**Layer 2 — boot smoke:** `scripts/qemu_boot_smoke.py` PASSES (regression check; futex is exercised by some boot paths).

**Layer 3 — selftests smoke:** `scripts/qemu_selftests_smoke.py` PASSES with all 4 scheduler sub-tests including the new `futex-deadline-fires`.

**Layer 4 — manual review:** the new `park_slot` body satisfies the same lock/IRQ invariants documented in `park_current`'s doc-comment:
- bucket lock NOT held across `wfi`
- IrqGuard NOT held across `wfi`
- no early return/break paths leave the function while state == Blocked

## Out of scope

- Refactoring `park_slot` to call `park_current` directly. Tried during design; the bucket-lock-and-woken-flag race makes that brittle. Keeping the bespoke park loop with adopted invariants is cleaner.
- Touching futex's other paths (FUTEX_WAKE, FUTEX_REQUEUE, etc.). They use `wake_thread(tid)` and don't depend on the deadline storage.
- Diagnostic encoding symmetry across all BlockReason variants. Asymmetry preserved for log-format compatibility.

## Reversibility

Tag `pre-futex-deadline-unification-2026-05-08` at branch HEAD before any change lands. Single-commit revert if regression surfaces.

🦇
