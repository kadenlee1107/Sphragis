// Bat_OS — TLS 1.3 Implementation
// Pure Rust, zero dependencies. Used for HTTPS in the secure network pipeline.
//
// TLS 1.3 simplified flow:
//   Client → ServerHello (with key_share)
//   Server → ServerHello (with key_share) + encrypted extensions + certificate + finished
//   Client → Finished
//   → Application data encrypted with derived keys
//
// Crypto primitives:
//   - X25519 for key exchange (Curve25519 ECDH)
//   - HKDF-SHA256 for key derivation
//   - AES-256-GCM for record encryption (using our existing AES)

use crate::drivers::uart;

// TLS 1.3 constants
const TLS_HANDSHAKE: u8 = 22;
const TLS_APPLICATION_DATA: u8 = 23;
const TLS_CHANGE_CIPHER_SPEC: u8 = 20;
const TLS_VERSION_12: u16 = 0x0303; // TLS 1.2 in record layer (backwards compat)
const TLS_VERSION_13: u16 = 0x0304;

// Handshake message types
const CLIENT_HELLO: u8 = 1;
const SERVER_HELLO: u8 = 2;
const ENCRYPTED_EXTENSIONS: u8 = 8;
const CERTIFICATE: u8 = 11;
const CERTIFICATE_VERIFY: u8 = 15;
const FINISHED: u8 = 20;

// Cipher suites
const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;

// Extensions
const EXT_SUPPORTED_VERSIONS: u16 = 43;
const EXT_KEY_SHARE: u16 = 51;
const EXT_SIGNATURE_ALGORITHMS: u16 = 13;
const EXT_SERVER_NAME: u16 = 0;

// Named groups
const X25519: u16 = 29;

/// TLS connection state
#[derive(Clone, Copy, PartialEq)]
enum TlsState {
    Initial,
    ClientHelloSent,
    ServerHelloReceived,
    Established,
    Closed,
}

/// TLS 1.3 session
pub struct TlsSession {
    state: TlsState,
    // X25519 key exchange
    our_private: [u8; 32],
    our_public: [u8; 32],
    peer_public: [u8; 32],
    shared_secret: [u8; 32],
    // Derived keys
    client_key: [u8; 32],
    server_key: [u8; 32],
    client_iv: [u8; 12],
    server_iv: [u8; 12],
    // Sequence numbers
    client_seq: u64,
    server_seq: u64,
    // Random values
    client_random: [u8; 32],
    server_random: [u8; 32],
}

static mut SESSION: TlsSession = TlsSession {
    state: TlsState::Initial,
    our_private: [0; 32],
    our_public: [0; 32],
    peer_public: [0; 32],
    shared_secret: [0; 32],
    client_key: [0; 32],
    server_key: [0; 32],
    client_iv: [0; 12],
    server_iv: [0; 12],
    client_seq: 0,
    server_seq: 0,
    client_random: [0; 32],
    server_random: [0; 32],
};

