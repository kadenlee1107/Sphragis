// Bat_OS — Demand paging for rebased-cave large mmap reservations.
//
// Chromium / V8 asks for huge VA reservations at startup (32 GB for
// pointer-compression, 16 GB for trusted-sandbox). `sys_mmap`'s
// huge-reservation stub returns the caller's hint without committing
// any physical memory. That was only half the deal — V8 then mprotects
// small sub-ranges and tries to use them. With no cave L2 entry for
// the reserved VA range, the first access data-aborts.
//
// This module fills in the other half: a reservation table + an
// EC=0x24 data-abort handler that lazily allocates 4 KB pages and
// installs them via L1→L2→L3 page-table walk. Each page is drawn from
// the general frame pool on first access.
//
// Scope limits (deliberate, for tonight's cut):
//
// - No page ageing, no swap, no sharing across caves. Once committed,
//   a page stays committed until the cave tears down.
// - No mprotect honouring — all committed pages get
//   `BLOCK_USER_RW_EXEC` like the legacy cave window does. Chromium's
//   prot argument (the odd `0x14d82074` value we see in the trace) is
//   ignored.
// - No MAP_FIXED into a reserved range. The huge-reservation stub
//   already rejects MAP_FIXED.
// - Up to `MAX_RESERVATIONS` regions per boot (global, not per-cave).
//   Chromium's two reservations + maybe a third for V8 code cages
//   comfortably fit.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use crate::drivers::uart;
use crate::kernel::mm::frame;

/// Bits in an L1/L2 table descriptor: valid + type=table.
const TABLE_DESC: u64 = 0b11;
/// L3 page descriptor bits. Matches `BLOCK_USER_RW_EXEC` but at 4 KB
/// granule (L3 entries are "pages", not "blocks" — bit 1 = 1 means
/// "page" for L3, which is the 0b11 pattern too).
const PAGE_VALID: u64 = 0b11;
const PAGE_AF: u64    = 1 << 10;
const PAGE_SH: u64    = 3 << 8;
const PAGE_ATTR: u64  = 0 << 2;
const PAGE_AP_EL0_RW: u64 = 1 << 6;
const PAGE_PXN: u64   = 1 << 53;
const PAGE_UXN: u64   = 1 << 54;

/// Flags we program into every committed user page by default. EL0 R/W,
/// PXN (EL1 can't accidentally execute here), *and UXN* — committed
/// demand-page frames are RW-only at EL0 so Chromium/V8 control-flow
/// that accidentally branches into data (V8 pointer-compression cage,
/// anonymous heap pages, etc.) traps cleanly as EC=0x20 (instruction
/// abort, lower EL) instead of silently fetching garbage and limping
/// along through a cascade of EC=0x19/0x1c/0x1d decoder-confusion
/// faults.
///
/// Regions that *need* execute permission (V8 JIT code areas, PLT
/// trampolines) must call `sys_mprotect` with `PROT_EXEC` to clear
/// UXN. See `sys_mprotect` in syscall.rs.
pub(crate) const USER_PAGE_FLAGS: u64 = PAGE_VALID | PAGE_AF | PAGE_SH | PAGE_ATTR
    | PAGE_AP_EL0_RW | PAGE_PXN | PAGE_UXN;

const MAX_RESERVATIONS: usize = 8;

#[derive(Copy, Clone)]
struct Reservation {
    start: u64,
    end:   u64,
    l1_phys: u64,    // which cave L1 this reservation belongs to (TTBR0 value)
}

// Flat array, no atomics on individual Reservation — we guard the
// whole table with a single IrqGuard during mutation / lookup.
static mut RESV_TABLE: [Reservation; MAX_RESERVATIONS] =
    [Reservation { start: 0, end: 0, l1_phys: 0 }; MAX_RESERVATIONS];

static RESV_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Total pages the demand-page handler has committed since boot.
/// Diagnostic — `chromium` status line prints this.
pub static COMMITTED_PAGES: AtomicU64 = AtomicU64::new(0);

