// Bat_OS — ELF Binary Loader
// Parses ARM64 ELF binaries and loads them into isolated address spaces.
// Handles both static and dynamically linked executables.
//
// ELF format (simplified):
//   ELF Header → Program Headers → Section data
//   We care about PT_LOAD segments (code + data to map)
//   and PT_INTERP (path to dynamic linker).

use crate::kernel::mm::frame;

// ELF magic
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

// ELF class
const ELFCLASS64: u8 = 2;

// ELF machine
const EM_AARCH64: u16 = 183;

// ELF type
const ET_EXEC: u16 = 2;  // Executable
const ET_DYN: u16 = 3;   // Shared object (PIE executable)

// Program header types
const PT_NULL: u32 = 0;
const PT_LOAD: u32 = 1;
const PT_INTERP: u32 = 3;
const PT_PHDR: u32 = 6;

// Program header flags
const PF_X: u32 = 1; // Execute
const PF_W: u32 = 2; // Write
const PF_R: u32 = 4; // Read

// Page size
const PAGE_SIZE: usize = 4096;

/// ELF64 Header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// ELF64 Program Header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// Loaded ELF info — everything needed to start execution
pub struct LoadedElf {
    pub entry_point: u64,
    pub base_addr: u64,      // Base address (for PIE)
    pub phdr_addr: u64,      // Address of program headers in memory
    pub phdr_num: u16,       // Number of program headers
    pub phdr_size: u16,      // Size of each program header
    pub interp: Option<InterpPath>, // Dynamic linker path
    pub stack_top: u64,      // Top of allocated stack
    pub brk_start: u64,      // Start of heap (end of last LOAD segment)
}

pub struct InterpPath {
    pub path: [u8; 256],
    pub len: usize,
}

/// Parse and validate an ELF header.
pub fn parse_header(data: &[u8]) -> Result<&Elf64Header, &'static str> {
    if data.len() < core::mem::size_of::<Elf64Header>() {
        return Err("file too small for ELF header");
    }

    let header = unsafe { &*(data.as_ptr() as *const Elf64Header) };

    // Validate magic
    if header.e_ident[0..4] != ELF_MAGIC {
        return Err("not an ELF file");
    }

    // Must be 64-bit
    if header.e_ident[4] != ELFCLASS64 {
        return Err("not a 64-bit ELF");
    }

    // Must be ARM64
    if header.e_machine != EM_AARCH64 {
        return Err("not an ARM64 binary");
    }

    // Must be executable or shared object (PIE)
    if header.e_type != ET_EXEC && header.e_type != ET_DYN {
        return Err("not an executable");
    }

    Ok(header)
}

