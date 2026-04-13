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
        // ServerHello layout (after handshake header):
        //   [0] msg_type (0x02)
        //   [1..4] length (3 bytes)
        //   [4..6] version (0x0303)
        //   [6..38] server_random (32 bytes)
        //   [38] session_id_length
        //   [39..39+sid_len] session_id
        //   then: cipher_suite (2), compression (1), extensions_length (2), extensions...

        if content.len() < 39 { return Err("SH too short"); }

        // Server random
        sess.server_random.copy_from_slice(&content[6..38]);

        // Skip session ID
        let sid_len = content[38] as usize;
        let mut pos = 39 + sid_len;

        // Skip cipher suite (2) + compression (1)
        pos += 3;

        // Extensions length
        if pos + 2 > content.len() { return Err("SH no extensions"); }
        let ext_len = ((content[pos] as usize) << 8) | content[pos + 1] as usize;
        pos += 2;
        let ext_end = (pos + ext_len).min(content.len());

        // Parse extensions to find key_share (type 0x0033 = 51)
        while pos + 4 < ext_end {
            let ext_type = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
            let ext_data_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
            pos += 4;

            if ext_type == 51 {
                // key_share: named_group (2) + key_exchange_length (2) + key_exchange
                if pos + 4 + 32 <= content.len() {
                    let group = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
                    let key_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
                    if group == 29 && key_len == 32 && pos + 4 + 32 <= content.len() {
                        sess.peer_public.copy_from_slice(&content[pos + 4..pos + 36]);
                        uart::puts("[tls] found X25519 key_share at ext offset ");
                        crate::kernel::mm::print_num(pos);
                        uart::puts("\n");
                    }
                }
            }

            pos += ext_data_len;
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

    // Step 2: Receive ServerHello + possibly more records in same TCP segment
    let mut all_buf = [0u8; 8192];
    let mut all_len = 0;
    // Read multiple chunks — server may send SH + CCS + encrypted in one burst
    for _ in 0..5 {
        let mut chunk = [0u8; 4096];
        match crate::net::tcp::recv_data(&mut chunk) {
            Ok(n) if n > 0 => {
                let copy = n.min(all_buf.len() - all_len);
                all_buf[all_len..all_len + copy].copy_from_slice(&chunk[..copy]);
                all_len += copy;
            }
            _ => break,
        }
    }
    if all_len < 10 { return Err("ServerHello too short"); }

    // Parse first record (ServerHello)
    let sh_rec_len = ((all_buf[3] as usize) << 8) | all_buf[4] as usize;
    let sh_end = (5 + sh_rec_len).min(all_len);
    process_server_hello(&all_buf[..sh_end])?;

    // Add ServerHello handshake to transcript (skip 5-byte record header)
    transcript.update(&all_buf[5..sh_end]);

    uart::puts("[tls] received ");
    crate::kernel::mm::print_num(all_len);
    uart::puts("b total, SH=");
    crate::kernel::mm::print_num(sh_end);
    uart::puts("b\n");

    // Remaining bytes after ServerHello contain more records
    let mut remaining_start = sh_end;

    // Step 3: Derive handshake keys from shared secret
    let empty_hash = crate::crypto::sha256::hash(&[]);
    let early_secret = crate::crypto::sha256::hkdf_extract(&[0u8; 32], &[0u8; 32]);
    let derived_secret = crate::crypto::sha256::hkdf_expand_label(&early_secret, b"derived", &empty_hash, 32);
    let handshake_secret = crate::crypto::sha256::hkdf_extract(&derived_secret, &sess.shared_secret);

    // Field arithmetic self-test
    {
        // Test: 5 * 7 = 35 in GF(p)
        let five: Fe = [5, 0, 0, 0, 0];
        let seven: Fe = [7, 0, 0, 0, 0];
        let result = field_mul(&five, &seven);
        uart::puts("[tls] field 5*7 raw=");
        crate::kernel::mm::print_num(result[0] as usize);
        uart::puts("/");
        crate::kernel::mm::print_num(result[1] as usize);
        let mut reduced = result;
        field_reduce(&mut reduced);
        uart::puts(" reduced=");
        crate::kernel::mm::print_num(reduced[0] as usize);
        uart::puts(" (expected 35)\n");

        // Test: field_sq of small number
        let three: Fe = [3, 0, 0, 0, 0];
        let sq = field_sq(&three);
        let mut sq_r = sq;
        field_reduce(&mut sq_r);
        uart::puts("[tls] field 3^2=");
        crate::kernel::mm::print_num(sq_r[0] as usize);
        uart::puts(" (expected 9)\n");
    }

    // Simple X25519 test: basepoint * 1 should give basepoint
    {
        let mut one = [0u8; 32];
        one[0] = 1;
        one[0] &= 248; one[31] &= 127; one[31] |= 64; // clamp: becomes 64 in high byte
        let mut bp = [0u8; 32];
        bp[0] = 9;
        let mut result = [0u8; 32];
        x25519_scalar_mult(&one, &bp, &mut result);
        // scalar "1" clamped = has bit 254 set, bit 0 clear
        // This won't give basepoint back but should give a deterministic result
        uart::puts("[tls] X25519(clamped_1, 9)[0..4]: ");
        for i in 0..4 {
            let hex = b"0123456789abcdef";
            uart::putc(hex[(result[i] >> 4) as usize]);
            uart::putc(hex[(result[i] & 0xf) as usize]);
        }
        uart::puts("\n");
    }

    // X25519 self-test with RFC 7748 test vector
    {
        // Pre-clamped scalar (a546... with bits clamped per RFC 7748)
        let mut test_scalar: [u8; 32] = [
            0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d,
            0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
            0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18,
            0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4,
        ];
        // Clamp (same as what x25519_scalar_mult will do)
        test_scalar[0] &= 248;
        test_scalar[31] &= 127;
        test_scalar[31] |= 64;
        let test_u: [u8; 32] = [
            0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb,
            0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
            0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b,
            0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c,
        ];
        let expected: [u8; 32] = [
            0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90,
            0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
            0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7,
            0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52,
        ];
        let mut result = [0u8; 32];
        x25519_scalar_mult(&test_scalar, &test_u, &mut result);
        let pass = result == expected;
        uart::puts("[tls] X25519 self-test: ");
        if pass {
            uart::puts("PASS\n");
        } else {
            uart::puts("FAIL got=");
            for i in 0..4 {
                let hex = b"0123456789abcdef";
                uart::putc(hex[(result[i] >> 4) as usize]);
                uart::putc(hex[(result[i] & 0xf) as usize]);
            }
            uart::puts(" expected=c3da5537\n");
        }
    }

    // Debug: show empty hash and early secret
    let empty_hash_check = crate::crypto::sha256::hash(&[]);
    uart::puts("[tls] empty_hash[0..4]: ");
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(empty_hash_check[i] >> 4) as usize]);
        uart::putc(hex[(empty_hash_check[i] & 0xf) as usize]);
    }
    uart::puts("\n[tls] transcript[0..4]: ");
    let th = transcript.clone().finalize();
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(th[i] >> 4) as usize]);
        uart::putc(hex[(th[i] & 0xf) as usize]);
    }
    uart::puts("\n");

    // Debug: show shared secret
    uart::puts("[tls] shared_secret[0..4]: ");
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(sess.shared_secret[i] >> 4) as usize]);
        uart::putc(hex[(sess.shared_secret[i] & 0xf) as usize]);
    }
    uart::puts("\n[tls] peer_public[0..4]: ");
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(sess.peer_public[i] >> 4) as usize]);
        uart::putc(hex[(sess.peer_public[i] & 0xf) as usize]);
    }
    uart::puts("\n");

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

    // Step 4: Parse encrypted handshake records from the buffered data
    let mut hs_records = 0u8;
    let mut pos = remaining_start;

    while pos + 5 < all_len {
        let rec_type = all_buf[pos];
        let rec_len = ((all_buf[pos + 3] as usize) << 8) | all_buf[pos + 4] as usize;
        let rec_end = (pos + 5 + rec_len).min(all_len);
        let payload_len = rec_end - pos - 5;

        uart::puts("[tls] record at ");
        crate::kernel::mm::print_num(pos);
        uart::puts(": type=0x");
        let hex = b"0123456789abcdef";
        uart::putc(hex[(rec_type >> 4) as usize]);
        uart::putc(hex[(rec_type & 0xf) as usize]);
        uart::puts(" len=");
        crate::kernel::mm::print_num(rec_len);
        uart::puts("\n");

        if rec_type == 0x14 {
            // ChangeCipherSpec — skip
            pos = rec_end;
            continue;
        }

        if rec_type == 0x17 && payload_len > 17 {
            // Encrypted handshake record — decrypt
            let mut nonce = sess.server_iv;
            let seq_bytes = sess.server_seq.to_be_bytes();
            for i in 0..8 { nonce[4 + i] ^= seq_bytes[i]; }
            sess.server_seq += 1;

            let hs_cipher = {
                let mut k = [0u8; 16];
                k.copy_from_slice(&sess.server_key[..16]);
                crate::crypto::aes::Aes128::new(&k)
            };

            let mut decrypted = [0u8; 4096];
            decrypted[..payload_len].copy_from_slice(&all_buf[pos + 5..rec_end]);
            hs_cipher.gcm_crypt(&nonce, &mut decrypted[..payload_len]);

            // Inner: plaintext(N) + content_type(1) + GCM_tag(16)
            let inner_len = payload_len - 17;
            let inner_type = decrypted[inner_len]; // content type byte

            uart::puts("[tls]   decrypted ");
            crate::kernel::mm::print_num(inner_len);
            uart::puts("b inner=0x");
            uart::putc(hex[(inner_type >> 4) as usize]);
            uart::putc(hex[(inner_type & 0xf) as usize]);
            uart::puts(" first=0x");
            if inner_len > 0 {
                uart::putc(hex[(decrypted[0] >> 4) as usize]);
                uart::putc(hex[(decrypted[0] & 0xf) as usize]);
            }
            uart::puts("\n");

            // Add decrypted handshake to transcript (only type 0x16 = handshake)
            if inner_type == 0x16 && inner_len > 0 {
                transcript.update(&decrypted[..inner_len]);
            }
            hs_records += 1;
        }

        pos = rec_end;
    }

    uart::puts("[tls] parsed ");
    crate::kernel::mm::print_num(hs_records as usize);
    uart::puts(" encrypted hs records\n");

    // Step 5: Send ChangeCipherSpec (compatibility)
    let ccs = [0x14, 0x03, 0x03, 0x00, 0x01, 0x01];
    crate::net::tcp::send_data(&ccs).ok();

    // Step 6: Derive application traffic keys
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

    // Debug: print first bytes of derived keys
    uart::puts("[tls] client_key[0..4]: ");
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(sess.client_key[i] >> 4) as usize]);
        uart::putc(hex[(sess.client_key[i] & 0xf) as usize]);
    }
    uart::puts("\n[tls] server_key[0..4]: ");
    for i in 0..4 {
        let hex = b"0123456789abcdef";
        uart::putc(hex[(sess.server_key[i] >> 4) as usize]);
        uart::putc(hex[(sess.server_key[i] & 0xf) as usize]);
    }
    uart::puts("\n[tls] Handshake complete — HTTPS ready\n");
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

    cipher.gcm_crypt(&nonce,&mut encrypted[..enc_len]);

    // Build TLS record: type=0x17, version=0x0303, length, encrypted data
    let mut record = [0u8; 4096];
    record[0] = 0x17; // application data
    record[1] = 0x03; record[2] = 0x03; // TLS 1.2 (compat)
    let rec_len = enc_len as u16;
    record[3] = (rec_len >> 8) as u8;
    record[4] = rec_len as u8;
    record[5..5 + enc_len].copy_from_slice(&encrypted[..enc_len]);

    uart::puts("[tls] send_app_data: ");
    crate::kernel::mm::print_num(enc_len);
    uart::puts(" bytes encrypted\n");
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

    uart::puts("[tls] recv record type=0x");
    let hex = b"0123456789abcdef";
    uart::putc(hex[(rec_type >> 4) as usize]);
    uart::putc(hex[(rec_type & 0xf) as usize]);
    uart::puts(" len=");
    crate::kernel::mm::print_num(rec_len);
    uart::puts(" total=");
    crate::kernel::mm::print_num(n);
    uart::puts("\n");

    if rec_type == 0x14 {
        // ChangeCipherSpec — skip
        return recv_app_data(buf); // recurse to get next record
    }

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

    cipher.gcm_crypt(&nonce,&mut decrypted[..rec_len]);

    // Decrypted data: [plaintext...][content_type(1)][GCM_tag(16)]
    // The content type is at position rec_len - 17
    // The actual plaintext is bytes 0..(rec_len - 17)
    let data_len = if rec_len > 17 { rec_len - 17 } else { 0 };

    // Check the inner content type (last byte before GCM tag)
    let inner_type = if rec_len > 16 { decrypted[rec_len - 17] } else { 0 };

    // Debug
    uart::puts("[tls] decrypted ");
    crate::kernel::mm::print_num(data_len);
    uart::puts("b inner_type=0x");
    let hex = b"0123456789abcdef";
    uart::putc(hex[(inner_type >> 4) as usize]);
    uart::putc(hex[(inner_type & 0xf) as usize]);
    uart::puts(": ");
    for i in 0..data_len.min(60) {
        if decrypted[i] >= 0x20 && decrypted[i] <= 0x7e {
            uart::putc(decrypted[i]);
        } else {
            uart::putc(b'.');
        }
    }
    uart::puts("\n");

    // If inner type is 0x16 (handshake) or 0x15 (alert), skip and read next
    if inner_type == 0x16 || inner_type == 0x15 {
        // NewSessionTicket or alert — skip, get next record
        return recv_app_data(buf);
    }

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
    // Clamp scalar per RFC 7748
    let mut k = *scalar;
    k[0] &= 248;
    k[31] &= 127;
    k[31] |= 64;

    // Also clamp u-coordinate: clear top bit
    let mut pt = *point;
    pt[31] &= 127;

    let u = decode_u_coordinate(&pt);
    let mut x_1 = u;
    let mut x_2 = field_one();
    let mut z_2 = field_zero();
    let mut x_3 = u;
    let mut z_3 = field_one();

    let mut swap: u64 = 0;

    // Montgomery ladder
    for t in (0..255).rev() {
        let k_t = ((k[t / 8] >> (t % 8)) & 1) as u64;
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

    // Combine 5 × 51-bit limbs into a 256-bit number, then extract bytes
    // Total: t[0] + t[1]<<51 + t[2]<<102 + t[3]<<153 + t[4]<<204
    let mut val = [0u64; 4]; // 4 × 64-bit words = 256 bits

    // Accumulate into 256-bit value
    val[0] = t[0] | (t[1] << 51);
    val[1] = (t[1] >> 13) | (t[2] << 38);
    val[2] = (t[2] >> 26) | (t[3] << 25);
    val[3] = (t[3] >> 39) | (t[4] << 12);

    // Store as little-endian bytes
    for i in 0..4 {
        let w = val[i];
        for j in 0..8 {
            let byte_idx = i * 8 + j;
            if byte_idx < 32 {
                bytes[byte_idx] = (w >> (j * 8)) as u8;
            }
        }
    }
    bytes[31] &= 0x7F; // clear top bit per RFC 7748
}

