// Bat_OS — minimal HTTP/1.0 + HTTPS fetch helpers for the renderer.
//
// Used by `cmd_render` to resolve `<link rel="stylesheet" href="...">`
// and remote `<img src="...">` references at render time, and to render
// from a live URL directly.
//
// API surface:
//   parse_url(url) -> Option<(scheme, host, port, path)>
//   fetch_url(url, out) -> Result<usize, &'static str>     // dispatches by scheme
//   fetch_http(url, out) -> Result<usize, &'static str>    // plain HTTP
//   fetch_https(url, out) -> Result<usize, &'static str>   // HTTPS (TLS 1.3)
//
// Uses the legacy single-PCB TCP path (`net::tcp::connect / send_data /
// recv_data / close`) which already does its own poll_once loop and
// timeout. We do NOT keep state across calls — every fetch is one
// connect / one (TLS hello +) GET / one drain / close.
//
// HTTPS note (STUMP #94): we run with `tls_pinning::is_strict()` set
// to false for the duration of a renderer fetch, so unpinned hosts
// (which is essentially everyone — PINS ships empty) connect anyway.
// This means the HTTPS pipe is encrypted but NOT authenticated; the
// renderer is opt-in best-effort, not a security boundary. Production
// caves keep strict mode on.

use super::{dns, tcp, tls};

/// Parse a URL into `(scheme, host, port, path)`.
/// `scheme` is one of "http" | "https"; default port follows.
pub fn parse_url(url: &str) -> Option<(&'static str, &str, u16, &str)> {
    let (scheme, default_port, rest) = if let Some(r) = url.strip_prefix("https://") {
        ("https", 443u16, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        ("http", 80u16, r)
    } else {
        return None;
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None    => (rest, "/"),
    };
    let (host, port) = match authority.find(':') {
        Some(i) => {
            let h = &authority[..i];
            let p: u16 = authority[i + 1..].parse().ok()?;
            (h, p)
        }
        None => (authority, default_port),
    };
    if host.is_empty() { return None; }
    Some((scheme, host, port, path))
}

/// Scheme-dispatched fetch. Renderer call site.
pub fn fetch_url(url: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    let (scheme, _, _, _) = parse_url(url).ok_or("bad URL")?;
    match scheme {
        "https" => fetch_https(url, out),
        _       => fetch_http(url, out),
    }
}

/// Best-effort `GET <path> HTTP/1.0` fetch. Writes the response BODY
/// (status + headers stripped) into `out`, returns the body length.
///
/// Caller-supplied buffer; the function will fill at most `out.len()`
/// bytes and return the actual length. Anything beyond is dropped on
/// the floor — for stylesheet/image use-cases we cap with the size of
/// `doc.css_text` / the img_pool slot.
///
/// Errors are stringly-typed so the renderer can log them without
/// pulling in a richer error type.
pub fn fetch_http(url: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    let (scheme, host, port, path) = parse_url(url).ok_or("bad URL")?;
    if scheme != "http" { return Err("fetch_http: not http URL"); }

    // 10.0.2.2 + numeric IPs: skip DNS so the QEMU-user host loopback
    // case ("http://10.0.2.2:8000/foo.css") works without any DNS server.
    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "DNS resolution failed")?
    };

    tcp::connect(ip, port).map_err(|_| "TCP connect failed")?;

    // Build "GET <path> HTTP/1.0\r\nHost: <host>\r\nConnection: close\r\n\r\n".
    // HTTP/1.0 + Connection: close means the server shuts down the TCP
    // half on body end, so our recv loop terminates naturally.
    let mut req = [0u8; 512];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"GET ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.0\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\nConnection: close\r\n\r\n");
    if pos > req.len() { tcp::close(); return Err("request too large"); }

    if tcp::send_data(&req[..pos]).is_err() {
        tcp::close();
        return Err("send failed");
    }

    // Drain into a scratch buffer up to MAX_TOTAL bytes, then split off
    // headers (\r\n\r\n) and copy body into `out`.
    const MAX_TOTAL: usize = 256 * 1024; // 256 KB ceiling per fetch
    static mut SCRATCH: [u8; MAX_TOTAL] = [0; MAX_TOTAL];
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(SCRATCH) };
    let mut total = 0usize;
    loop {
        if total >= scratch.len() { break; }
        match tcp::recv_data(&mut scratch[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(_) => break, // timeout / FIN — done
        }
    }
    tcp::close();

    if total == 0 { return Err("empty response"); }

    // Find header/body boundary.
    let body_start = match find_double_crlf(&scratch[..total]) {
        Some(i) => i + 4,
        None    => return Err("no header/body boundary"),
    };

    // Reject obvious non-2xx without parsing the full status line.
    if !scratch.starts_with(b"HTTP/1.") || scratch.len() < 12
        || scratch[9] != b'2'
    {
        return Err("non-2xx response");
    }

    let body_len = total - body_start;
    let copy_len = body_len.min(out.len());
    out[..copy_len].copy_from_slice(&scratch[body_start..body_start + copy_len]);
    Ok(copy_len)
}

