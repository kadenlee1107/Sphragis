// Bat_OS — Apple Silicon SoC Definitions
// M4 (T8132) memory map and hardware constants.
// Addresses derived from Asahi Linux device tree research.
//
// Apple Silicon uses a flat physical address space with MMIO regions
// mapped at fixed addresses per SoC generation.

/// SoC identification
pub const SOC_NAME: &str = "Apple M4 (T8132)";
pub const SOC_CHIP_ID: u32 = 0x8132;

// ─── MMIO Base Addresses (M4 / T8132) ───
// These are physical addresses for hardware peripherals.
// Derived from Asahi Linux device trees and m1n1 probing.

/// Apple UART (Samsung-derived S5L UART)
/// M4 has multiple UART blocks. UART0 is the debug/serial port.
pub const UART0_BASE: usize = 0x2_3520_0000;
pub const UART0_SIZE: usize = 0x4000;

/// Apple Interrupt Controller (AIC2 on M4)
/// Handles all hardware interrupts.
pub const AIC_BASE: usize = 0x2_8E10_0000;
pub const AIC_SIZE: usize = 0x10000;

/// Apple Display Controller (DCP)
/// Manages the display pipeline via mailbox messages.
pub const DCP_BASE: usize = 0x2_2810_0000;
pub const DCP_DART: usize = 0x2_2810_8000; // DCP IOMMU (DART)

/// Apple NVMe Controller (ANS)
/// Custom NVMe controller for the internal SSD.
pub const ANS_BASE: usize = 0x2_7BCC_0000;
pub const ANS_SIZE: usize = 0x40000;

/// SPI Controller (for keyboard/trackpad on MacBook)
pub const SPI0_BASE: usize = 0x2_3510_0000;
pub const SPI0_SIZE: usize = 0x4000;

/// Apple Mailbox (used for DCP, ANS, SEP communication)
pub const MBOX_BASE: usize = 0x2_2BC0_0000;

/// Secure Enclave Processor (SEP)
pub const SEP_BASE: usize = 0x2_4100_0000;

/// DART (Device Address Resolution Table — Apple's IOMMU)
/// Each device has its own DART for DMA isolation.
pub const DART_USB_BASE: usize = 0x2_3920_0000;
pub const DART_ANS_BASE: usize = 0x2_7BC8_0000;

/// Timer — Apple Silicon uses ARM Generic Timer
/// (same as QEMU, but frequency may differ)
pub const TIMER_FREQ_HZ: u64 = 24_000_000; // 24 MHz on Apple Silicon

/// RAM layout (M4 MacBook — varies by config)
/// Actual values come from m1n1/device tree at boot.
pub const RAM_BASE: usize = 0x8_0000_0000;  // Unified memory starts here
pub const RAM_DEFAULT_SIZE: usize = 16 * 1024 * 1024 * 1024; // 16GB default

/// Framebuffer — m1n1 sets up a simple framebuffer we can use initially
/// The actual address is passed via boot args from m1n1.
pub const EARLY_FB_DEFAULT: usize = 0x9_E000_0000; // Typical m1n1 FB location

/// Boot arguments structure from m1n1
#[repr(C)]
pub struct M1n1BootArgs {
    pub revision: u16,
    pub version: u16,
    pub virt_base: u64,
    pub phys_base: u64,
    pub mem_size: u64,
    pub top_of_kernel: u64,
    pub video_base: u64,
    pub video_row_bytes: u64,
    pub video_width: u64,
    pub video_height: u64,
    pub video_depth: u64,
    pub machine_type: u32,
    pub devtree_addr: u64,
    pub devtree_size: u32,
    pub cmdline: [u8; 256],
}

/// Platform info — populated at boot from device tree / m1n1 args
pub struct PlatformInfo {
    pub chip_id: u32,
    pub ram_base: usize,
    pub ram_size: usize,
    pub fb_base: usize,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
}

use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering};

static FB_BASE: AtomicUsize = AtomicUsize::new(0);
static FB_WIDTH: AtomicU32 = AtomicU32::new(0);
static FB_HEIGHT: AtomicU32 = AtomicU32::new(0);
static FB_STRIDE: AtomicU32 = AtomicU32::new(0);
static MEM_BASE: AtomicUsize = AtomicUsize::new(RAM_BASE);
static MEM_SIZE: AtomicUsize = AtomicUsize::new(0);

pub fn init_from_boot_args(args: &M1n1BootArgs) {
    FB_BASE.store(args.video_base as usize, Ordering::Relaxed);
    FB_WIDTH.store(args.video_width as u32, Ordering::Relaxed);
    FB_HEIGHT.store(args.video_height as u32, Ordering::Relaxed);
    FB_STRIDE.store(args.video_row_bytes as u32, Ordering::Relaxed);
    MEM_BASE.store(args.phys_base as usize, Ordering::Relaxed);
    MEM_SIZE.store(args.mem_size as usize, Ordering::Relaxed);
}

pub fn fb_base() -> usize { FB_BASE.load(Ordering::Relaxed) }
pub fn fb_width() -> u32 { FB_WIDTH.load(Ordering::Relaxed) }
pub fn fb_height() -> u32 { FB_HEIGHT.load(Ordering::Relaxed) }
pub fn fb_stride() -> u32 { FB_STRIDE.load(Ordering::Relaxed) }
pub fn mem_base() -> usize { MEM_BASE.load(Ordering::Relaxed) }
pub fn mem_size() -> usize { MEM_SIZE.load(Ordering::Relaxed) }
