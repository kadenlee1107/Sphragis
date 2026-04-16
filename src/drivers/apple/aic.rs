#![allow(dead_code)]
// Bat_OS — Apple Interrupt Controller (AIC2)
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
    unsafe { core::ptr::read_volatile((soc::AIC_BASE + offset) as *const u32) }
}

fn write32(offset: usize, val: u32) {
    unsafe { core::ptr::write_volatile((soc::AIC_BASE + offset) as *mut u32, val) }
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
