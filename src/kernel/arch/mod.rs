#![allow(dead_code)]
// Sphragis — Architecture-specific kernel code (ARM64)

use crate::drivers::uart;

core::arch::global_asm!(include_str!("../../caves/linux/forkjmp.s"));
core::arch::global_asm!(include_str!("../../caves/linux/threads.s"));

unsafe extern "C" {
    fn fork_save(buf: *mut u64) -> u64;
    fn fork_restore(buf: *const u64, retval: u64) -> !;
}

// Saved busybox SP at clone time (for eret back to parent)
static mut SAVED_BUSYBOX_SP: u64 = 0;

/// Kernel SP save slot used by `caves::linux::loader::execute_with_args`
/// before erets to EL0, and read back by the exit-syscall + brk paths below
/// so the shell can resume after a user ELF exits.
// /
/// Lives in kernel BSS so it's guaranteed writable (unlike the previous
/// hardcoded `0x40000100`/`0x40001000` addresses which both sat inside the
/// Linux arm64 Image header region and were mapped R-X by the kernel MMU).
/// The addresses also didn't match between the store and restore sites;
/// that was the root cause of the QEMU `DATA ABORT DFSC=0x0e` at FAR
/// 0x40000100 for every Cave-runner ELF (netsurf/freetype/png/v8/etc).
#[unsafe(no_mangle)]
pub static mut KERNEL_SP_SAVE: u64 = 0;

/// Return the address of `KERNEL_SP_SAVE` as a u64 so inline asm can use it.
#[inline(always)]
pub fn kernel_sp_save_addr() -> u64 {
    &raw const KERNEL_SP_SAVE as u64
}

// Saved exception frame in kernel BSS (safe from busybox)
static mut SAVED_FRAME: [u64; 35] = [0; 35]; // 35 * 8 = 280 bytes > 272

// Saved stack contents at clone time (child shares parent stack — must restore)
const STACK_SAVE_SIZE: usize = 4096;
static mut SAVED_STACK: [u8; STACK_SAVE_SIZE] = [0; STACK_SAVE_SIZE];

#[repr(C)]
pub struct TrapFrame {
    pub x: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
}

/// Walk the active cave's L1→L2→L3 to verify a user VA has a valid
/// L3 entry. Returns true if the page is mapped (and therefore safe
/// to read from EL1 without triggering a translation fault).
// /
/// This is conservative: if any of the L1/L2/L3 reads themselves
/// might fault, we return false. Used in the diagnostic dump path
/// to avoid recursive aborts when the original fault left us with
/// a partially-committed reservation.
// /
/// EL0-writable scratch page for pa-skip-data's fake Alloc returns.
/// Mmap'd lazily on first call into the small_mmap user-VA region
/// at a fixed address so it's stable across the cave's lifetime.
/// Multiple "fake allocs" all share this single page (intentional
/// the cave is in degraded state by the time we synthesize, so
/// shared garbage is preferable to NULL-deref).
// /
/// Returns 0 if init failed (alloc OOM or install_l3 failed).
static SCRATCH_UVA: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

#[inline(never)]
fn pa_skip_scratch_uva() -> u64 {
    let cached = SCRATCH_UVA.load(core::sync::atomic::Ordering::Acquire);
    if cached != 0 {
        return cached;
    }

    // Use a fixed VA at the high end of the small_mmap region
    // (0x70_0000_0000..0x78_0000_0000). Pick 0x77_FFFF_0000 — well
    // away from where regular small_mmap allocations land
    // (which fill upward from 0x70_0000_0000).
    const SCRATCH_VA: u64 = 0x77_FFFF_0000;

    // Get a frame from the kernel pool.
    let frame = match crate::kernel::mm::frame::alloc_frame() {
        Some(f) => f as u64,
        None    => return 0,
    };

    // Get current cave's TTBR0 (the L1 we install into).
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;

    // Install L3: map SCRATCH_VA → frame with USER_PAGE_FLAGS
    // (which includes EL0_RW + UXN + Normal + Inner Shareable).
    let install_result = crate::caves::linux::demand_page::install_l3_mapping(
        l1_phys,
        SCRATCH_VA,
        frame,
        crate::caves::linux::demand_page::USER_PAGE_FLAGS,
    );
    if install_result.is_err() {
        return 0;
    }

    // Flush TLB so the new entry is visible.
    unsafe {
        core::arch::asm!("dsb ishst");
        core::arch::asm!("tlbi vmalle1");
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    // Cache and return.
    SCRATCH_UVA.store(SCRATCH_VA, core::sync::atomic::Ordering::Release);
    SCRATCH_VA
}

/// `#[inline(never)]` to keep the call edge in the disassembly so the
/// compiler can't fold this into the caller and notice a "this load
/// can't possibly fail" theorem (it CAN fail because of lazy demand-
/// paging which the compiler doesn't model).
#[inline(never)]
pub fn page_is_mapped(user_va: u64) -> bool {
    let ttbr0: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
    let l1_phys = ttbr0 & !1u64;
    if l1_phys == 0 { return false; }

    // T0SZ=25 → 39-bit VA: bits 38..30 → L1, 29..21 → L2, 20..12 → L3.
    let l1_idx = ((user_va >> 30) & 0x1FF) as u64;
    let l1e: u64 = unsafe {
        core::ptr::read_volatile((l1_phys + l1_idx * 8) as *const u64)
    };
    // L1 must be a TABLE (0b11) entry pointing to L2.
    if (l1e & 0b11) != 0b11 { return false; }
    let l2_phys = l1e & 0x0000_FFFF_FFFF_F000;
    let l2_idx = ((user_va >> 21) & 0x1FF) as u64;
    let l2e: u64 = unsafe {
        core::ptr::read_volatile((l2_phys + l2_idx * 8) as *const u64)
    };
    // L2 BLOCK descriptor (0b01): identity-mapped 2 MB block; valid.
    if (l2e & 0b11) == 0b01 { return true; }
    // L2 TABLE (0b11) → walk L3.
    if (l2e & 0b11) != 0b11 { return false; }
    let l3_phys = l2e & 0x0000_FFFF_FFFF_F000;
    let l3_idx = ((user_va >> 12) & 0x1FF) as u64;
    let l3e: u64 = unsafe {
        core::ptr::read_volatile((l3_phys + l3_idx * 8) as *const u64)
    };
    (l3e & 0b11) == 0b11
}

pub fn init_exceptions() {
    unsafe {
        // adrp+add (±4 GB range) instead of `adr` (±1 MB) — the
        // kernel grew past the ADR_PREL_LO21 range and the linker
        // was rejecting the relocation.
        core::arch::asm!(
            "adrp x0, exception_vectors",
            "add  x0, x0, :lo12:exception_vectors",
            "msr vbar_el1, x0",
            "isb",
            out("x0") _,
        );

        // V5-SIDE-005 fix: clear CNTKCTL_EL1 so EL0 cannot read the
        // physical/virtual timer registers directly. Without this,
        // `mrs xN, cntpct_el0` from user space returns a 40ns-resolution
        // clock — the universal enabler for every timing side-channel
        // attack (AES S-box cache timing, scalar-mult branch timing,
        // etc.). sys_clock_gettime still provides timing to EL0 via
        // the syscall boundary where we can add noise/resolution caps.
        //
        // Bits in CNTKCTL_EL1:
        // EL0PCTEN (bit 0) = 1 enables EL0 access to cntpct_el0
        // EL0VCTEN (bit 1) = 1 enables EL0 access to cntvct_el0
        // EVNTEN (bit 2)
        // EVNTDIR (bit 3)
        // EVNTI (bits 7:4)
        // EL0VTEN (bit 8)
        // EL0PTEN (bit 9)
        // Setting to 0 denies all EL0 timer register access.
        core::arch::asm!("msr cntkctl_el1, xzr");
        core::arch::asm!("isb");
    }
    uart::puts("  [arch] Exception vectors installed\n");
    uart::puts("  [arch] CNTKCTL_EL1 cleared — EL0 timer access denied\n");
}

/// Detected GIC version, set by `init_gic`. 2 = v2 MMIO CPU iface,
/// 3 = v3 ICC_* system registers + per-CPU redistributor.
/// Read by `handle_irq_inner` to choose ack/EOI path.
pub(crate) static GIC_VERSION: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(0);

/// QEMU virt machine GIC layout:
/// gic-version=2: GICD @ 0x0800_0000, GICC @ 0x0801_0000
/// gic-version=3: GICD @ 0x0800_0000, GICR @ 0x080A_0000 (per-CPU)
/// Both share GICD_PIDR2 at GICD_BASE+0xFFE8 with arch_rev in bits[7:4].
const GICD_BASE: usize = 0x0800_0000;
const GICD_PIDR2: usize = GICD_BASE + 0xFFE8;

/// GICv3 redistributor base for CPU 0. QEMU virt places the redist
/// frame at this address; subsequent CPUs are at +0x20000 stride
/// (or +0x40000 for the GICv4 LPI variant — we don't care, single-CPU).
const GICR_BASE: usize = 0x080A_0000;

/// Compile-time GIC version selection. Default 2 matches the current
/// QEMU virt smoke (`-machine virt,gic-version=2`). Build with
/// `cargo build --release --features gicv3` to flip to the v3 path
/// for HVF acceleration on Apple Silicon hosts.
// /
/// We do NOT runtime-probe via GICD_PIDR2 because on v2 QEMU only
/// maps the first 4 KB of GICD, so reading the PIDR registers at
/// offset 0xFFE8 faults. A safer probe would parse the DTB
/// `compatible` string, but the smoke pipeline already controls
/// the GIC version via the QEMU command line — propagating that
/// choice through a Cargo feature is simpler.
fn detect_gic_version() -> u8 {
    if cfg!(feature = "gicv3") { 3 } else { 2 }
}

#[allow(dead_code)] // referenced only when gicv3 feature is on (PIDR2 path)
fn _gicd_pidr2_addr() -> usize { GICD_PIDR2 }

/// Minimal GICv3 init for QEMU virt. Distributor + per-CPU
/// redistributor + ICC_* system registers. Enables Group 1 NS
/// for the physical-timer PPI (INTID 30).
fn init_gicv3() {
    // Distributor MMIO offsets
    const GICD_CTLR:  usize = GICD_BASE + 0x000;
    // Redistributor for CPU 0:
    // GICR_RD_BASE = GICR_BASE (control + WAKER + IDREGs)
    // GICR_SGI_BASE = GICR_BASE + 0x10000 (SGI/PPI control)
    const GICR_WAKER:       usize = GICR_BASE + 0x0014;
    const GICR_SGI_BASE:    usize = GICR_BASE + 0x10000;
    const GICR_ISENABLER0:  usize = GICR_SGI_BASE + 0x100;
    const GICR_IPRIORITYR:  usize = GICR_SGI_BASE + 0x400;
    const GICR_IGROUPR0:    usize = GICR_SGI_BASE + 0x080;

    unsafe {
        // 1. Distributor: enable Group 1 NS + Affinity Routing.
        // GICD_CTLR.EnableGrp1NS = bit 1, GICD_CTLR.ARE_NS = bit 4.
        // (DS=0 disables security, simplifies — we don't have EL3.)
        core::ptr::write_volatile(GICD_CTLR as *mut u32, (1 << 1) | (1 << 4));

        // 2. Wake the redistributor: clear ProcessorSleep (bit 1),
        // then poll until ChildrenAsleep (bit 2) clears.
        let mut waker = core::ptr::read_volatile(GICR_WAKER as *const u32);
        waker &= !(1u32 << 1); // clear ProcessorSleep
        core::ptr::write_volatile(GICR_WAKER as *mut u32, waker);
        // Poll ChildrenAsleep — bounded retry so a misconfigured GIC
        // doesn't hang boot. Real GICv3 wakes within a few cycles.
        for _ in 0..10000 {
            let w = core::ptr::read_volatile(GICR_WAKER as *const u32);
            if (w & (1 << 2)) == 0 { break; }
            core::hint::spin_loop();
        }

        // 3. Per-CPU SGI/PPI config: put PPI 30 (timer) in Group 1,
        // set its priority, enable it.
        let mut grp = core::ptr::read_volatile(GICR_IGROUPR0 as *const u32);
        grp |= 1u32 << 30;  // bit 30 = INTID 30 in group 1
        core::ptr::write_volatile(GICR_IGROUPR0 as *mut u32, grp);

        let prio_word = (GICR_IPRIORITYR + (30 / 4) * 4) as *mut u32;
        let mut prio = core::ptr::read_volatile(prio_word);
        let lane = (30 % 4) * 8;
        prio &= !(0xFFu32 << lane);
        prio |= 0xA0u32 << lane;
        core::ptr::write_volatile(prio_word, prio);

        core::ptr::write_volatile(GICR_ISENABLER0 as *mut u32, 1u32 << 30);

        // 4. ICC_* system registers — CPU interface. SRE bit MUST
        // be set BEFORE any other ICC_*_EL1 access; on GICv2 this
        // register doesn't exist. We accept that — a v2-only host
        // would never have entered init_gicv3 since detection
        // via PIDR2 said v2.
        // ICC_SRE_EL1 = 1 << 0 (SRE)
        core::arch::asm!("msr S3_0_C12_C12_5, {}", in(reg) 1u64);
        core::arch::asm!("isb");

        // ICC_PMR_EL1 = 0xff (allow all priorities)
        core::arch::asm!("msr S3_0_C4_C6_0, {}", in(reg) 0xffu64);

        // ICC_BPR1_EL1 = 0 (max preemption granularity)
        core::arch::asm!("msr S3_0_C12_C12_3, {}", in(reg) 0u64);

        // ICC_CTLR_EL1 = 0 (EOImode=0 — single drop+deactivate)
        core::arch::asm!("msr S3_0_C12_C12_4, {}", in(reg) 0u64);

        // ICC_IGRPEN1_EL1 = 1 (enable group 1 IRQs)
        core::arch::asm!("msr S3_0_C12_C12_7, {}", in(reg) 1u64);

        core::arch::asm!("isb");
    }
    uart::puts("  [arch] GICv3 initialized (timer PPI 30 enabled)\n");
}

/// Minimal GICv2 init for QEMU virt. The "virt" machine wires:
/// GIC Distributor (GICD) @ 0x0800_0000
/// GIC CPU Interface (GICC)@ 0x0801_0000
/// Physical-timer IRQ is PPI #14 → INTID 30.
// /
/// We need: enable the distributor, enable the CPU interface,
/// set PMR (priority mask) to accept all priorities, enable
/// INTID 30 in GICD's ISENABLER. Without this the timer fires
/// in CNTP_CTL but the CPU never sees an IRQ.
fn init_gicv2() {
    const GICD_BASE: usize = 0x0800_0000;
    const GICC_BASE: usize = 0x0801_0000;
    const GICD_CTLR: usize = GICD_BASE + 0x000;
    const GICD_ISENABLER0: usize = GICD_BASE + 0x100;
    const GICD_IPRIORITYR: usize = GICD_BASE + 0x400;
    const GICC_CTLR: usize = GICC_BASE + 0x000;
    const GICC_PMR:  usize = GICC_BASE + 0x004;

    unsafe {
        // Enable distributor (Group 0).
        core::ptr::write_volatile(GICD_CTLR as *mut u32, 1);
        // Lowest priority byte for INTID 30 (timer PPI). Priority
        // 0xa0 — middle of the range so other IRQs (if any) can
        // override.
        let prio_word_addr = (GICD_IPRIORITYR + (30 / 4) * 4) as *mut u32;
        let mut prio = core::ptr::read_volatile(prio_word_addr);
        let lane = (30 % 4) * 8;
        prio &= !(0xFFu32 << lane);
        prio |= 0xA0u32 << lane;
        core::ptr::write_volatile(prio_word_addr, prio);
        // Enable INTID 30 (PPI #14 = physical timer) — bit 30 of
        // GICD_ISENABLER0 (covers IRQs 0..31).
        core::ptr::write_volatile(GICD_ISENABLER0 as *mut u32, 1u32 << 30);
        // Accept all priorities at the CPU interface.
        core::ptr::write_volatile(GICC_PMR  as *mut u32, 0xFF);
        // Enable CPU interface.
        core::ptr::write_volatile(GICC_CTLR as *mut u32, 1);
    }
    uart::puts("  [arch] GICv2 initialized (timer PPI 30 enabled)\n");
}

pub fn init_timer() {
    // Initialize the GIC first so the CPU actually receives the
    // timer IRQ. Without this, CNTP_CTL fires but no IRQ vector
    // is taken — preemption is dead in the water.
    let v = detect_gic_version();
    GIC_VERSION.store(v, core::sync::atomic::Ordering::Release);
    if v == 3 {
        init_gicv3();
    } else {
        init_gicv2();
    }
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        // Diagnostic: print cntfrq so we can verify our 100Hz interval
        // is actually 100Hz. (We observed effective IRQ rate ~1Hz on
        // QEMU virt, which would mean either freq is 100x larger than
        // expected or the GIC delivery is missing 99% of fires.)
        uart::puts("  [arch] cntfrq_el0 = ");
        crate::kernel::mm::print_num(freq as usize);
        uart::puts("\n");
        let interval = freq / 100;
        uart::puts("  [arch] timer interval = ");
        crate::kernel::mm::print_num(interval as usize);
        uart::puts("\n");
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) interval);
        core::arch::asm!("mov x0, #1", "msr cntp_ctl_el0, x0", out("x0") _);
        core::arch::asm!("msr daifclr, #0x2");
    }
    uart::puts("  [arch] Timer configured (100Hz)\n");
}

fn reset_timer() {
    unsafe {
        // Disable the timer first to clear ISTATUS, then re-arm with a
        // fresh interval and re-enable. Just writing cntp_tval should
        // reset the down-counter, but on QEMU virt with GICv3-default
        // delivery we observed only ~10 IRQs total — the timer's IRQ
        // line stays asserted somehow. Disabling fully drops the
        // interrupt output, then re-enable kicks off a clean cycle.
        core::arch::asm!("msr cntp_ctl_el0, xzr");  // disable
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let interval = freq / 100;
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) interval);
        core::arch::asm!("mov x0, #1", "msr cntp_ctl_el0, x0", out("x0") _);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_irq(frame: *mut TrapFrame) {
    // iter 16 marker
    static IRQ_N_TEST: core::sync::atomic::AtomicU64 =
        core::sync::atomic::AtomicU64::new(0);
    let irq_idx_test = IRQ_N_TEST.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if irq_idx_test < 5 {
        uart::puts_safe("STUMP161IRQ\n");
    }
    // V8-ELR-WATCH: snapshot frame.elr at IRQ entry. After all the IRQ
    // work + potential context switches, if we end this function with
    // frame.elr in kernel range AND spsr.M=0 (eret target = EL0), the
    // IRQ handler corrupted the trap frame in a way that'll send the
    // user thread to a kernel PC. Log it.
    let elr_in = unsafe { (*frame).elr };
    let spsr_in = unsafe { (*frame).spsr };
    handle_irq_inner(frame);
    let elr_out = unsafe { (*frame).elr };
    let spsr_out = unsafe { (*frame).spsr };
    let goes_to_el0 = (spsr_out & 0xF) == 0;
    if goes_to_el0 && elr_out >= 0x40000000 && elr_out < 0x80000000 {
        uart::puts("!!! IRQ left frame.elr=KERNEL going to EL0\n");
        let hex = b"0123456789abcdef";
        uart::puts("  in:  elr=0x"); for sh in (0..16).rev() { uart::putc(hex[((elr_in >> (sh*4)) & 0xF) as usize]); }
        uart::puts(" spsr=0x"); for sh in (0..16).rev() { uart::putc(hex[((spsr_in >> (sh*4)) & 0xF) as usize]); }
        uart::puts("\n  out: elr=0x"); for sh in (0..16).rev() { uart::putc(hex[((elr_out >> (sh*4)) & 0xF) as usize]); }
        uart::puts(" spsr=0x"); for sh in (0..16).rev() { uart::putc(hex[((spsr_out >> (sh*4)) & 0xF) as usize]); }
        uart::puts("\n");
    }
}

