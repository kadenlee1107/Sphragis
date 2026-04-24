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
static NETSURF_TEST_ELF: &[u8] = include_bytes!("../../../tests/netsurf_css_test");
pub fn netsurf_test_elf() -> &'static [u8] { NETSURF_TEST_ELF }
static FREETYPE_TEST_ELF: &[u8] = include_bytes!("../../../tests/freetype_test");
pub fn freetype_test_elf() -> &'static [u8] { FREETYPE_TEST_ELF }
static PNG_TEST_ELF: &[u8] = include_bytes!("../../../tests/png_test");
pub fn png_test_elf() -> &'static [u8] { PNG_TEST_ELF }
static POSIX_TEST_ELF: &[u8] = include_bytes!("../../../tests/posix_test");
pub fn posix_test_elf() -> &'static [u8] { POSIX_TEST_ELF }
static CXX_TEST_ELF: &[u8] = include_bytes!("../../../tests/cxx_test");
pub fn cxx_test_elf() -> &'static [u8] { CXX_TEST_ELF }
static V8_EXEC_ELF: &[u8] = include_bytes!("../../../tests/v8_exec");
pub fn v8_exec_elf() -> &'static [u8] { V8_EXEC_ELF }
static V8_TEST_ELF: &[u8] = include_bytes!("../../../tests/v8_test");
pub fn v8_test_elf() -> &'static [u8] { V8_TEST_ELF }
static BLINK_TEST_ELF: &[u8] = include_bytes!("../../../tests/blink_tokenizer_test");
pub fn blink_test_elf() -> &'static [u8] { BLINK_TEST_ELF }
static CSS_TOKENIZER_TEST_ELF: &[u8] = include_bytes!("../../../tests/css_tokenizer_test");
pub fn css_tokenizer_test_elf() -> &'static [u8] { CSS_TOKENIZER_TEST_ELF }

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

