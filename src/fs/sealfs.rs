#![allow(dead_code)]
// Sphragis — SealFS: Custom Encrypted Filesystem
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
// AUDIT-FS-C4 elite-tier (2026-05-16): migrate SealFS from
// ChaCha20-Poly1305 to AES-256-GCM-SIV. Same 32-byte key, same
// 12-byte nonce, same 16-byte tag — drop-in API replacement.
// GCM-SIV is misuse-resistant: if a nonce IS reused (operator
// bug, RAM corruption flipping the stored prefix, etc.),
// authenticity holds and only the two reused-nonce plaintexts
// leak (not the keystream). ChaCha20-Poly1305 catastrophically
// failed under nonce reuse — same-key+nonce twice → Poly1305 key
// recovery → forgery. The Week-2 FS-C4 fix bound the nonce into
// AAD as a stopgap; GCM-SIV makes the underlying primitive
// inherently safer.
//
// Pre-FS-C4-Week-8 ciphertexts (still ChaCha-Poly1305) will fail
// AEAD verify on read. Acceptable pre-production. Real-deployment
// migration would need a version byte in the inode + dual-AEAD
// decrypt path; for now the disk is non-persistent across builds.
use aes_gcm_siv::{
    aead::{AeadInPlace, KeyInit},
    Aes256GcmSiv,
};
// Type alias to minimize the diff against the prior code shape.
type Cipher = Aes256GcmSiv;
// Tag type re-export so the existing `&Cipher::Tag` casts work.
type AeadTag = aes_gcm_siv::Tag;

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
    /// MLS sensitivity label (gov-grade §3.2). Stamped at ns_create
    /// time from the active cave's `cave.sensitivity`. Bell-LaPadula
    /// `ns_read` rejects when `cave.sens < file.sens` (no read-up).
    /// 0 = Unclassified, 1 = Confidential, 2 = Secret, 3 = TopSecret.
    /// Files created via the un-prefixed admin path inherit 0.
    pub sensitivity: u8,
    /// Biba integrity label (gov-grade §3.2 dual lattice). Stamped
    /// at ns_create from `cave.integrity`. `ns_read` rejects when
    /// `cave.integ > file.integ` (no read-DOWN — a high-integrity
    /// subject must not be tainted by low-integrity input).
    pub integrity: u8,
    /// SELinux-style object type (gov-grade §3.2 TE-on-objects
    /// slice). Free-form 1-byte tag the operator assigns to
    /// distinguish file classes (e.g. `system_config`,
    /// `user_data`, `audit_log`, `crypto_material`, `tmp`).
    /// Default = 0 = `untyped`. Cross-cave reads are gated by
    /// `cave::can_read_object_type(cave_id, obj_type)` from
    /// `ns_read`.
    pub obj_type: u8,
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
            sensitivity: 0,
            integrity: 0,
            obj_type: 0,
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

/// Per-file taint bitmap (gov-grade §3.2 information-flow slice).
/// Parallel to FILES — indexed the same way. Each file carries 32
/// orthogonal taint sources; reads/writes propagate them between
/// caves and files. See `cave.rs` for the propagation model.
static mut FILE_TAINT: [u32; MAX_FILES] = [0u32; MAX_FILES];

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
// persistent-across-reboot path ( Phase 7 / `sealfs_disk.rs`)
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

/// HMAC-SHA256 of the Merkle root, keyed by the SealFS master key.
/// Updated by `seal_merkle_root()` on every create/delete (the same
/// points that call `rebuild_merkle`). Used by `verify_all_integrity`
/// as the externally-anchored expected value against which the
/// recomputed root is checked.
///
/// AUDIT-FS-C3 (2026-05-15): the prior `verify_all_integrity` was
/// tautological — it compared the recomputed Merkle root against
/// the same value it had just computed. Storing an HMAC keyed by
/// the master key means a tamperer must ALSO know the master key
/// (or have kernel-write to overwrite MERKLE_HMAC) to forge a
/// matching post-tamper value. In-RAM today; SEP-sealed export is
/// the follow-up that makes this resilient to kernel-write
/// tamperers.
static mut MERKLE_HMAC: [u8; 32] = [0u8; 32];
static mut MERKLE_HMAC_VALID: bool = false;

