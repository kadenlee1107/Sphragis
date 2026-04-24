// Bat_OS — In-Memory Virtual Filesystem for BatCave Containers
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
    Socket,     // network socket (TCP/UDP)
    ChromiumFb, // /batos/fb0 — Chromium↔kernel display shared-memory region
}

#[derive(Clone, Copy)]
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

// Helper: get the active VFS instance
fn active() -> &'static VfsInstance {
    unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        &(*core::ptr::addr_of!(INSTANCES))[idx]
    }
}
fn active_mut() -> &'static mut VfsInstance {
    unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        &mut (*core::ptr::addr_of_mut!(INSTANCES))[idx]
    }
}

// Compatibility: access nodes through active instance
macro_rules! NODES {
    () => { unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        &(*core::ptr::addr_of!(INSTANCES))[idx].nodes
    } };
}
macro_rules! NODES_MUT {
    () => { unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        &mut (*core::ptr::addr_of_mut!(INSTANCES))[idx].nodes
    } };
}

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

/// Switch to a specific cave's VFS instance.
pub fn switch_to_cave(cave_id: usize) {
    unsafe {
        let instances = &*core::ptr::addr_of!(INSTANCES);
        for i in 0..MAX_VFS_INSTANCES {
            if instances[i].cave_id == cave_id && instances[i].ready {
                core::ptr::write_volatile(core::ptr::addr_of_mut!(ACTIVE_INSTANCE), i);
                return;
            }
        }
    }
}

