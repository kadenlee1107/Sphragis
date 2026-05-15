#![allow(dead_code)]
// Sphragis — GPU Console
// Terminal emulator rendered to the framebuffer.
// Handles text output and cursor management.
// XXX Wave-2-temp: 1 old-WM call site commented out, restored in Task 7.

use crate::ui::gpu;
use super::font::{self, CHAR_W, CHAR_H};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

// palette matches the WM chrome and lock screen.
const BG: u32 = 0xFF0A0A0A;
const FG: u32 = 0xFFE5E7EB;     // INK
const FG_HI: u32 = 0xFFE5E7EB;
const FG_DIM: u32 = 0xFF4B5563;
const ACCENT_CYAN: u32 = 0xFF22D3EE;
const ACCENT_CYAN_DIM: u32 = 0xFF0E7490;
const ACCENT_GREEN: u32 = 0xFF22C55E;
const ACCENT_RED: u32 = 0xFFEF4444;

// bumped MARGIN_Y from 16 (which sat *inside* the new
// 24px title bar) to 32 (clears title bar + 8px inset) and
// STATUS_BAR_H from 32 to 28 to match the redesigned wm chrome.
const MARGIN_X: u32 = 16;
const MARGIN_Y: u32 = 32;
const STATUS_BAR_H: u32 = 28;

static CURSOR_X: AtomicU32 = AtomicU32::new(0);
static CURSOR_Y: AtomicU32 = AtomicU32::new(0);

// ─── Scrollback buffer ───────────────────────────────────
//
// 2D cell grid that mirrors the visible console region. Every char
// written via `putc` lands in BOTH the framebuffer and a Cell here, so
// `redraw_content` can replay the buffer when the WM clears the FB on
// tab switch / split / etc. Replaying chars also auto-fixes the
// cmd_buf-vs-screen consistency quirk: typed-but-not-Entered chars
// are buffer cells just like everything else, so on tab return the
// user sees their in-progress command exactly as they left it.
//
// 160 cols × 50 rows × 5 bytes/cell ≈ 40KB BSS. Rows scroll up via
// memmove of the buffer rows when content overflows.
const SB_COLS: usize = 160;
const SB_ROWS: usize = 50;

#[derive(Clone, Copy)]
struct Cell {
    ch: u8,
    fg: u32,
}
const SB_EMPTY: Cell = Cell { ch: 0, fg: 0 };

static mut SB_BUF: [[Cell; SB_COLS]; SB_ROWS] = [[SB_EMPTY; SB_COLS]; SB_ROWS];

// ── Visual selection mode (row-based) ─────────────────────────────
//
// Keyboard-driven scrollback selection. Triggered by Ctrl+S on the
// SH tab. While active, arrow keys move a single-row cursor through
// the scrollback; Shift+arrow extends the selection; Enter copies
// the selected rows (joined by newlines) to the system clipboard;
// Esc exits without copying.
//
// Row-based, not cell-based — keeps the rendering simple (just
// inverse-video the highlighted rows) while still giving the
// operator a real "highlight what I want on screen" UX. Cell-level
// selection (drag a rectangle) is a future upgrade once mouse
// support is wired in.
static mut SELECT_MODE: bool = false;
static mut SEL_ANCHOR: u16 = 0;
static mut SEL_CURSOR: u16 = 0;
const SELECT_BG: u32 = 0xFF2A2A2A;
const SELECT_FG: u32 = 0xFFFFFFFF;

pub fn select_mode_active() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECT_MODE)) }
}

/// Enter selection mode. Anchor + cursor land on the last *output*
/// row — skipping past the empty prompt row at the bottom so the
/// user doesn't accidentally copy `sphragis >` along with their
/// content. Arrow-down lands on the prompt row if you actually want
/// it; arrow-up walks back through history.
pub fn enter_select_mode() {
    let start = find_last_output_row();
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_ANCHOR), start);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_CURSOR), start);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECT_MODE), true);
    }
}

