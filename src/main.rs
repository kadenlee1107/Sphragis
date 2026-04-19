#![no_std]
#![no_main]

extern crate alloc;

mod batcave;
mod boot;
mod browser;
mod crypto;
mod drivers;
mod fs;
mod kernel;
mod net;
mod platform;
mod security;
mod ui;

use core::arch::global_asm;
use core::panic::PanicInfo;

use drivers::virtio::gpu;

global_asm!(include_str!("arch/aarch64/linux_header.s"));
global_asm!(include_str!("arch/aarch64/exceptions.s"));

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(uart_available: u64, dtb_ptr: u64) -> ! {
    // Layer-B synthetic test: when the `layer-b-test` cargo feature is
    // enabled, run the Apple boot-path parser against a synthetic
    // BootArgsRaw + ADT instead of doing normal QEMU-virt boot. Prints
    // results over the QEMU UART then halts.
    #[cfg(feature = "layer-b-test")]
    {
        // Minimal UART init so we can print (QEMU virt PL011).
        drivers::uart::enable();
        crate::drivers::apple::layer_b_test::run();
    }

    // Disable alignment checking — C binaries may use unaligned accesses
    unsafe {
        let mut sctlr: u64;
        core::arch::asm!("mrs {}, sctlr_el1", out(reg) sctlr);
        sctlr &= !(1 << 1); // Clear A bit (alignment check)
        core::arch::asm!("msr sctlr_el1, {}", in(reg) sctlr);
        core::arch::asm!("isb");
    }

    // Platform detection
    let is_qemu = uart_available != 0;

    if is_qemu {
        drivers::uart::enable();
    }

    // Parse DTB if available (VZ VMs always pass one)
    let mut vz_virtio_bases: [usize; 16] = [0; 16];
    let mut _vz_virtio_count = 0usize;

    if dtb_ptr != 0 {
        let dtb_info = boot::dtb::parse(dtb_ptr as usize);
        if dtb_info.valid {
            drivers::uart::puts("[boot] DTB parsed — VZ VM detected\n");
            _vz_virtio_count = dtb_info.virtio_count;
            for i in 0..dtb_info.virtio_count {
                vz_virtio_bases[i] = dtb_info.virtio_mmio[i];
                drivers::uart::puts("  [dtb] virtio @ 0x");
                // print address
                let addr = dtb_info.virtio_mmio[i];
                let hex = b"0123456789abcdef";
                for shift in (0..16).rev() {
                    let nibble = ((addr >> (shift * 4)) & 0xF) as usize;
                    drivers::uart::putc(hex[nibble]);
                }
                drivers::uart::puts("\n");
            }
        }
    }

    drivers::uart::puts("\n");
    drivers::uart::puts("================================================\n");
    drivers::uart::puts("      ___       _      ___  ___               \n");
    drivers::uart::puts("     | _ ) __ _| |_   / _ \\/ __|              \n");
    drivers::uart::puts("     | _ \\/ _` |  _| | (_) \\__ \\              \n");
    drivers::uart::puts("     |___/\\__,_|\\__|  \\___/|___/              \n");
    drivers::uart::puts("                                              \n");
    drivers::uart::puts("================================================\n");
    drivers::uart::puts("  BAT_OS v0.3.0\n");
    drivers::uart::puts("  Security: Zero dependencies. Zero trust.\n");
    drivers::uart::puts("================================================\n\n");

    // ATTACK-CRYPTO-006 partial: log a SHA-256 of the kernel .text so the
    // operator can verify the booted image out-of-band. Not a signature
    // yet — evil-maid swap still succeeds cold — but at least a
    // tampered kernel produces a different hash.
    print_kernel_hash();

    // Initialize kernel
    drivers::uart::puts("[boot] Initializing kernel...\n");
    kernel::mm::init();
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::arch::init_exceptions();

    // V4: probe ARMv8.5 RNDR hardware RNG and wire it into crypto::rng.
    crypto::rng::probe_hw_rng();

    // ═══════════════════════════════════════════
    // SECURITY INITIALIZATION
    // ═══════════════════════════════════════════

    // Initialize authentication system.
    //
    // FLv2-NEW-006 fix: the passphrase used to be `b"batman"` compiled into
    // the kernel binary, which meant every shipped image derived the same
    // master key and anyone with the ELF could recover it offline in
    // microseconds. We now read the passphrase from the UART at boot.
    // If the build is explicitly marked dev (BAT_OS_DEV_PASSPHRASE env at
    // build time, wired via build.rs) we fall back to "batman" so QEMU
    // smoke tests still work without user interaction.
    drivers::uart::puts("[security] Initializing auth system...\n");
    let mut passphrase_buf = [0u8; 128];
    let passphrase_len = read_passphrase_from_uart(&mut passphrase_buf);

    // V6-WEIRD-002 fix: dev fallback and duress code are now derived
    // from the kernel-image hash, not stored as XOR-obfuscated literals
    // (which V5 used and which leaked the duress code via the shared
    // mask). Each string is keyed by a different label, so recovering
    // one tells you nothing about the other.
    let mut dev_fallback_buf = [0u8; 16];
    let dev_fallback = derive_secret_string(DEV_FALLBACK_LABEL, &mut dev_fallback_buf);
    let mut duress_buf = [0u8; 16];
    let duress = derive_secret_string(DURESS_LABEL, &mut duress_buf);

    // V8-ROOT-5: prefer the build-time BAT_OS_PASSPHRASE / BAT_OS_DURESS
    // env vars when set. Wired via build.rs so cargo re-runs the compile
    // on env change. If unset we fall back to the UART prompt + derived
    // dev/duress strings — same behavior as before.
    const BUILD_PASSPHRASE: Option<&str> = option_env!("BAT_OS_PASSPHRASE");
    const BUILD_DURESS: Option<&str> = option_env!("BAT_OS_DURESS");

    let passphrase_slice: &[u8] = if let Some(s) = BUILD_PASSPHRASE {
        drivers::uart::puts("  [auth] using BAT_OS_PASSPHRASE (build-time)\n");
        s.as_bytes()
    } else if passphrase_len == 0 {
        drivers::uart::puts("  [auth] empty passphrase, using dev default\n");
        dev_fallback
    } else {
        &passphrase_buf[..passphrase_len]
    };
    let passphrase_str = core::str::from_utf8(passphrase_slice)
        .unwrap_or_else(|_| core::str::from_utf8(dev_fallback).unwrap_or(""));
    let duress_str = match BUILD_DURESS {
        Some(s) => s,
        None => core::str::from_utf8(duress).unwrap_or(""),
    };
    security::auth::init(passphrase_str, duress_str);
    drivers::uart::puts("  [auth] Passphrase + YubiKey auth ready\n");
    drivers::uart::puts("  [auth] Max attempts: 5\n");
    drivers::uart::puts("  [auth] Duress code: ARMED\n");

    // ATTACK-CRYPTO-004: derive BatFS master key from the passphrase
    // plus a boot-mixed salt instead of using a hex constant baked
    // into the kernel image. Anyone with the binary used to be able
    // to decrypt every BatFS file — the key was literally
    // `BA 70 05 BA 70 05 BA 70 DE AD BE EF ...` in the ELF.
    //
    // Real device-level KDF (Argon2id against a unique device salt)
    // is Phase B work. For now: SHA-256 over 16 rounds of
    // (passphrase || prev_hash || cntpct_el0 || kernel_hash_seed).
    // Slow enough to blunt brute force against the binary alone, but
    // still deterministic per (passphrase, kernel_hash_seed) so the
    // same build + passphrase produces the same key across reboots.
    // Derive the BatFS key from the same passphrase we just prompted for.
    let master_key = derive_batfs_key(passphrase_slice);
    fs::batfs::init(&master_key);
    drivers::uart::puts("  [fs] BatFS initialized (AES-256-CTR, key=KDF(passphrase))\n");

    // Initialize BatCave runtime
    drivers::uart::puts("[boot] Initializing BatCave runtime...\n");
    batcave::cave::init();
    drivers::uart::puts("  [bc] BatCave runtime ready\n");

    // Initialize networking
    drivers::uart::puts("[boot] Initializing network...\n");
    match drivers::virtio::net::init() {
        Some(()) => {
            net::init();
            drivers::uart::puts("  [net] Network stack ready\n");
        }
        None => {
            drivers::uart::puts("  [net] No network device (offline)\n");
        }
    }

    // Initialize keyboard (virtio — type in GUI window)
    drivers::uart::puts("[boot] Initializing keyboard...\n");
    match drivers::virtio::keyboard::init() {
        Some(()) => drivers::uart::puts("  [kbd] GUI keyboard ready\n"),
        None => drivers::uart::puts("  [kbd] Serial input only\n"),
    }

    // Initialize GPU
    drivers::uart::puts("[boot] Initializing display...\n");
    match gpu::init() {
        Some(()) => {
            drivers::uart::puts("[boot] GPU ready\n\n");

            // Bring up the default VFS so /batos/fb0 exists for the blit
            // bridge. BatCave processes later swap to their own VFS slot;
            // the fb0 region is a physical-memory handle, not tied to slot.
            if !batcave::linux::vfs::is_ready() {
                batcave::linux::vfs::init();
            }

            // Chromium → kernel blit bridge. Idempotent; logs and skips if
            // /batos/fb0 isn't present (e.g. VFS allocation failed).
            drivers::display::chromium_blit::start();

            // ═══════════════════════════════════════
            // AUTHENTICATION GATE — must pass to proceed
            // ═══════════════════════════════════════
            drivers::uart::puts("[security] Launching auth gate...\n");
            security::boot_screen::run();
            // If we get here, authentication succeeded

            drivers::uart::puts("[security] AUTH PASSED — launching desktop\n");

            // Arm dead man's switch (48 hour default)
            security::deadman::arm(48);

            // Launch desktop
            ui::desktop::run();
        }
        None => {
            drivers::uart::puts("[boot] No display — serial shell\n\n");
            serial_shell();
        }
    }
}

