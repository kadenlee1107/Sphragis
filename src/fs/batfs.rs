#![allow(dead_code)]
// Bat_OS — BatFS: Custom Encrypted Filesystem
//
// DESIGN_CRYPTO.md #2: per-file **ChaCha20-Poly1305 AEAD**. Replaces
// the prior AES-256-CTR + HMAC-SHA256 encrypt-then-MAC construction
// with a single AEAD primitive: one pass, constant-time, no timing
// side channels on the key, integrity bundled inline with the tag.
//
// Per-file key is still derived via `sha256::derive_key(master,
// filename)` — that's the HKDF role, unchanged and fast. The change
// is only to the record encryption primitive.

use crate::crypto::sha256;
use crate::kernel::mm::frame;
use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    ChaCha20Poly1305,
};

const MAX_FILES: usize = 128;
const MAX_FILENAME: usize = 64;
const MAX_FILE_SIZE: usize = 64 * 1024; // 64KB max per file for now
const BLOCK_SIZE: usize = 4096;

#[derive(Clone, Copy, PartialEq)]
pub enum FileState {
    Free,
    Active,
    Deleted,
}

#[derive(Clone, Copy)]
pub struct FileEntry {
    pub state: FileState,
    pub name: [u8; MAX_FILENAME],
    pub name_len: usize,
    pub size: usize,
    pub data_addr: usize,       // Physical address of encrypted data
    pub nonce: [u8; 12],        // CTR nonce (unique per file)
    pub hash: [u8; 32],         // SHA-256 of plaintext (integrity)
    pub encrypted: bool,
}

impl FileEntry {
    pub const fn empty() -> Self {
        Self {
            state: FileState::Free,
            name: [0u8; MAX_FILENAME],
            name_len: 0,
            size: 0,
            data_addr: 0,
            nonce: [0u8; 12],
            hash: [0u8; 32],
            encrypted: false,
        }
    }

    /// NEW-CRYPTO-029: validate UTF-8 instead of `from_utf8_unchecked`. A
    /// non-UTF8 filename slipping in (via a future raw-bytes API) would
    /// otherwise be UB. Returns "" on invalid UTF-8 — callers compare by
    /// byte slice via `name_bytes`, not by &str, for filename matching.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Raw bytes — preferred for byte-exact filename comparisons.
    pub fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len]
    }
}

static mut FILES: [FileEntry; MAX_FILES] = {
    const EMPTY: FileEntry = FileEntry::empty();
    [EMPTY; MAX_FILES]
};

static mut FILE_COUNT: usize = 0;
static mut MASTER_KEY: [u8; 32] = [0u8; 32];
// V5-CRYPTO-002 fix: NONCE_COUNTER is now atomic. The old `static mut
// NONCE_COUNTER: u64` with non-atomic `n = NONCE_COUNTER; NONCE_COUNTER
// += 1` could race between concurrent `create()` calls (even on a
// single-core kernel, the new V4 deferred-preemption scheduler can
// interleave them). Two creates racing on the same filename would
// produce the same derived key and the same nonce, meaning the two
// plaintexts xor to the same CTR keystream — recoverable.
static NONCE_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
// FL-027 / NEW-CRYPTO-007 fix: per-boot random 4-byte prefix mixed into
// every CTR nonce. Without this, re-encrypting the same filename across
// boots gave the same (key, IV) — a crib-drag on recurring files. The
// persistent-across-reboot path ( Phase 7 / `batfs_disk.rs`)
// stores the per-file `nonce` in the inode on disk, so recurring files
// keep their original nonce across reboots; the BOOT_NONCE_PREFIX only
// affects NEW files created after mount, where it still guarantees no
// reuse against the file-table images on the previous boot.
static mut BOOT_NONCE_PREFIX: [u8; 4] = [0u8; 4];
static mut INITIALIZED: bool = false;

// ─── Merkle Tree ───
// Binary hash tree over all file hashes.
// Leaf[i] = SHA-256(file[i].hash). Internal nodes = SHA-256(left || right).
// Tree has MAX_FILES leaves → 2*MAX_FILES nodes total.
const MERKLE_NODES: usize = MAX_FILES * 2;
static mut MERKLE_TREE: [[u8; 32]; MERKLE_NODES] = [[0u8; 32]; MERKLE_NODES];

