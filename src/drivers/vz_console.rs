#![allow(dead_code)]
// Sphragis — Virtualization.framework Console Output
// VZ VMs use virtio-console over PCI, but we don't know where.
//
// Strategy: VZLinuxBootLoader passes a device tree (DTB) in x0.
// The DTB contains the actual hardware layout including PCI ranges.
// We parse the DTB to find where virtio devices are.
//
// But simpler: VZ VMs also support "virtio-console" as an MMIO
// transport when using VZVirtioConsoleDeviceSerialPortConfiguration.
// The MMIO region starts at 0x10000 in some VZ configurations.
//
// Simplest approach: try writing to the HVC (Hypervisor Console)
// via the ARM semihosting or HVC call mechanism.
// VZLinuxBootLoader with console=hvc0 means the VM expects
// output via virtio-console, which is mapped as an MMIO device.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static VZ_ACTIVE: AtomicBool = AtomicBool::new(false);
static VZ_CONSOLE_BASE: AtomicUsize = AtomicUsize::new(0);

/// Try to detect and initialize VZ VM console.
/// Returns true if we found a working output method.
pub fn init() -> bool {
    // Method 1: Try the DTB passed by VZLinuxBootLoader
    // VZLinuxBootLoader stores DTB pointer, we can parse it to find MMIO ranges

    // Method 2: Known VZ MMIO ranges
    // Apple's Virtualization.framework maps virtio MMIO at predictable addresses
    // Try common locations
    let candidates: [usize; 8] = [
        0x0FFF0000,  // Some VZ configs
        0x10000000,  // Common ARM virt MMIO base
        0x0A000000,  // Same as QEMU (unlikely but try)
        0x20000000,
        0x30000000,
        0x40000000,  // Possible ECAM
        0x0E000000,
        0x0C000000,
    ];

    for &base in &candidates {
        // Check for virtio magic (0x74726976 = "virt")
        let magic = unsafe { core::ptr::read_volatile(base as *const u32) };
        if magic == 0x74726976 {
            let device_id = unsafe { core::ptr::read_volatile((base + 8) as *const u32) };
            if device_id != 0 {
                VZ_CONSOLE_BASE.store(base, Ordering::Relaxed);
                VZ_ACTIVE.store(true, Ordering::Relaxed);
                return true;
            }
        }
    }

    // Method 3: Use ARM semihosting for output
    // This works when the hypervisor supports it
    // SYS_WRITEC = 0x03
    if try_semihosting() {
        VZ_ACTIVE.store(true, Ordering::Relaxed);
        return true;
    }

    false
}

/// Try ARM semihosting output (write a test char)
fn try_semihosting() -> bool {
    // ARM semihosting: HLT #0xF000 with r0=operation, r1=params
    // Operation 0x03 = SYS_WRITEC (write character)
    // This works if the hypervisor implements semihosting
    //
    // Note: semihosting may not be available in all VZ configs
    // We return false for now and rely on other methods
    false
}

pub fn putc(c: u8) {
    if !VZ_ACTIVE.load(Ordering::Relaxed) { return; }

    let base = VZ_CONSOLE_BASE.load(Ordering::Relaxed);
    if base != 0 {
        // Write to virtio MMIO transmit
        unsafe {
            core::ptr::write_volatile(base as *mut u8, c);
        }
    }
}

pub fn puts(s: &str) {
    for b in s.bytes() {
        if b == b'\n' { putc(b'\r'); }
        putc(b);
    }
}

pub fn is_active() -> bool {
    VZ_ACTIVE.load(Ordering::Relaxed)
}
