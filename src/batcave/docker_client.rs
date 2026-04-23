// Bat_OS — Docker-BatCave client (Phase 2 of the design-alignment plan).
//
// ROLE
// ----
// Opens a TCP connection from Bat_OS → Mac-side `batcaved` daemon at
// 10.0.2.2:9999 (QEMU slirp's host alias). All lifecycle operations
// for Docker-backed BatCaves flow through this module so the shell
// surface in `cmd_batcave()` stays unified.
//
// See DESIGN_BATCAVES.md — this is how we satisfy:
//   * "Tools installed from Kali repos on demand" (the daemon does
//     the apt install on our behalf inside the container)
//   * "Every BatCave starts with ZERO access. Capabilities granted
//     explicitly per-cave" — we pass the cap-add list as an argument
//     on CREATE; the daemon translates to `docker run --cap-add ...`
//   * "Called 'BatCaves' — containers, sandboxes" (decision log #10)
//     — Docker containers ARE the Linux-cave sandbox primitive
//
// PROTOCOL (see scripts/batcaved.py docstring)
// -------
//   AUTH <token>                  - first line, always
//   CREATE <name> <image> <caps>  - comma-separated caps csv
//   RUN <name> <argv...>          - streams stdout; ends with EOF <rc>
//   LIST                          - lines of name\timage\tstatus, then EOF
//   DESTROY <name>                - OK / ERR <reason>
//   DESTROY_ALL                   - for security::wipe (Phase 5)
//   PING                          - keepalive → PONG
//   ARM <secs>                    - deadman armer (Phase 5)
//   QUIT
//
// FAILURE MODE
// ------------
// If the daemon is not running, `connect()` fails fast with the same
// ConnRefused-ish error the Bat_OS TCP stack would return on any
// refused destination. We surface a readable error to the shell
// rather than wedging the guest.

#![allow(dead_code)]

use crate::drivers::uart;
use crate::net::tcp;
use alloc::string::String;
use alloc::vec::Vec;

/// Mac host, seen from the QEMU guest via slirp's 10.0.2.2 alias.
const DAEMON_IP: u32 = 0x0A00_0202; // 10.0.2.2 in host byte order
const DAEMON_PORT: u16 = 9999;

/// Shared secret with the daemon. Baked at build time so the user can
/// override via `BATCAVED_TOKEN=foo cargo build`. Matches the
/// `DEFAULT_TOKEN` in scripts/batcaved.py.
///
/// SECURITY — Phase 2 cut. Production should derive this from the
/// passphrase-KDF (same path as BatFS), which couples daemon auth to
/// the auth gate. See design-alignment phase 3/5 TODOs.
const BUILD_TOKEN: Option<&str> = option_env!("BATCAVED_TOKEN");
const FALLBACK_TOKEN: &str = "BATMAN-DEV-2026";

fn token() -> &'static str {
    BUILD_TOKEN.unwrap_or(FALLBACK_TOKEN)
}

/// Establish a TCP connection to the daemon and authenticate.
/// Caller must `disconnect()` when done.
pub fn connect_and_auth() -> Result<(), &'static str> {
    tcp::connect(DAEMON_IP, DAEMON_PORT)?;

    let mut line = String::new();
    line.push_str("AUTH ");
    line.push_str(token());
    line.push('\n');
    tcp::send_data(line.as_bytes())?;

    // Read "OK authenticated\n" (or "ERR ...")
    let reply = recv_line()?;
    if !reply.starts_with("OK") {
        tcp::close();
        return Err("batcaved auth failed");
    }
    Ok(())
}

pub fn disconnect() {
    let _ = tcp::send_data(b"QUIT\n");
    // Give the daemon a moment to respond; drain anything it sends.
    let mut scratch = [0u8; 64];
    let _ = tcp::recv_data(&mut scratch);
    tcp::close();
}

/// Send a command line (adds the trailing newline).
fn send_cmd(cmd: &str) -> Result<(), &'static str> {
    tcp::send_data(cmd.as_bytes())?;
    tcp::send_data(b"\n")?;
    Ok(())
}

