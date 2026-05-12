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

/// What kind of isolation actually backs this cave.
// /
/// NATIVE caves run user ELFs under Bat_OS's own MMU page tables via
/// the `batcave::linux` loader. DOCKER caves live as Linux containers
/// on the Mac host, orchestrated by the `batcaved` daemon over TCP
/// (see `docker_client.rs`). From the user's perspective both are
/// just BatCaves — the backing field lets `batcave list/destroy/run`
/// route to the right implementation.
#[derive(Clone, Copy, PartialEq)]
pub enum CaveBacking {
    Native,
    Docker,
}

/// Max length of the image name we store alongside a docker-backed cave
/// (e.g. `kalilinux/kali-rolling`). 64 covers the typical registry+tag.
pub const MAX_IMAGE: usize = 64;

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
    /// Which isolation primitive actually backs this cave.
    pub backing: CaveBacking,
    /// For docker-backed caves: the image reference passed to
    /// `docker run` (e.g. "kalilinux/kali-rolling"). Empty for native.
    pub image: [u8; MAX_IMAGE],
    pub image_len: usize,
    /// per-cave L1 page-table physical address. 0 means
    /// "no L1 built yet" — the cave shares the kernel's PRIMARY_L1
    /// until first `enter()`. On first enter we lazy-allocate via
    /// `mmu::setup_native_cave_l1`, then call `mmu::switch_to_cave`
    /// to install it in TTBR0_EL1. Each cave's L1 maps kernel
    /// identity but no user window (native caves have no EL0 code),
    /// so the cave-switch's `tlbi vmalle1` gives TLB-level
    /// isolation between caves even without ASIDs. Freed by
    /// `mmu::free_cave_slot` on `destroy()`.
    pub cave_l1_phys: usize,
    /// CAVE_L1[] slot index — the index into linux::mmu's
    /// MAX_CAVE_PAGETABLES=8 slot array. usize::MAX = "not assigned".
    /// Stored separately from `cave_l1_phys` so destroy() can free
    /// the slot without re-scanning to find which one we own.
    pub cave_l1_slot: usize,
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
            backing: CaveBacking::Native,
            image: [0; MAX_IMAGE],
            image_len: 0,
            cave_l1_phys: 0,             // lazy-built on first enter
            cave_l1_slot: usize::MAX,    // "not assigned"
        }
    }

    pub fn image_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.image[..self.image_len]) }
    }

    pub fn is_docker(&self) -> bool {
        matches!(self.backing, CaveBacking::Docker)
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

    /// any-fs-cap test.
    // /
    /// Returns true if the cave has either bare `fs` (full FS access)
    /// OR any path-scoped `fs:<path>` cap. The syscall dispatcher uses
    /// this for the broad FileIO category gate so a cave granted only
    /// `fs:/tmp` makes it past the dispatcher; the per-syscall path
    /// check (via `can_access_path` at openat) then enforces the
    /// path scope.
    // /
    /// Without this, a cave with only `fs:/tmp` failed the bare `fs`
    /// check and got zero FS syscalls — so path-scoped caps were
    /// purely decorative.
    pub fn has_any_fs_cap(&self) -> bool {
        for i in 0..self.cap_count {
            if !self.caps[i].active { continue; }
            let cap = self.caps[i].name_str();
            if cap == "fs" { return true; }
            if cap.starts_with("fs:") { return true; }
        }
        false
    }
}

// Global registry
//
// Phase 6: made `pub` so the shell can read per-cave backing/image
// via `cave::CAVES[id].is_docker()` when routing `batcave run/destroy`.
// All mutation still goes through the pub fns in this file.
pub static mut CAVES: [BatCave; MAX_CAVES] = {
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

/// name of the currently-active cave for UI display
/// (title-bar cave indicator). Returns "kernel" when no cave is
/// active or the slot id is out of range.
pub fn active_name_str() -> &'static str {
    let id = ACTIVE_CAVE_ID.load(Ordering::Relaxed);
    if id == usize::MAX || id >= MAX_CAVES { return "kernel"; }
    unsafe {
        let cave = &CAVES[id];
        if cave.state == CaveState::Free { return "kernel"; }
        cave.name_str()
    }
}

