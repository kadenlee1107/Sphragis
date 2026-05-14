// Sphragis — In-Memory Virtual Filesystem for BatCave Containers
// Provides a Linux-like directory tree so busybox sees /bin, /etc, /tmp, etc.
// All data lives in RAM (frame allocator pages). No disk.

use crate::kernel::mm::frame;
use crate::drivers::uart;

const MAX_NODES: usize = 512;
const NAME_LEN: usize = 64;
const LINK_LEN: usize = 128;
const PAGE_SIZE: usize = 4096;
// Max pages per file (4KB each → 256KB max file)
const MAX_FILE_PAGES: usize = 64;

#[derive(Clone, Copy, PartialEq)]
pub enum NodeType {
    Free,
    Directory,
    File,
    Symlink,
    DevNull,
    DevZero,
    DevConsole,
    DevRandom,     // /dev/random + /dev/urandom — HW RNG via ARMv8.5 RNDR
    Socket,     // network socket (TCP/UDP)
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VfsNode {
    pub node_type: NodeType,
    pub name: [u8; NAME_LEN],
    pub name_len: usize,
    pub parent: u16,
    pub mode: u32,
    pub size: usize,
    pub data_addr: usize,       // physical address of file data pages
    pub link_target: [u8; LINK_LEN],
    pub link_len: usize,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
}

impl VfsNode {
    const fn empty() -> Self {
        VfsNode {
            node_type: NodeType::Free,
            name: [0; NAME_LEN],
            name_len: 0,
            parent: 0,
            mode: 0,
            size: 0,
            data_addr: 0,
            link_target: [0; LINK_LEN],
            link_len: 0,
            uid: 0,
            gid: 0,
            nlink: 1,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    pub fn link_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.link_target[..self.link_len]) }
    }
}

// Support up to 8 concurrent VFS instances (one per active BatCave)
const MAX_VFS_INSTANCES: usize = 8;

/// A complete VFS instance for one BatCave.
struct VfsInstance {
    nodes: [VfsNode; MAX_NODES],
    cwd: u16,
    ready: bool,
    cave_id: usize, // which BatCave owns this instance
}

impl VfsInstance {
    const fn empty() -> Self {
        VfsInstance {
            nodes: [VfsNode::empty(); MAX_NODES],
            cwd: 0,
            ready: false,
            cave_id: usize::MAX,
        }
    }
}

static mut INSTANCES: [VfsInstance; MAX_VFS_INSTANCES] = {
    const EMPTY: VfsInstance = VfsInstance::empty();
    [EMPTY; MAX_VFS_INSTANCES]
};
static mut ACTIVE_INSTANCE: usize = 0;

// Legacy compatibility statics — redirect to active instance
static mut VFS_READY: bool = false;

/// Initialize a VFS instance for a specific BatCave.
/// If cave_id == usize::MAX, initializes the default (slot 0).
pub fn init() {
    init_for_cave(usize::MAX);
}

/// Initialize VFS for a specific cave. Allocates a VFS slot.
pub fn init_for_cave(cave_id: usize) {
    // Find a free slot (or reuse existing for this cave)
    let slot = unsafe {
        let instances = &*core::ptr::addr_of!(INSTANCES);
        let mut found = usize::MAX;
        for i in 0..MAX_VFS_INSTANCES {
            if instances[i].cave_id == cave_id && instances[i].ready {
                found = i; break; // reuse existing
            }
        }
        if found == usize::MAX {
            for i in 0..MAX_VFS_INSTANCES {
                if !instances[i].ready {
                    found = i; break;
                }
            }
        }
        if found == usize::MAX { found = 0; } // fallback to slot 0
        core::ptr::write_volatile(core::ptr::addr_of_mut!(ACTIVE_INSTANCE), found);
        found
    };

    unsafe {
        let inst = &mut (*core::ptr::addr_of_mut!(INSTANCES))[slot];
        inst.cave_id = cave_id;
        for i in 0..MAX_NODES {
            inst.nodes[i] = VfsNode::empty();
        }

        // Node 0 = root directory "/"
        inst.nodes[0].node_type = NodeType::Directory;
        inst.nodes[0].name[0] = b'/';
        inst.nodes[0].name_len = 1;
        inst.nodes[0].parent = 0;
        inst.nodes[0].mode = 0o40755;
        inst.nodes[0].nlink = 2;

        inst.cwd = 0;
        inst.ready = true;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VFS_READY), true);
    }

    populate_rootfs();
    uart::puts("  [vfs] Filesystem ready (cave ");
    if cave_id == usize::MAX {
        uart::puts("default");
    } else {
        crate::kernel::mm::print_num(cave_id);
    }
    uart::puts(")\n");
}

