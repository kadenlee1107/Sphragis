//! Bat_OS package manager.
//!
//! Gap-audit item 033. Installs signed BPKG bundles into BatFS,
//! tracks what's installed, lets the operator remove packages.
//!
//! Trust model: every bundle is signed with the release-engineer
//! Ed25519 key whose public half is baked into the kernel at build
//! time (`BAT_OS_RELEASE_PUBKEY`). An unsigned-bundle install path
//! does not exist — there is no `--allow-untrusted` flag. The
//! kernel refuses to run without a baked pubkey (same posture as
//! `release-verify`).
//!
//! BPKG v1 binary layout (little-endian):
//!
//!     [0..4]    magic         "BPKG"
//!     [4]       version       0x01
//!     [5..7]    name_len      u16
//!     [7..]     name          UTF-8
//!     [...]     version_len   u16
//!               version       UTF-8
//!               file_count    u16
//!     per file:
//!               path_len      u16
//!               path          UTF-8
//!               size          u32
//!               sha256        32 bytes
//!               content       <size> bytes
//!     [tail-64..tail]  Ed25519 signature over all preceding bytes
//!
//! Install state is persisted as a single BatFS file
//! `installed_packages` — newline-separated `<name>\t<version>\t<paths...>`
//! tuples. Small enough to fit in one BatFS read.
//!
//! Limitations of this first cut:
//!   * No dependency resolution. Each package is independent.
//!   * No update path (`pkg install foo` over an existing foo
//!     refuses; remove first).
//!   * All files install into BatFS root (single namespace). Paths
//!     with directory separators would need BatFS hierarchy first.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::crypto::sig;
use crate::fs::batfs;

pub const MAGIC: &[u8] = b"BPKG";
pub const FORMAT_VERSION: u8 = 1;
pub const SIG_LEN: usize = 64;
pub const SHA256_LEN: usize = 32;
pub const MAX_BUNDLE: usize = 1024 * 1024;
pub const INSTALLED_DB: &str = "installed_packages";

#[derive(Debug)]
pub enum PkgError {
    BadMagic,
    BadVersion,
    BadFormat,
    Truncated,
    SizeOverflow,
    NoPubkey,
    SigVerifyFailed,
    Sha256Mismatch,
    AlreadyInstalled,
    NotInstalled,
    BatFsError(&'static str),
}

impl PkgError {
    pub fn as_str(&self) -> &'static str {
        match self {
            PkgError::BadMagic         => "bad magic (not a BPKG bundle)",
            PkgError::BadVersion       => "unsupported BPKG version",
            PkgError::BadFormat        => "bundle structure invalid",
            PkgError::Truncated        => "bundle truncated",
            PkgError::SizeOverflow     => "bundle exceeds size limit",
            PkgError::NoPubkey         => "no BAT_OS_RELEASE_PUBKEY baked at build time",
            PkgError::SigVerifyFailed  => "signature does not verify",
            PkgError::Sha256Mismatch   => "per-file sha256 mismatch (tampered payload)",
            PkgError::AlreadyInstalled => "package already installed (remove first)",
            PkgError::NotInstalled     => "package not installed",
            PkgError::BatFsError(e)    => e,
        }
    }
}

/// One file inside a bundle.
pub struct PkgFile<'a> {
    pub path: &'a str,
    pub sha256: [u8; SHA256_LEN],
    pub content: &'a [u8],
}

/// Parsed bundle. Borrows from the input buffer.
pub struct PkgBundle<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub files: Vec<PkgFile<'a>>,
}

/// Reader cursor over a bundle byte slice.
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self { Self { buf, pos: 0 } }

    fn take(&mut self, n: usize) -> Result<&'a [u8], PkgError> {
        if self.pos + n > self.buf.len() {
            return Err(PkgError::Truncated);
        }
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8, PkgError> {
        Ok(self.take(1)?[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, PkgError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn read_u32_le(&mut self) -> Result<u32, PkgError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_str(&mut self) -> Result<&'a str, PkgError> {
        let n = self.read_u16_le()? as usize;
        let raw = self.take(n)?;
        core::str::from_utf8(raw).map_err(|_| PkgError::BadFormat)
    }
}

/// Parse a bundle and verify its signature + per-file hashes.
/// Returns the parsed bundle on success — caller is responsible for
/// installing or inspecting it.
pub fn parse_and_verify<'a>(
    bundle: &'a [u8],
    pubkey: &[u8; 32],
) -> Result<PkgBundle<'a>, PkgError> {
    if bundle.len() > MAX_BUNDLE { return Err(PkgError::SizeOverflow); }
    if bundle.len() < MAGIC.len() + 1 + SIG_LEN {
        return Err(PkgError::Truncated);
    }
    let body = &bundle[..bundle.len() - SIG_LEN];
    let sig_bytes_slice = &bundle[bundle.len() - SIG_LEN..];
    let mut sig_arr = [0u8; SIG_LEN];
    sig_arr.copy_from_slice(sig_bytes_slice);

    // Verify signature over the body BEFORE we trust any byte of it
    // to compute lengths / offsets.
    sig::ed25519_verify(pubkey, &sig_arr, body)
        .map_err(|_| PkgError::SigVerifyFailed)?;

    let mut c = Cursor::new(body);
    let magic = c.take(MAGIC.len())?;
    if magic != MAGIC { return Err(PkgError::BadMagic); }
    let version = c.read_u8()?;
    if version != FORMAT_VERSION { return Err(PkgError::BadVersion); }
    let name = c.read_str()?;
    let ver  = c.read_str()?;
    let file_count = c.read_u16_le()? as usize;

    let mut files = Vec::with_capacity(file_count);
    for _ in 0..file_count {
        let path = c.read_str()?;
        let size = c.read_u32_le()? as usize;
        let sha_slice = c.take(SHA256_LEN)?;
        let mut sha = [0u8; SHA256_LEN];
        sha.copy_from_slice(sha_slice);
        let content = c.take(size)?;

        // Per-file integrity. The signature already covers the
        // whole bundle, so a tampered content byte would fail the
        // sig check first. But the per-file sha lets a forensic
        // viewer cross-check just one entry without re-verifying
        // the entire bundle.
        let actual = crate::crypto::sha256::hash(content);
        if actual != sha { return Err(PkgError::Sha256Mismatch); }

        files.push(PkgFile { path, sha256: sha, content });
    }

    Ok(PkgBundle { name, version: ver, files })
}

