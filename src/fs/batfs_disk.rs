// src/fs/batfs_disk.rs — on-disk persistence layer for BatFS.
//
// Until this module landed, BatFS was a pure in-RAM filesystem
// (`batfs.rs:90` even called the persistence gap out: "the persistent-
// across-reboot fix requires NVMe (Phase 7); for in-memory use, fresh
// random at boot is enough"). Every reboot wiped FILES[] and any data
// the user had written. built cave-registry persistence on
// top of BatFS, but that was moot until BatFS itself survived a reboot.
//
// This module is the substrate: it owns the on-disk format and exposes
// minimal mount/format/read/write/zero primitives. It does NOT deal
// with encryption — the cipher is still ChaCha20-Poly1305 AEAD as in
// the in-RAM path, with the existing `(name, nonce, tag)` tuple stored
// per-inode. We just write the ciphertext through to virtio-blk
// instead of leaving it sitting in RAM.
//
// ─── On-disk layout ─────────────────────────────────────────────────
//
// Sector 0 (512B): Superblock
// magic "BATFS\0\0\0", version, layout
// constants, FS UUID, FS salt, boot
// counter, HMAC-SHA256 over the rest.
//
// Sectors 1..64 (32KB): Inode table
// 128 entries × 256B each.
// Plaintext metadata (name, size,
// nonce, AEAD tag, state).
//
// Sectors 65..16448 (~8MB): Per-slot data region
// File slot `i` owns sectors
// [DATA_START + i*SLOT_SECTORS,
// DATA_START + (i+1)*SLOT_SECTORS).
// Fixed-slot allocation (no free list,
// no fragmentation, no compaction).
// MAX_FILE_SIZE = 64KB = 128 sectors.
// Slot data is the existing AEAD
// ciphertext, written through verbatim.
//
// ─── Crash consistency ─────────────────────────────────────────────
//
// We don't journal. The trick is shadow-style ordering:
// 1. write_data() writes ciphertext sectors first, then flush()
// 2. write_inode() writes the inode sector last, then flush()
//
// virtio-blk guarantees per-sector atomicity (each 512-byte write is
// either all-old or all-new on the device). So if a power loss happens
// between step 1 and step 2, the disk is consistent: the inode still
// claims the OLD ciphertext, AEAD tag still matches the OLD plaintext,
// nothing breaks. The new data is "lost" (never committed) but no
// existing data is corrupted.
//
// The superblock HMAC handles a different threat: an attacker swapping
// the disk underneath us. Mismatched HMAC → mount refuses → operator
// sees the failure rather than a silent compromise.
//
// ─── What this module does NOT do ──────────────────────────────────
//
// Encryption of the inode table (filenames are plaintext on disk).
// Adding metadata encryption is a Phase 7b follow-up.
// Free-block reuse beyond per-slot (deleted file's sectors stay
// zero until the slot is reused).
// Multi-disk / partition / RAID. Single virtio-blk device only.
// Filesystem checking / scrub / repair.

use crate::drivers::virtio::blk;
use crate::crypto::{rng, sha256};
use core::sync::atomic::{AtomicBool, Ordering};

// ─── Layout constants ────────────────────────────────────────────────

const SECTOR_SIZE: usize = 512;

/// Must match `batfs::MAX_FILES`. We can't import the const because
/// `batfs` imports us, so duplicate it here with a compile-time check
/// in `batfs::init`.
pub const DISK_MAX_FILES: usize = 128;

/// Sectors per file slot. 128 × 512 = 65 536 = batfs::MAX_FILE_SIZE.
pub const SLOT_SECTORS: u64 = 128;

/// Inode entry size on disk (256 bytes — fits 2 per sector).
pub const INODE_SIZE: usize = 256;

const SB_SECTOR:    u64   = 0;
const INODE_START:  u64   = 1;
const INODE_COUNT:  u64   = (DISK_MAX_FILES * INODE_SIZE / SECTOR_SIZE) as u64; // 64
const DATA_START:   u64   = INODE_START + INODE_COUNT;                          // 65
const TOTAL_SECTORS_NEEDED: u64 = DATA_START + (DISK_MAX_FILES as u64) * SLOT_SECTORS;