/// expose the per-boot master key for
/// per-cave fs_key derivation. Volatile read so the compiler can't
/// dead-store-eliminate a sensitive value in some future refactor.
pub fn master_key() -> [u8; 32] {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(MASTER_KEY)) }
}

/// Rebuild the Merkle tree from all file hashes.
pub fn rebuild_merkle() {
    unsafe {
        // Leaves: nodes[MAX_FILES..2*MAX_FILES] = hash of each file's hash
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active {
                MERKLE_TREE[MAX_FILES + i] = sha256::hash(&FILES[i].hash);
            } else {
                MERKLE_TREE[MAX_FILES + i] = [0u8; 32];
            }
        }
        // Internal nodes: bottom-up
        let mut level = MAX_FILES;
        while level > 1 {
            let parent_start = level / 2;
            for i in parent_start..level {
                let left = &MERKLE_TREE[i * 2];
                let right = &MERKLE_TREE[i * 2 + 1];
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(left);
                combined[32..].copy_from_slice(right);
                MERKLE_TREE[i] = sha256::hash(&combined);
            }
            level /= 2;
        }
    }
}

/// Get the Merkle root hash (integrity fingerprint of entire filesystem).
pub fn merkle_root() -> [u8; 32] {
    unsafe { MERKLE_TREE[1] }
}

/// Verify a specific file's integrity against the Merkle tree.
pub fn verify_file_integrity(idx: usize) -> bool {
    if idx >= MAX_FILES { return false; }
    unsafe {
        if FILES[idx].state != FileState::Active { return false; }
        // Recompute this leaf
        let expected = sha256::hash(&FILES[idx].hash);
        MERKLE_TREE[MAX_FILES + idx] == expected
    }
}

/// Verify the entire filesystem integrity.
pub fn verify_all_integrity() -> bool {
    let saved_root = merkle_root();
    rebuild_merkle();
    merkle_root() == saved_root
}

/// Initialize the filesystem with a master encryption key.
// /
/// FL-027 / NEW-CRYPTO-007: each boot gets a fresh 4-byte random nonce
/// prefix so re-encrypting the same filename under the same derived key
/// produces a different CTR stream than the previous boot did. The
/// counter itself still restarts at 1; prefix + counter is the full
/// unique value.
// /
/// if a virtio-blk device is attached, this also
/// mounts the on-disk format from sector 0 (or freshly formats it if
/// the disk is blank). Inodes + ciphertext are restored from disk into
/// `FILES[]` and per-file RAM pages. With no disk, we run as before
/// (RAM-only) — the operator gets a UART warning so they know.
pub fn init(master_key: &[u8; 32]) {
    use core::sync::atomic::Ordering;

    crate::critical_section! {
        unsafe {
            if INITIALIZED {
                // Re-init without wipe would cause keystream reuse
                // against existing file nonces. Refuse.
                return;
            }
            MASTER_KEY = *master_key;
            FILE_COUNT = 0;
            let mut rnd = [0u8; 4];
            crate::crypto::rng::fill_bytes(&mut rnd);
            BOOT_NONCE_PREFIX = rnd;
            // V8 fix: publish counter BEFORE flipping INITIALIZED so any
            // reader that sees INITIALIZED=true (with Acquire) observes a
            // consistent (prefix, counter). Readers on the Relaxed side
            // are now also safe because IRQ is masked.
            NONCE_COUNTER.store(1, Ordering::Release);
            INITIALIZED = true;
        }
    }

    // try to mount/format the on-disk BatFS. Outside the
    // critical_section above because virtio-blk uses MMIO + DMA + IRQ
    // and shouldn't run with IRQs masked the whole time.
    init_disk(master_key);
}

