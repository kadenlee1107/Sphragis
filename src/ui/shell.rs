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
        "dump-dom" | "dom" => cmd_dump_dom(parts[1]),
        "render" => cmd_render(parts[1], &parts),
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
        "pq-selftest" => cmd_pq_selftest(),
        "pq-sig-selftest" => cmd_pq_sig_selftest(),
        "ipc-selftest"    => cmd_ipc_selftest(),
        "secure-ipc-selftest" => cmd_secure_ipc_selftest(),
        "secure-ipc-wire-selftest" => cmd_secure_ipc_wire_selftest(),
        "cave-policy-selftest" => cmd_cave_policy_selftest(),
        "cpol-list"   => cmd_cpol_list(),
        "cpol-show"   => cmd_cpol_show(parts[1]),
        "cpol-add"    => cmd_cpol_add(&parts[1..]),
        "cpol-add-sni" => cmd_cpol_add_sni(&parts[1..]),
        "cpol-check"  => cmd_cpol_check(&parts[1..]),
        "cpol-sni-selftest" => cmd_cpol_sni_selftest(),
        "cave-syscall-deny"  => cmd_cave_syscall_deny(&parts[1..]),
        "cave-syscall-allow" => cmd_cave_syscall_allow(&parts[1..]),
        "cave-syscall-list"  => cmd_cave_syscall_list(parts[1]),
        "cave-syscall-clear" => cmd_cave_syscall_clear(parts[1]),
        "cave-syscall-selftest" => cmd_cave_syscall_selftest(),
        "cave-seal-selftest"    => cmd_cave_seal_selftest(),
        "blk-status"            => cmd_blk_status(),
        "blk-selftest"          => cmd_blk_selftest(),
        "cpol-clear"  => cmd_cpol_clear(parts[1]),
        "cpol-sync"   => cmd_cpol_sync(parts[1]),
        "cpol-rate"       => cmd_cpol_rate(&parts[1..]),
        "cpol-rate-show"  => cmd_cpol_rate_show(parts[1]),
        "cpol-rate-list"  => cmd_cpol_rate_list(),
        "cpol-rate-clear" => cmd_cpol_rate_clear(parts[1]),
        "cpol-rate-selftest" => cmd_cpol_rate_selftest(),
        "cpol-byte-rate"     => cmd_cpol_byte_rate(&parts[1..]),
        "nat-beacons"        => cmd_nat_beacons(),
        "nat-beacon-selftest" => cmd_nat_beacon_selftest(),
        "nat-beacon-reset"   => cmd_nat_beacon_reset(),
        "cpol-flow-rate"     => cmd_cpol_flow_rate(&parts[1..]),
        "cpol-flow-rate-selftest" => cmd_cpol_flow_rate_selftest(),
        "cpol-daemon-list" => cmd_cpol_daemon_list(),
        "cpol-daemon-show" => cmd_cpol_daemon_show(parts[1]),
        "nic-status"  => cmd_nic_status(),
        "nat-selftest"=> cmd_nat_selftest(),
        "nat-rewrite-selftest" => cmd_nat_rewrite_selftest(),
        "nat-gc-selftest"      => cmd_nat_gc_selftest(),
        "nat-gc-force"         => cmd_nat_gc_force(),
        "nat-frag-selftest"    => cmd_nat_frag_selftest(),
        "nat-stats"   => cmd_nat_stats(),
        "nat-reset"   => cmd_nat_reset(),
        "nat-bind"    => cmd_nat_bind(&parts[1..]),
        "nat-bindings" => cmd_nat_bindings(),
        "nat-pump"    => cmd_nat_pump(),
        "nat-forward" => cmd_nat_forward(),
        "nat-reply"   => cmd_nat_reply(),
        "nat-table"   => cmd_nat_table(),
        "nat-sync"    => cmd_nat_sync(),
        "pq-tls-selftest" => cmd_pq_tls_selftest(),
        "batcave-fw-allow" => cmd_batcave_fw_allow(parts[1]),
        "batcave-fw-deny"  => cmd_batcave_fw_deny(parts[1]),
        "batcave-fw-list"  => cmd_batcave_fw_list(),
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

// Integration #4: Bat_OS pushes firewall rules to the daemon's egress proxy.
fn cmd_batcave_fw_allow(target: &str) {
    if target.is_empty() {
        console::puts("  usage: batcave-fw-allow <host:port>  (or *:port / *)\n");
        return;
    }
    let r = crate::batcave::docker_client::with_daemon(|| {
        crate::batcave::docker_client::fw_allow(target)
    });
    match r {
        Ok(()) => {
            console::puts("  [fw] ALLOW "); console::puts(target);
            console::puts("  → daemon proxy will tunnel CONNECTs to this target\n");
        }
        Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
    }
}
fn cmd_batcave_fw_deny(target: &str) {
    if target.is_empty() {
        console::puts("  usage: batcave-fw-deny <host:port>\n");
        return;
    }
    let r = crate::batcave::docker_client::with_daemon(|| {
        crate::batcave::docker_client::fw_deny(target)
    });
    match r {
        Ok(()) => {
            console::puts("  [fw] DENY "); console::puts(target); console::puts("\n");
        }
        Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
    }
}
fn cmd_batcave_fw_list() {
    let r = crate::batcave::docker_client::with_daemon(|| {
        crate::batcave::docker_client::fw_list()
    });
    match r {
        Ok(list) => {
            console::puts_hi("  BATCAVE EGRESS ALLOWLIST (daemon-enforced)\n");
            console::puts("  ----------------------------------------------\n");
            if list.is_empty() {
                console::puts("  (empty — DEFAULT DENY ALL)\n");
            } else {
                for t in &list {
                    console::puts("  "); console::puts(t); console::puts("\n");
                }
            }
            console::puts("  ----------------------------------------------\n  ");
            print_num(list.len());
            console::puts(" entries\n");
        }
        Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
    }
}

// Integration #3: hybrid PQ in TLS 1.3 key_share format.
fn cmd_pq_tls_selftest() {
    console::puts_hi("  TLS-HYBRID KEY-SHARE SELF-TEST\n");
    console::puts("  TLS 1.3 key_share format carrying X25519 + ML-KEM-768\n");
    console::puts("  (IETF codepoint 0x11EC — X25519MlKem768)\n");

    match crate::net::tls_hybrid::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS  full client/server round-trip\n");
            console::puts("    named_group:    0x");
            let hex = b"0123456789abcdef";
            for shift in (0..4).rev() {
                console::putc(hex[((r.group_code >> (shift * 4)) & 0xf) as usize]);
            }
            console::puts("\n    ClientHello ks: ");
            print_num(r.client_ks_bytes);
            console::puts(" bytes (2 group | 2 len | 1216 pub)\n");
            console::puts("    ServerHello ks: ");
            print_num(r.server_ks_bytes);
            console::puts(" bytes (2 group | 2 len | 1120 ct)\n");
            console::puts("    shared prefix:  ");
            for &b in &r.shared_prefix {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// Followup #2: handshake-over-wire + AEAD IPC proof (secure_ipc module).
fn cmd_secure_ipc_wire_selftest() {
    console::puts_hi("  SECURE-IPC WIRE-LEVEL SELF-TEST\n");
    console::puts("  handshake exchanged as IPC messages → AEAD-sealed traffic\n");
    match crate::batcave::secure_ipc::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS end-to-end mock-bus\n");
            console::puts("    handshake messages: ");
            print_num(r.handshake_msgs);
            console::puts(" (offer each side)\n");
            console::puts("    offer size:         ");
            print_num(r.offer_len);
            console::puts(" bytes (fits 1 ipc::Message = 256 B)\n");
            console::puts("    plaintext:          ");
            print_num(r.plaintext_len);
            console::puts(" bytes\n");
            console::puts("    sealed frame:       ");
            print_num(r.sealed_len);
            console::puts(" bytes (fits 1 ipc::Message)\n");
            console::puts("    plaintext round-trip OK — session key derived on both sides\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// Followup #3a: per-cave kernel egress policy store.
// Prove the kernel now owns the allow/deny brain for per-cave
// destinations — independent of the daemon's FW_ALLOWLIST dict.
fn cmd_cave_policy_selftest() {
    console::puts_hi("  CAVE-POLICY SELF-TEST\n");
    console::puts("  per-cave egress allowlist (default deny, hostname + port + proto)\n");
    match crate::net::cave_policy::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS kernel-side policy store\n");
            console::puts("    caves remaining:       ");
            print_num(r.caves_installed);
            console::puts(" (after clearing cave A)\n");
            console::puts("    allow-path checks:     ");
            print_num(r.allow_checks);
            console::puts("\n");
            console::puts("    drop-path checks:      ");
            print_num(r.drop_checks);
            console::puts("\n");
            console::puts("    cross-cave isolation:  ");
            console::puts(if r.cross_cave_isolation_ok { "OK\n" } else { "FAILED\n" });
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// Followup #3b: shell drivers for the per-cave policy store.
//
// `cpol-list` prints every cave the kernel knows about + rule count.
// `cpol-show  <name>` dumps the full rule list for a cave.
// `cpol-add   <name> <host> <port> <proto>` appends one allow rule.
//   proto: "tcp", "udp", "any".  port: 0 = any.
// `cpol-check <name> <host> <port> <proto>` runs the decision path
//   and prints ALLOW / DROP — useful for verifying the kernel sees
//   the same policy the daemon advertises.
// `cpol-clear <name>` drops the cave's entry entirely.

fn cpol_parse_proto(s: &str) -> Option<u8> {
    match s {
        "tcp"  | "TCP"  | "6"  => Some(6),
        "udp"  | "UDP"  | "17" => Some(17),
        "icmp" | "ICMP" | "1"  => Some(1),
        "any"  | "*"    | "0"  => Some(0),
        _ => None,
    }
}

fn cpol_parse_port(s: &str) -> Option<u16> {
    s.parse::<u16>().ok()
}

fn cpol_print_id(id: &[u8; 16]) {
    let hex = |n: u8| if n < 10 { b'0' + n } else { b'a' + n - 10 };
    for b in id.iter().take(4) {
        console::putc(hex(b >> 4));
        console::putc(hex(b & 0x0F));
    }
    console::puts("..");
}

fn cmd_cpol_list() {
    console::puts_hi("  CAVE-POLICY LIST (kernel view)\n");
    let entries = crate::net::cave_policy::list_all();
    if entries.is_empty() {
        console::puts("  (no caves registered)\n");
        return;
    }
    for (id, n) in entries.iter() {
        console::puts("  ");
        cpol_print_id(id);
        console::puts("  rules=");
        print_num(*n);
        console::puts("\n");
    }
    console::puts("    total caves: ");
    print_num(entries.len());
    console::puts("\n");
}

fn cmd_cpol_show(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-show <cave_name>\n");
        return;
    }
    let rules = crate::net::cave_policy::rules_for_by_name(name);
    console::puts_hi("  cpol-show ");
    console::puts(name);
    console::puts("\n");
    if rules.is_empty() {
        console::puts("    (no rules — default deny)\n");
        return;
    }
    for r in rules.iter() {
        console::puts("    allow ");
        let proto_tag = match r.proto { 6 => "tcp", 17 => "udp", _ => "any" };
        console::puts(proto_tag);
        console::puts("  ");
        console::puts(if r.host.is_empty() { "*" } else { r.host.as_str() });
        console::puts(":");
        if r.port == 0 { console::puts("*"); } else { print_num(r.port as usize); }
        console::puts("\n");
    }
}

fn cmd_cpol_add(args: &[&str]) {
    // args[0]=name, args[1]=host, args[2]=port, args[3]=proto
    if args.len() < 4 || args[0].is_empty() || args[3].is_empty() {
        console::puts("  usage: cpol-add <cave_name> <host> <port> <tcp|udp|any>\n");
        return;
    }
    let name  = args[0];
    let host  = args[1];
    let port  = match cpol_parse_port(args[2]) {
        Some(p) => p,
        None => { console::puts("  bad port\n"); return; }
    };
    let proto = match cpol_parse_proto(args[3]) {
        Some(p) => p,
        None => { console::puts("  bad proto (tcp|udp|any)\n"); return; }
    };
    use crate::net::cave_policy::EgressRule;
    use alloc::string::ToString;
    let rule = EgressRule {
        host: host.to_ascii_lowercase().to_string(),
        port,
        proto,
        sni: None,
    };
    crate::net::cave_policy::add_rule_by_name(name, rule);
    console::puts("  cpol-add ");
    console::puts(name);
    console::puts(" -> ");
    console::puts(host);
    console::puts(":");
    print_num(port as usize);
    console::puts("/");
    console::puts(args[3]);
    console::puts("  OK\n");
}

/// cpol-add-sni <cave> <ip> <port> <sni_host>
/// Adds a TCP allow rule pinned to a specific TLS SNI. Any
/// ClientHello on this (ip, port) tuple whose SNI doesn't match
/// `sni_host` gets DropSni.
fn cmd_cpol_add_sni(args: &[&str]) {
    if args.len() < 4 || args[0].is_empty() || args[3].is_empty() {
        console::puts("  usage: cpol-add-sni <cave> <ip> <port> <sni_host>\n");
        return;
    }
    let name = args[0];
    let host = args[1];
    let port = match cpol_parse_port(args[2]) {
        Some(p) => p,
        None => { console::puts("  bad port\n"); return; }
    };
    let sni = args[3];
    use crate::net::cave_policy::EgressRule;
    let rule = EgressRule::tcp_sni(host, port, sni);
    crate::net::cave_policy::add_rule_by_name(name, rule);
    console::puts("  cpol-add-sni ");
    console::puts(name);
    console::puts(" -> tcp ");
    console::puts(host);
    console::puts(":");
    print_num(port as usize);
    console::puts(" sni=");
    console::puts(sni);
    console::puts("  OK\n");
}

// ── Per-cave syscall denylist ──────────────────────────────────────

fn cmd_cave_syscall_deny(args: &[&str]) {
    if args.len() < 2 || args[0].is_empty() {
        console::puts("  usage: cave-syscall-deny <cave_name> <nr>\n");
        return;
    }
    let nr: u64 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => { console::puts("  bad syscall number\n"); return; }
    };
    let id = match crate::batcave::cave::find_id(args[0]) {
        Some(i) => i,
        None => { console::puts("  no such cave\n"); return; }
    };
    crate::batcave::syscall_filter::deny(id, nr);
    console::puts("  cave-syscall-deny ");
    console::puts(args[0]);
    console::puts(" nr="); print_num(nr as usize);
    console::puts("  OK\n");
}

fn cmd_cave_syscall_allow(args: &[&str]) {
    if args.len() < 2 || args[0].is_empty() {
        console::puts("  usage: cave-syscall-allow <cave_name> <nr>\n");
        return;
    }
    let nr: u64 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => { console::puts("  bad syscall number\n"); return; }
    };
    let id = match crate::batcave::cave::find_id(args[0]) {
        Some(i) => i,
        None => { console::puts("  no such cave\n"); return; }
    };
    crate::batcave::syscall_filter::allow(id, nr);
    console::puts("  cave-syscall-allow OK\n");
}

fn cmd_cave_syscall_list(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cave-syscall-list <cave_name>\n");
        return;
    }
    let id = match crate::batcave::cave::find_id(name) {
        Some(i) => i,
        None => { console::puts("  no such cave\n"); return; }
    };
    let mut buf = [0u16; 32];
    let n = crate::batcave::syscall_filter::list(id, &mut buf);
    console::puts_hi("  CAVE SYSCALL DENYLIST for ");
    console::puts(name); console::puts("\n");
    if n == 0 { console::puts("  (no denials)\n"); return; }
    for i in 0..n {
        console::puts("    nr="); print_num(buf[i] as usize); console::puts("\n");
    }
}

fn cmd_cave_syscall_clear(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cave-syscall-clear <cave_name>\n");
        return;
    }
    let id = match crate::batcave::cave::find_id(name) {
        Some(i) => i,
        None => { console::puts("  no such cave\n"); return; }
    };
    crate::batcave::syscall_filter::clear(id);
    console::puts("  cave-syscall-clear OK\n");
}

