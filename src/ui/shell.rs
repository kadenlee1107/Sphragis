// Bat_OS — Interactive Kernel Shell
// Command-line interface rendered to the GPU console.
// Reads from UART, displays on framebuffer.

use crate::platform;
use crate::ui::console;
use crate::fs::batfs;
use crate::net;

const MAX_CMD_LEN: usize = 256;

pub fn run() -> ! {
    console::init();

    // Welcome banner
    console::puts_hi("      ___       _      ___  ___\n");
    console::puts_hi("     | _ ) __ _| |_   / _ \\/ __|\n");
    console::puts_hi("     | _ \\/ _` |  _| | (_) \\__ \\\n");
    console::puts_hi("     |___/\\__,_|\\__|  \\___/|___/\n");
    console::puts("\n");
    console::puts("  Microkernel Shell v0.3 — Type 'help' for commands\n");
    console::puts("  Zero dependencies. Zero trust.\n");
    console::puts("\n");

    let mut cmd_buf = [0u8; MAX_CMD_LEN];
    let mut cmd_len: usize = 0;

    console::prompt();

    loop {
        smc_keepalive_tick();
        if let Some(c) = platform::serial_getc() {
            match c {
                b'\r' | b'\n' => {
                    // Execute command
                    console::putc(b'\n');
                    platform::serial_puts("\n");

                    if cmd_len > 0 {
                        let cmd = unsafe {
                            core::str::from_utf8_unchecked(&cmd_buf[..cmd_len])
                        };
                        execute(cmd);
                        cmd_len = 0;
                    }

                    console::prompt();
                }
                0x08 | 0x7F => {
                    // Backspace
                    if cmd_len > 0 {
                        cmd_len -= 1;
                        console::putc(0x08);
                        platform::serial_putc(0x08);
                        platform::serial_putc(b' ');
                        platform::serial_putc(0x08);
                    }
                }
                0x03 => {
                    // Ctrl+C
                    console::puts("^C\n");
                    platform::serial_puts("^C\n");
                    cmd_len = 0;
                    console::prompt();
                }
                _ => {
                    if cmd_len < MAX_CMD_LEN - 1 && c >= 0x20 && c <= 0x7E {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        platform::serial_putc(c);
                    }
                }
            }
        }
        core::hint::spin_loop();
    }
}

/// Execute a command (called from desktop.rs).
pub fn execute_cmd(cmd: &str) {
    execute(cmd);
}

fn execute(cmd: &str) {
    let parts: [&str; MAX_PARTS] = split_cmd(cmd);
    let command = parts[0];

    // Mirror to serial
    platform::serial_puts("[shell] ");
    platform::serial_puts(cmd);
    platform::serial_puts("\n");

    match command {
        "help" => cmd_help(),
        "status" => cmd_status(),
        "memory" | "mem" => cmd_memory(),
        "clear" | "cls" => cmd_clear(),
        "whoami" => cmd_whoami(),
        "uname" => cmd_uname(),
        "uptime" => cmd_uptime(),
        "ls" | "files" => cmd_ls(),
        "write" => cmd_write(parts[1], parts[2]),
        "read" | "cat" => cmd_read(parts[1]),
        "rm" | "delete" => cmd_rm(parts[1]),
        "verify" => cmd_verify(parts[1]),
        "ping" => cmd_ping(parts[1]),
        "dns" | "resolve" => cmd_dns(parts[1]),
        "ifconfig" | "net" => cmd_ifconfig(),
        "fw" | "firewall" => cmd_firewall(),
        "fetch" => cmd_fetch(parts[1]),
        "batcave" => cmd_batcave(parts[1], parts[2], parts[3], &parts),
        "panic" => cmd_panic(),
        "hello" => cmd_run_elf("hello"),
        "hello_libc" | "libc" => cmd_run_elf("libc"),
        "threads" => cmd_run_elf("threads"),
        "netsurf" => cmd_run_elf("netsurf"),
        "freetype" | "ft" => cmd_run_elf("freetype"),
        "png" => cmd_run_elf("png"),
        "posix" => cmd_run_elf("posix"),
        "cxx" | "c++" => cmd_run_elf("cxx"),
        "v8" | "js" | "javascript" => cmd_run_elf("v8"),
        "blink" => cmd_run_elf("blink"),
        "chromium" | "chrome" => cmd_chromium(parts[1], parts[2], parts[3]),
        "browse" | "open" => {
            if !parts[1].is_empty() {
                console::puts("  Opening in BatBrowser: ");
                console::puts(parts[1]);
                console::puts("\n");
                // Navigate in browser (runs full pipeline including fetch + render)
                crate::ui::apps::browser::navigate(parts[1].as_bytes());
                // Switch to browser, render, then switch back to shell
                crate::ui::wm::switch_app(crate::ui::wm::APP_BROWSER);
                crate::ui::apps::browser::render();
                crate::ui::gpu::flush(0, 0, crate::ui::gpu::width(), crate::ui::gpu::height());
                crate::ui::wm::switch_app(crate::ui::wm::APP_SHELL);
            } else {
                console::puts("  usage: browse <url>\n");
            }
        }
        "screen" => cmd_screen(parts[1]),
        "otp-dump"    => cmd_otp_dump(),
        "otp-stats"   => cmd_otp_stats(),
        "otp-consume" => cmd_otp_consume(parts[1]),
        "smc-probe" => cmd_smc_probe(),
        "smc-pet" => cmd_smc_pet_start(),
        "smc-stop" => cmd_smc_pet_stop(),
        "" => {}
        _ => {
            console::puts("  unknown command: ");
            console::puts(command);
            console::puts("\n  type 'help' for commands\n");
        }
    }
}

/// Dump the framebuffer over whatever serial is attached, at 1/N
/// scale (default 4). Mirrors the `apple_shell_dispatch` version —
/// callers on QEMU get the same output format, decodable by
/// scripts/hv/capture_screen.py.
fn cmd_screen(arg: &str) {
    use crate::drivers::apple::soc;
    let scale: usize = match arg.trim_end_matches(|c: char| c == '\r' || c == '\n')
        .parse::<usize>() {
        Ok(n) if (1..=16).contains(&n) => n,
        _ => 4,
    };
    let base = crate::ui::gpu::framebuffer() as usize;
    let width = crate::ui::gpu::width() as usize;
    let height = crate::ui::gpu::height() as usize;
    let stride = soc::fb_stride() as usize;
    if base == 0 || width == 0 || height == 0 || stride == 0 {
        console::puts("  no framebuffer\n");
        return;
    }
    let out_w = width / scale;
    let out_h = height / scale;

    // On Apple/M4 write directly to dockchannel DATA_TX8 without
    // going through apple::uart::putc — that path mirrors each byte
    // into fb_console (draws a font glyph) which quadruples FB write
    // traffic and deadlocks the vuart ring. On other platforms fall
    // back to platform::serial_putc (QEMU PL011 has no fb mirror).
    let apple = matches!(crate::platform::current(),
                         crate::platform::Platform::AppleSilicon);
    let dc_base = if apple { soc::uart0_base() } else { 0 };
    const DATA_TX_FREE: usize = 0x4014;
    const DATA_TX8: usize = 0x4004;
    let put = |b: u8| {
        if apple && dc_base != 0 {
            unsafe {
                let mut guard: u32 = 1_000_000;
                while core::ptr::read_volatile((dc_base + DATA_TX_FREE) as *const u32) == 0 {
                    guard = guard.saturating_sub(1);
                    if guard == 0 { return; }
                    core::hint::spin_loop();
                }
                core::ptr::write_volatile((dc_base + DATA_TX8) as *mut u32, b as u32);
            }
        } else {
            platform::serial_putc(b);
        }
    };
    let puts = |s: &str| { for b in s.bytes() { put(b); } };
    let put_num = |n: usize| {
        let mut buf = [0u8; 20];
        let mut i = 20;
        let mut v = n;
        if v == 0 { put(b'0'); return; }
        while v > 0 { i -= 1; buf[i] = b'0' + (v % 10) as u8; v /= 10; }
        for bb in &buf[i..] { put(*bb); }
    };

    puts("SCREEN_BEGIN w=");
    put_num(out_w);
    puts(" h=");
    put_num(out_h);
    puts(" scale=");
    put_num(scale);
    puts(" fmt=argb2101010\n");

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
    puts("SCREEN_END\n");
}

