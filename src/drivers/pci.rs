// Bat_OS — PCI Bus Enumeration
// Virtualization.framework VMs use PCI for virtio devices.
// This probes the PCI config space to find virtio console/GPU.
//
// ARM64 VMs typically map PCI ECAM at 0x40000000
// and PCI MMIO windows at 0x50000000+

use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

// PCI ECAM base — standard for ARM64 VMs
const PCI_ECAM_BASE: usize = 0x4000_0000;

// Virtio PCI vendor/device IDs
const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_CONSOLE_DEV: u16 = 0x1043; // virtio console (modern)
const VIRTIO_GPU_DEV: u16 = 0x1050;     // virtio GPU (modern)
const VIRTIO_NET_DEV: u16 = 0x1041;     // virtio network (modern)

static CONSOLE_BAR: AtomicUsize = AtomicUsize::new(0);
static CONSOLE_FOUND: AtomicBool = AtomicBool::new(false);

/// ECAM config space address for a PCI device
fn ecam_addr(bus: u8, device: u8, function: u8, offset: u16) -> usize {
    PCI_ECAM_BASE +
        ((bus as usize) << 20) |
        ((device as usize) << 15) |
        ((function as usize) << 12) |
        (offset as usize)
}

fn pci_read32(bus: u8, dev: u8, func: u8, offset: u16) -> u32 {
    let addr = ecam_addr(bus, dev, func, offset);
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

fn pci_write32(bus: u8, dev: u8, func: u8, offset: u16, val: u32) {
    let addr = ecam_addr(bus, dev, func, offset);
    unsafe { core::ptr::write_volatile(addr as *mut u32, val); }
}

/// Probe the PCI bus for virtio devices.
pub fn probe() -> bool {
    let mut found_any = false;

    // Scan bus 0, devices 0-31
    for dev in 0..32u8 {
        let id = pci_read32(0, dev, 0, 0);
        if id == 0xFFFFFFFF || id == 0 {
            continue;
        }

        let vendor = (id & 0xFFFF) as u16;
        let device = ((id >> 16) & 0xFFFF) as u16;

        if vendor == VIRTIO_VENDOR {
            found_any = true;

            // Read BAR0 for MMIO base
            let bar0 = pci_read32(0, dev, 0, 0x10);
            let bar_addr = (bar0 & 0xFFFFFFF0) as usize;

            // Enable bus mastering + memory space
            let cmd = pci_read32(0, dev, 0, 0x04);
            pci_write32(0, dev, 0, 0x04, cmd | 0x06);

            match device {
                d if d == VIRTIO_CONSOLE_DEV || d == 0x1003 => {
                    // Found virtio console
                    CONSOLE_BAR.store(bar_addr, Ordering::Relaxed);
                    CONSOLE_FOUND.store(true, Ordering::Relaxed);
                }
                _ => {}
            }
        }
    }

    found_any
}

/// Write a byte to the PCI virtio console (if found).
pub fn console_putc(c: u8) {
    if !CONSOLE_FOUND.load(Ordering::Relaxed) { return; }
    let bar = CONSOLE_BAR.load(Ordering::Relaxed);
    if bar == 0 { return; }

    // Virtio modern: write to the transmit virtqueue
    // For a simple "just output a byte" we use the emerg/debug port
    // offset 0 in the device-specific config area
    unsafe {
        core::ptr::write_volatile(bar as *mut u8, c);
    }
}

pub fn is_console_available() -> bool {
    CONSOLE_FOUND.load(Ordering::Relaxed)
}