/// Destroy a cave's VFS instance (wipe all data).
// /
/// V11-FRESH-EYES: previously this only zeroed the node-metadata structs,
/// leaking **physical frames** that each node's `data_addr` referenced.
/// Two compounding bugs:
/// (1) plaintext file contents persisted in those frames forever
/// (because they were never returned to the frame allocator), and
/// (2) the frame bitmap bits stayed set, yielding an unprivileged
/// OOM-DoS primitive — any cave with the `mem` cap could exhaust
/// physical memory by cycling create-write-destroy.
/// Now we scrub and free each file-data frame before clearing the node.
pub fn destroy_cave_vfs(cave_id: usize) {
    unsafe {
        let instances = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for i in 0..MAX_VFS_INSTANCES {
            if instances[i].cave_id == cave_id {
                // V11 fix: walk every live node, zero its backing frame,
                // then return it to the allocator.
                //
                // V12b: skip nodes whose data_addr points into the
                // initrd BATARCH archive memory (populate_lib_from_archive
                // registers /lib/*.so that way). Those bytes are owned by
                // the initrd region — freeing them would corrupt the
                // archive + leak allocator state (the frames were never
                // handed out by alloc_frame to begin with).
                let (archive_lo, archive_hi) = crate::kernel::mm::initrd::blob_phys_range();
                for j in 0..MAX_NODES {
                    let n = &mut instances[i].nodes[j];
                    let in_archive = archive_lo != 0 && archive_hi > archive_lo
                        && n.data_addr >= archive_lo && n.data_addr < archive_hi;
                    if n.data_addr != 0
                        && !in_archive
                    {
                        // Volatile-zero the 4 KiB frame so a later
                        // `alloc_frame` tenant cannot observe the old
                        // file content even before alloc's str-xzr pass.
                        let base = n.data_addr as *mut u8;
                        for b in 0..4096usize {
                            core::ptr::write_volatile(base.add(b), 0);
                        }
                        crate::kernel::mm::frame::free_frame(n.data_addr);
                        n.data_addr = 0;
                    }
                    instances[i].nodes[j] = VfsNode::empty();
                }
                instances[i].ready = false;
                instances[i].cave_id = usize::MAX;
                return;
            }
        }
    }
}

/// Check if VFS is initialized.
pub fn is_ready() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VFS_READY)) }
}

// ─── Node Operations ───

/// Find a child node by name within a parent directory.
pub fn find_child(parent_idx: u16, name: &[u8]) -> Option<u16> {
    unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &(*core::ptr::addr_of!(INSTANCES))[idx].nodes;
        for i in 0..MAX_NODES {
            let n = &nodes[i];
            if n.node_type == NodeType::Free { continue; }
            if n.parent != parent_idx { continue; }
            if i as u16 == parent_idx && parent_idx == 0 { continue; } // skip root self-ref
            if n.name_len == name.len() {
                let mut eq = true;
                for j in 0..name.len() {
                    if n.name[j] != name[j] { eq = false; break; }
                }
                if eq { return Some(i as u16); }
            }
        }
    }
    None
}

/// Allocate a free node slot.
fn alloc_node() -> Option<u16> {
    unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &(*core::ptr::addr_of!(INSTANCES))[idx].nodes;
        for i in 1..MAX_NODES { // skip 0 (root)
            if nodes[i].node_type == NodeType::Free {
                return Some(i as u16);
            }
        }
    }
    None
}