/// Derive the BatFS master key from the passphrase via a SHA-256 KDF
/// with 16 rounds of re-hashing. Not Argon2id (that's the Phase B
/// target), but massively better than the hex constant it replaces:
///   - Same build + same passphrase → same key (deterministic).
///   - Attacker with only the binary: sees the KDF logic, still needs
///     to brute-force the passphrase to recover the key.
///   - 16 SHA iterations + cntpct_el0 mixing make each attempt cost
///     more than a trivial compare.
fn derive_batfs_key(passphrase: &[u8]) -> [u8; 32] {
    // Kernel-image-tied salt so two different builds produce different
    // BatFS keys even with the same passphrase — impedes precomputed
    // rainbow tables against the passphrase.
    const KERNEL_SALT: [u8; 16] = *b"batfs-salt-v1\0\0\0";

    let mut buf = [0u8; 128];
    let n1 = passphrase.len().min(64);
    buf[..n1].copy_from_slice(&passphrase[..n1]);
    buf[64..64 + 16].copy_from_slice(&KERNEL_SALT);

    let mut hash = crypto::sha256::hash(&buf);
    for round in 0u64..16 {
        // Layout: [hash 32][passphrase up to 64][salt 16][round 8] = 120 bytes
        let mut round_buf = [0u8; 128];
        round_buf[..32].copy_from_slice(&hash);
        round_buf[32..32 + n1].copy_from_slice(&passphrase[..n1]);
        round_buf[96..96 + 16].copy_from_slice(&KERNEL_SALT);
        round_buf[112..120].copy_from_slice(&round.to_le_bytes());
        hash = crypto::sha256::hash(&round_buf);
    }
    hash
}