/// Launch the baked Chromium `content_shell` blob with the given argv.
///
/// Returns `Err("no chromium blob")` if the kernel image was built
/// without `tools/bake_chromium.sh`. Otherwise loads the blob via the
/// shared ELF loader, registers the main thread with the scheduler,
/// and hands off to `loader::execute_with_args`.
pub fn run_chromium(url: &str, argv: &[&str]) -> Result<(), &'static str> {
    use crate::kernel::mm::initrd;

    let blob = match initrd::locate_chromium_blob() {
        Some(b) => b,
        None => return Err("no chromium blob"),
    };

    let bi = initrd::info().ok_or("blob info missing")?;
    if !bi.crc_valid {
        uart::puts("[runner] WARNING: Chromium blob CRC mismatch; refusing to load\n");
        return Err("chromium blob CRC mismatch");
    }
    // V5-SUPPLY-004 fix: changed from "warn on unsigned dev build" to
    // "refuse unless explicitly permitted". A shipped kernel with
    // INITRD_PUBKEY=[0u8;32] used to just print a warning and load the
    // blob, making the Ed25519 verify infrastructure a no-op on every
    // production binary. Now the default is REFUSE; dev builds opt in
    // explicitly via `BAT_OS_ALLOW_UNSIGNED_INITRD=1` at build time.
    // (build.rs declares this as a rerun-if-env-changed input so cargo
    // picks up changes without a `cargo clean`.)
    // Presence of BAT_OS_ALLOW_UNSIGNED_INITRD (any value) opts in. We'd
    // prefer `matches!(…, Some("1") | Some("true"))` but const `str` equality
    // isn't stable yet — is_some() is the same convention Cargo features use
    // (setting the flag at all means "yes, I know what I'm doing").
    const ALLOW_UNSIGNED_INITRD: bool =
        option_env!("BAT_OS_ALLOW_UNSIGNED_INITRD").is_some();
    let pk_nonzero = initrd::INITRD_PUBKEY.iter().any(|&b| b != 0);
    if !bi.sig_valid {
        if pk_nonzero {
            uart::puts("[runner] FATAL: Chromium blob signature INVALID — refusing\n");
            return Err("chromium blob signature invalid");
        }
        if !ALLOW_UNSIGNED_INITRD {
            uart::puts("[runner] FATAL: INITRD_PUBKEY=0 and ALLOW_UNSIGNED_INITRD=false — refusing\n");
            return Err("chromium blob unsigned");
        }
        uart::puts("[runner] WARNING: unsigned blob permitted by ALLOW_UNSIGNED_INITRD\n");
    }

    if !super::vfs::is_ready() {
        super::vfs::init();
    }
    super::fd::init();

    uart::puts("[runner] Loading content_shell (");
    crate::kernel::mm::print_num(blob.len() / (1024 * 1024));
    uart::puts(" MB) into sandboxed cave...\n");

    // ROOT-1: Chromium runs in a per-cave page table with its user VA
    // window at 0x10000000 — above MMIO (0x08M-0x0AM). The cave window
    // (mmu::CAVE_BLOCKS × 2 MB = 400 MB default) fits today's ~280 MB
    // content_shell plus stack + heap headroom.
    const CHROMIUM_VIRT_BASE: u64 = 0x10000000;

    let cave_slot = super::mmu::alloc_cave_slot().ok_or("no free cave slots")?;
    uart::puts("[runner] Cave slot "); crate::kernel::mm::print_num(cave_slot);
    uart::puts(" allocated\n");

    // If the initrd is a BATARCH multi-file archive (tools/bake_chromium_archive.sh),
    // load every file in it as a separate ELF in the same cave, with a
    // cross-module symbol-resolution pass to fix up content_shell's
    // undefined glibc/pthread/libm references to the real library bodies.
    // Falls back to the legacy single-ELF path for plain-blob initrds.
    let info = if initrd::is_archive() {
        let shell = initrd::archive_file("bin/content_shell").ok_or("archive has no bin/content_shell")?;
        // Collect libs in the order they appear in the archive. Main exe
        // MUST be files[0] because load_archive_multi treats it as the
        // entry point and sets the loader globals from it.
        let mut files: [(&[u8], &[u8]); 16] = [(&[], &[]); 16];
        let mut count = 0usize;
        files[0] = (b"bin/content_shell", shell);
        count += 1;
        // Walk the archive header for every lib/* entry.
        initrd::archive_for_each(|name, _sz| {
            if count >= files.len() { return; }
            if !name.starts_with("lib/") { return; }
            if let Some(bytes) = initrd::archive_file(name) {
                // Unchecked cast OK: archive_file returns an initrd-region
                // slice with 'static lifetime; we already bounded count.
                // Copy the name into a static-ish slot: we reuse the name
                // bytes from inside the archive's own header region.
                let name_bytes: &'static [u8] = {
                    // We don't have the name's slice pointer from archive_for_each;
                    // re-derive it by searching. archive_file copies the name up
                    // to the lookup — but we need the backing bytes for the table.
                    // Simpler: leak through a hardcoded matcher for the well-known
                    // libs we expect so we can point at a long-lived &str.
                    match_known_lib_name(name)
                };
                if !name_bytes.is_empty() {
                    files[count] = (name_bytes, bytes);
                    count += 1;
                }
            }
        });
        uart::puts("[runner] archive: "); crate::kernel::mm::print_num(count);
        uart::puts(" file(s)\n");
        match loader::load_archive_multi(&files[..count], CHROMIUM_VIRT_BASE) {
            Ok(i) => i,
            Err(e) => { super::mmu::free_cave_slot(cave_slot); return Err(e); }
        }
    } else {
        match loader::load_elf_rebased(blob, CHROMIUM_VIRT_BASE) {
            Ok(i) => i,
            Err(e) => { super::mmu::free_cave_slot(cave_slot); return Err(e); }
        }
    };

    let l1 = match super::mmu::setup_cave_pagetable_at(
        cave_slot, info.phys_base, CHROMIUM_VIRT_BASE) {
        Ok(l) => l,
        Err(e) => { super::mmu::free_cave_slot(cave_slot); return Err(e); }
    };

    uart::puts("[cave] chromium now on its own page table (L1=0x");
    let hex = b"0123456789abcdef";
    for i in (0..16).rev() {
        let nibble = ((l1 as u64 >> (i * 4)) & 0xF) as usize;
        uart::putc(hex[nibble]);
    }
    uart::puts(")\n");

    uart::puts("[runner] Launching on ");
    uart::puts(url);
    uart::puts("\n");

    super::threads::init_main_thread(info.virt_entry, 0);

    // Turn on per-syscall tracing while we're in the Chromium debug loop.
    // Prints one line per svc #0 so we can see what content_shell calls.
    super::syscall::SYSCALL_TRACE.store(true, core::sync::atomic::Ordering::Relaxed);

    // Ensure the MMU is enabled with PRIMARY_L1 before we switch to
    // chromium's cave L1. The cave path assumes MMU is already up (see
    // mmu::setup_and_enable's V2-NEW-026 comment) — if chromium is the
    // first user binary after boot, nobody has turned it on yet. Calling
    // setup_and_enable here builds PRIMARY_L1 and flips SCTLR.M=1; the
    // immediately-following switch_to_cave overwrites TTBR0 with our
    // rebased L1. The setup_and_enable call inside execute_with_args is
    // then a no-op (SCTLR.M==1), preserving chromium's page table.
    super::mmu::setup_and_enable(info.phys_base)?;
    super::mmu::switch_to_cave(l1);

    // Flush the entire icache before handing off to EL0 — our cache
    // maintenance in load_archive_multi happened while MMU was OFF, so
    // the VAs we invalidated with `ic ivau` were phys addresses. Once
    // the cave's TTBR0 is active, EL0 fetches come through new VAs
    // (virt_base + offset) that may still have stale icache lines from
    // the kernel's own fetches (ic ivau is VA-tagged). `ic iallu` is
    // a blanket "invalidate all icache" — overkill but safe.
    unsafe {
        core::arch::asm!("ic iallu");
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }



    let r = loader::execute_with_args(info.virt_entry, argv);
    super::mmu::switch_to_primary();
    // FLv2-NEW-017/018: free both the cave page-table frames AND the
    // ELF image frames. free_cave_slot frees the L1/L2_low/L2_high;
    // free_loaded_elf returns the ~150 MB Chromium image to the pool.
    super::mmu::free_cave_slot(cave_slot);
    loader::free_loaded_elf(&info);
    r
}