/// Create a new node under a parent directory.
pub fn create_node(parent: u16, name: &[u8], ntype: NodeType, mode: u32) -> Result<u16, i64> {
    if name.len() > NAME_LEN { return Err(-36); } // ENAMETOOLONG
    let idx = alloc_node().ok_or(-28i64)?; // ENOSPC

    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes;
        let n = &mut nodes[idx as usize];
        n.node_type = ntype;
        n.name_len = name.len();
        // CHROMIUM-PHASE-B: writes to n.name[i] through normal field
        // access were landing on the wrong bytes at runtime — the
        // name_len set on the line above persisted correctly but the
        // name array came out as junk ('a??' / 'TH?' / etc.) instead
        // of 'bin' / 'etc'. Root cause unknown (possibly stale I-cache
        // on the store buffer path into .data at boot, possibly an
        // alias/aliasing miscompile in release). Switched to an
        // explicit name-array pointer + volatile writes, which dodges
        // the issue and is also marginally faster than an indexed loop.
        let name_ptr = core::ptr::addr_of_mut!(n.name) as *mut u8;
        for i in 0..name.len() {
            core::ptr::write_volatile(name_ptr.add(i), name[i]);
        }
        // Zero any tail so leftover bytes from the previous occupant
        // don't leak through `name_str()`.
        for i in name.len()..NAME_LEN {
            core::ptr::write_volatile(name_ptr.add(i), 0);
        }
        n.parent = parent;
        n.mode = mode;
        n.size = 0;
        n.data_addr = 0;
        n.link_target = [0; LINK_LEN];
        n.link_len = 0;
        n.uid = 0;
        n.gid = 0;
        n.nlink = if ntype == NodeType::Directory { 2 } else { 1 };

        // Update parent nlink for directories
        if ntype == NodeType::Directory {
            nodes[parent as usize].nlink += 1;
        }
    }
    Ok(idx)
}

/// Create a symlink node.
pub fn create_symlink(parent: u16, name: &[u8], target: &[u8]) -> Result<u16, i64> {
    let idx = create_node(parent, name, NodeType::Symlink, 0o120777)?;
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
        n.link_len = target.len().min(LINK_LEN);
        for i in 0..n.link_len {
            n.link_target[i] = target[i];
        }
    }
    Ok(idx)
}

/// Write data to a file node (allocates pages as needed).
pub fn write_file_data(idx: u16, data: &[u8]) -> Result<(), i64> {
    let pages_needed = (data.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    if pages_needed > MAX_FILE_PAGES { return Err(-27); } // EFBIG

    // Allocate contiguous pages
    let base = frame::alloc_frame().ok_or(-12i64)?; // ENOMEM
    for _ in 1..pages_needed {
        frame::alloc_frame().ok_or(-12i64)?;
    }

    // Copy data using inline asm (HVF-safe)
    for i in 0..data.len() {
        unsafe {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) base + i,
                v = in(reg) data[i] as u32);
        }
    }

    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
        n.data_addr = base;
        n.size = data.len();
    }
    Ok(())
}

/// Set a node's size (ftruncate-style). Used by sys_ftruncate so that
/// Chromium's PlatformSharedMemoryRegion::TakeOrFail sees a non-zero
/// size on the freshly-created shm file. We don't actually allocate
/// backing data here — the small-mmap path provides demand-paged zero
/// pages on first access.
pub fn set_node_size(idx: u16, new_size: usize) -> Result<(), i64> {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
        if n.node_type != NodeType::File { return Err(-22); } // EINVAL
        n.size = new_size;
        Ok(())
    }
}

/// Read file data into a userspace buffer. Returns bytes read.
pub fn read_file_data(idx: u16, offset: usize, buf_addr: usize, count: usize) -> Result<usize, i64> {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &(*core::ptr::addr_of!(INSTANCES))[ai].nodes[idx as usize];
        if n.data_addr == 0 || offset >= n.size { return Ok(0); } // EOF

        let available = n.size - offset;
        let to_read = available.min(count);
        let src = n.data_addr + offset;

        for i in 0..to_read {
            let byte: u32;
            core::arch::asm!("ldrb {v:w}, [{a}]",
                a = in(reg) src + i, v = out(reg) byte);
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) buf_addr + i, v = in(reg) byte);
        }
        Ok(to_read)
    }
}

/// Write data from userspace buffer to a file. Returns bytes written.
pub fn write_to_file(idx: u16, offset: usize, buf_addr: usize, count: usize) -> Result<usize, i64> {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
        let new_size = offset + count;

        // Allocate data page if needed
        if n.data_addr == 0 {
            let pages = (new_size + PAGE_SIZE - 1) / PAGE_SIZE;
            let base = frame::alloc_frame().ok_or(-12i64)?;
            for _ in 1..pages {
                frame::alloc_frame().ok_or(-12i64)?;
            }
            n.data_addr = base;
            // Zero the allocated pages
            for i in 0..(pages * PAGE_SIZE) {
                core::arch::asm!("strb wzr, [{a}]", a = in(reg) base + i);
            }
        }

        let dst = n.data_addr + offset;
        for i in 0..count {
            let byte: u32;
            core::arch::asm!("ldrb {v:w}, [{a}]",
                a = in(reg) buf_addr + i, v = out(reg) byte);
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) dst + i, v = in(reg) byte);
        }

        if new_size > n.size {
            n.size = new_size;
        }
        Ok(count)
    }
}

