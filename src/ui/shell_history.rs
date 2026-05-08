//! Shell history ring + arrow-key recall.
//!
//! The shell records each Enter-committed command into a fixed-size
//! ring buffer. Up-arrow walks backward through history, down-arrow
//! walks forward toward the live edit. Typing any printable char
//! resets the cursor — recall is a "browse" mode that gets replaced
//! by editing.
//!
//! Storage is static: `HISTORY_SIZE × MAX_LINE_LEN` = 16 × 256 = 4 KB.
//! No heap, no allocator dependency. Entries are stored as u8 + len
//! (no UTF-8 invariants needed since the dispatcher already takes
//! arbitrary bytes).
//!
//! The arrow-key wire format on serial is the standard ANSI three-
//! byte sequence:
//!
//!   ESC `[` `A`  → up      (0x1B 0x5B 0x41)
//!   ESC `[` `B`  → down    (0x1B 0x5B 0x42)
//!   ESC `[` `C`  → right   (0x1B 0x5B 0x43, ignored for v1)
//!   ESC `[` `D`  → left    (0x1B 0x5B 0x44, ignored for v1)
//!
//! `EscState` runs the per-byte state machine that detects these,
//! exposed as `EscState::feed(byte) -> Option<ArrowKey>`.

#![allow(clippy::needless_range_loop)]

use core::sync::atomic::{AtomicUsize, Ordering};

/// Number of distinct commands the ring holds. 16 is plenty for an
/// interactive session.
pub const HISTORY_SIZE: usize = 16;

/// Maximum command length recorded. Matches the input-loop buffer
/// caps (256 / 255).
pub const MAX_LINE_LEN: usize = 256;

#[derive(Clone, Copy)]
struct Entry {
    bytes: [u8; MAX_LINE_LEN],
    len: u16,
}

impl Entry {
    const fn empty() -> Self {
        Self { bytes: [0; MAX_LINE_LEN], len: 0 }
    }
}

// SAFETY: single-threaded kernel; the shell input loop is the only
// writer to RING / HEAD / COUNT.
static mut RING: [Entry; HISTORY_SIZE] = [Entry::empty(); HISTORY_SIZE];
/// Next slot to write. Wraps modulo HISTORY_SIZE.
static HEAD: AtomicUsize = AtomicUsize::new(0);
/// Total entries actually populated, capped at HISTORY_SIZE.
static COUNT: AtomicUsize = AtomicUsize::new(0);

/// "Browse cursor" — None when not browsing (live edit). Set to
/// 0..COUNT-1 when up-arrow recalled an entry. 0 = oldest visible
/// entry, COUNT-1 = newest.
static mut CURSOR: Option<usize> = None;

/// Record a command line in the ring. Called when the user hits
/// Enter on a non-empty buffer. Empty lines are not recorded; if the
/// new line equals the most-recent entry it is also skipped (de-dup
/// — a common shell convention so up-arrow stays useful when
/// re-running).
pub fn record(line: &[u8]) {
    if line.is_empty() { return; }
    let n = line.len().min(MAX_LINE_LEN);
    let count = COUNT.load(Ordering::Acquire);
    if count > 0 {
        // Compare against newest entry.
        let head = HEAD.load(Ordering::Acquire);
        let last_idx = (head + HISTORY_SIZE - 1) % HISTORY_SIZE;
        unsafe {
            let last = &(*core::ptr::addr_of!(RING))[last_idx];
            if last.len as usize == n && last.bytes[..n] == line[..n] {
                return; // duplicate of most-recent — skip.
            }
        }
    }
    let head = HEAD.load(Ordering::Acquire);
    unsafe {
        let ring = &mut *core::ptr::addr_of_mut!(RING);
        ring[head].bytes[..n].copy_from_slice(&line[..n]);
        ring[head].len = n as u16;
    }
    HEAD.store((head + 1) % HISTORY_SIZE, Ordering::Release);
    if count < HISTORY_SIZE {
        COUNT.store(count + 1, Ordering::Release);
    }
    // Recording resets browse mode.
    unsafe { CURSOR = None; }
}

/// Reset the browse cursor — call when the user starts editing
/// (any printable keystroke or backspace).
pub fn reset_cursor() {
    unsafe { CURSOR = None; }
}

/// Move backward through history (older). Returns the recalled
/// entry's bytes, or None if there's nothing further back.
pub fn prev() -> Option<&'static [u8]> {
    let count = COUNT.load(Ordering::Acquire);
    if count == 0 { return None; }
    unsafe {
        // First up-arrow: jump to most-recent (count-1 in browse-index).
        let next_idx = match CURSOR {
            None => count - 1,
            Some(0) => return None, // already at oldest
            Some(i) => i - 1,
        };
        CURSOR = Some(next_idx);
        Some(entry_bytes(next_idx, count))
    }
}

/// Move forward through history (newer). Returns the recalled
/// entry's bytes, or None to indicate "back to live edit" (and
/// internally clears the cursor).
pub fn next() -> Option<&'static [u8]> {
    let count = COUNT.load(Ordering::Acquire);
    if count == 0 { return None; }
    unsafe {
        let cur = match CURSOR {
            None => return None, // not browsing — nothing to advance.
            Some(i) => i,
        };
        if cur + 1 >= count {
            // Was at newest — drop back to live edit.
            CURSOR = None;
            return None;
        }
        let next_idx = cur + 1;
        CURSOR = Some(next_idx);
        Some(entry_bytes(next_idx, count))
    }
}

