# Futex Deadline Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `deadline_ticks` from `futex.rs::WaitSlot` to `BlockReason::FutexWait`, extend `wake_expired_deadlines` to handle futex timeouts, rewrite `park_slot` to use the park-current loop shape, and add a 4th scheduler selftest.

**Architecture:** Bottom-up. Add the `deadline_ticks` field to `BlockReason::FutexWait` and the new `wake_expired_deadlines` arm first (additive — no callers yet but build stays clean). Then rewrite `park_slot` to use the new BlockReason field — at this commit deadline lives in BOTH places (transition state, no behavior change beyond the rewrite). Then drop `WaitSlot::deadline_ticks` and simplify `enqueue()` (single source of truth achieved). Selftest + smoke-harness updates close it out. Each phase ends with `cargo check` clean.

**Tech Stack:** Rust + Cargo, bare-metal `aarch64-unknown-none`, nightly toolchain. Same kernel as the prior threads.

**Reference spec:** `DESIGN_FUTEX_DEADLINE_UNIFICATION.md` (root). Read it first.

**Pre-implementation HEAD:** `61a84c60` on branch `feat/futex-unification` (the spec commit).

**Spec correction worth flagging:** the spec described "diagnostic site at futex.rs:708 (futex_dump)." Closer look during planning showed line 708 is inside `requeue_impl`, not a diagnostic dump — it's the read where the OLD WaitSlot's deadline gets passed to `enqueue()` to construct a SHADOW slot during cross-bucket requeue. After unification, this site simplifies (the requeue's shadow slot doesn't track deadlines anymore — the existing comment at lines 712-720 already notes the shadow is "correct-but-conservative" and the original waiter keeps using its original slot).

---

## Phase 0: Safety net

### Task 0.1: Tag the pre-implementation commit

- [ ] **Step 1: Verify HEAD**

Run: `git log -1 --format='%H %s'`
Expected: `61a84c60... 🎯 DESIGN_FUTEX_DEADLINE_UNIFICATION: ...` (or later if more docs landed).

- [ ] **Step 2: Verify branch**

Run: `git branch --show-current`
Expected: `feat/futex-unification`. If not, STOP.

- [ ] **Step 3: Create rescue tag**

Run: `git tag -a pre-futex-deadline-unification-2026-05-08 -m "Last commit before futex deadline unification."`

- [ ] **Step 4: Push tag**

Run: `git push origin pre-futex-deadline-unification-2026-05-08`
Expected: `* [new tag] pre-futex-deadline-unification-2026-05-08 ...`

### Task 0.2: Establish baseline

