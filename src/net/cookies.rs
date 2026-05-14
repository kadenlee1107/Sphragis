// Sphragis — HTTP cookie jar.
//
// Sprint 3.1 / . The renderer can't keep state across
// fetches without cookies — every login session, CSRF token, A/B test
// bucket, etc. lives in a cookie. Without this jar, every page acts
// like a brand-new visitor and any "stay signed in" flow breaks.
//
// Scope (intentionally small for the first pass):
// * Parse Set-Cookie response headers, store (host, name, value).
// * Send `Cookie: name1=v1; name2=v2` request header on subsequent
// fetches to the same host.
// * In-memory only; cleared on cave switch.
// * Ignored attributes: Domain, Path, Expires, Max-Age, Secure,
// HttpOnly, SameSite. Treat every cookie as host-only, session-
// scoped, all-paths. Compatibility with the most-common workflow
// (sign-in returns a session cookie, subsequent requests include
// it) is what matters.
//
// Privacy / security: cookies are sensitive. Each Set-Cookie /
// Cookie operation is audited (Category::Fetch, with the cookie NAME
// only — never values, which can be auth tokens). The jar is wiped
// on `reset_for_cave_switch` so a logged-out cave doesn't leak the
// previous tenant's session.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::drivers::uart;

/// the saturation alarm fires
/// once per boot. On a cave-switch the jar is wiped, so the *new*
/// cave's first flood event would be silent. We expose this flag at
/// module scope and re-arm it from `reset_for_cave_switch`.
static JAR_FULL_FIRST_FAIL: AtomicBool = AtomicBool::new(false);

const MAX_COOKIES: usize = 128;
const HOST_LEN: usize = 96;
const NAME_LEN: usize = 64;
const VALUE_LEN: usize = 256;

#[derive(Clone, Copy)]
pub struct Cookie {
    pub host: [u8; HOST_LEN],
    pub host_len: u16,
    pub name: [u8; NAME_LEN],
    pub name_len: u16,
    pub value: [u8; VALUE_LEN],
    pub value_len: u16,
    pub active: bool,
}

impl Cookie {
    const fn empty() -> Self {
        Cookie {
            host: [0; HOST_LEN], host_len: 0,
            name: [0; NAME_LEN], name_len: 0,
            value: [0; VALUE_LEN], value_len: 0,
            active: false,
        }
    }
    fn host_str(&self) -> &[u8] { &self.host[..self.host_len as usize] }
    fn name_str(&self) -> &[u8] { &self.name[..self.name_len as usize] }
    fn value_str(&self) -> &[u8] { &self.value[..self.value_len as usize] }
}

static mut JAR: [Cookie; MAX_COOKIES] = [Cookie::empty(); MAX_COOKIES];
static SLOTS_USED: AtomicUsize = AtomicUsize::new(0);

/// ASCII-lowercase host bytes into a stack
/// buffer so callers compare hostnames case-insensitively. RFC 3986:
/// hostnames are case-insensitive. Pre-fix, a cookie set on
/// `EXAMPLE.com` was never sent to `example.com`.
fn host_lower(host: &[u8], out: &mut [u8; HOST_LEN]) -> usize {
    let n = host.len().min(HOST_LEN);
    for i in 0..n { out[i] = host[i].to_ascii_lowercase(); }
    n
}

/// Look up an existing slot for (host, name) or allocate a new one.
/// Returns None if the jar is full and we'd need to evict — the
/// caller can then either drop the cookie or invoke
/// `evict_oldest_for_host` (not implemented yet — for now we drop).
fn find_or_alloc(host: &[u8], name: &[u8]) -> Option<usize> {
    unsafe {
        let jar = &mut *core::ptr::addr_of_mut!(JAR);
        for (i, c) in jar.iter().enumerate() {
            if c.active && c.host_str() == host && c.name_str() == name {
                return Some(i);
            }
        }
        for (i, c) in jar.iter().enumerate() {
            if !c.active { return Some(i); }
        }
        None
    }
}

/// Strip bytes that would break HTTP header semantics. without this, a `Set-Cookie: a=v\r\nCookie: stolen=`
/// response value is spliced raw into the next outgoing `Cookie:` header
/// header-injection. Reject CR / LF / NUL / control chars in cookie
/// names and values; truncate at the first such byte.
fn sanitize_in_place(bytes: &[u8]) -> usize {
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\r' || b == b'\n' || b == 0
            || b == b';' /* breaks Cookie: header field-value separator */
            || b < 0x20 /* any other control char */
        {
            return i;
        }
    }
    bytes.len()
}

