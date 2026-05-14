// src/caves/persist.rs —
//
// Cave registry persistence.
//
// DESIGN_BATCAVES.md line 87: "Persistent (default): survives reboots,
// tools stay installed". Until this module landed, that line was a lie:
// CAVES[] was a static-mut RAM array with no save/load wiring, so every
// reboot started with an empty cave table. Persistent caves' BatFS data
// survived (per-cave fs_key is deterministic from boot_master_key + name)
// but the OS no longer remembered the caves existed.
//
// This module makes the spec real. Each Persistent cave gets a manifest
// file in BatFS named "__cave__<name>". The manifest captures everything
// needed to rebuild the in-RAM CAVES[] entry on next boot — name, type,
// caps, tools, and (for docker-backed caves) image reference.
//
// What does NOT persist (re-derived/re-allocated at boot):
// state caves wake up Stopped
// fs_key deterministic from (boot master_key, name)
// display_x/y/w/h re-allocated by `caves enter`
//
// Lifecycle hooks — every cave-state mutation goes through one of:
// cave::create() → save() if Persistent
// cave::create_docker() → save() (re-saves with image)
// cave::grant_cap() → save() (refresh manifest)
// cave::revoke_cap() → save()
// cave::install_tool() → save()
// cave::seal() → delete() (no longer Persistent)
// cave::destroy() → delete()
// cave::init() → restore_all() (re-populate CAVES[])
//
// The manifest is encrypted by BatFS itself (ChaCha20-Poly1305 AEAD,
// keyed off the boot master_key + filename). It's just a regular BatFS
// file; the persistence layer doesn't add any crypto of its own.

use crate::caves::cave::{
    Cave, CaveBacking, CaveCap, CaveTool, CaveState, CaveType,
    MAX_NAME, MAX_CAPS, MAX_CAP_NAME, MAX_TOOLS, MAX_TOOL_NAME, MAX_IMAGE,
    CAVES, MAX_CAVES,
};
use crate::fs::batfs;
use crate::crypto::sha256;

// ─── Manifest format (v1) ──────────────────────────────────────────────
//
// Fixed-size 2048-byte buffer; offsets are stable across versions so a
// future v2 can extend the trailing reserved region without breaking
// existing manifests on disk.
//
// Offset Size Field
// 0 4 magic = b"CAVE"
// 4 1 version = 1
// 5 3 reserved (zero)
// 8 1 name_len
// 9 32 name (zero-padded, MAX_NAME = 32)
// 41 1 backing (0=Native, 1=Docker)
// 42 1 image_len (MAX_IMAGE = 64 fits in u8)
// 43 64 image (zero-padded)
// 107 1 cap_count
// 108 784 caps[16] × { 1 byte len + 48 bytes name } = 16 × 49
// 892 1 tool_count
// 893 1056 tools[32] × { 1 byte len + 32 bytes name } = 32 × 33
// 1949 99 reserved (zero) — room for created_ms, fs_quota, etc.
// 2048 — end

const MANIFEST_PREFIX:  &str = "__cave__";
const MANIFEST_MAGIC:   [u8; 4] = *b"CAVE";
const MANIFEST_VERSION: u8 = 1;
const MANIFEST_SIZE:    usize = 2048;

const OFF_NAME_LEN:    usize = 8;
const OFF_NAME:        usize = 9;
const OFF_BACKING:     usize = 41;
const OFF_IMAGE_LEN:   usize = 42;
const OFF_IMAGE:       usize = 43;
const OFF_CAP_COUNT:   usize = 107;
const OFF_CAPS:        usize = 108;
const CAP_ENTRY_SIZE:  usize = 1 + MAX_CAP_NAME;          // 49
const OFF_TOOL_COUNT:  usize = OFF_CAPS + MAX_CAPS * CAP_ENTRY_SIZE; // 892
const OFF_TOOLS:       usize = OFF_TOOL_COUNT + 1;        // 893
const TOOL_ENTRY_SIZE: usize = 1 + MAX_TOOL_NAME;         // 33

// Compile-time sanity check that the layout fits.
const _: () = {
    let needed = OFF_TOOLS + MAX_TOOLS * TOOL_ENTRY_SIZE;
    assert!(needed <= MANIFEST_SIZE);
};

// ─── Filename helper ───────────────────────────────────────────────────

/// Build "__cave__<name>" into `out`. Returns the number of bytes
/// written. Caller guarantees `out` is at least 64 bytes (BatFS
/// MAX_FILENAME).
fn manifest_name(cave_name: &str, out: &mut [u8]) -> usize {
    let pre = MANIFEST_PREFIX.as_bytes();
    let mut p = 0usize;
    let n = pre.len().min(out.len());
    out[..n].copy_from_slice(&pre[..n]);
    p += n;
    let nb = cave_name.as_bytes();
    let take = nb.len().min(out.len() - p);
    out[p..p + take].copy_from_slice(&nb[..take]);
    p + take
}

