// Bat_OS — String Intern Table
// All JS strings are stored once and referenced by StringId.
// Fast comparison (just compare ids), no duplication, compact NaN-box storage.

use super::value::StringId;

/// Maximum interned strings.
const MAX_STRINGS: usize = 4096;
/// Total byte storage for all string data.
const STRING_ARENA_SIZE: usize = 256 * 1024; // 256 KB

/// Entry in the string table.
#[derive(Clone, Copy)]
struct StringEntry {
    offset: u32,   // byte offset into data[]
    len: u16,      // string length in bytes
    hash: u16,     // fast hash for lookup
}

/// Intern table: all strings stored in a flat arena, referenced by index.
pub struct StringTable {
    data: [u8; STRING_ARENA_SIZE],
    data_len: usize,
    entries: [StringEntry; MAX_STRINGS],
    entry_count: usize,
}

impl StringTable {
    pub const fn new() -> Self {
        StringTable {
            data: [0; STRING_ARENA_SIZE],
            data_len: 0,
            entries: [StringEntry { offset: 0, len: 0, hash: 0 }; MAX_STRINGS],
            entry_count: 0,
        }
    }

    /// Initialize with well-known strings (called once at engine startup).
    /// These get fixed StringId values that the compiler can use directly.
    pub fn init_well_known(&mut self) {
        // StringId(0) = "" (empty string)
        self.intern(b"");
        // StringId(1..N) = commonly used property/method names
        self.intern(b"undefined");
        self.intern(b"null");
        self.intern(b"true");
        self.intern(b"false");
        self.intern(b"length");
        self.intern(b"prototype");
        self.intern(b"constructor");
        self.intern(b"toString");
        self.intern(b"valueOf");
        self.intern(b"__proto__");
        self.intern(b"hasOwnProperty");
        self.intern(b"push");
        self.intern(b"pop");
        self.intern(b"shift");
        self.intern(b"unshift");
        self.intern(b"indexOf");
        self.intern(b"slice");
        self.intern(b"splice");
        self.intern(b"join");
        self.intern(b"map");
        self.intern(b"filter");
        self.intern(b"forEach");
        self.intern(b"reduce");
        self.intern(b"concat");
        self.intern(b"keys");
        self.intern(b"values");
        self.intern(b"entries");
        self.intern(b"charAt");
        self.intern(b"substring");
        self.intern(b"split");
        self.intern(b"replace");
        self.intern(b"trim");
        self.intern(b"toLowerCase");
        self.intern(b"toUpperCase");
        self.intern(b"includes");
        self.intern(b"startsWith");
        self.intern(b"endsWith");
        self.intern(b"parseInt");
        self.intern(b"parseFloat");
        self.intern(b"isNaN");
        self.intern(b"NaN");
        self.intern(b"Infinity");
        self.intern(b"Math");
        self.intern(b"JSON");
        self.intern(b"console");
        self.intern(b"log");
        self.intern(b"warn");
        self.intern(b"error");
        self.intern(b"document");
        self.intern(b"getElementById");
        self.intern(b"querySelector");
        self.intern(b"querySelectorAll");
        self.intern(b"createElement");
        self.intern(b"createTextNode");
        self.intern(b"appendChild");
        self.intern(b"removeChild");
        self.intern(b"setAttribute");
        self.intern(b"getAttribute");
        self.intern(b"textContent");
        self.intern(b"innerHTML");
        self.intern(b"style");
        self.intern(b"classList");
        self.intern(b"addEventListener");
        self.intern(b"removeEventListener");
        self.intern(b"children");
        self.intern(b"parentElement");
        self.intern(b"window");
        self.intern(b"setTimeout");
        self.intern(b"setInterval");
        self.intern(b"clearTimeout");
        self.intern(b"clearInterval");
        self.intern(b"alert");
        self.intern(b"typeof");
        self.intern(b"Object");
        self.intern(b"Array");
        self.intern(b"String");
        self.intern(b"Number");
        self.intern(b"Boolean");
        self.intern(b"Function");
        self.intern(b"Error");
        self.intern(b"TypeError");
        self.intern(b"RangeError");
        self.intern(b"SyntaxError");
        self.intern(b"Date");
        self.intern(b"RegExp");
        self.intern(b"Promise");
        self.intern(b"Symbol");
        self.intern(b"call");
        self.intern(b"apply");
        self.intern(b"bind");
        self.intern(b"then");
        self.intern(b"catch");
        self.intern(b"finally");
        self.intern(b"resolve");
        self.intern(b"reject");
        self.intern(b"name");
        self.intern(b"message");
        self.intern(b"stack");
    }