/// Store a (host, name, value) triple in the jar. Overwrites any
/// existing cookie with the same (host, name). Drops silently if
/// host/name/value exceed their fixed buffers.
// /
/// name/value/host are sanitized — bytes from the first
/// CR/LF/NUL/control char (or `;` in name/value) onward are dropped
/// before storing. A cookie with `value="x\r\nCookie: stolen="` from a
/// hostile Set-Cookie header is stored as `value="x"` only.
pub fn set(host: &[u8], name: &[u8], value: &[u8]) {
    let host_n = sanitize_in_place(host);
    let name_n = sanitize_in_place(name);
    let value_n = sanitize_in_place(value);
    let mut host_lc = [0u8; HOST_LEN];
    let host_lc_len = host_lower(&host[..host_n], &mut host_lc);
    let host = &host_lc[..host_lc_len];
    let name = &name[..name_n];
    let value = &value[..value_n];
    if host.len() > HOST_LEN || name.is_empty() || name.len() > NAME_LEN {
        return;
    }
    let idx = match find_or_alloc(host, name) {
        Some(i) => i,
        None => {
            // silent drop on a
            // saturated jar lets a hostile origin flood us with 128
            // junk cookies so a real auth cookie from a later request
            // is dropped invisibly. One-shot audit + UART warning so
            // the operator (or post-incident reviewer) sees the
            // saturation event in the timeline. Subsequent drops stay
            // silent — we don't want a chatty per-request log to push
            // older audit entries out of the ring. Re-armed on
            // cave-switch reset so the next tenant gets its own warning.
            if !JAR_FULL_FIRST_FAIL.swap(true, Ordering::AcqRel) {
                crate::security::audit::record(
                    crate::security::audit::Category::Fetch,
                    b"cookie jar FULL (MAX_COOKIES=128) - dropping new cookies",
                );
                uart::puts("[cookies] WARNING: jar full - cookie dropped\n");
            }
            return;
        }
    };
    unsafe {
        let jar = &mut *core::ptr::addr_of_mut!(JAR);
        let c = &mut jar[idx];
        let was_new = !c.active;
        c.host[..host.len()].copy_from_slice(host);
        c.host_len = host.len() as u16;
        c.name[..name.len()].copy_from_slice(name);
        c.name_len = name.len() as u16;
        let vlen = value.len().min(VALUE_LEN);
        c.value[..vlen].copy_from_slice(&value[..vlen]);
        c.value_len = vlen as u16;
        c.active = true;
        if was_new {
            SLOTS_USED.fetch_add(1, Ordering::Relaxed);
        }
    }
    // Audit — name only, never the value (which may be a session token).
    let mut buf = [0u8; 192];
    let mut p = 0;
    let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
        let n = src.len().min(dst.len().saturating_sub(*p));
        dst[*p..*p + n].copy_from_slice(&src[..n]);
        *p += n;
    };
    copy(&mut buf, b"cookie set ", &mut p);
    copy(&mut buf, host, &mut p);
    copy(&mut buf, b" / ", &mut p);
    copy(&mut buf, name, &mut p);
    crate::security::audit::record(
        crate::security::audit::Category::Fetch,
        &buf[..p],
    );
}

/// Build a Cookie request-header value for the given host. Writes
/// `name1=v1; name2=v2; ...` into `out`, returns the byte count. If
/// no cookies exist for the host, returns 0 and the caller should
/// skip emitting the header.
// /
/// defense-in-depth — re-sanitize name/value as we
/// emit. If a stale cookie somehow contains CR/LF/NUL (it shouldn't,
/// since `set` strips them), we still refuse to emit those bytes.
pub fn build_header(host: &[u8], out: &mut [u8]) -> usize {
    let mut pos = 0usize;
    let mut host_lc = [0u8; HOST_LEN];
    let host_lc_len = host_lower(host, &mut host_lc);
    let host = &host_lc[..host_lc_len];
    unsafe {
        let jar = &*core::ptr::addr_of!(JAR);
        for c in jar.iter() {
            if !c.active { continue; }
            if c.host_str() != host { continue; }
            let name_full = c.name_str();
            let value_full = c.value_str();
            // Re-validate every emit (cheap and absolute).
            let name_n = sanitize_in_place(name_full);
            let value_n = sanitize_in_place(value_full);
            if name_n != name_full.len() || value_n != value_full.len() {
                continue; // skip any cookie that wouldn't survive sanitization
            }
            let name = &name_full[..name_n];
            let value = &value_full[..value_n];
            if pos > 0 {
                if pos + 2 > out.len() { break; }
                out[pos] = b';'; out[pos + 1] = b' '; pos += 2;
            }
            if pos + name.len() + 1 + value.len() > out.len() { break; }
            out[pos..pos + name.len()].copy_from_slice(name);
            pos += name.len();
            out[pos] = b'='; pos += 1;
            out[pos..pos + value.len()].copy_from_slice(value);
            pos += value.len();
        }
    }
    pos
}

