#![allow(dead_code)]
// Bat_OS — BatCave Core
// Isolated container runtime for running Kali Linux tools.
// Each BatCave has its own encrypted filesystem, capabilities, and process space.

use crate::crypto::sha256;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

pub const MAX_CAVES: usize = 32;
pub const MAX_NAME: usize = 32;
pub const MAX_TOOLS: usize = 32;
pub const MAX_TOOL_NAME: usize = 32;
pub const MAX_CAPS: usize = 16;
pub const MAX_CAP_NAME: usize = 48;

#[derive(Clone, Copy, PartialEq)]
pub enum CaveState {
    Free,
    Stopped,
    Running,
    Destroyed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CaveType {
    Persistent,
    Ephemeral,
}

#[derive(Clone, Copy)]
pub struct CaveCap {
    pub active: bool,
    pub name: [u8; MAX_CAP_NAME],
    pub name_len: usize,
}

impl CaveCap {
    pub const fn empty() -> Self {
        Self { active: false, name: [0; MAX_CAP_NAME], name_len: 0 }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }
}

#[derive(Clone, Copy)]
pub struct CaveTool {
    pub installed: bool,
    pub name: [u8; MAX_TOOL_NAME],
    pub name_len: usize,
}

impl CaveTool {
    pub const fn empty() -> Self {
        Self { installed: false, name: [0; MAX_TOOL_NAME], name_len: 0 }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }
}

pub struct BatCave {
    pub state: CaveState,
    pub cave_type: CaveType,
    pub name: [u8; MAX_NAME],
    pub name_len: usize,
    pub tools: [CaveTool; MAX_TOOLS],
    pub tool_count: usize,
    pub caps: [CaveCap; MAX_CAPS],
    pub cap_count: usize,
    pub fs_key: [u8; 32],  // Per-cave encryption key
    // Display sandbox: dedicated framebuffer region
    pub display_x: u32,     // top-left corner in framebuffer
    pub display_y: u32,
    pub display_w: u32,     // size of allocated region
    pub display_h: u32,
}

impl BatCave {
    pub const fn empty() -> Self {
        Self {
            state: CaveState::Free,
            cave_type: CaveType::Persistent,
            name: [0; MAX_NAME],
            name_len: 0,
            tools: [CaveTool::empty(); MAX_TOOLS],
            tool_count: 0,
            caps: [CaveCap::empty(); MAX_CAPS],
            cap_count: 0,
            fs_key: [0; 32],
            display_x: 0,
            display_y: 0,
            display_w: 0,
            display_h: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    pub fn is_ephemeral(&self) -> bool {
        self.cave_type == CaveType::Ephemeral
    }

    pub fn has_cap(&self, cap_name: &str) -> bool {
        for i in 0..self.cap_count {
            if !self.caps[i].active { continue; }
            let cap = self.caps[i].name_str();
            // Exact match (net, raw, display)
            if cap == cap_name { return true; }
            // Path-scoped fs capability: "fs:/tmp" grants access to /tmp/*
            if cap.starts_with("fs:") && cap_name.starts_with("fs:") {
                let granted_path = &cap[3..];
                let requested_path = &cap_name[3..];
                if requested_path.starts_with(granted_path) { return true; }
            }
            // IPC capability: "ipc:recon" grants IPC to cave named "recon"
            if cap.starts_with("ipc:") && cap_name.starts_with("ipc:") {
                if cap == cap_name { return true; }
            }
        }
        false
    }

    /// Check if this cave has fs access to a specific path.
    pub fn can_access_path(&self, path: &str) -> bool {
        // "fs" (no path) = full access
        if self.has_cap("fs") { return true; }
        // Check scoped fs caps
        let mut check = [0u8; 64];
        let prefix = b"fs:";
        check[..3].copy_from_slice(prefix);
        let plen = path.len().min(61);
        check[3..3+plen].copy_from_slice(&path.as_bytes()[..plen]);
        let check_str = unsafe { core::str::from_utf8_unchecked(&check[..3+plen]) };
        self.has_cap(check_str)
    }
}

// Global registry
static mut CAVES: [BatCave; MAX_CAVES] = {
    const EMPTY: BatCave = BatCave::empty();
    [EMPTY; MAX_CAVES]
};

static CAVE_COUNT: AtomicU8 = AtomicU8::new(0);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Track which BatCave is currently active (for syscall capability checks)
static ACTIVE_CAVE_ID: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(usize::MAX);

/// Set the active cave (called when entering a cave).
pub fn set_active(id: usize) {
    ACTIVE_CAVE_ID.store(id, Ordering::Relaxed);
}

/// Get the active cave ID (usize::MAX = none active).
pub fn get_active() -> usize {
    ACTIVE_CAVE_ID.load(Ordering::Relaxed)
}

/// Check if the active cave can access a filesystem path.
pub fn active_can_access_path(path: &str) -> bool {
    let id = get_active();
    if id == usize::MAX || id >= MAX_CAVES { return false; }
    unsafe { CAVES[id].can_access_path(path) }
}

/// Check if the active cave has a specific capability.
pub fn active_has_cap(cap: &str) -> bool {
    let id = get_active();
    if id == usize::MAX || id >= MAX_CAVES { return false; }
    unsafe {
        let cave = &CAVES[id];
        cave.state != CaveState::Free && cave.has_cap(cap)
    }
}

pub fn init() {
    INITIALIZED.store(true, Ordering::Relaxed);
}

/// Create a new BatCave.
pub fn create(name: &str, ephemeral: bool) -> Result<usize, &'static str> {
    if name.len() > MAX_NAME { return Err("name too long"); }

    unsafe {
        // Check duplicate
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free && CAVES[i].name_str() == name {
                return Err("BatCave already exists");
            }
        }

        // Find free slot
        let slot = (0..MAX_CAVES)
            .find(|&i| CAVES[i].state == CaveState::Free)
            .ok_or("max BatCaves reached")?;

        let cave = &mut CAVES[slot];
        cave.state = CaveState::Stopped;
        cave.cave_type = if ephemeral { CaveType::Ephemeral } else { CaveType::Persistent };
        cave.name[..name.len()].copy_from_slice(name.as_bytes());
        cave.name_len = name.len();
        cave.tool_count = 0;
        cave.cap_count = 0;

        // Derive per-cave encryption key from name
        let master: [u8; 32] = [
            0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
            0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
            0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
            0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
        ];
        cave.fs_key = sha256::derive_key(
            &master,
            name.as_bytes(),
        );

        let count = CAVE_COUNT.load(Ordering::Relaxed);
        CAVE_COUNT.store(count + 1, Ordering::Relaxed);

        Ok(slot)
    }
}

/// Install a tool into a BatCave.
pub fn install_tool(name: &str, tool: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;

    if cave.tool_count >= MAX_TOOLS {
        return Err("max tools reached");
    }

    // Check if already installed
    for i in 0..cave.tool_count {
        if cave.tools[i].installed && cave.tools[i].name_str() == tool {
            return Err("tool already installed");
        }
    }

    let t = &mut cave.tools[cave.tool_count];
    t.installed = true;
    let len = tool.len().min(MAX_TOOL_NAME);
    t.name[..len].copy_from_slice(&tool.as_bytes()[..len]);
    t.name_len = len;
    cave.tool_count += 1;

    Ok(())
}

/// Grant a capability to a BatCave.
pub fn grant_cap(name: &str, cap: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;

    if cave.cap_count >= MAX_CAPS {
        return Err("max capabilities reached");
    }

    // Check if already granted
    if cave.has_cap(cap) {
        return Err("capability already granted");
    }

    let c = &mut cave.caps[cave.cap_count];
    c.active = true;
    let len = cap.len().min(MAX_CAP_NAME);
    c.name[..len].copy_from_slice(&cap.as_bytes()[..len]);
    c.name_len = len;
    cave.cap_count += 1;

    Ok(())
}

/// Revoke a capability from a BatCave.
pub fn revoke_cap(name: &str, cap: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;

    for i in 0..cave.cap_count {
        if cave.caps[i].active && cave.caps[i].name_str() == cap {
            cave.caps[i].active = false;
            return Ok(());
        }
    }

    Err("capability not found")
}

// ─── Inter-BatCave IPC ───

/// IPC channel mapping between caves
const MAX_CAVE_IPC: usize = 16;
static mut CAVE_IPC: [(usize, usize, u64); MAX_CAVE_IPC] = [(usize::MAX, usize::MAX, 0); MAX_CAVE_IPC];

/// Create an IPC channel between two BatCaves.
/// Both caves must have `ipc:<other_name>` capability.
pub fn create_ipc(cave_a: &str, cave_b: &str) -> Result<u64, &'static str> {
    let id_a = find_id(cave_a).ok_or("cave A not found")?;
    let id_b = find_id(cave_b).ok_or("cave B not found")?;

