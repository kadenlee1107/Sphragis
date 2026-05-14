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

/// Check for panic hotkey (Ctrl+Shift+W = wipe NOW).
/// Called from the main input loop (Task 9 keyboard shortcut table wires this in).
#[allow(dead_code)]
pub fn check_panic_hotkey(c: u8) -> bool {
    // Ctrl+W = 0x17
    // This is the emergency wipe trigger
    if c == 0x17 {
        wipe::execute(wipe::WipeReason::Panic, false);
        return true;
    }
    false
}

/// Periodic security check — called from the main loop.
/// Checks dead man's switch timer (Task 9 keyboard shortcut table wires this in).
#[allow(dead_code)]
pub fn periodic_check() {
    if deadman::check() {
        // Dead man's switch expired — wipe everything
        wipe::execute(wipe::WipeReason::DeadManSwitch, false);
    }
}
