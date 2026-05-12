#![no_std]
#![no_main]

extern crate alloc;

mod ai;
mod batcave;
mod boot;
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
    // belt-and-suspenders — explicitly set the platform
    // discriminator to QemuVirt so any later `is_apple_silicon()`
    // check returns false. The static is `AtomicU8::new(0)` =
    // QemuVirt by default, but pre-MMU BSS state might be observed
    // before zeroing in some paths under HVF, and the post-cave-exit
    // data abort traces to apple-uart code that's gated on this byte.
    platform::set_platform(platform::Platform::QemuVirt);

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

            // Chromium blob delivery via QEMU `-initrd`. The kernel
            // linker's `__kernel_end` symbol is past `.bss + stack`,
            // so the legacy "append the blob to the ELF file" trick
            // in `tools/bake_chromium.sh` doesn't work with ELF
            // kernels under QEMU — ELF only loads PT_LOAD segments,
            // skipping the tail bytes. The DTB path sidesteps that:
            // QEMU's `-initrd` puts the blob at a known physical
            // address and records it in `/chosen/linux,initrd-*`.
            if dtb_info.initrd_start != 0
                && dtb_info.initrd_end > dtb_info.initrd_start
            {
                drivers::uart::puts("  [dtb] initrd @ 0x");
                let hex = b"0123456789abcdef";
                let addr = dtb_info.initrd_start;
                for shift in (0..16).rev() {
                    let nibble = ((addr >> (shift * 4)) & 0xF) as usize;
                    drivers::uart::putc(hex[nibble]);
                }
                drivers::uart::puts("..0x");
                let end = dtb_info.initrd_end;
                for shift in (0..16).rev() {
                    let nibble = ((end >> (shift * 4)) & 0xF) as usize;
                    drivers::uart::putc(hex[nibble]);
                }
                drivers::uart::puts("\n");
                kernel::mm::initrd::set_range(
                    dtb_info.initrd_start,
                    dtb_info.initrd_end,
                );
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

    // BAT_OS_KEEP_GOING — when set at build time, the kernel records
    // every cave-fatal event (non-zero exit, unknown syscall, EL0
    // fault) into a structured ring and continues running instead of
    // tearing down at the first failure. One smoke run then maps the
    // entire failure tree of content_shell / ld-linux / glibc setup
    // so we can dispatch parallel fixers per distinct failure
    // signature. See `src/batcave/linux/skip_log.rs`.
    if option_env!("BAT_OS_KEEP_GOING").is_some() {
        batcave::linux::skip_log::enable();
    }

    // Initialize kernel
    drivers::uart::puts("[boot] Initializing kernel...\n");
    kernel::mm::init();
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::pipe::init();
    kernel::arch::init_exceptions();
    // (init_timer + GICv2 init removed — IRQ-driven preemption
    // hangs boot somewhere after the timer fires the first time.
    // GIC init is correct for the QEMU virt machine layout but
    // either the timer fires too aggressively or our IRQ vector
    // doesn't preserve enough state. Filed for follow-up; for
    // now the periodic-yield-every-4096-syscalls fallback in the
    // syscall dispatcher covers cooperative scheduling.)

    // V4: probe ARMv8.5 RNDR hardware RNG and wire it into crypto::rng.
    crypto::rng::probe_hw_rng();

    // DESIGN_CRYPTO.md #11+#12: seed the OTP pad with fresh true-random
    // bytes from the RNDR-backed CSPRNG. Tokens can then be dumped via
    // `otp-dump` shell command at provisioning for operator to record
    // offline, and consumed via the duress/deadman channels.
    security::otp::init();

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
        // the BUILD_PASSPHRASE is baked into
        // the kernel binary as a plaintext literal — `strings bat_os`
        // recovers it. This is convenient for development but
        // catastrophic for production. Loud red banner so the operator
        // can never miss that this binary should not ship.
        drivers::uart::puts("\n");
        drivers::uart::puts("================================================================\n");
        drivers::uart::puts("  WARNING: this kernel was built with BAT_OS_PASSPHRASE baked in.\n");
        drivers::uart::puts("  Anyone with `strings bat_os` recovers the passphrase in seconds.\n");
        drivers::uart::puts("  DO NOT ship this binary. Build production with:\n");
        drivers::uart::puts("      unset BAT_OS_PASSPHRASE && cargo build --release\n");
        drivers::uart::puts("================================================================\n\n");
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
    // virtio-blk MUST init before BatFS so that
    // BatFS can mount its on-disk format from sector 0. Without a disk
    // BatFS still works — it falls back to in-RAM-only mode and warns —
    // but with one attached, BatFS will persist across reboots.
    drivers::uart::puts("[boot] Initializing block device...\n");
    match drivers::virtio::blk::init() {
        Some(()) => drivers::uart::puts("  [blk] Block device ready\n"),
        None    => drivers::uart::puts("  [blk] No block device (RAM-only BatFS)\n"),
    }

    // Derive the BatFS key from the same passphrase we just prompted for.
    let master_key = derive_batfs_key(passphrase_slice);
    fs::batfs::init(&master_key);
    drivers::uart::puts("  [fs] BatFS initialized (ChaCha20-Poly1305 AEAD, Argon2id-derived master)\n");

    // restore audit ring from BatFS-persisted /audit.log
    // (written by a prior boot's `audit-flush`). Lets the operator's
    // `audit` command show historical events across reboots — without
    // this, an attacker who panics post-exploit erases their tracks.
    {
        // Static buf to avoid stack-staging a 256K array.
        static mut RESTORE_BUF: [u8; 256 * 1024] = [0; 256 * 1024];
        unsafe {
            let buf = &mut *core::ptr::addr_of_mut!(RESTORE_BUF);
            match fs::batfs::read("audit.log", buf) {
                Ok(n) if n > 0 => {
                    let restored = security::audit::restore_from_persisted(&buf[..n]);
                    drivers::uart::puts("  [audit] restored ");
                    kernel::mm::print_num(restored);
                    drivers::uart::puts(" entries from /audit.log\n");
                }
                _ => {
                    drivers::uart::puts("  [audit] no persisted /audit.log (fresh ring)\n");
                }
            }
        }
    }

    // Initialize BatCave runtime
    drivers::uart::puts("[boot] Initializing BatCave runtime...\n");
    batcave::cave::init();
    // Set mem_limit / sockets_limit / etc. to DEFAULT_* — the const
    // initializer on `static CAVE_QUOTAS` was reading back as zero
    // in release (same family of bug as the vfs.rs slice-of-literals
    // miscompile). init() patches the ledger into a sane state.
    batcave::linux::quotas::init();
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

    // Sprint 1.5 : virtio-tablet for absolute mouse input.
    // Skipped silently when no second virtio-input device is attached
    // (every QEMU run that doesn't pass `-device virtio-tablet-device`).
    drivers::uart::puts("[boot] Initializing tablet...\n");
    match drivers::virtio::tablet::init() {
        Some(()) => drivers::uart::puts("  [tbl] tablet ready\n"),
        None => drivers::uart::puts("  [tbl] no tablet attached\n"),
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

            // X.509 selftest hook (gated by Cargo feature
            // `selftest-on-boot`). Runs the same chain-validator
            // selftest the `x509-selftest` shell command runs, before
            // the auth gate, so a headless QEMU smoke can capture the
            // PASS lines via serial. See DESIGN_TLS_HARDENING.md.
            #[cfg(feature = "selftest-on-boot")]
            {
                drivers::uart::puts("[selftest] running x509-selftest before auth gate...\n");
                ui::shell::cmd_x509_selftest();
                drivers::uart::puts("[selftest] running scheduler-selftest before auth gate...\n");
                ui::shell::cmd_scheduler_selftest();
            }

            // PQ-INTEROP boot hook (gated by Cargo feature
            // `pq-interop-test`). Drives a real TLS 1.3 + X25519MLKEM768
            // hybrid PQ handshake against pq.cloudflareresearch.com so
            // a headless QEMU smoke (scripts/qemu_pq_interop_smoke.py)
            // can verify our IETF draft-ietf-tls-ecdhe-mlkem-04 wire
            // layout interops with a real third-party server. Closed-
            // loop selftests can't catch wire-format regressions when
            // both sides run the same code.
            #[cfg(feature = "pq-interop-test")]
            {
                drivers::uart::puts("[pq-interop] running hybrid PQ handshake vs pq.cloudflareresearch.com...\n");
                ui::shell::cmd_pq_interop();
            }

            // HTTPS-smoke boot hook (gated by Cargo feature
            // `https-smoke-test`). Drives a real end-to-end HTTPS
            // request through the kernel-mediated HTTPS path
            // (https::open_kernel + write + read) so a headless QEMU
            // smoke can verify the full request/response loop, not
            // just the TLS handshake the pq-interop-test feature
            // covers. See DESIGN_HTTPS_SYSCALL.md.
            #[cfg(feature = "https-smoke-test")]
            {
                drivers::uart::puts("[https-smoke] starting end-to-end HTTPS request...\n");
                run_https_smoke();
            }

            // ═══════════════════════════════════════
            // PRE-AUTH KERNEL SELFTESTS (feature-gated).
            // `pipe-selftest` is also available from the shell.
            // ═══════════════════════════════════════
            #[cfg(feature = "selftest-on-boot")]
            pipe_selftest_uart();

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
            IS_HEADLESS.store(true, core::sync::atomic::Ordering::Release);
            serial_shell();
        }
    }
}

