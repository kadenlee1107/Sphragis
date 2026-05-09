# Scheduler Block-on-Deadline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the chain of changes from `DESIGN_SCHEDULER_BLOCK_ON.md` — replace `epoll_pwait`'s `SPIN_PER_MS` heuristic and `sys_nanosleep`'s hybrid spin-yield with `park_current(reason)`-based sleeps, add a `wake_expired_deadlines()` tick pass + `wake_epoll_waiters(epfd)` event waker, rename `BlockReason::EpollWait::timeout_ms` and `BlockReason::Nanosleep::deadline_ns` to `deadline_ticks`, clean up two stale `TODO(sched)` comments in `futex.rs`, add `cmd_scheduler_selftest`, extend `selftest-on-boot` to run it, and rename `qemu_x509_smoke.py` to `qemu_selftests_smoke.py` covering both selftests.

**Architecture:** Bottom-up. Add helpers and wakers first (`Phase 1-2`, additive). Hook the tick pass into the existing `kernel::scheduler::tick()` (`Phase 3`, one line). Atomic field rename + diagnostic update (`Phase 4`) — only the threads.rs:~1290 site references the old names today, so the rename has no other consumer fallout. Rewrite `sys_nanosleep` (`Phase 5`) and `epoll_pwait` (`Phase 6`) to use `park_current` and construct the new `BlockReason` variants. Comment cleanup (`Phase 7`). Test infrastructure (`Phase 8-11`). Final acceptance (`Phase 12`).

Each phase ends with `cargo check --target aarch64-unknown-none --release`. The branch is `feat/scheduler-block-on`, branched from `main` (the default branch was renamed from `feat/js-engine-browser-posix` to `main` on 2026-05-08 after the no-browser pivot — see commit `95d31371`).

**Tech Stack:** Rust + Cargo, bare-metal `aarch64-unknown-none`, nightly toolchain, single-binary kernel. ARMv8 generic timer (`cntpct_el0`, `cntfrq_el0`) is the clock source for all deadlines. WFI primitive is the kernel's idle-wait. No `cargo test` — verification is `cargo check` + `qemu_boot_smoke.py` (regression) + the new `qemu_selftests_smoke.py` (real proof of behavior).

**Reference spec:** `DESIGN_SCHEDULER_BLOCK_ON.md` (root). Read it before starting. This document is the *how*.

**Pre-deletion HEAD:** `54dbd855` on branch `feat/scheduler-block-on` (the spec commit). Verify before tagging.

---

## Phase 0: Safety net

### Task 0.1: Tag the pre-deletion commit

**Files:** none (git operation only).

- [ ] **Step 1: Verify HEAD matches the spec commit**

Run: `git log -1 --format='%H %s'`
Expected: `54dbd855... 🎯 DESIGN_SCHEDULER_BLOCK_ON: park-on-deadline for epoll + nanosleep` (or a later commit on `feat/scheduler-block-on` if other doc work has landed since).

- [ ] **Step 2: Verify branch is `feat/scheduler-block-on`**

Run: `git branch --show-current`
Expected: `feat/scheduler-block-on`. If it says `main` (or any other branch), STOP and create the feature branch first: `git switch -c feat/scheduler-block-on`.

- [ ] **Step 3: Create the rescue tag**

Run: `git tag -a pre-scheduler-block-on-2026-05-07 -m "Last commit before park-on-deadline scheduler changes. See DESIGN_SCHEDULER_BLOCK_ON.md for rationale."`

- [ ] **Step 4: Push the tag**

Run: `git push origin pre-scheduler-block-on-2026-05-07`
Expected: `* [new tag] pre-scheduler-block-on-2026-05-07 -> pre-scheduler-block-on-2026-05-07`.

### Task 0.2: Establish baseline

- [ ] **Step 1: Confirm cargo check passes**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. Record the warning count from the line `warning: bat_os (bin "bat_os") generated N warnings` — that's the post-TLS-hardening baseline (~216).

- [ ] **Step 2: Confirm release build with selftest-on-boot also passes**

Run: `cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5`
Expected: `Finished ...`. Phase 1-2 add new code that this feature path will eventually call; the build needs to remain clean across both feature configurations at every step.

- [ ] **Step 3: Confirm boot smoke passes pre-changes**

Run (with a 90s ceiling):
```bash
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
SMOKE_PID=$!
sleep 90
if kill -0 $SMOKE_PID 2>/dev/null; then kill $SMOKE_PID 2>/dev/null; fi
wait $SMOKE_PID 2>/dev/null
```
Expected: `[smoke] PASS — kernel boots, all required subsystems init, no deleted-browser symbols leaked through.` If the smoke fails here, STOP — the baseline is broken before scheduler work starts.

- [ ] **Step 4: Confirm x509 selftest smoke passes pre-changes**

Run (with a 120s ceiling because it builds the kernel with the feature):
```bash
python3 scripts/qemu_x509_smoke.py 2>&1 | tail -10 &
DRIVER_PID=$!
sleep 120
if kill -0 $DRIVER_PID 2>/dev/null; then kill $DRIVER_PID 2>/dev/null; fi
wait $DRIVER_PID 2>/dev/null
```
Expected: `[x509-smoke] PASS — both sub-tests reported PASS, no FAIL lines.`

---

## Phase 1: Add helpers (`cntpct_el0`, `ms_to_ticks`, `current_thread_blocked`, `park_current`)

Purely additive. Build stays clean. Lays groundwork for Phases 5-6.

### Task 1.1: Add time helpers

**Files:**
- Modify: `src/batcave/linux/threads.rs`

- [ ] **Step 1: Find a good insertion point**

Run: `grep -n '^pub fn schedule\|^pub fn current_tid\|^pub fn mark_current_runnable' src/batcave/linux/threads.rs | head -3`
Find a stable anchor near the bottom of the file's "public scheduler API" section. Insert the new helpers immediately above the first scheduler-API public function found.

- [ ] **Step 2: Add the time helpers**

Use Edit to insert this block (before `pub fn schedule()` at the location identified in Step 1):

```rust
// ─── Time helpers (cntpct_el0 / cntfrq_el0) ──────────────────────────────
//
// Bat_OS uses ARMv8 generic timer ticks as the canonical deadline unit.
// All deadlines stored in BlockReason are absolute cntpct_el0 values.
// See DESIGN_SCHEDULER_BLOCK_ON.md decision #2.

/// Read the ARM generic timer's current physical count (EL0).
/// Returns absolute ticks since boot (or wherever the firmware reset it).
#[inline]
pub fn cntpct_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

/// Read the ARM generic timer's frequency in Hz. Constant per boot.
#[inline]
fn cntfrq_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v); }
    v
}

/// Convert milliseconds to cntpct_el0 ticks using cntfrq_el0.
/// Multiply-then-divide preserves sub-1000Hz precision.
/// Saturating mul prevents overflow panic on absurd inputs.
#[inline]
pub fn ms_to_ticks(ms: u32) -> u64 {
    let freq = cntfrq_el0();
    (ms as u64).saturating_mul(freq) / 1000
}
```

- [ ] **Step 3: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. May produce `function never used` warnings on the new helpers — that's fine; subsequent phases call them.

### Task 1.2: Add `current_thread_blocked` helper

**Files:**
- Modify: `src/batcave/linux/threads.rs`

- [ ] **Step 1: Insert below the time helpers**