    // Check capabilities
    unsafe {
        let mut cap_check = [0u8; 48];
        // A needs ipc:<B>
        let b_len = cave_b.len().min(44);
        cap_check[..4].copy_from_slice(b"ipc:");
        cap_check[4..4+b_len].copy_from_slice(&cave_b.as_bytes()[..b_len]);
        let cap_b = core::str::from_utf8_unchecked(&cap_check[..4+b_len]);
        if !CAVES[id_a].has_cap(cap_b) { return Err("A lacks ipc cap"); }

        // B needs ipc:<A>
        let a_len = cave_a.len().min(44);
        cap_check[4..4+a_len].copy_from_slice(&cave_a.as_bytes()[..a_len]);
        let cap_a = core::str::from_utf8_unchecked(&cap_check[..4+a_len]);
        if !CAVES[id_b].has_cap(cap_a) { return Err("B lacks ipc cap"); }
    }

    // Create kernel IPC channel
    let channel = crate::kernel::ipc::create_channel().ok_or("no free channels")?;

    // Store mapping
    unsafe {
        for i in 0..MAX_CAVE_IPC {
            if CAVE_IPC[i].0 == usize::MAX {
                CAVE_IPC[i] = (id_a, id_b, channel);
                return Ok(channel);
            }
        }
    }
    Err("max IPC channels")
}