// ─── Path Resolution ───

/// Local copy of the syscall-layer `..` rejector so the VFS can refuse
/// symlink targets that escape the sandbox. Kept here (not pulled from
/// syscall::has_dotdot) to avoid a layering loop.
fn has_dotdot_bytes(path: &[u8]) -> bool {
    let mut i = 0usize;
    while i < path.len() {
        let start = if path[i] == b'/' { i + 1 } else { i };
        let mut end = start;
        while end < path.len() && path[end] != b'/' { end += 1; }
        if end - start == 2 && &path[start..end] == b".." { return true; }
        if end == path.len() { break; }
        i = end + 1;
    }
    false
}

/// Resolve a path string to a node index.
/// Handles absolute ("/bin/sh") and relative ("../etc") paths.
/// Follows symlinks up to 8 levels deep.
pub fn resolve_path(path: &[u8]) -> Result<u16, i64> {
    resolve_path_depth(path, 0)
}

fn resolve_path_depth(path: &[u8], depth: usize) -> Result<u16, i64> {
    if depth > 8 { return Err(-40); } // ELOOP
    if path.is_empty() { return Err(-2); } // ENOENT

    let mut current: u16 = if path[0] == b'/' {
        0 // absolute path → start at root
    } else {
        unsafe {
            let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
            (*core::ptr::addr_of!(INSTANCES))[ai].cwd
        } // relative → start at cwd
    };

    let mut i = 0;
    let len = path.len();

    while i < len {
        // Skip leading slashes
        while i < len && path[i] == b'/' { i += 1; }
        if i >= len { break; }

        // Extract component
        let start = i;
        while i < len && path[i] != b'/' { i += 1; }
        let component = &path[start..i];

        if component.is_empty() { continue; }

        // Handle "." and ".."
        if component == b"." { continue; }
        if component == b".." {
            current = unsafe {
                let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
                (*core::ptr::addr_of!(INSTANCES))[ai].nodes[current as usize].parent
            };
            continue;
        }

        // Look up child
        match find_child(current, component) {
            Some(child) => {
                // V8-ROOT-1 / V8-IRQ-#11 / V8-WEIRD: copy the symlink
                // target into a stack buffer under IRQ-mask so a
                // concurrent unlink+recreate can't swap the target
                // between has_dotdot_bytes() and the recursive resolve.
                let (is_symlink, target_buf, target_len) = {
                    let _g = crate::kernel::sync::IrqGuard::new();
                    unsafe {
                        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
                        let nodes = &(*core::ptr::addr_of!(INSTANCES))[ai].nodes;
                        let n = &nodes[child as usize];
                        if n.node_type == NodeType::Symlink {
                            let mut buf = [0u8; 256];
                            let len = n.link_len.min(256);
                            buf[..len].copy_from_slice(&n.link_target[..len]);
                            (true, buf, len)
                        } else {
                            (false, [0u8; 256], 0)
                        }
                    }
                };
                if is_symlink {
                    let target = &target_buf[..target_len];
                    // FLv2-NEW-013: refuse `..` in symlink target.
                    if has_dotdot_bytes(target) {
                        return Err(-13); // EACCES
                    }
                    if i < len {
                        let mut combined = [0u8; 256];
                        let mut clen = 0;
                        for &b in target { if clen < 255 { combined[clen] = b; clen += 1; } }
                        if clen < 255 { combined[clen] = b'/'; clen += 1; }
                        for j in i..len { if clen < 255 { combined[clen] = path[j]; clen += 1; } }
                        return resolve_path_depth(&combined[..clen], depth + 1);
                    } else {
                        let resolved = resolve_path_depth(target, depth + 1)?;
                        current = resolved;
                        continue;
                    }
                }
                current = child;
            }
            None => return Err(-2), // ENOENT
        }
    }

    Ok(current)
}

/// Resolve path but return the parent directory and the final component name.
/// Used for creating new files/dirs.
pub fn resolve_parent(path: &[u8]) -> Result<(u16, &[u8]), i64> {
    if path.is_empty() { return Err(-2); }

    // Find the last '/' to split parent path and basename
    let mut last_slash = None;
    for i in (0..path.len()).rev() {
        if path[i] == b'/' { last_slash = Some(i); break; }
    }

    match last_slash {
        Some(pos) => {
            let parent_path = if pos == 0 { &path[..1] } else { &path[..pos] };
            let name = &path[pos + 1..];
            let parent = resolve_path(parent_path)?;
            Ok((parent, name))
        }
        None => {
            // No slash → relative to CWD
            let parent = unsafe {
                let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
                (*core::ptr::addr_of!(INSTANCES))[ai].cwd
            };
            Ok((parent, path))
        }
    }
}