```rust
/// Returns `true` iff the current thread's slot in the table exists AND
/// its state is ThreadState::Blocked(_). Reads under the table lock.
/// Returns `false` (non-blocking) if the current slot is missing — the
/// caller (park_current) treats that as "not blocked" and falls through
/// gracefully rather than spinning forever.
pub fn current_thread_blocked() -> bool {
    let me = current_tid();
    with_table(|t| {
        let Some(idx) = slot_of(t, me) else { return false; };
        matches!(t[idx].state, ThreadState::Blocked(_))
    })
}
```

- [ ] **Step 2: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. The function uses `current_tid`, `with_table`, `slot_of`, and `ThreadState::Blocked` — all already exist in the file (you saw them during exploration).

### Task 1.3: Add `park_current(reason)` primitive

**Files:**
- Modify: `src/batcave/linux/threads.rs`

- [ ] **Step 1: Insert below `current_thread_blocked`**

```rust
/// Park the current thread on `reason`. Does NOT return while the
/// calling thread's state is ThreadState::Blocked(_). Loops between
/// `schedule()` (which switches away if anyone is Runnable) and `wfi`
/// (which idles until any interrupt fires; the timer IRQ runs
/// wake_expired_deadlines, which may flip our state Blocked→Runnable).
///
/// Lock + IRQ ordering invariants (see DESIGN_SCHEDULER_BLOCK_ON.md
/// decision #8):
///
///   - The threads-table lock is NEVER held across `wfi`.
///     mark_current_blocked takes the lock briefly, releases it.
///     schedule() takes its own lock internally. The wfi runs lock-free.
///   - Interrupts are NEVER masked across `wfi`. mark_current_blocked
///     may briefly take an IrqGuard for atomicity but releases it before
///     schedule(). The wfi must execute with interrupts enabled or the
///     timer IRQ can't fire and deadline-bearing sleepers never wake.
pub fn park_current(reason: BlockReason) {
    mark_current_blocked(reason);
    loop {
        // schedule() switches to another Runnable thread if any. When
        // control returns here, either:
        //   * Another thread ran, was eventually rescheduled away, and
        //     a waker (event-driven via wake_thread / wake_epoll_waiters,
        //     or deadline-driven via wake_expired_deadlines) flipped our
        //     state Blocked→Runnable. We resume; check below exits loop.
        //   * No other Runnable thread existed. schedule() returned
        //     immediately. Our state is still Blocked. Drop to wfi and
        //     wait for any interrupt; on resume re-check state.
        schedule();
        if !current_thread_blocked() { break; }
        // Still blocked, no one else to run. Idle until the next IRQ.
        // Interrupts must be enabled here (see invariant above).
        unsafe { core::arch::asm!("wfi"); }
    }
}
```

- [ ] **Step 2: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 3: Manual review checkpoint — park_current loop invariant**

Read the body of `park_current` you just inserted. Confirm by inspection:
- The loop's exit condition is `!current_thread_blocked()`.
- Inside the loop, `wfi` is reached only if the previous `schedule()` returned with the thread still Blocked.
- No early `return` / `break` paths exist that would leave the function while `state == Blocked`.

If any of these are violated, the lock/IRQ invariants in the doc comment do not hold either — fix the body before commit.

### Task 1.4: Commit Phase 1

- [ ] **Step 1: Commit**

Run:
```bash
git add src/batcave/linux/threads.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: time helpers + park_current primitive

Adds:
- cntpct_el0() / ms_to_ticks() in linux::threads — canonical
  ARM generic-timer helpers used by the new park-on-deadline paths.
  Multiply-then-divide ms→ticks for sub-1000Hz precision.
- current_thread_blocked() — non-blocking check that the current
  thread's slot is in ThreadState::Blocked(_). Returns false on
  missing slot rather than spinning.
- park_current(reason) — the primitive that holds the invariant
  'does not return while the calling thread is Blocked.' Loops
  schedule() + wfi until a waker flips state to Runnable. Lock /
  IRQ ordering documented inline.

Phase 1 of the scheduler block-on plan. Purely additive; no callers
yet (Phase 5 is the first consumer). See DESIGN_SCHEDULER_BLOCK_ON.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 2: Add wakers (`wake_expired_deadlines`, `wake_epoll_waiters`)

Purely additive. Both functions walk the threads table and flip Blocked→Runnable on matches.

### Task 2.1: Add `wake_expired_deadlines`

**Files:**
- Modify: `src/batcave/linux/threads.rs`

- [ ] **Step 1: Insert below `park_current`**

```rust
/// Walk the threads table for Blocked threads whose BlockReason carries
/// an expired deadline_ticks; transition each to Runnable. Bounded
/// O(MAX_THREADS) per call (currently 256).
///
/// Called from kernel::scheduler::tick() once per timer IRQ. The pass
/// is the only waker for sys_nanosleep and the deadline-driven half of
/// epoll_pwait. (Event-driven epoll wakes go through wake_epoll_waiters.)
///
/// Futex's per-WaitSlot deadline lives in futex.rs, not BlockReason, so
/// this pass does not see it. Futex's existing post-resume re-check loop
/// handles its own timeouts; this is intentional (see
/// DESIGN_SCHEDULER_BLOCK_ON.md decision #5).
pub fn wake_expired_deadlines() {
    let now = cntpct_el0();
    with_table(|t| {
        for slot in t.iter_mut() {
            let should_wake = match slot.state {
                ThreadState::Blocked(BlockReason::EpollWait { deadline_ticks, .. })
                    if deadline_ticks != 0 && now >= deadline_ticks => true,
                ThreadState::Blocked(BlockReason::Nanosleep { deadline_ticks })
                    if now >= deadline_ticks => true,
                _ => false,
            };
            if should_wake {
                slot.state = ThreadState::Runnable;
            }
        }
    });
}
```

### **DEPENDENCY:** run Phase 4 before this task

This function references `BlockReason::EpollWait { deadline_ticks, .. }` and `BlockReason::Nanosleep { deadline_ticks }` — both field names come from Phase 4 (the rename). The pre-Phase-4 variants have `timeout_ms` / `deadline_ns`, so Step 2 below won't compile until Phase 4's rename is in place.

**Two equivalent orderings, pick one:**

- **(A) Run Phase 4 first**, then return here. Cleanest separation; each phase compiles standalone.
- **(B) Collapse Phases 2 and 4 into one commit.** Both are small and the dependency is tight.

The plan keeps them separate for clarity but either ordering is acceptable. If you take (A), confirm before continuing:

- [ ] **Step 2: Confirm Phase 4 has run (or you'll combine)**

Run: `grep -n 'deadline_ticks' src/batcave/linux/threads.rs | head -3`
Expected: at least one hit inside the `pub enum BlockReason` definition. If empty, run Phase 4 first.

### Task 2.2: Add `wake_epoll_waiters`

**Files:**
- Modify: `src/batcave/linux/threads.rs`

- [ ] **Step 1: Insert below `wake_expired_deadlines`**

```rust
/// Walk the threads table for any thread Blocked on EpollWait with the
/// matching epfd; transition each to Runnable. Bounded O(MAX_THREADS).
///
/// Called by epoll::mark_ready(epfd, ev) after flipping the ready bit,
/// so a parked epoll_pwait waiter wakes promptly on a real FD event
/// rather than waiting for the next deadline tick.
pub fn wake_epoll_waiters(epfd: i32) {
    with_table(|t| {
        for slot in t.iter_mut() {
            if let ThreadState::Blocked(BlockReason::EpollWait { epfd: e, .. }) = slot.state {
                if e == epfd {
                    slot.state = ThreadState::Runnable;
                }
            }
        }
    });
}
```

- [ ] **Step 2: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. Both new functions may produce `function never used` warnings — that's fine; Phase 3 (the tick hook) consumes wake_expired_deadlines, Phase 6 (epoll rewrite) consumes wake_epoll_waiters.

### Task 2.3: Commit Phase 2

- [ ] **Step 1: Commit**

Run:
```bash
git add src/batcave/linux/threads.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: wake_expired_deadlines + wake_epoll_waiters

