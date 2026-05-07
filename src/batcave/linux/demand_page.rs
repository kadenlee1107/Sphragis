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
    //   0x20 — instruction abort from lower EL (user fetched code
    //          from uncommitted page, e.g. V8 JIT'd code in cage area
    //          that needs lazy executable commit)
    //   0x21 — instruction abort from current EL (rare; treated same
    //          as 0x20)
    // We USED to also accept EC=0 with in-reservation FAR, but FAR
    // isn't reliably updated for EC=0, and that path caused a tight
    // infinite loop — the "data abort" FAR was actually stale from
    // a prior fault, so each eret re-crashed EC=0 on the original
    // (non-DA) issue and we re-entered demand_page forever,
    // exhausting the frame pool.
    let ec = (esr >> 26) & 0x3F;
    let is_data_abort = ec == 0x24 || ec == 0x25;
    let is_inst_abort = ec == 0x20 || ec == 0x21;
    if !is_data_abort && !is_inst_abort { return false; }
    // 🎯 STUMP #7 follow-on: only handle TRANSLATION faults (page not
    // mapped). Permission faults (DFSC 0x0d/0x0e/0x0f) come from a page
    // that IS mapped but with wrong perms (e.g., kernel uaccess writing
    // to a page just mprotect'd to R/O). Allocating a fresh frame would
    // not fix permissions; idempotency guard would then return Ok
    // without doing anything, the caller eret'd, and we'd re-fault on
    // the same VA forever (820k loops observed before alloc_frame OOM).
    let dfsc = esr & 0x3F;
    let is_translation_fault = matches!(dfsc, 0x04..=0x07);
    // 🎯 STUMP #25 helper: also accept permission faults (DFSC 0x0d/0x0e/0x0f)
    // when they're instruction aborts. The page is already mapped (with UXN
    // set from a previous data-fault commit) but execution faults. Flip
    // UXN off below by re-installing with the inst-abort flags.
    let is_perm_fault_inst_abort = is_inst_abort
        && matches!(dfsc, 0x0d..=0x0f);
    // 🎯 STUMP #160 iter 2: data-abort permission faults inside a
    // registered V8/PA reservation. Chromium reserves huge regions
    // PROT_NONE then accesses them — sometimes BEFORE issuing an
    // mprotect (PA's pool-init pre-touches header bytes for layout
    // verification). The L3 entry exists (committed by an earlier
    // demand_page) but with no-EL0-access perms. We're in a
    // registered reservation OR in plausible-V8 VA range; upgrading
    // the perms to RW matches the "huge reserve, lazy commit"
    // semantics we already pretend to honour. Without this every
    // V8 init access ends up as SIGSEGV → thread retire and the
    // browser slowly bleeds threads.
    let is_perm_fault_data_abort = is_data_abort
        && matches!(dfsc, 0x0d..=0x0f);
    if !is_translation_fault
        && !is_perm_fault_inst_abort
        && !is_perm_fault_data_abort
    {
        return false;
    }

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
        // 🎯 No registered reservation matches. Fall back to lazy
        // commit anyway IF the FAR is in plausible user-VA range.
        // The cave's normal user window is 0x10000000..0x1c800000;
        // anything BEYOND content_shell BSS (0x1a224e58) but BELOW
        // kernel range (0x80_0000_0000) is most likely V8 cage /
        // cppgc heap that V8 chose without going through our
        // reserve-only mmap path.
        //
        // Reject NULL+small, cave-main-window content (where a fault
        // is a real bug), kernel range, etc.
        let far_plausible = far >= 0x2000_0000 && far < 0x80_0000_0000
            // Don't commit pages in cave's main window (those are
            // either binary content or unmapped padding).
            && !(far >= 0x10000000 && far < 0x1c800000);
        if !far_plausible {
            // STUMP #161: diagnostic for low-VA faults. If we're
            // seeing repeated faults at the SAME low-VA, log the
            // user PC (lr_at_fault) so we can identify which
            // library/function is making the access. Limit to 5
            // logs per boot to avoid log flood.
            use core::sync::atomic::{AtomicU32, Ordering as DOrd};
            static LOW_VA_LOG: AtomicU32 = AtomicU32::new(0);
            let n = LOW_VA_LOG.fetch_add(1, DOrd::Relaxed);
            if n < 5 && far < 0x10000000 {
                let elr_now: u64;
                unsafe {
                    core::arch::asm!("mrs {}, elr_el1", out(reg) elr_now);
                }
                uart::puts("[dp/low-va #");
                crate::kernel::mm::print_num(n as usize);
                uart::puts("] far=0x");
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((far >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" elr=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts("\n");
            }
            return false;
        }
        // Treat as virtual-reservation hit: drop through to alloc
        // a frame and install_l3_mapping for this VA.
    }

    // 🎯 STUMP #25 helper: for permission-fault instruction aborts,
    // the page is already mapped but with UXN set. Just flip UXN off
    // on the existing L3 entry — no new frame needed.
    if is_perm_fault_inst_abort {
        let page_va = far & !0xFFFu64;
        // Walk L1 -> L2 -> L3 to find the existing entry.
        let l1_idx = ((page_va >> 30) & 0x1FF) as u64;
        let l1_ent_addr = l1_phys + l1_idx * 8;
        let l1_ent = unsafe { core::ptr::read_volatile(l1_ent_addr as *const u64) };
        if (l1_ent & 0b11) != 0b11 { return false; }
        let l2_phys = l1_ent & 0x0000_FFFF_FFFF_F000;
        let l2_idx = ((page_va >> 21) & 0x1FF) as u64;
        let l2_ent_addr = l2_phys + l2_idx * 8;
        let l2_ent = unsafe { core::ptr::read_volatile(l2_ent_addr as *const u64) };
        if (l2_ent & 0b11) != 0b11 { return false; }
        let l3_phys = l2_ent & 0x0000_FFFF_FFFF_F000;
        let l3_idx = ((page_va >> 12) & 0x1FF) as u64;
        let l3_ent_addr = l3_phys + l3_idx * 8;
        let l3_ent = unsafe { core::ptr::read_volatile(l3_ent_addr as *const u64) };
        if (l3_ent & PAGE_VALID) != PAGE_VALID { return false; }
        // Clear UXN bit (bit 54).
        let new_ent = l3_ent & !PAGE_UXN;
        unsafe {
            core::ptr::write_volatile(l3_ent_addr as *mut u64, new_ent);
            core::arch::asm!("dc civac, {a}", a = in(reg) l3_ent_addr);
            core::arch::asm!("dsb ishst");
            core::arch::asm!("tlbi vaae1is, {a}", a = in(reg) page_va >> 12);
            core::arch::asm!("dsb ish");
            core::arch::asm!("isb");
        }
        return true;
    }

    // 🎯 STUMP #160 iter 2: data-abort permission fault inside a
    // V8/PA-style reservation. The L3 entry exists with restrictive
    // perms (AP=0b00 EL1-only, or AP=0b11 EL0-RO when the access
    // was a write). Upgrade to USER_PAGE_FLAGS (EL0 RW, no-exec)
    // on the EXISTING phys frame — no new allocation, no data loss.
    //
    // Only do this when EITHER:
    //   - hit_idx is Some (we're inside a registered reservation), OR
    //   - the FAR is in plausible-V8 range (already vetted above).
    // Otherwise we'd be silently widening permissions on legitimate
    // PROT_NONE pages that the user explicitly wanted to be NOT
    // accessible — breaking guard pages, stack red-zones, etc.
    if is_perm_fault_data_abort {
        let page_va = far & !0xFFFu64;
        let l1_idx = ((page_va >> 30) & 0x1FF) as u64;
        let l1_ent_addr = l1_phys + l1_idx * 8;
        let l1_ent = unsafe { core::ptr::read_volatile(l1_ent_addr as *const u64) };
        if (l1_ent & 0b11) != 0b11 { return false; }
        let l2_phys = l1_ent & 0x0000_FFFF_FFFF_F000;
        let l2_idx = ((page_va >> 21) & 0x1FF) as u64;
        let l2_ent_addr = l2_phys + l2_idx * 8;
        let l2_ent = unsafe { core::ptr::read_volatile(l2_ent_addr as *const u64) };
        if (l2_ent & 0b11) != 0b11 { return false; }
        let l3_phys = l2_ent & 0x0000_FFFF_FFFF_F000;
        let l3_idx = ((page_va >> 12) & 0x1FF) as u64;
        let l3_ent_addr = l3_phys + l3_idx * 8;
        let l3_ent = unsafe { core::ptr::read_volatile(l3_ent_addr as *const u64) };
        if (l3_ent & PAGE_VALID) != PAGE_VALID { return false; }
        // Preserve the physical frame address; replace AP/PXN/UXN/AF/SH
        // bits with the standard user-RW pattern.
        const MASK_FRAME: u64 = 0x0000_FFFF_FFFF_F000;
        const MASK_ATTR:  u64 = 0b111 << 2;
        let kept = l3_ent & (MASK_FRAME | MASK_ATTR);
        let new_ent = kept | USER_PAGE_FLAGS;
        unsafe {
            core::ptr::write_volatile(l3_ent_addr as *mut u64, new_ent);
            core::arch::asm!("dc civac, {a}", a = in(reg) l3_ent_addr);
            core::arch::asm!("dsb ishst");
            core::arch::asm!("tlbi vaae1is, {a}", a = in(reg) page_va >> 12);
            core::arch::asm!("dsb ish");
            core::arch::asm!("isb");
        }
        return true;
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
    // DIAGNOSTIC RESULT (STUMP #29 investigation, run 230800):
    // Tested with 0xA5 fill — cave died at 437 lines with
    // `fault=0xa5a5a5a5a5a5a5ad` from a deref of our 0xA5 pattern as
    // a pointer. This proves: cave reads OUR page contents, no
    // aliasing or page-table bug. So PA's "corruption" must come
    // from PA's expected free-slot metadata not matching our zeros.
    // Reverting to 0 — that's the Linux MAP_ANONYMOUS contract and
    // Chromium handles NULL pointers in C code far better than 0xA5.
    unsafe {
        let p = frame as *mut u8;
        for i in 0..4096 { core::ptr::write_volatile(p.add(i), 0); }
        // 🎯 STUMP #10c FINAL: clean every cache line we wrote to PoC.
        // We zeroed via the kernel identity map (EL1). EL0 will read
        // the same physical frame via the user VA we're about to install.
        // ARM64 normal memory is supposed to be coherent in the inner-
        // shareable domain, but the small_mmap region (0x70_0000_0000+)
        // is the hot path and stale cache lines from prior PA usage
        // can persist. PartitionAlloc's InSlotMetadata refcount check
        // (`ldar w8, [x24]; cmp w27, #0x1`) reads stale data → fails →
        // CorruptionDetected BRK.
        let mut line = frame as u64;
        while line < frame as u64 + 4096 {
            core::arch::asm!("dc civac, {a}", a = in(reg) line);
            line += 64;
        }
        core::arch::asm!("dsb sy");
    }

    let page_va = far & !0xFFFu64;
    // 🎯 STUMP #24: when the fault is an instruction abort (EC=0x20/0x21),
    // commit the page WITHOUT UXN so it can be executed. V8's JIT writes
    // code into a region and starts executing without an explicit mprotect
    // when the cage is allocated as a single big reservation. Letting the
    // page commit executable is safer than spurious-failing the fault.
    let page_flags = if is_inst_abort {
        USER_PAGE_FLAGS & !PAGE_UXN
    } else {
        USER_PAGE_FLAGS
    };
    let install_result = install_l3_mapping(l1_phys, page_va, frame as u64, page_flags);
    if let Err(why) = install_result {
        uart::puts("[demand_page] install_l3 failed va=0x");
        let hex = b"0123456789abcdef";
        for sh in (0..16).rev() {
            uart::putc(hex[((page_va >> (sh * 4)) & 0xF) as usize]);
        }
        uart::puts(" reason: "); uart::puts(why);
        // 🎯 STUMP #38 diagnostic: when alloc_kernel_frame returns None,
        // print pool stats so we can tell whether it's actually exhausted
        // or whether the new top-down scan logic has a bug.
        let (used, total) = frame::stats();
        uart::puts(" frames used=");
        crate::kernel::mm::print_num(used);
        uart::puts(" total=");
        crate::kernel::mm::print_num(total);
        uart::puts(" committed=");
        crate::kernel::mm::print_num(
            COMMITTED_PAGES.load(Ordering::Relaxed) as usize);
        uart::puts("\n");
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
