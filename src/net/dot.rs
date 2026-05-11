//! DNS-over-TLS (RFC 7858).
//!
//! DoT is the lower-overhead sibling of DoH (`src/net/dns.rs`'s DoH
//! path): same encryption guarantee, no HTTP layer.
//!
//! Wire format: each DNS message is prefixed with a 2-byte big-endian
//! length. One TLS connection can carry multiple queries serialized
//! end-to-end. We do single-query connections for simplicity — they
//! match our threat model (TXID randomization, port randomization,
//! per-cave reset). Connection reuse can come later.
//!
//! Default upstream: `1.1.1.1:853` (Cloudflare 1.1.1.1 for Families
//! also offers DoT at the same address). The TLS cert is the same
//! cert the existing DoH code validates against (Cloudflare's
//! cert SAN list includes the bare IP) so no trust-store change.
//!
//! Why we want DoT alongside DoH:
//!
//! - DoT is a single round-trip after TLS handshake; DoH is two
//!   (HTTP request, HTTP response).
//! - DoT traffic is easier to identify on the wire (port 853) than
//!   DoH (port 443, indistinguishable from regular HTTPS). For
//!   operator visibility this is a feature — censors that block DoT
//!   are loud about it; DoH that's blocked tends to fail silently.
//! - Some enterprise networks force DNS through a labelled proxy
//!   that speaks DoT but not DoH.
//!
//! We expose `query_a()` and `query_aaaa()`. The DNS message parser
//! in `dns.rs` already handles A/AAAA responses; we don't duplicate it.

#![allow(dead_code)]

// Use a small CPU-counter-derived TXID; we have a deeper RNG for
// crypto-grade randomness but TXID just needs to defeat blind
// spoofing.
#[inline]
fn rand_txid() -> u16 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {0}, cntpct_el0", out(reg) v); }
    ((v ^ (v >> 32) ^ (v >> 17)) as u16) | 0x8000
}

/// Cloudflare DoT. Same provider as our DoH so a single trust
/// anchor (ISRG X1, via the live cert) covers both transports.
pub const DOT_SERVER: u32 = 0x01010101; // 1.1.1.1
pub const DOT_PORT:   u16 = 853;

const MAX_QUERY:    usize = 512;
const MAX_RESPONSE: usize = 2048;
const DNS_TYPE_A:    u16 = 1;
const DNS_TYPE_AAAA: u16 = 28;
const DNS_CLASS_IN:  u16 = 1;

