// Sphragis — Secure Boot Screen
//
// Wave 1 lock-screen redesign (May 2026). See
// docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md.
// The previous design (terminal-cyberpunk + operator-tactical:
// status pills, boot log, clock block, crosshair corners,
// ACCESS DENIED overlay) is gone. The new screen is pure mono:
//
// * 5-color palette (BG, PANEL, HAIRLINE, INK, MID). No state accent.
// * Centered stack: Σ glyph (Plex Serif Italic 96px) + SPHRAGIS
//   wordmark (Plex Sans Medium 14px, letterspaced) + 480x56 field.
// * Three observable states: Idle, Typing(n), Granted(n).
//   Denied + LockedOut paint Idle to keep the screen pixel-identical
//   across success/failure boundaries — see `run()` for the silent-
//   denial and silent-lockout state machine.
//
// CRITICAL color note: the QEMU framebuffer is FORMAT_B8G8R8A8 and
// we write u32s in `(A<<24)|(R<<16)|(G<<8)|B` form, so the LE store
// lands as B,G,R,A in memory. Hex literals like 0xFFE5E7EB map
// directly to "alpha=FF, R=E5, G=E7, B=EB" — same ordering CSS uses.

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::draw;
use crate::platform;
use super::{auth, wipe, deadman};

/// Wall-clock hold via the ARM generic timer. Spin counts are CPU-rate-
/// dependent and turned the denied/granted flashes into 1-frame blinks
/// on M4 — use cntpct_el0 / cntfrq_el0 instead so the duration is real
/// seconds regardless of clock speed.
fn hold_ms(ms: u64) {
    let freq: u64;
    let start: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
    }
    let target_ticks = (freq / 1000).saturating_mul(ms);
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target_ticks { break; }
        core::hint::spin_loop();
    }
}

/// How long the screen holds in each transient state. Tuned for
/// "long enough to read the message" on a denial, "short enough not
/// to feel laggy" on success.
const HOLD_DENIED_MS:  u64 = 2500;
const HOLD_GRANTED_MS: u64 = 200;

// ─── Palette (Wave 1 lock-screen redesign — pure mono) ─────────────────
//
// 5 named colors. No state-color overlay. See
// docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md.
// Hex encoding is `(A<<24)|(R<<16)|(G<<8)|B` per the framebuffer note
// at the top of this file.
const BG:       u32 = 0xFF0D0D10; // background
const PANEL:    u32 = 0xFF18181C; // field panel
const HAIRLINE: u32 = 0xFF2A2A30; // field border
const INK:      u32 = 0xFFE5E7EB; // primary text + glyph
const MID:      u32 = 0xFF6B7280; // reserved (unused in v1; see spec)

// ─── Layout constants (1280x800 native; scales with the FB size) ──────

const FIELD_W:  u32 = 480;
const FIELD_H:  u32 = 56;
const DOT_PX:   u32 = 8;
const DOT_GAP:  u32 = 8;

// Σ glyph: 96-px cap height, rendered via Plex Serif Italic.
const GLYPH_SIZE_PX:    u16 = 96;
const WORDMARK_SIZE_PX: u16 = 14;

// Stack offset: how far above screen vertical center the glyph baseline
// sits. ~80 px keeps the composition off dead-middle without crowding
// the top of the screen.
const STACK_ABOVE_CENTER: u32 = 80;

// Wordmark sits 16 px below the glyph baseline; field sits 40 px below
// the wordmark.
const WORDMARK_GAP: u32 = 16;
const FIELD_GAP:    u32 = 40;

// ─── Top-level paint of the lock screen ─────────────────────────────────

/// State the screen is being painted in.
///
/// `Denied` is reserved (never constructed under the Wave 1 redesign —
/// Failed paints Idle silently, see `run()`), but kept in the enum so
/// the painter's match arms stay exhaustive over the full state
/// vocabulary should we revisit denial UI later.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LockState { Idle, Typing(u8), Denied, Granted(u8) }