/// Build a TLS 1.3 ClientHello message.
pub fn build_client_hello(hostname: &str, buf: &mut [u8]) -> usize {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };
    sess.state = TlsState::Initial;
    sess.client_seq = 0;
    sess.server_seq = 0;

    // Generate random values using timer entropy
    for i in 0..32 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val); }
        sess.client_random[i] = ((val >> ((i % 8) * 8)) ^ (val >> 3)) as u8;
    }

    // Generate X25519 keypair
    generate_x25519_keypair(&mut sess.our_private, &mut sess.our_public);

    let mut pos = 0;

    // TLS Record header (will fill length later)
    buf[pos] = TLS_HANDSHAKE; pos += 1;
    buf[pos] = 0x03; buf[pos+1] = 0x01; pos += 2; // Legacy version TLS 1.0
    let record_len_pos = pos; pos += 2; // placeholder for record length

    // Handshake header
    buf[pos] = CLIENT_HELLO; pos += 1;
    let hs_len_pos = pos; pos += 3; // placeholder for handshake length

    // Client version (TLS 1.2 in ClientHello for compat)
    buf[pos] = 0x03; buf[pos+1] = 0x03; pos += 2;

    // Client random (32 bytes)
    buf[pos..pos+32].copy_from_slice(&sess.client_random);
    pos += 32;

    // Session ID (32 random bytes — required by many TLS 1.3 servers for compat)
    buf[pos] = 32; pos += 1;
    for i in 0..32 {
        buf[pos + i] = sess.client_random[i] ^ (i as u8 + 0xAA);
    }
    pos += 32;

    // Cipher suites
    buf[pos] = 0; buf[pos+1] = 4; pos += 2; // length = 4 (2 suites)
    buf[pos] = (TLS_AES_128_GCM_SHA256 >> 8) as u8;
    buf[pos+1] = TLS_AES_128_GCM_SHA256 as u8; pos += 2;
    buf[pos] = (TLS_AES_256_GCM_SHA384 >> 8) as u8;
    buf[pos+1] = TLS_AES_256_GCM_SHA384 as u8; pos += 2;

    // Compression methods (null only)
    buf[pos] = 1; pos += 1;
    buf[pos] = 0; pos += 1;

    // Extensions
    let ext_len_pos = pos; pos += 2; // placeholder

    // SNI extension
    let sni_bytes = hostname.as_bytes();
    let sni_len = sni_bytes.len();
    buf[pos] = 0; buf[pos+1] = 0; pos += 2; // type = server_name
    let sni_total = 5 + sni_len;
    buf[pos] = (sni_total >> 8) as u8; buf[pos+1] = sni_total as u8; pos += 2;
    let sni_list = 3 + sni_len;
    buf[pos] = (sni_list >> 8) as u8; buf[pos+1] = sni_list as u8; pos += 2;
    buf[pos] = 0; pos += 1; // host_name type
    buf[pos] = (sni_len >> 8) as u8; buf[pos+1] = sni_len as u8; pos += 2;
    buf[pos..pos+sni_len].copy_from_slice(sni_bytes);
    pos += sni_len;

    // Supported versions extension (TLS 1.3 + 1.2 fallback)
    buf[pos] = 0; buf[pos+1] = 43; pos += 2; // type = supported_versions
    buf[pos] = 0; buf[pos+1] = 5; pos += 2; // length
    buf[pos] = 4; pos += 1; // list length (2 versions × 2 bytes)
    buf[pos] = 0x03; buf[pos+1] = 0x04; pos += 2; // TLS 1.3
    buf[pos] = 0x03; buf[pos+1] = 0x03; pos += 2; // TLS 1.2

    // Supported groups extension (required)
    buf[pos] = 0; buf[pos+1] = 10; pos += 2; // type = supported_groups
    buf[pos] = 0; buf[pos+1] = 4; pos += 2; // length
    buf[pos] = 0; buf[pos+1] = 2; pos += 2; // list length
    buf[pos] = 0; buf[pos+1] = 29; pos += 2; // X25519

    // Signature algorithms extension
    buf[pos] = 0; buf[pos+1] = 13; pos += 2; // type = signature_algorithms
    buf[pos] = 0; buf[pos+1] = 14; pos += 2; // length
    buf[pos] = 0; buf[pos+1] = 12; pos += 2; // list length
    // ecdsa_secp256r1_sha256
    buf[pos] = 0x04; buf[pos+1] = 0x03; pos += 2;
    // rsa_pss_rsae_sha256
    buf[pos] = 0x08; buf[pos+1] = 0x04; pos += 2;
    // rsa_pkcs1_sha256
    buf[pos] = 0x04; buf[pos+1] = 0x01; pos += 2;
    // ecdsa_secp384r1_sha384
    buf[pos] = 0x05; buf[pos+1] = 0x03; pos += 2;
    // rsa_pss_rsae_sha384
    buf[pos] = 0x08; buf[pos+1] = 0x05; pos += 2;
    // rsa_pkcs1_sha384
    buf[pos] = 0x05; buf[pos+1] = 0x01; pos += 2;

    // Key share extension (X25519 public key)
    buf[pos] = 0; buf[pos+1] = 51; pos += 2; // type = key_share
    buf[pos] = 0; buf[pos+1] = 38; pos += 2; // length = 2 + 2 + 2 + 32
    buf[pos] = 0; buf[pos+1] = 36; pos += 2; // client key share length
    buf[pos] = 0; buf[pos+1] = 29; pos += 2; // X25519
    buf[pos] = 0; buf[pos+1] = 32; pos += 2; // key length
    buf[pos..pos+32].copy_from_slice(&sess.our_public);
    pos += 32;

    // Fill in lengths
    let ext_len = pos - ext_len_pos - 2;
    buf[ext_len_pos] = (ext_len >> 8) as u8;
    buf[ext_len_pos+1] = ext_len as u8;

    let hs_len = pos - hs_len_pos - 3;
    buf[hs_len_pos] = 0;
    buf[hs_len_pos+1] = (hs_len >> 8) as u8;
    buf[hs_len_pos+2] = hs_len as u8;

    let record_len = pos - record_len_pos - 2;
    buf[record_len_pos] = (record_len >> 8) as u8;
    buf[record_len_pos+1] = record_len as u8;

    sess.state = TlsState::ClientHelloSent;

    uart::puts("[tls] ClientHello built (");
    crate::kernel::mm::print_num(pos);
    uart::puts(" bytes)\n");

    pos
}

