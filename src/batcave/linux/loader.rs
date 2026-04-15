// Bat_OS — Production ELF Loader
// Handles real-world static ARM64 Linux binaries (like busybox).

use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE_SIZE: usize = 4096;

static LOADED_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);
static LOADED_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static SAVED_RETURN_ADDR: AtomicUsize = AtomicUsize::new(0);
static SAVED_KERNEL_SP: AtomicUsize = AtomicUsize::new(0);

// Worker busybox instance (second copy for child applet execution)
pub static WORKER_ENTRY: AtomicUsize = AtomicUsize::new(0);
pub static WORKER_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static WORKER_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);

// Hello binary instance (standalone ELF, loaded on demand)
pub static HELLO_ENTRY: AtomicUsize = AtomicUsize::new(0);
pub static HELLO_PHYS_BASE: AtomicUsize = AtomicUsize::new(0);
pub static HELLO_ORIG_ENTRY: AtomicUsize = AtomicUsize::new(0);

/// Re-initialize a previously loaded ELF at the given phys_base.
/// Re-copies all PT_LOAD segments and re-applies relocations.
/// Does NOT allocate new pages — reuses existing allocation.
pub fn reinit_elf(data: &[u8], phys_base: usize) {
    if data.len() < 64 { return; }

    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    let mut min_addr: u64 = u64::MAX;
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        if vaddr < min_addr { min_addr = vaddr; }
    }

    let reloc_offset = phys_base as i64 - min_addr as i64;

    // Re-copy PT_LOAD segments (data + zero BSS)
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        let phys_addr = (vaddr as i64 + reloc_offset) as usize;

        if filesz > 0 && p_offset + filesz <= data.len() {
            for j in 0..filesz {
                let byte = data[p_offset + j];
                unsafe {
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) phys_addr + j, v = in(reg) byte as u32);
                }
            }
        }
        if memsz > filesz {
            for j in filesz..memsz {
                unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) phys_addr + j); }
            }
        }
    }

    // Re-apply relocations
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;

        let mut pos = dyn_offset;
        while pos + 16 <= data.len() && pos < dyn_offset + dyn_size {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag { 0 => break, 7 => rela_off = val as usize, 8 => rela_sz = val as usize, _ => {} }
            pos += 16;
        }

        if rela_off > 0 && rela_sz > 0 {
            let num = rela_sz / 24;
            for r in 0..num {
                let re = rela_off + r * 24;
                if re + 24 > data.len() { break; }
                let r_offset = u64_at(data, re);
                let r_info = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                if (r_info & 0xFFFFFFFF) as u32 == 0x403 {
                    let patch_addr = (r_offset as i64 + reloc_offset) as usize;
                    let value = (r_addend as i64 + reloc_offset) as u64;
                    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) patch_addr, v = in(reg) value); }
                }
            }
        }
    }
}

pub fn get_phys_base() -> usize { LOADED_PHYS_BASE.load(Ordering::Relaxed) }
pub fn get_orig_entry() -> usize { LOADED_ORIG_ENTRY.load(Ordering::Relaxed) }
pub fn get_entry() -> usize { LOADED_ENTRY.load(Ordering::Relaxed) }
pub fn set_phys_base(v: usize) { LOADED_PHYS_BASE.store(v, Ordering::Relaxed); }
pub fn set_orig_entry(v: usize) { LOADED_ORIG_ENTRY.store(v, Ordering::Relaxed); }
pub fn set_entry(v: usize) { LOADED_ENTRY.store(v, Ordering::Relaxed); }

