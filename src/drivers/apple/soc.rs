#![allow(dead_code)]
// Bat_OS — Apple Silicon SoC Definitions
// M4 (T8132) memory map and hardware constants.
// Addresses derived from Asahi Linux device tree research.
//
// Apple Silicon uses a flat physical address space with MMIO regions
// mapped at fixed addresses per SoC generation.

use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering};

/// SoC identification
pub const SOC_NAME: &str = "Apple M4 (T8132)";
pub const SOC_CHIP_ID: u32 = 0x8132;

// ─── MMIO Base Addresses (M4 / T8132) ───
// These are physical addresses for hardware peripherals.
// Derived from Asahi Linux device trees and m1n1 probing.

// V-ASAHI-1.3: runtime-resolved MMIO addresses.
//
// The previous hardcoded addresses here were all M1 (T8103) values
// (derived from Asahi's T8103 memmap doc). They're WRONG on M4
// (T8132) — e.g. UART is at 0x3ad200000 on M4, not 0x235200000.
//
// Apple moves peripherals around between SoC generations; there is
// NO reliable static map. The only correct source is the ADT that
// m1n1 hands us at boot — `discover_from_adt` walks it and populates
// the atomics below. Drivers call the accessor functions instead of
// reading a const.
//
// We still declare FALLBACK constants to avoid dead-code warnings and
// to provide something for QEMU-virt testing (where there is no ADT
// and hardcoded values are used by the generic drivers). On Apple
// Silicon hardware, the fallbacks are never used — if ADT discovery
// fails, `kernel_main_apple` halts instead of limping along with M1
// addresses on M4 hardware.
// V-GROUND-TRUTH: M4 (Mac16,1) actually uses a DOCKCHANNEL UART at
// 0x3_8812_8000, not the classical UART0 at 0x3_ad20_0000. m1n1's
// own EARLY_UART_BASE points at 0x3_ad20_0000 for T8132 but that
// register region is a *different* UART that is not wired out on
// Mac16,1 — writes land nowhere visible. Use dockchannel for the
// actual serial-over-USB proxy path. See docs/M4_GROUND_TRUTH.md §3.1.
const UART0_BASE_FALLBACK: usize = 0x3_8812_8000;
const AIC_BASE_FALLBACK: usize   = 0x2_8e10_0000;
const DCP_BASE_FALLBACK: usize   = 0x2_2810_0000;
const DCP_DART_FALLBACK: usize   = 0x2_2810_8000;
const ANS_BASE_FALLBACK: usize   = 0x2_7bcc_0000;
const SPI0_BASE_FALLBACK: usize  = 0x2_3510_0000;
const MBOX_BASE_FALLBACK: usize  = 0x2_2bc0_0000;
const SEP_BASE_FALLBACK: usize   = 0x2_4100_0000;
const DART_USB_FALLBACK: usize   = 0x2_3920_0000;
const DART_ANS_FALLBACK: usize   = 0x2_7bc8_0000;

pub const UART0_SIZE: usize = 0x4000;
pub const AIC_SIZE:   usize = 0x10000;
pub const ANS_SIZE:   usize = 0x40000;
pub const SPI0_SIZE:  usize = 0x4000;

static UART0_BASE_RT: AtomicUsize = AtomicUsize::new(0);
static AIC_BASE_RT:   AtomicUsize = AtomicUsize::new(0);
static DCP_BASE_RT:   AtomicUsize = AtomicUsize::new(0);
static DCP_DART_RT:   AtomicUsize = AtomicUsize::new(0);
static ANS_BASE_RT:   AtomicUsize = AtomicUsize::new(0);
static SPI0_BASE_RT:  AtomicUsize = AtomicUsize::new(0);
static MBOX_BASE_RT:  AtomicUsize = AtomicUsize::new(0);
static SEP_BASE_RT:   AtomicUsize = AtomicUsize::new(0);
static DART_USB_RT:   AtomicUsize = AtomicUsize::new(0);
static DART_ANS_RT:   AtomicUsize = AtomicUsize::new(0);

/// Pick the runtime value if set, otherwise the hardcoded fallback.
#[inline]
fn rt_or(rt: &AtomicUsize, fb: usize) -> usize {
    let v = rt.load(Ordering::Acquire);
    if v != 0 { v } else { fb }
}

