// Bat_OS — File Descriptor Table for BatCave Linux Processes
// Maps Linux fd numbers to VFS node indices with read/write positions.
// Fds 0/1/2 are hardwired to stdin/stdout/stderr (UART).

const MAX_FDS: usize = 64;

#[derive(Clone, Copy)]
pub struct FdEntry {
    pub active: bool,
    pub node_idx: u16,    // VFS node index
    pub position: usize,  // current read/write offset
    pub flags: u32,       // O_RDONLY, O_WRONLY, O_RDWR, O_APPEND, etc.
}

impl FdEntry {
    const fn empty() -> Self {
        FdEntry {
            active: false,
            node_idx: 0,
            position: 0,
            flags: 0,
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
    unsafe {
        for i in 3..MAX_FDS {
            if !FD_TABLE[i].active {
                FD_TABLE[i] = FdEntry {
                    active: true,
                    node_idx,
                    position: 0,
                    flags,
                };
                return Ok(i as u32);
            }
        }
    }
    Err(-24) // EMFILE — too many open files
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