/// Get the IPC channel between the active cave and another cave.
pub fn get_ipc_channel(other_name: &str) -> Option<u64> {
    let active = get_active();
    let other = find_id(other_name)?;
    unsafe {
        for i in 0..MAX_CAVE_IPC {
            let (a, b, ch) = CAVE_IPC[i];
            if (a == active && b == other) || (a == other && b == active) {
                return Some(ch);
            }
        }
    }
    None
}

/// Allocate a display sandbox region for a cave.
/// Each cave gets a non-overlapping framebuffer rectangle.
pub fn alloc_display(name: &str, x: u32, y: u32, w: u32, h: u32) -> Result<(), &'static str> {
    let cave = find_mut(name)?;
    cave.display_x = x;
    cave.display_y = y;
    cave.display_w = w;
    cave.display_h = h;
    Ok(())
}

/// Check if a pixel coordinate is within the active cave's display sandbox.
pub fn display_check(x: u32, y: u32) -> bool {
    let id = get_active();
    if id == usize::MAX || id >= MAX_CAVES { return true; } // no cave active → allow (kernel)
    unsafe {
        let cave = &CAVES[id];
        if cave.display_w == 0 || !cave.has_cap("display") { return false; } // no display cap
        x >= cave.display_x && x < cave.display_x + cave.display_w &&
        y >= cave.display_y && y < cave.display_y + cave.display_h
    }
}

/// Start a BatCave.
pub fn start(name: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;
    if cave.state == CaveState::Running {
        return Err("already running");
    }
    cave.state = CaveState::Running;
    Ok(())
}

/// Stop a BatCave.
pub fn stop(name: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;
    cave.state = CaveState::Stopped;
    Ok(())
}

/// Enter a BatCave (start + set up isolated VFS + mark active).
///
/// V5-XLAYER-001/002/003 fix: wipe all per-cave static state on every
/// cave switch (not only on destroy). Previously the signal handler
/// table, TLS key state, and fd table persisted across caves — one
/// cave could install a handler, exit, and a later cave would inherit
/// it. Same for TLS session keys on PCBs the previous tenant opened.
pub fn enter(name: &str) -> Result<(), &'static str> {
    start(name)?;
    if let Some(id) = find_id(name) {
        // V8-ROOT-1 fix: the entire park→reset→activate sequence is a
        // single critical section. V6's deferred-preempt scheduler could
        // fire a timer IRQ between any two steps here, letting another
        // thread observe partially-reset tables (xlayer-D: pointer
        // validated against dying cave's VA) or a sentinel `active==MAX`
        // state where quota charges silently no-op.
        //
        // vfs::init_for_cave is the ONLY call inside the CS that might
        // allocate (heap). That's fine because the heap allocator itself
        // masks DAIF.I (V6-TOCTOU-007) and IrqGuard is nestable.
        let prev_active = get_active();
        crate::critical_section! {
            if prev_active != usize::MAX {
                set_active(usize::MAX);
            }
            crate::batcave::linux::syscall::reset_cave_statics();
            crate::net::tls::reset_all_sessions();
            crate::batcave::linux::fd::reset_for_cave_switch();
            crate::batcave::linux::sockets::reset_for_cave_switch();
            crate::net::tcp::reset_for_cave_switch();
            set_active(id);
            crate::batcave::linux::vfs::init_for_cave(id);
        }
        let _ = prev_active;
    }
    Ok(())
}

/// Find a cave index by name.
pub fn find_id(name: &str) -> Option<usize> {
    unsafe {
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free && CAVES[i].name_str() == name {
                return Some(i);
            }
        }
    }
    None
}