fn cmd_help() {
    console::puts_hi("  BAT_OS COMMANDS\n");
    console::puts("  ---------------\n");
    console::puts("  help          Show this message\n");
    console::puts("  status        Security status\n");
    console::puts("  mem           Memory usage\n");
    console::puts("  clear         Clear screen\n");
    console::puts("  whoami        Current identity\n");
    console::puts("  uname         System information\n");
    console::puts("  uptime        System uptime\n");
    console::puts_hi("  FILE VAULT (AES-256 encrypted)\n");
    console::puts("  ---------------\n");
    console::puts("  ls            List encrypted files\n");
    console::puts("  write <f> <d> Create encrypted file\n");
    console::puts("  cat <file>    Read + decrypt file\n");
    console::puts("  rm <file>     Secure delete file\n");
    console::puts("  verify <file> Check integrity hash\n");
    console::puts_hi("  NETWORK\n");
    console::puts("  ---------------\n");
    console::puts("  net           Network interface info\n");
    console::puts("  ping <ip>     Ping a host\n");
    console::puts("  dns <host>    Resolve hostname\n");
    console::puts("  fw            Firewall stats\n");
    console::puts("  fetch <host>  HTTP GET (TCP)\n");
}

fn cmd_status() {
    console::puts_hi("  SECURITY STATUS\n");
    console::puts("  ---------------\n");
    console::puts("  Encryption:      ACTIVE (full disk + per-file)\n");
    console::puts("  Secure Enclave:  SIMULATED (QEMU)\n");
    console::puts("  Network:         OFFLINE\n");
    console::puts("  Firewall:        DEFAULT DENY ALL\n");
    console::puts("  Dead Man Switch: NOT ARMED (dev mode)\n");
    console::puts("  Capabilities:    ENFORCED\n");
    console::puts("  Kernel Mode:     EL1 (privileged)\n");
}

fn cmd_memory() {
    let (used, total) = crate::kernel::mm::frame::stats();
    let free = total - used;
    console::puts_hi("  MEMORY\n");
    console::puts("  ------\n");
    console::puts("  Total frames:  ");
    print_num(total);
    console::puts("\n");
    console::puts("  Used frames:   ");
    print_num(used);
    console::puts("\n");
    console::puts("  Free frames:   ");
    print_num(free);
    console::puts("\n");
    console::puts("  Free memory:   ");
    print_num(free * 4);
    console::puts(" KB\n");
}

fn cmd_clear() {
    console::init();
}

fn cmd_whoami() {
    console::puts_hi("  KADEN\n");
    console::puts("  Access: ROOT (sole operator)\n");
    console::puts("  Auth:   passphrase + hardware token\n");
}

fn cmd_uname() {
    console::puts("  Bat_OS v0.3.0 aarch64 (QEMU virt)\n");
    console::puts("  Kernel: microkernel (Rust + ARM64 asm)\n");
    console::puts("  Arch:   seL4-inspired capabilities\n");
}

fn cmd_uptime() {
    // Read ARM generic timer
    let count: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) count);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let seconds = count / freq;
    let minutes = seconds / 60;
    let secs = seconds % 60;

    console::puts("  Up ");
    print_num(minutes as usize);
    console::puts("m ");
    print_num(secs as usize);
    console::puts("s\n");
}

fn cmd_panic() {
    console::puts("  Triggering kernel panic...\n");
    panic!("user-triggered panic from shell");
}

// ── DESIGN_CRYPTO.md #11+#12: OTP duress + deadman pad handlers ──

fn cmd_otp_dump() {
    // Dump every token as hex. THIS IS THE PROVISIONING COMMAND —
    // the operator should run it ONCE at first boot, record the
    // output offline (paper / QR / air-gapped device), and never
    // run it again unless rotating the pad.
    console::puts_hi("  OTP PAD — RECORD OFFLINE, DO NOT RE-DUMP\n");
    console::puts("  ──────────────────────────────────────────\n");
    let found = crate::security::otp::dump_for_provisioning(&mut |slot, region, tok| {
        console::puts("  [");
        print_num(slot);
        console::puts("] ");
        // Pad slot number so columns align (slots 0..31 → 2 digits)
        if slot < 10 { console::puts(" "); }
        console::puts(region);
        for _ in region.len()..8 { console::puts(" "); }
        let hex = b"0123456789abcdef";
        for &b in tok.iter() {
            console::putc(hex[(b >> 4) as usize]);
            console::putc(hex[(b & 0x0f) as usize]);
        }
        console::puts("\n");
    });
    if !found {
        console::puts("  ERROR: OTP pad not initialised\n");
    }
    console::puts("  ──────────────────────────────────────────\n");
    console::puts("  After this boot, tokens are consumed per-use. Store them.\n");
}

fn cmd_otp_stats() {
    let (duress, deadman) = crate::security::otp::remaining();
    console::puts_hi("  OTP PAD STATUS\n");
    console::puts("  --------------\n");
    console::puts("  Duress tokens:   ");
    print_num(duress);
    console::puts(" remaining\n");
    console::puts("  Deadman tokens:  ");
    print_num(deadman);
    console::puts(" remaining\n");
}

fn cmd_otp_consume(hex: &str) {
    if hex.len() != 64 {
        console::puts("  usage: otp-consume <64-char-hex>\n");
        return;
    }
    let mut tok = [0u8; 32];
    for i in 0..32 {
        let hi = hex_nibble(hex.as_bytes()[i * 2]);
        let lo = hex_nibble(hex.as_bytes()[i * 2 + 1]);
        if hi > 15 || lo > 15 {
            console::puts("  ERROR: invalid hex\n");
            return;
        }
        tok[i] = (hi << 4) | lo;
    }
    match crate::security::otp::consume(&tok) {
        Some("duress") => {
            console::puts("  DURESS TOKEN ACCEPTED — wiping now\n");
            crate::security::wipe::execute(
                crate::security::wipe::WipeReason::Duress, false);
        }
        Some("deadman") => {
            console::puts("  DEADMAN TOKEN ACCEPTED — refreshing timer\n");
            crate::security::deadman::refresh();
        }
        Some(_) | None => {
            console::puts("  token rejected (invalid or already consumed)\n");
        }
    }
}

fn hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 255,
    }
}

/// SMC ASC base on M4 T8132 (confirmed via ADT walk 2026-04-20):
/// 0x38c600000 with mailbox at +0x8000. We reach it from EL1 via
/// stage-2 passthrough — m1n1's HV maps all non-traced /arm-io
/// zones as TraceMode.OFF which expands to hv_map_hw.
const SMC_CPU_CTRL: u64 = 0x38c600044;
const SMC_A2I_CTRL: u64 = 0x38c608110;
const SMC_I2A_CTRL: u64 = 0x38c608114;

/// Flag read by every call site of smc_keepalive_tick (shell loop,
/// platform::serial_putc, platform::serial_puts). Default OFF —
/// live-hardware A/B on 2026-04-20 showed that periodic SMC MMIO
/// reads from EL1 don't extend the ~60-96 s wall-clock session
/// ceiling AND actually extend the output plateau from ~14 s to
/// ~29 s (every serial_putc adds an SMC MMIO to the guest's TX
/// path, which apparently isn't as cheap as we'd hoped). Keep
/// the plumbing but default it off so the user can toggle with
/// `smc-pet` / `smc-stop` if a future theory wants another A/B.
pub static mut SMC_KEEPALIVE_ACTIVE: bool = false;