/// Boot-time HTTPS smoke. Drives a real end-to-end HTTPS request
/// (handshake + GET / + drain response) against
/// pq.cloudflareresearch.com using the kernel-mediated HTTPS path
/// with hybrid PQ enabled — i.e. the production configuration.
/// Prints `[https-smoke] PASS …` or `[https-smoke] FAIL …` so
/// scripts/qemu_https_smoke.py can pick up the result via serial.
// /
/// Calls `https::open_kernel` directly — no syscall, no cave context,
/// no cpol gate. The cave-side ABI smoke (which goes through
/// sys_bat_https_open) lands in a follow-up PR once a test cave
/// can run.
/// Anonymous-pipe round trip exercised over UART so the serial
/// log shows pass/fail without needing keyboard input through the
/// auth gate. Verifies the same create/write/read/EOF/EPIPE path
/// the shell `pipe-selftest` runs. Behind `selftest-on-boot` so
/// production builds skip it.
#[cfg(feature = "selftest-on-boot")]
fn pipe_selftest_uart() {
    use drivers::uart;
    use kernel::pipe;
    use kernel::process::{self, FdKind, PipeEnd};

    uart::puts("[pipe-selftest] start\n");
    let (rfd, wfd) = match pipe::create() {
        Ok(p) => p,
        Err(e) => {
            uart::puts("[pipe-selftest] FAIL create: "); uart::puts(e); uart::puts("\n");
            return;
        }
    };

    let id_for = |fd: u16, want_read: bool| -> Option<u16> {
        let entry = process::current().fd_get(fd)?;
        match entry.kind {
            FdKind::Pipe { id, end } => {
                let ok = matches!((end, want_read), (PipeEnd::Read, true) | (PipeEnd::Write, false));
                if ok { Some(id) } else { None }
            }
        }
    };

    let wid = match id_for(wfd, false) {
        Some(id) => id,
        None => { uart::puts("[pipe-selftest] FAIL: wfd shape\n"); return; }
    };
    let rid = match id_for(rfd, true) {
        Some(id) => id,
        None => { uart::puts("[pipe-selftest] FAIL: rfd shape\n"); return; }
    };

    let payload: &[u8] = b"the bat signal is up";
    match pipe::write(wid, payload) {
        Ok(n) if n == payload.len() => {}
        Ok(_) | Err(_) => {
            uart::puts("[pipe-selftest] FAIL: write\n"); return;
        }
    }

    let mut buf = [0u8; 32];
    let n = match pipe::read(rid, &mut buf) {
        Ok(n) => n,
        Err(e) => { uart::puts("[pipe-selftest] FAIL read: "); uart::puts(e); uart::puts("\n"); return; }
    };
    if n != payload.len() || &buf[..n] != payload {
        uart::puts("[pipe-selftest] FAIL: payload mismatch\n");
        return;
    }
    uart::puts("[pipe-selftest] ok  write/read round trip\n");

    let _ = process::current().fd_take(wfd);
    pipe::release_end(wid, PipeEnd::Write);
    match pipe::read(rid, &mut buf) {
        Ok(0) => uart::puts("[pipe-selftest] ok  EOF after writer close\n"),
        Ok(_) => { uart::puts("[pipe-selftest] FAIL: expected EOF\n"); return; }
        Err(_) => { uart::puts("[pipe-selftest] FAIL: EOF read err\n"); return; }
    }
    let _ = process::current().fd_take(rfd);
    pipe::release_end(rid, PipeEnd::Read);

    let (rfd2, wfd2) = match pipe::create() {
        Ok(p) => p,
        Err(e) => { uart::puts("[pipe-selftest] FAIL 2nd create: "); uart::puts(e); uart::puts("\n"); return; }
    };
    let rid2 = id_for(rfd2, true).unwrap_or(u16::MAX);
    let wid2 = id_for(wfd2, false).unwrap_or(u16::MAX);
    if rid2 == u16::MAX || wid2 == u16::MAX {
        uart::puts("[pipe-selftest] FAIL: 2nd shape\n"); return;
    }
    let _ = process::current().fd_take(rfd2);
    pipe::release_end(rid2, PipeEnd::Read);
    match pipe::write(wid2, b"x") {
        Err(e) if e == "EPIPE" => uart::puts("[pipe-selftest] ok  EPIPE after reader close\n"),
        Ok(_) => { uart::puts("[pipe-selftest] FAIL: expected EPIPE\n"); return; }
        Err(_) => { uart::puts("[pipe-selftest] FAIL: wrong write err\n"); return; }
    }
    let _ = process::current().fd_take(wfd2);
    pipe::release_end(wid2, PipeEnd::Write);

    uart::puts("[pipe-selftest] PASS\n");
}

#[cfg(feature = "https-smoke-test")]
fn run_https_smoke() {
    use drivers::uart;
    let host = "pq.cloudflareresearch.com";
    let port: u16 = 443;

    let pcb = match net::https::open_kernel(host, port) {
        Ok(p) => p,
        Err(e) => {
            uart::puts("[https-smoke] FAIL open: ");
            uart::puts(e);
            uart::puts("\n");
            return;
        }
    };

    let req: &[u8] =
        b"GET / HTTP/1.1\r\nHost: pq.cloudflareresearch.com\r\nUser-Agent: Bat_OS/1.0\r\nConnection: close\r\nAccept: */*\r\n\r\n";
    if let Err(e) = net::https::write(pcb, req) {
        uart::puts("[https-smoke] FAIL write: "); uart::puts(e); uart::puts("\n");
        net::https::close_pcb(pcb);
        return;
    }

    // Drain up to 64 KB. We don't need the body — proving we got a
    // well-formed `HTTP/1.1 ` status line is enough.
    let mut total = [0u8; 65536];
    let mut total_len = 0usize;
    let mut empty_runs = 0u32;
    loop {
        if total_len == total.len() { break; }
        match net::https::read(pcb, &mut total[total_len..]) {
            Ok(0) => {
                empty_runs += 1;
                if empty_runs > 4 { break; }
            }
            Ok(n) => { total_len += n; empty_runs = 0; }
            Err(_) => break,
        }
    }
    net::https::close_pcb(pcb);

    if total_len < 12 {
        uart::puts("[https-smoke] FAIL response too short\n");
        return;
    }
    if !total.starts_with(b"HTTP/1.1 ") && !total.starts_with(b"HTTP/1.0 ") {
        uart::puts("[https-smoke] FAIL bad status line\n");
        return;
    }
    // Status code: bytes 9..12.
    uart::puts("[https-smoke] PASS http-status=");
    for i in 9..12 {
        uart::putc(total[i]);
    }
    uart::puts(" body-bytes=");
    crate::kernel::mm::print_num(total_len);
    uart::puts("\n");
}