Adds two wakers in linux::threads:
- wake_expired_deadlines() — walks the threads table for Blocked
  threads whose BlockReason carries an expired deadline_ticks,
  flips each to Runnable. Bounded O(MAX_THREADS=256) per call.
  Called from kernel::scheduler::tick() in Phase 3.
- wake_epoll_waiters(epfd) — walks the threads table for any
  thread Blocked on EpollWait with a matching epfd, flips to
  Runnable. Called by epoll::mark_ready in Phase 6.

Phase 2 of the scheduler block-on plan. Both functions reference
the post-rename BlockReason field names (deadline_ticks); Phase 4
(the rename) must run before this Phase 2 or be combined into a
single commit. Currently no callers; Phase 3+6 wire them up.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 3: Hook `wake_expired_deadlines` into `kernel::scheduler::tick()`

Single-line addition. After this commit, blocked threads with deadlines wake within one timer tick.

### Task 3.1: Modify scheduler::tick

**Files:**
- Modify: `src/kernel/scheduler.rs`

- [ ] **Step 1: Locate `pub fn tick`**

Run: `grep -n '^pub fn tick' src/kernel/scheduler.rs`
Expected: a single hit around line 14.

- [ ] **Step 2: Add the wake-pass call**

Use Edit to find:

```rust
pub fn tick() {
    // Drain up to DRAIN_CHUNK bytes from the BatCave stdio ring to the UART.
    // Decouples Chromium's verbose stderr from PL011 back-pressure.
    crate::batcave::linux::stdio_ring::drain_to_uart();
    schedule();
}
```

Replace with:

```rust
pub fn tick() {
    // Drain up to DRAIN_CHUNK bytes from the BatCave stdio ring to the UART.
    // Decouples verbose stderr from PL011 back-pressure.
    crate::batcave::linux::stdio_ring::drain_to_uart();
    // Wake any thread whose deadline has passed. Bounded O(MAX_THREADS)
    // per tick. See DESIGN_SCHEDULER_BLOCK_ON.md decision #4.
    crate::batcave::linux::threads::wake_expired_deadlines();
    schedule();
}
```

- [ ] **Step 3: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. The previously-unused `wake_expired_deadlines` warning should disappear.

### Task 3.2: Commit Phase 3

- [ ] **Step 1: Commit**

Run:
```bash
git add src/kernel/scheduler.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: hook wake_expired_deadlines into tick()

One-line change to kernel::scheduler::tick(): call
linux::threads::wake_expired_deadlines() between the existing
stdio_ring drain and schedule(). Now any thread parked on a
BlockReason carrying deadline_ticks wakes within one timer tick
even if no event-driven waker fires.

Phase 3 of the scheduler block-on plan. Behavioral effect minimal
until Phase 5+6 actually park threads with deadline_ticks; this
phase's only job is making the wake pass run on every tick.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 4: BlockReason field rename + diagnostic update

The current `BlockReason` defines `EpollWait { epfd, timeout_ms }` and `Nanosleep { deadline_ns }` but has no constructors today (see exploration). Only the diagnostic encoding at threads.rs:1289-1302 references the fields. Atomic rename + diagnostic update is small.

### Task 4.1: Rename the BlockReason fields

**Files:**
- Modify: `src/batcave/linux/threads.rs` (the `pub enum BlockReason` definition, ~line 296-309)

- [ ] **Step 1: Locate the BlockReason enum**

Run: `grep -n '^pub enum BlockReason' src/batcave/linux/threads.rs`
Expected: a single hit around line 296.

- [ ] **Step 2: Read the variants**

Run: `sed -n '296,310p' src/batcave/linux/threads.rs`

You should see something like:

```rust
pub enum BlockReason {
    FutexWait { uaddr: u64, val: u32 },
    EpollWait { epfd: i32, timeout_ms: i32 },
    Nanosleep { deadline_ns: u64 },
    Join { target_tid: u32 },
    IoWait,
}
```

- [ ] **Step 3: Apply the rename**

Use Edit to replace the relevant lines:

```rust
    EpollWait { epfd: i32, timeout_ms: i32 },
    Nanosleep { deadline_ns: u64 },
```

with:

```rust
    EpollWait { epfd: i32, deadline_ticks: u64 },  // 0 = infinite (epoll-only sentinel)
    Nanosleep { deadline_ticks: u64 },             // always concrete; 0 = invalid
```

### Task 4.2: Update the diagnostic encoding

**Files:**
- Modify: `src/batcave/linux/threads.rs` (~lines 1289-1302)

- [ ] **Step 1: Locate the diagnostic match**

Run: `grep -n 'BlockReason::EpollWait { epfd, timeout_ms }' src/batcave/linux/threads.rs`
Expected: a single hit around line 1291.

- [ ] **Step 2: Update the (a1, a2) match arm**

Use Edit to find:

```rust
                            BlockReason::FutexWait { uaddr, val } => (uaddr, val as u64),
                            BlockReason::EpollWait { epfd, timeout_ms } =>
                                (epfd as u64, timeout_ms as u64),
                            BlockReason::Nanosleep { deadline_ns } => (deadline_ns, 0),
                            BlockReason::Join { target_tid } => (target_tid as u64, 0),
                            BlockReason::IoWait => (0, 0),
```

Replace with:

```rust
                            BlockReason::FutexWait { uaddr, val } => (uaddr, val as u64),
                            BlockReason::EpollWait { epfd, deadline_ticks } =>
                                (deadline_ticks, epfd as i64 as u64),
                            BlockReason::Nanosleep { deadline_ticks } => (deadline_ticks, 0),
                            BlockReason::Join { target_tid } => (target_tid as u64, 0),
                            BlockReason::IoWait => (0, 0),
```

The `epfd as i64 as u64` is intentional: it sign-extends so a `-1` epfd in test cases encodes as `0xffff_ffff_ffff_ffff` rather than the surprise `0x0000_0000_ffff_ffff` of a plain `as u64`. `kind_disc` values 0..4 unchanged.

### Task 4.3: Verify build + commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. If errors mention `unknown field` for `timeout_ms` or `deadline_ns`, search for any remaining references and update them too:

```bash
grep -rn 'timeout_ms\|deadline_ns' src/ --include='*.rs'
```

- [ ] **Step 2: Confirm grep is clean for the old field names**

Run:
```bash
rg 'timeout_ms\b|deadline_ns\b' src/batcave/linux/{epoll,threads,syscall}.rs
```
Expected: empty.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/batcave/linux/threads.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: BlockReason field rename — deadline_ticks

Renames:
- BlockReason::EpollWait::timeout_ms → deadline_ticks (u64; 0 = infinite)
- BlockReason::Nanosleep::deadline_ns → deadline_ticks (u64; always concrete)

Both new fields hold absolute cntpct_el0 ticks — the canonical
clock-source contract from DESIGN_SCHEDULER_BLOCK_ON.md decision #2.
Diagnostic encoding at threads.rs:~1291 updated:

  EpollWait => (deadline_ticks, epfd as i64 as u64)
  Nanosleep => (deadline_ticks, 0)

The epfd cast preserves sign so test cases with epfd=-1 encode
unambiguously. No tuple-shape break; a1 semantics shift from
relative-timeout-ish to absolute deadline_ticks.

Phase 4 of the scheduler block-on plan. The variants currently have
no constructors; Phase 5 (sys_nanosleep) and Phase 6 (epoll_pwait)
are the first call sites.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 5: Rewrite `sys_nanosleep`

### Task 5.1: Locate `sys_nanosleep`

**Files:**
- Modify: `src/batcave/linux/syscall.rs`

- [ ] **Step 1: Find the function**

Run: `grep -n '^fn sys_nanosleep' src/batcave/linux/syscall.rs`
Expected: a single hit around line 1048.

- [ ] **Step 2: Read the current body**

Run: `sed -n '1048,1100p' src/batcave/linux/syscall.rs`

You'll see the existing tight-poll-with-yield-every-256-iterations loop ending in `0`.

### Task 5.2: Replace with `park_current` loop

**Files:**
- Modify: `src/batcave/linux/syscall.rs:~1080-1095` (the loop body, plus the deadline computation)

- [ ] **Step 1: Replace the loop body**

Use Edit to find the existing body (from `// NEW-DOS-010/014/016/019 fix:` comment through the closing `}` of the loop):