/// Find the bottom-most row that is NOT a bare prompt. Searches up
/// from the actual bottom; if every row matches "sphragis > " or is
/// empty, falls back to bottom.
fn find_last_output_row() -> u16 {
    let bottom = find_bottom_row();
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        let mut r = bottom as i32;
        while r >= 0 {
            if !row_is_prompt_only(&buf[r as usize]) {
                return r as u16;
            }
            r -= 1;
        }
    }
    bottom
}

/// True if a scrollback row contains nothing but the prompt
/// "sphragis > " (plus trailing nulls / spaces). Used to skip prompt
/// rows when positioning the initial select cursor.
fn row_is_prompt_only(row: &[Cell; SB_COLS]) -> bool {
    let prompt = b"sphragis > ";
    if row.len() < prompt.len() { return false; }
    for (i, &want) in prompt.iter().enumerate() {
        if row[i].ch != want { return false; }
    }
    // Everything after the prompt should be empty (null) or space.
    for i in prompt.len()..SB_COLS {
        let c = row[i].ch;
        if c != 0 && c != b' ' { return false; }
    }
    true
}

pub fn exit_select_mode() {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECT_MODE), false);
    }
}

/// Move the selection cursor. `extend=false` drags the anchor along
/// (single-row cursor); `extend=true` keeps the anchor and grows
/// the range.
pub fn sel_move_up(extend: bool) {
    unsafe {
        let cur = core::ptr::read_volatile(core::ptr::addr_of!(SEL_CURSOR));
        let new = cur.saturating_sub(1);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_CURSOR), new);
        if !extend {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_ANCHOR), new);
        }
    }
}

pub fn sel_move_down(extend: bool) {
    unsafe {
        let cur = core::ptr::read_volatile(core::ptr::addr_of!(SEL_CURSOR));
        let bottom = find_bottom_row();
        let new = (cur + 1).min(bottom);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_CURSOR), new);
        if !extend {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(SEL_ANCHOR), new);
        }
    }
}

/// Copy selected rows to the system clipboard, joined by '\n'.
/// Returns the number of bytes copied. Trailing whitespace on each
/// row is trimmed so an unfilled scrollback row doesn't pad pasted
/// output with spaces.
pub fn sel_copy_to_clipboard() -> usize {
    let (lo, hi) = unsafe {
        let a = core::ptr::read_volatile(core::ptr::addr_of!(SEL_ANCHOR));
        let c = core::ptr::read_volatile(core::ptr::addr_of!(SEL_CURSOR));
        (a.min(c) as usize, a.max(c) as usize)
    };
    let mut out = [0u8; crate::ui::clipboard::CLIPBOARD_CAP];
    let mut off = 0;
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for r in lo..=hi {
            if r >= SB_ROWS { break; }
            // Find right edge of content on this row.
            let mut right = SB_COLS;
            while right > 0 && buf[r][right - 1].ch == 0 { right -= 1; }
            for c in 0..right {
                if off >= out.len() { break; }
                let ch = buf[r][c].ch;
                out[off] = if ch == 0 { b' ' } else { ch };
                off += 1;
            }
            if r < hi && off < out.len() {
                out[off] = b'\n';
                off += 1;
            }
        }
    }
    crate::ui::clipboard::set(&out[..off]);
    off
}

/// Find the lowest scrollback row that has any non-empty cell.
/// Used to clamp the selection cursor to actual content.
fn find_bottom_row() -> u16 {
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for r in (0..SB_ROWS).rev() {
            for c in 0..SB_COLS {
                if buf[r][c].ch != 0 {
                    return r as u16;
                }
            }
        }
    }
    0
}

/// Copy the last `n` non-empty rows of scrollback into the
/// clipboard, joined by '\n'. Used by the `clip yank-back N` shell
/// command. Returns bytes copied.
pub fn yank_last_rows(n: usize) -> usize {
    let bottom = find_bottom_row() as usize;
    let lo = bottom.saturating_sub(n.saturating_sub(1));
    let mut out = [0u8; crate::ui::clipboard::CLIPBOARD_CAP];
    let mut off = 0;
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for r in lo..=bottom {
            if r >= SB_ROWS { break; }
            let mut right = SB_COLS;
            while right > 0 && buf[r][right - 1].ch == 0 { right -= 1; }
            for c in 0..right {
                if off >= out.len() { break; }
                let ch = buf[r][c].ch;
                out[off] = if ch == 0 { b' ' } else { ch };
                off += 1;
            }
            if r < bottom && off < out.len() {
                out[off] = b'\n';
                off += 1;
            }
        }
    }
    crate::ui::clipboard::set(&out[..off]);
    off
}