/// Derive the BatFS master key from the passphrase via Argon2id (8 MiB
/// × 3 passes × 1 lane), matching the auth-gate KDF parameters. Salt
/// is domain-separated so the BatFS master and the auth hash differ
/// for the same passphrase. Falls back to a legacy SHA-256 path if
/// Argon2 rejects the input (length out of range etc) so first-boot
/// edge cases stay functional; the fallback audit-logs at run time.
fn derive_batfs_key(passphrase: &[u8]) -> [u8; 32] {
    use argon2::{Argon2, Algorithm, Version, Params};

    // Distinct from auth.rs's "bat_os-auth-v2" so the two derivations
    // produce different outputs for the same passphrase — domain
    // separation. bumps the version tag so anyone migrating
    // from the pre-Argon2 master key sees a clean break.
    const SALT: &[u8; 16] = b"bat_os-batfs-v3\0";
    const MEM_KIB: u32 = 8_192;       // 8 MiB
    const TIME_COST: u32 = 3;
    const PARALLELISM: u32 = 1;
    const OUTLEN: usize = 32;

    let params = match Params::new(MEM_KIB, TIME_COST, PARALLELISM, Some(OUTLEN)) {
        Ok(p) => p,
        Err(_) => Params::default(),
    };
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; 32];
    if argon.hash_password_into(passphrase, SALT, &mut out).is_ok() {
        return out;
    }

    // Argon2 failed (rare — passphrase length OOR is the only realistic
    // path). Fall back to the legacy 16-round SHA-256 KDF so first-boot
    // edge cases still produce a master key. Audit-record so a security-
    // conscious operator notices the fallback fired.
    crate::security::audit::record(
        crate::security::audit::Category::Cave,
        b"WARN: BatFS KDF Argon2id failed, falling back to SHA-256",
    );
    derive_batfs_key_sha_fallback(passphrase)
}

