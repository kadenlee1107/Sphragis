// Bat_OS — Priority Preemptive Scheduler
// Lower priority number = higher priority.
// Timer tick causes reschedule — highest priority ready task runs.
// Security services always preempt non-critical tasks.

use crate::kernel::process::{self, TaskId, TaskState, CpuContext, MAX_TASKS};
use crate::drivers::uart;

unsafe extern "C" {
    fn switch_context(old: *mut CpuContext, new: *const CpuContext);
}

/// cntpct timestamp from the last context-switch. Used to attribute
/// elapsed ticks to the just-descheduled task's cave (gap-audit
/// item 030 CPU slice, observability only — no enforcement yet).
static LAST_SWITCH_TICK: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

#[inline]
fn cntpct_now() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) v); }
    v
}

/// Pick the highest-priority ready task and switch to it.
pub fn schedule() {
    // Charge the active cave for the time it just spent on-CPU,
    // before we pick a new task.
    let now = cntpct_now();
    let prev = LAST_SWITCH_TICK.swap(now, core::sync::atomic::Ordering::Relaxed);
    if prev > 0 && now > prev {
        crate::batcave::cave::active_add_cpu_ticks(now - prev);
    }

    let current_id = process::current_id();
    let current = process::get(current_id);

    // If current task is still running, mark it ready
    if current.state == TaskState::Running {
        current.state = TaskState::Ready;
    }

    // Find highest priority (lowest number) ready task. `best`
    // starts as 256 (u16) so even priority 255 (kernel idle) is
    // strictly less than the initial bar — without that widening,
    // task 0 was unselectable, and a self-terminating helper task
    // could leave the runqueue with NO eligible Ready task, which
    // caused schedule() to return without switching and trap the
    // helper in its own current_terminate yield loop.
    let mut best_id: Option<TaskId> = None;
    let mut best_priority: u16 = 256;

    for i in 0..MAX_TASKS {
        let task = process::get(TaskId(i as u16));
        if task.state == TaskState::Ready && (task.priority as u16) < best_priority {
            best_priority = task.priority as u16;
            best_id = Some(task.id);
        }
    }

    // If no ready task, keep running current (or idle)
    let next_id = match best_id {
        Some(id) => id,
        None => {
            // Re-mark current as running
            if current.state == TaskState::Ready {
                current.state = TaskState::Running;
            }
            return;
        }
    };

    // If same task, just mark running and return
    if next_id.index() == current_id.index() {
        let task = process::get(next_id);
        task.state = TaskState::Running;
        return;
    }

    // Context switch
    let next = process::get(next_id);
    let next_cave_id = next.cave_id;
    next.state = TaskState::Running;
    process::set_current(next_id);

    // sys-caves Arc 1 — when this swap crosses a cave boundary,
    // also swap the TTBR0_EL1 user window so the new task sees the
    // L1 of its owning cave. Tasks tagged cave_id == 0 (kernel
    // namespace) stay on PRIMARY_L1; tasks tagged with a real
    // cave id that has a built L1 get switched. If the target
    // cave has no L1 built yet (cave_l1_phys == 0), we leave
    // TTBR0_EL1 alone — same effective behavior as today, no
    // regression risk.
    //
    // SAFETY DISCIPLINE: TLB invalidate + DSB + ISB are inside
    // `mmu::switch_to_cave`. We call it from the kernel context
    // here, BEFORE the userspace-level `switch_context` jumps to
    // the new task's PC — so by the time the new task's code
    // runs, the user-window mappings are already in effect.
    let cur_cave_id = process::get(current_id).cave_id;
    if next_cave_id != cur_cave_id {
        if next_cave_id == 0 {
            // Transitioning back to the kernel namespace — restore
            // PRIMARY_L1 so TTBR0 cleanly reflects "no cave-scoped
            // user window active." Without this, the previous
            // cave's L1 would linger in TTBR0 across transitions
            // to kernel-ns tasks. Harmless in Phase 2 (kernel-only
            // tasks don't access user VAs) but cleaner semantics,
            // and matters the moment we add real EL0 tasks.
            crate::batcave::linux::mmu::switch_to_primary();
        } else if let Some(target_l1) = crate::batcave::cave::get_cave_l1_phys(next_cave_id) {
            crate::batcave::linux::mmu::switch_to_cave(target_l1);
        }
        // No `else` for the "target cave but no L1 built yet" case
        // — leaving TTBR0 alone is correct then: the task wasn't
        // going to access cave-scoped user VAs anyway.
    }

    // Switch
    let old_ctx = &mut process::get(current_id).context as *mut CpuContext;
    let new_ctx = &process::get(next_id).context as *const CpuContext;

    // Cross-task Spectre barrier — emit `sb` (FEAT_SB, ARMv8.5;
    // NOP on older cores) before swapping CPU context so transient
    // execution started under the old task's PSTATE/PC can't
    // retire against the new task's register file. Paired with
    // the TTBR0-barrier in `mmu::switch_to_cave` (also fired above
    // when caves differ). Cheap on cores without FEAT_SB and a
    // real mitigation on cores that have it.
    unsafe {
        core::arch::asm!(".inst 0xd50330ff");
        switch_context(old_ctx, new_ctx);
    }
}