/// true iff the active cave has any FS-related cap
/// (bare `fs` for full access, or any `fs:<path>` for scoped). Used
/// by the syscall dispatcher's FileIO gate so caves with only
/// path-scoped caps don't get blocked at the broad-cap check before
/// the per-syscall path check ever runs.
pub fn active_has_any_fs_cap() -> bool {
    let id = get_active();
    if id == usize::MAX || id >= MAX_CAVES { return false; }
    unsafe { CAVES[id].has_any_fs_cap() }
}

/// Check if the active cave can access a filesystem path.
/// Active cave's mount-namespace prefix (gap-audit item 032).
/// Returns `<cave-name>:` for an active cave, or empty for the
/// kernel/admin context (no cave attached). Used by
/// `fs::batfs::ns_*` to scope file names per cave so two caves
/// can't see each other's filenames even though they share the
/// same BatFS storage.
pub fn active_mount_prefix(out: &mut [u8; 80]) -> usize {
    let id = get_active();
    if id == usize::MAX { return 0; }
    let name = unsafe { (*core::ptr::addr_of!(CAVES))[id].name_str() };
    let nlen = name.len().min(out.len() - 1);
    out[..nlen].copy_from_slice(&name.as_bytes()[..nlen]);
    out[nlen] = b':';
    nlen + 1
}

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

    // restore persistent caves from BatFS. Runs AFTER
    // `fs::batfs::init` (see main.rs ordering) so the filesystem is
    // already unlocked with the operator's passphrase. Each cave whose
    // manifest survives the boot is reinstalled into CAVES[] in
    // CaveState::Stopped — the operator brings it back up with
    // `batcave enter <name>` if they want to attach.
    let restored = crate::batcave::persist::restore_all();
    CAVE_COUNT.store(restored as u8, Ordering::Relaxed);
}

/// Ensure an "ambient" BatCave is active for the shell-launched ELF runner
/// paths (hello / libc / threads + the content_shell/netsurf/freetype/png/
/// v8/blink binaries that `cmd_run_elf` spawns).
// /
/// Without this, `cave::get_active()` returns `usize::MAX`, every
/// capability check fails (because `active_has_cap` can't index a cave
/// slot), and the ELF hits `EACCES` on the first `write`/`mmap`/... syscall.
// /
/// We install a single ephemeral cave named `"shell-host"` with a broad cap
/// set. This intentionally does NOT enforce isolation — it's the host
/// process equivalent on a UNIX-like system. Production-sensitive workloads
/// still create named BatCaves with narrower caps via the `batcave` shell
/// commands.
pub fn ensure_host_cave_active() {
    const HOST: &str = "shell-host";
    const HOST_CAPS: &[&str] = &["proc", "mem", "fs", "net", "raw", "display"];

    if get_active() != usize::MAX {
        return; // something else already owns the current thread
    }

    let id = match find_id(HOST) {
        Some(id) => id,
        None => match create(HOST, true) {
            Ok(id) => {
                for cap in HOST_CAPS {
                    let _ = grant_cap(HOST, cap);
                }
                id
            }
            Err(_e) => return, // out of slots — leave active = MAX, syscalls will block
        },
    };
    set_active(id);
}

