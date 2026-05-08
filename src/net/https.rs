//! Bat_OS — kernel-mediated HTTPS for caves.
//!
//! See DESIGN_HTTPS_SYSCALL.md.
//!
//! HTTPS is a Bat_OS-specific syscall (not a Linux ABI emulation): caves
//! call `bat_https_open(host, port)` and get back a TLS-backed fd. Plain
//! `read(fd, ...)` / `write(fd, ...)` then move plaintext through a
//! kernel-run TLS session — caves never see network bytes in plaintext
//! and never ship their own TLS library.
//!
//! This module owns the kernel-side primitive that:
//!   1. Allocates a fresh TCP PCB + TLS slot (1:1 paired)
//!   2. Resolves the host via the kernel resolver
//!   3. Drives `tcp::connect_blocking_pcb` then `tls::handshake_pcb`
//!   4. Returns the slot id (== TCP PCB id == TLS PCB id), or an error
//!
//! On error, both slots are released. On success, the caller owns the
//! slot and is responsible for closing it (`https::close_pcb`) when
//! done — usually via `close(fd)` from a cave, which `sys_close` routes
//! through here.

use crate::net::{tcp, tls, dns};

/// One-shot HTTPS handshake. Returns the slot id of the established
/// session, or an error string. Does NOT consult `cave_policy` — the
/// caller (syscall layer) is expected to gate first.
///
/// On any failure, the TCP PCB and TLS slot are released before
/// returning, so the caller never has to clean up after a failed open.
pub fn open_kernel(host: &str, port: u16) -> Result<usize, &'static str> {
    if host.is_empty() { return Err("https: empty host"); }
    if port == 0 { return Err("https: zero port"); }

    // Step 1: DNS resolve. Numeric IPv4 literals bypass DNS.
    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "https: dns resolve failed")?
    };

    // Step 2: Allocate a fresh TCP PCB. The matching TLS slot has the
    // same id (TLS_MAX_PCBS == tcp::MAX_PCBS by construction).
    let pcb = tcp::pcb_alloc().ok_or("https: no free TCP PCB")?;

    // Step 3: TCP connect. Free the PCB on failure so we don't leak.
    if let Err(e) = tcp::connect_blocking_pcb(pcb, ip, port) {
        tcp::close_pcb(pcb);
        return Err(e);
    }

    // Step 4: TLS handshake. tls::handshake_pcb already wipes secrets on
    // failure, but we still need to close the TCP PCB.
    if let Err(e) = tls::handshake_pcb(pcb, host) {
        tcp::close_pcb(pcb);
        return Err(e);
    }

    Ok(pcb)
}

/// Send `data` over an established HTTPS session. Caller writes raw HTTP
/// bytes (request line, headers, body); the kernel encrypts each TLS
/// record and ships it through TCP.
pub fn write(pcb: usize, data: &[u8]) -> Result<(), &'static str> {
    tls::send_app_data_pcb(pcb, data)
}

/// Receive up to `buf.len()` plaintext bytes from an established HTTPS
/// session. Returns the count actually written into `buf` (may be < len).
pub fn read(pcb: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    tls::recv_app_data_pcb(pcb, buf)
}

/// Tear down the HTTPS session. Closes both the TLS slot (zeroing all
/// derived secrets) and the underlying TCP PCB.
pub fn close_pcb(pcb: usize) {
    tls::close_pcb(pcb);
    tcp::close_pcb(pcb);
}

/// Parse "10.0.2.15"-style IPv4 literals. Returns None for non-literal
/// hostnames so the caller falls back to DNS.
fn parse_numeric_ipv4(s: &str) -> Option<u32> {
    let mut parts = s.split('.');
    let a: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    let c: u8 = parts.next()?.parse().ok()?;
    let d: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() { return None; }
    Some(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | d as u32)
}
