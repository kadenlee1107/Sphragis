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

/// STUMP #111 (audit H019): RAII guard that restores
/// `tls_pinning::Mode` + `tls::set_hybrid_enabled` on drop. Pre-fix,
/// every fetch_https / fetch_post_https Err path manually called
/// `set_mode(prev) + set_hybrid_enabled(prev)` — a panic in the
/// middle would skip the restore and leave the kernel stuck in
/// `Research` mode forever. With this guard, even a panic-unwind
/// (when we get one) restores cleanly.
struct ResearchModeGuard {
    prev_mode: super::tls_pinning::Mode,
    prev_hybrid: bool,
}

impl ResearchModeGuard {
    fn relax_for_renderer() -> Self {
        let prev_mode = super::tls_pinning::current_mode();
        let prev_hybrid = tls::hybrid_enabled();
        super::tls_pinning::set_mode(super::tls_pinning::Mode::Research);
        tls::set_hybrid_enabled(false);
        ResearchModeGuard { prev_mode, prev_hybrid }
    }
}

impl Drop for ResearchModeGuard {
    fn drop(&mut self) {
        super::tls_pinning::set_mode(self.prev_mode);
        tls::set_hybrid_enabled(self.prev_hybrid);
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
    Some((scheme, host, port, path))
}

/// Scheme-dispatched fetch. Renderer call site.
pub fn fetch_url(url: &str, out: &mut [u8]) -> Result<usize, &'static str> {
    let (scheme, _, _, _) = parse_url(url).ok_or("bad URL")?;
    let result = match scheme {
        "https" => fetch_https(url, out),
        _       => fetch_http(url, out),
    };
    // STUMP #103 — Sprint 2.3: log every URL fetch (success or fail)
    // to the audit ring. Result tag is appended so the operator can
    // see at a glance which URLs succeeded.
    let mut buf = [0u8; 192];
    let mut p = 0;
    p += copy_audit(&mut buf[p..], b"GET ");
    p += copy_audit(&mut buf[p..], url.as_bytes());
    match &result {
        Ok(n) => {
            p += copy_audit(&mut buf[p..], b" OK ");
            p += write_dec(&mut buf[p..], *n);
            p += copy_audit(&mut buf[p..], b"B");
        }
        Err(e) => {
            p += copy_audit(&mut buf[p..], b" FAIL ");
            p += copy_audit(&mut buf[p..], e.as_bytes());
        }
    }
    crate::security::audit::record(crate::security::audit::Category::Fetch, &buf[..p]);
    result
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
    // Sprint 3.1 (STUMP #105): if the cookie jar has anything for this
    // host, splice in a Cookie: header before Connection: close.
    let mut req = [0u8; 2048];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"GET ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.0\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\n");
    let mut cookie_buf = [0u8; 1024];
    let cookie_len = super::cookies::build_header(host.as_bytes(), &mut cookie_buf);
    if cookie_len > 0 {
        pos += copy_to(&mut req, pos, b"Cookie: ");
        pos += copy_to(&mut req, pos, &cookie_buf[..cookie_len]);
        pos += copy_to(&mut req, pos, b"\r\n");
    }
    pos += copy_to(&mut req, pos, b"Connection: close\r\n\r\n");
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
        if total >= scratch.len() { note_drain_capped("http"); break; }
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

    // STUMP #105: ingest Set-Cookie headers from the response. Done
    // BEFORE we copy the body so the jar is up-to-date even if the
    // body copy is short-circuited by a small `out` buffer.
    super::cookies::ingest_response_headers(host.as_bytes(), &scratch[..body_start.saturating_sub(4)]);

    let body_len = total - body_start;
    let copy_len = body_len.min(out.len());
    out[..copy_len].copy_from_slice(&scratch[body_start..body_start + copy_len]);
    Ok(copy_len)
}