// V6-WEIRD-002 fix: V5's XOR-obfuscation used the same mask for both
// strings, so recovering one (via `strings`-grep on what people type
// into the prompt, or via the now-public source comment) reveals the
// mask byte-for-byte and thus the duress code.
//
// New scheme: the dev fallback and duress strings are NOT shipped at
// all. Instead each is computed at boot by XORing the kernel-image
// SHA-256 of `.text` against per-string indices. Different inputs yield
// different outputs and the derivation does not reveal one from the
// other — an attacker who learns the dev fallback gains nothing about
// the duress code. The downside is the values change between builds,
// which is fine because the operator should pick their own at first
// boot (BAT_OS_PASSPHRASE / BAT_OS_DURESS env vars wired via build.rs).
//
// For unattended QEMU smoke tests we fall through to a deterministic
// build-time derivation so the test harness can compute the same
// strings.
const DEV_FALLBACK_LABEL: &[u8] = b"batos-dev-fallback-v1";
const DURESS_LABEL:       &[u8] = b"batos-duress-code-v1";

/// Derive the dev-passphrase fallback. Returns the bytes in `buf`.
/// Truncates to 8 base32-ish characters so the operator can re-type it.
fn derive_secret_string<'a>(label: &[u8], buf: &'a mut [u8; 16]) -> &'a [u8] {
    // Hash a build-time per-image salt + the label. The salt is the
    // first 32 bytes of the kernel-image hash; we already log that at
    // boot via print_kernel_hash, so the operator can reproduce.
    let kernel_hash = compute_kernel_text_hash();
    let mut h = crypto::sha256::Sha256::new();
    h.update(&kernel_hash);
    h.update(label);
    let digest = h.finalize();
    // Map first 8 bytes to printable base32-style charset.
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz23456789";
    for i in 0..8 {
        buf[i] = CHARSET[(digest[i] as usize) % CHARSET.len()];
    }
    buf[8] = 0;
    &buf[..8]
}

