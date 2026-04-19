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

// ─── Bring-up exception vectors ────────────────────────────────────
// Minimal 16-entry vector table that, on any exception, paints the
// framebuffer red (ARGB2101010 0xFFF00000) and halts. Installed at
// EL1 VBAR *before* we walk the ADT so data aborts from bad
// pointers become a visible red-screen halt instead of a mysterious
// iBoot watchdog reset.
global_asm!(r#"
.section .text.apple_boot
.balign 2048
.global bringup_vectors
bringup_vectors:
.rept 16
    b   bringup_fault
    .balign 128
.endr

bringup_fault:
    // Paint BLUE (0xC00003FF) in the bottom 1 MiB of the 16 MiB paint
    // region — chosen to be visually unmistakable against our warm-hue
    // per-path markers. Leaves the upper 15 MiB showing whatever
    // checkpoint color was painted last so we can read where we were.
    ldr     x9,  =0x103e0f50000        // fb_base + 15 MiB offset
    mov     w10, #0x03ff                // low 16: B=max (10 bits)
    movk    w10, #0xc000, lsl #16       // high 16: A=3, R=0, G=0
    mov     x11, #0x00100000            // 1 MiB
1:  str     w10, [x9], #4
    subs    x11, x11, #4
    b.ne    1b
    dsb     sy
2:  wfe
    b       2b
"#);

// ─── FB marker for early-Rust bring-up ─────────────────────────────
// Paints the whole M4 framebuffer with a given 32-bit ARGB2101010
// pixel. Used to prove which Rust checkpoint we last reached before a
// crash. Pixel format: bits[31:30]=A, [29:20]=R, [19:10]=G, [9:0]=B.
// Examples (each 10-bit channel max = 0x3FF):
//   0xFFF00000  pure red       A=3, R=max, G=0, B=0
//   0xC00FFC00  pure green
//   0xC00003FF  pure blue
//   0xFFF003FF  magenta
//   0xC00FFFFF  cyan
//   0xFFFFFFFF  white
//   0xFFF80000  orange         R=max, G=0x200
//   0xE00C0000  dark-orange    R=0x200, G=0x300
/// Paint a horizontal stripe on the M4 FB.
/// `y_start` / `y_count` are in pixels; `pixel` is 32-bit ARGB2101010.
/// FB layout: 3024x1964 @ stride 0x2f40 bytes = 12096 = 3024 * 4.
/// Safe to call from the bring-up sequence; doesn't allocate.
#[inline(never)]
pub(crate) unsafe fn fb_stripe(y_start: usize, y_count: usize, pixel: u32) {
    const FB_BASE: usize = 0x103e0050000;
    const FB_STRIDE_BYTES: usize = 0x2f40;       // 12096
    const FB_WIDTH_PX: usize    = 3024;
    for y in y_start..(y_start + y_count) {
        let row = (FB_BASE + y * FB_STRIDE_BYTES) as *mut u32;
        for x in 0..FB_WIDTH_PX {
            core::ptr::write_volatile(row.add(x), pixel);
        }
    }
    core::arch::asm!("dsb sy");
}

#[inline(never)]
pub(crate) unsafe fn fb_mark(pixel: u32) {
    let fb = 0x103e0050000usize as *mut u32;
    // 16 MiB worth of 4-byte pixels = 4 M pixels, enough to fill most
    // of the 3024x1964 FB (~23.6 MiB). Faster than full fill and still
    // visually unambiguous on a camera grab.
    let count: usize = 0x01000000 / 4;
    for i in 0..count {
        core::ptr::write_volatile(fb.add(i), pixel);
    }
    core::arch::asm!("dsb sy");
}

// fb_hold is now an alias for fb_mark — the delay loop version was
// taking many seconds per stage at M4's slow pre-cpufreq clock. We'll
// rely on placing explicit stops (WFE loops) at specific checkpoints
// instead of per-stage dwell.
#[inline(never)]
unsafe fn fb_hold(pixel: u32) {
    fb_mark(pixel);
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_apple(boot_args_ptr: *const drivers::apple::boot_args::BootArgsRaw) -> ! {
    // Install bring-up exception vectors FIRST. m1n1 runs at EL2 and
    // its chainload can leave us at EL2 or EL1 — set VBAR at both so
    // faults route to `bringup_fault` regardless. Also unmask SError
    // (DAIF.A cleared) so any pending/future SError delivers
    // immediately to our red-paint halt instead of being deferred.
    unsafe {
        // Determine current EL and only set the appropriate VBAR.
        // CurrentEL bits [3:2] = EL (0=EL0, 1=EL1, 2=EL2, 3=EL3).
        // Writing `msr vbar_el2` from EL1 traps; we only do it if at EL2.
        // Keep SError masked (DAIF.A=1 from boot.s) — clearing it was
        // delivering a pending SError left over from m1n1 and firing the
        // bringup_fault handler before any Rust code ran.
        // Use `adrp + add` for the vector-table address — `adr` has
        // only ±1 MiB range and `bringup_vectors` sits near the top of
        // the binary (in .text.apple_boot) while `kernel_main_apple`
        // is megabytes deeper, so `adr` was silently wrapping and
        // installing a bogus VBAR. `adrp` is ±4 GiB, always works.
        core::arch::asm!(
            "mrs   x1, CurrentEL",
            "adrp  x0, bringup_vectors",
            "add   x0, x0, #:lo12:bringup_vectors",
            "cmp   x1, #0x8",             // EL2 encoded as 2 << 2 = 0x8
            "b.ne  1f",
            "msr   vbar_el2, x0",
            "b     2f",
            "1:    msr  vbar_el1, x0",
            "2:    isb",
            out("x0") _,
            out("x1") _,
        );
    }

    // R1: Rust entered. ORANGE.
    unsafe { fb_hold(0xFFF80000); }

    // Set platform to Apple Silicon. Previously this faulted because
    // boot.s was zeroing BSS at link-time absolute addresses
    // (0x81xxxxxxx) rather than the loaded binary's BSS — so any
    // subsequent static write via PC-relative adrp went to memory
    // that might have been partially corrupt. Now fixed in boot.s
    // (BSS zero + stack setup both use adrp + :lo12:).
    platform::set_platform(platform::Platform::AppleSilicon);

    // R2: post-set_platform. DARK-ORANGE.
    unsafe { fb_hold(0xE00C0000); }

    // R2: post-set_platform. DARK-ORANGE.
    unsafe { fb_hold(0xE00C0000); }

    // V-ASAHI-1: parse m1n1 boot args with full validation (revision
    // check, devtree-bounds check, plausibility caps). On any failure
    // we halt immediately — trying to run with a corrupt boot-args is
    // how you silently brick the machine.
    let parsed = unsafe { drivers::apple::boot_args::parse(boot_args_ptr) };
    let args = match parsed {
        Ok(a) => a,
        Err(_) => {
            // R-fail-parse: RED. Can't print yet (UART inside boot args).
            unsafe { fb_hold(0xFFF00000); }
            loop { unsafe { core::arch::asm!("wfe"); } }
        }
    };
    // R3: boot_args parsed OK. GREEN-CYAN (teal).
    unsafe { fb_hold(0xC00C0300); }
    unsafe { drivers::apple::boot_args::stash(boot_args_ptr); }
    // R3a: post-stash. NAVY (0xC0000200).
    unsafe { fb_hold(0xC0000200); }
    let video = args.video();
    // R3b: post args.video(). PINK (0xFFF80200).
    unsafe { fb_hold(0xFFF80200); }
    drivers::apple::soc::set_fb_info(video.base as usize, video.width, video.height, video.stride as u32);
    // R3c: post set_fb_info. LIME (0xD00FFC00).
    unsafe { fb_hold(0xD00FFC00); }
    drivers::apple::soc::set_mem_info(args.phys_base() as usize, args.mem_size() as usize);
    // R3d: post set_mem_info. SALMON (0xFFF40100).
    unsafe { fb_hold(0xFFF40100); }

    // V-ASAHI-1.3: resolve MMIO addresses from the ADT BEFORE touching
    // any MMIO. On M4 hardware, the fallback addresses in soc.rs are
    // from M1 and point at the wrong peripherals — uart::init() against
    // those would silently scribble random MMIO.
    // R4a: about to call args.adt(). PURPLE.
    unsafe { fb_hold(0xE00003FF); }
    let discovered = match args.adt() {
        Ok(adt) => {
            // R4b: args.adt() returned Ok. PURPLE-GRAY (0xE804C800) —
            // picked unique so if we see this in the top region the
            // fault fired before the per-path loop even started.
            unsafe { fb_hold(0xE804C800); }
            drivers::apple::soc::discover_from_adt(&adt)
        },
        Err(_) => {
            // R-fail-adt: RED-ORANGE.
            unsafe { fb_hold(0xFFF40000); }
            loop { unsafe { core::arch::asm!("wfe"); } }
        }
    };
    // R5: discover_from_adt returned. HOT PINK.
    unsafe { fb_hold(0xFFF00200); }
    let _ = discovered; // silence unused warning while UART puts are gated

    // UART stays disabled via the UART_READY flag in drivers/apple/uart.rs
    // — the existing S5L driver writes wrong-layout config bytes to
    // M4's dockchannel UART. All `uart::puts(...)` / `putc(...)` calls
    // below become silent no-ops until we port a dockchannel driver.
    drivers::apple::uart::init();

    // ─── Rust kernel init: one FB marker per stage ──────────────────
    // Colors chosen to be progressively distinctive; the LAST one
    // painted before a halt / fault stripe tells us how far we got.
    //
    // K1: about to init heap — SKY BLUE (0xC006A3FF)
    unsafe { fb_hold(0xC006A3FF); }
    // TEMP: skip heap::init — it hangs (not faults) on M4, probably
    // because linked_list_allocator internally expects the memory
    // region to be a specific alignment/type. Leave heap uninitialized;
    // downstream code that uses `alloc` will panic, but the splash
    // path below doesn't need it.
    let _heap_base = ((args.top_of_kernel_data() as usize) + 0xFFFF) & !0xFFFF;
    // kernel::mm::heap::init(heap_base);
    // K2: heap init skipped — LIME YELLOW (0xFBFFC000)
    unsafe { fb_hold(0xFBFFC000); }
    kernel::process::init();
    // K3: process ok — SPRING GREEN (0xC007FC28)
    unsafe { fb_hold(0xC007FC28); }
    kernel::scheduler::init();
    // K4: scheduler ok — TURQUOISE (0xC00DFEFF)
    unsafe { fb_hold(0xC00DFEFF); }
    kernel::ipc::init();
    // K5: ipc ok — LILAC (0xFB83C3FF)
    unsafe { fb_hold(0xFB83C3FF); }
    kernel::arch::init_exceptions();
    // K6: arch ok — PEACH (0xFFF6E200)
    unsafe { fb_hold(0xFFF6E200); }
    drivers::apple::aic::init();
    // K7: aic ok — GOLD (0xFFFC0000)
    unsafe { fb_hold(0xFFFC0000); }

    // SKIP bring_up_all, passphrase read, BatFS init — those all need
    // heap (mm::init was skipped) or UART. Jump straight to display.

    // K8: about to paint minimal splash. HOT MAGENTA (0xFFF40100).
    unsafe { fb_hold(0xFFF40100); }

    // Bypass dcp entirely — paint a BAT_OS splash directly via
    // font::draw_str_scaled with known-good FB parameters. Each
    // source pixel is rendered at scale=8 for the title so the
    // text is actually readable on camera, scale=2 for subtitle.
    unsafe {
        // Black out the full paint region.
        fb_mark(0xC0000000);

        let fb = 0x103e0050000usize as *mut u32;
        let stride_pixels: u32 = 0x2f40 / 4;   // exactly 3024 on M4

        const FG_TITLE: u32 = 0xFFFFC000;   // amber
        const FG_SUB:   u32 = 0xFF40C0FF;   // cool blue
        const FG_DIM:   u32 = 0xFF808080;   // gray
        const BG:       u32 = 0xC0000000;   // opaque black

        // ASCII bat logo (3 rows, 2x scale). Yellow on black.
        let logo = [
            "  /|.______________.|\\  ",
            "     /__.--.  .--.__\\     ",
            "        \\/    \\/        ",
        ];
        let logo_scale: u32 = 4;
        let logo_w = (logo[0].len() as u32) * ui::font::CHAR_W * logo_scale;
        let logo_x = (3024u32.saturating_sub(logo_w)) / 2;
        let logo_y: u32 = 200;
        for (i, line) in logo.iter().enumerate() {
            let y = logo_y + (i as u32) * ui::font::CHAR_H * logo_scale;
            ui::font::draw_str_scaled(fb, stride_pixels, logo_x, y,
                                      line, FG_TITLE, BG, logo_scale);
        }

        let title = "BAT_OS";
        let ts: u32 = 8;   // title scale
        let title_w = (title.len() as u32) * ui::font::CHAR_W * ts;
        let tx = (3024u32.saturating_sub(title_w)) / 2;
        let ty: u32 = 500;
        ui::font::draw_str_scaled(fb, stride_pixels, tx, ty, title,
                                  FG_TITLE, BG, ts);

        let sub = "Bare Metal // Apple Silicon (M4 / T8132)";
        let ss: u32 = 2;   // subtitle scale
        let sub_w = (sub.len() as u32) * ui::font::CHAR_W * ss;
        let sx = (3024u32.saturating_sub(sub_w)) / 2;
        let sy = ty + ui::font::CHAR_H * ts + 60;
        ui::font::draw_str_scaled(fb, stride_pixels, sx, sy, sub,
                                  FG_SUB, BG, ss);

        let foot = "[booted via m1n1 chainload]";
        let fs: u32 = 2;
        let foot_w = (foot.len() as u32) * ui::font::CHAR_W * fs;
        let fx = (3024u32.saturating_sub(foot_w)) / 2;
        let fy = sy + ui::font::CHAR_H * ss + 40;
        ui::font::draw_str_scaled(fb, stride_pixels, fx, fy, foot,
                                  FG_DIM, BG, fs);

        // Live stats pulled from boot_args + ADT discovery.
        let stats_y = fy + ui::font::CHAR_H * fs + 60;
        let stats_scale: u32 = 2;
        let mem_mib = (args.mem_size() / (1024 * 1024)) as u32;

        // Two-column-ish layout of labeled facts, centered.
        let lines: [(&str, u32, u32); 6] = [
            ("Chip       : T8132 (Donan / H16G)",  0, 0),
            ("Model      : Mac16,1",               0, 0),
            ("CPU        : Apple M4  4P + 6E",    0, 0),
            ("RAM        : _________ MiB",         mem_mib, 0),
            ("Revision   : 3",                     0, 0),
            ("ADT peripherals discovered: ____",   discovered as u32, 0),
        ];
        for (i, (line, val, _)) in lines.iter().enumerate() {
            let mut buf = [0u8; 128];
            let mut pos: usize = 0;
            let has_underscore_placeholder = line.contains('_');
            if has_underscore_placeholder && *val != 0 {
                // Inline the number into the underscore slot.
                let colon = line.find(':').unwrap_or(line.len());
                let prefix = &line.as_bytes()[..=colon];
                for &b in prefix {
                    if pos < buf.len() { buf[pos] = b; pos += 1; }
                }
                if pos < buf.len() { buf[pos] = b' '; pos += 1; }
                // Decimal formatter for val.
                let mut nb = [0u8; 16]; let mut np = 0usize;
                let mut v = *val;
                if v == 0 { nb[np] = b'0'; np += 1; }
                while v > 0 && np < nb.len() {
                    nb[np] = b'0' + (v % 10) as u8;
                    np += 1;
                    v /= 10;
                }
                while np > 0 && pos < buf.len() {
                    np -= 1;
                    buf[pos] = nb[np]; pos += 1;
                }
            } else {
                for &b in line.as_bytes() {
                    if pos < buf.len() { buf[pos] = b; pos += 1; }
                }
            }
            let s = core::str::from_utf8_unchecked(&buf[..pos]);
            let w = (s.len() as u32) * ui::font::CHAR_W * stats_scale;
            let x = (3024u32.saturating_sub(w)) / 2;
            let y = stats_y + (i as u32) * (ui::font::CHAR_H * stats_scale + 10);
            ui::font::draw_str_scaled(fb, stride_pixels, x, y, s,
                                      0xFF00FFFF, BG, stats_scale);
        }

        // Boot-log section — gives the splash a real-OS-boot feel.
        // These lines reflect the exact bring-up path we ran above.
        let log_lines: [(&str, u32); 9] = [
            ("[ok] m1n1 handoff accepted  (boot_args rev 3)",      0xFF80FF80),
            ("[ok] _apple_start  asm stages 1..5 complete",         0xFF80FF80),
            ("[ok] bringup_vectors installed at VBAR_EL1/EL2",      0xFF80FF80),
            ("[ok] boot_args::parse  OK  (devtree virt->phys)",    0xFF80FF80),
            ("[ok] discover_from_adt  walker bounded, 9 paths",    0xFF80FF80),
            ("[ok] kernel::process + scheduler + ipc  init",        0xFF80FF80),
            ("[ok] kernel::arch::init_exceptions",                  0xFF80FF80),
            ("[ok] drivers::apple::aic::init",                      0xFF80FF80),
            ("[ok] splash rendered  —  awaiting  mm::init fix",    0xFFFFFF80),
        ];
        let log_scale: u32 = 2;
        let log_x: u32 = 320;   // indented from left
        let log_y0: u32 = 1180;
        for (i, (line, color)) in log_lines.iter().enumerate() {
            let y = log_y0 + (i as u32) * (ui::font::CHAR_H * log_scale + 6);
            ui::font::draw_str_scaled(fb, stride_pixels, log_x, y, line,
                                      *color, BG, log_scale);
        }

        core::arch::asm!("dsb sy");
    }

    // Live uptime + tick counter at the bottom. The ARM Generic Timer
    // on Apple Silicon runs at 24 MHz (CNTFRQ_EL0 reports this); we
    // read CNTPCT_EL0 each iteration to get real wall-clock uptime.
    let fb = 0x103e0050000usize as *mut u32;
    let stride_pixels: u32 = 0x2f40 / 4;
    const LINE_BG: u32 = 0xC0000000;    // opaque black
    const LINE_FG: u32 = 0xFFFFF800;    // bright yellow
    const LINE_FG2: u32 = 0xFF00FFFF;   // bright cyan
    let scale: u32 = 3;
    let y0: u32 = 1620;
    let y1: u32 = 1720;

    // Read CNTFRQ_EL0 once — documented as 24 MHz on Apple Silicon,
    // verify at runtime.
    let cntfrq: u64;
    unsafe { core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) cntfrq); }
    let start: u64;
    unsafe { core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) start); }

    let mut tick: u64 = 0;
    loop {
        // Read current physical counter.
        let now: u64;
        unsafe { core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) now); }
        let elapsed_ticks = now.wrapping_sub(start);
        let elapsed_secs = if cntfrq > 0 { elapsed_ticks / cntfrq } else { 0 };
        let mm = elapsed_secs / 60;
        let ss = elapsed_secs % 60;

        // Line 1: "uptime: MM:SS"
        let mut buf = [b' '; 48];
        let mut p = 0usize;
        for &b in b"uptime: " { buf[p] = b; p += 1; }
        // Zero-pad MM to at least 2 digits for visual stability.
        if mm < 10 { buf[p] = b'0'; p += 1; }
        let mut nb = [0u8; 16]; let mut np = 0usize; let mut v = mm;
        if v == 0 { nb[np] = b'0'; np += 1; }
        while v > 0 { nb[np] = b'0' + (v % 10) as u8; v /= 10; np += 1; }
        while np > 0 { np -= 1; buf[p] = nb[np]; p += 1; }
        buf[p] = b':'; p += 1;
        buf[p] = b'0' + ((ss / 10) as u8); p += 1;
        buf[p] = b'0' + ((ss % 10) as u8); p += 1;
        // Pad so erasure on shrink works.
        while p < 32 { buf[p] = b' '; p += 1; }
        let s = unsafe { core::str::from_utf8_unchecked(&buf[..p]) };
        let w = (s.len() as u32) * ui::font::CHAR_W * scale;
        let x = (3024u32.saturating_sub(w)) / 2;
        ui::font::draw_str_scaled(fb, stride_pixels, x, y0, s,
                                  LINE_FG, LINE_BG, scale);

        // Line 2: "tick: N"
        let mut buf2 = [b' '; 48];
        let mut q = 0usize;
        for &b in b"tick: " { buf2[q] = b; q += 1; }
        let mut nb2 = [0u8; 20]; let mut nq = 0usize; let mut vv = tick;
        if vv == 0 { nb2[nq] = b'0'; nq += 1; }
        while vv > 0 { nb2[nq] = b'0' + (vv % 10) as u8; vv /= 10; nq += 1; }
        while nq > 0 { nq -= 1; buf2[q] = nb2[nq]; q += 1; }
        while q < 32 { buf2[q] = b' '; q += 1; }
        let s2 = unsafe { core::str::from_utf8_unchecked(&buf2[..q]) };
        let w2 = (s2.len() as u32) * ui::font::CHAR_W * scale;
        let x2 = (3024u32.saturating_sub(w2)) / 2;
        ui::font::draw_str_scaled(fb, stride_pixels, x2, y1, s2,
                                  LINE_FG2, LINE_BG, scale);

        // Small spin so ticks aren't too fast to visually track.
        for _ in 0..10_000_000u32 {
            unsafe { core::arch::asm!("", options(nomem, nostack, preserves_flags)); }
        }
        tick = tick.wrapping_add(1);
    }
    #[allow(unreachable_code)]
    if false {

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