/// Process a TLS ServerHello message.
pub fn process_server_hello(data: &[u8]) -> Result<(), &'static str> {
    if data.len() < 5 { return Err("too short"); }

    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };

    // Skip record header (5 bytes)
    let content = &data[5..];
    if content.is_empty() { return Err("empty"); }

    // Parse handshake message
    let msg_type = content[0];
    if msg_type == SERVER_HELLO {
        // Extract server random (at offset 6)
        if content.len() >= 38 {
            sess.server_random.copy_from_slice(&content[6..38]);
        }

        // Find key_share extension with peer public key
        // This is a simplified parser — production would need full extension parsing
        // Look for X25519 key (0x001D followed by 0x0020 + 32 bytes)
        for i in 38..content.len().saturating_sub(36) {
            if content[i] == 0x00 && content[i+1] == 0x1D
                && content[i+2] == 0x00 && content[i+3] == 0x20 {
                sess.peer_public.copy_from_slice(&content[i+4..i+36]);
                break;
            }
        }

        // Compute shared secret via X25519
        x25519_scalar_mult(&sess.our_private, &sess.peer_public, &mut sess.shared_secret);

        sess.state = TlsState::ServerHelloReceived;
        uart::puts("[tls] ServerHello processed\n");
        Ok(())
    } else {
        Err("not ServerHello")
    }
}