/// Current "pen" color used by the next `putc` write. Callers swap
/// it with `set_pen()` to color sections of output (e.g. the prompt's
/// "sphragis" in INK, " > " in CYAN). Defaults to FG.
static mut PEN_COLOR: u32 = FG;

/// Set the foreground color used by subsequent `putc` calls.
pub fn set_pen(color: u32) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(PEN_COLOR), color); }
}

/// Reset pen to the default INK color. Call after a colored region.
pub fn reset_pen() { set_pen(FG); }

/// Move the cursor to an explicit (col, row). Used by `shell_banner`
/// to land the prompt below the bat-glyph banner row.
pub fn set_cursor(col: u32, row: u32) {
    CURSOR_X.store(col, Ordering::Relaxed);
    CURSOR_Y.store(row, Ordering::Relaxed);
}

/// Read the current cursor as (col, row). Callers that need to
/// restore cursor across a banner repaint use this.
pub fn cursor() -> (u32, u32) {
    (CURSOR_X.load(Ordering::Relaxed), CURSOR_Y.load(Ordering::Relaxed))
}

/// Map a cell coordinate (col, row) to a pixel position in framebuffer
/// coordinates. Used by external overlays (e.g. SHELL's cursor block)
/// that need to draw on top of the console without leaking the private
/// margin / status-bar constants.
pub fn cell_pixel_pos(col: u32, row: u32) -> (u32, u32) {
    let x = MARGIN_X + col * CHAR_W;
    let y = MARGIN_Y + row * CHAR_H;
    (x, y)
}

/// Cell width / height in pixels. Mirrors the font module's constants
/// under the console module's name for external callers that don't
/// want to depend on `font` directly.
pub fn cell_size() -> (u32, u32) {
    (CHAR_W, CHAR_H)
}

/// Map the current console cursor (CURSOR_X, CURSOR_Y) to a pixel
/// position WITHIN `rect`, mirroring how `redraw_in_rect` maps
/// scrollback rows onto rect rows (anchored at the bottom of the
/// visible window). External overlays use this to draw a cursor
/// block at the same position the prompt is rendered.
pub fn cursor_pixel_pos_in_rect(rect: crate::ui::wm::WindowRect) -> (u32, u32) {
    let col = CURSOR_X.load(Ordering::Relaxed);
    let row = CURSOR_Y.load(Ordering::Relaxed) as usize;
    let visible_rows = (rect.h / CHAR_H) as usize;
    if visible_rows == 0 { return (rect.x, rect.y); }
    let last_row = row.min(SB_ROWS.saturating_sub(1));
    let take_rows = visible_rows.min(last_row + 1);
    let start_row = last_row + 1 - take_rows;
    let visible_row_idx = row.saturating_sub(start_row);
    let px = rect.x + col * CHAR_W;
    let py = rect.y + (visible_row_idx as u32) * CHAR_H;
    (px, py)
}

fn write_cell(cx: u32, cy: u32, ch: u8, fg: u32) {
    if (cx as usize) < SB_COLS && (cy as usize) < SB_ROWS {
        unsafe {
            let row_ptr = core::ptr::addr_of_mut!(SB_BUF[cy as usize][cx as usize]);
            core::ptr::write_volatile(row_ptr, Cell { ch, fg });
        }
    }
}

fn clear_cell(cx: u32, cy: u32) {
    if (cx as usize) < SB_COLS && (cy as usize) < SB_ROWS {
        unsafe {
            let row_ptr = core::ptr::addr_of_mut!(SB_BUF[cy as usize][cx as usize]);
            core::ptr::write_volatile(row_ptr, SB_EMPTY);
        }
    }
}