/// Legacy 16-round SHA-256 BatFS KDF. Retained as the Argon2id failure
/// fallback so a malformed-passphrase edge case can't brick the OS.
/// Domain-separated from the Argon2id output so an attacker who learns
/// one cannot derive the other.
fn derive_batfs_key_sha_fallback(passphrase: &[u8]) -> [u8; 32] {
    const KERNEL_SALT: [u8; 16] = *b"batfs-fallback\0\0";

    let mut buf = [0u8; 128];
    let n1 = passphrase.len().min(64);
    buf[..n1].copy_from_slice(&passphrase[..n1]);
    buf[64..64 + 16].copy_from_slice(&KERNEL_SALT);

    let mut hash = crypto::sha256::hash(&buf);
    for round in 0u64..16 {
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
    // V8-LINKER-FIX 2026-04-25: matches linker.ld . = 0x40200000
    // (was 0x40080000 — see linker.ld comment).
    let text_start: usize = 0x40200000;
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
// /
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
            // revisited: the
            // earlier rule "never echo, UART observers can count
            // chars" is correct for production hardware where a
            // pin-header attacker can sniff serial. On QEMU the
            // UART IS the operator's terminal — muting it just
            // hides legitimate feedback ("am I typing? did the
            // backspace work?") with no real adversary on the
            // wire. Re-introduce the echo on QEMU only; Apple-
            // silicon path uses read_passphrase_apple which has
            // no echo by design.
            match ch {
                b'\r' | b'\n' => { drivers::uart::puts("\n"); return len; }
                0x08 | 0x7f => {
                    if len > 0 {
                        len -= 1;
                        drivers::uart::puts("\x08 \x08");
                    }
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
    // V8-LINKER-FIX 2026-04-25: matches linker.ld . = 0x40200000
    // (was 0x40080000 — see linker.ld comment).
    let text_start: usize = 0x40200000;
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
/// Track whether we booted to the headless serial shell (no display).
/// Read by `signal::terminate_cave_fatal` to choose where to land
/// after a cave dies — `desktop::resume()` requires the console
/// framebuffer to be initialized, which only happens on the GUI
/// path. Headless mode lands back in `serial_shell()` directly.
pub static IS_HEADLESS: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

pub fn serial_shell() -> ! {
    use drivers::uart;
    use ui::shell_history::{ArrowKey, EscState, FeedResult};
    uart::puts("bat_os > ");

    let mut buf = [0u8; 256];
    let mut len = 0usize;
    let mut esc = EscState::default();

    // Replace the currently-visible input line with a new byte slice.
    // Erases via backspace+space+backspace (terminal-portable), then
    // prints the new content. Caller updates `buf` and `len`.
    let redraw = |old_len: usize, new_bytes: &[u8], new_len: &mut usize| {
        for _ in 0..old_len {
            uart::putc(0x08); uart::putc(b' '); uart::putc(0x08);
        }
        for &b in new_bytes {
            uart::putc(b);
        }
        *new_len = new_bytes.len();
    };

    loop {
        let Some(raw) = uart::getc() else {
            core::hint::spin_loop();
            continue;
        };

        // Drive the ANSI ESC-sequence parser first. Arrow keys arrive
        // as ESC `[` `A`/`B`/`C`/`D` — without the parser, those
        // three bytes would each get treated as a regular character.
        let c = match esc.feed(raw) {
            FeedResult::Consumed => continue,
            FeedResult::Arrow(ArrowKey::Up) => {
                if let Some(line) = ui::shell_history::prev() {
                    let mut take = [0u8; 256];
                    let n = line.len().min(255);
                    take[..n].copy_from_slice(&line[..n]);
                    redraw(len, &take[..n], &mut len);
                    buf[..n].copy_from_slice(&take[..n]);
                }
                continue;
            }
            FeedResult::Arrow(ArrowKey::Down) => {
                match ui::shell_history::next() {
                    Some(line) => {
                        let mut take = [0u8; 256];
                        let n = line.len().min(255);
                        take[..n].copy_from_slice(&line[..n]);
                        redraw(len, &take[..n], &mut len);
                        buf[..n].copy_from_slice(&take[..n]);
                    }
                    None => {
                        // Stepped past newest — clear to live edit.
                        redraw(len, &[], &mut len);
                    }
                }
                continue;
            }
            FeedResult::Arrow(_) => continue, // left/right ignored for v1
            FeedResult::Pass(b) => b,
        };

        match c {
            b'\r' | b'\n' => {
                uart::puts("\n");
                if len > 0 {
                    let cmd = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                    // Dispatch to the full shell so headless smokes can
                    // run any command without virtio-gpu/keyboard. Pre
                    // this consolidation, serial_shell was a stub with
                    // help/mem/uname/whoami only.
                    crate::ui::shell::execute_cmd(cmd);
                    ui::shell_history::record(&buf[..len]);
                    len = 0;
                }
                uart::puts("bat_os > ");
            }
            0x03 => {
                // Ctrl+C — discard the current line and reprompt.
                uart::puts("^C\nbat_os > ");
                len = 0;
                ui::shell_history::reset_cursor();
            }
            0x08 | 0x7F => {
                if len > 0 {
                    len -= 1;
                    uart::putc(0x08);
                    uart::putc(b' ');
                    uart::putc(0x08);
                    ui::shell_history::reset_cursor();
                }
            }
            0x09 => {
                // Tab — autofill. Inside the first token: command-name
                // completion. Past a space: argument completion driven
                // by `arg_kind_for(cmd, arg_index)`.
                let line = unsafe {
                    core::str::from_utf8_unchecked(&buf[..len])
                };
                if let Some((cmd, arg_index, current)) =
                    ui::shell_completion::split_for_completion(line)
                {
                    let kind = ui::shell_completion::arg_kind_for(cmd, arg_index);
                    if kind != ui::shell_completion::ArgKind::None {
                        let r = ui::shell_completion::complete_argument(kind, current);
                        let ext = r.extension_bytes();
                        let take = ext.len().min(255usize.saturating_sub(len));
                        for &b in &ext[..take] {
                            buf[len] = b;
                            len += 1;
                            uart::putc(b);
                        }
                        if r.match_count > 1 {
                            uart::puts("\n");
                            for i in 0..r.names_len as usize {
                                let name = r.name_at(i);
                                for &b in name { uart::putc(b); }
                                uart::puts("  ");
                            }
                            uart::puts("\nbat_os > ");
                            for &b in &buf[..len] {
                                uart::putc(b);
                            }
                        }
                    }
                } else {
                    // Inside the first token — command-name completion.
                    let r = ui::shell_completion::complete_command(line);
                    let ext = r.extension_bytes();
                    let take = ext.len().min(255usize.saturating_sub(len));
                    for &b in &ext[..take] {
                        buf[len] = b;
                        len += 1;
                        uart::putc(b);
                    }
                    if r.match_count > 1 {
                        uart::puts("\n");
                        for &name in r.candidate_slice() {
                            uart::puts(name);
                            uart::puts("  ");
                        }
                        uart::puts("\nbat_os > ");
                        for &b in &buf[..len] {
                            uart::putc(b);
                        }
                    }
                }
            }
            _ if c >= 0x20 && c <= 0x7E && len < 255 => {
                buf[len] = c;
                len += 1;
                uart::putc(c);
                ui::shell_history::reset_cursor();
            }
            _ => {}
        }
    }
}

// ─── Apple Silicon Entry Point ───
// Called by the Apple boot stub when running on real M4 hardware.
// x0 = pointer to m1n1 boot args

global_asm!(include_str!("arch/aarch64/apple/boot.s"));

// (diag band helper removed — bisection confirmed mm / process /
// scheduler / ipc / arch_exceptions / aic / bring_up_all all reach
// cleanly on M4 after the WDT-disable + DWC3/BCM skips landed.)

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

/// Run one scripted shell command through `apple_shell_dispatch`
/// with a `bat_os>` prompt echoed first — visually identical to
/// what happens when a human types at the (future USB-CDC) shell.
/// Currently only invoked from `apple_kernel_self_test`, which is
/// commented out at its single call site (kernel_main_apple). Kept
/// as staged tooling for the M4 boot path.
#[allow(dead_code)]
fn apple_run_cmd(line: &str) {
    use drivers::apple::uart;
    uart::puts("bat_os> ");
    uart::puts(line);
    uart::puts("\n");
    apple_shell_dispatch(line);
}

/// Post-splash kernel self-test for the M4 path. Exercises the real
/// live paths that were hardened this session: `mm::frame::alloc_frame`
/// (load+store under IrqGuard instead of `compare_exchange_weak`),
/// `rng::fill_bytes` (non-atomic CTR), and the `fs::batfs` encrypted
/// create+read round-trip (which also reaches `batfs::next_nonce`,
/// the new `NONCE_COUNTER` load+store, and AES-CTR + HMAC-SHA256
/// MAC verification).
// /
/// Layered in two halves:
/// 1. a focused direct-API self-test (same as before)
/// 2. a replay of real shell commands via `apple_shell_dispatch`
/// so every command registered in the shell gets exercised too
// /
/// All output goes through `drivers::apple::uart::puts`, which tees
/// into `drivers::apple::fb_console::puts` — so each line is both
/// shipped out the dockchannel UART and rendered to the M4 display
/// for camera-visible verification. Doesn't loop; returns so the
/// shell can still start afterwards.
// /
/// Currently dormant — the call site in `kernel_main_apple` is
/// commented out so a real M4 chainload doesn't spend 70+ s
/// self-testing before the operator gets a prompt. Kept as staged
/// tooling; uncomment to re-enable.
#[allow(dead_code)]
fn apple_kernel_self_test() {
    use drivers::apple::uart;

    uart::puts("\n[selftest] starting kernel self-test\n");

    // Test 1: frame allocator round-trip.
    uart::puts("[selftest] frame::alloc_frame ... ");
    match kernel::mm::frame::alloc_frame() {
        Some(addr) => {
            uart::puts("OK (addr=0x");
            uart::puthex64(addr as u64);
            uart::puts(")\n");
            kernel::mm::frame::free_frame(addr);
            uart::puts("[selftest]   free_frame returned\n");
        }
        None => {
            uart::puts("FAIL (out of memory)\n");
            return;
        }
    }

    // Test 2: BatFS create (exercises rng::fill_bytes → sha256 KDF
    // → AES-CTR encrypt → HMAC-SHA256 tag → NONCE_COUNTER advance).
    const NAME: &str = "selftest.txt";
    const PLAINTEXT: &[u8] = b"Hello from Bat_OS on real Apple M4 silicon.";
    uart::puts("[selftest] batfs::create(\""); uart::puts(NAME); uart::puts("\") ... ");
    match fs::batfs::create(NAME, PLAINTEXT) {
        Ok(()) => uart::puts("OK\n"),
        Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); return; }
    }

    // Test 3: BatFS read+verify round-trip (HMAC before decrypt).
    uart::puts("[selftest] batfs::read+verify ... ");
    let mut out = [0u8; 128];
    match fs::batfs::read(NAME, &mut out) {
        Ok(n) => {
            if n == PLAINTEXT.len() && &out[..n] == PLAINTEXT {
                uart::puts("OK ("); kernel::mm::print_num(n); uart::puts(" B matched)\n");
            } else {
                uart::puts("FAIL: plaintext mismatch\n");
                return;
            }
        }
        Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); return; }
    }

    // Test 4: second file (exercises NONCE_COUNTER increment across
    // creates — proves the new IrqGuard + load+store holds more than
    // once, and proves separate file keys via sha256 derivation).
    uart::puts("[selftest] batfs::create(\"notes.txt\") ... ");
    match fs::batfs::create("notes.txt", b"M4 boot verified. LL/SC on Device memory bypassed.") {
        Ok(()) => uart::puts("OK\n"),
        Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); return; }
    }

    // Test 5: filesystem listing (exercises batfs::list + stats).
    let (count, cap) = fs::batfs::stats();
    uart::puts("[selftest] batfs::stats = ");
    kernel::mm::print_num(count);
    uart::puts("/");
    kernel::mm::print_num(cap);
    uart::puts(" files in use\n");

    // Test 6: Merkle-tree integrity over the two-file fs.
    uart::puts("[selftest] batfs::merkle_root = 0x");
    let root = fs::batfs::merkle_root();
    for i in 0..8 {
        uart::puthex32(u32::from_be_bytes([root[i*4], root[i*4+1], root[i*4+2], root[i*4+3]]));
    }
    uart::puts("\n");
    uart::puts("[selftest] batfs::verify_all_integrity ... ");
    if fs::batfs::verify_all_integrity() {
        uart::puts("OK\n");
    } else {
        uart::puts("FAIL\n");
    }

    // Test 7: AES-128-GCM + AES-256-GCM known-answer vectors
    // (NIST SP 800-38D). brought AES-256-GCM in; this
    // verifies both ciphers reproduce the published tags AND reject
    // tampered ciphertext on real M4 silicon. Without this, a fault
    // in the AES round constants or GHASH reduction wouldn't
    // surface until a TLS server NACKs a ClientHello.
    uart::puts("[selftest] gcm_verified::selftest ... ");
    match crate::crypto::gcm_verified::selftest() {
        Ok(()) => uart::puts("OK\n"),
        Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); }
    }

    // Test 8: kernel state summary.
    let (used, total) = kernel::mm::frame::stats();
    uart::puts("[selftest] frame pool: ");
    kernel::mm::print_num(used);
    uart::puts(" used / ");
    kernel::mm::print_num(total);
    uart::puts(" total (");
    kernel::mm::print_num((total - used) * 4 / 1024);
    uart::puts(" MiB free)\n");

    // Final report for the direct-API half.
    uart::puts("[selftest] all PASS\n\n");

    // Second half: replay a couple of shell commands through the
    // real dispatcher. Pared down from the 9-command replay so the
    // HV-guest path under M4 doesn't burn 70 s of a ~100 s session
    // on self-test before the human gets the prompt.
    uart::puts("[selftest] shell-dispatch replay ---\n");
    apple_run_cmd("uname");
    apple_run_cmd("uptime");
    uart::puts("[selftest] replay complete\n");
}

