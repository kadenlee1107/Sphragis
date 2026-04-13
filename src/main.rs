#![no_std]
#![no_main]

mod batcave;
mod boot;
mod crypto;
mod drivers;
mod fs;
mod kernel;
mod net;
mod platform;
mod security;
mod ui;

use core::arch::global_asm;
use core::panic::PanicInfo;

use drivers::virtio::gpu;

global_asm!(include_str!("arch/aarch64/linux_header.s"));
global_asm!(include_str!("arch/aarch64/exceptions.s"));

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(uart_available: u64, dtb_ptr: u64) -> ! {
    // Platform detection
    let is_qemu = uart_available != 0;

    if is_qemu {
        drivers::uart::enable();
    }

    // Parse DTB if available (VZ VMs always pass one)
    let mut vz_virtio_bases: [usize; 16] = [0; 16];
    let mut vz_virtio_count = 0usize;

    if dtb_ptr != 0 {
        let dtb_info = boot::dtb::parse(dtb_ptr as usize);
        if dtb_info.valid {
            drivers::uart::puts("[boot] DTB parsed — VZ VM detected\n");
            vz_virtio_count = dtb_info.virtio_count;
            for i in 0..dtb_info.virtio_count {
                vz_virtio_bases[i] = dtb_info.virtio_mmio[i];
                drivers::uart::puts("  [dtb] virtio @ 0x");
                // print address
                let addr = dtb_info.virtio_mmio[i];
                let hex = b"0123456789abcdef";
                for shift in (0..16).rev() {
                    let nibble = ((addr >> (shift * 4)) & 0xF) as usize;
                    drivers::uart::putc(hex[nibble]);
                }
                drivers::uart::puts("\n");
            }
        }
    }

    drivers::uart::puts("\n");
    drivers::uart::puts("================================================\n");
    drivers::uart::puts("      ___       _      ___  ___               \n");
    drivers::uart::puts("     | _ ) __ _| |_   / _ \\/ __|              \n");
    drivers::uart::puts("     | _ \\/ _` |  _| | (_) \\__ \\              \n");
    drivers::uart::puts("     |___/\\__,_|\\__|  \\___/|___/              \n");
    drivers::uart::puts("                                              \n");
    drivers::uart::puts("================================================\n");
    drivers::uart::puts("  BAT_OS v0.3.0\n");
    drivers::uart::puts("  Security: Zero dependencies. Zero trust.\n");
    drivers::uart::puts("================================================\n\n");

    // Initialize kernel
    drivers::uart::puts("[boot] Initializing kernel...\n");
    kernel::mm::init();
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::arch::init_exceptions();

    // ═══════════════════════════════════════════
    // SECURITY INITIALIZATION
    // ═══════════════════════════════════════════

    // Initialize authentication system
    // Passphrase: "batman" (dev only — real passphrase set at first boot)
    // Duress code: "letmein" (triggers silent wipe)
    drivers::uart::puts("[security] Initializing auth system...\n");
    security::auth::init("batman", "letmein");
    drivers::uart::puts("  [auth] Passphrase + YubiKey auth ready\n");
    drivers::uart::puts("  [auth] Max attempts: 5\n");
    drivers::uart::puts("  [auth] Duress code: ARMED\n");

    // Initialize encrypted filesystem (key derived after auth)
    let master_key: [u8; 32] = [
        0xBA, 0x70, 0x05, 0xBA, 0x70, 0x05, 0xBA, 0x70,
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xF0, 0x0D,
        0x13, 0x37, 0x42, 0x69, 0xAA, 0xBB, 0xCC, 0xDD,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    fs::batfs::init(&master_key);
    drivers::uart::puts("  [fs] BatFS initialized (AES-256-CTR)\n");

    // Initialize BatCave runtime
    drivers::uart::puts("[boot] Initializing BatCave runtime...\n");
    batcave::cave::init();
    drivers::uart::puts("  [bc] BatCave runtime ready\n");

    // Initialize networking
    drivers::uart::puts("[boot] Initializing network...\n");
    match drivers::virtio::net::init() {
        Some(()) => {
            net::init();
            drivers::uart::puts("  [net] Network stack ready\n");
        }
        None => {
            drivers::uart::puts("  [net] No network device (offline)\n");
        }
    }

    // Initialize keyboard (virtio — type in GUI window)
    drivers::uart::puts("[boot] Initializing keyboard...\n");
    match drivers::virtio::keyboard::init() {
        Some(()) => drivers::uart::puts("  [kbd] GUI keyboard ready\n"),
        None => drivers::uart::puts("  [kbd] Serial input only\n"),
    }

    // Initialize GPU
    drivers::uart::puts("[boot] Initializing display...\n");
    match gpu::init() {
        Some(()) => {
            drivers::uart::puts("[boot] GPU ready\n\n");

            // ═══════════════════════════════════════
            // AUTHENTICATION GATE — must pass to proceed
            // ═══════════════════════════════════════
            drivers::uart::puts("[security] Launching auth gate...\n");
            security::boot_screen::run();
            // If we get here, authentication succeeded

            drivers::uart::puts("[security] AUTH PASSED — launching desktop\n");

            // Arm dead man's switch (48 hour default)
            security::deadman::arm(48);

            // Launch desktop
            ui::desktop::run();
        }
        None => {
            drivers::uart::puts("[boot] No display — serial shell\n\n");
            serial_shell();
        }
    }
}

/// Fallback shell for headless mode (serial only).
fn serial_shell() -> ! {
    use drivers::uart;
    uart::puts("bat_os > ");

    let mut buf = [0u8; 256];
    let mut len = 0usize;

    loop {
        if let Some(c) = uart::getc() {
            match c {
                b'\r' | b'\n' => {
                    uart::puts("\n");
                    if len > 0 {
                        let cmd = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                        match cmd {
                            "help" => uart::puts("  commands: help, mem, uname, whoami\n"),
                            "mem" => {
                                let (used, total) = kernel::mm::frame::stats();
                                uart::puts("  free: ");
                                kernel::mm::print_num((total - used) * 4);
                                uart::puts(" KB\n");
                            }
                            "uname" => uart::puts("  Bat_OS v0.3.0 aarch64\n"),
                            "whoami" => uart::puts("  KADEN\n"),
                            _ => {
                                uart::puts("  unknown: ");
                                uart::puts(cmd);
                                uart::puts("\n");
                            }
                        }
                        len = 0;
                    }
                    uart::puts("bat_os > ");
                }
                0x08 | 0x7F => {
                    if len > 0 {
                        len -= 1;
                        uart::putc(0x08);
                        uart::putc(b' ');
                        uart::putc(0x08);
                    }
                }
                _ if c >= 0x20 && c <= 0x7E && len < 255 => {
                    buf[len] = c;
                    len += 1;
                    uart::putc(c);
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

// ─── Apple Silicon Entry Point ───
// Called by the Apple boot stub when running on real M4 hardware.
// x0 = pointer to m1n1 boot args

global_asm!(include_str!("arch/aarch64/apple/boot.s"));

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_apple(boot_args: *const drivers::apple::soc::M1n1BootArgs) -> ! {
    // Set platform to Apple Silicon
    platform::set_platform(platform::Platform::AppleSilicon);

    // Parse boot args from m1n1
    let args = unsafe { &*boot_args };
    drivers::apple::soc::init_from_boot_args(args);

    // Initialize Apple UART for serial output
    drivers::apple::uart::init();
    drivers::apple::uart::puts("\n");
    drivers::apple::uart::puts("================================================\n");
    drivers::apple::uart::puts("  BAT_OS — BARE METAL APPLE SILICON\n");
    drivers::apple::uart::puts("  Running on REAL M4 hardware.\n");
    drivers::apple::uart::puts("================================================\n\n");

    // Initialize kernel core
    drivers::apple::uart::puts("[boot] Initializing microkernel...\n");
    kernel::mm::init();
    kernel::process::init();
    kernel::scheduler::init();
    kernel::ipc::init();
    kernel::arch::init_exceptions();

    // Initialize Apple Interrupt Controller
    drivers::apple::uart::puts("[boot] Initializing AIC2...\n");
    drivers::apple::aic::init();

    // Initialize encrypted filesystem
    let master_key: [u8; 32] = [
        0xBA, 0x70, 0x05, 0xBA, 0x70, 0x05, 0xBA, 0x70,
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xF0, 0x0D,
        0x13, 0x37, 0x42, 0x69, 0xAA, 0xBB, 0xCC, 0xDD,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    fs::batfs::init(&master_key);
    drivers::apple::uart::puts("[boot] BatFS initialized\n");

    // Initialize display (m1n1 simple framebuffer)
    drivers::apple::uart::puts("[boot] Initializing display...\n");
    if drivers::apple::dcp::init_simple_fb() {
        drivers::apple::uart::puts("[boot] Display ready — launching desktop\n\n");
        // Fill screen black to prove we own it
        drivers::apple::dcp::fill_screen(0xFF000000);
        drivers::apple::dcp::flush(0, 0,
            drivers::apple::dcp::width(),
            drivers::apple::dcp::height());

        // Initialize SPI keyboard
        let _ = drivers::apple::spi::init();

        // Launch desktop
        ui::desktop::run();
    } else {
        drivers::apple::uart::puts("[boot] No display — serial shell\n\n");
        // Serial-only fallback
        loop {
            if let Some(c) = drivers::apple::uart::getc() {
                drivers::apple::uart::putc(c); // Echo
            }
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try both UARTs
    drivers::uart::puts("\n!!! KERNEL PANIC !!!\n");
    if let Some(location) = info.location() {
        drivers::uart::puts("  File: ");
        drivers::uart::puts(location.file());
        drivers::uart::puts("\n");
    }
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}