/// Recompute MERKLE_HMAC over the current Merkle root. Called from
/// every code path that calls `rebuild_merkle` (create, delete,
/// declassify, init_disk restore).
fn seal_merkle_root() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let key = core::ptr::read_volatile(core::ptr::addr_of!(MASTER_KEY));
        let root = (*core::ptr::addr_of!(MERKLE_TREE))[1];
        let hmac = sha256::hmac(&key, &root);
        *core::ptr::addr_of_mut!(MERKLE_HMAC) = hmac;
        core::ptr::write_volatile(core::ptr::addr_of_mut!(MERKLE_HMAC_VALID), true);
        // Zero the stack-local master-key copy so it doesn't linger.
        let mut k = key;
        crate::security::zeroize::zeroize(&mut k);
    }
}

/// Verify the entire filesystem integrity.
///
/// AUDIT-FS-C3 (2026-05-15): replaced the prior tautology
/// (recompute root → compare to recomputed root) with HMAC-based
/// verification. Recomputes the Merkle root from current FILES[],
/// then HMACs it with the master key, then constant-time-compares
/// against MERKLE_HMAC (which was sealed at last create/delete).
/// Detects:
/// * Off-line disk tampering (an attacker who edits inode/data
///   sectors without knowing the master key cannot forge a
///   matching HMAC).
/// * In-RAM tampering that doesn't reach MERKLE_HMAC (e.g. via
///   a write-primitive that only writes FILES[]).
/// Does NOT detect:
/// * An attacker with kernel-write who overwrites both FILES[]
///   AND MERKLE_HMAC. Mitigation: SEP-seal the HMAC. Follow-up
///   wave once sep.rs lands.
pub fn verify_all_integrity() -> bool {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        if !core::ptr::read_volatile(core::ptr::addr_of!(MERKLE_HMAC_VALID)) {
            // Never sealed (fresh boot before any write). Trivially OK.
            return true;
        }
        rebuild_merkle();
        let key = core::ptr::read_volatile(core::ptr::addr_of!(MASTER_KEY));
        let root = (*core::ptr::addr_of!(MERKLE_TREE))[1];
        let expected_hmac = sha256::hmac(&key, &root);
        let stored = *core::ptr::addr_of!(MERKLE_HMAC);
        // Zero the stack-local master-key copy.
        let mut k = key;
        crate::security::zeroize::zeroize(&mut k);
        // Constant-time compare.
        let mut diff: u8 = 0;
        for i in 0..32 {
            diff |= expected_hmac[i] ^ stored[i];
        }
        diff == 0
    }
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

    // try to mount/format the on-disk SealFS. Outside the
    // critical_section above because virtio-blk uses MMIO + DMA + IRQ
    // and shouldn't run with IRQs masked the whole time.
    init_disk(master_key);
}

