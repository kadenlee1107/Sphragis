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
    // Leftover buffer: when TCP coalesces multiple TLS records into one read,
    // extra bytes after the first record are saved here for the next recv call.
    leftover: [u8; 17408],
    leftover_len: usize,
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
    leftover: [0; 17408],
    leftover_len: 0,
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

    // ALPN extension — advertise http/1.1 to prevent HTTP/2 negotiation
    buf[pos] = 0; buf[pos+1] = 16; pos += 2;  // type = ALPN (0x0010)
    buf[pos] = 0; buf[pos+1] = 11; pos += 2;  // extension length = 11
    buf[pos] = 0; buf[pos+1] = 9;  pos += 2;  // protocol list length = 9
    buf[pos] = 8; pos += 1;                    // protocol name length = 8
    let alpn = b"http/1.1";
    buf[pos..pos+8].copy_from_slice(alpn); pos += 8;

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
                        uart::puts("[tls] found X25519 key_share\n");
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
    sess.leftover_len = 0; // Reset leftover buffer for new session

    // Step 1: Send ClientHello
    let mut ch_buf = [0u8; 512];
    let ch_len = build_client_hello(hostname, &mut ch_buf);
    crate::net::tcp::send_data(&ch_buf[..ch_len]).map_err(|_| "send ClientHello failed")?;
    uart::puts("[tls] ClientHello sent\n");

    // Keep transcript hash of all handshake messages
    let ch_inner = &ch_buf[5..ch_len]; // skip record header
    let mut transcript = crate::crypto::sha256::Sha256::new();
    transcript.update(ch_inner);

    // Step 2: Receive ServerHello + all encrypted handshake records
    // Google's certificate is large — need a big buffer
    let mut all_buf = [0u8; 16384];
    let mut all_len = 0;
    // Read multiple chunks until we have all handshake data
    for _ in 0..10 {
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

    // Remaining bytes after ServerHello contain more records
    let mut remaining_start = sh_end;

    // Check if we have the complete encrypted handshake record
    // If not, keep reading until we do
    let mut need_more = true;
    while need_more && all_len < all_buf.len() - 4096 {
        need_more = false;
        let mut scan = sh_end;
        while scan + 5 < all_len {
            let rt = all_buf[scan];
            let rl = ((all_buf[scan + 3] as usize) << 8) | all_buf[scan + 4] as usize;
            if scan + 5 + rl > all_len {
                // Record extends beyond buffer — need more data
                need_more = true;
                break;
            }
            scan = scan + 5 + rl;
        }
        if need_more {
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
    }

    uart::puts("[tls] buf=");
    crate::kernel::mm::print_num(all_len);
    uart::puts(" SH=");
    crate::kernel::mm::print_num(sh_end);
    uart::puts("\n");

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

    // Step 4: Parse encrypted handshake records from the buffered data
    let mut pos = remaining_start;

    while pos + 5 < all_len {
        let rec_type = all_buf[pos];
        let rec_len = ((all_buf[pos + 3] as usize) << 8) | all_buf[pos + 4] as usize;
        let rec_end = (pos + 5 + rec_len).min(all_len);
        let payload_len = rec_end - pos - 5;

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

            uart::puts("[tls] hs inner=0x");
            let hx = b"0123456789abcdef";
            uart::putc(hx[(inner_type >> 4) as usize]);
            uart::putc(hx[(inner_type & 0xf) as usize]);
            uart::puts(" len=");
            crate::kernel::mm::print_num(inner_len);
            uart::puts("\n");

            // Add decrypted handshake to transcript (only type 0x16 = handshake)
            if inner_type == 0x16 && inner_len > 0 {
                transcript.update(&decrypted[..inner_len]);
            }
        }

        pos = rec_end;
    }

    // Step 5: Send ChangeCipherSpec (compatibility) + client Finished
    let ccs = [0x14, 0x03, 0x03, 0x00, 0x01, 0x01];
    crate::net::tcp::send_data(&ccs).ok();

    // Compute and send client Finished
    let finished_transcript = transcript.clone().finalize();
    let finished_key = crate::crypto::sha256::hkdf_expand_label(&client_hs_secret, b"finished", &[], 32);
    let verify_data = crate::crypto::sha256::hmac(&finished_key, &finished_transcript);

    // Build Finished handshake message: type=0x14, length=32, verify_data
    let mut finished_msg = [0u8; 36];
    finished_msg[0] = 0x14; // Finished
    finished_msg[1] = 0;
    finished_msg[2] = 0;
    finished_msg[3] = 32; // length
    finished_msg[4..36].copy_from_slice(&verify_data);

    // IMPORTANT: derive app keys from transcript BEFORE adding client Finished
    // RFC 8446 Section 7.1: app secrets use Transcript-Hash(CH..server Finished)
    let hs_transcript = transcript.clone().finalize();

    // NOW add client Finished to transcript (for resumption, not for app keys)
    transcript.update(&finished_msg);

    // Encrypt with client handshake key
    let client_hs_cipher = {
        let mut k = [0u8; 16];
        k.copy_from_slice(&sess.client_key[..16]);
        crate::crypto::aes::Aes128::new(&k)
    };
    let nonce = sess.client_iv;
    // Inner plaintext: finished_msg(36) + content_type(1) = 37 bytes
    let mut fin_plain = [0u8; 64];
    fin_plain[..36].copy_from_slice(&finished_msg);
    fin_plain[36] = 0x16; // inner content type = handshake
    let inner_len = 37;
    let enc_len = inner_len + 16; // + GCM tag

    // AAD = record header
    let fin_aad = [0x17u8, 0x03, 0x03, (enc_len >> 8) as u8, enc_len as u8];
    let fin_tag = client_hs_cipher.gcm_encrypt(&nonce, &fin_aad, &mut fin_plain[..inner_len]);

    // Build TLS record
    let mut fin_record = [0u8; 80];
    fin_record[0] = 0x17;
    fin_record[1] = 0x03; fin_record[2] = 0x03;
    fin_record[3] = (enc_len >> 8) as u8;
    fin_record[4] = enc_len as u8;
    fin_record[5..5 + inner_len].copy_from_slice(&fin_plain[..inner_len]);
    fin_record[5 + inner_len..5 + enc_len].copy_from_slice(&fin_tag);
    crate::net::tcp::send_data(&fin_record[..5 + enc_len]).ok();

    uart::puts("[tls] Client Finished sent\n");

    // Step 6: Derive application traffic keys (using hs_transcript from above)

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

    // Post-handshake records (NewSessionTickets) are handled inline
    // by recv_app_data — it skips inner_type 0x16 and recurses.

    uart::puts("[tls] Handshake complete\n");
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

    // Encrypt with AES-128-GCM (with real GHASH auth tag)
    let cipher = {let mut k=[0u8;16]; k.copy_from_slice(&sess.client_key[..16]); crate::crypto::aes::Aes128::new(&k)};
    let mut plaintext = [0u8; 4096];
    let len = data.len().min(4000);
    plaintext[..len].copy_from_slice(&data[..len]);
    plaintext[len] = 0x17; // inner content type = application data
    let inner_len = len + 1;
    let enc_len = inner_len + 16; // ciphertext + GCM tag

    // AAD = TLS record header
    let aad = [0x17u8, 0x03, 0x03, (enc_len >> 8) as u8, enc_len as u8];

    // Encrypt and get real auth tag
    let tag = cipher.gcm_encrypt(&nonce, &aad, &mut plaintext[..inner_len]);

    // Build TLS record
    let mut record = [0u8; 4096];
    record[0] = 0x17;
    record[1] = 0x03; record[2] = 0x03;
    record[3] = (enc_len >> 8) as u8;
    record[4] = enc_len as u8;
    record[5..5 + inner_len].copy_from_slice(&plaintext[..inner_len]);
    record[5 + inner_len..5 + enc_len].copy_from_slice(&tag);

    uart::puts("[tls] send_app_data\n");
    crate::net::tcp::send_data(&record[..5 + enc_len])
}