/// mount the on-disk BatFS layout, or format it
/// fresh if the disk is blank / a different layout. Restores the inode
/// table + per-file ciphertext into `FILES[]` and RAM pages.
fn init_disk(master_key: &[u8; 32]) {
    use crate::drivers::uart;
    use super::batfs_disk;

    if !crate::drivers::virtio::blk::is_ready() {
        uart::puts("  [fs] no virtio-blk attached — BatFS is RAM-only this boot\n");
        return;
    }

    match batfs_disk::mount_or_format(master_key) {
        Ok(true) => {
            uart::puts("  [fs] disk was blank — formatted fresh BatFS layout\n");
            return;
        }
        Ok(false) => {
            uart::puts("  [fs] mounted existing BatFS from disk\n");
        }
        Err(e) => {
            uart::puts("  [fs] disk mount failed: ");
            uart::puts(e);
            uart::puts(" — running RAM-only this boot\n");
            return;
        }
    }

    // Disk mounted. Read the inode table; for each Active inode,
    // allocate contiguous RAM pages, copy the ciphertext from disk,
    // populate FILES[i].
    let mut inodes = [batfs_disk::DiskInode::empty(); batfs_disk::DISK_MAX_FILES];
    if let Err(e) = batfs_disk::read_all_inodes(&mut inodes) {
        uart::puts("  [fs] inode read failed: ");
        uart::puts(e);
        uart::puts(" — RAM-only fallback\n");
        return;
    }

    let mut restored: usize = 0;
    let mut count_buf = [0u8; 16];

    unsafe {
        for i in 0..MAX_FILES {
            let inode = &inodes[i];
            if inode.state != batfs_disk::DiskInode::STATE_ACTIVE { continue; }
            let size = inode.size as usize;
            if size == 0 || size > MAX_FILE_SIZE { continue; }

            // Allocate contiguous frames so the existing data_addr
            // (linear copy) layout still works.
            let pages = (size + BLOCK_SIZE - 1) / BLOCK_SIZE;
            let pages = if pages == 0 { 1 } else { pages };
            let data_addr = match crate::kernel::mm::frame::alloc_contig(pages) {
                Some(a) => a,
                None    => {
                    uart::puts("  [fs] OOM during disk restore — partial mount\n");
                    break;
                }
            };

            // Pull ciphertext from disk into the freshly-allocated pages.
            let dest_slice = core::slice::from_raw_parts_mut(
                data_addr as *mut u8, pages * BLOCK_SIZE);
            if batfs_disk::read_data(i, dest_slice, size).is_err() {
                uart::puts("  [fs] disk data read failed mid-restore\n");
                continue;
            }

            // Install the FILES[] entry. Filename + nonce + tag are taken
            // from the inode; AEAD verification happens lazily on read().
            FILES[i] = FileEntry::empty();
            let nl = (inode.name_len as usize).min(MAX_FILENAME);
            FILES[i].name[..nl].copy_from_slice(&inode.name[..nl]);
            FILES[i].name_len = nl;
            FILES[i].size = size;
            FILES[i].data_addr = data_addr;
            FILES[i].nonce = inode.nonce;
            // hash[..16] holds the Poly1305 tag; rest is unused for now.
            FILES[i].hash[..16].copy_from_slice(&inode.tag);
            FILES[i].encrypted = inode.encrypted != 0;
            FILES[i].state = FileState::Active;
            FILE_COUNT += 1;
            restored += 1;
        }
    }

    let n_written = write_dec(restored, &mut count_buf);
    uart::puts("  [fs] restored ");
    let s = unsafe { core::str::from_utf8_unchecked(&count_buf[..n_written]) };
    uart::puts(s);
    uart::puts(" file(s) from disk\n");

    rebuild_merkle();
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
    for i in 0..len { out[i] = tmp[p - 1 - i]; }
    len
}

fn next_nonce() -> [u8; 12] {
    // M4 / MMU-off: `fetch_add` lowers to LDXR/STXR which hangs on
    // Device-nGnRnE memory. Under IrqGuard on single-CPU bring-up the
    // load + store is exclusive. When SMP lands this needs either a
    // real lock or `+lse`. See docs/M4_GROUND_TRUTH.md §2.
    let n = {
        let _g = crate::kernel::sync::IrqGuard::new();
        let cur = NONCE_COUNTER.load(core::sync::atomic::Ordering::Acquire);
        NONCE_COUNTER.store(cur.wrapping_add(1), core::sync::atomic::Ordering::Release);
        cur
    };
    let mut nonce = [0u8; 12];
    let prefix = unsafe {
        core::ptr::read_volatile(core::ptr::addr_of!(BOOT_NONCE_PREFIX))
    };
    nonce[..4].copy_from_slice(&prefix);
    nonce[4..12].copy_from_slice(&n.to_be_bytes());
    nonce
}