/// Seal a persistent BatCave to ephemeral (one-way, irreversible).
pub fn seal(name: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;

    if cave.cave_type == CaveType::Ephemeral {
        return Err("already ephemeral");
    }

    cave.cave_type = CaveType::Ephemeral;
    Ok(())
}

/// Destroy a BatCave — secure wipe.
///
/// V2-NEW-009/019/031/032/033 + ESC-029/033: clear every `static mut`
/// piece of per-cave state that survived into the next cave. Previously
/// SIGNAL_HANDLERS (128 × u64 of attacker-controllable handler addresses),
/// CLONE_CHILD_STACK, IN_CHILD, IS_THREAD_CHILD, LAST_CHILD_TID, PIPE_BUF,
/// UDP RX queue, SAVED_FRAME, SAVED_STACK all carried over to the next
/// cave — a cheap cross-cave info-leak and gadget-plant primitive.
pub fn destroy(name: &str) -> Result<(), &'static str> {
    // V6-CHAIN-001 fix: capture the cave id BEFORE wiping state, since
    // find_id() filters out Free caves and the original V5 code did
    // the lookup AFTER setting state=Free → quotas::reset was dead.
    let cave_id_for_reset = find_id(name);

    // V8-ROOT-1 fix: the entire destroy sequence — active-deactivate,
    // VFS tear-down, fs_key zero, tool/cap clear, state=Free, stats
    // decrement, static-mut reset, quota reset — is one critical
    // section. IRQ audit #2: if a thread in the destroyed cave resumes
    // mid-destroy it sees `fs_key=[0;32]` and encrypts/decrypts with
    // a zero key (AES-GCM keystream = AES(0, counter) — deterministic,
    // recoverable).
    let _irq_guard = crate::kernel::sync::IrqGuard::new();

    // Wipe the cave's VFS instance (filesystem data)
    if let Some(id) = cave_id_for_reset {
        crate::batcave::linux::vfs::destroy_cave_vfs(id);
        // If this was the active cave, clear active
        if get_active() == id {
            set_active(usize::MAX);
        }
    }

    let cave = find_mut(name)?;

    // Zero the encryption key — data is now unrecoverable
    cave.fs_key = [0; 32];

    // Clear all tools
    for i in 0..cave.tool_count {
        cave.tools[i] = CaveTool::empty();
    }
    cave.tool_count = 0;

    // Clear all caps
    for i in 0..cave.cap_count {
        cave.caps[i] = CaveCap::empty();
    }
    cave.cap_count = 0;

    cave.state = CaveState::Free;
    cave.name_len = 0;

    let count = CAVE_COUNT.load(Ordering::Relaxed);
    if count > 0 { CAVE_COUNT.store(count - 1, Ordering::Relaxed); }

    // V2-NEW-009+: clear the cross-cave static globals in the Linux
    // compat layer so the next cave starts with a clean state.
    crate::batcave::linux::syscall::reset_cave_statics();

    // V5-CHAIN-004 + V6-CHAIN-001 fix: use the id we captured at the
    // top, not a fresh find_id (which now returns None because we
    // just set state=Free above).
    if let Some(id) = cave_id_for_reset {
        crate::batcave::linux::quotas::reset(id);
    }

    Ok(())
}

/// Destroy ALL BatCaves — called by wipe system.
pub fn destroy_all() {
    unsafe {
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free {
                CAVES[i].fs_key = [0; 32];
                CAVES[i].state = CaveState::Free;
                CAVES[i].tool_count = 0;
                CAVES[i].cap_count = 0;
                CAVES[i].name_len = 0;
            }
        }
    }
    CAVE_COUNT.store(0, Ordering::Relaxed);
}

/// List all active BatCaves.
pub fn list<F: FnMut(&BatCave)>(mut callback: F) {
    unsafe {
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free {
                callback(&CAVES[i]);
            }
        }
    }
}

pub fn count() -> usize {
    CAVE_COUNT.load(Ordering::Relaxed) as usize
}

pub fn state_str(state: CaveState) -> &'static str {
    match state {
        CaveState::Free => "FREE",
        CaveState::Stopped => "STOPPED",
        CaveState::Running => "RUNNING",
        CaveState::Destroyed => "DESTROYED",
    }
}

pub fn type_str(t: CaveType) -> &'static str {
    match t {
        CaveType::Persistent => "persistent",
        CaveType::Ephemeral => "ephemeral",
    }
}

fn find_mut(name: &str) -> Result<&'static mut BatCave, &'static str> {
    unsafe {
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free && CAVES[i].name_str() == name {
                return Ok(&mut CAVES[i]);
            }
        }
    }
    Err("BatCave not found")
}
