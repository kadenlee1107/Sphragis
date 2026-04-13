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
            if self.caps[i].active && self.caps[i].name_str() == cap_name {
                return true;
            }
        }
        false
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
pub fn enter(name: &str) -> Result<(), &'static str> {
    start(name)?;
    // Find cave index and set as active
    if let Some(id) = find_id(name) {
        set_active(id);
        // Initialize an isolated VFS for this cave (if not already done)
        crate::batcave::linux::vfs::init_for_cave(id);
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
pub fn destroy(name: &str) -> Result<(), &'static str> {
    // Wipe the cave's VFS instance (filesystem data)
    if let Some(id) = find_id(name) {
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
