#![allow(dead_code)]
#![allow(unused_assignments)]
// Sphragis — Flattened Device Tree (FDT/DTB) Parser
// VZLinuxBootLoader passes a standard DTB in x0 at boot.
// This tells us where EVERY device is in the VM.
//
// FDT format:
//   Header (40 bytes) → structure block → strings block
//   Structure tokens: FDT_BEGIN_NODE, FDT_PROP, FDT_END_NODE, FDT_END

const FDT_MAGIC: u32 = 0xD00DFEED;
const FDT_BEGIN_NODE: u32 = 0x01;
const FDT_PROP: u32 = 0x03;
const FDT_END_NODE: u32 = 0x02;
const FDT_END: u32 = 0x09;
const FDT_NOP: u32 = 0x04;

#[repr(C)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

pub struct DtbInfo {
    pub virtio_mmio: [usize; 16],
    pub virtio_count: usize,
    pub memory_base: usize,
    pub memory_size: usize,
    /// Initrd physical range populated by QEMU via `-initrd`.
    /// Both zero when no initrd was supplied. Range is inclusive
    /// start, exclusive end per Linux FDT convention.
    pub initrd_start: usize,
    pub initrd_end:   usize,
    pub valid: bool,
}

impl DtbInfo {
    fn new() -> Self {
        Self {
            virtio_mmio: [0; 16],
            virtio_count: 0,
            memory_base: 0,
            memory_size: 0,
            initrd_start: 0,
            initrd_end:   0,
            valid: false,
        }
    }
}

fn be32(addr: usize) -> u32 {
    let val = unsafe { core::ptr::read_volatile(addr as *const u32) };
    u32::from_be(val)
}

fn be64(addr: usize) -> u64 {
    let hi = be32(addr) as u64;
    let lo = be32(addr + 4) as u64;
    (hi << 32) | lo
}

/// Parse a DTB at the given address.
/// Returns info about virtio devices and memory layout.
pub fn parse(dtb_addr: usize) -> DtbInfo {
    let mut info = DtbInfo::new();

    if dtb_addr == 0 {
        return info;
    }

    // Check magic
    let magic = be32(dtb_addr);
    if magic != FDT_MAGIC {
        return info;
    }

    let header = dtb_addr;
    let struct_offset = be32(header + 8) as usize;
    let strings_offset = be32(header + 12) as usize;

    let struct_base = dtb_addr + struct_offset;
    let strings_base = dtb_addr + strings_offset;

    info.valid = true;

    // Walk the structure block
    let mut pos = struct_base;
    let mut current_node = [0u8; 64];
    let mut node_len = 0usize;
    let mut _depth = 0u32;

    loop {
        let token = be32(pos);
        pos += 4;

        match token {
            FDT_BEGIN_NODE => {
                // Read node name (null-terminated, 4-byte aligned)
                node_len = 0;
                loop {
                    let b = unsafe { *(pos as *const u8) };
                    if b == 0 { break; }
                    if node_len < 63 {
                        current_node[node_len] = b;
                        node_len += 1;
                    }
                    pos += 1;
                }
                pos += 1; // skip null
                pos = (pos + 3) & !3; // align to 4
                _depth += 1;

                // Check if this is a virtio-mmio node
                let name = unsafe { core::str::from_utf8_unchecked(&current_node[..node_len]) };
                if name.starts_with("virtio_mmio@") || name.starts_with("virtio@") {
                    // Extract address from node name (after @)
                    if let Some(at_pos) = name.find('@') {
                        let addr_str = &name[at_pos + 1..];
                        if let Some(addr) = parse_hex(addr_str) {
                            if info.virtio_count < 16 {
                                info.virtio_mmio[info.virtio_count] = addr;
                                info.virtio_count += 1;
                            }
                        }
                    }
                }
            }
            FDT_PROP => {
                let val_len = be32(pos) as usize;
                let name_off = be32(pos + 4) as usize;
                pos += 8;

                // Get property name from strings table
                let prop_name_addr = strings_base + name_off;
                let prop_name = read_str(prop_name_addr);

                // Check for interesting properties
                let node_name = unsafe { core::str::from_utf8_unchecked(&current_node[..node_len]) };

                if prop_name == "reg" && node_name.starts_with("memory") {
                    if val_len >= 16 {
                        info.memory_base = be64(pos) as usize;
                        info.memory_size = be64(pos + 8) as usize;
                    }
                }

                // `/chosen/linux,initrd-start` + `linux,initrd-end`.
                // QEMU emits u32 or u64 depending on DTB address-cells.
                // Accept both widths.
                if node_name == "chosen"
                    && (prop_name == "linux,initrd-start"
                        || prop_name == "linux,initrd-end")
                {
                    let val = match val_len {
                        4 => be32(pos) as usize,
                        8 => be64(pos) as usize,
                        _ => 0,
                    };
                    if prop_name == "linux,initrd-start" {
                        info.initrd_start = val;
                    } else {
                        info.initrd_end = val;
                    }
                }

                // Skip value (4-byte aligned)
                pos += (val_len + 3) & !3;
            }
            FDT_END_NODE => {
                _depth -= 1;
                node_len = 0;
            }
            FDT_NOP => {}
            FDT_END => break,
            _ => break, // Unknown token, stop
        }
    }

    info
}

fn read_str(addr: usize) -> &'static str {
    let mut len = 0;
    loop {
        let b = unsafe { *((addr + len) as *const u8) };
        if b == 0 { break; }
        len += 1;
        if len >= 64 { break; }
    }
    unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(addr as *const u8, len)) }
}

fn parse_hex(s: &str) -> Option<usize> {
    let mut result: usize = 0;
    for b in s.bytes() {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as usize,
            b'a'..=b'f' => (b - b'a' + 10) as usize,
            b'A'..=b'F' => (b - b'A' + 10) as usize,
            _ => return Some(result), // Stop at non-hex char
        };
        result = result * 16 + digit;
    }
    Some(result)
}
