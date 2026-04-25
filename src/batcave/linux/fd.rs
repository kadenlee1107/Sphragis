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

static mut FD_TABLE: [FdEntry; MAX_FDS] = [FdEntry::empty(); MAX_FDS];

/// Monotonically increasing allocation cursor. New fds come from
/// here, NOT from a linear scan that would reuse closed slots.
/// When the cursor hits MAX_FDS we fall back to scanning for free
/// slots (CHROMIUM-PHASE-D — Chromium's FD ownership tracker
/// FATALs on reused fd numbers).
static ALLOC_CURSOR: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(3);

/// V6-XLAYER-005/006 fix: clear every fd on cave switch. Without this
/// a new cave inherited the previous cave's open fds — including
/// sockets pointing at established TCP streams. Re-establishes
/// stdin/stdout/stderr at 0/1/2 like fresh boot.
pub fn reset_for_cave_switch() {
    init();
}

/// Initialize the fd table. Fds 0/1/2 are reserved (UART).
pub fn init() {
    unsafe {
        for i in 0..MAX_FDS {
            FD_TABLE[i] = FdEntry::empty();
        }
        // Mark stdin/stdout/stderr as active (handled specially)
        FD_TABLE[0].active = true;
        FD_TABLE[1].active = true;
        FD_TABLE[2].active = true;
    }
}

/// Allocate a new fd pointing to a VFS node.
pub fn alloc_fd(node_idx: u16, flags: u32) -> Result<u32, i64> {
    use core::sync::atomic::Ordering;
    unsafe {
        // Prefer the monotonic cursor: hands out fresh fd numbers
        // never previously seen, which keeps Chromium's FD
        // ownership tracker from confusing the new owner with the
        // old (closed) one.
        let cur = ALLOC_CURSOR.fetch_add(1, Ordering::AcqRel);
        if cur < MAX_FDS && !FD_TABLE[cur].active {
            FD_TABLE[cur] = FdEntry {
                active: true,
                node_idx,
                position: 0,
                flags,
                kind: FdKind::Vfs,
            };
            return Ok(cur as u32);
        }
        // Cursor ran out (or that slot is somehow live — e.g. the
        // 0/1/2 init slots). Fall back to a linear scan from 3.
        for i in 3..MAX_FDS {
            if !FD_TABLE[i].active {
                FD_TABLE[i] = FdEntry {
                    active: true,
                    node_idx,
                    position: 0,
                    flags,
                    kind: FdKind::Vfs,
                };
                return Ok(i as u32);
            }
        }
    }
    Err(-24) // EMFILE — too many open files
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
    if pair_slot >= 0x8000 { return Err(-22); } // EINVAL
    let packed = ((pair_slot as u16) << 1) | (side as u16 & 1);
    unsafe {
        let cur = ALLOC_CURSOR.fetch_add(1, Ordering::AcqRel);
        if cur < MAX_FDS && !FD_TABLE[cur].active {
            FD_TABLE[cur] = FdEntry {
                active: true,
                node_idx,
                position: 0,
                flags,
                kind: FdKind::Pipe(packed),
            };
            return Ok(cur as u32);
        }
        for i in 3..MAX_FDS {
            if !FD_TABLE[i].active {
                FD_TABLE[i] = FdEntry {
                    active: true,
                    node_idx,
                    position: 0,
                    flags,
                    kind: FdKind::Pipe(packed),
                };
                return Ok(i as u32);
            }
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
    unsafe {
        if FD_TABLE[fd].active { Some(&FD_TABLE[fd]) } else { None }
    }
}

/// Get an fd entry (mutable) for updating position.
pub fn get_mut(fd: u32) -> Option<&'static mut FdEntry> {
    let fd = fd as usize;
    if fd >= MAX_FDS { return None; }
    unsafe {
        if FD_TABLE[fd].active { Some(&mut FD_TABLE[fd]) } else { None }
    }
}

/// Close an fd.
pub fn close(fd: u32) -> Result<(), i64> {
    let fd = fd as usize;
    if fd < 3 { return Ok(()); } // don't close stdin/stdout/stderr
    if fd >= MAX_FDS { return Err(-9); } // EBADF
    unsafe {
        if !FD_TABLE[fd].active { return Err(-9); }
        FD_TABLE[fd] = FdEntry::empty();
    }
    Ok(())
}

/// Duplicate an fd.
pub fn dup(old_fd: u32) -> Result<u32, i64> {
    let old = old_fd as usize;
    if old >= MAX_FDS { return Err(-9); }
    unsafe {
        if !FD_TABLE[old].active { return Err(-9); }
        let entry = FD_TABLE[old];
        alloc_fd(entry.node_idx, entry.flags)
    }
}

/// Duplicate an fd to a specific new fd number.
pub fn dup2(old_fd: u32, new_fd: u32) -> Result<u32, i64> {
    let old = old_fd as usize;
    let new = new_fd as usize;
    if old >= MAX_FDS || new >= MAX_FDS { return Err(-9); }
    unsafe {
        if !FD_TABLE[old].active { return Err(-9); }
        // Close new_fd if open
        if new >= 3 && FD_TABLE[new].active {
            FD_TABLE[new] = FdEntry::empty();
        }
        FD_TABLE[new] = FD_TABLE[old];
    }
    Ok(new_fd)
}

/// Reset the fd table (called when a new process starts).
pub fn reset() {
    init();
}