/// mount the on-disk SealFS layout, or format it
/// fresh if the disk is blank / a different layout. Restores the inode
/// table + per-file ciphertext into `FILES[]` and RAM pages.
fn init_disk(master_key: &[u8; 32]) {
    use crate::drivers::uart;
    use super::sealfs_disk;

    if !crate::drivers::virtio::blk::is_ready() {
        uart::puts("  [fs] no virtio-blk attached — SealFS is RAM-only this boot\n");
        return;
    }

    match sealfs_disk::mount_or_format(master_key) {
        Ok(true) => {
            uart::puts("  [fs] disk was blank — formatted fresh SealFS layout\n");
            return;
        }
        Ok(false) => {
            uart::puts("  [fs] mounted existing SealFS from disk\n");
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
    let mut inodes = [sealfs_disk::DiskInode::empty(); sealfs_disk::DISK_MAX_FILES];
    if let Err(e) = sealfs_disk::read_all_inodes(&mut inodes) {
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
            if inode.state != sealfs_disk::DiskInode::STATE_ACTIVE { continue; }
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
            if sealfs_disk::read_data(i, dest_slice, size).is_err() {
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
    // AUDIT-FS-C3: seal the restored state's root under the master
    // key so subsequent verify_all_integrity() can detect post-mount
    // tampering.
    seal_merkle_root();
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

    // Inner hash: SHA-256(i_pad || "sealfs-integrity-v2" || name || nonce || ciphertext)
    // Version bumped from v1 to v2 alongside the 2026-05-17 byte-
    // constant rename (BATFS magic + HMAC + KDF salts all rolled
    // forward in one breaking change while still pre-production).
    // Any file MAC'd under "batfs-integrity-v1" will fail to verify
    // — that's the desired behavior, since the disk magic also
    // changed and a pre-rename image won't even mount.
    let mut inner = sha256::Sha256::new();
    inner.update(&i_pad);
    inner.update(b"sealfs-integrity-v2");
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
    let plen = crate::caves::cave::active_mount_prefix(&mut prefix_buf);
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
    crate::caves::cave::active_charge_pages(pages)?;
    let mut full = [0u8; MAX_FILENAME];
    let full_name = match ns_compose(name, &mut full) {
        Ok(s) => s,
        Err(e) => {
            crate::caves::cave::active_release_pages(pages);
            return Err(e);
        }
    };
    match create(full_name, data) {
        Ok(()) => {
            // Information-flow propagation on write: the newly-
            // created file inherits every taint bit the writer
            // cave currently carries. Admin / kernel context
            // contributes 0 — same passive shape as the rest of
            // the cave-context-aware paths.
            let cave_t = crate::caves::cave::active_taint();
            if cave_t != 0 {
                add_file_taint(full_name, cave_t);
            }
            Ok(())
        }
        Err(e) => {
            crate::caves::cave::active_release_pages(pages);
            Err(e)
        }
    }
}

/// Mount-namespace aware [`read`].
///
/// Enforces TWO orthogonal MLS rules (gov-grade §3.2):
///   * Bell-LaPadula simple security property (no read-up):
///     `cave.sens >= file.sens`. Else `Err("mls: no read-up")`.
///   * Biba simple-integrity property (no read-DOWN):
///     `cave.integ <= file.integ`. Else `Err("mls: no read-down")`.
///     A high-integrity cave refuses to read low-integrity input
///     because that would taint its own outputs.
///
/// Both must pass. Admin/kernel context defaults to
/// Unclassified/Untrusted: it can read Unclassified files (passes
/// both rules) and Untrusted files (passes both rules) but is
/// fenced out of Confidential+ AND SystemTrusted+ until a cave
/// is attached with the right labels.
pub fn ns_read(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full)?;
    if let Some((file_sens_u8, file_integ_u8)) = file_labels(full_name) {
        use crate::caves::cave::{self, MlsOp, Sensitivity, Integrity};
        let cave_sens = cave::active_sensitivity();
        let file_sens = Sensitivity::from_u8(file_sens_u8);
        if !cave::can_flow(cave_sens, file_sens, MlsOp::Read) {
            return Err("mls: no read-up");
        }
        let cave_integ = cave::active_integrity();
        let file_integ = Integrity::from_u8(file_integ_u8);
        if !cave::can_flow_integrity(cave_integ, file_integ, MlsOp::Read) {
            return Err("mls: no read-down");
        }
    }
    // SELinux-style TE on objects (gov-grade §3.2 TE slice).
    // Consult the per-cave object-type DENY matrix. Admin /
    // kernel context always passes (cave_id == u16::MAX); cave
    // context fails fast with the policy-specific error string.
    if let Some(obj_t) = obj_type_for_full_name(full_name) {
        use crate::caves::cave::{self, ObjOp};
        let active = cave::get_active();
        let active_id = if active == usize::MAX { u16::MAX } else { active as u16 };
        if !cave::can_access_object(active_id, obj_t, ObjOp::Read) {
            return Err("te-obj: read denied by policy");
        }
    }
    let result = read(full_name, buf);
    // Information-flow propagation: on successful read the active
    // cave inherits every taint bit the file carried. No-op when
    // file_taint is 0 (the common case) so the hot path stays
    // cheap. Admin / kernel context (no active cave) silently
    // discards the propagation — same shape as the MLS/Biba
    // checks above.
    if result.is_ok() {
        let t = file_taint(full_name);
        if t != 0 {
            crate::caves::cave::active_add_taint(t);
        }
    }
    result
}

/// Look up the taint bitmap for `full_name` (already mount-prefix
/// composed). Returns 0 for unknown files — fail-open at lookup
/// time matches SealFS's "no entry = no policy" shape used by
/// labels and object types.
pub fn file_taint(full_name: &str) -> u32 {
    // AUDIT-FS-C1 (2026-05-15): the prior F3 spinlock + RAII guard
    // around every FILES/FILE_TAINT/FILE_COUNT-touching function
    // regressed somewhere in the Wave 1-8 churn. Without it, a timer
    // IRQ can interleave a delete (mark Deleted + free frames + zero
    // pages) with this scan, dereferencing a freed-and-reallocated
    // entry. Restore the prior pattern with IrqGuard (single-CPU
    // critical section). SMP retrofit will need a real SpinLock.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                return FILE_TAINT[i];
            }
        }
    }
    0
}

/// Set the taint bitmap on `full_name`. Used by the
/// `taint-stamp` admin path so an operator can mark a file as
/// PII / compliance-restricted / etc. before it ever gets read.
pub fn set_file_taint(full_name: &str, bits: u32) -> Result<(), &'static str> {
    // AUDIT-FS-C1: see file_taint() for rationale.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                FILE_TAINT[i] = bits;
                return Ok(());
            }
        }
    }
    Err("taint: no such file")
}