/// POST a `application/x-www-form-urlencoded` body to a URL. Same
/// shape as fetch_url but with a method override and a request body.
/// `scheme` chosen by URL prefix; HTTPS goes through fetch_post_https,
/// HTTP through fetch_post_http. Used by the renderer's `<form>`
/// submit path (Sprint 1.3 — STUMP #97).
pub fn fetch_post_url(
    url: &str,
    body: &[u8],
    out: &mut [u8],
) -> Result<usize, &'static str> {
    let (scheme, _, _, _) = parse_url(url).ok_or("bad URL")?;
    let result = match scheme {
        "https" => fetch_post_https(url, body, out),
        _       => fetch_post_http(url, body, out),
    };
    // STUMP #103: log POSTs with the BODY SIZE only — never body
    // contents (could be a passphrase, credit card, etc).
    let mut buf = [0u8; 192];
    let mut p = 0;
    p += copy_audit(&mut buf[p..], b"POST ");
    p += copy_audit(&mut buf[p..], url.as_bytes());
    p += copy_audit(&mut buf[p..], b" body=");
    p += write_dec(&mut buf[p..], body.len());
    p += copy_audit(&mut buf[p..], b"B ");
    match &result {
        Ok(n) => {
            p += copy_audit(&mut buf[p..], b"OK ");
            p += write_dec(&mut buf[p..], *n);
            p += copy_audit(&mut buf[p..], b"B");
        }
        Err(e) => {
            p += copy_audit(&mut buf[p..], b"FAIL ");
            p += copy_audit(&mut buf[p..], e.as_bytes());
        }
    }
    crate::security::audit::record(crate::security::audit::Category::FormSubmit, &buf[..p]);
    result
}

pub fn fetch_post_http(
    url: &str,
    body: &[u8],
    out: &mut [u8],
) -> Result<usize, &'static str> {
    let (scheme, host, port, path) = parse_url(url).ok_or("bad URL")?;
    if scheme != "http" { return Err("fetch_post_http: not http URL"); }
    let ip = if let Some(numeric) = parse_numeric_ipv4(host) {
        numeric
    } else {
        dns::resolve(host).map_err(|_| "DNS resolution failed")?
    };
    tcp::connect(ip, port).map_err(|_| "TCP connect failed")?;
    let mut req = [0u8; 2048];
    let mut pos = 0;
    pos += copy_to(&mut req, pos, b"POST ");
    pos += copy_to(&mut req, pos, path.as_bytes());
    pos += copy_to(&mut req, pos, b" HTTP/1.0\r\nHost: ");
    pos += copy_to(&mut req, pos, host.as_bytes());
    pos += copy_to(&mut req, pos, b"\r\nUser-Agent: Bat_OS/1.0\r\nContent-Type: application/x-www-form-urlencoded\r\n");
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
    if pos + body.len() > req.len() { tcp::close(); return Err("request too large"); }
    if tcp::send_data(&req[..pos]).is_err() { tcp::close(); return Err("send headers failed"); }
    if !body.is_empty() && tcp::send_data(body).is_err() {
        tcp::close();
        return Err("send body failed");
    }
    drain_http_response_with_host(host, out)
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
    // STUMP #111 (audit H019): RAII guard handles mode + hybrid
    // restore on every exit path, including future panic-unwind.
    let _mode_guard = ResearchModeGuard::relax_for_renderer();
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

    // STUMP #94 + #111 (audit H019): relax pinning + disable PQ-hybrid
    // for this fetch via the RAII ResearchModeGuard. Our hybrid
    // key-derivation has a real-world bug against major HTTPS servers
    // when they pick the hybrid group; plain X25519 handshakes
    // cleanly. The guard's Drop restores the previous mode + hybrid
    // setting on EVERY exit path including future panic-unwind.
    let _mode_guard = ResearchModeGuard::relax_for_renderer();

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
    // headers (\r\n\r\n) and copy body into `out`. Reuse the same SCRATCH
    // size as fetch_http — declared inside this function so we don't share
    // state between concurrent fetches (we don't have any, single-threaded,
    // but explicit > implicit).
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