fn paint_lock_screen(fb: *mut u32, w: u32, h: u32, state: LockState, _attempts: u8) {
    // Background — single solid fill, nothing layered on top except the
    // glyph / wordmark / field.
    gpu::fill_screen(BG);

    let cx = (w / 2) as i32;
    let cy = (h / 2) as i32;

    // ── Σ glyph (Plex Serif Italic, 96 px) ────────────────────────────
    //
    // Centered horizontally. Vertical position chosen so the glyph
    // baseline sits STACK_ABOVE_CENTER above screen vertical center.
    // truetype::draw_glyph expects (x, y) as the top-left of the glyph
    // bitmap, so we measure the glyph's advance to center it, and
    // approximate its visual height as size_px (close enough for the
    // single-glyph layout — anti-aliased descender / ascender don't
    // matter at this scale).
    let glyph_advance = crate::ui::truetype::glyph_advance(
        0x03A3, // Σ
        crate::ui::truetype::FontFace::PlexSerifItalic,
        GLYPH_SIZE_PX,
    );
    let glyph_x = cx - glyph_advance / 2;
    let glyph_y = cy - STACK_ABOVE_CENTER as i32 - GLYPH_SIZE_PX as i32;
    crate::ui::truetype::draw_glyph(
        fb, w, h,
        glyph_x, glyph_y,
        0x03A3,
        crate::ui::truetype::FontFace::PlexSerifItalic,
        GLYPH_SIZE_PX,
        INK,
    );

    // ── Wordmark "SPHRAGIS" (Plex Sans Medium, 14 px, letterspaced) ──
    //
    // Plex Sans doesn't have built-in letterspacing — we add it by
    // drawing each character separately with a fixed extra advance.
    // 0.4em letterspacing at 14px = ~5.6 px per gap; round to 6.
    const WORDMARK_LETTERSPACE: i32 = 6;
    let wordmark = "SPHRAGIS";
    let base_advance = crate::ui::truetype::text_advance(
        wordmark,
        crate::ui::truetype::FontFace::PlexSansMedium,
        WORDMARK_SIZE_PX,
    );
    let extra = (wordmark.len() as i32 - 1) * WORDMARK_LETTERSPACE;
    let wordmark_total_w = base_advance + extra;
    let mut wordmark_x = cx - wordmark_total_w / 2;
    let wordmark_y = glyph_y + GLYPH_SIZE_PX as i32 + WORDMARK_GAP as i32;
    for ch in wordmark.chars() {
        let adv = crate::ui::truetype::draw_glyph(
            fb, w, h,
            wordmark_x, wordmark_y,
            ch as u32,
            crate::ui::truetype::FontFace::PlexSansMedium,
            WORDMARK_SIZE_PX,
            INK,
        );
        wordmark_x += adv + WORDMARK_LETTERSPACE;
    }

    // ── Field panel ──────────────────────────────────────────────────
    let field_x = (w / 2).saturating_sub(FIELD_W / 2);
    let field_y = (wordmark_y as u32) + WORDMARK_SIZE_PX as u32 + FIELD_GAP;
    gpu::fill_rect(field_x, field_y, FIELD_W, FIELD_H, PANEL);
    draw::draw_border(field_x, field_y, FIELD_W, FIELD_H, HAIRLINE);

    // ── Typed dots (Typing / Granted only). Denied paints same as Idle. ──
    let dots: u8 = match state {
        LockState::Typing(n)  => n,
        LockState::Granted(n) => n,
        // Idle + Denied: zero dots — pixel-identical screens.
        _ => 0,
    };
    if dots > 0 {
        // Center the dot strip horizontally inside the field.
        let strip_w = (dots as u32) * DOT_PX + (dots.saturating_sub(1) as u32) * DOT_GAP;
        let dots_x = field_x + (FIELD_W - strip_w) / 2;
        let dots_y = field_y + (FIELD_H - DOT_PX) / 2;
        for i in 0..dots {
            let dx = dots_x + (i as u32) * (DOT_PX + DOT_GAP);
            gpu::fill_rect(dx, dots_y, DOT_PX, DOT_PX, INK);
        }
    }
}

