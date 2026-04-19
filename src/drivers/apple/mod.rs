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

    // 1. DART bypass on the three DARTs we know about. Each returns
    //    quickly even if the target DART is wrong-addressed on M4
    //    (writes to Device-nGnRnE that nobody reads just go into
    //    the void).
    r.dart_usb  = if dart::Dart::usb().set_bypass(0).is_ok()  { 1 } else { 2 };
    r.dart_ans  = if dart::Dart::ans().set_bypass(0).is_ok()  { 1 } else { 2 };
    r.dart_disp = if dart::Dart::disp0().set_bypass(0).is_ok(){ 1 } else { 2 };

    // 2. ANS firmware check — M4 ADT doesn't expose /arm-io/ans, so
    //    `ans_base_resolved()` is false and we skip cleanly.
    if soc::ans_base_resolved() {
        let ans = ans_nvme::AnsNvme::new(
            soc::ans_base(),
            asc::Asc::new(soc::ans_base() + 0x4_0000, soc::ans_base() + 0x4_4000),
            dart::Dart::ans(),
        );
        r.ans_firmware = if ans.check_boot_status().is_ok() { 1 } else { 2 };
    }

    // 3. DWC3 USB — SKIP on M4. The existing `Dwc3::new` takes the
    //    DART base as the USB controller MMIO, but per
    //    M4_GROUND_TRUTH §3.2 those are separate peripherals on M4
    //    (drd0 ctl = 0x4_0228_0000 vs DART = 0x4_02f0_0000). Needs a
    //    `/arm-io/usb-drd0` discovery path + dedicated
    //    `soc::usb_drd0_base()` before it can safely run. Leaves
    //    r.dwc3_reset at 0 (not-run).

    // 4. BCM Wi-Fi chip-ID probe — SKIP on M4. The existing code
    //    passes `soc::ans_base()` as the BCM base (wrong peripheral
    //    entirely — BCM is Wi-Fi over APCIe, not NVMe). Needs a
    //    real `soc::bcm_base()` wired from `/arm-io/apcie0`.

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
