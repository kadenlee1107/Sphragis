//! Eng-3 (2026-05-17 push) gov-grade §3.2:
//! Runtime exercise of the capability-token + MLS-label IPC entry
//! points. Mirrors the six §3 charter TDD scenarios so the
//! `qemu_cap_mls_selftest.py` smoke can observe PASS/FAIL per line.
//!
//! Public entry: `run()`. The shell command `cap-mls-selftest`
//! delegates here so shell.rs doesn't grow another 200-line
//! dispatch function and so a future test author can call the
//! same body from a kernel-internal harness.

#![allow(dead_code)]

use crate::caves::cap_token::{self, CapError, CapToken, RIGHT_IPC_CALL, RIGHT_IPC_WRITE};
use crate::caves::cave::{self, Integrity, Sensitivity};
use crate::caves::mls_ipc::{self, CapIpcError};
use crate::caves::mls_label::{LabelViolation, MlsLabel};
use crate::caves::sys_caves;
use crate::ui::console;

/// Run the six §3 scenarios in sequence. Emits a `✓` line per
/// passing scenario and an `✗ FAIL: <name>` line on the first
/// failure, then returns. The final line on success is
/// `✓ Cap-token + MLS-label: all 6 scenarios verified` — the
/// QEMU smoke matches that string.
pub fn run() {
    console::puts_hi("  CAP-TOKEN + MLS-LABEL SELF-TEST (Eng-3 §3.2)\n");

    // ── 1. label_dominance_self ──
    let lbl = MlsLabel::new(Sensitivity::Secret, Integrity::SystemTrusted);
    if !lbl.dominates(&lbl) {
        console::puts("  ✗ FAIL: label_dominance_self\n");
        return;
    }
    console::puts("  ✓ label_dominance_self\n");

    // ── 2. label_dominance_strict ──
    let chain = [
        MlsLabel::new(Sensitivity::Unclassified, Integrity::Untrusted),
        MlsLabel::new(Sensitivity::Confidential, Integrity::Sandboxed),
        MlsLabel::new(Sensitivity::Secret,       Integrity::SystemTrusted),
        MlsLabel::new(Sensitivity::TopSecret,    Integrity::HighIntegrity),
    ];
    for hi_idx in 1..chain.len() {
        for lo_idx in 0..hi_idx {
            if !chain[hi_idx].strictly_dominates(&chain[lo_idx]) {
                console::puts("  ✗ FAIL: label_dominance_strict (forward)\n");
                return;
            }
            if chain[lo_idx].dominates(&chain[hi_idx]) {
                console::puts("  ✗ FAIL: label_dominance_strict (backward)\n");
                return;
            }
        }
    }
    console::puts("  ✓ label_dominance_strict\n");

    let sys_wg_id = match sys_caves::sys_wg_id() {
        Some(id) => id as u16,
        None => { console::puts("  ✗ FAIL: sys-wg not initialised\n"); return; }
    };
    let kns_id = match sys_caves::kernel_ns_id() {
        Some(id) => id as u16,
        None => { console::puts("  ✗ FAIL: kernel-ns not initialised\n"); return; }
    };
    mls_ipc::drain(sys_wg_id);
    mls_ipc::drain(kns_id);

    // Local cleanup: reset both caves to bottom-of-lattice and
    // drain mailboxes at every early return. Closes the test
    // hermetically so subsequent shell commands inherit a clean
    // state.
    let cleanup = |sys_wg_id: u16, kns_id: u16| {
        let _ = cave::set_sensitivity_by_name("sys-wg",    Sensitivity::Unclassified);
        let _ = cave::set_sensitivity_by_name("kernel-ns", Sensitivity::Unclassified);
        let _ = cave::set_integrity_by_name("sys-wg",      Integrity::Untrusted);
        let _ = cave::set_integrity_by_name("kernel-ns",   Integrity::Untrusted);
        mls_ipc::drain(sys_wg_id);
        mls_ipc::drain(kns_id);
    };

    // ── 3. bell_lapadula_read_up_denied ──
    // kernel-ns (C) tries to recv from sys-wg (S). Token verifies;
    // the LABEL check must reject with ReadUp.
    let _ = cave::set_sensitivity_by_name("kernel-ns", Sensitivity::Confidential);
    let _ = cave::set_sensitivity_by_name("sys-wg",    Sensitivity::Secret);
    let _ = cave::set_integrity_by_name("kernel-ns",   Integrity::Sandboxed);
    let _ = cave::set_integrity_by_name("sys-wg",      Integrity::Sandboxed);
    let recv_tok = cap_token::mint(
        CapToken::KERNEL_ISSUER, kns_id, sys_wg_id, RIGHT_IPC_CALL,
    );
    let mut buf = [0u8; 32];
    match mls_ipc::call_with_token_recv(&recv_tok, kns_id, sys_wg_id, &mut buf) {
        Err(CapIpcError::Label(LabelViolation::ReadUp)) => {
            console::puts("  ✓ bell_lapadula_read_up_denied\n");
        }
        _ => {
            console::puts("  ✗ FAIL: bell_lapadula_read_up_denied\n");
            cleanup(sys_wg_id, kns_id);
            return;
        }
    }

    // ── 4. biba_write_up_denied ──
    // Equalise BLP at U so the Biba axis drives the verdict.
    // sys-wg Untrusted, kns SystemTrusted. sys-wg sends to kns
    // -> WriteUp.
    let _ = cave::set_sensitivity_by_name("sys-wg",    Sensitivity::Unclassified);
    let _ = cave::set_sensitivity_by_name("kernel-ns", Sensitivity::Unclassified);
    let _ = cave::set_integrity_by_name("sys-wg",      Integrity::Untrusted);
    let _ = cave::set_integrity_by_name("kernel-ns",   Integrity::SystemTrusted);
    let send_tok = cap_token::mint(
        CapToken::KERNEL_ISSUER, sys_wg_id, kns_id, RIGHT_IPC_CALL,
    );
    match mls_ipc::call_with_token_send(&send_tok, sys_wg_id, kns_id, b"taint:U->ST") {
        Err(CapIpcError::Label(LabelViolation::WriteUp)) => {
            console::puts("  ✓ biba_write_up_denied\n");
        }
        _ => {
            console::puts("  ✗ FAIL: biba_write_up_denied\n");
            cleanup(sys_wg_id, kns_id);
            return;
        }
    }

    // ── 5. cap_token_forge_attempt ──
    // Mint a real token, flip one byte of its MAC, present it.
    // verify must reject with BadMac.
    let mut bad = cap_token::mint(
        CapToken::KERNEL_ISSUER, kns_id, sys_wg_id, RIGHT_IPC_WRITE,
    );
    bad.mac[0] ^= 0x01;
    match cap_token::verify(&bad, kns_id, sys_wg_id, RIGHT_IPC_WRITE) {
        Err(CapError::BadMac) => {
            console::puts("  ✓ cap_token_forge_attempt\n");
        }
        _ => {
            console::puts("  ✗ FAIL: cap_token_forge_attempt\n");
            cleanup(sys_wg_id, kns_id);
            return;
        }
    }

    // ── 6. cap_token_valid_call_passes ──
    // Equalise both caves at bottom; mint a fresh token; send;
    // expect Ok(n) where n == body.len().
    cleanup(sys_wg_id, kns_id);
    let tok = cap_token::mint(
        CapToken::KERNEL_ISSUER, kns_id, sys_wg_id, RIGHT_IPC_CALL,
    );
    match mls_ipc::call_with_token_send(&tok, kns_id, sys_wg_id, b"hello-cap-mls") {
        Ok(n) if n == b"hello-cap-mls".len() => {
            console::puts("  ✓ cap_token_valid_call_passes\n");
        }
        _ => {
            console::puts("  ✗ FAIL: cap_token_valid_call_passes\n");
            cleanup(sys_wg_id, kns_id);
            return;
        }
    }

    cleanup(sys_wg_id, kns_id);
    console::puts("  ✓ Cap-token + MLS-label: all 6 scenarios verified\n");
}
