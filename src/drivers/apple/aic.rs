#![allow(dead_code)]
// Sphragis — Apple Interrupt Controller (AIC2)
// Custom interrupt controller used on all Apple Silicon.
// NOT a standard GIC — Apple uses their own design.
// Reference: Asahi Linux drivers/irqchip/irq-apple-aic.c

use super::soc;

// AIC2 Register Offsets (M4)
const AIC_INFO: usize = 0x0004;
const AIC_CONFIG: usize = 0x0010;
const AIC_EVENT: usize = 0x2004;      // Event register — read to get pending IRQ
const AIC_IPI_SET: usize = 0x2008;    // Trigger IPI
const AIC_IPI_CLR: usize = 0x200C;    // Clear IPI
const AIC_IPI_MASK_SET: usize = 0x2024;
const AIC_IPI_MASK_CLR: usize = 0x2028;
const AIC_TARGET_CPU: usize = 0x3000; // IRQ target CPU (per-IRQ)
const AIC_SW_SET: usize = 0x4000;     // Software trigger
const AIC_SW_CLR: usize = 0x4080;     // Software clear
const AIC_MASK_SET: usize = 0x4100;   // Mask (disable) IRQ
const AIC_MASK_CLR: usize = 0x4180;   // Unmask (enable) IRQ

// Event types from AIC_EVENT
const AIC_EVENT_TYPE_MASK: u32 = 0xFFFF_0000;
const AIC_EVENT_NUM_MASK: u32 = 0x0000_FFFF;
const AIC_EVENT_IRQ: u32 = 1 << 16;
const AIC_EVENT_IPI: u32 = 4 << 16;
const AIC_EVENT_NONE: u32 = 0;

fn read32(offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((soc::aic_base() + offset) as *const u32) }
}

fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::aic_base() + offset) as *mut u32, val) }
}

/// Initialize the AIC.
pub fn init() {
    // Read AIC info to determine number of IRQs
    let info = read32(AIC_INFO);
    let num_irqs = info & 0xFFFF;

    // Mask all IRQs initially (DEFAULT DENY — like our firewall)
    for i in 0..(num_irqs + 31) / 32 {
        write32(AIC_MASK_SET + (i as usize) * 4, 0xFFFF_FFFF);
    }

    // Unmask IPIs
    write32(AIC_IPI_MASK_CLR, 0x1);
}

/// Read the next pending event from AIC.
pub fn read_event() -> (u32, u32) {
    let event = read32(AIC_EVENT);
    let event_type = event & AIC_EVENT_TYPE_MASK;
    let event_num = event & AIC_EVENT_NUM_MASK;
    (event_type, event_num)
}

/// Acknowledge and complete an IRQ.
pub fn ack_irq(irq: u32) {
    let reg = (irq / 32) as usize;
    let bit = irq % 32;
    write32(AIC_SW_CLR + reg * 4, 1 << bit);
}

/// Enable a specific IRQ.
pub fn enable_irq(irq: u32) {
    let reg = (irq / 32) as usize;
    let bit = irq % 32;
    write32(AIC_MASK_CLR + reg * 4, 1 << bit);
}

/// Disable a specific IRQ.
pub fn disable_irq(irq: u32) {
    let reg = (irq / 32) as usize;
    let bit = irq % 32;
    write32(AIC_MASK_SET + reg * 4, 1 << bit);
}

/// Send an IPI (inter-processor interrupt) to another core.
pub fn send_ipi(cpu: u32) {
    write32(AIC_IPI_SET, 1 << cpu);
}

/// Clear IPI.
pub fn clear_ipi() {
    write32(AIC_IPI_CLR, 0x1);
}

// ─── V-ASAHI-2.2: IRQ handler dispatch ──────────────────────────────
//
// AIC delivers interrupts as 32-bit "events" of the form
// `(event_type << 16) | irq_num`. Drivers register a Rust function
// to handle a specific IRQ; when that IRQ fires, our handle_irq()
// looks up the function pointer and calls it. The handler runs with
// IRQs masked (already true at exception entry); it must not block
// or yield.
//
// Storage: a fixed-size table indexed by irq number. Apple SoCs have
// up to ~1024 IRQs; we cap at 1024 here and reject registrations
// beyond that. Each slot holds an AtomicPtr so register/unregister
// are race-free without a lock.
//
// IPI handling uses a separate slot since IPIs and IRQs are dispatched
// from the same AIC_EVENT register but have different event-type tags.

