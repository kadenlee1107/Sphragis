// Bat_OS — File Descriptor Table for BatCave Linux Processes
// Maps Linux fd numbers to VFS node indices with read/write positions.
// Fds 0/1/2 are hardwired to stdin/stdout/stderr (UART).

// CHROMIUM-PHASE-C: bumped from 64 to 256 — Chromium's Mojo IPC
// and thread-pool each want a handful of fds; 64 was tight for a
// full content_shell run.
//
// CHROMIUM-PHASE-D: bumped to 1024 to back the no-reuse alloc
// cursor (see ALLOC_CURSOR below). Chromium's FD ownership
// tracker FATALs when a previously-closed fd number gets reused
// for a different role (e.g. epoll_create1 returns 5, but
// Chromium's tracker still has fd 5 attributed to a closed
// SubsystemA scoped_fd). Bumping MAX_FDS + handing out
// monotonically increasing fd numbers keeps that table happy.
const MAX_FDS: usize = 1024;

/// Kind-tag for fds that have backing beyond a VFS node. `Pipe`
/// carries a pair_slot (0..pipe_buf::MAX_PAIRS) and a side (0 or 1)
/// packed into a single u16: low bit = side, upper bits = slot.
#[derive(Clone, Copy, PartialEq)]
pub enum FdKind {
    Vfs,         // position indexes into node data
    Pipe(u16),   // (slot<<1)|side — see pipe_buf.rs
}

#[derive(Clone, Copy)]
pub struct FdEntry {
    pub active: bool,
    pub node_idx: u16,    // VFS node index
    pub position: usize,  // current read/write offset
    pub flags: u32,       // O_RDONLY, O_WRONLY, O_RDWR, O_APPEND, etc.
    pub kind: FdKind,
}

impl FdEntry {
    const fn empty() -> Self {
        FdEntry {
            active: false,
            node_idx: 0,
            position: 0,
            flags: 0,
            kind: FdKind::Vfs,
        }
    }
}

// Linux open flags
pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32 = 2;
pub const O_CREAT: u32 = 0o100;
pub const O_TRUNC: u32 = 0o1000;
pub const O_APPEND: u32 = 0o2000;
pub const O_DIRECTORY: u32 = 0o200000;
pub const O_CLOEXEC: u32 = 0o2000000;

// AT_FDCWD for *at() syscalls
pub const AT_FDCWD: i32 = -100;

// CHROMIUM-PHASE-D: per-cave FD tables. POSIX semantics: each
// process has its own fd table; child's close() doesn't affect
// parent. Without this, a forked child closing fd N (its own
// post-fork dup) closes parent's fd N too, and parent's later
// sendmsg/read on that fd FATALs.
//
// Indexed by cave-slot from `mmu::current_cave_slot()`. On fork,
// `clone_fd_table()` copies parent's table to child's slot.
const NUM_CAVES: usize = crate::batcave::linux::mmu::NUM_CAVES;
static mut FD_TABLES: [[FdEntry; MAX_FDS]; NUM_CAVES] =
    [[FdEntry::empty(); MAX_FDS]; NUM_CAVES];
static ALLOC_CURSORS: [core::sync::atomic::AtomicUsize; NUM_CAVES] = [
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
    core::sync::atomic::AtomicUsize::new(3),
];

#[inline]
fn current_table() -> &'static mut [FdEntry; MAX_FDS] {
    let slot = crate::batcave::linux::mmu::current_cave_slot();
    unsafe {
        let tables = &mut *core::ptr::addr_of_mut!(FD_TABLES);
        &mut tables[slot]
    }
}

#[inline]
fn current_cursor() -> &'static core::sync::atomic::AtomicUsize {
    let slot = crate::batcave::linux::mmu::current_cave_slot();
    &ALLOC_CURSORS[slot]
}

/// Copy the host cave's (slot 0) FD table to the given child slot.
/// Called from threads::real_fork after the child cave is
/// allocated so the child inherits the parent's open fds — POSIX
/// fork semantics. Subsequent close() / dup() in the child only
/// touches the child's slot.
pub fn clone_fd_table(child_slot: usize) {
    if child_slot >= NUM_CAVES { return; }
    let parent_slot = 0; // host cave is always slot 0
    if child_slot == parent_slot { return; }
    use core::sync::atomic::Ordering;
    unsafe {
        let tables = &mut *core::ptr::addr_of_mut!(FD_TABLES);
        // Force-copy the array by swapping out via tmp.
        let parent_copy = tables[parent_slot];
        tables[child_slot] = parent_copy;
    }
    // Child's allocator cursor starts where parent's was — fds
    // before that point are inherited / valid; fds after start
    // fresh in the child's table.
    let parent_cur = ALLOC_CURSORS[parent_slot].load(Ordering::Acquire);
    ALLOC_CURSORS[child_slot].store(parent_cur, Ordering::Release);
}

/// V6-XLAYER-005/006 fix: clear every fd on cave switch. Without this
/// a new cave inherited the previous cave's open fds — including
/// sockets pointing at established TCP streams. Re-establishes
/// stdin/stdout/stderr at 0/1/2 like fresh boot.
pub fn reset_for_cave_switch() {
    init();
}

