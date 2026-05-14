// Sphragis — Desktop Environment
// Main event loop. Handles keyboard input, app switching, rendering.
// Ctrl+1-5 switches between apps.
// XXX Wave-2-temp: 65 old-WM call sites commented out, restored in Task 7.

use crate::platform;
use crate::security;
#[allow(unused_imports)]
use super::{wm, console, apps};

/// Resume desktop after Cave exit — skip reinitialization.
pub fn resume() -> ! {
    let in_shell = true;
    let mut cmd_buf = [0u8; 256];
    let mut cmd_len: usize = 0;

    // Just show prompt and continue
    console::prompt();

    loop {
        // same dual-source read as desktop::run. Without
        // this, after a cave exit the resumed shell wouldn't accept
        // QEMU-window keystrokes either.
        // ALSO drain tablet/mouse key ring — pointer
        // devices steal EV_KEY events on QEMU. See uart.rs for the
        // full explanation.
        crate::drivers::virtio::keyboard::poll();
        crate::drivers::virtio::tablet::poll();
        let next_char = platform::serial_getc()
            .or_else(crate::drivers::virtio::keyboard::getc)
            .or_else(crate::drivers::virtio::tablet::getc_key);
        if let Some(c) = next_char {
            if security::check_panic_hotkey(c) {
                loop { unsafe { core::arch::asm!("wfe") }; }
            }
            security::periodic_check();

            match c {
                // XXX Wave-2-temp: switch_to(wm::APP_SHELL); in_shell = true;
                // XXX Wave-2-temp: switch_to(wm::APP_DASHBOARD); in_shell = false;
                // XXX Wave-2-temp: switch_to(wm::APP_NETMON); in_shell = false;
                // XXX Wave-2-temp: switch_to(wm::APP_EDITOR); in_shell = false;
                // XXX Wave-2-temp: let next = (wm::active_app() + 1) % 9; switch_to(next); in_shell = next == wm::APP_SHELL;
                0x01 | 0x02 | 0x04 | 0x05 | 0x09 => { let _ = in_shell; continue; }
                _ => {}
            }

            if in_shell {
                match c {
                    b'\r' | b'\n' => {
                        console::putc(b'\n');
                        platform::serial_puts("\r\n");
                        if cmd_len > 0 {
                            let cmd = unsafe { core::str::from_utf8_unchecked(&cmd_buf[..cmd_len]) };
                            super::shell::execute_cmd(cmd);
                            cmd_len = 0;
                        }
                        console::prompt();
                        // XXX Wave-2-temp: wm::flush_all();
                    }
                    0x08 | 0x7F => {
                        if cmd_len > 0 {
                            cmd_len -= 1;
                            console::putc(0x08);
                            platform::serial_putc(0x08); platform::serial_putc(b' '); platform::serial_putc(0x08);
                            // force fullscreen flush —
                            // small per-rect flushes don't reach
                            // QEMU's host display on Mac cocoa.
                            // XXX Wave-2-temp: wm::flush_all();
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        platform::serial_putc(c);
                        // XXX Wave-2-temp: wm::flush_all();
                    }
                    _ => {}
                }
            }
        }
        core::hint::spin_loop();
    }
}

/// Main desktop loop — runs forever.
pub fn run() -> ! {
    // Initialize pane system + render shell
    // XXX Wave-2-temp: wm::init_panes_pub();
    // XXX Wave-2-temp: wm::switch_app(wm::APP_SHELL);
    render_current();

    let mut in_shell = true;
    let mut cmd_buf = [0u8; 256];
    let mut cmd_len: usize = 0;

    // Partial-input scrubber. Called at every tab-switch site so the
    // user can't carry a half-typed command from one tab into another
    // (the buffer is shared between SH/FS/CM/BC; without this it was
    // possible to start typing on FS, switch to SH, and see your FS
    // partial sitting at the new prompt). The visible characters get
    // backspaced from scrollback so what the user sees matches the
    // empty buffer state.
    macro_rules! clear_input {
        () => {
            // If the user was mid-selection on SH, bailing out of
            // the tab implicitly cancels the selection — without
            // this, SELECT_MODE persists across tab switches and
            // confuses the next return to SH.
            if console::select_mode_active() {
                console::exit_select_mode();
            }
            for _ in 0..cmd_len {
                console::putc(0x08);
                platform::serial_putc(0x08);
                platform::serial_putc(b' ');
                platform::serial_putc(0x08);
            }
            cmd_len = 0;
        };
    }

    // Inject clipboard bytes into the current input as if the user
    // had typed them. Each printable byte goes through the same
    // putc + serial-echo path as a real keystroke; non-printable
    // bytes are skipped (clipboard bytes injected as keystrokes
    // shouldn't be able to deliver a hidden Ctrl-something).
    macro_rules! paste_at_input {
        () => {
            let n = super::clipboard::len();
            for i in 0..n {
                let b = super::clipboard::byte_at(i).unwrap_or(0);
                if b >= 0x20 && b <= 0x7E && cmd_len < 255 {
                    cmd_buf[cmd_len] = b;
                    cmd_len += 1;
                    console::putc(b);
                    platform::serial_putc(b);
                }
            }
            // XXX Wave-2-temp: wm::flush_all();
        };
    }

    // Draw initial shell prompt
    console::init_in_window();
    shell_banner();
    console::prompt();
    // XXX Wave-2-temp: wm::flush_all();

    loop {
        // Check for keyboard input from EITHER serial (host terminal)
        // or virtio-keyboard (QEMU GUI window). pre-fix
        // only serial was read, so a Mac user typing into the QEMU
        // window saw zero feedback. Pump virtio events first so they
        // land in the keystroke ring, then prefer serial.
        // drain tablet/mouse key ring too (QEMU pointer
        // devices steal EV_KEY).
        crate::drivers::virtio::keyboard::poll();
        crate::drivers::virtio::tablet::poll();
        let next_char = platform::serial_getc()
            .or_else(crate::drivers::virtio::keyboard::getc)
            .or_else(crate::drivers::virtio::tablet::getc_key);
        if let Some(c) = next_char {
            // Check for Ctrl+1-5 (switch apps)
            if c == 0x11 { // Ctrl+Q (or we use raw codes)
                // Alternative: use Escape sequences
            }

            // PANIC HOTKEY: Ctrl+W = instant wipe
            if security::check_panic_hotkey(c) {
                loop { unsafe { core::arch::asm!("wfe") }; }
            }

            // Periodic security check (dead man's switch)
            security::periodic_check();

            match c {
                // Ctrl+A through Ctrl+E for app switching
                // XXX Wave-2-temp: 0x01 => { clear_input!(); switch_to(wm::APP_SHELL); in_shell = true; continue; }
                // XXX Wave-2-temp: 0x02 => { clear_input!(); switch_to(wm::APP_DASHBOARD); in_shell = false; continue; }
                0x03 => { // Ctrl+C — if in shell, cancel line; otherwise noop (app switch removed)
                    if in_shell && cmd_len > 0 {
                        console::puts("^C\n");
                        cmd_len = 0;
                        console::prompt();
                        // XXX Wave-2-temp: wm::flush_all();
                        continue;
                    } else if !in_shell {
                        clear_input!();
                        // XXX Wave-2-temp: switch_to(wm::APP_FILES);
                        in_shell = false;
                        continue;
                    }
                }
                // XXX Wave-2-temp: 0x04 => { clear_input!(); switch_to(wm::APP_NETMON); in_shell = false; continue; }
                // XXX Wave-2-temp: 0x05 => { clear_input!(); switch_to(wm::APP_EDITOR); in_shell = false; continue; }
                0x01 | 0x02 | 0x04 | 0x05 => { clear_input!(); continue; }

                // Tab key — cycle focus (floating WM: wm::cycle_focus in Task 4)
                0x09 => {
                    platform::serial_puts("[tab] received\r\n");
                    // XXX Wave-2-temp: if wm::is_close_focused() { ... wm::unfocus_close_button(); wm::switch_app(wm::APP_SHELL); }
                    // XXX Wave-2-temp: let cur = wm::active_app(); if cur == wm::APP_CAVES { wm::focus_close_button(); }
                    // XXX Wave-2-temp: wm::switch_app(next); in_shell = next == wm::APP_SHELL;
                    render_current();
                    continue;
                }
                // Enter key — halt if applicable
                // XXX Wave-2-temp: 0x0D | 0x0A if wm::is_close_focused() => { halt_sphragis(); }
                0x0D | 0x0A => {
                    // fall through to shell input handling below
                }

                // XXX Wave-2-temp: 0x0C => { wm::split_vertical(); render_current(); continue; }
                // XXX Wave-2-temp: 0x0B => { wm::split_horizontal(); render_current(); continue; }
                // XXX Wave-2-temp: 0x17 => { wm::split_toggle_focus(); render_current(); continue; }
                // XXX Wave-2-temp: 0x80 => { wm::split_toggle_focus(); render_current(); continue; }
                // XXX Wave-2-temp: 0x11 => { wm::close_pane(); render_current(); in_shell = wm::active_app() == wm::APP_SHELL; continue; }
                0x0C | 0x0B | 0x17 | 0x80 | 0x11 => { render_current(); continue; }

                _ => {}
            }

            // Route keyboard input to the active app
            // XXX Wave-2-temp: let active = wm::active_app();
            let active: u8 = 0; // Wave-2-temp: always route to shell until Task 7 desktop rewrite
            // XXX Wave-2-temp: match active { wm::APP_SHELL => { ... } wm::APP_FILES | wm::APP_COMMS | wm::APP_CAVES => { ... } wm::APP_EDITOR => { ... } }
            match active {
                0 => {
                    // Shell (always active in Wave-2 until Task 7 replaces this)
                    // ── Visual selection mode override ─────────────
                    if console::select_mode_active() {
                        use crate::drivers::virtio::keyboard::{
                            KEY_ARROW_UP, KEY_ARROW_DOWN,
                            KEY_SHIFT_ARROW_UP, KEY_SHIFT_ARROW_DOWN,
                        };
                        match c {
                            KEY_ARROW_UP        => { console::sel_move_up(false); render_current(); }
                            KEY_ARROW_DOWN      => { console::sel_move_down(false); render_current(); }
                            KEY_SHIFT_ARROW_UP  => { console::sel_move_up(true);  render_current(); }
                            KEY_SHIFT_ARROW_DOWN=> { console::sel_move_down(true); render_current(); }
                            b'\r' | b'\n' => {
                                let n = console::sel_copy_to_clipboard();
                                console::exit_select_mode();
                                render_current();
                                console::puts("\n  -> copied ");
                                if n < 10 { console::putc(b'0' + n as u8); }
                                else { let mut tmp = [0u8; 8]; let mut i = 0; let mut nn = n;
                                       while nn > 0 && i < 8 { tmp[i] = b'0' + (nn % 10) as u8; nn /= 10; i += 1; }
                                       for j in 0..i { console::putc(tmp[i - 1 - j]); } }
                                console::puts(" bytes to clipboard (Ctrl+V to paste)\n");
                                console::prompt();
                                // XXX Wave-2-temp: wm::flush_all();
                            }
                            0x1B => {
                                console::exit_select_mode();
                                render_current();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    // Shell input
                    match c {
                        b'\r' | b'\n' => {
                            console::putc(b'\n');
                            platform::serial_puts("\r\n");
                            if cmd_len > 0 {
                                let cmd = unsafe { core::str::from_utf8_unchecked(&cmd_buf[..cmd_len]) };
                                super::shell::execute_cmd(cmd);
                                cmd_len = 0;
                            }
                            console::prompt();
                            // XXX Wave-2-temp: wm::flush_all();
                        }
                        0x08 | 0x7F => {
                            if cmd_len > 0 {
                                cmd_len -= 1;
                                console::putc(0x08);
                                platform::serial_putc(0x08);
                                platform::serial_putc(b' ');
                                platform::serial_putc(0x08);
                                // XXX Wave-2-temp: wm::flush_all();
                            }
                        }
                        // Ctrl+V — paste from system clipboard.
                        0x16 => {
                            paste_at_input!();
                        }
                        // Ctrl+Y — yank current input line into clipboard.
                        0x19 => {
                            super::clipboard::set(&cmd_buf[..cmd_len]);
                        }
                        // Ctrl+S — enter visual selection mode.
                        0x13 => {
                            clear_input!();
                            console::enter_select_mode();
                            render_current();
                        }
                        _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                            cmd_buf[cmd_len] = c;
                            cmd_len += 1;
                            console::putc(c);
                            platform::serial_putc(c);
                            // XXX Wave-2-temp: wm::flush_all();
                        }
                        _ => {}
                    }
                }
                _ => {
                    // Other apps: restored in Task 7 desktop rewrite
                }
            }
        }

        // Followup 3c-autopump: drive the NAT forwarder from the main
        // idle loop. Bounded to 256 frames per direction per tick
        // (inside nat::tick) so a flood can't starve the UI. Cheap
        // no-op when nic 1 isn't present or table is empty.
        let _ = crate::net::nat::tick();

        core::hint::spin_loop();
    }
}

// XXX Wave-2-temp: fn switch_to(app: u8) — restored in Task 7 desktop rewrite
#[allow(dead_code)]
fn switch_to(_app: u8) {
    // XXX Wave-2-temp: wm::switch_app(app);
    render_current();
    // XXX Wave-2-temp: wm::flush_all();
}

fn render_current() {
    // XXX Wave-2-temp: wm::draw_frame() — restored in Task 7 desktop rewrite
    // XXX Wave-2-temp: for i in 0..wm::pane_count() { wm::set_render_target(i); let r = wm::content_rect(); ... render_app(wm::pane_app(i)); }
    // XXX Wave-2-temp: wm::set_render_target(0); wm::flush_all();
    // Minimal shell-only render for Wave 2.
    super::font::clear_clip();
    let (saved_cx, saved_cy) = console::cursor();
    console::redraw_content();
    shell_banner();
    console::set_cursor(saved_cx, saved_cy);
}

// XXX Wave-2-temp: fn render_app(app: u8) — restored in Task 7 desktop rewrite
// (body used wm::APP_* as match patterns; entire fn replaced by Task 7)
#[allow(dead_code)]
fn render_app(_app: u8) {
    // XXX Wave-2-temp: match app { wm::APP_SHELL => { ... } wm::APP_DASHBOARD => ... wm::APP_FILES => ... ... }
}

/// Clean Sphragis shutdown — paint a "Shutdown" banner over the
/// framebuffer, write a marker on the serial UART so the host
/// supervisor knows the halt is intentional (not a watchdog
/// reset), then WFE forever. m1n1's HV stays alive in EL2; the
/// guest just stops doing anything.
// /
/// Useful now that the M4 ~118 s AP-watchdog is disabled (see
/// SESSION_JOURNAL 2026-04-20 21:30) — the Mac will keep running
/// until externally rebooted.
#[allow(dead_code)]
fn halt_sphragis() -> ! {
    use crate::ui::{font, gpu};

    platform::serial_puts("[halt] enter\r\n");

    // Banner — fill screen with a "shutdown" message
    let w = gpu::width();
    let h = gpu::height();
    let fb = gpu::framebuffer();
    platform::serial_puts("[halt] got fb\r\n");
    font::clear_clip();
    platform::serial_puts("[halt] clear_clip\r\n");
    gpu::fill_screen(0xFF000000); // black
    platform::serial_puts("[halt] fill_screen done\r\n");
    let cx = w / 2;
    let cy = h / 2;
    font::draw_str(fb, w, cx - 96, cy - 16, "SPHRAGIS HALTED", 0xFFFFFFFF, 0xFF000000);
    platform::serial_puts("[halt] draw1 done\r\n");
    font::draw_str(fb, w, cx - 144, cy + 8,  "(close pressed; m1n1 retains control)",
                   0xFF707070, 0xFF000000);
    platform::serial_puts("[halt] draw2 done\r\n");
    font::draw_str(fb, w, cx - 80, cy + 32,  "Reboot the Mac to restart.",
                   0xFF505050, 0xFF000000);
    platform::serial_puts("[halt] draw3 done\r\n");
    // XXX Wave-2-temp: wm::flush_all();
    platform::serial_puts("[halt] flush_all done\r\n");

    // Serial marker — supervisor + interactive driver can grep for this
    platform::serial_puts("\r\n[SPHRAGIS] halt requested via UI close button — entering wfe loop\r\n");

    // Clean HV exit — Apple impdef CYC_OVRD_EL1 (S3_5_C15_C5_0) bit 0
    // (CYC_OVRD_DISABLE_WFI_RET). Upstream m1n1's HV handle_sync treats a
    // guest write of bit 0 to this reg as "guest is shutting down CPU":
    // it calls hv_exit_cpu() and removes the CPU from started_cpus. Once
    // the last CPU exits, hv.start() returns on the Python side → the
    // proxy returns to stock "Running proxy..." mode and chainload.py
    // works again without a physical power-cycle. Harmless no-op if the
    // trap isn't armed (we wfe below anyway).
    platform::serial_puts("[halt] signalling HV clean exit via CYC_OVRD\r\n");
    unsafe {
        core::arch::asm!(
            "mov x0, #1",
            "msr S3_5_C15_C5_0, x0",
            out("x0") _,
        );
    }

    // Loop forever in WFE. m1n1's HV stays in EL2; we drop out of EL1
    // execution. The Mac stays alive (no watchdog, see hv.c M15).
    loop { unsafe { core::arch::asm!("wfe") } }
}

fn shell_banner() {
    // replace the old ASCII-art "BAT OS" letterforms with
    // the geometric project glyph + a structured banner per the spec.
    // Layout below mirrors `docs/design/desktop-shell/desktop-shell.jsx`'s
    // ShellBanner component, painted directly to the FB so the
    // console doesn't have to know about pixel art.
    use crate::ui::draw;
    use crate::ui::font;
    use crate::ui::gpu;

    const INK:      u32 = 0xFFE5E7EB;
    const MID:      u32 = 0xFF9CA3AF;
    const DIM_TXT:  u32 = 0xFF4B5563;
    const FAINT:    u32 = 0xFF374151;
    const CYAN:     u32 = 0xFF22D3EE;
    const CYAN_DIM: u32 = 0xFF0E7490;
    const BG:       u32 = 0xFF0A0A0A;

    let fb = gpu::framebuffer();
    let w = gpu::width();
    // XXX Wave-2-temp: let pr = wm::content_rect(); — use fixed origin for Wave 2 shell-only mode
    let bx: u32 = 16;
    let by: u32 = 16;

    // Project glyph (36×24 simplified, drawn at full source resolution).
    draw::draw_project_glyph_mini_full(bx as i32, by as i32, CYAN);

    // Wordmark + version + hint lines beside the bat.
    let tx = bx + 50;
    // Row 1: SPHRAGIS · version · "Microkernel Shell"
    font::draw_str(fb, w, tx,                      by + 0,  "BAT", INK, BG);
    font::draw_str(fb, w, tx + 3 * 8,              by + 0,  "_",   CYAN, BG);
    font::draw_str(fb, w, tx + 4 * 8,              by + 0,  "OS",  INK, BG);
    font::draw_str(fb, w, tx + 7 * 8 + 8,          by + 0,  "v0.5.0-DEV",       DIM_TXT, BG);
    font::draw_str(fb, w, tx + 7 * 8 + 8 + 11 * 8, by + 0,  "MICROKERNEL SHELL", MID,    BG);

    // Row 2: tab hint with chord codes.
    let r2 = by + 18;
    font::draw_str(fb, w, tx, r2,
        "tab to switch apps  .  ^1:SH ^2:DS ^3:FS ^4:NM ^5:ED ^6:SK ^7:CM ^8:BC",
        DIM_TXT, BG);

    // Row 3: command call-outs (cyan keywords, dim glue).
    let r3 = by + 36;
    font::draw_str(fb, w, tx,                       r3, "type ",          DIM_TXT, BG);
    font::draw_str(fb, w, tx + 5 * 8,               r3, "help",           CYAN,    BG);
    font::draw_str(fb, w, tx + 9 * 8,               r3, " for commands  .  ", DIM_TXT, BG);
    let mut x = tx + 9 * 8 + 18 * 8;
    font::draw_str(fb, w, x, r3, "tls-mode",  CYAN, BG); x += 8 * 8;
    font::draw_str(fb, w, x, r3, " . ",       FAINT, BG); x += 3 * 8;
    font::draw_str(fb, w, x, r3, "render",    CYAN, BG); x += 6 * 8;
    font::draw_str(fb, w, x, r3, " . ",       FAINT, BG); x += 3 * 8;
    font::draw_str(fb, w, x, r3, "audit",     CYAN, BG); x += 5 * 8;
    font::draw_str(fb, w, x, r3, " . ",       FAINT, BG); x += 3 * 8;
    font::draw_str(fb, w, x, r3, "origin-allow", CYAN, BG);

    let _ = CYAN_DIM; // reserved for future scrollback echo styling

    // position the console cursor below the banner using
    // an explicit set_cursor instead of emitting `\n` chars (which
    // would write empty cells to the scrollback and shift cursor
    // arbitrarily relative to wherever it was).
    console::set_cursor(0, 4);
}