/// Destroy a cave's VFS instance (wipe all data).
///
/// V11-FRESH-EYES: previously this only zeroed the node-metadata structs,
/// leaking **physical frames** that each node's `data_addr` referenced.
/// Two compounding bugs:
///   (1) plaintext file contents persisted in those frames forever
///   (because they were never returned to the frame allocator), and
///   (2) the frame bitmap bits stayed set, yielding an unprivileged
///   OOM-DoS primitive — any cave with the `mem` cap could exhaust
///   physical memory by cycling create-write-destroy.
/// Now we scrub and free each file-data frame before clearing the node.
pub fn destroy_cave_vfs(cave_id: usize) {
    unsafe {
        let instances = &mut *core::ptr::addr_of_mut!(INSTANCES);
        for i in 0..MAX_VFS_INSTANCES {
            if instances[i].cave_id == cave_id {
                // V11 fix: walk every live node, zero its backing frame,
                // then return it to the allocator.
                //
                // V12 REGRESSION-FIX: skip `ChromiumFb` nodes here.
                // `/batos/fb0` is a special shared 5 MiB contiguous
                // allocation whose lifetime is managed by the
                // chromium_blit subsystem (reset_for_cave_switch zeros
                // the region). Blindly calling `free_frame` on it would
                // only free page 0 (leaking the other 1280 pages and
                // desynchronizing the bitmap), AND the 4 KiB zero-loop
                // below would leak ~5238 KiB of the region.
                // V12b: also skip nodes whose data_addr points into the
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
                        && n.node_type != NodeType::ChromiumFb
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
        for i in 0..name.len() {
            n.name[i] = name[i];
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

// ─── Rootfs Population ───

fn populate_rootfs() {
    // Create standard directories
    let dirs: &[&[u8]] = &[
        b"bin", b"etc", b"usr", b"tmp", b"dev", b"proc", b"root", b"var", b"sbin",
    ];
    for &name in dirs {
        let mode = if name == b"tmp" { 0o41777 } else { 0o40755 };
        create_node(0, name, NodeType::Directory, mode).ok();
    }

    // /usr/bin
    if let Some(usr) = find_child(0, b"usr") {
        create_node(usr, b"bin", NodeType::Directory, 0o40755).ok();
        create_node(usr, b"sbin", NodeType::Directory, 0o40755).ok();
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
    }

    // /batos — Bat_OS-native namespace (Chromium display bridge, etc.)
    // Created unconditionally; missing backing memory turns the node into
    // an inert file (read/write/mmap will all return EIO / EINVAL).
    if create_node(0, b"batos", NodeType::Directory, 0o40755).is_ok() {
        if let Some(batos) = find_child(0, b"batos") {
            create_chromium_fb(batos, b"fb0");
        }
    }

    // /bin/busybox (the actual binary marker — code is already in memory)
    let bin = find_child(0, b"bin").unwrap();
    create_node(bin, b"busybox", NodeType::File, 0o100755).ok();

    // /bin/hello — standalone test binary (not a busybox applet)
    create_node(bin, b"hello", NodeType::File, 0o100755).ok();

    // Busybox applet symlinks → /bin/busybox
    let applets: &[&[u8]] = &[
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

    for &applet in applets {
        create_symlink(bin, applet, b"/bin/busybox").ok();
    }

    // Also populate /usr/bin with symlinks
    if let Some(usr_bin) = find_child(find_child(0, b"usr").unwrap_or(0), b"bin") {
        for &applet in applets {
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
///
/// Called from two places: (1) populate_rootfs() during normal VFS init,
/// and (2) the Chromium runner right before execve of ld-linux (as a
/// safety net — populate_rootfs has a pre-existing panic path on the
/// `find_child(0, b"bin").unwrap()` line if it races with something,
/// and we don't want to lose the /lib files when that fires).
///
/// The node's `data_addr` is the archive-memory pointer, NOT an
/// allocator-owned frame. destroy_cave_vfs() must therefore skip these
/// nodes during its free-frame walk — otherwise we'd try to free bytes
/// the initrd owns. We mark them with a distinct mode bit to tell them
/// apart from real files: 0o100444 (read-only regular file, world-readable).
///
/// Idempotent: calling twice is a no-op (find_child skips duplicates).
pub fn populate_lib_from_archive() {
    use crate::kernel::mm::initrd;
    uart::puts("  [vfs] populate_lib_from_archive: entry\n");
    if !initrd::is_archive() {
        uart::puts("  [vfs] populate_lib_from_archive: not an archive\n");
        return;
    }

    // /lib must exist to park files under. populate_rootfs() doesn't
    // create it by default (busybox uses /bin), so create it now.
    let lib_dir = match find_child(0, b"lib") {
        Some(i) => i,
        None => match create_node(0, b"lib", NodeType::Directory, 0o40755) {
            Ok(i) => i,
            Err(_) => {
                uart::puts("  [vfs] /lib create failed\n");
                return;
            }
        },
    };

    let mut added: usize = 0;
    initrd::archive_for_each(|name, _sz| {
        if !name.starts_with("lib/") { return; }
        let leaf = &name.as_bytes()[4..]; // strip "lib/"
        if leaf.is_empty() || leaf.len() > NAME_LEN { return; }
        let bytes = match initrd::archive_file(name) {
            Some(b) => b,
            None => return,
        };
        // Dodge duplicate work if this VFS instance was already populated
        // (init_for_cave can be called twice for the same cave slot).
        if find_child(lib_dir, leaf).is_some() { return; }
        let idx = match create_node(lib_dir, leaf, NodeType::File, 0o100444) {
            Ok(i) => i,
            Err(_) => return,
        };
        unsafe {
            let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
            let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
            n.data_addr = bytes.as_ptr() as usize;
            n.size = bytes.len();
        }
        added += 1;
    });

    if added > 0 {
        uart::puts("  [vfs] /lib populated with ");
        crate::kernel::mm::print_num(added);
        uart::puts(" archive file(s)\n");
    }
}

// ───────────────────────── Chromium framebuffer bridge ─────────────────────────
//
// /batos/fb0 — a single well-known shared-memory region that Chromium's patched
// Ozone headless backend maps via mmap(MAP_SHARED). Our chromium_blit kthread
// reads from the same physical pages and blits to the virtio-gpu scanout.
//
// Layout:  [ 128-byte BatosFbHeader ] [ width * stride bytes of BGRA pixels ]
// Size:    FB_REGION_SIZE = 128 + 1280*1024*4 = 5_242_880 + 128 bytes
//
// Contract: ports/chromium_port/PHASE5_DISPLAY.md §4.

pub const FB_MAGIC: u32 = 0x4246_4231; // 'BFB1' — keep in sync with chromium_blit.rs
pub const FB_HEADER_SIZE: usize = 128;
pub const FB_WIDTH: u32 = 1280;
pub const FB_HEIGHT: u32 = 1024;
pub const FB_STRIDE: u32 = FB_WIDTH * 4;
pub const FB_PIXEL_BYTES: usize = (FB_WIDTH * FB_HEIGHT * 4) as usize;
pub const FB_REGION_SIZE: usize = FB_HEADER_SIZE + FB_PIXEL_BYTES;
pub const FB_FORMAT_BGRA8888: u32 = 1;

// Base physical address of the /batos/fb0 region (0 = not yet allocated).
static mut FB_REGION_BASE: usize = 0;
static mut FB_NODE_IDX: u16 = 0;

/// Returns (base, size) of the /batos/fb0 shared region, or (0, 0) if absent.
pub fn chromium_fb_region() -> (usize, usize) {
    unsafe {
        let base = core::ptr::read_volatile(core::ptr::addr_of!(FB_REGION_BASE));
        if base == 0 {
            (0, 0)
        } else {
            (base, FB_REGION_SIZE)
        }
    }
}

/// Returns the VFS node index of /batos/fb0, or None.
pub fn chromium_fb_node() -> Option<u16> {
    unsafe {
        let idx = core::ptr::read_volatile(core::ptr::addr_of!(FB_NODE_IDX));
        if idx == 0 { None } else { Some(idx) }
    }
}

/// Returns true if `idx` is the /batos/fb0 ChromiumFb node.
pub fn is_chromium_fb(idx: u16) -> bool {
    match chromium_fb_node() {
        Some(fb_idx) => fb_idx == idx,
        None => false,
    }
}

/// Allocate backing pages for /batos/fb0 and register it as a ChromiumFb node.
fn create_chromium_fb(parent: u16, name: &[u8]) {
    // Allocate FB_REGION_SIZE bytes of contiguous physical memory.
    // The frame allocator hands out sequential pages on a fresh boot,
    // so repeated alloc_frame() calls yield a contiguous run.
    let pages = (FB_REGION_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
    let base = match frame::alloc_frame() {
        Some(b) => b,
        None => {
            uart::puts("  [vfs] /batos/fb0: no memory for 5 MiB region\n");
            return;
        }
    };
    let mut last = base;
    let mut contiguous = true;
    for _ in 1..pages {
        match frame::alloc_frame() {
            Some(addr) => {
                if addr != last + PAGE_SIZE { contiguous = false; }
                last = addr;
            }
            None => {
                uart::puts("  [vfs] /batos/fb0: partial allocation, aborting\n");
                return;
            }
        }
    }
    if !contiguous {
        uart::puts("  [vfs] /batos/fb0: WARN region not contiguous\n");
        // Fall through — best-effort; likely benign on fresh boot.
    }

    // Zero the whole region, then stamp the header.
    unsafe {
        for i in 0..FB_REGION_SIZE {
            core::ptr::write_volatile((base + i) as *mut u8, 0);
        }
        // Header field layout (keep in sync with chromium_blit::FbHeader):
        //   u32 magic         @ 0
        //   u32 version       @ 4
        //   u32 width         @ 8
        //   u32 height        @ 12
        //   u32 stride        @ 16
        //   u32 format        @ 20
        //   u32 seq           @ 24   (producer: Chromium, consumer: kernel)
        //   u32 last_seen_seq @ 28   (kernel's ack for user-space diagnostics)
        //   u32 damage_x      @ 32
        //   u32 damage_y      @ 36
        //   u32 damage_w      @ 40
        //   u32 damage_h      @ 44
        //   u64 pts_ns        @ 48
        //   u32 reserved[8]   @ 56..88
        core::ptr::write_volatile((base + 0) as *mut u32, FB_MAGIC);
        core::ptr::write_volatile((base + 4) as *mut u32, 1); // version
        core::ptr::write_volatile((base + 8) as *mut u32, FB_WIDTH);
        core::ptr::write_volatile((base + 12) as *mut u32, FB_HEIGHT);
        core::ptr::write_volatile((base + 16) as *mut u32, FB_STRIDE);
        core::ptr::write_volatile((base + 20) as *mut u32, FB_FORMAT_BGRA8888);
    }

    // Create the VFS node. It's a ChromiumFb, not a File — so regular
    // read/write/mmap paths must special-case the type and read from
    // `data_addr` (= physical base of the region).
    let idx = match create_node(parent, name, NodeType::ChromiumFb, 0o100666) {
        Ok(i) => i,
        Err(_) => {
            uart::puts("  [vfs] /batos/fb0: node alloc failed\n");
            return;
        }
    };
    unsafe {
        let ai = core::ptr::read_volatile(core::ptr::addr_of!(ACTIVE_INSTANCE));
        let n = &mut (*core::ptr::addr_of_mut!(INSTANCES))[ai].nodes[idx as usize];
        n.data_addr = base;
        n.size = FB_REGION_SIZE;
    }

    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FB_REGION_BASE), base);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FB_NODE_IDX), idx);
    }

    uart::puts("  [vfs] /batos/fb0: 5 MiB BGRA region @ 0x");
    let hex = b"0123456789abcdef";
    for shift in (0..16).rev() {
        let nibble = ((base >> (shift * 4)) & 0xF) as usize;
        uart::putc(hex[nibble]);
    }
    uart::puts("\n");
}