pub fn load_elf(data: &[u8]) -> Result<u64, &'static str> {
    if data.len() < 64 { return Err("too small"); }
    if &data[0..4] != b"\x7fELF" { return Err("not ELF"); }

    let entry = u64_at(data, 24);
    let phoff = u64_at(data, 32) as usize;
    let phnum = u16_at(data, 56) as usize;
    let phentsz = u16_at(data, 54) as usize;

    uart::puts("[loader] Entry: 0x"); print_hex(entry); uart::puts("\n");

    let mut min_addr: u64 = u64::MAX;
    let mut max_addr: u64 = 0;

    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        let ptype = u32_at(data, ph);
        if ptype != 1 { continue; }
        let vaddr = u64_at(data, ph + 16);
        let memsz = u64_at(data, ph + 40);
        if vaddr < min_addr { min_addr = vaddr; }
        if vaddr + memsz > max_addr { max_addr = vaddr + memsz; }
    }

    let total_size = (max_addr - min_addr) as usize;
    let total_pages = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;

    // Allocate 2MB-aligned for MMU block mapping
    let mut phys_base = frame::alloc_frame().ok_or("oom")?;
    while phys_base & 0x1FFFFF != 0 {
        phys_base = frame::alloc_frame().ok_or("oom")?;
    }
    for _ in 1..total_pages {
        frame::alloc_frame().ok_or("oom")?;
    }

    uart::puts("[loader] Physical base: 0x"); print_hex(phys_base as u64); uart::puts("\n");

    let reloc_offset = phys_base as i64 - min_addr as i64;

    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        let ptype = u32_at(data, ph);
        if ptype != 1 { continue; }

        let p_offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        let phys_addr = (vaddr as i64 + reloc_offset) as usize;

        if filesz > 0 && p_offset + filesz <= data.len() {
            for j in 0..filesz {
                let byte = data[p_offset + j];
                unsafe {
                    core::arch::asm!("strb {v:w}, [{a}]",
                        a = in(reg) phys_addr + j, v = in(reg) byte as u32);
                }
            }
        }
        if memsz > filesz {
            for j in filesz..memsz {
                unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) phys_addr + j); }
            }
        }
    }

    let phys_entry = (entry as i64 + reloc_offset) as u64;

    // Apply relocations (PT_DYNAMIC)
    for i in 0..phnum {
        let ph = phoff + i * phentsz;
        if ph + phentsz > data.len() { break; }
        if u32_at(data, ph) != 2 { continue; }

        let dyn_offset = u64_at(data, ph + 8) as usize;
        let dyn_size = u64_at(data, ph + 32) as usize;
        let mut rela_off: usize = 0;
        let mut rela_sz: usize = 0;

        let mut pos = dyn_offset;
        while pos + 16 <= data.len() && pos < dyn_offset + dyn_size {
            let tag = u64_at(data, pos);
            let val = u64_at(data, pos + 8);
            match tag { 0 => break, 7 => rela_off = val as usize, 8 => rela_sz = val as usize, _ => {} }
            pos += 16;
        }

        if rela_off > 0 && rela_sz > 0 {
            let num = rela_sz / 24;
            uart::puts("[loader] Applying "); crate::kernel::mm::print_num(num); uart::puts(" relocations\n");
            uart::puts("[loader] reloc_offset=0x"); print_hex(reloc_offset as u64); uart::puts("\n");
            let mut applied = 0usize;
            for r in 0..num {
                let re = rela_off + r * 24;
                if re + 24 > data.len() { break; }
                let r_offset = u64_at(data, re);
                let r_info = u64_at(data, re + 8);
                let r_addend = u64_at(data, re + 16);
                if (r_info & 0xFFFFFFFF) as u32 == 0x403 {
                    let patch_addr = (r_offset as i64 + reloc_offset) as usize;
                    let value = (r_addend as i64 + reloc_offset) as u64;
                    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) patch_addr, v = in(reg) value); }
                    applied += 1;
                    // Debug first and prop_dispatch relocation
                    if applied <= 2 || r_offset == 0x7b4e8 {
                        uart::puts("[reloc] vaddr=0x"); print_hex(r_offset);
                        uart::puts(" → phys=0x"); print_hex(patch_addr as u64);
                        uart::puts(" val=0x"); print_hex(value);
                        uart::puts("\n");
                    }
                }
            }
            uart::puts("[loader] Applied "); crate::kernel::mm::print_num(applied); uart::puts(" R_RELATIVE\n");
        }
    }

    // Flush ALL loaded pages INCLUDING relocated data: clean data cache + invalidate icache
    // Must cover the FULL memory range (max_addr - min_addr) not just file data,
    // because relocations patch addresses in BSS and data sections
    unsafe {
        let start = phys_base & !63;
        let end = phys_base + total_size + 0x20000; // extra 128KB to cover BSS + relocations
        let mut addr = start;
        while addr < end {
            core::arch::asm!("dc cvac, {a}", a = in(reg) addr);
            core::arch::asm!("ic ivau, {a}", a = in(reg) addr);
            addr += 64;
        }
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }

    LOADED_ENTRY.store(phys_entry as usize, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(entry as usize, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(phys_base, Ordering::Relaxed);
    Ok(phys_entry)
}

pub fn execute(entry: u64) -> Result<(), &'static str> {
    execute_with_args(entry, &["busybox"])
}