/// Returns `true` if `[addr, addr+len)` falls inside a registered
/// huge reservation for the CURRENT cave (matched via TTBR0). Used by
/// `uaccess::is_user_range` so syscalls that dereference user pointers
/// don't reject a buffer sitting in V8's pointer-compression area
/// just because it's outside the cave's L2 window — the demand-page
/// handler will commit a real frame on first access.
pub fn is_in_active_reservation(addr: usize, len: usize) -> bool {
    let a = addr as u64;
    let end = match a.checked_add(len as u64) {
        Some(e) => e,
        None => return false,
    };
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let count = RESV_COUNT.load(Ordering::Acquire);
        let table = &*core::ptr::addr_of!(RESV_TABLE);
        for i in 0..count {
            let r = &table[i];
            if r.l1_phys != l1_phys { continue; }
            if a >= r.start && end <= r.end { return true; }
        }
    }
    false
}

/// Record a huge mmap reservation so the fault handler will lazily
/// back its pages. `sys_mmap` calls this after its hint-return path.
pub fn register_reservation(start: u64, end: u64, l1_phys: u64) {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let slot = RESV_COUNT.load(Ordering::Acquire);
        if slot >= MAX_RESERVATIONS {
            uart::puts("[demand_page] WARN: reservation table full\n");
            return;
        }
        core::ptr::write(
            core::ptr::addr_of_mut!(RESV_TABLE[slot]),
            Reservation { start, end, l1_phys },
        );
        RESV_COUNT.store(slot + 1, Ordering::Release);
    }
}

/// Data-abort handler. Returns `true` if the fault was inside a
/// registered reservation and we satisfied it by committing a fresh
/// page — the caller should eret to retry. Returns `false` for any
/// fault we don't own.
pub fn try_handle(far: u64, esr: u64) -> bool {
    // EC is bits 31:26. We accept:
    //   0x24 — data abort from lower EL (user touched uncommitted page)
    //   0x25 — data abort from current EL (kernel uaccess hit
    //          uncommitted user page, e.g. pipe_buf::write copying
    //          from a user iov whose backing page hasn't been
    //          demand-committed yet)
    // We USED to also accept EC=0 with in-reservation FAR, but FAR
    // isn't reliably updated for EC=0, and that path caused a tight
    // infinite loop — the "data abort" FAR was actually stale from
    // a prior fault, so each eret re-crashed EC=0 on the original
    // (non-DA) issue and we re-entered demand_page forever,
    // exhausting the frame pool.
    let ec = (esr >> 26) & 0x3F;
    if ec != 0x24 && ec != 0x25 { return false; }
    // 🎯 STUMP #7 follow-on: only handle TRANSLATION faults (page not
    // mapped). Permission faults (DFSC 0x0d/0x0e/0x0f) come from a page
    // that IS mapped but with wrong perms (e.g., kernel uaccess writing
    // to a page just mprotect'd to R/O). Allocating a fresh frame would
    // not fix permissions; idempotency guard would then return Ok
    // without doing anything, the caller eret'd, and we'd re-fault on
    // the same VA forever (820k loops observed before alloc_frame OOM).
    let dfsc = esr & 0x3F;
    let is_translation_fault = matches!(dfsc, 0x04..=0x07);
    if !is_translation_fault { return false; }

    // Find the active cave's L1 phys from TTBR0_EL1.
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    // TTBR0 low bit is CnP; mask it off.
    let l1_phys = ttbr0 & !1u64;

    // Lookup.
    let _g = crate::kernel::sync::IrqGuard::new();
    let count = RESV_COUNT.load(Ordering::Acquire);
    let mut hit_idx: Option<usize> = None;
    for i in 0..count {
        let r = unsafe { core::ptr::read(core::ptr::addr_of!(RESV_TABLE[i])) };
        if r.l1_phys == l1_phys && far >= r.start && far < r.end {
            hit_idx = Some(i);
            break;
        }
    }
    if hit_idx.is_none() {
        return false;
    }

    // Allocate a fresh page + install it.
    let frame = match frame::alloc_frame() {
        Some(f) => f,
        None => {
            let (used, total) = frame::stats();
            uart::puts("[demand_page] OOM — frames used=");
            crate::kernel::mm::print_num(used);
            uart::puts(" total=");
            crate::kernel::mm::print_num(total);
            uart::puts(" committed_pages=");
            crate::kernel::mm::print_num(
                COMMITTED_PAGES.load(Ordering::Relaxed) as usize);
            uart::puts("\n");
            return false;
        }
    };
    // Zero the page so EL0 reads well-defined data (MAP_ANONYMOUS
    // Linux guarantee).
    unsafe {
        let p = frame as *mut u8;
        for i in 0..4096 { core::ptr::write_volatile(p.add(i), 0); }
    }

    let page_va = far & !0xFFFu64;
    let install_result = install_l3_mapping(l1_phys, page_va, frame as u64, USER_PAGE_FLAGS);
    if let Err(why) = install_result {
        uart::puts("[demand_page] install_l3 failed va=0x");
        let hex = b"0123456789abcdef";
        for sh in (0..16).rev() {
            uart::putc(hex[((page_va >> (sh * 4)) & 0xF) as usize]);
        }
        uart::puts(" reason: "); uart::puts(why); uart::puts("\n");
        // Leak the page — frame is still allocated but unmapped. Next
        // iteration will grab a fresh one. Small-leak < kernel crash.
        uart::puts("[demand_page] page-table install failed\n");
        return false;
    }

    // Invalidate TLB so the next EL0 access picks up the new L3 entry.
    // Use `vmalle1` (all stage-1, all ASIDs) as a sledgehammer while
    // we debug — per-VA `vaae1` was somehow not landing. Cost is a
    // full TLB flush per demand-paged fault; acceptable on the debug
    // loop, to be tightened later.
    unsafe {
        core::arch::asm!("dsb ishst");
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }


    let n = COMMITTED_PAGES.fetch_add(1, Ordering::Relaxed);
    // Rate-limited trace so we can at least see the count advance
    // without drowning the log on active workloads.
    if n < 5 || (n & 0xFF) == 0 {
        uart::puts("[dp] commit #");
        crate::kernel::mm::print_num(n as usize);
        uart::puts(" va=0x");
        let hex = b"0123456789abcdef";
        for i in (0..16).rev() {
            uart::putc(hex[((page_va >> (i * 4)) & 0xF) as usize]);
        }
        uart::puts("\n");
    }
    true
}