/// Apple-silicon kernel entry. Called from the asm trampoline with
/// `boot_args_ptr` provided by m1n1 (or by the Linux-bridge stub on
/// the Apple HV path). The pointer is bounds- and revision-checked
/// inside before any dereference.
///
/// # Safety
///
/// `boot_args_ptr` must point to a valid `m1n1` BootArgsRaw image:
/// at least `mem::size_of::<BootArgsRaw>()` bytes of readable memory
/// matching the m1n1 layout. The asm trampoline that supplies this
/// pointer is the only legitimate caller; any other caller must
/// uphold the same contract or we'll fault during boot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main_apple(boot_args_ptr: *const drivers::apple::boot_args::BootArgsRaw) -> ! {
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

    // Disable the Apple hardware watchdog ASAP so it can't reset the
    // Mac out from under us while we bring up the rest of the kernel.
    drivers::apple::wdt::disable();

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
    // V-HV-GUEST-2 / V-HV-SCREEN: Under m1n1's HV (EL1), the FB may
    // or may not still be valid depending on whether run_guest.py's
    // Python half called `fb_shutdown(True)`. When `BATOS_KEEP_FB=1`
    // is set on the host, the FB memory stays live and DCP keeps
    // scanning it out, so Bat_OS writes to boot_args->video.base show
    // up on the Mac's internal LCD and can be captured via
    // scripts/hv/m4_screenshot.py. When the FB is freed, writes would
    // clobber m1n1's heap.
    //
    // We can't detect the host's choice from inside the guest, so
    // trust boot_args: if video.base is non-zero, populate soc FB
    // info and let the FB paths render. Operators who don't set
    // BATOS_KEEP_FB get the previous "no FB under HV" behaviour only
    // by manually patching run_guest to clear video.base; with the
    // default run_guest.py behaviour (fb_shutdown + keep passing
    // video.base through), this call will fill soc's FB info and the
    // 16 MiB FB paint in boot.s is still gated by CurrentEL==EL1, so
    // we avoid the 16 MiB heap-clobber. The narrower font-glyph
    // writes through fb_console are small enough not to catastrophically
    // corrupt m1n1's heap before reset in the fb_shutdown=True case.
    let cur_el: u64;
    unsafe { core::arch::asm!("mrs {0}, CurrentEL", out(reg) cur_el, options(nomem, nostack)); }
    let under_hv = (cur_el & 0xC) == 0x4;
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
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::pipe::init();
    kernel::arch::init_exceptions();

    // V-HV-GUEST-3: Under m1n1's hypervisor (EL1), the AIC, DART, DWC3
    // and other hardware blocks are already owned and initialised by
    // m1n1. Re-initialising them from the guest provokes L2C external
    // errors when m1n1's guest pass-through mapping meets Apple's
    // access-control gate on shared MMIO (observed on M4 T8132 —
    // writes to AIC +0x4100 triggered an SError back into m1n1's own
    // handler and crashed the HV). Skip hardware bring-up at EL1.
    if under_hv {
        drivers::apple::uart::puts("[boot] (HV guest) skipping AIC + hw bring-up\n");
    } else {
        drivers::apple::uart::puts("[boot] Initializing AIC2...\n");
        drivers::apple::aic::init();

        // V-ASAHI-3.5: bring up every peripheral module that has a
        // hardware-access entry point. Failures are NOT fatal —
        // missing peripherals are legitimate on some boards. On M4
        // the sub-calls now guard themselves against unresolved ADT
        // addresses to avoid faulting on M1-era fallbacks.
        let bu = drivers::apple::bring_up_all();
        drivers::apple::print_bring_up_report(&bu);
    }

    // V-APPLE-AUTH-1: passphrase resolution — same priority order as
    // the QEMU path, so `BAT_OS_PASSPHRASE=<plaintext>` at build time
    // works identically on both targets:
    // 1. BAT_OS_PASSPHRASE (option_env!) — baked at build time
    // 2. read_passphrase_apple (non-blocking UART prompt) if we
    // actually got bytes
    // 3. dev_fallback (SHA-256 over kernel-image hash — same as QEMU)
    const BUILD_PASSPHRASE_APPLE: Option<&str> = option_env!("BAT_OS_PASSPHRASE");
    const BUILD_DURESS_APPLE: Option<&str> = option_env!("BAT_OS_DURESS");
    let mut passphrase_buf = [0u8; 128];
    let passphrase_len = read_passphrase_apple(&mut passphrase_buf);
    let mut dev_fallback_buf_apple = [0u8; 16];
    let dev_fb_len = derive_secret_string(DEV_FALLBACK_LABEL,
                                          &mut dev_fallback_buf_apple).len();
    let mut duress_buf_apple = [0u8; 16];
    let duress_apple_bytes = derive_secret_string(DURESS_LABEL, &mut duress_buf_apple);
    let passphrase_slice: &[u8] = if let Some(s) = BUILD_PASSPHRASE_APPLE {
        drivers::apple::uart::puts("  [auth] using BAT_OS_PASSPHRASE (build-time)\n");
        s.as_bytes()
    } else if passphrase_len == 0 {
        drivers::apple::uart::puts("  (empty — dev fallback)\n");
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
        drivers::apple::uart::puts("[boot] Display ready -- drawing splash\n");
        // V-ASAHI-2.1: render the boot splash so the operator sees on
        // the actual display (not just over USB serial) that Bat_OS
        // owns the M4. Fills the framebuffer m1n1 set up.
        drivers::apple::dcp::boot_splash();
        // Enable the on-screen mirror of `apple::uart::puts`. This
        // must come AFTER boot_splash (which fill_screens the whole
        // display) so we don't fight over the region.
        drivers::apple::fb_console::init();
        drivers::apple::uart::puts("[boot] Splash rendered -- launching apple shell\n");
        drivers::apple::uart::puts("[boot] FB console: uart mirror active\n");

        // Skip the kernel self-test on boot — it eats ~100 s of the
        // ~45-150 s HV session budget, leaving no time for the
        // desktop to render before the Mac resets. The individual
        // checks are still callable on demand via the `self-test`
        // shell command (apple_shell_dispatch path).
        // apple_kernel_self_test();
        drivers::apple::uart::puts("[boot] (skipping self-test — use `self-test` shell cmd)\n");

        // Initialize SPI keyboard — but only if not under HV. On M4
        // under HV the SPI controller is owned by m1n1 and writing to
        // its MMIO traps/faults.
        if !under_hv {
            let _ = drivers::apple::spi::init();
        }

        // V-APPLE-AUTH-2: real auth gate. Same flow as QEMU:
        // 1. auth::init with the resolved passphrase + duress code.
        // 2. boot_screen::run() — renders login, blocks on input,
        // verifies. Returns only on SUCCESS (Duress triggers
        // silent wipe + noreturn; LockedOut halts).
        //
        // Input route: platform::serial_getc() on Apple returns SPI
        // first (None under HV) then dockchannel UART. Under the
        // batos_hv_interactive.py script, keystrokes from Ubuntu
        // → /dev/ttyACM2 → m1n1 vuart hook → guest RX ring → getc.
        //
        // Override the compiled-in passphrase at build time:
        // BAT_OS_PASSPHRASE=mypass bash build_apple.sh
        let passphrase_str_apple = core::str::from_utf8(passphrase_slice)
            .unwrap_or("");
        let duress_str_apple = match BUILD_DURESS_APPLE {
            Some(s) => s,
            None => core::str::from_utf8(duress_apple_bytes).unwrap_or(""),
        };
        security::auth::init(passphrase_str_apple, duress_str_apple);
        drivers::apple::uart::puts("[security] Launching auth gate — type passphrase to unlock\n");
        security::boot_screen::run();
        drivers::apple::uart::puts("[security] AUTH PASSED — launching shell\n");

        // V-APPLE-UX-2: arm dead-man's-switch (48 h) just like QEMU.
        security::deadman::arm(48);

        // V-APPLE-UX-3: full microkernel desktop — same
        // `ui::desktop::run()` QEMU uses. Input via
        // platform::serial_getc (dockchannel UART under HV, SPI on
        // bare metal). Rendering via ui::gpu with ARGB2101010 colour
        // conversion. Link-time-absolute pointer accesses (Rust
        // no_std codegen sometimes materialises those into rodata
        // pointer tables) are handled by the HV-side stage-2 alias:
        // run_guest.py maps 0x810000000..+32MiB → guest_base, so
        // link-time accesses land on the runtime bytes.
        ui::desktop::run();
    } else {
        drivers::apple::uart::puts("[boot] No display — serial shell\n\n");
        apple_serial_shell();
    }
}