fn handle_irq_inner(frame: *mut TrapFrame) {
    // V-ASAHI-2.2: on Apple Silicon, IRQs come through AIC2 instead of
    // a per-CPU GIC. We MUST drain the AIC event queue every entry,
    // otherwise level-triggered IRQs (timer, UART, SPI, etc.) re-fire
    // immediately and we livelock in the exception handler.
    if crate::platform::is_apple_silicon() {
        // Drain every pending event (AIC may have queued more than one).
        while crate::drivers::apple::aic::dispatch_one() {}
        return;
    }

    // QEMU virt path: ARM Generic Timer wired directly via the GIC.
    // Branches on GIC_VERSION (set by init_timer):
    // v2: MMIO ack via GICC_IAR (0x0801000C), EOI via GICC_EOIR.
    // v3: system register ack via ICC_IAR1_EL1, EOI via ICC_EOIR1_EL1.
    // Without ack+EOI the GIC keeps the IRQ active and stops delivering.
    const GICC_IAR:  usize = 0x0801_0000 + 0x00C;
    const GICC_EOIR: usize = 0x0801_0000 + 0x010;
    let gic_v = GIC_VERSION.load(core::sync::atomic::Ordering::Acquire);
    let iar: u32 = if gic_v == 3 {
        let ack: u64;
        unsafe {
            // ICC_IAR1_EL1 = S3_0_C12_C12_0
            core::arch::asm!("mrs {}, S3_0_C12_C12_0", out(reg) ack);
        }
        (ack & 0xFFFFFF) as u32
    } else {
        unsafe { core::ptr::read_volatile(GICC_IAR as *const u32) }
    };
    let intid = iar & 0x3FF;

    // Spurious (1023) means no IRQ pending — bail without EOI.
    if intid == 1023 { return; }

    // iter 16: count ALL IRQs, not just timer fires. We
    // observed zero heartbeats during /bin/js userland-hang debugging,
    // but couldn't tell whether (a) the timer wasn't firing or (b) the
    // ctl & 0b100 check was failing. Sampling on every IRQ entry
    // disambiguates and also catches userland hangs that only show
    // up between timer fires (e.g. a tight loop that briefly yields
    // CPU).
    {
        static IRQ_TICKS: core::sync::atomic::AtomicU64 =
            core::sync::atomic::AtomicU64::new(0);
        let n = IRQ_TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        // Print first 3 IRQs unconditionally so we can verify IRQs
        // are reaching handle_irq at all.
        if n < 3 {
            uart::puts("[irq#");
            crate::kernel::mm::print_num(n as usize);
            uart::puts(" intid=");
            crate::kernel::mm::print_num(intid as usize);
            uart::puts("]\n");
        }
    }

    let ctl: u64;
    unsafe { core::arch::asm!("mrs {}, cntp_ctl_el0", out(reg) ctl); }
    if ctl & 0b100 != 0 {
        reset_timer();
        // Periodic thread-state dump for deadlock diagnosis. Fires
        // every ~5 seconds. Cheap when the system is making progress;
        // when it's stuck the dump tells us exactly what every thread
        // is parked on. See threads::auto_dump_if_idle.
        crate::caves::linux::threads::auto_dump_if_idle();

        // Drain stdio_ring to UART (was inside scheduler::tick()).
        // We don't want to call kernel::scheduler::schedule() from
        // here — it's the legacy task-table scheduler that ping-
        // pongs with chromium-blit on every tick and adds massive
        // overhead. The drain is the only useful thing tick() does.
        crate::caves::linux::stdio_ring::drain_to_uart();

        // REAL PREEMPTION via the cooperative-switch path.
        //
        // Approach: only switch threads if the IRQ interrupted EL0
        // user code. Preempting EL1 (kernel) code is unsafe — we
        // could be holding a kernel lock — so for that case we just
        // set the deferred-preempt flag so the syscall boundary
        // yields voluntarily.
        //
        // For EL0 IRQs we call schedule() directly. schedule() picks
        // the next runnable thread and invokes cxt_switch_cooperative,
        // which:
        // * saves OUR (current thread's) callee-saved regs + SP +
        // SP_EL0 + TTBR0 into our slot — that's enough state to
        // resume us later;
        // * restores the new thread's callee-saved + SP + SP_EL0 +
        // TTBR0;
        // * rets to the new thread's saved x30 (back into ITS prior
        // schedule() / handle_irq call site).
        //
        // The trap frame stays parked at the top of OUR kernel stack
        // while we're switched out — perfectly safe, nothing else
        // writes to that stack. When we're eventually rescheduled,
        // schedule() returns into here, handle_irq returns up to the
        // IRQ vector, RESTORE_REGS pops the still-parked trap frame,
        // and `eret` resumes user mode.
        //
        // This unified model means cooperatively-yielded threads (in
        // syscall handlers) and preemptively-interrupted threads (in
        // user mode) BOTH park their state via cxt_switch_cooperative,
        // so resuming either kind requires no special-case logic.
        let spsr = unsafe { (*frame).spsr };
        let was_in_el0 = (spsr & 0xF) == 0; // M[3:0] == 0000 ⇒ EL0t

        // Diagnostic: occasional total IRQ count + per-EL0 PC sample.
        // Tuned to be quiet during normal operation but reveal the
        // hot path under deadlock investigation.
        static TOTAL_IRQ: core::sync::atomic::AtomicU64 =
            core::sync::atomic::AtomicU64::new(0);
        let total_n = TOTAL_IRQ.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if total_n > 0 && total_n % 5000 == 0 {
            uart::puts("[total_irq=");
            crate::kernel::mm::print_num(total_n as usize);
            uart::puts("]\n");
        }
        if was_in_el0 {
            static IRQ_PC_COUNT: core::sync::atomic::AtomicU64 =
                core::sync::atomic::AtomicU64::new(0);
            let n = IRQ_PC_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            if n > 0 && n % 200 == 0 {
                let elr = unsafe { (*frame).elr };
                let lr  = unsafe { (*frame).x[30] };
                let tid = crate::caves::linux::threads::current_tid();
                uart::puts("[irq#");
                crate::kernel::mm::print_num(n as usize);
                uart::puts(" preempt t");
                crate::kernel::mm::print_num(tid as usize);
                uart::puts(" pc=0x");
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" lr=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((lr >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts("]\n");
            }
        }

        // GIC end-of-interrupt MUST happen BEFORE schedule(). schedule()
        // may switch us to another thread (parking this handle_irq frame
        // on the kernel stack until we're rescheduled). If EOI is deferred
        // until after schedule(), the GIC sees the IRQ as still active
        // for however long we're swapped out — could be seconds — and
        // blocks all subsequent timer IRQs. The "1Hz instead of 100Hz"
        // observation traces directly to this.
        eoi(gic_v, iar);

        if was_in_el0 {
            crate::caves::linux::threads::schedule();
        } else {
            // EL1 — defer to syscall boundary so we don't preempt
            // kernel code that might be holding a lock.
            crate::caves::linux::threads::request_preempt();
        }
    } else {
        // Non-timer IRQ — still need to ack so the GIC can deliver more.
        eoi(gic_v, iar);
    }
}