/// Translate a 0..count "browse index" (0 = oldest) into the actual
/// ring slot, then borrow that entry's bytes.
unsafe fn entry_bytes(browse_idx: usize, count: usize) -> &'static [u8] {
    let head = HEAD.load(Ordering::Acquire);
    // The oldest entry sits at HEAD-COUNT (mod size); browse_idx 0
    // is that oldest one.
    let start = (head + HISTORY_SIZE - count) % HISTORY_SIZE;
    let slot = (start + browse_idx) % HISTORY_SIZE;
    let ring = unsafe { &*core::ptr::addr_of!(RING) };
    &ring[slot].bytes[..ring[slot].len as usize]
}

// ─── Arrow-key ESC sequence parser ──────────────────────────────────────

/// Decoded arrow-key direction.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArrowKey {
    Up,
    Down,
    Right,
    Left,
}

/// Per-loop state for accumulating an ESC sequence across multiple
/// reads. The shell input loop owns one of these and feeds each byte.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum EscState {
    /// Not inside a sequence. The next ESC byte starts one.
    #[default]
    Idle,
    /// Saw ESC; expecting `[`.
    SeenEsc,
    /// Saw `ESC [`; expecting the final byte (A/B/C/D).
    SeenCsi,
}

/// Result of feeding one byte to the parser.
#[derive(Clone, Copy)]
pub enum FeedResult {
    /// Byte was consumed by the ESC parser; loop should NOT echo or
    /// dispatch it.
    Consumed,
    /// Byte ends a complete arrow-key sequence; loop should handle
    /// the recall.
    Arrow(ArrowKey),
    /// Byte is a normal one (no sequence in progress); loop should
    /// dispatch as usual.
    Pass(u8),
}

impl EscState {
    /// Feed one input byte. Drives the state machine + returns what
    /// the input loop should do.
    pub fn feed(&mut self, b: u8) -> FeedResult {
        match (*self, b) {
            (EscState::Idle, 0x1B) => {
                *self = EscState::SeenEsc;
                FeedResult::Consumed
            }
            (EscState::Idle, _) => FeedResult::Pass(b),
            (EscState::SeenEsc, b'[') => {
                *self = EscState::SeenCsi;
                FeedResult::Consumed
            }
            (EscState::SeenEsc, _) => {
                // Lone ESC followed by something unexpected — drop
                // the sequence and pass the new byte through.
                *self = EscState::Idle;
                FeedResult::Pass(b)
            }
            (EscState::SeenCsi, b'A') => { *self = EscState::Idle; FeedResult::Arrow(ArrowKey::Up) }
            (EscState::SeenCsi, b'B') => { *self = EscState::Idle; FeedResult::Arrow(ArrowKey::Down) }
            (EscState::SeenCsi, b'C') => { *self = EscState::Idle; FeedResult::Arrow(ArrowKey::Right) }
            (EscState::SeenCsi, b'D') => { *self = EscState::Idle; FeedResult::Arrow(ArrowKey::Left) }
            (EscState::SeenCsi, _) => {
                // Some other CSI sequence we don't care about — drop
                // both this byte and any future ones until the
                // sequence terminator. For v1 we just reset and lose
                // the byte; v2 could implement full CSI parsing.
                *self = EscState::Idle;
                FeedResult::Consumed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        unsafe {
            (*core::ptr::addr_of_mut!(RING)).fill(Entry::empty());
            CURSOR = None;
        }
        HEAD.store(0, Ordering::Release);
        COUNT.store(0, Ordering::Release);
    }

    #[test]
    fn record_and_walk_back() {
        reset();
        record(b"first");
        record(b"second");
        record(b"third");
        assert_eq!(prev(), Some(&b"third"[..]));
        assert_eq!(prev(), Some(&b"second"[..]));
        assert_eq!(prev(), Some(&b"first"[..]));
        assert_eq!(prev(), None); // at oldest
    }

    #[test]
    fn dedup_consecutive_equal() {
        reset();
        record(b"foo");
        record(b"foo"); // skipped
        record(b"bar");
        record(b"bar"); // skipped
        record(b"foo"); // not skipped (intervening "bar")
        assert_eq!(COUNT.load(Ordering::Acquire), 3);
        assert_eq!(prev(), Some(&b"foo"[..]));
        assert_eq!(prev(), Some(&b"bar"[..]));
        assert_eq!(prev(), Some(&b"foo"[..]));
    }

    #[test]
    fn down_arrow_returns_to_live_edit() {
        reset();
        record(b"first");
        record(b"second");
        // Browse to oldest, then walk back to live edit.
        assert_eq!(prev(), Some(&b"second"[..]));
        assert_eq!(prev(), Some(&b"first"[..]));
        assert_eq!(next(), Some(&b"second"[..]));
        assert_eq!(next(), None); // back to live edit
        assert_eq!(next(), None); // still none
    }

    #[test]
    fn esc_sequence_parser() {
        let mut s = EscState::default();
        assert!(matches!(s.feed(0x1B), FeedResult::Consumed));
        assert!(matches!(s.feed(b'['), FeedResult::Consumed));
        assert!(matches!(s.feed(b'A'), FeedResult::Arrow(ArrowKey::Up)));
        // Lone byte after ESC[A returns to Idle and is passed.
        assert!(matches!(s.feed(b'x'), FeedResult::Pass(b'x')));
    }

    #[test]
    fn lone_esc_passes_next_byte() {
        // A bare ESC followed by something unexpected drops back to
        // Idle and the unexpected byte is passed (not eaten).
        let mut s = EscState::default();
        assert!(matches!(s.feed(0x1B), FeedResult::Consumed));
        assert!(matches!(s.feed(b'q'), FeedResult::Pass(b'q')));
    }
}