// ─── Serialization ─────────────────────────────────────────────────────

fn serialize(cave: &Cave, out: &mut [u8; MANIFEST_SIZE]) {
    // Defensive: zero the whole buffer so no stale stack bytes leak
    // through reserved regions.
    for b in out.iter_mut() { *b = 0; }

    // Header
    out[0..4].copy_from_slice(&MANIFEST_MAGIC);
    out[4] = MANIFEST_VERSION;

    // Name
    let nl = cave.name_len.min(MAX_NAME);
    out[OFF_NAME_LEN] = nl as u8;
    out[OFF_NAME..OFF_NAME + nl].copy_from_slice(&cave.name[..nl]);

    // Backing
    out[OFF_BACKING] = match cave.backing {
        CaveBacking::Native => 0,
        CaveBacking::Docker => 1,
    };

    // Image
    let il = cave.image_len.min(MAX_IMAGE);
    out[OFF_IMAGE_LEN] = il as u8;
    out[OFF_IMAGE..OFF_IMAGE + il].copy_from_slice(&cave.image[..il]);

    // Caps
    let cc = cave.cap_count.min(MAX_CAPS);
    out[OFF_CAP_COUNT] = cc as u8;
    for i in 0..cc {
        let cap = &cave.caps[i];
        if !cap.active { continue; }
        let entry = OFF_CAPS + i * CAP_ENTRY_SIZE;
        let nl = cap.name_len.min(MAX_CAP_NAME);
        out[entry] = nl as u8;
        out[entry + 1..entry + 1 + nl].copy_from_slice(&cap.name[..nl]);
    }

    // Tools
    let tc = cave.tool_count.min(MAX_TOOLS);
    out[OFF_TOOL_COUNT] = tc as u8;
    for i in 0..tc {
        let t = &cave.tools[i];
        if !t.installed { continue; }
        let entry = OFF_TOOLS + i * TOOL_ENTRY_SIZE;
        let nl = t.name_len.min(MAX_TOOL_NAME);
        out[entry] = nl as u8;
        out[entry + 1..entry + 1 + nl].copy_from_slice(&t.name[..nl]);
    }
}

fn deserialize(buf: &[u8; MANIFEST_SIZE], cave: &mut Cave) -> Result<(), &'static str> {
    if buf[0..4] != MANIFEST_MAGIC { return Err("bad magic"); }
    if buf[4] != MANIFEST_VERSION { return Err("bad version"); }

    // Reset cave to a clean Persistent/Stopped baseline before populating.
    *cave = Cave::empty();
    cave.cave_type = CaveType::Persistent;
    cave.state = CaveState::Stopped;

    // Name
    let nl = (buf[OFF_NAME_LEN] as usize).min(MAX_NAME);
    cave.name[..nl].copy_from_slice(&buf[OFF_NAME..OFF_NAME + nl]);
    cave.name_len = nl;

    // Backing
    cave.backing = match buf[OFF_BACKING] {
        0 => CaveBacking::Native,
        1 => CaveBacking::Docker,
        _ => return Err("bad backing"),
    };

    // Image
    let il = (buf[OFF_IMAGE_LEN] as usize).min(MAX_IMAGE);
    cave.image[..il].copy_from_slice(&buf[OFF_IMAGE..OFF_IMAGE + il]);
    cave.image_len = il;

    // Caps
    let cc = (buf[OFF_CAP_COUNT] as usize).min(MAX_CAPS);
    for i in 0..cc {
        let entry = OFF_CAPS + i * CAP_ENTRY_SIZE;
        let nl = (buf[entry] as usize).min(MAX_CAP_NAME);
        cave.caps[i] = CaveCap::empty();
        cave.caps[i].name_len = nl;
        cave.caps[i].name[..nl].copy_from_slice(&buf[entry + 1..entry + 1 + nl]);
        cave.caps[i].active = nl > 0;
    }
    cave.cap_count = cc;

    // Tools
    let tc = (buf[OFF_TOOL_COUNT] as usize).min(MAX_TOOLS);
    for i in 0..tc {
        let entry = OFF_TOOLS + i * TOOL_ENTRY_SIZE;
        let nl = (buf[entry] as usize).min(MAX_TOOL_NAME);
        cave.tools[i] = CaveTool::empty();
        cave.tools[i].name_len = nl;
        cave.tools[i].name[..nl].copy_from_slice(&buf[entry + 1..entry + 1 + nl]);
        cave.tools[i].installed = nl > 0;
    }
    cave.tool_count = tc;

    // fs_key is re-derived deterministically from the boot master key
    // and the cave name (same formula `cave::create` uses). It was never
    // serialized to disk in plaintext.
    cave.fs_key = sha256::derive_key(
        &batfs::master_key(),
        &cave.name[..cave.name_len],
    );

    Ok(())
}