/// Create a new BatCave.
pub fn create(name: &str, ephemeral: bool) -> Result<usize, &'static str> {
    if name.len() > MAX_NAME {
        // every cave-creation
        // attempt is operationally significant and worth logging — both
        // the success path AND every failure mode. Pre-fix the failure
        // paths returned silently. An attacker (or buggy script) trying
        // to flood the cave table left no breadcrumbs in the audit ring.
        crate::security::audit::record(
            crate::security::audit::Category::Cave,
            b"cave create FAILED: name too long",
        );
        return Err("name too long");
    }

    unsafe {
        // Check duplicate
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free && CAVES[i].name_str() == name {
                let mut buf = [0u8; 192];
                let mut p = 0usize;
                let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
                    let n = src.len().min(dst.len().saturating_sub(*p));
                    dst[*p..*p + n].copy_from_slice(&src[..n]);
                    *p += n;
                };
                copy(&mut buf, b"cave create FAILED: duplicate name ", &mut p);
                copy(&mut buf, name.as_bytes(), &mut p);
                crate::security::audit::record(
                    crate::security::audit::Category::Cave,
                    &buf[..p],
                );
                return Err("BatCave already exists");
            }
        }

        // Find free slot
        let slot = match (0..MAX_CAVES)
            .find(|&i| CAVES[i].state == CaveState::Free)
        {
            Some(s) => s,
            None => {
                // Cave-table saturation. This is a SECURITY-relevant
                // event: an attacker with shell access trying to DoS
                // the OS by spinning up 32 caves leaves a clear trace
                // here. Always logged (not one-shot) since the operator
                // needs to see EVERY rejected creation to understand
                // the scope of the attempt.
                crate::security::audit::record(
                    crate::security::audit::Category::Cave,
                    b"cave create FAILED: max caves reached (table full)",
                );
                crate::drivers::uart::puts("[cave] WARNING: MAX_CAVES reached - creation rejected\n");
                return Err("max BatCaves reached");
            }
        };

        let cave = &mut CAVES[slot];
        cave.state = CaveState::Stopped;
        cave.cave_type = if ephemeral { CaveType::Ephemeral } else { CaveType::Persistent };
        cave.name[..name.len()].copy_from_slice(name.as_bytes());
        cave.name_len = name.len();
        cave.tool_count = 0;
        cave.cap_count = 0;

        // pre-fix the per-cave fs_key was
        // SHA256(`0xBA7CA7E0…` constant || cave_name). Both inputs
        // were trivially recoverable — the constant is in the kernel
        // binary, the name is in `info caves`/audit logs/IPC discovery.
        // Anyone with read access to either could decrypt every cave's
        // BatFS files.
        //
        // Now: HMAC-style derivation against the boot-time BatFS
        // master key (which is itself derived from the operator's
        // passphrase via SHA256 in derive_batfs_key, with per-boot
        // entropy via the salt in main.rs). Knowing the cave name no
        // longer suffices; the attacker also needs the operator
        // passphrase. Defense-in-depth against an attacker with
        // kernel-image read access.
        cave.fs_key = sha256::derive_key(
            &crate::fs::batfs::master_key(),
            name.as_bytes(),
        );

        // New caves default to Native backing. `create_docker` upgrades
        // the backing + stores the image name after this returns.
        cave.backing = CaveBacking::Native;
        cave.image_len = 0;

        let count = CAVE_COUNT.load(Ordering::Relaxed);
        CAVE_COUNT.store(count + 1, Ordering::Relaxed);

        // success-path log.
        // Pairs with the failure-path entries above so the audit ring
        // tells a complete cave-lifecycle story.
        let mut buf = [0u8; 192];
        let mut p = 0usize;
        let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
            let n = src.len().min(dst.len().saturating_sub(*p));
            dst[*p..*p + n].copy_from_slice(&src[..n]);
            *p += n;
        };
        copy(&mut buf, b"cave create OK ", &mut p);
        copy(&mut buf, name.as_bytes(), &mut p);
        copy(&mut buf, if ephemeral { b" (ephemeral)" } else { b" (persistent)" }, &mut p);
        crate::security::audit::record(
            crate::security::audit::Category::Cave,
            &buf[..p],
        );

        // write the cave manifest to BatFS so the registry
        // entry survives reboot. No-op for Ephemeral caves. Native /
        // Docker caves both go through this — `create_docker` re-saves
        // afterwards with the upgraded backing+image fields.
        crate::batcave::persist::save(&CAVES[slot]);

        Ok(slot)
    }
}