fn derive_file_key(filename: &str) -> [u8; 32] {
    unsafe {
        let mut key = core::ptr::read_volatile(core::ptr::addr_of!(MASTER_KEY));
        let derived = sha256::derive_key(&key, filename.as_bytes());
        // V8-ROOT-6: zero the stack-local master-key copy so it doesn't
        // linger in the stack frame after return — a subsequent kernel
        // heap-walk / stack-unwind could recover it otherwise.
        crate::security::zeroize::zeroize(&mut key);
        derived
    }
}

/// FL-028 fix: HMAC-SHA256 over (filename || nonce || ciphertext) keyed by
/// the master key. Previously the integrity check was a plain SHA-256 of
/// the plaintext stored beside the ciphertext in the same static — any
/// kernel write primitive could update both and pass verification.
/// HMAC under the master key means the tag is only forgeable by someone
/// who holds the master key (i.e. the user with the passphrase).
// /
/// Computed incrementally so we don't need a 64 KB stack buffer.
fn compute_file_mac(name: &str, nonce: &[u8; 12], ciphertext: &[u8]) -> [u8; 32] {
    let mut key = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(MASTER_KEY)) };
    // Build i_key_pad / o_key_pad.
    let mut i_pad = [0x36u8; 64];
    let mut o_pad = [0x5cu8; 64];
    for i in 0..32 { i_pad[i] ^= key[i]; o_pad[i] ^= key[i]; }
    // V8-ROOT-6: zero the stack-local master-key copy ASAP now that we've
    // mixed it into the pads.
    crate::security::zeroize::zeroize(&mut key);

    // Inner hash: SHA-256(i_pad || "batfs-integrity-v1" || name || nonce || ciphertext)
    let mut inner = sha256::Sha256::new();
    inner.update(&i_pad);
    inner.update(b"batfs-integrity-v1");
    inner.update(name.as_bytes());
    inner.update(nonce);
    inner.update(ciphertext);
    let inner_digest = inner.finalize();

    let mut outer = sha256::Sha256::new();
    outer.update(&o_pad);
    outer.update(&inner_digest);
    outer.finalize()
}

/// Compose `<active-mount-prefix><name>` for a namespaced operation.
/// Returns the composed `&str` slice borrowing from `out`. When no
/// cave is active (kernel/admin context), returns `name` unchanged
/// — admin operates on the un-prefixed namespace.
///
/// Gap-audit item 032 (mount-namespace auto-application). The
/// existing un-prefixed `create` / `read` / `delete` / `list` stay
/// in place for kernel-administered files (audit.log, signed pkg
/// bundles, etc.); the `ns_*` wrappers below route cave-visible
/// operations through this composer.
fn ns_compose<'a>(name: &str, out: &'a mut [u8; MAX_FILENAME]) -> Result<&'a str, &'static str> {
    if name.is_empty() {
        return Err("filename empty");
    }
    let mut prefix_buf = [0u8; 80];
    let plen = crate::batcave::cave::active_mount_prefix(&mut prefix_buf);
    if plen + name.len() > MAX_FILENAME {
        return Err("filename too long");
    }
    out[..plen].copy_from_slice(&prefix_buf[..plen]);
    out[plen..plen + name.len()].copy_from_slice(name.as_bytes());
    Ok(unsafe { core::str::from_utf8_unchecked(&out[..plen + name.len()]) })
}

/// Pages a write of `data_len` bytes will occupy on disk. Matches
/// the rounding in `create`: at least one page even for empty
/// files.
fn pages_for(data_len: usize) -> u32 {
    let p = (data_len + BLOCK_SIZE - 1) / BLOCK_SIZE;
    if p == 0 { 1 } else { p as u32 }
}

