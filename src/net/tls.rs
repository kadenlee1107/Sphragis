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

/// NET2-016 / NEW-CRYPTO-026 / debug-fingerprint scrub: TLS-internal debug
/// prints are gated behind this flag. The default is `false` so production
/// builds emit no per-record metadata over the UART (sequence numbers,
/// record lengths, inner types, "waiting for record", etc.). Flip to `true`
/// only when actively debugging TLS handshakes; do not ship `true`.
const TLS_DEBUG: bool = false;

#[inline(always)]
fn tdbg(s: &str) {
    if TLS_DEBUG { uart::puts(s); }
}
#[inline(always)]
fn tdbg_num(n: usize) {
    if TLS_DEBUG { crate::kernel::mm::print_num(n); }
}
#[inline(always)]
fn tdbg_byte(b: u8) {
    if TLS_DEBUG { uart::putc(b); }
}

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
    // ATTACK-CRYPTO-007: hostname the client expects to be talking to. Saved
    // at handshake start so the (future) X.509 SAN/CN check can compare
    // against it without having to thread the hostname back through the API.
    expected_hostname: [u8; 256],
    expected_hostname_len: usize,
    // V4: peer's cert SubjectPublicKeyInfo (DER) and its algorithm, used
    // to verify the TLS 1.3 CertificateVerify signature.
    peer_spki: [u8; 512],
    peer_spki_len: usize,
    peer_pubkey_alg: u8,
    // V6-PARSER-105 fix: V5's `finished_seen` was a local inside the
    // record loop, so once that record returned, a NEW encrypted
    // record could carry a second Certificate that overwrote
    // peer_spki. Moving this onto TlsSession makes it persistent
    // across records — once Finished is observed, no further
    // Certificate / CertificateVerify / EncryptedExtensions will
    // be accepted within the same handshake.
    pub finished_seen: bool,
}

// ATTACK-NET-034: one TLS session per TCP PCB instead of a single global.
//
// The public API (build_client_hello / handshake / send_app_data / …) still
// operates on the legacy PCB 0 slot so existing browser.rs call sites don't
// need to change. New `*_pcb(id, …)` variants let concurrent HTTPS connections
// each own their own keystream without clobbering each other.
const TLS_MAX_PCBS: usize = crate::net::tcp::MAX_PCBS;

const EMPTY_TLS_SESSION: TlsSession = TlsSession {
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
    expected_hostname: [0; 256],
    expected_hostname_len: 0,
    peer_spki: [0; 512],
    peer_spki_len: 0,
    peer_pubkey_alg: 0,
    finished_seen: false,
};

static mut TLS_STATES: [TlsSession; TLS_MAX_PCBS] =
    [EMPTY_TLS_SESSION; TLS_MAX_PCBS];

/// Legacy PCB used by the synchronous handshake/send/recv functions.
const LEGACY_TLS_PCB: usize = 0;

#[inline]
fn session_mut(id: usize) -> &'static mut TlsSession {
    let idx = if id < TLS_MAX_PCBS { id } else { LEGACY_TLS_PCB };
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TLS_STATES) as *mut TlsSession;
        &mut *ptr.add(idx)
    }
}

#[inline]
fn session_ref(id: usize) -> &'static TlsSession {
    let idx = if id < TLS_MAX_PCBS { id } else { LEGACY_TLS_PCB };
    unsafe {
        let ptr = core::ptr::addr_of!(TLS_STATES) as *const TlsSession;
        &*ptr.add(idx)
    }
}

// Back-compat shim: the legacy global is now slot 0 of the array.
#[allow(non_snake_case)]
fn SESSION_ptr() -> *mut TlsSession { session_mut(LEGACY_TLS_PCB) as *mut _ }