// virtio-blk (DESIGN.md Phase 3 gap) ───────────────────────────────

fn cmd_blk_status() {
    use crate::drivers::virtio::blk;
    console::puts_hi("  BLK STATUS\n");
    if !blk::is_ready() {
        console::puts("    device not present (QEMU needs -drive + virtio-blk-device)\n");
        return;
    }
    console::puts("    ready.  capacity: ");
    print_num(blk::capacity_sectors() as usize);
    console::puts(" sectors (");
    print_num((blk::capacity_sectors() as usize) / 2);
    console::puts(" KiB)\n");
}

fn cmd_blk_selftest() {
    use crate::drivers::virtio::blk;
    console::puts_hi("  BLK SELF-TEST (sector round-trip)\n");
    match blk::selftest() {
        Ok(r) if !r.ready => {
            console::puts("  - no block device; skipping\n");
        }
        Ok(r) => {
            console::puts("  ✓ PASS sector 42 write+read round-trip\n");
            console::puts("    capacity: ");
            print_num(r.capacity_sectors as usize);
            console::puts(" sectors\n");
            console::puts("    write ok: ");
            console::puts(if r.write_ok { "yes\n" } else { "no\n" });
            console::puts("    readback OK: ");
            console::puts(if r.readback_ok { "yes\n" } else { "no (MISMATCH)\n" });
            console::puts("    first byte of pattern: 0x");
            let hex = b"0123456789abcdef";
            console::putc(hex[((r.first_byte >> 4) & 0xF) as usize]);
            console::putc(hex[(r.first_byte & 0xF) as usize]);
            console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_cave_seal_selftest() {
    console::puts_hi("  CAVE-SEAL SELF-TEST (anti-coercion one-way ratchet)\n");
    match crate::batcave::cave::seal_selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS seal semantics\n");
            console::puts("    before: Persistent:   ");
            console::puts(if r.before_was_persistent { "yes\n" } else { "no\n" });
            console::puts("    after:  Ephemeral:    ");
            console::puts(if r.after_is_ephemeral { "yes\n" } else { "no\n" });
            console::puts("    fs_key zeroed:        ");
            console::puts(if r.fs_key_zeroed { "yes\n" } else { "no (INCORRECT)\n" });
            console::puts("    re-seal rejected:     ");
            console::puts(if r.reseal_rejected { "yes\n" } else { "no (INCORRECT)\n" });
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_cave_syscall_selftest() {
    console::puts_hi("  CAVE-SYSCALL-FILTER SELF-TEST\n");
    match crate::batcave::syscall_filter::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS per-cave denylist semantics\n");
            console::puts("    installed (dup suppressed): ");
            print_num(r.installed); console::puts("  (expected 3)\n");
            console::puts("    CONNECT (203) denied:       ");
            console::puts(if r.is_denied_203 { "yes\n" } else { "no\n" });
            console::puts("    GETSOCKNAME (204) allowed:  ");
            console::puts(if !r.is_denied_204 { "yes\n" } else { "no\n" });
            console::puts("    after clear: entries=");
            print_num(r.after_clear); console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_cpol_sni_selftest() {
    console::puts_hi("  CPOL-SNI SELF-TEST (TLS ClientHello parser)\n");
    match crate::net::nat::sni_selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS SNI parse + match\n");
            console::puts("    extracted SNI: "); console::puts(&r.parsed); console::puts("\n");
            console::puts("    matching rule accepted:    ");
            console::puts(if r.match_ok { "yes\n" } else { "no\n" });
            console::puts("    non-matching rule rejected:");
            console::puts(if r.mismatch_ok { " yes\n" } else { " no\n" });
            console::puts("    no-payload SYN admitted:   ");
            console::puts(if r.syn_admitted { "yes\n" } else { "no\n" });
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_cpol_check(args: &[&str]) {
    if args.len() < 4 || args[0].is_empty() || args[3].is_empty() {
        console::puts("  usage: cpol-check <cave_name> <host> <port> <tcp|udp|any>\n");
        return;
    }
    let name  = args[0];
    let host  = args[1];
    let port  = match cpol_parse_port(args[2]) {
        Some(p) => p,
        None => { console::puts("  bad port\n"); return; }
    };
    let proto = match cpol_parse_proto(args[3]) {
        Some(p) => p,
        None => { console::puts("  bad proto\n"); return; }
    };
    let v = crate::net::cave_policy::check_by_name(name, host, port, proto);
    console::puts("  cpol-check ");
    console::puts(name);
    console::puts(" ");
    console::puts(host);
    console::puts(":");
    print_num(port as usize);
    console::puts(" -> ");
    match v {
        crate::net::cave_policy::Verdict::Allow => console::puts("ALLOW\n"),
        crate::net::cave_policy::Verdict::Drop  => console::puts("DROP\n"),
    }
}

fn cmd_cpol_clear(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-clear <cave_name>\n");
        return;
    }
    crate::net::cave_policy::clear_by_name(name);
    console::puts("  cpol-clear ");
    console::puts(name);
    console::puts("  OK\n");
}

// Followup 3b-sync: push the kernel's cave_policy view of <cave> to
// the daemon's CAVE_POLICY_MIRROR. Round-trip walk: cpol-sync clears
// the daemon entry first, then pushes every rule. Result is that the
// daemon's mirror is byte-equivalent to the kernel's view.
fn cmd_cpol_sync(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-sync <cave_name>\n");
        return;
    }
    let rules = crate::net::cave_policy::rules_for_by_name(name);
    let n = rules.len();
    let r = crate::batcave::docker_client::with_daemon(|| {
        crate::batcave::docker_client::cpol_clear(name)?;
        for rule in rules.iter() {
            crate::batcave::docker_client::cpol_push(
                name,
                rule.host.as_str(),
                rule.port,
                rule.proto,
            )?;
        }
        Ok(())
    });
    match r {
        Ok(()) => {
            console::puts("  cpol-sync ");
            console::puts(name);
            console::puts(" -> daemon  (");
            print_num(n);
            console::puts(" rules pushed)\n");
        }
        Err(e) => {
            console::puts("  cpol-sync FAILED: "); console::puts(e); console::puts("\n");
        }
    }
}

fn cmd_cpol_daemon_list() {
    let r = crate::batcave::docker_client::with_daemon(
        crate::batcave::docker_client::cpol_list,
    );
    match r {
        Ok(caves) => {
            console::puts_hi("  CPOL DAEMON MIRROR (caves)\n");
            if caves.is_empty() {
                console::puts("  (daemon mirror is empty)\n");
                return;
            }
            for c in caves.iter() {
                console::puts("  "); console::puts(c.as_str()); console::puts("\n");
            }
            console::puts("    total: "); print_num(caves.len()); console::puts("\n");
        }
        Err(e) => {
            console::puts("  cpol-daemon-list FAILED: "); console::puts(e); console::puts("\n");
        }
    }
}

// ── Per-cave rate limiter (cave_shaper) shell drivers ─────────────

fn cmd_cpol_rate(args: &[&str]) {
    if args.len() < 3 || args[0].is_empty() {
        console::puts("  usage: cpol-rate <cave_name> <pkts/sec> <burst>\n");
        return;
    }
    let name = args[0];
    let pps: u32   = match args[1].parse() { Ok(n) => n, _ => { console::puts("  bad pps\n"); return; } };
    let burst: u32 = match args[2].parse() { Ok(n) => n, _ => { console::puts("  bad burst\n"); return; } };
    crate::net::cave_shaper::set_rate_by_name(name, pps, burst);
    console::puts("  cpol-rate ");
    console::puts(name);
    console::puts(" -> pps=");
    print_num(pps as usize);
    console::puts(" burst=");
    print_num(burst as usize);
    console::puts("  OK\n");
}

fn cmd_cpol_rate_show(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-rate-show <cave_name>\n");
        return;
    }
    match crate::net::cave_shaper::rate_for(name) {
        Some((pps, burst)) => {
            console::puts_hi("  cpol-rate-show "); console::puts(name); console::puts("\n");
            console::puts("    pps:   "); print_num(pps as usize); console::puts("\n");
            console::puts("    burst: "); print_num(burst as usize); console::puts("\n");
        }
        None => {
            console::puts("  "); console::puts(name);
            console::puts(" has no rate limit (unlimited)\n");
        }
    }
}

fn cmd_cpol_rate_list() {
    let entries = crate::net::cave_shaper::list();
    console::puts_hi("  CPOL RATE LIMITS\n");
    if entries.is_empty() {
        console::puts("  (no caves with rate limits)\n");
        return;
    }
    for (id, pps, burst, tok_now) in entries.iter() {
        console::puts("    ");
        let hex = b"0123456789abcdef";
        for b in id.iter().take(4) {
            console::putc(hex[(b >> 4) as usize]);
            console::putc(hex[(b & 0x0F) as usize]);
        }
        console::puts("..  pps=");
        print_num(*pps as usize);
        console::puts(" burst=");
        print_num(*burst as usize);
        console::puts(" tokens_now=");
        print_num(*tok_now as usize);
        console::puts("\n");
    }
}

/// cpol-flow-rate <cave> <pps> <burst>
/// Set the per-flow rate limit for all destinations this cave reaches.
/// pps=0 clears the default. Each (dst_ip, dst_port) gets its own
/// bucket lazy-allocated from this default.
fn cmd_cpol_flow_rate(args: &[&str]) {
    if args.len() < 3 || args[0].is_empty() {
        console::puts("  usage: cpol-flow-rate <cave> <pps> <burst>\n");
        return;
    }
    let pps: u32 = match args[1].parse() {
        Ok(n) => n, Err(_) => { console::puts("  bad pps\n"); return; }
    };
    let burst: u32 = match args[2].parse() {
        Ok(n) => n, Err(_) => { console::puts("  bad burst\n"); return; }
    };
    crate::net::flow_shaper::set_default_by_name(args[0], pps, burst);
    console::puts("  cpol-flow-rate "); console::puts(args[0]);
    console::puts(" -> per-flow pps="); print_num(pps as usize);
    console::puts(" burst="); print_num(burst as usize);
    console::puts("  OK\n");
}

fn cmd_cpol_flow_rate_selftest() {
    console::puts_hi("  CPOL-FLOW-RATE SELF-TEST (per-flow buckets)\n");
    match crate::net::flow_shaper::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS flow_shaper: per-destination independence\n");
            console::puts("    flow A allowed: "); print_num(r.flow_a_allowed as usize);
            console::puts("  (expected 5)\n");
            console::puts("    flow B allowed: "); print_num(r.flow_b_allowed as usize);
            console::puts("  (expected 5)\n");
            console::puts("    independent:    ");
            console::puts(if r.both_independently_capped { "yes\n" } else { "no\n" });
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_nat_beacons() {
    use crate::net::beacon;
    let flagged = beacon::list_flagged();
    console::puts_hi("  FLAGGED BEACONS (low-CoV periodic flows)\n");
    console::puts("    total samples recorded: ");
    print_num(beacon::total_samples() as usize);
    console::puts("\n    total flow-flags:       ");
    print_num(beacon::total_flags() as usize);
    console::puts("\n");
    if flagged.is_empty() {
        console::puts("    (no flows currently flagged)\n");
        return;
    }
    for (id, dst_ip, dst_port, mean, n) in flagged.iter() {
        console::puts("    cave=");
        let hex = b"0123456789abcdef";
        for b in id.iter().take(4) {
            console::putc(hex[(b >> 4) as usize]);
            console::putc(hex[(b & 0x0F) as usize]);
        }
        console::puts(".. dst=");
        let bs = [((dst_ip>>24)&0xFF) as u8, ((dst_ip>>16)&0xFF) as u8,
                  ((dst_ip>>8)&0xFF) as u8, (dst_ip&0xFF) as u8];
        for i in 0..4 {
            print_num(bs[i] as usize);
            if i < 3 { console::putc(b'.'); }
        }
        console::puts(":");
        print_num(*dst_port as usize);
        console::puts(" mean_ticks=");
        // mean can be huge — print in units of 1M ticks to keep readable
        print_num((*mean / 1_000_000) as usize);
        console::puts("M samples=");
        print_num(*n as usize);
        console::puts("\n");
    }
}

fn cmd_nat_beacon_selftest() {
    console::puts_hi("  NAT BEACON DETECTOR SELF-TEST\n");
    match crate::net::beacon::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS periodic vs jittery classification\n");
            console::puts("    beacon flagged: ");
            console::puts(if r.beacon_flagged { "yes\n" } else { "no\n" });
            console::puts("    jitter flagged: ");
            console::puts(if !r.jitter_flagged { "no (correct)\n" } else { "yes (INCORRECT)\n" });
            console::puts("    total_flags bumped: ");
            print_num(r.total_flags as usize); console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_nat_beacon_reset() {
    crate::net::beacon::reset();
    console::puts("  beacon detector: cleared\n");
}

/// cpol-byte-rate <cave> <bps> <byte_burst>
/// Companion to cpol-rate: limits by bytes/sec instead of pkts/sec.
/// Set either, both, or neither. 0 on both = unlimited bytes (the
/// pps limit, if any, still applies).
fn cmd_cpol_byte_rate(args: &[&str]) {
    if args.len() < 3 || args[0].is_empty() {
        console::puts("  usage: cpol-byte-rate <cave> <bytes/sec> <byte_burst>\n");
        return;
    }
    let bps: u32 = match args[1].parse() {
        Ok(n) => n, Err(_) => { console::puts("  bad bps\n"); return; }
    };
    let bb: u32 = match args[2].parse() {
        Ok(n) => n, Err(_) => { console::puts("  bad byte_burst\n"); return; }
    };
    crate::net::cave_shaper::set_byte_rate_by_name(args[0], bps, bb);
    console::puts("  cpol-byte-rate "); console::puts(args[0]);
    console::puts(" -> bps="); print_num(bps as usize);
    console::puts(" byte_burst="); print_num(bb as usize);
    console::puts("  OK\n");
}

fn cmd_cpol_rate_clear(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-rate-clear <cave_name>\n");
        return;
    }
    crate::net::cave_shaper::clear_rate_by_name(name);
    console::puts("  cpol-rate-clear "); console::puts(name);
    console::puts("  -> unlimited\n");
}

fn cmd_cpol_rate_selftest() {
    console::puts_hi("  CPOL-RATE SELF-TEST (token bucket)\n");
    match crate::net::cave_shaper::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS token-bucket semantics\n");
            console::puts("    allowed in burst:       ");
            print_num(r.allowed_in_burst as usize); console::puts("  (expected 5)\n");
            console::puts("    denied immediately:     ");
            print_num(r.denied_immediately as usize); console::puts(" (expected 15)\n");
            console::puts("    cross-cave unaffected:  ");
            console::puts(if r.cross_cave_unaffected { "yes\n" } else { "no\n" });
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

// Followup 3c-nat: parse IP octet fields
fn parse_ipv4(s: &str) -> Option<u32> {
    let mut ip: u32 = 0;
    let mut n = 0;
    for (i, part) in s.split('.').enumerate() {
        if i >= 4 { return None; }
        let oct: u32 = part.parse().ok()?;
        if oct > 255 { return None; }
        ip = (ip << 8) | oct;
        n += 1;
    }
    if n == 4 { Some(ip) } else { None }
}

fn cmd_nat_selftest() {
    console::puts_hi("  NAT SELF-TEST (packet-layer cave_policy gate)\n");
    match crate::net::nat::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS synthetic-frame classifier\n");
            console::puts("    allow:            "); print_num(r.allow as usize); console::puts("\n");
            console::puts("    drop-policy:      "); print_num(r.drop_policy as usize); console::puts("\n");
            console::puts("    drop-unknown-src: "); print_num(r.drop_unknown_src as usize); console::puts("\n");
            console::puts("    drop-parse:       "); print_num(r.drop_parse as usize); console::puts("\n");
            console::puts("    bindings:         "); print_num(r.bindings_installed); console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_nat_stats() {
    let s = crate::net::nat::stats();
    console::puts_hi("  NAT COUNTERS\n");
    console::puts("    allow:            "); print_num(s.allow as usize); console::puts("\n");
    console::puts("    drop-policy:      "); print_num(s.drop_policy as usize); console::puts("\n");
    console::puts("    drop-unknown-src: "); print_num(s.drop_unknown_src as usize); console::puts("\n");
    console::puts("    drop-parse:       "); print_num(s.drop_parse as usize); console::puts("\n");
    console::puts("    drop-fragment:    "); print_num(s.drop_fragment as usize); console::puts("\n");
    console::puts("    arp-replies:      "); print_num(s.arp_replies as usize); console::puts("\n");
    console::puts("    arp-ignored:      "); print_num(s.arp_ignored as usize); console::puts("\n");
    console::puts("    icmp-forwarded:   "); print_num(s.icmp_forwarded as usize); console::puts("\n");
    console::puts("    icmp-delivered:   "); print_num(s.icmp_delivered as usize); console::puts("\n");
    console::puts("    icmp-error-deliv: "); print_num(s.icmp_error_delivered as usize); console::puts("\n");
    console::puts("    nat-gc-evicted:   "); print_num(s.nat_gc_evicted as usize); console::puts("\n");
    console::puts("    host-frames-pass: "); print_num(s.host_frames_passed as usize); console::puts("\n");
    console::puts("    frag-reassembled: "); print_num(s.frag_reassembled as usize); console::puts("\n");
    console::puts("    frag-timeout:     "); print_num(s.frag_timeout as usize); console::puts("\n");
    console::puts("    frag-refragd:     "); print_num(s.frag_refragmented as usize); console::puts("\n");
    console::puts("    icmp-redir-drop:  "); print_num(s.icmp_redirect_dropped as usize); console::puts("\n");
    console::puts("    icmp-squench-drp: "); print_num(s.icmp_src_quench_dropped as usize); console::puts("\n");
    console::puts("    drop-rate:        "); print_num(s.drop_rate as usize); console::puts("\n");
    console::puts("    drop-sni:         "); print_num(s.drop_sni as usize); console::puts("\n");
}

fn cmd_nat_bind(args: &[&str]) {
    if args.len() < 2 || args[0].is_empty() {
        console::puts("  usage: nat-bind <ipv4> <cave_name>\n");
        return;
    }
    let Some(ip) = parse_ipv4(args[0]) else {
        console::puts("  bad IPv4\n"); return;
    };
    crate::net::nat::bind_ip(ip, args[1]);
    console::puts("  nat-bind "); console::puts(args[0]); console::puts(" -> ");
    console::puts(args[1]); console::puts("  OK\n");
}

fn cmd_nat_reset() {
    crate::net::nat::reset_stats();
    crate::net::nat::nat_table_clear();
    console::puts("  nat-reset: counters + table zeroed\n");
}

fn cmd_nat_frag_selftest() {
    console::puts_hi("  NAT FRAGMENT CLASSIFIER SELF-TEST\n");
    match crate::net::nat::fragment_selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS fragment detection distinct from parse-drop\n");
            console::puts("    drop-fragment: "); print_num(r.frag_count as usize); console::puts("\n");
            console::puts("    drop-parse:    "); print_num(r.parse_count as usize); console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_nat_gc_selftest() {
    console::puts_hi("  NAT GC SELF-TEST (TTL eviction per-proto)\n");
    match crate::net::nat::gc_selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS NAT TTL GC\n");
            console::puts("    entries before: "); print_num(r.entries_before); console::puts("\n");
            console::puts("    evicted:        "); print_num(r.evicted as usize); console::puts("\n");
            console::puts("    entries after:  "); print_num(r.entries_after); console::puts("\n");
            console::puts("    TCP kept fresh: ");
            console::puts(if r.kept_fresh { "yes\n" } else { "no\n" });
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn cmd_nat_gc_force() {
    let n = crate::net::nat::nat_gc_force(None);
    console::puts("  nat-gc-force: evicted ");
    print_num(n as usize);
    console::puts(" stale entries\n");
}

fn cmd_nat_rewrite_selftest() {
    console::puts_hi("  NAT REWRITE SELF-TEST (outbound → inbound round-trip)\n");
    match crate::net::nat::rewrite_selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS outbound rewrite + NAT table + inbound reverse\n");
            console::puts("    outbound dst_ip:  0x"); print_hex32(r.outbound_dst_ip); console::puts("\n");
            console::puts("    outbound src_ip:  0x"); print_hex32(r.outbound_src_ip); console::puts("  (nic 0)\n");
            console::puts("    outbound src_port: "); print_num(r.outbound_src_port as usize); console::puts("  (NAT eph)\n");
            console::puts("    inbound  dst_ip:  0x"); print_hex32(r.inbound_dst_ip); console::puts("  (cave)\n");
            console::puts("    inbound  dst_port: "); print_num(r.inbound_dst_port as usize); console::puts("  (cave orig)\n");
            console::puts("    NAT slots in use:  "); print_num(r.nat_slots_in_use); console::puts("\n");
        }
        Err(e) => { console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n"); }
    }
}

fn print_hex32(v: u32) {
    let hex = b"0123456789abcdef";
    for s in (0..8).rev() {
        let nyb = ((v >> (s * 4)) & 0xF) as usize;
        console::putc(hex[nyb]);
    }
}

fn cmd_nat_forward() {
    use crate::drivers::virtio::net;
    let nic0_mac = net::mac_n(0);
    // Static for now: nic 0 = slirp 10.0.2.15, gateway 10.0.2.2 has a
    // virtual MAC the slirp host never actually advertises (QEMU just
    // accepts anything). Use 52:55:0a:00:02:02 as a conventional slirp
    // GW MAC; real value is irrelevant because slirp is L4-NAT itself.
    let nic0_ip:  u32 = 0x0A00020F;
    let gw_mac = [0x52, 0x55, 0x0A, 0x00, 0x02, 0x02];
    let (drained, forwarded) = crate::net::nat::pump_and_forward(nic0_ip, nic0_mac, gw_mac);
    console::puts("  nat-forward: drained "); print_num(drained);
    console::puts(" → forwarded "); print_num(forwarded); console::puts(" on nic 0\n");
}

fn cmd_nat_reply() {
    use crate::drivers::virtio::net;
    let nic1_mac = net::mac_n(1);
    let (drained, delivered) = crate::net::nat::pump_replies(nic1_mac);
    console::puts("  nat-reply: drained "); print_num(drained);
    console::puts(" from nic 0 → delivered "); print_num(delivered);
    console::puts(" on nic 1\n");
}

fn cmd_nat_table() {
    let n = crate::net::nat::nat_table_size();
    console::puts_hi("  NAT TABLE\n");
    console::puts("    entries active: "); print_num(n); console::puts("\n");
}

/// Pull (ip, cave) bindings from batcaved's CAVE_NET_IP table and
/// mirror them into nat::bind_ip. This is how containers' actual
/// Docker-bridge IPs get known to the kernel's packet classifier.
fn cmd_nat_sync() {
    let r = crate::batcave::docker_client::with_daemon(
        crate::batcave::docker_client::cpol_bind_list,
    );
    match r {
        Ok(binds) => {
            let mut installed = 0usize;
            for (ip_s, cave) in binds.iter() {
                if let Some(ip) = parse_ipv4(ip_s) {
                    crate::net::nat::bind_ip(ip, cave);
                    installed += 1;
                }
            }
            console::puts_hi("  nat-sync (daemon → kernel IP bindings)\n");
            console::puts("    pulled:    "); print_num(binds.len()); console::puts("\n");
            console::puts("    installed: "); print_num(installed); console::puts("\n");
        }
        Err(e) => {
            console::puts("  nat-sync FAILED: "); console::puts(e); console::puts("\n");
        }
    }
}

fn cmd_nat_pump() {
    let n = crate::net::nat::pump();
    console::puts("  nat-pump: drained ");
    print_num(n);
    console::puts(" frames from nic 1\n");
    let s = crate::net::nat::stats();
    console::puts("    after: allow=");  print_num(s.allow as usize);
    console::puts(" drop-policy="); print_num(s.drop_policy as usize);
    console::puts(" drop-unk-src="); print_num(s.drop_unknown_src as usize);
    console::puts(" drop-parse="); print_num(s.drop_parse as usize);
    console::puts("\n");
}

fn cmd_nat_bindings() {
    let bs = crate::net::nat::list_bindings();
    console::puts_hi("  NAT IP BINDINGS\n");
    if bs.is_empty() { console::puts("  (no bindings)\n"); return; }
    for (ip, cave) in bs.iter() {
        let b = [
            ((ip >> 24) & 0xFF) as u8,
            ((ip >> 16) & 0xFF) as u8,
            ((ip >>  8) & 0xFF) as u8,
            ( ip        & 0xFF) as u8,
        ];
        console::puts("    ");
        for i in 0..4 {
            print_num(b[i] as usize);
            if i < 3 { console::putc(b'.'); }
        }
        console::puts(" -> ");
        console::puts(cave.as_str());
        console::puts("\n");
    }
}

// Followup 3c-multinic: multi-NIC observability.
// Prints how many virtio-net NICs came up + MAC of each. nic 0 is
// historically the host/slirp control plane; nic 1 (when present)
// is the caves-side interface the NAT forwarder owns.
fn cmd_nic_status() {
    use crate::drivers::virtio::net;
    console::puts_hi("  NIC STATUS\n");
    let n = net::count();
    console::puts("    brought up: ");
    print_num(n as usize);
    console::puts(" of max ");
    print_num(net::MAX_NICS);
    console::puts("\n");
    for id in 0..net::MAX_NICS {
        if !net::is_ready_n(id) { continue; }
        let mac = net::mac_n(id);
        console::puts("    nic ");
        print_num(id);
        console::puts(":  ready  mac=");
        let hex = b"0123456789abcdef";
        for i in 0..6 {
            let b = mac[i];
            console::putc(hex[(b >> 4) as usize]);
            console::putc(hex[(b & 0xF) as usize]);
            if i < 5 { console::putc(b':'); }
        }
        match id {
            0 => console::puts("  (host / slirp)\n"),
            1 => console::puts("  (caves / packet pipeline)\n"),
            _ => console::puts("\n"),
        }
    }
}

fn cmd_cpol_daemon_show(name: &str) {
    if name.is_empty() {
        console::puts("  usage: cpol-daemon-show <cave_name>\n");
        return;
    }
    let r = crate::batcave::docker_client::with_daemon(|| {
        crate::batcave::docker_client::cpol_show(name)
    });
    match r {
        Ok(rules) => {
            console::puts_hi("  cpol-daemon-show ");
            console::puts(name);
            console::puts("\n");
            if rules.is_empty() {
                console::puts("    (no mirror entries for this cave)\n");
                return;
            }
            for (host, port, proto) in rules.iter() {
                console::puts("    allow ");
                let tag = match *proto { 6 => "tcp", 17 => "udp", _ => "any" };
                console::puts(tag);
                console::puts("  ");
                console::puts(if host.is_empty() { "*" } else { host.as_str() });
                console::puts(":");
                if *port == 0 { console::puts("*"); } else { print_num(*port as usize); }
                console::puts("\n");
            }
        }
        Err(e) => {
            console::puts("  cpol-daemon-show FAILED: "); console::puts(e); console::puts("\n");
        }
    }
}

// Integration #1: secure_channel on top of ipc_session.
fn cmd_secure_ipc_selftest() {
    console::puts_hi("  SECURE-IPC END-TO-END SELF-TEST\n");
    console::puts("  handshake → session key → AEAD-framed channel\n");
    console::puts("  (confidentiality + integrity + replay resistance)\n");

    match crate::batcave::secure_channel::selftest() {
        Ok(r) => {
            console::puts("  ✓ PASS\n");
            console::puts("    plaintext:      ");
            print_num(r.plaintext_len); console::puts(" bytes\n");
            console::puts("    wire frame:     ");
            print_num(r.frame_len); console::puts(" bytes (+");
            print_num(r.expansion); console::puts(" AEAD overhead)\n");
            console::puts("    round 1 match:  "); console::puts(if r.round_1_matched {"✓"} else {"✗"}); console::puts("\n");
            console::puts("    round 2 match:  "); console::puts(if r.round_2_matched {"✓"} else {"✗"}); console::puts("\n");
            console::puts("    tamper rejected:"); console::puts(if r.tamper_rejected {"✓"} else {"✗"}); console::puts("\n");
            console::puts("    replay rejected:"); console::puts(if r.replay_rejected {"✓"} else {"✗"}); console::puts("\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// DESIGN_CRYPTO.md #10+#13: Noise-style IPC session handshake self-test.
fn cmd_ipc_selftest() {
    console::puts_hi("  INTER-CAVE IPC SESSION SELF-TEST\n");
    console::puts("  Ed25519 identity + X25519 ephemeral, mutual auth + FS\n");
    console::puts("  Simulating Alice ↔ Bob handshake round-trip...\n");

    match crate::batcave::ipc_session::selftest_round_trip() {
        Ok((a_pfx, b_pfx, matched)) => {
            if matched {
                console::puts("  ✓ PASS  both sides derived identical 32-byte session key\n");
            } else {
                console::puts("  ✗ FAIL  session keys disagree\n");
            }
            let hex = b"0123456789abcdef";
            console::puts("    Alice key prefix: ");
            for &b in &a_pfx {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n    Bob   key prefix: ");
            for &b in &b_pfx {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n");
            console::puts("    Forward-secret (ephemerals discarded), mutually authenticated\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// DESIGN_CRYPTO.md #6: post-quantum hybrid signature self-test.
fn cmd_pq_sig_selftest() {
    console::puts_hi("  POST-QUANTUM HYBRID SIGNATURE SELF-TEST\n");
    console::puts("  Ed25519 + ML-DSA-65  (classical + PQ signatures)\n");
    console::puts("  Keygen + sign + verify + tamper-detection round trip...\n");

    match crate::crypto::pq_hybrid_sig::selftest() {
        Ok((pub_len, sig_len, _prefix)) => {
            console::puts("  ✓ PASS  verify OK + tamper rejected on BOTH halves\n");
            console::puts("    hybrid public key: ");
            print_num(pub_len);
            console::puts(" bytes (32 Ed25519 pub + 1952 ML-DSA-65 pub)\n");
            console::puts("    hybrid signature:  ");
            print_num(sig_len);
            console::puts(" bytes (64 Ed25519 sig + 3309 ML-DSA-65 sig)\n");
            console::puts("    Unforgeable under classical AND quantum attack\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// DESIGN_CRYPTO.md #5: post-quantum hybrid self-test.
fn cmd_pq_selftest() {
    console::puts_hi("  POST-QUANTUM HYBRID SELF-TEST\n");
    console::puts("  X25519 + ML-KEM-768  (classical + PQ key exchange)\n");
    console::puts("  Running encap/decap round trip...\n");

    match crate::crypto::pq_hybrid::selftest() {
        Ok((blob_len, prefix)) => {
            console::puts("  ✓ PASS  shared secrets match on both sides\n");
            console::puts("    hybrid ciphertext size: ");
            print_num(blob_len);
            console::puts(" bytes (32 X25519 pub + 1088 ML-KEM-768 ct)\n");
            console::puts("    shared-secret prefix: ");
            let hex = b"0123456789abcdef";
            for &b in &prefix {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n");
            console::puts("    Security: safe against classical AND quantum adversaries\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
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
            let mut persistent_vol = false;
            for i in 3..MAX_PARTS {
                let p = parts[i];
                if p.is_empty() { continue; }
                if p == "--ephemeral" { ephemeral = true; }
                else if p == "--persistent" { persistent_vol = true; }
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
                    if persistent_vol {
                        crate::batcave::docker_client::create_persistent(
                            arg1, docker_image, &caps_buf[..n], &key)
                    } else {
                        crate::batcave::docker_client::create_with_key(
                            arg1, docker_image, &caps_buf[..n], Some(&key))
                    }
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

                                // Followup 3c-daemon-bind: pull the fresh
                                // ip→cave binding the daemon just learned
                                // from docker inspect, so the kernel's
                                // packet classifier knows about this cave
                                // before the container starts talking.
                                let sync = crate::batcave::docker_client::with_daemon(
                                    crate::batcave::docker_client::cpol_bind_list,
                                );
                                if let Ok(binds) = sync {
                                    let mut n = 0usize;
                                    for (ip_s, cave_n) in binds.iter() {
                                        if let Some(ip) = parse_ipv4(ip_s) {
                                            crate::net::nat::bind_ip(ip, cave_n);
                                            n += 1;
                                        }
                                    }
                                    console::puts("    nat-sync: ");
                                    print_num(n);
                                    console::puts(" IP bindings\n");
                                }
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
            } else if cave::find_id(arg1)
                .map(|id| unsafe { cave::CAVES[id].is_docker() })
                .unwrap_or(false)
            {
                // Docker-backed cave: dispatch to the daemon, which will
                // `docker exec apt-get install` (or apk / dnf) inside the
                // container. This is the path that gets real Kali tools
                // into real containers — busybox symlink trick below only
                // works for native caves.
                console::puts("  Installing "); console::puts(arg2);
                console::puts(" in "); console::puts(arg1);
                console::puts(" via daemon apt/apk...\n");
                let r = crate::batcave::docker_client::with_daemon(|| {
                    crate::batcave::docker_client::install_tool(arg1, arg2)
                });
                match r {
                    Ok(()) => {
                        // Also register in the cave table so `batcave list`
                        // shows it.
                        let _ = cave::install_tool(arg1, arg2);
                        console::puts("  "); console::puts(arg2);
                        console::puts(" installed in Docker cave\n");
                    }
                    Err(e) => { console::puts("  Error: "); console::puts(e); console::puts("\n"); }
                }
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
    let mut dump_dom = false;
    let mut window_size: &str = "1280x1024";
    let mut url: &str = "";

    // Fold over a1..a3, flags first then URL.
    for tok in [a1, a2, a3].iter() {
        if tok.is_empty() { continue; }
        if tok.starts_with("--") {
            if *tok == "--headless" { headless = true; }
            else if *tok == "--no-sandbox" { no_sandbox = true; }
            else if *tok == "--disable-gpu" { disable_gpu = true; }
            else if *tok == "--dump-dom" { dump_dom = true; }
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
        console::puts("         --dump-dom            (print parsed DOM to stdout)\n");
        console::puts("         --window-size=WxH     (default 1280x1024)\n");
        console::puts("\n  built-in test URL: `chromium file:///bin/hello.html`\n");
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
    // argv[0] is the full VFS path to the executable — lets Chromium's
    // PathService::Get(DIR_EXE, ...) resolve to /bin when /proc/self/exe
    // isn't available. (Doesn't currently unlock ICU — that path
    // specifically wants a pre-set fd via base::i18n::SetIcuFile.
    // But passing a real path costs nothing and helps other PathService
    // lookups that cascade off argv[0].)
    let mut argv: [&str; 20] = [""; 20];
    let mut n = 0;
    argv[n] = "/bin/content_shell"; n += 1;
    if headless    { argv[n] = "--headless";     n += 1; }
    if no_sandbox  { argv[n] = "--no-sandbox";   n += 1; }
    if disable_gpu { argv[n] = "--disable-gpu";  n += 1; }
    if dump_dom    { argv[n] = "--dump-dom";     n += 1; }
    argv[n] = "--single-process";          n += 1;
    argv[n] = "--ozone-platform=headless"; n += 1;
    // 2026-04-30: turn off the storage / IPC subsystems that spin
    // on /tmp/.config/content_shell/* sqlite-shm files we can't
    // currently keep open. Each Disable* feature here was observed
    // in the smoke logs as a source of unbounded retries that
    // prevented FileURLLoader::Start from completing.
    argv[n] = "--disable-features=SharedDictionary,SharedDictionaryAPI,DIPS,LevelDBProto,UseDnsHttpsSvcb"; n += 1;
    argv[n] = "--user-data-dir=/dev/shm/cs"; n += 1;
    argv[n] = "--no-startup-window";        n += 1;
    argv[n] = "--enable-logging=stderr";   n += 1;
    argv[n] = "--v=1";                     n += 1;
    // STUMP #35 honors V8's CodeRange hint, eliminating V8 OOM. New
    // ceiling: PA / MessagePumpEpoll crashes around 1.2K lines.
    argv[n] = size_arg_str;                n += 1;
    argv[n] = url;                         n += 1;

    console::puts("  launching content_shell on ");
    console::puts(url);
    console::puts("\n");

    // Reset the skip ring so this run produces an independent map
    // of failures (no stale events from a prior chromium invocation).
    crate::batcave::linux::skip_log::reset();

    match runner::run_chromium(url, &argv[..n]) {
        Ok(()) => {
            crate::drivers::uart::puts("  chromium exited OK\n");
        }
        Err(e) => {
            crate::drivers::uart::puts("  chromium: ");
            crate::drivers::uart::puts(e);
            crate::drivers::uart::puts("\n");
        }
    }

    // Always dump the skip-summary AFTER the cave finishes (whether
    // successful, errored, or terminated by terminate_cave_fatal).
    // Parser-friendly markers `[SKIP-SUMMARY ...]` / `[SKIP-DETAIL
    // ...]` let scripts/agents pull the failure tree out in one pass.
    crate::batcave::linux::skip_log::dump_summary();
}

/// `dump-dom <url-or-path>` — parse HTML through Bat_OS's own engine
/// and print the DOM tree to UART. No Chromium, no IPC, no livelock —
/// just our `browser::html::parser` + `browser::dom::Document`. This is
/// the path to first DOM render: bypass content_shell entirely.
///
/// Accepted URL forms:
///   - `file:///bin/foo.html`     → `archive_file("bin/foo.html")`
///   - `/bin/foo.html`            → same as above (leading slash trimmed)
///   - `bin/foo.html`             → archive lookup
fn cmd_dump_dom(url: &str) {
    use crate::browser::dom::{Document, NodeType};
    use crate::browser::html;
    use crate::drivers::uart;
    use crate::kernel::mm::initrd;

    if url.is_empty() {
        crate::drivers::uart::puts("  usage: dump-dom <url|path>\n");
        crate::drivers::uart::puts("  e.g.   dump-dom file:///bin/hello.html\n");
        crate::drivers::uart::puts("         dump-dom /bin/hello.html\n");
        return;
    }

    // Normalize: strip "file://", leading "/", ".".
    let mut path = url;
    if let Some(rest) = path.strip_prefix("file://") { path = rest; }
    while path.starts_with('/') { path = &path[1..]; }

    let bytes = match initrd::archive_file(path) {
        Some(b) => b,
        None => {
            crate::drivers::uart::puts("  dump-dom: not in archive: ");
            crate::drivers::uart::puts(path);
            crate::drivers::uart::puts("\n");
            return;
        }
    };

    crate::drivers::uart::puts("  dump-dom: read ");
    crate::kernel::mm::print_num(bytes.len());
    crate::drivers::uart::puts(" bytes from ");
    crate::drivers::uart::puts(path);
    crate::drivers::uart::puts("\n");

    // Parse with our native engine. Document is a multi-MB arena, so
    // reuse the same static DOM_DOC the visual browser uses (in BSS,
    // not the stack — Box::new(Document::new()) overflows the kernel
    // stack before it ever copies into the heap).
    let doc: &mut Document = unsafe {
        &mut *core::ptr::addr_of_mut!(crate::ui::apps::browser::DOM_DOC)
    };
    html::parser::parse(bytes, doc);

    crate::drivers::uart::puts("  dump-dom: parsed ");
    crate::kernel::mm::print_num(doc.len());
    crate::drivers::uart::puts(" node(s)\n");

    // Walk and print. Indented tree — element opens print "<tag attr=\"v\">",
    // children are indented +2, closes print "</tag>", text nodes
    // print verbatim with indent.
    fn walk(doc: &Document, idx: usize, depth: usize) {
        use crate::drivers::uart;
        let n = doc.get(idx);
        for _ in 0..depth { uart::putc(b' '); uart::putc(b' '); }
        match n.node_type {
            NodeType::Document => {
                crate::drivers::uart::puts("#document\n");
            }
            NodeType::Element => {
                uart::putc(b'<');
                crate::drivers::uart::puts(n.tag_str());
                for i in 0..n.attr_count {
                    uart::putc(b' ');
                    uart::puts(n.attrs[i].name_str());
                    uart::puts("=\"");
                    uart::puts(n.attrs[i].value_str());
                    uart::putc(b'"');
                }
                crate::drivers::uart::puts(">\n");
            }
            NodeType::Text => {
                let t = n.text_str();
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    uart::puts("\"");
                    uart::puts(trimmed);
                    uart::puts("\"\n");
                } else {
                    // Suppress whitespace-only text — matches what
                    // chromium --dump-dom does for readability.
                    return;
                }
            }
            NodeType::Comment => {
                crate::drivers::uart::puts("<!-- ");
                crate::drivers::uart::puts(n.text_str());
                crate::drivers::uart::puts(" -->\n");
            }
            NodeType::Empty => return,
        }
        // Recurse into children
        for c in doc.children(idx) {
            walk(doc, c, depth + 1);
        }
        // Closing tag for elements
        if n.node_type == NodeType::Element {
            for _ in 0..depth { uart::putc(b' '); uart::putc(b' '); }
            crate::drivers::uart::puts("</");
            crate::drivers::uart::puts(n.tag_str());
            crate::drivers::uart::puts(">\n");
        }
    }

    crate::drivers::uart::puts("=== DOM ===\n");
    walk(&doc, 0, 0);
    crate::drivers::uart::puts("=== END ===\n");
}

/// `render <url>` — parse HTML, run layout, paint into a software
/// framebuffer (in BSS), and dump the resulting pixels over the UART
/// as a base64-encoded RGBA stream. The companion host-side script
/// `scripts/render_to_png.py` decodes that into a PNG file you can
/// open. Lets you SEE the page Bat_OS renders, without needing the
/// host to have a working virtio-gpu under HVF.
fn cmd_render(url: &str, parts: &[&str; MAX_PARTS]) {
    use crate::browser::dom::Document;
    use crate::browser::{html, layout, paint};
    use crate::drivers::uart;
    use crate::kernel::mm::initrd;

    if url.is_empty() {
        crate::drivers::uart::puts("  usage: render <url|path> [type=<id>=<value>] ...\n");
        crate::drivers::uart::puts("  e.g.   render file:///bin/login.html type=email=foo@bar.com type=pw=hunter2\n");
        return;
    }

    // STUMP #92/#94: render directly from a live URL — http:// is plain
    // TCP, https:// is TLS 1.3 (with strict-pinning relaxed for the
    // duration of the fetch; see net/fetch.rs SECURITY comment). file://
    // is unchanged.
    static mut HTML_FETCH_BUF: [u8; 256 * 1024] = [0; 256 * 1024];
    let bytes: &[u8] = if url.starts_with("http://") || url.starts_with("https://") {
        crate::drivers::uart::puts("  render: fetching "); uart::puts(url); uart::puts("\n");
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(HTML_FETCH_BUF) };
        match crate::net::fetch::fetch_url(url, buf) {
            Ok(n) => {
                crate::drivers::uart::puts("  render: fetched ");
                crate::kernel::mm::print_num(n);
                crate::drivers::uart::puts(" bytes\n");
                &buf[..n]
            }
            Err(e) => {
                crate::drivers::uart::puts("  render: fetch failed: ");
                crate::drivers::uart::puts(e); uart::puts("\n");
                return;
            }
        }
    } else {
        let mut path = url;
        if let Some(rest) = path.strip_prefix("file://") { path = rest; }
        while path.starts_with('/') { path = &path[1..]; }
        match initrd::archive_file(path) {
            Some(b) => b,
            None => {
                crate::drivers::uart::puts("  render: not in archive: ");
                crate::drivers::uart::puts(path); uart::puts("\n");
                return;
            }
        }
    };

    // Use the URL itself as the "path" for the success log line.
    let path = url;

    // Backing framebuffer — capped at 1900px tall. The host-side
    // session that reads back the PNG cannot accept images >2000px
    // in either dimension; exceeding it corrupts the conversation.
    // Pages taller than the cap get truncated (their bottom rows
    // clip). Render once at small height to verify a full page
    // fits, paginate via separate URLs if not.
    const MAX_W: u32 = 1024;
    const MAX_H: u32 = 1900;
    const FB_LEN: usize = (MAX_W * MAX_H) as usize;
    static mut RENDER_FB: [u32; FB_LEN] = [0u32; FB_LEN];

    // Take the visual browser's DOM_DOC + LAYOUT_TREE statics. They
    // already exist in BSS (~5 MB combined); reusing avoids stack
    // overflow from Box::new(Document::new()).
    let doc: &mut Document = unsafe {
        &mut *core::ptr::addr_of_mut!(crate::ui::apps::browser::DOM_DOC)
    };
    let tree: &mut layout::LayoutTree = unsafe {
        &mut *core::ptr::addr_of_mut!(crate::ui::apps::browser::LAYOUT_TREE)
    };

    crate::drivers::uart::puts("  render: read ");
    crate::kernel::mm::print_num(bytes.len());
    crate::drivers::uart::puts(" bytes from "); uart::puts(path); uart::puts("\n");

    html::parser::parse(bytes, doc);
    crate::drivers::uart::puts("  render: parsed "); crate::kernel::mm::print_num(doc.len());
    crate::drivers::uart::puts(" nodes\n");

    // STUMP #90: apply any `type=<id>=<value>` shell args BEFORE layout
    // and JS execution. The renderer is one-shot today (no event loop
    // yet), so we simulate user typing by mutating the DOM's `value`
    // attribute on matching <input>/<textarea> nodes. The existing
    // input layout already prefers `value` over `placeholder`, so the
    // typed text shows up in the screenshot.
    //
    // STUMP #93: a sibling `click=<id>` flag appends the matched
    // node's `onclick` attribute (inline JS) onto doc.js_text so the
    // existing pre-layout JS execution path runs it. With STUMP #93's
    // method-call fix in place, attribute handlers like
    //   onclick="document.getElementById('out').innerText='clicked'"
    // round-trip cleanly.
    for pi in 2..MAX_PARTS {
        let arg = parts[pi];
        if let Some(payload) = arg.strip_prefix("type=") {
            let eq = match payload.find('=') {
                Some(e) => e,
                None => {
                    uart::puts("  render: bad type= arg (need id=value): ");
                    uart::puts(arg); uart::puts("\n");
                    continue;
                }
            };
            let id = &payload[..eq];
            let val = &payload[eq + 1..];
            match doc.find_by_id(id) {
                Some(idx) => {
                    doc.nodes[idx].set_attr("value", val);
                    uart::puts("  render: typed into #");
                    uart::puts(id); uart::puts(" = ");
                    uart::puts(val); uart::puts("\n");
                }
                None => {
                    uart::puts("  render: no element with id="); uart::puts(id);
                    uart::puts("\n");
                }
            }
            continue;
        }
        if let Some(id) = arg.strip_prefix("click=") {
            // Snapshot the onclick body into a stack-local buffer so
            // we can release the immutable borrow on doc before we
            // mutate doc.js_text below.
            let mut handler = [0u8; 1024];
            let mut hlen = 0usize;
            let found = match doc.find_by_id(id) {
                Some(idx) => {
                    if let Some(oc) = doc.nodes[idx].get_attr("onclick") {
                        let n = oc.len().min(handler.len());
                        handler[..n].copy_from_slice(&oc.as_bytes()[..n]);
                        hlen = n;
                        true
                    } else {
                        uart::puts("  render: #"); uart::puts(id);
                        uart::puts(" has no onclick attribute\n");
                        false
                    }
                }
                None => {
                    uart::puts("  render: no element with id="); uart::puts(id);
                    uart::puts("\n"); false
                }
            };
            if found && hlen > 0 {
                let avail = crate::browser::dom::MAX_JS - doc.js_len;
                let need = hlen + 2; // body + ";\n"
                if avail >= need {
                    doc.js_text[doc.js_len..doc.js_len + hlen]
                        .copy_from_slice(&handler[..hlen]);
                    doc.js_len += hlen;
                    doc.js_text[doc.js_len] = b';';
                    doc.js_text[doc.js_len + 1] = b'\n';
                    doc.js_len += 2;
                    uart::puts("  render: queued onclick for #");
                    uart::puts(id); uart::puts("\n");
                } else {
                    uart::puts("  render: js buffer full, click skipped\n");
                }
            }
            continue;
        }
    }

    // STUMP #88: fetch any captured `<link rel=stylesheet>` URLs over
    // HTTP and append the bodies onto doc.css_text. The sheet matcher
    // already reads from css_text, so the rules light up automatically
    // on next layout pass. Failures are logged + ignored — a missing
    // stylesheet shouldn't block the render entirely.
    for li in 0..doc.link_count {
        let url_len = doc.link_lens[li] as usize;
        let url_bytes = &doc.link_urls[li][..url_len];
        let url = match core::str::from_utf8(url_bytes) {
            Ok(s) => s,
            Err(_) => { uart::puts("  render: link href not utf-8, skip\n"); continue; }
        };
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            // file:// hrefs get loaded via the regular initrd path
            // through layout; relative URLs aren't resolved yet.
            continue;
        }
        crate::drivers::uart::puts("  render: fetching link "); uart::puts(url); uart::puts("\n");
        let avail = crate::browser::dom::MAX_CSS - doc.css_len;
        if avail == 0 { uart::puts("  render: css buffer full, skip\n"); break; }
        // Append directly into the tail of css_text.
        let dst_start = doc.css_len;
        let dst_end = doc.css_len + avail;
        match crate::net::fetch::fetch_url(url, &mut doc.css_text[dst_start..dst_end]) {
            Ok(n) => {
                doc.css_len += n;
                crate::drivers::uart::puts("  render: fetched ");
                crate::kernel::mm::print_num(n);
                crate::drivers::uart::puts(" bytes of CSS\n");
            }
            Err(e) => {
                crate::drivers::uart::puts("  render: link fetch failed: ");
                crate::drivers::uart::puts(e);
                crate::drivers::uart::puts("\n");
            }
        }
    }

    // 🎯 STUMP #84+#86: run captured <script> content through the
    // JS engine BEFORE layout, so DOM mutations are reflected in
    // the render. STUMP #86 capped the sibling walks in compile_node
    // so a malformed AST can't hang the compiler.
    if doc.js_len > 0 {
        crate::drivers::uart::puts("  render: running ");
        crate::kernel::mm::print_num(doc.js_len);
        crate::drivers::uart::puts(" bytes of JS\n");
        static mut JS_VM: crate::browser::js::vm::Vm =
            crate::browser::js::vm::Vm::new();
        let vm: &mut crate::browser::js::vm::Vm =
            unsafe { &mut *core::ptr::addr_of_mut!(JS_VM) };
        // STUMP #93: hand the DOM ptr to the JS DOM API so
        // document.getElementById / setAttribute / appendChild etc.
        // resolve against the same Document the renderer just parsed.
        // Without this, getElementById returns null and onclick
        // handlers silently no-op.
        crate::browser::js::dom_api::set_document(doc);
        vm.init();
        let src_bytes = &doc.js_text[..doc.js_len];
        match vm.execute(src_bytes) {
            Ok(_) => {
                if vm.console_len > 0 {
                    uart::puts("=== JS console ===\n");
                    let cb = unsafe {
                        core::slice::from_raw_parts(
                            core::ptr::addr_of!(vm.console_buf) as *const u8,
                            vm.console_len,
                        )
                    };
                    for &b in cb { uart::putc(b); }
                    if !cb.last().map(|&b| b == b'\n').unwrap_or(false) {
                        uart::puts("\n");
                    }
                    uart::puts("=== /JS console ===\n");
                }
            }
            Err(_) => {
                crate::drivers::uart::puts("  render: JS execution error\n");
                if vm.console_len > 0 {
                    uart::puts("=== JS console (partial) ===\n");
                    let cb = unsafe {
                        core::slice::from_raw_parts(
                            core::ptr::addr_of!(vm.console_buf) as *const u8,
                            vm.console_len,
                        )
                    };
                    for &b in cb { uart::putc(b); }
                    uart::puts("\n=== /JS console ===\n");
                }
            }
        }
    }

    let rw: u32 = 800; // viewport width — same as default browsers' first guess
    layout::build(doc, tree, rw as i32);
    crate::drivers::uart::puts("  render: laid out "); crate::kernel::mm::print_num(tree.box_count);
    crate::drivers::uart::puts(" boxes (page_height=");
    crate::kernel::mm::print_num(tree.page_height as usize);
    crate::drivers::uart::puts(")\n");

    // STUMP #97 — Sprint 1.1: coordinate hit-testing via `click_xy=x,y`.
    // The renderer is still one-shot, but we can simulate a real mouse
    // click by hit-testing the post-layout tree, walking up to find an
    // ancestor with an `onclick` attribute, queuing that JS, re-running
    // the engine, and re-laying-out. Multiple click_xy args fire in
    // order. Each one triggers another JS+layout pass — heavy but fine
    // for development; the live event loop (Sprint 1.4) will batch
    // events and run the engine once per frame.
    // Debug: `dump_layout=1` arg prints box coords for hit-test debugging.
    // Sprint 1.4: `live=1` blits the rendered framebuffer onto the real
    // virtio-gpu FB and flushes, so a user looking at the QEMU window
    // sees the page (rather than only the base64 PNG dump). Pairs well
    // with `-display cocoa`/`-display gtk` instead of `-display none`.
    let mut dump_layout = false;
    let mut live_mode = false;
    for pi in 2..MAX_PARTS {
        if parts[pi] == "dump_layout=1" { dump_layout = true; }
        if parts[pi] == "live=1" { live_mode = true; }
    }
    if dump_layout {
        for i in 0..tree.box_count.min(30) {
            let b = &tree.boxes[i];
            if !b.active { continue; }
            let dn = b.dom_node as usize;
            let tag = if dn < doc.node_count {
                unsafe { core::str::from_utf8_unchecked(&doc.nodes[dn].tag[..doc.nodes[dn].tag_len.min(16)]) }
            } else { "?" };
            crate::drivers::uart::puts("  box ");
            crate::kernel::mm::print_num(i);
            crate::drivers::uart::puts(" <"); uart::puts(tag); uart::puts(">");
            crate::drivers::uart::puts(" x="); crate::kernel::mm::print_num(b.x.max(0) as usize);
            crate::drivers::uart::puts(" y="); crate::kernel::mm::print_num(b.y.max(0) as usize);
            crate::drivers::uart::puts(" w="); crate::kernel::mm::print_num(b.width.max(0) as usize);
            crate::drivers::uart::puts(" h="); crate::kernel::mm::print_num(b.height.max(0) as usize);
            if dn < doc.node_count && doc.nodes[dn].get_attr("onclick").is_some() {
                crate::drivers::uart::puts(" [onclick]");
            }
            crate::drivers::uart::puts("\n");
        }
    }

    let mut had_click_xy = false;
    for pi in 2..MAX_PARTS {
        let arg = parts[pi];
        let payload = match arg.strip_prefix("click_xy=") {
            Some(p) => p,
            None => continue,
        };
        let comma = match payload.find(',') {
            Some(c) => c,
            None => {
                crate::drivers::uart::puts("  render: bad click_xy= arg (need x,y): ");
                crate::drivers::uart::puts(arg); uart::puts("\n");
                continue;
            }
        };
        let qx: i32 = match payload[..comma].parse() {
            Ok(v) => v,
            Err(_) => { uart::puts("  render: bad click_xy x\n"); continue; }
        };
        let qy: i32 = match payload[comma + 1..].parse() {
            Ok(v) => v,
            Err(_) => { uart::puts("  render: bad click_xy y\n"); continue; }
        };

        let hit = match tree.hit_test(qx, qy) {
            Some(idx) => idx,
            None => {
                crate::drivers::uart::puts("  render: click_xy hit nothing at ");
                crate::kernel::mm::print_num(qx as usize);
                crate::drivers::uart::puts(",");
                crate::kernel::mm::print_num(qy as usize);
                crate::drivers::uart::puts("\n");
                continue;
            }
        };

        // Walk up from the hit box looking for an onclick attribute.
        // Most click handlers are on <button>/<a>/<input>, not the
        // text-node child the user actually pointed at.
        let owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
            n.get_attr("onclick").is_some()
        });

        match owner {
            Some(box_idx) => {
                let dom_idx = tree.boxes[box_idx].dom_node as usize;
                // Snapshot the onclick body, then mutate doc.js_text.
                let mut handler = [0u8; 1024];
                let mut hlen = 0usize;
                if let Some(oc) = doc.nodes[dom_idx].get_attr("onclick") {
                    let n = oc.len().min(handler.len());
                    handler[..n].copy_from_slice(&oc.as_bytes()[..n]);
                    hlen = n;
                }
                if hlen > 0 {
                    let avail = crate::browser::dom::MAX_JS - doc.js_len;
                    let need = hlen + 2;
                    if avail >= need {
                        doc.js_text[doc.js_len..doc.js_len + hlen]
                            .copy_from_slice(&handler[..hlen]);
                        doc.js_len += hlen;
                        doc.js_text[doc.js_len] = b';';
                        doc.js_text[doc.js_len + 1] = b'\n';
                        doc.js_len += 2;
                        uart::puts("  render: click_xy(");
                        crate::kernel::mm::print_num(qx as usize);
                        uart::puts(",");
                        crate::kernel::mm::print_num(qy as usize);
                        uart::puts(") hit box ");
                        crate::kernel::mm::print_num(hit);
                        uart::puts(" → onclick on box ");
                        crate::kernel::mm::print_num(box_idx);
                        uart::puts("\n");
                        had_click_xy = true;
                    } else {
                        uart::puts("  render: js buffer full, click_xy skipped\n");
                    }
                }
            }
            None => {
                // Sprint 1.3: <form> submit. If the click hit a submit
                // button (<input type=submit>, <input type=image>, or
                // <button type=submit>/<button> inside a <form>), walk
                // up to the enclosing <form>, gather all <input
                // name=value> children into a urlencoded body, and
                // POST to the form's action URL. The response replaces
                // the current document.
                let submit_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
                    let t = n.tag_str();
                    if t == "button" {
                        // Default <button> type is "submit" per HTML spec.
                        let bt = n.get_attr("type").unwrap_or("submit");
                        bt == "submit"
                    } else if t == "input" {
                        let it = n.get_attr("type").unwrap_or("text");
                        it == "submit" || it == "image"
                    } else { false }
                });
                if let Some(box_idx) = submit_owner {
                    // Climb the DOM (not the layout) to find the form.
                    let mut form_dom: Option<usize> = None;
                    let mut cur = tree.boxes[box_idx].dom_node as usize;
                    while cur < doc.node_count {
                        if doc.nodes[cur].tag_str() == "form" {
                            form_dom = Some(cur);
                            break;
                        }
                        let p = doc.nodes[cur].parent;
                        if p == 0xFFFF { break; }
                        cur = p as usize;
                    }
                    if let Some(form_idx) = form_dom {
                        // Snapshot the action URL.
                        let mut action_buf = [0u8; 512];
                        let mut action_len = 0usize;
                        if let Some(a) = doc.nodes[form_idx].get_attr("action") {
                            let n = a.len().min(action_buf.len());
                            action_buf[..n].copy_from_slice(&a.as_bytes()[..n]);
                            action_len = n;
                        }
                        let action = unsafe {
                            core::str::from_utf8_unchecked(&action_buf[..action_len])
                        };
                        // Build urlencoded body by walking every
                        // descendant <input>/<textarea>/<select> with
                        // a `name=` attribute. Skip those without name
                        // (per HTML spec) and submit inputs themselves.
                        let mut body = [0u8; 4096];
                        let mut blen = 0usize;
                        for di in 0..doc.node_count {
                            // Is `di` a transitive descendant of form_idx?
                            let mut anc = di;
                            let mut in_form = false;
                            for _ in 0..32 {
                                if anc == form_idx { in_form = true; break; }
                                let p = doc.nodes[anc].parent;
                                if p == 0xFFFF { break; }
                                anc = p as usize;
                            }
                            if !in_form { continue; }
                            let n = &doc.nodes[di];
                            let tag = n.tag_str();
                            if tag != "input" && tag != "textarea" && tag != "select" { continue; }
                            // Skip submit inputs.
                            if tag == "input" {
                                let it = n.get_attr("type").unwrap_or("text");
                                if it == "submit" || it == "image" || it == "button" { continue; }
                            }
                            let name = match n.get_attr("name") {
                                Some(v) => v,
                                None => continue,
                            };
                            let value = n.get_attr("value").unwrap_or("");
                            if blen > 0 && blen < body.len() { body[blen] = b'&'; blen += 1; }
                            blen += url_encode(name.as_bytes(), &mut body[blen..]);
                            if blen < body.len() { body[blen] = b'='; blen += 1; }
                            blen += url_encode(value.as_bytes(), &mut body[blen..]);
                        }

                        if action_len > 0 {
                            uart::puts("  render: form submit → POST ");
                            uart::puts(action);
                            uart::puts(" body=");
                            uart::puts(unsafe { core::str::from_utf8_unchecked(&body[..blen]) });
                            uart::puts("\n");
                            // Only absolute URLs supported in 1.3.
                            if action.starts_with("http://") || action.starts_with("https://") {
                                let buf = unsafe { &mut *core::ptr::addr_of_mut!(HTML_FETCH_BUF) };
                                match crate::net::fetch::fetch_post_url(action, &body[..blen], buf) {
                                    Ok(n) => {
                                        uart::puts("  render: POST returned ");
                                        crate::kernel::mm::print_num(n);
                                        uart::puts(" bytes\n");
                                        doc.init();
                                        tree.box_count = 0;
                                        tree.text_len = 0;
                                        tree.page_height = 0;
                                        html::parser::parse(&buf[..n], doc);
                                        layout::build(doc, tree, rw as i32);
                                        continue;
                                    }
                                    Err(e) => {
                                        uart::puts("  render: POST failed: ");
                                        uart::puts(e); uart::puts("\n");
                                    }
                                }
                            } else {
                                uart::puts("  render: form action is relative — not supported yet\n");
                            }
                        } else {
                            uart::puts("  render: form has no action — would self-submit (not supported)\n");
                        }
                    }
                }

                // Sprint 1.2: link-click navigation. If the click hit
                // an <a href="..."> (or descendant of one), treat that
                // as a navigation request: replace the current document
                // with the href target, re-parse, re-layout. Only
                // absolute http(s):// hrefs are followed today;
                // relative URL resolution is on the navigation
                // milestone but not done here.
                let link_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
                    n.tag_str() == "a" && n.get_attr("href").is_some()
                });
                if let Some(box_idx) = link_owner {
                    let dom_idx = tree.boxes[box_idx].dom_node as usize;
                    let mut href_buf = [0u8; 512];
                    let mut href_len = 0usize;
                    if let Some(h) = doc.nodes[dom_idx].get_attr("href") {
                        let n = h.len().min(href_buf.len());
                        href_buf[..n].copy_from_slice(&h.as_bytes()[..n]);
                        href_len = n;
                    }
                    let href = unsafe {
                        core::str::from_utf8_unchecked(&href_buf[..href_len])
                    };
                    if href.starts_with("http://") || href.starts_with("https://") {
                        uart::puts("  render: click_xy → navigating to ");
                        uart::puts(href); uart::puts("\n");
                        // Fetch + re-parse in place. Reuse HTML_FETCH_BUF
                        // (the same static the initial load used).
                        let buf = unsafe { &mut *core::ptr::addr_of_mut!(HTML_FETCH_BUF) };
                        match crate::net::fetch::fetch_url(href, buf) {
                            Ok(n) => {
                                uart::puts("  render: nav fetched ");
                                crate::kernel::mm::print_num(n);
                                uart::puts(" bytes\n");
                                doc.init();
                                tree.box_count = 0;
                                tree.text_len = 0;
                                tree.page_height = 0;
                                html::parser::parse(&buf[..n], doc);
                                uart::puts("  render: nav parsed ");
                                crate::kernel::mm::print_num(doc.len());
                                uart::puts(" nodes\n");
                                layout::build(doc, tree, rw as i32);
                                uart::puts("  render: nav laid out ");
                                crate::kernel::mm::print_num(tree.box_count);
                                uart::puts(" boxes\n");
                                // Skip the JS replay block below — we
                                // already have the fresh post-nav layout.
                                continue;
                            }
                            Err(e) => {
                                uart::puts("  render: nav fetch failed: ");
                                uart::puts(e); uart::puts("\n");
                            }
                        }
                    } else if href_len > 0 {
                        uart::puts("  render: click_xy hit relative <a href=");
                        uart::puts(href);
                        uart::puts("> — relative-URL nav not wired yet\n");
                    }
                } else {
                    uart::puts("  render: click_xy(");
                    crate::kernel::mm::print_num(qx as usize);
                    uart::puts(",");
                    crate::kernel::mm::print_num(qy as usize);
                    uart::puts(") hit box ");
                    crate::kernel::mm::print_num(hit);
                    uart::puts(" — no onclick or href ancestor\n");
                }
            }
        }
    }

    // If any click_xy fired, re-run JS and re-layout against the
    // mutated DOM so paint shows the post-click state.
    if had_click_xy && doc.js_len > 0 {
        static mut REPLAY_VM: crate::browser::js::vm::Vm =
            crate::browser::js::vm::Vm::new();
        let vm = unsafe { &mut *core::ptr::addr_of_mut!(REPLAY_VM) };
        crate::browser::js::dom_api::set_document(doc);
        vm.init();
        let _ = vm.execute(&doc.js_text[..doc.js_len]);
        if vm.console_len > 0 {
            crate::drivers::uart::puts("=== JS console (replay) ===\n");
            let cb = unsafe {
                core::slice::from_raw_parts(
                    core::ptr::addr_of!(vm.console_buf) as *const u8,
                    vm.console_len,
                )
            };
            for &b in cb { uart::putc(b); }
            if !cb.last().map(|&b| b == b'\n').unwrap_or(false) {
                crate::drivers::uart::puts("\n");
            }
            crate::drivers::uart::puts("=== /JS console ===\n");
        }
        // Re-layout so paint sees the post-click DOM.
        // Reset the layout tree so build() doesn't append on top of
        // stale boxes from the first pass.
        tree.box_count = 0;
        tree.text_len = 0;
        tree.page_height = 0;
        layout::build(doc, tree, rw as i32);
        crate::drivers::uart::puts("  render: re-laid out "); crate::kernel::mm::print_num(tree.box_count);
        crate::drivers::uart::puts(" boxes after click_xy (page_height=");
        crate::kernel::mm::print_num(tree.page_height as usize);
        crate::drivers::uart::puts(")\n");
    }

    // Pick the body's effective background color so the area below
    // content matches the theme instead of staying white. Fall back
    // to white when the body has no explicit background.
    let body_bg = tree.boxes[0].style.background_color;
    let bg_word = if body_bg == crate::browser::css::style::Color::TRANSPARENT {
        0xFFFFFFFF
    } else {
        body_bg.raw()
    };

    use core::sync::atomic::Ordering as O2;
    let fb_ptr = unsafe { core::ptr::addr_of_mut!(RENDER_FB) as *mut u32 };

    // STUMP #89: paginated render. Each page is up to MAX_H tall; we
    // emit ceil(page_height / MAX_H) RENDER-BEGIN/END blocks so the
    // host script can split them into separate PNGs. Capped at
    // MAX_PAGES so a runaway layout doesn't dump megabytes per page
    // forever. Single-page pages still emit exactly one block, and
    // the existing render_to_png.py finds it via the same regex.
    //
    // STUMP #96: bumped 4 → 12. Wikipedia's Cat article is ~7700 px
    // tall just for the chrome (TOC + 200-language sidebar) before
    // the actual article body starts. 4 pages put the entire visible
    // output in the language list. 12 lets us reach the article.
    const MAX_PAGES: u32 = 12;
    let total_h = tree.page_height.max(600) as u32;
    let mut n_pages = (total_h + MAX_H - 1) / MAX_H;
    if n_pages == 0 { n_pages = 1; }
    if n_pages > MAX_PAGES { n_pages = MAX_PAGES; }

    for page in 0..n_pages {
        let scroll_y = (page * MAX_H) as i32;
        // Last page may be shorter than MAX_H if there's less content
        // remaining; keep at least 600 px tall so a short trailing
        // chunk still looks like a screenshot.
        let rh: u32 = ((total_h - page * MAX_H).min(MAX_H)).max(600);

        // Point gpu at our buffer for THIS page. Re-set per page so
        // SOFT_W/H reflect the (possibly shorter) trailing page.
        crate::drivers::virtio::gpu::SOFT_FB.store(fb_ptr as usize, O2::Release);
        crate::drivers::virtio::gpu::SOFT_W.store(rw, O2::Release);
        crate::drivers::virtio::gpu::SOFT_H.store(rh, O2::Release);

        crate::drivers::virtio::gpu::fill_screen(bg_word);
        // paint() already supports scroll_y — boxes whose content sits
        // outside the visible viewport are skipped via its clip check.
        paint::paint(tree, 0, 0, scroll_y, rw as i32, rh as i32);

        // Dump as base64-encoded raw BGRA. Each page is its own
        // RENDER-BEGIN/END block; the page index is part of the
        // header line so the host script can name the files.
        crate::drivers::uart::puts("=== RENDER-BEGIN ");
        crate::kernel::mm::print_num(rw as usize);
        crate::drivers::uart::puts("x");
        crate::kernel::mm::print_num(rh as usize);
        crate::drivers::uart::puts(" page=");
        crate::kernel::mm::print_num(page as usize);
        crate::drivers::uart::puts("/");
        crate::kernel::mm::print_num(n_pages as usize);
        crate::drivers::uart::puts(" ===\n");
        emit_b64_dump(fb_ptr, rw, rh);
        crate::drivers::uart::puts("=== RENDER-END ===\n");
    }

    // Restore so other code paths (e.g. console::puts) don't keep
    // writing into our buffer.
    crate::drivers::virtio::gpu::SOFT_FB.store(0, O2::Release);
    crate::drivers::virtio::gpu::SOFT_W.store(0, O2::Release);
    crate::drivers::virtio::gpu::SOFT_H.store(0, O2::Release);

    // Sprint 1.4 / 1.5 (STUMP #97/#98): live mode. Blit the FIRST page
    // onto the real virtio-gpu framebuffer + flush so the user
    // looking at the QEMU display window sees the page rendered. With
    // a virtio-tablet attached we then enter an interactive loop:
    // poll mouse + keyboard, draw a cursor, dispatch real clicks
    // through the same hit-test / onclick / link-nav machinery as
    // click_xy. ESC (or 'q' on serial) exits the loop and returns to
    // the shell prompt.
    if live_mode {
        let real_w = crate::drivers::virtio::gpu::width();
        let real_h = crate::drivers::virtio::gpu::height();
        let real_fb = crate::drivers::virtio::gpu::framebuffer();
        if !real_fb.is_null() && real_w > 0 && real_h > 0 {
            let copy_w = rw.min(real_w);
            let copy_h = MAX_H.min(real_h).min(total_h);
            let x_off = ((real_w - copy_w) / 2) as usize;
            let y_off = 0usize;

            blit_render_to_gpu(fb_ptr, rw, copy_w, copy_h,
                               real_fb, real_w, x_off, y_off, bg_word);
            crate::drivers::virtio::gpu::flush(0, 0, real_w, real_h);
            crate::drivers::uart::puts("  render: live blit ");
            crate::kernel::mm::print_num(copy_w as usize);
            crate::drivers::uart::puts("x");
            crate::kernel::mm::print_num(copy_h as usize);
            crate::drivers::uart::puts(" → virtio-gpu\n");

            // Sprint 1.5b: interactive loop. Only entered when the
            // tablet driver came up (i.e. QEMU launched with
            // -device virtio-tablet-device). Otherwise we just leave
            // the static page on screen and return.
            if crate::drivers::virtio::tablet::is_ready() {
                crate::drivers::uart::puts("  render: entering interactive loop (ESC to exit)\n");
                interactive_loop(
                    doc, tree,
                    fb_ptr, rw,
                    real_fb, real_w, real_h,
                    copy_w, copy_h, x_off, y_off,
                    bg_word, body_bg,
                );
                crate::drivers::uart::puts("  render: interactive loop exited\n");
            } else {
                crate::drivers::uart::puts("  render: no tablet attached → static window\n");
            }
        } else {
            crate::drivers::uart::puts("  render: live=1 set but no virtio-gpu (use -display gtk/cocoa)\n");
        }
    }
}

/// Sprint 1.5: copy our private render framebuffer into the real
/// virtio-gpu framebuffer, centered horizontally. The destination's
/// row stride is `real_w` pixels; the source's is `src_w`. Areas
/// outside the page are filled with `bg_word`.
fn blit_render_to_gpu(
    src_fb: *mut u32, src_w: u32,
    copy_w: u32, copy_h: u32,
    dst_fb: *mut u32, dst_w: u32,
    x_off: usize, y_off: usize,
    bg_word: u32,
) {
    crate::drivers::virtio::gpu::fill_screen(bg_word);
    for row in 0..copy_h as usize {
        let src_row_start = row * src_w as usize;
        let dst_row_start = (y_off + row) * dst_w as usize + x_off;
        for col in 0..copy_w as usize {
            let pixel = unsafe {
                core::ptr::read_volatile(src_fb.add(src_row_start + col))
            };
            unsafe {
                core::ptr::write_volatile(
                    dst_fb.add(dst_row_start + col),
                    pixel,
                );
            }
        }
    }
}

/// Sprint 1.5b — STUMP #98 — interactive event loop.
///
/// Polls the tablet + keyboard between repaint frames. Tablet
/// movement repaints (the cheap path: re-blit the cached page +
/// stamp a 12 px arrow cursor on top). Tablet clicks hit-test the
/// layout tree, walk up to find an `onclick` ancestor or an `<a
/// href>` ancestor, and dispatch — re-parse / re-layout / re-blit
/// when navigation lands. ESC or 'q' returns to the shell.
///
/// Single-page only today: scrolling the layout tree under the
/// cursor is the next milestone.
fn interactive_loop(
    doc: &mut crate::browser::dom::Document,
    tree: &mut crate::browser::layout::LayoutTree,
    src_fb: *mut u32, src_w: u32,
    dst_fb: *mut u32, dst_w: u32, dst_h: u32,
    copy_w: u32, copy_h: u32,
    x_off: usize, y_off: usize,
    bg_word: u32,
    body_bg: crate::browser::css::style::Color,
) {
    use crate::drivers::virtio::tablet::{self, InputEvent};
    use crate::drivers::virtio::keyboard;
    let _ = body_bg;

    let mut last_cx = -1i32;
    let mut last_cy = -1i32;
    let mut needs_redraw = true;
    // Layout viewport width — same value cmd_render computed.
    let viewport_w = src_w as i32;
    // Sprint 1.5c: focused input. Tracked by DOM index (not box index)
    // because re-layout invalidates box indices but DOM nodes are
    // stable. None = nothing focused; keystrokes are ignored.
    let mut focus_dom: Option<usize> = None;

    'main: loop {
        // Pump device polls. Cheap if no events queued.
        keyboard::poll();
        tablet::poll();

        // STUMP #98 fix: read from BOTH the GUI virtio-keyboard AND
        // the UART/serial console so it doesn't matter whether the
        // user types into the QEMU window (cocoa display) or the
        // host terminal that ran `make render-live`. On Mac the
        // QEMU window often needs an explicit click-to-focus before
        // it accepts keystrokes, and even then the cocoa input grab
        // is finicky; routing serial in too means the user can just
        // keep typing in the terminal where they launched it.
        let mut next_char = || -> Option<u8> {
            if let Some(c) = keyboard::getc() { return Some(c); }
            platform::serial_getc()
        };

        // STUMP #100 (sprint 1.5d): keyboard-driven cursor.
        // QEMU-cocoa-on-Mac silently drops virtio-tablet motion
        // events, so the user can never click anything with the
        // physical mouse. We expose a parallel cursor controller:
        //   Ctrl+W/A/S/D = move cursor 16 px up/left/down/right
        //   Ctrl+E       = "click" at the cursor's current spot
        //   Ctrl+G       = recenter cursor to (copy_w/2, copy_h/2)
        // Plain WASD/ESC stay available for typing into focused inputs
        // and quitting the loop. The Ctrl+letter codes (0x17, 0x01,
        // 0x13, 0x04, 0x05, 0x07) come straight off the wire from the
        // host terminal so no escape-sequence parsing is needed.
        const CURSOR_STEP: i32 = 16;

        // Drain typed characters. ESC always exits the loop. With a
        // focused input, printable characters append to its `value`
        // attribute and backspace removes the last character; either
        // mutation triggers a re-layout + repaint.
        while let Some(ch) = next_char() {
            // Diagnostic: every key reaching the loop is logged so we
            // can tell at a glance whether the issue is "keys not
            // arriving" vs "keys arriving but not landing in the
            // input". Remove once the wiring is confirmed.
            crate::drivers::uart::puts("  [loop] key=");
            crate::kernel::mm::print_num(ch as usize);
            crate::drivers::uart::puts(" focus=");
            match focus_dom {
                Some(d) => { crate::kernel::mm::print_num(d); }
                None    => { crate::drivers::uart::puts("none"); }
            }
            crate::drivers::uart::puts("\n");
            if ch == 27 { break 'main; } // ESC

            // Ctrl+WASD cursor move. Initialise on first use so the
            // cursor starts somewhere visible instead of (-1, -1).
            let init_cursor = || (copy_w as i32 / 2, copy_h as i32 / 2);
            match ch {
                0x17 /* Ctrl+W */ => {
                    if last_cx < 0 { let (a, b) = init_cursor(); last_cx = a; last_cy = b; }
                    last_cy = (last_cy - CURSOR_STEP).max(0);
                    needs_redraw = true;
                    continue;
                }
                0x01 /* Ctrl+A */ => {
                    if last_cx < 0 { let (a, b) = init_cursor(); last_cx = a; last_cy = b; }
                    last_cx = (last_cx - CURSOR_STEP).max(0);
                    needs_redraw = true;
                    continue;
                }
                0x13 /* Ctrl+S */ => {
                    if last_cx < 0 { let (a, b) = init_cursor(); last_cx = a; last_cy = b; }
                    last_cy = (last_cy + CURSOR_STEP).min(copy_h as i32 - 1);
                    needs_redraw = true;
                    continue;
                }
                0x04 /* Ctrl+D */ => {
                    if last_cx < 0 { let (a, b) = init_cursor(); last_cx = a; last_cy = b; }
                    last_cx = (last_cx + CURSOR_STEP).min(copy_w as i32 - 1);
                    needs_redraw = true;
                    continue;
                }
                0x07 /* Ctrl+G — center cursor */ => {
                    let (a, b) = init_cursor();
                    last_cx = a; last_cy = b;
                    needs_redraw = true;
                    continue;
                }
                0x05 /* Ctrl+E — click at cursor */ => {
                    if last_cx >= 0 && last_cy >= 0 {
                        // Translate to layout coords (already are,
                        // since cursor is in copy_w/copy_h space).
                        let lx = last_cx;
                        let ly = last_cy;
                        if let Some(hit) = tree.hit_test(lx, ly) {
                            let input_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
                                let t = n.tag_str();
                                t == "input" || t == "textarea"
                            });
                            if let Some(box_idx) = input_owner {
                                let dom_idx = tree.boxes[box_idx].dom_node as usize;
                                focus_dom = Some(dom_idx);
                                crate::drivers::uart::puts("  [ctrl+E] focus → DOM ");
                                crate::kernel::mm::print_num(dom_idx);
                                crate::drivers::uart::puts("\n");
                            } else {
                                focus_dom = None;
                            }
                        }
                        handle_interactive_click(
                            doc, tree, lx, ly, viewport_w,
                            src_fb, src_w, dst_fb, dst_w, dst_h,
                            copy_w, copy_h, x_off, y_off, bg_word,
                        );
                        needs_redraw = true;
                    }
                    continue;
                }
                _ => {}
            }

            if let Some(dom_idx) = focus_dom {
                if ch == 0x08 || ch == 0x7F {
                    // Backspace / DEL: drop last byte of value.
                    let cur = doc.nodes[dom_idx].get_attr("value")
                        .map(|s| s.as_bytes()).unwrap_or(b"");
                    if !cur.is_empty() {
                        let mut buf = [0u8; 256];
                        let new_len = (cur.len() - 1).min(buf.len());
                        buf[..new_len].copy_from_slice(&cur[..new_len]);
                        let s = unsafe { core::str::from_utf8_unchecked(&buf[..new_len]) };
                        doc.nodes[dom_idx].set_attr("value", s);
                        rerun_layout_and_repaint(doc, tree, viewport_w, src_fb, src_w, bg_word);
                        needs_redraw = true;
                    }
                } else if ch == b'\r' || ch == b'\n' {
                    // Enter on a focused input: leave focus alone but
                    // do nothing else for now. Implicit form submit
                    // is on the Sprint 2 polish list.
                } else if ch >= 0x20 && ch < 0x7F {
                    // Printable ASCII: append to value.
                    let cur = doc.nodes[dom_idx].get_attr("value")
                        .map(|s| s.as_bytes()).unwrap_or(b"");
                    let mut buf = [0u8; 256];
                    let copy = cur.len().min(buf.len() - 1);
                    buf[..copy].copy_from_slice(&cur[..copy]);
                    buf[copy] = ch;
                    let s = unsafe { core::str::from_utf8_unchecked(&buf[..copy + 1]) };
                    doc.nodes[dom_idx].set_attr("value", s);
                    rerun_layout_and_repaint(doc, tree, viewport_w, src_fb, src_w, bg_word);
                    needs_redraw = true;
                }
            }
        }

        // Drain tablet events. We coalesce moves and run at most one
        // click per frame (since each click triggers a re-render).
        let mut click_at: Option<(i32, i32)> = None;
        while let Some(ev) = tablet::pop_event() {
            match ev {
                InputEvent::Move { x, y } => {
                    last_cx = x;
                    last_cy = y;
                    needs_redraw = true;
                }
                InputEvent::ButtonDown { x, y, .. } => {
                    click_at = Some((x, y));
                }
                InputEvent::ButtonUp { .. } => {}
            }
        }

        if let Some((cx, cy)) = click_at {
            let lx = cx - x_off as i32;
            let ly = cy - y_off as i32;
            if lx >= 0 && ly >= 0 && lx < copy_w as i32 && ly < copy_h as i32 {
                // STUMP #98 / Sprint 1.5c: focus follows click. If
                // the click landed inside an <input>/<textarea>, the
                // next keystrokes append to that node's `value`.
                if let Some(hit) = tree.hit_test(lx, ly) {
                    let input_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
                        let t = n.tag_str();
                        t == "input" || t == "textarea"
                    });
                    if let Some(box_idx) = input_owner {
                        let dom_idx = tree.boxes[box_idx].dom_node as usize;
                        focus_dom = Some(dom_idx);
                        crate::drivers::uart::puts("  [click] focus → DOM ");
                        crate::kernel::mm::print_num(dom_idx);
                        crate::drivers::uart::puts("\n");
                    } else {
                        focus_dom = None;
                    }
                }
                handle_interactive_click(
                    doc, tree, lx, ly, viewport_w,
                    src_fb, src_w, dst_fb, dst_w, dst_h,
                    copy_w, copy_h, x_off, y_off, bg_word,
                );
                needs_redraw = true;
            }
        }

        if needs_redraw {
            blit_render_to_gpu(src_fb, src_w, copy_w, copy_h,
                               dst_fb, dst_w, x_off, y_off, bg_word);
            if last_cx >= 0 && last_cy >= 0 {
                draw_cursor(dst_fb, dst_w as usize, dst_h as usize,
                            last_cx as usize, last_cy as usize);
            }
            crate::drivers::virtio::gpu::flush(0, 0, dst_w, dst_h);
            needs_redraw = false;
        }
    }
}

/// STUMP #98: re-layout + repaint the doc into RENDER_FB. Used after
/// any DOM mutation in the interactive loop (focused-input typing,
/// onclick re-runs). The caller's blit picks the new pixels up on
/// the next redraw.
fn rerun_layout_and_repaint(
    doc: &mut crate::browser::dom::Document,
    tree: &mut crate::browser::layout::LayoutTree,
    viewport_w: i32,
    src_fb: *mut u32,
    src_w: u32,
    bg_word: u32,
) {
    tree.box_count = 0;
    tree.text_len = 0;
    tree.page_height = 0;
    crate::browser::layout::build(doc, tree, viewport_w);
    repaint_to_render_fb(tree, src_fb, src_w, bg_word);
}

/// STUMP #98: handle a real click in interactive mode. Mirrors the
/// click_xy code in cmd_render but inlines the navigation /
/// onclick-replay / form-POST decision tree and re-paints to
/// RENDER_FB before the caller's blit pass.
fn handle_interactive_click(
    doc: &mut crate::browser::dom::Document,
    tree: &mut crate::browser::layout::LayoutTree,
    lx: i32, ly: i32, viewport_w: i32,
    src_fb: *mut u32, src_w: u32,
    dst_fb: *mut u32, dst_w: u32, dst_h: u32,
    copy_w: u32, copy_h: u32,
    x_off: usize, y_off: usize,
    bg_word: u32,
) {
    let _ = (dst_fb, dst_w, dst_h, copy_w, copy_h, x_off, y_off, bg_word);

    let hit = match tree.hit_test(lx, ly) {
        Some(h) => h,
        None => {
            crate::drivers::uart::puts("  [click] hit nothing at ");
            crate::kernel::mm::print_num(lx as usize);
            crate::drivers::uart::puts(",");
            crate::kernel::mm::print_num(ly as usize);
            crate::drivers::uart::puts("\n");
            return;
        }
    };

    // Link first — most common case in real pages.
    let link_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
        n.tag_str() == "a" && n.get_attr("href").is_some()
    });
    if let Some(box_idx) = link_owner {
        let dom_idx = tree.boxes[box_idx].dom_node as usize;
        let mut href_buf = [0u8; 512];
        let mut href_len = 0usize;
        if let Some(h) = doc.nodes[dom_idx].get_attr("href") {
            let n = h.len().min(href_buf.len());
            href_buf[..n].copy_from_slice(&h.as_bytes()[..n]);
            href_len = n;
        }
        let href = unsafe { core::str::from_utf8_unchecked(&href_buf[..href_len]) };
        if href.starts_with("http://") || href.starts_with("https://") {
            crate::drivers::uart::puts("  [click] navigating → ");
            crate::drivers::uart::puts(href);
            crate::drivers::uart::puts("\n");
            static mut NAV_BUF: [u8; 256 * 1024] = [0; 256 * 1024];
            let buf = unsafe { &mut *core::ptr::addr_of_mut!(NAV_BUF) };
            match crate::net::fetch::fetch_url(href, buf) {
                Ok(n) => {
                    doc.init();
                    tree.box_count = 0;
                    tree.text_len = 0;
                    tree.page_height = 0;
                    crate::browser::html::parser::parse(&buf[..n], doc);
                    crate::browser::layout::build(doc, tree, viewport_w);
                    repaint_to_render_fb(tree, src_fb, src_w, bg_word);
                }
                Err(e) => {
                    crate::drivers::uart::puts("  [click] nav fetch failed: ");
                    crate::drivers::uart::puts(e);
                    crate::drivers::uart::puts("\n");
                }
            }
            return;
        }
    }

    // onclick — run the inline JS, re-layout against the post-JS DOM.
    let onclick_owner = tree.nearest_ancestor_with_attr(hit, doc, |n| {
        n.get_attr("onclick").is_some()
    });
    if let Some(box_idx) = onclick_owner {
        let dom_idx = tree.boxes[box_idx].dom_node as usize;
        let mut handler = [0u8; 1024];
        let mut hlen = 0usize;
        if let Some(oc) = doc.nodes[dom_idx].get_attr("onclick") {
            let n = oc.len().min(handler.len());
            handler[..n].copy_from_slice(&oc.as_bytes()[..n]);
            hlen = n;
        }
        if hlen > 0 {
            crate::drivers::uart::puts("  [click] onclick on box ");
            crate::kernel::mm::print_num(box_idx);
            crate::drivers::uart::puts("\n");
            let avail = crate::browser::dom::MAX_JS - doc.js_len;
            if avail >= hlen + 2 {
                doc.js_text[doc.js_len..doc.js_len + hlen].copy_from_slice(&handler[..hlen]);
                doc.js_len += hlen;
                doc.js_text[doc.js_len] = b';';
                doc.js_text[doc.js_len + 1] = b'\n';
                doc.js_len += 2;
            }
            // Run the JS engine against the appended handler.
            static mut LIVE_VM: crate::browser::js::vm::Vm =
                crate::browser::js::vm::Vm::new();
            let vm = unsafe { &mut *core::ptr::addr_of_mut!(LIVE_VM) };
            crate::browser::js::dom_api::set_document(doc);
            vm.init();
            let _ = vm.execute(&doc.js_text[..doc.js_len]);
            tree.box_count = 0;
            tree.text_len = 0;
            tree.page_height = 0;
            crate::browser::layout::build(doc, tree, viewport_w);
            repaint_to_render_fb(tree, src_fb, src_w, bg_word);
        }
        return;
    }

    crate::drivers::uart::puts("  [click] hit box ");
    crate::kernel::mm::print_num(hit);
    crate::drivers::uart::puts(" — no link/onclick\n");
}

/// STUMP #98: re-paint the layout tree into our private render
/// framebuffer at MAX_H tall. The caller's blit pass copies it onto
/// the real GPU FB.
fn repaint_to_render_fb(
    tree: &crate::browser::layout::LayoutTree,
    fb_ptr: *mut u32,
    rw: u32,
    bg_word: u32,
) {
    use core::sync::atomic::Ordering as O2;
    const MAX_W: u32 = 1024;
    const MAX_H: u32 = 1900;
    crate::drivers::virtio::gpu::SOFT_FB.store(fb_ptr as usize, O2::Release);
    crate::drivers::virtio::gpu::SOFT_W.store(rw, O2::Release);
    crate::drivers::virtio::gpu::SOFT_H.store(MAX_H, O2::Release);
    crate::drivers::virtio::gpu::fill_screen(bg_word);
    crate::browser::paint::paint(tree, 0, 0, 0, rw as i32, MAX_H as i32);
    crate::drivers::virtio::gpu::SOFT_FB.store(0, O2::Release);
    crate::drivers::virtio::gpu::SOFT_W.store(0, O2::Release);
    crate::drivers::virtio::gpu::SOFT_H.store(0, O2::Release);
    let _ = MAX_W;
}

/// STUMP #98: stamp a tiny arrow cursor onto the destination
/// framebuffer. Pixel-art black border + white fill so it shows over
/// any background. Bounds-checked against the FB dimensions so a
/// cursor near the edge clips cleanly.
fn draw_cursor(fb: *mut u32, fb_w: usize, fb_h: usize, cx: usize, cy: usize) {
    // 12-row classic arrow. 1 = white fill, 2 = black border, 0 = transparent.
    static SHAPE: [&[u8]; 12] = [
        b"2",
        b"22",
        b"212",
        b"2112",
        b"21112",
        b"211112",
        b"2111112",
        b"21111112",
        b"211222",
        b"2122",
        b"212",
        b"22",
    ];
    for (dy, row) in SHAPE.iter().enumerate() {
        for (dx, &px) in row.iter().enumerate() {
            if px == b'0' { continue; }
            let x = cx + dx;
            let y = cy + dy;
            if x >= fb_w || y >= fb_h { continue; }
            let color = if px == b'1' { 0xFFFFFFFFu32 } else { 0xFF000000u32 };
            unsafe { core::ptr::write_volatile(fb.add(y * fb_w + x), color); }
        }
    }
}

/// STUMP #97: write `application/x-www-form-urlencoded` octets into
/// `out`, returning the number written. Only writes printable bytes
/// directly; other bytes are escaped as `%XX`. Caller must size `out`
/// to ≥ 3 × input length.
fn url_encode(input: &[u8], out: &mut [u8]) -> usize {
    let mut n = 0usize;
    for &b in input {
        let safe = matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~');
        if safe {
            if n < out.len() { out[n] = b; n += 1; }
        } else if b == b' ' {
            if n < out.len() { out[n] = b'+'; n += 1; }
        } else {
            if n + 2 < out.len() {
                let h = b"0123456789ABCDEF";
                out[n] = b'%';
                out[n + 1] = h[(b >> 4) as usize];
                out[n + 2] = h[(b & 0xF) as usize];
                n += 3;
            }
        }
    }
    n
}

/// STUMP #89: base64 raw-BGRA dumper extracted so the paginated
/// renderer can call it once per page. 76 base64 chars per line
/// (= 57 raw bytes), `read_volatile` per byte so the optimizer can't
/// hoist the framebuffer reads outside the loop.
fn emit_b64_dump(fb_ptr: *mut u32, rw: u32, rh: u32) {
    use crate::drivers::uart;
    static B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let total_bytes: usize = (rw * rh) as usize * 4;
    let mut col = 0usize;
    let raw_ptr = fb_ptr as *const u8;
    let mut i = 0usize;
    while i < total_bytes {
        let b0 = unsafe { core::ptr::read_volatile(raw_ptr.add(i)) };
        let b1 = if i + 1 < total_bytes { unsafe { core::ptr::read_volatile(raw_ptr.add(i + 1)) } } else { 0 };
        let b2 = if i + 2 < total_bytes { unsafe { core::ptr::read_volatile(raw_ptr.add(i + 2)) } } else { 0 };
        uart::putc(B64[((b0 >> 2) & 0x3F) as usize]);
        uart::putc(B64[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize]);
        col += 2;
        if i + 1 < total_bytes {
            uart::putc(B64[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize]);
            col += 1;
        } else {
            uart::putc(b'='); col += 1;
        }
        if i + 2 < total_bytes {
            uart::putc(B64[(b2 & 0x3F) as usize]);
            col += 1;
        } else {
            uart::putc(b'='); col += 1;
        }
        i += 3;
        if col >= 76 { uart::putc(b'\n'); col = 0; }
    }
    if col != 0 { uart::putc(b'\n'); }
}