- [ ] **Step 1: Confirm both feature configs build**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -3
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -3
```
Both expected: `Finished ...`. Record the warning count (~216).

- [ ] **Step 2: Boot smoke (regression baseline)**

Run:
```bash
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
PID=$!; sleep 90
if kill -0 $PID 2>/dev/null; then kill $PID 2>/dev/null; fi
wait $PID 2>/dev/null
```
Expected: `[smoke] PASS`.

- [ ] **Step 3: Selftests smoke (3 scheduler sub-tests today)**

Run:
```bash
python3 scripts/qemu_selftests_smoke.py 2>&1 | tail -15 &
PID=$!; sleep 120
if kill -0 $PID 2>/dev/null; then kill $PID 2>/dev/null; fi
wait $PID 2>/dev/null
```
Expected: `[selftests-smoke] PASS — all sub-tests reported PASS, no FAIL lines.`

If either fails, STOP — baseline is broken.

---

## Phase 1: Add `deadline_ticks` field to `BlockReason::FutexWait` + new wake arm

Both edits in `src/batcave/linux/threads.rs`. Combined into one commit because the wake arm references the new field.

### Task 1.1: Update `BlockReason::FutexWait` variant

**Files:**
- Modify: `src/batcave/linux/threads.rs` (around line 295-302, the `pub enum BlockReason`)

- [ ] **Step 1: Locate the enum**

Run: `grep -n '^pub enum BlockReason' src/batcave/linux/threads.rs`
Expected: hit around line 295.

- [ ] **Step 2: Update the FutexWait variant**

Use Edit. Find:

```rust
pub enum BlockReason {
    FutexWait { uaddr: u64, val: u32 },
    EpollWait { epfd: i32, deadline_ticks: u64 },  // 0 = infinite (epoll-only sentinel)
    Nanosleep { deadline_ticks: u64 },             // always concrete; 0 = invalid
    Join { target_tid: u32 },
    IoWait,
}
```

Replace with:

```rust
pub enum BlockReason {
    FutexWait { uaddr: u64, val: u32, deadline_ticks: u64 },  // 0 = no deadline
    EpollWait { epfd: i32, deadline_ticks: u64 },              // 0 = infinite (epoll-only sentinel)
    Nanosleep { deadline_ticks: u64 },                          // always concrete; 0 = invalid
    Join { target_tid: u32 },
    IoWait,
}
```

### Task 1.2: Update existing FutexWait pattern matches

The new field needs handling in every place that matches `BlockReason::FutexWait`. The compiler will tell us if we miss any.

- [ ] **Step 1: Find all FutexWait matches**

Run: `grep -n 'BlockReason::FutexWait' src/batcave/linux/threads.rs`

You should see:
- ~line 297: the enum definition (already updated in Task 1.1).
- ~line 1290: diagnostic encoding match. Update to ignore the new field via `..`:
  ```rust
  BlockReason::FutexWait { uaddr, val, .. } => (uaddr, val as u64),
  ```
  Reason: spec says preserve `(a1, a2)` semantics for SKIP-DEADLOCK log compat. The new field is intentionally NOT in the encoding.
- ~line 1298: `kind_disc` match — uses `..` already; no change needed.
- ~line 1715: `wake_n_on` walks slots and matches `FutexWait { uaddr: a, .. }` — already uses `..`; no change needed.
- ~line 1739: `block_current_thread(BlockReason::FutexWait { uaddr, val })` — UPDATE this construction to include `deadline_ticks: 0` (legacy non-deadline-bearing call).
- ~line 1858: dump code `match` — UPDATE to `FutexWait { uaddr, val, .. }`. The dump line at 1859 prints uaddr; deadline is fine to drop here.

- [ ] **Step 2: Apply each fix**

Use Edit for the diagnostic encoding (line ~1290):

Find:
```rust
                            BlockReason::FutexWait { uaddr, val } => (uaddr, val as u64),
```
Replace with:
```rust
                            BlockReason::FutexWait { uaddr, val, .. } => (uaddr, val as u64),
```

Use Edit for `block_current_thread` call (line ~1739):

Find:
```rust
    block_current_thread(BlockReason::FutexWait { uaddr, val });
```
Replace with:
```rust
    block_current_thread(BlockReason::FutexWait { uaddr, val, deadline_ticks: 0 });