pub fn uart0_base() -> usize { rt_or(&UART0_BASE_RT, UART0_BASE_FALLBACK) }
pub fn aic_base()   -> usize { rt_or(&AIC_BASE_RT,   AIC_BASE_FALLBACK) }
pub fn dcp_base()   -> usize { rt_or(&DCP_BASE_RT,   DCP_BASE_FALLBACK) }
pub fn dcp_dart()   -> usize { rt_or(&DCP_DART_RT,   DCP_DART_FALLBACK) }
pub fn ans_base()   -> usize { rt_or(&ANS_BASE_RT,   ANS_BASE_FALLBACK) }
pub fn spi0_base()  -> usize { rt_or(&SPI0_BASE_RT,  SPI0_BASE_FALLBACK) }
pub fn mbox_base()  -> usize { rt_or(&MBOX_BASE_RT,  MBOX_BASE_FALLBACK) }
pub fn sep_base()   -> usize { rt_or(&SEP_BASE_RT,   SEP_BASE_FALLBACK) }
pub fn dart_usb()   -> usize { rt_or(&DART_USB_RT,   DART_USB_FALLBACK) }
pub fn dart_ans()   -> usize { rt_or(&DART_ANS_RT,   DART_ANS_FALLBACK) }

/// Resolve a peripheral's `reg[0]` through the ADT, translating
/// through every parent's `ranges` property. Returns `None` if the
/// node doesn't exist in the ADT (e.g. the machine doesn't have that
/// peripheral at all, which happens: Mac mini has no SPI keyboard).
fn lookup_reg0(adt: &super::adt::Adt, path: &str) -> Option<usize> {
    // Walk and remember each node so reg_absolute can translate.
    let segs = path.split('/').filter(|s| !s.is_empty());
    let mut trail: [Option<super::adt::Node>; 8] = [None; 8];
    let root = adt.root().ok()?;
    trail[0] = Some(root);
    let mut depth = 1;
    let mut cursor = root;
    for seg in segs {
        if depth >= trail.len() { return None; }
        cursor = cursor.subnode(seg).ok()?;
        trail[depth] = Some(cursor);
        depth += 1;
    }
    // Compact trail[..depth] into a contiguous slice of `Node` for
    // reg_absolute (which wants a root-to-here path).
    let mut path_nodes: [super::adt::Node; 8] = [root; 8];
    for i in 0..depth { path_nodes[i] = trail[i]?; }
    let (addr, _size) = cursor.reg_absolute(&path_nodes[..depth], 0).ok()?;
    Some(addr as usize)
}

/// V-ASAHI-1.3: walk the ADT and populate every runtime MMIO atomic.
/// Any peripheral whose path doesn't exist on this machine silently
/// stays at 0 (which resolves to the fallback const). We return the
/// count of successfully resolved peripherals so the caller can log
/// "resolved N/10 peripherals from ADT" for sanity.
pub fn discover_from_adt(adt: &super::adt::Adt) -> usize {
    let mut n = 0;
    let table: &[(&str, &AtomicUsize)] = &[
        ("/arm-io/uart0",   &UART0_BASE_RT),
        ("/arm-io/aic",     &AIC_BASE_RT),
        ("/arm-io/disp0",   &DCP_BASE_RT),    // M4 uses disp0 naming
        ("/arm-io/dart-disp0", &DCP_DART_RT),
        ("/arm-io/ans",     &ANS_BASE_RT),
        ("/arm-io/spi0",    &SPI0_BASE_RT),
        ("/arm-io/sep",     &SEP_BASE_RT),
        ("/arm-io/dart-usb", &DART_USB_RT),
        ("/arm-io/dart-ans", &DART_ANS_RT),
    ];
    for (path, atomic) in table {
        if let Some(addr) = lookup_reg0(adt, path) {
            atomic.store(addr, Ordering::Release);
            n += 1;
        }
    }
    n
}

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

/// V-ASAHI-1: new accessors populated from the validated
/// `boot_args::BootArgs` view. These supersede `init_from_boot_args`
/// which reads the legacy struct shape directly (and got the field
/// order wrong pre-revision-3).
pub fn set_fb_info(base: usize, width: u32, height: u32, stride: u32) {
    FB_BASE.store(base, Ordering::Release);
    FB_WIDTH.store(width, Ordering::Release);
    FB_HEIGHT.store(height, Ordering::Release);
    FB_STRIDE.store(stride, Ordering::Release);
}
pub fn set_mem_info(base: usize, size: usize) {
    MEM_BASE.store(base, Ordering::Release);
    MEM_SIZE.store(size, Ordering::Release);
}

pub fn fb_base() -> usize { FB_BASE.load(Ordering::Relaxed) }
pub fn fb_width() -> u32 { FB_WIDTH.load(Ordering::Relaxed) }
pub fn fb_height() -> u32 { FB_HEIGHT.load(Ordering::Relaxed) }
pub fn fb_stride() -> u32 { FB_STRIDE.load(Ordering::Relaxed) }
pub fn mem_base() -> usize { MEM_BASE.load(Ordering::Relaxed) }
pub fn mem_size() -> usize { MEM_SIZE.load(Ordering::Relaxed) }
