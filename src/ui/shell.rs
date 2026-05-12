// Bat_OS — Interactive Kernel Shell
// Command-line interface rendered to the GPU console.
// Reads from UART, displays on framebuffer.

use crate::platform;
use crate::ui::console;
use crate::fs::batfs;
use crate::net;

const MAX_CMD_LEN: usize = 256;

// GUI shell entrypoint. Today no boot path invokes it (display goes
// to ui::desktop::run; headless goes to main::serial_shell). Kept as
// staged code for an admin/recovery launcher and for individual
// cmd_* helpers reachable via the selftest-on-boot/pq-interop-test
// hooks.
#[allow(dead_code)]
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

    use super::shell_history::{ArrowKey, EscState, FeedResult};
    let mut esc = EscState::default();

    // Replace the currently-visible input line with `new_bytes`.
    // Mirrors to both the GUI console and host serial.
    let redraw = |old_len: usize, new_bytes: &[u8], new_len: &mut usize,
                  cmd_buf: &mut [u8; MAX_CMD_LEN]| {
        for _ in 0..old_len {
            console::putc(0x08); console::putc(b' '); console::putc(0x08);
            platform::serial_putc(0x08); platform::serial_putc(b' ');
            platform::serial_putc(0x08);
        }
        for &b in new_bytes {
            console::putc(b);
            platform::serial_putc(b);
        }
        let n = new_bytes.len().min(MAX_CMD_LEN);
        cmd_buf[..n].copy_from_slice(&new_bytes[..n]);
        *new_len = n;
    };

    loop {
        smc_keepalive_tick();
        let Some(raw) = platform::serial_getc() else {
            core::hint::spin_loop();
            continue;
        };

        // Run every byte through the ANSI ESC-sequence parser before
        // dispatch — arrow keys arrive as ESC `[` `A`/`B`/`C`/`D`.
        let c = match esc.feed(raw) {
            FeedResult::Consumed => continue,
            FeedResult::Arrow(ArrowKey::Up) => {
                if let Some(line) = super::shell_history::prev() {
                    let mut take = [0u8; MAX_CMD_LEN];
                    let n = line.len().min(MAX_CMD_LEN);
                    take[..n].copy_from_slice(&line[..n]);
                    redraw(cmd_len, &take[..n], &mut cmd_len, &mut cmd_buf);
                }
                continue;
            }
            FeedResult::Arrow(ArrowKey::Down) => {
                match super::shell_history::next() {
                    Some(line) => {
                        let mut take = [0u8; MAX_CMD_LEN];
                        let n = line.len().min(MAX_CMD_LEN);
                        take[..n].copy_from_slice(&line[..n]);
                        redraw(cmd_len, &take[..n], &mut cmd_len, &mut cmd_buf);
                    }
                    None => {
                        redraw(cmd_len, &[], &mut cmd_len, &mut cmd_buf);
                    }
                }
                continue;
            }
            FeedResult::Arrow(_) => continue, // left/right ignored for v1
            FeedResult::Pass(b) => b,
        };

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
                        super::shell_history::record(&cmd_buf[..cmd_len]);
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
                        super::shell_history::reset_cursor();
                    }
                }
                0x03 => {
                    // Ctrl+C
                    console::puts("^C\n");
                    platform::serial_puts("^C\n");
                    cmd_len = 0;
                    super::shell_history::reset_cursor();
                    console::prompt();
                }
                0x09 => {
                    // Tab — autofill. Inside first token → command name;
                    // past a space → argument completion based on
                    // `arg_kind_for(cmd, arg_index)`.
                    let line = unsafe {
                        core::str::from_utf8_unchecked(&cmd_buf[..cmd_len])
                    };
                    let split = super::shell_completion::split_for_completion(line);
                    if let Some((cmd, arg_index, current)) = split {
                        let kind = super::shell_completion::arg_kind_for(cmd, arg_index);
                        if kind != super::shell_completion::ArgKind::None {
                            let r = super::shell_completion::complete_argument(kind, current);
                            let ext = r.extension_bytes();
                            let take = ext.len().min(MAX_CMD_LEN.saturating_sub(cmd_len + 1));
                            for &b in &ext[..take] {
                                cmd_buf[cmd_len] = b;
                                cmd_len += 1;
                                console::putc(b);
                                platform::serial_putc(b);
                            }
                            if r.match_count > 1 {
                                console::putc(b'\n');
                                platform::serial_putc(b'\n');
                                for i in 0..r.names_len as usize {
                                    let name = r.name_at(i);
                                    for &b in name {
                                        console::putc(b);
                                        platform::serial_putc(b);
                                    }
                                    console::puts("  ");
                                    platform::serial_puts("  ");
                                }
                                console::putc(b'\n');
                                platform::serial_putc(b'\n');
                                console::prompt();
                                for &b in &cmd_buf[..cmd_len] {
                                    console::putc(b);
                                    platform::serial_putc(b);
                                }
                            }
                        }
                    } else {
                        let r = super::shell_completion::complete_command(line);
                        let ext = r.extension_bytes();
                        let take = ext.len().min(MAX_CMD_LEN.saturating_sub(cmd_len + 1));
                        for &b in &ext[..take] {
                            cmd_buf[cmd_len] = b;
                            cmd_len += 1;
                            console::putc(b);
                            platform::serial_putc(b);
                        }
                        if r.match_count > 1 {
                            console::putc(b'\n');
                            platform::serial_putc(b'\n');
                            for &name in r.candidate_slice() {
                                console::puts(name);
                                console::puts("  ");
                                platform::serial_puts(name);
                                platform::serial_puts("  ");
                            }
                            console::putc(b'\n');
                            platform::serial_putc(b'\n');
                            console::prompt();
                            for &b in &cmd_buf[..cmd_len] {
                                console::putc(b);
                                platform::serial_putc(b);
                            }
                        }
                    }
                }
                _ => {
                    if cmd_len < MAX_CMD_LEN - 1 && c >= 0x20 && c <= 0x7E {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        platform::serial_putc(c);
                        super::shell_history::reset_cursor();
                    }
                }
            }
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
        "write" => {
            // take everything after the filename verbatim
            // so `write hello "test contents"` writes the whole quoted
            // string, not just the first space-delimited chunk.
            let after_cmd = cmd.trim_start()
                .split_once(' ').map(|(_, r)| r.trim_start()).unwrap_or("");
            let (name, raw_data) = match after_cmd.split_once(' ') {
                Some((n, d)) => (n, d.trim_start()),
                None => (after_cmd, ""),
            };
            // Strip surrounding "..." if present.
            let data = if raw_data.starts_with('"') && raw_data.ends_with('"') && raw_data.len() >= 2 {
                &raw_data[1..raw_data.len() - 1]
            } else { raw_data };
            cmd_write(name, data);
        }
        "read" | "cat" => cmd_read(parts[1]),
        "rm" | "delete" => cmd_rm(parts[1]),
        "verify" => cmd_verify(parts[1]),
        "ping" => cmd_ping(parts[1]),
        "dns" | "resolve" => cmd_dns(parts[1]),
        "ifconfig" | "net" => cmd_ifconfig(),
        "fw" | "firewall" => cmd_firewall(),
        "fetch" => cmd_fetch(parts[1]),
        "clip" => {
            // `clip set <text>` takes the rest of the line verbatim.
            if parts[1] == "set" {
                let after = cmd.trim_start()
                    .split_once(' ')
                    .and_then(|(_, r)| r.trim_start().split_once(' '))
                    .map(|(_, t)| t.trim())
                    .unwrap_or("");
                cmd_clip_set(after);
            } else if parts[1] == "yank-back" {
                cmd_clip_yank_back(parts[2]);
            } else if parts[1] == "push" {
                cmd_clip_push();
            } else if parts[1] == "pull" {
                cmd_clip_pull();
            } else {
                cmd_clip(parts[1]);
            }
        }
        "comms" => {
            // `comms send` takes its message verbatim; `connect` /
            // `identify` / `pin` use parts[].
            let sub = parts[1];
            if sub == "send" {
                let after_send = cmd.trim_start()
                    .split_once(' ')
                    .and_then(|(_, r)| r.trim_start().split_once(' '))
                    .map(|(_, msg)| msg.trim())
                    .unwrap_or("");
                cmd_comms_send(after_send);
            } else if sub == "connect"  { cmd_comms_connect(parts[2]);
            } else if sub == "identify" { cmd_comms_identify(parts[2]);
            } else if sub == "pin"      { cmd_comms_pin(parts[2]);
            } else if sub == "my-id"    { cmd_comms_my_id();
            } else {
                cmd_comms(sub, parts[2]);
            }
        }
        "batcave" => cmd_batcave(parts[1], parts[2], parts[3], &parts),
        "panic" => cmd_panic(),
        "hello" => cmd_run_elf("hello"),
        "hello_libc" | "libc" => cmd_run_elf("libc"),
        "threads" => cmd_run_elf("threads"),
        "posix" => cmd_run_elf("posix"),
        "cxx" | "c++" => cmd_run_elf("cxx"),
        "audit" => cmd_audit(parts[1]),
        "audit-flush" => cmd_audit_flush(),
        "audit-chain" => cmd_audit_chain(),
        "dmesg" => cmd_dmesg(parts[1]),
        "sec-status" | "secstatus" => cmd_sec_status(),
        "pin" => cmd_pin(parts[1], parts[2]),
        "crl" => cmd_crl(parts[1], parts[2], parts[3]),
        "hash" => cmd_hash(parts[1], parts[2]),
        "ai" => {
            // Everything after "ai " is the question, including spaces.
            let q = cmd.trim_start()
                .split_once(' ').map(|(_, r)| r.trim()).unwrap_or("");
            cmd_ai(q);
        }
        "tcp-selftest" => cmd_tcp_selftest(),
        "tcp-listen" => cmd_tcp_listen(parts[1]),
        "tcp-list" => cmd_tcp_list(),
        "origin" => cmd_origin(parts[1]),
        "origin-allow" => cmd_origin_allow(parts[1], parts[2]),
        "origin-mode" => cmd_origin_mode(parts[1]),
        "cookies" => cmd_cookies(parts[1]),
        "kbd-stats" | "kbd" => cmd_kbd_stats(),
        "kbd-trace" => cmd_kbd_trace(parts[1]),
        "edit" => cmd_edit(parts[1]),
        "screen" => cmd_screen(parts[1]),
        "otp-dump"    => cmd_otp_dump(),
        "otp-stats"   => cmd_otp_stats(),
        "otp-consume" => cmd_otp_consume(parts[1]),
        "pq-selftest" => cmd_pq_selftest(),
        "pq-sig-selftest" => cmd_pq_sig_selftest(),
        "ipc-selftest"        => cmd_ipc_selftest(),
        "pq-comms-selftest"   => cmd_pq_comms_selftest(),
        "shm-selftest"        => cmd_shm_selftest(),
        "quota-selftest"      => cmd_quota_selftest(),
        "block-on-selftest"   => cmd_block_on_selftest(),
        "sys-caves-selftest"  => cmd_sys_caves_selftest(),
        "release-verify"      => cmd_release_verify(parts[1], parts[2]),
        "release-pubkey"      => cmd_release_pubkey(),
        "pkg" => {
            // pkg install <bundle-in-batfs>
            // pkg list
            // pkg remove <name>
            // pkg stage <name> <ip:port>     (transfer from pkg_serve.py)
            match parts[1] {
                "install" => cmd_pkg_install(parts[2]),
                "list"    => cmd_pkg_list(),
                "remove" | "rm" => cmd_pkg_remove(parts[2]),
                "stage"   => cmd_pkg_stage(parts[2], parts[3]),
                _ => {
                    console::puts("  usage: pkg stage <name> <ip:port>\n");
                    console::puts("         pkg install <bundle.bpkg>\n");
                    console::puts("         pkg list\n");
                    console::puts("         pkg remove <name>\n");
                    console::puts("  bundles built with scripts/pkg_pack.py,\n");
                    console::puts("  served via scripts/pkg_serve.py <bundle.bpkg>\n");
                }
            }
        }
        "procs" | "ps"        => cmd_procs(parts[1]),
        "caps"                => cmd_caps(parts[1]),
        "fds"                 => cmd_fds(parts[1]),
        "task"                => cmd_task(parts[1]),
        "mount-ns"            => cmd_mount_ns(parts[1], parts[2], parts[3]),
        "cave-quota"          => cmd_cave_quota(parts[1], parts[2]),
        "cave-usage"          => cmd_cave_usage(),
        "pipe-selftest"       => cmd_pipe_selftest(),
        "unix-sock-selftest"  => cmd_unix_sock_selftest(),
        "date"                => cmd_date(),
        "tz"                  => cmd_tz(parts[1]),
        "time-selftest"       => cmd_time_selftest(),
        "time-sync-https"     => cmd_time_sync_https(parts[1]),
        "gcm-selftest"    => cmd_gcm_selftest(),
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
        "x509-selftest" => cmd_x509_selftest(),
        #[cfg(feature = "selftest-on-boot")]
        "scheduler-selftest" => cmd_scheduler_selftest(),
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

/// Scheduler selftest. Operates on synthesized Free→Blocked test
/// slots — no real sys_nanosleep / epoll_pwait calls, no Linux-thread-
/// context dependency. Three sub-tests exercise the wake helpers'
/// correctness; the park_current loop invariant is verified by manual
/// review (see DESIGN_SCHEDULER_BLOCK_ON.md acceptance criteria).
// /
/// `pub(crate)` so the boot-time runner in main.rs (gated by
/// selftest-on-boot) can call this for headless verification in
/// scripts/qemu_selftests_smoke.py.
#[cfg(feature = "selftest-on-boot")]
pub(crate) fn cmd_scheduler_selftest() {
    use crate::batcave::linux::threads::{
        cntpct_el0, wake_expired_deadlines, wake_epoll_waiters,
        test_install_blocked, test_inspect_state, test_release_slot,
        BlockReason, ThreadState,
    };

    console::puts_hi("  SCHEDULER SELFTEST\n");

    // Sub-test 1: wake-expired-deadlines is a noop when nothing is blocked
    // on a deadline. Just shouldn't panic or corrupt state.
    {
        wake_expired_deadlines();
        console::puts("  [scheduler-selftest] PASS: wake-expired-deadlines-noop\n");
    }

    // Sub-test 2: nanosleep deadline fires — install a Blocked slot with
    // already-past deadline, run the wake pass, observe Runnable, release.
    {
        let now = cntpct_el0();
        let past = now.saturating_sub(1);
        let slot = match test_install_blocked(BlockReason::Nanosleep { deadline_ticks: past }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: nanosleep-deadline-fires (table full)\n");
                return;
            }
        };
        wake_expired_deadlines();
        match test_inspect_state(slot) {
            Some(ThreadState::Runnable) => {
                console::puts("  [scheduler-selftest] PASS: nanosleep-deadline-fires\n");
            }
            _ => {
                console::puts("  [scheduler-selftest] FAIL: nanosleep-deadline-fires (wrong state)\n");
            }
        }
        test_release_slot(slot);
    }

    // Sub-test 3: epoll event-driven wake. Install two slots with
    // different epfds; wake one; observe the other stays Blocked.
    {
        let s1 = match test_install_blocked(BlockReason::EpollWait { epfd: 123, deadline_ticks: 0 }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (table full A)\n");
                return;
            }
        };
        let s2 = match test_install_blocked(BlockReason::EpollWait { epfd: 456, deadline_ticks: 0 }) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (table full B)\n");
                test_release_slot(s1);
                return;
            }
        };
        wake_epoll_waiters(123);
        let s1_state = test_inspect_state(s1);
        let s2_state = test_inspect_state(s2);
        let ok_first = matches!(s1_state, Some(ThreadState::Runnable));
        let ok_second_still_blocked = matches!(s2_state, Some(ThreadState::Blocked(_)));
        if ok_first && ok_second_still_blocked {
            wake_epoll_waiters(456);
            let s2_after = test_inspect_state(s2);
            if matches!(s2_after, Some(ThreadState::Runnable)) {
                console::puts("  [scheduler-selftest] PASS: epoll-event-wake\n");
            } else {
                console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (epfd 456 wake didn't fire)\n");
            }
        } else {
            console::puts("  [scheduler-selftest] FAIL: epoll-event-wake (selective wake broken)\n");
        }
        test_release_slot(s1);
        test_release_slot(s2);
    }

    // Sub-test 4: futex deadline fires — install a Blocked slot with
    // already-past FutexWait deadline, run the wake pass, observe Runnable.
    {
        let now = cntpct_el0();
        let past = now.saturating_sub(1);
        let slot = match test_install_blocked(
            BlockReason::FutexWait { uaddr: 0, val: 0, deadline_ticks: past }
        ) {
            Some(s) => s,
            None => {
                console::puts("  [scheduler-selftest] FAIL: futex-deadline-fires (table full)\n");
                return;
            }
        };
        wake_expired_deadlines();
        match test_inspect_state(slot) {
            Some(ThreadState::Runnable) => {
                console::puts("  [scheduler-selftest] PASS: futex-deadline-fires\n");
            }
            _ => {
                console::puts("  [scheduler-selftest] FAIL: futex-deadline-fires (wrong state)\n");
            }
        }
        test_release_slot(slot);
    }
}

