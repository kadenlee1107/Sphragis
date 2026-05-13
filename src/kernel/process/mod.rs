#![allow(dead_code)]
// Bat_OS — Process/Task Abstraction
// Each task represents an isolated execution context.
// In the microkernel, everything outside the kernel is a task:
// drivers, services, and applications.

use crate::kernel::mm::{frame, page_table::AddressSpace};
use crate::kernel::capability::CapabilitySet;

pub const MAX_TASKS: usize = 64;
const TASK_STACK_PAGES: usize = 4; // 16KB stack per task
pub const MAX_FDS_PER_TASK: usize = 16;

/// Kind-tagged file descriptor. Pipe and Socket are the live kinds
/// today; File variant slots in later when BatFS exposes inode-based
/// open(). The Task layout doesn't change across kinds — only the
/// payload of FdEntry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FdKind {
    Pipe { id: u16, end: PipeEnd },
    /// AF_UNIX socket fd. `id` indexes into the kernel socket table;
    /// `role` distinguishes listener fds (which `accept()` consumes)
    /// from connected stream fds (which `read`/`write` use).
    Socket { id: u16, role: SocketRole },
    /// POSIX shared-memory region fd. `id` indexes into the shm
    /// table. Read/write go directly to the region's bytes via
    /// `kernel::shm::region_bytes_mut`.
    Shm { id: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeEnd { Read, Write }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketRole {
    /// Fresh socket created by `socket()`; not yet bound or
    /// connected. `bind`/`connect` transitions it.
    Unbound,
    /// `listen()` was called; `accept()` pulls a connected fd off
    /// this socket's backlog.
    Listener,
    /// One end of an established AF_UNIX stream pair. `read`/`write`
    /// move bytes; `close` decrements the peer pair's refcount.
    Connected,
}

#[derive(Clone, Copy, Debug)]
pub struct FdEntry {
    pub kind: FdKind,
}

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
    pub fds: [Option<FdEntry>; MAX_FDS_PER_TASK],
    /// PID-namespace tag (gap-audit item 031). 0 = kernel namespace
    /// (always visible). Non-zero = cave id, only listed by
    /// `list_for_cave(cave_id)`. process::set_cave() updates this
    /// for a task; create_kernel_task tags new tasks 0 by default.
    pub cave_id: u16,
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
            fds: [None; MAX_FDS_PER_TASK],
            cave_id: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    /// Allocate the lowest-numbered free fd slot. Returns the fd
    /// number on success, or None if the task has run out.
    pub fn fd_alloc(&mut self, entry: FdEntry) -> Option<u16> {
        for i in 0..MAX_FDS_PER_TASK {
            if self.fds[i].is_none() {
                self.fds[i] = Some(entry);
                return Some(i as u16);
            }
        }
        None
    }

    pub fn fd_get(&self, fd: u16) -> Option<FdEntry> {
        let i = fd as usize;
        if i < MAX_FDS_PER_TASK { self.fds[i] } else { None }
    }

    /// Clear an fd slot and return its previous entry. Caller is
    /// responsible for any kind-specific refcount work (e.g.
    /// `pipe::release_end`) — this only forgets the mapping.
    pub fn fd_take(&mut self, fd: u16) -> Option<FdEntry> {
        let i = fd as usize;
        if i < MAX_FDS_PER_TASK { self.fds[i].take() } else { None }
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
        task.fds = [None; MAX_FDS_PER_TASK];

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

/// Tag a task with a PID-namespace cave id. Tasks tagged 0 are
/// visible from every namespace (kernel idle, system services);
/// tasks tagged with a non-zero cave id are only listed from the
/// matching cave's view.
pub fn set_cave(id: TaskId, cave_id: u16) {
    let task = get(id);
    task.cave_id = cave_id;
}

/// Voluntarily terminate the current kernel task. Marks the task
/// Dead (so the scheduler stops considering it Ready) and yields,
/// which forces a reschedule onto a different ready task.
///
/// Called by short-lived helper threads (e.g. the sys-caves
/// selftest worker) after they finish their work — without this,
/// a high-priority looping worker would starve the lower-priority
/// shell task it was supposed to hand control back to.
///
/// Diverges: by definition the dead task is never picked again,
/// so we never return from `yield_now`. Caller's stack and Task
/// slot live until the next reaper pass (not implemented yet —
/// for now they leak; the M4 has 4 GiB of RAM and tasks are
/// rare). Marked `-> !` so callers can use it in `fn() -> !`
/// task entry points.
pub fn current_terminate() -> ! {
    current().state = TaskState::Dead;
    loop {
        crate::kernel::scheduler::yield_now();
        // Defensive: if a buggy scheduler ever picks a Dead task,
        // re-mark and yield again rather than running forward.
        current().state = TaskState::Dead;
    }
}

/// Park the current task in `Blocked` state and yield. The
/// scheduler ignores Blocked tasks when picking, so the caller
/// stays off-CPU until some other task calls `process::wake(id)`
/// to flip the state back to Ready.
///
/// Used by long-running service tasks (e.g. `sys_wg_ipc`'s
/// service_main) that have nothing to do until a client posts
/// a request and wakes them.
pub fn current_block() {
    loop {
        current().state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();
        // On resume the scheduler's prologue runs with our state
        // == Ready (a wake flipped it before pick); the prologue
        // then flips Ready->Running. If somehow we resumed while
        // still Blocked (shouldn't happen with the current
        // scheduler), re-block.
        if current().state == TaskState::Running { break; }
    }
}

/// Flip a task from `Blocked` back to `Ready` so the scheduler
/// can pick it. No-op if the task is already Ready/Running or
/// Dead/Free.
///
/// Caller is responsible for any happens-before discipline the
/// woken task needs (e.g. posting a request payload to shared
/// memory + memory barrier before calling `wake`).
pub fn wake(id: TaskId) {
    let task = get(id);
    if task.state == TaskState::Blocked {
        task.state = TaskState::Ready;
    }
}

/// Iterate over tasks visible to the given cave. If cave_id == 0,
/// returns every active task (the global "root" view used by the
/// kernel for diagnostics). Otherwise filters to tasks whose
/// cave_id matches `cave_id` plus the always-visible kernel tasks
/// (cave_id == 0).
pub fn list_for_cave<F: FnMut(&Task)>(cave_id: u16, mut f: F) {
    unsafe {
        for i in 0..MAX_TASKS {
            let task = &(*core::ptr::addr_of!(TASKS))[i];
            if task.state == TaskState::Free { continue; }
            if cave_id == 0 || task.cave_id == 0 || task.cave_id == cave_id {
                f(task);
            }
        }
    }
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
