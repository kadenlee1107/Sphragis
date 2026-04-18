#![allow(dead_code)]
// Bat_OS — Tor-style Onion Routing
// Provides anonymous network access for BatCave traffic.
// Builds 3-hop circuits through Tor relay nodes.
//
// Architecture:
//   BatCave traffic → encrypt(layer3) → encrypt(layer2) → encrypt(layer1) → Guard
//   Guard → decrypts layer1 → Middle → decrypts layer2 → Exit → decrypts layer3 → destination
//
// Each hop uses X25519 key exchange + AES-256-CTR encryption.
// The exit relay sees plaintext but doesn't know the origin.

use crate::drivers::uart;
use crate::crypto::aes;

/// Tor circuit state
#[derive(Clone, Copy, PartialEq)]
pub enum CircuitState {
    Idle,
    Building,       // Creating circuit (extending through relays)
    Ready,          // Circuit built, ready for traffic
    Destroyed,
}

/// A single Tor relay hop
#[derive(Clone, Copy)]
pub struct TorRelay {
    pub ip: u32,
    pub port: u16,
    pub key: [u8; 32],     // Shared key with this relay (from X25519)
    pub active: bool,
}

impl TorRelay {
    const fn empty() -> Self {
        TorRelay { ip: 0, port: 0, key: [0; 32], active: false }
    }
}

/// Tor circuit (3 hops)
pub struct TorCircuit {
    pub state: CircuitState,
    pub guard: TorRelay,    // Entry relay
    pub middle: TorRelay,   // Middle relay
    pub exit: TorRelay,     // Exit relay
    pub circuit_id: u32,
}

static mut CIRCUIT: TorCircuit = TorCircuit {
    state: CircuitState::Idle,
    guard: TorRelay::empty(),
    middle: TorRelay::empty(),
    exit: TorRelay::empty(),
    circuit_id: 0,
};

/// Configure Tor relays (normally from consensus, here hardcoded for now).
pub fn configure_circuit(
    guard_ip: u32, guard_port: u16, guard_key: &[u8; 32],
    middle_ip: u32, middle_port: u16, middle_key: &[u8; 32],
    exit_ip: u32, exit_port: u16, exit_key: &[u8; 32],
) {
    unsafe {
        let c = &mut *core::ptr::addr_of_mut!(CIRCUIT);
        c.guard = TorRelay { ip: guard_ip, port: guard_port, key: *guard_key, active: true };
        c.middle = TorRelay { ip: middle_ip, port: middle_port, key: *middle_key, active: true };
        c.exit = TorRelay { ip: exit_ip, port: exit_port, key: *exit_key, active: true };
        c.state = CircuitState::Ready;
        c.circuit_id = 1;
    }
    uart::puts("[tor] Circuit configured (3 hops)\n");
}

/// Encrypt data through the onion (3 layers of encryption).
/// Returns total encrypted size.
pub fn onion_encrypt(plaintext: &[u8], output: &mut [u8]) -> usize {
    unsafe {
        let c = &*core::ptr::addr_of!(CIRCUIT);
        if c.state != CircuitState::Ready { return 0; }

        let mut buf = [0u8; 1400];
        let len = plaintext.len().min(1300);
        buf[..len].copy_from_slice(&plaintext[..len]);

        // Layer 3: encrypt with exit key
        let nonce3 = build_nonce(3);
        ctr_encrypt(&c.exit.key, &nonce3, &mut buf[..len]);

        // Layer 2: encrypt with middle key
        let nonce2 = build_nonce(2);
        ctr_encrypt(&c.middle.key, &nonce2, &mut buf[..len]);

        // Layer 1: encrypt with guard key
        let nonce1 = build_nonce(1);
        ctr_encrypt(&c.guard.key, &nonce1, &mut buf[..len]);

        output[..len].copy_from_slice(&buf[..len]);
        len
    }
}

/// Decrypt data from the onion (peel 3 layers).
pub fn onion_decrypt(ciphertext: &[u8], output: &mut [u8]) -> usize {
    unsafe {
        let c = &*core::ptr::addr_of!(CIRCUIT);
        if c.state != CircuitState::Ready { return 0; }

        let mut buf = [0u8; 1400];
        let len = ciphertext.len().min(1400);
        buf[..len].copy_from_slice(&ciphertext[..len]);

        // Peel layer 1 (guard)
        let nonce1 = build_nonce(1);
        ctr_encrypt(&c.guard.key, &nonce1, &mut buf[..len]);

        // Peel layer 2 (middle)
        let nonce2 = build_nonce(2);
        ctr_encrypt(&c.middle.key, &nonce2, &mut buf[..len]);

        // Peel layer 3 (exit)
        let nonce3 = build_nonce(3);
        ctr_encrypt(&c.exit.key, &nonce3, &mut buf[..len]);

        output[..len].copy_from_slice(&buf[..len]);
        len
    }
}

/// Check if Tor circuit is ready.
pub fn is_ready() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CIRCUIT.state)) == CircuitState::Ready }
}

/// Destroy the Tor circuit (zero all keys).
pub fn destroy_circuit() {
    unsafe {
        let c = &mut *core::ptr::addr_of_mut!(CIRCUIT);
        c.guard.key = [0; 32];
        c.middle.key = [0; 32];
        c.exit.key = [0; 32];
        c.state = CircuitState::Destroyed;
    }
    uart::puts("[tor] Circuit destroyed\n");
}

/// V8-ROOT-2 (V10 regression fix): drop the full Tor circuit on cave
/// switch. Without this, a new cave inherits the prior cave's guard/
/// middle/exit relay keys — a cross-cave identity/anonymity leak that
/// would let the new cave impersonate the outgoing cave's Tor session.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        let c = &mut *core::ptr::addr_of_mut!(CIRCUIT);
        c.guard = TorRelay::empty();
        c.middle = TorRelay::empty();
        c.exit = TorRelay::empty();
        c.circuit_id = 0;
        c.state = CircuitState::Idle;
    }
}

/// AES-256-CTR encrypt/decrypt in-place.
fn ctr_encrypt(key: &[u8; 32], nonce: &[u8; 16], data: &mut [u8]) {
    let cipher = aes::Aes256::new(key);
    let mut counter = *nonce;
    let mut pos = 0;
    while pos < data.len() {
        let mut block = counter;
        cipher.encrypt_block(&mut block);
        let remaining = data.len() - pos;
        let chunk = remaining.min(16);
        for i in 0..chunk {
            data[pos + i] ^= block[i];
        }
        pos += 16;
        let mut carry = 1u16;
        for i in (12..16).rev() {
            let sum = counter[i] as u16 + carry;
            counter[i] = sum as u8;
            carry = sum >> 8;
        }
    }
}

fn build_nonce(layer: u8) -> [u8; 16] {
    let mut nonce = [0u8; 16];
    nonce[0] = layer;
    unsafe {
        let cid = core::ptr::read_volatile(core::ptr::addr_of!(CIRCUIT.circuit_id)).to_le_bytes();
        nonce[4..8].copy_from_slice(&cid);
    }
    nonce
}