/// Read exactly one line from the daemon (blocking up to the TCP stack's
/// internal timeout).
fn recv_line() -> Result<String, &'static str> {
    let mut out = String::new();
    let mut byte = [0u8; 1];
    // Bound the loop so a mis-framed daemon response can't wedge us.
    for _ in 0..8192 {
        let n = tcp::recv_data(&mut byte)?;
        if n == 0 { break; }
        if byte[0] == b'\n' { return Ok(out); }
        if byte[0] != b'\r' {
            out.push(byte[0] as char);
        }
    }
    if out.is_empty() { Err("no response") } else { Ok(out) }
}

/// Read until we see a line starting with `marker`. The marker line
/// itself is returned. All preceding lines are passed to `sink`.
fn recv_until<F: FnMut(&str)>(marker: &str, mut sink: F) -> Result<String, &'static str> {
    for _ in 0..100_000 {
        let line = recv_line()?;
        if line.starts_with(marker) { return Ok(line); }
        sink(&line);
    }
    Err("marker not found")
}

// ───── High-level operations ─────────────────────────────────────

/// `batcave create --docker` equivalent. Creates a running container
/// from `image`, attaches the listed capabilities, and returns the
/// container's short ID on success.
///
/// Phase 3 overload: `create_with_key` additionally passes the cave's
/// per-cave AES-256 key (derived in Bat_OS via `sha256::derive_key`
/// on cave create). The daemon uses it to AES-encrypt the cave's
/// audit log at rest, and zeroes it on destroy. The key never touches
/// the Mac's disk in plaintext.
pub fn create(name: &str, image: &str, caps: &[&str]) -> Result<String, &'static str> {
    create_with_key(name, image, caps, None)
}

pub fn create_with_key(
    name: &str,
    image: &str,
    caps: &[&str],
    key: Option<&[u8; 32]>,
) -> Result<String, &'static str> {
    create_full(name, image, caps, key, false)
}

/// Integration #2: create a cave backed by an AES-256-encrypted APFS
/// disk image on the Mac host. The daemon uses the per-cave key as
/// the volume passphrase; destroy detaches AND deletes the .dmg.
pub fn create_persistent(
    name: &str,
    image: &str,
    caps: &[&str],
    key: &[u8; 32],
) -> Result<String, &'static str> {
    create_full(name, image, caps, Some(key), true)
}

fn create_full(
    name: &str,
    image: &str,
    caps: &[&str],
    key: Option<&[u8; 32]>,
    persistent: bool,
) -> Result<String, &'static str> {
    let mut cmd = String::from("CREATE ");
    cmd.push_str(name);
    cmd.push(' ');
    cmd.push_str(image);
    cmd.push(' ');
    let mut first = true;
    for c in caps {
        if !first { cmd.push(','); }
        cmd.push_str(c);
        first = false;
    }
    if caps.is_empty() { cmd.push_str("-"); } // daemon treats - / empty as no caps

    // Phase 3: append the per-cave key as hex (64 chars) when provided.
    // Key is sent over the LOCAL loopback-only TCP channel to the daemon;
    // the daemon's TCP listener is bound to 127.0.0.1. For stronger
    // protection against an attacker on the same Mac, Phase 3 v2 will
    // switch the daemon to a unix-socket + peercred check.
    if let Some(k) = key {
        cmd.push(' ');
        let hex = b"0123456789abcdef";
        for &b in k {
            cmd.push(hex[(b >> 4) as usize] as char);
            cmd.push(hex[(b & 0x0f) as usize] as char);
        }
    } else if persistent {
        // Persistent volumes require a key; protocol would otherwise
        // place --persistent in the key slot.
        return Err("persistent caves require a key");
    }

    // Integration #2: --persistent flag. Daemon provisions an encrypted
    // DMG via hdiutil using the key above as the passphrase.
    if persistent {
        cmd.push_str(" --persistent");
    }

    send_cmd(&cmd)?;
    let reply = recv_line()?;
    if let Some(rest) = reply.strip_prefix("OK ") {
        Ok(String::from(rest))
    } else {
        Err("create rejected")
    }
}