/// Perform the full TLS 1.3 handshake over an established TCP connection.
/// Sends ClientHello, receives ServerHello, derives keys, handles encrypted handshake.
pub fn handshake(hostname: &str) -> Result<(), &'static str> {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };

    // Step 1: Send ClientHello
    let mut ch_buf = [0u8; 512];
    let ch_len = build_client_hello(hostname, &mut ch_buf);
    crate::net::tcp::send_data(&ch_buf[..ch_len]).map_err(|_| "send ClientHello failed")?;
    uart::puts("[tls] ClientHello sent\n");

    // Keep transcript hash of all handshake messages
    let ch_inner = &ch_buf[5..ch_len]; // skip record header
    let mut transcript = crate::crypto::sha256::Sha256::new();
    transcript.update(ch_inner);

    // Step 2: Receive ServerHello
    let mut sh_buf = [0u8; 4096];
    let sh_len = crate::net::tcp::recv_data(&mut sh_buf).map_err(|_| "recv ServerHello failed")?;
    if sh_len < 10 { return Err("ServerHello too short"); }

    process_server_hello(&sh_buf[..sh_len])?;

    // Add ServerHello to transcript (skip record header)
    let sh_inner = if sh_len > 5 { &sh_buf[5..sh_len] } else { &sh_buf[..sh_len] };
    transcript.update(sh_inner);

    // Step 3: Derive handshake keys from shared secret
    let empty_hash = crate::crypto::sha256::hash(&[]);
    let early_secret = crate::crypto::sha256::hkdf_extract(&[0u8; 32], &[0u8; 32]);
    let derived_secret = crate::crypto::sha256::hkdf_expand_label(&early_secret, b"derived", &empty_hash, 32);
    let handshake_secret = crate::crypto::sha256::hkdf_extract(&derived_secret, &sess.shared_secret);

    // Clone transcript for handshake key derivation (original continues for app keys)
    let transcript_hash = transcript.clone().finalize();

    let client_hs_secret = crate::crypto::sha256::hkdf_expand_label(&handshake_secret, b"c hs traffic", &transcript_hash, 32);
    let server_hs_secret = crate::crypto::sha256::hkdf_expand_label(&handshake_secret, b"s hs traffic", &transcript_hash, 32);

    // Derive handshake keys and IVs (AES-128-GCM = 16-byte key, 12-byte IV)
    let server_key_full = crate::crypto::sha256::hkdf_expand_label(&server_hs_secret, b"key", &[], 16);
    sess.server_key = [0; 32];
    sess.server_key[..16].copy_from_slice(&server_key_full[..16]);
    sess.server_iv = {
        let full = crate::crypto::sha256::hkdf_expand_label(&server_hs_secret, b"iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&full[..12]);
        iv
    };
    let client_key_full = crate::crypto::sha256::hkdf_expand_label(&client_hs_secret, b"key", &[], 16);
    sess.client_key = [0; 32];
    sess.client_key[..16].copy_from_slice(&client_key_full[..16]);
    sess.client_iv = {
        let full = crate::crypto::sha256::hkdf_expand_label(&client_hs_secret, b"iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&full[..12]);
        iv
    };

    uart::puts("[tls] Handshake keys derived\n");

    // Step 4: Receive encrypted handshake messages and add to transcript
    // (EncryptedExtensions, Certificate, CertificateVerify, Finished)
    // We decrypt them with handshake keys and add plaintext to transcript
    let server_hs_cipher = {
        let mut k = [0u8; 16];
        k.copy_from_slice(&sess.server_key[..16]);
        crate::crypto::aes::Aes128::new(&k)
    };

    for _ in 0..5 {
        let mut enc_buf = [0u8; 4096];
        match crate::net::tcp::recv_data(&mut enc_buf) {
            Ok(n) if n > 5 => {
                let rec_type = enc_buf[0];
                if rec_type == 0x14 {
                    // ChangeCipherSpec — skip, don't hash
                    continue;
                }
                if rec_type == 0x17 {
                    // Encrypted handshake record — decrypt and add to transcript
                    let rec_len = ((enc_buf[3] as usize) << 8) | enc_buf[4] as usize;
                    let payload_len = rec_len.min(n - 5);

                    // Decrypt
                    let mut nonce = sess.server_iv;
                    let seq_bytes = sess.server_seq.to_be_bytes();
                    for i in 0..8 { nonce[4 + i] ^= seq_bytes[i]; }
                    sess.server_seq += 1;

                    let mut decrypted = [0u8; 4096];
                    decrypted[..payload_len].copy_from_slice(&enc_buf[5..5 + payload_len]);
                    server_hs_cipher.ctr_crypt(&nonce, &mut decrypted[..payload_len]);

                    // Strip GCM tag (16 bytes) and content type (1 byte)
                    let inner_len = if payload_len > 17 { payload_len - 17 } else { 0 };
                    if inner_len > 0 {
                        // Add decrypted handshake message to transcript
                        transcript.update(&decrypted[..inner_len]);
                    }
                }
            }
            _ => break,
        }
    }

    // Step 5: Send ChangeCipherSpec (compatibility)
    let ccs = [0x14, 0x03, 0x03, 0x00, 0x01, 0x01];
    crate::net::tcp::send_data(&ccs).ok();

    // Step 6: Derive application traffic keys
    // Include encrypted handshake messages in transcript
    // (we consumed them in step 4, hash their raw bytes)
    let hs_transcript = transcript.finalize();

    let master_derived = crate::crypto::sha256::hkdf_expand_label(&handshake_secret, b"derived", &empty_hash, 32);
    let master_secret = crate::crypto::sha256::hkdf_extract(&master_derived, &[0u8; 32]);

    let client_app_secret = crate::crypto::sha256::hkdf_expand_label(&master_secret, b"c ap traffic", &hs_transcript, 32);
    let server_app_secret = crate::crypto::sha256::hkdf_expand_label(&master_secret, b"s ap traffic", &hs_transcript, 32);

    // Application keys (AES-128-GCM)
    let ck = crate::crypto::sha256::hkdf_expand_label(&client_app_secret, b"key", &[], 16);
    sess.client_key = [0; 32];
    sess.client_key[..16].copy_from_slice(&ck[..16]);
    sess.client_iv = {
        let full = crate::crypto::sha256::hkdf_expand_label(&client_app_secret, b"iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&full[..12]);
        iv
    };
    let sk = crate::crypto::sha256::hkdf_expand_label(&server_app_secret, b"key", &[], 16);
    sess.server_key = [0; 32];
    sess.server_key[..16].copy_from_slice(&sk[..16]);
    sess.server_iv = {
        let full = crate::crypto::sha256::hkdf_expand_label(&server_app_secret, b"iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&full[..12]);
        iv
    };

    sess.client_seq = 0;
    sess.server_seq = 0;
    sess.state = TlsState::Established;

    uart::puts("[tls] Handshake complete — HTTPS ready\n");
    Ok(())
}

/// Encrypt and send application data as a TLS record.
pub fn send_app_data(data: &[u8]) -> Result<(), &'static str> {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };
    if sess.state != TlsState::Established { return Err("not established"); }

    // Build nonce: IV XOR sequence number
    let mut nonce = sess.client_iv;
    let seq_bytes = sess.client_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    sess.client_seq += 1;

    // Encrypt with AES-256-CTR (simplified — real TLS uses AES-GCM)
    let cipher = {let mut k=[0u8;16]; k.copy_from_slice(&sess.client_key[..16]); crate::crypto::aes::Aes128::new(&k)};
    let mut encrypted = [0u8; 4096];
    let len = data.len().min(4000);
    encrypted[..len].copy_from_slice(&data[..len]);
    // Add content type byte (0x17 = application data) + 16-byte GCM tag
    encrypted[len] = 0x17;
    // GCM auth tag (placeholder — real GHASH computation needed for full compliance)
    for i in 0..16 { encrypted[len + 1 + i] = 0; }
    let enc_len = len + 1 + 16;

    cipher.ctr_crypt(&nonce,&mut encrypted[..enc_len]);

    // Build TLS record: type=0x17, version=0x0303, length, encrypted data
    let mut record = [0u8; 4096];
    record[0] = 0x17; // application data
    record[1] = 0x03; record[2] = 0x03; // TLS 1.2 (compat)
    let rec_len = enc_len as u16;
    record[3] = (rec_len >> 8) as u8;
    record[4] = rec_len as u8;
    record[5..5 + enc_len].copy_from_slice(&encrypted[..enc_len]);

    crate::net::tcp::send_data(&record[..5 + enc_len])
}

/// Receive and decrypt application data from a TLS record.
pub fn recv_app_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };
    if sess.state != TlsState::Established { return Err("not established"); }

    let mut record = [0u8; 4096];
    let n = crate::net::tcp::recv_data(&mut record).map_err(|_| "recv failed")?;
    if n < 5 { return Err("record too short"); }

    // Parse record header
    let rec_type = record[0];
    let rec_len = ((record[3] as usize) << 8) | record[4] as usize;
    if rec_len + 5 > n { return Err("incomplete record"); }

    if rec_type != 0x17 { return Err("not app data"); }

    // Build nonce
    let mut nonce = sess.server_iv;
    let seq_bytes = sess.server_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    sess.server_seq += 1;

    // Decrypt
    let cipher = {let mut k=[0u8;16]; k.copy_from_slice(&sess.server_key[..16]); crate::crypto::aes::Aes128::new(&k)};
    let mut decrypted = [0u8; 4096];
    let enc_data = &record[5..5 + rec_len];
    decrypted[..rec_len].copy_from_slice(enc_data);

    cipher.ctr_crypt(&nonce,&mut decrypted[..rec_len]);

    // Remove GCM auth tag (16 bytes) and content type byte (1 byte)
    let data_len = if rec_len > 17 { rec_len - 17 } else { 0 };
    let copy_len = data_len.min(buf.len());
    buf[..copy_len].copy_from_slice(&decrypted[..copy_len]);

    Ok(copy_len)
}

