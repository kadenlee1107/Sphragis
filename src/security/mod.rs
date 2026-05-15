pub mod audit;
pub mod audit_chain;
pub mod audit_forwarder;
pub mod auth;
pub mod boot_screen;
pub mod deadman;
pub mod origin;
pub mod otp;
pub mod tpi;
pub mod wipe;
pub mod zeroize;

/// Returns true if `c` is the panic-wipe hotkey (0x17 / Ctrl+W).
/// On a true match this invokes `wipe::execute(WipeReason::Panic,
/// false)`, which halts the SoC on real M4 hardware and returns
/// normally under QEMU.
///
/// Called from handle_key() in desktop.rs BEFORE the regular shortcut
/// match table so the wipe takes priority over all other Ctrl+W bindings.
pub fn check_panic_hotkey(c: u8) -> bool {
    // Ctrl+W = 0x17
    // This is the emergency wipe trigger
    if c == 0x17 {
        wipe::execute(wipe::WipeReason::Panic, false);
        return true;
    }
    false
}

/// Periodic security check — called from the idle arm of desktop::run().
/// Checks dead man's switch timer; triggers wipe on expiry.
pub fn periodic_check() {
    if deadman::check() {
        // Dead man's switch expired — wipe everything
        wipe::execute(wipe::WipeReason::DeadManSwitch, false);
    }
}