```

Use Edit for the dump match (line ~1858):

Find:
```rust
                ThreadState::Blocked(BlockReason::FutexWait { uaddr, val }) => {
```
Replace with:
```rust
                ThreadState::Blocked(BlockReason::FutexWait { uaddr, val, .. }) => {
```

### Task 1.3: Add the FutexWait arm to `wake_expired_deadlines`

**Files:**
- Modify: `src/batcave/linux/threads.rs` (the `pub fn wake_expired_deadlines` body)

- [ ] **Step 1: Locate the function**

Run: `grep -n '^pub fn wake_expired_deadlines' src/batcave/linux/threads.rs`
Expected: a single hit.

- [ ] **Step 2: Add the arm**

Use Edit. Find:

```rust
            let should_wake = match slot.state {
                ThreadState::Blocked(BlockReason::EpollWait { deadline_ticks, .. })
                    if deadline_ticks != 0 && now >= deadline_ticks => true,
                ThreadState::Blocked(BlockReason::Nanosleep { deadline_ticks })
                    if now >= deadline_ticks => true,
                _ => false,
            };
```

Replace with:

```rust
            let should_wake = match slot.state {
                ThreadState::Blocked(BlockReason::EpollWait { deadline_ticks, .. })
                    if deadline_ticks != 0 && now >= deadline_ticks => true,
                ThreadState::Blocked(BlockReason::Nanosleep { deadline_ticks })
                    if now >= deadline_ticks => true,
                ThreadState::Blocked(BlockReason::FutexWait { deadline_ticks, .. })
                    if deadline_ticks != 0 && now >= deadline_ticks => true,
                _ => false,
            };
```

### Task 1.4: Verify and commit

- [ ] **Step 1: Build check both feature configs**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -5
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5
```
Both expected: `Finished ...`. If errors mention missing `deadline_ticks` on `FutexWait` patterns, find the missed match site and fix it (use `..` to ignore the field unless the new field is needed there).

- [ ] **Step 2: Commit**

Run:
```bash
git add src/batcave/linux/threads.rs
git commit -m "$(cat <<'EOF'
🎯 futex-unification: add deadline_ticks to BlockReason::FutexWait

Adds the deadline_ticks field to BlockReason::FutexWait (u64; 0 =
no deadline). wake_expired_deadlines gains a third arm that wakes
any FutexWait waiter whose deadline has passed, symmetric with
the existing EpollWait + Nanosleep arms.

Existing FutexWait pattern matches updated to use `..` to ignore
the new field. The diagnostic encoding at threads.rs:~1290
intentionally keeps (a1, a2) = (uaddr, val) — preserves
SKIP-DEADLOCK log compat per the spec. block_current_thread's
internal FutexWait construction sets deadline_ticks: 0 (legacy
no-timeout path).

Phase 1 of the futex deadline unification plan. Purely additive
on the threads.rs side; futex.rs continues to use WaitSlot::
deadline_ticks until Phase 3.

See DESIGN_FUTEX_DEADLINE_UNIFICATION.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 2: Rewrite `park_slot` to use the new BlockReason field + park-current loop shape

After this commit, the deadline lives in BOTH `WaitSlot::deadline_ticks` AND `BlockReason::FutexWait::deadline_ticks` — duplication is intentional for one commit so the rewrite can be small. Phase 3 removes the WaitSlot field.

### Task 2.1: Update `park_slot` signature + body

**Files:**
- Modify: `src/batcave/linux/futex.rs` (the `fn park_slot` definition and body, ~lines 275-340)

- [ ] **Step 1: Locate park_slot**

Run: `grep -n '^fn park_slot' src/batcave/linux/futex.rs`
Expected: single hit at line ~275.

- [ ] **Step 2: Read the current body**

Run: `sed -n '270,340p' src/batcave/linux/futex.rs`

- [ ] **Step 3: Replace the function**

Use Edit. Find the entire current function body (from `// ─── Park loop ───` comment block through the closing `}`). Replace with:

```rust
// ─── Park loop ───────────────────────────────────────────────────────────
//
// Block-and-resume: mark the current thread Blocked
// (BlockReason::FutexWait), call schedule() to yield, re-check the
// woken flag + deadline on each resume. FUTEX_WAKE flips matching
// slots' woken bits and wakes their threads via wake_thread(tid). The
// timer-tick wake_expired_deadlines pass also wakes us when our
// deadline expires (via the FutexWait arm on BlockReason).
//
// Lock + IRQ ordering invariants (mirrors park_current in
// linux::threads): bucket lock is NEVER held across schedule() or
// wfi; IrqGuard is NEVER held across schedule() or wfi.
fn park_slot(b: &Bucket, slot: usize, uaddr: u64, val: u32, deadline_ticks: u64) -> i64 {
    let s = &b.slots[slot];
    let _ = val;
    loop {
        let g = crate::kernel::sync::IrqGuard::new();
        bucket_lock(b);
        // Fast-path: already woken?
        if s.woken.load(Ordering::Acquire) {
            release(b, slot);
            bucket_unlock(b);
            drop(g);
            return 0;
        }
        // Deadline expired?
        if deadline_ticks != 0 && cntpct() >= deadline_ticks {
            release(b, slot);
            bucket_unlock(b);
            drop(g);
            return ETIMEDOUT;
        }
        // Mark self Blocked with the deadline carried on BlockReason.
        // The wake_expired_deadlines tick pass observes this and flips
        // our state to Runnable when cntpct_el0 crosses deadline_ticks.
        crate::batcave::linux::threads::mark_current_blocked(
            crate::batcave::linux::threads::BlockReason::FutexWait {
                uaddr,
                val,
                deadline_ticks,
            },
        );
        bucket_unlock(b);
        drop(g);
        // Yield. schedule() switches to another Runnable thread or
        // returns immediately if there's no one else to run.
        crate::batcave::linux::threads::schedule();
        // Resumed (or schedule() returned because no other Runnable):
        //   * If a waker (FUTEX_WAKE → wake_thread, or
        //     wake_expired_deadlines from the timer tick) flipped our
        //     state, current_thread_blocked() returns false and we
        //     loop to re-check the bucket.
        //   * If we're still Blocked (single-thread cave with no
        //     pending wake), wfi until any IRQ fires, then loop.
        if !crate::batcave::linux::threads::current_thread_blocked() {
            continue;
        }
        unsafe { core::arch::asm!("wfi"); }
        // implicit loop continue → re-check
    }
}
```

Key changes from the prior body:
- `deadline_ticks: u64` added as a parameter (read from caller, not from `s.deadline_ticks`).
- The `mark_current_blocked` call now constructs `FutexWait { uaddr, val, deadline_ticks }` with the deadline.
- Removed `mark_current_runnable()` after `schedule()` (defensive marker; conflicts with blocker-marks-blocked rule).
- Added `wfi` between schedule resumes.
- The `let _ = val;` line addresses an unused-variable lint when `val` is only consumed by the `mark_current_blocked` macro expansion.

### Task 2.2: Update park_slot call sites

**Files:**
- Modify: `src/batcave/linux/futex.rs` (the two `park_slot(...)` calls in `futex_wait` and `futex_wait_bitset`)

- [ ] **Step 1: Find call sites**

Run: `grep -n 'park_slot(' src/batcave/linux/futex.rs`
Expected: 2 call sites (one in futex_wait, one in futex_wait_bitset). Plus the function definition itself.

- [ ] **Step 2: Update call sites to pass deadline**

In each call site, the `deadline` local variable is in scope (computed earlier in the function from `timeout_ns`). Use Edit at each:

Find (in `futex_wait`):
```rust
    let result = park_slot(b, slot, uaddr, val);
```
or whatever the exact call shape is. Replace with:
```rust
    let result = park_slot(b, slot, uaddr, val, deadline);
```

Same for the futex_wait_bitset call site.

If the call shapes use different names or destructured args, adapt — the goal is to pass the existing `deadline` local through.

### Task 2.3: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. The function signature change should be locally consistent (definition + 2 call sites updated).

- [ ] **Step 2: Commit**

Run:
```bash
git add src/batcave/linux/futex.rs
git commit -m "$(cat <<'EOF'
🎯 futex-unification: park_slot uses deadline param + BlockReason field

Rewrites park_slot to use the park-current loop shape from the
scheduler thread:
- deadline_ticks now passed as a u64 parameter (read from caller's
  computed deadline, not from WaitSlot::deadline_ticks).
- mark_current_blocked constructs FutexWait { uaddr, val,
  deadline_ticks } so the timer-tick wake_expired_deadlines pass
  can wake us on deadline expiry.
- Defensive mark_current_runnable() after schedule() removed (per
  the blocker-marks-blocked, waker-marks-runnable rule).
- wfi added between schedule resumes so single-thread caves don't
  busy-loop schedule() calls.

Call sites in futex_wait and futex_wait_bitset updated to pass the
existing local `deadline` variable through.

Phase 2 of the futex deadline unification plan. After this commit,
the deadline lives in BOTH WaitSlot::deadline_ticks AND BlockReason
::FutexWait — Phase 3 removes the WaitSlot field for single source
of truth.

See DESIGN_FUTEX_DEADLINE_UNIFICATION.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 3: Drop `WaitSlot::deadline_ticks` + simplify `enqueue()` signature

Single source of truth: deadline now lives only on BlockReason. `WaitSlot` loses the field; `enqueue()` loses the parameter.

### Task 3.1: Remove the field from `WaitSlot`

**Files:**
- Modify: `src/batcave/linux/futex.rs` (struct definition ~line 87-101, ctor ~line 103-114)

- [ ] **Step 1: Locate the struct**

Run: `grep -nE '^struct WaitSlot|deadline_ticks' src/batcave/linux/futex.rs | head -10`

- [ ] **Step 2: Update struct definition**

Use Edit. Find:
```rust
struct WaitSlot {
    in_use: AtomicBool,
    uaddr: AtomicU64,
    tid: AtomicUsize,
    woken: AtomicBool,
    // Deadline in cntpct_el0 ticks. 0 == no deadline.
    deadline_ticks: AtomicU64,
    bitset: AtomicU32,
}
```

Replace with:
```rust
struct WaitSlot {
    in_use: AtomicBool,
    uaddr: AtomicU64,
    tid: AtomicUsize,
    woken: AtomicBool,
    bitset: AtomicU32,
}
```

- [ ] **Step 3: Update the const ctor `new()`**

Use Edit. Find:
```rust
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
```

Replace with:
```rust
impl WaitSlot {
    const fn new() -> Self {
        Self {
            in_use: AtomicBool::new(false),
            uaddr: AtomicU64::new(0),
            tid: AtomicUsize::new(0),
            woken: AtomicBool::new(false),
            bitset: AtomicU32::new(0xFFFF_FFFF),
        }
    }
}
```

- [ ] **Step 4: Update the field-listing comment near line 24**

Use Edit. Find:
```
//   - Each slot stores: { in_use, uaddr, tid, woken_flag, deadline_ticks }.
```

Replace with:
```
//   - Each slot stores: { in_use, uaddr, tid, woken_flag, bitset }.
//     (Deadlines live on BlockReason::FutexWait, not on the slot.)
```

### Task 3.2: Update `enqueue()` signature

**Files:**
- Modify: `src/batcave/linux/futex.rs` (the `fn enqueue` definition ~line 241 and its body)

- [ ] **Step 1: Update the signature + body**

Use Edit. Find:

```rust
fn enqueue(b: &Bucket, uaddr: u64, tid: usize, deadline_ticks: u64, bitset: u32) -> Option<usize> {
```

Replace with:

```rust
fn enqueue(b: &Bucket, uaddr: u64, tid: usize, bitset: u32) -> Option<usize> {
```

Then find the line in the body (around line 246) that stored the deadline:

```rust
            s.deadline_ticks.store(deadline_ticks, Ordering::Relaxed);
```

Delete it.

- [ ] **Step 2: Update `release()` to drop the deadline reset**

Run: `grep -n 'fn release' src/batcave/linux/futex.rs`
Expected: hit at line ~259.

In `release()`, remove the line:
```rust
    s.deadline_ticks.store(0, Ordering::Relaxed);
```

The other release fields (`in_use`, `uaddr`, `tid`, `woken`, `bitset`) stay.

### Task 3.3: Update `enqueue()` call sites

The signature change breaks 3 call sites:

- [ ] **Step 1: Find them all**

Run: `grep -n 'enqueue(' src/batcave/linux/futex.rs`
Expected: 4 hits — 1 definition + 3 calls (lines ~397 in futex_wait, ~444 in futex_wait_bitset, ~704 in requeue_impl).

- [ ] **Step 2: Update futex_wait call (~line 397)**

Find:
```rust
    let slot = match enqueue(b, uaddr, current_tid(), deadline, 0xFFFF_FFFF) {
```

Replace with:
```rust
    let slot = match enqueue(b, uaddr, current_tid(), 0xFFFF_FFFF) {
```

The `deadline` local stays in scope — it's still passed to `park_slot` (Phase 2 wired that). Just dropping the now-unused param to enqueue.

- [ ] **Step 3: Update futex_wait_bitset call (~line 444)**

Find:
```rust
    let slot = match enqueue(b, uaddr, current_tid(), deadline, bitset) {
```

Replace with:
```rust
    let slot = match enqueue(b, uaddr, current_tid(), bitset) {
```

- [ ] **Step 4: Update requeue_impl call (~line 704)**

Find:
```rust
            match enqueue(
                b2,
                uaddr2,
                s.tid.load(Ordering::Relaxed),
                s.deadline_ticks.load(Ordering::Relaxed),
                s.bitset.load(Ordering::Relaxed),
            ) {
```

Replace with:
```rust
            match enqueue(
                b2,
                uaddr2,
                s.tid.load(Ordering::Relaxed),
                s.bitset.load(Ordering::Relaxed),
            ) {
```

Note: this site previously read `s.deadline_ticks` to construct the new shadow slot's deadline. After this change, the shadow slot has no deadline tracking at the WaitSlot level — but the original waiter's BlockReason still carries the deadline (set by park_slot in Phase 2), so the wake mechanism continues to work for the original waiter. The shadow slot is "correct-but-conservative" per the existing comment.

### Task 3.4: Verify and commit

- [ ] **Step 1: Final grep check**

Run: `grep -n 'deadline_ticks' src/batcave/linux/futex.rs`

Expected output: ZERO hits. If anything remains, either it's a stale comment (delete it) or a missed read site (fix it). Update the doc-comment block at lines ~30-44 if it still references `deadline_ticks` on the slot.

- [ ] **Step 2: Build check**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -5
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5
```
Both expected: `Finished ...`.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/batcave/linux/futex.rs
git commit -m "$(cat <<'EOF'
🎯 futex-unification: drop WaitSlot::deadline_ticks; simplify enqueue()

Removes WaitSlot::deadline_ticks (struct field, ctor, release reset,
3 enqueue() call sites). enqueue() signature drops the deadline_ticks
parameter. Deadline now lives only on BlockReason::FutexWait, set
by park_slot via mark_current_blocked.

The requeue_impl shadow-slot path no longer carries deadline. The
existing 'correct-but-conservative' comment notes that the original
waiter keeps using its original slot, so its deadline (on
BlockReason) continues to be observed by wake_expired_deadlines.

Phase 3 of the futex deadline unification plan. Single source of
truth achieved: all blocker deadlines parametrize via BlockReason.

See DESIGN_FUTEX_DEADLINE_UNIFICATION.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 4: Add 4th scheduler selftest sub-test

### Task 4.1: Add `futex-deadline-fires` sub-test

**Files:**
- Modify: `src/ui/shell.rs` (the `cmd_scheduler_selftest` function, which is `#[cfg(feature = "selftest-on-boot")]`)

- [ ] **Step 1: Locate cmd_scheduler_selftest**

Run: `grep -n '^pub(crate) fn cmd_scheduler_selftest\|^fn cmd_scheduler_selftest' src/ui/shell.rs`
Expected: a single hit.

- [ ] **Step 2: Add the new sub-test as a 4th block**

Use Edit. Find the closing `}` of the `epoll-event-wake` sub-test block (the third sub-test, ending with `test_release_slot(s2);` then `}`). Insert a new block before the function's closing `}`:

```rust

    // Sub-test 4: futex deadline fires — install a Blocked slot with
    // already-past FutexWait deadline, run the wake pass, observe Runnable.
    {
        let now = cntpct_el0();
        let past = now.saturating_sub(1);
        let slot = match test_install_blocked(
            BlockReason::FutexWait { uaddr: 0, val: 0, deadline_ticks: past }
        ) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: futex-deadline-fires (table full)\n");
                return;
            }
        };
        wake_expired_deadlines();
        match test_inspect_state(slot) {
            Some(ThreadState::Runnable) => {
                console::puts("  [scheduler-selftest] PASS: futex-deadline-fires\n");
            }
            _ => {
                console::puts("  [scheduler-selftest] FAIL: futex-deadline-fires (wrong state)\n");
            }
        }
        test_release_slot(slot);
    }
```

The block goes BEFORE the function's closing `}`. The exact insertion site is right after the `test_release_slot(s2);` line of sub-test 3.

### Task 4.2: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 2: Commit**

Run:
```bash
git add src/ui/shell.rs
git commit -m "🎯 futex-unification: 4th scheduler sub-test — futex-deadline-fires

Adds futex-deadline-fires to cmd_scheduler_selftest. Same pattern
as nanosleep-deadline-fires: install a Blocked slot with already-
past FutexWait deadline_ticks, run wake_expired_deadlines, assert
the slot transitioned to Runnable, release.

Phase 4 of the futex deadline unification plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

---

## Phase 5: Update `qemu_selftests_smoke.py` threshold + run smoke

### Task 5.1: Bump scheduler PASS count threshold from 3 to 4

**Files:**
- Modify: `scripts/qemu_selftests_smoke.py`

- [ ] **Step 1: Find the threshold check**

Run: `grep -n 'sched_pass.*<\|len(sched_pass)' scripts/qemu_selftests_smoke.py`
Expected: a single hit, around the line that currently says `if len(sched_pass) < 3:`.

- [ ] **Step 2: Update from 3 to 4**

Use Edit. Find:
```python
        if len(sched_pass) < 3:
            print(
                f"[selftests-smoke] FAIL — expected 3 scheduler PASS sub-tests, got {len(sched_pass)}",
                file=sys.stderr,
            )
```

Replace with:
```python
        if len(sched_pass) < 4:
            print(
                f"[selftests-smoke] FAIL — expected 4 scheduler PASS sub-tests, got {len(sched_pass)}",
                file=sys.stderr,
            )
```

### Task 5.2: Run the smoke

- [ ] **Step 1: Run with 120s ceiling**

Run:
```bash
python3 scripts/qemu_selftests_smoke.py 2>&1 | tail -15 &
PID=$!; sleep 120
if kill -0 $PID 2>/dev/null; then kill $PID 2>/dev/null; fi
wait $PID 2>/dev/null
```

Expected output includes 4 unique scheduler PASS lines and overall PASS:
```
[selftests-smoke]   x509 PASS: bad-bytes
[selftests-smoke]   x509 PASS: hostname-mismatch
[selftests-smoke]   scheduler PASS: epoll-event-wake
[selftests-smoke]   scheduler PASS: futex-deadline-fires
[selftests-smoke]   scheduler PASS: nanosleep-deadline-fires
[selftests-smoke]   scheduler PASS: wake-expired-deadlines-noop
[selftests-smoke] PASS — all sub-tests reported PASS, no FAIL lines.
```

If any FAIL appears, capture the line and STOP — most likely cause is a wake-arm bug in Phase 1 or a park_slot bug in Phase 2.

### Task 5.3: Commit

- [ ] **Step 1: Commit**

Run:
```bash
git add scripts/qemu_selftests_smoke.py
git commit -m "🎯 futex-unification: bump qemu_selftests_smoke scheduler threshold to 4

The new futex-deadline-fires sub-test brings scheduler-selftest
PASS count from 3 to 4. Threshold updated.

Smoke output (verified locally):
  [selftests-smoke]   scheduler PASS: epoll-event-wake
  [selftests-smoke]   scheduler PASS: futex-deadline-fires
  [selftests-smoke]   scheduler PASS: nanosleep-deadline-fires
  [selftests-smoke]   scheduler PASS: wake-expired-deadlines-noop
  [selftests-smoke] PASS — all sub-tests reported PASS, no FAIL lines.

Phase 5 of the futex deadline unification plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

---

## Phase 6: Final acceptance + journal + push

### Task 6.1: Static greps

- [ ] **Step 1: Confirm no WaitSlot::deadline_ticks references**

Run: `grep -n 'deadline_ticks' src/batcave/linux/futex.rs`
Expected: no matches outside doc comments referencing BlockReason. The struct field, ctor, release reset, enqueue() param, and 3 call sites should all be clean.

- [ ] **Step 2: Confirm BlockReason::FutexWait constructions all include deadline_ticks**

Run: `grep -n 'BlockReason::FutexWait' src/`
Expected: every construction includes `deadline_ticks` (either explicit value or `: 0`). Pattern matches use `..` if they don't need the field.

### Task 6.2: Run full smoke + boot smoke

- [ ] **Step 1: Boot smoke (regression)**

Run:
```bash
cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
PID=$!; sleep 90
if kill -0 $PID 2>/dev/null; then kill $PID 2>/dev/null; fi
wait $PID 2>/dev/null
```
Expected: `[smoke] PASS`.

- [ ] **Step 2: Selftests smoke (already run in Phase 5; verify still passes)**

Skip if Phase 5's run was clean.

### Task 6.3: Manual review checkpoint — park_slot lock/IRQ invariants

- [ ] **Step 1: Re-read park_slot**

Open `src/batcave/linux/futex.rs`, find `fn park_slot`, re-read carefully.

Confirm by inspection:
- The bucket lock is taken via `bucket_lock(b)` and released via `bucket_unlock(b)` BEFORE `schedule()` and BEFORE `wfi`.
- `IrqGuard` is constructed inside the loop iteration and dropped (via `drop(g)`) BEFORE `schedule()` and BEFORE `wfi`.
- The `wfi` is reached only when `current_thread_blocked()` returns true (i.e., we're still in `Blocked` state after `schedule()` returned).
- No early `return` paths leave the function while state == Blocked. The two `return` paths (woken, ETIMEDOUT) both call `release(b, slot)` first — release leaves the slot Free, but the thread state at those returns is still whatever it was when we entered the loop (Running, since we haven't called mark_current_blocked yet on this iteration).

If any invariant is violated, fix and add a Phase 6.x commit before the journal.

### Task 6.4: Write journal entry

**Files:**
- Modify: `docs/SESSION_JOURNAL.md` — prepend a new entry above the captures-untrack one.

- [ ] **Step 1: Draft the entry**

Cover:
- This thread's context (continuation of post-no-browser cleanup; first of the 3 trimmed items).
- 5 implementation phases with commit hashes.
- LOC stats from `git log --shortstat pre-futex-deadline-unification-2026-05-08..HEAD`.
- State of tree: futex deadlines now live on BlockReason; wake_expired_deadlines covers all three blocker types.
- Tag `pre-futex-deadline-unification-2026-05-08` for revival.
- What's next: TLS PQ bug investigation, then warning cleanup.

Match journal voice (terse, technical, honest). ~80-120 lines.

- [ ] **Step 2: Commit**

Run:
```bash
git add docs/SESSION_JOURNAL.md
git commit -m "📝 SESSION_JOURNAL: futex deadline unification complete

Phase 6 of the futex deadline unification plan — handoff point.
See DESIGN_FUTEX_DEADLINE_UNIFICATION.md for strategy,
PLAN_FUTEX_DEADLINE_UNIFICATION.md for the per-phase how.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

### Task 6.5: Push

- [ ] **Step 1: Push branch + tag**

Run:
```bash
git push -u origin feat/futex-unification 2>&1 | tail -5
git push origin pre-futex-deadline-unification-2026-05-08 2>&1 | tail -3
```

### Task 6.6: Hand off to finishing-a-development-branch

- [ ] **Step 1: Invoke the skill**

Use `superpowers:finishing-a-development-branch`. Recommended: **Push and create a Pull Request** against `feat/js-engine-browser-posix`.

PR title: `futex-unification: deadline_ticks moves to BlockReason::FutexWait`

PR body should reference `DESIGN_FUTEX_DEADLINE_UNIFICATION.md`, the rescue tag, summarize the 5 implementation phases, and list the verified tests (cargo check both feature configs, boot smoke, selftests smoke with 4 scheduler sub-tests).

---

## Done

After Phase 6:
- `feat/futex-unification` has ~6 commits documenting each phase.
- Tag `pre-futex-deadline-unification-2026-05-08` preserves pre-implementation state.
- Single source of truth: blocker deadlines all live on BlockReason.
- `wake_expired_deadlines` handles FutexWait, EpollWait, Nanosleep symmetrically.
- park_slot uses the park-current loop shape with documented lock/IRQ invariants.
- 4 unique scheduler selftest sub-tests pass.

Next up per the post-priority cleanup queue: **TLS PQ handshake bug investigation**, then **warning cleanup pass**.

🦇
