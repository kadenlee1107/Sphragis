#![allow(dead_code)]
// Bat_OS — Web Storage API (localStorage / sessionStorage)
// Simple key-value store backed by in-memory arrays.

const MAX_ENTRIES: usize = 64;
const MAX_KEY: usize = 32;
const MAX_VAL: usize = 128;

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
