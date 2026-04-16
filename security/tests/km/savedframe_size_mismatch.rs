//! ATTACK-KM-013 — SAVED_FRAME memcpy length does not match TrapFrame size.

use km_attacks::trap_frame::{TrapFrame, HANDLER_COPY_BYTES, SAVED_FRAME_BYTES};

#[test]
fn trap_frame_size_does_not_match_handler_copy_length() {
    let tf = std::mem::size_of::<TrapFrame>();
    // TrapFrame: 31 u64 + elr u64 + spsr u64 = 33 * 8 = 264.
    assert_eq!(tf, 264, "TrapFrame size changed; update audit");
    assert_eq!(
        HANDLER_COPY_BYTES, 272,
        "Handler hardcodes 272; see kernel/arch/mod.rs:190"
    );
    assert!(
        HANDLER_COPY_BYTES > tf,
        "ATTACK-KM-013 confirmed: handler reads {} bytes past end of TrapFrame",
        HANDLER_COPY_BYTES - tf
    );
    // And SAVED_FRAME is big enough to RECEIVE them, so the OOB read is
    // silent (no fault), and those 8 bytes propagate into the parent's
    // restore sequence.
    assert!(SAVED_FRAME_BYTES >= HANDLER_COPY_BYTES);
}
