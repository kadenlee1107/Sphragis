// Bat_OS — minimal HTTP/1.0 fetch helper for the renderer.
//
// Used by `cmd_render` to resolve `<link rel="stylesheet" href="...">`
// and remote `<img src="http://...">` references at render time. Only
// HTTP (port 80) is wired right now — the TLS stack works but pinning
// rules + hostname-cert validation make HTTPS demo-fragile, so we
// punt on `https://` until we have a curated allowlist.
//
// API surface is intentionally tiny:
//   parse_url(url) -> Option<(host, port, path)>
//   fetch_http(url, out) -> Result<usize, &'static str>  // body bytes
//
// Uses the legacy single-PCB TCP path (`net::tcp::connect / send_data /
// recv_data / close`) which already does its own poll_once loop and
// timeout. We do NOT keep state across calls — every fetch is one
// connect / one GET / one drain / close.

use super::{dns, tcp};

/// Parse `http://host[:port][/path]`. Returns `(host, port, path)`.
/// `https://` is rejected today (TLS demo-fragile, see file header).
pub fn parse_url(url: &str) -> Option<(&str, u16, &str)> {
    let rest = url.strip_prefix("http://")?;
    // scheme stripped; rest is `host[:port][/path]`
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
        None => (authority, 80u16),
    };
    if host.is_empty() { return None; }
    Some((host, port, path))
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
    let (host, port, path) = parse_url(url).ok_or("bad URL")?;

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
