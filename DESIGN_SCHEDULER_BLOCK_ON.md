# DESIGN: Scheduler Block-on-Deadline — Epoll + Nanosleep

**Status:** Active proposal as of 2026-05-07.
**Follows:** `DESIGN_NO_BROWSER.md`, `DESIGN_TLS_HARDENING.md`. Third thread of the post-no-browser priority list.
**Touches:** `src/batcave/linux/epoll.rs`, `src/batcave/linux/syscall.rs` (sys_nanosleep), `src/batcave/linux/threads.rs`, `src/batcave/linux/futex.rs` (comment cleanup only), `src/kernel/scheduler.rs`, `src/main.rs`, `src/ui/shell.rs`, `scripts/qemu_x509_smoke.py` (renamed).
**Adds:** `cmd_scheduler_selftest` + dispatch arm in shell.rs.

## Goal

Replace the two remaining spin-wait paths (`epoll_pwait`, `sys_nanosleep`)
with proper park-on-deadline. Add a timer-tick wakeup pass in
`linux::threads` so a blocked thread with a deadline always wakes within
one tick, regardless of whether an event-driven wake fires. Clean up a
stale TODO in `futex.rs` that lies about already-done work.

## Why now

Per the post-no-browser roadmap, scheduler quality gates anything beyond
"trivial workloads in caves." `epoll_pwait` currently spins via a
`SPIN_PER_MS = 100` heuristic that approximates timeout by counting
iterations; `sys_nanosleep` reads `cntpct_el0` in a tight loop, yielding
to the scheduler every 256 iterations but never parking. Both burn CPU
during what should be sleep.

The earlier sweep's "futex/epoll spin replacement" framing was partially
wrong. **Futex `FUTEX_WAIT` already blocks** via
`mark_current_blocked(BlockReason::FutexWait)` + `threads::schedule()` +
resume-on-wake; the `TODO(sched): replace this spin with scheduler.
block_on()` comment in `futex.rs` is stale and pre-dates the block
implementation. The real gap is epoll + nanosleep, plus the missing
deadline-wake mechanism that lets timeouts fire even when no event-driven
wake arrives.

## Decisions locked in

1. **Auth model:** "Blocker marks blocked; waker marks runnable." After
   `threads::schedule()` resumes a thread, the resumed code does NOT
   call `mark_current_runnable()` defensively — if it's running again,
   some waker (event or tick pass) already set it Runnable. Defensive
   self-marking would paper over wakeup bugs.
2. **Clock-source contract:** All deadline math uses **`cntpct_el0`
   absolute ticks**. Not nanoseconds, not milliseconds, not
   timer-tick-counts. The wakeup pass compares a thread's
   `deadline_ticks` against `cntpct_el0()` directly, no conversion.
3. **Sentinel:** `deadline_ticks = 0` means "no deadline / infinite
   wait" (used for `epoll_pwait` with `timeout < 0`). Callers with
   zero-timeout (e.g. `epoll_pwait` with `timeout == 0`) **never park**;
   they poll once and return.
4. **Wakeup pass location:** New
   `linux::threads::wake_expired_deadlines()` walks the threads table
   for Blocked threads whose `BlockReason` carries an expired
   `deadline_ticks`, transitions them to `Runnable`. Called from
   `kernel::scheduler::tick()` between the existing
   `stdio_ring::drain_to_uart()` and `schedule()` calls. Bounded
   O(MAX_THREADS) per tick — `MAX_THREADS = 256` today, trivial cost.
5. **Futex untouched:** Futex's per-`WaitSlot` `deadline_ticks` lives in
   `futex.rs`, not in `BlockReason`, so the new wake pass does not see
   it. Futex's existing post-resume re-check loop continues to handle
   timeouts within the existing wake-on-`FUTEX_WAKE` path. Unifying
   futex deadlines into `BlockReason` is a separate future thread (see
   Out of Scope).
6. **`mark_ready` becomes a real waker:** After flipping the ready bit,
   `epoll::mark_ready(epfd, ev)` calls
   `threads::wake_epoll_waiters(epfd)` which transitions any
   `Blocked(BlockReason::EpollWait { epfd: that_epfd, .. })` to
   `Runnable`.
7. **`ms_to_ticks` precision:** Multiply-then-divide:
   `(ms as u64).saturating_mul(freq) / 1000`. Saturating mul prevents
   overflow panic on absurd inputs; integer divide at the end preserves
   sub-1000Hz precision.