/// Load an ELF binary into memory.
/// `data` is the raw ELF file content.
/// Returns info needed to start execution.
pub fn load(data: &[u8]) -> Result<LoadedElf, &'static str> {
    let header = parse_header(data)?;

    let is_pie = header.e_type == ET_DYN;

    // PIE binaries need a base address. Use a fixed high address
    // that doesn't conflict with kernel space.
    let base: u64 = if is_pie { 0x0040_0000 } else { 0 };

    let ph_offset = header.e_phoff as usize;
    let ph_size = header.e_phentsize as usize;
    let ph_num = header.e_phnum as usize;

    let mut interp: Option<InterpPath> = None;
    let mut brk_end: u64 = 0;
    let mut phdr_addr: u64 = 0;

    // First pass: find interpreter and calculate memory needs
    for i in 0..ph_num {
        let ph_start = ph_offset + i * ph_size;
        if ph_start + ph_size > data.len() { break; }

        let phdr = unsafe { &*(data[ph_start..].as_ptr() as *const Elf64Phdr) };

        match phdr.p_type {
            PT_INTERP => {
                // Read dynamic linker path
                let path_start = phdr.p_offset as usize;
                let path_len = (phdr.p_filesz as usize).min(255);
                if path_start + path_len <= data.len() {
                    let mut ip = InterpPath { path: [0; 256], len: path_len };
                    ip.path[..path_len].copy_from_slice(&data[path_start..path_start + path_len]);
                    // Remove null terminator
                    if path_len > 0 && ip.path[path_len - 1] == 0 {
                        ip.len = path_len - 1;
                    }
                    interp = Some(ip);
                }
            }
            PT_LOAD => {
                let seg_end = phdr.p_vaddr + phdr.p_memsz;
                if seg_end > brk_end {
                    brk_end = seg_end;
                }
            }
            PT_PHDR => {
                phdr_addr = base + phdr.p_vaddr;
            }
            _ => {}
        }
    }

    // Second pass: load PT_LOAD segments into memory
    for i in 0..ph_num {
        let ph_start = ph_offset + i * ph_size;
        if ph_start + ph_size > data.len() { break; }

        let phdr = unsafe { &*(data[ph_start..].as_ptr() as *const Elf64Phdr) };

        if phdr.p_type != PT_LOAD { continue; }

        let vaddr = (base + phdr.p_vaddr) as usize;
        let memsz = phdr.p_memsz as usize;
        let filesz = phdr.p_filesz as usize;
        let file_offset = phdr.p_offset as usize;

        // Allocate pages for this segment
        let page_start = vaddr & !(PAGE_SIZE - 1);
        let page_end = (vaddr + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let num_pages = (page_end - page_start) / PAGE_SIZE;

        for p in 0..num_pages {
            let page_addr = page_start + p * PAGE_SIZE;
            // Allocate frame and identity-map it
            // (In a full implementation, we'd use the BatCave's address space)
            let _frame = frame::alloc_frame().ok_or("out of memory loading ELF")?;
        }

        // Copy file data to memory
        if filesz > 0 && file_offset + filesz <= data.len() {
            unsafe {
                let src = data[file_offset..].as_ptr();
                let dst = vaddr as *mut u8;
                for j in 0..filesz {
                    core::arch::asm!(
                        "ldrb {tmp:w}, [{src}]",
                        "strb {tmp:w}, [{dst}]",
                        src = in(reg) src.add(j),
                        dst = in(reg) dst.add(j),
                        tmp = out(reg) _,
                    );
                }
            }
        }

        // Zero BSS (memsz > filesz region)
        if memsz > filesz {
            unsafe {
                let bss_start = vaddr + filesz;
                for j in 0..(memsz - filesz) {
                    core::arch::asm!(
                        "strb wzr, [{addr}]",
                        addr = in(reg) bss_start + j,
                    );
                }
            }
        }
    }

    // Allocate user stack (64KB)
    let stack_pages = 16;
    let mut stack_base = 0usize;
    for i in 0..stack_pages {
        let page = frame::alloc_frame().ok_or("out of memory for stack")?;
        if i == 0 { stack_base = page; }
    }
    let stack_top = stack_base + stack_pages * PAGE_SIZE;

    Ok(LoadedElf {
        entry_point: base + header.e_entry,
        base_addr: base,
        phdr_addr,
        phdr_num: header.e_phnum,
        phdr_size: header.e_phentsize,
        interp,
        stack_top: stack_top as u64,
        brk_start: base + brk_end,
    })
}

/// Build the initial stack for a Linux process.
/// Linux expects: argc, argv pointers, NULL, envp pointers, NULL, auxv pairs.
/// Returns the adjusted stack pointer.
pub fn setup_stack(
    stack_top: u64,
    argv: &[&str],
    envp: &[&str],
    elf: &LoadedElf,
) -> u64 {
    let mut sp = stack_top as usize;

    // Write strings first (at top of stack, growing down)
    let mut arg_ptrs = [0u64; 32];
    let mut env_ptrs = [0u64; 32];

    // Write argv strings
    for (i, arg) in argv.iter().enumerate() {
        sp -= arg.len() + 1; // +1 for null terminator
        unsafe {
            for (j, &b) in arg.as_bytes().iter().enumerate() {
                core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32);
            }
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + arg.len());
        }
        if i < 32 { arg_ptrs[i] = sp as u64; }
    }

    // Write envp strings
    for (i, env) in envp.iter().enumerate() {
        sp -= env.len() + 1;
        unsafe {
            for (j, &b) in env.as_bytes().iter().enumerate() {
                core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32);
            }
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + env.len());
        }
        if i < 32 { env_ptrs[i] = sp as u64; }
    }

    // Align stack to 16 bytes
    sp &= !0xF;

    // Auxiliary vector (auxv) — Linux expects these
    let auxv: [(u64, u64); 8] = [
        (6, PAGE_SIZE as u64),                 // AT_PAGESZ
        (9, elf.entry_point),                  // AT_ENTRY
        (3, elf.phdr_addr),                    // AT_PHDR
        (4, elf.phdr_size as u64),             // AT_PHENT
        (5, elf.phdr_num as u64),              // AT_PHNUM
        (11, 0),                               // AT_UID
        (12, 0),                               // AT_EUID
        (0, 0),                                // AT_NULL (terminator)
    ];

    // Push auxv (backwards)
    for &(key, val) in auxv.iter().rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) val); }
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) key); }
    }

    // Push NULL (envp terminator)
    sp -= 8;
    unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }

    // Push envp pointers
    for i in (0..envp.len().min(32)).rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env_ptrs[i]); }
    }

    // Push NULL (argv terminator)
    sp -= 8;
    unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }

    // Push argv pointers
    for i in (0..argv.len().min(32)).rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) arg_ptrs[i]); }
    }

    // Push argc
    sp -= 8;
    let argc = argv.len() as u64;
    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) argc); }

    sp as u64
}