/// Mount-namespace aware [`create`]. Prepends the active cave's
/// mount prefix to `name` before delegating; kernel/admin context
/// (no active cave) is identical to the un-prefixed `create`.
///
/// Gap-audit item 030 (memory-quota across allocators): the
/// data pages are charged against the active cave's quota BEFORE
/// the encryption/allocation work — quota-exceeded callers fail
/// fast without dragging frames through AEAD. On any downstream
/// error, the charge is released so the cave is not penalised
/// for a failed write.
pub fn ns_create(name: &str, data: &[u8]) -> Result<(), &'static str> {
    let pages = pages_for(data.len());
    crate::batcave::cave::active_charge_pages(pages)?;
    let mut full = [0u8; MAX_FILENAME];
    let full_name = match ns_compose(name, &mut full) {
        Ok(s) => s,
        Err(e) => {
            crate::batcave::cave::active_release_pages(pages);
            return Err(e);
        }
    };
    match create(full_name, data) {
        Ok(()) => Ok(()),
        Err(e) => {
            crate::batcave::cave::active_release_pages(pages);
            Err(e)
        }
    }
}

/// Mount-namespace aware [`read`].
pub fn ns_read(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full)?;
    read(full_name, buf)
}

/// Mount-namespace aware [`delete`]. Releases the same number of
/// quota pages the matching `ns_create` charged.
pub fn ns_delete(name: &str) -> Result<(), &'static str> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full)?;
    let pages = file_size(full_name).map(pages_for);
    delete(full_name)?;
    if let Some(p) = pages {
        crate::batcave::cave::active_release_pages(p);
    }
    Ok(())
}

/// Mount-namespace aware [`list`]. From inside a cave, callers see
/// only files whose on-disk name begins with the cave's prefix,
/// and the prefix is stripped before invoking the callback —
/// the cave never learns the on-disk naming scheme. From kernel/
/// admin context (no active cave), the entire BatFS namespace is
/// visible (same as the un-prefixed `list`).
pub fn ns_list<F: FnMut(&str, usize, bool)>(mut callback: F) {
    let mut prefix_buf = [0u8; 80];
    let plen = crate::batcave::cave::active_mount_prefix(&mut prefix_buf);
    if plen == 0 {
        list(callback);
        return;
    }
    let prefix = unsafe { core::str::from_utf8_unchecked(&prefix_buf[..plen]) };
    list(|name, size, enc| {
        if let Some(visible) = name.strip_prefix(prefix) {
            callback(visible, size, enc);
        }
        // else: belongs to another namespace — invisible to this cave.
    });
}

/// Mount-namespace aware [`stats`]. Returns `(visible_count, MAX_FILES)`
/// where `visible_count` is the number of files reachable in the
/// caller's mount namespace.
pub fn ns_stats() -> (usize, usize) {
    let mut prefix_buf = [0u8; 80];
    let plen = crate::batcave::cave::active_mount_prefix(&mut prefix_buf);
    if plen == 0 {
        return stats();
    }
    let prefix = unsafe { core::str::from_utf8_unchecked(&prefix_buf[..plen]) };
    let mut count = 0usize;
    list(|name, _, _| {
        if name.starts_with(prefix) {
            count += 1;
        }
    });
    (count, MAX_FILES)
}

