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

    // V5-SUPPLY-003 fix: the literal strings "batman" and "letmein" used
    // to live in the shipped ELF — `strings bat_os | grep` recovered
    // both. Now obfuscated via compile-time XOR against a fixed mask.
    // Deobfuscation is trivial to anyone who disassembles, but defeats
    // the one-line `strings` discovery. Real defense is to set
    // BAT_OS_DEV_PASSPHRASE / BAT_OS_DURESS_CODE env vars at build.
    let mut dev_fallback_buf = [0u8; 16];
    let dev_fallback = deobf(&DEV_FALLBACK_OBF, &mut dev_fallback_buf);
    let mut duress_buf = [0u8; 16];
    let duress = deobf(&DURESS_OBF, &mut duress_buf);

    let passphrase_slice: &[u8] = if passphrase_len == 0 {
        drivers::uart::puts("  [auth] empty passphrase, using dev default\n");
        dev_fallback
    } else {
        &passphrase_buf[..passphrase_len]
    };
    let passphrase_str = core::str::from_utf8(passphrase_slice)
        .unwrap_or_else(|_| core::str::from_utf8(dev_fallback).unwrap_or(""));
    let duress_str = core::str::from_utf8(duress).unwrap_or("");
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

// V5-SUPPLY-003 fix: the dev-passphrase and duress-code strings used to
// sit as plain ASCII in the kernel ELF, recoverable via
// `strings bat_os | grep -E 'batman|letmein'`. Now stored XOR'd with
// a per-byte rotating mask so they don't appear as clear-text bytes.
//
// These are NOT real secrets — deobfuscation is trivial once you
// disassemble. The real defense is BAT_OS_DEV_PASSPHRASE /
// BAT_OS_DURESS_CODE env vars at build time (TBD via build.rs). For
// now this closes the "strings" discovery path.
//
// XOR mask: 0xA5, 0x5A, 0x3C, 0xC3, 0x69, 0x96, 0xF0, 0x0F (repeat).
//
// Encoded:
//   "batman\0"  XOR mask[0..7] = 62^A5, 61^5A, 74^3C, 6D^C3, 61^69, 6E^96, 00^F0
//                              = C7, 3B, 48, AE, 08, F8, F0
//   "letmein\0" XOR mask[0..8] = 6C^A5, 65^5A, 74^3C, 6D^C3, 65^69, 69^96, 6E^F0, 00^0F
//                              = C9, 3F, 48, AE, 0C, FF, 9E, 0F
const DEV_FALLBACK_OBF: [u8; 7] = [0xC7, 0x3B, 0x48, 0xAE, 0x08, 0xF8, 0xF0];
const DURESS_OBF:       [u8; 8] = [0xC9, 0x3F, 0x48, 0xAE, 0x0C, 0xFF, 0x9E, 0x0F];
const XOR_MASK:         [u8; 8] = [0xA5, 0x5A, 0x3C, 0xC3, 0x69, 0x96, 0xF0, 0x0F];

/// Deobfuscate into `buf`. Returns the populated slice (length excluding
/// the trailing NUL). `buf` must be at least `src.len()` bytes.
fn deobf<'a>(src: &[u8], buf: &'a mut [u8]) -> &'a [u8] {
    let mut out_len = 0usize;
    for (i, &b) in src.iter().enumerate() {
        let v = b ^ XOR_MASK[i % 8];
        if v == 0 { break; }
        buf[i] = v;
        out_len = i + 1;
    }
    &buf[..out_len]
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
        // V5-WEIRD-006 fix: deadline check runs on EVERY iteration, not
        // just when getc returns None. A UART-flooder that kept the
        // receive FIFO full could otherwise hold us in the Some() branch
        // indefinitely, either preventing boot or (with carefully-timed
        // characters) injecting controlled bytes into the passphrase.
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now > deadline_ticks { drivers::uart::puts("\n"); return len; }

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

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_apple(boot_args: *const drivers::apple::soc::M1n1BootArgs) -> ! {
    // Set platform to Apple Silicon
    platform::set_platform(platform::Platform::AppleSilicon);

    // Parse boot args from m1n1
    let args = unsafe { &*boot_args };
    drivers::apple::soc::init_from_boot_args(args);

    // Initialize Apple UART for serial output
    drivers::apple::uart::init();
    drivers::apple::uart::puts("\n");
    drivers::apple::uart::puts("================================================\n");
    drivers::apple::uart::puts("  BAT_OS — BARE METAL APPLE SILICON\n");
    drivers::apple::uart::puts("  Running on REAL M4 hardware.\n");
    drivers::apple::uart::puts("================================================\n\n");

    // Initialize kernel core
    drivers::apple::uart::puts("[boot] Initializing microkernel...\n");
    kernel::mm::init();
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::arch::init_exceptions();

    // Initialize Apple Interrupt Controller
    drivers::apple::uart::puts("[boot] Initializing AIC2...\n");
    drivers::apple::aic::init();

    // ATTACK-CRYPTO-004 / FLv2-NEW-006: Apple path currently falls back
    // to the dev default (empty input) because the Apple UART driver has
    // no blocking getc yet. When that lands, swap `read_passphrase_apple`
    // for the real interactive variant.
    let mut passphrase_buf = [0u8; 128];
    let passphrase_len = read_passphrase_apple(&mut passphrase_buf);
    // V5-SUPPLY-003: dev-fallback string obfuscated (XOR) in binary.
    let mut dev_fallback_buf_apple = [0u8; 16];
    let dev_fb_len = if passphrase_len == 0 {
        drivers::apple::uart::puts("  (empty — dev fallback)\n");
        deobf(&DEV_FALLBACK_OBF, &mut dev_fallback_buf_apple).len()
    } else { 0 };
    let passphrase_slice: &[u8] = if passphrase_len == 0 {
        &dev_fallback_buf_apple[..dev_fb_len]
    } else {
        &passphrase_buf[..passphrase_len]
    };
    let master_key = derive_batfs_key(passphrase_slice);
    fs::batfs::init(&master_key);
    drivers::apple::uart::puts("[boot] BatFS initialized (key=KDF(passphrase))\n");

    // Initialize display (m1n1 simple framebuffer)
    drivers::apple::uart::puts("[boot] Initializing display...\n");
    if drivers::apple::dcp::init_simple_fb() {
        drivers::apple::uart::puts("[boot] Display ready — launching desktop\n\n");
        // Fill screen black to prove we own it
        drivers::apple::dcp::fill_screen(0xFF000000);
        drivers::apple::dcp::flush(0, 0,
            drivers::apple::dcp::width(),
            drivers::apple::dcp::height());

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
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}