```rust
    // NEW-DOS-010/014/016/019 fix: yield to the scheduler instead of burning
    // CPU in a spin-loop. A cave that nanosleep()s for 30 s used to pin the
    // core; now co-scheduled caves get a slice via threads::schedule().
    // We still check the timer every ~100 iterations so wakeup latency stays
    // sub-ms on a lightly loaded system.
    let mut it = 0u32;
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target_ticks { break; }
        it = it.wrapping_add(1);
        if it % 256 == 0 {
            super::threads::schedule();
        } else {
            core::hint::spin_loop();
        }
    }
    0
}
```

Replace with:

```rust
    // Park-on-deadline: compute absolute cntpct_el0 deadline once, then
    // mark blocked + schedule until the timer-tick wake pass observes
    // our deadline has passed. Loop on the deadline check defends against
    // spurious wakes (force-wake-on-deadlock STUMP #63, future signal
    // delivery, etc.) — see DESIGN_SCHEDULER_BLOCK_ON.md decision.
    let deadline_ticks = start.saturating_add(target_ticks);
    while super::threads::cntpct_el0() < deadline_ticks {
        super::threads::park_current(
            super::threads::BlockReason::Nanosleep { deadline_ticks },
        );
    }
    0
}
```

The `start` and `target_ticks` variables already exist in the surrounding code (the old loop computed `start` from `cntpct_el0` and `target_ticks` from secs/nsecs). `saturating_add` defends against the (already-capped) input arithmetic from re-introducing overflow.

- [ ] **Step 2: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. The compiler should now see that `BlockReason::Nanosleep { deadline_ticks }` is constructed (Phase 4 added the field; this is the first constructor).

### Task 5.3: Commit Phase 5

- [ ] **Step 1: Commit**

Run:
```bash
git add src/batcave/linux/syscall.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: sys_nanosleep parks on deadline

Replaces the tight cntpct_el0 poll-with-yield-every-256-iterations
loop with park_current(BlockReason::Nanosleep { deadline_ticks }).
Loops on cntpct_el0() < deadline_ticks so spurious / forced /
IO-driven wakes (STUMP #63 force-wake-on-deadlock, future signal
delivery) re-park instead of returning early.

A 30s nanosleep no longer pins the core; the thread parks via
WFI between timer ticks and wakes when wake_expired_deadlines
observes the deadline has passed.

Phase 5 of the scheduler block-on plan. First constructor of
BlockReason::Nanosleep. See DESIGN_SCHEDULER_BLOCK_ON.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 6: Rewrite `epoll_pwait` + hook `mark_ready`

### Task 6.1: Rewrite `epoll_pwait`

**Files:**
- Modify: `src/batcave/linux/epoll.rs:450-509` (the function body)

- [ ] **Step 1: Locate `epoll_pwait`**

Run: `grep -n '^pub fn epoll_pwait' src/batcave/linux/epoll.rs`
Expected: a single hit around line 450.

- [ ] **Step 2: Replace the spin loop with park_current**

Use Edit to find the body from the `// Cooperative spin loop.` comment through the closing `}` of the loop:

```rust
    // Cooperative spin loop. Each iteration: scan interests for ready
    // bits, copy them out, return. If nothing is ready, yield and
    // maybe loop (depending on timeout).
    //
    // With no timer subsystem we approximate `timeout_ms` by counting
    // spin iterations — roughly 10µs each on the current runner. A
    // proper implementation will block on a waitqueue once the
    // scheduler grows timers.
    const SPIN_PER_MS: i32 = 100;
    let mut remaining: i64 = match timeout {
        0 => 0,
        t if t < 0 => i64::MAX, // indefinite
        t => (t as i64) * SPIN_PER_MS as i64,
    };

    loop {
        let n = drain_ready(slot, events, maxevents as usize);
        if n > 0 {
            return n as i64;
        }
        if remaining <= 0 {
            return 0; // timed out with no events
        }
        // yield/wait hint — lets other cooperative tasks run and is a
        // no-op on bare silicon if we're alone.
        cooperative_yield();
        remaining -= 1;
    }
}
```

Replace with:

```rust
    // Compute deadline. timeout==0 polls once and returns; timeout<0
    // is indefinite (deadline_ticks=0 sentinel); timeout>0 computes
    // absolute cntpct_el0 deadline once and parks until either an FD
    // event (mark_ready → wake_epoll_waiters) or the wake-expired-
    // deadlines tick pass moves us Runnable.
    let deadline_ticks: u64 = match timeout {
        0 => {
            return drain_ready(slot, events, maxevents as usize) as i64;
        }
        t if t < 0 => 0u64, // infinite
        t => crate::batcave::linux::threads::cntpct_el0()
            .saturating_add(crate::batcave::linux::threads::ms_to_ticks(t as u32)),
    };

    loop {
        let n = drain_ready(slot, events, maxevents as usize);
        if n > 0 {
            return n as i64;
        }
        // Deadline expired? deadline_ticks=0 means infinite — never times out.
        if deadline_ticks != 0 && crate::batcave::linux::threads::cntpct_el0() >= deadline_ticks {
            return 0;
        }
        crate::batcave::linux::threads::park_current(
            crate::batcave::linux::threads::BlockReason::EpollWait { epfd, deadline_ticks },
        );
        // park_current does not return while we're Blocked. On loop
        // re-entry, drain_ready re-checks readiness and the deadline
        // check above re-checks expiry.
    }
}
```

- [ ] **Step 3: Remove `cooperative_yield` (now orphaned)**

Run: `grep -n 'fn cooperative_yield\|cooperative_yield()' src/batcave/linux/epoll.rs`

Find both the function definition (around line 701) and any remaining call sites. The previous step removed the only call site; if grep shows only the definition, delete the definition (along with its preceding doc comment).

Use Edit to delete:

```rust
/// Cooperative yield. Calls into the BatCave thread scheduler so other
/// runnable threads in this cave can make progress while we wait for
/// an FD to become ready. Replaces the old asm-only `yield` hint which
/// only nudged the CPU but did NOT switch threads — that meant a
/// renderer thread spinning in epoll_pwait could starve the very
/// browser thread that was supposed to write the eventfd that would
/// wake it. Now each unsuccessful drain_ready hands the CPU back.
#[inline(always)]
fn cooperative_yield() {
    crate::batcave::linux::threads::schedule();
}
```

