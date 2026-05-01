// Bat_OS — Secure Boot Screen
// The authentication gate rendered on the GPU.
// Displays passphrase prompt, handles input, manages auth flow.
//
// Visual design:
// - Minimal black screen, bat emblem, passphrase field
// - No version info, no hints, no information leakage
// - Wrong attempts show remaining count
// - Duress code triggers fake boot animation + silent wipe

use crate::ui::gpu;
use crate::ui::font;
use crate::platform;
use super::{auth, wipe, deadman};

const BLACK: u32 = 0xFF000000;
const WHITE: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF3A3A3A;
const RED: u32 = 0xFF0000FF;
const GREEN: u32 = 0xFF00FF00;

/// Dev helper: paint the login screen exactly once (same design as
/// `run`), then return after a fixed delay. Lets the operator see /
/// screenshot the auth UI without needing a real passphrase. Use on
/// the Apple HV path during development where auth isn't the focus.
pub fn run_dev_preview(hold_ms: u64) {
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();

    gpu::fill_screen(BLACK);
    let cx = w / 2;
    let cy = h / 2 - 80;
    draw_bat(fb, w, cx, cy - 40);
    font::draw_str(fb, w, cx - 28, cy + 20, "BAT_OS", WHITE, BLACK);
    font::draw_str(fb, w, cx - 80, cy + 80, "PASSPHRASE:", DIM, BLACK);
    let field_x = cx - 120;
    let field_y = cy + 100;
    let field_w = 240u32;
    let field_h = 24u32;
    gpu::fill_rect(field_x, field_y, field_w, 1, DIM);
    gpu::fill_rect(field_x, field_y + field_h, field_w, 1, DIM);
    gpu::fill_rect(field_x, field_y, 1, field_h, DIM);
    gpu::fill_rect(field_x + field_w, field_y, 1, field_h, DIM);
    font::draw_str(fb, w, cx - 80, cy + 140, "YUBIKEY:", DIM, BLACK);
    font::draw_str(fb, w, cx, cy + 140, "[DEV PREVIEW]", DIM, BLACK);
    gpu::flush(0, 0, w, h);

    // Busy-wait with CNTPCT so we hold the screen long enough to be
    // captured via `screen` or a phone photo.
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
    // Route debug traces through the platform-neutral serial so this
    // works on both QEMU (PL011) and Apple M4 (dockchannel). Direct
    // `drivers::apple::uart::puts` here used to fault on QEMU because
    // the M4 dockchannel MMIO is unmapped there.
    platform::serial_puts("[bs] enter run\n");
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    platform::serial_puts("[bs] fb obtained\n");

    loop {
        // Draw the auth screen
        platform::serial_puts("[bs] fill_screen\n");
        gpu::fill_screen(BLACK);
        platform::serial_puts("[bs] fill_screen done\n");

        let cx = w / 2;
        let cy = h / 2 - 80;

        // Bat emblem (simplified)
        platform::serial_puts("[bs] draw_bat\n");
        draw_bat(fb, w, cx, cy - 40);
        platform::serial_puts("[bs] draw_bat done\n");

        // "BAT_OS" text
        platform::serial_puts("[bs] draw BAT_OS title\n");
        font::draw_str(fb, w, cx - 28, cy + 20, "BAT_OS", WHITE, BLACK);

        // Passphrase prompt
        font::draw_str(fb, w, cx - 80, cy + 80, "PASSPHRASE:", DIM, BLACK);

        // Input field border
        platform::serial_puts("[bs] field border\n");
        let field_x = cx - 120;
        let field_y = cy + 100;
        let field_w = 240u32;
        let field_h = 24u32;
        gpu::fill_rect(field_x, field_y, field_w, 1, DIM);
        gpu::fill_rect(field_x, field_y + field_h, field_w, 1, DIM);
        gpu::fill_rect(field_x, field_y, 1, field_h, DIM);
        gpu::fill_rect(field_x + field_w, field_y, 1, field_h, DIM);

        // YubiKey status
        platform::serial_puts("[bs] yubikey label\n");
        font::draw_str(fb, w, cx - 80, cy + 140, "YUBIKEY:", DIM, BLACK);
        font::draw_str(fb, w, cx, cy + 140, "[SIMULATED OK]", DIM, BLACK);

        // Attempts remaining
        platform::serial_puts("[bs] attempts\n");
        let remaining = auth::attempts_remaining();
        if remaining < 5 {
            font::draw_str(fb, w, cx - 60, cy + 180, "ATTEMPTS:", RED, BLACK);
            let ch = b'0' + remaining;
            font::draw_char(fb, w, cx + 20, cy + 180, ch, RED, BLACK);
        }

        platform::serial_puts("[bs] flush\n");
        gpu::flush(0, 0, w, h);
        platform::serial_puts("[bs] flush done — entering input loop\n");

        // Read passphrase
        let mut buf = [0u8; 128];
        let mut len = 0usize;
        let mut cursor_x = field_x + 4;

        loop {
            // STUMP #99: read from BOTH the host serial console
            // (terminal) AND the virtio-keyboard (QEMU GUI window).
            // Pre-fix the boot screen ignored GUI typing entirely —
            // a Mac user clicking into the QEMU window and typing
            // their passphrase saw zero feedback and assumed input
            // was broken. We pump the virtio-keyboard each pass so
            // events from the GUI window land in the keystroke ring,
            // then prefer serial (already-buffered when called from
            // the kernel shell) and fall through to the GUI ring.
            crate::drivers::virtio::keyboard::poll();
            let c_opt = platform::serial_getc()
                .or_else(crate::drivers::virtio::keyboard::getc);
            if let Some(c) = c_opt {
                // Diagnostic: log every char the boot screen receives.
                // Also echo to the terminal so the user can see typing
                // is being delivered (the GUI dots are easy to miss).
                platform::serial_puts("[bs] got char ");
                crate::kernel::mm::print_num(c as usize);
                platform::serial_puts("\n");
                match c {
                    b'\r' | b'\n' => break,
                    0x08 | 0x7F => {
                        if len > 0 {
                            len -= 1;
                            cursor_x -= 8;
                            // Erase dot
                            gpu::fill_rect(cursor_x, field_y + 4, 8, 16, BLACK);
                            gpu::flush(cursor_x, field_y + 4, 8, 16);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && len < 127 => {
                        buf[len] = c;
                        len += 1;
                        // Show dot (not the actual character — no screen shoulder surfing)
                        font::draw_char(fb, w, cursor_x, field_y + 4, 0x07, WHITE, BLACK);
                        cursor_x += 8;
                        gpu::flush(cursor_x - 8, field_y + 4, 8, 16);
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
                // Auth passed — show brief success message
                gpu::fill_rect(field_x, field_y, field_w, field_h + 2, BLACK);
                font::draw_str(fb, w, cx - 48, field_y + 4, "ACCESS GRANTED", GREEN, BLACK);
                gpu::flush(0, 0, w, h);

                // Brief pause so user sees "GRANTED"
                for _ in 0..5_000_000 { core::hint::spin_loop(); }

                // Refresh dead man's switch
                deadman::refresh();

                return; // Proceed to desktop
            }
            auth::AuthResult::Failed => {
                // Show error
                gpu::fill_rect(field_x, field_y, field_w, field_h + 2, BLACK);
                font::draw_str(fb, w, cx - 48, field_y + 4, "ACCESS DENIED", RED, BLACK);
                gpu::flush(0, 0, w, h);

                for _ in 0..3_000_000 { core::hint::spin_loop(); }

                // Loop back to prompt
                continue;
            }
            auth::AuthResult::Duress => {
                // DURESS — show fake boot, wipe silently
                fake_boot_and_wipe(fb, w, h);
                // Never returns
            }
            auth::AuthResult::LockedOut => {
                // MAX ATTEMPTS — wipe everything
                gpu::fill_rect(field_x, field_y, field_w, field_h + 2, BLACK);
                font::draw_str(fb, w, cx - 60, field_y + 4, "SYSTEM LOCKED", RED, BLACK);
                gpu::flush(0, 0, w, h);

                wipe::execute(wipe::WipeReason::Lockout, false);

                // Halt — system is dead
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

    // Clear screen
    gpu::fill_screen(BLACK);

    // Show fake "BAT_OS" boot
    font::draw_str(fb, w, cx - 28, cy - 40, "BAT_OS", WHITE, BLACK);
    font::draw_str(fb, w, cx - 64, cy, "Loading system...", DIM, BLACK);

    // Fake progress bar
    let bar_x = cx - 100;
    let bar_y = cy + 30;
    let bar_w = 200u32;
    let bar_h = 12u32;

    // Border
    gpu::fill_rect(bar_x, bar_y, bar_w, 1, DIM);
    gpu::fill_rect(bar_x, bar_y + bar_h, bar_w, 1, DIM);
    gpu::fill_rect(bar_x, bar_y, 1, bar_h, DIM);
    gpu::fill_rect(bar_x + bar_w, bar_y, 1, bar_h, DIM);

    gpu::flush(0, 0, w, h);

    // Slowly fill progress bar while wiping in background
    // Wipe is triggered silently
    wipe::execute(wipe::WipeReason::Duress, true);

    // Animate progress bar (system is already destroyed)
    for progress in 0..bar_w {
        gpu::fill_rect(bar_x + 1, bar_y + 1, progress, bar_h - 1, WHITE);

        // Update percentage
        let pct = (progress * 100) / bar_w;
        let mut pct_str = [b' ', b' ', b' ', b'%'];
        if pct >= 100 { pct_str[0] = b'1'; pct_str[1] = b'0'; pct_str[2] = b'0'; }
        else if pct >= 10 { pct_str[1] = b'0' + (pct / 10) as u8; pct_str[2] = b'0' + (pct % 10) as u8; }
        else { pct_str[2] = b'0' + pct as u8; }

        font::draw_str(fb, w, cx - 16, bar_y + bar_h + 16,
            unsafe { core::str::from_utf8_unchecked(&pct_str) }, DIM, BLACK);

        gpu::flush(bar_x, bar_y, bar_w + 2, bar_h + 32);

        // Slow it down — make it look real
        for _ in 0..200_000 { core::hint::spin_loop(); }
    }

    // "Crash" the fake boot — attacker thinks it's broken
    for _ in 0..5_000_000 { core::hint::spin_loop(); }

    // Show fake kernel panic
    gpu::fill_screen(BLACK);
    font::draw_str(fb, w, 16, 16, "panic: unable to mount root filesystem", WHITE, BLACK);
    font::draw_str(fb, w, 16, 32, "kernel: VFS: unable to mount root fs", WHITE, BLACK);
    font::draw_str(fb, w, 16, 48, "---[ end Kernel panic - not syncing ]---", WHITE, BLACK);
    gpu::flush(0, 0, w, h);

    // System is dead — data is gone — halt forever
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}

fn draw_bat(_fb: *mut u32, _w: u32, cx: u32, cy: u32) {
    // Small bat silhouette
    gpu::fill_rect(cx - 8, cy - 5, 16, 20, WHITE);
    gpu::fill_rect(cx - 6, cy - 12, 12, 8, WHITE);
    gpu::fill_rect(cx - 8, cy - 16, 4, 6, WHITE);
    gpu::fill_rect(cx + 4, cy - 16, 4, 6, WHITE);
    gpu::fill_rect(cx - 50, cy - 2, 42, 5, WHITE);
    gpu::fill_rect(cx + 8, cy - 2, 42, 5, WHITE);
    gpu::fill_rect(cx - 40, cy - 6, 30, 4, WHITE);
    gpu::fill_rect(cx + 10, cy - 6, 30, 4, WHITE);
    gpu::fill_rect(cx - 30, cy + 3, 22, 4, WHITE);
    gpu::fill_rect(cx + 8, cy + 3, 22, 4, WHITE);
}
