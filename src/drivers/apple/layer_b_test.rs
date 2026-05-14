#![allow(dead_code)]
// Sphragis — Layer-B synthetic-args test harness
//
// Purpose: exercise the Apple boot-path parser code (boot_args::parse,
// adt::Adt, soc::discover_from_adt) on real M4 silicon via QEMU+HVF,
// WITHOUT needing m1n1 installed on the M4's disk.
//
// Approach: build a synthetic `BootArgsRaw` pointing at a synthetic
// minimal ADT, both in static kernel memory, then invoke the parser
// chain exactly as `kernel_main_apple` would on a real m1n1 chainload.
// All output goes via the QEMU UART (which works under HVF).
//
// Activation: cargo build --features layer-b-test
// Gated entirely behind the `layer-b-test` feature so production
// builds don't carry this code.

use super::{adt, boot_args, soc};
use crate::drivers::uart;

// ─── Synthetic ADT construction ─────────────────────────────────────
//
// We hand-build a small but parser-valid ADT:
//
//   root:
//     name = "device-tree"
//     #address-cells = 2
//     #size-cells = 2
//     arm-io:
//       name = "arm-io"
//       #address-cells = 2
//       #size-cells = 2
//       uart0:
//         name = "uart0"
//         reg = (addr=0x3ad200000, size=0x4000)
//
// Layout matches the binary format documented in m1n1/rust/src/adt.rs:
//   AdtNodeHdr { property_count: u32, child_count: u32 } (8 bytes)
//     AdtPropHdr { name: [u8;32], size: u32 }  (36 bytes, value follows, padded to 4)

const ADT_BUF_SIZE: usize = 4096;

static mut SYNTHETIC_ADT: [u8; ADT_BUF_SIZE] = [0; ADT_BUF_SIZE];

/// Write `val` as little-endian u32 into `buf[off..off+4]`, advancing off.
#[inline]
fn put_u32(buf: &mut [u8], off: &mut usize, val: u32) {
    buf[*off..*off + 4].copy_from_slice(&val.to_le_bytes());
    *off += 4;
}

/// Write a fixed 32-byte name field (nul-padded).
#[inline]
fn put_name(buf: &mut [u8], off: &mut usize, name: &[u8]) {
    let n = name.len().min(31);
    buf[*off..*off + n].copy_from_slice(&name[..n]);
    for b in &mut buf[*off + n..*off + 32] { *b = 0; }
    *off += 32;
}

#[inline]
fn align_up(n: usize) -> usize { (n + 3) & !3 }

/// Write one property (name + size + value, padded). Returns new offset.
fn put_prop(buf: &mut [u8], off: &mut usize, name: &[u8], value: &[u8]) {
    put_name(buf, off, name);
    put_u32(buf, off, value.len() as u32);
    buf[*off..*off + value.len()].copy_from_slice(value);
    *off += value.len();
    // Pad value to 4-byte alignment for the next property
    while *off % 4 != 0 {
        buf[*off] = 0;
        *off += 1;
    }
}

/// Helper: emit a child node `name` with a `reg` property pointing at
/// `(addr, size)` (2-cell each, MMIO at `addr` of `size` bytes). Also
/// adds a `compatible` property so we can exercise the string-list iter.
unsafe fn put_periph(
    buf: &mut [u8],
    off: &mut usize,
    name: &[u8],
    addr: u64,
    size: u64,
    compatible: &[u8],
) {
    // 3 properties (name, compatible, reg), 0 children.
    put_u32(buf, off, 3);
    put_u32(buf, off, 0);
    put_prop(buf, off, b"name", name);
    put_prop(buf, off, b"compatible", compatible);
    let mut reg = [0u8; 16];
    reg[0..4].copy_from_slice(&(addr as u32).to_le_bytes());
    reg[4..8].copy_from_slice(&((addr >> 32) as u32).to_le_bytes());
    reg[8..12].copy_from_slice(&(size as u32).to_le_bytes());
    reg[12..16].copy_from_slice(&((size >> 32) as u32).to_le_bytes());
    put_prop(buf, off, b"reg", &reg);
}

