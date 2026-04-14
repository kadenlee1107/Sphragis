// Bat_OS — BatCave Linux Binary Runner
// Loads ELF binaries into BatCave memory and executes them.

use super::loader;
use crate::drivers::uart;
use crate::kernel::mm::frame;

// Embedded test binaries
static TEST_HELLO: &[u8] = include_bytes!("../../../test_binaries/hello_batcave.elf");
static TEST_UNAME: &[u8] = include_bytes!("../../../test_binaries/uname_test.elf");
// Real busybox — static PIE, 1.2MB, 300+ tools
static BUSYBOX: &[u8] = include_bytes!("../../../test_binaries/busybox-musl-aarch64");
// Standalone test binaries
static HELLO_ELF: &[u8] = include_bytes!("../../../tests/hello");
static HELLO_LIBC_ELF: &[u8] = include_bytes!("../../../tests/hello_libc");
static HELLO_THREADS_ELF: &[u8] = include_bytes!("../../../tests/hello_threads");

pub fn busybox_elf() -> &'static [u8] { BUSYBOX }
pub fn hello_elf() -> &'static [u8] { HELLO_ELF }
pub fn hello_libc_elf() -> &'static [u8] { HELLO_LIBC_ELF }
pub fn hello_threads_elf() -> &'static [u8] { HELLO_THREADS_ELF }

/// Run the "hello" test binary.
pub fn run_test() -> Result<(), &'static str> {
    run_small_elf(TEST_HELLO, "hello")
}

/// Run the "uname" test binary.
pub fn run_uname_test() -> Result<(), &'static str> {
    run_small_elf(TEST_UNAME, "uname_test")
}

/// Run busybox with no arguments (shows help).
pub fn run_busybox() -> Result<(), &'static str> {
    run_busybox_cmd(&["busybox"])
}

/// Run a busybox tool with arguments.
/// e.g., run_busybox_cmd(&["echo", "hello", "world"])
pub fn run_busybox_cmd(argv: &[&str]) -> Result<(), &'static str> {
    // Initialize VFS and FD table if not already done
    if !super::vfs::is_ready() {
        super::vfs::init();
    }
    super::fd::init();

    // Load primary busybox (for ash shell)
    uart::puts("[runner] Loading busybox (primary)...\n");
    let entry = loader::load_elf(BUSYBOX)?;
    let primary_phys = loader::get_phys_base();
    let primary_orig = loader::get_orig_entry();

    // Load worker busybox (separate copy for child applet execution)
    uart::puts("[runner] Loading busybox (worker)...\n");
    let worker_entry = loader::load_elf(BUSYBOX)?;
    loader::WORKER_ENTRY.store(worker_entry as usize, core::sync::atomic::Ordering::Relaxed);
    loader::WORKER_PHYS_BASE.store(loader::get_phys_base(), core::sync::atomic::Ordering::Relaxed);
    loader::WORKER_ORIG_ENTRY.store(loader::get_orig_entry(), core::sync::atomic::Ordering::Relaxed);

    // Restore primary's values for MMU setup (execute_with_args uses these)
    loader::set_phys_base(primary_phys);
    loader::set_orig_entry(primary_orig);
    loader::set_entry(entry as usize);

    uart::puts("[runner] Worker loaded, executing ash...\n");
    loader::execute_with_args(entry, argv)
}

/// Run a small test ELF (single-segment, simple format).
fn run_small_elf(elf_data: &[u8], name: &str) -> Result<(), &'static str> {
    uart::puts("[runner] Loading ");
    uart::puts(name);
    uart::puts(" (");
    crate::kernel::mm::print_num(elf_data.len());
    uart::puts(" bytes)...\n");

    let code_offset = 120;
    let code_size = elf_data.len() - code_offset;

    let code_page = frame::alloc_frame().ok_or("out of memory")?;

    for i in 0..code_size {
        let byte = elf_data[code_offset + i];
        unsafe {
            core::arch::asm!("strb {v:w}, [{a}]",
                a = in(reg) code_page + i, v = in(reg) byte as u32);
        }
    }

    let stack_base = frame::alloc_frame().ok_or("out of memory")?;
    for _ in 0..3 { frame::alloc_frame(); }
    let stack_top = stack_base + 4 * 4096;

    uart::puts("[runner] Executing...\n");
    uart::puts("[runner] --- output below ---\n");

    let kernel_sp: u64;
    unsafe { core::arch::asm!("mov {}, sp", out(reg) kernel_sp); }

    unsafe {
        core::arch::asm!(
            "mov sp, {user_sp}",
            "blr {entry}",
            user_sp = in(reg) stack_top,
            entry = in(reg) code_page,
            out("x0") _, out("x1") _, out("x2") _,
            out("x8") _,
            clobber_abi("C"),
        );
    }

    unsafe { core::arch::asm!("mov sp, {}", in(reg) kernel_sp); }

    uart::puts("[runner] --- end output ---\n");
    Ok(())
}