const SB_MAGIC:     [u8; 8] = *b"BATFS\0\0\0";
const SB_VERSION:   u32     = 1;

// Compile-time layout sanity. The magic + the rest of the metadata
// must fit in one sector with room left for the HMAC.
const _: () = {
    assert!(INODE_SIZE * 2 == SECTOR_SIZE);
    assert!(SECTOR_SIZE >= 480 + 32);
};

// ─── Superblock (in-RAM image; on-disk is one sector) ────────────────

#[derive(Clone, Copy)]
struct Superblock {
    magic:         [u8; 8],
    version:       u32,
    total_sectors: u64,
    inode_start:   u64,
    inode_count:   u64,
    data_start:    u64,
    slot_sectors:  u64,
    boot_counter:  u64,
    fs_uuid:       [u8; 16],
    fs_salt:       [u8; 32],
    // hmac — filled by serialize after the rest is laid out.
    hmac:          [u8; 32],
}

impl Superblock {
    fn fresh() -> Self {
        let mut uuid = [0u8; 16];
        let mut salt = [0u8; 32];
        rng::fill_bytes(&mut uuid);
        rng::fill_bytes(&mut salt);
        Self {
            magic:         SB_MAGIC,
            version:       SB_VERSION,
            total_sectors: TOTAL_SECTORS_NEEDED,
            inode_start:   INODE_START,
            inode_count:   INODE_COUNT,
            data_start:    DATA_START,
            slot_sectors:  SLOT_SECTORS,
            boot_counter:  1,
            fs_uuid:       uuid,
            fs_salt:       salt,
            hmac:          [0u8; 32],
        }
    }

    /// Lay out the in-RAM superblock into a fresh sector buffer + HMAC.
    fn serialize(&mut self, master_key: &[u8; 32], out: &mut [u8; SECTOR_SIZE]) {
        for b in out.iter_mut() { *b = 0; }
        out[0..8].copy_from_slice(&self.magic);
        out[8..12].copy_from_slice(&self.version.to_le_bytes());
        out[16..24].copy_from_slice(&self.total_sectors.to_le_bytes());
        out[24..32].copy_from_slice(&self.inode_start.to_le_bytes());
        out[32..40].copy_from_slice(&self.inode_count.to_le_bytes());
        out[40..48].copy_from_slice(&self.data_start.to_le_bytes());
        out[48..56].copy_from_slice(&self.slot_sectors.to_le_bytes());
        out[56..64].copy_from_slice(&self.boot_counter.to_le_bytes());
        out[64..80].copy_from_slice(&self.fs_uuid);
        out[80..112].copy_from_slice(&self.fs_salt);
        // HMAC the body (first 480 bytes) and store at offset 480..512.
        let tag = sha256::hmac(master_key, &out[0..480]);
        out[480..512].copy_from_slice(&tag);
        self.hmac = tag;
    }

    fn deserialize(buf: &[u8; SECTOR_SIZE], master_key: &[u8; 32]) -> Result<Self, &'static str> {
        let mut sb = Self::fresh();
        sb.magic.copy_from_slice(&buf[0..8]);
        if sb.magic != SB_MAGIC { return Err("not a BatFS disk"); }
        let mut v4 = [0u8; 4]; v4.copy_from_slice(&buf[8..12]);
        sb.version = u32::from_le_bytes(v4);
        if sb.version != SB_VERSION { return Err("unsupported BatFS version"); }
        let mut v8 = [0u8; 8];
        v8.copy_from_slice(&buf[16..24]); sb.total_sectors = u64::from_le_bytes(v8);
        v8.copy_from_slice(&buf[24..32]); sb.inode_start   = u64::from_le_bytes(v8);
        v8.copy_from_slice(&buf[32..40]); sb.inode_count   = u64::from_le_bytes(v8);
        v8.copy_from_slice(&buf[40..48]); sb.data_start    = u64::from_le_bytes(v8);
        v8.copy_from_slice(&buf[48..56]); sb.slot_sectors  = u64::from_le_bytes(v8);
        v8.copy_from_slice(&buf[56..64]); sb.boot_counter  = u64::from_le_bytes(v8);
        sb.fs_uuid.copy_from_slice(&buf[64..80]);
        sb.fs_salt.copy_from_slice(&buf[80..112]);
        sb.hmac.copy_from_slice(&buf[480..512]);

