//! Top-bar badge config: which badges to show, in what order.
//! Persists to /system/desktop/topbar.cfg in SealFS as a one-line
//! ASCII letter sequence ("NDC" = NET, DEADMAN, CLOCK).

#![allow(dead_code)]

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Badge {
    NetMode, Deadman, Clock, Caves, Attempts,
    Memory, Cpu, Audit, CaveFocus,
}

fn badge_letter(b: Badge) -> u8 {
    match b {
        Badge::NetMode   => b'N',  Badge::Deadman => b'D',
        Badge::Clock     => b'C',  Badge::Caves   => b'V',
        Badge::Attempts  => b'A',  Badge::Memory  => b'M',
        Badge::Cpu       => b'P',  Badge::Audit   => b'U',
        Badge::CaveFocus => b'F',
    }
}

fn letter_badge(c: u8) -> Option<Badge> {
    Some(match c {
        b'N' => Badge::NetMode,  b'D' => Badge::Deadman,
        b'C' => Badge::Clock,    b'V' => Badge::Caves,
        b'A' => Badge::Attempts, b'M' => Badge::Memory,
        b'P' => Badge::Cpu,      b'U' => Badge::Audit,
        b'F' => Badge::CaveFocus, _ => return None,
    })
}

pub const MAX_BADGES: usize = 9;

// Whole-array volatile read/write pattern (matching wm.rs convention).
// Badge is Copy + Eq, so [Option<Badge>; 9] is Copy — the full-array
// shuffle is cheap and avoids Rust-2024 static_mut_refs errors.
static mut BADGES: [Option<Badge>; MAX_BADGES] = [
    Some(Badge::NetMode), Some(Badge::Deadman), Some(Badge::Clock),
    None, None, None, None, None, None,
];

fn read_badges() -> [Option<Badge>; MAX_BADGES] {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BADGES)) }
}

fn write_badges(new: [Option<Badge>; MAX_BADGES]) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(BADGES), new) }
}

const CONFIG_FILE: &str = "/system/desktop/topbar.cfg";

pub fn iter() -> impl Iterator<Item = Badge> {
    read_badges()
        .iter()
        .filter_map(|b| *b)
        .collect::<alloc::vec::Vec<_>>()
        .into_iter()
}

pub fn toggle(badge: Badge) {
    let mut arr = read_badges();
    // Remove if present.
    for slot in arr.iter_mut() {
        if *slot == Some(badge) {
            *slot = None;
            // Compact: shift non-None entries to the front.
            let mut compacted: [Option<Badge>; MAX_BADGES] = [None; MAX_BADGES];
            for (j, &b) in arr.iter().filter_map(|s| s.as_ref()).enumerate() {
                compacted[j] = Some(b);
            }
            write_badges(compacted);
            save();
            return;
        }
    }
    // Not present — append if there is a free slot.
    for slot in arr.iter_mut() {
        if slot.is_none() { *slot = Some(badge); write_badges(arr); save(); return; }
    }
}

fn save() {
    let mut buf = [0u8; MAX_BADGES + 1];
    let mut n = 0;
    for b in iter() { buf[n] = badge_letter(b); n += 1; }
    buf[n] = b'\n'; n += 1;
    let _ = crate::fs::sealfs::create(CONFIG_FILE, &buf[..n]);
}

pub fn load() {
    let mut buf = [0u8; MAX_BADGES + 1];
    if let Ok(n) = crate::fs::sealfs::read(CONFIG_FILE, &mut buf) {
        let mut new_badges: [Option<Badge>; MAX_BADGES] = [None; MAX_BADGES];
        let mut j = 0;
        for &c in &buf[..n] {
            if c == b'\n' { break; }
            if let Some(b) = letter_badge(c) {
                new_badges[j] = Some(b); j += 1;
                if j >= MAX_BADGES { break; }
            }
        }
        if j > 0 { write_badges(new_badges); }
    }
}