/// Get the full path of a node (by walking parent pointers).
pub fn node_path(idx: u16, buf: &mut [u8]) -> usize {
    let mut stack = [0u16; 32];
    let mut depth = 0;

    // Walk up to root
    let mut cur = idx;
    while cur != 0 && depth < 32 {
        stack[depth] = cur;
        depth += 1;
        cur = unsafe {
            let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
            (*core::ptr::addr_of!(INSTANCES))[ai].nodes[cur as usize].parent
        };
    }

    if idx == 0 {
        // Root
        if !buf.is_empty() { buf[0] = b'/'; }
        return 1;
    }

    let mut pos = 0;
    // Build path from root down
    for d in (0..depth).rev() {
        let n = unsafe {
            let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
            &(*core::ptr::addr_of!(INSTANCES))[ai].nodes[stack[d] as usize]
        };
        if pos < buf.len() { buf[pos] = b'/'; pos += 1; }
        for i in 0..n.name_len {
            if pos < buf.len() { buf[pos] = n.name[i]; pos += 1; }
        }
    }

    pos
}

/// Get a node reference by index.
pub fn get_node(idx: u16) -> &'static VfsNode {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        &(*core::ptr::addr_of!(INSTANCES))[ai].nodes[idx as usize]
    }
}

/// Get current working directory index.
pub fn get_cwd() -> u16 {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        (*core::ptr::addr_of!(INSTANCES))[ai].cwd
    }
}

/// Set current working directory.
pub fn set_cwd(idx: u16) {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        (*core::ptr::addr_of_mut!(INSTANCES))[ai].cwd = idx;
    }
}

/// List children of a directory. Calls `f` for each child (index, node).
pub fn list_children<F: FnMut(u16, &VfsNode)>(parent_idx: u16, mut f: F) {
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &(*core::ptr::addr_of!(INSTANCES))[ai].nodes;
        for i in 1..MAX_NODES {
            let n = &nodes[i];
            if n.node_type == NodeType::Free { continue; }
            if n.parent == parent_idx {
                f(i as u16, n);
            }
        }
    }
}

/// Delete a node (mark as Free). Does not free data pages (no free_frame yet).
pub fn remove_node(idx: u16) -> Result<(), i64> {
    if idx == 0 { return Err(-16); } // EBUSY — can't remove root
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes;
        let ntype = nodes[idx as usize].node_type;
        // Check directory is empty
        if ntype == NodeType::Directory {
            let mut has_children = false;
            for i in 1..MAX_NODES {
                if nodes[i].node_type != NodeType::Free && nodes[i].parent == idx {
                    has_children = true;
                    break;
                }
            }
            if has_children { return Err(-39); } // ENOTEMPTY
        }
        nodes[idx as usize].node_type = NodeType::Free;
    }
    Ok(())
}

/// Rename `oldpath` → `newpath` within the active VFS instance.
// /
/// Semantics (best-effort POSIX `rename(2)`):
/// If oldpath doesn't exist → ENOENT
/// If newpath's parent doesn't exist → ENOENT
/// If newpath exists and is a non-empty directory → ENOTEMPTY
/// If newpath exists and is a non-directory → atomically replaced
/// (its node freed, source node re-parented + renamed)
/// Otherwise: source node has its `parent` and `name` fields
/// overwritten to the new location
// /
/// iter 3: Chromium's LevelDB writes `MANIFEST.tmp`, then
/// renames it to `CURRENT`. Pre-iter-3 our renameat was sys_stub_zero
/// (returns success but does nothing) — the leveldb open path then
/// failed reading `CURRENT` because nothing actually moved, and
/// every leveldb-using subsystem (SharedDictionary, shared_proto_db,
/// SimpleCache index) spent the rest of the run in a 200+ retry loop.
pub fn rename_node(oldpath: &[u8], newpath: &[u8]) -> Result<(), i64> {
    // 1. Find the source node.
    let src_idx = resolve_path(oldpath)?;
    if src_idx == 0 { return Err(-16); } // EBUSY — can't rename root

    // 2. Find the destination parent + new name.
    let (new_parent, new_name) = resolve_parent(newpath)?;
    if new_name.is_empty() { return Err(-22); } // EINVAL
    if new_name.len() > NAME_LEN { return Err(-36); } // ENAMETOOLONG

    // 3. If destination exists, remove it. POSIX requires this be
    // atomic; for our single-threaded-VFS-with-IrqGuard this
    // is fine because we hold no lock points between unlink
    // and the parent/name update.
    if let Some(existing) = find_child(new_parent, new_name) {
        if existing == src_idx {
            // Renaming to itself: no-op success.
            return Ok(());
        }
        remove_node(existing)?;
    }

    // 4. Update src_idx's parent + name in place.
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let nodes = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes;
        let n = &mut nodes[src_idx as usize];
        n.parent = new_parent;
        // Rewrite the name buffer through volatile stores (matches
        // create_node's idiom — the indexed-write path was observed
        // miscompiling in release builds).
        let name_ptr = core::ptr::addr_of_mut!(n.name) as *mut u8;
        for i in 0..new_name.len() {
            core::ptr::write_volatile(name_ptr.add(i), new_name[i]);
        }
        for i in new_name.len()..NAME_LEN {
            core::ptr::write_volatile(name_ptr.add(i), 0);
        }
        n.name_len = new_name.len();
    }

    Ok(())
}