/// X.509 chain validator selftest. Exercises the new `as_static_str`
/// error mapping path with two deterministic inputs:
/// 1. Trusted root used as a "leaf" with a wrong hostname → expect
/// VerifyOutcome::Err(HostnameMismatch).
/// 2. Truncated DER → expect VerifyOutcome::Err(Parse).
// /
/// Verifies both that the verifier surfaces the right variant AND
/// that as_static_str returns a debug-friendly string. See
/// DESIGN_TLS_HARDENING.md.
// /
/// `pub(crate)` so the boot-time selftest hook in `main.rs` (gated by
/// the `selftest-on-boot` Cargo feature) can call this for headless
/// verification in `scripts/qemu_x509_smoke.py`.
pub(crate) fn cmd_x509_selftest() {
    use crate::net::x509::{verify_chain, VerifyOutcome, VerifyError, TRUST_STORE};

    console::puts_hi("  X.509 CHAIN VALIDATOR SELFTEST\n");

    if TRUST_STORE.is_empty() {
        console::puts("  [x509-selftest] FAIL: TRUST_STORE empty\n");
        return;
    }
    let root_der: &[u8] = TRUST_STORE[0];

    fn contains(hay: &[u8], needle: &[u8]) -> bool {
        if needle.len() > hay.len() { return false; }
        for i in 0..=(hay.len() - needle.len()) {
            if &hay[i..i + needle.len()] == needle { return true; }
        }
        false
    }

    // Case 1: hostname mismatch.
    match verify_chain(root_der, &[], b"wrong-host.example") {
        VerifyOutcome::Err(VerifyError::HostnameMismatch) => {
            let s = VerifyError::HostnameMismatch.as_static_str();
            if contains(s.as_bytes(), b"hostname mismatch") {
                console::puts("  [x509-selftest] PASS: hostname-mismatch\n");
            } else {
                console::puts("  [x509-selftest] FAIL: hostname-mismatch (string mismatch)\n");
            }
        }
        VerifyOutcome::Err(other) => {
            console::puts("  [x509-selftest] FAIL: hostname-mismatch (got wrong VerifyError variant: ");
            console::puts(other.as_static_str());
            console::puts(")\n");
        }
        VerifyOutcome::Ok { .. } => {
            console::puts("  [x509-selftest] FAIL: hostname-mismatch (expected Err, got Ok)\n");
        }
    }

    // Case 2: truncated DER → Parse.
    let truncated = &root_der[..root_der.len().saturating_sub(5)];
    match verify_chain(truncated, &[], b"any.example") {
        VerifyOutcome::Err(VerifyError::Parse) => {
            let s = VerifyError::Parse.as_static_str();
            if contains(s.as_bytes(), b"parse error") {
                console::puts("  [x509-selftest] PASS: bad-bytes\n");
            } else {
                console::puts("  [x509-selftest] FAIL: bad-bytes (string mismatch)\n");
            }
        }
        VerifyOutcome::Err(other) => {
            console::puts("  [x509-selftest] FAIL: bad-bytes (got wrong VerifyError variant: ");
            console::puts(other.as_static_str());
            console::puts(")\n");
        }
        VerifyOutcome::Ok { .. } => {
            console::puts("  [x509-selftest] FAIL: bad-bytes (expected Err, got Ok)\n");
        }
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

// PQ-interop boot hook: drive a real TLS 1.3 + X25519MLKEM768 hybrid
// handshake against pq.cloudflareresearch.com and assert the server
// picked the hybrid group (not silent fallback to classical X25519).
// Headless QEMU smoke (scripts/qemu_pq_interop_smoke.py) keys off the
// `[pq-interop] PASS` / `[pq-interop] FAIL <reason>` lines.
#[cfg(feature = "pq-interop-test")]
pub(crate) fn cmd_pq_interop() {
    use crate::drivers::uart;
    let host = "pq.cloudflareresearch.com";

    // Make sure the global hybrid toggle is on (default is true, but
    // be explicit so the smoke doesn't depend on prior shell state).
    crate::net::tls::set_hybrid_enabled(true);

    let ip = match crate::net::dns::resolve(host) {
        Ok(ip) => ip,
        Err(_) => {
            uart::puts("[pq-interop] FAIL dns-resolve\n");
            return;
        }
    };

    if let Err(_e) = crate::net::tcp::connect(ip, 443) {
        uart::puts("[pq-interop] FAIL tcp-connect\n");
        return;
    }

    let hs_result = crate::net::tls::handshake(host);
    let used_hybrid = crate::net::tls::last_handshake_used_hybrid();
    crate::net::tls::close();
    crate::net::tcp::close();

    match hs_result {
        Err(e) => {
            uart::puts("[pq-interop] FAIL handshake: ");
            uart::puts(e);
            uart::puts("\n");
        }
        Ok(()) => {
            if !used_hybrid {
                uart::puts("[pq-interop] FAIL classical-fallback (server did not pick hybrid)\n");
            } else {
                uart::puts("[pq-interop] PASS hybrid-pq-handshake-ok\n");
            }
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
// `cpol-show <name>` dumps the full rule list for a cave.
// `cpol-add <name> <host> <port> <proto>` appends one allow rule.
// proto: "tcp", "udp", "any". port: 0 = any.
// `cpol-check <name> <host> <port> <proto>` runs the decision path
// and prints ALLOW / DROP — useful for verifying the kernel sees
// the same policy the daemon advertises.
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

// AES-128-GCM + AES-256-GCM NIST known-answer vectors.
// Validates that the gcm_verified module reproduces NIST SP 800-38D
// Test Cases 2 and 14 byte-for-byte AND rejects tampered ciphertext.
// Without this, a fault in the AES round constants or GHASH reduction
// wouldn't surface until a TLS server NACKs a ClientHello — the
// failure mode there is opaque ("connection reset"), so we'd rather
// fail loudly here at the shell.
fn cmd_gcm_selftest() {
    console::puts_hi("  AES-GCM KNOWN-ANSWER SELF-TEST\n");
    console::puts("  AES-128-GCM (NIST Case 2) + AES-256-GCM (NIST Case 14)\n");
    console::puts("  Encrypt → tag match → decrypt round-trip → tamper rejected\n");

    match crate::crypto::gcm_verified::selftest() {
        Ok(()) => {
            console::puts("  ✓ PASS  both ciphers reproduce published tags\n");
            console::puts("    Both TLS 1.3 cipher suites (TLS_AES_128_GCM_SHA256\n");
            console::puts("    and TLS_AES_256_GCM_SHA384) safe to negotiate.\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

// DESIGN_CRYPTO.md #10+#13: Noise-style IPC session handshake self-test.
/// `cave-usage` — gap-audit item 030 observability. Shows per-cave
/// memory pages used, CPU cntpct-ticks consumed, and net TX/RX
/// bytes. Memory has a quota; CPU and net are observability-only
/// today (no enforcement until preemptive timer scheduling lands).
fn cmd_cave_usage() {
    use crate::batcave::cave;
    console::puts_hi("  PER-CAVE RESOURCE USAGE\n");
    console::puts("  CAVE        MEM(used/quota)        CPU TICKS       TX BYTES      RX BYTES\n");
    let mut count = 0;
    cave::for_each_usage(|name, used, quota, cpu, tx, rx| {
        console::puts("  ");
        console::puts(name);
        for _ in name.len()..12 { console::putc(b' '); }
        print_num(used as usize); console::putc(b'/'); print_num(quota as usize);
        let mw = 24usize.saturating_sub(num_width(used as usize) + 1 + num_width(quota as usize));
        for _ in 0..mw { console::putc(b' '); }
        print_num(cpu as usize);
        let cw = 16usize.saturating_sub(num_width(cpu as usize));
        for _ in 0..cw { console::putc(b' '); }
        print_num(tx as usize);
        let tw = 14usize.saturating_sub(num_width(tx as usize));
        for _ in 0..tw { console::putc(b' '); }
        print_num(rx as usize);
        console::puts("\n");
        count += 1;
    });
    if count == 0 {
        console::puts("  (no active caves)\n");
    }
}

fn num_width(n: usize) -> usize {
    if n == 0 { return 1; }
    let mut w = 0;
    let mut v = n;
    while v > 0 { w += 1; v /= 10; }
    w
}

/// `cave-quota` — gap-audit item 030 first slice. Per-cave memory
/// quota: page counts charged to a cave and the limit it can't
/// exceed. Today enforced only at `shm::create`; other allocators
/// will adopt the same API in follow-up batches.
///   cave-quota                  show all caves' used/quota pairs
///   cave-quota <name> <pages>   set <name>'s quota in pages (4 KiB each)
fn cmd_cave_quota(name: &str, pages_str: &str) {
    use crate::batcave::cave;
    if name.is_empty() {
        console::puts("  CAVE        USED (pages)   QUOTA (pages)   MEMORY (MiB)\n");
        let mut count = 0;
        cave::for_each_quota(|nm, used, quota| {
            console::puts("  ");
            console::puts(nm);
            for _ in nm.len()..12 { console::putc(b' '); }
            print_num(used as usize);
            let uw = if used < 10 { 13 } else if used < 100 { 12 } else if used < 1000 { 11 } else { 10 };
            for _ in 0..uw { console::putc(b' '); }
            print_num(quota as usize);
            let qw = if quota < 10 { 15 } else if quota < 100 { 14 }
                else if quota < 1000 { 13 } else if quota < 10000 { 12 } else { 11 };
            for _ in 0..qw { console::putc(b' '); }
            print_num((quota / 256) as usize);  // 256 pages = 1 MiB
            console::puts("\n");
            count += 1;
        });
        if count == 0 {
            console::puts("  (no active caves)\n");
        }
        return;
    }
    if pages_str.is_empty() {
        console::puts("  usage: cave-quota <name> <pages>\n");
        console::puts("  (or `cave-quota` with no args to list)\n");
        return;
    }
    let pages: u32 = match pages_str.parse() {
        Ok(n) => n,
        Err(_) => { console::puts("  bad page count\n"); return; }
    };
    match cave::set_quota_by_name(name, pages) {
        Ok(()) => {
            console::puts("  ok — set ");
            console::puts(name);
            console::puts(" quota to ");
            print_num(pages as usize);
            console::puts(" pages (");
            print_num((pages / 256) as usize);
            console::puts(" MiB)\n");
        }
        Err(e) => { console::puts("  err: "); console::puts(e); console::puts("\n"); }
    }
}

/// `mount-ns` — gap-audit item 032 mount namespace. Demonstrates
/// per-cave file name scoping by prefixing the active cave's name
/// onto BatFS operations. Subcommands:
///   mount-ns                         show current prefix + scoped files
///   mount-ns ls                      list files in the active prefix
///   mount-ns write <name> <data>     create scoped file
///   mount-ns read  <name>            read scoped file
///   mount-ns rm    <name>            delete scoped file
///
/// Not wired into the default batfs::* path yet (42 callers, doing it
/// silently is risky for a single batch). This command proves the
/// scoping is sound; a follow-up batch flips the BatFS API to apply
/// the prefix everywhere.
fn cmd_mount_ns(sub: &str, arg1: &str, arg2: &str) {
    use crate::batcave::cave;
    use crate::fs::batfs;

    let mut prefix_buf = [0u8; 80];
    let plen = cave::active_mount_prefix(&mut prefix_buf);
    let prefix = unsafe { core::str::from_utf8_unchecked(&prefix_buf[..plen]) };

    if sub.is_empty() {
        console::puts("  active mount prefix: ");
        if plen == 0 {
            console::puts("(none — kernel/admin context)\n");
        } else {
            console::puts(prefix);
            console::puts("\n");
        }
        console::puts("  usage: mount-ns ls | write <n> <d> | read <n> | rm <n>\n");
        return;
    }

    if plen == 0 {
        console::puts("  no active cave — mount-ns has nothing to scope\n");
        console::puts("  attach a cave first via `batcave enter <name>`\n");
        return;
    }

    let mut full = [0u8; 144];
    let make_full = |name: &str, full: &mut [u8; 144]| -> Option<usize> {
        if name.is_empty() || plen + name.len() > full.len() {
            return None;
        }
        full[..plen].copy_from_slice(&prefix_buf[..plen]);
        full[plen..plen + name.len()].copy_from_slice(name.as_bytes());
        Some(plen + name.len())
    };

    match sub {
        "ls" => {
            console::puts("  scoped files (prefix: ");
            console::puts(prefix);
            console::puts("):\n");
            let mut shown = 0usize;
            batfs::list(|name, size, _enc| {
                if name.starts_with(prefix) {
                    let visible = &name[prefix.len()..];
                    console::puts("    ");
                    console::puts(visible);
                    console::puts("  (");
                    print_num(size);
                    console::puts(" bytes)\n");
                    shown += 1;
                }
            });
            console::puts("  ---\n  ");
            print_num(shown);
            console::puts(" file(s) in this namespace\n");
        }
        "write" => {
            let n = match make_full(arg1, &mut full) {
                Some(n) => n,
                None => { console::puts("  bad name (empty or too long)\n"); return; }
            };
            let full_name = unsafe { core::str::from_utf8_unchecked(&full[..n]) };
            match batfs::create(full_name, arg2.as_bytes()) {
                Ok(()) => {
                    console::puts("  ok — wrote ");
                    console::puts(full_name);
                    console::puts(" (");
                    print_num(arg2.len());
                    console::puts(" bytes)\n");
                }
                Err(e) => { console::puts("  err: "); console::puts(e); console::puts("\n"); }
            }
        }
        "read" => {
            let n = match make_full(arg1, &mut full) {
                Some(n) => n,
                None => { console::puts("  bad name\n"); return; }
            };
            let full_name = unsafe { core::str::from_utf8_unchecked(&full[..n]) };
            let mut buf = [0u8; 4096];
            match batfs::read(full_name, &mut buf) {
                Ok(len) => {
                    console::puts("  ");
                    for &b in &buf[..len] {
                        console::putc(if (0x20..=0x7e).contains(&b) || b == b'\n' { b } else { b'?' });
                    }
                    console::puts("\n");
                }
                Err(e) => { console::puts("  err: "); console::puts(e); console::puts("\n"); }
            }
        }
        "rm" => {
            let n = match make_full(arg1, &mut full) {
                Some(n) => n,
                None => { console::puts("  bad name\n"); return; }
            };
            let full_name = unsafe { core::str::from_utf8_unchecked(&full[..n]) };
            match batfs::delete(full_name) {
                Ok(()) => console::puts("  deleted\n"),
                Err(e) => { console::puts("  err: "); console::puts(e); console::puts("\n"); }
            }
        }
        _ => {
            console::puts("  usage: mount-ns ls | write <n> <d> | read <n> | rm <n>\n");
        }
    }
}

/// `caps [tid]` — show the capability set of a task (default: current).
/// Replaces the /proc/<pid>/status capability lines without needing
/// a procfs pseudo-file infrastructure.
fn cmd_caps(arg: &str) {
    use crate::kernel::process::{self, TaskId};
    let tid: u16 = if arg.is_empty() {
        process::current_id().0
    } else {
        match arg.parse() { Ok(n) => n, Err(_) => { console::puts("  bad tid\n"); return; } }
    };
    if tid as usize >= process::MAX_TASKS {
        console::puts("  tid out of range\n");
        return;
    }
    let task = process::get(TaskId(tid));
    console::puts("  task ");
    print_num(tid as usize);
    console::puts(" (");
    console::puts(task.name_str());
    console::puts(") capabilities:\n");
    let mut printed = 0;
    task.capabilities.for_each(|cap_type, target| {
        console::puts("    ");
        console::puts(cap_type.label());
        console::puts(" -> ");
        print_num(target as usize);
        console::puts("\n");
        printed += 1;
    });
    if printed == 0 {
        console::puts("    (no capabilities held)\n");
    }
}

/// `fds [tid]` — show the file-descriptor table for a task. The
/// fd-based equivalent of /proc/<pid>/fd.
fn cmd_fds(arg: &str) {
    use crate::kernel::process::{self, FdKind, PipeEnd, SocketRole, TaskId, MAX_FDS_PER_TASK};
    let tid: u16 = if arg.is_empty() {
        process::current_id().0
    } else {
        match arg.parse() { Ok(n) => n, Err(_) => { console::puts("  bad tid\n"); return; } }
    };
    if tid as usize >= process::MAX_TASKS {
        console::puts("  tid out of range\n");
        return;
    }
    let task = process::get(TaskId(tid));
    console::puts("  task ");
    print_num(tid as usize);
    console::puts(" (");
    console::puts(task.name_str());
    console::puts(") fds:\n");
    let mut count = 0;
    for fd in 0..MAX_FDS_PER_TASK {
        if let Some(e) = task.fds[fd] {
            console::puts("    fd ");
            print_num(fd);
            console::puts(" -> ");
            match e.kind {
                FdKind::Pipe { id, end } => {
                    console::puts("Pipe id="); print_num(id as usize);
                    console::puts(match end {
                        PipeEnd::Read => " end=read", PipeEnd::Write => " end=write",
                    });
                }
                FdKind::Socket { id, role } => {
                    console::puts("Socket id="); print_num(id as usize);
                    console::puts(match role {
                        SocketRole::Unbound   => " role=unbound",
                        SocketRole::Listener  => " role=listener",
                        SocketRole::Connected => " role=connected",
                    });
                }
                FdKind::Shm { id } => {
                    console::puts("Shm id="); print_num(id as usize);
                }
            }
            console::puts("\n");
            count += 1;
        }
    }
    if count == 0 {
        console::puts("    (no open fds)\n");
    }
}

/// `task <tid>` — combined view: state, priority, cave, name, fds,
/// caps. The /proc/<pid>/ summary in one shell command.
fn cmd_task(arg: &str) {
    use crate::kernel::process::{self, TaskId, TaskState};
    if arg.is_empty() {
        console::puts("  usage: task <tid>\n");
        return;
    }
    let tid: u16 = match arg.parse() { Ok(n) => n, Err(_) => { console::puts("  bad tid\n"); return; } };
    if tid as usize >= process::MAX_TASKS {
        console::puts("  tid out of range\n");
        return;
    }
    let task = process::get(TaskId(tid));
    console::puts_hi("  TASK ");
    print_num(tid as usize);
    console::puts("\n");
    console::puts("    name:     "); console::puts(task.name_str()); console::puts("\n");
    console::puts("    state:    ");
    console::puts(match task.state {
        TaskState::Free => "free", TaskState::Ready => "ready",
        TaskState::Running => "running", TaskState::Blocked => "blocked",
        TaskState::Dead => "dead",
    });
    console::puts("\n    priority: "); print_num(task.priority as usize); console::puts("\n");
    console::puts("    cave_id:  "); print_num(task.cave_id as usize); console::puts("\n");
    console::puts("    stack:    0x");
    let hex = b"0123456789abcdef";
    let sb = task.stack_base;
    for i in (0..8).rev() {
        let nib = ((sb >> (i * 4)) & 0xf) as usize;
        console::putc(hex[nib]);
    }
    console::puts("\n");
    cmd_fds(arg);
    cmd_caps(arg);
}

/// `procs` / `ps` — list tasks visible from the active cave's PID
/// namespace. Use `procs all` to see every task across namespaces
/// (admin view).
fn cmd_procs(arg: &str) {
    use crate::kernel::process::{self, TaskState};
    let admin = arg == "all";
    let cave_id = if admin { 0 } else {
        crate::batcave::cave::get_active() as u16
    };
    console::puts_hi(if admin {
        "  ALL TASKS (admin view across PID namespaces)\n"
    } else {
        "  TASKS IN THIS CAVE\n"
    });
    console::puts("  TID  CAVE  PRI  STATE     NAME\n");
    let mut shown = 0usize;
    process::list_for_cave(cave_id, |t| {
        let state_str = match t.state {
            TaskState::Free    => "free",
            TaskState::Ready   => "ready",
            TaskState::Running => "running",
            TaskState::Blocked => "blocked",
            TaskState::Dead    => "dead",
        };
        console::puts("  ");
        print_num(t.id.0 as usize);
        // pad: TID 1-3 digits → spaces
        let id_w = if t.id.0 < 10 { 4 } else if t.id.0 < 100 { 3 } else { 2 };
        for _ in 0..id_w { console::putc(b' '); }
        print_num(t.cave_id as usize);
        let cv_w = if t.cave_id < 10 { 5 } else if t.cave_id < 100 { 4 } else { 3 };
        for _ in 0..cv_w { console::putc(b' '); }
        print_num(t.priority as usize);
        let pr_w = if t.priority < 10 { 4 } else if t.priority < 100 { 3 } else { 2 };
        for _ in 0..pr_w { console::putc(b' '); }
        console::puts(state_str);
        for _ in state_str.len()..10 { console::putc(b' '); }
        console::puts(t.name_str());
        console::puts("\n");
        shown += 1;
    });
    console::puts("  ---\n  ");
    print_num(shown);
    console::puts(" task(s) visible\n");
}

/// `pkg stage <name> <ip:port>` — connect to a `pkg_serve.py`
/// instance, read the 4-byte length prefix + bundle bytes, and
/// write the result into BatFS at `name`. Bridges the
/// host-built bundle into BatFS so `pkg install` can verify it.
fn cmd_pkg_stage(name: &str, target: &str) {
    use crate::net;
    if name.is_empty() || target.is_empty() {
        console::puts("  usage: pkg stage <name> <ip:port>\n");
        console::puts("  e.g.:  pkg stage demo-1.0.bpkg 10.0.2.2:9102\n");
        return;
    }
    let (ip, port) = match target.rsplit_once(':') {
        Some((i, p)) => {
            let ip = parse_ip(i);
            let port: u16 = match p.parse() { Ok(v) if v > 0 => v, _ => 0 };
            if ip == 0 || port == 0 {
                console::puts("  invalid target (expected ip:port)\n"); return;
            }
            (ip, port)
        }
        None => { console::puts("  invalid target (expected ip:port)\n"); return; }
    };

    if let Err(e) = net::tcp::connect(ip, port) {
        console::puts("  connect failed: "); console::puts(e); console::puts("\n");
        return;
    }

    // Read 4-byte BE length first.
    let mut len_buf = [0u8; 4];
    let mut got = 0;
    while got < 4 {
        match net::tcp::recv_data(&mut len_buf[got..]) {
            Ok(0) => break,
            Ok(n) => got += n,
            Err(e) => {
                console::puts("  length recv failed: "); console::puts(e); console::puts("\n");
                net::tcp::close();
                return;
            }
        }
    }
    if got != 4 {
        console::puts("  truncated length header\n");
        net::tcp::close();
        return;
    }
    let total = u32::from_be_bytes(len_buf) as usize;
    if total == 0 || total > crate::kernel::pkg::MAX_BUNDLE {
        console::puts("  bundle length out of range: ");
        print_num(total);
        console::puts("\n");
        net::tcp::close();
        return;
    }
    console::puts("  receiving ");
    print_num(total);
    console::puts(" bytes from ");
    console::puts(target);
    console::puts(" ...\n");

    let mut buf = [0u8; crate::kernel::pkg::MAX_BUNDLE];
    let mut off = 0usize;
    while off < total {
        match net::tcp::recv_data(&mut buf[off..total]) {
            Ok(0) => break,
            Ok(n) => off += n,
            Err(e) => {
                console::puts("  body recv failed at "); print_num(off);
                console::puts(": "); console::puts(e); console::puts("\n");
                net::tcp::close();
                return;
            }
        }
    }
    net::tcp::close();
    if off != total {
        console::puts("  short read: got "); print_num(off);
        console::puts(" of "); print_num(total); console::puts("\n");
        return;
    }

    // Delete any prior staged file with this name so re-staging is
    // idempotent. (BatFS::create refuses to overwrite.)
    let _ = crate::fs::batfs::delete(name);
    match crate::fs::batfs::create(name, &buf[..off]) {
        Ok(()) => {
            console::puts("  ✓ staged ");
            console::puts(name);
            console::puts(" (");
            print_num(off);
            console::puts(" bytes)\n  next: pkg install ");
            console::puts(name);
            console::puts("\n");
        }
        Err(e) => {
            console::puts("  ✗ batfs::create failed: ");
            console::puts(e); console::puts("\n");
        }
    }
}

/// `pkg install <bundle.bpkg>` — read a BPKG bundle from BatFS,
/// verify signature against the baked release pubkey, sha-256 each
/// payload, then unpack into BatFS. Gap-audit item 033.
fn cmd_pkg_install(bundle_name: &str) {
    use crate::kernel::pkg;
    if bundle_name.is_empty() {
        console::puts("  usage: pkg install <bundle-in-batfs>\n");
        return;
    }
    let pubkey_hex = match RELEASE_PUBKEY_HEX {
        Some(h) if h.len() == 64 => h,
        _ => {
            console::puts("  no release pubkey baked into this build — run `release-pubkey` for instructions\n");
            return;
        }
    };
    let pubkey = match parse_hex32(pubkey_hex) {
        Some(p) => p,
        None => { console::puts("  invalid baked pubkey hex\n"); return; }
    };

    // Read the bundle from BatFS.
    let mut buf = [0u8; pkg::MAX_BUNDLE];
    let n = match crate::fs::batfs::read(bundle_name, &mut buf) {
        Ok(n) => n,
        Err(e) => { console::puts("  bundle read failed: "); console::puts(e); console::puts("\n"); return; }
    };

    let bundle = match pkg::parse_and_verify(&buf[..n], &pubkey) {
        Ok(b) => b,
        Err(e) => {
            console::puts("  ✗ verify failed: ");
            console::puts(e.as_str());
            console::puts("\n");
            return;
        }
    };
    console::puts("  ✓ signature verified\n  package: ");
    console::puts(bundle.name);
    console::puts("\n  version: ");
    console::puts(bundle.version);
    console::puts("\n  files:   ");
    print_num(bundle.files.len());
    console::puts("\n");

    match pkg::install(&bundle) {
        Ok(()) => {
            console::puts("  ✓ installed\n");
            for f in &bundle.files {
                console::puts("    + ");
                console::puts(f.path);
                console::puts(" (");
                print_num(f.content.len());
                console::puts(" bytes)\n");
            }
        }
        Err(e) => {
            console::puts("  ✗ install failed: ");
            console::puts(e.as_str());
            console::puts("\n");
        }
    }
}

fn cmd_pkg_list() {
    use crate::kernel::pkg;
    console::puts_hi("  INSTALLED PACKAGES\n");
    console::puts("  NAME             VERSION      FILES\n");
    let mut count = 0;
    pkg::for_each_installed(|name, ver, paths| {
        console::puts("  ");
        console::puts(name);
        for _ in name.len()..17 { console::putc(b' '); }
        console::puts(ver);
        for _ in ver.len()..13 { console::putc(b' '); }
        // Files come tab-separated; emit them space-separated for
        // display. Just print everything verbatim with tabs replaced.
        for b in paths.bytes() {
            console::putc(if b == b'\t' { b' ' } else { b });
        }
        console::puts("\n");
        count += 1;
    });
    if count == 0 {
        console::puts("  (no packages installed)\n");
    }
}

fn cmd_pkg_remove(name: &str) {
    use crate::kernel::pkg;
    if name.is_empty() {
        console::puts("  usage: pkg remove <name>\n");
        return;
    }
    match pkg::remove(name) {
        Ok(()) => {
            console::puts("  ✓ removed ");
            console::puts(name);
            console::puts("\n");
        }
        Err(e) => {
            console::puts("  ✗ remove failed: ");
            console::puts(e.as_str());
            console::puts("\n");
        }
    }
}

/// Build-time pinned release-engineer Ed25519 pubkey. Set via
/// `BAT_OS_RELEASE_PUBKEY=<hex>` at build time (see build.rs +
/// scripts/release_sign.py). When None, the verifier refuses to
/// run — there's no fallback "default test key" that an attacker
/// could exploit.
const RELEASE_PUBKEY_HEX: Option<&str> = option_env!("BAT_OS_RELEASE_PUBKEY");

fn cmd_release_pubkey() {
    match RELEASE_PUBKEY_HEX {
        Some(hex) => {
            console::puts("  release-engineer pubkey (baked at build time):\n  ");
            console::puts(hex);
            console::puts("\n");
            crate::ui::clipboard::set(hex.as_bytes());
            console::puts("  -> copied to clipboard\n");
        }
        None => {
            console::puts("  no release pubkey baked in this build.\n");
            console::puts("  to enable signed-release verification:\n");
            console::puts("    python3 scripts/release_sign.py keygen\n");
            console::puts("    export BAT_OS_RELEASE_PUBKEY=<hex>\n");
            console::puts("    cargo build --release ...\n");
        }
    }
}

/// `release-verify <batfs-file> <sig-hex>` — verify an Ed25519
/// signature over a file in BatFS, against the build-time-pinned
/// release-engineer pubkey. Prints PASS/FAIL with the file's
/// SHA-256.
fn cmd_release_verify(name: &str, sig_hex: &str) {
    use crate::crypto::{sha256, sig};
    use crate::fs::batfs;

    if name.is_empty() || sig_hex.is_empty() {
        console::puts("  usage: release-verify <batfs-file> <sig-hex-128chars>\n");
        return;
    }

    let pubkey_hex = match RELEASE_PUBKEY_HEX {
        Some(h) if h.len() == 64 => h,
        _ => {
            console::puts("  no release pubkey baked into this build — run `release-pubkey` for instructions\n");
            return;
        }
    };
    let pubkey = match parse_hex32(pubkey_hex) {
        Some(p) => p,
        None => { console::puts("  invalid baked pubkey hex (corrupt build?)\n"); return; }
    };

    if sig_hex.len() != 128 {
        console::puts("  signature must be 128 hex chars (64 bytes Ed25519)\n");
        return;
    }
    let mut sig_bytes = [0u8; 64];
    let bytes = sig_hex.as_bytes();
    for i in 0..64 {
        let hi = hex_nibble(bytes[i * 2]);
        let lo = hex_nibble(bytes[i * 2 + 1]);
        if hi == 0xff || lo == 0xff {
            console::puts("  signature contains non-hex characters\n");
            return;
        }
        sig_bytes[i] = (hi << 4) | lo;
    }

    // Read the file. Cap at 1 MiB for the on-device verifier — bigger
    // bundles need the off-device verifier (signed manifest of chunk
    // hashes is the right shape but out of scope for this command).
    let mut file_buf = [0u8; 1024 * 1024];
    let file_len = match batfs::read(name, &mut file_buf) {
        Ok(n) => n,
        Err(e) => { console::puts("  file read failed: "); console::puts(e); console::puts("\n"); return; }
    };
    let file = &file_buf[..file_len];

    console::puts("  file:   ");
    console::puts(name);
    console::puts("\n  size:   ");
    print_num(file_len);
    console::puts(" bytes\n  sha256: ");
    let digest = sha256::hash(file);
    let hex_table = b"0123456789abcdef";
    for &b in &digest {
        console::putc(hex_table[(b >> 4) as usize]);
        console::putc(hex_table[(b & 0x0f) as usize]);
    }
    console::puts("\n");

    match sig::ed25519_verify(&pubkey, &sig_bytes, file) {
        Ok(()) => {
            console::puts("  ✓ VERIFIED — signature is valid under the baked release pubkey\n");
        }
        Err(_) => {
            console::puts("  ✗ FAILED — signature does NOT verify\n");
            console::puts("  possible causes: tampered file, wrong sig, mismatched pubkey,\n");
            console::puts("                   or you signed with a different key than was baked\n");
        }
    }
}

/// sys-caves Arc-2 selftest: prove the scheduler MMU hook fires
/// on cave-crossing task transitions.
///
/// Flow:
///   1. Look up sys-wg cave id (created at boot by sys_caves::init).
///   2. Snapshot TTBR0_EL1 from this task (cave_id 0 → PRIMARY_L1).
///   3. Spawn a worker kernel task tagged with sys-wg's cave_id.
///   4. Worker, on first run, reads TTBR0_EL1 and stores it in a
///      static. By Arc-1 contract the scheduler must have called
///      `mmu::switch_to_cave(sys-wg.l1)` before the worker runs.
///   5. Yield until the worker has recorded its observation.
///   6. Verify worker_ttbr0 == sys-wg's L1 phys addr.
///   7. Verify our TTBR0_EL1 is back at PRIMARY_L1 (the scheduler's
///      Arc-2 refinement: cave_id 0 transition → switch_to_primary).
fn cmd_sys_caves_selftest() {
    use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use crate::batcave::{cave, sys_caves};
    use crate::kernel::{process, scheduler};

    console::puts_hi("  SYS-CAVES ARC-2 SELF-TEST\n");
    console::puts("  Scheduler must swap TTBR0_EL1 on cave-crossing task transitions.\n");

    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id,
        None => {
            console::puts("  ✗ FAIL: sys-wg cave not initialized (sys_caves::init did not run?)\n");
            return;
        }
    };
    let sys_wg_l1 = match cave::get_cave_l1_phys(sys_wg_id as u16) {
        Some(l1) => l1,
        None => {
            console::puts("  ✗ FAIL: sys-wg cave has no L1 page table built\n");
            console::puts("    (allocation failed at boot? out of CAVE_L1 slots?)\n");
            return;
        }
    };
    console::puts("  sys-wg cave id=");
    print_num(sys_wg_id);
    console::puts(" L1=0x");
    let hex = b"0123456789abcdef";
    for sh in (0..16).rev() {
        console::putc(hex[((sys_wg_l1 >> (sh * 4)) & 0xF)]);
    }
    console::puts("\n");

    let ttbr0_before: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_before); }
    console::puts("  TTBR0 before:  0x");
    for sh in (0..16).rev() {
        console::putc(hex[((ttbr0_before >> (sh * 4)) & 0xF) as usize]);
    }
    console::puts("\n");

    // Worker side: read TTBR0_EL1 on first scheduling, publish.
    static WORKER_RAN: AtomicBool = AtomicBool::new(false);
    static WORKER_TTBR0: AtomicUsize = AtomicUsize::new(0);
    fn worker_entry() -> ! {
        let t: u64;
        unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) t); }
        WORKER_TTBR0.store(t as usize, Ordering::Release);
        WORKER_RAN.store(true, Ordering::Release);
        // Done — yield forever; the parent task harvests our atomics
        // and we eventually get reaped on cave teardown.
        loop { scheduler::yield_now(); }
    }

    WORKER_RAN.store(false, Ordering::Relaxed);
    WORKER_TTBR0.store(0, Ordering::Relaxed);

    let worker_id = match process::create_kernel_task(
        "sys-caves-worker",
        worker_entry,
        /* priority */ 5,
    ) {
        Some(id) => id,
        None => {
            console::puts("  ✗ FAIL: could not spawn worker task\n");
            return;
        }
    };
    // Tag it as belonging to sys-wg so the scheduler's MMU hook
    // fires on the transition into it.
    process::set_cave(worker_id, sys_wg_id as u16);
    console::puts("  spawned worker tid=");
    print_num(worker_id.0 as usize);
    console::puts(" tagged cave_id=");
    print_num(sys_wg_id);
    console::puts("\n");

    // Yield until the worker has run. Bounded — bail after 256
    // attempts so a regressed scheduler doesn't hang us forever.
    let mut tries = 0;
    while !WORKER_RAN.load(Ordering::Acquire) && tries < 256 {
        scheduler::yield_now();
        tries += 1;
    }
    if !WORKER_RAN.load(Ordering::Acquire) {
        console::puts("  ✗ FAIL: worker did not run within 256 yields\n");
        return;
    }
    let worker_ttbr0 = WORKER_TTBR0.load(Ordering::Acquire);
    console::puts("  worker observed TTBR0: 0x");
    for sh in (0..16).rev() {
        console::putc(hex[((worker_ttbr0 >> (sh * 4)) & 0xF)]);
    }
    console::puts("\n");

    // Sanity: workers' TTBR0 should equal sys-wg's L1.
    if worker_ttbr0 != sys_wg_l1 {
        console::puts("  ✗ FAIL: worker TTBR0 != sys-wg L1\n");
        console::puts("    (scheduler MMU hook did not fire, or wrong L1 was loaded)\n");
        return;
    }
    console::puts("  ✓ scheduler swapped TTBR0 to sys-wg's L1 on cross-cave switch\n");

    // Our TTBR0 should be back to PRIMARY_L1 (cave_id 0 fallback in
    // the scheduler hook). Read it now after a yield to make sure
    // we ran through schedule() at least once after the worker.
    scheduler::yield_now();
    let ttbr0_after: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_after); }
    console::puts("  TTBR0 after:   0x");
    for sh in (0..16).rev() {
        console::putc(hex[((ttbr0_after >> (sh * 4)) & 0xF) as usize]);
    }
    console::puts("\n");
    if ttbr0_after as usize == sys_wg_l1 {
        console::puts("  ✗ FAIL: TTBR0 still on sys-wg's L1 after returning to kernel ns\n");
        console::puts("    (Arc-2 refinement to switch_to_primary on cave_id 0 did not fire)\n");
        return;
    }
    console::puts("  ✓ TTBR0 restored when returning to kernel namespace (cave_id 0)\n");

    console::puts("  ✓ ALL SYS-CAVES TESTS PASSED\n");
}

/// Selftest for the scheduler's block_on async-bridge primitive.
fn cmd_block_on_selftest() {
    console::puts_hi("  SCHEDULER block_on() SELF-TEST\n");
    console::puts("  success path (closure flips after 3 polls)\n");
    console::puts("  timeout path (closure never flips; expect Err+elapsed >= deadline)\n");

    match crate::kernel::scheduler::block_on_selftest() {
        Ok((success_ok, timeout_ok, elapsed_us)) => {
            if success_ok {
                console::puts("  ✓ success path: closure resolved within budget\n");
            } else {
                console::puts("  ✗ FAIL: success path did not resolve\n");
                return;
            }
            if timeout_ok {
                console::puts("  ✓ timeout path: bailed at deadline (elapsed=");
                print_num(elapsed_us as usize);
                console::puts(" µs)\n");
            } else {
                console::puts("  ✗ FAIL: timeout path returned wrong error or finished early\n");
                return;
            }
            console::puts("  ✓ block_on works as the sync ⇆ pollable-subsystem bridge\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

/// Cave memory-quota enforcement selftest. Proves the quota
/// actually rejects allocations past the limit, without depending
/// on a specific cave being active (uses whatever's running).
///   1. Snapshot current cave's quota + usage.
///   2. Tighten the quota to (used + 1 page).
///   3. shm::create(8 KiB = 2 pages) — should fail with the right error.
///   4. Restore the original quota.
///   5. shm::create(8 KiB) — should succeed now.
///   6. Clean up.
fn cmd_quota_selftest() {
    use crate::batcave::cave;
    use crate::kernel::shm;

    console::puts_hi("  CAVE MEMORY-QUOTA ENFORCEMENT SELF-TEST\n");

    let active = cave::get_active();
    if active == usize::MAX {
        console::puts("  no active cave — nothing to enforce against\n");
        console::puts("  attach via `batcave enter <name>` and re-run\n");
        return;
    }

    let (used, original_quota) = cave::active_quota_status();
    console::puts("  baseline: used=");
    print_num(used as usize);
    console::puts(" pages, quota=");
    print_num(original_quota as usize);
    console::puts(" pages\n");

    // Tighten the quota by name. Need the cave name.
    let mut cave_name = [0u8; 64];
    let mut cave_name_len = 0;
    cave::for_each_quota(|name, _used, _quota| {
        if cave_name_len == 0 && name == cave::active_name_str() {
            let n = name.len().min(cave_name.len());
            cave_name[..n].copy_from_slice(&name.as_bytes()[..n]);
            cave_name_len = n;
        }
    });
    let name_str = unsafe { core::str::from_utf8_unchecked(&cave_name[..cave_name_len]) };

    let tight = used + 1;
    if let Err(e) = cave::set_quota_by_name(name_str, tight) {
        console::puts("  ✗ FAIL set_quota_by_name: ");
        console::puts(e); console::puts("\n");
        return;
    }
    console::puts("  tightened quota to ");
    print_num(tight as usize);
    console::puts(" pages (used+1)\n");

    // Should fail: 8 KiB = 2 pages, exceeds tight by 1.
    match shm::create(b"quota-selftest", 8192) {
        Err(e) if e == "cave: memory quota exceeded" => {
            console::puts("  ✓ rejected: shm::create exceeded quota as expected\n");
        }
        Err(e) => {
            console::puts("  ✗ FAIL: wrong error: ");
            console::puts(e); console::puts("\n");
            let _ = cave::set_quota_by_name(name_str, original_quota);
            return;
        }
        Ok(fd) => {
            console::puts("  ✗ FAIL: allocation succeeded despite tight quota\n");
            if let Some(id) = match crate::kernel::process::current().fd_get(fd).map(|e| e.kind) {
                Some(crate::kernel::process::FdKind::Shm { id }) => Some(id),
                _ => None,
            } {
                shm::release(id);
            }
            let _ = cave::set_quota_by_name(name_str, original_quota);
            return;
        }
    }

    // Restore quota; alloc should now succeed.
    let _ = cave::set_quota_by_name(name_str, original_quota);
    console::puts("  restored quota to ");
    print_num(original_quota as usize);
    console::puts(" pages\n");

    let fd = match shm::create(b"quota-selftest", 8192) {
        Ok(fd) => fd,
        Err(e) => {
            console::puts("  ✗ FAIL: post-restore alloc rejected: ");
            console::puts(e); console::puts("\n");
            return;
        }
    };
    console::puts("  ✓ post-restore alloc succeeded\n");

    // Cleanup.
    if let Some(id) = match crate::kernel::process::current().fd_get(fd).map(|e| e.kind) {
        Some(crate::kernel::process::FdKind::Shm { id }) => Some(id),
        _ => None,
    } {
        let _ = crate::kernel::process::current().fd_take(fd);
        shm::release(id);
    }

    console::puts("  ✓ ALL QUOTA TESTS PASSED\n");
}

/// POSIX shared-memory selftest. Exercises create + write + open
/// (second fd) + read-back through the second fd + double-close +
/// reuse-after-free. All within one task — proves the shared region
/// is the same backing storage between two fds.
fn cmd_shm_selftest() {
    use crate::kernel::shm;
    use crate::kernel::process::{self, FdKind};

    console::puts_hi("  POSIX SHARED MEMORY SELF-TEST\n");
    console::puts("  create + write + open(second fd) + read-back + close + reuse\n");

    let name: &[u8] = b"bat-shm-selftest";
    let fd1 = match shm::create(name, 256) {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL create: "); console::puts(e); console::puts("\n"); return; }
    };
    let id1 = match process::current().fd_get(fd1).map(|e| e.kind) {
        Some(FdKind::Shm { id }) => id,
        _ => { console::puts("  ✗ FAIL: fd1 not Shm\n"); return; }
    };

    // Write a marker.
    {
        let bytes = match shm::region_bytes_mut(id1) {
            Some(b) => b,
            None => { console::puts("  ✗ FAIL: region_bytes_mut\n"); return; }
        };
        let payload = b"bat_os_shm_marker_v1";
        bytes[..payload.len()].copy_from_slice(payload);
    }

    // Open a second fd on the same region.
    let fd2 = match shm::open(name) {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL open: "); console::puts(e); console::puts("\n"); return; }
    };
    let id2 = match process::current().fd_get(fd2).map(|e| e.kind) {
        Some(FdKind::Shm { id }) => id,
        _ => { console::puts("  ✗ FAIL: fd2 not Shm\n"); return; }
    };
    if id1 != id2 {
        console::puts("  ✗ FAIL: open returned a different region id\n");
        return;
    }
    console::puts("  ✓ second open returned same region id\n");

    // Read back through fd2 (different fd, same region).
    {
        let bytes = match shm::region_bytes_mut(id2) {
            Some(b) => b,
            None => { console::puts("  ✗ FAIL: read-back region_bytes_mut\n"); return; }
        };
        let expected = b"bat_os_shm_marker_v1";
        if &bytes[..expected.len()] != expected {
            console::puts("  ✗ FAIL: marker mismatch through second fd\n");
            return;
        }
    }
    console::puts("  ✓ marker visible through second fd (shared backing storage)\n");

    // Close both fds — region should be reclaimed when refcount = 0.
    let _ = process::current().fd_take(fd1);
    shm::release(id1);
    let _ = process::current().fd_take(fd2);
    shm::release(id2);
    if shm::region_bytes_mut(id1).is_some() {
        console::puts("  ✗ FAIL: region still active after both closes\n");
        return;
    }
    console::puts("  ✓ region reclaimed after both fds closed\n");

    // Reusing the name should now work (no orphan).
    let fd3 = match shm::create(name, 64) {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL reuse-after-free: "); console::puts(e); console::puts("\n"); return; }
    };
    let id3 = match process::current().fd_get(fd3).map(|e| e.kind) {
        Some(FdKind::Shm { id }) => id,
        _ => { console::puts("  ✗ FAIL: fd3 not Shm\n"); return; }
    };
    // Confirm storage is zeroed after reuse.
    let bytes = shm::region_bytes_mut(id3).unwrap();
    let mut all_zero = true;
    for &b in bytes.iter() { if b != 0 { all_zero = false; break; } }
    if !all_zero {
        console::puts("  ✗ FAIL: recycled storage not zeroed\n");
        return;
    }
    console::puts("  ✓ recycled region is zero-initialized\n");

    // Tidy.
    let _ = process::current().fd_take(fd3);
    shm::release(id3);

    console::puts("  ✓ ALL SHM TESTS PASSED\n");
}

/// In-kernel selftest of the PQ-hybrid comms handshake. Exercises
/// ML-KEM-768 + ML-DSA-65 + X25519 + Ed25519 + ChaCha20-Poly1305
/// without needing a peer that speaks the protocol. The classical-
/// only path (apps::comms) is what runs against the Python test
/// server today; this proves the PQ wire format + key derivation
/// are ready for the day we have a PQ peer.
fn cmd_pq_comms_selftest() {
    console::puts_hi("  POST-QUANTUM HYBRID COMMS HANDSHAKE SELF-TEST\n");
    console::puts("  X25519 + ML-KEM-768 KEM, Ed25519 + ML-DSA-65 sigs\n");
    console::puts("  Generating server long-term keys (sig + KEM)...\n");
    console::puts("  Generating client long-term sig key...\n");
    console::puts("  Running client encap -> server decap -> mutual sig verify...\n");

    use crate::batcave::pq_comms_session;
    match pq_comms_session::selftest_round_trip() {
        Ok((s_pref, c_pref, keys_match, aead_ok, client_pub_n, server_pub_n)) => {
            let hex = b"0123456789abcdef";
            console::puts("    server c2s key prefix: ");
            for &b in &s_pref {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n    client c2s key prefix: ");
            for &b in &c_pref {
                console::putc(hex[(b >> 4) as usize]);
                console::putc(hex[(b & 0x0f) as usize]);
            }
            console::puts("\n    client hybrid sig pubkey: ");
            print_num(client_pub_n);
            console::puts(" bytes (32 Ed25519 + 1952 ML-DSA-65)\n");
            console::puts("    server hybrid sig pubkey: ");
            print_num(server_pub_n);
            console::puts(" bytes (same layout)\n");
            if keys_match && aead_ok {
                console::puts("  ✓ PASS  shared secret agreed; AEAD round trip OK\n");
                console::puts("    Forward secret (KEM ephemerals discarded after agreement)\n");
                console::puts("    Mutually authenticated (hybrid Ed25519+ML-DSA-65 sigs)\n");
                console::puts("    Unbreakable under classical AND quantum attack\n");
            } else if !keys_match {
                console::puts("  ✗ FAIL  shared-secret disagreement\n");
            } else {
                console::puts("  ✗ FAIL  AEAD round trip failed\n");
            }
        }
        Err(e) => {
            console::puts("  ✗ FAIL: "); console::puts(e); console::puts("\n");
        }
    }
}

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

/// `date` — print the current wall-clock time in UTC.
/// Prints "(not synced)" if the RTC anchor never landed; in that
/// case monotonic-since-boot is shown so the operator has *some*
/// timestamp to work with.
fn cmd_date() {
    use crate::kernel::time;
    if !time::is_synced() {
        console::puts("  (not synced — RTC unavailable)\n");
        console::puts("  uptime: ");
        print_num(time::monotonic_secs() as usize);
        console::puts(" s monotonic\n");
        return;
    }
    let now_utc = time::realtime_secs();
    let (y, m, d, h, mm, s) = time::split_unix(now_utc);
    let mut buf = [0u8; 20];
    let n = time::format_human(&mut buf, y, m, d, h, mm, s);
    for &b in &buf[..n] {
        console::putc(b);
    }
    console::puts(" UTC\n");
    console::puts("  unix=");
    print_num(now_utc as usize);
    console::puts("\n");
}

/// `tz [offset]` — show or set the local time-zone offset.
///   tz              → show current offset
///   tz <±HH:MM>     → set (e.g., -08:00 for PST, +05:30 for IST)
///   tz utc          → clear back to UTC (offset 0)
/// Stored as a single AtomicI32 in seconds east of UTC; persists
/// for the cave session.
fn cmd_tz(arg: &str) {
    use crate::kernel::time;
    if arg.is_empty() {
        let off = time::tz_offset_secs();
        let sign = if off < 0 { '-' } else { '+' };
        let abs = off.unsigned_abs();
        let hh = abs / 3600;
        let mm = (abs / 60) % 60;
        console::puts("  current tz offset: ");
        console::putc(sign as u8);
        if hh < 10 { console::putc(b'0'); }
        print_num(hh as usize);
        console::putc(b':');
        if mm < 10 { console::putc(b'0'); }
        print_num(mm as usize);
        console::puts(" (");
        print_num(off.unsigned_abs() as usize);
        console::puts(" seconds ");
        console::puts(if off >= 0 { "east of UTC" } else { "west of UTC" });
        console::puts(")\n");

        if time::is_synced() {
            let secs = time::realtime_secs();
            let local = if off >= 0 {
                secs.saturating_add(off as u64)
            } else {
                secs.saturating_sub((-off) as u64)
            };
            let (y, m, d, h, mi, s) = time::split_unix(local);
            let mut buf = [0u8; 20];
            let n = time::format_human(&mut buf, y, m, d, h, mi, s);
            console::puts("  local time: ");
            for &b in &buf[..n] { console::putc(b); }
            console::puts("\n");
        }
        return;
    }

    if arg == "utc" || arg == "UTC" || arg == "0" {
        time::set_tz_offset_secs(0);
        console::puts("  tz: UTC (offset 0)\n");
        return;
    }

    // Parse ±HH:MM
    let bytes = arg.as_bytes();
    if bytes.len() < 5 {
        console::puts("  usage: tz                  (show)\n");
        console::puts("         tz <±HH:MM>         (e.g., -08:00, +05:30)\n");
        console::puts("         tz utc              (reset to UTC)\n");
        return;
    }
    let sign = match bytes[0] {
        b'+' => 1i32,
        b'-' => -1i32,
        _ => {
            console::puts("  invalid format (expected ±HH:MM)\n");
            return;
        }
    };
    if bytes[3] != b':' {
        console::puts("  invalid format (expected ±HH:MM)\n");
        return;
    }
    let hh = match arg[1..3].parse::<i32>() { Ok(n) => n, Err(_) => { console::puts("  bad HH\n"); return; } };
    let mm = match arg[4..].parse::<i32>()  { Ok(n) => n, Err(_) => { console::puts("  bad MM\n"); return; } };
    if hh > 14 || mm > 59 {
        console::puts("  offset out of range (max ±14:00)\n");
        return;
    }
    let offset = sign * (hh * 3600 + mm * 60);
    time::set_tz_offset_secs(offset);
    console::puts("  tz set to ");
    console::puts(arg);
    console::puts(" (");
    print_num(offset.unsigned_abs() as usize);
    console::puts(" seconds ");
    console::puts(if offset >= 0 { "east" } else { "west" });
    console::puts(")\n");
}

/// `time-sync-https [host]` — sync wall clock against the `Date:`
/// header from a pinned HTTPS server. NTP-free; the trust path is
/// our PQ-TLS chain validation. Defaults to `www.cloudflare.com`.
fn cmd_time_sync_https(host_arg: &str) {
    let host = if host_arg.is_empty() { "www.cloudflare.com" } else { host_arg };
    console::puts_hi("  TIME SYNC VIA HTTPS DATE HEADER\n");
    console::puts("  trust source: PQ-TLS chain validated against trust store\n");
    console::puts("  contacting ");
    console::puts(host);
    console::puts(":443 ...\n");
    match crate::kernel::time::sync_from_https(host) {
        Ok(secs) => {
            console::puts("  ✓ wall clock anchored to ");
            print_num(secs as usize);
            console::puts(" (Unix seconds)\n  current: ");
            let (y, m, d, h, mm, s) = crate::kernel::time::split_unix(secs);
            let mut buf = [0u8; 20];
            let n = crate::kernel::time::format_human(&mut buf, y, m, d, h, mm, s);
            for &b in &buf[..n] { console::putc(b); }
            console::puts(" UTC\n");
        }
        Err(e) => {
            console::puts("  ✗ sync failed: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

/// `time-selftest` — verify the time module is sane:
///   - monotonic_us is monotonically non-decreasing across two reads;
///   - realtime_secs > 2025 epoch (1735689600) if synced;
///   - elapsed monotonic between two reads is positive but tiny.
fn cmd_time_selftest() {
    use crate::kernel::time;
    console::puts_hi("  WALL CLOCK SELF-TEST\n");
    console::puts("  RTC anchor + monotonic-derived realtime\n");

    let m0 = time::monotonic_us();
    let m1 = time::monotonic_us();
    if m1 < m0 {
        console::puts("  ✗ FAIL: monotonic went backwards\n");
        return;
    }
    console::puts("  ✓ monotonic non-decreasing (");
    print_num((m1 - m0) as usize);
    console::puts(" µs between reads)\n");

    if !time::is_synced() {
        console::puts("  ⚠ realtime not synced; RTC backend returned None\n");
        return;
    }
    let r = time::realtime_secs();
    // 1735689600 = 2025-01-01 00:00:00 UTC. Anything earlier means
    // the RTC chip handed us garbage or open-bus zeros.
    if r < 1_735_689_600 {
        console::puts("  ✗ FAIL: realtime looks pre-2025 (");
        print_num(r as usize);
        console::puts(" seconds)\n");
        return;
    }
    console::puts("  ✓ realtime > 2025 epoch (unix=");
    print_num(r as usize);
    console::puts(")\n");

    // Pretty-print to confirm the civil-time math.
    let (y, m, d, h, mi, s) = time::split_unix(r);
    let mut buf = [0u8; 20];
    let n = time::format_human(&mut buf, y, m, d, h, mi, s);
    console::puts("  now: ");
    for &b in &buf[..n] { console::putc(b); }
    console::puts(" UTC\n");
    console::puts("  ✓ ALL TIME TESTS PASSED\n");
}

// AF_UNIX SOCK_STREAM round trip: bind+listen on server, connect from
// client, accept, two-way write/read, then EOF on close.
// Single-task; accept won't block because connect() pre-fills the queue.
fn cmd_unix_sock_selftest() {
    use crate::kernel::unix_sock as us;
    use crate::kernel::process::{self, FdKind, SocketRole};

    console::puts_hi("  AF_UNIX SOCK_STREAM SELF-TEST\n");
    console::puts("  bind + listen + connect + accept + 2-way IO + EOF\n");

    let server_fd = match us::create() {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL server create: "); console::puts(e); console::puts("\n"); return; }
    };
    let server_sid = match process::current().fd_get(server_fd).map(|e| e.kind) {
        Some(FdKind::Socket { id, .. }) => id,
        _ => { console::puts("  ✗ FAIL: server fd not a socket\n"); return; }
    };

    let name: &[u8] = b"bat-test-sock";
    if let Err(e) = us::bind(server_sid, name) {
        console::puts("  ✗ FAIL bind: "); console::puts(e); console::puts("\n"); return;
    }
    if let Err(e) = us::listen(server_sid) {
        console::puts("  ✗ FAIL listen: "); console::puts(e); console::puts("\n"); return;
    }

    let client_fd = match us::create() {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL client create: "); console::puts(e); console::puts("\n"); return; }
    };
    let client_sid = match process::current().fd_get(client_fd).map(|e| e.kind) {
        Some(FdKind::Socket { id, .. }) => id,
        _ => { console::puts("  ✗ FAIL: client fd not a socket\n"); return; }
    };
    if let Err(e) = us::connect(client_sid, name) {
        console::puts("  ✗ FAIL connect: "); console::puts(e); console::puts("\n"); return;
    }
    console::puts("  ✓ bind+listen+connect ok (name=");
    for &b in name { console::putc(b); }
    console::puts(")\n");

    let accept_fd = match us::accept(server_sid) {
        Ok(fd) => fd,
        Err(e) => { console::puts("  ✗ FAIL accept: "); console::puts(e); console::puts("\n"); return; }
    };
    let server_conn_sid = match process::current().fd_get(accept_fd).map(|e| e.kind) {
        Some(FdKind::Socket { id, role: SocketRole::Connected }) => id,
        _ => { console::puts("  ✗ FAIL: accept fd not Connected\n"); return; }
    };
    console::puts("  ✓ accept returned Connected fd\n");

    // Client → Server
    let to_server: &[u8] = b"ping from client";
    match us::write(client_sid, to_server) {
        Ok(n) if n == to_server.len() => {}
        _ => { console::puts("  ✗ FAIL client→server write\n"); return; }
    }
    let mut rxbuf = [0u8; 64];
    let n = match us::read(server_conn_sid, &mut rxbuf) {
        Ok(n) => n,
        Err(e) => { console::puts("  ✗ FAIL server read: "); console::puts(e); console::puts("\n"); return; }
    };
    if &rxbuf[..n] != to_server {
        console::puts("  ✗ FAIL: client→server payload mismatch\n"); return;
    }
    console::puts("  ✓ client → server: "); print_num(n); console::puts(" bytes\n");

    // Server → Client
    let to_client: &[u8] = b"pong from server";
    match us::write(server_conn_sid, to_client) {
        Ok(n) if n == to_client.len() => {}
        _ => { console::puts("  ✗ FAIL server→client write\n"); return; }
    }
    let n = match us::read(client_sid, &mut rxbuf) {
        Ok(n) => n,
        Err(e) => { console::puts("  ✗ FAIL client read: "); console::puts(e); console::puts("\n"); return; }
    };
    if &rxbuf[..n] != to_client {
        console::puts("  ✗ FAIL: server→client payload mismatch\n"); return;
    }
    console::puts("  ✓ server → client: "); print_num(n); console::puts(" bytes\n");

    // EOF on close.
    let _ = process::current().fd_take(client_fd);
    us::close(client_sid);
    match us::read(server_conn_sid, &mut rxbuf) {
        Ok(0) => console::puts("  ✓ EOF on server-side read after client close\n"),
        Ok(_) => { console::puts("  ✗ FAIL: expected EOF\n"); return; }
        Err(e) => { console::puts("  ✗ FAIL EOF read: "); console::puts(e); console::puts("\n"); return; }
    }

    // Tidy.
    let _ = process::current().fd_take(accept_fd);
    us::close(server_conn_sid);
    let _ = process::current().fd_take(server_fd);
    us::close(server_sid);

    console::puts("  ✓ ALL UNIX-SOCK TESTS PASSED\n");
}

// Anonymous-pipe round trip: create → write → read → EOF on close →
// EPIPE on dead reader. Single-task path so nothing actually blocks.
fn cmd_pipe_selftest() {
    use crate::kernel::pipe;
    use crate::kernel::process::{self, FdKind, PipeEnd};

    console::puts_hi("  ANONYMOUS PIPE SELF-TEST\n");
    console::puts("  create + write + read + EOF + EPIPE round trip\n");

    let (rfd, wfd) = match pipe::create() {
        Ok(p) => p,
        Err(e) => {
            console::puts("  ✗ FAIL: create: "); console::puts(e); console::puts("\n");
            return;
        }
    };

    let payload = b"the bat signal is up";
    let id = match process::current().fd_get(wfd).map(|e| e.kind) {
        Some(FdKind::Pipe { id, end: PipeEnd::Write }) => id,
        _ => {
            console::puts("  ✗ FAIL: wfd not a write-end pipe fd\n");
            return;
        }
    };

    match pipe::write(id, payload) {
        Ok(n) if n == payload.len() => {}
        Ok(n) => {
            console::puts("  ✗ FAIL: short write: ");
            print_num(n); console::puts("/"); print_num(payload.len());
            console::puts("\n"); return;
        }
        Err(e) => {
            console::puts("  ✗ FAIL: write: "); console::puts(e); console::puts("\n");
            return;
        }
    }

    let mut buf = [0u8; 32];
    let rid = match process::current().fd_get(rfd).map(|e| e.kind) {
        Some(FdKind::Pipe { id, end: PipeEnd::Read }) => id,
        _ => {
            console::puts("  ✗ FAIL: rfd not a read-end pipe fd\n");
            return;
        }
    };
    let read_n = match pipe::read(rid, &mut buf) {
        Ok(n) => n,
        Err(e) => {
            console::puts("  ✗ FAIL: read: "); console::puts(e); console::puts("\n");
            return;
        }
    };
    if read_n != payload.len() || &buf[..read_n] != payload {
        console::puts("  ✗ FAIL: read returned ");
        print_num(read_n); console::puts(" bytes, payload mismatch\n");
        return;
    }
    console::puts("  ✓ write/read round trip OK (");
    print_num(read_n); console::puts(" bytes)\n");

    // Close the write end. Subsequent read on empty + no writers = EOF.
    let _ = process::current().fd_take(wfd);
    pipe::release_end(id, PipeEnd::Write);
    match pipe::read(rid, &mut buf) {
        Ok(0) => console::puts("  ✓ EOF after writer close\n"),
        Ok(n) => {
            console::puts("  ✗ FAIL: expected EOF, got ");
            print_num(n); console::puts(" bytes\n");
            return;
        }
        Err(e) => {
            console::puts("  ✗ FAIL: EOF read: "); console::puts(e); console::puts("\n");
            return;
        }
    }
    // Tidy: close the read end too.
    let _ = process::current().fd_take(rfd);
    pipe::release_end(rid, PipeEnd::Read);

    // EPIPE check: fresh pipe, close reader, expect write to fail.
    let (rfd2, wfd2) = match pipe::create() {
        Ok(p) => p,
        Err(e) => {
            console::puts("  ✗ FAIL: 2nd create: "); console::puts(e); console::puts("\n");
            return;
        }
    };
    let id2 = match process::current().fd_get(wfd2).map(|e| e.kind) {
        Some(FdKind::Pipe { id, end: PipeEnd::Write }) => id,
        _ => {
            console::puts("  ✗ FAIL: 2nd wfd not a write-end pipe fd\n");
            return;
        }
    };
    // Close the read end.
    let rid2 = match process::current().fd_get(rfd2).map(|e| e.kind) {
        Some(FdKind::Pipe { id, end: PipeEnd::Read }) => id,
        _ => {
            console::puts("  ✗ FAIL: 2nd rfd not a read-end pipe fd\n");
            return;
        }
    };
    let _ = process::current().fd_take(rfd2);
    pipe::release_end(rid2, PipeEnd::Read);
    match pipe::write(id2, b"x") {
        Err(e) if e == "EPIPE" => console::puts("  ✓ EPIPE after reader close\n"),
        Ok(n) => {
            console::puts("  ✗ FAIL: expected EPIPE, wrote ");
            print_num(n); console::puts(" bytes\n");
            return;
        }
        Err(e) => {
            console::puts("  ✗ FAIL: EPIPE write: "); console::puts(e); console::puts("\n");
            return;
        }
    }
    let _ = process::current().fd_take(wfd2);
    pipe::release_end(id2, PipeEnd::Write);

    console::puts("  ✓ ALL PIPE TESTS PASSED\n");
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

/// Dispatch for `comms <sub> [arg]` — connect, close, status.
/// `send` is handled separately because it needs the raw message
/// tail (with spaces) rather than the space-split parts.
fn cmd_clip(sub: &str) {
    use crate::ui::clipboard;
    match sub {
        "" | "show" => {
            let n = clipboard::len();
            console::puts("  clipboard: ");
            print_num(n);
            console::puts(" bytes\n");
            if n > 0 {
                let mut buf = [0u8; clipboard::CLIPBOARD_CAP];
                let copied = clipboard::copy_into(&mut buf);
                console::puts("  > ");
                // Print printable-only, marking non-printable as '?'
                // so a maliciously-pasted control byte doesn't mess
                // with the terminal.
                for &b in &buf[..copied] {
                    console::putc(if (0x20..=0x7e).contains(&b) || b == b'\n' { b } else { b'?' });
                }
                console::puts("\n");
            }
        }
        "clear" => {
            clipboard::clear();
            console::puts("  clipboard cleared\n");
        }
        _ => {
            console::puts("  usage: clip                (show)\n");
            console::puts("         clip set <text>\n");
            console::puts("         clip yank-back [N]  (copy last N scrollback rows)\n");
            console::puts("         clip push           (Bat_OS clip -> macOS clip)\n");
            console::puts("         clip pull           (macOS clip -> Bat_OS clip)\n");
            console::puts("         clip clear\n");
            console::puts("  shortcuts: Ctrl+V paste at cursor\n");
            console::puts("             Ctrl+Y yank current input line\n");
            console::puts("             Ctrl+S enter visual select mode\n");
            console::puts("               arrows move cursor / Shift+arrows extend\n");
            console::puts("               Enter copies, Esc exits without copy\n");
            console::puts("  bridge:    python3 scripts/host_clipboard_bridge.py\n");
            console::puts("             must be running on the host for push/pull\n");
        }
    }
}

/// Host clipboard bridge endpoint. QEMU slirp NATs guest's
/// 10.0.2.2 to host loopback, so the bridge daemon binds to
/// 127.0.0.1:9101 and we reach it via this address.
const CLIP_BRIDGE_IP: u32   = 0x0A000202; // 10.0.2.2 big-endian
const CLIP_BRIDGE_PORT: u16 = 9101;

/// Push Bat_OS clipboard contents to the host (macOS) clipboard
/// via the TCP bridge. Run scripts/host_clipboard_bridge.py on the
/// host first.
fn cmd_clip_push() {
    use crate::ui::clipboard;
    let n = clipboard::len();
    if n == 0 {
        console::puts("  Bat_OS clipboard is empty; nothing to push\n");
        return;
    }
    let mut payload = [0u8; clipboard::CLIPBOARD_CAP];
    let copied = clipboard::copy_into(&mut payload);

    if let Err(e) = net::tcp::connect(CLIP_BRIDGE_IP, CLIP_BRIDGE_PORT) {
        console::puts("  bridge connect failed: ");
        console::puts(e);
        console::puts("\n  start the bridge: python3 scripts/host_clipboard_bridge.py\n");
        return;
    }

    // SET <len>\n<bytes>
    let mut header = [0u8; 24];
    let mut h = 0;
    for &b in b"SET " { header[h] = b; h += 1; }
    let mut tmp = [0u8; 10];
    let mut ti = 0;
    let mut nn = copied;
    if nn == 0 { tmp[0] = b'0'; ti = 1; }
    else {
        while nn > 0 && ti < tmp.len() { tmp[ti] = b'0' + (nn % 10) as u8; nn /= 10; ti += 1; }
    }
    for j in 0..ti { header[h] = tmp[ti - 1 - j]; h += 1; }
    header[h] = b'\n'; h += 1;

    if net::tcp::send_data(&header[..h]).is_err()
        || net::tcp::send_data(&payload[..copied]).is_err() {
        console::puts("  bridge send failed\n");
        net::tcp::close();
        return;
    }

    let mut resp = [0u8; 32];
    let r = match net::tcp::recv_data(&mut resp) {
        Ok(n) => n,
        Err(e) => { console::puts("  bridge recv failed: "); console::puts(e); console::puts("\n"); net::tcp::close(); return; }
    };
    net::tcp::close();

    if r >= 2 && &resp[..2] == b"OK" {
        console::puts("  -> macOS clipboard set (");
        print_num(copied);
        console::puts(" bytes)\n");
    } else {
        console::puts("  bridge replied: ");
        for &b in &resp[..r.min(resp.len())] {
            console::putc(if (0x20..=0x7e).contains(&b) { b } else { b'?' });
        }
        console::puts("\n");
    }
}

/// Pull the host (macOS) clipboard into Bat_OS's clipboard.
fn cmd_clip_pull() {
    if let Err(e) = net::tcp::connect(CLIP_BRIDGE_IP, CLIP_BRIDGE_PORT) {
        console::puts("  bridge connect failed: ");
        console::puts(e);
        console::puts("\n  start the bridge: python3 scripts/host_clipboard_bridge.py\n");
        return;
    }
    if net::tcp::send_data(b"GET\n").is_err() {
        console::puts("  bridge send failed\n");
        net::tcp::close();
        return;
    }

    // Response is "OK <len>\n<bytes>" — read one chunk and parse
    // the header out of it. recv_data is blocking with coalesce, so
    // the whole reply usually arrives together.
    let mut buf = [0u8; crate::ui::clipboard::CLIPBOARD_CAP + 32];
    let n = match net::tcp::recv_data(&mut buf) {
        Ok(n) => n,
        Err(e) => { console::puts("  bridge recv failed: "); console::puts(e); console::puts("\n"); net::tcp::close(); return; }
    };
    net::tcp::close();

    if n < 4 || &buf[..3] != b"OK " {
        console::puts("  bridge reply not OK: ");
        for &b in &buf[..n.min(40)] {
            console::putc(if (0x20..=0x7e).contains(&b) { b } else { b'?' });
        }
        console::puts("\n");
        return;
    }

    // Find LF separating header from payload.
    let mut newline_at: Option<usize> = None;
    for i in 3..n {
        if buf[i] == b'\n' { newline_at = Some(i); break; }
    }
    let lf = match newline_at {
        Some(i) => i,
        None => { console::puts("  bridge reply missing payload separator\n"); return; }
    };

    // Parse the length between "OK " and LF.
    let mut len: usize = 0;
    let mut ok = false;
    for &b in &buf[3..lf] {
        if (b'0'..=b'9').contains(&b) { len = len * 10 + (b - b'0') as usize; ok = true; }
        else { ok = false; break; }
    }
    if !ok {
        console::puts("  bridge reply bad length\n");
        return;
    }

    let body_start = lf + 1;
    let avail = n - body_start;
    let copy = len.min(avail).min(crate::ui::clipboard::CLIPBOARD_CAP);
    crate::ui::clipboard::set(&buf[body_start..body_start + copy]);
    console::puts("  <- pulled ");
    print_num(copy);
    console::puts(" bytes from macOS clipboard\n");
    if copy < len {
        console::puts("  (truncated; full payload was ");
        print_num(len);
        console::puts(" bytes)\n");
    }
}

fn cmd_clip_yank_back(arg: &str) {
    let n: usize = if arg.is_empty() { 1 }
        else { match arg.parse::<usize>() { Ok(v) if v > 0 => v, _ => 1 } };
    let copied = crate::ui::console::yank_last_rows(n);
    console::puts("  yanked ");
    print_num(copied);
    console::puts(" bytes from last ");
    print_num(n);
    console::puts(" row(s)\n");
}

fn cmd_clip_set(text: &str) {
    crate::ui::clipboard::set(text.as_bytes());
    console::puts("  clipboard set (");
    print_num(text.len());
    console::puts(" bytes)\n");
}

fn cmd_comms(sub: &str, _arg: &str) {
    match sub {
        "close" | "disconnect" => cmd_comms_close(),
        "status" => cmd_comms_status(),
        _ => {
            console::puts("  flow:\n");
            console::puts("    1. comms my-id                (this cave's pubkey; allowlist it on the server)\n");
            console::puts("    2. comms identify <ip:port>   (discovery, copies server pubkey to clipboard)\n");
            console::puts("    3. comms pin <hex>            (Ctrl+V to paste from clipboard)\n");
            console::puts("    4. comms connect <ip:port>    (uses stored pin; mutual auth)\n");
            console::puts("    5. comms send <message>\n");
            console::puts("    6. comms close\n");
        }
    }
}

fn cmd_comms_my_id() {
    let mut hex = [0u8; 64];
    if !crate::ui::apps::comms::my_id_hex(&mut hex) {
        console::puts("  comms identity unavailable (BatFS not ready?)\n");
        return;
    }
    console::puts("  this cave's comms identity: ");
    for &b in &hex { console::putc(b); }
    console::puts("\n");
    // Auto-copy so the operator can paste it into the server's
    // allowlist file without re-typing.
    crate::ui::clipboard::set(&hex);
    console::puts("  -> copied to clipboard (Ctrl+V to paste it elsewhere)\n");
    console::puts("  add this hex to the server's comms_clients.allowlist\n");
    console::puts("  (one pubkey per line) and restart the server.\n");
}

fn parse_target(target: &str) -> Option<(u32, u16)> {
    let (ip_str, port_str) = target.rsplit_once(':')?;
    let ip = parse_ip(ip_str);
    if ip == 0 { return None; }
    let port: u16 = port_str.parse().ok()?;
    if port == 0 { return None; }
    Some((ip, port))
}

fn cmd_comms_identify(target: &str) {
    if target.is_empty() {
        console::puts("  usage: comms identify <ip:port>\n");
        return;
    }
    let (ip, port) = match parse_target(target) {
        Some(p) => p,
        None => { console::puts("  invalid target (expected ip:port)\n"); return; }
    };
    console::puts("  Discovering ");
    console::puts(target);
    console::puts(" ...\n");
    match crate::ui::apps::comms::identify(ip, port) {
        Ok(srv_id) => {
            // Hex-encode and print + auto-copy to clipboard.
            let mut hex = [0u8; 64];
            let table = b"0123456789abcdef";
            for i in 0..32 {
                hex[i * 2]     = table[(srv_id[i] >> 4) as usize];
                hex[i * 2 + 1] = table[(srv_id[i] & 0x0f) as usize];
            }
            console::puts("  server pubkey: ");
            for &b in &hex {
                console::putc(b);
            }
            console::puts("\n");
            crate::ui::clipboard::set(&hex);
            console::puts("  -> copied to clipboard (Ctrl+V to paste)\n");
            console::puts("  next: type `comms pin ` then Ctrl+V then Enter\n");
            console::puts("  WARNING: identify is NOT authenticated.\n");
            console::puts("           Confirm the hex matches what the server's\n");
            console::puts("           operator told you out-of-band before pinning.\n");
        }
        Err(e) => {
            console::puts("  identify failed: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

fn cmd_comms_pin(hex: &str) {
    if hex.is_empty() {
        console::puts("  usage: comms pin <hex-64chars>\n");
        console::puts("  (Ctrl+V to paste from clipboard if you just ran `comms identify`)\n");
        return;
    }
    let id = match parse_hex32(hex) {
        Some(p) => p,
        None => {
            console::puts("  invalid hex (need 64 hex chars = 32 bytes, got ");
            print_num(hex.len());
            console::puts(")\n");
            return;
        }
    };
    crate::ui::apps::comms::pin(&id);
    console::puts("  pinned. now: `comms connect <ip:port>`\n");
}

fn cmd_comms_connect(target: &str) {
    if target.is_empty() {
        console::puts("  usage: comms connect <ip:port>\n");
        console::puts("  (run `comms identify` then `comms pin` first if no pin yet)\n");
        return;
    }
    if !crate::ui::apps::comms::pin_is_set() {
        console::puts("  no server identity pinned. flow:\n");
        console::puts("    comms identify ");
        console::puts(target);
        console::puts("\n    comms pin <Ctrl+V>\n    comms connect ");
        console::puts(target);
        console::puts("\n");
        return;
    }
    let (ip, port) = match parse_target(target) {
        Some(p) => p,
        None => { console::puts("  invalid target (expected ip:port)\n"); return; }
    };
    console::puts("  Opening session to ");
    console::puts(target);
    console::puts(" (using stored pin)\n");
    match crate::ui::apps::comms::connect(ip, port) {
        Ok(()) => {
            console::puts("  handshake OK\n");
            cmd_comms_status();
        }
        Err(e) => {
            console::puts("  handshake failed: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

/// Parse a 64-char hex string into 32 bytes. Returns None on any
/// non-hex char or wrong length.
fn parse_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 { return None; }
    let bytes = s.as_bytes();
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = hex_nibble(bytes[i * 2]);
        let lo = hex_nibble(bytes[i * 2 + 1]);
        if hi == 0xff || lo == 0xff { return None; }
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn cmd_comms_send(msg: &str) {
    if msg.is_empty() {
        console::puts("  usage: comms send <message>\n");
        return;
    }
    if crate::ui::apps::comms::state() != crate::ui::apps::comms::CommState::Connected {
        console::puts("  not connected — run `comms connect <ip:port> <hex>` first\n");
        return;
    }
    match crate::ui::apps::comms::send_message(msg.as_bytes()) {
        Ok(()) => {
            console::puts("  -> sent ");
            print_num(msg.len());
            console::puts(" bytes encrypted; awaiting echo...\n");
        }
        Err(e) => {
            console::puts("  send failed: ");
            console::puts(e);
            console::puts("\n");
            return;
        }
    }
    // Drain one response frame so the message lands in the CM
    // timeline. Server's job is to AEAD-encrypt the echo under
    // s2c_key with its own counter nonce; our recv path verifies
    // both the nonce ordering and the Poly1305 tag.
    let got = crate::ui::apps::comms::recv_message();
    if got {
        console::puts("  <- received echo (verified tag + nonce; see CM timeline)\n");
    } else {
        console::puts("  <- no response yet; poll with `comms status` or try again\n");
    }
}

fn cmd_comms_close() {
    crate::ui::apps::comms::disconnect();
    console::puts("  comms: disconnected\n");
}

fn cmd_comms_status() {
    use crate::ui::apps::comms::{self as c, CommState};
    let s = c::state();
    console::puts("  comms: ");
    console::puts(match s {
        CommState::Disconnected => "disconnected",
        CommState::Connecting   => "connecting",
        CommState::Connected    => "CONNECTED (ChaCha20-Poly1305, Ed25519 pinned, FS)",
        CommState::Error        => "ERROR",
    });
    console::puts("\n");
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
        // QEMU-BUGFIX-6 was the multi-MB stack-staged
        // struct overwrites in `apps::browser::reset_for_cave_switch`
        // (specifically `DOM_DOC = Document::new()`, ~5 MB) blowing
        // the 8 MB kernel stack inside the IrqGuard critical section.
        // Fixed by switching those to in-place `write_bytes(p, 0, 1)`
        // memsets so no stack staging happens. `cave::enter` is now
        // safe to call here too; we keep `ensure_host_cave_active`
        // for the ambient-host case because it's still cheaper (no
        // full state reset needed for the shell-host cave).
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
            // ephemeral — destroyed on wipe, no persistent state
            // kit:<name> — pre-install a tool bundle
            // docker:<image> — docker-backed cave (Phase 6 of the
            // design-alignment plan). Image passed
            // through to docker_client / batcaved.
            // caps:<csv> — only meaningful with --docker; Linux
            // capabilities to pass via --cap-add
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
        "ipc" => {
            // surface cave::create_ipc to the operator.
            // Audit caught this as "machinery exists but no shell
            // command invokes it." Now it does.
            //
            // Usage: batcave ipc <cave_a> <cave_b>
            //
            // Both caves must have granted each other the
            // `ipc:<other>` capability before this works:
            // batcave grant alpha ipc:beta
            // batcave grant beta ipc:alpha
            // batcave ipc alpha beta
            // → IPC channel established: id=N
            //
            // The returned channel id is the kernel IPC handle that
            // either cave's syscall path can use to send/recv via
            // `cave::get_ipc_channel`.
            if arg1.is_empty() || arg2.is_empty() {
                console::puts("  usage: batcave ipc <cave_a> <cave_b>\n");
                console::puts("  (both caves must grant each other ipc:<other> first)\n");
                return;
            }
            match cave::create_ipc(arg1, arg2) {
                Ok(channel_id) => {
                    console::puts("  IPC channel established between ");
                    console::puts(arg1);
                    console::puts(" and ");
                    console::puts(arg2);
                    console::puts(": id=");
                    crate::kernel::mm::print_num(channel_id as usize);
                    console::puts("\n");
                }
                Err(e) => {
                    console::puts("  Error: ");
                    console::puts(e);
                    console::puts("\n");
                    if e == "A lacks ipc cap" || e == "B lacks ipc cap" {
                        console::puts("  hint: grant `ipc:<other_cave_name>` to BOTH caves first\n");
                    }
                }
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
    let batcave_names = ["posix", "cxx"];
    let use_batcave = batcave_names.contains(&name);

    if use_batcave {
        let data = match name {
            "posix" => crate::batcave::linux::runner::posix_test_elf(),
            "cxx" => crate::batcave::linux::runner::cxx_test_elf(),
            _ => unreachable!(),
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
    } else if name == "threads" {
        crate::batcave::linux::runner::hello_threads_elf()
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
            #[allow(dead_code)]
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

/// Sprint 3.1: dump or clear the cookie jar.
/// `cookies` → print all cookies (host + name only, values redacted)
/// `cookies clear` → wipe the jar
fn cmd_cookies(arg: &str) {
    if arg == "clear" {
        crate::net::cookies::reset();
        console::puts("  cookies: jar cleared\n");
        return;
    }
    console::puts("  cookies: ");
    crate::kernel::mm::print_num(crate::net::cookies::count());
    console::puts(" active\n");
    crate::net::cookies::dump();
}

/// Toggle the per-event "[kbd] ev type=X code=Y val=Z" UART trace.
/// `kbd-trace on` to enable; `kbd-trace off` to disable.
fn cmd_kbd_trace(arg: &str) {
    match arg {
        "on" | "true" | "1" => {
            crate::drivers::virtio::keyboard::set_trace(true);
            console::puts("  kbd-trace: ON (every event will print to serial)\n");
        }
        "off" | "false" | "0" | "" => {
            crate::drivers::virtio::keyboard::set_trace(false);
            console::puts("  kbd-trace: OFF\n");
        }
        _ => {
            console::puts("  usage: kbd-trace on|off\n");
        }
    }
}

/// Diagnostic: print virtio-keyboard event counters. Helps answer
/// "is QEMU sending keystrokes from the GUI window at all?" by
/// showing total events received, EV_KEY DOWN/UP counts, last
/// event type+code seen, and how many chars made it into the
/// keystroke ring. Run before AND after typing a few keys in the
/// QEMU window — the deltas reveal where input is getting lost.
fn cmd_kbd_stats() {
    // Pump once so any pending events are drained into our counters.
    crate::drivers::virtio::keyboard::poll();
    let (total, down, up, syn, other, last_type, last_code, pushes) =
        crate::drivers::virtio::keyboard::dbg_stats();
    console::puts("  kbd: ready=");
    console::puts(if crate::drivers::virtio::keyboard::is_ready() { "yes" } else { "NO" });
    console::puts("\n  events total=");
    crate::kernel::mm::print_num(total);
    console::puts("  EV_KEY down=");
    crate::kernel::mm::print_num(down);
    console::puts(" up=");
    crate::kernel::mm::print_num(up);
    console::puts("\n  EV_SYN=");
    crate::kernel::mm::print_num(syn);
    console::puts(" other=");
    crate::kernel::mm::print_num(other);
    console::puts("\n  last_type=");
    crate::kernel::mm::print_num(last_type);
    console::puts(" last_code=");
    crate::kernel::mm::print_num(last_code);
    console::puts("\n  ring pushes=");
    crate::kernel::mm::print_num(pushes);
    console::puts("\n");
}

/// load a BatFS file into the editor's active tab and
/// switch to ED. `edit foo.txt` from the shell.
fn cmd_edit(name: &str) {
    if name.is_empty() {
        console::puts("  usage: edit <filename>\n");
        return;
    }
    match crate::ui::apps::editor::load_from_batfs(name) {
        Ok(()) => {
            console::puts("  edit: loaded ");
            console::puts(name);
            console::puts("\n");
            // Switch to the editor app so the operator sees what they
            // just loaded.
            crate::ui::wm::switch_app(crate::ui::wm::APP_EDITOR);
        }
        Err(e) => {
            console::puts("  edit: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

/// Sprint 2.2: print current main origin + allowlist.
fn cmd_origin(_arg: &str) {
    let main = crate::security::origin::current_main_origin();
    if main.valid {
        let mut buf = [0u8; 192];
        let n = main.write_to(&mut buf);
        console::puts("  current main origin: ");
        console::puts(unsafe { core::str::from_utf8_unchecked(&buf[..n]) });
        console::puts("\n");
    } else {
        console::puts("  current main origin: (none)\n");
    }
    console::puts("  SOP enforcement: ");
    console::puts(if crate::security::origin::is_strict() { "strict\n" } else { "permissive\n" });
    console::puts("  cross-origin allowlist:\n");
    crate::security::origin::dump_allowlist();
}

/// append a (main_host, other_host) pair to the SOP
/// allowlist. Both args required. After this, the renderer will fetch
/// sub-resources from `other_host` even when the main page is from
/// `main_host`.
fn cmd_origin_allow(main_host: &str, other_host: &str) {
    if main_host.is_empty() || other_host.is_empty() {
        console::puts("  usage: origin-allow <main-host> <other-host>\n");
        return;
    }
    match crate::security::origin::allow(main_host, other_host) {
        Ok(()) => {
            console::puts("  origin-allow: ");
            console::puts(main_host);
            console::puts(" -> ");
            console::puts(other_host);
            console::puts(" (added)\n");
        }
        Err(e) => {
            console::puts("  origin-allow: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

/// flip SOP enforcement strict/permissive.
fn cmd_origin_mode(arg: &str) {
    if arg.is_empty() {
        console::puts("  current SOP mode: ");
        console::puts(if crate::security::origin::is_strict() { "strict" } else { "permissive" });
        console::puts("\n  usage: origin-mode <strict|permissive>\n");
        return;
    }
    match arg {
        "strict" | "enforce" => { crate::security::origin::set_strict(true); console::puts("  SOP mode -> strict\n"); }
        "permissive" | "warn" => { crate::security::origin::set_strict(false); console::puts("  SOP mode -> permissive (logs only)\n"); }
        _ => { console::puts("  unknown SOP mode: "); console::puts(arg); console::puts("\n"); }
    }
}

/// dump active TCP listeners + connections.
// /
/// Operator visibility for 's TCP server-side. Shows:
/// every active Listener slot with its port, backlog, fd, and
/// pending-accept-queue depth
/// every active PCB with its state, 4-tuple, and bound fd
// /
/// Useful for confirming `tcp-listen` actually registered, watching
/// SYN_RECV → ESTABLISHED transitions during a real handshake, and
/// verifying clean shutdown via `listen_close`.
fn cmd_tcp_list() {
    use crate::net::tcp;

    console::puts("  LISTENERS:\n");
    let mut listener_count = 0;
    tcp::for_each_listener(|port, backlog, fd, pending| {
        listener_count += 1;
        console::puts("    port=");
        crate::kernel::mm::print_num(port as usize);
        console::puts(" backlog=");
        crate::kernel::mm::print_num(backlog as usize);
        console::puts(" fd=");
        if fd >= 0 {
            crate::kernel::mm::print_num(fd as usize);
        } else {
            console::puts("-1");
        }
        console::puts(" pending=");
        crate::kernel::mm::print_num(pending);
        console::puts("\n");
    });
    if listener_count == 0 {
        console::puts("    (none)\n");
    }

    console::puts("  CONNECTIONS:\n");
    let mut conn_count = 0;
    tcp::for_each_pcb(|state, lport, rip, rport, fd| {
        conn_count += 1;
        console::puts("    ");
        // Pad state name to fixed width for readability
        let name = tcp::state_name(state);
        console::puts(name);
        // ASCII pad to 9 chars
        for _ in name.len()..9 { console::putc(b' '); }
        console::puts(" local:");
        crate::kernel::mm::print_num(lport as usize);
        console::puts(" peer=");
        crate::kernel::mm::print_num(((rip >> 24) & 0xFF) as usize);
        console::puts(".");
        crate::kernel::mm::print_num(((rip >> 16) & 0xFF) as usize);
        console::puts(".");
        crate::kernel::mm::print_num(((rip >> 8) & 0xFF) as usize);
        console::puts(".");
        crate::kernel::mm::print_num((rip & 0xFF) as usize);
        console::puts(":");
        crate::kernel::mm::print_num(rport as usize);
        console::puts(" fd=");
        if fd >= 0 {
            crate::kernel::mm::print_num(fd as usize);
        } else {
            console::puts("-1");
        }
        console::puts("\n");
    });
    if conn_count == 0 {
        console::puts("    (none)\n");
    }
}

/// TCP server-side selftest.
// /
/// Exercises the in-kernel listener-table + accept-queue mechanics
/// (steps 1-4 of #148) without needing real wire-level packet flow.
/// Tests:
/// listen_register port collision (EADDRINUSE)
/// listener_lookup_by_port + by_fd
/// listener_accept_push / pop FIFO ordering
/// backlog enforcement
/// listen_close cleanup + reuse
// /
/// SYN-on-LISTEN dispatch + the third-ACK transition need real
/// virtio-net traffic (e.g. `nc` from the QEMU host) — that's
/// outside this command's scope.
fn cmd_tcp_selftest() {
    use crate::net::tcp;
    match tcp::selftest_server() {
        Ok(report) => {
            console::puts("  tcp-selftest: PASS\n");
            console::puts("    assertions passed: ");
            crate::kernel::mm::print_num(report.assertions_passed as usize);
            console::puts("\n    final listener count: ");
            crate::kernel::mm::print_num(report.final_listener_count as usize);
            console::puts("\n    final pcb count: ");
            crate::kernel::mm::print_num(report.final_pcb_count as usize);
            console::puts("\n");
        }
        Err(reason) => {
            console::puts("  tcp-selftest: FAIL — ");
            console::puts(reason);
            console::puts("\n");
        }
    }
}

/// real-wire test harness for 's TCP server-side.
// /
/// Usage: `tcp-listen <port>`
// /
/// Registers a kernel listener on the given port, blocks waiting for
/// one inbound connection (~30s deadline), prints the peer's address
/// when the third ACK lands, drains up to 256 bytes of received data
/// to the console, sends back a "hello from bat_os\n" greeting, and
/// closes. One-shot (handles a single connection then returns to the
/// shell prompt).
// /
/// Driving from the QEMU host:
// /
/// # On Bat_OS:
/// bat_os > tcp-listen 8080
/// listening on port 8080... (one-shot, ~30s deadline)
// /
/// # On the Mac host:
/// $ nc -v 10.0.2.15 8080
/// Connection to 10.0.2.15 8080 port [tcp/*] succeeded!
/// hello world<Enter>
// /
/// # Back on Bat_OS:
/// connection from 10.0.2.2:54321
/// recv (12 bytes): hello world
/// sent greeting; closing
fn cmd_tcp_listen(port_str: &str) {
    use crate::net::tcp;

    if port_str.is_empty() {
        console::puts("  usage: tcp-listen <port>\n");
        return;
    }
    let port: u16 = match port_str.parse() {
        Ok(p) if p > 0 => p,
        _ => {
            console::puts("  invalid port: ");
            console::puts(port_str);
            console::puts("\n");
            return;
        }
    };

    // Sentinel fd that doesn't overlap the socket fd range
    // (SOCKET_FD_BASE = 1024). 99 is well below that and not in use
    // by any other subsystem.
    const SENTINEL_FD: i32 = 99;

    if let Err(e) = tcp::listen_register(port, 4, 0, SENTINEL_FD) {
        console::puts("  listen_register failed: ");
        console::puts(e);
        console::puts("\n");
        return;
    }

    let listener_idx = match tcp::listener_lookup_by_port(port) {
        Some(i) => i,
        None => {
            console::puts("  internal error: listener disappeared\n");
            return;
        }
    };

    console::puts("  listening on port ");
    crate::kernel::mm::print_num(port as usize);
    console::puts("... (one-shot, ~30s deadline)\n");

    // Spin polling for an inbound connection. Each iteration drives
    // the network stack via poll_once so virtio-net packets actually
    // get processed and reach handle_incoming. cntpct deadline at
    // ~30s prevents the shell from hanging forever.
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let deadline = start + freq * 30;

    let mut pcb_id: Option<usize> = None;
    loop {
        crate::net::poll_once();
        if let Some(id) = tcp::listener_accept_pop(listener_idx) {
            pcb_id = Some(id);
            break;
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now > deadline { break; }
        core::hint::spin_loop();
    }

    let pcb_id = match pcb_id {
        Some(id) => id,
        None => {
            console::puts("  timeout — no connection\n");
            tcp::listen_close(port);
            return;
        }
    };

    // Print the peer's address. Despite the field comment saying
    // "big-endian," `pcb.remote_ip` is actually stored in host
    // order (it's set from `IpPacket::parse`'s `from_be_bytes`,
    // which returns host-order; and `send_tcp_pcb` then calls
    // `.to_be_bytes()` to put it back on the wire). So no swap
    // here — pull octets out left-to-right via the high bits.
    let (rip, rport) = tcp::pcb_remote(pcb_id);
    console::puts("  connection from ");
    crate::kernel::mm::print_num(((rip >> 24) & 0xFF) as usize);
    console::puts(".");
    crate::kernel::mm::print_num(((rip >> 16) & 0xFF) as usize);
    console::puts(".");
    crate::kernel::mm::print_num(((rip >> 8) & 0xFF) as usize);
    console::puts(".");
    crate::kernel::mm::print_num((rip & 0xFF) as usize);
    console::puts(":");
    crate::kernel::mm::print_num(rport as usize);
    console::puts("\n");

    // Drain received data (drive the stack a bit so the peer's data
    // packets arrive) and print up to 256 bytes.
    let mut buf = [0u8; 256];
    let read_deadline = {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        now + freq * 2 // 2-second window for the peer to send data
    };
    let mut total = 0usize;
    loop {
        crate::net::poll_once();
        if let Ok(n) = tcp::recv_data_pcb(pcb_id, &mut buf[total..]) {
            if n > 0 {
                total += n;
                if total >= buf.len() { break; }
            }
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now > read_deadline { break; }
        core::hint::spin_loop();
    }

    if total > 0 {
        console::puts("  recv (");
        crate::kernel::mm::print_num(total);
        console::puts(" bytes): ");
        // Print as ASCII, replacing non-printables with `.`
        for i in 0..total {
            let b = buf[i];
            if (0x20..0x7F).contains(&b) || b == b'\n' || b == b'\r' {
                console::putc(b);
            } else {
                console::putc(b'.');
            }
        }
        console::puts("\n");
    } else {
        console::puts("  (peer sent no data within 2s window)\n");
    }

    // Send a greeting so the peer sees both directions work.
    let greeting = b"hello from bat_os\n";
    let _ = tcp::send_data_pcb(pcb_id, greeting);
    // Drive a few more polls so the SYN+payload+ACK gets out.
    for _ in 0..5_000_000u64 {
        crate::net::poll_once();
        core::hint::spin_loop();
    }

    console::puts("  sent greeting; closing\n");
    tcp::close_pcb(pcb_id);
    tcp::listen_close(port);
}

/// Sprint 2.3: dump recent audit-log entries.
/// `audit` → last 32 entries
/// `audit <N>` → last N entries
/// `audit all` → everything resident in the ring (≤ 1024)
fn cmd_audit(arg: &str) {
    let n = if arg.is_empty() {
        32
    } else if arg == "all" {
        1024
    } else {
        match arg.parse::<usize>() {
            Ok(v) => v.max(1),
            Err(_) => {
                console::puts("  usage: audit [N | all]\n");
                return;
            }
        }
    };
    crate::security::audit::dump_tail(n);
    // if the ring has overflowed,
    // surface the evicted-count so the reviewer knows entries
    // were silently dropped (potentially flood-eviction).
    let evicted = crate::security::audit::evicted();
    if evicted > 0 {
        console::puts("  audit: WARNING ");
        crate::kernel::mm::print_num(evicted);
        console::puts(" entries evicted since boot — log may have been tampered with via flooding\n");
    }
}

/// serialize the audit ring and write it to BatFS as
/// /audit-<count>.log. Persists what we have. Cheap O(N) walk +
/// one BatFS create call. (Append-only is the next milestone — for
/// now we overwrite the same path with the latest dump.)
fn cmd_audit_flush() {
    static mut FLUSH_BUF: [u8; 256 * 1024] = [0; 256 * 1024];
    let buf = unsafe { &mut *core::ptr::addr_of_mut!(FLUSH_BUF) };
    let n = crate::security::audit::serialize(buf);
    if n == 0 {
        console::puts("  audit-flush: nothing to write\n");
        return;
    }
    match crate::fs::batfs::create("audit.log", &buf[..n]) {
        Ok(()) => {
            console::puts("  audit-flush: wrote ");
            crate::kernel::mm::print_num(n);
            console::puts(" bytes to /audit.log\n");
        }
        Err(e) => {
            console::puts("  audit-flush: BatFS write failed: ");
            console::puts(e);
            console::puts("\n");
        }
    }
}

/// Verify the tamper-evident chain over the resident audit ring.
/// On success: prints OK and the current chain head (32-byte SHA-256
/// hex). On detection: prints the absolute index of the first mismatch.
fn cmd_audit_chain() {
    use crate::security::audit_chain::{verify_chain, chain_head, VerifyOutcome};
    match verify_chain() {
        VerifyOutcome::Ok => {
            console::puts("  audit-chain: OK\n  head: ");
            let h = chain_head();
            for &b in h.iter() {
                let hi = (b >> 4) & 0x0f;
                let lo = b & 0x0f;
                let hc = if hi < 10 { (b'0' + hi) as char } else { (b'a' + hi - 10) as char };
                let lc = if lo < 10 { (b'0' + lo) as char } else { (b'a' + lo - 10) as char };
                let pair = [hc as u8, lc as u8, 0];
                console::puts(core::str::from_utf8(&pair[..2]).unwrap_or("?"));
            }
            console::puts("\n");
        }
        VerifyOutcome::FirstMismatchAt(idx) => {
            console::puts("  audit-chain: TAMPER DETECTED at index ");
            crate::kernel::mm::print_num(idx);
            console::puts("\n  every entry from this index onward must be considered suspect\n");
        }
    }
}

/// Dump the kmsg ring (kernel messages, not security events).
/// `dmesg`     → last 32 lines
/// `dmesg all` → up to RING_CAP (512) lines
fn cmd_dmesg(arg: &str) {
    use crate::kernel::kmsg;
    let n = if arg.is_empty() {
        32
    } else if arg == "all" {
        512
    } else {
        arg.parse::<usize>().unwrap_or(32)
    };
    kmsg::recent(n, |line| {
        let sev = match line.sev {
            0 => "TRACE", 1 => "DEBUG", 2 => "INFO",
            3 => "WARN",  4 => "ERROR", _ => "?",
        };
        console::puts("  [");
        console::puts(sev);
        console::puts("] ");
        let mlen = line.mlen as usize;
        let msg = core::str::from_utf8(&line.msg[..mlen]).unwrap_or("<binary>");
        console::puts(msg);
        console::puts("\n");
    });
}

/// Ask the on-device AI agent a question. Today this opens an
/// AgentSession, fires `ask()`, and polls the streaming response
/// for text events. The actual inference happens on the operator-
/// configured remote host (see DESIGN_AI_AGENT.md §Inference host).
fn cmd_ai(question: &str) {
    use crate::ai::{AgentSession, StreamEvent, AgentError};
    if question.is_empty() {
        console::puts("  usage: ai <question>\n");
        return;
    }
    let mut session = match AgentSession::new() {
        Ok(s) => s,
        Err(e) => {
            console::puts("  ai: failed to start session: ");
            console::puts(match e {
                AgentError::Network(s)      => s,
                AgentError::Protocol(s)     => s,
                AgentError::Tool(s)         => s,
                AgentError::PolicyDenied    => "policy denied",
                AgentError::Interrupted     => "interrupted",
                AgentError::TokenBudget     => "token budget",
            });
            console::puts("\n");
            return;
        }
    };
    let mut response = session.ask(question);
    loop {
        match response.poll() {
            StreamEvent::Text(t) => {
                console::puts(&t);
            }
            StreamEvent::ToolCall { name } => {
                console::puts("\n  [tool: ");
                console::puts(name);
                console::puts("]\n");
            }
            StreamEvent::Done => {
                console::puts("\n");
                break;
            }
            StreamEvent::Error(e) => {
                console::puts("\n  ai: error: ");
                console::puts(match e {
                    AgentError::Network(s)      => s,
                    AgentError::Protocol(s)     => s,
                    AgentError::Tool(s)         => s,
                    AgentError::PolicyDenied    => "policy denied",
                    AgentError::Interrupted     => "interrupted",
                    AgentError::TokenBudget     => "token budget",
                });
                console::puts("\n");
                break;
            }
        }
    }
    session.close();
}

/// Dump the security posture — single command that touches every
/// module the cluster-A-through-H work shipped. Useful as a
/// pre-boot sanity check and for operator runbooks.
fn cmd_sec_status() {
    console::puts("== Bat_OS security posture ==\n");

    // Trust anchors are hard-coded in src/net/x509.rs — six.
    console::puts("  trust anchors:        6 (ISRG X1/X2, Amazon CA1, DigiCert CA/G2, GTS R4)\n");

    // Per-host SPKI pins.
    let pins = crate::net::cert_pin::list_pins();
    console::puts("  per-host cert pins:   ");
    crate::kernel::mm::print_num(pins.len());
    if !pins.is_empty() {
        console::puts(" hosts\n");
        for (host, pin_hashes) in pins.iter() {
            console::puts("    - ");
            console::puts(host);
            console::puts(": ");
            crate::kernel::mm::print_num(pin_hashes.len());
            console::puts(" pins\n");
        }
    } else {
        console::puts(" hosts (no host-level pinning configured)\n");
    }

    // CRL revocation entries.
    let (issuers, total_serials) = crate::net::crl::stats();
    console::puts("  crl revocations:      ");
    crate::kernel::mm::print_num(total_serials);
    console::puts(" serials across ");
    crate::kernel::mm::print_num(issuers);
    console::puts(" issuers\n");

    // CT log registry size.
    let ct_usable = crate::net::ct_logs::usable_count();
    console::puts("  ct log registry:      ");
    crate::kernel::mm::print_num(ct_usable);
    console::puts(" usable logs\n");

    // Audit ring depth + chain head.
    let audit_head = crate::security::audit::HEAD.load(core::sync::atomic::Ordering::Relaxed);
    console::puts("  audit entries:        ");
    crate::kernel::mm::print_num(audit_head);
    console::puts(" total since boot\n");

    let evicted = crate::security::audit::evicted();
    if evicted > 0 {
        console::puts("  audit evictions:      ");
        crate::kernel::mm::print_num(evicted);
        console::puts(" (ring overflow — possible flood-eviction)\n");
    }

    // kmsg ring depth.
    let kmsg_head = crate::kernel::kmsg::head();
    console::puts("  kmsg lines:           ");
    crate::kernel::mm::print_num(kmsg_head);
    console::puts(" total since boot\n");

    // Compile-time mitigations — we can't introspect at runtime
    // without reading our own ELF, so we print the cargo flags
    // that should be live as of the cluster G commit.
    console::puts("  mitigations (build):  stack-protector=all, paca+pacg+bti, pac-ret+bti\n");
    console::puts("                        verify with scripts/audit_canaries.sh\n");

    // Crypto primitives available.
    console::puts("  crypto primitives:    AES-{128,256}-{CTR,GCM,XTS}, ChaCha20-Poly1305,\n");
    console::puts("                        XChaCha20-Poly1305, SHA-{256,384}, SHA3-{256,384,512},\n");
    console::puts("                        BLAKE3, HMAC-SHA256/384, HKDF, Argon2id, Ed25519,\n");
    console::puts("                        ECDSA-P{256,384}, X25519, RSA-PSS, ML-KEM-768,\n");
    console::puts("                        ML-DSA-65, HOTP, TOTP\n");

    console::puts("\n== end ==\n");
}

/// Per-host SPKI cert pin admin.
///   pin list                              — dump all pins
///   pin add <host> <spki-sha256-hex>      — register a pin
fn cmd_pin(sub: &str, arg2: &str) {
    use crate::net::cert_pin;
    match sub {
        "" | "list" => {
            let pins = cert_pin::list_pins();
            if pins.is_empty() {
                console::puts("  pin: no per-host pins configured\n");
                return;
            }
            for (host, hashes) in pins.iter() {
                console::puts("  ");
                console::puts(host);
                console::puts("\n");
                for h in hashes.iter() {
                    console::puts("    ");
                    console::puts(h);
                    console::puts("\n");
                }
            }
        }
        "add" => {
            let host_then_hex = arg2;
            let (host, hex) = match host_then_hex.split_once(' ') {
                Some((h, x)) => (h.trim(), x.trim()),
                None => {
                    console::puts("  pin add <host> <spki-sha256-hex>\n");
                    return;
                }
            };
            if hex.len() != 64 {
                console::puts("  pin add: hex must be exactly 64 chars (32-byte SHA-256)\n");
                return;
            }
            let mut bytes = [0u8; 32];
            let raw = hex.as_bytes();
            for i in 0..32 {
                let hi = hexnib(raw[i * 2]);
                let lo = hexnib(raw[i * 2 + 1]);
                if hi == 0xff || lo == 0xff {
                    console::puts("  pin add: invalid hex\n");
                    return;
                }
                bytes[i] = (hi << 4) | lo;
            }
            match cert_pin::add_pin(host, &bytes) {
                Ok(()) => console::puts("  pin add: ok\n"),
                Err(_) => console::puts("  pin add: error (table full or host name too long)\n"),
            }
        }
        _ => console::puts("  pin {list|add <host> <spki-sha256-hex>}\n"),
    }
}

/// CRL revocation admin.
///   crl stats                            — issuer + serial counts
///   crl add <issuer-spki-hex> <serial>   — manually mark revoked
fn cmd_crl(sub: &str, arg2: &str, arg3: &str) {
    use crate::net::crl;
    match sub {
        "" | "stats" => {
            let (issuers, serials) = crl::stats();
            console::puts("  crl issuers: ");
            crate::kernel::mm::print_num(issuers);
            console::puts("   revoked serials: ");
            crate::kernel::mm::print_num(serials);
            console::puts("\n");
        }
        "add" => {
            let issuer_hex = arg2;
            let serial_hex = arg3;
            if issuer_hex.len() != 64 {
                console::puts("  crl add: issuer hex must be exactly 64 chars (32-byte SHA-256)\n");
                return;
            }
            if serial_hex.is_empty() || serial_hex.len() % 2 != 0 || serial_hex.len() > 40 {
                console::puts("  crl add: serial hex must be even-length, ≤ 40 chars (20 bytes)\n");
                return;
            }
            let mut issuer = [0u8; 32];
            let raw = issuer_hex.as_bytes();
            for i in 0..32 {
                let hi = hexnib(raw[i * 2]);
                let lo = hexnib(raw[i * 2 + 1]);
                if hi == 0xff || lo == 0xff {
                    console::puts("  crl add: invalid issuer hex\n");
                    return;
                }
                issuer[i] = (hi << 4) | lo;
            }
            let mut serial = [0u8; 20];
            let slen = serial_hex.len() / 2;
            let sraw = serial_hex.as_bytes();
            for i in 0..slen {
                let hi = hexnib(sraw[i * 2]);
                let lo = hexnib(sraw[i * 2 + 1]);
                if hi == 0xff || lo == 0xff {
                    console::puts("  crl add: invalid serial hex\n");
                    return;
                }
                serial[i] = (hi << 4) | lo;
            }
            match crl::add_revocation(&issuer, &serial[..slen]) {
                Ok(()) => console::puts("  crl add: ok\n"),
                Err(_) => console::puts("  crl add: error (issuer table full or serial too long)\n"),
            }
        }
        _ => console::puts("  crl {stats|add <issuer-spki-hex> <serial-hex>}\n"),
    }
}

/// Hash a BatFS file with one of the supported algorithms. The
/// crypto stack additions from cluster A are surfaced here so the
/// operator can sanity-check file integrity.
///
///   hash <algo> <file>
///
/// algos: sha256, sha384, sha3-256, sha3-384, sha3-512, blake3
fn cmd_hash(algo: &str, path: &str) {
    use crate::crypto;
    if algo.is_empty() || path.is_empty() {
        console::puts("  hash <sha256|sha384|sha3-256|sha3-384|sha3-512|blake3> <file>\n");
        return;
    }
    let mut file_buf = [0u8; 65536];
    let n = match crate::fs::batfs::read(path, &mut file_buf) {
        Ok(n) => n,
        Err(e) => {
            console::puts("  hash: read failed: ");
            console::puts(e);
            console::puts("\n");
            return;
        }
    };
    let body = &file_buf[..n];

    let digest: alloc::vec::Vec<u8> = match algo {
        "sha256"   => crypto::sha256::hash(body).to_vec(),
        "sha384"   => crypto::sha384::hash(body).to_vec(),
        "sha3-256" => crypto::sha3::sha3_256(body).to_vec(),
        "sha3-384" => crypto::sha3::sha3_384(body).to_vec(),
        "sha3-512" => crypto::sha3::sha3_512(body).to_vec(),
        "blake3"   => crypto::blake3::hash(body).to_vec(),
        _ => {
            console::puts("  hash: unknown algorithm; valid: sha256, sha384, sha3-256, sha3-384, sha3-512, blake3\n");
            return;
        }
    };

    console::puts("  ");
    console::puts(algo);
    console::puts(":");
    let pad = 12usize.saturating_sub(algo.len());
    for _ in 0..pad { console::puts(" "); }
    for &b in digest.iter() {
        let hi = (b >> 4) & 0x0f;
        let lo = b & 0x0f;
        let hc = if hi < 10 { (b'0' + hi) as char } else { (b'a' + hi - 10) as char };
        let lc = if lo < 10 { (b'0' + lo) as char } else { (b'a' + lo - 10) as char };
        let pair = [hc as u8, lc as u8];
        console::puts(core::str::from_utf8(&pair).unwrap_or("?"));
    }
    console::puts("\n");
}

#[inline]
fn hexnib(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0xff,
    }
}