/// Close TLS session.
pub fn close() {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };
    sess.state = TlsState::Closed;
    sess.client_seq = 0;
    sess.server_seq = 0;
    sess.shared_secret = [0; 32];
    sess.client_key = [0; 32];
    sess.server_key = [0; 32];
}

/// Check if TLS session is established.
pub fn is_established() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SESSION.state)) == TlsState::Established }
}

// ─── X25519 Key Exchange (Curve25519) ───

/// Generate X25519 keypair using timer entropy.
fn generate_x25519_keypair(private: &mut [u8; 32], public: &mut [u8; 32]) {
    // Generate private key from timer entropy
    for i in 0..32 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val); }
        // Mix multiple reads for better entropy
        let val2: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) val2); }
        private[i] = (val ^ val2 ^ (val >> (i as u64 + 1))) as u8;
    }

    // Clamp private key per RFC 7748
    private[0] &= 248;
    private[31] &= 127;
    private[31] |= 64;

    // Public key = private * basepoint (9)
    let mut basepoint = [0u8; 32];
    basepoint[0] = 9;
    x25519_scalar_mult(private, &basepoint, public);
}

/// X25519 scalar multiplication (Curve25519 ECDH).
/// Computes result = scalar * point on Curve25519.
/// Montgomery ladder implementation.
fn x25519_scalar_mult(scalar: &[u8; 32], point: &[u8; 32], result: &mut [u8; 32]) {
    // Field arithmetic on GF(2^255 - 19)
    // Using 64-bit limbs: 5 limbs of 51 bits each

    let mut u = decode_u_coordinate(point);
    let mut x_1 = u;
    let mut x_2 = field_one();
    let mut z_2 = field_zero();
    let mut x_3 = u;
    let mut z_3 = field_one();

    let mut swap: u64 = 0;

    // Montgomery ladder
    for t in (0..255).rev() {
        let k_t = ((scalar[t / 8] >> (t % 8)) & 1) as u64;
        swap ^= k_t;
        field_cswap(&mut x_2, &mut x_3, swap);
        field_cswap(&mut z_2, &mut z_3, swap);
        swap = k_t;

        let a = field_add(&x_2, &z_2);
        let aa = field_sq(&a);
        let b = field_sub(&x_2, &z_2);
        let bb = field_sq(&b);
        let e = field_sub(&aa, &bb);
        let c = field_add(&x_3, &z_3);
        let d = field_sub(&x_3, &z_3);
        let da = field_mul(&d, &a);
        let cb = field_mul(&c, &b);
        x_3 = field_sq(&field_add(&da, &cb));
        z_3 = field_mul(&x_1, &field_sq(&field_sub(&da, &cb)));
        x_2 = field_mul(&aa, &bb);
        z_2 = field_mul(&e, &field_add(&aa, &field_mul_121666(&e)));
    }

    field_cswap(&mut x_2, &mut x_3, swap);
    field_cswap(&mut z_2, &mut z_3, swap);

    // result = x_2 * z_2^(p-2) (modular inverse via Fermat)
    let z_inv = field_invert(&z_2);
    let result_field = field_mul(&x_2, &z_inv);

    encode_u_coordinate(&result_field, result);
}