// ─── Rootfs Population ───

fn populate_rootfs() {
    // Create standard directories
    //
    // CHROMIUM-PHASE-B: a `let dirs: &[&[u8]] = &[b"bin", b"etc", ...]`
    // array declared inside this function was reaching create_node as
    // GARBAGE bytes ('@??', 'Tlj', ...) with correct lengths but
    // scrambled contents. Hoisted to a `const DIRS` so it's a direct
    // .rodata reference instead of a function-local temporary — fixed.
    // (Root cause still unconfirmed: likely a release-mode optimiser
    // bug around function-local slice-of-slices, or an ARM64 cache-
    // maintenance gap during boot. Same fix applied to `APPLETS`.)
    const DIRS: &[(&[u8], u32)] = &[
        (b"bin",  0o40755),
        (b"etc",  0o40755),
        (b"usr",  0o40755),
        (b"tmp",  0o41777),
        (b"dev",  0o40755),
        (b"proc", 0o40755),
        (b"root", 0o40755),
        (b"var",  0o40755),
        (b"sbin", 0o40755),
    ];
    for &(name, mode) in DIRS {
        let _ = create_node(0, name, NodeType::Directory, mode);
    }


    // /usr/bin + /usr/share/fonts — fontconfig looks here for fonts
    if let Some(usr) = find_child(0, b"usr") {
        create_node(usr, b"bin", NodeType::Directory, 0o40755).ok();
        create_node(usr, b"sbin", NodeType::Directory, 0o40755).ok();
        let _ = create_node(usr, b"share", NodeType::Directory, 0o40755);
        if let Some(share) = find_child(usr, b"share") {
            create_node(share, b"fonts", NodeType::Directory, 0o40755).ok();
        }
    }

    // /var/tmp
    if let Some(var) = find_child(0, b"var") {
        create_node(var, b"tmp", NodeType::Directory, 0o41777).ok();
    }

    // /etc files
    if let Some(etc) = find_child(0, b"etc") {
        if let Ok(passwd) = create_node(etc, b"passwd", NodeType::File, 0o100644) {
            write_file_data(passwd, b"root:x:0:0:root:/root:/bin/sh\n").ok();
        }
        if let Ok(group) = create_node(etc, b"group", NodeType::File, 0o100644) {
            write_file_data(group, b"root:x:0:\n").ok();
        }
        if let Ok(hostname) = create_node(etc, b"hostname", NodeType::File, 0o100644) {
            write_file_data(hostname, b"batcave\n").ok();
        }
        if let Ok(profile) = create_node(etc, b"profile", NodeType::File, 0o100644) {
            write_file_data(profile, b"export PATH=/bin:/usr/bin:/sbin\nexport HOME=/root\n").ok();
        }
        if let Ok(shells) = create_node(etc, b"shells", NodeType::File, 0o100644) {
            write_file_data(shells, b"/bin/sh\n/bin/ash\n").ok();
        }
        // DNS resolver config — QEMU user-net DNS at 10.0.2.3
        if let Ok(resolv) = create_node(etc, b"resolv.conf", NodeType::File, 0o100644) {
            write_file_data(resolv, b"nameserver 10.0.2.3\n").ok();
        }
        // Hosts file
        if let Ok(hosts) = create_node(etc, b"hosts", NodeType::File, 0o100644) {
            write_file_data(hosts, b"127.0.0.1 localhost\n10.0.2.15 batcave\n10.0.2.2 host\n").ok();
        }
        // nsswitch for musl
        if let Ok(nss) = create_node(etc, b"nsswitch.conf", NodeType::File, 0o100644) {
            write_file_data(nss, b"hosts: files dns\n").ok();
        }
    }

    // /dev special files
    if let Some(dev) = find_child(0, b"dev") {
        create_node(dev, b"null", NodeType::DevNull, 0o20666).ok();
        create_node(dev, b"zero", NodeType::DevZero, 0o20666).ok();
        create_node(dev, b"console", NodeType::DevConsole, 0o20600).ok();
        create_node(dev, b"urandom", NodeType::DevRandom, 0o20666).ok();
        create_node(dev, b"random",  NodeType::DevRandom, 0o20666).ok();
        // /dev/shm — POSIX shared-memory tmpfs. Chromium accesses
        // it for shm_open / mmap-of-shared-memory. We don't have a
        // real tmpfs but creating the directory + giving it world-
        // RWX gets past the access(W_OK|X_OK) check that otherwise
        // FATALs base/memory/platform_shared_memory_region_posix.cc
        // very early in init.
        create_node(dev, b"shm", NodeType::Directory, 0o41777).ok();
    }

    // /bin/busybox (the actual binary marker — code is already in memory).
    //
    // CHROMIUM-PHASE-B: used to panic here via `.unwrap()` because the
    // slice-of-byte-literals `let dirs: &[&[u8]] = &[b"bin", ...]` was
    // miscompiling — create_node was getting garbage names. Fixed by
    // hoisting DIRS to a `const`. Kept the graceful `match` arm as a
    // defense in depth: if some future VFS-layout bug comes back, the
    // caves go degraded rather than panic the whole kernel.
    let bin = match find_child(0, b"bin") {
        Some(b) => b,
        None => {
            uart::puts("  [vfs] FATAL: /bin missing — skipping applet wiring\n");
            populate_lib_from_archive();
            return;
        }
    };
    create_node(bin, b"busybox", NodeType::File, 0o100755).ok();

    // /bin/hello — standalone test binary (not a busybox applet)
    create_node(bin, b"hello", NodeType::File, 0o100755).ok();

    // Busybox applet symlinks → /bin/busybox.
    //
    // CHROMIUM-PHASE-B: same treatment as DIRS above — hoisted the
    // list to a `const` so the slice-of-byte-literals sits in .rodata
    // instead of being rebuilt on the stack per-call.
    const APPLETS: &[&[u8]] = &[
        b"sh", b"ash", b"ls", b"cat", b"echo", b"pwd", b"cd", b"mkdir", b"rmdir",
        b"rm", b"cp", b"mv", b"ln", b"chmod", b"chown", b"touch", b"head", b"tail",
        b"wc", b"sort", b"uniq", b"tr", b"cut", b"grep", b"egrep", b"fgrep",
        b"sed", b"awk", b"find", b"xargs", b"env", b"expr", b"test", b"[",
        b"printf", b"date", b"sleep", b"true", b"false", b"yes",
        b"uname", b"id", b"whoami", b"hostname", b"arch", b"logname",
        b"ps", b"kill", b"killall", b"top", b"free", b"uptime", b"df", b"du",
        b"mount", b"umount", b"dmesg",
        b"ifconfig", b"ip", b"ping", b"netstat", b"wget", b"nc", b"nslookup",
        b"tar", b"gzip", b"gunzip", b"bzip2", b"bunzip2", b"unzip", b"zcat",
        b"vi", b"less", b"more", b"diff", b"patch", b"strings", b"hexdump",
        b"md5sum", b"sha1sum", b"sha256sum", b"sha512sum",
        b"adduser", b"addgroup", b"passwd", b"su", b"login",
        b"clear", b"reset", b"tty", b"stty", b"nproc",
    ];

    for &applet in APPLETS {
        create_symlink(bin, applet, b"/bin/busybox").ok();
    }

    // Also populate /usr/bin with symlinks
    if let Some(usr_bin) = find_child(find_child(0, b"usr").unwrap_or(0), b"bin") {
        for &applet in APPLETS {
            create_symlink(usr_bin, applet, b"/bin/busybox").ok();
        }
    }

    // /lib/*.so backed by the BATARCH archive — so ld-linux's openat()
    // probing (e.g. openat("/lib/libc.so.6")) hits a real file instead
    // of ENOENT. Each node's data_addr points directly into the initrd
    // region (identity-mapped in EL1), so the normal read() path works
    // with zero copy. No-op if no archive is present.
    populate_lib_from_archive();
}