/// Create a docker-backed BatCave. Thin wrapper over `create` that also
/// stores the image ref so `list` / `destroy` / `run` can route to the
/// `batcaved` daemon via `docker_client`.
// /
/// This function does NOT spin up the container — the shell handler is
/// responsible for calling `docker_client::create` so a daemon-side
/// failure can be surfaced BEFORE we've polluted the cave table.
pub fn create_docker(name: &str, image: &str, ephemeral: bool)
    -> Result<usize, &'static str>
{
    if image.len() > MAX_IMAGE { return Err("image name too long"); }
    let slot = create(name, ephemeral)?;
    unsafe {
        let cave = &mut CAVES[slot];
        cave.backing = CaveBacking::Docker;
        cave.image[..image.len()].copy_from_slice(image.as_bytes());
        cave.image_len = image.len();
    }
    // Followup 3a/3b: register the cave with the kernel's policy store
    // with ZERO rules. Any egress attempt will hit default-deny until a
    // grant arrives via `cpol-add` / `batcave-fw-allow`. This guarantees
    // the kernel is aware of the cave's existence at creation time —
    // the 3c packet path will be able to map container source addresses
    // to a known policy entry.
    crate::net::cave_policy::set_policy_by_name(
        name,
        alloc::vec::Vec::new(),
    );
    // re-save manifest now that backing=Docker + image are
    // populated. The first save() inside create() captured a Native
    // baseline; this overwrites with the docker-aware version so the
    // cave wakes up correctly from disk.
    unsafe { crate::batcave::persist::save(&CAVES[slot]); }
    Ok(slot)
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

    // refresh manifest so installed tools survive reboot.
    crate::batcave::persist::save(cave);

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

    // refresh manifest so granted caps survive reboot.
    crate::batcave::persist::save(cave);

    Ok(())
}

