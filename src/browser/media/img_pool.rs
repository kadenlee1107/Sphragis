// Bat_OS — `<img>` decode pool.
//
// Layout calls `load_png_for_src("foo.png")` and gets back a slot
// index it can stash into LayoutBox.image_slot. Paint then calls
// `get(slot)` to retrieve the decoded PngImage and `draw_png` it.
//
// Pool is fixed-size in BSS (PngImage is ~1 MB each, so 8 slots = 8 MB).
// Reset between renders so a long browsing session doesn't leak.

use super::png::{PngImage, decode};

const SLOTS: usize = 8;

static mut POOL: [PngImage; SLOTS] = [
    PngImage::empty(), PngImage::empty(), PngImage::empty(), PngImage::empty(),
    PngImage::empty(), PngImage::empty(), PngImage::empty(), PngImage::empty(),
];
static mut USED: usize = 0;

/// Drop all loaded images. Call at the start of each render so a
/// new page doesn't see stale slots from the previous one.
pub fn reset() {
    unsafe {
        USED = 0;
        for i in 0..SLOTS {
            (*core::ptr::addr_of_mut!(POOL))[i].valid = false;
            (*core::ptr::addr_of_mut!(POOL))[i].width = 0;
            (*core::ptr::addr_of_mut!(POOL))[i].height = 0;
        }
    }
}

/// Decode `bytes` as PNG into a free pool slot. Returns the slot
/// index, or 0xFFFF on full pool / decode failure.
pub fn load(bytes: &[u8]) -> u16 {
    unsafe {
        if USED >= SLOTS { return 0xFFFF; }
        let slot = USED;
        let pool = &mut *core::ptr::addr_of_mut!(POOL);
        if decode(bytes, &mut pool[slot]).is_err() {
            return 0xFFFF;
        }
        if !pool[slot].valid { return 0xFFFF; }
        USED += 1;
        slot as u16
    }
}

/// Look up a previously-loaded image by slot index.
pub fn get(slot: u16) -> Option<&'static PngImage> {
    if slot as usize >= SLOTS { return None; }
    unsafe {
        let pool = &*core::ptr::addr_of!(POOL);
        if pool[slot as usize].valid { Some(&pool[slot as usize]) } else { None }
    }
}
