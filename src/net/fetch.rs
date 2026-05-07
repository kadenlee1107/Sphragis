// Bat_OS — HTTPS fetch helpers (chain-only strict).
//
// API surface:
//   parse_url(url) -> Option<(scheme, host, port, path)>
//   fetch_https(url, out) -> Result<usize, &'static str>
//   fetch_post_https(url, body, out) -> Result<usize, &'static str>
//
// All HTTPS goes through fetch_https / fetch_post_https. Strict
// chain validation against TRUST_STORE; hybrid PQ on; no fallback
// trust paths. See DESIGN_TLS_HARDENING.md.
//
// Uses the legacy single-PCB TCP path (`net::tcp::connect / send_data /
// recv_data / close`) which already does its own poll_once loop and
// timeout. We do NOT keep state across calls — every fetch is one
// connect / one TLS hello + GET (or POST) / one drain / close.

use super::{dns, tcp, tls};

/// STUMP #111 (audit M-body-truncate-silent): every drain path caps at
/// 256 KB. Pre-fix the cap-hit was a silent `break`, so a server (or
/// MITM) sending a > 256 KB response truncated the body invisibly —
/// the renderer rendered a half-page and the operator never knew the
/// cause. One-shot audit + UART warning the first time the cap fires.
/// We don't log every hit because the cap fires on EVERY large
/// download once it starts happening on a given page; the first
/// occurrence is what tells the reviewer "this is happening."
fn note_drain_capped(scheme: &str) {
    use core::sync::atomic::{AtomicBool, Ordering};
    static FIRST_FAIL: AtomicBool = AtomicBool::new(false);
    if !FIRST_FAIL.swap(true, Ordering::AcqRel) {
        let mut buf = [0u8; 192];
        let mut p = 0usize;
        let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
            let n = src.len().min(dst.len().saturating_sub(*p));
            dst[*p..*p + n].copy_from_slice(&src[..n]);
            *p += n;
        };
        copy(&mut buf, b"fetch ", &mut p);
        copy(&mut buf, scheme.as_bytes(), &mut p);
        copy(&mut buf, b" body capped at 256 KB - response truncated", &mut p);
        crate::security::audit::record(
            crate::security::audit::Category::Fetch,
            &buf[..p],
        );
        crate::drivers::uart::puts("[fetch] WARNING: 256 KB body cap reached - response truncated\n");
    }
}

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
    // STUMP #111 (audit H003): reject URLs with embedded userinfo
    // ("user@host" or "user:pass@host"). Pre-fix, parse_url accepted
    // `http://attacker@victim.com/` and treated `attacker@victim.com`
    // as the literal host. The Host: header sent to the server differs
    // from what the operator typed — phishing/HSTS-bypass class attack.
    // Spec-compliant clients either parse out the userinfo and use it
    // for Basic Auth, or reject. We don't support Basic Auth, so we
    // reject — the safer of the two.
    if authority.contains('@') { return None; }
    let (host, port) = match authority.find(':') {
        Some(i) => {
            let h = &authority[..i];
            let p: u16 = authority[i + 1..].parse().ok()?;
            (h, p)
        }
        None => (authority, default_port),
    };
    if host.is_empty() { return None; }
    // STUMP #111 (audit M-url-crlf-injection): reject URLs whose host
    // or path contains CR / LF / NUL / any other control byte. Rust
    // `&str` happily accepts these (they're valid UTF-8 scalar values),
    // so without this check `http://victim.com/foo\r\nEvil: header/`
    // got spliced unmodified into the outgoing GET line — the server
    // saw a request-smuggled extra header. Reject every control byte
    // (< 0x20 except none — we don't allow even tab in hostnames) and
    // 0x7f. Space inside path is also rejected (must be percent-encoded).
    for b in host.as_bytes() {
        if *b < 0x20 || *b == 0x7f || *b == b' ' { return None; }
    }
    for b in path.as_bytes() {
        if *b < 0x20 || *b == 0x7f || *b == b' ' { return None; }
    }
    Some((scheme, host, port, path))
}

fn copy_audit(dst: &mut [u8], src: &[u8]) -> usize {
    let n = src.len().min(dst.len());
    dst[..n].copy_from_slice(&src[..n]);
    n
}

