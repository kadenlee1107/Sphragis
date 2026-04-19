// Bat_OS — Apple Silicon Hardware Drivers
// All hardware-specific code for Apple M4 (T8132) lives here.
// Reference: Asahi Linux reverse-engineering documentation

pub mod uart;
pub mod aic;
pub mod dcp;
pub mod ans;
pub mod spi;
pub mod soc;
pub mod adt;
pub mod agx;
pub mod ane;
pub mod ans_nvme;
pub mod asc;
pub mod bcm_wifi;
pub mod boot_args;
pub mod dart;
pub mod dwc3;
pub mod wdt;
pub mod rtkit;
pub mod sio;
pub mod smc;
#[cfg(feature = "layer-b-test")]
pub mod layer_b_test;

// ─── Unified bring-up ──────────────────────────────────────────────
//
// Collects every peripheral that's been brought to "hardware access
// ready" (MMIO base resolved + module skeleton that has a `bring_up`
// entry point) and logs its status over the UART. Called from
// `kernel_main_apple` after ADT discovery + UART init.

/// Result of `bring_up_all`. Each field is a one-byte flag:
///   0 = not attempted, 1 = ok, 2 = failed/unimplemented
pub struct BringUpReport {
    pub dart_usb:     u8,
    pub dart_ans:     u8,
    pub dart_disp:    u8,
    pub ans_firmware: u8,
    pub dwc3_reset:   u8,
    pub bcm_probe:    u8,
}

impl BringUpReport {
    pub const fn new() -> Self {
        BringUpReport {
            dart_usb: 0, dart_ans: 0, dart_disp: 0,
            ans_firmware: 0, dwc3_reset: 0, bcm_probe: 0,
        }
    }
}

/// Attempt to bring up every peripheral that's implemented past the
/// skeleton stage. Safe to call after ADT discovery + UART init.
/// Returns a status report — no single failure halts the whole thing,
/// because peripherals may legitimately not exist on every M4 board
/// (e.g. Mac mini has no SPI keyboard, no battery).
pub fn bring_up_all() -> BringUpReport {
    let mut r = BringUpReport::new();

    // 1. DART bypass on the three DARTs we know about.
    r.dart_usb  = if dart::Dart::usb().set_bypass(0).is_ok()  { 1 } else { 2 };
    r.dart_ans  = if dart::Dart::ans().set_bypass(0).is_ok()  { 1 } else { 2 };
    r.dart_disp = if dart::Dart::disp0().set_bypass(0).is_ok(){ 1 } else { 2 };

    // 2. ANS firmware check. Only run if the ADT actually gave us a
    //    resolved base; on M4 the ADT path for ANS isn't populated
    //    yet (see M4_GROUND_TRUTH §3.1 "unknown") and the fallback
    //    MMIO address is from M1-era, so reading it silently faults.
    if soc::ans_base_resolved() {
        let ans = ans_nvme::AnsNvme::new(
            soc::ans_base(),
            asc::Asc::new(soc::ans_base() + 0x4_0000, soc::ans_base() + 0x4_4000),
            dart::Dart::ans(),
        );
        r.ans_firmware = if ans.check_boot_status().is_ok() { 1 } else { 2 };
    }

    // 3. DWC3 USB. Same guard — only attempt if base is ADT-resolved.
    if soc::dart_usb_resolved() {
        let mut usb = dwc3::Dwc3::new(soc::dart_usb(), dart::Dart::usb());
        r.dwc3_reset = if usb.bring_up(dwc3::Mode::Device).is_ok() { 1 } else { 2 };
    }

    // 4. BCM Wi-Fi chip-ID probe. Same guard.
    if soc::ans_base_resolved() {
        let mut wifi = bcm_wifi::BcmWifi::new(
            soc::ans_base(),
            dart::Dart::usb(),
        );
        r.bcm_probe = if wifi.probe_chip_id().is_ok() { 1 } else { 2 };
    }

    r
}

/// One-liner "Apple Silicon" banner for the UART boot log. Prints a
/// compact summary of `bring_up_all()` results.
pub fn print_bring_up_report(r: &BringUpReport) {
    use crate::drivers::apple::uart;
    uart::puts("[apple] bring-up: dart_usb=");
    uart::putc(b'0' + r.dart_usb);
    uart::puts(" dart_ans=");
    uart::putc(b'0' + r.dart_ans);
    uart::puts(" dart_disp=");
    uart::putc(b'0' + r.dart_disp);
    uart::puts(" ans_fw=");
    uart::putc(b'0' + r.ans_firmware);
    uart::puts(" dwc3=");
    uart::putc(b'0' + r.dwc3_reset);
    uart::puts(" bcm=");
    uart::putc(b'0' + r.bcm_probe);
    uart::puts("  (0=not-run, 1=ok, 2=fail)\n");
}