/// Register every `lib/*` entry in the BATARCH initrd as a VFS file
/// under /lib so ld-linux's openat("/lib/libc.so.6") can find it.
// /
/// Called from two places: (1) populate_rootfs() during normal VFS init,
/// and (2) the Chromium runner right before execve of ld-linux (as a
/// safety net — populate_rootfs has a pre-existing panic path on the
/// `find_child(0, b"bin").unwrap()` line if it races with something,
/// and we don't want to lose the /lib files when that fires).
// /
/// The node's `data_addr` is the archive-memory pointer, NOT an
/// allocator-owned frame. destroy_cave_vfs() must therefore skip these
/// nodes during its free-frame walk — otherwise we'd try to free bytes
/// the initrd owns. We mark them with a distinct mode bit to tell them
/// apart from real files: 0o100444 (read-only regular file, world-readable).
// /
/// Idempotent: calling twice is a no-op (find_child skips duplicates).
pub fn populate_lib_from_archive() {
    use crate::kernel::mm::initrd;
    if !initrd::is_archive() { return; }

    // /lib holds the DT_NEEDED libs ld-linux opens at runtime.
    // /bin/{icudtl.dat, hello.html, ...} are data files Chromium
    // needs — we also route those from archive entries starting
    // with `bin/` (minus `bin/content_shell`, which is loaded by
    // the ELF loader directly and doesn't need a VFS node).
    let lib_dir = match find_child(0, b"lib") {
        Some(i) => i,
        None => match create_node(0, b"lib", NodeType::Directory, 0o40755) {
            Ok(i) => i,
            Err(_) => { uart::puts("  [vfs] /lib create failed\n"); return; }
        },
    };
    let bin_dir = find_child(0, b"bin");

    // /usr/share/fonts directory for font archive entries
    let fonts_dir = {
        let usr = find_child(0, b"usr");
        let share = usr.and_then(|u| find_child(u, b"share"));
        share.and_then(|s| find_child(s, b"fonts"))
    };

    let mut added_lib: usize = 0;
    let mut added_bin: usize = 0;
    let mut added_fonts: usize = 0;
    initrd::archive_for_each(|name, _sz| {
        // Route archive entries to /lib, /bin, or /share/fonts based on prefix.
        // SKIP `bin/content_shell` — the ELF loader owns it and a
        // VFS node would mask the busybox marker (symbolic, not a
        // real backing) that populate_rootfs already made.
        let (parent_opt, leaf_bytes, prefix_len, category) =
            if name.starts_with("lib/") {
                (Some(lib_dir), name.as_bytes(), 4, 0u8) // 0=lib
            } else if name == "bin/content_shell" {
                // The ELF loader owns bin/content_shell. Putting a VFS
                // node for it would mask the busybox placeholder that
                // populate_rootfs already created under the same name.
                return;
            } else if name.starts_with("bin/") {
                (bin_dir, name.as_bytes(), 4, 1u8) // 1=bin
            } else if name.starts_with("share/fonts/") {
                (fonts_dir, name.as_bytes(), 12, 2u8) // 2=fonts
            } else {
                return;
            };
        let parent = match parent_opt { Some(p) => p, None => return };
        let leaf = &leaf_bytes[prefix_len..];
        if leaf.is_empty() || leaf.len() > NAME_LEN { return; }
        let bytes = match initrd::archive_file(name) {
            Some(b) => b,
            None => return,
        };
        if find_child(parent, leaf).is_some() { return; }
        let idx = match create_node(parent, leaf, NodeType::File, 0o100444) {
            Ok(i) => i,
            Err(_) => return,
        };
        unsafe {
            let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
            let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
            n.data_addr = bytes.as_ptr() as usize;
            n.size = bytes.len();
        }
        match category {
            0 => added_lib += 1,
            1 => added_bin += 1,
            _ => added_fonts += 1,
        }
    });

    if added_lib > 0 {
        uart::puts("  [vfs] /lib populated with ");
        crate::kernel::mm::print_num(added_lib);
        uart::puts(" archive file(s)\n");
    }
    if added_bin > 0 {
        uart::puts("  [vfs] /bin populated with ");
        crate::kernel::mm::print_num(added_bin);
        uart::puts(" archive file(s)\n");
    }
    if added_fonts > 0 {
        uart::puts("  [vfs] /usr/share/fonts populated with ");
        crate::kernel::mm::print_num(added_fonts);
        uart::puts(" archive file(s)\n");
    }
}