8. **`park_current(reason)` is the primitive, not raw
   `mark_current_blocked + schedule()`.** Current `threads::schedule()`
   picks the next Runnable thread and returns. If the table contains
   no Runnable thread other than us, **`schedule()` returns
   immediately** — it does not park. A naive `mark_current_blocked +
   schedule()` sequence would then continue in the calling code while
   the thread is still in Blocked state, defeating the whole point.
   The fix: a new `park_current(reason)` primitive that holds the
   invariant **"does not return while the current thread's state is
   `Blocked`."** It marks self Blocked, then loops:

   ```rust
   pub fn park_current(reason: BlockReason) {
       mark_current_blocked(reason);
       loop {
           threads::schedule();
           // Resumed. If a waker (event-driven via wake_thread /
           // wake_epoll_waiters, or deadline-driven via
           // wake_expired_deadlines from the timer-tick pass) flipped
           // our state Blocked→Runnable, exit the loop.
           if !current_thread_blocked() { break; }
           // No runnable thread other than us, and we're still
           // Blocked. Wait for any interrupt — the timer IRQ runs
           // wake_expired_deadlines, an IO IRQ may run mark_ready →
           // wake_epoll_waiters, or another thread on a different CPU
           // could mutate our state directly.
           unsafe { core::arch::asm!("wfi"); }
       }
       // Loop exits with state == Runnable. Caller proceeds. The
       // self-running transition happens on the next schedule() call;
       // running-while-Runnable is observable but harmless.
   }
   ```

   `current_thread_blocked()` is a small helper that reads the current
   thread's state under the table lock. It looks up the slot for the
   current TID; if the current slot is missing (no current thread
   registered, or the TID isn't in the table), it returns `false`
   (i.e., "not blocked") so `park_current` exits the loop and falls
   through to the caller — non-blocking on missing slot. Both
   `park_current` and `current_thread_blocked` are
   `pub(crate)`-or-tighter; production callers go through the
   higher-level `epoll_pwait` / `sys_nanosleep` paths which call
   `park_current` internally.

   **Loop invariant** the implementer must preserve: the function
   does not return while the calling thread's state is
   `ThreadState::Blocked(_)`. Spurious wakes (forced wake on deadlock,
   future signal delivery, etc.) re-enter the loop body; a deadline
   that hasn't fired and an event that hasn't arrived both leave the
   thread Blocked, so the loop body re-parks via WFI.

   **Lock + IRQ ordering invariants** the implementer must preserve:

   - `park_current` MUST NOT hold the threads-table lock across
     `wfi`. Holding it would deadlock: the timer-IRQ wakeup pass
     (`wake_expired_deadlines`) and the event-driven wakers
     (`wake_thread`, `wake_epoll_waiters`) all need the same lock to
     transition state. Drop the lock before `wfi`; reacquire on the
     next loop iteration via `current_thread_blocked()`.
   - `park_current` MUST NOT hold an `IrqGuard` (or otherwise mask
     interrupts) across `wfi`. Interrupts must be enabled when
     `wfi` executes; otherwise the timer IRQ can't fire and
     deadline-bearing sleepers never wake. The
     `mark_current_blocked` step may briefly take an `IrqGuard` to
     keep the state-mutation atomic against a same-CPU IRQ, but
     that guard drops before `schedule()` returns and certainly
     before `wfi`.
   - `mark_current_blocked` itself takes the table lock to mutate
     state, then releases it. `schedule()` takes its own lock
     internally. The path between them — and the `wfi` after — runs
     lock-free.

## What gets changed

### `epoll_pwait` rewrite (`src/batcave/linux/epoll.rs`)

Current behavior:

```text
timeout < 0:  remaining = i64::MAX, loop drain_ready / cooperative_yield / decrement
timeout > 0:  remaining = timeout * SPIN_PER_MS, same loop
timeout == 0: poll once
```

New behavior:

```text
timeout == 0: drain_ready once, return  (no park)
timeout < 0:  deadline_ticks = 0   (sentinel: infinite)
timeout > 0:  deadline_ticks = cntpct_el0() + ms_to_ticks(timeout)

common loop:
  n = drain_ready(); if n > 0 return n
  if deadline_ticks != 0 && cntpct_el0() >= deadline_ticks: return 0
  park_current(BlockReason::EpollWait { epfd, deadline_ticks })
  // park_current does not return while we're Blocked.
  // Loop iteration re-checks ready bits and deadline.
```

`SPIN_PER_MS` constant removed. `cooperative_yield()` removed (no caller
left). `remaining` countdown removed.

### `sys_nanosleep` rewrite (`src/batcave/linux/syscall.rs`)

Current behavior: tight `cntpct_el0` poll loop with `schedule()` every
256 iterations.

New behavior:

```text
deadline_ticks = cntpct_el0() + (secs_capped*freq + nsecs_capped*freq/1_000_000_000)
if deadline_ticks <= cntpct_el0(): return 0   (already past — never park)

while cntpct_el0() < deadline_ticks:
  park_current(BlockReason::Nanosleep { deadline_ticks })
  // Spurious / forced / IO-driven wakes can return park_current early
  // even though the deadline hasn't fired. Re-check and re-park.
return 0
```

Looping on the deadline check (not assuming a single park is enough)
defends against forced wake-on-deadlock paths (STUMP #63), future
signal delivery, or any other waker that doesn't honor "deadline-only"
semantics. The existing `secs_capped`/`nsecs_capped` overflow guards
(V8-ROOT-3) stay — they cap input range, not deadline arithmetic.

### `BlockReason` field changes (`src/batcave/linux/threads.rs`)

```rust
// Before
EpollWait { epfd: i32, timeout_ms: i32 },
Nanosleep { deadline_ns: u64 },

// After
EpollWait { epfd: i32, deadline_ticks: u64 },  // 0 = infinite (epoll only)
Nanosleep { deadline_ticks: u64 },             // always concrete; 0 = invalid
```

### Diagnostic encoding update (`src/batcave/linux/threads.rs:1289-1302`)

The `(a1, a2)` + `kind_disc` pack used by the SKIP-DEADLOCK forced-wake
log gets:

```rust
BlockReason::EpollWait { epfd, deadline_ticks } => (deadline_ticks, epfd as i64 as u64),
BlockReason::Nanosleep { deadline_ticks }       => (deadline_ticks, 0),
```

`epfd as i64 as u64` preserves sign — `-1` (invalid epfd in test cases)
encodes as `0xffff_ffff_ffff_ffff` rather than the surprise `0x0000_0000_ffff_ffff`
of a plain `as u64`. `kind_disc` values 0..4 unchanged.

**No tuple-shape break; `a1` semantics become absolute `deadline_ticks`.**
Tools that already parse the `(kind, a1, a2)` triple keep working;
their interpretation of `a1` shifts from relative-timeout-ish to
absolute-cntpct-tick.

### New: `linux::threads::wake_expired_deadlines()`

```rust
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

Bounded O(MAX_THREADS) per call. No allocation, no I/O. Safe to invoke
from a timer IRQ handler.

### New: `linux::threads::wake_epoll_waiters(epfd)`

```rust
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

Called by `epoll::mark_ready(epfd, ev)` after flipping the ready bit.

### New helpers (`src/batcave/linux/threads.rs` or reuse existing)

```rust
#[inline]
pub fn cntpct_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

#[inline]
pub fn ms_to_ticks(ms: u32) -> u64 {
    let freq = cntfrq_el0();
    (ms as u64).saturating_mul(freq) / 1000
}

#[inline]
fn cntfrq_el0() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v); }
    v
}
```

If equivalents already exist in `linux::threads` or elsewhere, reuse
them; the implementation phase greps before adding.

### `kernel::scheduler::tick()` change

```rust
pub fn tick() {
    crate::batcave::linux::stdio_ring::drain_to_uart();
    crate::batcave::linux::threads::wake_expired_deadlines();  // NEW
    schedule();
}
```

**One line added.** `schedule()` was already there; not new behavior,
just preserved.

## What gets cleaned up

1. **`futex.rs:44-45`** — top-of-file overview's stale "When the
   scheduler gains a real Blocked state, replace the spin body in
   `park_slot` with `scheduler::block_on(slot)` and have `futex_wake`
   call `scheduler::unblock(tid)`." Rewrite to describe the existing
   block-and-resume loop accurately.
2. **`futex.rs:273-276`** — same stale TODO above `park_slot`. Replace
   with a comment describing what the function actually does.
3. **`scripts/qemu_x509_smoke.py` → `scripts/qemu_selftests_smoke.py`** —
   rename to reflect that it now covers x509 + scheduler selftests.

## Test helpers (`#[cfg(feature = "selftest-on-boot")]`)

Three small helpers in `linux::threads`, gated to keep the test
interface out of production builds:

```rust
#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_install_blocked(reason: BlockReason) -> Option<usize> {
    // Find a Free slot, mark it Blocked with the given reason, return
    // its index. None if the table is full.
    //
    // Snapshot/free invariant: this function operates ONLY on Free
    // slots. It does NOT mutate any other slot's fields. test_release_slot
    // restores the same Free invariant.
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_inspect_state(slot: usize) -> Option<ThreadState> {
    // Read back the ThreadState at slot. None if slot out of range.
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn test_release_slot(slot: usize) {
    // Reset the slot to the exact Free invariant it had before
    // test_install_blocked: state = Free, all other fields cleared
    // (tid, regs, wait metadata). Idempotent.
}
```

Operating only on Free slots means the tests never touch a real running
thread's metadata. `test_release_slot` restores the exact empty/free slot
invariant.

## `cmd_scheduler_selftest` (`src/ui/shell.rs`)

Three sub-tests using the helpers above:

1. **`wake-expired-deadlines-noop`** — call `wake_expired_deadlines()`
   when no thread is in any deadline-bearing Blocked state. Verify no
   panic and no observable state change. Catches table-walk bugs.
2. **`nanosleep-deadline-fires`** — `test_install_blocked(Nanosleep
   { deadline_ticks: now - 1 })` (already past). Call
   `wake_expired_deadlines()`. Assert
   `test_inspect_state(slot) == Some(Runnable)`. Release the slot.
3. **`epoll-event-wake`** — install two slots:
   `EpollWait { epfd: 123, deadline_ticks: 0 }` and
   `EpollWait { epfd: 456, deadline_ticks: 0 }`. Call
   `wake_epoll_waiters(123)`. Assert slot 1 is `Runnable`, slot 2 is
   still `Blocked`. Call `wake_epoll_waiters(456)`. Assert slot 2 also
   `Runnable`. Release both slots.

Each sub-test prints `[scheduler-selftest] PASS: <case>` or `FAIL: <case>`
to UART. Wired via `"scheduler-selftest" => cmd_scheduler_selftest()`
shell dispatch. `pub(crate)` like `cmd_x509_selftest` so the boot-time
runner can call it.

### `selftest-on-boot` extension (`src/main.rs`)

```rust
#[cfg(feature = "selftest-on-boot")]
{
    drivers::uart::puts("[selftest] running x509-selftest before auth gate...\n");
    ui::shell::cmd_x509_selftest();
    drivers::uart::puts("[selftest] running scheduler-selftest before auth gate...\n");
    ui::shell::cmd_scheduler_selftest();
}
```

The earlier spec promised "When more selftests want headless validation,
this gets generalized into a real harness." Today's threshold: two
selftests. Two doesn't justify a registry. If a third lands in a future
thread, that's the trigger for `pub fn run_all_selftests()`.

### `qemu_selftests_smoke.py` (renamed from `qemu_x509_smoke.py`)

Same harness pattern as before. Builds with `--features
gicv3,selftest-on-boot`, boots, waits for `[security] Launching auth
gate` banner, scans the captured log for both selftests' PASS/FAIL
lines. Pass criterion:

- ≥2 unique x509 sub-tests pass (`hostname-mismatch`, `bad-bytes`)
- ≥3 unique scheduler sub-tests pass (`wake-expired-deadlines-noop`,
  `nanosleep-deadline-fires`, `epoll-event-wake`)
- Zero FAIL lines from either selftest
- No kernel panic markers

## Approximate diff

- ~5 modified files, ~40-60 lines deleted (spin loop bodies, stale
  comments, old field names), ~120-150 lines added (new wake pass,
  test helpers, `cmd_scheduler_selftest`, `qemu_selftests_smoke.py`
  scheduler section).
- 0 files deleted, 1 file renamed (`qemu_x509_smoke.py` →
  `qemu_selftests_smoke.py`).

## Default behavior after this PR

- `epoll_pwait` callers with non-zero timeout park instead of spin;
  CPU stays cool during waits.
- `nanosleep` callers park; a 30s sleep no longer keeps the core warm.
- Any thread blocked on an expired deadline wakes within one timer
  tick. Timer tick rate is set elsewhere; spec doesn't change it.
- Futex behavior unchanged (still uses its own park-and-resume loop;
  any future "unify futex deadline" work is a separate thread).
- `selftest-on-boot` builds run two selftests now (x509 + scheduler).
- Default builds (no feature) are unchanged behaviorally except for
  the spin→park rewrite of epoll/nanosleep.

## Testing & verification

**Layer 1 — build verification:**
```bash
cargo check --target aarch64-unknown-none --release
cargo build --release --target aarch64-unknown-none --features gicv3
cargo check --target aarch64-unknown-none --release --features selftest-on-boot
```
All three pass. Warnings should not rise from the post-TLS-hardening
baseline (~216 in release).

**Layer 2 — boot smoke (regression check only):**
`scripts/qemu_boot_smoke.py` PASSES. The boot path may or may not
exercise nanosleep depending on driver init timing, so this isn't proof
of the new wait behavior. Its job is "the kernel still boots after the
rename + tick-pass-hook + epoll/nanosleep rewrites."

**Layer 3 — `cmd_scheduler_selftest`** (described above). Manual run via
`scheduler-selftest` shell command, or headless via Layer 5.

**Layer 4 — `selftest-on-boot` extension** (described above).

**Layer 5 — `qemu_selftests_smoke.py`** (described above). Real proof
that the new wait behavior works — exercises the wake pass, the epoll
event-wake path, and the table-walk noop guard.

**Acceptance criteria:**

- ✅ `cargo build --release --features gicv3` clean
- ✅ `cargo build --release --features gicv3,selftest-on-boot` clean
- ✅ `qemu_boot_smoke.py` PASSES
- ✅ `qemu_selftests_smoke.py` PASSES (≥2 x509 + ≥3 scheduler sub-tests
  pass, zero FAIL)
- ✅ Static grep returns empty:
  ```bash
  rg 'SPIN_PER_MS|timeout_ms\b|deadline_ns\b' src/batcave/linux/{epoll,threads,syscall}.rs
  rg 'TODO\(sched\)' src/batcave/linux/futex.rs
  ```
- ✅ **Park-loop invariant review checkpoint** (manual, not
  automated): a reviewer reads `park_current` and confirms by
  inspection that the function cannot return while the calling
  thread's state is `ThreadState::Blocked(_)`. Specifically:
  - The loop's exit condition is `!current_thread_blocked()`.
  - Inside the loop, `wfi` is reached only when the thread is still
    Blocked.
  - No early `return` / `break` paths exist that would leave the
    function while `state == Blocked`.

  This is the structural counterpart to the runtime-deterministic
  selftests above. The selftests prove the wake helpers; the review
  proves park doesn't leak.

## Out of scope

- **Unifying futex deadlines into `BlockReason`** — futex keeps its
  per-`WaitSlot` deadline. A future thread can fold that into the new
  wake pass for full uniformity.
- **Pipe / socket / signalfd / timerfd waitqueues** — none of these
  currently spin in surviving non-browser paths; if a real caller
  surfaces a spin, that's its own thread.
- **Kernel-task blocking** — Sphragis's kernel-side tasks
  (`kernel::process`) don't currently block on user events. The new
  wake pass operates on `linux::threads`, not `kernel::process`. If
  kernel tasks ever need to block on deadlines, that's a refactor.
- **Generalized selftest registry** — keeping `selftest-on-boot` as an
  inline two-call sequence in `main.rs`. A real `run_all_selftests()`
  registry waits for a third selftest.
- **Changing the timer tick rate** — orthogonal to this thread.
  Whatever rate the timer fires at is the wakeup-latency upper bound.

## Reversibility

Tag `pre-scheduler-block-on-2026-05-07` will be applied to the current
branch HEAD before any deletion lands, matching the no-browser /
TLS-hardening pivot conventions. The diff is small enough that
single-PR revert is mechanical if anything goes sideways post-merge.

## Implementation plan

A separate plan doc (drafted via `superpowers:writing-plans` after
this design is approved) handles the actual phasing — which files in
what order, with `cargo check` between phases. This design doc is the
*why*; the plan is the *how*.

🦇
