// Bat_OS — Desktop Environment
// Main event loop. Handles keyboard input, app switching, rendering.
// Ctrl+1-5 switches between apps.

use crate::drivers::uart;
use crate::drivers::virtio::gpu;
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
        if let Some(c) = uart::getc() {
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
                    let next = (wm::active_app() + 1) % 6;
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
                        uart::puts("\r\n");
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
                            uart::putc(0x08); uart::putc(b' '); uart::putc(0x08);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        uart::putc(c);
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
    // Initial render: shell
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
        // Check for keyboard input
        if let Some(c) = uart::getc() {
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

                // Tab key — cycle through apps
                0x09 => {
                    let next = (wm::active_app() + 1) % 6;
                    switch_to(next);
                    in_shell = next == wm::APP_SHELL;
                    continue;
                }

                _ => {}
            }

            // If in shell mode, handle shell input
            if in_shell {
                match c {
                    b'\r' | b'\n' => {
                        console::putc(b'\n');
                        uart::puts("\r\n");
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
                            uart::putc(0x08);
                            uart::putc(b' ');
                            uart::putc(0x08);
                        }
                    }
                    _ if c >= 0x20 && c <= 0x7E && cmd_len < 255 => {
                        cmd_buf[cmd_len] = c;
                        cmd_len += 1;
                        console::putc(c);
                        uart::putc(c);
                    }
                    _ => {}
                }
            }
        }

        core::hint::spin_loop();
    }
}

fn switch_to(app: u8) {
    wm::switch_app(app);
    render_current();
    wm::flush_all();
}

fn render_current() {
    let app = wm::active_app();
    wm::draw_frame();

    match app {
        wm::APP_SHELL => {
            // Console already manages its own content
            console::redraw_content();
        }
        wm::APP_DASHBOARD => apps::dashboard::render(),
        wm::APP_FILES => apps::filemanager::render(),
        wm::APP_NETMON => apps::netmon::render(),
        wm::APP_EDITOR => apps::editor::render(),
        wm::APP_BATCAVE => apps::batcave_mgr::render(),
        _ => {}
    }
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