/// Create a new file with the given name and plaintext content.
/// Content is encrypted with a per-file derived key.
pub fn create(name: &str, data: &[u8]) -> Result<(), &'static str> {
    if data.len() > MAX_FILE_SIZE {
        return Err("file too large");
    }
    if name.len() > MAX_FILENAME {
        return Err("filename too long");
    }

    unsafe {
        if !INITIALIZED {
            return Err("filesystem not initialized");
        }

        // Check for duplicate name
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                return Err("file exists");
            }
        }

        // Find free slot
        let slot = (0..MAX_FILES)
            .find(|&i| FILES[i].state == FileState::Free)
            .ok_or("filesystem full")?;

        // Allocate pages for data
        let pages_needed = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let pages_needed = if pages_needed == 0 { 1 } else { pages_needed };
        let data_addr = frame::alloc_frame().ok_or("out of memory")?;
        for _ in 1..pages_needed {
            frame::alloc_frame().ok_or("out of memory")?;
        }

        // DESIGN_CRYPTO.md #2: ChaCha20-Poly1305 AEAD.
        // * Key — sha256::derive_key(master, filename). Unchanged.
        // * Nonce — 12 bytes, file-unique (AEAD allows nonce reuse
        // only to break confidentiality, not integrity — still
        // fatal, so we use our monotonic next_nonce pattern).
        // * AAD — filename bytes. Binds ciphertext to its filename;
        // an attacker can't rename a file to an accessible slot
        // and reuse the ciphertext.
        // * Output — ciphertext (same length as plaintext) + 16-byte
        // Poly1305 authentication tag stored in entry.hash[..16].
        let file_key = derive_file_key(name);
        let cipher = ChaCha20Poly1305::new(&file_key.into());
        let nonce_full = next_nonce();
        // entry.nonce is 12 bytes already (ChaCha20-Poly1305 native size)
        let mut nonce: [u8; 12] = [0; 12];
        nonce.copy_from_slice(&nonce_full[..12]);

        // Copy plaintext into allocated memory, then encrypt in place.
        let dest = data_addr as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
        let slice_mut = core::slice::from_raw_parts_mut(dest, data.len());

        let tag = match cipher.encrypt_in_place_detached(
                (&nonce).into(), name.as_bytes(), slice_mut) {
            Ok(t) => t,
            Err(_) => return Err("encryption failed (AEAD)"),
        };

        // Store the 16-byte Poly1305 tag in the 32-byte hash field.
        // Remaining bytes left zero — future fields (e.g. Merkle
        // neighbor) can reuse the slack.
        let mut hash = [0u8; 32];
        hash[..16].copy_from_slice(&tag);

        let entry = &mut FILES[slot];
        entry.state = FileState::Active;
        entry.name[..name.len()].copy_from_slice(name.as_bytes());
        entry.name_len = name.len();
        entry.size = data.len();
        entry.data_addr = data_addr;
        entry.nonce = nonce;
        entry.hash = hash;
        entry.encrypted = true;

        FILE_COUNT += 1;

        // write the ciphertext + inode through to
        // disk. Order matters for crash consistency — data first, then
        // inode (the metadata write is the commit point). If we crash
        // between the two, the slot's old inode still references its
        // OLD ciphertext (which is intact in RAM/disk because we
        // allocated fresh frames and wrote into a different slot's
        // sectors), so the FS stays consistent.
        //
        // Inside the unsafe block so we can read FILES[slot] directly
        // and so `slot` is still in scope.
        if super::batfs_disk::is_mounted() {
            let entry_ro = &FILES[slot];
            let cipher_slice = core::slice::from_raw_parts(
                entry_ro.data_addr as *const u8, entry_ro.size);
            let _ = super::batfs_disk::write_data(slot, cipher_slice);

            let mut inode = super::batfs_disk::DiskInode::empty();
            inode.state = super::batfs_disk::DiskInode::STATE_ACTIVE;
            inode.encrypted = if entry_ro.encrypted { 1 } else { 0 };
            inode.name_len = entry_ro.name_len as u32;
            let nl = entry_ro.name_len.min(64);
            inode.name[..nl].copy_from_slice(&entry_ro.name[..nl]);
            inode.size = entry_ro.size as u64;
            inode.nonce = entry_ro.nonce;
            inode.tag.copy_from_slice(&entry_ro.hash[..16]);
            let _ = super::batfs_disk::write_inode(slot, &inode);
            let _ = super::batfs_disk::flush();
        }
    }

    // Update Merkle tree
    rebuild_merkle();

    Ok(())
}