- [ ] **Step 4: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. `BlockReason::EpollWait { epfd, deadline_ticks }` is now constructed (first constructor).

### Task 6.2: Hook `mark_ready` to wake parked waiters

**Files:**
- Modify: `src/batcave/linux/epoll.rs` — the `mark_ready` function (search for its definition).

- [ ] **Step 1: Locate `mark_ready`**

Run: `grep -n '^pub fn mark_ready\|^fn mark_ready' src/batcave/linux/epoll.rs`
Expected: a single hit (the function that consumers like `tcp.rs::recv` will eventually call).

- [ ] **Step 2: Read the current body**

Run: `sed -n '<line>,<line+30>p' src/batcave/linux/epoll.rs` substituting the line number from Step 1.

The function flips a ready bit on the relevant interest entry. After the bit-flip, add the wake call.

- [ ] **Step 3: Add the wake call after the ready-bit flip**

Use Edit to insert at the END of `mark_ready`'s body (before the closing `}`):

```rust
    // Wake any thread parked on this epfd. Bounded O(MAX_THREADS).
    crate::batcave::linux::threads::wake_epoll_waiters(epfd);
```

If `mark_ready` takes `epfd: i32` directly, the call works as-is. If `mark_ready` takes a different identifier (`slot`, `instance_idx`, etc.), trace it back to the `epfd` value associated with that slot — there's likely a helper like `epfd_for_slot(slot)` already, or `mark_ready` receives the epfd as an additional parameter from its callers.

If neither path is clean, change `mark_ready`'s signature to accept `epfd: i32` explicitly. Update all callers (currently none in the surviving non-browser tree, but check via grep `mark_ready\(`).

- [ ] **Step 4: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

### Task 6.3: Commit Phase 6

- [ ] **Step 1: Confirm grep hygiene**

Run:
```bash
rg 'SPIN_PER_MS|cooperative_yield' src/batcave/linux/epoll.rs
```
Expected: empty.

- [ ] **Step 2: Commit**

