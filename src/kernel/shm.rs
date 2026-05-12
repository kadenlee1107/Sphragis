//! POSIX shared memory.
//!
//! Gap-audit item 027. Matches the `shm_open` / `mmap` shape: named
//! regions of bytes that multiple tasks can open by name and read
//! /write directly. We don't have separate userspace address spaces
//! yet — every task runs in the kernel address space sharing the
//! same page tables — so "shared memory" is just "every task that
//! opens this name sees the same `&mut [u8]`". Phase 3 (real
//! userspace) will need page-table mapping; the API stays.
//!
//! Lifecycle:
//!   - `create(name, size)` allocates a fresh region, hands the
//!     caller an fd. The region exists until every fd referencing
//!     it has been closed.
//!   - `open(name)` finds an existing region by name and installs
//!     an additional fd on the caller's table. Refcount bumps.
//!   - `close` (via the existing SYS_CLOSE) decrements; when the
//!     last reference goes, the backing storage is wiped + freed.
//!
//! Audit:
//!   - `Category::Shm` logs create / open / close with the name and
//!     pid. Per-byte traffic is not logged — same rate-limit
//!     philosophy as the Pipe and Socket categories.
//!
//! Security model:
//!   - Names live in the kernel global namespace (no per-cave
//!     scoping yet). On cave switch the table is wiped so a new
//!     tenant can't open the prior tenant's regions, but within a
//!     single cave any task can open any name.
//!   - Backing storage is zeroed on free.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use crate::drivers::uart;
use crate::kernel::process::{self, FdEntry, FdKind, TaskId};
use crate::security::audit::{self, Category};

pub const MAX_REGIONS:    usize = 32;
pub const MAX_NAME_LEN:   usize = 64;
pub const MAX_REGION_LEN: usize = 16 * 1024; // 16 KiB cap per region

struct Region {
    active: bool,
    name: [u8; MAX_NAME_LEN],
    name_len: u8,
    /// Refcount of live fds pointing at this region across all
    /// tasks. Region reclaimed when this hits zero.
    refs: u16,
    /// Backing bytes. None when the slot is free.
    data: Option<Vec<u8>>,
    /// Creator's task id — recorded for audit attribution.
    owner: TaskId,
}

impl Region {
    const fn empty() -> Self {
        Self {
            active: false,
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            refs: 0,
            data: None,
            owner: TaskId(0),
        }
    }

    fn name_str(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }
}

static mut REGIONS: [Region; MAX_REGIONS] = {
    const EMPTY: Region = Region::empty();
    [EMPTY; MAX_REGIONS]
};

pub fn init() {
    uart::puts("  [shm] POSIX shared memory table initialized\n");
}

fn region_mut(id: u16) -> Option<&'static mut Region> {
    let i = id as usize;
    if i >= MAX_REGIONS { return None; }
    unsafe { Some(&mut (*core::ptr::addr_of_mut!(REGIONS))[i]) }
}

fn find_by_name(name: &[u8]) -> Option<u16> {
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return None;
    }
    unsafe {
        for i in 0..MAX_REGIONS {
            let r = &(*core::ptr::addr_of!(REGIONS))[i];
            if r.active && r.name_str() == name {
                return Some(i as u16);
            }
        }
    }
    None
}

/// Create a fresh named region of `size` bytes and install an fd on
/// the caller's fd table. Errors:
///   "name taken"       — another region already has this name
///   "name too long"    — > MAX_NAME_LEN
///   "size too large"   — > MAX_REGION_LEN
///   "no free region"   — table exhausted
///   "out of memory"    — heap allocation failed
///   "fd table full"    — caller has no fd slots
pub fn create(name: &[u8], size: usize) -> Result<u16, &'static str> {
    if name.is_empty() {
        return Err("empty name");
    }
    if name.len() > MAX_NAME_LEN {
        return Err("name too long");
    }
    if size == 0 || size > MAX_REGION_LEN {
        return Err("size too large");
    }
    if find_by_name(name).is_some() {
        return Err("name taken");
    }

    let owner = process::current_id();

    // Find a free slot.
    let mut slot = None;
    unsafe {
        for i in 0..MAX_REGIONS {
            if !(*core::ptr::addr_of!(REGIONS))[i].active {
                slot = Some(i);
                break;
            }
        }
    }
    let slot = slot.ok_or("no free region")?;

    // Allocate backing storage. Vec::with_capacity + resize_with
    // gives us zero-initialized bytes without using `vec![0u8; N]`
    // which can fail silently on alloc failure in no_std contexts.
    let mut v: Vec<u8> = Vec::new();
    if v.try_reserve_exact(size).is_err() {
        return Err("out of memory");
    }
    v.resize(size, 0);

    let r = region_mut(slot as u16).unwrap();
    r.active = true;
    r.name[..name.len()].copy_from_slice(name);
    r.name_len = name.len() as u8;
    r.owner = owner;
    r.refs = 1;
    r.data = Some(v);

    // Install fd on caller's table.
    let task = process::get(owner);
    let fd = match task.fd_alloc(FdEntry { kind: FdKind::Shm { id: slot as u16 } }) {
        Some(fd) => fd,
        None => {
            // Roll back the region.
            r.active = false;
            r.data = None;
            r.refs = 0;
            return Err("fd table full");
        }
    };

    audit_evt(b"create", slot as u16, name);
    Ok(fd)
}