/// Read a file — decrypts and verifies integrity.
/// Returns a buffer with plaintext content.
pub fn read(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    unsafe {
        let entry = find_file(name)?;

        if entry.size > buf.len() {
            return Err("buffer too small");
        }

        // Copy ciphertext to output buffer (decrypt in place).
        let src = entry.data_addr as *const u8;
        core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), entry.size);

        // DESIGN_CRYPTO.md #2: ChaCha20-Poly1305 AEAD decrypt.
        // AAD is the filename (binds ciphertext to file slot).
        // Tag is the first 16 bytes of entry.hash (stored at write).
        // decrypt_in_place_detached returns Err on tag mismatch;
        // Poly1305 is a constant-time MAC so no timing leak.
        let file_key = derive_file_key(name);
        let cipher = ChaCha20Poly1305::new(&file_key.into());
        let tag_bytes: [u8; 16] = match entry.hash[..16].try_into() {
            Ok(b) => b,
            Err(_) => return Err("internal: tag slice"),
        };
        let tag: &chacha20poly1305::Tag = (&tag_bytes).into();
        cipher.decrypt_in_place_detached(
                (&entry.nonce).into(),
                name.as_bytes(),
                &mut buf[..entry.size],
                tag,
            ).map_err(|_| "INTEGRITY VIOLATION — file tampered")?;

        Ok(entry.size)
    }
}

/// Delete a file — zeroes the encrypted data before freeing.
pub fn delete(name: &str) -> Result<(), &'static str> {
    // find the slot index up front so we can target the
    // disk wipe at exactly that slot's sector range.
    let slot = unsafe {
        let mut found: Option<usize> = None;
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                found = Some(i); break;
            }
        }
        match found {
            Some(s) => s,
            None    => return Err("file not found"),
        }
    };

    unsafe {
        let entry = &mut FILES[slot];

        // Zero the encrypted data (secure delete)
        let ptr = entry.data_addr as *mut u8;
        let pages = (entry.size + BLOCK_SIZE - 1) / BLOCK_SIZE;
        let pages = if pages == 0 { 1 } else { pages };
        core::ptr::write_bytes(ptr, 0, pages * BLOCK_SIZE);

        // Free pages
        for i in 0..pages {
            frame::free_frame(entry.data_addr + i * BLOCK_SIZE);
        }

        entry.state = FileState::Deleted;
        FILE_COUNT -= 1;
    }

    // wipe the slot's data sectors and clear the
    // inode on disk. Order: data sectors first (so a crash mid-delete
    // leaves the inode pointing at zeroed ciphertext, which AEAD will
    // reject anyway), inode commit second.
    if super::batfs_disk::is_mounted() {
        let _ = super::batfs_disk::zero_data(slot);
        let _ = super::batfs_disk::free_inode(slot);
        let _ = super::batfs_disk::flush();
    }

    // Update Merkle tree
    rebuild_merkle();
    Ok(())
}

/// List all active files. Calls the provided closure for each.
pub fn list<F: FnMut(&str, usize, bool)>(mut callback: F) {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active {
                callback(FILES[i].name_str(), FILES[i].size, FILES[i].encrypted);
            }
        }
    }
}

/// Get filesystem stats.
pub fn stats() -> (usize, usize) {
    unsafe { (FILE_COUNT, MAX_FILES) }
}

/// Look up a file's plaintext size without decrypting. Returns
/// `None` if no active file with that name exists. Used by the
/// `ns_delete` wrapper to know how many quota pages to release.
pub fn file_size(name: &str) -> Option<usize> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                return Some(FILES[i].size);
            }
        }
    }
    None
}

/// V8-ROOT-6: panic-handler-only master-key wipe. Uses volatile writes so
/// the compiler cannot DCE. No locks. Best-effort: if we panic mid-write
/// the first N bytes are still zeroed, which already degrades an
/// attacker's recovered key.
// /
/// # Safety
/// May only be called from the panic handler (via wipe::emergency_wipe).
/// After this runs BatFS read/write WILL fail; the kernel is halting.
pub unsafe fn panic_wipe() {
    let key_ptr = core::ptr::addr_of_mut!(MASTER_KEY) as *mut u8;
    for i in 0..32 {
        unsafe { core::ptr::write_volatile(key_ptr.add(i), 0); }
    }
}

fn find_file(name: &str) -> Result<&'static FileEntry, &'static str> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                return Ok(&FILES[i]);
            }
        }
        Err("file not found")
    }
}

fn find_file_mut(name: &str) -> Result<&'static mut FileEntry, &'static str> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                return Ok(&mut FILES[i]);
            }
        }
        Err("file not found")
    }
}