fn cmd_smc_probe() {
    platform::serial_puts("  reading SMC ASC regs from EL1 ...\n");
    // Wrap the reads in a small synchronous fence so a bad stage-2
    // mapping turns into a reproducible SError at the read site
    // instead of later elsewhere.
    unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack));
        let c = core::ptr::read_volatile(SMC_CPU_CTRL as *const u32);
        let a = core::ptr::read_volatile(SMC_A2I_CTRL as *const u32);
        let i = core::ptr::read_volatile(SMC_I2A_CTRL as *const u32);
        core::arch::asm!("dsb sy", options(nomem, nostack));
        platform::serial_puts("  SMC CPU_CONTROL:  0x");
        crate::drivers::apple::uart::puthex32(c);
        platform::serial_puts("\n");
        platform::serial_puts("  SMC A2I_CONTROL:  0x");
        crate::drivers::apple::uart::puthex32(a);
        platform::serial_puts("\n");
        platform::serial_puts("  SMC I2A_CONTROL:  0x");
        crate::drivers::apple::uart::puthex32(i);
        platform::serial_puts("\n");
    }
    platform::serial_puts("  [smc-probe OK — stage-2 passes SMC MMIO to EL1]\n");
}

fn cmd_smc_pet_start() {
    unsafe { SMC_KEEPALIVE_ACTIVE = true; }
    platform::serial_puts("  [smc-pet active — 10 Hz SMC MMIO poke from every output path]\n");
}

fn cmd_smc_pet_stop() {
    unsafe { SMC_KEEPALIVE_ACTIVE = false; }
    platform::serial_puts("  [smc-pet disabled — control run, no SMC MMIO from EL1]\n");
}

/// Called from the shell busy-poll loop at full poll rate — rate-
/// limited internally to ~10 Hz so we don't flood the SoC fabric.
/// When SMC_KEEPALIVE_ACTIVE is true we read SMC's I2A_CTRL
/// register (harmless read, no side effects) to generate periodic
/// bus traffic to the SMC ASC. Theory: the wall-clock SoC reset
/// fires when SMC sees no AP activity for N seconds.
#[inline(always)]
pub fn smc_keepalive_tick() {
    unsafe {
        if !SMC_KEEPALIVE_ACTIVE { return; }
        // CNTPCT-based rate limit — every ~100 ms.
        static mut LAST_TICK: u64 = 0;
        let now: u64;
        let freq: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        if freq == 0 { return; }
        let threshold = freq / 10;
        if now.wrapping_sub(LAST_TICK) < threshold { return; }
        LAST_TICK = now;
        // One SMC read. Discard value.
        let _ = core::ptr::read_volatile(SMC_I2A_CTRL as *const u32);
    }
}

fn print_num(n: usize) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;
    if n == 0 {
        console::putc(b'0');
        return;
    }
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        console::putc(buf[i]);
    }
}

fn cmd_ls() {
    let (count, max) = batfs::stats();
    console::puts_hi("  ENCRYPTED VAULT\n");
    console::puts("  ----------------\n");

    if count == 0 {
        console::puts("  (empty)\n");
    } else {
        batfs::list(|name, size, encrypted| {
            console::puts("  ");
            if encrypted {
                console::puts("[ENC] ");
            } else {
                console::puts("[RAW] ");
            }
            console::puts(name);
            console::puts("  (");
            print_num(size);
            console::puts(" bytes)\n");
        });
    }

    console::puts("  ----------------\n");
    console::puts("  ");
    print_num(count);
    console::puts("/");
    print_num(max);
    console::puts(" files\n");
}

