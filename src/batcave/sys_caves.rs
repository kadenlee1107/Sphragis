//! Boot-time `sys-*` cave bring-up.
//!
//! Implements Arc 2 of the plan in DESIGN_SYS_CAVES.md: at boot,
//! the kernel spawns named service caves with their cap sets
//! pre-wired and their per-cave L1 page tables pre-built. Future
//! cave-services (sys-wg, sys-net, sys-tor, ...) hand work to
//! these caves via authenticated IPC instead of touching the
//! global kernel state directly.
//!
//! Today this module brings up exactly **sys-wg** — the cave that
//! will eventually own the WireGuard static keypair and per-peer
//! transport state. The cave's address space exists from boot
//! onward; Arc 3 relocates the WireGuard library code into it.
//!
//! Build ordering: must run AFTER `fs::batfs::init` (because
//! `cave::create` derives `fs_key` from the BatFS master key) and
//! AFTER `security::auth::init` (because the master key isn't
//! valid until the operator's passphrase has been mixed in). Hence
//! the call site lives in `main.rs` between the auth-init step
//! and the auth-gate launch.

#![allow(dead_code)]

use crate::batcave::cave;
use crate::drivers::uart;
use crate::kernel::kmsg;

/// Resolved id of the sys-wg cave. `usize::MAX` until `init()` has
/// run and the cave has been created.
static mut SYS_WG_ID: usize = usize::MAX;

/// Return the resolved sys-wg cave id, or None if init hasn't run
/// (or the create failed at boot — see warnings in `init`).
pub fn sys_wg_id() -> Option<usize> {
    unsafe {
        let v = core::ptr::read_volatile(core::ptr::addr_of!(SYS_WG_ID));
        if v == usize::MAX { None } else { Some(v) }
    }
}

/// Boot-time bring-up. Creates the sys-wg cave with the `net`
/// capability and immediately builds its L1 page table so the
/// scheduler MMU hook (Arc 1) has a real target to switch to.
///
/// On any failure we leave SYS_WG_ID == usize::MAX and log a
/// warning. The kernel continues to boot — sys-wg is enhancement,
/// not load-bearing for the auth path or the desktop.
pub fn init() {
    // 0. Reserve slot 0 with a sentinel "kernel-ns" cave. The
    //    `task.cave_id` field uses `0` to mean "kernel namespace —
    //    no cave attached" (scheduler.rs's MMU hook switches to
    //    PRIMARY_L1 in that case). If a real cave were allowed to
    //    land at slot 0, its cave_id (== slot) would collide with
    //    the kernel-ns sentinel and the scheduler would treat
    //    transitions INTO it as transitions OUT of any cave.
    //    Reserving slot 0 explicitly here means real caves always
    //    start at slot 1+. Failure is non-fatal: it just means
    //    sys-wg might still land at slot 0 (in which case the MMU
    //    hook is degenerate for sys-wg, but other caves work).
    if let Err(e) = cave::create("kernel-ns", /* ephemeral */ true) {
        uart::puts("  [sys-caves] WARN: kernel-ns sentinel create failed: ");
        uart::puts(e);
        uart::puts("\n");
    }

    // 1. Create the cave. `create(name, ephemeral=false)` registers
    //    the cave persistently — its fs_key + identity survive
    //    reboots once we wire that up. Ephemeral would tear down on
    //    reboot, which is wrong for a long-lived service cave.
    let id = match cave::create("sys-wg", /* ephemeral */ false) {
        Ok(id) => id,
        Err(e) => {
            uart::puts("  [sys-caves] WARN: sys-wg create failed: ");
            uart::puts(e);
            uart::puts("\n");
            kmsg::warn(b"sys-caves: sys-wg create failed");
            return;
        }
    };

    // 2. Grant the network capability so this cave is allowed to
    //    drive WireGuard's UDP socket once we have one. No other
    //    caps — sys-wg has no business touching the FS, the
    //    display, or other caves' memory.
    if let Err(e) = cave::grant_cap("sys-wg", "net") {
        uart::puts("  [sys-caves] WARN: grant_cap(sys-wg, net) failed: ");
        uart::puts(e);
        uart::puts("\n");
    }

    // 3. Build the L1 immediately so the scheduler MMU hook can
    //    switch to it. Same lazy-build path cave::enter normally
    //    uses, hoisted to boot time.
    if let Some(slot) = crate::batcave::linux::mmu::alloc_native_cave_slot() {
        match crate::batcave::linux::mmu::setup_native_cave_l1(slot) {
            Ok(l1) => unsafe {
                let ptr = core::ptr::addr_of_mut!(crate::batcave::cave::CAVES);
                (*ptr)[id].cave_l1_phys = l1;
                (*ptr)[id].cave_l1_slot = slot;
            },
            Err(e) => {
                uart::puts("  [sys-caves] WARN: sys-wg L1 build failed: ");
                uart::puts(e);
                uart::puts("\n");
                kmsg::warn(b"sys-caves: sys-wg L1 build failed");
                // Continue — cave still exists, just without per-cave TLB
                // isolation. Same fallback as cave::enter's L1-alloc
                // failure branch.
            }
        }
    } else {
        uart::puts("  [sys-caves] WARN: out of CAVE_L1 slots; sys-wg lacks MMU isolation\n");
        kmsg::warn(b"sys-caves: no free CAVE_L1 slot for sys-wg");
    }

    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SYS_WG_ID), id);
    }
    uart::puts("  [sys-caves] sys-wg cave ready (id=");
    crate::kernel::mm::print_num(id);
    uart::puts(")\n");
    kmsg::info(b"sys-caves: sys-wg cave ready");
}