/// Initialize the fd table for the CURRENT cave. Fds 0/1/2
/// reserved (UART). Note: only the host cave (slot 0) typically
/// calls init(); forked children inherit the parent's table via
/// `clone_fd_table()`.
pub fn init() {
    let table = current_table();
    for i in 0..MAX_FDS {
        table[i] = FdEntry::empty();
    }
    table[0].active = true;
    table[1].active = true;
    table[2].active = true;
}

/// Allocate a new fd pointing to a VFS node.
pub fn alloc_fd(node_idx: u16, flags: u32) -> Result<u32, i64> {
    use core::sync::atomic::Ordering;
    let table = current_table();
    let cursor = current_cursor();
    let cur = cursor.fetch_add(1, Ordering::AcqRel);
    if cur < MAX_FDS && !table[cur].active {
        table[cur] = FdEntry {
            active: true,
            node_idx,
            position: 0,
            flags,
            kind: FdKind::Vfs,
        };
        return Ok(cur as u32);
    }
    for i in 3..MAX_FDS {
        if !table[i].active {
            table[i] = FdEntry {
                active: true,
                node_idx,
                position: 0,
                flags,
                kind: FdKind::Vfs,
            };
            return Ok(i as u32);
        }
    }
    Err(-24) // EMFILE
}

/// Allocate an fd backed by a pipe-buffer pair slot. Used by
/// socketpair(2) / pipe2(2). `pair_slot` is the index returned by
/// `pipe_buf::alloc_pair`, `side` is 0 (end A) or 1 (end B). The
/// caller should also create a fresh VFS Socket node and pass its
/// index in `node_idx` so stat / poll-by-type work.
pub fn alloc_fd_pipe(node_idx: u16, flags: u32, pair_slot: usize, side: u8)
    -> Result<u32, i64>
{
    use core::sync::atomic::Ordering;
    if pair_slot >= 0x8000 { return Err(-22); }
    let packed = ((pair_slot as u16) << 1) | (side as u16 & 1);
    let table = current_table();
    let cursor = current_cursor();
    let cur = cursor.fetch_add(1, Ordering::AcqRel);
    if cur < MAX_FDS && !table[cur].active {
        table[cur] = FdEntry {
            active: true,
            node_idx,
            position: 0,
            flags,
            kind: FdKind::Pipe(packed),
        };
        return Ok(cur as u32);
    }
    for i in 3..MAX_FDS {
        if !table[i].active {
            table[i] = FdEntry {
                active: true,
                node_idx,
                position: 0,
                flags,
                kind: FdKind::Pipe(packed),
            };
            return Ok(i as u32);
        }
    }
    Err(-24)
}

/// If this fd is a pipe-end, return (pair_slot, side). Else None.
pub fn pipe_info(fd: u32) -> Option<(usize, u8)> {
    let entry = get(fd)?;
    match entry.kind {
        FdKind::Pipe(packed) => {
            Some(((packed >> 1) as usize, (packed & 1) as u8))
        }
        _ => None,
    }
}

/// Get an fd entry (immutable).
pub fn get(fd: u32) -> Option<&'static FdEntry> {
    let fd = fd as usize;
    if fd >= MAX_FDS { return None; }
    let table = current_table();
    if table[fd].active { Some(&table[fd]) } else { None }
}

/// Get an fd entry (mutable) for updating position.
pub fn get_mut(fd: u32) -> Option<&'static mut FdEntry> {
    let fd = fd as usize;
    if fd >= MAX_FDS { return None; }
    let table = current_table();
    if table[fd].active { Some(&mut table[fd]) } else { None }
}

/// Close an fd.
pub fn close(fd: u32) -> Result<(), i64> {
    let fd = fd as usize;
    if fd < 3 { return Ok(()); }
    if fd >= MAX_FDS { return Err(-9); }
    let table = current_table();
    if !table[fd].active { return Err(-9); }
    table[fd] = FdEntry::empty();
    Ok(())
}

/// Duplicate an fd.
pub fn dup(old_fd: u32) -> Result<u32, i64> {
    let old = old_fd as usize;
    if old >= MAX_FDS { return Err(-9); }
    let table = current_table();
    if !table[old].active { return Err(-9); }
    let entry = table[old];
    alloc_fd(entry.node_idx, entry.flags)
}

/// Duplicate an fd to a specific new fd number.
pub fn dup2(old_fd: u32, new_fd: u32) -> Result<u32, i64> {
    let old = old_fd as usize;
    let new = new_fd as usize;
    if old >= MAX_FDS || new >= MAX_FDS { return Err(-9); }
    let table = current_table();
    if !table[old].active { return Err(-9); }
    if new >= 3 && table[new].active {
        table[new] = FdEntry::empty();
    }
    table[new] = table[old];
    Ok(new_fd)
}

/// Reset the fd table (called when a new process starts).
pub fn reset() {
    init();
}