Run:
```bash
git add src/batcave/linux/epoll.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: epoll_pwait parks; mark_ready wakes parked waiters

Replaces the SPIN_PER_MS heuristic + cooperative_yield loop with
park_current(BlockReason::EpollWait { epfd, deadline_ticks }).

  timeout == 0 → poll once, return.
  timeout < 0  → deadline_ticks = 0 (infinite sentinel), park until event.
  timeout > 0  → deadline_ticks = cntpct_el0() + ms_to_ticks(timeout).

mark_ready(epfd, ev) now calls threads::wake_epoll_waiters(epfd) after
flipping the ready bit, so a parked epoll_pwait waiter wakes promptly
on a real FD event rather than waiting for the next deadline tick.

Removes:
- SPIN_PER_MS constant
- fn cooperative_yield (no callers left)

Phase 6 of the scheduler block-on plan. First constructor of
BlockReason::EpollWait. See DESIGN_SCHEDULER_BLOCK_ON.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 7: Stale futex TODO cleanup

Comment-only changes. No behavior change.

### Task 7.1: Update top-of-file overview

**Files:**
- Modify: `src/batcave/linux/futex.rs:34-45` (the comment block describing block/wake integration).

- [ ] **Step 1: Read the current block**

Run: `sed -n '30,46p' src/batcave/linux/futex.rs`

- [ ] **Step 2: Replace with accurate description**

Use Edit to find:

```rust
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
```

Replace with:

```rust
//   FUTEX_WAIT publishes the waiter into the hash table, then enters a
//   block-and-resume loop in `park_slot`: it marks the current thread
//   Blocked (BlockReason::FutexWait), calls schedule() to yield, and
//   re-checks the woken flag + deadline on each resume. FUTEX_WAKE
//   transitions matching threads Runnable via wake_thread(tid).
//
//   Futex's deadline lives on its WaitSlot, not on BlockReason — the
//   wake_expired_deadlines tick pass (DESIGN_SCHEDULER_BLOCK_ON.md)
//   does not see it. Futex's resume-loop re-check handles its own
//   timeouts; unifying that into BlockReason is a future thread.
```

### Task 7.2: Update `park_slot` TODO

**Files:**
- Modify: `src/batcave/linux/futex.rs` — the comment block above `fn park_slot` (~line 270-276).

- [ ] **Step 1: Locate**

Run: `grep -n 'TODO(sched)' src/batcave/linux/futex.rs`
Expected: 1-2 hits.

- [ ] **Step 2: Replace**

Use Edit to find:

```rust
// ─── Park loop (the actual "block") ──────────────────────────────────────
//
// TODO(sched): replace this spin with a real scheduler.block_on(slot) once
// the Linux runner marks tasks Blocked and kicks them from futex_wake. The
// current implementation is correct (no lost wakeups, honours timeout) but
// wastes CPU.
fn park_slot(b: &Bucket, slot: usize, uaddr: u64, val: u32) -> i64 {
```

Replace with:

```rust
// ─── Park loop ───────────────────────────────────────────────────────────
//
// Block-and-resume: mark the current thread Blocked
// (BlockReason::FutexWait), call schedule() to yield, re-check the woken
// flag + deadline on each resume. FUTEX_WAKE flips matching slots'
// woken bits and wakes their threads via wake_thread(tid).
fn park_slot(b: &Bucket, slot: usize, uaddr: u64, val: u32) -> i64 {
```

- [ ] **Step 3: Verify no remaining TODO(sched)**

Run: `grep -n 'TODO(sched)' src/batcave/linux/futex.rs`
Expected: empty.

### Task 7.3: Commit Phase 7

- [ ] **Step 1: Commit**

Run:
```bash
git add src/batcave/linux/futex.rs
git commit -m "🎯 scheduler-block-on: clean up stale TODO(sched) comments in futex.rs

Two doc blocks claimed FUTEX_WAIT spins because 'the Linux runner
does not mark tasks as Blocked' and pointed at a future
scheduler::block_on(slot). Both untrue — futex.rs::park_slot
already calls mark_current_blocked + schedule() and FUTEX_WAKE
calls wake_thread(tid). Replaced with accurate descriptions of
the existing block-and-resume loop, plus a pointer to
DESIGN_SCHEDULER_BLOCK_ON.md noting that futex's per-WaitSlot
deadline is intentionally not unified with the new
wake_expired_deadlines pass.

Phase 7 of the scheduler block-on plan. Comment-only; no behavior
change.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

---

## Phase 8: Test helpers (`#[cfg(feature = "selftest-on-boot")]`)

Add three small helpers in `linux::threads` that operate only on Free slots so the test can synthesize Blocked states without touching real running threads.

### Task 8.1: Add the helpers

**Files:**
- Modify: `src/batcave/linux/threads.rs` (insert near the bottom of the public API section).

- [ ] **Step 1: Find a good insertion point**

Insert at the END of the file, after all production functions. The `#[cfg(feature = "selftest-on-boot")]` gate ensures these don't compile in production builds.

- [ ] **Step 2: Add the helper block**

```rust
// ─── Test helpers (feature-gated) ────────────────────────────────────────
//
// Operate only on Free slots so they never touch real running threads.
// Used by cmd_scheduler_selftest in src/ui/shell.rs and exercised in
// scripts/qemu_selftests_smoke.py. Not exposed in production builds.
//
// See DESIGN_SCHEDULER_BLOCK_ON.md "Test helpers" section.

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_install_blocked(reason: BlockReason) -> Option<usize> {
    // Find a Free slot and mark it Blocked with the given reason.
    // Returns its index. None if the table is full.
    //
    // Snapshot/free invariant: this function operates ONLY on Free
    // slots. It does NOT mutate any other slot's fields. test_release_slot
    // restores the same Free invariant.
    with_table(|t| {
        for (i, slot) in t.iter_mut().enumerate() {
            if slot.state == ThreadState::Free {
                slot.state = ThreadState::Blocked(reason);
                // Don't touch tid, regs, or wait metadata — keep them
                // at their Free defaults so test_release_slot can
                // simply re-zero state without coordinating with the
                // rest of the slot.
                return Some(i);
            }
        }
        None
    })
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_inspect_state(slot: usize) -> Option<ThreadState> {
    with_table(|t| {
        if slot >= t.len() { return None; }
        Some(t[slot].state)
    })
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_release_slot(slot: usize) {
    // Reset the slot to the Free invariant. Idempotent.
    with_table(|t| {
        if slot >= t.len() { return; }
        t[slot].state = ThreadState::Free;
        // Leave tid/regs/wait metadata as they were — they're Free's
        // "don't care" by convention. If the production Free invariant
        // requires more zeroing, mirror it here.
    });
}
```

NOTE: the spec says `test_release_slot` "restores the exact empty/free slot invariant." The implementer must verify what `Free` requires (currently looking only at `state == Free`); if other fields matter, mirror their Free defaults here.

- [ ] **Step 3: Verify both feature configurations build**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -5
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5
```
Both expected: `Finished ...`. The default build doesn't see the test helpers (cfg-gated); the selftest-on-boot build does and produces unused-function warnings until Phase 9 calls them.

### Task 8.2: Commit Phase 8

- [ ] **Step 1: Commit**

Run:
```bash
git add src/batcave/linux/threads.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: feature-gated test helpers in linux::threads

Adds three pub(crate) helpers gated by selftest-on-boot:
- test_install_blocked(reason) -> Option<slot> — finds a Free
  slot, marks it Blocked with the given reason, returns its index.
  Operates only on Free slots so real running threads are untouched.
- test_inspect_state(slot) -> Option<ThreadState> — read-only.
- test_release_slot(slot) — restore the Free invariant, idempotent.

Used by cmd_scheduler_selftest (Phase 9) to construct synthetic
Blocked states and verify the wake helpers without driving real
sys_nanosleep / epoll_pwait calls from the shell (which would have
no Linux-thread context).

Phase 8 of the scheduler block-on plan. No production-build impact;
helpers are #[cfg(feature = "selftest-on-boot")].

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 9: `cmd_scheduler_selftest` + dispatch

### Task 9.1: Add the function

**Files:**
- Modify: `src/ui/shell.rs` — insert near `cmd_x509_selftest` (around line 538) so selftests stay grouped.

- [ ] **Step 1: Find `cmd_x509_selftest`**

Run: `grep -n '^pub(crate) fn cmd_x509_selftest\|^fn cmd_x509_selftest' src/ui/shell.rs`
Expected: a single hit.

- [ ] **Step 2: Add `cmd_scheduler_selftest` immediately above it (or below — doesn't matter, just keep them grouped)**

Use Edit to insert this block:

```rust
/// Scheduler selftest. Operates on synthesized Free→Blocked test
/// slots — no real sys_nanosleep / epoll_pwait calls, no Linux-thread-
/// context dependency. Three sub-tests exercise the wake helpers'
/// correctness; the park_current loop invariant is verified by manual
/// review (see DESIGN_SCHEDULER_BLOCK_ON.md acceptance criteria).
///
/// `pub(crate)` so the boot-time runner in main.rs (gated by
/// selftest-on-boot) can call this for headless verification in
/// scripts/qemu_selftests_smoke.py.
#[cfg(feature = "selftest-on-boot")]
pub(crate) fn cmd_scheduler_selftest() {
    use crate::batcave::linux::threads::{
        cntpct_el0, wake_expired_deadlines, wake_epoll_waiters,
        test_install_blocked, test_inspect_state, test_release_slot,
        BlockReason, ThreadState,
    };

    console::puts_hi("  SCHEDULER SELFTEST\n");

    // Sub-test 1: wake-expired-deadlines is a noop when nothing is blocked
    // on a deadline. Just shouldn't panic or corrupt state.
    {
        wake_expired_deadlines();
        console::puts("  [scheduler-selftest] PASS: wake-expired-deadlines-noop\n");
    }

    // Sub-test 2: nanosleep deadline fires — install a Blocked slot with
    // already-past deadline, run the wake pass, observe Runnable, release.
    {
        let now = cntpct_el0();
        let past = now.saturating_sub(1);
        let slot = match test_install_blocked(BlockReason::Nanosleep { deadline_ticks: past }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: nanosleep-deadline-fires (table full)\n");
                return;
            }
        };
        wake_expired_deadlines();
        match test_inspect_state(slot) {
            Some(ThreadState::Runnable) => {
                console::puts("  [scheduler-selftest] PASS: nanosleep-deadline-fires\n");
            }
            Some(other) => {
                console::puts("  [scheduler-selftest] FAIL: nanosleep-deadline-fires (state ");
                let kind = match other {
                    ThreadState::Free => "Free",
                    ThreadState::Runnable => "Runnable",
                    ThreadState::Running => "Running",
                    ThreadState::Blocked(_) => "Blocked",
                    _ => "Unknown",
                };
                console::puts(kind);
                console::puts(")\n");
            }
            None => {
                console::puts("  [scheduler-selftest] FAIL: nanosleep-deadline-fires (slot vanished)\n");
            }
        }
        test_release_slot(slot);
    }

    // Sub-test 3: epoll event-driven wake. Install two slots with
    // different epfds; wake one; observe the other stays Blocked.
    {
        let s1 = match test_install_blocked(BlockReason::EpollWait { epfd: 123, deadline_ticks: 0 }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (table full A)\n");
                return;
            }
        };
        let s2 = match test_install_blocked(BlockReason::EpollWait { epfd: 456, deadline_ticks: 0 }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (table full B)\n");
                test_release_slot(s1);
                return;
            }
        };
        wake_epoll_waiters(123);
        let s1_state = test_inspect_state(s1);
        let s2_state = test_inspect_state(s2);
        let ok_first = matches!(s1_state, Some(ThreadState::Runnable));
        let ok_second_still_blocked = matches!(s2_state, Some(ThreadState::Blocked(_)));
        if ok_first && ok_second_still_blocked {
            wake_epoll_waiters(456);
            let s2_after = test_inspect_state(s2);
            if matches!(s2_after, Some(ThreadState::Runnable)) {
                console::puts("  [scheduler-selftest] PASS: epoll-event-wake\n");
            } else {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (epfd 456 wake didn't fire)\n");
            }
        } else {
            console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (selective wake broken)\n");
        }
        test_release_slot(s1);
        test_release_slot(s2);
    }
}
```

- [ ] **Step 3: Add the dispatch arm**

Run: `grep -n '"x509-selftest"' src/ui/shell.rs`
Expected: a single hit. Use Edit to add a new arm just below it:

```rust
        "x509-selftest" => cmd_x509_selftest(),
        "scheduler-selftest" => cmd_scheduler_selftest(),
```

NOTE: `cmd_scheduler_selftest` is `#[cfg(feature = "selftest-on-boot")]`. The dispatch arm needs to either (a) also be cfg-gated, or (b) we drop the cfg gate from the function and let it always exist.

The cleanest move: cfg-gate both. Use Edit to wrap the dispatch arm:

```rust
        "x509-selftest" => cmd_x509_selftest(),
        #[cfg(feature = "selftest-on-boot")]
        "scheduler-selftest" => cmd_scheduler_selftest(),
```

Match arms support `#[cfg(...)]` attributes in stable Rust.

ALTERNATIVELY: drop the `#[cfg]` from `cmd_scheduler_selftest` so it's always compiled. The test helpers (`test_install_blocked` etc.) ARE cfg-gated, so leaving cmd_scheduler_selftest non-gated would fail to compile in default builds. So either (a) cfg-gate both the function and dispatch arm, or (b) un-gate the test helpers and the function. Choice (a) is consistent with cmd_x509_selftest's pattern (which is non-gated only because it doesn't need test helpers; the whole function uses production code). Stick with (a).

- [ ] **Step 4: Verify both feature configurations build**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -5
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5
```
Both expected: `Finished ...`. Default build: no scheduler-selftest dispatch arm, no function. selftest-on-boot build: both present, function still unused (Phase 10 calls it).

### Task 9.2: Commit Phase 9

- [ ] **Step 1: Commit**

Run:
```bash
git add src/ui/shell.rs
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: cmd_scheduler_selftest

Adds a feature-gated scheduler selftest with three sub-tests:
- wake-expired-deadlines-noop — call wake_expired_deadlines with
  no Blocked threads; verify no panic, no state corruption.
- nanosleep-deadline-fires — install Blocked(Nanosleep{ already-past
  deadline_ticks }), call wake_expired_deadlines, assert Runnable.
- epoll-event-wake — install two Blocked(EpollWait) slots with
  different epfds; wake_epoll_waiters(123) wakes slot 1 only; then
  wake_epoll_waiters(456) wakes slot 2.

Each sub-test prints PASS or FAIL to UART. Wired via
'scheduler-selftest' shell dispatch (also cfg-gated).

Function gated by selftest-on-boot to keep test infrastructure out
of production builds.

Phase 9 of the scheduler block-on plan. The actual park_current
loop invariant is verified by manual review per the spec; this
selftest covers the wake-helper mechanics.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 10: Extend `selftest-on-boot` in `main.rs`

### Task 10.1: Add the call

**Files:**
- Modify: `src/main.rs` — the `#[cfg(feature = "selftest-on-boot")]` block added in PR #2.

- [ ] **Step 1: Locate the existing selftest-on-boot block**

Run: `grep -n 'selftest-on-boot\|cmd_x509_selftest' src/main.rs`
Expected: hits inside a `#[cfg(feature = "selftest-on-boot")]` block.

- [ ] **Step 2: Add the scheduler-selftest call**

Use Edit to find:

```rust
            #[cfg(feature = "selftest-on-boot")]
            {
                drivers::uart::puts("[selftest] running x509-selftest before auth gate...\n");
                ui::shell::cmd_x509_selftest();
            }
```

Replace with:

```rust
            #[cfg(feature = "selftest-on-boot")]
            {
                drivers::uart::puts("[selftest] running x509-selftest before auth gate...\n");
                ui::shell::cmd_x509_selftest();
                drivers::uart::puts("[selftest] running scheduler-selftest before auth gate...\n");
                ui::shell::cmd_scheduler_selftest();
            }
```

- [ ] **Step 3: Verify both feature configurations build**

Run:
```bash
cargo check --target aarch64-unknown-none --release 2>&1 | tail -5
cargo check --target aarch64-unknown-none --release --features selftest-on-boot 2>&1 | tail -5
```
Both expected: `Finished ...`.

### Task 10.2: Commit Phase 10

- [ ] **Step 1: Commit**

Run:
```bash
git add src/main.rs
git commit -m "🎯 scheduler-block-on: extend selftest-on-boot with cmd_scheduler_selftest

main.rs now runs cmd_x509_selftest then cmd_scheduler_selftest
under the selftest-on-boot feature, before the auth gate. Two
selftests doesn't justify a registry; if a third lands in a future
thread, that's the trigger for a real run_all_selftests() helper.

Phase 10 of the scheduler block-on plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

---

## Phase 11: Rename and extend `qemu_x509_smoke.py` → `qemu_selftests_smoke.py`

### Task 11.1: Rename and extend the script

**Files:**
- Rename: `scripts/qemu_x509_smoke.py` → `scripts/qemu_selftests_smoke.py`
- Modify: extend the post-rename file to verify scheduler subtests too.

- [ ] **Step 1: Rename**

Run: `git mv scripts/qemu_x509_smoke.py scripts/qemu_selftests_smoke.py`

- [ ] **Step 2: Update internal docstring**

Use Edit to replace the existing docstring:

```python
"""Headless x509-selftest smoke.
```

with:

```python
"""Headless smoke for all selftest-on-boot kernel selftests.
```

And update the rest of the docstring to mention both selftests. Replace any `x509-smoke` strings in print statements with `selftests-smoke`.

- [ ] **Step 3: Extend the regex scanning to cover both selftests**

Find the existing PASS/FAIL regex section (currently only matches `[x509-selftest]`):

```python
        pass_raw = re.findall(rb"\[x509-selftest\] PASS: (\S+)", log_bytes)
        fail_raw = re.findall(rb"\[x509-selftest\] FAIL: (\S+)", log_bytes)
        pass_subtests = sorted(set(s.decode("utf-8", "replace") for s in pass_raw))
        fail_subtests = sorted(set(s.decode("utf-8", "replace") for s in fail_raw))

        for s in pass_subtests:
            print(f"[x509-smoke]   PASS: {s}")
        for s in fail_subtests:
            print(f"[x509-smoke]   FAIL: {s}")

        if fail_subtests:
            print("[x509-smoke] FAIL — selftest reported failures.", file=sys.stderr)
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Expect at least 2 unique sub-tests passing.
        if len(pass_subtests) < 2:
            print(
                f"[x509-smoke] FAIL — expected 2 PASS sub-tests, got {len(pass_subtests)}",
                file=sys.stderr,
            )
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1
```

Replace with:

```python
        x509_pass_raw = re.findall(rb"\[x509-selftest\] PASS: (\S+)", log_bytes)
        x509_fail_raw = re.findall(rb"\[x509-selftest\] FAIL: (\S+)", log_bytes)
        sched_pass_raw = re.findall(rb"\[scheduler-selftest\] PASS: (\S+)", log_bytes)
        sched_fail_raw = re.findall(rb"\[scheduler-selftest\] FAIL: (\S+)", log_bytes)

        x509_pass = sorted(set(s.decode("utf-8", "replace") for s in x509_pass_raw))
        x509_fail = sorted(set(s.decode("utf-8", "replace") for s in x509_fail_raw))
        sched_pass = sorted(set(s.decode("utf-8", "replace") for s in sched_pass_raw))
        sched_fail = sorted(set(s.decode("utf-8", "replace") for s in sched_fail_raw))

        for s in x509_pass:
            print(f"[selftests-smoke]   x509 PASS: {s}")
        for s in x509_fail:
            print(f"[selftests-smoke]   x509 FAIL: {s}")
        for s in sched_pass:
            print(f"[selftests-smoke]   scheduler PASS: {s}")
        for s in sched_fail:
            print(f"[selftests-smoke]   scheduler FAIL: {s}")

        if x509_fail or sched_fail:
            print("[selftests-smoke] FAIL — one or more selftests reported failures.", file=sys.stderr)
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1

        if len(x509_pass) < 2:
            print(
                f"[selftests-smoke] FAIL — expected 2 x509 PASS sub-tests, got {len(x509_pass)}",
                file=sys.stderr,
            )
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if len(sched_pass) < 3:
            print(
                f"[selftests-smoke] FAIL — expected 3 scheduler PASS sub-tests, got {len(sched_pass)}",
                file=sys.stderr,
            )
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1
```

Update remaining print statements in the file from `[x509-smoke]` to `[selftests-smoke]` for consistency.

- [ ] **Step 4: Verify the script parses**

Run: `python3 -c "import ast; ast.parse(open('scripts/qemu_selftests_smoke.py').read())"`
Expected: no output (parse success).

- [ ] **Step 5: Run the smoke**

Run (with a 120s ceiling because it builds the kernel):
```bash
python3 scripts/qemu_selftests_smoke.py 2>&1 | tail -15 &
DRIVER_PID=$!
sleep 120
if kill -0 $DRIVER_PID 2>/dev/null; then kill $DRIVER_PID 2>/dev/null; fi
wait $DRIVER_PID 2>/dev/null
```
Expected: `[selftests-smoke] PASS — both sub-tests reported PASS, no FAIL lines.` (or similar; exact wording from the post-rename script).

If a sub-test FAILs, capture the log line, name the failing sub-test, and STOP. Most likely cause: a wake-helper bug introduced in Phase 2 or 6, or a test-helper bug from Phase 8.

### Task 11.2: Commit Phase 11

- [ ] **Step 1: Commit**

Run:
```bash
git add scripts/qemu_x509_smoke.py scripts/qemu_selftests_smoke.py
git commit -m "$(cat <<'EOF'
🎯 scheduler-block-on: rename qemu_x509_smoke.py → qemu_selftests_smoke.py

Renames the headless smoke harness to reflect that it now covers
both x509 and scheduler selftests. Extends the log-scan regex to
match both [x509-selftest] and [scheduler-selftest] PASS/FAIL lines.

Pass criterion: ≥2 unique x509 sub-tests + ≥3 unique scheduler
sub-tests pass, zero FAIL lines from either selftest, no kernel
panic markers.

Phase 11 of the scheduler block-on plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 12: Final acceptance + journal + push

### Task 12.1: Final static grep

- [ ] **Step 1: Run all the acceptance greps**

Run:
```bash
rg 'SPIN_PER_MS|timeout_ms\b|deadline_ns\b' src/batcave/linux/{epoll,threads,syscall}.rs
```
Expected: empty.

```bash
rg 'TODO\(sched\)' src/batcave/linux/futex.rs
```
Expected: empty.

```bash
rg 'cooperative_yield' src/batcave/linux/epoll.rs
```
Expected: empty.

If any of these return hits, name the surviving symbol and trace which phase missed it.

### Task 12.2: Final boot smoke (regression check)

- [ ] **Step 1: Run boot smoke**

Run:
```bash
cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
SMOKE_PID=$!
sleep 90
if kill -0 $SMOKE_PID 2>/dev/null; then kill $SMOKE_PID 2>/dev/null; fi
wait $SMOKE_PID 2>/dev/null
```
Expected: `[smoke] PASS — kernel boots, all required subsystems init, no deleted-browser symbols leaked through.`

### Task 12.3: Final selftests smoke (real proof of behavior)

- [ ] **Step 1: Run selftests smoke**

Run (already invoked in Phase 11 Step 5; this is the final acceptance run):
```bash
python3 scripts/qemu_selftests_smoke.py 2>&1 | tail -15 &
DRIVER_PID=$!
sleep 120
if kill -0 $DRIVER_PID 2>/dev/null; then kill $DRIVER_PID 2>/dev/null; fi
wait $DRIVER_PID 2>/dev/null
```
Expected: PASS, ≥2 x509 + ≥3 scheduler sub-tests, zero FAIL.

### Task 12.4: Manual review checkpoint — park_current loop invariant

- [ ] **Step 1: Re-read `park_current` body**

Open `src/batcave/linux/threads.rs`, find `pub fn park_current`, re-read carefully.

Confirm by inspection:
- Loop exit condition is `!current_thread_blocked()`.
- `wfi` is reached only when the previous `schedule()` returned with the thread still Blocked.
- No early `return` / `break` paths leave the function while `state == Blocked`.
- Threads-table lock is NOT held across `wfi`.
- `IrqGuard` (or any interrupt-masking primitive) is NOT held across `wfi`.

If any invariant is violated, add the fix as a Phase-12.4 commit before the journal entry lands.

### Task 12.5: Write SESSION_JOURNAL entry

**Files:**
- Modify: `docs/SESSION_JOURNAL.md` — prepend a new entry above the TLS-hardening one.

- [ ] **Step 1: Draft the entry**

Cover:
- Strategic context (third post-no-browser thread; futex re-framing).
- Each phase as a paragraph with commit hash.
- LOC stats from `git log --shortstat pre-scheduler-block-on-2026-05-07..HEAD`.
- State of tree: epoll + nanosleep park instead of spin; deadline-bearing waiters wake within one tick; futex untouched.
- Tag `pre-scheduler-block-on-2026-05-07` for revival.
- What's next per the priority list: captures cleanup.

Match existing journal voice (terse, technical, honest). ~150-200 lines.

- [ ] **Step 2: Commit**

Run:
```bash
git add docs/SESSION_JOURNAL.md
git commit -m "📝 SESSION_JOURNAL: scheduler block-on complete

Phase 12 of the scheduler block-on plan — handoff point. See
DESIGN_SCHEDULER_BLOCK_ON.md for strategy, PLAN_SCHEDULER_BLOCK_ON.md
for the per-phase how.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

### Task 12.6: Push

- [ ] **Step 1: Push the branch**

Run: `git push -u origin feat/scheduler-block-on 2>&1 | tail -5`
Expected: `* [new branch] feat/scheduler-block-on -> feat/scheduler-block-on` or fast-forward if already pushed.

- [ ] **Step 2: Confirm tag is on origin**

Run: `git push origin pre-scheduler-block-on-2026-05-07 2>&1 | tail -3`
Expected: tag confirmation or `up to date`.

### Task 12.7: Hand off to finishing-a-development-branch

- [ ] **Step 1: Invoke the skill**

Use the `superpowers:finishing-a-development-branch` skill. Recommended: **Push and create a Pull Request** against `main` (matches the no-browser and TLS-hardening threads — PRs #18-#24).

PR title: `scheduler-block-on: park-on-deadline for epoll + nanosleep`

PR body should reference `DESIGN_SCHEDULER_BLOCK_ON.md`, the rescue tag, summarize the deletions/additions, and list the verified tests.

---

## Done

After Phase 12:
- `feat/scheduler-block-on` has ~12 commits documenting each phase.
- Tag `pre-scheduler-block-on-2026-05-07` preserves the pre-deletion state.
- `cargo build --release --features gicv3` produces a working kernel binary.
- `cargo build --release --features gicv3,selftest-on-boot` also produces a working binary.
- `qemu_boot_smoke.py` PASSES (regression check).
- `qemu_selftests_smoke.py` PASSES with ≥2 x509 + ≥3 scheduler sub-tests; zero FAIL.
- Static grep returns empty for `SPIN_PER_MS`, `timeout_ms\b`, `deadline_ns\b`, `TODO\(sched\)`, `cooperative_yield`.
- Manual park_current loop invariant review checkpoint passed.
- `epoll_pwait` + `sys_nanosleep` park on deadline. Blocked threads with deadlines wake within one timer tick.
- Futex untouched.

🦇