// ─── Public entry points (preserved API) ────────────────────────────────

/// Dev helper: paint the login screen exactly once, then return after a
/// fixed delay. Lets the operator screenshot the auth UI without needing
/// a real passphrase. Kept for the Apple HV preview path.
#[allow(dead_code)]
pub fn run_dev_preview(hold_ms: u64) {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    paint_lock_screen(fb, w, h, LockState::Idle, 5);
    gpu::flush(0, 0, w, h);

    let freq: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq); }
    let start: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) start); }
    let target = (freq / 1000) * hold_ms;
    loop {
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now.wrapping_sub(start) >= target { break; }
        core::hint::spin_loop();
    }
}

/// Run the boot authentication screen.
/// Returns only on successful authentication.
/// On duress or lockout, never returns.
pub fn run() {
    platform::serial_puts("[bs] enter run\n");
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    platform::serial_puts("[bs] fb obtained\n");

    loop {
        let attempts = auth::attempts_remaining();
        platform::serial_puts("[bs] paint idle\n");
        paint_lock_screen(fb, w, h, LockState::Idle, attempts);
        gpu::flush(0, 0, w, h);
        platform::serial_puts("[bs] paint done — input loop\n");

        let mut buf = [0u8; 128];
        let mut len = 0usize;

        loop {
            // + #112 keyboard plumbing — drain serial,
            // virtio-keyboard, AND the pointer-device's mis-routed
            // EV_KEY ring.
            crate::drivers::virtio::keyboard::poll();
            crate::drivers::virtio::tablet::poll();
            let c_opt = platform::serial_getc()
                .or_else(crate::drivers::virtio::keyboard::getc)
                .or_else(crate::drivers::virtio::tablet::getc_key);
            if let Some(c) = c_opt {
                match c {
                    b'\r' | b'\n' => break,
                    0x08 | 0x7F => {
                        if len > 0 {
                            len -= 1;
                            // Repaint with one fewer dot.
                            let s = if len == 0 { LockState::Idle } else { LockState::Typing(len as u8) };
                            paint_lock_screen(fb, w, h, s, attempts);
                            gpu::flush(0, 0, w, h);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && len < 127 => {
                        buf[len] = c;
                        len += 1;
                        // Repaint with one more dot. This is full-frame
                        // every keystroke which is overkill, but it
                        // matches the design and types-per-second is
                        // human-rate so it's fine.
                        paint_lock_screen(fb, w, h, LockState::Typing(len as u8), attempts);
                        gpu::flush(0, 0, w, h);
                    }
                    _ => {}
                }
            }
            core::hint::spin_loop();
        }

        if len == 0 { continue; }

        let input = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
        let result = auth::authenticate(input);

        match result {
            auth::AuthResult::Success => {
                paint_lock_screen(fb, w, h, LockState::Granted(len as u8), attempts);
                gpu::flush(0, 0, w, h);
                hold_ms(HOLD_GRANTED_MS);
                deadman::refresh();
                return;
            }
            auth::AuthResult::Failed => {
                // Silent denial. Three things in order:
                //   1. Paint Idle *immediately* so dots vanish the instant
                //      Enter is pressed (otherwise an attacker can time
                //      the field reset).
                //   2. Hold for HOLD_DENIED_MS, draining any keystrokes
                //      typed during the cooldown so they don't burst-paint
                //      when input resumes (which would leak the cooldown
                //      end time).
                //   3. Continue the outer loop, which repaints Idle
                //      (harmless redundant paint).
                paint_lock_screen(fb, w, h, LockState::Idle, attempts);
                gpu::flush(0, 0, w, h);

                let freq: u64;
                let start: u64;
                unsafe {
                    core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
                    core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
                }
                let target_ticks = (freq / 1000).saturating_mul(HOLD_DENIED_MS);
                loop {
                    let now: u64;
                    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
                    if now.wrapping_sub(start) >= target_ticks { break; }
                    // Drain + discard any keys typed during the cooldown.
                    crate::drivers::virtio::keyboard::poll();
                    crate::drivers::virtio::tablet::poll();
                    let _ = platform::serial_getc()
                        .or_else(crate::drivers::virtio::keyboard::getc)
                        .or_else(crate::drivers::virtio::tablet::getc_key);
                    core::hint::spin_loop();
                }
                continue;
            }
            auth::AuthResult::Duress => {
                fake_boot_and_wipe(fb, w, h);
            }
            auth::AuthResult::LockedOut => {
                // Silent lockout. Clear the dots before starting the wipe
                // for the same reason as Failed — otherwise the final
                // failed attempt is the only one where dots stay frozen
                // on screen, which is itself a signal. Then no banner,
                // straight to wipe, then halt.
                paint_lock_screen(fb, w, h, LockState::Idle, 0);
                gpu::flush(0, 0, w, h);
                wipe::execute(wipe::WipeReason::Lockout, false);
                loop { unsafe { core::arch::asm!("wfe") }; }
            }
        }
    }
}

/// Fake boot animation — attacker thinks the system is loading.
/// Behind the scenes, everything is being destroyed.
fn fake_boot_and_wipe(fb: *mut u32, w: u32, h: u32) {
    let cx = w / 2;
    let cy = h / 2;

    gpu::fill_screen(BG);
    let title = "SPHRAGIS";
    // 8 = bitmap font cell width (was CHAR_W; the new lock screen uses
    // TrueType so the constant is gone, but fake_boot_and_wipe still
    // paints with the legacy 8x16 bitmap font).
    let t_x = cx - (title.len() as u32 * 8) / 2;
    font::draw_str(fb, w, t_x, cy - 40, title, INK, BG);
    let loading = "LOADING SYSTEM ...";
    let l_x = cx - (loading.len() as u32 * 8) / 2;
    font::draw_str(fb, w, l_x, cy, loading, MID, BG);

    let bar_x = cx - 100;
    let bar_y = cy + 30;
    let bar_w: u32 = 200;
    let bar_h: u32 = 12;
    draw::draw_border(bar_x, bar_y, bar_w, bar_h, HAIRLINE);
    gpu::flush(0, 0, w, h);

    wipe::execute(wipe::WipeReason::Duress, true);

    for progress in 0..bar_w {
        gpu::fill_rect(bar_x + 1, bar_y + 1, progress, bar_h - 2, INK);
        let pct = (progress * 100) / bar_w;
        let mut pct_str = *b"   %";
        if pct >= 100 { pct_str[0] = b'1'; pct_str[1] = b'0'; pct_str[2] = b'0'; }
        else if pct >= 10 { pct_str[1] = b'0' + (pct / 10) as u8; pct_str[2] = b'0' + (pct % 10) as u8; }
        else { pct_str[2] = b'0' + pct as u8; }
        font::draw_str(fb, w, cx - 16, bar_y + bar_h + 16,
            unsafe { core::str::from_utf8_unchecked(&pct_str) }, MID, BG);
        gpu::flush(bar_x, bar_y, bar_w + 2, bar_h + 32);
        for _ in 0..200_000 { core::hint::spin_loop(); }
    }
    for _ in 0..5_000_000 { core::hint::spin_loop(); }

    gpu::fill_screen(BG);
    font::draw_str(fb, w, 16, 16, "panic: unable to mount root filesystem", INK, BG);
    font::draw_str(fb, w, 16, 32, "kernel: VFS: unable to mount root fs",   INK, BG);
    font::draw_str(fb, w, 16, 48, "---[ end Kernel panic - not syncing ]---", INK, BG);
    gpu::flush(0, 0, w, h);

    loop { unsafe { core::arch::asm!("wfe") }; }
}