// ─── Public API ────────────────────────────────────────────────────────

/// Save a cave's manifest to BatFS. Idempotent — overwrites if a stale
/// manifest is already present. No-op for Ephemeral or Free caves.
pub fn save(cave: &Cave) {
    if cave.cave_type == CaveType::Ephemeral { return; }
    if cave.state == CaveState::Free { return; }

    let mut nb = [0u8; 64];
    let nlen = manifest_name(cave.name_str(), &mut nb);
    let path = unsafe { core::str::from_utf8_unchecked(&nb[..nlen]) };

    let mut data = [0u8; MANIFEST_SIZE];
    serialize(cave, &mut data);

    // BatFS::create errors on duplicate name. Delete-then-create is the
    // simple "upsert". Both calls are silent on error (we can't recover
    // from BatFS being uninitialized this far in, and an audit entry is
    // already produced by callers like grant_cap/destroy).
    let _ = batfs::delete(path);
    let _ = batfs::create(path, &data);
}

/// Remove a cave's manifest from BatFS. Idempotent — silent if no
/// manifest exists. Called on `seal` (cave is no longer Persistent) and
/// `destroy` (cave is gone).
pub fn delete(name: &str) {
    let mut nb = [0u8; 64];
    let nlen = manifest_name(name, &mut nb);
    let path = unsafe { core::str::from_utf8_unchecked(&nb[..nlen]) };
    let _ = batfs::delete(path);
}

/// Scan BatFS for "__cave__*" manifests, deserialize each, and install
/// it into a free CAVES[] slot. Returns the number of caves restored.
// /
/// Called once during `cave::init()` after `batfs::init()` has unlocked
/// the filesystem with the operator's passphrase.
pub fn restore_all() -> usize {
    // First pass — collect the manifest filenames into a fixed buffer.
    // We can't deserialize during the `list` callback because that would
    // need to mutate CAVES while batfs holds its internal locks.
    let mut names = [[0u8; 64]; MAX_CAVES];
    let mut name_lens = [0usize; MAX_CAVES];
    let mut name_count = 0usize;

    batfs::list(|name, _size, _enc| {
        if !name.starts_with(MANIFEST_PREFIX) { return; }
        if name_count >= MAX_CAVES { return; }
        let nb = name.as_bytes();
        let nl = nb.len().min(64);
        names[name_count][..nl].copy_from_slice(&nb[..nl]);
        name_lens[name_count] = nl;
        name_count += 1;
    });

    // Second pass — read + deserialize each manifest.
    let mut restored = 0usize;
    let mut buf = [0u8; MANIFEST_SIZE];

    for i in 0..name_count {
        let path = unsafe {
            core::str::from_utf8_unchecked(&names[i][..name_lens[i]])
        };
        // Zero the read buffer in case the file is shorter than expected
        // (deserialize would otherwise read trailing stack noise).
        for b in buf.iter_mut() { *b = 0; }
        match batfs::read(path, &mut buf) {
            Ok(_n) => {
                unsafe {
                    let slot = (0..MAX_CAVES)
                        .find(|&j| CAVES[j].state == CaveState::Free);
                    if let Some(slot) = slot {
                        if deserialize(&buf, &mut CAVES[slot]).is_ok() {
                            restored += 1;
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    // Audit ring entry so the operator can see in `info audit` that
    // the cave registry was restored from disk this boot.
    if restored > 0 {
        let mut buf = [0u8; 64];
        let pre = b"caves restored from BatFS: ";
        let mut p = 0usize;
        let n = pre.len().min(buf.len());
        buf[..n].copy_from_slice(&pre[..n]);
        p += n;
        p += write_dec(restored, &mut buf[p..]);
        crate::security::audit::record(
            crate::security::audit::Category::Cave,
            &buf[..p],
        );
    }

    restored
}

fn write_dec(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 {
        if !out.is_empty() { out[0] = b'0'; return 1; }
        return 0;
    }
    let mut tmp = [0u8; 20];
    let mut p = 0usize;
    while n > 0 && p < tmp.len() {
        tmp[p] = b'0' + (n % 10) as u8;
        n /= 10;
        p += 1;
    }
    let len = p.min(out.len());
    for i in 0..len {
        out[i] = tmp[p - 1 - i];
    }
    len
}