/// Install a parsed bundle. Writes every entry into BatFS root and
/// records the package in `installed_packages`. Refuses to overwrite
/// an existing package — the operator must `pkg remove <name>` first.
pub fn install(bundle: &PkgBundle) -> Result<(), PkgError> {
    if is_installed(bundle.name) {
        return Err(PkgError::AlreadyInstalled);
    }

    // Pre-flight: refuse if any target path already exists. We don't
    // want a half-installed package to clobber an unrelated file. The
    // "already installed" check above handles the same-package case;
    // this catches the cross-package-conflict case.
    for f in &bundle.files {
        let mut probe = [0u8; 4];
        if batfs::read(f.path, &mut probe).is_ok() {
            return Err(PkgError::BatFsError("a target file already exists"));
        }
    }

    // Write files.
    for f in &bundle.files {
        batfs::create(f.path, f.content).map_err(PkgError::BatFsError)?;
    }

    // Append to installed DB.
    let mut db = read_db();
    let mut line = String::new();
    line.push_str(bundle.name);
    line.push('\t');
    line.push_str(bundle.version);
    for f in &bundle.files {
        line.push('\t');
        line.push_str(f.path);
    }
    line.push('\n');
    db.push_str(&line);
    write_db(&db).map_err(PkgError::BatFsError)?;
    Ok(())
}

/// Iterate over installed packages with (name, version, paths-tsv).
/// `paths-tsv` is tab-separated since BatFS file paths are
/// path-separator-free today (single namespace).
pub fn for_each_installed<F: FnMut(&str, &str, &str)>(mut f: F) {
    let db = read_db();
    for line in db.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let mut parts = line.splitn(3, '\t');
        let name = match parts.next() { Some(s) => s, None => continue };
        let ver  = match parts.next() { Some(s) => s, None => continue };
        let paths = parts.next().unwrap_or("");
        f(name, ver, paths);
    }
}

/// Returns true when `name` already appears in the installed DB.
pub fn is_installed(name: &str) -> bool {
    let db = read_db();
    for line in db.lines() {
        if let Some(rest) = line.split('\t').next() {
            if rest == name { return true; }
        }
    }
    false
}

/// Remove an installed package — delete every file the manifest
/// recorded, then rewrite the DB without it.
pub fn remove(name: &str) -> Result<(), PkgError> {
    let db = read_db();
    let mut found = false;
    let mut new_db = String::new();
    for line in db.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let mut parts = trimmed.splitn(3, '\t');
        let pkg_name = parts.next().unwrap_or("");
        if pkg_name == name {
            found = true;
            // Delete every recorded path.
            if let Some(_ver) = parts.next() {
                if let Some(paths) = parts.next() {
                    for path in paths.split('\t') {
                        if !path.is_empty() {
                            let _ = batfs::delete(path);
                        }
                    }
                }
            }
            continue; // drop this line
        }
        new_db.push_str(line);
        new_db.push('\n');
    }
    if !found { return Err(PkgError::NotInstalled); }
    write_db(&new_db).map_err(PkgError::BatFsError)?;
    Ok(())
}

fn read_db() -> String {
    let mut buf = [0u8; 8192];
    match batfs::read(INSTALLED_DB, &mut buf) {
        Ok(n) => String::from_utf8_lossy(&buf[..n]).into_owned(),
        Err(_) => String::new(),
    }
}

fn write_db(content: &str) -> Result<(), &'static str> {
    // BatFS doesn't have "create or replace" — delete first if exists.
    let _ = batfs::delete(INSTALLED_DB);
    batfs::create(INSTALLED_DB, content.as_bytes())
}