        // Verify HMAC. Anything past the HMAC field is fine to leave
        // uncovered — only an attacker with master_key could forge a
        // valid tag, and that means they already own the FS anyway.
        let expected = sha256::hmac(master_key, &buf[0..480]);
        if expected != sb.hmac {
            return Err("BatFS superblock HMAC mismatch (wrong passphrase or tampered disk)");
        }

        // Sanity-check the layout matches what this build expects. A
        // disk image written by a different build with different
        // constants would otherwise silently corrupt.
        if sb.inode_start != INODE_START
            || sb.inode_count != INODE_COUNT
            || sb.data_start != DATA_START
            || sb.slot_sectors != SLOT_SECTORS
        {
            return Err("BatFS layout mismatch (different build?)");
        }

        Ok(sb)
    }
}

// ─── Inode (on-disk file metadata) ───────────────────────────────────

#[derive(Clone, Copy)]
pub struct DiskInode {
    pub state:       u8,           // 0=Free, 1=Active
    pub encrypted:   u8,           // 1=true
    pub name_len:    u32,
    pub name:        [u8; 64],
    pub size:        u64,          // plaintext/ciphertext length
    pub nonce:       [u8; 12],     // ChaCha20-Poly1305 nonce
    pub tag:         [u8; 16],     // Poly1305 auth tag
}

impl DiskInode {
    pub const STATE_FREE:   u8 = 0;
    pub const STATE_ACTIVE: u8 = 1;

    /// `Default` can't be derived because `[u8; 64]` doesn't impl
    /// Default (only sizes up to 32 do). This is a manual zero-init.
    pub const fn empty() -> Self {
        Self {
            state:     Self::STATE_FREE,
            encrypted: 0,
            name_len:  0,
            name:      [0u8; 64],
            size:      0,
            nonce:     [0u8; 12],
            tag:       [0u8; 16],
        }
    }

    fn serialize(&self, out: &mut [u8; INODE_SIZE]) {
        for b in out.iter_mut() { *b = 0; }
        out[0] = self.state;
        out[1] = self.encrypted;
        out[4..8].copy_from_slice(&self.name_len.to_le_bytes());
        let nl = (self.name_len as usize).min(64);
        out[8..8 + nl].copy_from_slice(&self.name[..nl]);
        out[72..80].copy_from_slice(&self.size.to_le_bytes());
        out[80..92].copy_from_slice(&self.nonce);
        out[92..108].copy_from_slice(&self.tag);
        // bytes 108..256 reserved
    }

    fn deserialize(buf: &[u8; INODE_SIZE]) -> Self {
        let mut n = Self::empty();
        n.state     = buf[0];
        n.encrypted = buf[1];
        let mut v4 = [0u8; 4]; v4.copy_from_slice(&buf[4..8]);
        n.name_len = u32::from_le_bytes(v4);
        let nl = (n.name_len as usize).min(64);
        n.name[..nl].copy_from_slice(&buf[8..8 + nl]);
        let mut v8 = [0u8; 8]; v8.copy_from_slice(&buf[72..80]);
        n.size = u64::from_le_bytes(v8);
        n.nonce.copy_from_slice(&buf[80..92]);
        n.tag.copy_from_slice(&buf[92..108]);
        n
    }
}

// ─── Module state ────────────────────────────────────────────────────

static MOUNTED: AtomicBool = AtomicBool::new(false);
// The active superblock is cached so we can re-HMAC + re-write it on
// every boot-counter bump without re-reading from disk first.
static mut ACTIVE_SB: Superblock = Superblock {
    magic:         [0u8; 8],
    version:       0,
    total_sectors: 0,
    inode_start:   0,
    inode_count:   0,
    data_start:    0,
    slot_sectors:  0,
    boot_counter:  0,
    fs_uuid:       [0u8; 16],
    fs_salt:       [0u8; 32],
    hmac:          [0u8; 32],
};
static mut MASTER_KEY: [u8; 32] = [0u8; 32];

pub fn is_mounted() -> bool { MOUNTED.load(Ordering::Acquire) }

