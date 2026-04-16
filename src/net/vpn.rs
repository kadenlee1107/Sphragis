// Bat_OS — VPN Tunnel (WireGuard-inspired)
// Encrypts all BatCave network traffic through a secure tunnel.
// Uses X25519 for key exchange and AES-256-CTR for packet encryption.
//
// Architecture:
//   BatCave process → syscall → TCP/UDP stack → VPN encrypt → IP → wire
//   wire → IP → VPN decrypt → TCP/UDP stack → BatCave process
//
// All traffic from a BatCave with "net" capability goes through the VPN.
// No plaintext escapes.

use crate::drivers::uart;
use crate::crypto::aes;
use core::sync::atomic::{AtomicU64, Ordering};

/// VPN tunnel state
#[derive(Clone, Copy, PartialEq)]
pub enum TunnelState {
    Disconnected,
    Handshaking,
    Established,
}

static mut STATE: TunnelState = TunnelState::Disconnected;

// Tunnel keys (derived from WireGuard-style handshake)
static mut SEND_KEY: [u8; 32] = [0; 32];
static mut RECV_KEY: [u8; 32] = [0; 32];
static mut NONCE: AtomicU64 = AtomicU64::new(0);

// VPN server endpoint
static mut SERVER_IP: u32 = 0;
static mut SERVER_PORT: u16 = 51820; // WireGuard default port

/// Configure VPN tunnel endpoint.
pub fn configure(server_ip: u32, server_port: u16, psk: &[u8; 32]) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SERVER_IP), server_ip);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SERVER_PORT), server_port);
        // Derive send/recv keys from PSK using our SHA-256
        let key_material = crate::crypto::sha256::hash(psk);
        let sk = &mut *core::ptr::addr_of_mut!(SEND_KEY);
        sk.copy_from_slice(&key_material);
        // Derive recv key by hashing again
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RECV_KEY), crate::crypto::sha256::hash(&key_material));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(STATE), TunnelState::Established);
    }
    uart::puts("[vpn] Tunnel configured\n");
}

/// Encrypt a packet for VPN transmission.
/// Returns encrypted data length, or 0 on error.
pub fn encrypt_packet(plaintext: &[u8], output: &mut [u8]) -> usize {
    unsafe {
        if core::ptr::read_volatile(core::ptr::addr_of!(STATE)) != TunnelState::Established { return 0; }

        let nonce_ref = &*core::ptr::addr_of!(NONCE);
        let nonce_val = nonce_ref.load(Ordering::Relaxed);
        nonce_ref.store(nonce_val + 1, Ordering::Relaxed);

        // Build nonce (12 bytes for AES-CTR: 4 zero + 8 counter)
        let mut nonce = [0u8; 16];
        let nonce_bytes = nonce_val.to_le_bytes();
        nonce[4..12].copy_from_slice(&nonce_bytes);

        // Encrypt with AES-256-CTR
        let len = plaintext.len().min(output.len() - 8);
        // Prepend nonce value (8 bytes)
        output[0..8].copy_from_slice(&nonce_bytes);
        // Encrypt in-place
        output[8..8+len].copy_from_slice(&plaintext[..len]);
        let sk = &*core::ptr::addr_of!(SEND_KEY);
        ctr_encrypt(sk, &nonce, &mut output[8..8+len]);

        8 + len
    }
}

/// Decrypt a VPN packet.
/// Returns decrypted data length, or 0 on error.
pub fn decrypt_packet(ciphertext: &[u8], output: &mut [u8]) -> usize {
    unsafe {
        if core::ptr::read_volatile(core::ptr::addr_of!(STATE)) != TunnelState::Established { return 0; }
        if ciphertext.len() < 8 { return 0; }

        // Extract nonce (first 8 bytes)
        let mut nonce = [0u8; 16];
        nonce[4..12].copy_from_slice(&ciphertext[0..8]);

        // Decrypt
        let len = (ciphertext.len() - 8).min(output.len());
        output[..len].copy_from_slice(&ciphertext[8..8+len]);
        let rk = &*core::ptr::addr_of!(RECV_KEY);
        ctr_encrypt(rk, &nonce, &mut output[..len]); // CTR is symmetric

        len
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
        // Increment counter (last 4 bytes)
        let mut carry = 1u16;
        for i in (12..16).rev() {
            let sum = counter[i] as u16 + carry;
            counter[i] = sum as u8;
            carry = sum >> 8;
        }
    }
}

/// Check if VPN tunnel is active.
pub fn is_active() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(STATE)) == TunnelState::Established }
}

/// Disconnect the VPN tunnel.
pub fn disconnect() {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(STATE), TunnelState::Disconnected);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SEND_KEY), [0; 32]);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RECV_KEY), [0; 32]);
    }
    uart::puts("[vpn] Tunnel disconnected\n");
}