fn scroll_buffer_up() {
    // Shift all rows up by one; clear the bottom row.
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(SB_BUF);
        for r in 1..SB_ROWS {
            buf[r - 1] = buf[r];
        }
        for c in 0..SB_COLS {
            buf[SB_ROWS - 1][c] = SB_EMPTY;
        }
    }
}

fn cols() -> u32 {
    (gpu::width() - MARGIN_X * 2) / CHAR_W
}

fn rows() -> u32 {
    (gpu::height() - MARGIN_Y * 2 - STATUS_BAR_H) / CHAR_H
}

/// Initialize the console: clear screen, draw status bar.
pub fn init() {
    gpu::fill_screen(BG);
    draw_status_bar();
    CURSOR_X.store(0, Ordering::Relaxed);
    CURSOR_Y.store(0, Ordering::Relaxed);
    gpu::flush(0, 0, gpu::width(), gpu::height());
}

/// Initialize console within the window manager frame.
pub fn init_in_window() {
    CURSOR_X.store(0, Ordering::Relaxed);
    CURSOR_Y.store(0, Ordering::Relaxed);
}

/// V12: reset console cursor on cave switch (minor UX / read-pointer leak).
/// also wipe the scrollback buffer so a logged-out cave
/// doesn't leave its command output / typed input visible to the
/// next tenant.
// /
/// the buffer wipe alone left stale
/// pixels on the framebuffer because the desktop only redraws the SH
/// pane on tab-switch. When a `caves enter <name>` ran from the
/// shell, the new (empty) console buffer rendered the cave's prompt
/// at row 0 while the bottom of the screen still showed the old
/// `sphragis >` history that was on the framebuffer before the reset.
/// Confused users into thinking input was going "to a new line above."
/// Now we explicitly fill the active pane with BG and replay the
/// (post-wipe, empty) buffer — which paints the cleared rect and
/// leaves nothing behind. Almost-always on SH at cave-switch time so
/// this fixes the SH artifact; if some other tab happens to be active
/// the wrong pane gets cleared but the next tab-switch repaints it
/// from the app's own state, so no permanent damage.
pub fn reset_for_cave_switch() {
    CURSOR_X.store(0, Ordering::Release);
    CURSOR_Y.store(0, Ordering::Release);
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(SB_BUF);
        for r in 0..SB_ROWS {
            for c in 0..SB_COLS {
                buf[r][c] = SB_EMPTY;
            }
        }
    }
    reset_pen();
    // flush the SH-pane wipe to the framebuffer.
    // Chrome redraw (for the CAVE indicator) is done separately by
    // the caller AFTER `set_active(id)` runs, so the indicator
    // reads the NEW cave name not the old one. See `cave::enter`.
    redraw_content();
}

/// Paint the *tail* of the scrollback buffer into an arbitrary rect.
/// Used by apps that embed a shell view at the bottom of their pane
/// (FS, CM, BC) so the operator can type shell commands in context
/// without swapping to the SH tab.
///
/// We show the last `rect.h / CHAR_H` rows, clipped to `rect.w /
/// CHAR_W` columns. The buffer itself is left untouched.
pub fn redraw_in_rect(rect: crate::ui::wm::WindowRect) {
    let fb = gpu::framebuffer();
    let w = gpu::width();

    // Clear the strip first so we paint over a clean slate.
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, BG);

    let visible_rows = (rect.h / CHAR_H) as usize;
    let visible_cols = (rect.w / CHAR_W) as usize;
    if visible_rows == 0 || visible_cols == 0 { return; }

    // Find the highest non-empty row in the buffer so we know what
    // "the tail" actually is — otherwise we'd waste pixels on
    // never-touched blank rows below the cursor.
    let cur_y = CURSOR_Y.load(Ordering::Relaxed) as usize;
    let last_row = cur_y.min(SB_ROWS.saturating_sub(1));
    let take_rows = visible_rows.min(last_row + 1);
    let start_row = last_row + 1 - take_rows;
    let take_cols = visible_cols.min(SB_COLS);

    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for i in 0..take_rows {
            let src_y = start_row + i;
            let py = rect.y + (i as u32) * CHAR_H;
            if py + CHAR_H > rect.y + rect.h { break; }
            for cx in 0..take_cols {
                let cell = buf[src_y][cx];
                if cell.ch == 0 { continue; }
                let px = rect.x + (cx as u32) * CHAR_W;
                if px + CHAR_W > rect.x + rect.w { break; }
                font::draw_char(fb, w, px, py, cell.ch, cell.fg, BG);
            }
        }
    }
}