/// Open an existing region by name and install an additional fd on
/// the caller's table. Refcount bumps.
pub fn open(name: &[u8]) -> Result<u16, &'static str> {
    let id = find_by_name(name).ok_or("no such name")?;
    let r = region_mut(id).ok_or("bad region id")?;
    if !r.active {
        return Err("region closed");
    }
    if r.refs == u16::MAX {
        return Err("refcount overflow");
    }
    r.refs += 1;

    let task = process::current();
    let fd = match task.fd_alloc(FdEntry { kind: FdKind::Shm { id } }) {
        Some(fd) => fd,
        None => {
            r.refs -= 1;
            return Err("fd table full");
        }
    };
    audit_evt(b"open", id, name);
    Ok(fd)
}

/// Drop a reference. Called from SYS_CLOSE when the fd's kind is
/// `FdKind::Shm`. Reclaims storage when the refcount hits zero.
pub fn release(id: u16) {
    let Some(r) = region_mut(id) else { return; };
    if !r.active { return; }

    let mut name_copy = [0u8; MAX_NAME_LEN];
    let nlen = r.name_len as usize;
    name_copy[..nlen].copy_from_slice(&r.name[..nlen]);

    if r.refs > 0 { r.refs -= 1; }
    audit_evt(b"close", id, &name_copy[..nlen]);

    if r.refs == 0 {
        // Wipe storage before dropping so a future allocator reuse
        // doesn't leak this region's contents into another path.
        if let Some(buf) = r.data.as_mut() {
            for b in buf.iter_mut() {
                unsafe { core::ptr::write_volatile(b as *mut u8, 0); }
            }
        }
        r.data = None;
        r.active = false;
        r.name_len = 0;
    }
}

/// Borrow the region's bytes mutably. Returns None if the id is
/// out of range or the region is no longer active. Caller is
/// trusted not to alias — same convention as the rest of the
/// kernel's `&mut` access to shared state.
pub fn region_bytes_mut(id: u16) -> Option<&'static mut [u8]> {
    let r = region_mut(id)?;
    if !r.active { return None; }
    let v = r.data.as_mut()?;
    Some(unsafe { core::slice::from_raw_parts_mut(v.as_mut_ptr(), v.len()) })
}

pub fn region_size(id: u16) -> Option<usize> {
    let r = unsafe { &(*core::ptr::addr_of!(REGIONS))[id as usize] };
    if !r.active { return None; }
    r.data.as_ref().map(|v| v.len())
}

/// Wipe everything on cave switch. A new cave can't open the
/// previous tenant's shm regions.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..MAX_REGIONS {
            let r = &mut (*core::ptr::addr_of_mut!(REGIONS))[i];
            if let Some(buf) = r.data.as_mut() {
                for b in buf.iter_mut() {
                    core::ptr::write_volatile(b as *mut u8, 0);
                }
            }
            r.data = None;
            r.active = false;
            r.name_len = 0;
            r.refs = 0;
        }
    }
}

// (the unsafe block in reset_for_cave_switch covers the write_volatile
//  calls above — `unsafe { ... }` wraps the whole loop body.)

// ── audit helper ──────────────────────────────────────────────────

fn audit_evt(verb: &[u8], id: u16, name: &[u8]) {
    let mut buf = [0u8; 128];
    let mut at = 0;
    at = push(&mut buf, at, verb);
    at = push(&mut buf, at, b" pid=");
    at = u16_dec(&mut buf, at, process::current_id().0);
    at = push(&mut buf, at, b" id=");
    at = u16_dec(&mut buf, at, id);
    if !name.is_empty() {
        at = push(&mut buf, at, b" name=");
        let take = name.len().min(buf.len() - at);
        for i in 0..take {
            let b = name[i];
            buf[at + i] = if (0x20..=0x7e).contains(&b) { b } else { b'?' };
        }
        at += take;
    }
    audit::record(Category::Shm, &buf[..at]);
}

fn push(buf: &mut [u8], at: usize, s: &[u8]) -> usize {
    let n = s.len().min(buf.len().saturating_sub(at));
    buf[at..at + n].copy_from_slice(&s[..n]);
    at + n
}

fn u16_dec(buf: &mut [u8], at: usize, v: u16) -> usize {
    if v == 0 {
        if at < buf.len() { buf[at] = b'0'; }
        return at + 1;
    }
    let mut tmp = [0u8; 5];
    let mut i = 0;
    let mut n = v;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in 0..i {
        if at + j < buf.len() { buf[at + j] = tmp[i - 1 - j]; }
    }
    at + i
}