    /// Intern a string: if it already exists, return its id. Otherwise, store it.
    pub fn intern(&mut self, bytes: &[u8]) -> StringId {
        let hash = Self::hash(bytes);

        // Search for existing entry
        for i in 0..self.entry_count {
            let e = &self.entries[i];
            if e.hash == hash && e.len as usize == bytes.len() {
                // Verify exact match
                let stored = &self.data[e.offset as usize..e.offset as usize + e.len as usize];
                if stored == bytes {
                    return StringId(i as u32);
                }
            }
        }

        // Not found — add new entry
        if self.entry_count >= MAX_STRINGS {
            // Table full — return empty string
            return StringId::EMPTY;
        }
        if self.data_len + bytes.len() > STRING_ARENA_SIZE {
            // Arena full
            return StringId::EMPTY;
        }

        let offset = self.data_len;
        let len = bytes.len();
        self.data[offset..offset + len].copy_from_slice(bytes);
        self.data_len += len;

        let id = self.entry_count;
        self.entries[id] = StringEntry {
            offset: offset as u32,
            len: len as u16,
            hash,
        };
        self.entry_count += 1;

        StringId(id as u32)
    }

    /// Get string bytes by id.
    pub fn get(&self, id: StringId) -> &[u8] {
        let idx = id.0 as usize;
        if idx >= self.entry_count {
            return b"";
        }
        let e = &self.entries[idx];
        &self.data[e.offset as usize..e.offset as usize + e.len as usize]
    }

    /// Get string as &str (assumes valid UTF-8).
    pub fn get_str(&self, id: StringId) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.get(id)) }
    }

    /// Get string length.
    pub fn len(&self, id: StringId) -> usize {
        let idx = id.0 as usize;
        if idx >= self.entry_count { 0 } else { self.entries[idx].len as usize }
    }

    /// Concatenate two strings and intern the result.
    pub fn concat(&mut self, a: StringId, b: StringId) -> StringId {
        let a_bytes = self.get(a);
        let b_bytes = self.get(b);
        let total = a_bytes.len() + b_bytes.len();
        if total > 4096 { return StringId::EMPTY; } // safety limit

        // Build in a temp buffer
        let mut tmp = [0u8; 4096];
        let a_len = a_bytes.len();
        let b_len = b_bytes.len();
        tmp[..a_len].copy_from_slice(a_bytes);
        // Re-fetch b in case intern moved things (it won't, but be safe)
        let b_bytes2 = self.get(b);
        tmp[a_len..a_len + b_len].copy_from_slice(b_bytes2);
        self.intern(&tmp[..total])
    }

    /// Number of interned strings.
    pub fn count(&self) -> usize { self.entry_count }

    /// Simple hash function (FNV-1a style).
    fn hash(bytes: &[u8]) -> u16 {
        let mut h: u32 = 2166136261;
        for &b in bytes {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        (h ^ (h >> 16)) as u16
    }
}

// ─── Well-known StringId constants ───
// These match the order in init_well_known().

pub mod well_known {
    use super::StringId;

    pub const EMPTY: StringId       = StringId(0);
    pub const UNDEFINED: StringId   = StringId(1);
    pub const NULL: StringId        = StringId(2);
    pub const TRUE: StringId        = StringId(3);
    pub const FALSE: StringId       = StringId(4);
    pub const LENGTH: StringId      = StringId(5);
    pub const PROTOTYPE: StringId   = StringId(6);
    pub const CONSTRUCTOR: StringId = StringId(7);
    pub const TO_STRING: StringId   = StringId(8);
    pub const VALUE_OF: StringId    = StringId(9);
    pub const PROTO: StringId       = StringId(10);
}