pub fn execute_with_args(entry: u64, argv: &[&str]) -> Result<(), &'static str> {
    let orig_entry = LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64;
    let phys_base = LOADED_PHYS_BASE.load(Ordering::Relaxed);

    let stack_base = frame::alloc_frame().ok_or("oom")?;
    for _ in 0..15 { frame::alloc_frame(); }
    let stack_top = stack_base + 16 * PAGE_SIZE;

    // CRITICAL: Zero the entire stack to prevent garbage from being
    // read as pointers by C code (local variable initialization)
    unsafe {
        let ptr = stack_base as *mut u8;
        for i in 0..(16 * PAGE_SIZE) {
            core::ptr::write_volatile(ptr.add(i), 0);
        }
    }

    let mut sp = stack_top;

    // AT_RANDOM
    sp -= 16;
    let random_addr = sp;
    for i in 0..16usize {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val); }
        let byte = ((val >> (i % 8 * 8)) ^ (i as u64 * 37)) as u8;
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + i, v = in(reg) byte as u32); }
    }

    // Write argv strings
    let mut arg_addrs = [0usize; 16];
    let argc = argv.len().min(16);
    for i in 0..argc {
        let arg = argv[i].as_bytes();
        sp -= arg.len() + 1;
        arg_addrs[i] = sp;
        for (j, &b) in arg.iter().enumerate() {
            unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
        }
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) sp + arg.len()); }
    }

    // envp
    sp -= 10;
    let env0 = sp;
    for (j, &b) in b"PATH=/bin\0".iter().enumerate() {
        unsafe { core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) sp + j, v = in(reg) b as u32); }
    }

    sp = (sp - 64) & !0xF;

    // auxv: AT_NULL, AT_RANDOM, AT_PAGESZ
    for &(k, v) in &[(0u64, 0u64), (25u64, random_addr as u64), (6u64, 4096u64)] {
        sp -= 16;
        unsafe {
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) k);
            core::arch::asm!("str {v}, [{a}]", a = in(reg) sp + 8, v = in(reg) v);
        }
    }

    // envp NULL + pointer
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    sp -= 8; unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) env0 as u64); }

    // argv NULL + pointers
    sp -= 8; unsafe { core::arch::asm!("str xzr, [{a}]", a = in(reg) sp); }
    for i in (0..argc).rev() {
        sp -= 8;
        unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) arg_addrs[i] as u64); }
    }

    // argc
    sp -= 8;
    unsafe { core::arch::asm!("str {v}, [{a}]", a = in(reg) sp, v = in(reg) argc as u64); }

    // TLS
    let tls_page = frame::alloc_frame().ok_or("oom")?;
    for i in 0..PAGE_SIZE {
        unsafe { core::arch::asm!("strb wzr, [{a}]", a = in(reg) tls_page + i); }
    }
    unsafe { core::arch::asm!("msr tpidr_el0, {}", in(reg) (tls_page + PAGE_SIZE - 256) as u64); }

    // Enable MMU
    super::mmu::setup_and_enable(phys_base)?;

    let virt_entry = orig_entry;
    uart::puts("[loader] --- executing ---\n");

    // Save kernel SP to a FIXED memory location (0x40000100)
    // This address is in the first page of RAM, always accessible
    const SP_SAVE_ADDR: usize = 0x40000100;
    unsafe {
        let ksp: u64;
        core::arch::asm!("mov {}, sp", out(reg) ksp);
        core::arch::asm!("str {v}, [{a}]", a = in(reg) SP_SAVE_ADDR, v = in(reg) ksp);
    }

    // Save return address
    let ret_addr: u64;
    unsafe { core::arch::asm!("adr {ret}, 99f", ret = out(reg) ret_addr); }
    SAVED_RETURN_ADDR.store(ret_addr as usize, Ordering::Relaxed);

    // Jump to busybox
    unsafe {
        core::arch::asm!(
            "mov sp, {user_sp}",
            "br {entry}",
            user_sp = in(reg) sp,
            entry = in(reg) virt_entry,
        );
    }

    // Label 99: exit handler returns here after disabling MMU
    // ALL registers are corrupt. Use hardcoded address to restore SP.
    unsafe {
        core::arch::asm!(
            "99:",
            "movz {tmp}, #0x1000",
            "movk {tmp}, #0x4000, lsl #16",
            "ldr {tmp}, [{tmp}]",
            "mov sp, {tmp}",
            tmp = out(reg) _,
        );
    }

    // If we get here, SP is restored and we can use the UART
    uart::puts("[loader] BACK!\n");

    uart::puts("[loader] --- done ---\n");
    Ok(())
}

/// Load a standalone (non-busybox) ELF binary and store its metadata
/// in the HELLO_* atomics. Returns (phys_entry, phys_base, orig_entry).
pub fn load_hello_elf(data: &[u8]) -> Result<(u64, usize, u64), &'static str> {
    // Save current loader state (so we don't clobber busybox metadata)
    let saved_entry = LOADED_ENTRY.load(Ordering::Relaxed);
    let saved_phys = LOADED_PHYS_BASE.load(Ordering::Relaxed);
    let saved_orig = LOADED_ORIG_ENTRY.load(Ordering::Relaxed);

    let phys_entry = load_elf(data)?;
    let phys_base = LOADED_PHYS_BASE.load(Ordering::Relaxed);
    let orig_entry = LOADED_ORIG_ENTRY.load(Ordering::Relaxed) as u64;

    // Store in HELLO_* atomics
    HELLO_ENTRY.store(phys_entry as usize, Ordering::Relaxed);
    HELLO_PHYS_BASE.store(phys_base, Ordering::Relaxed);
    HELLO_ORIG_ENTRY.store(orig_entry as usize, Ordering::Relaxed);

    // Restore busybox loader state
    LOADED_ENTRY.store(saved_entry, Ordering::Relaxed);
    LOADED_PHYS_BASE.store(saved_phys, Ordering::Relaxed);
    LOADED_ORIG_ENTRY.store(saved_orig, Ordering::Relaxed);

    Ok((phys_entry, phys_base, orig_entry))
}

fn u64_at(data: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3],
        data[off+4], data[off+5], data[off+6], data[off+7]])
}
fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]])
}
fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off+1]])
}
fn print_hex(val: u64) {
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() { uart::putc(hex[((val >> (i * 4)) & 0xF) as usize]); }
}