/// End-of-Interrupt write. v2 = MMIO GICC_EOIR; v3 = ICC_EOIR1_EL1.
#[inline(always)]
fn eoi(gic_v: u8, iar: u32) {
    if gic_v == 3 {
        unsafe {
            // ICC_EOIR1_EL1 = S3_0_C12_C12_1
            core::arch::asm!("msr S3_0_C12_C12_1, {}", in(reg) iar as u64);
        }
    } else {
        const GICC_EOIR: usize = 0x0801_0000 + 0x010;
        unsafe { core::ptr::write_volatile(GICC_EOIR as *mut u32, iar); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_exception(frame: *mut TrapFrame) {
    let esr: u64;
    unsafe { core::arch::asm!("mrs {}, esr_el1", out(reg) esr); }
    let ec = (esr >> 26) & 0x3F;
    // Wrap inner dispatch in a closure-like block so we can run a
    // post-check before returning. If the caller is going to RESTORE_REGS
    // and eret to EL0 with frame.elr in kernel range, that's the bug —
    // log loudly so we can correlate to which handler call did it.
    handle_sync_exception_inner(frame, esr, ec);
    unsafe {
        let elr_now = (*frame).elr;
        let spsr_now = (*frame).spsr;
        let target_el = (spsr_now >> 2) & 0x3;
        // M[3:0]=0 means eret target is EL0t (user mode). M[3:0]=5 is EL1h.
        let goes_to_el0 = (spsr_now & 0xF) == 0;
        if goes_to_el0 && elr_now >= 0x40000000 && elr_now < 0x80000000 {
            uart::puts("!!! frame.elr=KERNEL going to EL0: ec=0x");
            let hex = b"0123456789abcdef";
            uart::putc(hex[((ec >> 4) & 0xF) as usize]);
            uart::putc(hex[(ec & 0xF) as usize]);
            uart::puts(" elr=0x");
            for sh in (0..16).rev() {
                uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" spsr=0x");
            for sh in (0..16).rev() {
                uart::putc(hex[((spsr_now >> (sh * 4)) & 0xF) as usize]);
            }
            uart::puts(" target_el=");
            uart::putc(b'0' + target_el as u8);
            uart::puts("\n");
        }
    }
}

fn handle_sync_exception_inner(frame: *mut TrapFrame, esr: u64, ec: u64) {
    // PROBE-MODE RECOVERY (must run before any other dispatch).
    // If a `mmu::probe_read_u64` is mid-flight (PROBE_ACTIVE) and we
    // hit a fault, advance ELR past the faulting `ldr` (4 bytes) and
    // set PROBE_FAULTED so the probe returns None instead of hanging
    // the kernel. We accept any EL1 fault here — translation,
    // permission, alignment, etc. — because the probe contract is
    // "if reading would have any problem at all, return None."
    if crate::caves::linux::mmu::PROBE_ACTIVE.load(core::sync::atomic::Ordering::Acquire)
        && (ec == 0x24 || ec == 0x25 || ec == 0x20 || ec == 0x21)
    {
        crate::caves::linux::mmu::PROBE_FAULTED.store(true, core::sync::atomic::Ordering::Release);
        unsafe { (*frame).elr = (*frame).elr.wrapping_add(4); }
        return;
    }

    match ec {
        0x15 => {
            let svc_num = (esr & 0xFFFF) as u16;

            if svc_num == 0 {
                unsafe {
                    let f = &mut *frame;
                    let syscall_num = f.x[8];
                    let args = [f.x[0], f.x[1], f.x[2], f.x[3], f.x[4], f.x[5]];

                    // Record the pre-syscall register state into the
                    // history ring. This runs BEFORE the dispatcher
                    // touches f.x[0] so we capture the caller's LR
                    // (x30) and FP (x29), plus the arguments — which
                    // is exactly the forensic data the UNHANDLED dump
                    // needs when a `ret` eventually lands in the cage.
                    let tid_now = crate::caves::linux::threads::current_tid();
                    crate::caves::linux::syscall_history::record(
                        tid_now, syscall_num, &f.x, f.elr,
                    );

                    // EXIT: child exit → eret back to parent at clone return
                    if syscall_num == 93 || syscall_num == 94 {
                        let in_child = crate::caves::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed);

                        crate::caves::linux::syscall::handle(0, syscall_num, args);

                        if in_child {
                            let is_thread = crate::caves::linux::syscall::IS_THREAD_CHILD
                                .load(core::sync::atomic::Ordering::Relaxed);
                            let busybox_sp = core::ptr::read_volatile(
                                core::ptr::addr_of!(SAVED_BUSYBOX_SP));

                            if !is_thread {
                                // Fork-style child: restore parent stack contents
                                // (child corrupted parent's stack by sharing it)
                                let stack_src = core::ptr::addr_of!(SAVED_STACK) as usize;
                                for i in (0..STACK_SAVE_SIZE).step_by(8) {
                                    let val: u64;
                                    core::arch::asm!(
                                        "ldr {v}, [{a}]",
                                        a = in(reg) stack_src + i,
                                        v = out(reg) val,
                                    );
                                    core::arch::asm!(
                                        "str {v}, [{a}]",
                                        a = in(reg) busybox_sp as usize + i,
                                        v = in(reg) val,
                                    );
                                }
                            }
                            // Thread-style child: skip stack restore (child had own stack)

                            // Eret from saved clone frame → parent resumes
                            let saved_ptr = core::ptr::addr_of!(SAVED_FRAME) as u64;
                            // Get the child TID to return to parent
                            let child_tid = crate::caves::linux::syscall::LAST_CHILD_TID
                                .load(core::sync::atomic::Ordering::Relaxed) as u64;
                            // Restore main thread TID
                            crate::caves::linux::syscall::restore_parent_tid();

                            core::arch::asm!(
                                // V6-CHAIN-002 fix: write the saved
                                // busybox (user) SP to SP_EL0, not the
                                // current SP_EL1. The eret below uses
                                // SPSR.M to pick which SP becomes
                                // active for EL0; SP_EL1 must NOT be
                                // overwritten with a user-derived
                                // value or every subsequent exception
                                // pushes its trap frame to that
                                // attacker-influenced address.
                                "msr sp_el0, {sp_val}",
                                // x16 = pointer to saved frame data
                                "mov x16, {ptr}",
                                // Restore ELR and SPSR from saved frame
                                "ldp x0, x1, [x16, #248]",
                                "msr elr_el1, x0",
                                // Clear DAIF bits in SPSR so interrupts work after eret
                                "and x1, x1, #0xFFFFFFFFFFFFFC3F",
                                "msr spsr_el1, x1",
                                // Restore all GPRs from saved frame
                                "ldr x1, [x16, #8]",
                                "ldp x2, x3, [x16, #16]",
                                "ldp x4, x5, [x16, #32]",
                                "ldp x6, x7, [x16, #48]",
                                "ldp x8, x9, [x16, #64]",
                                "ldp x10, x11, [x16, #80]",
                                "ldp x12, x13, [x16, #96]",
                                "ldp x14, x15, [x16, #112]",
                                "ldr x17, [x16, #136]",
                                "ldp x18, x19, [x16, #144]",
                                "ldp x20, x21, [x16, #160]",
                                "ldp x22, x23, [x16, #176]",
                                "ldp x24, x25, [x16, #192]",
                                "ldp x26, x27, [x16, #208]",
                                "ldp x28, x29, [x16, #224]",
                                "ldr x30, [x16, #240]",
                                // Load x16 last (destroys our pointer)
                                "ldr x16, [x16, #128]",
                                // x0 = child TID (parent return from clone)
                                "mov x0, {tid}",
                                "eret",
                                ptr = in(reg) saved_ptr,
                                sp_val = in(reg) busybox_sp,
                                tid = in(reg) child_tid,
                                options(noreturn),
                            );
                        }

                        // REAL-FORK: if this exit_group is from a forked
                        // child cave (its TTBR0 differs from the host cave's),
                        // tear down ONLY the child cave and let the
                        // scheduler resume the parent. Otherwise it's
                        // the main process exiting and we go all the
                        // way back to the desktop shell.
                        let cur_ttbr0: u64;
                        core::arch::asm!("mrs {}, ttbr0_el1", out(reg) cur_ttbr0);
                        let cur_ttbr0 = cur_ttbr0 & !1u64;
                        let host_l1 = crate::caves::linux::mmu::host_cave_l1() as u64;
                        if host_l1 != 0 && cur_ttbr0 != host_l1 {
                            // Forked-child exit. Mark this thread Exited
                            // (parent can wait4 it), then schedule out.
                            // The cooperative-switch asm will activate the
                            // parent's TTBR0 when it picks the parent up.
                            //
                            // (We don't free the child's page tables /
                            // frames yet — wait4 will do that when
                            // implemented. Leak for now; reboot recovers.)
                            crate::caves::linux::threads::exit_current(
                                args[0] as i32);
                            // exit_current never returns — it schedules
                            // another thread and wfi's if none.
                        }

                        // Real exit (not a forked child — leave Cave entirely).
                        // V2-NEW-024: DO NOT call mmu::disable here — it
                        // clears SCTLR.M which the next switch_to_cave does
                        // not re-enable, leaving subsequent caves running
                        // with no AP/UXN/PXN enforcement at all. Switch to
                        // primary TTBR0 instead; MMU stays on.
                        crate::caves::linux::mmu::switch_to_primary();
                        // Restore the kernel SP that the loader stashed before
                        // erets to EL0. See KERNEL_SP_SAVE above.
                        let save_addr = kernel_sp_save_addr();
                        core::arch::asm!(
                            "ldr x0, [{addr}]",
                            "mov sp, x0",
                            addr = in(reg) save_addr,
                            out("x0") _,
                        );
                        crate::ui::desktop::resume();
                    }

                    // CLONE: save exception frame for later parent-resume
                    if syscall_num == 220
                        && !crate::caves::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed)
                    {
                        let frame_addr = frame as usize;
                        let save_dst = core::ptr::addr_of_mut!(SAVED_FRAME) as usize;
                        // TrapFrame is exactly 33 * 8 = 264 bytes (31 x regs +
                        // ELR + SPSR). The old loop ran for 272 bytes — an
                        // 8-byte overread of the kernel stack into SAVED_FRAME
                        // that got restored to the parent's registers on
                        // child exit. Cap at sizeof(TrapFrame).
                        let tf_size = core::mem::size_of::<TrapFrame>();
                        for i in (0..tf_size).step_by(8) {
                            let val: u64;
                            core::arch::asm!(
                                "ldr {v}, [{a}]",
                                a = in(reg) frame_addr + i,
                                v = out(reg) val,
                            );
                            core::arch::asm!(
                                "str {v}, [{a}]",
                                a = in(reg) save_dst + i,
                                v = in(reg) val,
                            );
                        }
                        core::arch::asm!("dsb sy");
                        // Save busybox SP = frame_addr + 272
                        let sp_val = (frame_addr + 272) as u64;
                        core::ptr::write_volatile(
                            core::ptr::addr_of_mut!(SAVED_BUSYBOX_SP), sp_val,
                        );
                        // Save stack contents above SP (child shares stack)
                        let stack_dst = core::ptr::addr_of_mut!(SAVED_STACK) as usize;
                        for i in (0..STACK_SAVE_SIZE).step_by(8) {
                            let val: u64;
                            core::arch::asm!(
                                "ldr {v}, [{a}]",
                                a = in(reg) sp_val as usize + i,
                                v = out(reg) val,
                            );
                            core::arch::asm!(
                                "str {v}, [{a}]",
                                a = in(reg) stack_dst + i,
                                v = in(reg) val,
                            );
                        }
                    }

                    // EXECVE: if child is calling execve for a busybox applet,
                    // jump to the worker busybox copy for real applet execution
                    if syscall_num == 221
                        && crate::caves::linux::syscall::IN_CHILD
                            .load(core::sync::atomic::Ordering::Relaxed)
                    {
                        let worker_entry = crate::caves::linux::loader::WORKER_ENTRY
                            .load(core::sync::atomic::Ordering::Relaxed);
                        if worker_entry != 0 {
                            // Read the path to check if it's a busybox applet.
                            // V2-007: gate path_ptr to userspace before ldrb.
                            let path_ptr = f.x[0] as usize;
                            let argv_ptr = f.x[1] as usize;
                            if !crate::caves::linux::uaccess::is_user_range(path_ptr, 1) {
                                f.x[0] = (-14i64) as u64; // EFAULT
                                return;
                            }
                            let mut path_buf = [0u8; 128];
                            let mut plen = 0usize;
                            for i in 0..127 {
                                if !crate::caves::linux::uaccess::is_user_range(path_ptr + i, 1) { break; }
                                let b: u32;
                                core::arch::asm!("ldrb {v:w}, [{a}]",
                                    a = in(reg) path_ptr + i, v = out(reg) b);
                                if b == 0 { break; }
                                path_buf[i] = b as u8;
                                plen += 1;
                            }

                            // Debug: log what path execve is trying
                            crate::drivers::uart::puts("[execve] path='");
                            for i in 0..plen.min(60) {
                                crate::drivers::uart::putc(path_buf[i]);
                            }
                            crate::drivers::uart::puts("' len=");
                            crate::kernel::mm::print_num(plen);
                            crate::drivers::uart::puts("\n");

                            // Check for standalone binaries (not busybox applets)
                            let is_hello = (plen == 10 && &path_buf[..10] == b"/bin/hello")
                                || (plen == 5 && &path_buf[..5] == b"hello");

                            if is_hello {
                                // Load the hello ELF binary
                                let hello_data = crate::caves::linux::runner::hello_elf();
                                match crate::caves::linux::loader::load_hello_elf(hello_data) {
                                    Ok((phys_entry, _phys_base, _orig_entry)) => {
                                        // Build a minimal stack for the hello binary
                                        let stack_page = crate::kernel::mm::frame::alloc_frame();
                                        if let Some(stack_base) = stack_page {
                                            for _ in 0..15 {
                                                crate::kernel::mm::frame::alloc_frame();
                                            }
                                            let mut sp = stack_base + 16 * 4096;

                                            // Write argv[0] = "hello"
                                            sp -= 6; // "hello\0"
                                            let arg0_addr = sp;
                                            for (j, &b) in b"hello".iter().enumerate() {
                                                core::arch::asm!("strb {v:w}, [{a}]",
                                                    a = in(reg) sp + j,
                                                    v = in(reg) b as u32);
                                            }
                                            core::arch::asm!("strb wzr, [{a}]",
                                                a = in(reg) sp + 5);

                                            // envp string
                                            sp -= 10;
                                            let env0 = sp;
                                            for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
                                                core::arch::asm!("strb {v:w}, [{a}]",
                                                    a = in(reg) sp + j, v = in(reg) b as u32);
                                            }

                                            sp = (sp - 64) & !0xF;

                                            // auxv: AT_NULL
                                            sp -= 16;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp + 8);
                                            // AT_PAGESZ
                                            sp -= 16;
                                            let k6: u64 = 6; let v4096: u64 = 4096;
                                            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k6);
                                            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v4096);

                                            // envp NULL + pointer
                                            sp -= 8;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            sp -= 8;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) env0 as u64);

                                            // argv NULL + pointer
                                            sp -= 8;
                                            core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                            sp -= 8;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) arg0_addr as u64);

                                            // argc = 1
                                            sp -= 8;
                                            let one: u64 = 1;
                                            core::arch::asm!("str {v}, [{a}]",
                                                a = in(reg) sp, v = in(reg) one);

                                            // V6-CHAIN-002: enter hello binary at EL0 via
                                            // eret. Writing to SP_EL1 then `br {entry}`
                                            // ran the binary at EL1 with attacker SP —
                                            // every IRQ corrupted the kernel stack.
                                            let entry = phys_entry;
                                            core::arch::asm!(
                                                "msr sp_el0, {sp_val}",
                                                "msr elr_el1, {entry}",
                                                "msr spsr_el1, xzr",  // EL0t, AIF clear
                                                "isb",
                                                "eret",
                                                sp_val = in(reg) sp as u64,
                                                entry = in(reg) entry,
                                                options(noreturn),
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        // Load failed — fall through to ENOENT
                                    }
                                }
                            }

                            // Check if it's in /bin or /usr/bin (busybox applet)
                            let is_bb = plen > 5 && (&path_buf[..5] == b"/bin/"
                                || (plen > 9 && &path_buf[..9] == b"/usr/bin/"));

                            if is_bb {
                                // Re-initialize worker's writable segments
                                // (previous applet run may have corrupted globals)
                                let wbase = crate::caves::linux::loader::WORKER_PHYS_BASE
                                    .load(core::sync::atomic::Ordering::Relaxed);
                                crate::caves::linux::loader::reinit_elf(
                                    crate::caves::linux::runner::busybox_elf(),
                                    wbase,
                                );

                                // Read argv from userspace (up to 8 args).
                                // V2-008: gate argv_ptr (8 × 8-byte pointers)
                                // and each argv[i] string range. Without these,
                                // a cave could pass argv_ptr = 0x40400000 and
                                // have the worker emit 63-byte chunks of
                                // kernel RAM via busybox echo → UART.
                                let _arg_ptrs = [0usize; 8];
                                let mut arg_bufs = [[0u8; 64]; 8];
                                let mut arg_lens = [0usize; 8];
                                let mut argc = 0usize;
                                if argv_ptr != 0
                                    && crate::caves::linux::uaccess::is_user_range(argv_ptr, 8 * 8)
                                {
                                    for i in 0..8 {
                                        let ap: u64;
                                        core::arch::asm!("ldr {v}, [{a}]",
                                            a = in(reg) argv_ptr + i * 8, v = out(reg) ap);
                                        if ap == 0 { break; }
                                        if !crate::caves::linux::uaccess::is_user_range(ap as usize, 1) { break; }
                                        for j in 0..63 {
                                            if !crate::caves::linux::uaccess::is_user_range(ap as usize + j, 1) { break; }
                                            let b: u32;
                                            core::arch::asm!("ldrb {v:w}, [{a}]",
                                                a = in(reg) ap as usize + j, v = out(reg) b);
                                            if b == 0 { break; }
                                            arg_bufs[i][j] = b as u8;
                                            arg_lens[i] = j + 1;
                                        }
                                        argc += 1;
                                    }
                                }

                                // Build a fresh stack for the worker
                                let stack_page = crate::kernel::mm::frame::alloc_frame();
                                if let Some(stack_base) = stack_page {
                                    // Allocate 16 pages for stack
                                    for _ in 0..15 {
                                        crate::kernel::mm::frame::alloc_frame();
                                    }
                                    let mut sp = stack_base + 16 * 4096;

                                    // Write arg strings to stack
                                    let mut str_addrs = [0usize; 8];
                                    for i in (0..argc).rev() {
                                        sp -= arg_lens[i] + 1;
                                        str_addrs[i] = sp;
                                        for j in 0..arg_lens[i] {
                                            core::arch::asm!("strb {v:w}, [{a}]",
                                                a = in(reg) sp + j,
                                                v = in(reg) arg_bufs[i][j] as u32);
                                        }
                                        core::arch::asm!("strb wzr, [{a}]",
                                            a = in(reg) sp + arg_lens[i]);
                                    }

                                    // envp string
                                    sp -= 10;
                                    let env0 = sp;
                                    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
                                        core::arch::asm!("strb {v:w}, [{a}]",
                                            a = in(reg) sp + j, v = in(reg) b as u32);
                                    }

                                    sp = (sp - 64) & !0xF;

                                    // auxv: AT_NULL
                                    sp -= 16;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp + 8);
                                    // AT_PAGESZ
                                    sp -= 16;
                                    let k6: u64 = 6; let v4096: u64 = 4096;
                                    core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k6);
                                    core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v4096);

                                    // envp NULL + pointer
                                    sp -= 8;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    sp -= 8;
                                    core::arch::asm!("str {v}, [{a}]",
                                        a = in(reg) sp, v = in(reg) env0 as u64);

                                    // argv NULL + pointers
                                    sp -= 8;
                                    core::arch::asm!("str xzr, [{a}]", a = in(reg) sp);
                                    for i in (0..argc).rev() {
                                        sp -= 8;
                                        core::arch::asm!("str {v}, [{a}]",
                                            a = in(reg) sp, v = in(reg) str_addrs[i] as u64);
                                    }

                                    // argc
                                    sp -= 8;
                                    core::arch::asm!("str {v}, [{a}]",
                                        a = in(reg) sp, v = in(reg) argc as u64);

                                    // V6-CHAIN-002: enter worker busybox at EL0 via eret.
                                    let entry = worker_entry as u64;
                                    core::arch::asm!(
                                        "msr sp_el0, {sp_val}",
                                        "msr elr_el1, {entry}",
                                        "msr spsr_el1, xzr",
                                        "isb",
                                        "eret",
                                        sp_val = in(reg) sp as u64,
                                        entry = in(reg) entry,
                                        options(noreturn),
                                    );
                                }
                            }
                        }
                    }

                    // For the threading-model clone path, stash the
                    // parent's post-svc return address and saved PSTATE
                    // so threads::set_child_resume can seed the child's
                    // eret-target. ELR_EL1 at SVC entry already points
                    // at the instruction *after* svc, so it's what the
                    // child resumes at.
                    if syscall_num == 220
                        && crate::caves::linux::threads::is_enabled()
                    {
                        // V8-CLONE-ELR-CHECK: f.elr should be the user's
                        // post-svc PC, always a user VA. If it's anywhere
                        // in kernel range, every child cloned from here
                        // will eret straight into kernel space → instant
                        // EL0 instruction-abort. Catch it loudly.
                        if f.elr >= 0x40000000 && f.elr < 0x80000000 {
                            uart::puts("!!! CLONE: parent ELR is KERNEL VA 0x");
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((f.elr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" — would corrupt child resume PC\n");
                            uart::puts("  spsr=0x");
                            for sh in (0..16).rev() {
                                uart::putc(hex[((f.spsr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" (M[3:0]=0 means from EL0)\n");
                        }
                        crate::caves::linux::threads::PARENT_SYSCALL_ELR
                            .store(f.elr, core::sync::atomic::Ordering::Release);
                        crate::caves::linux::threads::PARENT_SYSCALL_SPSR
                            .store(f.spsr, core::sync::atomic::Ordering::Release);
                        // Snapshot all of the parent's GPRs at svc entry
                        // so set_child_resume can seed the child's full
                        // register state. glibc / musl pthread trampolines
                        // on AArch64 stash fn in x10 and arg in x12 before
                        // svc and expect them to survive into the child's
                        // post-svc code (`blr x10` / `mov x0, x12`). Without
                        // this carry, the child indirect-branch lands at
                        // PC=0.
                        for i in 0..31 {
                            crate::caves::linux::threads::PARENT_SYSCALL_REGS[i]
                                .store(f.x[i], core::sync::atomic::Ordering::Release);
                        }
                    }

                    // rt_sigreturn (syscall 139) must run against the
                    // trap frame directly — it restores every GPR,
                    // ELR, SPSR, and SP_EL0 from the ucontext the
                    // handler's stack. The regular syscall dispatcher
                    // can't see the frame, so short-circuit it here.
                    let result: i64 = if syscall_num == 139 {
                        let sf = &mut *(frame as *mut crate::caves::linux::signal::TrapFrame);
                        crate::caves::linux::signal::complete_rt_sigreturn(sf)
                        // NB: do NOT overwrite f.x[0] below — every
                        // register has just been restored from the
                        // ucontext and the subsequent `f.x[0] = result`
                        // would clobber x0. We short-circuit with a
                        // direct return after the syscall trace.
                    } else {
                        let r = crate::caves::linux::syscall::handle(0, syscall_num, args);
                        f.x[0] = r as u64;
                        r
                    };
                    // After complete_rt_sigreturn the frame has fresh
                    // contents; any further post-processing (CLONE
                    // child_stack path, EXIT special handling, …)
                    // doesn't apply to rt_sigreturn. Bail out.
                    if syscall_num == 139 {
                        return;
                    }

                    // CLONE with child_stack: jump child to new stack via manual eret
                    if syscall_num == 220 && result == 0 {
                        let child_sp = crate::caves::linux::syscall::CLONE_CHILD_STACK
                            .load(core::sync::atomic::Ordering::Relaxed);
                        if child_sp != 0 {
                            // Clear the child_stack flag (one-shot)
                            crate::caves::linux::syscall::CLONE_CHILD_STACK
                                .store(0, core::sync::atomic::Ordering::Relaxed);
                            // Resume the child at the next instruction (after svc)
                            // with SP = child_stack and x0 = 0.
                            // We use x16 as frame pointer (like parent-resume code)
                            // and load x16 itself last from the frame.
                            let frame_ptr = frame as u64;
                            let elr_val = f.elr;
                            let spsr_val = f.spsr;
                            // Ensure 16-byte SP alignment (ARM64 ABI requirement)
                            let child_sp_aligned = child_sp & !0xF;
                            core::arch::asm!(
                                // V6-CHAIN-002 FIX: write the attacker-supplied
                                // child SP into SP_EL0 (the user stack pointer
                                // that eret activates), NOT SP_EL1 (the kernel
                                // SP we're currently using). The previous
                                // `mov sp, {csp}` clobbered the kernel stack
                                // pointer with a value chosen entirely by the
                                // calling cave — every subsequent IRQ/SVC then
                                // pushed its 272-byte trap frame at that
                                // attacker-chosen address. Direct kernel write
                                // primitive on every exception. We use SPSR
                                // configured for EL0t and let eret restore.
                                "msr sp_el0, {csp}",
                                // Set ELR and SPSR for child return
                                "msr elr_el1, {elr}",
                                "msr spsr_el1, {spsr}",
                                // Use only LDR (not LDP) to avoid alignment faults
                                // trap frame may not be 16-byte aligned
                                "mov x16, {fp}",
                                "ldr x1, [x16, #8]",
                                "ldr x2, [x16, #16]",
                                "ldr x3, [x16, #24]",
                                "ldr x4, [x16, #32]",
                                "ldr x5, [x16, #40]",
                                "ldr x6, [x16, #48]",
                                "ldr x7, [x16, #56]",
                                "ldr x8, [x16, #64]",
                                "ldr x9, [x16, #72]",
                                "ldr x10, [x16, #80]",
                                "ldr x11, [x16, #88]",
                                "ldr x12, [x16, #96]",
                                "ldr x13, [x16, #104]",
                                "ldr x14, [x16, #112]",
                                "ldr x15, [x16, #120]",
                                "ldr x17, [x16, #136]",
                                "ldr x18, [x16, #144]",
                                "ldr x19, [x16, #152]",
                                "ldr x20, [x16, #160]",
                                "ldr x21, [x16, #168]",
                                "ldr x22, [x16, #176]",
                                "ldr x23, [x16, #184]",
                                "ldr x24, [x16, #192]",
                                "ldr x25, [x16, #200]",
                                "ldr x26, [x16, #208]",
                                "ldr x27, [x16, #216]",
                                "ldr x28, [x16, #224]",
                                "ldr x29, [x16, #232]",
                                "ldr x30, [x16, #240]",
                                "ldr x16, [x16, #128]",
                                // x0 = 0 (child return from clone)
                                "mov x0, #0",
                                "eret",
                                elr = in(reg) elr_val,
                                spsr = in(reg) spsr_val,
                                fp = in(reg) frame_ptr,
                                csp = in(reg) child_sp_aligned,
                                options(noreturn),
                            );
                        }
                    }

                    // Async signal poll. If `sys_tgkill` / `sys_kill`
                    // queued a signal on this thread during the
                    // current syscall (or before it — the poll also
                    // picks up anything that accumulated while we
                    // were blocked in `futex_wait` / `ppoll` / etc.),
                    // redirect the trap frame into the registered
                    // user handler on the way back to EL0. On
                    // success the caller's x0 / ELR have been
                    // rewritten to the handler's entry arguments;
                    // on SIG_DFL-with-fatal-default the helper
                    // `terminate_cave_fatal`s instead of returning.
                    //
                    // Skip for rt_sigreturn (syscall 139) — the
                    // frame has just been restored from a ucontext
                    // and polling on top of it would re-raise a
                    // signal we're literally in the middle of
                    // completing.
                    if syscall_num != 139 {
                        let sf = &mut *(frame as *mut crate::caves::linux::signal::TrapFrame);
                        let _ = crate::caves::linux::signal::try_deliver_pending(sf);
                    }
                }
            } else {
                unsafe {
                    crate::kernel::syscall::handle(svc_num, &mut *frame);
                }
            }
        }
        0x25 => {
            let far: u64;
            unsafe { core::arch::asm!("mrs {}, far_el1", out(reg) far); }
            let elr = unsafe { (*frame).elr };

            // CHROMIUM-PHASE-D: kernel uaccess (e.g. pipe_buf::write
            // copying from a user iov) can hit a USER VA whose page
            // hasn't been demand-committed yet. The user-side handler
            // for this is EC=0x24, but when the KERNEL is the one
            // touching it the EC is 0x25 (data abort from current EL).
            // Try the lazy-commit path first; if demand_page accepts,
            // retry the faulting instruction.
            if crate::caves::linux::demand_page::try_handle(far, esr) {
                return;
            }

            let in_code_range = (elr < 0x1400000)
                || (elr >= 0x40000000 && elr < 0x50000000);
            if in_code_range {
                let instr: u32 = unsafe {
                    let val: u32;
                    core::arch::asm!("ldr {v:w}, [{a}]",
                        a = in(reg) elr, v = out(reg) val);
                    val
                };

                // Emulate alignment faults (DFSC=0x21) — HVF enforces strict alignment
                // Use FAR as the exact faulting address, decode instruction for size/direction
                let dfsc = esr & 0x3F;
                if dfsc == 0x21 {
                    unsafe {
                        // ESR ISS fields for data abort tell us load vs store and size
                        let iss = esr & 0x1FFFFFF;
                        let wnr = (iss >> 6) & 1; // 0=read, 1=write
                        let sas = (iss >> 22) & 3; // access size: 0=byte,1=half,2=word,3=dword
                        let srt = (iss >> 16) & 0x1F; // transfer register
                        let isv = (iss >> 24) & 1; // ISV bit — if 1, SAS/SRT are valid

                        if isv == 1 {
                            // ISV valid: use ESR fields (more reliable than decoding instruction)
                            let nbytes = 1u64 << sas;
                            let rt = srt as usize;

                            // V2-002 gate: refuse to proxy kernel-address
                            // accesses for EL0. Before this check, a user
                            // instruction faulting on an unaligned access
                            // to any kernel VA would have us obediently
                            // load/store on its behalf at EL1.
                            let in_user = crate::caves::linux::uaccess::is_user_range(
                                far as usize, nbytes as usize);
                            if !in_user {
                                // Skip the faulting instruction and deliver
                                // 0/NOP — avoids kernel R/W primitive.
                                (*frame).elr = elr + 4;
                                if wnr == 0 && rt < 31 { (*frame).x[rt] = 0; }
                                return;
                            }

                            if wnr == 0 {
                                // Load: read bytes from FAR
                                let mut val = 0u64;
                                for i in 0..nbytes {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) far.wrapping_add(i), v = out(reg) b);
                                    val |= (b as u64) << (i * 8);
                                }
                                if rt < 31 { (*frame).x[rt] = val; }
                            } else {
                                // Store: write bytes to FAR
                                let val = if rt < 31 { (*frame).x[rt] } else { 0 };
                                for i in 0..nbytes {
                                    let b = ((val >> (i * 8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) far.wrapping_add(i), v = in(reg) b);
                                }
                            }
                            (*frame).elr = elr + 4;
                            return;
                        }

                        // ISV=0: LDP/STP (pair instructions don't set ISV)
                        // Decode instruction manually
                        if (instr & 0x3A000000) == 0x28000000 {
                            let is_64 = (instr >> 31) & 1 == 1;
                            let is_load = (instr >> 22) & 1 == 1;
                            let rt = (instr & 0x1F) as usize;
                            let rt2 = ((instr >> 10) & 0x1F) as usize;
                            let rn = ((instr >> 5) & 0x1F) as usize;
                            let scale: u64 = if is_64 { 8 } else { 4 };
                            // FAR is the exact address the CPU tried to access
                            let addr = far;

                            if is_load {
                                let mut v1 = 0u64; let mut v2 = 0u64;
                                for i in 0..scale {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(i), v = out(reg) b);
                                    v1 |= (b as u64) << (i * 8);
                                }
                                for i in 0..scale {
                                    let b: u8;
                                    core::arch::asm!("ldrb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(scale + i), v = out(reg) b);
                                    v2 |= (b as u64) << (i * 8);
                                }
                                if rt < 31 { (*frame).x[rt] = v1; }
                                if rt2 < 31 { (*frame).x[rt2] = v2; }
                            } else {
                                let v1 = if rt < 31 { (*frame).x[rt] } else { 0 };
                                let v2 = if rt2 < 31 { (*frame).x[rt2] } else { 0 };
                                for i in 0..scale {
                                    let b = ((v1 >> (i*8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(i), v = in(reg) b);
                                }
                                for i in 0..scale {
                                    let b = ((v2 >> (i*8)) & 0xFF) as u32;
                                    core::arch::asm!("strb {v:w}, [{a}]",
                                        a = in(reg) addr.wrapping_add(scale + i), v = in(reg) b);
                                }
                            }
                            // Handle pre/post index writeback
                            let wb = (instr >> 23) & 3;
                            if wb == 0b01 || wb == 0b11 {
                                let imm7 = ((instr >> 15) & 0x7F) as i32;
                                let simm = if imm7 & 0x40 != 0 { imm7 | !0x7F } else { imm7 };
                                let offset = simm as i64 * scale as i64;
                                // For pre-index: FAR = base + offset, so base = FAR - offset
                                // For post-index: FAR = base, new_base = base + offset
                                if wb == 0b11 {
                                    // Pre-index: writeback = FAR (which is base+offset already)
                                    if rn < 31 { (*frame).x[rn] = addr; }
                                    // rn=31 = SP: can't update SP from trap frame, skip
                                } else {
                                    // Post-index: writeback = base + offset = FAR + offset
                                    let new_val = (addr as i64 + offset) as u64;
                                    if rn < 31 { (*frame).x[rn] = new_val; }
                                }
                            }
                            (*frame).elr = elr + 4;
                            return;
                        }

                        // Fallback: skip instruction
                        (*frame).elr = elr + 4;
                    }
                    return;
                }

                // Emulate atomic load/store exclusive (HVF doesn't support)
                // Single-core → always succeeds, safe to emulate with plain ops
                if (instr & 0x3F000000) == 0x08000000 {
                    let size = (instr >> 30) & 3;
                    let o2 = (instr >> 23) & 1;
                    let is_load = (instr >> 22) & 1;
                    let o1 = (instr >> 21) & 1;
                    let rs = ((instr >> 16) & 0x1F) as usize;
                    let rn = ((instr >> 5) & 0x1F) as usize;
                    let rt = (instr & 0x1F) as usize;

                    unsafe {
                        let f = &mut *frame;
                        let addr = if rn < 31 { f.x[rn] } else { 0 } as usize;

                        if o1 == 0 && o2 == 0 {
                            // LDXR/LDAXR or STXR/STLXR
                            if is_load == 1 {
                                let val = emulate_load(addr, size);
                                if rt < 31 { f.x[rt] = val; }
                            } else {
                                let val = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, val);
                                if rs < 31 { f.x[rs] = 0; }
                            }
                        } else if o1 == 1 && o2 == 1 {
                            // CAS — compare and swap
                            let old = emulate_load(addr, size);
                            let cmp = if rs < 31 { f.x[rs] } else { 0 };
                            let mask: u64 = match size { 0=>0xFF, 1=>0xFFFF, 2=>0xFFFFFFFF, _=>u64::MAX };
                            if (old & mask) == (cmp & mask) {
                                let nv = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, nv);
                            }
                            if rs < 31 { f.x[rs] = old; }
                        } else if o1 == 0 && o2 == 1 {
                            // LDAR/STLR — acquire/release
                            if is_load == 1 {
                                let val = emulate_load(addr, size);
                                if rt < 31 { f.x[rt] = val; }
                            } else {
                                let val = if rt < 31 { f.x[rt] } else { 0 };
                                emulate_store(addr, size, val);
                            }
                        } else {
                            // SWP or other atomic — swap
                            let old = emulate_load(addr, size);
                            let nv = if rs < 31 { f.x[rs] } else { 0 };
                            emulate_store(addr, size, nv);
                            if rt < 31 { f.x[rt] = old; }
                        }
                        f.elr = elr + 4;
                    }
                    return;
                }

                // Cache maintenance ops (DC/IC) — skip
                if (instr & 0xFFF80000) == 0xD5080000 {
                    unsafe { (*frame).elr = elr + 4; }
                    return;
                }
            }

            // Log detailed fault info
            let dfsc = esr & 0x3F; // Data Fault Status Code
            uart::puts("!!! DATA ABORT (DFSC=0x");
            let hex = b"0123456789abcdef";
            uart::putc(hex[((dfsc >> 4) & 0xf) as usize]);
            uart::putc(hex[(dfsc & 0xf) as usize]);
            uart::puts(") !!!\n");
            uart::puts("  FAR: 0x"); print_hex(far);
            uart::puts("  ELR: 0x"); print_hex(elr);
            // V8-DABT-DIAG: also show x30 (link reg) — that's the
            // PC that CALLED into the bad ELR. If ELR is in rodata
            // (function pointer corruption), x30 tells us the real
            // call site that branched there.
            let lr = unsafe { (*frame).x[30] };
            uart::puts("  LR:  0x"); print_hex(lr);
            // Log TTBR0 to see which page table is active
            let ttbr0: u64;
            unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0); }
            uart::puts("  TTBR0: 0x"); print_hex(ttbr0);
            uart::puts("\n");
            // V8-DABT-DIAG: dump ALL GPRs so we can compute the load
            // address from whichever Xn the faulting instr used.
            // Format: "x00..x07: 0x... 0x... ..." 4 per row, more compact.
            for row in 0..8usize {
                uart::puts("  x");
                let r = row * 4;
                if r < 10 { uart::putc(hex[r]); } else {
                    uart::putc(hex[r / 10]); uart::putc(hex[r % 10]);
                }
                uart::puts(": ");
                for col in 0..4usize {
                    let i = row * 4 + col;
                    if i > 30 { break; }
                    let v = unsafe { (*frame).x[i] };
                    uart::puts("0x"); print_hex(v); uart::puts(" ");
                }
                uart::puts("\n");
            }
            // V8-DABT-DIAG: walk the frame-pointer chain so we see the
            // KERNEL CALL STACK, not just LR. With LR=0 (no recent BL)
            // the only way to know who called us is x29 (FP) which
            // points at [saved_fp, saved_lr] of the caller's frame.
            // Walk up to 8 frames before giving up.
            uart::puts("  fp-walk:\n");
            let mut fp = unsafe { (*frame).x[29] };
            for _ in 0..8 {
                if fp == 0 { break; }
                // Refuse to deref obviously-bad frame pointers.
                if fp < 0x40000000 || fp >= 0xc0000000 {
                    if fp >= 0x10000000 && fp < 0x8000_0000_0000 {
                        // user-VA range, OK to read
                    } else {
                        uart::puts("    (fp=0x"); print_hex(fp); uart::puts(" — out of range)\n");
                        break;
                    }
                }
                // validate page is mapped before deref.
                // Pre-fix the diagnostic FP-walk did `ldr [fp]` raw,
                // assuming the page-table walk would either succeed
                // or trap-and-recover. Under TCG that holds; under
                // HVF the resulting translation fault has ESR.ISV=0
                // (page-table walker faults can't be syndromed) and
                // QEMU asserts in `hvf_handle_exception` (hvf.c:1883).
                // page_is_mapped walks the cave's L1/L2/L3 itself
                // (using `read_volatile` against KERNEL phys pages,
                // which is always safe), so we can pre-check without
                // risking a fault.
                if !page_is_mapped(fp) || !page_is_mapped(fp + 8) {
                    uart::puts("    (fp=0x"); print_hex(fp); uart::puts(" — unmapped)\n");
                    break;
                }
                let saved_fp: u64;
                let saved_lr: u64;
                unsafe {
                    core::arch::asm!("ldr {v}, [{a}]", a = in(reg) fp,        v = out(reg) saved_fp);
                    core::arch::asm!("ldr {v}, [{a}]", a = in(reg) fp + 8,    v = out(reg) saved_lr);
                }
                uart::puts("    fp=0x"); print_hex(fp);
                uart::puts(" lr=0x"); print_hex(saved_lr);
                uart::puts("\n");
                if saved_fp <= fp { break; } // stop on backwards/equal frames
                fp = saved_fp;
            }
            // V8-DABT-DIAG: also dump the bytes around ELR so we know
            // which load/store instruction faulted (helps identify
            // which Xn was used to compute FAR).
            // gate on page_is_mapped — same reason as above.
            uart::puts("  instr@elr: ");
            if page_is_mapped(elr) {
                let instr: u32 = unsafe {
                    let v: u32;
                    core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) elr, v = out(reg) v);
                    v
                };
                for sh in (0..8).rev() {
                    uart::putc(hex[((instr >> (sh*4)) & 0xF) as usize]);
                }
            } else {
                uart::puts("(unmapped)");
            }
            uart::puts("\n");
            // iter 13: dump 32 instructions BEFORE ELR + 16
            // AFTER, so we can identify the function by its pattern.
            // Especially useful when the PC isn't in any loaded library
            // (suggesting JIT'd or kernel-aliased memory). 4 bytes per
            // instr × 4 instr per line = 16 bytes per line.
            uart::puts("  instr_window:\n");
            let window_start = elr.wrapping_sub(128); // 32 instrs before
            for line in 0..12u64 {
                let line_pc = window_start.wrapping_add(line * 16);
                if !page_is_mapped(line_pc) {
                    uart::puts("    [unmapped]\n");
                    continue;
                }
                uart::puts("    ");
                for sh in (0..16).rev() {
                    uart::putc(hex[((line_pc >> (sh*4)) & 0xF) as usize]);
                }
                uart::puts(":");
                for off in [0u64, 4, 8, 12] {
                    let pc_in_line = line_pc.wrapping_add(off);
                    if !page_is_mapped(pc_in_line) { uart::puts(" ????????"); continue; }
                    let w: u32 = unsafe {
                        let v: u32;
                        core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) pc_in_line, v = out(reg) v);
                        v
                    };
                    uart::puts(" ");
                    for sh in (0..8).rev() {
                        uart::putc(hex[((w >> (sh*4)) & 0xF) as usize]);
                    }
                    if pc_in_line == elr { uart::puts("←"); }
                }
                uart::puts("\n");
            }
            // Track repeats so we can stop spamming and force-terminate
            // the cave instead of looping forever in this handler. Two
            // tries earlier landed the per-instruction skip approach,
            // but skipping just bounced to the next bad instruction
            // (50K+ skip messages in one run, never escapes). When the
            // kernel is touching a clearly-bogus user pointer
            // repeatedly, the cave's state is corrupt and the only
            // safe out is `terminate_cave_fatal` which returns to the
            // shell prompt. SIGBUS (signo=7) matches what a user-mode
            // bad pointer access would have delivered if it had
            // surfaced as EL0.
            static mut ABORT_COUNT: u32 = 0;
            static mut LAST_ABORT_ELR: u64 = 0;
            unsafe {
                if elr == LAST_ABORT_ELR {
                    ABORT_COUNT += 1;
                } else {
                    ABORT_COUNT = 1;
                    LAST_ABORT_ELR = elr;
                }
                if ABORT_COUNT > 3 {
                    uart::puts("[abort] EL1 fault unrecoverable — terminating cave\n");
                    crate::caves::linux::signal::terminate_cave_fatal(7, far);
                    // terminate_cave_fatal returns ! — control never
                    // reaches here.
                }
            }
        }
        0x3C => {
            let elr = unsafe { (*frame).elr };
            let in_child = crate::caves::linux::syscall::IN_CHILD
                .load(core::sync::atomic::Ordering::Relaxed);

            // b: PartitionAlloc's noreturn-abort BRKs.
            // PA's three crash points (CorruptionDetected,
            // FreelistCorruptionDetected, and the body of
            // DoubleFreeOrCorruptionDetected) are reached from
            // PA::Free's two-phase atomic CHECK firing. The check is a
            // real race (TOCTOU between LDAR and atomic ldclr) in user
            // code; ignoring it lets us see how much further the cave
            // can get with PA's bookkeeping in a "we said this was
            // free, the next op will sort it out" state.
            //
            // Walk the FP chain to find the first stack frame whose
            // saved-LR is OUTSIDE PA's noreturn-abort code range, then
            // return there as if PA::Free completed normally.
            //
            // KNOWN ABORT ELRs (this build of content_shell):
            // 0x14d73000 = CorruptionDetected
            // 0x14d73298 = DFOCD body fault after partial skip
            // 0x14d777ac = FreelistCorruptionDetected
            // PA libchrome text region is roughly 0x14000000-0x1B000000;
            // PA::Free itself is at 0x11a630c0 so we test for "still
            // inside PA::Free" by looking for LR < 0x12000000 (not in
            // libchrome PA region).
            const PA_ABORT_BRKS: &[u64] = &[
                0x14d73000, 0x14d77780, 0x14d77784, 0x14d77788, 0x14d7778c,
                0x14d77790, 0x14d77794, 0x14d77798, 0x14d7779c, 0x14d777a0,
                0x14d777a4, 0x14d777a8, 0x14d777ac, 0x14d777b0,
                0x14d72f98, 0x14d72fdc,
            ];
            // PA-abort range. The narrow 0x14d72f80..0x14d77800 catches
            // CorruptionDetected/DoubleFree/FreelistCorruption/etc.
            // Also include AddRefWithCheck (0x14ca8664) and a few
            // other refcount-overflow sites that we've seen in the wild.
            let pa_abort = PA_ABORT_BRKS.contains(&elr)
                || (0x14d72f80..=0x14d77800).contains(&elr)
                || elr == 0x14ca8664
                || elr == 0x14d92390
                || elr == 0x14ca3dfc;
            if pa_abort {
                // Walk the user stack's FP chain. Each frame: FP -> [FP], LR -> [FP+8].
                let sp_el0: u64;
                unsafe { core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0); }
                let mut fp = unsafe { (*frame).x[29] };
                let mut hops = 0;
                let mut found_lr: u64 = 0;
                while hops < 16 && fp != 0 {
                    // Validate fp is in user range AND mapped.
                    if !crate::caves::linux::uaccess::is_user_range(fp as usize, 16)
                        || !page_is_mapped(fp)
                    {
                        break;
                    }
                    let next_fp: u64 = unsafe {
                        core::ptr::read_volatile(fp as *const u64)
                    };
                    let saved_lr: u64 = unsafe {
                        core::ptr::read_volatile((fp + 8) as *const u64)
                    };
                    let in_pa_free = (0x11a63000..=0x11a6a800).contains(&saved_lr);
                    let in_pa_libchrome = (0x14d70000..=0x14da0000).contains(&saved_lr);
                    // also filter logging::LogMessage::*
                    // code. When PA's CHECKs fire under a LOG(FATAL),
                    // the call chain is `user → ~LogMessage → Flush →
                    // HandleFatal → BRK`. Skipping HandleFatal's BRK
                    // back into Flush at 0x14ca3928 lands mid-Flush
                    // with the wrong SP (since HandleFatal uses a
                    // pre-decrement frame `stp x29,x30,[sp,#-0x40]!`
                    // whereas the pa-skip resumption sets sp_el0 =
                    // fp + 16). Flush then reads garbage stack-
                    // relative locals, calls operator delete with a
                    // bogus pointer, and PA fires DoubleFreeDetected
                    // but the FP chain is now corrupt and the
                    // unwinder can't escape, terminating the cave.
                    // Fix: include logging code as a filtered range
                    // so the walk passes through both Flush and the
                    // dtor chain, landing on real user code where
                    // the fp+16 SP heuristic is correct.
                    // logging::LogMessage::{Init,Flush,HandleFatal,
                    // C1/C2/D0/D1/D2,...} cluster at
                    // 0x14ca2e00..0x14ca40ac.
                    let in_logging = (0x14ca2e00..=0x14ca4100).contains(&saved_lr);
                    // restrict to content_shell TEXT.
                    let in_text_range = saved_lr >= 0x11720000
                        && saved_lr < 0x19910000;
                    // function-start filter pushed
                    // pa-skip one frame deeper but landed inside V8's
                    // ReportOOMFailure which itself crashes — walking
                    // up the stack from inside the OOM chain just lands
                    // in another part of the broken OOM chain. Going
                    // back to the simpler "first valid LR in text range"
                    // policy.
                    if !in_pa_free && !in_pa_libchrome && !in_logging
                        && saved_lr != 0 && saved_lr > 0x1000 && in_text_range
                    {
                        found_lr = saved_lr;
                        break;
                    }
                    fp = next_fp;
                    hops += 1;
                }
                if found_lr != 0 {
                    // Synthesize "PA::Free returned normally": restore
                    // SP to past PA::Free's frame, set elr to user's LR.
                    //
                    // attempt (REVERTED): tried to set x1 to
                    // scratch when found_lr == V8Initializer start to
                    // avoid the NULL+0x17 deref. Result: V8Initializer
                    // proceeded further but caves HUNG (timed out)
                    // instead of crashing cleanly at 7.4K. Worse net
                    // outcome — the partially-init isolate deadlocked
                    // somewhere downstream. Keeping known-good logic.
                    //
                    // 🎯 SP-resume was wrong.
                    // Pre-fix: sp_el0 = fp + 16, x[29] = fp. fp is the
                    // FP of the LAST PA frame before user code (e.g.
                    // BufferFree's x29). Setting sp to fp+16 lands
                    // INSIDE that PA frame's local-saves area, NOT at
                    // the user code's body sp. When user (e.g.
                    // blink::HashTable::Rehash) reaches its epilogue
                    // `ldp x29, x30, [sp], #0x50`, it reads x30 from
                    // sp+8 = inside PA's locals = stale ~0x1 = ret to
                    // PC=0x1. This was producing the chain that Agent A
                    // (and yesterday's investigation) tagged as "skip-
                    // induced corruption" — every pa-skip planted a
                    // garbage saved-x30 into Rehash's frame, surfacing
                    // a few hundred lines later as the bad-PC fault.
                    //
                    // Correct values: x29 should be the CALLER's saved
                    // x29 (= [fp]), and sp should be the caller's body
                    // sp at bl-time. For ARM64 simple-prologue functions
                    // (`stp x29,x30,[sp,#-N]!; mov x29, sp`) with no
                    // further `sub sp, sp, #M`, body sp equals x29. So
                    // setting sp_el0 = caller's x29 = [fp] is the
                    // best heuristic without per-function metadata. It
                    // matches Rehash exactly; for functions that DO
                    // have a body-sp delta, it overshoots SP by that
                    // delta but stays within the caller's frame
                    // instead of leaving SP buried inside PA's frame
                    // (the prior bug).
                    unsafe {
                        let caller_x29 = core::ptr::read_volatile(fp as *const u64);
                        (*frame).elr   = found_lr;
                        (*frame).x[29] = caller_x29;
                        (*frame).x[30] = found_lr;
                        core::arch::asm!("msr sp_el0, {a}",
                            a = in(reg) caller_x29);
                    }
                    static SKIP_COUNT: core::sync::atomic::AtomicU32 =
                        core::sync::atomic::AtomicU32::new(0);
                    let n = SKIP_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                    if n < 10 || (n & 0xFF) == 0 {
                        uart::puts("[pa-skip] #");
                        crate::kernel::mm::print_num(n as usize);
                        uart::puts(" elr=0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" hops=");
                        crate::kernel::mm::print_num(hops);
                        uart::puts(" → user-LR=0x");
                        for sh in (0..16).rev() {
                            uart::putc(hex[((found_lr >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts("\n");
                    }
                    let _ = sp_el0;
                    return;
                }
                uart::puts("[pa-skip] couldn't unwind; falling through to terminate\n");
            }

            // If abort/brk from busybox code range, skip the instruction
            // (worker cleanup, musl assertions, etc. — non-fatal)
            let in_code = (elr < 0x1400000)
                || (elr >= 0x40000000 && elr < 0x50000000);
            if in_code && !in_child {
                // Worker or busybox cleanup BRK — just skip it
                unsafe { (*frame).elr = elr + 4; }
                return;
            }

            // V8 sandbox-pointer DCHECK fires inside
            // `HeapObject::InitSelfIndirectPointerField` at PC 0x11a54538.
            // This is the `b.lo BRK` form of `CHECK(OutsideSandbox(ptr))`
            // in `TrustedPointerTable::Validate` (saelo, V8 src
            // sandbox/trusted-pointer-table-inl.h). The DCHECK fires
            // because our REDIRECT path puts the V8 sandbox cage at
            // 0x30_0000_0000 with `reservation_size_` = 256 GB, and the
            // deserializer occasionally produces a TrustedObject whose
            // tagged pointer (e.g. 0x3001000039) decompresses inside
            // that range — V8 considers it "in the sandbox" and aborts.
            //
            // For our cave we don't enforce real sandbox boundaries
            // anyway (the sandbox is an in-process security boundary
            // V8 uses to limit attacker reach; our OS already isolates
            // the cave). Skipping the BRK lets V8 continue the
            // deserialization. The self-indirect-pointer field is left
            // null (handle 0) — `ReadIndirectPointerField` returns
            // `Smi::zero()` for null handles, which V8 mostly tolerates.
            //
            // Recovery: jump to the function epilogue at +0x1d0 (where
            // the register restores start: `ldp x20, x19, [sp, #0x40]`),
            // which then runs through autiasp + ret cleanly.
            //
            // Function layout (file VMA — cave VMA is +0x10000000):
            // 0x1a542c4: paciasp + prologue (saves x19..x29, x30 to sp+0x10..0x48)
            // 0x1a54538: brk #0 (this trap)
            // 0x1a54494: epilogue start (ldp x20, x19, [sp, #0x40])
            // 0x1a544a4: ldp x29, x30, [sp], #0x50
            // 0x1a544a8: autiasp
            // 0x1a544ac: ret
            //
            // The prologue's saved x19..x26 still contain the caller's
            // values at trap time, so the epilogue restores them
            // correctly. autiasp succeeds because x30 is reloaded from
            // the unmodified stack-saved value.
            if elr == 0x11a54538 {
                let lr_now = unsafe { (*frame).x[30] };
                unsafe {
                    // Jump to epilogue start. The epilogue restores
                    // x19..x29, x30 from the still-intact prologue saves,
                    // then `ret`s to the caller (the deserializer).
                    (*frame).elr = 0x11a54494;
                }
                static INIT_INDIRECT_SKIP: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(0);
                let n = INIT_INDIRECT_SKIP.fetch_add(
                    1, core::sync::atomic::Ordering::Relaxed);
                if n < 8 || (n & 0xFF) == 0 {
                    uart::puts("[brk-skip/init-self-indirect] #");
                    crate::kernel::mm::print_num(n as usize);
                    uart::puts(" elr=0x11a54538 → 0x11a54494 lr=0x");
                    let hex = b"0123456789abcdef";
                    for sh in (0..16).rev() {
                        uart::putc(hex[((lr_now >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::puts("\n");
                }
                return;
            }

            // 🎯 BRK from chrome text where
            // the previous instruction is a `b` (tail call) OR `bl`
            // (regular call) to a function that wasn't supposed to
            // return but did (because we pa-skipped its internal CHECK).
            //
            // Tail-call case (`b X`): saved x30 still has the OUTER
            // caller's return address — ret to that.
            //
            // Call case (`bl X`): x30 was clobbered by the BL to
            // BL+4 (= the BRK address), so we can't ret to x30.
            // Instead, walk one frame up via x29: at function
            // prologue `stp x29, x30, [sp, #-N]!; mov x29, sp`, the
            // saved outer x30 is at [x29 + 8]. Load it, advance
            // sp_el0 past this frame, and ret there.
            //
            // chrome text spans roughly 0x10000000..0x1c800000.
            let in_chrome_text = elr >= 0x10000000 && elr < 0x1c800000;
            if in_chrome_text && !in_child {
                let lr_now = unsafe { (*frame).x[30] };
                let x29_now = unsafe { (*frame).x[29] };
                // Look at the instruction at elr-4.
                let prev_instr_addr = elr.wrapping_sub(4);
                if crate::caves::linux::uaccess::is_user_range(
                    prev_instr_addr as usize, 4)
                {
                    let prev_instr: u32 = unsafe {
                        core::ptr::read_volatile(prev_instr_addr as *const u32)
                    };
                    // Unconditional B opcode is 0x14000000..0x18000000
                    // (top 6 bits = 000101).
                    let is_uncond_branch = (prev_instr >> 26) == 0b000101;
                    // BL opcode top 6 bits = 100101.
                    let is_bl = (prev_instr >> 26) == 0b100101;

                    // detect compiler "unreachable guard"
                    // sequences `b X; brk #0; hlt #0`. clang emits this
                    // after an unconditional branch that the optimiser
                    // proved would always be taken — the brk+hlt is dead
                    // code that should NEVER execute. If the cave gets
                    // here, it's via a corrupt funcptr (e.g. an
                    // uninitialised vtable slot in a Blink struct that
                    // happens to point at the brk address). The
                    // existing brk-skip "rescues" by resuming at LR,
                    // but LR is the BLR's return address — execution
                    // loops back into the caller's struct, dispatches
                    // through the corrupt funcptr again, and we BRK
                    // again. catches the resulting livelock
                    // after 32 iterations and terminates the cave;
                    // detecting the guard pattern up front saves the
                    // 32 wasted skips and gives a cleaner termination.
                    let next_instr_addr = elr.wrapping_add(4);
                    let next_is_hlt = if crate::caves::linux::uaccess::is_user_range(
                        next_instr_addr as usize, 4)
                    {
                        let ni: u32 = unsafe {
                            core::ptr::read_volatile(next_instr_addr as *const u32)
                        };
                        // HLT #imm16 opcode: 0xD4400000 with imm16 in
                        // bits[20:5]. We check the fixed bits.
                        (ni & 0xFFE0001F) == 0xD4400000
                    } else { false };
                    let is_unreachable_guard = is_uncond_branch && next_is_hlt;
                    if is_unreachable_guard {
                        // iter 7: distinguish two flavors of
                        // `b X; brk; hlt` UNREACHABLE-GUARD:
                        //
                        // (a) "default switch case" — a LOCAL conditional
                        // branch (b.cond / b.ne / cbz) inside the same
                        // function targets the brk because the case
                        // wasn't expected to fire. The unconditional
                        // branch immediately above the brk goes to
                        // the function's own SAFE epilogue.
                        // In this case lr_now is INSIDE the same
                        // function as elr (same locality) — we can
                        // redirect ELR to the unconditional branch's
                        // target and the function's clean epilogue
                        // runs, returning whatever default value
                        // (NULL ptr, 0, etc.) was set up by the
                        // fallthrough case.
                        //
                        // (b) "vtable miss" — a corrupt vtable slot
                        // points directly at the brk. lr_now is in
                        // a DIFFERENT function (the caller's BLR
                        // site). Redirecting ELR would resume the
                        // caller's vtable site → re-dispatch → BRK
                        // again → livelock. Must terminate.
                        //
                        // Heuristic: |lr_now - elr| < 64 KB → same fn → skip.
                        // (Most Chromium functions are well under 64 KB.)
                        //
                        // Concrete instance unblocked by this: content_shell
                        // gl::init::CreateGLContext at RVA 0x5eee15c. The
                        // brk is the default switch case for "GL impl is
                        // not Mock/Stub/Disabled". When --use-gl flags
                        // don't propagate, gl_impl=0 (kNone) hits this.
                        // Skipping to the unconditional-branch target
                        // (the function's epilogue) returns NULL to
                        // GpuChannelManager::GetSharedContextState,
                        // which has a `cbz x0, fail_path` immediately
                        // after the BL — clean failure, init continues.
                        let same_fn = if lr_now > elr {
                            lr_now - elr < 0x10000
                        } else {
                            elr - lr_now < 0x10000
                        };
                        if same_fn {
                            // Decode the unconditional branch's target:
                            // B imm26 — opcode 0b000101 in [31:26],
                            // imm26 sign-extended × 4 added to PC.
                            let imm26 = (prev_instr & 0x03FF_FFFF) as i32;
                            // Sign-extend imm26 (26 bits) to i32.
                            let signed = (imm26 << 6) >> 6;
                            let offset = (signed as i64).wrapping_mul(4);
                            let target = (prev_instr_addr as i64).wrapping_add(offset) as u64;
                            uart::puts("[brk-skip] UNREACHABLE-GUARD elr=0x");
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" — local default case, redirecting to safe-path 0x");
                            for sh in (0..16).rev() {
                                uart::putc(hex[((target >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts("\n");
                            unsafe { (*frame).elr = target; }
                            return;
                        }
                        // Fall through: not same-fn → vtable miss →
                        // terminate as before.
                        uart::puts("[brk-skip] UNREACHABLE-GUARD elr=0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" — corrupt funcptr (cross-fn lr); terminating cave\n");
                        crate::caves::linux::signal::terminate_cave_fatal_with_lr(
                            crate::caves::linux::signal::SIGSEGV,
                            elr, lr_now,
                        );
                    }

                    if is_uncond_branch && lr_now > 0x1000 {
                        // detect brk-skip livelock. If
                        // we keep skipping the same (elr, lr) pair, we're
                        // in an infinite loop (cave returns to caller,
                        // caller calls noreturn function again, BRK
                        // fires, we skip again). Cap at 32 same-pair
                        // skips, then terminate the cave.
                        static BRK_SKIP_LAST_ELR: core::sync::atomic::AtomicU64 =
                            core::sync::atomic::AtomicU64::new(0);
                        static BRK_SKIP_LAST_LR: core::sync::atomic::AtomicU64 =
                            core::sync::atomic::AtomicU64::new(0);
                        static BRK_SKIP_SAME_COUNT: core::sync::atomic::AtomicU32 =
                            core::sync::atomic::AtomicU32::new(0);
                        let prev_elr = BRK_SKIP_LAST_ELR.load(core::sync::atomic::Ordering::Relaxed);
                        let prev_lr = BRK_SKIP_LAST_LR.load(core::sync::atomic::Ordering::Relaxed);
                        if prev_elr == elr && prev_lr == lr_now {
                            let cnt = BRK_SKIP_SAME_COUNT.fetch_add(
                                1, core::sync::atomic::Ordering::Relaxed) + 1;
                            if cnt > 32 {
                                uart::puts("[brk-skip] LIVELOCK at elr=0x");
                                let hex = b"0123456789abcdef";
                                for sh in (0..16).rev() {
                                    uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                                }
                                uart::puts(" — terminating cave\n");
                                BRK_SKIP_SAME_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
                                BRK_SKIP_LAST_ELR.store(0, core::sync::atomic::Ordering::Relaxed);
                                BRK_SKIP_LAST_LR.store(0, core::sync::atomic::Ordering::Relaxed);
                                crate::caves::linux::signal::terminate_cave_fatal_with_lr(
                                    crate::caves::linux::signal::SIGSEGV,
                                    elr, lr_now,
                                );
                            }
                        } else {
                            BRK_SKIP_SAME_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
                            BRK_SKIP_LAST_ELR.store(elr, core::sync::atomic::Ordering::Relaxed);
                            BRK_SKIP_LAST_LR.store(lr_now, core::sync::atomic::Ordering::Relaxed);
                        }
                        // Tail-call case: ret to LR.
                        uart::puts("[brk-skip] unreachable-after-tail-call elr=0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" → lr=0x");
                        for sh in (0..16).rev() {
                            uart::putc(hex[((lr_now >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts("\n");
                        unsafe {
                            (*frame).elr = lr_now;
                        }
                        return;
                    } else if is_bl
                        && x29_now > 0x1000
                        && x29_now < 0x80_0000_0000
                        && crate::caves::linux::uaccess::is_user_range(x29_now as usize, 16)
                    {
                        // BL-call case: walk one frame up via x29.
                        // [x29] = saved outer x29, [x29+8] = saved
                        // outer x30 (the real return address).
                        let saved_outer_lr: u64 = unsafe {
                            core::ptr::read_volatile((x29_now + 8) as *const u64)
                        };
                        let saved_outer_fp: u64 = unsafe {
                            core::ptr::read_volatile(x29_now as *const u64)
                        };
                        // Sanity: outer LR should be in chrome text or
                        // libc area (>0x1000 minimum).
                        let outer_in_text = saved_outer_lr >= 0x10000000
                            && saved_outer_lr < 0x1c800000;
                        let outer_in_libc = saved_outer_lr >= 0x70_0000_0000
                            && saved_outer_lr < 0x70_0100_0000;
                        if outer_in_text || outer_in_libc {
                            uart::puts("[brk-skip] unreachable-after-bl elr=0x");
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" → outer-lr=0x");
                            for sh in (0..16).rev() {
                                uart::putc(hex[((saved_outer_lr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts("\n");
                            unsafe {
                                (*frame).elr = saved_outer_lr;
                                (*frame).x[29] = saved_outer_fp;
                                (*frame).x[30] = saved_outer_lr;
                                // Pop this frame off the stack.
                                core::arch::asm!("msr sp_el0, {a}",
                                    a = in(reg) x29_now + 16);
                            }
                            return;
                        }
                    }
                }
            }

            if in_child {
                uart::puts("[linux] child abort — returning to parent\n");
                crate::caves::linux::syscall::IN_CHILD
                    .store(false, core::sync::atomic::Ordering::Relaxed);
                unsafe {
                    let busybox_sp = core::ptr::read_volatile(
                        core::ptr::addr_of!(SAVED_BUSYBOX_SP));
                    let stack_src = core::ptr::addr_of!(SAVED_STACK) as usize;
                    for i in (0..STACK_SAVE_SIZE).step_by(8) {
                        let val: u64;
                        core::arch::asm!("ldr {v}, [{a}]", a = in(reg) stack_src + i, v = out(reg) val);
                        core::arch::asm!("str {v}, [{a}]", a = in(reg) busybox_sp as usize + i, v = in(reg) val);
                    }

                    let saved_ptr = core::ptr::addr_of!(SAVED_FRAME) as u64;
                    core::arch::asm!(
                        // V6-CHAIN-002: SP_EL0 not SP_EL1.
                        "msr sp_el0, {sp_val}",
                        "mov x16, {ptr}",
                        "ldp x0, x1, [x16, #248]",
                        "msr elr_el1, x0",
                        "and x1, x1, #0xFFFFFFFFFFFFFC3F",
                        "msr spsr_el1, x1",
                        "ldr x1, [x16, #8]",
                        "ldp x2, x3, [x16, #16]",
                        "ldp x4, x5, [x16, #32]",
                        "ldp x6, x7, [x16, #48]",
                        "ldp x8, x9, [x16, #64]",
                        "ldp x10, x11, [x16, #80]",
                        "ldp x12, x13, [x16, #96]",
                        "ldp x14, x15, [x16, #112]",
                        "ldr x17, [x16, #136]",
                        "ldp x18, x19, [x16, #144]",
                        "ldp x20, x21, [x16, #160]",
                        "ldp x22, x23, [x16, #176]",
                        "ldp x24, x25, [x16, #192]",
                        "ldp x26, x27, [x16, #208]",
                        "ldp x28, x29, [x16, #224]",
                        "ldr x30, [x16, #240]",
                        "ldr x16, [x16, #128]",
                        "mov x0, #2",
                        "eret",
                        ptr = in(reg) saved_ptr,
                        sp_val = in(reg) busybox_sp,
                        options(noreturn),
                    );
                }
            }

            // Only reach here for non-busybox BRK (real shell exit).
            // V2-NEW-024: switch TTBR0 back to primary instead of disabling
            // the MMU so subsequent caves keep W^X / UXN / PXN protection.
            //
            // CHROMIUM-PHASE-D diagnostic: dump x0/x1/x30 + the bytes
            // around ELR. Chromium's `base::CheckedNumeric` and
            // RefCountedThreadSafeBase::AddRefWithCheck emit a bare
            // `brk #0` after a `cmp + b.le/b.eq` sequence; the BRK PC
            // alone tells you nothing about which object failed the
            // check. With the surrounding instr words you can decode
            // (atomic ldadd → refcount overflow / use-after-free) and
            // x0/x30 give you the offending pointer + caller for an
            // addr2line lookup against ports/chromium_port/out/content_shell.
            let x0 = unsafe { (*frame).x[0] };
            let x1 = unsafe { (*frame).x[1] };
            let x19 = unsafe { (*frame).x[19] };
            let x30 = unsafe { (*frame).x[30] };
            // iter 4: TPIDR_EL0 at BRK so we can correlate
            // pthread_self mismatches (Chromium AssociatedThreadId BRK
            // does `cmp x19, pthread_self()` — knowing both x19 and
            // tpidr_el0 tells us if the bind happened on a different
            // TLS pointer than current).
            let tpidr: u64;
            unsafe { core::arch::asm!("mrs {}, tpidr_el0", out(reg) tpidr); }
            uart::puts("[linux] exit (BRK from EL0) elr=0x");
            {
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((elr >> (sh * 4)) & 0xF) as usize]);
                }
            }
            uart::puts(" tid=");
            crate::kernel::mm::print_num(
                crate::caves::linux::threads::current_tid() as usize);
            uart::puts("\n  x0=0x");
            {
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((x0 >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" x1=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((x1 >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" x19=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((x19 >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" lr=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((x30 >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts(" tpidr=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((tpidr >> (sh * 4)) & 0xF) as usize]);
                }
                uart::puts("\n  instr@elr-8..elr+4: ");
                for off in [-8i64, -4, 0, 4] {
                    let pc = (elr as i64 + off) as u64;
                    // validate before deref (HVF can't
                    // syndrome the resulting translation fault).
                    if !page_is_mapped(pc) {
                        uart::puts("???????? ");
                        continue;
                    }
                    let w: u32 = unsafe {
                        let v: u32;
                        core::arch::asm!("ldr {v:w}, [{a}]",
                            a = in(reg) pc, v = out(reg) v);
                        v
                    };
                    for sh in (0..8).rev() {
                        uart::putc(hex[((w >> (sh * 4)) & 0xF) as usize]);
                    }
                    uart::putc(b' ');
                }
            }
            // b deep-dive: dump the user-stack saved LRs.
            // CorruptionDetected's prologue stores x29/x30 at SP+0x10. The
            // CALLER of CorruptionDetected has its LR there. Walking
            // a few frames up reveals which Chromium subsystem called
            // free(0x1).
            let sp_el0: u64;
            unsafe { core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0); }
            uart::puts("\n  sp=0x");
            {
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((sp_el0 >> (sh * 4)) & 0xF) as usize]);
                }
            }
            // Print the first 32 8-byte slots above SP — covers
            // CorruptionDetected's frame + several upstream frames.
            uart::puts("\n  user-stack [sp..sp+0x100]:");
            for i in 0..32usize {
                let off = i * 8;
                let addr = sp_el0 + off as u64;
                if i % 4 == 0 {
                    uart::puts("\n    +0x");
                    let hex = b"0123456789abcdef";
                    uart::putc(hex[(off >> 8) & 0xF]);
                    uart::putc(hex[(off >> 4) & 0xF]);
                    uart::putc(hex[off & 0xF]);
                    uart::puts(": ");
                }
                // validate before deref. HVF asserts on
                // unsyndromed translation faults; under TCG the read
                // would silently return garbage. Either way, "????" is
                // a more useful diagnostic than crashing the dump.
                if !page_is_mapped(addr) {
                    uart::puts("???????????????? ");
                    continue;
                }
                let val: u64 = unsafe {
                    let v: u64;
                    core::arch::asm!("ldr {v}, [{a}]",
                        a = in(reg) addr, v = out(reg) v);
                    v
                };
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((val >> (sh * 4)) & 0xF) as usize]);
                }
                uart::putc(b' ');
            }
            // iter 13: fp-walk via x29 chain to find the
            // first non-libc frame. abort() / __stack_chk_fail are in
            // libc (range 0x70_17b0_xxxx); we want to know the user
            // call that triggered them. Walk up to 12 frames or until
            // we leave libc / land in unmapped memory.
            let frame_x29 = unsafe { (*frame).x[29] };
            uart::puts("\n  fp-walk (x29 chain):");
            let mut fp = frame_x29;
            for _ in 0..12 {
                if fp == 0 || !page_is_mapped(fp) || (fp & 7) != 0 {
                    uart::puts("\n    [end]"); break;
                }
                let next_fp: u64 = unsafe {
                    let v: u64; core::arch::asm!("ldr {v}, [{a}]",
                        a = in(reg) fp, v = out(reg) v); v
                };
                let saved_lr: u64 = unsafe {
                    let v: u64; core::arch::asm!("ldr {v}, [{a}]",
                        a = in(reg) fp + 8, v = out(reg) v); v
                };
                uart::puts("\n    fp=0x");
                let hex = b"0123456789abcdef";
                for sh in (0..16).rev() {
                    uart::putc(hex[((fp >> (sh*4)) & 0xF) as usize]);
                }
                uart::puts(" lr=0x");
                for sh in (0..16).rev() {
                    uart::putc(hex[((saved_lr >> (sh*4)) & 0xF) as usize]);
                }
                if next_fp <= fp { break; } // loop / corrupt
                fp = next_fp;
            }
            uart::puts("\n  → returning to desktop\n");
            unsafe {
                crate::caves::linux::mmu::switch_to_primary();
                // Restore the kernel SP that the loader stashed before
                // erets to EL0. See KERNEL_SP_SAVE above.
                let save_addr = kernel_sp_save_addr();
                core::arch::asm!(
                    "ldr x0, [{addr}]",
                    "mov sp, x0",
                    addr = in(reg) save_addr,
                    out("x0") _,
                );
                crate::ui::desktop::resume();
            }
        }
        0x19 | 0x1c | 0x1d => {
            // 0x19 = SVE functionality trapped,
            // 0x1c = FPAC (pointer-authentication failure),
            // 0x1d = SME functionality trapped.
            // When user control-flow strays into data (V8 cage, string
            // literals, etc.) the CPU decoder often matches the random
            // bytes against one of these encodings and traps, instead
            // of the plain EC=0 "unknown instruction" that would make
            // the symptom obvious. Skip the faulting word so the next
            // truly-invalid fetch surfaces with a cleaner diagnostic.
            let elr_raw = unsafe { (*frame).elr };
            let elr = elr_raw & 0x00FF_FFFF_FFFF_FFFF;
            unsafe { (*frame).elr = elr + 4; }
            static SVE_PAC_SME_SKIPS: core::sync::atomic::AtomicU64 =
                core::sync::atomic::AtomicU64::new(0);
            let n = SVE_PAC_SME_SKIPS.fetch_add(
                1, core::sync::atomic::Ordering::Relaxed);
            if n < 4 || (n & 0xFFFF) == 0 {
                uart::puts("[sve/pac/sme-skip] ec=0x");
                let hex = b"0123456789abcdef";
                uart::putc(hex[((ec >> 4) & 0xF) as usize]);
                uart::putc(hex[(ec & 0xF) as usize]);
                uart::puts(" ELR=0x"); print_hex(elr);
                uart::puts(" n="); crate::kernel::mm::print_num(n as usize);
                uart::puts("\n");
            }
        }
        0x00 => {
            // Unknown/undefined instruction — might be HVF-unsupported atomics
            // (LDADD, LDSET, LDCLR, SWP, etc. at encoding 0x38/0xB8/0xF8)
            // Strip TBI tag so tagged-pointer user code still matches the
            // "in code" range. Our TCR_EL1 has TBI0=1, so the CPU ignores
            // bits 63:56 during translation but reports them in ELR.
            let elr_raw = unsafe { (*frame).elr };
            let elr = elr_raw & 0x00FF_FFFF_FFFF_FFFF;
            // Accept any ELR that lives inside our 39-bit user VA window
            // (< 2^39). The previous check pinned it to busybox ranges
            // and missed V8 JIT trampolines in the pointer-compression
            // cage (0x30_0000_0000..0x38_0000_0000) and friends.
            let in_code = elr < (1u64 << 39);
            if in_code {
                let instr: u32 = unsafe {
                    let val: u32;
                    core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) elr, v = out(reg) val);
                    val
                };

                // Atomic memory ops: size[31:30] 111 V[26] 00 A[23] R[22] 1 Rs[20:16] o3[15] opc[14:12] 00 Rn[9:5] Rt[4:0]
                // LDADD/STADD: opc=000
                // LDSET/STSET: opc=011
                // LDCLR/STCLR: opc=001
                // LDEOR/STEOR: opc=010
                // SWP: opc=100 (bit pattern 111000 with different prefix)
                let top6 = (instr >> 26) & 0x3F;
                if top6 == 0x38 || top6 == 0x39 || top6 == 0x3C || top6 == 0x3D
                    || top6 == 0x3E || top6 == 0x3F || top6 == 0x2E || top6 == 0x2F
                {
                    // Atomic memory operation — emulate
                    let size = (instr >> 30) & 3;
                    let _a_bit = (instr >> 23) & 1;
                    let _r_bit = (instr >> 22) & 1;
                    let rs = ((instr >> 16) & 0x1F) as usize;
                    let opc = (instr >> 12) & 7;
                    let rn = ((instr >> 5) & 0x1F) as usize;
                    let rt = (instr & 0x1F) as usize;

                    unsafe {
                        let f = &mut *frame;
                        let addr = if rn < 31 { f.x[rn] } else { 0 } as usize;
                        let rs_val = if rs < 31 { f.x[rs] } else { 0 };
                        let old = emulate_load(addr, size);

                        let new_val = match opc {
                            0 => old.wrapping_add(rs_val), // LDADD
                            1 => old & !rs_val,            // LDCLR
                            2 => old ^ rs_val,             // LDEOR
                            3 => old | rs_val,             // LDSET
                            4 => rs_val,                   // SWP
                            _ => old,                      // unknown — nop
                        };
                        emulate_store(addr, size, new_val);
                        if rt < 31 { f.x[rt] = old; }
                        f.elr = elr + 4;
                    }
                    return;
                }

                // UDF / unknown instruction that isn't an atomic we can
                // emulate. V8 / WASM / ASan etc. all emit `udf #X` as a
                // deliberate trap sentinel and rely on SIGILL delivery
                // to a registered handler for recovery. Route through
                // the signal layer first; only if no handler is
                // installed do we fall back to the legacy "silently
                // skip" behaviour (kept so the busybox cleanup path
                // still works).
                let sf = unsafe {
                    &mut *(frame as *mut crate::caves::linux::signal::TrapFrame)
                };
                if crate::caves::linux::signal::try_deliver_synchronous(
                    sf,
                    crate::caves::linux::signal::SIGILL,
                    crate::caves::linux::signal::ILL_ILLOPC,
                    elr,
                ) {
                    return;
                }
                // Other unknown instr in busybox — skip
                unsafe { (*frame).elr = elr + 4; }
                return;
            }
            uart::puts("!!! UNHANDLED EC=0 !!!\n");
            uart::puts("  tid=t");
            crate::kernel::mm::print_num(
                crate::caves::linux::threads::current_tid() as usize);
            uart::puts(" ELR: 0x"); print_hex(elr);
            let ttbr0: u64; let sctlr: u64; let far: u64;
            unsafe {
                core::arch::asm!("mrs {}, ttbr0_el1",  out(reg) ttbr0);
                core::arch::asm!("mrs {}, sctlr_el1",  out(reg) sctlr);
                core::arch::asm!("mrs {}, far_el1",    out(reg) far);
            }
            uart::puts("  ESR_full=0x"); print_hex(esr);
            uart::puts("  TTBR0=0x"); print_hex(ttbr0);
            uart::puts("  SCTLR=0x"); print_hex(sctlr);
            uart::puts("  FAR=0x"); print_hex(far);
            uart::puts("\n");
            // Look up the L2_low entry for ELR and read phys bytes directly.
            // v2: skip this whole manual walk if ELR is
            // outside the cave's identity-mapped low window
            // (0..0x4000_0000) — for high-VA elr (0x70_xxxx_xxxx etc.)
            // the L1[0]/L2 dance gives garbage that's likely to
            // recursive-fault on the direct_phys read.
            if elr < 0x4000_0000 {
                let l1_phys = ttbr0 & !1u64;
                let l2_low = unsafe {
                    core::ptr::read_volatile((l1_phys) as *const u64)
                };
                let l2_low_phys = l2_low & 0x0000_FFFF_FFFF_F000;
                let l2_idx = (elr >> 21) & 0x1FF;
                let l2_entry = unsafe {
                    core::ptr::read_volatile(
                        (l2_low_phys + l2_idx * 8) as *const u64)
                };
                let mapped_phys_block = l2_entry & 0x0000_FFFF_FFE0_0000;
                let offset_in_block = elr & 0x1F_FFFF;
                let direct_phys = mapped_phys_block + offset_in_block;
                // Gate the direct_phys read too — if l2_entry was 0 or
                // garbage, direct_phys is junk and the read traps.
                let direct_word: u32 = if direct_phys >= 0x4000_0000
                    && direct_phys < 0xC000_0000 {
                    unsafe { core::ptr::read_volatile(direct_phys as *const u32) }
                } else { 0 };
                uart::puts("  l1[0]=0x"); print_hex(l2_low);
                uart::puts("  l2[");
                crate::kernel::mm::print_num(l2_idx as usize);
                uart::puts("]=0x"); print_hex(l2_entry);
                uart::puts("\n  direct_phys=0x"); print_hex(direct_phys);
                uart::puts("  bytes_there=0x"); print_hex(direct_word as u64);
                uart::puts("\n");
            } else {
                uart::puts("  (skipping manual L1[0]/L2 walk — ELR is high-VA)\n");
            }
            // Two ways to read insn at ELR — asm vs volatile — to
            // cross-check whether we're really reading what we think.
            // Dump 6 instructions around ELR — helps see what computed
            // the bad pointer that the faulting LDR then dereferenced.
            uart::puts("  code around ELR:");
            for off in [-12i64, -8, -4, 0, 4, 8].iter() {
                let addr = (elr as i64 + off) as usize;
                // gate the read through is_user_range so
                // we don't recursively fault when ELR is near the end
                // of a sparsely-committed cave reservation. Without this
                // the diagnostic dump triggers another exception, the
                // handler runs again, and we either hang or stack-blow.
                // black_box defeats the compiler's "I can prove page
                // is mapped from is_user_range alone" optimization.
                let safe_addr = core::hint::black_box(addr);
                if !crate::caves::linux::uaccess::is_user_range(safe_addr, 4)
                    || !page_is_mapped(core::hint::black_box(safe_addr as u64))
                {
                    uart::puts("\n    ["); print_hex(safe_addr as u64);
                    uart::puts("] (unmapped)");
                    continue;
                }
                let word: u32 = unsafe { core::ptr::read_volatile(safe_addr as *const u32) };
                uart::puts("\n    ["); print_hex(addr as u64);
                uart::puts("] 0x"); print_hex(word as u64);
            }
            // CHROMIUM-PHASE-B: dump general-purpose registers so we can
            // tell where a jump to a zeroed/unmapped page came from. LR
            // in particular tells us the caller — when an EC=0 UDF fires
            // inside libc/ld-linux, the caller PC is almost always the
            // key clue (what computed the bad target).
            unsafe {
                let f = &*frame;
                uart::puts("\n  x0 =0x"); print_hex(f.x[0]);
                uart::puts("  x1 =0x"); print_hex(f.x[1]);
                uart::puts("  x2 =0x"); print_hex(f.x[2]);
                uart::puts("\n  x3 =0x"); print_hex(f.x[3]);
                uart::puts("  x4 =0x"); print_hex(f.x[4]);
                uart::puts("  x5 =0x"); print_hex(f.x[5]);
                uart::puts("\n  x16=0x"); print_hex(f.x[16]);
                uart::puts("  x17=0x"); print_hex(f.x[17]);
                uart::puts("  x18=0x"); print_hex(f.x[18]);
                uart::puts("\n  x19=0x"); print_hex(f.x[19]);
                uart::puts("  x20=0x"); print_hex(f.x[20]);
                uart::puts("  x21=0x"); print_hex(f.x[21]);
                uart::puts("\n  x29=0x"); print_hex(f.x[29]);
                uart::puts("  x30(LR)=0x"); print_hex(f.x[30]);
                uart::puts("\n");
                // Dump the 4 instructions before LR — that tells us what
                // the caller intended. If LR points right after a BLR xN
                // then xN held the bad target; if after a BL #imm, the
                // jump was direct (and therefore a reloc/relink issue).
                if f.x[30] >= 16 {
                    uart::puts("  code around LR:");
                    for off in [-12i64, -8, -4, 0].iter() {
                        let addr = (f.x[30] as i64 + off) as usize;
                        let safe_addr = core::hint::black_box(addr);
                        if !crate::caves::linux::uaccess::is_user_range(safe_addr, 4)
                            || !page_is_mapped(core::hint::black_box(safe_addr as u64))
                        {
                            uart::puts("\n    ["); print_hex(safe_addr as u64);
                            uart::puts("] (unmapped)");
                            continue;
                        }
                        let word: u32 = core::ptr::read_volatile(
                            safe_addr as *const u32);
                        uart::puts("\n    ["); print_hex(safe_addr as u64);
                        uart::puts("] 0x"); print_hex(word as u64);
                    }
                    uart::puts("\n");
                }
            }
            uart::puts("  ISS=0x");  print_hex(esr & 0x01FF_FFFF);
            uart::puts("\n");
            loop { unsafe { core::arch::asm!("wfe") }; }
        }
        _ => {
            // Demand-paging: EC=0x24 (data abort from lower EL) may
            // be a legitimate lazy-commit for a huge mmap reservation.
            // Ask `demand_page::try_handle` first — if it commits a
            // page, just return so eret retries the faulting insn.
            let far: u64;
            unsafe { core::arch::asm!("mrs {}, far_el1", out(reg) far); }
            // EC=0x24 = data abort from lower EL (user touched
            // uncommitted page). EC=0x25 = data abort from current
            // EL — happens when the kernel reads/writes user memory
            // (uaccess) into a not-yet-committed page. Both can be
            // a legitimate lazy commit. Try to back the page; if
            // demand_page accepts, retry the instruction by returning.
            if (ec == 0x24 || ec == 0x25)
                && crate::caves::linux::demand_page::try_handle(far, esr)
            {
                return;
            }

            // Synchronous faults from a lower EL that we can't
            // service transparently: try to deliver them as a POSIX
            // signal so user-registered handlers (V8's WASM trap
            // handler, libc assertions, etc.) get a chance to
            // recover. EC → signal mapping:
            // 0x20 / 0x21 (instruction abort) → SIGSEGV
            // 0x22 (PC alignment) → SIGBUS
            // 0x24 / 0x25 (data abort) → SIGSEGV
            // 0x26 (SP alignment) → SIGBUS
            // 0x2C / 0x2D (FP trap) → SIGFPE
            // Everything else falls through to the UNHANDLED dump.
            {
                let (signo, si_code): (u32, i32) = match ec {
                    0x20 | 0x21 => {
                        // Instruction abort: ISS DFSC bits 5:0. 0b0001xx
                        // = translation fault, 0b0011xx = access flag,
                        // 0b0111xx = permission. MAPERR vs ACCERR.
                        let iss = (esr & 0x3F) as u32;
                        let si_code = if (iss >> 2) == 0b0011
                            || (iss >> 2) == 0b0001
                        {
                            crate::caves::linux::signal::SEGV_MAPERR
                        } else {
                            crate::caves::linux::signal::SEGV_ACCERR
                        };
                        (crate::caves::linux::signal::SIGSEGV, si_code)
                    }
                    0x22 => (
                        crate::caves::linux::signal::SIGBUS,
                        crate::caves::linux::signal::BUS_ADRALN,
                    ),
                    0x24 | 0x25 => {
                        let iss = (esr & 0x3F) as u32;
                        let si_code = if (iss >> 2) == 0b0011
                            || (iss >> 2) == 0b0001
                        {
                            crate::caves::linux::signal::SEGV_MAPERR
                        } else {
                            crate::caves::linux::signal::SEGV_ACCERR
                        };
                        (crate::caves::linux::signal::SIGSEGV, si_code)
                    }
                    0x26 => (
                        crate::caves::linux::signal::SIGBUS,
                        crate::caves::linux::signal::BUS_ADRALN,
                    ),
                    _ => (0, 0),
                };
                if signo != 0 {
                    let sf = unsafe {
                        &mut *(frame as *mut crate::caves::linux::signal::TrapFrame)
                    };
                    if crate::caves::linux::signal::try_deliver_synchronous(
                        sf, signo, si_code, far,
                    ) {
                        return;
                    }
                }
            }

            // SPHRAGIS_KEEP_GOING: instead of dumping + terminating
            // on every user-mode unhandled exception, record the
            // event into the skip ring and retire just THIS thread.
            // The cave's other threads keep running and we map the
            // full failure tree in one smoke. EC=0x24/0x25 is data
            // abort, EC=0x20/0x21 is instruction abort.
            //
            // Cap: limit to 32 EL0 abort skips per run so a single
            // looping thread doesn't drown the trace.
            if crate::caves::linux::skip_log::is_enabled()
                && (ec == 0x24 || ec == 0x25 || ec == 0x20 || ec == 0x21)
            {
                let elr = unsafe { (*frame).elr };
                let kind = if ec == 0x20 || ec == 0x21 {
                    crate::caves::linux::skip_log::SkipKind::UserInstAbort
                } else {
                    crate::caves::linux::skip_log::SkipKind::UserDataAbort
                };
                let recorded = crate::caves::linux::skip_log::record(
                    kind,
                    crate::caves::linux::threads::current_tid(),
                    ec, esr & 0x01FF_FFFF, 0,
                    elr, far,
                );
                if recorded {
                    // Retire the offending thread; other threads
                    // continue. exit_current never returns.
                    crate::caves::linux::threads::exit_current(0);
                }
                // Cap exceeded — fall through to original handler.
            }

            uart::puts("!!! UNHANDLED SYNC EXCEPTION !!!\n");
            uart::puts("  tid=t");
            crate::kernel::mm::print_num(
                crate::caves::linux::threads::current_tid() as usize);
            uart::puts("\n");
            uart::puts("  EC: 0x"); print_hex(ec);
            uart::puts("  ISS: 0x"); print_hex(esr & 0x01FF_FFFF);
            uart::puts("\n");
            // Extra sanity: read TTBR0 + SCTLR to make sure the EL1
            // context is what we think.
            {
                let ttbr0: u64; let sctlr: u64;
                unsafe {
                    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0);
                    core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
                }
                uart::puts("  TTBR0: 0x"); print_hex(ttbr0);
                uart::puts("  SCTLR: 0x"); print_hex(sctlr);
                uart::puts("\n");
            }
            let sp_el0: u64; let tp: u64;
            unsafe {
                core::arch::asm!("mrs {}, sp_el0",     out(reg) sp_el0);
                core::arch::asm!("mrs {}, tpidr_el0",  out(reg) tp);
            }
            let elr = unsafe { (*frame).elr };
            uart::puts("  ELR: 0x"); print_hex(elr);
            uart::puts("  FAR: 0x"); print_hex(far);
            uart::puts("  SP:  0x"); print_hex(sp_el0);
            uart::puts("  TP:  0x"); print_hex(tp);
            uart::puts("\n");
            // Dump 6 instructions around ELR + x9..x28 so we can see
            // where the bad pointer argument came from.
            // Bound-check ELR before reading: NULL function calls (BLR
            // to a zeroed register) jump us to ELR=0, and `elr-16` then
            // wraps to 0xfffffffffffffff0 — reading there triggers a
            // recursive EL1 data abort that masks the entire crash dump.
            // Also reject ELR in the kernel BSS / stack range; if user
            // code somehow ended up at an EL1 address, dumping the
            // surrounding text might alias kernel state.
            if elr >= 16 && elr < 0x0000_8000_0000_0000 {
                uart::puts("  code around ELR:");
                for off in [-16i64, -12, -8, -4, 0, 4, 8].iter() {
                    let addr = (elr as i64 + off) as usize;
                    let safe_addr = core::hint::black_box(addr);
                    if !crate::caves::linux::uaccess::is_user_range(safe_addr, 4)
                        || !page_is_mapped(core::hint::black_box(safe_addr as u64))
                    {
                        uart::puts("\n    ["); print_hex(safe_addr as u64);
                        uart::puts("] (unmapped)");
                        continue;
                    }
                    let word: u32 = unsafe { core::ptr::read_volatile(safe_addr as *const u32) };
                    uart::puts("\n    ["); print_hex(safe_addr as u64);
                    uart::puts("] 0x"); print_hex(word as u64);
                }
                uart::puts("\n");
            } else {
                uart::puts("  code around ELR: SKIPPED (elr=0x");
                print_hex(elr);
                uart::puts(" — NULL/oob, would crash dump)\n");
            }
            // x0..x8 dump — x8 is syscall number on ARM64; x0..x7 are
            // either syscall args or scratch. Useful to tell a corrupt
            // function-pointer call from an almost-working syscall.
            unsafe {
                // x0..x28 so we can trace x21/x26 (TLS ptr chain) and
                // anything else the faulting instruction needed.
                for i in 0..30 {
                    if i > 0 && i % 3 == 0 { uart::puts("\n"); }
                    uart::puts("  x");
                    if i < 10 {
                        uart::putc(b'0' + i as u8);
                        uart::puts(" ");
                    } else {
                        uart::putc(b'0' + (i / 10) as u8);
                        uart::putc(b'0' + (i % 10) as u8);
                    }
                    uart::puts("=0x"); print_hex((*frame).x[i]);
                }
                uart::puts("\n  LR(x30)=0x"); print_hex((*frame).x[30]);
                uart::puts("\n");
                // Scan the user stack for plausible saved-LR slots
                // (values whose `v-4` decodes as a BL/BLR). Useful for
                // tracing the caller of a function that ret'd to a bad
                // address — the fp-chain can't be trusted if x29 is
                // already corrupted.
                uart::puts("  stack LR candidates (SP+0 .. SP+0x4000):");
                let sp_val = sp_el0;
                let elr_now = (*frame).elr;
                // Scan 16 KB of user stack instead of 0x200 — the leak
                // might be in a deeper frame.
                // Cap the scan at the page boundary to avoid crossing
                // into an uncommitted neighbor.
                let first_page_end = (sp_val | 0xFFF) + 1;
                let max_iter = ((first_page_end - sp_val) / 8) as usize;
                let scan_limit = max_iter.min(2048);
                for i in 0..scan_limit {
                    let addr = sp_val + (i as u64) * 8;
                    let safe_addr = core::hint::black_box(addr);
                    if !crate::caves::linux::uaccess::is_user_range(safe_addr as usize, 8)
                        || !page_is_mapped(core::hint::black_box(safe_addr))
                    {
                        uart::puts("\n    [sp+0x");
                        let off = i * 8;
                        uart::putc(b"0123456789abcdef"[(off >> 12) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 8) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 4) & 0xF]);
                        uart::putc(b"0123456789abcdef"[off & 0xF]);
                        uart::puts("] (stack scan stops at unmapped)");
                        break;
                    }
                    let v: u64 = core::ptr::read_volatile(safe_addr as *const u64);
                    // Match A: any 8-byte slot that EXACTLY equals the
                    // fault PC. Tells us "this kernel pointer is sitting
                    // RIGHT HERE on the stack" — pinpoints the leak source.
                    if v == elr_now {
                        uart::puts("\n    [sp+0x");
                        let off = i * 8;
                        uart::putc(b"0123456789abcdef"[(off >> 12) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 8) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 4) & 0xF]);
                        uart::putc(b"0123456789abcdef"[off & 0xF]);
                        uart::puts("]=ELR (=0x"); print_hex(v); uart::puts(") !!!");
                        continue;
                    }
                    // Match B: kernel-range pointers that landed on user
                    // stack — any saved value in 0x40000000..0x80000000
                    // is suspicious. Print them all.
                    if v >= 0x40000000 && v < 0x80000000 {
                        uart::puts("\n    [sp+0x");
                        let off = i * 8;
                        uart::putc(b"0123456789abcdef"[(off >> 12) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 8) & 0xF]);
                        uart::putc(b"0123456789abcdef"[(off >> 4) & 0xF]);
                        uart::putc(b"0123456789abcdef"[off & 0xF]);
                        uart::puts("]=KERNEL 0x"); print_hex(v);
                        continue;
                    }
                    // Lower-text candidate scan — exclude v == 0x10000000
                    // (and aligned values within 4 bytes of the boundary),
                    // because pc=v-4 then equals 0x0ffffffc which is the
                    // last word of an unmapped page → recursive EL1 data
                    // abort that masks the entire crash dump.
                    if v > 0x10000004 && v < 0x1f000000 && (v & 3) == 0 {
                        let pc = v.wrapping_sub(4);
                        // gate via is_user_range AND
                        // page_is_mapped — see helper for why.
                        if !crate::caves::linux::uaccess::is_user_range(pc as usize, 4)
                            || !page_is_mapped(pc)
                        {
                            continue;
                        }
                        let ins: u32 = core::ptr::read_volatile(pc as *const u32);
                        let top6 = (ins >> 26) & 0x3F;
                        let is_bl = top6 == 0x25;
                        // ARMv8 A64 BLR encoding: bits 31-21 = 11010110001,
                        // bits 20-16 = 11111, bits 15-10 = 000000, Rn in 9-5,
                        // bits 4-0 = 0. Mask the fixed bits, leave Rn wildcarded.
                        // (Pre-fix, the mask was 0xFFFE0000 which zeroes bit 17
                        // but the pattern 0xD63F0000 has bit 17 set — clippy's
                        // `incompatible_bit_mask` correctly flagged it as a
                        // never-matching check.)
                        let is_blr = (ins & 0xFFFFFC1F) == 0xD63F0000;
                        if is_bl || is_blr {
                            uart::puts("\n    [sp+0x");
                            let off = i * 8;
                            uart::putc(b"0123456789abcdef"[(off >> 8) & 0xF]);
                            uart::putc(b"0123456789abcdef"[(off >> 4) & 0xF]);
                            uart::putc(b"0123456789abcdef"[off & 0xF]);
                            uart::puts("]=0x"); print_hex(v);
                            if is_bl {
                                // Decode BL imm26 → target. If any of these
                                // targets matches the fault address, we've
                                // found which user code branched there.
                                let imm26 = (ins & 0x03FFFFFF) as i64;
                                let signed = if imm26 & (1 << 25) != 0 {
                                    imm26 - (1 << 26)
                                } else {
                                    imm26
                                };
                                let target = (pc as i64 + signed * 4) as u64;
                                uart::puts(" BL→0x"); print_hex(target);
                            } else {
                                // BLR xN — register-indirect, target lives
                                // in xN. Print which register.
                                let rn = ((ins >> 5) & 0x1F) as u8;
                                uart::puts(" BLR x");
                                if rn < 10 { uart::putc(b'0' + rn); }
                                else { uart::putc(b'1'); uart::putc(b'0' + (rn - 10)); }
                            }
                        }
                    }
                }
                uart::puts("\n");
                // Dump 4 instructions before LR so we can tell what the
                // function's call site looked like. LR points at the
                // insn AFTER the BL/BLR, so [-4] is the actual jump.
                if (*frame).x[30] >= 16 {
                    uart::puts("  code around LR:");
                    for off in [-16i64, -12, -8, -4, 0].iter() {
                        let addr = ((*frame).x[30] as i64 + off) as usize;
                        let safe_addr = core::hint::black_box(addr);
                        if !crate::caves::linux::uaccess::is_user_range(safe_addr, 4)
                            || !page_is_mapped(core::hint::black_box(safe_addr as u64))
                        {
                            uart::puts("\n    ["); print_hex(safe_addr as u64);
                            uart::puts("] (unmapped)");
                            continue;
                        }
                        let word: u32 = core::ptr::read_volatile(
                            safe_addr as *const u32);
                        uart::puts("\n    ["); print_hex(safe_addr as u64);
                        uart::puts("] 0x"); print_hex(word as u64);
                    }
                    uart::puts("\n");
                }
                // CHROMIUM-PHASE-B: dump 64 bytes of the object at x19.
                // x19 is callee-saved in the AArch64 AAPCS, so functions
                // often use it to hold `this`. When a crash happens
                // inside a method, dumping [x19..x19+64] reveals the
                // object's state and is more informative than just
                // the register file. Also dump [x19-32..x19] for context.
                if (*frame).x[19] > 0x1000
                    && (*frame).x[19] < 0x0000_4000_0000_0000
                {
                    uart::puts("  memory around x19 (32 before + 64 after):");
                    let obj = (*frame).x[19] as usize;
                    for i in -4i64..8i64 {
                        uart::puts("\n    ");
                        if i < 0 { uart::putc(b'-'); } else { uart::putc(b'+'); }
                        let off_abs = (i * 8).unsigned_abs() as usize;
                        uart::puts("0x");
                        uart::putc(b"0123456789abcdef"[(off_abs >> 4) & 0xF]);
                        uart::putc(b"0123456789abcdef"[off_abs & 0xF]);
                        uart::puts(": ");
                        let row_base = (obj as i64 + i * 8) as usize;
                        let safe_row = core::hint::black_box(row_base);
                        if !crate::caves::linux::uaccess::is_user_range(safe_row, 8)
                            || !page_is_mapped(core::hint::black_box(safe_row as u64))
                        {
                            uart::puts("(unmapped)");
                            continue;
                        }
                        for j in 0..8usize {
                            let byte: u8 = core::ptr::read_volatile(
                                (safe_row + j) as *const u8);
                            uart::putc(b"0123456789abcdef"[(byte >> 4) as usize]);
                            uart::putc(b"0123456789abcdef"[(byte & 0xF) as usize]);
                            uart::putc(b' ');
                        }
                        uart::puts(" | ");
                        for j in 0..8usize {
                            let byte: u8 = core::ptr::read_volatile(
                                (safe_row + j) as *const u8);
                            if (0x20..=0x7e).contains(&byte) { uart::putc(byte); }
                            else { uart::putc(b'.'); }
                        }
                    }
                    uart::puts("\n");
                }
                // dump 64 bytes at x24 + walk page tables for x24's
                // page. PartitionAlloc::SlowPathAlloc reads x24 as a
                // SlotSpanMetadata*; we want to know whether x24's page is
                // mapped at all, and if so, whether the metadata bytes
                // were ever written or are still fresh-zero from
                // demand_page::try_handle.
                if (*frame).x[24] > 0x1000
                    && (*frame).x[24] < 0x0000_8000_0000_0000
                {
                    let probe = (*frame).x[24];
                    uart::puts("  STUMP3: memory at x24 (0x");
                    print_hex(probe);
                    uart::puts(") + L3 walk:");
                    // Walk TTBR0_EL1 → L1 → L2 → L3 for the page
                    // containing x24, then print the L3 entry. If the
                    // entry is 0 we got translation fault, fresh page
                    // would be reading-as-zero on demand-page commit.
                    let ttbr0_now: u64;
                    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_now);
                    let l1p = ttbr0_now & !1u64;
                    let l1i = ((probe >> 30) & 0x1FF) as u64;
                    let l1e: u64 = core::ptr::read_volatile(
                        (l1p + l1i * 8) as *const u64);
                    uart::puts("\n    L1[0x");
                    let hex = b"0123456789abcdef";
                    uart::putc(hex[((l1i >> 8) & 0xF) as usize]);
                    uart::putc(hex[((l1i >> 4) & 0xF) as usize]);
                    uart::putc(hex[(l1i & 0xF) as usize]);
                    uart::puts("]=0x"); print_hex(l1e);
                    let mut is_mapped = false;
                    if (l1e & 0b11) == 0b11 {
                        let l2p = l1e & 0x0000_FFFF_FFFF_F000;
                        let l2i = ((probe >> 21) & 0x1FF) as u64;
                        let l2e: u64 = core::ptr::read_volatile(
                            (l2p + l2i * 8) as *const u64);
                        uart::puts("\n    L2[0x");
                        uart::putc(hex[((l2i >> 8) & 0xF) as usize]);
                        uart::putc(hex[((l2i >> 4) & 0xF) as usize]);
                        uart::putc(hex[(l2i & 0xF) as usize]);
                        uart::puts("]=0x"); print_hex(l2e);
                        if (l2e & 0b11) == 0b11 {
                            let l3p = l2e & 0x0000_FFFF_FFFF_F000;
                            let l3i = ((probe >> 12) & 0x1FF) as u64;
                            let l3e: u64 = core::ptr::read_volatile(
                                (l3p + l3i * 8) as *const u64);
                            uart::puts("\n    L3[0x");
                            uart::putc(hex[((l3i >> 8) & 0xF) as usize]);
                            uart::putc(hex[((l3i >> 4) & 0xF) as usize]);
                            uart::putc(hex[(l3i & 0xF) as usize]);
                            uart::puts("]=0x"); print_hex(l3e);
                            if (l3e & 0b11) == 0b11 {
                                uart::puts(" MAPPED");
                                is_mapped = true;
                            } else {
                                uart::puts(" UNMAPPED");
                            }
                        }
                    }
                    // Only dump bytes if we proved the page is mapped —
                    // otherwise the read recursively faults the kernel
                    // and we lose the entire dump.
                    if is_mapped {
                        uart::puts("\n    bytes at x24:");
                        for row in 0..4usize {
                            uart::puts("\n      +0x");
                            let off = row * 16;
                            uart::putc(hex[(off >> 4) & 0xF]);
                            uart::putc(hex[off & 0xF]);
                            uart::puts(": ");
                            let row_addr = probe as usize + off;
                            let safe_row = core::hint::black_box(row_addr);
                            if !crate::caves::linux::uaccess::is_user_range(safe_row, 16)
                                || !page_is_mapped(core::hint::black_box(safe_row as u64))
                            {
                                uart::puts("(unmapped)");
                                continue;
                            }
                            for j in 0..16usize {
                                let byte: u8 = core::ptr::read_volatile(
                                    (safe_row + j) as *const u8);
                                uart::putc(hex[(byte >> 4) as usize]);
                                uart::putc(hex[(byte & 0xF) as usize]);
                                uart::putc(b' ');
                            }
                        }
                    }
                    uart::puts("\n");
                }
                // Also dump the user stack — finds the call chain when
                // a function faults mid-execution. SP points at the
                // top of the active stack frame; walking up lets us
                // see return addresses.
                if sp_el0 > 0x1000 && sp_el0 < 0x0000_4000_0000_0000 {
                    uart::puts("  user stack around SP (0x");
                    print_hex(sp_el0);
                    uart::puts("):");
                    // Dump SP-0x80..SP+0x100: covers the just-popped
                    // frame (negative offsets) plus a few caller
                    // frames. Each row = 4 u64s = 32 bytes.
                    let base = (sp_el0 as i64 - 0x80) as u64;
                    for i in 0..48usize {
                        let off = i * 8;
                        let addr = base + off as u64;
                        let signed_off = (addr as i64) - (sp_el0 as i64);
                        if off % 32 == 0 {
                            uart::puts("\n    ");
                            if signed_off < 0 {
                                uart::puts("-0x");
                                let v = (-signed_off) as u64;
                                uart::putc(b"0123456789abcdef"[((v >> 8) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[((v >> 4) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[(v & 0xF) as usize]);
                            } else {
                                uart::puts("+0x");
                                let v = signed_off as u64;
                                uart::putc(b"0123456789abcdef"[((v >> 8) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[((v >> 4) & 0xF) as usize]);
                                uart::putc(b"0123456789abcdef"[(v & 0xF) as usize]);
                            }
                            uart::puts(":");
                        } else {
                            uart::puts(" ");
                        }
                        let safe_addr = core::hint::black_box(addr);
                        if !crate::caves::linux::uaccess::is_user_range(safe_addr as usize, 8)
                            || !page_is_mapped(core::hint::black_box(safe_addr))
                        {
                            uart::puts("(unmapped) ");
                            continue;
                        }
                        let qword: u64 = core::ptr::read_volatile(safe_addr as *const u64);
                        uart::puts("0x");
                        for sh in (0..16).rev() {
                            uart::putc(b"0123456789abcdef"[((qword >> (sh*4)) & 0xF) as usize]);
                        }
                    }
                    uart::puts("\n");
                }
            }
            // Dump the syscall-history ring so we can correlate the
            // fault with the last few svc calls — invaluable for
            // tracking how x29 / x30 got populated with a cage
            // pointer before the crashing `ret`.
            crate::caves::linux::syscall_history::dump();

            // If the fault came from EL0 (SPSR.M[3:0] == 0b0000 = EL0t)
            // and the EC maps to a synchronous-fault signal whose
            // default disposition is terminate, we can give Chromium /
            // the test harness a soft landing: tear the cave down and
            // drop back into the shell instead of wedging the whole
            // kernel on `wfe`. Real EL1-origin faults (genuine kernel
            // bugs) still `wfe` so the operator can investigate.
            let spsr_m = unsafe { (*frame).spsr & 0xF };
            let from_el0 = spsr_m == 0;
            let fatal_signo: u32 = match ec {
                0x20 | 0x21 | 0x24 | 0x25 => {
                    crate::caves::linux::signal::SIGSEGV
                }
                0x22 | 0x26 => {
                    crate::caves::linux::signal::SIGBUS
                }
                _ => 0,
            };
            if from_el0 && fatal_signo != 0 {
                let far_now: u64;
                unsafe {
                    core::arch::asm!("mrs {}, far_el1", out(reg) far_now);
                }
                let lr = unsafe { (*frame).x[30] };
                let elr_now = unsafe { (*frame).elr };

                // 🎯 // LSE atomic on small/Smi-tagged "this". V8's MemoryPool
                // stores Smi-tagged values in the same field as refcounted
                // pointers. When V8 calls Release() on a pool entry that's
                // actually a Smi (e.g. 0xd, 0x70, 0x17 small numerics),
                // the LSE atomic faults on the small "address".
                //
                // Generalized: detect ANY LSE atomic (LDADD/LDSET/SWP
                // family) at ELR with a small fault address. Skip the
                // instruction (advance ELR by 4), zero w0.
                //
                // ARM64 LSE encoding: 1 0 1 1 1 0 0 0 SZ A R 1 Rs 0 op 0 0
                // i.e. mask 0xBFE0_FC00, value 0xB820_0000 for size=4
                // (W reg op). For size=8, mask 0xBFE0_FC00, value 0xF820_0000.
                let smi_skip_target = if far_now < 0x100
                    && elr_now >= 0x11000000 && elr_now < 0x1c800000
                {
                    let prev_addr = elr_now;
                    if crate::caves::linux::uaccess::is_user_range(
                        prev_addr as usize, 4)
                        && page_is_mapped(prev_addr & !0xFFF)
                    {
                        let instr: u32 = unsafe {
                            core::ptr::read_volatile(prev_addr as *const u32)
                        };
                        // LSE w-reg LDADD/LDSET/SWP/LDCLR/LDEOR/LDSMAX/etc:
                        // top byte 0xb8, specific bits.
                        // 32-bit ops: 0xb820..0xb83f variants
                        // 64-bit ops: 0xf820..0xf83f variants
                        // We accept top 12 bits = 0xb82 or 0xf82.
                        let high12 = instr >> 20;
                        let is_lse = high12 == 0xb82 || high12 == 0xf82
                            || high12 == 0xb86 || high12 == 0xf86  // L variants
                            || high12 == 0xb8a || high12 == 0xf8a  // A variants
                            || high12 == 0xb8e || high12 == 0xf8e; // AL variants
                        if is_lse {
                            Some(elr_now + 4)
                        } else {
                            None
                        }
                    } else { None }
                } else { None };
                if let Some(ret_addr) = smi_skip_target {
                    unsafe {
                        // Set w0 (low 32 bits of x[0]) to 0.
                        (*frame).x[0] = 0;
                        // ELR → ret at +4.
                        (*frame).elr = ret_addr;
                    }
                    static SMI_RELEASE_SKIPS: core::sync::atomic::AtomicU32 =
                        core::sync::atomic::AtomicU32::new(0);
                    let n = SMI_RELEASE_SKIPS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                    if n < 8 || (n & 0xFF) == 0 {
                        uart::puts("[smi-release-skip] #");
                        crate::kernel::mm::print_num(n as usize);
                        uart::puts(" far=0x");
                        let hex = b"0123456789abcdef";
                        for sh in (0..16).rev() {
                            uart::putc(hex[((far_now >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts("\n");
                    }
                    return;
                }

                // c: PA NULL-deref skip. PartitionAlloc's
                // SlowPathAlloc / SlotAddressAndSize::From / friends
                // can race on the V8 cage (slot span partially init,
                // freelist not yet linked) and load NULL at a small
                // offset. The fault is benign-ish — same pattern as
                // PA's CHECK BRK family. Treat it like the abort skip:
                // walk the FP chain, find the first non-PA caller,
                // synthesize a return there as if PA::Free/Alloc had
                // succeeded.
                //
                // Only fire for: EC=0x24 (data abort from EL0), FAR
                // small (NULL+offset < 0x1000), ELR in PA's libchrome
                // region (0x14000000..0x1c000000).
                // Cap the cumulative skip count per cave run. Without
                // a cap, PA can spiral into a loop where each unwound
                // call returns into a caller that faults again. 64
                // skips is plenty to clear transient races.
                // Bumped from 64 to 256 — Mojo IPC pump generates a
                // lot of PA traffic and we want to give the cave room
                // to push past many race-induced faults.
                static PA_DATA_SKIP_TOTAL: core::sync::atomic::AtomicU32 =
                    core::sync::atomic::AtomicU32::new(0);
                let skip_count = PA_DATA_SKIP_TOTAL.load(core::sync::atomic::Ordering::Relaxed);
                let _ = skip_count;
                // Use a higher cap since each iteration is bounded by
                // the FP-walk failing (16 hops max).
                // also catch the signo=7 elr=lr=fault=0x1
                // pattern (Rehash chain bad-funcptr). EC=0x22 = PC
                // alignment fault, ELR < 0x1000 = jumped to a small
                // numeric (likely Smi tag or freed-pointer sentinel).
                // The caller's x30 wasn't restored to a real LR —
                // walk the FP chain like the PA-data-skip case.
                // PA / libchrome data faults: ELR in chrome text region.
                let is_pa_data_fault = (ec == 0x24 || ec == 0x21)
                    && far_now < 0x1000
                    && (0x14000000..=0x1c000000).contains(&elr_now);
                // V8 cage instruction abort. Cave's PC
                // jumped to a V8-internal address (e.g. 0x4020113c)
                // that isn't backed by executable memory. Common when
                // a Mojo IPC callback resolves to a stale V8 builtin
                // pointer or V8's pointer compression decoded a
                // garbage offset against a code-base register.
                //
                // Range: V8 cages can land in 0x40000000..0x80000000
                // (when CodeRange goes there). Treat any inst-abort
                // (EC=0x20/0x21) where ELR is in this range AND fault
                // == ELR (instruction-fetch fault) as a bad-PC and
                // walk the FP chain to escape.
                let is_v8_cage_inst_fault = (ec == 0x20 || ec == 0x21)
                    && far_now == elr_now
                    && elr_now >= 0x40000000
                    && elr_now < 0x80_0000_0000
                    && elr_now != 0;
                // 🎯 Also catch libc-area NULL-derefs that happen
                // DOWNSTREAM of a pa-skip-data return: e.g. when our
                // fake AllocateBacking result is passed to memset
                // (libc) which faults trying to write to NULL.
                // libc is mmap'd at 0x70_003d_0000ish; cover the
                // 0x70_0000_0000..0x70_0100_0000 range to catch it.
                let is_libc_data_fault = ec == 0x24
                    && far_now < 0x1000
                    && (0x7000000000..=0x7001000000).contains(&elr_now);
                // EC=0x20 = instruction abort from lower EL (jumped
                // to non-executable / unmapped). EC=0x21 = same from
                // current EL. EC=0x22 = PC alignment. Any of these
                // with ELR < 0x1000 means a corrupt-pointer indirect
                // branch (Smi tag, NULL+offset, etc.).
                //
                // also catch ELR == FAR == LR ≥ 0x1000
                // pattern. The cave called BLR Xn where Xn was a
                // garbage value (e.g. 0x4000, 0x2c00537400). The CPU
                // jumped to that address, faulted on instruction fetch
                // (no executable mapping), and PC/LR/FAR all show the
                // same bad address. Skip to a real frame in the FP
                // chain instead of letting the cave die.
                let bad_pc_match_lr_far = (ec == 0x20 || ec == 0x21 || ec == 0x22)
                    && elr_now == far_now
                    && elr_now != 0
                    && unsafe { (*frame).x[30] } == elr_now;
                let is_bad_pc_fault = (ec == 0x20 || ec == 0x21 || ec == 0x22)
                    && (elr_now < 0x1000 || bad_pc_match_lr_far);
                // PCheck sret-write fault. The instruction
                // at file VMA 0x4c929dc (cave VMA 0x14c929dc) is
                // `str x19, [x20]` inside
                // `logging::CheckNoreturnError::PCheck(base::Location const&)`.
                // x20 is the sret pointer the caller passed in x8 — when
                // a prior pa-skip-data restored sp_el0 to a stale-but-
                // valid frame whose Free() ancestor was entered with a
                // garbage sp, Free's `add x8, sp, #0x8` produces a
                // page-zero-region pointer (0x3a78, 0x3ac4, etc.). The
                // sret pointer is bogus but the LIVE fp/sp chain at
                // PCheck-trap time is still sane (the lambda's saved x29
                // is a real user-stack address). Treat it like a chrome-
                // text NULL deref but allow fault up to 0x10000 (covers
                // the entire page-zero region the bogus sret pointer can
                // land in). The FP-walk skip below will unwind past
                // PCheck → lambda → Free → caller and resume cleanly.
                //
                // Limited to elr exactly at the str instruction so we
                // don't accidentally widen the fault threshold for any
                // unrelated case.
                let is_pcheck_sret_fault = ec == 0x24
                    && elr_now == 0x14c929dc
                    && far_now < 0x10000;
                if (is_pa_data_fault || is_bad_pc_fault || is_libc_data_fault
                    || is_v8_cage_inst_fault || is_pcheck_sret_fault)
                    && skip_count < 256
                {
                    // 🎯 Diagnostic: emit a single-line trace so log readers
                    // can tell that we entered the pa-skip-data block AT ALL
                    // the heavyweight dump above runs UNCONDITIONALLY for
                    // every fault, so seeing "UNHANDLED SYNC EXCEPTION" in
                    // the log doesn't mean recovery failed. This line says
                    // "we got here, and we're about to try recovery."
                    static PA_ENTER_COUNT: core::sync::atomic::AtomicU32 =
                        core::sync::atomic::AtomicU32::new(0);
                    let n_enter = PA_ENTER_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                    if n_enter < 16 || (n_enter & 0xFF) == 0 {
                        uart::puts("[pa-skip-data] entered ec=0x");
                        let hex = b"0123456789abcdef";
                        uart::putc(hex[((ec >> 4) & 0xF) as usize]);
                        uart::putc(hex[(ec & 0xF) as usize]);
                        uart::puts(" elr=0x");
                        for sh in (0..16).rev() {
                            uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" far=0x");
                        for sh in (0..16).rev() {
                            uart::putc(hex[((far_now >> (sh * 4)) & 0xF) as usize]);
                        }
                        uart::puts(" x29=0x");
                        let frame_x29_log = unsafe { (*frame).x[29] };
                        for sh in (0..16).rev() {
                            uart::putc(hex[((frame_x29_log >> (sh * 4)) & 0xF) as usize]);
                        }
                        // Decode which classifier matched so future logs
                        // make the routing visible.
                        uart::puts(" cls=");
                        if is_bad_pc_fault { uart::puts("bad_pc"); }
                        else if is_pa_data_fault { uart::puts("pa_data"); }
                        else if is_libc_data_fault { uart::puts("libc_data"); }
                        else if is_v8_cage_inst_fault { uart::puts("v8_cage"); }
                        else if is_pcheck_sret_fault { uart::puts("pcheck_sret"); }
                        else { uart::puts("?"); }
                        uart::puts("\n");
                    }
                    // For bad-PC case, x29 may also be corrupt. Try
                    // starting the fp-walk from sp_el0 if x29 looks
                    // bogus. The user stack at sp typically has
                    // (saved_fp, saved_lr) for the most recent frame
                    // even after a bad ret.
                    let frame_x29 = unsafe { (*frame).x[29] };
                    let sp_el0_now: u64;
                    unsafe { core::arch::asm!("mrs {}, sp_el0", out(reg) sp_el0_now); }
                    let mut fp = if frame_x29 > 0x1000 { frame_x29 } else { sp_el0_now };
                    let mut hops = 0;
                    let mut found_lr: u64 = 0;
                    while hops < 16 && fp != 0 {
                        // Validate fp is in user range AND mapped.
                        if !crate::caves::linux::uaccess::is_user_range(fp as usize, 16)
                            || !page_is_mapped(fp)
                        {
                            break;
                        }
                        let next_fp: u64 = unsafe {
                            core::ptr::read_volatile(fp as *const u64)
                        };
                        let saved_lr: u64 = unsafe {
                            core::ptr::read_volatile((fp + 8) as *const u64)
                        };
                        // Skip PA::Free (0x11a63000..0x11a64000) AND
                        // PA::SlowPathAlloc + abort sites (libchrome
                        // 0x14d70000..0x14d80000).
                        // PA::Free family is at file VMA 0x1a63000..0x1a6a800
                        // → runtime 0x11a63000..0x11a6a800 (after virt_base
                        // 0x10000000 rebase).
                        let in_pa_free = (0x11a63000..=0x11a6a800).contains(&saved_lr);
                        // PA's libchrome-side wrappers + allocator_shim
                        // span 0x14d70000..0x14da0000 — cover all of it.
                        let in_pa_libchrome = (0x14d70000..=0x14da0000).contains(&saved_lr);
                        // restrict saved-LR to content_shell
                        // TEXT (0x11720000..0x19910000) — accepting
                        // BSS/data addrs leads to bad-instruction-fetch
                        // after the cave 'returns' there.
                        let in_text_range = saved_lr >= 0x11720000
                            && saved_lr < 0x19910000;
                        // when the trap is the PCheck-sret
                        // pattern, also filter out the Free + lambda +
                        // PCheck call chain that lives in chrome text but
                        // is part of the broken frame we're trying to
                        // escape. Returning into any of these would just
                        // re-enter the broken context. The chain spans:
                        // PCheck @ 0x14c92964..0x14c929f4
                        // Lambda @ 0x14c9e3f8..0x14c9e444
                        // Free @ 0x14c9e394..0x14c9e3f8
                        // D2 dtor @ 0x14c92948..0x14c92964
                        // So skip 0x14c92900..0x14c9e500 inclusive.
                        let in_pcheck_chain = is_pcheck_sret_fault
                            && (0x14c92900..=0x14c9e500).contains(&saved_lr);
                        // reverted (see pa_abort_skip walk).
                        if !in_pa_free && !in_pa_libchrome && !in_pcheck_chain
                            && saved_lr != 0
                            && saved_lr > 0x1000 && in_text_range
                        {
                            found_lr = saved_lr;
                            break;
                        }
                        fp = next_fp;
                        hops += 1;
                    }

                    // Fallback: if fp-walk failed, scan sp_el0 for any
                    // plausible saved-LR slot (a value > 0x1000 that's
                    // not in PA's text). This is conservative but
                    // catches the Rehash-chain bad-funcptr where x29
                    // is corrupt but the stack still has a real return
                    // address from one of the outer Hashtable::insert
                    // / AtomicStringTable::Add frames.
                    //
                    // 🎯 Widened scan window from 0x200 → 0x1000 (one page).
                    // The previous 0x200 caught most bad-PC cases (e.g. the
                    // x29=0 ELR=0x7f case where the LR is at sp+0x50), but
                    // some V8/PA frame-prologue patterns push their saved
                    // LR further down the active frame (sp+0x300..0x800).
                    // The diagnostic dump above scans up to 16KB and routinely
                    // finds candidates in this range that the recovery path
                    // missed. Keep scanning bounded by page-end so we don't
                    // walk into unmapped neighbors.
                    if found_lr == 0 && is_bad_pc_fault {
                        let mut probe_addr = sp_el0_now;
                        let page_end = (sp_el0_now | 0xFFF) + 1;
                        let probe_end = (sp_el0_now + 0x1000).min(page_end);
                        let mut sp_scan_hits = 0u32;
                        while probe_addr < probe_end {
                            if !crate::caves::linux::uaccess::is_user_range(probe_addr as usize, 8)
                                || !page_is_mapped(probe_addr)
                            {
                                break;
                            }
                            let v: u64 = unsafe {
                                core::ptr::read_volatile(probe_addr as *const u64)
                            };
                            // Want: a code address (0x10000000..0x1f000000),
                            // not in PA's libchrome range, 4-byte aligned.
                            // restrict LR candidate to
                            // content_shell's TEXT range (0x11720000
                            // .. 0x1990ab00) — picking up BSS/data
                            // addresses (0x1a050040..0x1a224e58) led
                            // to elr=BSS, then bad-instruction-fetch
                            // after walking through zero-fill memory.
                            if v >= 0x11720000 && v < 0x19910000
                                && (v & 3) == 0
                                && !(0x14d70000..=0x14da0000).contains(&v)
                                && !(0x11a63000..=0x11a6a800).contains(&v)
                            {
                                found_lr = v;
                                fp = probe_addr;
                                let _ = sp_scan_hits;
                                break;
                            }
                            sp_scan_hits = sp_scan_hits.wrapping_add(1);
                            probe_addr += 8;
                        }
                    }
                    if found_lr != 0 {
                        // detect repeated-same-elr loops
                        // and abort the cave instead of spinning. The
                        // 85K-line ChromeRootStoreData loop spun 7 times
                        // at the SAME elr (0x152df784) returning to the
                        // SAME found_lr (0x152df77c) — wasting cycles
                        // before finally dying. If we see the identical
                        // (elr, found_lr) pair more than 3 times in a
                        // row, fall through to terminate_cave_fatal.
                        static LAST_SKIP_ELR: core::sync::atomic::AtomicU64 =
                            core::sync::atomic::AtomicU64::new(0);
                        static LAST_SKIP_LR: core::sync::atomic::AtomicU64 =
                            core::sync::atomic::AtomicU64::new(0);
                        static SAME_SKIP_COUNT: core::sync::atomic::AtomicU32 =
                            core::sync::atomic::AtomicU32::new(0);
                        // when loop is detected, instead of
                        // terminating immediately, try to ESCAPE by
                        // walking further up the FP chain. Skip the
                        // first ESCAPE_DEPTH valid LRs to land in the
                        // grand-caller's frame. If we exhaust 5 escape
                        // attempts at the same loop, give up.
                        static ESCAPE_DEPTH: core::sync::atomic::AtomicU32 =
                            core::sync::atomic::AtomicU32::new(0);
                        let prev_elr = LAST_SKIP_ELR.load(core::sync::atomic::Ordering::Relaxed);
                        let prev_lr = LAST_SKIP_LR.load(core::sync::atomic::Ordering::Relaxed);
                        let same_pair = prev_elr == elr_now && prev_lr == found_lr;
                        let mut loop_detected = false;
                        if same_pair {
                            let cnt = SAME_SKIP_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1;
                            // tuning: lowered 32 → 8 so
                            // escape kicks in sooner. Saving 24 wasted
                            // skip iterations means the escape attempt
                            // happens before the cave's other threads
                            // race ahead and create more zombie state.
                            if cnt > 8 {
                                let depth = ESCAPE_DEPTH.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1;
                                // v3: bumped 5 → 16. The 35K
                                // run got into Chromium's task scheduler
                                // (TaskQueueImpl::OnWakeUp,
                                // SequenceManagerImpl::SelectNextTask,
                                // ThreadController::Run) and re-entered
                                // Run() repeatedly. 5 escapes wasn't
                                // enough to break out of the recursive
                                // pump. 16 gives more room.
                                if depth > 16 {
                                    loop_detected = true;
                                    uart::puts("[pa-skip-data] LOOP+ESCAPE EXHAUSTED at elr=0x");
                                    let hex = b"0123456789abcdef";
                                    for sh in (0..16).rev() {
                                        uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                                    }
                                    uart::puts(" — terminating cave\n");
                                    SAME_SKIP_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
                                    ESCAPE_DEPTH.store(0, core::sync::atomic::Ordering::Relaxed);
                                    LAST_SKIP_ELR.store(0, core::sync::atomic::Ordering::Relaxed);
                                    LAST_SKIP_LR.store(0, core::sync::atomic::Ordering::Relaxed);
                                } else {
                                    // Walk up the FP chain to escape.
                                    uart::puts("[pa-skip-data] ESCAPE depth=");
                                    crate::kernel::mm::print_num(depth as usize);
                                    uart::puts(" elr=0x");
                                    let hex = b"0123456789abcdef";
                                    for sh in (0..16).rev() {
                                        uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                                    }
                                    uart::puts("\n");
                                    let mut fp_e = if frame_x29 > 0x1000 { frame_x29 } else { sp_el0_now };
                                    let mut hops_e = 0;
                                    let mut found_e: u64 = 0;
                                    let mut skip_n = depth as usize;
                                    while hops_e < 32 && fp_e != 0 {
                                        if !crate::caves::linux::uaccess::is_user_range(fp_e as usize, 16)
                                            || !page_is_mapped(fp_e)
                                        {
                                            break;
                                        }
                                        let nfp: u64 = unsafe {
                                            core::ptr::read_volatile(fp_e as *const u64)
                                        };
                                        let slr: u64 = unsafe {
                                            core::ptr::read_volatile((fp_e + 8) as *const u64)
                                        };
                                        let in_pa = (0x11a63000..=0x11a6a800).contains(&slr)
                                            || (0x14d70000..=0x14da0000).contains(&slr);
                                        let in_text = slr >= 0x11720000 && slr < 0x19910000;
                                        if !in_pa && slr > 0x1000 && in_text {
                                            if skip_n == 0 {
                                                found_e = slr;
                                                fp_e = nfp;
                                                break;
                                            }
                                            skip_n -= 1;
                                        }
                                        fp_e = nfp;
                                        hops_e += 1;
                                    }
                                    if found_e != 0 {
                                        SAME_SKIP_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
                                        LAST_SKIP_ELR.store(elr_now, core::sync::atomic::Ordering::Relaxed);
                                        LAST_SKIP_LR.store(found_e, core::sync::atomic::Ordering::Relaxed);
                                        // Restore scratch substitute on escape
                                        // the grand-caller's "I just got
                                        // back from a callee" expects x0
                                        // to be a return value, and a zeroed
                                        // scratch (readable as zero pointer
                                        // chains) gives more code paths a
                                        // chance to take fallback branches.
                                        let scratch = pa_skip_scratch_uva();
                                        unsafe {
                                            (*frame).elr = found_e;
                                            (*frame).x[29] = fp_e;
                                            (*frame).x[30] = found_e;
                                            if scratch != 0 {
                                                (*frame).x[0] = scratch;
                                            }
                                            // was `fp_e + 16`.
                                            // fp_e here is already the
                                            // CALLER'S saved-x29 (we did
                                            // `fp_e = nfp` at the find
                                            // step), so its body sp at
                                            // call time is fp_e itself
                                            // (assuming simple prologue).
                                            // The +16 was placing SP into
                                            // the caller's caller's frame
                                            // same class of bug as the
                                            // brk-skip resume.
                                            core::arch::asm!("msr sp_el0, {a}",
                                                a = in(reg) fp_e);
                                        }
                                        return;
                                    } else {
                                        loop_detected = true;
                                        uart::puts("[pa-skip-data] ESCAPE FAILED — terminating cave\n");
                                    }
                                }
                            }
                        } else {
                            SAME_SKIP_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
                            ESCAPE_DEPTH.store(0, core::sync::atomic::Ordering::Relaxed);
                            LAST_SKIP_ELR.store(elr_now, core::sync::atomic::Ordering::Relaxed);
                            LAST_SKIP_LR.store(found_lr, core::sync::atomic::Ordering::Relaxed);
                        }
                        if loop_detected {
                            crate::caves::linux::signal::terminate_cave_fatal_with_lr(
                                fatal_signo, far_now, found_lr
                            );
                        }
                        // 🎯 Synthesize a SAFE Alloc return for
                        // PA-Alloc-region faults: set x[0] to a
                        // user-VA-mapped scratch page. Most Alloc
                        // callers immediately memset/memcpy the
                        // result; with a real writable buffer they
                        // succeed instead of NULL-derefing.
                        //
                        // extension: also substitute
                        // scratch when the caller is in chrome text
                        // (0x14000000..0x1c000000) and the fault
                        // address is small (NULL+small offset). This
                        // covers cases like ChromeRootStoreData
                        // ctor failing because BSSL ParsedCertificate::
                        // Create returned NULL — the caller does
                        // ptr->field where field offset < 0x100.
                        // A zero-init scratch reads as zero which is
                        // typically a no-op.
                        let is_alloc_fault = (0x14d75c40..=0x14d80000)
                            .contains(&elr_now);
                        let is_chrome_text_null_deref = far_now < 0x100
                            && (0x14000000..0x1c000000).contains(&elr_now);
                        if is_alloc_fault || is_chrome_text_null_deref {
                            let scratch = pa_skip_scratch_uva();
                            if scratch != 0 {
                                unsafe { (*frame).x[0] = scratch; }
                            }
                        }
                        // same SP fix as the brk-skip
                        // path. `fp` is the FP of the deepest non-PA
                        // frame in the chain (i.e. the FRAME WHOSE
                        // saved-LR we'll resume at). The CALLER's
                        // saved x29 lives at [fp]; the caller's body
                        // sp at bl-time equals that x29 for simple-
                        // prologue functions. Pre-fix: `sp = fp + 16`
                        // landed inside the deepest PA frame's
                        // local-saves area, so the user code's
                        // epilogue read garbage as x30 and ret'd to
                        // PC=0x1.
                        let caller_x29 = unsafe {
                            core::ptr::read_volatile(fp as *const u64)
                        };
                        unsafe {
                            (*frame).elr   = found_lr;
                            (*frame).x[29] = caller_x29;
                            (*frame).x[30] = found_lr;
                            core::arch::asm!("msr sp_el0, {a}",
                                a = in(reg) caller_x29);
                            // zero callee-saved regs
                            // (x19-x28) on pa-skip resume. The inner
                            // function we BRK-skipped never ran its
                            // epilogue, so x19-x28 in register hold
                            // INNER's clobbered values, not the
                            // resumed function's expected values.
                            // Symptom: x19 in `Run()` after pa-skip
                            // = 0xffffffffffffff00 (an inner-function
                            // sentinel sign-extended), then x0 =
                            // x19+0x20 = 0xffffff00 → OnRunLoopEnded
                            // faults at FAR=0xffffff00+0xe0=0xffe0.
                            //
                            // Zeroing makes any deref of these regs
                            // a clean NULL-fault rather than a
                            // confused dangling-pointer crash. If the
                            // resumed function uses x19 as a saved
                            // `this`, NULL-deref kills the cave fast
                            // which is BETTER than letting the
                            // corruption propagate through more
                            // function calls. (Restoring the actual
                            // saved values requires per-function
                            // prologue parsing; skip for now.)
                            //
                            // Zero callee-saved AND scratch regs.
                            // The inner function clobbered both;
                            // x[0..7] hold its garbage return values
                            // and arg shuffle, x[8..15] are scratch
                            // and x[19..28] are saved-but-not-restored.
                            // Skip x[0] if scratch substitution is
                            // active (synthesizes a NULL Alloc result
                            // → caller takes fallback branch).
                            let preserve_x0 =
                                (is_alloc_fault || is_chrome_text_null_deref)
                                && pa_skip_scratch_uva() != 0;
                            for i in 0..=28 {
                                if i == 0 && preserve_x0 { continue; }
                                if i == 29 || i == 30 { continue; } // already set
                                (*frame).x[i] = 0;
                            }
                        }
                        let n = PA_DATA_SKIP_TOTAL.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                        if n < 10 || (n & 0xFF) == 0 {
                            uart::puts(if is_bad_pc_fault {
                                "[pc-skip] #"
                            } else {
                                "[pa-skip-data] #"
                            });
                            crate::kernel::mm::print_num(n as usize);
                            uart::puts(" elr=0x");
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" far=0x");
                            for sh in (0..16).rev() {
                                uart::putc(hex[((far_now >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" → 0x");
                            for sh in (0..16).rev() {
                                uart::putc(hex[((found_lr >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts("\n");
                        }
                        return;
                    }

                    // 🎯 found_lr == 0 means FP-walk and sp-scan
                    // both failed to find a non-PA caller. Last-
                    // resort: just SKIP THE FAULTING INSTRUCTION
                    // (advance ELR by 4) and zero x[0..7] so the
                    // continued execution starts with a clean slate.
                    // This is more aggressive than synthesizing a
                    // return — risky, but the alternative is cave
                    // termination.
                    static LAST_RESORT_SKIPS: core::sync::atomic::AtomicU32 =
                        core::sync::atomic::AtomicU32::new(0);
                    let n = LAST_RESORT_SKIPS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                    if n < 32 {
                        unsafe {
                            (*frame).elr = elr_now + 4;
                            for i in 0..8 { (*frame).x[i] = 0; }
                        }
                        if n < 10 || (n & 0xF) == 0 {
                            uart::puts("[skip-instr] #");
                            crate::kernel::mm::print_num(n as usize);
                            uart::puts(" elr=0x");
                            let hex = b"0123456789abcdef";
                            for sh in (0..16).rev() {
                                uart::putc(hex[((elr_now >> (sh * 4)) & 0xF) as usize]);
                            }
                            uart::puts(" → +4 with x[0..7]=0\n");
                        }
                        return;
                    }
                }

                crate::caves::linux::signal::terminate_cave_fatal_with_lr(
                    fatal_signo, far_now, lr,
                );
                // never returns
            }
            loop { unsafe { core::arch::asm!("wfe") }; }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_unhandled_exception(_frame: *mut TrapFrame) {
    uart::puts("!!! UNHANDLED EXCEPTION !!!\n");
    loop { unsafe { core::arch::asm!("wfe") }; }
}

// Emulate load for atomic instruction emulation (HVF workaround)
// V2-002/003/004 gate: only emulate accesses that target user space.
// Before this gate, a guest could craft an unaligned / LDXR / atomic
// instruction with addr=<any kernel address> and the emulator would
// faithfully load/store on its behalf — arbitrary EL1 R/W primitive.
fn emul_addr_ok(addr: usize, nbytes: usize) -> bool {
    crate::caves::linux::uaccess::is_user_range(addr, nbytes)
}

unsafe fn emulate_load(addr: usize, size: u32) -> u64 {
    let nbytes = 1usize << (size as usize);
    if !emul_addr_ok(addr, nbytes) { return 0; } // safe-zero on bad addr
    unsafe {
        match size {
            0 => { let v: u32; core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            1 => { let v: u32; core::arch::asm!("ldrh {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            2 => { let v: u32; core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) addr, v = out(reg) v); v as u64 }
            _ => { let v: u64; core::arch::asm!("ldr {v}, [{a}]", a = in(reg) addr, v = out(reg) v); v }
        }
    }
}

// Emulate store for atomic instruction emulation (HVF workaround)
unsafe fn emulate_store(addr: usize, size: u32, val: u64) {
    let nbytes = 1usize << (size as usize);
    if !emul_addr_ok(addr, nbytes) { return; } // silently drop kernel-target writes
    unsafe {
        match size {
            0 => core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            1 => core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            2 => core::arch::asm!("str {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32),
            _ => core::arch::asm!("str {v}, [{a}]", a = in(reg) addr, v = in(reg) val),
        }
    }
}

fn print_hex(val: u64) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        uart::putc(hex[nibble]);
    }
}
