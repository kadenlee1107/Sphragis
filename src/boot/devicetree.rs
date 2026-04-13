// Bat_OS — Apple Device Tree Parser
// When iBoot loads us, it passes a device tree (ADT) in x0.
// The ADT contains the ACTUAL hardware addresses for this specific
// machine — UART base, framebuffer address, memory map, etc.
//
// Apple's device tree format is similar to FDT (Flattened Device Tree)
// but with Apple-specific structure.
//
// Format:
// - Properties are stored as name-value pairs
// - Nodes are nested
// - Each node: n_properties(u32), n_children(u32), then properties, then children
// - Each property: name[32], length(u32), value[length]

use crate::drivers::uart;

const MAX_NAME: usize = 32;

#[repr(C)]
struct AdtNodeHeader {
    n_properties: u32,
    n_children: u32,
}

#[repr(C)]
struct AdtProperty {
    name: [u8; 32],
    length: u32,
}

/// Parse the Apple Device Tree and extract critical hardware info.
pub fn parse(adt_addr: usize) -> DeviceInfo {
    let mut info = DeviceInfo::default();

    if adt_addr == 0 {
        return info;
    }

    uart::puts("  [adt] Parsing device tree at 0x");
    print_hex_short(adt_addr as u64);
    uart::puts("\n");

    // Walk the root node
    let mut offset = adt_addr;
    offset = parse_node(offset, "", &mut info, 0);

    uart::puts("  [adt] Parse complete\n");
    info
}

fn parse_node(addr: usize, path: &str, info: &mut DeviceInfo, depth: usize) -> usize {
    let header = unsafe { &*(addr as *const AdtNodeHeader) };
    let n_props = header.n_properties as usize;
    let n_children = header.n_children as usize;

    let mut offset = addr + 8; // Skip header

    // Read properties
    let mut node_name = [0u8; 32];

    for _ in 0..n_props {
        let prop = unsafe { &*(offset as *const AdtProperty) };
        let name = prop_name(&prop.name);
        let value_len = prop.length as usize;
        let value_addr = offset + 36; // 32 (name) + 4 (length)

        // Extract specific properties we care about
        match name {
            "name" => {
                let len = value_len.min(31);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        value_addr as *const u8,
                        node_name.as_mut_ptr(),
                        len,
                    );
                }
            }
            "reg" if depth > 0 => {
                // "reg" contains the MMIO base address
                if value_len >= 8 {
                    let base = unsafe { *(value_addr as *const u64) };
                    let node = prop_name(&node_name);
                    match node {
                        "uart0" | "serial0" => {
                            info.uart_base = base as usize;
                        }
                        "framebuffer" | "display" => {
                            info.fb_base = base as usize;
                        }
                        _ => {}
                    }
                }
            }
            "width" => {
                if value_len >= 4 {
                    info.fb_width = unsafe { *(value_addr as *const u32) };
                }
            }
            "height" => {
                if value_len >= 4 {
                    info.fb_height = unsafe { *(value_addr as *const u32) };
                }
            }
            "stride" => {
                if value_len >= 4 {
                    info.fb_stride = unsafe { *(value_addr as *const u32) };
                }
            }
            "AAPL,phandle" => {}
            _ => {}
        }

        // Advance past this property
        let aligned_len = (value_len + 3) & !3; // 4-byte aligned
        offset = value_addr + aligned_len;
    }

    // Parse children recursively
    for _ in 0..n_children {
        offset = parse_node(offset, "", info, depth + 1);
    }

    offset
}

fn prop_name(raw: &[u8; 32]) -> &str {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(32);
    unsafe { core::str::from_utf8_unchecked(&raw[..end]) }
}

#[derive(Default)]
pub struct DeviceInfo {
    pub uart_base: usize,
    pub fb_base: usize,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
    pub ram_base: usize,
    pub ram_size: usize,
}

fn print_hex_short(val: u64) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        uart::putc(hex[nibble]);
    }
}