/// Build the synthetic ADT into `SYNTHETIC_ADT`. Returns byte count used.
///
/// LAYER C: structurally-realistic ADT mirroring what we expect to find
/// on real M4 silicon. Includes all 9 peripherals our `discover_from_adt`
/// looks up, at their real M4 addresses (per m1n1 source + Asahi docs).
/// Using 1:1 identity `ranges` on /arm-io so the addresses map straight
/// through (matches Asahi's layout for most arm-io children).
unsafe fn build_synthetic_adt() -> usize {
    let buf = &mut *core::ptr::addr_of_mut!(SYNTHETIC_ADT);
    let mut off = 0usize;

    // Root: 3 properties, 1 child (arm-io).
    put_u32(buf, &mut off, 3);
    put_u32(buf, &mut off, 1);
    put_prop(buf, &mut off, b"name", b"device-tree\0");
    put_prop(buf, &mut off, b"#address-cells", &2u32.to_le_bytes());
    put_prop(buf, &mut off, b"#size-cells",    &2u32.to_le_bytes());

    // arm-io: 4 properties, 9 children (one per discoverable peripheral).
    put_u32(buf, &mut off, 4);
    put_u32(buf, &mut off, 9);
    put_prop(buf, &mut off, b"name", b"arm-io\0");
    put_prop(buf, &mut off, b"#address-cells", &2u32.to_le_bytes());
    put_prop(buf, &mut off, b"#size-cells",    &2u32.to_le_bytes());
    // Identity ranges so peripheral addresses map 1:1 to physical.
    let mut ranges = [0u8; 24];
    ranges[16..24].copy_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes());
    put_prop(buf, &mut off, b"ranges", &ranges);

    // 9 peripherals, in the order soc::discover_from_adt walks them.
    // Addresses are real M4/T8132 values from m1n1 source + Asahi docs.
    put_periph(buf, &mut off, b"uart0\0",      0x3_ad20_0000, 0x4000,
               b"uart-1,samsung\0apple,uart\0\0");
    put_periph(buf, &mut off, b"aic\0",        0x2_8e10_0000, 0x10000,
               b"aic,2\0apple,aic2\0\0");
    put_periph(buf, &mut off, b"disp0\0",      0x2_2810_0000, 0x10000,
               b"apple,h16-display-pipe\0\0");
    put_periph(buf, &mut off, b"dart-disp0\0", 0x2_2810_8000, 0x4000,
               b"apple,t8110-dart\0\0");
    put_periph(buf, &mut off, b"ans\0",        0x2_7bcc_0000, 0x40000,
               b"apple,nvme-ans2\0\0");
    put_periph(buf, &mut off, b"spi0\0",       0x2_3510_0000, 0x4000,
               b"apple,t8132-spi\0apple,spi\0\0");
    put_periph(buf, &mut off, b"sep\0",        0x2_4100_0000, 0x4000,
               b"apple,sep\0\0");
    put_periph(buf, &mut off, b"dart-usb\0",   0x2_3920_0000, 0x4000,
               b"apple,t8110-dart\0\0");
    put_periph(buf, &mut off, b"dart-ans\0",   0x2_7bc8_0000, 0x4000,
               b"apple,t8110-dart\0\0");

    off
}

// ─── Synthetic BootArgsRaw ──────────────────────────────────────────

static mut SYNTHETIC_ARGS: core::mem::MaybeUninit<boot_args::BootArgsRaw> =
    core::mem::MaybeUninit::uninit();