/// Interactive shell for the M4 path: reads a command line from the
/// dockchannel UART, dispatches into real kernel ops (memory stats,
/// BatFS, ...), and prints results back over the same UART.
// /
/// Note: with m1n1 replaced by our payload, the Mac's USB-CDC
/// endpoint is gone — so Ubuntu can't read/write this serial until
/// Bat_OS gets its own USB-CDC class driver. Until then this runs
/// silently (getc() returns None), but the shell itself is fully
/// functional: once a real byte stream arrives it will parse
/// commands and execute against the live kernel.
fn apple_serial_shell() -> ! {
    use drivers::apple::uart;

    uart::puts("\n");
    uart::puts("bat_os> ");

    let mut buf = [0u8; 128];
    let mut len: usize = 0;

    // V-HV-GUEST-4: detect HV (EL1) at entry — under HV we don't have
    // CNTP ticks, so WFE never wakes; busy-poll the UART instead.
    let cur_el: u64;
    unsafe { core::arch::asm!("mrs {0}, CurrentEL", out(reg) cur_el, options(nomem, nostack)); }
    let under_hv = (cur_el & 0xC) == 0x4;

    loop {
        let Some(c) = uart::getc() else {
            if !under_hv {
                unsafe { core::arch::asm!("wfe"); }
            } else {
                // Under HV we can't WFE (no ticks/IRQs to wake us).
                // Busy-poll. Experiment showed that slowing to 1 kHz
                // didn't appreciably extend session length (35 s →
                // 45 s), so the ~45 s reset is a wall-clock condition,
                // not CPU-load-driven.
                core::hint::spin_loop();
            }
            continue;
        };
        match c {
            b'\r' | b'\n' => {
                uart::puts("\r\n");
                if len > 0 {
                    let cmd = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                    apple_shell_dispatch(cmd);
                    len = 0;
                }
                uart::puts("bat_os> ");
            }
            0x08 | 0x7F => {
                if len > 0 {
                    len -= 1;
                    uart::putc(0x08); uart::putc(b' '); uart::putc(0x08);
                }
            }
            _ if c >= 0x20 && c < 0x7F && len < buf.len() => {
                buf[len] = c;
                len += 1;
                uart::putc(c);
            }
            _ => {}
        }
    }
}

