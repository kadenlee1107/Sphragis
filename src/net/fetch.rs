// Sphragis — URL parsing helper.
//
// Used by `security::origin::check_subresource` to extract a host
// from a URL string for cross-origin policy checks.
//
// History: this file used to host the entire HTTPS-fetch dance
// (`fetch_https`, `fetch_post_https`, response drain, etc.) for the
// in-tree browser. After the no-browser pivot the only thing left
// referencing it was `parse_url`, so the rest got deleted. If a cave
// later wants HTTPS without re-implementing the dance, the old
// shape is still in git history (commit 7dac3655 and earlier).

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
    // reject URLs with embedded userinfo
    // ("user@host" or "user:pass@host"). Pre-fix, parse_url accepted
    // `http://attacker@victim.com/` and treated `attacker@victim.com`
    // as the literal host. The Host: header sent to the server differs
    // from what the operator typed — phishing/HSTS-bypass class attack.
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
    // reject URLs whose host
    // or path contains CR / LF / NUL / any other control byte.
    for b in host.as_bytes() {
        if *b < 0x20 || *b == 0x7f || *b == b' ' { return None; }
    }
    for b in path.as_bytes() {
        if *b < 0x20 || *b == 0x7f || *b == b' ' { return None; }
    }
    Some((scheme, host, port, path))
}