/// Build a TLS 1.3 ClientHello message.
pub fn build_client_hello(hostname: &str, buf: &mut [u8]) -> usize {
    let sess = session_mut(LEGACY_TLS_PCB);
    sess.state = TlsState::Initial;
    sess.client_seq = 0;
    sess.server_seq = 0;
    // ATTACK-CRYPTO-007: stash the hostname for later cert SAN/CN check.
    let hb = hostname.as_bytes();
    let hl = hb.len().min(sess.expected_hostname.len());
    sess.expected_hostname[..hl].copy_from_slice(&hb[..hl]);
    sess.expected_hostname_len = hl;
    // V6-PARSER-105: fresh handshake starts with finished_seen=false.
    sess.finished_seen = false;
    // Fresh handshake also clears any stale peer SPKI from a prior one.
    sess.peer_spki_len = 0;
    sess.peer_pubkey_alg = 0;

    // NEW-CRYPTO-005: route all TLS randomness through the SHA-chained DRBG
    // in `crypto::rng` instead of reading `cntpct_el0` directly. An observer
    // who can estimate boot time could otherwise narrow scalars to ~2^20.
    crate::crypto::rng::fill_bytes(&mut sess.client_random);

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
    // NEW-CRYPTO-005: independent DRBG draw, not a derivable XOR of client_random.
    buf[pos] = 32; pos += 1;
    let mut sid = [0u8; 32];
    crate::crypto::rng::fill_bytes(&mut sid);
    buf[pos..pos+32].copy_from_slice(&sid);
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

    // Supported versions extension — TLS 1.3 ONLY.
    // ATTACK-CRYPTO-008 fix: do not advertise TLS 1.2. The server-side
    // ServerHello parser already requires supported_versions=0x0304
    // (NET2-003), so advertising 1.2 only widened the cipher attack
    // surface and provided no useful fallback.
    buf[pos] = 0; buf[pos+1] = 43; pos += 2; // type = supported_versions
    buf[pos] = 0; buf[pos+1] = 3; pos += 2; // length
    buf[pos] = 2; pos += 1; // list length (1 version × 2 bytes)
    buf[pos] = 0x03; buf[pos+1] = 0x04; pos += 2; // TLS 1.3

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

    let sess = session_mut(LEGACY_TLS_PCB);

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

        // V5-WEIRD-001 fix: the ServerHello-selected cipher suite used
        // to be unconditionally skipped — the handshake was hard-wired
        // to AES-128-GCM-SHA256 regardless of what the server picked.
        // If the server picked TLS_AES_256_GCM_SHA384 (0x1302) or
        // TLS_CHACHA20_POLY1305_SHA256 (0x1303) our decrypt used the
        // wrong cipher / hash, producing record-auth failures that
        // looked like MITM when they weren't, and (worse) an MITM
        // could exploit the silent inconsistency. Now we require
        // AES-128-GCM-SHA256 or abort.
        if pos + 3 > content.len() { return Err("SH truncated before cipher"); }
        let selected_cs = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
        if selected_cs != TLS_AES_128_GCM_SHA256 {
            uart::puts("[tls] server selected unsupported cipher suite — abort\n");
            return Err("TLS: unsupported cipher suite");
        }
        pos += 3; // cipher (2) + compression (1)

        // Extensions length
        if pos + 2 > content.len() { return Err("SH no extensions"); }
        let ext_len = ((content[pos] as usize) << 8) | content[pos + 1] as usize;
        pos += 2;
        let ext_end = (pos + ext_len).min(content.len());

        // Parse extensions: must find key_share (51) AND supported_versions
        // (43) reporting TLS 1.3. NET2-003: without the supported_versions
        // check, an active MITM can strip the extension and downgrade us —
        // the negotiated TLS 1.3 keys still derive, so the GCM tag check
        // won't notice a protocol downgrade.
        let mut saw_tls13 = false;
        let mut saw_key_share = false;
        let ext_walk_end = ext_end.saturating_sub(4);
        while pos + 4 <= ext_walk_end + 4 && pos + 4 <= ext_end {
            if pos + 4 > content.len() { break; }
            let ext_type = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
            let ext_data_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
            pos += 4;
            if pos + ext_data_len > ext_end { break; }

            if ext_type == 43 && ext_data_len >= 2 {
                // supported_versions in ServerHello carries exactly one
                // selected_version (2 bytes). TLS 1.3 = 0x0304.
                let v = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
                if v == 0x0304 {
                    saw_tls13 = true;
                }
            }

            if ext_type == 51 {
                // key_share: named_group (2) + key_exchange_length (2) + key_exchange
                if ext_data_len >= 4 + 32 && pos + 4 + 32 <= content.len() {
                    let group = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
                    let key_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
                    if group == 29 && key_len == 32 {
                        sess.peer_public.copy_from_slice(&content[pos + 4..pos + 36]);
                        saw_key_share = true;
                        tdbg("[tls] found X25519 key_share\n");
                    }
                }
            }

            pos += ext_data_len;
        }

        if !saw_tls13 {
            uart::puts("[tls] ServerHello missing supported_versions=TLS1.3 — abort\n");
            return Err("TLS: ServerHello did not select TLS 1.3 (downgrade?)");
        }
        if !saw_key_share {
            return Err("TLS: ServerHello missing X25519 key_share");
        }

        // ATTACK-CRYPTO-008: reject small-order / identity X25519 peer
        // public keys. RFC 7748 §6.1 lists the 12 known low-order inputs;
        // on Curve25519 they all force shared_secret = 0, which would
        // derive known session keys. An active MITM can inject one of
        // these in the server key_share to pwn the handshake.
        if is_low_order_x25519(&sess.peer_public) {
            uart::puts("[tls] rejected low-order X25519 peer public\n");
            return Err("X25519 peer public key has small order");
        }

        // Compute shared secret via X25519
        x25519_scalar_mult(&sess.our_private, &sess.peer_public, &mut sess.shared_secret);

        // Defense in depth: even if the low-order check missed a new
        // point, if the shared secret is all-zero we've been pwned.
        // RFC 8446 §7.4.2 requires aborting in that case.
        if sess.shared_secret.iter().all(|&b| b == 0) {
            uart::puts("[tls] shared_secret is all-zero — abort\n");
            return Err("X25519 shared secret is zero");
        }

        sess.state = TlsState::ServerHelloReceived;
        tdbg("[tls] ServerHello processed\n");
        Ok(())
    } else {
        Err("not ServerHello")
    }
}