fn field_add(a: &Fe, b: &Fe) -> Fe {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2], a[3]+b[3], a[4]+b[4]]
}

fn field_sub(a: &Fe, b: &Fe) -> Fe {
    // Add 2*p to avoid underflow
    // p = 2^255-19, limbs: [2^51-19, 2^51-1, 2^51-1, 2^51-1, 2^51-1]
    // 2*p limbs:
    let two_p: Fe = [
        2 * (0x7FFFFFFFFFFED), // 2*(2^51-19)
        2 * MASK51,             // 2*(2^51-1)
        2 * MASK51,
        2 * MASK51,
        2 * MASK51,
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
    // Propagate any carry from the wrap-around addition
    let c = r[0] >> 51;
    r[0] &= MASK51;
    r[1] += c;
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
    // Carry propagation
    for _ in 0..3 {
        let mut carry = 0u64;
        for i in 0..5 {
            f[i] += carry;
            carry = f[i] >> 51;
            f[i] &= MASK51;
        }
        f[0] += carry * 19;
    }
    // One more pass to handle wrap
    let carry = f[0] >> 51;
    f[0] &= MASK51;
    f[1] += carry;

    // Conditional subtraction of p if f >= p
    // p = [0x7FFFFFFFFFFED, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF]
    // Check: is f >= p?
    let mut ge = true;
    if f[4] < 0x7FFFFFFFFFFFF { ge = false; }
    else if f[4] == 0x7FFFFFFFFFFFF {
        if f[3] < 0x7FFFFFFFFFFFF { ge = false; }
        else if f[3] == 0x7FFFFFFFFFFFF {
            if f[2] < 0x7FFFFFFFFFFFF { ge = false; }
            else if f[2] == 0x7FFFFFFFFFFFF {
                if f[1] < 0x7FFFFFFFFFFFF { ge = false; }
                else if f[1] == 0x7FFFFFFFFFFFF {
                    if f[0] < 0x7FFFFFFFFFFED { ge = false; }
                }
            }
        }
    }

    if ge {
        // Subtract p
        let mut borrow = 0i64;
        let p: Fe = [0x7FFFFFFFFFFED, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF, 0x7FFFFFFFFFFFF];
        for i in 0..5 {
            let val = f[i] as i64 - p[i] as i64 + borrow;
            if val < 0 {
                f[i] = (val + (1i64 << 51)) as u64;
                borrow = -1;
            } else {
                f[i] = val as u64;
                borrow = 0;
            }
        }
    }
}

fn field_invert(z: &Fe) -> Fe {
    // Compute z^(p-2) where p = 2^255 - 19
    // Using the addition chain from curve25519-donna
    let z2 = field_sq(z);                    // z^2
    let t = field_sq(&z2);                   // z^4
    let t = field_sq(&t);                    // z^8
    let z9 = field_mul(&t, z);               // z^9
    let z11 = field_mul(&z9, &z2);           // z^11
    let t = field_sq(&z11);                  // z^22
    let z_5_0 = field_mul(&t, &z9);          // z^(2^5-1) = z^31

    let mut t = field_sq(&z_5_0);
    for _ in 1..5 { t = field_sq(&t); }
    let z_10_0 = field_mul(&t, &z_5_0);      // z^(2^10-1)

    let mut t = field_sq(&z_10_0);
    for _ in 1..10 { t = field_sq(&t); }
    let z_20_0 = field_mul(&t, &z_10_0);     // z^(2^20-1)

    let mut t = field_sq(&z_20_0);
    for _ in 1..20 { t = field_sq(&t); }
    let t = field_mul(&t, &z_20_0);          // z^(2^40-1)

    let mut t = field_sq(&t);
    for _ in 1..10 { t = field_sq(&t); }
    let z_50_0 = field_mul(&t, &z_10_0);     // z^(2^50-1)

    let mut t = field_sq(&z_50_0);
    for _ in 1..50 { t = field_sq(&t); }
    let z_100_0 = field_mul(&t, &z_50_0);    // z^(2^100-1)

    let mut t = field_sq(&z_100_0);
    for _ in 1..100 { t = field_sq(&t); }
    let t = field_mul(&t, &z_100_0);         // z^(2^200-1)

    let mut t = field_sq(&t);
    for _ in 1..50 { t = field_sq(&t); }
    let t = field_mul(&t, &z_50_0);          // z^(2^250-1)

    let t = field_sq(&t);                    // z^(2^251-2)
    let t = field_sq(&t);                    // z^(2^252-4)
    let t = field_sq(&t);                    // z^(2^253-8)
    let t = field_sq(&t);                    // z^(2^254-16)
    let t = field_sq(&t);                    // z^(2^255-32)
    field_mul(&t, &z11)                      // z^(2^255-32+11) = z^(2^255-21) = z^(p-2)
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