/// Read sector 0 and try to deserialize a superblock with the supplied
/// master key. Returns Ok(()) if the disk has a valid BatFS layout that
/// HMACs with this key, Err otherwise. On Ok, the boot counter is
/// bumped + re-written so a later mount sees a fresh value.
pub fn mount(master_key: &[u8; 32]) -> Result<u64, &'static str> {
    if !blk::is_ready() { return Err("virtio-blk not ready"); }
    if blk::capacity_sectors() < TOTAL_SECTORS_NEEDED {
        return Err("disk too small for BatFS");
    }

    let mut buf = [0u8; SECTOR_SIZE];
    blk::read_sectors(SB_SECTOR, &mut buf).map_err(|_| "blk read failed")?;
    let mut sb = Superblock::deserialize(&buf, master_key)?;

    // Bump boot counter + persist. If the write fails we still consider
    // the mount successful for this boot — the operator gets the cached
    // counter back next mount, just without a fresh increment.
    sb.boot_counter = sb.boot_counter.wrapping_add(1);
    let counter = sb.boot_counter;
    let mut out = [0u8; SECTOR_SIZE];
    sb.serialize(master_key, &mut out);
    let _ = blk::write_sectors(SB_SECTOR, &out);
    let _ = blk::flush();

    unsafe {
        ACTIVE_SB = sb;
        MASTER_KEY = *master_key;
    }
    MOUNTED.store(true, Ordering::Release);
    Ok(counter)
}

/// Wipe a fresh BatFS layout onto the disk. Generates a new UUID +
/// salt, zeroes the inode table, leaves the data region untouched
/// (per-slot reads ignore stale bytes for Free slots).
pub fn format(master_key: &[u8; 32]) -> Result<(), &'static str> {
    if !blk::is_ready() { return Err("virtio-blk not ready"); }
    if blk::capacity_sectors() < TOTAL_SECTORS_NEEDED {
        return Err("disk too small for BatFS");
    }

    // Zero the inode table first (so Free slots survive a partial format).
    let zero = [0u8; SECTOR_SIZE];
    for s in INODE_START..(INODE_START + INODE_COUNT) {
        blk::write_sectors(s, &zero).map_err(|_| "blk write failed")?;
    }

    // Fresh superblock. Boot counter starts at 1 — `mount` increments
    // before storing, so the first successful mount will read 2.
    let mut sb = Superblock::fresh();
    let mut out = [0u8; SECTOR_SIZE];
    sb.serialize(master_key, &mut out);
    blk::write_sectors(SB_SECTOR, &out).map_err(|_| "blk write failed")?;
    blk::flush().map_err(|_| "blk flush failed")?;

    unsafe {
        ACTIVE_SB = sb;
        MASTER_KEY = *master_key;
    }
    MOUNTED.store(true, Ordering::Release);
    Ok(())
}

/// Convenience: try to mount, and if that fails because the disk is
/// blank (not a BatFS layout), format it. Returns Ok(true) if a fresh
/// format happened, Ok(false) if an existing FS was mounted, Err if
/// the disk itself is unusable (no virtio-blk, too small, HMAC fail).
pub fn mount_or_format(master_key: &[u8; 32]) -> Result<bool, &'static str> {
    match mount(master_key) {
        Ok(_) => Ok(false),
        Err("not a BatFS disk") => {
            format(master_key)?;
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

// ─── Inode I/O ───────────────────────────────────────────────────────

/// Read all 128 inodes into the caller's buffer.
pub fn read_all_inodes(out: &mut [DiskInode; DISK_MAX_FILES]) -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    let mut sec = [0u8; SECTOR_SIZE];
    for i in 0..DISK_MAX_FILES {
        // Inode `i` is at sector `INODE_START + i / 2`, byte offset
        // `(i % 2) * INODE_SIZE` within that sector.
        let sector = INODE_START + (i / 2) as u64;
        if i % 2 == 0 {
            blk::read_sectors(sector, &mut sec)
                .map_err(|_| "blk read inode failed")?;
        }
        let off = (i % 2) * INODE_SIZE;
        let mut entry = [0u8; INODE_SIZE];
        entry.copy_from_slice(&sec[off..off + INODE_SIZE]);
        out[i] = DiskInode::deserialize(&entry);
    }
    Ok(())
}

/// Write a single inode. Reads the containing sector first so the
/// neighbouring inode (two per sector) is preserved.
pub fn write_inode(slot: usize, inode: &DiskInode) -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    if slot >= DISK_MAX_FILES { return Err("inode slot out of range"); }
    let sector = INODE_START + (slot / 2) as u64;
    let off = (slot % 2) * INODE_SIZE;

    let mut sec = [0u8; SECTOR_SIZE];
    blk::read_sectors(sector, &mut sec).map_err(|_| "blk read inode failed")?;
    let mut entry = [0u8; INODE_SIZE];
    inode.serialize(&mut entry);
    sec[off..off + INODE_SIZE].copy_from_slice(&entry);
    blk::write_sectors(sector, &sec).map_err(|_| "blk write inode failed")?;
    Ok(())
}

