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

    // Find highest priority (lowest number) ready task
    let mut best_id: Option<TaskId> = None;
    let mut best_priority: u8 = 255;

    for i in 0..MAX_TASKS {
        let task = process::get(TaskId(i as u16));
        if task.state == TaskState::Ready && task.priority < best_priority {
            best_priority = task.priority;
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
    next.state = TaskState::Running;
    process::set_current(next_id);

    // Switch
    let old_ctx = &mut process::get(current_id).context as *mut CpuContext;
    let new_ctx = &process::get(next_id).context as *const CpuContext;

    unsafe {
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