fn write_dec(dst: &mut [u8], mut v: usize) -> usize {
    if v == 0 { if !dst.is_empty() { dst[0] = b'0'; return 1; } return 0; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while v > 0 && i < tmp.len() { tmp[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    let n = i.min(dst.len());
    for j in 0..n { dst[j] = tmp[i - 1 - j]; }
    n
}

pub fn fetch_post_https(
    url: &str,
    body: &[u8],
    out: &mut [u8],
) -> Result<usize, &'static str> {
    let (scheme, host, port, path) = parse_url(url).ok_or("bad URL")?;
    if scheme != "https" { return Err("fetch_post_https: not https URL"); }
    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "DNS resolution failed")?
    };
    tcp::connect(ip, port).map_err(|_| "TCP connect failed")?;
    if let Err(e) = tls::handshake(host) {
        tcp::close();
        return Err(e);
    }
    let mut req = [0u8; 2048];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"POST ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.1\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\nAccept: text/html\r\nContent-Type: application/x-www-form-urlencoded\r\n");
    let mut cookie_buf = [0u8; 1024];
    let cookie_len = super::cookies::build_header(host.as_bytes(), &mut cookie_buf);
    if cookie_len > 0 {
        pos += copy_to(&mut req, pos, b"Cookie: ");
        pos += copy_to(&mut req, pos, &cookie_buf[..cookie_len]);
        pos += copy_to(&mut req, pos, b"\r\n");
    }
    pos += copy_to(&mut req, pos, b"Content-Length: ");
    let mut clen_buf = [0u8; 16];
    let clen_len = write_usize_dec(body.len(), &mut clen_buf);
    pos += copy_to(&mut req, pos, &clen_buf[..clen_len]);
    pos += copy_to(&mut req, pos, b"\r\nConnection: close\r\n\r\n");
    if pos > req.len() {
        tls::close();
        return Err("request too large");
    }
    if let Err(e) = tls::send_app_data(&req[..pos]) {
        tls::close();
        return Err(e);
    }
    if !body.is_empty() {
        if let Err(e) = tls::send_app_data(body) {
            tls::close();
            return Err(e);
        }
    }

    const MAX_TOTAL: usize = 256 * 1024;
    static mut POST_SCRATCH: [u8; MAX_TOTAL] = [0; MAX_TOTAL];
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(POST_SCRATCH) };
    let mut total = 0usize;
    let mut iters = 0u32;
    let mut consecutive_empty = 0u32;
    loop {
        if total >= scratch.len() { note_drain_capped("https-post"); break; }
        if iters > 500 { break; }
        iters += 1;
        match tls::recv_app_data(&mut scratch[total..]) {
            Ok(0) => {
                consecutive_empty += 1;
                if consecutive_empty > 8 { break; }
            }
            Ok(n) => { total += n; consecutive_empty = 0; }
            Err(_) => break,
        }
    }
    tls::close();
    // _mode_guard's Drop restores prev_mode + prev_hybrid here.

    if total == 0 { return Err("empty response"); }
    let body_start = match find_double_crlf(&scratch[..total]) {
        Some(i) => i + 4,
        None    => return Err("no header/body boundary"),
    };
    if !scratch.starts_with(b"HTTP/1.") || scratch.len() < 12 {
        return Err("malformed status line");
    }
    // STUMP #105: capture Set-Cookie before we hand the body off.
    super::cookies::ingest_response_headers(host.as_bytes(), &scratch[..body_start.saturating_sub(4)]);
    let body_len = total - body_start;
    let copy_len = body_len.min(out.len());
    out[..copy_len].copy_from_slice(&scratch[body_start..body_start + copy_len]);
    Ok(copy_len)
}

/// Helper: drain the HTTP response from the legacy TCP PCB into the
/// caller's `out` buffer. Splits headers from body.
fn drain_http_response(out: &mut [u8]) -> Result<usize, &'static str> {
    drain_http_response_with_host("", out)
}

/// STUMP #105: same drain path, but feeds the response headers
/// through cookies::ingest_response_headers if the host is known.
fn drain_http_response_with_host(host: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    const MAX_TOTAL: usize = 256 * 1024;
    static mut SCRATCH: [u8; MAX_TOTAL] = [0; MAX_TOTAL];
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(SCRATCH) };
    let mut total = 0usize;
    loop {
        if total >= scratch.len() { note_drain_capped("http-drain"); break; }
        match tcp::recv_data(&mut scratch[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(_) => break,
        }
    }
    tcp::close();
    if total == 0 { return Err("empty response"); }
    let body_start = match find_double_crlf(&scratch[..total]) {
        Some(i) => i + 4,
        None    => return Err("no header/body boundary"),
    };
    if !host.is_empty() {
        super::cookies::ingest_response_headers(host.as_bytes(), &scratch[..body_start.saturating_sub(4)]);
    }
    let body_len = total - body_start;
    let copy_len = body_len.min(out.len());
    out[..copy_len].copy_from_slice(&scratch[body_start..body_start + copy_len]);
    Ok(copy_len)
}