/// replay the scrollback buffer to the framebuffer.
/// Called on every tab switch back to SH so the shell content
/// survives the FB clear in `wm::draw_frame`.
pub fn redraw_content() {
    let fb = gpu::framebuffer();
    let w = gpu::width();
    // XXX Wave-2-temp: let pr = crate::ui::wm::content_rect();
    let pr = crate::ui::wm::WindowRect { x: 0, y: 0, w: gpu::width(), h: gpu::height() };

    // Clear the pane content rect so we paint over a clean slate.
    gpu::fill_rect(pr.x, pr.y, pr.w, pr.h, BG);

    // Selection range (when in select mode), expressed inclusive.
    let sel: Option<(usize, usize)> = if select_mode_active() {
        unsafe {
            let a = core::ptr::read_volatile(core::ptr::addr_of!(SEL_ANCHOR)) as usize;
            let c = core::ptr::read_volatile(core::ptr::addr_of!(SEL_CURSOR)) as usize;
            Some((a.min(c), a.max(c)))
        }
    } else { None };

    // Replay every non-empty cell at its absolute (MARGIN_X+col*W, MARGIN_Y+row*H).
    unsafe {
        let buf = &*core::ptr::addr_of!(SB_BUF);
        for cy in 0..SB_ROWS {
            let row_is_selected = sel.map_or(false, |(lo, hi)| cy >= lo && cy <= hi);

            // Paint the row-wide highlight strip first if selected.
            if row_is_selected {
                let py = MARGIN_Y + (cy as u32) * CHAR_H;
                if py >= pr.y && py + CHAR_H <= pr.y + pr.h {
                    gpu::fill_rect(pr.x, py, pr.w, CHAR_H, SELECT_BG);
                }
            }

            for cx in 0..SB_COLS {
                let cell = buf[cy][cx];
                if cell.ch == 0 { continue; }
                let px = MARGIN_X + (cx as u32) * CHAR_W;
                let py = MARGIN_Y + (cy as u32) * CHAR_H;
                if px >= pr.x && py >= pr.y
                    && px + CHAR_W <= pr.x + pr.w
                    && py + CHAR_H <= pr.y + pr.h
                {
                    let (fg, bg) = if row_is_selected {
                        (SELECT_FG, SELECT_BG)
                    } else {
                        (cell.fg, BG)
                    };
                    font::draw_char(fb, w, px, py, cell.ch, fg, bg);
                }
            }
        }
    }
}

fn draw_status_bar() {
    let w = gpu::width();
    let h = gpu::height();
    let bar_y = h - STATUS_BAR_H;
    let fb = gpu::framebuffer();

    // Bar background
    gpu::fill_rect(0, bar_y, w, STATUS_BAR_H, 0xFF0A0A0A);
    // Separator line
    gpu::fill_rect(0, bar_y, w, 1, 0xFF1E1E1E);

    // Status text
    let text_y = bar_y + 8;
    font::draw_str(fb, w, 12, text_y, "ENCRYPTED", ACCENT_GREEN, 0xFF0A0A0A);
    font::draw_str(fb, w, 108, text_y, "|", FG_DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, 120, text_y, "SECURE", ACCENT_GREEN, 0xFF0A0A0A);
    font::draw_str(fb, w, 180, text_y, "|", FG_DIM, 0xFF0A0A0A);
    font::draw_str(fb, w, 192, text_y, "SPHRAGIS v0.3", FG_DIM, 0xFF0A0A0A);
}