/// Revoke a capability from a BatCave.
pub fn revoke_cap(name: &str, cap: &str) -> Result<(), &'static str> {
    let cave = find_mut(name)?;

    for i in 0..cave.cap_count {
        if cave.caps[i].active && cave.caps[i].name_str() == cap {
            cave.caps[i].active = false;
            // refresh manifest so revoked caps don't reappear
            // on reboot. The caps slot stays in place (active=false) so
            // existing code paths don't shift indices mid-iteration.
            crate::batcave::persist::save(cave);
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
// /
/// V5-XLAYER-001/002/003 fix: wipe all per-cave static state on every
/// cave switch (not only on destroy). Previously the signal handler
/// table, TLS key state, and fd table persisted across caves — one
/// cave could install a handler, exit, and a later cave would inherit
/// it. Same for TLS session keys on PCBs the previous tenant opened.
pub fn enter(name: &str) -> Result<(), &'static str> {
    start(name)?;
    if let Some(id) = find_id(name) {
        // lazy-build the cave's L1 page table BEFORE the
        // critical section. The setup helper allocates 6 frames + does
        // a 6×512-entry initialization pass — too long to run with
        // IRQs masked. Building outside the CS is safe because we
        // store the result in CAVES[id] (atomic-by-being-static-mut +
        // single-CPU) and re-read it inside the CS.
        //
        // Skip docker-backed caves: their isolation is the Mac
        // kernel's container, not our MMU. Building a Bat_OS L1 for
        // them would be wasted memory.
        let needs_l1 = unsafe {
            !CAVES[id].is_docker() && CAVES[id].cave_l1_phys == 0
        };
        if needs_l1 {
            if let Some(slot) = crate::batcave::linux::mmu::alloc_native_cave_slot() {
                if let Ok(l1) = crate::batcave::linux::mmu::setup_native_cave_l1(slot) {
                    unsafe {
                        CAVES[id].cave_l1_phys = l1;
                        CAVES[id].cave_l1_slot = slot;
                    }
                }
                // On allocation failure (frame-pool OOM, MAX_CAVE_PAGETABLES
                // exhausted) we fall through with cave_l1_phys=0 — the cave
                // still works using PRIMARY_L1, just without per-cave TLB
                // isolation. Audit-log so the operator sees the regression.
                if unsafe { CAVES[id].cave_l1_phys } == 0 {
                    crate::security::audit::record(
                        crate::security::audit::Category::Cave,
                        b"WARN: cave L1 allocation failed; using primary TTBR0",
                    );
                }
            }
        }

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
            // V8-ROOT-2: route every per-cave reset through the central hub
            // so adding a new subsystem requires updating ONE place. Missing
            // one of these previously was the root of half a dozen
            // cross-cave information-leak bugs.
            reset_all_globals_for_cave_switch();
            set_active(id);
            crate::batcave::linux::vfs::init_for_cave(id);

            // install the cave's L1 in TTBR0_EL1.
            // `switch_to_cave` does the canonical pre-write `tlbi vmalle1
            // dsb sy ; isb` → `msr ttbr0_el1, x` → `isb` → post-write
            // `tlbi vmalle1 ; dsb sy ; isb` sequence. With this in place
            // every native cave's TLB entries are isolated from every
            // other cave's, closing the audit's "memory isolation is
            // fiction" verdict for Layer 1.
            //
            // If lazy-allocation failed above (cave_l1_phys=0), we
            // explicitly switch to PRIMARY_L1 so the previous cave's
            // TTBR0 doesn't leak forward — better to share kernel L1
            // than to keep a stale per-cave one active.
            let l1 = unsafe { CAVES[id].cave_l1_phys };
            if l1 != 0 {
                crate::batcave::linux::mmu::switch_to_cave(l1);
            } else {
                crate::batcave::linux::mmu::switch_to_primary();
            }

            // repaint the title-bar chrome NOW (after
            // set_active(id) above). The CAVE indicator reads
            // `active_name_str()` which now returns the new cave's
            // name. Doing this earlier (e.g. inside
            // console::reset_for_cave_switch) would have rendered
            // the old cave's name because we always set
            // ACTIVE_CAVE_ID = usize::MAX before the reset and only
            // restore it after. wm::draw_frame is just FB writes —
            // safe inside the IrqGuard'd critical section.
            crate::ui::wm::draw_frame();
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
// /
/// Anti-coercion design per DESIGN_BATCAVES.md §"Seal": once an
/// operator is facing duress and seals a cave, the persistent-state
/// guarantees go away IMMEDIATELY — not at next-reboot. This means:
// /
/// 1. cave_type flips to Ephemeral (blocks future "already sealed"
/// re-seals and will get swept by destroy_all like any ephemeral).
/// 2. fs_key is zeroed RIGHT NOW. Any BatFS blob that survives
/// on disk becomes undecryptable — even to the operator, even
/// with the passphrase. One-way ratchet.
/// 3. For Docker-backed caves: the daemon's encrypted APFS volume
/// is destroyed. The bind-mount inside the container becomes
/// inert — the container keeps running for its current session
/// but has nothing persistent backing /data. On next wipe /
/// reboot the remaining container state dies with the cave.
// /
/// There is no `unseal`. Callers that get "already ephemeral" have
/// either sealed this cave before OR created it as ephemeral; in
/// either case the operator is not losing anything by the error.
pub fn seal(name: &str) -> Result<(), &'static str> {
    // Capture Docker-ness + image name BEFORE we flip state, because
    // the daemon call needs the existing identifiers.
    let (is_docker, already_ephemeral) = {
        let cave = find_mut(name)?;
        (matches!(cave.backing, CaveBacking::Docker),
         cave.cave_type == CaveType::Ephemeral)
    };

    if already_ephemeral {
        return Err("already ephemeral");
    }

    // Tell the daemon to destroy the encrypted volume + drop the
    // per-cave key mapping. Best-effort — if the daemon is down we
    // still flip state locally so the ratchet holds.
    if is_docker {
        let _ = crate::batcave::docker_client::with_daemon(|| {
            crate::batcave::docker_client::cave_seal(name)
        });
    }

    // Zero the BatFS key + flip state. `find_mut` is re-run because
    // the daemon call above may have yielded.
    let cave = find_mut(name)?;
    cave.fs_key = [0; 32];
    cave.cave_type = CaveType::Ephemeral;

    // a sealed cave is no longer Persistent — drop its
    // manifest so it doesn't reincarnate as Persistent on next boot.
    // The seal ratchet must hold across reboots too; otherwise an
    // attacker who panics → coerces a seal → reboots the box would
    // see the original Persistent cave back. Anti-coercion is the
    // entire point of seal, so this delete is non-negotiable.
    crate::batcave::persist::delete(name);

    // persist the audit ring so the seal event itself
    // (and everything before it) survives a panic-induced reboot.
    // Pairs with the persist::delete above — if the attacker triggers
    // a reboot to escape the trail, the trail is already on disk.
    let _ = crate::security::audit::flush_to_batfs();

    Ok(())
}

/// Destroy a BatCave — secure wipe.
// /
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
        // If this was the active cave, clear active. also
        // swap TTBR0 back to PRIMARY_L1 so the about-to-be-freed L1
        // doesn't keep getting walked. `mmu::free_cave_slot` below
        // tears down the L1 frames; running with a freed L1 in TTBR0
        // is asking for a use-after-free in the page-table walker.
        if get_active() == id {
            set_active(usize::MAX);
            crate::batcave::linux::mmu::switch_to_primary();
        }
        // free this cave's L1/L2 frames + clear the slot
        // so the next cave can claim it. Idempotent — safe even if
        // cave_l1_slot was usize::MAX (the cave never entered).
        unsafe {
            let slot = CAVES[id].cave_l1_slot;
            if slot != usize::MAX {
                crate::batcave::linux::mmu::free_cave_slot(slot);
                CAVES[id].cave_l1_slot = usize::MAX;
                CAVES[id].cave_l1_phys = 0;
            }
        }
    }

    let cave = match find_mut(name) {
        Ok(c) => c,
        Err(e) => {
            // every
            // destroy attempt is logged regardless of outcome. The
            // failure path here is most often "operator typo" but it
            // ALSO covers a malicious destroy probe (someone scanning
            // for cave names by trying to delete them). Keep the trail.
            let mut buf = [0u8; 192];
            let mut p = 0usize;
            let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
                let n = src.len().min(dst.len().saturating_sub(*p));
                dst[*p..*p + n].copy_from_slice(&src[..n]);
                *p += n;
            };
            copy(&mut buf, b"cave destroy FAILED ", &mut p);
            copy(&mut buf, name.as_bytes(), &mut p);
            crate::security::audit::record(
                crate::security::audit::Category::Cave,
                &buf[..p],
            );
            return Err(e);
        }
    };

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

    // Followup 3a/3b: a destroyed cave must not leave stale egress rules
    // the kernel would match against a fresh cave that reuses the name.
    crate::net::cave_policy::clear_by_name(name);

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

    // success-path
    // log so the audit ring shows full lifecycle (create → use →
    // destroy) for every cave. Critical for post-incident review.
    let mut buf = [0u8; 192];
    let mut p = 0usize;
    let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
        let n = src.len().min(dst.len().saturating_sub(*p));
        dst[*p..*p + n].copy_from_slice(&src[..n]);
        *p += n;
    };
    copy(&mut buf, b"cave destroy OK ", &mut p);
    copy(&mut buf, name.as_bytes(), &mut p);
    crate::security::audit::record(
        crate::security::audit::Category::Cave,
        &buf[..p],
    );

    // drop the persisted manifest so the cave stays
    // destroyed across reboots. Idempotent — silently does nothing if
    // the cave was Ephemeral and never had a manifest.
    crate::batcave::persist::delete(name);

    // persist the audit ring so the destroy event is
    // durable across reboot. Same anti-coercion reasoning as seal.
    let _ = crate::security::audit::flush_to_batfs();

    Ok(())
}