unsafe fn build_synthetic_args(adt_bytes: usize) {
    let args = (&mut *core::ptr::addr_of_mut!(SYNTHETIC_ARGS)).as_mut_ptr();
    let adt_ptr = core::ptr::addr_of!(SYNTHETIC_ADT) as *const u8;
    // Write the struct field-by-field (avoids constructing the literal
    // which requires all fields and the cmdline [u8; 1024] to be spelled).
    core::ptr::write(&mut (*args).revision as *mut u16, 3u16);
    core::ptr::write(&mut (*args).version  as *mut u16, 1u16);
    core::ptr::write(&mut (*args).virt_base as *mut u64, 0x4008_0000u64);
    core::ptr::write(&mut (*args).phys_base as *mut u64, 0x4000_0000u64);
    core::ptr::write(&mut (*args).mem_size  as *mut u64, 0x8000_0000u64); // 2 GiB
    core::ptr::write(&mut (*args).top_of_kernel_data as *mut u64, 0x4100_0000u64);
    let video = boot_args::BootVideo {
        base: 0, display: 0, stride: 0, width: 0, height: 0, depth: 0,
    };
    core::ptr::write(&mut (*args).video, video);
    core::ptr::write(&mut (*args).machine_type as *mut u32, 0x8132u32);
    core::ptr::write(&mut (*args).devtree as *mut *const u8, adt_ptr);
    core::ptr::write(&mut (*args).devtree_size as *mut u32, adt_bytes as u32);
    // cmdline buffer: zero it
    let cmdline = core::ptr::addr_of_mut!((*args).cmdline) as *mut u8;
    for i in 0..boot_args::CMDLINE_LEN_RV3 {
        core::ptr::write(cmdline.add(i), 0);
    }
    core::ptr::write(&mut (*args).boot_flags as *mut u64, 0);
    core::ptr::write(&mut (*args).mem_size_actual as *mut u64, 0x8000_0000u64);

    // plausibility: adt_ptr must lie within [phys_base, phys_base+mem_size).
    // In QEMU virt, SYNTHETIC_ADT lives in kernel RAM which is well below
    // phys_base=0x40000000. So we widen phys_base to include the kernel
    // by lowering it. Hack-ish but legal: declare phys_base=0 and
    // mem_size=huge so the ADT passes plausibility.
    core::ptr::write(&mut (*args).phys_base as *mut u64, 0u64);
    core::ptr::write(&mut (*args).mem_size  as *mut u64, 0xFFFF_FFFFu64);
}

// ─── Public entry ──────────────────────────────────────────────────