/// OR `bits` into `full_name`'s existing taint. Idempotent —
/// repeated calls with the same bits leave the bitmap unchanged.
/// Used by `ns_create` to inherit the writer cave's taint.
pub fn add_file_taint(full_name: &str, bits: u32) {
    // AUDIT-FS-C1: see file_taint() for rationale.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                FILE_TAINT[i] |= bits;
                return;
            }
        }
    }
}

/// Zero the taint of every file. Used by selftests + the
/// `taint-reset-all` admin operation.
pub fn clear_all_file_taints() {
    // AUDIT-FS-C1: see file_taint() for rationale.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let ptr = core::ptr::addr_of_mut!(FILE_TAINT);
        for i in 0..MAX_FILES {
            (*ptr)[i] = 0;
        }
    }
}

/// Like `obj_type_of` but takes a fully-composed (already prefixed)
/// name — saves the double mount-namespace prefix when ns_read has
/// already done it.
fn obj_type_for_full_name(full_name: &str) -> Option<u8> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                return Some(FILES[i].obj_type);
            }
        }
    }
    None
}

/// Trusted-subject declassification (gov-grade §3.23 MLS
/// downgrade path). Re-stamps a file with new sens + integ
/// labels AND re-runs AEAD encryption so the new labels are
/// AAD-bound. Use only from privileged code that's already
/// passed a TPI quorum check — the kernel doesn't enforce that
/// here, the shell command wrapper does.
///
/// Returns Ok with the new (sens, integ) bytes on success.
pub fn declassify(name: &str, new_sens: u8, new_integ: u8) -> Result<(u8, u8), &'static str> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full)?;
    // Decrypt under current labels, re-encrypt under new ones.
    // Doing this in two passes via `read` + `delete` + `create`
    // would lose the AEAD-bound metadata; instead poke the
    // FileEntry directly and recompute the tag.
    let mut buf = [0u8; MAX_FILE_SIZE];
    let plain_len = {
        let mut found = None;
        unsafe {
            for i in 0..MAX_FILES {
                if FILES[i].state == FileState::Active && FILES[i].name_str() == full_name {
                    found = Some(i);
                    break;
                }
            }
        }
        let slot = match found {
            Some(s) => s,
            None => return Err("declassify: file not found"),
        };
        // Decrypt with current labels into local buffer.
        let entry = unsafe { &FILES[slot] };
        let src = entry.data_addr as *const u8;
        unsafe { core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), entry.size); }
        let file_key = derive_file_key(full_name);
        let cipher = Cipher::new(&file_key.into());
        let mut tag_bytes = [0u8; 16];
        tag_bytes.copy_from_slice(&entry.hash[..16]);
        let tag: &AeadTag = (&tag_bytes).into();
        // AUDIT-FS-C4 (2026-05-15): nonce bound into AAD for both
        // the decrypt-under-old-labels and encrypt-under-new-labels
        // legs. See create() comment for rationale.
        let mut aad_old = [0u8; MAX_FILENAME + 2 + 12];
        aad_old[..full_name.len()].copy_from_slice(full_name.as_bytes());
        aad_old[full_name.len()]     = entry.sensitivity;
        aad_old[full_name.len() + 1] = entry.integrity;
        aad_old[full_name.len() + 2 .. full_name.len() + 14]
            .copy_from_slice(&entry.nonce);
        cipher.decrypt_in_place_detached(
            (&entry.nonce).into(),
            &aad_old[..full_name.len() + 14],
            &mut buf[..entry.size],
            tag,
        ).map_err(|_| "declassify: current AEAD verify failed")?;
        let plain_len = entry.size;

        // Re-encrypt with new labels under a fresh nonce.
        let new_nonce_full = next_nonce();
        let mut new_nonce = [0u8; 12];
        new_nonce.copy_from_slice(&new_nonce_full[..12]);
        let mut aad_new = [0u8; MAX_FILENAME + 2 + 12];
        aad_new[..full_name.len()].copy_from_slice(full_name.as_bytes());
        aad_new[full_name.len()]     = new_sens;
        aad_new[full_name.len() + 1] = new_integ;
        aad_new[full_name.len() + 2 .. full_name.len() + 14]
            .copy_from_slice(&new_nonce);
        let new_tag = cipher.encrypt_in_place_detached(
            (&new_nonce).into(),
            &aad_new[..full_name.len() + 14],
            &mut buf[..plain_len],
        ).map_err(|_| "declassify: re-encrypt failed")?;

        // Commit: write ciphertext back to its frame, update
        // metadata fields.
        unsafe {
            let dst = entry.data_addr as *mut u8;
            core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, plain_len);
            let e = &mut FILES[slot];
            e.nonce = new_nonce;
            let mut hash = [0u8; 32];
            hash[..16].copy_from_slice(&new_tag);
            e.hash = hash;
            e.sensitivity = new_sens;
            e.integrity   = new_integ;
        }
        plain_len
    };
    rebuild_merkle();
    seal_merkle_root();  // AUDIT-FS-C3
    let _ = plain_len; // silence unused-binding warning in release builds
    Ok((new_sens, new_integ))
}

