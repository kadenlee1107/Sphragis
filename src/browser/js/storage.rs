#![allow(dead_code)]
// Bat_OS — Web Storage API (localStorage / sessionStorage)
// Simple key-value store backed by in-memory arrays.
//
// STUMP #108 — Sprint 3.5: actually wired to JS now. Process-global
// LOCAL_STORAGE instance, exposed as `localStorage` in JS with the
// standard `getItem` / `setItem` / `removeItem` / `clear` methods.
// Cleared on cave switch so a logged-out cave doesn't inherit the
// previous tenant's UI state.

use core::sync::atomic::{AtomicBool, Ordering};

const MAX_ENTRIES: usize = 64;
const MAX_KEY: usize = 32;
const MAX_VAL: usize = 128;

/// STUMP #111 (audit M-storage-saturation): one-shot saturation alarm.
/// Pre-fix `set_item` silently dropped new keys when count == MAX_ENTRIES,
/// so a script could fill 64 entries with junk and the operator's auth
/// state (e.g. "remember-me-token") never made it in. Re-armed on
/// cave switch so the next tenant gets a fresh "first" event.
static SET_FULL_FIRST_FAIL: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
struct StorageEntry {
    active: bool,
    key: [u8; MAX_KEY],
    key_len: usize,
    value: [u8; MAX_VAL],
    value_len: usize,
}

impl StorageEntry {
    const fn empty() -> Self {
        StorageEntry {
            active: false,
            key: [0; MAX_KEY], key_len: 0,
            value: [0; MAX_VAL], value_len: 0,
        }
    }
}

pub struct WebStorage {
    entries: [StorageEntry; MAX_ENTRIES],
    count: usize,
}

impl WebStorage {
    pub const fn new() -> Self {
        WebStorage {
            entries: [StorageEntry::empty(); MAX_ENTRIES],
            count: 0,
        }
    }

    pub fn get_item(&self, key: &str) -> Option<&str> {
        for i in 0..self.count {
            let e = &self.entries[i];
            if e.active && e.key_len == key.len() {
                let k = unsafe { core::str::from_utf8_unchecked(&e.key[..e.key_len]) };
                if k == key {
                    return Some(unsafe { core::str::from_utf8_unchecked(&e.value[..e.value_len]) });
                }
            }
        }
        None
    }

    pub fn set_item(&mut self, key: &str, value: &str) {
        // Check if key exists → update
        for i in 0..self.count {
            let e = &mut self.entries[i];
            if e.active {
                let k = unsafe { core::str::from_utf8_unchecked(&e.key[..e.key_len]) };
                if k == key {
                    let vlen = value.len().min(MAX_VAL);
                    e.value[..vlen].copy_from_slice(&value.as_bytes()[..vlen]);
                    e.value_len = vlen;
                    return;
                }
            }
        }
        // Create new
        if self.count < MAX_ENTRIES {
            let e = &mut self.entries[self.count];
            e.active = true;
            let klen = key.len().min(MAX_KEY);
            e.key[..klen].copy_from_slice(&key.as_bytes()[..klen]);
            e.key_len = klen;
            let vlen = value.len().min(MAX_VAL);
            e.value[..vlen].copy_from_slice(&value.as_bytes()[..vlen]);
            e.value_len = vlen;
            self.count += 1;
        } else {
            // STUMP #111 (audit M-storage-saturation): table full —
            // one-shot audit + UART warning so a flooding script is
            // visible. Re-armed on cave switch.
            if !SET_FULL_FIRST_FAIL.swap(true, Ordering::AcqRel) {
                crate::security::audit::record(
                    crate::security::audit::Category::Script,
                    b"localStorage FULL (MAX_ENTRIES=64) - dropping new keys",
                );
                crate::drivers::uart::puts("[storage] WARNING: localStorage full - key dropped\n");
            }
        }
    }

    pub fn remove_item(&mut self, key: &str) {
        for i in 0..self.count {
            let e = &mut self.entries[i];
            if e.active {
                let k = unsafe { core::str::from_utf8_unchecked(&e.key[..e.key_len]) };
                if k == key {
                    e.active = false;
                    return;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.count {
            self.entries[i].active = false;
        }
        self.count = 0;
    }

    pub fn length(&self) -> usize {
        self.entries[..self.count].iter().filter(|e| e.active).count()
    }

    pub fn key(&self, index: usize) -> Option<&str> {
        let mut count = 0;
        for i in 0..self.count {
            if self.entries[i].active {
                if count == index {
                    return Some(unsafe { core::str::from_utf8_unchecked(&self.entries[i].key[..self.entries[i].key_len]) });
                }
                count += 1;
            }
        }
        None
    }
}

// ── STUMP #108 ── module-level singleton + reset hook for cave switch.
// One instance per kernel; the JS engine's localStorage methods read
// and write this. No allocator dependency — fixed-size arrays.
static mut LOCAL: WebStorage = WebStorage::new();
static LOCAL_DIRTY: AtomicBool = AtomicBool::new(false);

#[inline]
fn local_mut() -> &'static mut WebStorage {
    unsafe { &mut *core::ptr::addr_of_mut!(LOCAL) }
}

#[inline]
fn local_ref() -> &'static WebStorage {
    unsafe { &*core::ptr::addr_of!(LOCAL) }
}

pub fn local_get_item(key: &str) -> Option<&'static str> {
    local_ref().get_item(key)
}

pub fn local_set_item(key: &str, value: &str) {
    local_mut().set_item(key, value);
    LOCAL_DIRTY.store(true, Ordering::Relaxed);
}

pub fn local_remove_item(key: &str) {
    local_mut().remove_item(key);
    LOCAL_DIRTY.store(true, Ordering::Relaxed);
}

pub fn local_clear() {
    local_mut().clear();
    LOCAL_DIRTY.store(true, Ordering::Relaxed);
}

pub fn local_length() -> usize { local_ref().length() }

pub fn local_dirty() -> bool { LOCAL_DIRTY.load(Ordering::Relaxed) }
pub fn clear_dirty() { LOCAL_DIRTY.store(false, Ordering::Relaxed); }

/// STUMP #108: cave-switch hook. Wipes all keys so a logged-out cave
/// doesn't inherit the previous tenant's UI state (theme, last URL,
/// shopping cart, ...).
pub fn reset_for_cave_switch() {
    local_mut().clear();
    LOCAL_DIRTY.store(false, Ordering::Relaxed);
    // STUMP #111: re-arm the saturation alarm so the next cave's
    // first localStorage flood event is audible.
    SET_FULL_FIRST_FAIL.store(false, Ordering::Release);
}