/// Walk/create L1→L2→L3 for `user_va` under the given cave L1, and
/// install an L3 entry mapping `user_va`'s 4 KB page to `phys_page`.
/// Allocates intermediate tables from the kernel frame pool on demand.
pub(crate) fn install_l3_mapping(
    l1_phys: u64,
    user_va: u64,
    phys_page: u64,
    flags: u64,
) -> Result<(), &'static str> {
    // 🎯 STUMP #9: hold IrqGuard across the entire walk-and-install so
    // we can't be preempted mid-install. Without this, sys_mmap or
    // sys_mprotect calling install_l3_mapping (Stump #4 / Stump #8
    // paths) can be interrupted by a timer IRQ that schedules another
    // thread; if that thread also lands in install_l3_mapping for the
    // same VA, we get conflicting allocations + L3 writes that show
    // up as PartitionAlloc CorruptionDetected later.
    let _g = crate::kernel::sync::IrqGuard::new();

    // T0SZ=25 gives a 39-bit VA: bits 38..30 → L1, 29..21 → L2, 20..12 → L3.
    let l1_idx = ((user_va >> 30) & 0x1FF) as usize;
    let l2_idx = ((user_va >> 21) & 0x1FF) as usize;
    let l3_idx = ((user_va >> 12) & 0x1FF) as usize;

    // Step 1: ensure L1[l1_idx] is a TABLE descriptor pointing to an L2.
    let l1_entry_ptr = (l1_phys + (l1_idx * 8) as u64) as *mut u64;
    let l1_entry = unsafe { core::ptr::read_volatile(l1_entry_ptr) };
    let l2_phys = if (l1_entry & TABLE_DESC) == TABLE_DESC {
        l1_entry & 0x0000_FFFF_FFFF_F000
    } else if l1_entry == 0 {
        // Allocate a new L2 table from the kernel pool.
        let t = frame::alloc_kernel_frame()
            .ok_or("demand_page: oom for L2 table")?;
        // Zero it.
        unsafe {
            let p = t as *mut u64;
            for i in 0..512 { core::ptr::write_volatile(p.add(i), 0); }
        }
        let desc = (t as u64) | TABLE_DESC;
        unsafe { core::ptr::write_volatile(l1_entry_ptr, desc); }
        // 🎯 STUMP #7 follow-on: flush the L1 entry's cache line + the
        // freshly-zeroed L2 table to PoC. Without this, the walker
        // can hit a stale (zero) cache line for L1[l1_idx] and re-fault
        // on the same VA → infinite alloc_frame loop → OOM.
        unsafe {
            core::arch::asm!("dc civac, {a}", a = in(reg) l1_entry_ptr as u64);
            let mut line = t as u64;
            while line < t as u64 + 4096u64 {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
            core::arch::asm!("dsb sy");
        }
        t as u64
    } else {
        // L1 entry is a BLOCK descriptor (e.g., the cave's user
        // window already lives here). We don't mess with those.
        return Err("demand_page: L1 entry is a block, not a table");
    };

    // Step 2: same for L2[l2_idx] → L3.
    let l2_entry_ptr = (l2_phys + (l2_idx * 8) as u64) as *mut u64;
    let l2_entry = unsafe { core::ptr::read_volatile(l2_entry_ptr) };
    let l3_phys = if (l2_entry & TABLE_DESC) == TABLE_DESC {
        l2_entry & 0x0000_FFFF_FFFF_F000
    } else if l2_entry == 0 {
        let t = frame::alloc_kernel_frame()
            .ok_or("demand_page: oom for L3 table")?;
        unsafe {
            let p = t as *mut u64;
            for i in 0..512 { core::ptr::write_volatile(p.add(i), 0); }
        }
        let desc = (t as u64) | TABLE_DESC;
        unsafe { core::ptr::write_volatile(l2_entry_ptr, desc); }
        // Same flush as the L1-create path above.
        unsafe {
            core::arch::asm!("dc civac, {a}", a = in(reg) l2_entry_ptr as u64);
            let mut line = t as u64;
            while line < t as u64 + 4096u64 {
                core::arch::asm!("dc civac, {a}", a = in(reg) line);
                line += 64;
            }
            core::arch::asm!("dsb sy");
        }
        t as u64
    } else {
        return Err("demand_page: L2 entry is a block, not a table");
    };

    // Step 3: write the L3 page entry.
    // NOTE: L3 page descriptors need bit 1 SET (page=1, block=0 is
    // undefined at L3). Our `flags` comes from USER_PAGE_FLAGS which
    // uses `PAGE_VALID = 0b11` — sets both bit 0 (valid) and bit 1
    // (page), so that part is correct.
    let l3_entry_ptr = (l3_phys + (l3_idx * 8) as u64) as *mut u64;
    let desc = (phys_page & 0x0000_FFFF_FFFF_F000) | flags;
    // IDEMPOTENCY GUARD: if the L3 entry is already valid, do NOT
    // overwrite it. A second install for the same VA would silently
    // leak the old physical frame AND clobber any user data the
    // caller wrote to it — exactly the symptom we saw with
    // `PartitionAlloc::CorruptionDetected()`. This can happen when
    // sys_mmap's Stump #4 path pre-installs a file-backed page and
    // a later spurious EC=0x24/0x25 fault routes through
    // `try_handle`, which would otherwise allocate a fresh zero
    // frame and overwrite the file content. Returning Ok here is
    // safe: the page is already mapped, so the caller's "retry"
    // (eret) will succeed.
    let existing = unsafe { core::ptr::read_volatile(l3_entry_ptr) };
    if (existing & PAGE_VALID) == PAGE_VALID {
        return Ok(());
    }
    unsafe { core::ptr::write_volatile(l3_entry_ptr, desc); }

    // Verify the write took effect — if this read shows a different
    // value, either our write is going somewhere else or the MMU's
    // table walker is hitting a cached stale entry.
    let readback = unsafe { core::ptr::read_volatile(l3_entry_ptr) };
    if readback != desc {
        uart::puts("[dp/install] mismatch write=0x");
        let hex = b"0123456789abcdef";
        for i in (0..16).rev() {
            uart::putc(hex[((desc >> (i * 4)) & 0xF) as usize]);
        }
        uart::puts(" read=0x");
        for i in (0..16).rev() {
            uart::putc(hex[((readback >> (i * 4)) & 0xF) as usize]);
        }
        uart::puts("\n");
    }

    // Flush the L3 entry's cache line to PoC so the table walker sees
    // the new value. Without this, the walk hits a stale line even
    // though we wrote via write_volatile.
    unsafe {
        core::arch::asm!("dc civac, {a}", a = in(reg) l3_entry_ptr as u64);
        core::arch::asm!("dsb sy");
    }

    Ok(())
}
