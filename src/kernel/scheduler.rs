// Bat_OS — Priority Preemptive Scheduler
// Lower priority number = higher priority.
// Timer tick causes reschedule — highest priority ready task runs.
// Security services always preempt non-critical tasks.

use crate::kernel::process::{self, TaskId, TaskState, CpuContext, MAX_TASKS};
use crate::drivers::uart;

unsafe extern "C" {
    fn switch_context(old: *mut CpuContext, new: *const CpuContext);
}

/// Pick the highest-priority ready task and switch to it.
pub fn schedule() {
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