/// Retag a file's SELinux-style object type. Doesn't re-encrypt
/// (type isn't bound into the AEAD's AAD today — only sens +
/// integ are). Returns `false` if no matching active file exists.
/// Admin operation: caller is expected to be in admin context.
pub fn set_obj_type(name: &str, obj_type: u8) -> bool {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = match ns_compose(name, &mut full) {
        Ok(s) => s,
        Err(_) => return false,
    };
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                FILES[i].obj_type = obj_type;
                return true;
            }
        }
    }
    false
}

/// Look up a file's stored object type. None if no matching file.
pub fn obj_type_of(name: &str) -> Option<u8> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full).ok()?;
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active
                && FILES[i].name_str() == full_name
            {
                return Some(FILES[i].obj_type);
            }
        }
    }
    None
}

/// Test-only: flip the on-disk MLS labels of a file without
/// re-running the AEAD. Used by `mls-binding-selftest` to prove
/// that label tampering causes `ns_read` to fail with the AEAD
/// integrity error rather than silently honour the downgraded
/// label. SAFETY: caller must not race with `create`/`read` on
/// the same file (cooperative single-CPU makes this trivial in
/// selftest paths).
#[allow(dead_code)]
pub unsafe fn tamper_test_flip_labels(name: &str, new_sens: u8, new_integ: u8) -> bool {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                FILES[i].sensitivity = new_sens;
                FILES[i].integrity   = new_integ;
                return true;
            }
        }
    }
    false
}

/// Look up a file's stored MLS labels by name. Returns `None` if
/// no active file matches. Returns `(sensitivity, integrity)`.
fn file_labels(name: &str) -> Option<(u8, u8)> {
    unsafe {
        for i in 0..MAX_FILES {
            if FILES[i].state == FileState::Active && FILES[i].name_str() == name {
                return Some((FILES[i].sensitivity, FILES[i].integrity));
            }
        }
    }
    None
}

/// Mount-namespace aware [`delete`]. Releases the same number of
/// quota pages the matching `ns_create` charged.
pub fn ns_delete(name: &str) -> Result<(), &'static str> {
    let mut full = [0u8; MAX_FILENAME];
    let full_name = ns_compose(name, &mut full)?;
    let pages = file_size(full_name).map(pages_for);
    delete(full_name)?;
    if let Some(p) = pages {
        crate::caves::cave::active_release_pages(p);
    }
    Ok(())
}