fn write_usize_dec(n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut v = n;
    while v > 0 && i < buf.len() { buf[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    let len = i;
    for j in 0..len { out[j] = buf[len - 1 - j]; }
    len
}

/// HTTPS fetch (TLS 1.3 over TCP/443 by default). Writes the response
/// body into `out`, returns body length.
///
/// Uses the kernel's existing TLS singleton: tcp::connect → tls::handshake
/// → tls::send_app_data → tls::recv_app_data loop → tls::close.
///
/// Strict chain validation against TRUST_STORE; hybrid PQ on; failure
/// aborts. See DESIGN_TLS_HARDENING.md.
pub fn fetch_https(url: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    let (scheme, host, port, path) = parse_url(url).ok_or("bad URL")?;
    if scheme != "https" { return Err("fetch_https: not https URL"); }

    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "DNS resolution failed")?
    };

    tcp::connect(ip, port).map_err(|_| "TCP connect failed")?;

    if let Err(e) = tls::handshake(host) {
        tcp::close();
        return Err(e);
    }

    // Build "GET <path> HTTP/1.1\r\nHost: <host>\r\n...\r\n\r\n".
    // HTTP/1.1 (not 1.0) because some HTTPS servers reject 1.0; "Connection:
    // close" still terminates the response cleanly.
    let mut req = [0u8; 2048];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"GET ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.1\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\nAccept: text/html\r\n");
    let mut cookie_buf = [0u8; 1024];
    let cookie_len = super::cookies::build_header(host.as_bytes(), &mut cookie_buf);
    if cookie_len > 0 {
        pos += copy_to(&mut req, pos, b"Cookie: ");
        pos += copy_to(&mut req, pos, &cookie_buf[..cookie_len]);
        pos += copy_to(&mut req, pos, b"\r\n");
    }
    pos += copy_to(&mut req, pos, b"Connection: close\r\n\r\n");
    if pos > req.len() {
        tls::close();
        return Err("request too large");
    }

    if let Err(e) = tls::send_app_data(&req[..pos]) {
        tls::close();
        return Err(e);
    }

    // Drain into a scratch buffer up to MAX_TOTAL bytes, then split off
    // headers (\r\n\r\n) and copy body into `out`. Declared inside the
    // function so we don't share state between concurrent fetches (we
    // don't have any, single-threaded, but explicit > implicit).
    // STUMP #96: keep looping past Ok(0). recv_app_data returns Ok(0)
    // to mean "I just consumed a non-data record (NewSessionTicket,
    // ChangeCipherSpec, etc.) — try again for the next record." If
    // we treated Ok(0) as EOF (which we did, pre-fix), then on
    // servers like Wikipedia that send NewSessionTicket between
    // handshake and the actual response, fetch_https returned
    // "empty response" without ever pulling the body. Cap by
    // total iterations so a server replying only with empty
    // records can't spin forever.
    const MAX_TOTAL: usize = 256 * 1024;
    static mut TLS_SCRATCH: [u8; MAX_TOTAL] = [0; MAX_TOTAL];
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(TLS_SCRATCH) };
    let mut total = 0usize;
    let mut iters = 0u32;
    let mut consecutive_empty = 0u32;
    loop {
        if total >= scratch.len() { note_drain_capped("https"); break; }
        if iters > 500 { break; } // pathological-server guard
        iters += 1;
        match tls::recv_app_data(&mut scratch[total..]) {
            Ok(0) => {
                // Non-data TLS record (NewSessionTicket / CCS / etc.).
                // Loop, but bail if we get a long run of empty reads —
                // that's the "server is silent, give up" signal.
                consecutive_empty += 1;
                if consecutive_empty > 8 { break; }
            }
            Ok(n) => {
                total += n;
                consecutive_empty = 0;
            }
            Err(_) => break, // timeout / FIN / alert
        }
    }

    tls::close();
    // _mode_guard's Drop restores prev_mode + prev_hybrid on return.

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
