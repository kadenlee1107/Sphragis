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
    // (Stay on the zygote path: --no-zygote hits an ICU CharString
    // bug at ~300 syscalls (verified again 2026-04-25). The zygote
    // path goes much further; deadlock comes later in IPC pump.)
    // --enable-logging=stderr --v=1 — make Chromium's own LOG(INFO)
    // + VLOG(1) lines hit stderr so we see what it's doing right
    // before it crashes. Cheap to enable; no-op if logging is
    // compiled out.
    argv[n] = "--enable-logging=stderr";   n += 1;
    argv[n] = "--v=1";                     n += 1;
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