/// Mount-namespace aware [`list`]. From inside a cave, callers see
/// only files whose on-disk name begins with the cave's prefix,
/// and the prefix is stripped before invoking the callback —
/// the cave never learns the on-disk naming scheme. From kernel/
/// admin context (no active cave), the entire SealFS namespace is
/// visible (same as the un-prefixed `list`).
pub fn ns_list<F: FnMut(&str, usize, bool)>(mut callback: F) {
    let mut prefix_buf = [0u8; 80];
    let plen = crate::caves::cave::active_mount_prefix(&mut prefix_buf);
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
    let plen = crate::caves::cave::active_mount_prefix(&mut prefix_buf);
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
    // SP-B1.6.2 (2026-05-16): policy gate. SealFS uses AES-256-GCM-SIV
    // which IS on the CNSA 2.0 allowlist — so this gate is a for-the-
    // record assertion under gov-strict (always succeeds) but creates
    // a structural failure point if someone ever swaps SealFS to a
    // non-allowlisted AEAD primitive. Community build unaffected.
    crate::crypto::policy::ensure_permitted(
        crate::crypto::policy::Algo::Aes256GcmSiv,
    )?;

    if data.len() > MAX_FILE_SIZE {
        return Err("file too large");
    }
    if name.len() > MAX_FILENAME {
        return Err("filename too long");
    }

    // AUDIT-FS-C1 (2026-05-15): prior F3 fix is regressed; restore the
    // IrqGuard-based critical section around the multi-step state
    // transition. Without this, a timer IRQ between slot-find,
    // frame::alloc_frame, AEAD encrypt, FILES[slot] mutation, and
    // FILE_COUNT bump can interleave with a concurrent delete and
    // resurrect the UAF/double-free race the prior fix closed.
    // Note: holds across disk I/O for write_data/write_inode/flush —
    // sealfs_disk uses MMIO-poll completion (no IRQ wait), so this is
    // safe on single-CPU. SMP retrofit needs a real SpinLock.
    let _g = crate::kernel::sync::IrqGuard::new();
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
        // * AAD — filename bytes || MLS sensitivity || MLS integrity.
        // The filename binding (already present pre-MLS) prevents an
        // attacker who can swap data_addr from reusing ciphertext
        // under a different name. The MLS label binding (gov-grade
        // §3.2 hardening, 2026-05-13) makes the in-memory
        // `entry.sensitivity` and `entry.integrity` bytes
        // cryptographically load-bearing: a byte-flip on either
        // field at rest causes the AAD at decrypt time to differ
        // from encrypt time, so AEAD verification fails. Without
        // this binding, an attacker who can corrupt RAM could
        // downgrade a file's label to bypass `ns_read`'s
        // Bell-LaPadula / Biba checks.
        // * Output — ciphertext (same length as plaintext) + 16-byte
        // Poly1305 authentication tag stored in entry.hash[..16].
        let file_key = derive_file_key(name);
        let cipher = Cipher::new(&file_key.into());
        let nonce_full = next_nonce();
        // entry.nonce is 12 bytes already (ChaCha20-Poly1305 native size)
        let mut nonce: [u8; 12] = [0; 12];
        nonce.copy_from_slice(&nonce_full[..12]);

        // Snapshot the labels the file is about to inherit; they
        // become both the on-disk metadata AND part of the AAD
        // bound into the ciphertext.
        let sens_byte  = crate::caves::cave::active_sensitivity() as u8;
        let integ_byte = crate::caves::cave::active_integrity()   as u8;
        // AUDIT-FS-C4 (2026-05-15): bind the per-file nonce into AAD.
        // Prior AAD = filename || sens || integ. Without the nonce in
        // AAD, an attacker who can corrupt the inode's stored nonce
        // (without knowing the master key) could force same-key /
        // same-nonce reuse against an older ciphertext with the same
        // filename — Poly1305 key-recovery attack. With the nonce in
        // AAD, changing the stored nonce changes the AAD at decrypt
        // time and the AEAD tag check fails.
        // New AAD = filename || sens || integ || nonce(12). 15 bytes
        // tail on top of filename. Breaks compat with pre-FS-C4
        // ciphertexts — acceptable pre-production.
        let mut aad = [0u8; MAX_FILENAME + 2 + 12];
        aad[..name.len()].copy_from_slice(name.as_bytes());
        aad[name.len()]     = sens_byte;
        aad[name.len() + 1] = integ_byte;
        aad[name.len() + 2 .. name.len() + 14].copy_from_slice(&nonce);
        let aad_slice = &aad[..name.len() + 14];

        // Copy plaintext into allocated memory, then encrypt in place.
        let dest = data_addr as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
        let slice_mut = core::slice::from_raw_parts_mut(dest, data.len());

        let tag = match cipher.encrypt_in_place_detached(
                (&nonce).into(), aad_slice, slice_mut) {
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
        // MLS stamp (§3.2): bake the same labels we just AAD-bound
        // into the ciphertext into the on-disk entry. `ns_read`
        // checks these against the active cave's labels (BLP +
        // Biba); the AEAD then re-verifies they haven't been
        // tampered since write.
        entry.sensitivity = sens_byte;
        entry.integrity   = integ_byte;
        // Object-type defaults to "untyped" (0). Operators retag
        // via the future `sealfs::ns_create_typed` API + the
        // `te-obj-tag` shell command.
        entry.obj_type = 0;

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
        if super::sealfs_disk::is_mounted() {
            let entry_ro = &FILES[slot];
            let cipher_slice = core::slice::from_raw_parts(
                entry_ro.data_addr as *const u8, entry_ro.size);
            let _ = super::sealfs_disk::write_data(slot, cipher_slice);

            let mut inode = super::sealfs_disk::DiskInode::empty();
            inode.state = super::sealfs_disk::DiskInode::STATE_ACTIVE;
            inode.encrypted = if entry_ro.encrypted { 1 } else { 0 };
            inode.name_len = entry_ro.name_len as u32;
            let nl = entry_ro.name_len.min(64);
            inode.name[..nl].copy_from_slice(&entry_ro.name[..nl]);
            inode.size = entry_ro.size as u64;
            inode.nonce = entry_ro.nonce;
            inode.tag.copy_from_slice(&entry_ro.hash[..16]);
            let _ = super::sealfs_disk::write_inode(slot, &inode);
            let _ = super::sealfs_disk::flush();
        }
    }

    // Update Merkle tree
    rebuild_merkle();
    seal_merkle_root();  // AUDIT-FS-C3

    // SP-AUD-003 emit-site (2026-05-16): record file-mutation events.
    // Skip the audit-internal paths (audit/worm/*, audit.log,
    // audit-binary.log) to avoid recursion via audit_worm::worm_append
    // → sealfs::create → audit::record → audit_worm::worm_append.
    if !name.starts_with("audit/") && name != "audit.log" && name != "audit-binary.log" {
        let mut msg = [0u8; 192];
        let prefix = b"FileAccess: create ";
        let nlen = name.len().min(192 - prefix.len());
        msg[..prefix.len()].copy_from_slice(prefix);
        msg[prefix.len()..prefix.len() + nlen].copy_from_slice(&name.as_bytes()[..nlen]);
        crate::security::audit::record(
            crate::security::audit::Category::FileAccess,
            &msg[..prefix.len() + nlen],
        );
    }

    Ok(())
}

