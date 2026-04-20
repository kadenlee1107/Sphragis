#![allow(dead_code)]
// Bat_OS — BatFS: Custom Encrypted Filesystem
// In-memory filesystem with per-file AES-256-CTR encryption.
// Each file gets a unique derived key. Merkle tree integrity.
// Phase 4 runs in RAM; Phase 7 will back this with NVMe.

use crate::crypto::aes::Aes256;
use crate::crypto::sha256;
use crate::kernel::mm::frame;

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
// persistent-across-reboot fix requires NVMe (Phase 7); for in-memory
// use, fresh random at boot is enough to ensure no cross-boot reuse.
static mut BOOT_NONCE_PREFIX: [u8; 4] = [0u8; 4];
static mut INITIALIZED: bool = false;

// ─── Merkle Tree ───
// Binary hash tree over all file hashes.
// Leaf[i] = SHA-256(file[i].hash). Internal nodes = SHA-256(left || right).
// Tree has MAX_FILES leaves → 2*MAX_FILES nodes total.
const MERKLE_NODES: usize = MAX_FILES * 2;
static mut MERKLE_TREE: [[u8; 32]; MERKLE_NODES] = [[0u8; 32]; MERKLE_NODES];

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
///
/// FL-027 / NEW-CRYPTO-007: each boot gets a fresh 4-byte random nonce
/// prefix so re-encrypting the same filename under the same derived key
/// produces a different CTR stream than the previous boot did. The
/// counter itself still restarts at 1; prefix + counter is the full
/// unique value.
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
///
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

        // Derive per-file encryption key
        let file_key = derive_file_key(name);
        let cipher = Aes256::new(&file_key);
        let nonce = next_nonce();

        // Copy data to allocated memory and encrypt in place
        let dest = data_addr as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());

        // Encrypt
        let encrypted_slice = core::slice::from_raw_parts_mut(dest, data.len());
        cipher.ctr_crypt(&nonce, encrypted_slice);

        // FL-028: HMAC-SHA256(master_key, name||nonce||ciphertext) as the
        // integrity tag. An attacker with a kernel-write primitive can't
        // forge this without the master key.
        let hash = compute_file_mac(name, &nonce, encrypted_slice);

        // Store file entry
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

        // Copy encrypted data to output buffer
        let src = entry.data_addr as *const u8;
        core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), entry.size);

        // FL-028: verify HMAC over the CIPHERTEXT + nonce + name BEFORE
        // decrypting. Constant-time byte-compare against the stored tag.
        let expected = compute_file_mac(name, &entry.nonce, &buf[..entry.size]);
        let mut diff: u8 = 0;
        for i in 0..32 { diff |= expected[i] ^ entry.hash[i]; }
        if diff != 0 {
            return Err("INTEGRITY VIOLATION — file tampered");
        }

        // Decrypt after MAC verification (Encrypt-then-MAC pattern).
        let file_key = derive_file_key(name);
        let cipher = Aes256::new(&file_key);
        cipher.ctr_crypt(&entry.nonce, &mut buf[..entry.size]);

        Ok(entry.size)
    }
}

/// Delete a file — zeroes the encrypted data before freeing.
pub fn delete(name: &str) -> Result<(), &'static str> {
    unsafe {
        let entry = find_file_mut(name)?;

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

/// V8-ROOT-6: panic-handler-only master-key wipe. Uses volatile writes so
/// the compiler cannot DCE. No locks. Best-effort: if we panic mid-write
/// the first N bytes are still zeroed, which already degrades an
/// attacker's recovered key.
///
/// # Safety
/// May only be called from the panic handler (via wipe::emergency_wipe).
/// After this runs BatFS read/write WILL fail; the kernel is halting.
pub unsafe fn panic_wipe() {
    let key_ptr = core::ptr::addr_of_mut!(MASTER_KEY) as *mut u8;
    for i in 0..32 {
        core::ptr::write_volatile(key_ptr.add(i), 0);
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
