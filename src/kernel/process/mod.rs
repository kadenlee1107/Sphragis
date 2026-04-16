#![allow(dead_code)]
// Bat_OS — Process/Task Abstraction
// Each task represents an isolated execution context.
// In the microkernel, everything outside the kernel is a task:
// drivers, services, and applications.

use crate::kernel::mm::{frame, page_table::AddressSpace};
use crate::kernel::capability::CapabilitySet;

pub const MAX_TASKS: usize = 64;
const TASK_STACK_PAGES: usize = 4; // 16KB stack per task

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Free,
    Ready,
    Running,
    Blocked,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TaskId(pub u16);

impl TaskId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// Saved CPU context for context switching.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CpuContext {
    // Callee-saved registers x19-x30 + sp
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64, // frame pointer
    pub x30: u64, // link register (return address)
    pub sp: u64,
    pub elr: u64,  // exception link register (PC to return to)
    pub spsr: u64, // saved processor state
}

impl CpuContext {
    pub const fn zero() -> Self {
        Self {
            x19: 0, x20: 0, x21: 0, x22: 0, x23: 0, x24: 0,
            x25: 0, x26: 0, x27: 0, x28: 0, x29: 0, x30: 0,
            sp: 0, elr: 0, spsr: 0,
        }
    }
}

pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub priority: u8, // 0 = highest priority
    pub context: CpuContext,
    pub stack_base: usize,    // physical base of stack
    pub address_space: Option<AddressSpace>,
    pub capabilities: CapabilitySet,
    pub name: [u8; 32],
    pub name_len: usize,
}

impl Task {
    pub const fn empty() -> Self {
        Self {
            id: TaskId(0),
            state: TaskState::Free,
            priority: 255,
            context: CpuContext::zero(),
            stack_base: 0,
            address_space: None,
            capabilities: CapabilitySet::empty(),
            name: [0u8; 32],
            name_len: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }
}

/// Global task table.
static mut TASKS: [Task; MAX_TASKS] = {
    const EMPTY: Task = Task::empty();
    [EMPTY; MAX_TASKS]
};

static mut CURRENT_TASK: usize = 0;

pub fn init() {
    // Task 0 is the kernel idle task
    unsafe {
        TASKS[0].id = TaskId(0);
        TASKS[0].state = TaskState::Running;
        TASKS[0].priority = 255; // lowest priority
        set_task_name(&mut TASKS[0], "kernel");
    }
}

pub fn current_id() -> TaskId {
    unsafe { TaskId(CURRENT_TASK as u16) }
}

pub fn current() -> &'static mut Task {
    unsafe { &mut TASKS[CURRENT_TASK] }
}

pub fn get(id: TaskId) -> &'static mut Task {
    unsafe { &mut TASKS[id.index()] }
}

pub fn set_current(id: TaskId) {
    unsafe { CURRENT_TASK = id.index(); }
}

/// Create a new kernel task (runs in EL1).
/// For Phase 2, all tasks run in kernel mode.
/// Phase 3+ will add EL0 userspace tasks.
pub fn create_kernel_task(
    name: &str,
    entry: fn() -> !,
    priority: u8,
) -> Option<TaskId> {
    unsafe {
        // Find a free slot
        let tasks_ptr = core::ptr::addr_of_mut!(TASKS);
        let mut slot = None;
        for i in 0..MAX_TASKS {
            if (*tasks_ptr)[i].state == TaskState::Free {
                slot = Some(i);
                break;
            }
        }
        let slot = slot?;

        // Allocate stack
        let mut stack_top = 0usize;
        let mut stack_base = 0usize;
        for i in 0..TASK_STACK_PAGES {
            let page = frame::alloc_frame()?;
            if i == 0 {
                stack_base = page;
            }
            stack_top = page + frame::PAGE_SIZE;
        }

        let task = &mut TASKS[slot];
        task.id = TaskId(slot as u16);
        task.state = TaskState::Ready;
        task.priority = priority;
        task.stack_base = stack_base;
        task.capabilities = CapabilitySet::empty();

        // Set up context so the scheduler can "resume" into this task
        task.context = CpuContext::zero();
        task.context.x30 = entry as u64; // return address = entry point
        task.context.sp = stack_top as u64; // stack pointer
        task.context.elr = entry as u64;
        // SPSR: EL1h, interrupts enabled
        task.context.spsr = 0b00000000_00000000_00000000_00000101;

        set_task_name(task, name);

        Some(task.id)
    }
}

fn set_task_name(task: &mut Task, name: &str) {
    let len = name.len().min(32);
    task.name[..len].copy_from_slice(&name.as_bytes()[..len]);
    task.name_len = len;
}

/// Count of ready/running tasks (for scheduler).
pub fn count_ready() -> usize {
    let mut count = 0;
    for i in 0..MAX_TASKS {
        let task = get(TaskId(i as u16));
        if task.state == TaskState::Ready || task.state == TaskState::Running {
            count += 1;
        }
    }
    count
}