/// Destroy ALL BatCaves — called by wipe system.
pub fn destroy_all() {
    // Phase 5: every wipe event (deadman, duress, panic, emergency_wipe)
    // must take out docker-backed caves too. Fan out to the batcaved
    // daemon via docker_client::destroy_all BEFORE we zero the local
    // cave table — otherwise the daemon has no way to know which caves
    // were Bat_OS's (the `name_len = 0` reset below wipes that).
    //
    // Errors from the daemon are logged but not fatal — the local
    // teardown still runs. If the daemon is unreachable (operator
    // killed it, network blip), we can't help the remote containers,
    // but the in-Bat_OS state still gets zeroed. A daemon restart
    // reconciles via its own state (it tracks every `batcave-*` name
    // it knows about through docker ps).
    let had_docker = unsafe {
        let mut any = false;
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free && CAVES[i].is_docker() {
                any = true; break;
            }
        }
        any
    };
    if had_docker {
        let r = crate::batcave::docker_client::with_daemon(|| {
            crate::batcave::docker_client::destroy_all()
        });
        match r {
            Ok(n) => {
                crate::drivers::uart::puts("  [wipe] docker caves destroyed: ");
                crate::kernel::mm::print_num(n);
                crate::drivers::uart::puts("\n");
            }
            Err(e) => {
                crate::drivers::uart::puts("  [wipe] docker destroy_all failed: ");
                crate::drivers::uart::puts(e);
                crate::drivers::uart::puts(" (daemon unreachable?)\n");
            }
        }
    }

    // take down persisted manifests first, BEFORE we zero
    // the in-RAM names. After the loop below `name_len = 0` so the
    // names would be unrecoverable. Wipe events (deadman/duress/panic)
    // must clear the manifests too — otherwise the next boot would
    // resurrect every cave from disk and the wipe would have done
    // nothing useful for the cave registry.
    unsafe {
        let mut name_buf = [[0u8; MAX_NAME]; MAX_CAVES];
        let mut name_lens = [0usize; MAX_CAVES];
        let mut count = 0usize;
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free
               && CAVES[i].cave_type == CaveType::Persistent {
                let nl = CAVES[i].name_len.min(MAX_NAME);
                name_buf[count][..nl].copy_from_slice(&CAVES[i].name[..nl]);
                name_lens[count] = nl;
                count += 1;
            }
        }
        for i in 0..count {
            let name = core::str::from_utf8_unchecked(&name_buf[i][..name_lens[i]]);
            crate::batcave::persist::delete(name);
        }
    }

    unsafe {
        for i in 0..MAX_CAVES {
            if CAVES[i].state != CaveState::Free {
                CAVES[i].fs_key = [0; 32];
                CAVES[i].state = CaveState::Free;
                CAVES[i].tool_count = 0;
                CAVES[i].cap_count = 0;
                CAVES[i].name_len = 0;
                CAVES[i].backing = CaveBacking::Native;
                CAVES[i].image_len = 0;
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

// ── Seal ratchet selftest ─────────────────────────────────────────

pub struct SealReport {
    pub before_was_persistent: bool,
    pub after_is_ephemeral: bool,
    pub fs_key_zeroed: bool,
    pub reseal_rejected: bool,
}

/// Pure in-kernel proof of the seal ratchet:
/// 1. Create a persistent cave with a non-zero fs_key (derived from
/// the name).
/// 2. Inspect pre-state: type=Persistent, fs_key != 0.
/// 3. Call `seal()`.
/// 4. Inspect post-state: type=Ephemeral AND fs_key == [0;32].
/// 5. Call `seal()` again — must return "already ephemeral" (one-way).
/// 6. Destroy cave to clean up.
pub fn seal_selftest() -> Result<SealReport, &'static str> {
    let name = "seal-selftest-cave";
    // Fresh slate in case a prior run left residue.
    let _ = destroy(name);

    let _slot = create(name, false)?;   // ephemeral=false → Persistent
    let (ptype_before, key_before) = {
        let c = find_mut(name)?;
        (c.cave_type, c.fs_key)
    };
    if ptype_before != CaveType::Persistent { return Err("expected Persistent before seal"); }
    if key_before == [0u8; 32] { return Err("fresh cave's fs_key should not be all-zero"); }

    // Seal. This is a native cave so there's no daemon round-trip.
    seal(name)?;

    let (ptype_after, key_after) = {
        let c = find_mut(name)?;
        (c.cave_type, c.fs_key)
    };
    if ptype_after != CaveType::Ephemeral { return Err("expected Ephemeral after seal"); }
    let key_zeroed = key_after == [0u8; 32];

    // Re-seal must reject.
    let reseal = seal(name);
    let reseal_rejected = reseal.is_err();

    // Cleanup.
    let _ = destroy(name);

    Ok(SealReport {
        before_was_persistent: ptype_before == CaveType::Persistent,
        after_is_ephemeral: ptype_after == CaveType::Ephemeral,
        fs_key_zeroed: key_zeroed,
        reseal_rejected,
    })
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

/// V8-ROOT-2: single hub that resets EVERY subsystem holding cross-cave
/// state. Call this from cave::enter (and only from cave::enter) when
/// switching the active cave. Adding a new subsystem with cave-local
/// state requires updating exactly one place — here.
// /
/// Caller must already hold a critical_section! / IrqGuard. Each callee
/// also acquires its own IrqGuard, which is safe (IrqGuard is nestable).
fn reset_all_globals_for_cave_switch() {
    crate::batcave::linux::syscall::reset_cave_statics();
    crate::net::tls::reset_all_sessions();
    crate::batcave::linux::fd::reset_for_cave_switch();
    crate::batcave::linux::sockets::reset_for_cave_switch();
    crate::net::tcp::reset_for_cave_switch();
    // Sprint 3.1: cookie jar wipe on cave switch so a
    // logged-out cave doesn't inherit the previous tenant's session
    // tokens.
    crate::net::cookies::reset_for_cave_switch();

    // ROOT 2 additions — previously missing, each one was a cross-cave
    // information-leak surface.
    crate::batcave::linux::epoll::reset_for_cave_switch();
    crate::batcave::linux::futex::reset_for_cave_switch();
    crate::batcave::linux::async_fds::reset_for_cave_switch();
    crate::batcave::linux::stdio_ring::reset_for_cave_switch();
    crate::net::psk_overlay::reset_for_cave_switch();
    crate::net::dns::reset_for_cave_switch();

    // ROOT 2 V9-re-audit additions — threads table, ARP cache.
    // Each was a cross-cave leak the first pass missed.
    crate::batcave::linux::threads::reset_for_cave_switch();
    crate::net::arp::reset_for_cave_switch();

    // ROOT 2 V10-re-audit additions — batpipe inter-tool buffer + the
    // loader's saved RA/SP (which were a cross-cave control-flow PIVOT,
    // most severe item from the V10 sweep). removed the
    // `tor::reset_for_cave_switch` call because tor.rs was deleted —
    // it was 3 layers of CTR with hardcoded keys, not real Tor. When
    // real onion routing lands its reset hook goes back here.
    crate::batcave::batpipe::reset_for_cave_switch();
    crate::batcave::linux::loader::reset_for_cave_switch();

    // ROOT 2 V11-re-audit additions — keyboard-input leak (typed pass-
    // phrases), comms session key + chat history, browser state (URL,
    // page buffer, DOM, tabs, bookmarks), Blink heap (HTML residue),
    // UI surface state (font clip + wm panes), and the Apple-SPI
    // keyboard mirror. Last mechanical gaps from the V11 sweep.
    crate::drivers::virtio::keyboard::reset_for_cave_switch();
    crate::drivers::apple::spi::reset_for_cave_switch();
    crate::ui::apps::comms::reset_for_cave_switch();
    crate::ui::font::reset_for_cave_switch();
    crate::ui::wm::reset_for_cave_switch();
    crate::ui::console::reset_for_cave_switch();

}