/// Read a file — decrypts and verifies integrity.
/// Returns a buffer with plaintext content.
pub fn read(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    // AUDIT-FS-C1: see create() for rationale.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let entry = find_file(name)?;

        if entry.size > buf.len() {
            return Err("buffer too small");
        }

        // Copy ciphertext to output buffer (decrypt in place).
        let src = entry.data_addr as *const u8;
        core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), entry.size);

        // DESIGN_CRYPTO.md #2: ChaCha20-Poly1305 AEAD decrypt.
        // AAD layout matches `create` byte-for-byte:
        //   filename || sensitivity_byte || integrity_byte
        // The labels come from the live entry — if an attacker has
        // flipped either byte at rest, the AAD here differs from
        // the encrypt-time AAD and decrypt_in_place_detached
        // returns Err. The MLS labels are therefore
        // cryptographically bound: tamper-and-bypass requires the
        // file key (which would also let them just decrypt).
        let file_key = derive_file_key(name);
        let cipher = Cipher::new(&file_key.into());
        let tag_bytes: [u8; 16] = match entry.hash[..16].try_into() {
            Ok(b) => b,
            Err(_) => return Err("internal: tag slice"),
        };
        let tag: &AeadTag = (&tag_bytes).into();
        // AUDIT-FS-C4 (2026-05-15): nonce bound into AAD, must
        // mirror the create-side construction. See create() comment.
        let mut aad = [0u8; MAX_FILENAME + 2 + 12];
        aad[..name.len()].copy_from_slice(name.as_bytes());
        aad[name.len()]     = entry.sensitivity;
        aad[name.len() + 1] = entry.integrity;
        aad[name.len() + 2 .. name.len() + 14].copy_from_slice(&entry.nonce);
        let aad_slice = &aad[..name.len() + 14];
        cipher.decrypt_in_place_detached(
                (&entry.nonce).into(),
                aad_slice,
                &mut buf[..entry.size],
                tag,
            ).map_err(|_| "INTEGRITY VIOLATION — file tampered or label flipped")?;

        Ok(entry.size)
    }
}

/// Delete a file — zeroes the encrypted data before freeing.
pub fn delete(name: &str) -> Result<(), &'static str> {
    // AUDIT-FS-C1: delete is the primary half of the prior F3
    // UAF race — slot transitions Active → Deleted while data_addr
    // is still readable by a concurrent reader that captured it.
    // Hold IrqGuard across the entire mark + zero + free sequence so
    // no IRQ-driven reader can observe the half-deleted state.
    let _g = crate::kernel::sync::IrqGuard::new();
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
        // Clear the slot's taint bitmap so a future file that
        // reuses this slot starts at 0 instead of inheriting the
        // deleted file's taint set.
        FILE_TAINT[slot] = 0;
    }

    // wipe the slot's data sectors and clear the
    // inode on disk. Order: data sectors first (so a crash mid-delete
    // leaves the inode pointing at zeroed ciphertext, which AEAD will
    // reject anyway), inode commit second.
    if super::sealfs_disk::is_mounted() {
        let _ = super::sealfs_disk::zero_data(slot);
        let _ = super::sealfs_disk::free_inode(slot);
        let _ = super::sealfs_disk::flush();
    }

    // Update Merkle tree
    rebuild_merkle();
    seal_merkle_root();  // AUDIT-FS-C3

    // SP-AUD-003 emit-site (2026-05-16): record file deletion. Same
    // audit-internal-path filter as create() to avoid recursion.
    if !name.starts_with("audit/") && name != "audit.log" && name != "audit-binary.log" {
        let mut msg = [0u8; 192];
        let prefix = b"FileAccess: delete ";
        let nlen = name.len().min(192 - prefix.len());
        msg[..prefix.len()].copy_from_slice(prefix);
        msg[prefix.len()..prefix.len() + nlen].copy_from_slice(&name.as_bytes()[..nlen]);
        crate::security::audit::record(
            crate::security::audit::Category::FileAccess,
            &msg[..prefix.len() + nlen],
        );
    }

    Ok(())
}