/// Perform the full TLS 1.3 handshake over an established TCP connection.
/// Sends ClientHello, receives ServerHello, derives keys, handles encrypted handshake.
pub fn handshake(hostname: &str) -> Result<(), &'static str> {
    let sess = session_mut(LEGACY_TLS_PCB);
    sess.leftover_len = 0; // Reset leftover buffer for new session

    // Step 1: Send ClientHello
    let mut ch_buf = [0u8; 512];
    let ch_len = build_client_hello(hostname, &mut ch_buf);
    crate::net::tcp::send_data(&ch_buf[..ch_len]).map_err(|_| "send ClientHello failed")?;
    tdbg("[tls] ClientHello sent\n");

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

    tdbg("[tls] buf=");
    tdbg_num(all_len);
    tdbg(" SH=");
    tdbg_num(sh_end);
    tdbg("\n");

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

    tdbg("[tls] Handshake keys derived\n");

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

            // ROOT-4: authenticated decryption for handshake records too.
            // Same AAD format as recv_app_data (5-byte TLS record header).
            let mut k16 = [0u8; 16];
            k16.copy_from_slice(&sess.server_key[..16]);
            let hs_gcm = crate::crypto::gcm_verified::Aes128Gcm::new(&k16);
            let aad = [
                all_buf[pos], all_buf[pos + 1], all_buf[pos + 2],
                all_buf[pos + 3], all_buf[pos + 4],
            ];

            let mut decrypted = [0u8; 4096];
            decrypted[..payload_len].copy_from_slice(&all_buf[pos + 5..rec_end]);

            let plaintext_len = match hs_gcm.decrypt_inplace(
                &nonce, &aad, &mut decrypted[..payload_len]
            ) {
                Ok(n) => n,
                Err(_e) => {
                    // NET2-008 / NEW-CRYPTO-026: wipe derived handshake
                    // keys on auth failure and emit only a generic
                    // "handshake auth failed" line. The old message
                    // forwarded the inner error string which could vary
                    // with cipher-state and leak protocol fingerprint.
                    uart::puts("[tls] handshake record auth failed\n");
                    sess.client_key = [0u8; 32];
                    sess.server_key = [0u8; 32];
                    sess.client_iv  = [0u8; 12];
                    sess.server_iv  = [0u8; 12];
                    sess.shared_secret = [0u8; 32];
                    sess.our_private   = [0u8; 32];
                    return Err("TLS handshake record authentication failed");
                }
            };

            // After authentication, inner plaintext ends with a 1-byte
            // content_type. The 16-byte tag has already been stripped
            // and verified by decrypt_inplace.
            let inner_len = if plaintext_len > 0 { plaintext_len - 1 } else { 0 };
            let inner_type = if plaintext_len > 0 { decrypted[inner_len] } else { 0 };

            if TLS_DEBUG {
                let hx = b"0123456789abcdef";
                uart::puts("[tls] hs inner=0x");
                uart::putc(hx[(inner_type >> 4) as usize]);
                uart::putc(hx[(inner_type & 0xf) as usize]);
                uart::puts(" len=");
                crate::kernel::mm::print_num(inner_len);
                uart::puts("\n");
            }

            // NEW-CRYPTO-011 / NET2-002: parse handshake messages inside the
            // decrypted record. For every msg_type != Finished(0x14) we add
            // it to the transcript as before; when we see Finished, we
            // compute the expected HMAC over the transcript-up-to-that-point
            // and constant-time-compare against the 32 bytes of verify_data.
            // Mismatch aborts the connection — this is the MITM defense
            // that was missing previously.
            if inner_type == 0x16 && inner_len > 0 {
                let mut hp = 0usize;
                // V6-PARSER-105 fix: read the persistent flag from
                // session state so subsequent RECORDS also refuse
                // post-Finished Certificate / CertificateVerify. The
                // V5 local-flag only covered the current record.
                while hp + 4 <= inner_len {
                    if sess.finished_seen { break; }
                    let msg_type = decrypted[hp];
                    let msg_len = ((decrypted[hp + 1] as usize) << 16)
                                | ((decrypted[hp + 2] as usize) << 8)
                                | (decrypted[hp + 3] as usize);
                    let msg_end = hp + 4 + msg_len;
                    if msg_end > inner_len { break; }

                    if msg_type == 0x0b {
                        // NEW-CRYPTO-010 / NET2-001: Certificate message.
                        // Parse: 1-byte ctx length + ctx + 3-byte certs_len
                        // + entries. First entry = leaf cert (3-byte len +
                        // cert DER + 2-byte exts_len + exts).
                        let body = &decrypted[hp + 4 .. hp + 4 + msg_len];
                        if body.len() < 4 {
                            return Err("TLS: Certificate body too short");
                        }
                        let ctx_len = body[0] as usize;
                        if ctx_len + 4 > body.len() { return Err("TLS: bad ctx_len"); }
                        let after_ctx = 1 + ctx_len;
                        let certs_len = ((body[after_ctx] as usize) << 16)
                                      | ((body[after_ctx + 1] as usize) << 8)
                                      |  (body[after_ctx + 2] as usize);
                        if after_ctx + 3 + certs_len > body.len() {
                            return Err("TLS: bad certs_len");
                        }

                        // V4: collect every cert in the chain (leaf first)
                        // into a Vec of DER slices, run full validation.
                        //
                        // V6-KMEM-005 fix: cap chain depth at 8 (more than
                        // any real-world cert chain) and per-cert DER at
                        // 32 KB (RFC 8446 ServerCertificate limit). These
                        // bound the recursion depth in x509-cert's DER
                        // parser and the heap-alloc total. Without them, a
                        // malicious chain with 100 deeply-nested SEQUENCEs
                        // could overflow the kernel stack into TLS_STATES
                        // (which sits in .bss right past the stack).
                        const MAX_CHAIN_DEPTH: usize = 8;
                        const MAX_CERT_DER:    usize = 32 * 1024;
                        use alloc::vec::Vec;
                        let mut certs: Vec<&[u8]> = Vec::new();
                        let mut p = after_ctx + 3;
                        let end = after_ctx + 3 + certs_len;
                        while p + 3 <= end {
                            if certs.len() >= MAX_CHAIN_DEPTH {
                                return Err("TLS: cert chain too deep");
                            }
                            let clen = ((body[p] as usize) << 16)
                                    | ((body[p + 1] as usize) << 8)
                                    |  (body[p + 2] as usize);
                            p += 3;
                            if clen > MAX_CERT_DER {
                                return Err("TLS: cert entry too large");
                            }
                            if p + clen > end { return Err("TLS: cert entry truncated"); }
                            certs.push(&body[p..p + clen]);
                            p += clen;
                            // Skip 2-byte extensions length + extensions.
                            if p + 2 > end { break; }
                            let ext_len = ((body[p] as usize) << 8) | body[p + 1] as usize;
                            p += 2 + ext_len;
                        }
                        if certs.is_empty() {
                            return Err("TLS: no certs in Certificate message");
                        }

                        let host = &sess.expected_hostname[..sess.expected_hostname_len];
                        let leaf = certs[0];
                        let intermediates: Vec<&[u8]> = certs[1..].iter().copied().collect();
                        match crate::net::x509::verify_chain(leaf, &intermediates, host) {
                            crate::net::x509::VerifyOutcome::Ok { pubkey_der, pubkey_algorithm } => {
                                sess.peer_spki_len = pubkey_der.len().min(sess.peer_spki.len());
                                sess.peer_spki[..sess.peer_spki_len]
                                    .copy_from_slice(&pubkey_der[..sess.peer_spki_len]);
                                sess.peer_pubkey_alg = pubkey_algorithm as u8;
                                uart::puts("[tls] cert chain ok (x509)\n");
                            }
                            crate::net::x509::VerifyOutcome::Err(e) => {
                                let _ = e;
                                // V5-CHAIN-001 / V5-CRYPTO-001: even on
                                // chain failure, ALWAYS extract the leaf
                                // SPKI so the CertificateVerify step can
                                // check the peer actually holds the key.
                                // Before this, fallback paths left
                                // peer_spki_len=0 and CertificateVerify
                                // was silently skipped = full MITM bypass.
                                match crate::net::x509::leaf_info(leaf) {
                                    Ok((spki, alg)) => {
                                        sess.peer_spki_len = spki.len().min(sess.peer_spki.len());
                                        sess.peer_spki[..sess.peer_spki_len]
                                            .copy_from_slice(&spki[..sess.peer_spki_len]);
                                        sess.peer_pubkey_alg = alg as u8;
                                    }
                                    Err(_) => {
                                        return Err("TLS: leaf cert unparseable");
                                    }
                                }
                                // V5-WEIRD uart-leak fix: do not distinguish
                                // x509-fail vs pin-ok via log timing. Same
                                // single log line for both outcomes.
                                match crate::net::tls_pinning::check_cert(host, leaf) {
                                    crate::net::tls_pinning::PinDecision::Match => {
                                        tdbg("[tls] leaf accepted (pin)\n");
                                    }
                                    crate::net::tls_pinning::PinDecision::Mismatch => {
                                        return Err("TLS: cert pin mismatch (MITM?)");
                                    }
                                    crate::net::tls_pinning::PinDecision::NoPin => {
                                        if crate::net::tls_pinning::STRICT_MODE {
                                            return Err("TLS: no pin / bad chain (strict)");
                                        } else {
                                            tdbg("[tls] leaf accepted (no pin, non-strict)\n");
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if msg_type == 0x0f {
                        // CertificateVerify (RFC 8446 §4.4.3).
                        // Body layout: 2B SignatureScheme + 2B length + sig.
                        let body = &decrypted[hp + 4 .. hp + 4 + msg_len];
                        if body.len() < 4 {
                            return Err("TLS: CertificateVerify too short");
                        }
                        let scheme = ((body[0] as u16) << 8) | body[1] as u16;
                        let sig_len = ((body[2] as usize) << 8) | body[3] as usize;
                        if body.len() < 4 + sig_len {
                            return Err("TLS: CertificateVerify truncated");
                        }
                        let sig_bytes = &body[4..4 + sig_len];
                        // Transcript hash covers ClientHello..Certificate.
                        let th = transcript.clone().finalize();
                        if sess.peer_spki_len == 0 {
                            // V5-CRYPTO-001 hardening: fail closed instead
                            // of silently skipping. If we got here without
                            // extracting SPKI at the Certificate step,
                            // something is very wrong (missing Certificate
                            // message, MITM with crafted ordering).
                            return Err("TLS: CertificateVerify without peer SPKI");
                        } else {
                            use crate::net::x509::PubkeyAlg;
                            let alg = match sess.peer_pubkey_alg {
                                x if x == PubkeyAlg::EcdsaP256 as u8 => PubkeyAlg::EcdsaP256,
                                x if x == PubkeyAlg::EcdsaP384 as u8 => PubkeyAlg::EcdsaP384,
                                x if x == PubkeyAlg::Rsa as u8 => PubkeyAlg::Rsa,
                                x if x == PubkeyAlg::Ed25519 as u8 => PubkeyAlg::Ed25519,
                                _ => PubkeyAlg::Unknown,
                            };
                            let spki = &sess.peer_spki[..sess.peer_spki_len];
                            match crate::net::x509::tls13_verify_cert_verify(
                                alg, spki, sig_bytes, &th, scheme,
                            ) {
                                Ok(()) => uart::puts("[tls] CertificateVerify ok\n"),
                                Err(_) => {
                                    uart::puts("[tls] CertificateVerify FAILED — aborting\n");
                                    return Err("TLS: CertificateVerify failed");
                                }
                            }
                        }
                    }

                    if msg_type == 0x14 {
                        // Finished — verify BEFORE hashing it.
                        if msg_len != 32 {
                            return Err("TLS: server Finished length != 32");
                        }
                        let finished_key = crate::crypto::sha256::hkdf_expand_label(
                            &server_hs_secret, b"finished", &[], 32);
                        let th = transcript.clone().finalize();
                        let expected = crate::crypto::sha256::hmac(&finished_key, &th);
                        let mut diff: u8 = 0;
                        for i in 0..32 {
                            diff |= expected[i] ^ decrypted[hp + 4 + i];
                        }
                        if diff != 0 {
                            uart::puts("[tls] server Finished HMAC mismatch — aborting\n");
                            return Err("TLS: server Finished HMAC mismatch (MITM?)");
                        }
                        tdbg("[tls] server Finished HMAC ok\n");
                        sess.finished_seen = true;
                    }

                    // Always feed the full handshake-message bytes into the
                    // transcript (matches RFC 8446 §4.4.1 Transcript-Hash).
                    transcript.update(&decrypted[hp..msg_end]);
                    hp = msg_end;
                }
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

    // V4: migrate Client Finished encryption off the legacy
    // Aes128::gcm_encrypt onto the audited gcm_verified implementation.
    // Functionally equivalent; removes the footgun that the legacy path
    // used pure-XOR gcm_crypt without tag verification if a caller ever
    // reached that arm.
    let mut k16 = [0u8; 16];
    k16.copy_from_slice(&sess.client_key[..16]);
    let hs_gcm = crate::crypto::gcm_verified::Aes128Gcm::new(&k16);
    let nonce = sess.client_iv;
    // Inner plaintext: finished_msg(36) + content_type(1) = 37 bytes
    let inner_len = 37;
    let enc_len = inner_len + 16; // + GCM tag
    let fin_aad = [0x17u8, 0x03, 0x03, (enc_len >> 8) as u8, enc_len as u8];

    // Buffer layout for encrypt_inplace: plaintext | 16-byte tag space.
    let mut fin_buf = [0u8; 80];
    fin_buf[..36].copy_from_slice(&finished_msg);
    fin_buf[36] = 0x16; // inner content type = handshake
    // encrypt_inplace encrypts fin_buf[..inner_len] and writes the tag
    // immediately after at fin_buf[inner_len..inner_len+16].
    hs_gcm.encrypt_inplace(&nonce, &fin_aad, &mut fin_buf[..enc_len], inner_len);

    let mut fin_record = [0u8; 80];
    fin_record[0] = 0x17;
    fin_record[1] = 0x03; fin_record[2] = 0x03;
    fin_record[3] = (enc_len >> 8) as u8;
    fin_record[4] = enc_len as u8;
    fin_record[5..5 + enc_len].copy_from_slice(&fin_buf[..enc_len]);
    crate::net::tcp::send_data(&fin_record[..5 + enc_len]).ok();

    tdbg("[tls] Client Finished sent\n");

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

    tdbg("[tls] Handshake complete\n");
    Ok(())
}

/// Encrypt and send application data as a TLS record.
pub fn send_app_data(data: &[u8]) -> Result<(), &'static str> {
    let sess = session_mut(LEGACY_TLS_PCB);
    if sess.state != TlsState::Established { return Err("not established"); }

    // Build nonce: IV XOR sequence number
    let mut nonce = sess.client_iv;
    let seq_bytes = sess.client_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    sess.client_seq += 1;

    // Encrypt with AES-128-GCM using the audited RustCrypto-backed impl.
    // Matches the recv-side migration: same primitives, same AAD format,
    // constant-time GHASH + tag construction.
    let mut key16 = [0u8; 16];
    key16.copy_from_slice(&sess.client_key[..16]);
    let gcm = crate::crypto::gcm_verified::Aes128Gcm::new(&key16);

    let mut plaintext = [0u8; 4096];
    let len = data.len().min(4000);
    plaintext[..len].copy_from_slice(&data[..len]);
    plaintext[len] = 0x17; // inner content type = application data
    let inner_len = len + 1;
    let enc_len = inner_len + 16; // ciphertext + GCM tag

    // AAD = TLS record header
    let aad = [0x17u8, 0x03, 0x03, (enc_len >> 8) as u8, enc_len as u8];

    // Buffer that will hold ciphertext + tag in place.
    let mut ct_and_tag = [0u8; 4096];
    ct_and_tag[..inner_len].copy_from_slice(&plaintext[..inner_len]);
    let written = gcm.encrypt_inplace(&nonce, &aad, &mut ct_and_tag, inner_len);
    debug_assert_eq!(written, enc_len);

    // Build TLS record
    let mut record = [0u8; 4096];
    record[0] = 0x17;
    record[1] = 0x03; record[2] = 0x03;
    record[3] = (enc_len >> 8) as u8;
    record[4] = enc_len as u8;
    record[5..5 + enc_len].copy_from_slice(&ct_and_tag[..enc_len]);

    tdbg("[tls] send_app_data\n");
    crate::net::tcp::send_data(&record[..5 + enc_len])
}

/// Receive and decrypt application data from a TLS record.
///
/// NET2-006 / NEW-CRYPTO-023 fix: the previous implementation called
/// `recv_app_data(buf)` recursively on ChangeCipherSpec records (and the
/// 16 KB `record` / `decrypted` stack frames compounded with every call).
/// A server that streamed CCS records could overflow the kernel stack.
/// We now loop up to 8 times for CCS skipping instead of recursing.
pub fn recv_app_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let sess = session_mut(LEGACY_TLS_PCB);
    if sess.state != TlsState::Established { return Err("not established"); }

    let mut record = [0u8; 17408]; // 16KB max TLS record + 1KB header/overhead

    // Loop at most a small fixed number of times so CCS / dummy records do
    // not become a stack-exhaustion vector.
    let mut ccs_skips = 0u32;
    loop {
    let mut n;

    // Use leftover data from previous recv if available
    if sess.leftover_len > 0 {
        let copy = sess.leftover_len.min(record.len());
        record[..copy].copy_from_slice(&sess.leftover[..copy]);
        n = copy;
        sess.leftover_len = 0;
    } else {
        tdbg("[tls] waiting for record...\n");
        n = crate::net::tcp::recv_data(&mut record).map_err(|e| {
            tdbg("[tls] recv error\n");
            let _ = e; // do not echo transport error string
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
        tdbg("[tls] incomplete record, reading more TCP\n");
        for _attempt in 0..8 {
            if rec_len + 5 <= n { break; }
            let space = record.len() - n;
            if space == 0 { break; }
            match crate::net::tcp::recv_data(&mut record[n..n + space]) {
                Ok(got) if got > 0 => { n += got; }
                Err(_e) => { break; }
                _ => { break; }
            }
        }
        if rec_len + 5 > n {
            tdbg("[tls] record still incomplete after recovery\n");
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

    if TLS_DEBUG {
        let hex = b"0123456789abcdef";
        uart::puts("[tls] record outer=0x");
        uart::putc(hex[(rec_type >> 4) as usize]);
        uart::putc(hex[(rec_type & 0xf) as usize]);
        uart::puts(" len=");
        crate::kernel::mm::print_num(rec_len);
        uart::puts("\n");
    }

    if rec_type == 0x14 {
        // ChangeCipherSpec — skip. Loop instead of recursing so a server
        // sending many CCS records cannot exhaust the kernel stack.
        ccs_skips += 1;
        if ccs_skips > 8 {
            return Err("TLS: too many ChangeCipherSpec records");
        }
        continue;
    }

    if rec_type != 0x17 { return Err("not app data"); }

    // Build nonce
    let mut nonce = sess.server_iv;
    let seq_bytes = sess.server_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    sess.server_seq += 1;

    // Decrypt with AUTHENTICATION (ROOT-4).
    //
    // TLS 1.3 AEAD additional_data is the 5-byte record header:
    // type(1) || legacy_version(2) || length(2)  (RFC 8446 §5.2).
    // The wire format for the ciphertext payload is
    //   [ciphertext || 16-byte tag]
    // where the LAST inner byte of the plaintext is the content type
    // (inner record type), optionally preceded by zero padding.
    //
    // The old code called gcm_crypt (pure XOR stream, no tag) and
    // relied on inner-content-type heuristics for integrity — that
    // was broken. Now we compute GHASH over AAD||ciphertext and
    // verify the tag in constant time BEFORE touching the plaintext.
    let mut key16 = [0u8; 16];
    key16.copy_from_slice(&sess.server_key[..16]);
    let gcm = crate::crypto::gcm_verified::Aes128Gcm::new(&key16);

    let aad = [record[0], record[1], record[2], record[3], record[4]];
    let mut decrypted = [0u8; 16896];
    let crypt_len = rec_len.min(decrypted.len());
    decrypted[..crypt_len].copy_from_slice(&record[5..5 + crypt_len]);

    let plaintext_len = match gcm.decrypt_inplace(&nonce, &aad, &mut decrypted[..crypt_len]) {
        Ok(n) => n,
        Err(_e) => {
            // NET2-008 / NEW-CRYPTO-026: wipe session-level keys on app
            // record auth failure. The caller's close() path does run, but
            // not every caller unwinds cleanly — zeroing here closes the
            // window where a follow-on bug could exfil the keys.
            uart::puts("[tls] record auth FAILED — closing session\n");
            sess.client_key = [0u8; 32];
            sess.server_key = [0u8; 32];
            sess.client_iv  = [0u8; 12];
            sess.server_iv  = [0u8; 12];
            return Err("TLS record authentication failed");
        }
    };

    // Authenticated plaintext is [data...][inner_content_type].
    // data_len = plaintext_len - 1 (strip the content type byte).
    let data_len = if plaintext_len > 0 { plaintext_len - 1 } else { 0 };
    let inner_type = if plaintext_len > 0 { decrypted[plaintext_len - 1] } else { 0 };

    if TLS_DEBUG {
        let hex2 = b"0123456789abcdef";
        uart::puts("[tls] decrypted seq=");
        crate::kernel::mm::print_num((sess.server_seq - 1) as usize);
        uart::puts(" ");
        crate::kernel::mm::print_num(data_len);
        uart::puts("b type=0x");
        uart::putc(hex2[(inner_type >> 4) as usize]);
        uart::putc(hex2[(inner_type & 0xf) as usize]);
        uart::puts("\n");
    }

    // Validate inner content type — if invalid, decryption probably failed
    // (wrong nonce/key producing garbage). Valid types: 0x14=CCS, 0x15=alert,
    // 0x16=handshake, 0x17=application data.
    if inner_type != 0x17 && inner_type != 0x16 && inner_type != 0x15 && inner_type != 0x14 {
        // Generic decryption-failed message; do not echo any inner state.
        uart::puts("[tls] decryption failed\n");
        return Err("decryption failed");
    }

    // If inner type is 0x16 (handshake = NewSessionTicket), skip it.
    // Don't recurse — return 0 so the caller can retry. This prevents
    // consuming all leftover data (including our response) during a
    // recursive call that then times out waiting for more data.
    if inner_type == 0x16 {
        tdbg("[tls] skipping NewSessionTicket\n");
        return Ok(0); // caller will retry and get actual app data
    }
    if inner_type == 0x15 {
        // NET2-031: parse the alert. RFC 8446 §6: { level: u8, description: u8 }
        // level 1=warning, 2=fatal; treat fatal (or close_notify=0) as hard close.
        let (level, desc) = if data_len >= 2 {
            (decrypted[0], decrypted[1])
        } else {
            (0, 0)
        };
        uart::puts("[tls] alert lvl=");
        crate::kernel::mm::print_num(level as usize);
        uart::puts(" desc=");
        crate::kernel::mm::print_num(desc as usize);
        uart::puts("\n");
        // Wipe session keys before returning — peer signalled termination.
        sess.client_key = [0u8; 32];
        sess.server_key = [0u8; 32];
        return Err("TLS alert");
    }

    let copy_len = data_len.min(buf.len());
    buf[..copy_len].copy_from_slice(&decrypted[..copy_len]);

    return Ok(copy_len);
    } // end loop
}

/// V5-XLAYER-001 fix: reset every TLS_STATES entry (not just the legacy
/// slot 0) so a cave switch wipes session keys, SPKI, expected_hostname,
/// and cert-pinning state inherited from a prior tenant. Called from
/// cave::enter on every switch.
///
/// V8-ROOT-1: wrap the 64-session wipe in a critical section. A timer
/// IRQ mid-loop would leave sessions 0..N cleared and N..64 live. The
/// caller (cave::enter) already holds an IrqGuard, so this is nestable.
pub fn reset_all_sessions() {
    use crate::security::zeroize::zeroize;
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        for i in 0..TLS_MAX_PCBS {
            let s = &mut (*core::ptr::addr_of_mut!(TLS_STATES))[i];
            s.state = TlsState::Initial;
            s.client_seq = 0;
            s.server_seq = 0;
            s.leftover_len = 0;
            s.peer_spki_len = 0;
            s.peer_pubkey_alg = 0;
            s.expected_hostname_len = 0;
            s.finished_seen = false;
            zeroize(&mut s.shared_secret);
            zeroize(&mut s.client_key);
            zeroize(&mut s.server_key);
            zeroize(&mut s.our_private);
            zeroize(&mut s.our_public);
            zeroize(&mut s.peer_public);
            zeroize(&mut s.client_iv);
            zeroize(&mut s.server_iv);
            zeroize(&mut s.client_random);
            zeroize(&mut s.server_random);
            zeroize(&mut s.peer_spki);
            zeroize(&mut s.expected_hostname);
        }
    }
}

/// Close TLS session.
pub fn close() {
    use crate::security::zeroize::zeroize;
    let sess = session_mut(LEGACY_TLS_PCB);
    sess.state = TlsState::Closed;
    sess.client_seq = 0;
    sess.server_seq = 0;
    // ATTACK-CRYPTO-010: volatile-wipe ALL secret-bearing fields, not
    // just the three keys the old impl touched. Cold-boot / HVF
    // snapshot / DMA can recover anything we leave behind.
    zeroize(&mut sess.shared_secret);
    zeroize(&mut sess.client_key);
    zeroize(&mut sess.server_key);
    zeroize(&mut sess.our_private);
    zeroize(&mut sess.peer_public);
    zeroize(&mut sess.client_iv);
    zeroize(&mut sess.server_iv);
    zeroize(&mut sess.client_random);
    zeroize(&mut sess.server_random);
    zeroize(&mut sess.leftover);
    sess.leftover_len = 0;
}

/// Check if TLS session is established.
pub fn is_established() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*SESSION_ptr()).state)) == TlsState::Established }
}

// ─── X25519 Key Exchange (Curve25519) ───

/// Generate X25519 keypair via the SHA-chained DRBG.
///
/// NEW-CRYPTO-005: the previous implementation read `cntpct_el0` directly,
/// giving a passive observer who could estimate boot time ~2^20 candidate
/// scalars. The DRBG in `crate::crypto::rng` chains SHA-256 over 8 spaced
/// timer reads plus prior state, so a single observed output cannot recover
/// the scalar even with boot-time knowledge.
fn generate_x25519_keypair(private: &mut [u8; 32], public: &mut [u8; 32]) {
    crate::crypto::rng::fill_bytes(private);

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
/// Known low-order / small-subgroup Curve25519 points (RFC 7748 §6.1).
/// Multiplying a scalar by any of these yields an all-zero shared secret
/// on the canonical curve, which an active MITM uses to fix the session
/// keys to known values. We reject in constant-time.
fn is_low_order_x25519(pk: &[u8; 32]) -> bool {
    // The 12 documented points. Each is 32 little-endian bytes.
    const LOW_ORDER: [[u8; 32]; 12] = [
        // 0 (identity)
        [0; 32],
        // 1
        [1, 0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0],
        // 325606250916557431795983626356110631294008115727848805560023387167927233504
        [0xe0,0xeb,0x7a,0x7c,0x3b,0x41,0xb8,0xae,0x16,0x56,0xe3,0xfa,0xf1,0x9f,0xc4,0x6a,
         0xda,0x09,0x8d,0xeb,0x9c,0x32,0xb1,0xfd,0x86,0x62,0x05,0x16,0x5f,0x49,0xb8,0x00],
        // 39382357235489614581723060781553021112529911719440698176882885853963445705823
        [0x5f,0x9c,0x95,0xbc,0xa3,0x50,0x8c,0x24,0xb1,0xd0,0xb1,0x55,0x9c,0x83,0xef,0x5b,
         0x04,0x44,0x5c,0xc4,0x58,0x1c,0x8e,0x86,0xd8,0x22,0x4e,0xdd,0xd0,0x9f,0x11,0x57],
        // p-1
        [0xec,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f],
        // p (all 0xff high bit clear 0xff..0x7f same as above? include a few more)
        [0xed,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f],
        [0xee,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f],
        [0xcd,0xeb,0x7a,0x7c,0x3b,0x41,0xb8,0xae,0x16,0x56,0xe3,0xfa,0xf1,0x9f,0xc4,0x6a,
         0xda,0x09,0x8d,0xeb,0x9c,0x32,0xb1,0xfd,0x86,0x62,0x05,0x16,0x5f,0x49,0xb8,0x80],
        [0x4c,0x9c,0x95,0xbc,0xa3,0x50,0x8c,0x24,0xb1,0xd0,0xb1,0x55,0x9c,0x83,0xef,0x5b,
         0x04,0x44,0x5c,0xc4,0x58,0x1c,0x8e,0x86,0xd8,0x22,0x4e,0xdd,0xd0,0x9f,0x11,0xd7],
        [0xd9,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff],
        [0xda,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff],
        [0xdb,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
         0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff],
    ];

    // NEW-CRYPTO-009 / NET2-017: RFC 7748 §5 says u-coordinate decoding
    // MUST mask the high bit of byte 31 before comparing. Without this,
    // an attacker could send any of the 12 points with the top bit set
    // and the compare would miss. We compare against a normalized copy.
    let mut pk_norm = *pk;
    pk_norm[31] &= 0x7f;

    // Constant-time: OR equality bits so the attacker can't use timing
    // to learn which point matched.
    let mut matched: u8 = 0;
    for p in LOW_ORDER.iter() {
        let mut pn = *p;
        pn[31] &= 0x7f;
        let mut acc: u8 = 0;
        for i in 0..32 {
            acc |= pn[i] ^ pk_norm[i];
        }
        // acc == 0 iff pk == p. Turn into a 1-bit mask without branching.
        let eq = ((acc as u16).wrapping_sub(1) >> 8) as u8 & 1;
        matched |= eq;
    }
    matched != 0
}

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