/// We need a `&'static [u8]` for each lib name so the multi-ELF loader
/// can record it in `LoadedLib::name_bytes`. Returning a reference into
/// the archive's header region would work but would require preserving
/// pointers through `archive_for_each`. A fixed match for the libs
/// `tools/bake_chromium_archive.sh` actually packs is simpler and makes
/// new libs explicit (you have to add a branch here when you add one to
/// the bake).
fn match_known_lib_name(name: &str) -> &'static [u8] {
    match name {
        "lib/ld-linux-aarch64.so.1" => b"lib/ld-linux-aarch64.so.1",
        "lib/libc.so.6"             => b"lib/libc.so.6",
        "lib/libdl.so.2"            => b"lib/libdl.so.2",
        "lib/libexpat.so.1"         => b"lib/libexpat.so.1",
        "lib/libgcc_s.so.1"         => b"lib/libgcc_s.so.1",
        "lib/libm.so.6"             => b"lib/libm.so.6",
        "lib/libnspr4.so"           => b"lib/libnspr4.so",
        "lib/libnss3.so"            => b"lib/libnss3.so",
        "lib/libnssutil3.so"        => b"lib/libnssutil3.so",
        "lib/libplc4.so"            => b"lib/libplc4.so",
        "lib/libplds4.so"           => b"lib/libplds4.so",
        "lib/libpthread.so.0"       => b"lib/libpthread.so.0",
        _ => b"",
    }
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