/// Run the full Apple boot-args + ADT parse path against synthetic data.
/// Called from `kernel_main` when the `layer-b-test` feature is enabled.
/// Prints results via QEMU UART then halts.
pub fn run() -> ! {
    uart::puts("\n=== Sphragis Layer-B Synthetic Apple-Path Test ===\n");
    uart::puts("[layer-b] building synthetic ADT...\n");

    let adt_bytes = unsafe { build_synthetic_adt() };
    uart::puts("[layer-b] ADT built, bytes: ");
    crate::kernel::mm::print_num(adt_bytes);
    uart::puts("\n");

    uart::puts("[layer-b] building synthetic BootArgsRaw...\n");
    unsafe { build_synthetic_args(adt_bytes) };

    let args_ptr = unsafe {
        (&*core::ptr::addr_of!(SYNTHETIC_ARGS)).as_ptr()
    };

    uart::puts("[layer-b] calling boot_args::parse()...\n");
    let parsed = unsafe { boot_args::parse(args_ptr) };
    match parsed {
        Ok(args) => {
            uart::puts("[layer-b]  PARSE OK\n");
            uart::puts("  revision: ");
            crate::kernel::mm::print_num(args.revision() as usize);
            uart::puts("\n  machine_type: 0x");
            print_hex32(args.machine_type());
            uart::puts("\n  mem_size: ");
            crate::kernel::mm::print_num((args.mem_size() / (1024 * 1024)) as usize);
            uart::puts(" MiB\n  devtree: ");
            crate::kernel::mm::print_num(args.devtree_bytes().len());
            uart::puts(" bytes\n");

            uart::puts("[layer-b] calling adt::root() + walking...\n");
            match args.adt() {
                Ok(adt) => {
                    match adt.root() {
                        Ok(root) => {
                            match root.name() {
                                Ok(n) => {
                                    uart::puts("  root.name = ");
                                    uart::puts(n);
                                    uart::puts("\n");
                                }
                                Err(_) => uart::puts("  root.name: ERR\n"),
                            }
                            match root.subnode("arm-io") {
                                Ok(io) => {
                                    uart::puts("  found /arm-io\n");
                                    match io.subnode("uart0") {
                                        Ok(_) => uart::puts("  found /arm-io/uart0\n"),
                                        Err(_) => uart::puts("  uart0 NOT found\n"),
                                    }
                                }
                                Err(_) => uart::puts("  /arm-io NOT found\n"),
                            }

                            uart::puts("[layer-b] calling soc::discover_from_adt...\n");
                            let n = soc::discover_from_adt(&adt);
                            uart::puts("  resolved peripherals: ");
                            crate::kernel::mm::print_num(n);
                            uart::puts(" / 9\n");
                            // LAYER C: print every resolved address so we
                            // can spot-check each peripheral.
                            print_periph("uart0",      soc::uart0_base() as u64, 0x3_ad20_0000);
                            print_periph("aic",        soc::aic_base()   as u64, 0x2_8e10_0000);
                            print_periph("dcp",        soc::dcp_base()   as u64, 0x2_2810_0000);
                            print_periph("dcp_dart",   soc::dcp_dart()   as u64, 0x2_2810_8000);
                            print_periph("ans",        soc::ans_base()   as u64, 0x2_7bcc_0000);
                            print_periph("spi0",       soc::spi0_base()  as u64, 0x2_3510_0000);
                            print_periph("sep",        soc::sep_base()   as u64, 0x2_4100_0000);
                            print_periph("dart_usb",   soc::dart_usb()   as u64, 0x2_3920_0000);
                            print_periph("dart_ans",   soc::dart_ans()   as u64, 0x2_7bc8_0000);

                            // LAYER C: iterate /arm-io children to exercise
                            // child-iter + compatible-string-list iter at scale.
                            if let Ok(io_node) = root.subnode("arm-io") {
                                uart::puts("[layer-b] /arm-io children:\n");
                                let mut count = 0usize;
                                for child in io_node.children() {
                                    count += 1;
                                    if let Ok(name) = child.name() {
                                        uart::puts("  - ");
                                        uart::puts(name);
                                        if let Ok(c) = child.prop("compatible") {
                                            uart::puts(" [");
                                            let mut first = true;
                                            for s in c.strings() {
                                                if !first { uart::puts(", "); }
                                                uart::puts(s);
                                                first = false;
                                            }
                                            uart::puts("]");
                                        }
                                        uart::puts("\n");
                                    }
                                }
                                uart::puts("  total children iterated: ");
                                crate::kernel::mm::print_num(count);
                                uart::puts("\n");
                            }
                        }
                        Err(_) => uart::puts("  ADT root: ERR\n"),
                    }
                }
                Err(_) => uart::puts("  ADT wrap: ERR\n"),
            }
        }
        Err(e) => {
            uart::puts("[layer-b]  PARSE ERR: ");
            match e {
                boot_args::BootArgsError::NullPointer => uart::puts("NullPointer\n"),
                boot_args::BootArgsError::UnsupportedRevision(_) => uart::puts("UnsupportedRevision\n"),
                boot_args::BootArgsError::ImplausibleDevtree { .. } => uart::puts("ImplausibleDevtree\n"),
                boot_args::BootArgsError::MemSizeZero => uart::puts("MemSizeZero\n"),
                boot_args::BootArgsError::VideoBad => uart::puts("VideoBad\n"),
            }
        }
    }

    uart::puts("\n=== Layer-B test complete. Halting. ===\n");
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

/// Print a peripheral resolution result in `name: 0xHHHH (expected 0xEEEE) [OK|MISMATCH]` form.
fn print_periph(name: &str, got: u64, expected: u64) {
    uart::puts("  ");
    uart::puts(name);
    uart::puts(": 0x");
    print_hex64(got);
    uart::puts(if got == expected { " [OK]\n" } else { " [MISMATCH]\n" });
}

fn print_hex32(val: u32) {
    const HX: &[u8; 16] = b"0123456789abcdef";
    for i in (0..8).rev() {
        let nib = ((val >> (i * 4)) & 0xF) as usize;
        uart::putc(HX[nib]);
    }
}

fn print_hex64(val: u64) {
    print_hex32((val >> 32) as u32);
    print_hex32(val as u32);
}