#[derive(Debug)]
pub enum DotError {
    HostnameTooLong,
    Connect(&'static str),
    Tls(&'static str),
    Send(&'static str),
    Recv(&'static str),
    Truncated,
    NoAnswer,
    Refused,
}

/// Resolve an A record (IPv4) via DoT. Returns the first A record's
/// 32-bit network-order address.
pub fn query_a(hostname: &str) -> Result<u32, DotError> {
    let mut resp = [0u8; MAX_RESPONSE];
    let n = exchange(hostname, DNS_TYPE_A, &mut resp)?;
    parse_first_a(&resp[..n])
}

/// Run one query/response exchange. Returns the number of bytes
/// of DNS response body (after stripping the 2-byte length prefix).
fn exchange(hostname: &str, qtype: u16, resp: &mut [u8]) -> Result<usize, DotError> {
    if hostname.len() > 253 {
        return Err(DotError::HostnameTooLong);
    }

    // Build the DNS query.
    let mut query = [0u8; MAX_QUERY];
    let qlen = build_query(hostname, qtype, &mut query);

    // 2-byte length prefix per RFC 7858 §3.3.
    let mut framed = [0u8; MAX_QUERY + 2];
    let length = qlen as u16;
    framed[0..2].copy_from_slice(&length.to_be_bytes());
    framed[2..2 + qlen].copy_from_slice(&query[..qlen]);

    // TCP -> TLS -> send -> recv -> close.
    super::tcp::connect(DOT_SERVER, DOT_PORT)
        .map_err(|_| DotError::Connect("tcp connect"))?;

    if let Err(e) = super::tls::handshake("1.1.1.1") {
        super::tcp::close();
        return Err(DotError::Tls(e));
    }

    if super::tls::send_app_data(&framed[..2 + qlen]).is_err() {
        super::tls::close();
        super::tcp::close();
        return Err(DotError::Send("tls send"));
    }

    let mut raw = [0u8; MAX_RESPONSE + 2];
    let recv = super::tls::recv_app_data(&mut raw);
    super::tls::close();
    super::tcp::close();

    let n = recv.map_err(|e| DotError::Recv(e))?;
    if n < 2 {
        return Err(DotError::Truncated);
    }
    let body_len = u16::from_be_bytes([raw[0], raw[1]]) as usize;
    if body_len + 2 > n {
        return Err(DotError::Truncated);
    }
    if body_len > resp.len() {
        return Err(DotError::Truncated);
    }
    resp[..body_len].copy_from_slice(&raw[2..2 + body_len]);
    Ok(body_len)
}

/// Construct a standard recursion-desired DNS query. Returns its length.
fn build_query(hostname: &str, qtype: u16, out: &mut [u8]) -> usize {
    let txid = rand_txid();
    out[0..2].copy_from_slice(&txid.to_be_bytes());
    out[2..4].copy_from_slice(&0x0100u16.to_be_bytes()); // RD=1
    out[4..6].copy_from_slice(&1u16.to_be_bytes());      // QDCOUNT=1
    out[6..8].copy_from_slice(&0u16.to_be_bytes());      // ANCOUNT
    out[8..10].copy_from_slice(&0u16.to_be_bytes());     // NSCOUNT
    out[10..12].copy_from_slice(&0u16.to_be_bytes());    // ARCOUNT
    let mut p = 12usize;
    for part in hostname.as_bytes().split(|&b| b == b'.') {
        out[p] = part.len() as u8;
        p += 1;
        out[p..p + part.len()].copy_from_slice(part);
        p += part.len();
    }
    out[p] = 0; p += 1;
    out[p..p + 2].copy_from_slice(&qtype.to_be_bytes()); p += 2;
    out[p..p + 2].copy_from_slice(&DNS_CLASS_IN.to_be_bytes()); p += 2;
    p
}

/// Walk the answer section, return the first A record's 4-byte IP
/// packed big-endian.
fn parse_first_a(body: &[u8]) -> Result<u32, DotError> {
    if body.len() < 12 {
        return Err(DotError::Truncated);
    }
    let flags = u16::from_be_bytes([body[2], body[3]]);
    let rcode = flags & 0x000f;
    if rcode != 0 {
        return Err(DotError::Refused);
    }
    let ancount = u16::from_be_bytes([body[6], body[7]]) as usize;
    if ancount == 0 {
        return Err(DotError::NoAnswer);
    }
    // Skip the question section.
    let mut p = 12usize;
    while p < body.len() && body[p] != 0 {
        let len = body[p] as usize;
        // Compression pointer in question — non-standard but be defensive.
        if (body[p] & 0xc0) == 0xc0 {
            p += 2; break;
        }
        p += 1 + len;
    }
    if p < body.len() && body[p] == 0 { p += 1; }
    p += 4; // qtype + qclass
    // Now we're at the answer section. Walk records.
    let mut remaining = ancount;
    while remaining > 0 && p < body.len() {
        // Name (skip — may be a compression pointer).
        if (body[p] & 0xc0) == 0xc0 {
            p += 2;
        } else {
            while p < body.len() && body[p] != 0 {
                let len = body[p] as usize;
                p += 1 + len;
            }
            if p < body.len() && body[p] == 0 { p += 1; }
        }
        if p + 10 > body.len() {
            return Err(DotError::Truncated);
        }
        let rtype = u16::from_be_bytes([body[p], body[p + 1]]);
        let rdlength = u16::from_be_bytes([body[p + 8], body[p + 9]]) as usize;
        p += 10;
        if rtype == DNS_TYPE_A && rdlength == 4 && p + 4 <= body.len() {
            let ip = u32::from_be_bytes([body[p], body[p + 1], body[p + 2], body[p + 3]]);
            return Ok(ip);
        }
        p += rdlength;
        remaining -= 1;
    }
    Err(DotError::NoAnswer)
}