/// Print a character to the console.
// /
/// every char also lands in the scrollback buffer with
/// the current pen color so `redraw_content` can replay the screen
/// after the WM clears the FB.
pub fn putc(c: u8) {
    // Output-capture short-circuit: redirect to the capture buffer
    // and skip framebuffer + scrollback paint. Serial mirror still
    // runs so test harnesses and dev observers see the bytes.
    if CAPTURE_ACTIVE.load(Ordering::Acquire) {
        capture_push(core::slice::from_ref(&c));
        if matches!(crate::platform::current(), crate::platform::Platform::QemuVirt) {
            crate::drivers::uart::putc(c);
        }
        return;
    }
    let mut cx = CURSOR_X.load(Ordering::Relaxed);
    let mut cy = CURSOR_Y.load(Ordering::Relaxed);
    let max_cols = cols();
    let max_rows = rows();
    let pen = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEN_COLOR)) };

    match c {
        b'\n' | b'\r' => {
            cx = 0;
            cy += 1;
        }
        0x08 | 0x7F => {
            // Backspace — paint a space, clear the buffer cell.
            if cx > 0 {
                cx -= 1;
                let px = MARGIN_X + cx * CHAR_W;
                let py = MARGIN_Y + cy * CHAR_H;
                font::draw_char(gpu::framebuffer(), gpu::width(), px, py, b' ', BG, BG);
                gpu::flush(px, py, CHAR_W, CHAR_H);
                clear_cell(cx, cy);
            }
        }
        _ => {
            let px = MARGIN_X + cx * CHAR_W;
            let py = MARGIN_Y + cy * CHAR_H;
            font::draw_char(gpu::framebuffer(), gpu::width(), px, py, c, pen, BG);
            gpu::flush(px, py, CHAR_W, CHAR_H);
            // Write to scrollback. Non-printable ASCII stamps as `?`
            // (matches font::draw_char's own out-of-range fallback).
            let stored = if c >= 0x20 && c < 0x7F { c } else { b'?' };
            write_cell(cx, cy, stored, pen);
            cx += 1;
            if cx >= max_cols {
                cx = 0;
                cy += 1;
            }
        }
    }

    // QEMU-only: mirror single-char writes to PL011 so byte-granular
    // output (like `print_num(n)` which calls putc per digit) is visible
    // to test harnesses. Note: shell input-echo also writes via
    // `console::putc(c)` + `platform::serial_putc(c)` — which together
    // produce a doubled char on serial here. That's cosmetic (test
    // harnesses strip duplicates) and worth the tradeoff: without this
    // mirror, inline numbers are invisible to harnesses.
    if matches!(crate::platform::current(), crate::platform::Platform::QemuVirt) {
        crate::drivers::uart::putc(c);
    }

    // Scroll if needed.
    if cy >= max_rows {
        scroll_up();
        scroll_buffer_up();
        cy = max_rows - 1;
    }

    CURSOR_X.store(cx, Ordering::Relaxed);
    CURSOR_Y.store(cy, Ordering::Relaxed);
}

/// Mirror console writes to PL011 on QEMU so test harnesses can observe
/// shell output over serial. On Apple, `fb_console` already mirrors
/// serial→FB so we don't mirror FB→serial (would double-write).
#[inline]
fn mirror_to_serial(s: &str) {
    if matches!(crate::platform::current(), crate::platform::Platform::QemuVirt) {
        crate::drivers::uart::puts(s);
    }
}

// ===========================================================================
// Output-capture mode — gap-audit item 039 (shell pipes / job control).
//
// `begin_capture()` swaps console output from the framebuffer +
// scrollback to a fixed-size byte buffer. Serial mirror still runs
// (so test harnesses + dev debug keep working). `end_capture()`
// returns the buffered bytes and restores normal sink behaviour.
//
// This is the load-bearing primitive behind the shell's `>`
// redirect operator: the shell's `execute` checks for ` > <file>`,
// wraps the inner command in begin/end_capture, then writes the
// captured bytes to BatFS via `batfs::ns_create`.
//
// Future `|` pipes will use the same buffer as the donor of the
// left side and feed it as input to the right side; a handful of
// commands need a buffer-input shape for that to work (hash, etc.)
// — that's the follow-up arc this primitive unblocks.
// ===========================================================================

const CAPTURE_CAP: usize = 32 * 1024;
static mut CAPTURE_BUF: [u8; CAPTURE_CAP] = [0u8; CAPTURE_CAP];
static CAPTURE_LEN:    AtomicUsize = AtomicUsize::new(0);
static CAPTURE_ACTIVE: AtomicBool  = AtomicBool::new(false);