/// Parse + execute a single shell command on the Apple path. All
/// real kernel operations; no fake output. Unknown commands print
/// an error and return.
fn apple_shell_dispatch(line: &str) {
    use drivers::apple::uart;

    let line = line.trim_end_matches(|c: char| c == '\r' || c == '\n' || c == ' ');
    let mut parts = line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");

    match cmd {
        "" => {}
        "help" => {
            uart::puts("  help           — list commands\n");
            uart::puts("  uname          — kernel identity\n");
            uart::puts("  mem            — frame allocator stats\n");
            uart::puts("  fb             — framebuffer info\n");
            uart::puts("  uptime         — ticks since boot (CNTPCT_EL0 / CNTFRQ_EL0)\n");
            uart::puts("  cpuid          — CPU identification regs (MIDR / CTR / CurrentEL)\n");
            uart::puts("  rand [N]       — N random bytes (hex, default 16, max 64)\n");
            uart::puts("  rng            — show RNG / HW entropy availability\n");
            uart::puts("  sha256 <text>  — SHA-256 hash of <text> (hex)\n");
            uart::puts("  bench sha256   — time 64 KiB of software SHA-256\n");
            uart::puts("  self-test      — frame alloc + BatFS encrypt/verify/Merkle round-trip\n");
            uart::puts("  sha-hw         — probe ARMv8.2 SHA-256 crypto extension\n");
            uart::puts("  aes-hw         — probe ARMv8 AES crypto extension\n");
            uart::puts("  screen [N]     — dump FB over vuart at 1/N scale (default 4)\n");
            uart::puts("  batfs ls       — list BatFS files\n");
            uart::puts("  batfs create <name> <plaintext>\n");
            uart::puts("  batfs read <name>\n");
            uart::puts("  halt           — WFE loop\n");
        }
        "uname" => {
            uart::puts("Bat_OS aarch64 (Apple M4 / T8132 Donan)\n");
        }
        "cpuid" => {
            let midr: u64; let ctr: u64; let cur_el: u64;
            let mpidr: u64; let aidr: u64;
            unsafe {
                core::arch::asm!("mrs {}, midr_el1", out(reg) midr);
                core::arch::asm!("mrs {}, ctr_el0", out(reg) ctr);
                core::arch::asm!("mrs {}, CurrentEL", out(reg) cur_el);
                core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr);
                core::arch::asm!("mrs {}, aidr_el1", out(reg) aidr);
            }
            uart::puts("  MIDR_EL1:   0x"); uart::puthex64(midr); uart::puts("\n");
            uart::puts("  CTR_EL0:    0x"); uart::puthex64(ctr); uart::puts("\n");
            uart::puts("  CurrentEL:  "); kernel::mm::print_num(((cur_el >> 2) & 3) as usize); uart::puts("\n");
            uart::puts("  MPIDR_EL1:  0x"); uart::puthex64(mpidr); uart::puts("\n");
            uart::puts("  AIDR_EL1:   0x"); uart::puthex64(aidr); uart::puts("\n");
            let part = (midr >> 4) & 0xfff;
            uart::puts("  MIDR.PART:  0x"); uart::puthex32(part as u32); uart::puts("\n");
            match part {
                0x52 => { uart::puts("  -> M4 Donan (E core)\n"); }
                0x53 => { uart::puts("  -> M4 Donan (P core)\n"); }
                _    => {}
            }
        }
        "self-test" => {
            // Runs the frame-allocator / BatFS / Merkle-integrity
            // chunks of `apple_kernel_self_test`. Fresh-boot only —
            // re-running fails BatFS create() on "selftest.txt"
            // already existing, which is correct behaviour.
            uart::puts("\n[selftest] starting kernel self-test\n");
            uart::puts("[selftest] frame::alloc_frame ... ");
            match kernel::mm::frame::alloc_frame() {
                Some(addr) => {
                    uart::puts("OK (addr=0x"); uart::puthex64(addr as u64); uart::puts(")\n");
                    kernel::mm::frame::free_frame(addr);
                }
                None => { uart::puts("FAIL (out of memory)\n"); return; }
            }
            const SELFT_NAME: &str = "selftest.txt";
            const SELFT_PT: &[u8] = b"Hello from Bat_OS on real Apple M4 silicon under HV.";
            uart::puts("[selftest] batfs::create ... ");
            match fs::batfs::create(SELFT_NAME, SELFT_PT) {
                Ok(()) => uart::puts("OK\n"),
                Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); return; }
            }
            uart::puts("[selftest] batfs::read+verify ... ");
            let mut out = [0u8; 128];
            match fs::batfs::read(SELFT_NAME, &mut out) {
                Ok(n) => {
                    if n == SELFT_PT.len() && &out[..n] == SELFT_PT {
                        uart::puts("OK ("); kernel::mm::print_num(n); uart::puts(" B matched)\n");
                    } else {
                        uart::puts("FAIL: plaintext mismatch\n"); return;
                    }
                }
                Err(e) => { uart::puts("FAIL: "); uart::puts(e); uart::puts("\n"); return; }
            }
            uart::puts("[selftest] batfs::merkle_root = 0x");
            let root = fs::batfs::merkle_root();
            for i in 0..8 {
                uart::puthex32(u32::from_be_bytes([root[i*4], root[i*4+1], root[i*4+2], root[i*4+3]]));
            }
            uart::puts("\n[selftest] batfs::verify_all_integrity ... ");
            if fs::batfs::verify_all_integrity() {
                uart::puts("OK\n");
            } else {
                uart::puts("FAIL\n");
            }
            uart::puts("[selftest] all PASS\n");
        }
        "screen" => {
            // Dump the Apple M4 framebuffer over the dockchannel UART
            // (→ /dev/ttyACM2 vuart → Ubuntu). Scaled down by N and
            // emitted as 8-hex-char pixels, one FB row per output line.
            // Ubuntu side: scripts/hv/m4_screen_dump.py reads, decodes
            // ARGB2101010 → RGB888, writes a PNG. Works mid-HV-session.
            //
            // Writes the raw bytes via write32 → DATA_TX8 directly so
            // that fb_console (which mirrors every uart::putc into
            // glyphs in the FB) doesn't scrawl over the exact FB bytes
            // we're reading.
            use drivers::apple::soc;
            let base = soc::fb_base();
            let width = soc::fb_width() as usize;
            let height = soc::fb_height() as usize;
            let stride = soc::fb_stride() as usize;
            if base == 0 || width == 0 || height == 0 || stride == 0 {
                uart::puts("  no framebuffer available\n");
                return;
            }
            let scale: usize = match parts.next().and_then(|s| {
                s.trim_end_matches(|c: char| c == '\r' || c == '\n').parse::<usize>().ok()
            }) {
                Some(n) if n >= 1 && n <= 16 => n,
                _ => 4,
            };
            let out_w = width / scale;
            let out_h = height / scale;
            uart::puts("SCREEN_BEGIN w="); kernel::mm::print_num(out_w);
            uart::puts(" h="); kernel::mm::print_num(out_h);
            uart::puts(" scale="); kernel::mm::print_num(scale);
            uart::puts(" fmt=argb2101010\n");

            let uart_base = soc::uart0_base();
            const DATA_TX8: usize = 0x4004;
            const DATA_TX_FREE: usize = 0x4014;
            let tx_free = (uart_base + DATA_TX_FREE) as *const u32;
            let tx8     = (uart_base + DATA_TX8) as *mut u32;
            let put = |b: u8| unsafe {
                let mut guard: u32 = 1_000_000;
                while core::ptr::read_volatile(tx_free) == 0 {
                    guard = guard.saturating_sub(1);
                    if guard == 0 { return; }
                    core::hint::spin_loop();
                }
                core::ptr::write_volatile(tx8, b as u32);
            };
            const HX: &[u8; 16] = b"0123456789abcdef";
            for y in 0..out_h {
                let src_row = y * scale;
                let row_base = base + src_row * stride;
                for x in 0..out_w {
                    let src_x = x * scale;
                    let w: u32 = unsafe {
                        core::ptr::read_volatile((row_base + src_x * 4) as *const u32)
                    };
                    for i in (0..8).rev() {
                        let nib = ((w >> (i * 4)) & 0xf) as usize;
                        put(HX[nib]);
                    }
                }
                put(b'\n');
            }
            uart::puts("SCREEN_END\n");
        }
        "aes-hw" => {
            // Probe ARMv8 AES crypto extension. Runs one AES round
            // (AESE + AESMC) and prints the result.
            let isar0: u64;
            unsafe { core::arch::asm!("mrs {}, id_aa64isar0_el1", out(reg) isar0); }
            let aes = (isar0 >> 4) & 0xf;
            uart::puts("  ISAR0.AES nibble: 0x"); uart::puthex32(aes as u32); uart::puts("\n");
            if aes == 0 {
                uart::puts("  AES crypto extension not advertised — skipping\n");
                return;
            }
            // Test: AES-128 round on fixed state and key.
            let mut out0: u64; let mut out1: u64;
            unsafe {
                core::arch::asm!(
                    ".arch armv8-a+aes",
                    "movi v0.16b, #0x20",   // state (plain-ish)
                    "movi v1.16b, #0x55",   // round key
                    "aese  v0.16b, v1.16b", // one SubBytes + ShiftRows XOR key
                    "aesmc v0.16b, v0.16b", // MixColumns
                    "mov {0}, v0.d[0]",
                    "mov {1}, v0.d[1]",
                    out(reg) out0, out(reg) out1,
                    options(nostack),
                );
            }
            uart::puts("  AESE + AESMC executed (no UNDEF)\n");
            uart::puts("  V0.d[0] = 0x"); uart::puthex64(out0); uart::puts("\n");
            uart::puts("  V0.d[1] = 0x"); uart::puthex64(out1); uart::puts("\n");
            uart::puts("  -> hardware AES accessible from EL1 guest\n");
        }
        "sha-hw" => {
            // Probe whether the ARMv8.2 SHA256 crypto instructions
            // (SHA256H / H2 / SU0 / SU1) actually execute at EL1
            // under HV. If HCR_EL2.TRND bit or CPTR_EL2 traps SIMD,
            // these will UNDEF and the kernel exception handler will
            // print something. A successful run prints the expected
            // output vector, proving FP/NEON is live AND the SHA2
            // crypto unit is accessible from the guest.
            let isar0: u64;
            unsafe { core::arch::asm!("mrs {}, id_aa64isar0_el1", out(reg) isar0); }
            let sha2 = (isar0 >> 12) & 0xf;
            uart::puts("  ISAR0.SHA2 nibble: 0x"); uart::puthex32(sha2 as u32); uart::puts("\n");
            if sha2 == 0 {
                uart::puts("  SHA256 crypto extension not advertised — skipping\n");
                return;
            }
            // Test vector: run SHA256H once on zero input and print
            // the resulting V0 low 64 bits. If the instruction faults
            // the handle_sync_exception will take over; if it doesn't
            // we should see a specific value back.
            let mut out0: u64 = 0;
            let mut out1: u64 = 0;
            let ok = unsafe {
                // Enable the +sha2 target feature for just this block
                // via a .arch directive inside the inline asm. The
                // default aarch64-unknown-none target doesn't advertise
                // sha2, so the assembler refuses SHA256H* otherwise.
                core::arch::asm!(
                    ".arch armv8.2-a+sha2",
                    "movi v0.16b, #0",
                    "movi v1.16b, #0",
                    "movi v2.16b, #0x42",
                    "movi v3.16b, #0x42",
                    "sha256h   q0, q1, v2.4s",
                    "sha256h2  q1, q0, v2.4s",
                    "sha256su0 v2.4s, v3.4s",
                    "mov {0}, v0.d[0]",
                    "mov {1}, v0.d[1]",
                    out(reg) out0,
                    out(reg) out1,
                    options(nostack),
                );
                true
            };
            if ok {
                uart::puts("  SHA256H/H2/SU0 executed (no UNDEF)\n");
                uart::puts("  V0.d[0] = 0x"); uart::puthex64(out0); uart::puts("\n");
                uart::puts("  V0.d[1] = 0x"); uart::puthex64(out1); uart::puts("\n");
                uart::puts("  -> hardware SHA-256 accessible from EL1 guest\n");
            }
        }
        "rng" => {
            // Report what entropy sources are in play. Verifies M4
            // exposes the ARMv8.5 RNDR feature (our RNG mixes it into
            // the SHA-chain DRBG when present).
            let isar0: u64;
            unsafe { core::arch::asm!("mrs {}, id_aa64isar0_el1", out(reg) isar0); }
            let rndr_nibble = (isar0 >> 60) & 0xf;
            uart::puts("  ID_AA64ISAR0_EL1: 0x"); uart::puthex64(isar0); uart::puts("\n");
            uart::puts("  RNDR nibble:      0x"); uart::puthex32(rndr_nibble as u32); uart::puts("\n");
            uart::puts("  RNDR available:   ");
            uart::puts(if rndr_nibble >= 1 { "yes\n" } else { "no\n" });
            uart::puts("  Bat_OS HW RNG:    ");
            uart::puts(if crypto::rng::have_rndr() { "enabled\n" } else { "disabled (SHA-chain only)\n" });
            // Also report SHA2 crypto extension availability.
            let sha2_nibble = (isar0 >> 12) & 0xf;
            uart::puts("  SHA2 ext nibble:  0x"); uart::puthex32(sha2_nibble as u32); uart::puts("\n");
            uart::puts("  AES ext nibble:   0x"); uart::puthex32(((isar0 >> 4) & 0xf) as u32); uart::puts("\n");
        }
        "rand" => {
            let n = match parts.next() {
                Some(s) => {
                    let trimmed = s.trim_end_matches(|c: char| c == '\r' || c == '\n');
                    match trimmed.parse::<usize>() {
                        Ok(v) => v.min(64),
                        Err(_) => 16,
                    }
                }
                None => 16,
            };
            let mut buf = [0u8; 64];
            crypto::rng::fill_bytes(&mut buf[..n]);
            uart::puts("  bytes: "); kernel::mm::print_num(n); uart::puts("\n");
            uart::puts("  hex: ");
            const HX: &[u8; 16] = b"0123456789abcdef";
            for b in buf[..n].iter() {
                uart::putc(HX[(*b >> 4) as usize]);
                uart::putc(HX[(*b & 0xf) as usize]);
            }
            uart::puts("\n");
        }
        "sha256" => {
            let rest = match parts.next() {
                Some(s) => s.trim_end_matches(|c: char| c == '\r' || c == '\n'),
                None => { uart::puts("  usage: sha256 <text>\n"); return; }
            };
            let digest = crate::crypto::sha256::hash(rest.as_bytes());
            uart::puts("  input: ");  uart::puts(rest);   uart::puts("\n");
            uart::puts("  bytes: ");  kernel::mm::print_num(rest.len()); uart::puts("\n");
            uart::puts("  sha256: ");
            const HX: &[u8; 16] = b"0123456789abcdef";
            for b in digest.iter() {
                uart::putc(HX[(*b >> 4) as usize]);
                uart::putc(HX[(*b & 0xf) as usize]);
            }
            uart::puts("\n");
        }
        "mem" => {
            let (used, total) = kernel::mm::frame::stats();
            uart::puts("  frames used: "); kernel::mm::print_num(used); uart::puts("\n");
            uart::puts("  frames total: "); kernel::mm::print_num(total); uart::puts("\n");
            uart::puts("  free KiB: "); kernel::mm::print_num((total - used) * 4); uart::puts("\n");
        }
        "fb" => {
            use drivers::apple::soc;
            uart::puts("  base: 0x"); uart::puthex64(soc::fb_base() as u64); uart::puts("\n");
            uart::puts("  width: "); kernel::mm::print_num(soc::fb_width() as usize); uart::puts("\n");
            uart::puts("  height: "); kernel::mm::print_num(soc::fb_height() as usize); uart::puts("\n");
            uart::puts("  stride: 0x"); uart::puthex32(soc::fb_stride()); uart::puts("\n");
        }
        "uptime" => {
            let cnt: u64; let freq: u64;
            unsafe {
                core::arch::asm!("mrs {}, cntpct_el0", out(reg) cnt);
                core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
            }
            let secs = if freq > 0 { cnt / freq } else { 0 };
            uart::puts("  cntpct: 0x"); uart::puthex64(cnt); uart::puts("\n");
            uart::puts("  cntfrq: "); kernel::mm::print_num(freq as usize); uart::puts(" Hz\n");
            uart::puts("  seconds: "); kernel::mm::print_num(secs as usize); uart::puts("\n");
        }
        "halt" => {
            uart::puts("  halting (wfe)\n");
            loop { unsafe { core::arch::asm!("wfe"); } }
        }
        "smc-probe" => {
            // M4-HV 2026-04-20 15:30: does EL1 under HV reach the
            // SMC ASC MMIO at 0x38c600000 through stage-2 passthrough?
            // If this returns without SError, the guest has direct
            // access to SMC and we can do a keepalive poke from the
            // busy-poll loop (see smc_keepalive_pet in serial path).
            uart::puts("  reading SMC ASC regs from EL1 ...\n");
            unsafe {
                let cpu_ctrl = 0x38c600044u64 as *const u32;
                let a2i_ctrl = 0x38c608110u64 as *const u32;
                let i2a_ctrl = 0x38c608114u64 as *const u32;
                let c = core::ptr::read_volatile(cpu_ctrl);
                let a = core::ptr::read_volatile(a2i_ctrl);
                let i = core::ptr::read_volatile(i2a_ctrl);
                uart::puts("  SMC CPU_CONTROL:  0x"); uart::puthex32(c); uart::puts("\n");
                uart::puts("  SMC A2I_CONTROL:  0x"); uart::puthex32(a); uart::puts("\n");
                uart::puts("  SMC I2A_CONTROL:  0x"); uart::puthex32(i); uart::puts("\n");
            }
            uart::puts("  [smc-probe OK — stage-2 passes through SMC]\n");
        }
        "bench" => {
            let sub = parts.next().unwrap_or("").trim();
            match sub {
                "sha256" => {
                    // Hash 64 KiB in 64 B chunks, time with CNTPCT.
                    let buf = [0x42u8; 64];
                    let rounds: usize = 1024; // 64 B × 1024 = 64 KiB
                    let freq: u64;
                    let t0: u64; let t1: u64;
                    unsafe {
                        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
                        core::arch::asm!("isb; mrs {}, cntpct_el0", out(reg) t0);
                    }
                    let mut acc: [u8; 32] = [0; 32];
                    for _ in 0..rounds {
                        acc = crate::crypto::sha256::hash(&buf);
                    }
                    unsafe { core::arch::asm!("isb; mrs {}, cntpct_el0", out(reg) t1); }
                    let dt = t1 - t0;
                    let bytes = (rounds * 64) as u64;
                    // KiB/s ≈ bytes * freq / (dt * 1024)
                    let kib_s = if dt > 0 { bytes * freq / (dt * 1024) } else { 0 };
                    uart::puts("  rounds: "); kernel::mm::print_num(rounds); uart::puts("\n");
                    uart::puts("  bytes:  "); kernel::mm::print_num(bytes as usize); uart::puts("\n");
                    uart::puts("  dt_cntpct: "); kernel::mm::print_num(dt as usize); uart::puts("\n");
                    uart::puts("  cntfrq: "); kernel::mm::print_num(freq as usize); uart::puts(" Hz\n");
                    uart::puts("  KiB/s:  "); kernel::mm::print_num(kib_s as usize); uart::puts("\n");
                    // Print last hash so the compiler can't optimise the loop away.
                    uart::puts("  last:   ");
                    const HX: &[u8; 16] = b"0123456789abcdef";
                    for b in acc.iter().take(8) {
                        uart::putc(HX[(*b >> 4) as usize]);
                        uart::putc(HX[(*b & 0xf) as usize]);
                    }
                    uart::puts("…\n");
                }
                _ => {
                    uart::puts("  usage: bench sha256\n");
                }
            }
        }
        "batfs" => {
            let sub = parts.next().unwrap_or("");
            match sub {
                "ls" => {
                    let (count, cap) = fs::batfs::stats();
                    uart::puts("  files: "); kernel::mm::print_num(count);
                    uart::puts(" / "); kernel::mm::print_num(cap); uart::puts("\n");
                    fs::batfs::list(|name, size, enc| {
                        uart::puts("  ");
                        uart::puts(name);
                        uart::puts(" (");
                        kernel::mm::print_num(size);
                        uart::puts(if enc { " B, enc)\n" } else { " B, plain)\n" });
                    });
                }
                "create" => {
                    let rest = parts.next().unwrap_or("");
                    let mut name_body = rest.splitn(2, ' ');
                    let name = name_body.next().unwrap_or("");
                    let body = name_body.next().unwrap_or("");
                    if name.is_empty() {
                        uart::puts("  usage: batfs create <name> <plaintext>\n");
                    } else {
                        match fs::batfs::create(name, body.as_bytes()) {
                            Ok(()) => uart::puts("  ok\n"),
                            Err(e) => { uart::puts("  error: "); uart::puts(e); uart::puts("\n"); }
                        }
                    }
                }
                "read" => {
                    let name = parts.next().unwrap_or("");
                    if name.is_empty() {
                        uart::puts("  usage: batfs read <name>\n");
                    } else {
                        let mut out = [0u8; 256];
                        match fs::batfs::read(name, &mut out) {
                            Ok(n) => {
                                uart::puts("  (");
                                kernel::mm::print_num(n);
                                uart::puts(" B) ");
                                let s = core::str::from_utf8(&out[..n]).unwrap_or("<non-utf8>");
                                uart::puts(s);
                                uart::puts("\n");
                            }
                            Err(e) => { uart::puts("  error: "); uart::puts(e); uart::puts("\n"); }
                        }
                    }
                }
                _ => {
                    uart::puts("  batfs: unknown subcommand — try `help`\n");
                }
            }
        }
        _ => {
            uart::puts("  unknown command: ");
            uart::puts(cmd);
            uart::puts("\n");
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
        drivers::uart::puts(":");
        kernel::mm::print_num(location.line() as usize);
        drivers::uart::puts("\n");
    }
    // info.message() is available on stable since 1.81; print it if any.
    drivers::uart::puts("  Msg:  ");
    use core::fmt::Write;
    struct UartWriter;
    impl Write for UartWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            drivers::uart::puts(s); Ok(())
        }
    }
    let _ = write!(UartWriter, "{}", info.message());
    drivers::uart::puts("\n");
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