/// HTTPS fetch (TLS 1.3 over TCP/443 by default). Same surface as
/// fetch_http: writes the response body into `out`, returns body len.
///
/// Uses the kernel's existing TLS singleton: tcp::connect → tls::handshake
/// → tls::send_app_data → tls::recv_app_data loop → tls::close.
///
/// SECURITY (STUMP #94): briefly flips `tls_pinning::set_strict(false)`
/// for the duration of the fetch so unpinned hostnames connect.
/// The TLS bytes are encrypted but NOT authenticated against a CA chain
/// (TRUST_STORE ships empty, PINS ships empty). Suitable for the
/// demo renderer; NOT suitable for credential exchange. Restored to
/// strict on every exit path so production code paths in the same
/// process stay safe.
pub fn fetch_https(url: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    let (scheme, host, port, path) = parse_url(url).ok_or("bad URL")?;
    if scheme != "https" { return Err("fetch_https: not https URL"); }

    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "DNS resolution failed")?
    };

    tcp::connect(ip, port).map_err(|_| "TCP connect failed")?;

    // STUMP #94: relax pinning + disable PQ-hybrid key-share for the
    // duration of this fetch. Our hybrid key-derivation has a real-
    // world bug ("handshake record auth failed") against major HTTPS
    // servers when they pick the hybrid group; plain X25519
    // handshakes cleanly. Restored to previous values on every exit.
    let prev_strict = super::tls_pinning::is_strict();
    let prev_hybrid = tls::hybrid_enabled();
    super::tls_pinning::set_strict(false);
    tls::set_hybrid_enabled(false);

    if let Err(e) = tls::handshake(host) {
        super::tls_pinning::set_strict(prev_strict);
        tls::set_hybrid_enabled(prev_hybrid);
        tcp::close();
        return Err(e);
    }

    // Build "GET <path> HTTP/1.1\r\nHost: <host>\r\nConnection: close\r\n\r\n".
    // HTTP/1.1 (not 1.0) because some HTTPS servers reject 1.0; "Connection:
    // close" still terminates the response cleanly.
    let mut req = [0u8; 1024];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"GET ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.1\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\nAccept: text/html\r\nConnection: close\r\n\r\n");
    if pos > req.len() {
        super::tls_pinning::set_strict(prev_strict);
        tls::set_hybrid_enabled(prev_hybrid);
        tls::close();
        return Err("request too large");
    }

    if let Err(e) = tls::send_app_data(&req[..pos]) {
        super::tls_pinning::set_strict(prev_strict);
        tls::set_hybrid_enabled(prev_hybrid);
        tls::close();
        return Err(e);
    }

    // Drain into a scratch buffer up to MAX_TOTAL bytes, then split off
    // headers (\r\n\r\n) and copy body into `out`. Reuse the same SCRATCH
    // size as fetch_http — declared inside this function so we don't share
    // state between concurrent fetches (we don't have any, single-threaded,
    // but explicit > implicit).
    const MAX_TOTAL: usize = 256 * 1024;
    static mut TLS_SCRATCH: [u8; MAX_TOTAL] = [0; MAX_TOTAL];
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(TLS_SCRATCH) };
    let mut total = 0usize;
    loop {
        if total >= scratch.len() { break; }
        match tls::recv_app_data(&mut scratch[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(_) => break,
        }
    }

    super::tls_pinning::set_strict(prev_strict);
    tls::set_hybrid_enabled(prev_hybrid);
    tls::close();

    if total == 0 { return Err("empty response"); }

    let body_start = match find_double_crlf(&scratch[..total]) {
        Some(i) => i + 4,
        None    => return Err("no header/body boundary"),
    };

    // Status check — accept 2xx and (best-effort) 3xx so a redirected
    // landing-page render still produces *some* bytes the user can see.
    if !scratch.starts_with(b"HTTP/1.") || scratch.len() < 12 {
        return Err("malformed status line");
    }
    let status_class = scratch[9];
    if status_class != b'2' && status_class != b'3' {
        return Err("non-2xx response");
    }

    let body_len = total - body_start;
    let copy_len = body_len.min(out.len());
    out[..copy_len].copy_from_slice(&scratch[body_start..body_start + copy_len]);
    Ok(copy_len)
}

fn copy_to(dst: &mut [u8], pos: usize, src: &[u8]) -> usize {
    let n = src.len().min(dst.len().saturating_sub(pos));
    dst[pos..pos + n].copy_from_slice(&src[..n]);
    n
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    let needle = b"\r\n\r\n";
    if buf.len() < needle.len() { return None; }
    for i in 0..=buf.len() - needle.len() {
        if &buf[i..i + 4] == needle { return Some(i); }
    }
    None
}

/// Parse "a.b.c.d" → big-endian u32. Returns None if not 4 numeric octets.
fn parse_numeric_ipv4(s: &str) -> Option<u32> {
    let mut octets = [0u32; 4];
    let mut idx = 0;
    let mut cur: u32 = 0;
    let mut have_digit = false;
    for &b in s.as_bytes() {
        if b == b'.' {
            if !have_digit || idx >= 3 || cur > 255 { return None; }
            octets[idx] = cur;
            idx += 1;
            cur = 0;
            have_digit = false;
        } else if b.is_ascii_digit() {
            cur = cur * 10 + (b - b'0') as u32;
            if cur > 255 { return None; }
            have_digit = true;
        } else {
            return None;
        }
    }
    if !have_digit || idx != 3 { return None; }
    octets[3] = cur;
    Some((octets[0] << 24) | (octets[1] << 16) | (octets[2] << 8) | octets[3])
}