/// Begin capturing console output. While active, `puts` / `putc`
/// writes go to the capture buffer instead of the framebuffer +
/// scrollback. Serial mirror is unchanged (debug visibility kept).
/// Idempotent: re-calling resets the buffer but keeps capture on.
pub fn begin_capture() {
    CAPTURE_LEN.store(0, Ordering::Relaxed);
    CAPTURE_ACTIVE.store(true, Ordering::Release);
}

/// End capture; return the captured bytes as a `'static` slice.
/// The slice remains valid until the next `begin_capture`.
pub fn end_capture() -> &'static [u8] {
    CAPTURE_ACTIVE.store(false, Ordering::Release);
    let n = CAPTURE_LEN.load(Ordering::Relaxed).min(CAPTURE_CAP);
    unsafe {
        let p = core::ptr::addr_of!(CAPTURE_BUF) as *const u8;
        core::slice::from_raw_parts(p, n)
    }
}

/// True iff capture mode is currently active.
pub fn capture_active() -> bool {
    CAPTURE_ACTIVE.load(Ordering::Acquire)
}

fn capture_push(s: &[u8]) {
    let mut head = CAPTURE_LEN.load(Ordering::Relaxed);
    if head >= CAPTURE_CAP { return; }
    let take = s.len().min(CAPTURE_CAP - head);
    unsafe {
        let dst = core::ptr::addr_of_mut!(CAPTURE_BUF) as *mut u8;
        core::ptr::copy_nonoverlapping(s.as_ptr(), dst.add(head), take);
    }
    head += take;
    CAPTURE_LEN.store(head, Ordering::Relaxed);
}

/// Print a string to the console.
pub fn puts(s: &str) {
    if CAPTURE_ACTIVE.load(Ordering::Acquire) {
        capture_push(s.as_bytes());
        mirror_to_serial(s);
        return;
    }
    for b in s.bytes() {
        putc(b);
    }
    mirror_to_serial(s);
}

/// Print a string in highlight color.
// /
/// route through `putc` so the cells land in the
/// scrollback buffer with the FG_HI color. Pre-fix this drew
/// directly via `font::draw_char` and bypassed the buffer, so
/// banner / prompt content vanished on tab switch.
pub fn puts_hi(s: &str) {
    set_pen(FG_HI);
    for b in s.bytes() { putc(b); }
    reset_pen();
    mirror_to_serial(s);
}

/// Print a string in an arbitrary pen color. Resets pen to FG after.
pub fn puts_color(s: &str, color: u32) {
    set_pen(color);
    for b in s.bytes() { putc(b); }
    reset_pen();
    mirror_to_serial(s);
}

/// Print the shell prompt.
// /
/// switched from direct `font::draw_str` calls to
/// `putc`-via-pen so the prompt cells land in the scrollback buffer.
/// Without this the prompt vanished from the screen the moment the
/// WM cleared the FB on tab switch.
pub fn prompt() {
    set_pen(FG_HI);
    for b in b"sphragis" { putc(*b); }
    set_pen(ACCENT_CYAN);
    for b in b" > " { putc(*b); }
    reset_pen();
    mirror_to_serial("sphragis > ");
}

fn scroll_up() {
    // Move framebuffer content up by one text row
    let fb = gpu::framebuffer();
    let w = gpu::width() as usize;
    let row_pixels = CHAR_H as usize;
    let start_y = MARGIN_Y as usize;
    let end_y = (gpu::height() - STATUS_BAR_H) as usize;

    unsafe {
        for y in start_y..(end_y - row_pixels) {
            let src = (y + row_pixels) * w;
            let dst = y * w;
            core::ptr::copy(fb.add(src), fb.add(dst), w);
        }
        // Clear last row
        for y in (end_y - row_pixels)..end_y {
            for x in 0..w {
                core::ptr::write_volatile(fb.add(y * w + x), BG);
            }
        }
    }
    gpu::flush(0, 0, gpu::width(), gpu::height());
}