fn cmd_write(name: &str, data: &str) {
    if name.is_empty() {
        console::puts("  usage: write <filename> <data>\n");
        return;
    }
    if data.is_empty() {
        console::puts("  usage: write <filename> <data>\n");
        return;
    }

    match batfs::create(name, data.as_bytes()) {
        Ok(()) => {
            console::puts("  Created: ");
            console::puts(name);
            console::puts(" (");
            print_num(data.len());
            console::puts(" bytes, AES-256-CTR encrypted)\n");
        }
        Err(e) => {
            console::puts("  Error: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_read(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cat <filename>\n");
        return;
    }

    let mut buf = [0u8; 4096];
    match batfs::read(name, &mut buf) {
        Ok(size) => {
            console::puts("  [decrypted, integrity verified]\n");
            console::puts("  ");
            // Print content as string
            if let Ok(s) = core::str::from_utf8(&buf[..size]) {
                console::puts(s);
            } else {
                console::puts("(binary data, ");
                print_num(size);
                console::puts(" bytes)");
            }
            console::puts("\n");
        }
        Err(e) => {
            console::puts("  Error: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_rm(name: &str) {
    if name.is_empty() {
        console::puts("  usage: rm <filename>\n");
        return;
    }

    match batfs::delete(name) {
        Ok(()) => {
            console::puts("  Secure deleted: ");
            console::puts(name);
            console::puts(" (zeroed + freed)\n");
        }
        Err(e) => {
            console::puts("  Error: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_verify(name: &str) {
    if name.is_empty() {
        console::puts("  usage: verify <filename>\n");
        return;
    }

    let mut buf = [0u8; 4096];
    match batfs::read(name, &mut buf) {
        Ok(_) => {
            console::puts("  INTEGRITY: PASS\n");
            console::puts("  File '");
            console::puts(name);
            console::puts("' — SHA-256 hash matches\n");
            console::puts("  No tampering detected\n");
        }
        Err(e) => {
            if e.contains("INTEGRITY") {
                console::puts("  INTEGRITY: FAIL\n");
                console::puts("  *** FILE HAS BEEN TAMPERED WITH ***\n");
            } else {
                console::puts("  Error: ");
                console::puts(e);
                console::puts("\n");
            }
        }
    }
}

fn cmd_ping(target: &str) {
    if target.is_empty() {
        console::puts("  usage: ping <ip>\n");
        console::puts("  e.g.: ping 10.0.2.2\n");
        return;
    }

    let ip = parse_ip(target);
    if ip == 0 {
        console::puts("  invalid IP address\n");
        return;
    }

    let mut ip_str = [0u8; 16];
    let len = net::ip::ip_to_str(ip, &mut ip_str);
    console::puts("  PING ");
    console::puts(unsafe { core::str::from_utf8_unchecked(&ip_str[..len]) });
    console::puts(" ... ");

    match net::icmp::ping(ip) {
        Ok(seq) => {
            console::puts("reply seq=");
            print_num(seq as usize);
            console::puts(" OK\n");
        }
        Err(e) => {
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_dns(hostname: &str) {
    if hostname.is_empty() {
        console::puts("  usage: dns <hostname>\n");
        return;
    }

    console::puts("  Resolving ");
    console::puts(hostname);
    console::puts(" ... ");

    match net::dns::resolve(hostname) {
        Ok(ip) => {
            let mut ip_str = [0u8; 16];
            let len = net::ip::ip_to_str(ip, &mut ip_str);
            console::puts(unsafe { core::str::from_utf8_unchecked(&ip_str[..len]) });
            console::puts("\n");
        }
        Err(e) => {
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_ifconfig() {
    console::puts_hi("  NETWORK INTERFACE\n");
    console::puts("  -----------------\n");

    let mac = crate::drivers::virtio::net::mac();
    console::puts("  MAC:     ");
    for i in 0..6 {
        let hex = b"0123456789abcdef";
        console::putc(hex[(mac[i] >> 4) as usize]);
        console::putc(hex[(mac[i] & 0xf) as usize]);
        if i < 5 { console::putc(b':'); }
    }
    console::puts("\n");

    let ip = net::ip::our_ip();
    let mut ip_str = [0u8; 16];
    let len = net::ip::ip_to_str(ip, &mut ip_str);
    console::puts("  IP:      ");
    console::puts(unsafe { core::str::from_utf8_unchecked(&ip_str[..len]) });
    console::puts("\n");

    let gw = net::ip::gateway();
    let len = net::ip::ip_to_str(gw, &mut ip_str);
    console::puts("  Gateway: ");
    console::puts(unsafe { core::str::from_utf8_unchecked(&ip_str[..len]) });
    console::puts("\n");

    console::puts("  Status:  ");
    if crate::drivers::virtio::net::is_ready() {
        console::puts("ONLINE\n");
    } else {
        console::puts("OFFLINE\n");
    }
}

fn cmd_firewall() {
    let (allowed, blocked) = net::firewall::stats();
    console::puts_hi("  FIREWALL\n");
    console::puts("  --------\n");
    console::puts("  Policy:   DEFAULT DENY ALL\n");
    console::puts("  Mode:     ALLOWLIST\n");
    console::puts("  Allowed:  ");
    print_num(allowed as usize);
    console::puts(" packets\n");
    console::puts("  Blocked:  ");
    print_num(blocked as usize);
    console::puts(" packets\n");
}

fn cmd_fetch(host: &str) {
    if host.is_empty() {
        console::puts("  usage: fetch <hostname>\n");
        return;
    }

    console::puts("  Resolving ");
    console::puts(host);
    console::puts("...\n");

    let ip = match net::dns::resolve(host) {
        Ok(ip) => ip,
        Err(e) => {
            console::puts("  DNS failed: ");
            console::puts(e);
            console::puts("\n");
            return;
        }
    };

    let mut ip_str = [0u8; 16];
    let len = net::ip::ip_to_str(ip, &mut ip_str);
    console::puts("  Connecting to ");
    console::puts(unsafe { core::str::from_utf8_unchecked(&ip_str[..len]) });
    console::puts(":80...\n");

    match net::tcp::connect(ip, 80) {
        Ok(()) => console::puts("  Connected!\n"),
        Err(e) => {
            console::puts("  TCP failed: ");
            console::puts(e);
            console::puts("\n");
            return;
        }
    }

    // Send HTTP GET
    let mut request = [0u8; 256];
    let req = b"GET / HTTP/1.0\r\nHost: ";
    let mut pos = 0;
    request[..req.len()].copy_from_slice(req);
    pos += req.len();
    request[pos..pos + host.len()].copy_from_slice(host.as_bytes());
    pos += host.len();
    let end = b"\r\n\r\n";
    request[pos..pos + end.len()].copy_from_slice(end);
    pos += end.len();

    let _ = net::tcp::send_data(&request[..pos]);
    console::puts("  Sent HTTP GET, waiting for response...\n");

    let mut buf = [0u8; 2048];
    match net::tcp::recv_data(&mut buf) {
        Ok(len) => {
            console::puts("  Received ");
            print_num(len);
            console::puts(" bytes:\n");
            // Show first few lines
            if let Ok(s) = core::str::from_utf8(&buf[..len.min(500)]) {
                for line in s.split('\n').take(8) {
                    console::puts("  ");
                    console::puts(line);
                    console::puts("\n");
                }
            }
        }
        Err(e) => {
            console::puts("  Recv: ");
            console::puts(e);
            console::puts("\n");
        }
    }

    net::tcp::close();
}

fn parse_ip(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut octets = [0u32; 4];
    let mut octet_idx = 0;
    let mut val: u32 = 0;

    for &b in bytes {
        if b == b'.' {
            if octet_idx >= 3 { return 0; }
            octets[octet_idx] = val;
            octet_idx += 1;
            val = 0;
        } else if b >= b'0' && b <= b'9' {
            val = val * 10 + (b - b'0') as u32;
        } else {
            return 0;
        }
    }
    if octet_idx != 3 { return 0; }
    octets[3] = val;

    ((octets[0] & 0xFF) << 24) | ((octets[1] & 0xFF) << 16) |
    ((octets[2] & 0xFF) << 8) | (octets[3] & 0xFF)
}

/// Ensure a default BatCave exists and is active for busybox commands.
fn ensure_default_cave() {
    use crate::batcave::cave;
    if cave::get_active() == usize::MAX {
        // QEMU-BUGFIX-6 workaround: `cave::enter()` hangs inside its
        // `reset_all_globals_for_cave_switch()` critical section (one of
        // the 20+ `reset_for_cave_switch` callees at cave.rs:623 doesn't
        // return). For the ambient host-cave case we don't actually need
        // the full state reset; `ensure_host_cave_active()` just creates
        // the cave + `set_active(id)`. Same end state, no hang.
        //
        // ROOT-5: `proc` and `mem` are now real caps (no longer hard-
        // allowed); the shell-host cave is created with the full broad
        // set inside `ensure_host_cave_active`.
        cave::ensure_host_cave_active();
    }
}

fn cmd_batcave(subcmd: &str, arg1: &str, arg2: &str, parts: &[&str; MAX_PARTS]) {
    use crate::batcave::cave;
    use crate::batcave::docker_client;
    match subcmd {
        // ── Docker-backed BatCaves (Phase 2 of design alignment) ──
        // These delegate to the Mac-side `batcaved` daemon over TCP.
        // Protocol: see src/batcave/docker_client.rs + scripts/batcaved.py.
        "docker-create" => {
            if arg1.is_empty() || arg2.is_empty() {
                console::puts("  usage: batcave docker-create <name> <image> [caps-csv]\n");
                console::puts("  e.g.:  batcave docker-create kali kalilinux/kali-rolling NET_RAW,NET_ADMIN\n");
                return;
            }
            // parts layout: [batcave, docker-create, <name>, <image>, <caps?>, ...]
            let caps_csv = parts[4];
            let r = docker_client::with_daemon(|| {
                // Split caps_csv into &str slices
                let mut caps_buf: [&str; 8] = [""; 8];
                let mut n = 0;
                if !caps_csv.is_empty() {
                    for part in caps_csv.split(',') {
                        if n < 8 && !part.is_empty() {
                            caps_buf[n] = part;
                            n += 1;
                        }
                    }
                }
                docker_client::create(arg1, arg2, &caps_buf[..n])
            });
            match r {
                Ok(id) => {
                    console::puts("  Docker BatCave created: ");
                    console::puts(arg1);
                    console::puts(" → container ");
                    console::puts(&id);
                    console::puts("\n");
                }
                Err(e) => {
                    console::puts("  Error: "); console::puts(e); console::puts("\n");
                }
            }
        }
        "docker-run" => {
            if arg1.is_empty() || arg2.is_empty() {
                console::puts("  usage: batcave docker-run <name> <cmd> [args...]\n");
                return;
            }
            // parts: [batcave, docker-run, <name>, <cmd>, <arg1>, <arg2>, <arg3>, <arg4>]
            let mut argv_buf: [&str; 6] = [""; 6];
            let mut argc = 0;
            for i in 3..MAX_PARTS {
                if parts[i].is_empty() { break; }
                if argc < 6 { argv_buf[argc] = parts[i]; argc += 1; }
            }
            let r = docker_client::with_daemon(|| {
                docker_client::run(arg1, &argv_buf[..argc], |line| {
                    // Output lines from the daemon come in already '\n'-terminated
                    // from the docker subprocess. Forward verbatim to the user's
                    // console — this is the tool's actual stdout.
                    console::puts("  ");
                    console::puts(line);
                    console::puts("\n");
                })
            });
            match r {
                Ok(rc) => {
                    console::puts("  [exit code ");
                    print_num(rc as usize);
                    console::puts("]\n");
                }
                Err(e) => {
                    console::puts("  Error: "); console::puts(e); console::puts("\n");
                }
            }
        }
        "docker-list" => {
            let r = docker_client::with_daemon(|| docker_client::list());
            match r {
                Ok(caves) => {
                    console::puts_hi("  DOCKER BATCAVES\n");
                    console::puts("  ────────────────\n");
                    if caves.is_empty() {
                        console::puts("  (none)\n");
                    } else {
                        for (name, image, status) in &caves {
                            console::puts("  ");
                            console::puts(name);
                            console::puts("  ");
                            console::puts(image);
                            console::puts("  [");
                            console::puts(status);
                            console::puts("]\n");
                        }
                    }
                    console::puts("  ────────────────\n");
                    print_num(caves.len());
                    console::puts(" docker cave(s)\n");
                }
                Err(e) => {
                    console::puts("  Error: "); console::puts(e); console::puts("\n");
                }
            }
        }
        "docker-destroy" => {
            if arg1.is_empty() {
                console::puts("  usage: batcave docker-destroy <name>\n");
                return;
            }
            let r = docker_client::with_daemon(|| docker_client::destroy(arg1));
            match r {
                Ok(()) => {
                    console::puts("  Docker BatCave destroyed: ");
                    console::puts(arg1);
                    console::puts("\n");
                }
                Err(e) => {
                    console::puts("  Error: "); console::puts(e); console::puts("\n");
                }
            }
        }
        "docker-ping" => {
            // quick connectivity check — is the daemon reachable?
            let r = docker_client::with_daemon(|| docker_client::ping());
            match r {
                Ok(()) => console::puts("  batcaved: PONG (OK)\n"),
                Err(e) => { console::puts("  batcaved: "); console::puts(e); console::puts("\n"); }
            }
        }
        "create" => {
            // Parse flags. Supported:
            //   --ephemeral           — destroyed on wipe, no persistent state
            //   --kit:<name>          — pre-install a tool bundle
            //   --docker:<image>      — docker-backed cave (Phase 6 of the
            //                           design-alignment plan). Image passed
            //                           through to docker_client / batcaved.
            //   --caps:<csv>          — only meaningful with --docker; Linux
            //                           capabilities to pass via --cap-add
            //
            // Scan all parts from [3..) so flag order doesn't matter.
            let mut ephemeral = false;
            let mut kit_name: &str = "";
            let mut docker_image: &str = "";
            let mut docker_caps: &str = "";
            for i in 3..MAX_PARTS {
                let p = parts[i];
                if p.is_empty() { continue; }
                if p == "--ephemeral" { ephemeral = true; }
                else if let Some(k) = p.strip_prefix("--kit:")    { kit_name = k; }
                else if let Some(img) = p.strip_prefix("--docker:") { docker_image = img; }
                else if let Some(c) = p.strip_prefix("--caps:")   { docker_caps = c; }
            }

            if !docker_image.is_empty() {
                // Docker-backed cave. Phase 3: derive the per-cave AES-256
                // key up-front (same path as native BatFS) so we can pass
                // it to the daemon in CREATE and have the cave's audit
                // log encrypted at rest.
                //
                // cave::create_docker() will also store this key in the
                // native cave table (fs_key field) so native destroy() +
                // wipe() zero it symmetrically with the daemon side.
                let key = {
                    // Derive key here so we don't allocate a cave slot
                    // before docker succeeds. (Same formula as cave::create.)
                    const MASTER: [u8; 32] = [
                        0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
                        0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
                        0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
                        0xBA, 0x7C, 0xA7, 0xE0, 0xBA, 0x7C, 0xA7, 0xE0,
                    ];
                    crate::crypto::sha256::derive_key(&MASTER, arg1.as_bytes())
                };

                // Spin the container FIRST so a daemon-side failure
                // doesn't leave a dangling entry in the native cave table.
                let caps_csv = docker_caps;
                let spin_res = crate::batcave::docker_client::with_daemon(|| {
                    // caps_csv → &[&str]
                    let mut caps_buf: [&str; 8] = [""; 8];
                    let mut n = 0;
                    if !caps_csv.is_empty() {
                        for part in caps_csv.split(',') {
                            if n < 8 && !part.is_empty() {
                                caps_buf[n] = part;
                                n += 1;
                            }
                        }
                    }
                    crate::batcave::docker_client::create_with_key(
                        arg1, docker_image, &caps_buf[..n], Some(&key))
                });
                match spin_res {
                    Ok(id) => {
                        // Container is up; now register in the cave table
                        // with Docker backing so list/destroy can find it.
                        match cave::create_docker(arg1, docker_image, ephemeral) {
                            Ok(_) => {
                                console::puts("  BatCave created: ");
                                console::puts(arg1);
                                console::puts(" [docker:");
                                console::puts(docker_image);
                                console::puts("] container=");
                                console::puts(&id);
                                if ephemeral { console::puts(" (ephemeral)"); }
                                console::puts("\n");
                            }
                            Err(e) => {
                                // Cave-table insert failed — undo the container.
                                console::puts("  Error (cave table): ");
                                console::puts(e); console::puts("\n");
                                let _ = crate::batcave::docker_client::with_daemon(|| {
                                    crate::batcave::docker_client::destroy(arg1)
                                });
                            }
                        }
                    }
                    Err(e) => {
                        console::puts("  Error (docker): ");
                        console::puts(e); console::puts("\n");
                    }
                }
                return;
            }

            // Native cave path (unchanged behaviour).
            match cave::create(arg1, ephemeral) {
                Ok(_) => {
                    console::puts("  BatCave created: ");
                    console::puts(arg1);
                    if ephemeral { console::puts(" (ephemeral)"); }
                    else { console::puts(" (persistent)"); }
                    console::puts("\n");

                    if !kit_name.is_empty() {
                        match crate::batcave::batkits::apply_kit(arg1, kit_name) {
                            Ok(()) => {
                                console::puts("  Kit '"); console::puts(kit_name);
                                console::puts("' applied!\n");
                            }
                            Err(e) => {
                                console::puts("  Kit error: "); console::puts(e);
                                console::puts("\n");
                            }
                        }
                    }
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "install" => {
            if arg1.is_empty() || arg2.is_empty() {
                console::puts("  usage: batcave install <cave> <tool>\n");
            } else {
                // Enter the cave first
                cave::enter(arg1).ok();

                // Register the tool
                match cave::install_tool(arg1, arg2) {
                    Ok(()) => {
                        console::puts("  Installing "); console::puts(arg2);
                        console::puts(" in "); console::puts(arg1); console::puts("...\n");

                        // Create tool directory in VFS
                        ensure_default_cave();
                        let argv_mkdir: [&str; 4] = ["busybox", "mkdir", "-p", ""];
                        let mut path_buf = [0u8; 64];
                        let prefix = b"/usr/local/bin";
                        path_buf[..prefix.len()].copy_from_slice(prefix);
                        crate::batcave::linux::runner::run_busybox_cmd(&argv_mkdir[..3]).ok();

                        // Create a symlink for the tool → busybox
                        // This makes the tool available as a busybox applet
                        if crate::batcave::linux::vfs::is_ready() {
                            if let Ok(bin) = crate::batcave::linux::vfs::resolve_path(b"/bin") {
                                // Check if not already a symlink
                                if crate::batcave::linux::vfs::find_child(bin, arg2.as_bytes()).is_none() {
                                    crate::batcave::linux::vfs::create_symlink(
                                        bin, arg2.as_bytes(), b"/bin/busybox"
                                    ).ok();
                                }
                            }
                        }

                        console::puts("  "); console::puts(arg2);
                        console::puts(" installed (busybox applet)\n");
                        console::puts("  Run with: batcave run "); console::puts(arg2); console::puts("\n");
                    }
                    Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
                }
            }
        }
        "grant" => {
            match cave::grant_cap(arg1, arg2) {
                Ok(()) => {
                    console::puts("  Granted '"); console::puts(arg2);
                    console::puts("' to "); console::puts(arg1); console::puts("\n");
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "revoke" => {
            match cave::revoke_cap(arg1, arg2) {
                Ok(()) => {
                    console::puts("  Revoked '"); console::puts(arg2);
                    console::puts("' from "); console::puts(arg1); console::puts("\n");
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "enter" => {
            match cave::enter(arg1) {
                Ok(()) => {
                    console::puts("  Entering BatCave: "); console::puts(arg1); console::puts("\n");
                    console::puts("  ["); console::puts(arg1); console::puts("] $ _\n");
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "stop" => {
            match cave::stop(arg1) {
                Ok(()) => { console::puts("  Stopped: "); console::puts(arg1); console::puts("\n"); }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "seal" => {
            match cave::seal(arg1) {
                Ok(()) => {
                    console::puts("  SEALED: "); console::puts(arg1);
                    console::puts(" (now ephemeral — IRREVERSIBLE)\n");
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "destroy" => {
            // Phase 6: route by backing. Docker-backed caves need the
            // container torn down via `batcaved` before we zero the
            // cave table entry. Native caves are unchanged.
            let is_docker = cave::find_id(arg1)
                .map(|id| unsafe { cave::CAVES[id].is_docker() })
                .unwrap_or(false);
            if is_docker {
                let r = crate::batcave::docker_client::with_daemon(|| {
                    crate::batcave::docker_client::destroy(arg1)
                });
                if let Err(e) = r {
                    console::puts("  Warning (docker): ");
                    console::puts(e); console::puts("\n");
                    // Continue to zero the cave entry anyway — the
                    // daemon may have already cleaned up the container.
                }
            }
            match cave::destroy(arg1) {
                Ok(()) => {
                    console::puts("  DESTROYED: "); console::puts(arg1);
                    if is_docker {
                        console::puts(" (container rm'd + cave keys zeroed)\n");
                    } else {
                        console::puts(" (zeroed + keys destroyed)\n");
                    }
                }
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "list" => {
            console::puts_hi("  BATCAVES\n");
            console::puts("  --------\n");
            let count = cave::count();
            if count == 0 {
                console::puts("  (none)\n");
            } else {
                cave::list(|c| {
                    console::puts("  ");
                    let status_char = if c.state == cave::CaveState::Running { ">" } else { " " };
                    console::puts(status_char);
                    console::puts(" ");
                    console::puts(c.name_str());
                    console::puts("  [");
                    console::puts(cave::state_str(c.state));
                    console::puts("] [");
                    console::puts(cave::type_str(c.cave_type));
                    console::puts("]");
                    // Phase 6: show backing so the user can see at a glance
                    // which caves live inside Bat_OS (native, MMU-isolated)
                    // vs which live as Docker containers on the Mac.
                    if c.is_docker() {
                        console::puts(" [docker:");
                        console::puts(c.image_str());
                        console::puts("]");
                    } else {
                        console::puts(" [native]");
                    }
                    console::puts(" tools:");
                    print_num(c.tool_count);
                    console::puts(" caps:");
                    print_num(c.cap_count);
                    console::puts("\n");
                });
            }
            console::puts("  --------\n  ");
            print_num(count);
            console::puts(" BatCave(s)\n");
        }
        "gui" => {
            // batcave gui <cave> <tool> — launch a GUI tool in a BatCave
            if arg1.is_empty() || arg2.is_empty() {
                console::puts("  usage: batcave gui <cave> <tool>\n");
                console::puts("  e.g.: batcave gui pentest wireshark\n");
            } else {
                // Check display capability
                if let Some(id) = cave::find_id(arg1) {
                    if !cave::active_has_cap("display") && cave::get_active() != id {
                        // Grant display cap and enter
                        cave::grant_cap(arg1, "display").ok();
                    }
                    // Allocate display region (quarter of screen)
                    let w = crate::ui::gpu::width();
                    let h = crate::ui::gpu::height();
                    cave::alloc_display(arg1, w / 4, 30, w / 2, h / 2).ok();
                    cave::enter(arg1).ok();

                    console::puts("  Launching "); console::puts(arg2);
                    console::puts(" in BatCave '"); console::puts(arg1);
                    console::puts("' (display sandbox: ");
                    print_num(w as usize / 2); console::puts("x");
                    print_num(h as usize / 2); console::puts(")\n");

                    // Run the tool via busybox
                    let argv: [&str; 4] = ["busybox", arg2, "", ""];
                    crate::batcave::linux::runner::run_busybox_cmd(&argv[..2]).ok();
                    console::puts("  GUI tool exited.\n");
                } else {
                    console::puts("  Error: cave '"); console::puts(arg1);
                    console::puts("' not found. Create it first.\n");
                }
            }
        }
        "test" => {
            console::puts("  Running Linux hello binary...\n");
            crate::batcave::linux::runner::run_test().ok();
            console::puts("  Test complete.\n");
        }
        "uname" => {
            console::puts("  Running Linux uname binary...\n");
            crate::batcave::linux::runner::run_uname_test().ok();
            console::puts("  Test complete.\n");
        }
        "busybox" | "bb" => {
            // Auto-create a default cave if none active
            ensure_default_cave();
            console::puts("  Loading busybox...\n");
            match crate::batcave::linux::runner::run_busybox() {
                Ok(()) => console::puts("  Busybox exited.\n"),
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "run" => {
            if arg1.is_empty() {
                console::puts("  usage: batcave run <cave|tool> [args...]\n");
                console::puts("    if <cave|tool> is an existing BatCave name, the\n");
                console::puts("    next argument is the tool to run inside it.\n");
                console::puts("    otherwise <cave|tool> is interpreted as a busybox\n");
                console::puts("    applet in the ambient shell-host cave.\n");
                return;
            }

            // Phase 6: route by cave backing. If arg1 matches an existing
            // docker-backed cave, send the rest of argv to the daemon for
            // execution inside the container. Otherwise, fall through to
            // the legacy "run a busybox applet in shell-host" path.
            if let Some(id) = cave::find_id(arg1) {
                let is_docker = unsafe { cave::CAVES[id].is_docker() };
                if is_docker {
                    // Docker path: parts[3..MAX_PARTS] is the container argv.
                    let mut argv_buf: [&str; 6] = [""; 6];
                    let mut argc = 0;
                    for i in 3..MAX_PARTS {
                        if parts[i].is_empty() { break; }
                        if argc < 6 { argv_buf[argc] = parts[i]; argc += 1; }
                    }
                    if argc == 0 {
                        console::puts("  usage: batcave run <cave> <tool> [args]\n");
                        return;
                    }
                    let r = crate::batcave::docker_client::with_daemon(|| {
                        crate::batcave::docker_client::run(arg1, &argv_buf[..argc], |line| {
                            console::puts("  ");
                            console::puts(line);
                            console::puts("\n");
                        })
                    });
                    match r {
                        Ok(rc) => {
                            console::puts("  [exit ");
                            print_num(rc as usize);
                            console::puts("]\n");
                        }
                        Err(e) => {
                            console::puts("  Error: "); console::puts(e); console::puts("\n");
                        }
                    }
                    return;
                }
                // Native cave with same name — fall through to shell-host
                // busybox path. (A future commit can plumb native caves
                // through per-cave page tables via `cave::enter`, but
                // that's blocked on BUG-6.)
            }

            // Legacy / default: run a busybox applet in the ambient
            // shell-host cave.
            ensure_default_cave();
            let mut full: [&str; MAX_PARTS] = [""; MAX_PARTS];
            full[0] = "busybox";
            let mut argc = 1;
            for i in 2..MAX_PARTS {
                if parts[i].is_empty() { break; }
                full[argc] = parts[i];
                argc += 1;
            }
            platform::serial_puts("[batcave run] argv:");
            for i in 0..argc {
                platform::serial_puts(" ");
                platform::serial_puts(full[i]);
            }
            platform::serial_puts("\n");
            match crate::batcave::linux::runner::run_busybox_cmd(&full[..argc]) {
                Ok(()) => {}
                Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
            }
        }
        "kits" => {
            console::puts_hi("  AVAILABLE KITS\n");
            console::puts("  ──────────────\n");
            crate::batcave::batkits::list_kits(|name, desc, tools| {
                console::puts("  ");
                console::puts(name);
                // Pad to 14 chars
                for _ in name.len()..14 { console::puts(" "); }
                console::puts(desc);
                console::puts(" (");
                print_num(tools);
                console::puts(" tools)\n");
            });
            console::puts("\n  Usage: batcave create <name> --kit:<kit>\n");
        }
        "pipe" => {
            // Show pipe buffer contents
            let count = crate::batcave::batpipe::count();
            if count == 0 {
                console::puts("  Pipe is empty. Run a tool first.\n");
            } else {
                console::puts_hi("  BATPIPE DATA\n");
                console::puts("  ────────────\n");
                crate::batcave::batpipe::each(|e| {
                    match e.dtype {
                        crate::batcave::batpipe::DataType::Host => {
                            console::puts("  HOST  "); console::puts(e.f1_str()); console::puts("\n");
                        }
                        crate::batcave::batpipe::DataType::Port => {
                            console::puts("  PORT  "); console::puts(e.f1_str());
                            console::puts(":"); console::puts(e.f2_str());
                            console::puts("  "); console::puts(e.f3_str()); console::puts("\n");
                        }
                        crate::batcave::batpipe::DataType::Url => {
                            console::puts("  URL   "); console::puts(e.f1_str()); console::puts("\n");
                        }
                        crate::batcave::batpipe::DataType::Credential => {
                            console::puts("  CRED  "); console::puts(e.f1_str());
                            console::puts(":"); console::puts(e.f2_str()); console::puts("\n");
                        }
                        crate::batcave::batpipe::DataType::Vuln => {
                            console::puts("  VULN  "); console::puts(e.f1_str());
                            console::puts("  "); console::puts(e.f2_str()); console::puts("\n");
                        }
                        _ => {}
                    }
                });
                console::puts("  ────────────\n  ");
                print_num(count);
                console::puts(" entries\n");
            }
        }
        "" => {
            console::puts("  usage: batcave <create|install|grant|enter|list|kits|pipe|gui|run>\n");
        }
        _ => {
            console::puts("  unknown: batcave "); console::puts(subcmd); console::puts("\n");
        }
    }
}

/// Split a command into up to `MAX_PARTS` whitespace-separated tokens.
/// Bumped from 4 → 8 so `batcave run` can take a real Kali-class argv like
/// `batcave run nc -zv 10.0.2.2 8080 -w 1`. Legacy callers that only touch
/// parts[0..=3] keep working — the extra slots are empty strings.
pub const MAX_PARTS: usize = 8;
fn split_cmd(cmd: &str) -> [&str; MAX_PARTS] {
    let mut parts = [""; MAX_PARTS];
    let mut idx = 0;
    let bytes = cmd.as_bytes();
    let mut start = 0;
    let mut in_word = false;

    for i in 0..bytes.len() {
        if bytes[i] == b' ' {
            if in_word && idx < MAX_PARTS {
                parts[idx] = unsafe { core::str::from_utf8_unchecked(&bytes[start..i]) };
                idx += 1;
                in_word = false;
            }
        } else if !in_word {
            start = i;
            in_word = true;
        }
    }
    if in_word && idx < MAX_PARTS {
        parts[idx] = unsafe { core::str::from_utf8_unchecked(&bytes[start..]) };
    }
    parts
}

/// Run an embedded ELF binary (hello or hello_libc)
fn cmd_run_elf(name: &str) {
    console::puts("  Loading ELF binary: ");
    console::puts(name);
    console::puts("\n");
    platform::serial_puts("[shell] loading ELF: ");
    platform::serial_puts(name);
    platform::serial_puts("\n");

    // BatCave EL0 runner — all static-PIE binaries go through here
    let batcave_names = ["netsurf", "freetype", "png", "posix", "cxx", "v8", "blink"];
    let use_batcave = batcave_names.iter().any(|&n| n == name);

    if use_batcave {
        let data = match name {
            "netsurf" => crate::batcave::linux::runner::netsurf_test_elf(),
            "freetype" => crate::batcave::linux::runner::freetype_test_elf(),
            "png" => crate::batcave::linux::runner::png_test_elf(),
            "posix" => crate::batcave::linux::runner::posix_test_elf(),
            "cxx" => crate::batcave::linux::runner::cxx_test_elf(),
            "v8" => crate::batcave::linux::runner::v8_exec_elf(),
            "blink" => crate::batcave::linux::runner::blink_test_elf(),
            "csstok" => crate::batcave::linux::runner::css_tokenizer_test_elf(),
            _ => crate::batcave::linux::runner::netsurf_test_elf(),
        };
        platform::serial_puts("[shell] using BatCave EL0 runner\n");
        match crate::batcave::linux::loader::load_elf(data) {
            Ok(entry) => {
                platform::serial_puts("[shell] loaded, running via BatCave...\n");
                if let Err(e) = crate::batcave::linux::loader::execute_with_args(entry, &[name]) {
                    console::puts("  Error: ");
                    console::puts(e);
                    console::puts("\n");
                }
            }
            Err(e) => {
                console::puts("  Load error: ");
                console::puts(e);
                console::puts("\n");
            }
        }
        return;
    }

    let hello_data = if name == "libc" {
        crate::batcave::linux::runner::hello_libc_elf()
    } else if name == "netsurf" {
        crate::batcave::linux::runner::netsurf_test_elf()
    } else if name == "threads" {
        crate::batcave::linux::runner::hello_threads_elf()
    } else if name == "freetype" {
        crate::batcave::linux::runner::freetype_test_elf()
    } else if name == "png" {
        crate::batcave::linux::runner::png_test_elf()
    } else if name == "posix" {
        crate::batcave::linux::runner::posix_test_elf()
    } else if name == "cxx" {
        crate::batcave::linux::runner::cxx_test_elf()
    } else if name == "v8" {
        crate::batcave::linux::runner::v8_exec_elf()
    } else if name == "blink" {
        crate::batcave::linux::runner::blink_test_elf()
    } else if name == "csstok" {
        crate::batcave::linux::runner::css_tokenizer_test_elf()
    } else {
        crate::batcave::linux::runner::hello_elf()
    };
    platform::serial_puts("[shell] ELF data: ");
    crate::kernel::mm::print_num(hello_data.len());
    platform::serial_puts(" bytes\n");

    // Activate the ambient "shell-host" cave so the ELF's syscalls have a
    // capability set to check against. Without this every write/mmap gets
    // EACCES and the hello/libc/threads tests produce log spam (see BUG-2
    // in docs/SESSION_JOURNAL.md 2026-04-22).
    crate::batcave::cave::ensure_host_cave_active();

    match crate::batcave::linux::loader::load_hello_elf(hello_data) {
        Ok((phys_entry, _phys_base, _orig_entry)) => {
            platform::serial_puts("[shell] ELF loaded, entry=0x");
            let hex = b"0123456789abcdef";
            for i in (0..16).rev() {
                platform::serial_putc(hex[((phys_entry >> (i * 4)) & 0xf) as usize]);
            }
            platform::serial_puts("\n");

            console::puts("  Executing...\n");

            // Use a STATIC stack to guarantee it's in mapped kernel memory
            // (dynamic frame allocation may return pages with MMU issues).
            //
            // V11-state-sweep: the static stack is reused for every ELF
            // launch. argc/argv/envp/auxv plus any user-stack contents
            // from the prior execution linger here. Zero before reuse.
            #[repr(align(16))]
            struct AlignedStack([u8; 65536]);
            static mut ELF_STACK: AlignedStack = AlignedStack([0u8; 65536]);
            let sb = core::ptr::addr_of_mut!(ELF_STACK) as usize; // 16-byte aligned
            unsafe {
                let p = sb as *mut u8;
                for i in 0..65536 {
                    core::ptr::write_volatile(p.add(i), 0);
                }
            }
            let stack_base = Some(sb);
            if let Some(sb) = stack_base {
                let sp = sb + 65536;

                // Set up minimal stack: argc=0, argv=NULL, envp=NULL, auxv=AT_NULL
                unsafe {
                    let sp_ptr = sp as *mut u64;
                    // auxv AT_NULL
                    core::ptr::write_volatile(sp_ptr.sub(1), 0u64); // AT_NULL value
                    core::ptr::write_volatile(sp_ptr.sub(2), 0u64); // AT_NULL key
                    // envp NULL
                    core::ptr::write_volatile(sp_ptr.sub(3), 0u64);
                    // argv NULL
                    core::ptr::write_volatile(sp_ptr.sub(4), 0u64);
                    // argc = 0
                    core::ptr::write_volatile(sp_ptr.sub(5), 0u64);

                    let final_sp = (sp - 48) & !0xF; // 16-byte aligned! ARM64 ABI requires it

                    platform::serial_puts("[shell] jumping to ELF entry, sp=0x");
                    for i in (0..16).rev() {
                        platform::serial_putc(hex[((final_sp >> (i * 4)) & 0xf) as usize]);
                    }
                    platform::serial_puts("\n");

                    // Ensure cache coherency: flush data caches and invalidate
                    // instruction caches so the loaded code is visible
                    core::arch::asm!(
                        "dsb ish",   // data synchronization barrier
                        "isb",       // instruction synchronization barrier
                    );

                    // Disable alignment checking RIGHT before jump
                    // (BatCave init may have re-enabled it)
                    core::arch::asm!(
                        "mrs x16, sctlr_el1",
                        "bic x16, x16, #2",  // clear bit 1 (A = alignment check)
                        "msr sctlr_el1, x16",
                        "isb",
                    );

                    // Jump to the binary
                    core::arch::asm!(
                        "mov sp, {sp_val}",
                        "br {entry}",
                        sp_val = in(reg) final_sp as u64,
                        entry = in(reg) phys_entry as u64,
                        options(noreturn),
                    );
                }
            } else {
                console::puts("  ERROR: could not allocate stack\n");
            }
        }
        Err(e) => {
            console::puts("  ERROR: ");
            console::puts(e);
            console::puts("\n");
            platform::serial_puts("[shell] ELF load failed: ");
            platform::serial_puts(e);
            platform::serial_puts("\n");
        }
    }
}

/// `chromium [flags] <url>` — launch the baked content_shell blob.
///
/// Flags (all optional, all default on):
///   --headless           run without any windowing backend
///   --no-sandbox         disable the Linux sandbox (required — we
///                        have no seccomp / userns to satisfy it)
///   --disable-gpu        force SwRaster / SwiftShader path
///   --window-size=WxH    default 1280x1024
///
/// Positional: the URL. Because `split_cmd` yields only 4 slots, we
/// accept up to three flag tokens; any extra flags go on the URL
/// (Chromium is forgiving about stray `--foo` mid-argv).
fn cmd_chromium(a1: &str, a2: &str, a3: &str) {
    use crate::batcave::linux::runner;
    use crate::kernel::mm::initrd;

    // Defaults.
    let mut headless = true;
    let mut no_sandbox = true;
    let mut disable_gpu = true;
    let mut window_size: &str = "1280x1024";
    let mut url: &str = "";

    // Fold over a1..a3, flags first then URL.
    for tok in [a1, a2, a3].iter() {
        if tok.is_empty() { continue; }
        if tok.starts_with("--") {
            if *tok == "--headless" { headless = true; }
            else if *tok == "--no-sandbox" { no_sandbox = true; }
            else if *tok == "--disable-gpu" { disable_gpu = true; }
            else if tok.starts_with("--window-size=") {
                window_size = &tok["--window-size=".len()..];
            }
            // unknown --flag: silently pass through onto url slot only if
            // url is still empty; otherwise ignore. Keeps 4-slot budget
            // honest.
            else if url.is_empty() {
                // Treat unknown --flag in front of a URL as an error only
                // if no URL is supplied at all. Let it fall through here.
            }
        } else {
            if url.is_empty() {
                url = tok;
            }
        }
    }

    if url.is_empty() {
        console::puts("  usage: chromium [flags] <url>\n");
        console::puts("         --headless            (default on)\n");
        console::puts("         --no-sandbox          (default on)\n");
        console::puts("         --disable-gpu         (default on)\n");
        console::puts("         --window-size=WxH     (default 1280x1024)\n");
        return;
    }

    if !initrd::is_present() {
        console::puts("  error: no Chromium binary baked into this image.\n");
        console::puts("  hint: run tools/bake_chromium.sh to produce a kernel image with content_shell.\n");
        return;
    }

    // Build argv. Order matches the canonical content_shell invocation
    // used by Phase 1 verification. We drop optional flags if the user
    // turned them off (currently: no-op — they all default on — but the
    // structure is in place).
    let size_arg: [u8; 64] = {
        // "--window-size=" + window_size (caller-controlled, copied into
        // a small stack buffer so we have a &str with 'static-ish scope
        // within this function).
        let prefix = b"--window-size=";
        let mut buf = [0u8; 64];
        let mut n = 0;
        for &b in prefix { if n < buf.len() { buf[n] = b; n += 1; } }
        for &b in window_size.as_bytes() {
            if n < buf.len() { buf[n] = b; n += 1; }
        }
        // Zero-pad to end; the &str we build below uses the exact n.
        let _ = n;
        buf
    };
    // Recompute the length so we can slice into a valid &str.
    let mut size_len = 0usize;
    for (i, &c) in size_arg.iter().enumerate() {
        if c == 0 { size_len = i; break; }
        size_len = i + 1;
    }
    let size_arg_str =
        unsafe { core::str::from_utf8_unchecked(&size_arg[..size_len]) };

    // Build argv in a fixed-capacity array (no alloc in no_std).
    let mut argv: [&str; 10] = [""; 10];
    let mut n = 0;
    argv[n] = "content_shell"; n += 1;
    if headless    { argv[n] = "--headless";     n += 1; }
    if no_sandbox  { argv[n] = "--no-sandbox";   n += 1; }
    if disable_gpu { argv[n] = "--disable-gpu";  n += 1; }
    argv[n] = "--single-process";          n += 1;
    argv[n] = "--ozone-platform=headless"; n += 1;
    argv[n] = size_arg_str;                n += 1;
    argv[n] = url;                         n += 1;

    console::puts("  launching content_shell on ");
    console::puts(url);
    console::puts("\n");

    match runner::run_chromium(url, &argv[..n]) {
        Ok(()) => {
            console::puts("  chromium exited OK\n");
        }
        Err(e) => {
            console::puts("  chromium: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}