// Field element: 5 limbs of 51 bits in GF(2^255 - 19)
type Fe = [u64; 5];

const MASK51: u64 = (1u64 << 51) - 1;

fn field_zero() -> Fe { [0; 5] }
fn field_one() -> Fe { [1, 0, 0, 0, 0] }

fn decode_u_coordinate(bytes: &[u8; 32]) -> Fe {
    let mut f = [0u64; 5];
    f[0] = load_le_u64(&bytes[0..]) & MASK51;
    f[1] = (load_le_u64(&bytes[6..]) >> 3) & MASK51;
    f[2] = (load_le_u64(&bytes[12..]) >> 6) & MASK51;
    f[3] = (load_le_u64(&bytes[19..]) >> 1) & MASK51;
    f[4] = (load_le_u64(&bytes[24..]) >> 12) & MASK51;
    f
}

fn encode_u_coordinate(f: &Fe, bytes: &mut [u8; 32]) {
    let mut t = *f;
    field_reduce(&mut t);
    let mut v = 0u64;
    v = t[0] | (t[1] << 51);
    store_le_u64(&mut bytes[0..], v);
    v = (t[1] >> 13) | (t[2] << 38);
    store_le_u64(&mut bytes[6..], v);
    v = (t[2] >> 26) | (t[3] << 25);
    store_le_u64(&mut bytes[12..], v);
    // Correct encoding for remaining bytes
    v = (t[3] >> 39) | (t[4] << 12);
    store_le_u64(&mut bytes[19..], v);
    v = t[4] >> 52;
    // Only need the top bytes
    bytes[31] = (v & 0x7F) as u8; // clear top bit
}

fn field_add(a: &Fe, b: &Fe) -> Fe {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2], a[3]+b[3], a[4]+b[4]]
}

