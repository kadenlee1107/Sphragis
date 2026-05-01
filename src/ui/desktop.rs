// Bat_OS — Desktop Environment
// Main event loop. Handles keyboard input, app switching, rendering.
// Ctrl+1-5 switches between apps.

use crate::platform;
use crate::security;
use super::{wm, console, apps};

/// Resume desktop after BatCave exit — skip reinitialization.
pub fn resume() -> ! {
    let mut in_shell = true;
    let mut cmd_buf = [0u8; 256];
    let mut cmd_len: usize = 0;

    // Just show prompt and continue
    console::prompt();

    loop {
        // STUMP #99: same dual-source read as desktop::run. Without
        // this, after a cave exit the resumed shell wouldn't accept
        // QEMU-window keystrokes either.
        crate::drivers::virtio::keyboard::poll();
        let next_char = platform::serial_getc()
            .or_else(crate::drivers::virtio::keyboard::getc);
        if let Some(c) = next_char {
            if security::check_panic_hotkey(c) {
                loop { unsafe { core::arch::asm!("wfe") }; }
            }
            security::periodic_check();

            match c {
                0x01 => { switch_to(wm::APP_SHELL); in_shell = true; continue; }
                0x02 => { switch_to(wm::APP_DASHBOARD); in_shell = false; continue; }
                0x04 => { switch_to(wm::APP_NETMON); in_shell = false; continue; }
                0x05 => { switch_to(wm::APP_EDITOR); in_shell = false; continue; }
                0x09 => {
                    let next = (wm::active_app() + 1) % 9;
                    switch_to(next);
                    in_shell = next == wm::APP_SHELL;
                    continue;
                }
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
                        wm::flush_all();
                    }
                    0x08 | 0x7F => {
                        if cmd_len > 0 {
                            cmd_len -= 1;
                            console::putc(0x08);
                            platform::serial_putc(0x08); platform::serial_putc(b' '); platform::serial_putc(0x08);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        platform::serial_putc(c);
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
    wm::init_panes_pub();
    wm::switch_app(wm::APP_SHELL);
    render_current();

    let mut in_shell = true;
    let mut cmd_buf = [0u8; 256];
    let mut cmd_len: usize = 0;

    // Draw initial shell prompt
    console::init_in_window();
    shell_banner();
    console::prompt();
    wm::flush_all();

    loop {
        // Check for keyboard input from EITHER serial (host terminal)
        // or virtio-keyboard (QEMU GUI window). STUMP #99: pre-fix
        // only serial was read, so a Mac user typing into the QEMU
        // window saw zero feedback. Pump virtio events first so they
        // land in the keystroke ring, then prefer serial.
        crate::drivers::virtio::keyboard::poll();
        let next_char = platform::serial_getc()
            .or_else(crate::drivers::virtio::keyboard::getc);
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
                0x01 => { switch_to(wm::APP_SHELL); in_shell = true; continue; }
                0x02 => { switch_to(wm::APP_DASHBOARD); in_shell = false; continue; }
                0x03 => { // Ctrl+C — if in shell, cancel line; otherwise switch to files
                    if in_shell && cmd_len > 0 {
                        console::puts("^C\n");
                        cmd_len = 0;
                        console::prompt();
                        wm::flush_all();
                        continue;
                    } else if !in_shell {
                        switch_to(wm::APP_FILES);
                        in_shell = false;
                        continue;
                    }
                }
                0x04 => { switch_to(wm::APP_NETMON); in_shell = false; continue; }
                0x05 => { switch_to(wm::APP_EDITOR); in_shell = false; continue; }

                // Tab key — cycle app in focused pane.
                // 2026-04-20 21:45: cycle goes 0..8 → close-button-X → 0
                0x09 => {
                    platform::serial_puts("[tab] received\r\n");
                    if wm::is_close_focused() {
                        // Currently on the X — wrap back to app 0
                        platform::serial_puts("[tab] unfocus+switch_app(0)\r\n");
                        wm::unfocus_close_button();
                        wm::switch_app(wm::APP_SHELL);
                        in_shell = true;
                        platform::serial_puts("[tab] calling render_current\r\n");
                        render_current();
                        platform::serial_puts("[tab] render_current done\r\n");
                        continue;
                    }
                    let cur = wm::active_app();
                    if cur == 8 {
                        // Last app → tab onto the close button
                        platform::serial_puts("[tab] cur=8 → focus_close_button\r\n");
                        wm::focus_close_button();
                        // Don't change active_app — keep it on 8 so the
                        // pane content stays visible behind the X.
                        in_shell = false;
                        platform::serial_puts("[tab] calling render_current (X)\r\n");
                        render_current();
                        platform::serial_puts("[tab] render_current done (X)\r\n");
                        continue;
                    }
                    let next = cur + 1;
                    platform::serial_puts("[tab] switching to next app\r\n");
                    wm::switch_app(next);
                    in_shell = next == wm::APP_SHELL;
                    platform::serial_puts("[tab] calling render_current\r\n");
                    render_current();
                    platform::serial_puts("[tab] render_current done\r\n");
                    continue;
                }
                // Enter key — if close button is focused, halt Bat_OS.
                // CR (0x0D) and LF (0x0A) both treated as Enter here.
                0x0D | 0x0A if wm::is_close_focused() => {
                    platform::serial_puts("[enter] close focused — calling halt_bat_os\r\n");
                    halt_bat_os();
                    // halt_bat_os never returns
                }

                // Ctrl+L — vertical split (left | right)
                0x0C => { wm::split_vertical(); render_current(); continue; }
                // Ctrl+K — horizontal split (top / bottom)
                0x0B => { wm::split_horizontal(); render_current(); continue; }
                // Ctrl+W — switch focus between split panels
                0x17 => { wm::split_toggle_focus(); render_current(); continue; }
                // Option+Tab (0x80) — cycle focus between panes
                0x80 => { wm::split_toggle_focus(); render_current(); continue; }
                // Ctrl+Q — close focused pane
                0x11 => { wm::close_pane(); render_current(); in_shell = wm::active_app() == wm::APP_SHELL; continue; }

                _ => {}
            }

            // Route keyboard input to the active app
            let active = wm::active_app();
            match active {
                wm::APP_SHELL => {
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
                            wm::flush_all();
                        }
                        0x08 | 0x7F => {
                            if cmd_len > 0 {
                                cmd_len -= 1;
                                console::putc(0x08);
                                platform::serial_putc(0x08);
                                platform::serial_putc(b' ');
                                platform::serial_putc(0x08);
                            }
                        }
                        _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                            cmd_buf[cmd_len] = c;
                            cmd_len += 1;
                            console::putc(c);
                            platform::serial_putc(c);
                        }
                        _ => {}
                    }
                }
                wm::APP_BROWSER => {
                    apps::browser::handle_key(c);
                    render_current();
                }
                wm::APP_COMMS => {
                    apps::comms::handle_key(c);
                    render_current();
                }
                _ => {
                    // Other apps: no keyboard input handling yet
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

fn switch_to(app: u8) {
    wm::switch_app(app);
    render_current();
    wm::flush_all();
}

fn render_current() {
    // Clear clip for frame drawing
    super::font::clear_clip();
    wm::draw_frame();

    // Render each pane with clipping to its bounds
    for i in 0..wm::pane_count() {
        wm::set_render_target(i);
        let r = wm::content_rect();
        super::font::set_clip(r.x, r.y, r.w, r.h);
        render_app(wm::pane_app(i));
    }

    super::font::clear_clip();
    wm::set_render_target(0);
    wm::flush_all();
}

fn render_app(app: u8) {
    match app {
        wm::APP_SHELL => {
            console::redraw_content();
        }
        wm::APP_DASHBOARD => apps::dashboard::render(),
        wm::APP_FILES => apps::filemanager::render(),
        wm::APP_NETMON => apps::netmon::render(),
        wm::APP_EDITOR => apps::editor::render(),
        wm::APP_SECURITY => apps::security::render(),
        wm::APP_COMMS => apps::comms::render(),
        wm::APP_BROWSER => apps::browser::render(),
        wm::APP_BATCAVE => apps::batcave_mgr::render(),
        _ => {}
    }
}

/// Clean Bat_OS shutdown — paint a "Shutdown" banner over the
/// framebuffer, write a marker on the serial UART so the host
/// supervisor knows the halt is intentional (not a watchdog
/// reset), then WFE forever. m1n1's HV stays alive in EL2; the
/// guest just stops doing anything.
///
/// Useful now that the M4 ~118 s AP-watchdog is disabled (see
/// SESSION_JOURNAL 2026-04-20 21:30) — the Mac will keep running
/// until externally rebooted.
fn halt_bat_os() -> ! {
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
    font::draw_str(fb, w, cx - 96, cy - 16, "BAT_OS HALTED", 0xFFFFFFFF, 0xFF000000);
    platform::serial_puts("[halt] draw1 done\r\n");
    font::draw_str(fb, w, cx - 144, cy + 8,  "(close pressed; m1n1 retains control)",
                   0xFF707070, 0xFF000000);
    platform::serial_puts("[halt] draw2 done\r\n");
    font::draw_str(fb, w, cx - 80, cy + 32,  "Reboot the Mac to restart.",
                   0xFF505050, 0xFF000000);
    platform::serial_puts("[halt] draw3 done\r\n");
    wm::flush_all();
    platform::serial_puts("[halt] flush_all done\r\n");

    // Serial marker — supervisor + interactive driver can grep for this
    platform::serial_puts("\r\n[BATOS] halt requested via UI close button — entering wfe loop\r\n");

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
    console::puts_hi("      ___       _      ___  ___\n");
    console::puts_hi("     | _ ) __ _| |_   / _ \\/ __|\n");
    console::puts_hi("     | _ \\/ _` |  _| | (_) \\__ \\\n");
    console::puts_hi("     |___/\\__,_|\\__|  \\___/|___/\n");
    console::puts("\n");
    console::puts("  Microkernel Shell — Tab to switch apps\n");
    console::puts("  Ctrl+A:SH  Ctrl+B:DS  Ctrl+D:NM  Ctrl+E:ED\n\n");
}