/// Receive and decrypt application data from a TLS record.
pub fn recv_app_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let sess = unsafe { &mut *core::ptr::addr_of_mut!(SESSION) };
    if sess.state != TlsState::Established { return Err("not established"); }

    let mut record = [0u8; 17408]; // 16KB max TLS record + 1KB header/overhead
    let mut n;

    // Use leftover data from previous recv if available
    if sess.leftover_len > 0 {
        let copy = sess.leftover_len.min(record.len());
        record[..copy].copy_from_slice(&sess.leftover[..copy]);
        n = copy;
        sess.leftover_len = 0;
    } else {
        uart::puts("[tls] waiting for record...\n");
        n = crate::net::tcp::recv_data(&mut record).map_err(|e| {
            uart::puts("[tls] recv error: ");
            uart::puts(e);
            uart::puts("\n");
            e
        })?;
    }
    if n < 5 { return Err("record too short"); }

    // Parse record header
    let rec_type = record[0];
    let rec_len = ((record[3] as usize) << 8) | record[4] as usize;

    // If record is incomplete, read more TCP data directly into the record buffer.
    // IMPORTANT: read into record[n..] (not a small tmp buffer) to capture ALL
    // available TCP data. recv_data discards excess beyond buf.len()!
    if rec_len + 5 > n {
        uart::puts("[tls] incomplete: need ");
        crate::kernel::mm::print_num(rec_len + 5);
        uart::puts(" have ");
        crate::kernel::mm::print_num(n);
        uart::puts(", reading more TCP...\n");
        for attempt in 0..8 {
            if rec_len + 5 <= n { break; }
            let space = record.len() - n;
            if space == 0 { break; }
            match crate::net::tcp::recv_data(&mut record[n..n + space]) {
                Ok(got) if got > 0 => {
                    uart::puts("[tls] got ");
                    crate::kernel::mm::print_num(got);
                    uart::puts("b more, total=");
                    crate::kernel::mm::print_num(n + got);
                    uart::puts("\n");
                    n += got;
                }
                Err(e) => {
                    uart::puts("[tls] recovery recv failed: ");
                    uart::puts(e);
                    uart::puts("\n");
                    break;
                }
                _ => { break; }
            }
        }
        if rec_len + 5 > n {
            uart::puts("[tls] record still incomplete after recovery\n");
            return Err("incomplete record");
        }
    }

    // Save any leftover bytes after this record for the next call
    let consumed = rec_len + 5;
    if n > consumed {
        let extra = n - consumed;
        let save = extra.min(sess.leftover.len());
        sess.leftover[..save].copy_from_slice(&record[consumed..consumed + save]);
        sess.leftover_len = save;
    }

    // Debug: log record details
    uart::puts("[tls] record: outer_type=0x");
    let hex = b"0123456789abcdef";
    uart::putc(hex[(rec_type >> 4) as usize]);
    uart::putc(hex[(rec_type & 0xf) as usize]);
    uart::puts(" rec_len=");
    crate::kernel::mm::print_num(rec_len);
    uart::puts(" n=");
    crate::kernel::mm::print_num(n);
    uart::puts(" seq=");
    crate::kernel::mm::print_num(sess.server_seq as usize);
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
    let mut decrypted = [0u8; 16896]; // 16KB max TLS record payload
    let crypt_len = rec_len.min(decrypted.len());
    let enc_data = &record[5..5 + crypt_len];
    decrypted[..crypt_len].copy_from_slice(enc_data);

    cipher.gcm_crypt(&nonce,&mut decrypted[..crypt_len]);

    // Decrypted data: [plaintext...][content_type(1)][GCM_tag(16)]
    // The content type is at position rec_len - 17
    // The actual plaintext is bytes 0..(rec_len - 17)
    let data_len = if rec_len > 17 { rec_len - 17 } else { 0 };

    // Check the inner content type (last byte before GCM tag)
    let inner_type = if rec_len > 16 { decrypted[rec_len - 17] } else { 0 };

    uart::puts("[tls] decrypted seq=");
    crate::kernel::mm::print_num((sess.server_seq - 1) as usize);
    uart::puts(" ");
    crate::kernel::mm::print_num(data_len);
    uart::puts("b type=0x");
    let hex2 = b"0123456789abcdef";
    uart::putc(hex2[(inner_type >> 4) as usize]);
    uart::putc(hex2[(inner_type & 0xf) as usize]);
    uart::puts(": ");
    for i in 0..data_len.min(80) {
        if decrypted[i] >= 0x20 && decrypted[i] <= 0x7e {
            uart::putc(decrypted[i]);
        } else {
            uart::putc(b'.');
        }
    }
    uart::puts("\n");

    // Debug: show bytes around inner_type position
    if rec_len > 20 {
        uart::puts("[tls] bytes at inner_type pos: ");
        let start = if rec_len > 22 { rec_len - 22 } else { 0 };
        for i in start..rec_len.min(start + 25) {
            uart::putc(hex[(decrypted[i] >> 4) as usize]);
            uart::putc(hex[(decrypted[i] & 0xf) as usize]);
            if i == rec_len - 17 { uart::puts("[<-IT]"); }
            uart::putc(b' ');
        }
        uart::puts("\n");
    }

    // Validate inner content type — if invalid, decryption probably failed
    // (wrong nonce/key producing garbage). Valid types: 0x14=CCS, 0x15=alert,
    // 0x16=handshake, 0x17=application data.
    if inner_type != 0x17 && inner_type != 0x16 && inner_type != 0x15 && inner_type != 0x14 {
        uart::puts("[tls] bad inner_type after decrypt — decryption failed\n");
        return Err("decryption failed");
    }

    // If inner type is 0x16 (handshake = NewSessionTicket), skip it.
    // Don't recurse — return 0 so the caller can retry. This prevents
    // consuming all leftover data (including our response) during a
    // recursive call that then times out waiting for more data.
    if inner_type == 0x16 {
        uart::puts("[tls] skipping NewSessionTicket\n");
        return Ok(0); // caller will retry and get actual app data
    }
    if inner_type == 0x15 {
        // Alert
        uart::puts("[tls] alert received\n");
        return Err("TLS alert");
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
        z_2 = field_mul(&e, &field_add(&aa, &field_mul_a24(&e)));
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

/// Multiply field element by a24 = (A-2)/4 = 121665 for Curve25519.
fn field_mul_a24(a: &Fe) -> Fe {
    let mut r = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..5 {
        let v = (a[i] as u128) * 121665 + carry;
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