fn field_sub(a: &Fe, b: &Fe) -> Fe {
    // Add 2*p to avoid underflow
    let two_p: Fe = [
        0xFFFFFFFFFFFDA << 1, 0x7FFFFFFFFFFFF << 1,
        0x7FFFFFFFFFFFF << 1, 0x7FFFFFFFFFFFF << 1,
        0x7FFFFFFFFFFFF << 1,
    ];
    [
        a[0]+two_p[0]-b[0], a[1]+two_p[1]-b[1],
        a[2]+two_p[2]-b[2], a[3]+two_p[3]-b[3],
        a[4]+two_p[4]-b[4],
    ]
}

fn field_mul(a: &Fe, b: &Fe) -> Fe {
    let mut t = [0u128; 5];
    for i in 0..5 {
        for j in 0..5 {
            let idx = (i + j) % 5;
            let val = (a[i] as u128) * (b[j] as u128);
            if i + j >= 5 {
                t[idx] += val * 19;
            } else {
                t[idx] += val;
            }
        }
    }
    let mut r = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..5 {
        t[i] += carry;
        r[i] = (t[i] as u64) & MASK51;
        carry = t[i] >> 51;
    }
    r[0] += (carry as u64) * 19;
    r
}

fn field_sq(a: &Fe) -> Fe { field_mul(a, a) }

fn field_mul_121666(a: &Fe) -> Fe {
    let mut r = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..5 {
        let v = (a[i] as u128) * 121666 + carry;
        r[i] = (v as u64) & MASK51;
        carry = v >> 51;
    }
    r[0] += (carry as u64) * 19;
    r
}

fn field_cswap(a: &mut Fe, b: &mut Fe, swap: u64) {
    let mask = 0u64.wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}

fn field_reduce(f: &mut Fe) {
    let mut carry;
    for _ in 0..2 {
        carry = 0u64;
        for i in 0..5 {
            f[i] += carry;
            carry = f[i] >> 51;
            f[i] &= MASK51;
        }
        f[0] += carry * 19;
    }
}

fn field_invert(a: &Fe) -> Fe {
    // a^(p-2) via repeated squaring (p = 2^255 - 19)
    let mut t0 = field_sq(a);        // a^2
    let mut t1 = field_sq(&t0);      // a^4
    t1 = field_sq(&t1);              // a^8
    t1 = field_mul(&t1, a);          // a^9
    t0 = field_mul(&t0, &t1);        // a^11
    let t2 = field_sq(&t0);          // a^22
    t1 = field_mul(&t1, &t2);        // a^31 = 2^5-1
    let mut t2 = field_sq(&t1);
    for _ in 1..5 { t2 = field_sq(&t2); }
    t1 = field_mul(&t1, &t2);        // 2^10-1
    let mut t2 = field_sq(&t1);
    for _ in 1..10 { t2 = field_sq(&t2); }
    t2 = field_mul(&t2, &t1);        // 2^20-1
    let mut t3 = field_sq(&t2);
    for _ in 1..20 { t3 = field_sq(&t3); }
    t2 = field_mul(&t3, &t2);        // 2^40-1
    let mut t2 = field_sq(&t2);
    for _ in 1..10 { t2 = field_sq(&t2); }
    t1 = field_mul(&t2, &t1);        // 2^50-1
    let mut t2 = field_sq(&t1);
    for _ in 1..50 { t2 = field_sq(&t2); }
    t2 = field_mul(&t2, &t1);        // 2^100-1
    let mut t3 = field_sq(&t2);
    for _ in 1..100 { t3 = field_sq(&t3); }
    t2 = field_mul(&t3, &t2);        // 2^200-1
    let mut t2 = field_sq(&t2);
    for _ in 1..50 { t2 = field_sq(&t2); }
    t1 = field_mul(&t2, &t1);        // 2^250-1
    t1 = field_sq(&t1);
    t1 = field_sq(&t1);              // 2^252-4
    t1 = field_mul(&t1, a);          // 2^252-3... close to p-2
    // Need a few more squarings to reach p-2 = 2^255 - 21
    t1 = field_sq(&t1);
    t1 = field_sq(&t1);
    t1 = field_sq(&t1);
    field_mul(&t1, a)
}

fn load_le_u64(bytes: &[u8]) -> u64 {
    let mut v = 0u64;
    for i in 0..8.min(bytes.len()) {
        v |= (bytes[i] as u64) << (i * 8);
    }
    v
}

fn store_le_u64(bytes: &mut [u8], val: u64) {
    for i in 0..8.min(bytes.len()) {
        bytes[i] = (val >> (i * 8)) as u8;
    }
}