/// `batcave run --docker` — exec `argv` inside the cave. Streams stdout
/// to the kernel UART (which mirrors to the framebuffer console on
/// Apple). Returns the container process's exit code.
pub fn run<F: FnMut(&str)>(
    name: &str, argv: &[&str], mut sink: F,
) -> Result<i32, &'static str> {
    let mut cmd = String::from("RUN ");
    cmd.push_str(name);
    for a in argv {
        cmd.push(' ');
        // Very lightweight quoting — the daemon uses shlex on its side.
        // Wrap in double quotes if the arg contains spaces.
        let needs_quote = a.contains(' ') || a.contains('\t');
        if needs_quote { cmd.push('"'); cmd.push_str(a); cmd.push('"'); }
        else { cmd.push_str(a); }
    }
    send_cmd(&cmd)?;

    // Stream until we see `EOF <rc>`
    let final_line = recv_until("EOF", |line| sink(line))?;
    // parse exit code
    let rc_str = final_line.strip_prefix("EOF ").unwrap_or("").trim();
    Ok(rc_str.parse::<i32>().unwrap_or(-1))
}

/// `batcave destroy --docker` — stops + removes the container. Daemon
/// also collapses the shared network when the last cave is destroyed.
pub fn destroy(name: &str) -> Result<(), &'static str> {
    let mut cmd = String::from("DESTROY ");
    cmd.push_str(name);
    send_cmd(&cmd)?;
    let reply = recv_line()?;
    if reply.starts_with("OK") { Ok(()) } else { Err("destroy rejected") }
}

/// Nuke every docker-managed cave. Phase 5 calls this from the wipe
/// path so `security::wipe` destroys docker caves alongside native.
pub fn destroy_all() -> Result<usize, &'static str> {
    send_cmd("DESTROY_ALL")?;
    let reply = recv_line()?;
    // Expect "OK wiped N"
    if let Some(rest) = reply.strip_prefix("OK wiped ") {
        Ok(rest.trim().parse::<usize>().unwrap_or(0))
    } else {
        Err("destroy_all rejected")
    }
}

/// `batcave list` — returns vector of (name, image, status) for every
/// docker-managed cave the daemon knows about.
pub fn list() -> Result<Vec<(String, String, String)>, &'static str> {
    send_cmd("LIST")?;
    let mut out: Vec<(String, String, String)> = Vec::new();
    loop {
        let line = recv_line()?;
        if line == "EOF" { break; }
        // split on \t
        let mut it = line.splitn(3, '\t');
        let a = String::from(it.next().unwrap_or(""));
        let b = String::from(it.next().unwrap_or(""));
        let c = String::from(it.next().unwrap_or(""));
        if !a.is_empty() {
            out.push((a, b, c));
        }
        if out.len() > 512 { break; } // sanity
    }
    Ok(out)
}

// ───── Heartbeat / deadman (Phase 5 scaffold) ────────────────────

/// Arm the daemon's deadman timer: if Bat_OS doesn't PING within
/// `secs` seconds, the daemon wipes every cave. Called from
/// `security::deadman::arm()`.
pub fn arm_deadman(secs: u64) -> Result<(), &'static str> {
    let mut cmd = String::from("ARM ");
    // no_std: format u64 manually
    let mut buf = [0u8; 20];
    let mut n = secs;
    let mut i = buf.len();
    if n == 0 { i -= 1; buf[i] = b'0'; }
    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    cmd.push_str(core::str::from_utf8(&buf[i..]).unwrap_or("0"));
    send_cmd(&cmd)?;
    let reply = recv_line()?;
    if reply.starts_with("OK") { Ok(()) } else { Err("arm rejected") }
}

pub fn ping() -> Result<(), &'static str> {
    send_cmd("PING")?;
    let reply = recv_line()?;
    if reply == "PONG" { Ok(()) } else { Err("no pong") }
}

// ───── Convenience wrapper: open-do-close ─────────────────────────

/// Run a closure with an authenticated connection; auto-disconnect on
/// exit. Most shell handlers will use this wrapper.
pub fn with_daemon<F, R>(f: F) -> Result<R, &'static str>
where F: FnOnce() -> Result<R, &'static str>,
{
    connect_and_auth().map_err(|e| {
        uart::puts("[docker] daemon connect failed: ");
        uart::puts(e);
        uart::puts(" (is batcaved running on the Mac? `python3 scripts/batcaved.py`)\n");
        e
    })?;
    let r = f();
    disconnect();
    r
}