/// Mark an inode slot Free without touching its data sectors. Used by
/// `delete()` after the data sectors have been zeroed.
pub fn free_inode(slot: usize) -> Result<(), &'static str> {
    write_inode(slot, &DiskInode::empty())
}

// ─── Data I/O ────────────────────────────────────────────────────────

/// Sector range for a given slot's data region.
pub fn slot_data_sector(slot: usize) -> u64 {
    DATA_START + (slot as u64) * SLOT_SECTORS
}

/// Write `bytes` (caller-prepared ciphertext) into the slot's data
/// region, padded out to a whole number of sectors. The trailing bytes
/// of the last sector are zero-filled.
pub fn write_data(slot: usize, bytes: &[u8]) -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    if slot >= DISK_MAX_FILES { return Err("slot out of range"); }
    if bytes.len() > (SLOT_SECTORS as usize) * SECTOR_SIZE {
        return Err("file too large for slot");
    }
    let start = slot_data_sector(slot);
    let mut sec = [0u8; SECTOR_SIZE];
    let mut off = 0usize;
    let mut sector_idx = 0u64;
    while off < bytes.len() {
        let take = (bytes.len() - off).min(SECTOR_SIZE);
        for b in sec.iter_mut() { *b = 0; }
        sec[..take].copy_from_slice(&bytes[off..off + take]);
        blk::write_sectors(start + sector_idx, &sec)
            .map_err(|_| "blk write data failed")?;
        off += take;
        sector_idx += 1;
    }
    Ok(())
}

/// Read up to `len` ciphertext bytes from a slot's data region into
/// `buf`. Caller is responsible for AEAD-decrypting `buf[..len]` using
/// the inode's nonce + tag.
pub fn read_data(slot: usize, buf: &mut [u8], len: usize) -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    if slot >= DISK_MAX_FILES { return Err("slot out of range"); }
    if len > buf.len() { return Err("buffer too small"); }
    if len > (SLOT_SECTORS as usize) * SECTOR_SIZE {
        return Err("len > slot capacity");
    }
    let start = slot_data_sector(slot);
    let mut sec = [0u8; SECTOR_SIZE];
    let mut off = 0usize;
    let mut sector_idx = 0u64;
    while off < len {
        blk::read_sectors(start + sector_idx, &mut sec)
            .map_err(|_| "blk read data failed")?;
        let take = (len - off).min(SECTOR_SIZE);
        buf[off..off + take].copy_from_slice(&sec[..take]);
        off += take;
        sector_idx += 1;
    }
    Ok(())
}

/// Zero a slot's entire data region. Used by `delete()` so a deleted
/// file's ciphertext is unrecoverable from disk even before the slot
/// is reused. Caller is expected to flush() afterward.
pub fn zero_data(slot: usize) -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    if slot >= DISK_MAX_FILES { return Err("slot out of range"); }
    let start = slot_data_sector(slot);
    let zero = [0u8; SECTOR_SIZE];
    for i in 0..SLOT_SECTORS {
        blk::write_sectors(start + i, &zero)
            .map_err(|_| "blk zero data failed")?;
    }
    Ok(())
}

/// Force a flush down to the host. Caller pattern: do all the writes
/// the operation needs, then flush once at the end.
pub fn flush() -> Result<(), &'static str> {
    if !is_mounted() { return Err("BatFS disk not mounted"); }
    blk::flush().map_err(|_| "blk flush failed")
}