/// Parse a single `Set-Cookie:` header value (the bytes AFTER
/// "Set-Cookie:"). Extracts the first name=value pair, ignores
/// the rest of the directives. `host` is the request's host —
/// the cookie is bound to it. Returns true if a cookie was stored.
pub fn parse_set_cookie(host: &[u8], header_value: &[u8]) -> bool {
    // Trim leading whitespace
    let mut i = 0;
    while i < header_value.len() && (header_value[i] == b' ' || header_value[i] == b'\t') { i += 1; }
    // Find '=' and ';' or end-of-line
    let mut eq = None;
    let mut semi = header_value.len();
    let mut j = i;
    while j < header_value.len() {
        let b = header_value[j];
        if b == b'=' && eq.is_none() { eq = Some(j); }
        else if b == b';' || b == b'\r' || b == b'\n' { semi = j; break; }
        j += 1;
    }
    let eq = match eq { Some(e) => e, None => return false };
    let name = &header_value[i..eq];
    let value = &header_value[eq + 1..semi];
    if name.is_empty() { return false; }
    set(host, name, value);
    true
}

/// Walk a full HTTP response header block (everything before the
/// blank line) and parse every `Set-Cookie:` line found. Header line
/// matching is case-insensitive. Stops at \r\n\r\n or end-of-input.
pub fn ingest_response_headers(host: &[u8], headers: &[u8]) {
    let mut start = 0usize;
    while start < headers.len() {
        let mut end = start;
        while end + 1 < headers.len() && !(headers[end] == b'\r' && headers[end + 1] == b'\n') {
            end += 1;
        }
        let line = &headers[start..end];
        if line.is_empty() { return; } // blank line = end of headers
        // Case-insensitive prefix match on "Set-Cookie:"
        const KEY: &[u8] = b"Set-Cookie:";
        if line.len() > KEY.len() {
            let mut matches = true;
            for k in 0..KEY.len() {
                let a = line[k] | 0x20;
                let b = KEY[k] | 0x20;
                if a != b { matches = false; break; }
            }
            if matches {
                let val = &line[KEY.len()..];
                parse_set_cookie(host, val);
            }
        }
        start = end + 2;
        if end + 1 >= headers.len() { return; }
    }
}

/// Total active cookies in the jar.
pub fn count() -> usize { SLOTS_USED.load(Ordering::Relaxed) }

/// Print the jar to UART. Names + hosts only; values are redacted
/// (length shown). Used by the `cookies` shell command.
pub fn dump() {
    unsafe {
        let jar = &*core::ptr::addr_of!(JAR);
        let mut shown = 0;
        for c in jar.iter() {
            if !c.active { continue; }
            uart::puts("  ");
            uart::puts(core::str::from_utf8_unchecked(c.host_str()));
            uart::puts(" / ");
            uart::puts(core::str::from_utf8_unchecked(c.name_str()));
            uart::puts(" = (");
            crate::kernel::mm::print_num(c.value_len as usize);
            uart::puts(" bytes)\n");
            shown += 1;
        }
        if shown == 0 {
            uart::puts("  (jar is empty)\n");
        }
    }
}

/// Wipe everything. Per-cave-switch hook + manual `cookies-clear`.
pub fn reset() {
    unsafe {
        let jar = &mut *core::ptr::addr_of_mut!(JAR);
        for c in jar.iter_mut() { *c = Cookie::empty(); }
    }
    SLOTS_USED.store(0, Ordering::Relaxed);
}

pub fn reset_for_cave_switch() {
    reset();
    // re-arm the saturation alarm so the next cave's
    // first jar-full event is audible (otherwise the second cave
    // floods silently).
    JAR_FULL_FIRST_FAIL.store(false, Ordering::Release);
}