/// Voluntarily yield the current time slice.
pub fn yield_now() {
    schedule();
}

pub fn init() {
    uart::puts("  [sched] Scheduler initialized (priority preemptive)\n");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    /// `block_on` exceeded the caller-supplied deadline.
    Timeout,
}

/// Block until `f` returns `Some(T)` or the deadline expires.
/// Yields between polls so other ready tasks get a turn.
///
/// This is the synchronous bridge into pollable subsystems —
/// pipes, sockets, the TCP recv path, and (eventually) any future
/// async-shaped surface. Lets a caller write
/// `block_on(|| pipe::try_read(id, buf), 5_000_000)` instead of
/// hand-rolling the same loop everywhere.
///
/// Pass `0` for `timeout_us` to wait forever — the loop ignores
/// the deadline check. Useful for shell prompts where we genuinely
/// have no upper bound.
pub fn block_on<F, T>(mut f: F, timeout_us: u64) -> Result<T, BlockError>
where F: FnMut() -> Option<T>
{
    let start = crate::kernel::time::monotonic_us();
    loop {
        if let Some(v) = f() {
            return Ok(v);
        }
        if timeout_us != 0 {
            let now = crate::kernel::time::monotonic_us();
            if now.saturating_sub(start) >= timeout_us {
                return Err(BlockError::Timeout);
            }
        }
        yield_now();
    }
}

/// In-process selftest of the block_on primitive. Validates both
/// the success path (closure flips after a few yields → Ok) and
/// the timeout path (closure never flips → Err(Timeout) within
/// the expected wall-clock window). Used by `block-on-selftest`.
pub fn block_on_selftest() -> Result<(bool, bool, u64), &'static str> {
    use core::cell::Cell;

    // ── Success path: closure becomes ready after 3 polls. ─────
    let counter = Cell::new(0usize);
    let success_ok = match block_on(
        || {
            counter.set(counter.get() + 1);
            if counter.get() >= 3 { Some(counter.get()) } else { None }
        },
        1_000_000, // 1 second timeout
    ) {
        Ok(v) if v == 3 => true,
        _ => false,
    };

    // ── Timeout path: closure never returns Some; verify we
    //    bail with Err(Timeout) and that the wall-clock elapsed
    //    is at least the timeout we asked for. 10 ms is enough
    //    to be observable without slowing the boot path.
    let t0 = crate::kernel::time::monotonic_us();
    let timeout_us = 10_000;
    let timeout_ok = matches!(
        block_on::<_, ()>(|| None, timeout_us),
        Err(BlockError::Timeout)
    );
    let t1 = crate::kernel::time::monotonic_us();
    let elapsed = t1.saturating_sub(t0);
    let elapsed_meets_deadline = elapsed >= timeout_us;

    Ok((success_ok, timeout_ok && elapsed_meets_deadline, elapsed))
}