/// List all active files. Calls the provided closure for each.
pub fn list<F: FnMut(&str, usize, bool)>(mut callback: F) {
    // AUDIT-FS-C1: list iterates Active entries; a concurrent delete
    // can swap state mid-walk. Hold IrqGuard. Note: callback runs
    // inside the critical section — callbacks must NOT yield, do
    // network I/O, or call back into sealfs.
    let _g = crate::kernel::sync::IrqGuard::new();
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
    // AUDIT-FS-C1: single-read of FILE_COUNT under IrqGuard for
    // consistency with the rest of the API. On single-CPU a torn
    // read is impossible for usize, but the discipline is uniform.
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe { (FILE_COUNT, MAX_FILES) }
}

/// Look up a file's plaintext size without decrypting. Returns
/// `None` if no active file with that name exists. Used by the
/// `ns_delete` wrapper to know how many quota pages to release.
pub fn file_size(name: &str) -> Option<usize> {
    // AUDIT-FS-C1: see file_taint() for rationale.
    let _g = crate::kernel::sync::IrqGuard::new();
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
/// After this runs SealFS read/write WILL fail; the kernel is halting.
pub unsafe fn panic_wipe() {
    let key_ptr = core::ptr::addr_of_mut!(MASTER_KEY) as *mut u8;
    for i in 0..32 {
        unsafe { core::ptr::write_volatile(key_ptr.add(i), 0); }
    }
}

/// AUDIT-FS-C2 (2026-05-15): controlled master-key wipe for the
/// operator-triggered wipe path. `wipe::destroy_keys()` previously
/// called `sealfs::init(&zero_key)` twice expecting the second call
/// to overwrite the master key. But `init` returns early when
/// `INITIALIZED == true`, so the real master key was never zeroed
/// and the "Encryption keys destroyed" line was a lie.
///
/// This function actually zeroes:
///   * MASTER_KEY            — the AEAD root key
///   * BOOT_NONCE_PREFIX     — the per-boot nonce salt
///   * FILES[].nonce / .hash — per-file nonces + tags
///   * FILE_TAINT[]          — taint bitmaps
///   * FILE_COUNT            — slot occupancy
///   * INITIALIZED           — flipped back to false so a future
///                              init() can re-mount fresh
///
/// All writes are volatile so the compiler can't DCE under LTO.
/// Held under IrqGuard so the timer IRQ can't observe a half-wiped
/// MASTER_KEY between writes.
///
/// MUST be called AFTER `wipe::wipe_filesystem` returns. `delete()`
/// needs the key to compute the AEAD AAD for on-disk inode
/// zeroization; calling this first would leave inodes HMAC-tagged
/// with the real key on disk.
///
/// After this runs, all subsequent `create`/`read`/`delete` calls
/// return "filesystem not initialized" until a fresh `init()` runs.
pub fn wipe_master_key() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        // Zero MASTER_KEY (volatile so it can't be DCE'd).
        let key_ptr = core::ptr::addr_of_mut!(MASTER_KEY) as *mut u8;
        for i in 0..32 {
            core::ptr::write_volatile(key_ptr.add(i), 0);
        }
        // Zero BOOT_NONCE_PREFIX.
        let pfx_ptr = core::ptr::addr_of_mut!(BOOT_NONCE_PREFIX) as *mut u8;
        for i in 0..4 {
            core::ptr::write_volatile(pfx_ptr.add(i), 0);
        }
        // Zero per-file nonces and hash tags (the AEAD-tag halves).
        for i in 0..MAX_FILES {
            FILES[i].nonce = [0u8; 12];
            FILES[i].hash = [0u8; 32];
        }
        // Clear taint bitmaps.
        let taint_ptr = core::ptr::addr_of_mut!(FILE_TAINT);
        for i in 0..MAX_FILES {
            (*taint_ptr)[i] = 0;
        }
        // Reset slot occupancy and unmount.
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FILE_COUNT), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(INITIALIZED), false);
    }
    // Also reset the nonce counter so a future re-init starts clean.
    NONCE_COUNTER.store(0, core::sync::atomic::Ordering::Release);
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