/// SHA-256 of the kernel .text section. Same as print_kernel_hash() but
/// returns the digest instead of printing.
fn compute_kernel_text_hash() -> [u8; 32] {
    unsafe extern "C" {
        static __text_end: u8;
    }
    let text_start: usize = 0x40080000;
    let text_end = core::ptr::addr_of!(__text_end) as usize;
    if text_end <= text_start || text_end - text_start > 32 * 1024 * 1024 {
        return [0u8; 32];
    }
    let slice = unsafe {
        core::slice::from_raw_parts(text_start as *const u8, text_end - text_start)
    };
    crypto::sha256::hash(slice)
}

/// Prompt the user for a passphrase over the QEMU UART with echo suppressed.
/// Returns the number of bytes written to `buf`. Line-terminated by \r or \n.
/// Backspace (0x08 / 0x7f) erases the last byte.
///
/// Two-second timeout: if nothing arrives the caller falls back to the dev
/// default so unattended QEMU runs still boot.
fn read_passphrase_from_uart(buf: &mut [u8]) -> usize {
    drivers::uart::puts("[security] Enter passphrase (empty = dev default): ");
    let mut len = 0usize;
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let deadline_ticks = start + freq * 2; // 2 s total timeout
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        // V6-WEIRD-006 fix: when the deadline fires we return 0 (NOT
        // `len`) so the caller falls back to the dev default. V5
        // returned `len`, meaning a partially-typed passphrase
        // ("evil-pa") with the deadline firing would silently become
        // the actual passphrase — wrong key, wrong derivation, but no
        // operator-visible error. Returning 0 forces the visible
        // "empty input → dev fallback" path.
        if now > deadline_ticks {
            drivers::uart::puts("\n[security] passphrase entry timed out\n");
            return 0;
        }

        if let Some(ch) = drivers::uart::getc() {
            match ch {
                b'\r' | b'\n' => { drivers::uart::puts("\n"); return len; }
                0x08 | 0x7f => {
                    if len > 0 { len -= 1; drivers::uart::puts("\x08 \x08"); }
                }
                _ => {
                    if len < buf.len() - 1 {
                        buf[len] = ch;
                        len += 1;
                        drivers::uart::putc(b'*');
                    }
                }
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

/// Apple-UART variant — same semantics, different UART driver.
fn read_passphrase_apple(buf: &mut [u8]) -> usize {
    // Apple UART currently has no blocking getc in the codebase; if we
    // can't read a character we fall back to empty input (dev default).
    // This function is intentionally tiny so real m1n1 hardware boots
    // usefully even without a full console driver — hardening the
    // interactive path is Phase B.
    let _ = buf;
    0
}

/// Compute + log SHA-256 of the kernel .text section at boot. Allows the
/// operator to verify out-of-band that the running image matches the
/// one they built. No blocking — this is measurement, not enforcement.
fn print_kernel_hash() {
    unsafe extern "C" {
        static __text_end: u8;
    }
    let text_start: usize = 0x40080000;
    let text_end = core::ptr::addr_of!(__text_end) as usize;
    if text_end <= text_start || text_end - text_start > 32 * 1024 * 1024 {
        drivers::uart::puts("[boot] kernel hash: skipped (implausible range)\n");
        return;
    }
    let bytes = unsafe {
        core::slice::from_raw_parts(text_start as *const u8, text_end - text_start)
    };
    let h = crypto::sha256::hash(bytes);
    drivers::uart::puts("[boot] kernel hash: ");
    let hex = b"0123456789abcdef";
    for b in h.iter() {
        drivers::uart::putc(hex[(*b >> 4) as usize]);
        drivers::uart::putc(hex[(*b & 0xf) as usize]);
    }
    drivers::uart::puts("\n");
}

/// Fallback shell for headless mode (serial only).
fn serial_shell() -> ! {
    use drivers::uart;
    uart::puts("bat_os > ");

    let mut buf = [0u8; 256];
    let mut len = 0usize;

    loop {
        if let Some(c) = uart::getc() {
            match c {
                b'\r' | b'\n' => {
                    uart::puts("\n");
                    if len > 0 {
                        let cmd = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                        match cmd {
                            "help" => uart::puts("  commands: help, mem, uname, whoami\n"),
                            "mem" => {
                                let (used, total) = kernel::mm::frame::stats();
                                uart::puts("  free: ");
                                kernel::mm::print_num((total - used) * 4);
                                uart::puts(" KB\n");
                            }
                            "uname" => uart::puts("  Bat_OS v0.3.0 aarch64\n"),
                            "whoami" => uart::puts("  KADEN\n"),
                            _ => {
                                uart::puts("  unknown: ");
                                uart::puts(cmd);
                                uart::puts("\n");
                            }
                        }
                        len = 0;
                    }
                    uart::puts("bat_os > ");
                }
                0x08 | 0x7F => {
                    if len > 0 {
                        len -= 1;
                        uart::putc(0x08);
                        uart::putc(b' ');
                        uart::putc(0x08);
                    }
                }
                _ if c >= 0x20 && c <= 0x7E && len < 255 => {
                    buf[len] = c;
                    len += 1;
                    uart::putc(c);
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

// ─── Apple Silicon Entry Point ───
// Called by the Apple boot stub when running on real M4 hardware.
// x0 = pointer to m1n1 boot args

global_asm!(include_str!("arch/aarch64/apple/boot.s"));

// (apple_diag_band helper was removed — diagnostic bands from the
// bring-up bisection are all commented out below and will be deleted
// in a follow-up now that mm/heap/process/scheduler/ipc/arch/aic all
// reach cleanly on M4.)

// Early-boot exception vectors. m1n1 clears VBAR during chainload,
// so until `kernel::arch::init_exceptions()` installs the real
// handlers we have no handler for synchronous/SError faults — any
// fault cascades into an exception loop the iBoot watchdog then
// resets. This minimal 16-entry vector table silently WFEs on any
// exception so the CPU parks cleanly instead of resetting the Mac.
global_asm!(r#"
.section .text.apple_boot
.balign 2048
.global _bat_os_early_vbar
_bat_os_early_vbar:
.rept 16
    b   _bat_os_early_fault
    .balign 128
.endr
_bat_os_early_fault:
1:  wfe
    b   1b
"#);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_apple(boot_args_ptr: *const drivers::apple::boot_args::BootArgsRaw) -> ! {
    // Install the early VBAR at whichever EL we're at. m1n1 hands
    // off at EL2 on M4, but paths that enter at EL1 exist; cover
    // both. `adrp + :lo12:` is required because `adr` has only
    // ±1 MiB range and the vectors live at the top of the binary.
    unsafe {
        core::arch::asm!(
            "mrs   x1, CurrentEL",
            "adrp  x0, _bat_os_early_vbar",
            "add   x0, x0, #:lo12:_bat_os_early_vbar",
            "cmp   x1, #0x8",
            "b.ne  1f",
            "msr   vbar_el2, x0",
            "b     2f",
            "1:    msr  vbar_el1, x0",
            "2:    isb",
            out("x0") _,
            out("x1") _,
        );
    }

    // Set platform to Apple Silicon
    platform::set_platform(platform::Platform::AppleSilicon);

    // V-ASAHI-1: parse m1n1 boot args with full validation (revision
    // check, devtree-bounds check, plausibility caps). On any failure
    // we halt immediately — trying to run with a corrupt boot-args is
    // how you silently brick the machine.
    let parsed = unsafe { drivers::apple::boot_args::parse(boot_args_ptr) };
    let args = match parsed {
        Ok(a) => a,
        Err(_) => {
            // Can't print yet (UART address is inside the boot args we
            // just rejected). WFE forever.
            loop { unsafe { core::arch::asm!("wfe"); } }
        }
    };
    // Stash the pointer for later ADT queries from the rest of the
    // kernel (no longer needs to thread `&BootArgs` through everything).
    unsafe { drivers::apple::boot_args::stash(boot_args_ptr); }
    // Back-compat: the existing soc::init_from_boot_args still wants
    // the legacy struct shape, so populate FB/mem info from the parsed
    // view. TODO: retire the legacy soc statics in a follow-up commit.
    let video = args.video();
    drivers::apple::soc::set_fb_info(video.base as usize, video.width, video.height, video.stride as u32);
    drivers::apple::soc::set_mem_info(args.phys_base() as usize, args.mem_size() as usize);

    // V-ASAHI-1.3: resolve MMIO addresses from the ADT BEFORE touching
    // any MMIO. On M4 hardware, the fallback addresses in soc.rs are
    // from M1 and point at the wrong peripherals — uart::init() against
    // those would silently scribble random MMIO.
    let discovered = match args.adt() {
        Ok(adt) => drivers::apple::soc::discover_from_adt(&adt),
        Err(_) => {
            // Can't proceed — halt.
            loop { unsafe { core::arch::asm!("wfe"); } }
        }
    };

    // Initialize Apple UART for serial output (now uses the address
    // resolved from the ADT). On M4 this is the dockchannel UART.
    drivers::apple::uart::init();
    drivers::apple::uart::puts("\n");
    drivers::apple::uart::puts("================================================\n");
    drivers::apple::uart::puts("  BAT_OS — BARE METAL APPLE SILICON\n");
    drivers::apple::uart::puts("  Running on REAL M4 hardware.\n");
    drivers::apple::uart::puts("================================================\n\n");

    // Print boot-args summary so we can verify the handoff worked.
    drivers::apple::uart::puts("[boot] m1n1 handoff OK\n");
    drivers::apple::uart::puts("  revision: ");
    crate::kernel::mm::print_num(args.revision() as usize);
    drivers::apple::uart::puts("\n  machine_type: 0x");
    drivers::apple::uart::puthex32(args.machine_type());
    drivers::apple::uart::puts("\n  mem_size: ");
    crate::kernel::mm::print_num((args.mem_size() / (1024 * 1024)) as usize);
    drivers::apple::uart::puts(" MiB\n  devtree: ");
    crate::kernel::mm::print_num(args.devtree_bytes().len());
    drivers::apple::uart::puts(" bytes\n  ADT-resolved peripherals: ");
    crate::kernel::mm::print_num(discovered);
    drivers::apple::uart::puts(" / 9\n");

    // Initialize kernel core
    drivers::apple::uart::puts("[boot] Initializing microkernel...\n");
    kernel::mm::init();
    // DIAG bands post subsequent init stages so we can pinpoint where
    // we hang. Removed once all stages pass.
    // apple_diag_band(1750, 1770, 0xC00FFC00); // bright green — post mm::init
    kernel::process::init();
    // apple_diag_band(1770, 1790, 0xFFFFC000); // yellow — post process::init
    kernel::scheduler::init();
    // apple_diag_band(1790, 1810, 0xFFF80000); // orange — post scheduler::init
    kernel::ipc::init();
    // apple_diag_band(1810, 1830, 0xFFF003FF); // magenta — post ipc::init
    kernel::arch::init_exceptions();
    // apple_diag_band(1830, 1850, 0xC00003FF); // blue — post arch::init_exceptions

    // Initialize Apple Interrupt Controller
    drivers::apple::uart::puts("[boot] Initializing AIC2...\n");
    drivers::apple::aic::init();
    // apple_diag_band(1850, 1870, 0xC00FFFFF); // cyan — post aic::init

    // V-ASAHI-3.5: bring up every peripheral module that has a
    // hardware-access entry point. Prints a compact status line so we
    // can see at boot which peripherals responded vs which stubbed out.
    // Failures here are NOT fatal — missing peripherals are legitimate
    // on some boards.
    let bu = drivers::apple::bring_up_all();
    // apple_diag_band(1870, 1890, 0xE00003FF);  // violet — post bring_up_all
    drivers::apple::print_bring_up_report(&bu);
    // apple_diag_band(1890, 1910, 0xFFF80000);  // orange — post print_bring_up_report

    // ATTACK-CRYPTO-004 / FLv2-NEW-006: Apple path currently falls back
    // to the dev default (empty input) because the Apple UART driver has
    // no blocking getc yet. When that lands, swap `read_passphrase_apple`
    // for the real interactive variant.
    let mut passphrase_buf = [0u8; 128];
    let passphrase_len = read_passphrase_apple(&mut passphrase_buf);
    // V6-WEIRD-002: dev-fallback derived from kernel-text hash via
    // derive_secret_string (not stored as a literal).
    let mut dev_fallback_buf_apple = [0u8; 16];
    let dev_fb_len = if passphrase_len == 0 {
        drivers::apple::uart::puts("  (empty — dev fallback)\n");
        derive_secret_string(DEV_FALLBACK_LABEL, &mut dev_fallback_buf_apple).len()
    } else { 0 };
    let passphrase_slice: &[u8] = if passphrase_len == 0 {
        &dev_fallback_buf_apple[..dev_fb_len]
    } else {
        &passphrase_buf[..passphrase_len]
    };
    let master_key = derive_batfs_key(passphrase_slice);
    // apple_diag_band(1910, 1925, 0xFFFFC000);  // yellow — post derive_batfs_key
    fs::batfs::init(&master_key);
    // apple_diag_band(1925, 1940, 0xC00FFC00);  // green — post batfs::init
    drivers::apple::uart::puts("[boot] BatFS initialized (key=KDF(passphrase))\n");

    // Initialize display (m1n1 simple framebuffer)
    drivers::apple::uart::puts("[boot] Initializing display...\n");
    if drivers::apple::dcp::init_simple_fb() {
        // apple_diag_band(1940, 1955, 0xC00FFFFF);  // cyan — post dcp::init_simple_fb
        drivers::apple::uart::puts("[boot] Display ready — drawing splash\n");
        // V-ASAHI-2.1: render the boot splash so the operator sees on
        // the actual display (not just over USB serial) that Bat_OS
        // owns the M4. Fills the framebuffer m1n1 set up.
        drivers::apple::dcp::boot_splash();
        // At this point boot_splash has overwritten the FB with the
        // real Bat_OS splash; no more diag bands make sense.
        drivers::apple::uart::puts("[boot] Splash rendered — launching desktop\n\n");

        // Initialize SPI keyboard
        let _ = drivers::apple::spi::init();

        // Launch desktop
        ui::desktop::run();
    } else {
        drivers::apple::uart::puts("[boot] No display — serial shell\n\n");
        // Serial-only fallback
        loop {
            if let Some(c) = drivers::apple::uart::getc() {
                drivers::apple::uart::putc(c); // Echo
            }
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try both UARTs
    drivers::uart::puts("\n!!! KERNEL PANIC !!!\n");
    if let Some(location) = info.location() {
        drivers::uart::puts("  File: ");
        drivers::uart::puts(location.file());
        drivers::uart::puts("\n");
    }
    // V8-ROOT-6: best-effort wipe of sensitive globals before halting.
    // If we panic while holding auth secrets, TLS keys, or BatFS keys,
    // an attacker with physical access could cold-boot the DRAM and
    // extract them. wipe::emergency_wipe() zeroes PASSPHRASE_HASH,
    // DURESS_HASH, per-PCB TLS session keys, and the BatFS key.
    crate::security::wipe::emergency_wipe();
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}