use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

pub type IrqHandler = fn(irq: u32);

const MAX_IRQS: usize = 1024;

/// Per-IRQ Rust handler. Storing a `*mut ()` because we need
/// AtomicPtr; we transmute back to the function pointer at dispatch.
static HANDLERS: [AtomicPtr<()>; MAX_IRQS] = {
    const INIT: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
    [INIT; MAX_IRQS]
};

/// Diagnostic: count of IRQs dispatched, indexed by event-type
/// nybble. Useful when debugging "is the timer firing at all?".
static IRQ_COUNT: AtomicU64 = AtomicU64::new(0);
static SPURIOUS_COUNT: AtomicU64 = AtomicU64::new(0);

/// Register a handler for a specific IRQ. Replacing an existing
/// handler is allowed (returns false if the IRQ is out of range).
/// The IRQ remains masked until you call [`enable_irq`].
pub fn register(irq: u32, handler: IrqHandler) -> bool {
    if (irq as usize) >= MAX_IRQS { return false; }
    HANDLERS[irq as usize].store(handler as *mut (), Ordering::Release);
    true
}

/// Remove a handler. The IRQ should also be disabled separately
/// via [`disable_irq`] to stop further deliveries.
pub fn unregister(irq: u32) {
    if (irq as usize) >= MAX_IRQS { return; }
    HANDLERS[irq as usize].store(core::ptr::null_mut(), Ordering::Release);
}

/// Pull one IRQ off the AIC, look up its handler, dispatch, and ack.
/// Returns `true` if work was performed (so the caller can loop until
/// the queue is empty), `false` if AIC reported "nothing pending".
///
/// This is the entry point called from the kernel's exception handler
/// when an IRQ exception is taken on Apple Silicon.
///
/// M4 note: the stat counters (`IRQ_COUNT` / `SPURIOUS_COUNT`) used
/// to be updated with `fetch_add`, but under the MMU-off bring-up
/// regime every region is Device-nGnRnE and LDXR/STXR never succeeds
/// — so a `fetch_add` would spin forever on the very first IRQ and
/// wedge the kernel. We're running single-CPU with IRQ handlers
/// effectively-serialized by the exception, so a plain load +
/// wrapping_add + store is exclusive. Revisit once MMU is on and
/// SMP lands. See `docs/M4_GROUND_TRUTH.md §2`.
pub fn dispatch_one() -> bool {
    let raw = read32(AIC_EVENT);
    let event_type = raw & AIC_EVENT_TYPE_MASK;
    let event_num = raw & AIC_EVENT_NUM_MASK;

    if event_type == AIC_EVENT_NONE {
        return false;
    }

    let irq_n = IRQ_COUNT.load(Ordering::Relaxed);
    IRQ_COUNT.store(irq_n.wrapping_add(1), Ordering::Relaxed);

    if event_type == AIC_EVENT_IRQ {
        if (event_num as usize) < MAX_IRQS {
            let raw_ptr = HANDLERS[event_num as usize].load(Ordering::Acquire);
            if !raw_ptr.is_null() {
                // SAFETY: We only ever store `IrqHandler` (a `fn(u32)`)
                // function pointers in HANDLERS via `register()`.
                // Transmuting back to that exact type is sound.
                let handler: IrqHandler = unsafe {
                    core::mem::transmute(raw_ptr)
                };
                handler(event_num);
            } else {
                bump_spurious();
            }
            ack_irq(event_num);
        } else {
            bump_spurious();
        }
        return true;
    }

    if event_type == AIC_EVENT_IPI {
        clear_ipi();
        // No per-CPU IPI handler yet — UP system. Just consume.
        return true;
    }

    // Unknown event type — count as spurious + drain.
    bump_spurious();
    true
}

#[inline(always)]
fn bump_spurious() {
    let n = SPURIOUS_COUNT.load(Ordering::Relaxed);
    SPURIOUS_COUNT.store(n.wrapping_add(1), Ordering::Relaxed);
}

/// Diagnostic counters (for "is the IRQ wire alive?" debugging).
pub fn stats() -> (u64, u64) {
    (IRQ_COUNT.load(Ordering::Relaxed), SPURIOUS_COUNT.load(Ordering::Relaxed))
}
