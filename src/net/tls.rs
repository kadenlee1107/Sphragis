// Sphragis — TLS 1.3 Implementation
// Pure Rust, zero dependencies. Used for HTTPS in the secure network pipeline.
//
// TLS 1.3 simplified flow:
// Client → ServerHello (with key_share)
// Server → ServerHello (with key_share) + encrypted extensions + certificate + finished
// Client → Finished
// → Application data encrypted with derived keys
//
// Crypto primitives:
// X25519 for key exchange (Curve25519 ECDH)
// HKDF-SHA256 for key derivation
// AES-256-GCM for record encryption (using our existing AES)

use crate::drivers::uart;

// TLS 1.3 record types (RFC 8446 §5.1). Only the ones we actually
// emit/parse are named — others (TLS_APPLICATION_DATA, etc.) are
// matched against literals at the few sites that need them.
const TLS_HANDSHAKE: u8 = 22;

// Handshake message types we send or recognise. Other RFC 8446
// types (ENCRYPTED_EXTENSIONS, CERTIFICATE, CERTIFICATE_VERIFY,
// FINISHED) are matched as literals inside the encrypted-handshake
// dispatch rather than imported as named constants here.
const CLIENT_HELLO: u8 = 1;
const SERVER_HELLO: u8 = 2;

// Cipher suites we offer. ChaCha20-Poly1305 is not advertised — both
// AES-GCM suites pass FIPS-140 ciphers and are universally supported.
// AUDIT-CRYPTO-F11 (2026-05-16): TLS_AES_256_GCM_SHA384 (0x1302)
// is intentionally not advertised — see ClientHello builder. The
// constant remains for ergonomic reference / future plumbing.
#[allow(dead_code)]
const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
const TLS_AES_128_GCM_SHA256: u16 = 0x1301;

// Named groups. Extension codepoints (supported_versions=43,
// key_share=51, signature_algorithms=13, server_name=0) appear as
// literals at their single emit/parse sites — extracting them here
// would only help if more of the extension table got named.
const X25519: u16 = 29;
const X25519_MLKEM768: u16 = 0x11EC;

/// Toggle advertising of post-quantum hybrid key_share alongside
/// classical X25519. When `true`, ClientHello includes BOTH entries
/// and the server picks. When `false`, only X25519 is offered
/// (identical to pre-integration behaviour).
// /
/// Safe to flip — servers that don't understand 0x11EC MUST ignore it
/// per RFC 8446's extensibility rules.
// /
/// was `const`. Promoted to AtomicBool so the renderer's
/// fetch_https can disable the hybrid path for the duration of one
/// fetch. Our hybrid key-derivation has a real-world bug that
/// surfaces as "handshake record auth failed" against major HTTPS
/// servers (example.com, etc.) when the server picks the hybrid
/// group; falling back to plain X25519 handshakes cleanly. Toggle
/// is process-global (single-threaded kernel); restore on every
/// exit path so caves' production TLS stays PQ-protected.
use core::sync::atomic::{AtomicBool, Ordering as TlsOrdering};
static TLS_HYBRID_ENABLED_FLAG: AtomicBool = AtomicBool::new(true);

#[inline]
pub fn hybrid_enabled() -> bool {
    TLS_HYBRID_ENABLED_FLAG.load(TlsOrdering::Relaxed)
}

// Kept as a public toggle: callers that need to force-disable hybrid
// for one handshake (post-quantum interop debugging, mostly) flip this
// before tls::handshake() and restore it after. No in-tree caller on
// main; PR #6's pq-interop boot hook uses it explicitly.
#[allow(dead_code)]
#[inline]
pub fn set_hybrid_enabled(v: bool) {
    TLS_HYBRID_ENABLED_FLAG.store(v, TlsOrdering::Relaxed);
}

/// Did the most recent handshake on the legacy TLS PCB actually
/// negotiate the hybrid PQ group (X25519MLKEM768)? Used by the
/// `pq-interop-test` boot hook to assert that the server picked the
/// hybrid group rather than silently falling back to plain X25519 —
/// without that check the smoke would pass even if the hybrid wire
/// format were broken, since the classical group always succeeds.
/// Cfg-gated caller, so the lint only sees it as unused under the
/// default feature set.
#[cfg_attr(not(feature = "pq-interop-test"), allow(dead_code))]
#[inline]
pub fn last_handshake_used_hybrid() -> bool {
    session_mut(LEGACY_TLS_PCB).hybrid_used
}

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
    // (EC)DHE input for the TLS 1.3 key schedule. 32 bytes for classical
    // X25519; 64 bytes for hybrid X25519MLKEM768 (ml_kem_ss || x25519_ss
    // per draft-ietf-tls-ecdhe-mlkem-04 §3). shared_secret_len tracks the
    // active size; HKDF-Extract reads &shared_secret[..shared_secret_len].
    shared_secret: [u8; 64],
    shared_secret_len: usize,
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
    // AUDIT-CRYPTO-F1 / AUDIT-CRYPTO-F2 (2026-05-15): strict RFC 8446
    // §4.4 handshake-message ordering. The prior code processed
    // Certificate / CertificateVerify / Finished as independent arms
    // with no cross-message ordering check, so an MITM could (a) send
    // Finished without ever presenting CertificateVerify (the only
    // step that ties the cert to the live ECDH), or (b) send Finished
    // FIRST and then have any subsequent Certificate be skipped by
    // the post-Finished gate at line 908. Both fully bypass server
    // authentication on the X25519 the MITM negotiated with us.
    // Fix: track saw_cert + saw_cv, reject Finished unless both are
    // true. Also enforce no duplicates and no out-of-order processing.
    pub saw_cert: bool,
    pub saw_cv:   bool,
    // Integration #3 wiring (DESIGN_CRYPTO.md #5): PQ-hybrid
    // key_share material. Populated when ClientHello advertises
    // X25519MLKEM768 (0x11EC) alongside classical X25519. If the
    // server picks the hybrid group in its ServerHello, we
    // deserialize mlkem_dk_bytes and decapsulate the server's blob
    // instead of computing classical X25519(our_private, peer_public).
    //
    // ML-KEM-768 decapsulation key size per NIST FIPS 203 is 2400 B;
    // we store 2432 (rounded) so the fixed array accommodates the full
    // encoded form plus a small slack byte.
    pub hybrid_x25519_sk: [u8; 32],
    pub hybrid_mlkem_dk: [u8; 2432],
    pub hybrid_mlkem_dk_len: usize,
    pub hybrid_active: bool,   // we advertised hybrid this session
    pub hybrid_used:   bool,   // server actually picked hybrid
}

impl TlsSession {
    /// V8-ROOT-6: zero every secret in the session struct. Call this on any
    /// handshake-error exit path so partial key derivation can't leak via a
    /// later reader of the static session-pool memory (e.g. a reallocated
    /// slot for a new connection). Public buffers (expected_hostname,
    /// leftover, random nonces) are not secrets and are left alone — the
    /// caller resets them separately.
    pub fn zeroize_secrets(&mut self) {
        self.our_private   = [0; 32];
        self.shared_secret = [0; 64];
        self.shared_secret_len = 0;
        self.client_key    = [0; 32];
        self.server_key    = [0; 32];
        self.client_iv     = [0; 12];
        self.server_iv     = [0; 12];
        self.peer_spki       = [0; 512];
        self.peer_spki_len   = 0;
        self.peer_pubkey_alg = 0;
        self.state = TlsState::Initial;
    }
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
    shared_secret: [0; 64],
    shared_secret_len: 0,
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
    saw_cert: false,
    saw_cv: false,
    hybrid_x25519_sk: [0; 32],
    hybrid_mlkem_dk: [0; 2432],
    hybrid_mlkem_dk_len: 0,
    hybrid_active: false,
    hybrid_used: false,
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

/// Build a TLS 1.3 ClientHello message.
// /
/// V8-ROOT-1 / V8-IRQ-#12: random + X25519 keypair + ClientHello write
/// + session-state init are one critical section. Without IRQ mask, a
/// timer preempt mid-init lets a concurrent recv_app_data observe a
/// half-initialized session (state=Initial but client_random already
/// fresh) and could make decisions on it.
pub fn build_client_hello(pcb_id: usize, hostname: &str, buf: &mut [u8]) -> usize {
    let _g = crate::kernel::sync::IrqGuard::new();
    let sess = session_mut(pcb_id);
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
    // AUDIT-CRYPTO-F1/F2: reset handshake-ordering flags.
    sess.saw_cert = false;
    sess.saw_cv = false;
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

    // AUDIT-CRYPTO-F11 (2026-05-16): advertise only the cipher
    // suite we actually implement end-to-end. Prior code listed
    // both TLS_AES_128_GCM_SHA256 (0x1301) and TLS_AES_256_GCM_SHA384
    // (0x1302), but process_server_hello rejected anything other
    // than 0x1301 — the SHA-384 path through HKDF / key schedule
    // / AEAD sizes was never plumbed. Advertising a suite we won't
    // accept misleads peers AND any auditor sampling our
    // ClientHello.
    //
    // Honest single-suite advertise. CNSA 2.0 alignment
    // (AES-256-GCM-SHA384 end-to-end) is a follow-up wave that
    // requires real TLS peer testing to validate the SHA-384 key
    // schedule before shipping; can't be done autonomously.
    buf[pos] = 0; buf[pos+1] = 2; pos += 2; // length = 2 (1 suite)
    buf[pos] = (TLS_AES_128_GCM_SHA256 >> 8) as u8;
    buf[pos+1] = TLS_AES_128_GCM_SHA256 as u8; pos += 2;

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

    // Supported groups extension (required). Advertise hybrid first so
    // servers that support both pick it (server-side typically picks
    // the first mutually-supported group).
    buf[pos] = 0; buf[pos+1] = 10; pos += 2; // type = supported_groups
    if hybrid_enabled() {
        buf[pos] = 0; buf[pos+1] = 6;  pos += 2; // length = 2 + 4
        buf[pos] = 0; buf[pos+1] = 4;  pos += 2; // list length = 4
        buf[pos..pos+2].copy_from_slice(&X25519_MLKEM768.to_be_bytes()); pos += 2;
        buf[pos..pos+2].copy_from_slice(&X25519.to_be_bytes());          pos += 2;
    } else {
        buf[pos] = 0; buf[pos+1] = 4;  pos += 2; // length = 2 + 2
        buf[pos] = 0; buf[pos+1] = 2;  pos += 2; // list length
        buf[pos..pos+2].copy_from_slice(&X25519.to_be_bytes()); pos += 2;
    }

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

    // Key share extension. When hybrid is enabled we include BOTH a
    // classical X25519 key_share AND a hybrid X25519+ML-KEM-768 one.
    // Server picks one based on its supported_groups intersection.
    //
    // Size accounting (hybrid branch):
    // entry = 2 group + 2 len + 1216 payload = 1220 B per hybrid
    // X25519 entry = 2 + 2 + 32 = 36 B
    // client_key_share = 36 + 1220 = 1256 B
    // extension payload = 2 list_len + 1256 = 1258 B
    // extension = 2 type + 2 len + 1258 = 1262 B
    //
    // This is why the ClientHello buffer in `handshake_inner` was bumped
    // from 512 → 4096.
    buf[pos] = 0; buf[pos+1] = 51; pos += 2; // type = key_share
    let ks_len_pos = pos; pos += 2;          // ext len placeholder
    let ks_inner_len_pos = pos; pos += 2;    // inner list len placeholder

    if hybrid_enabled() {
        // Generate hybrid keypair for this session. Stash the decap
        // material in the session so we can complete the handshake
        // when/if the server picks hybrid.
        let kp = crate::crypto::pq_hybrid::HybridKeyPair::generate();
        let pub_bytes = kp.public_bytes();
        let sk_bytes = kp.x25519_sk_bytes();
        let dk_bytes = kp.mlkem_dk_bytes();
        sess.hybrid_x25519_sk = sk_bytes;
        let dk_n = dk_bytes.len().min(sess.hybrid_mlkem_dk.len());
        sess.hybrid_mlkem_dk[..dk_n].copy_from_slice(&dk_bytes[..dk_n]);
        sess.hybrid_mlkem_dk_len = dk_n;
        sess.hybrid_active = true;

        // Hybrid key_share entry (preferred)
        buf[pos..pos+2].copy_from_slice(&X25519_MLKEM768.to_be_bytes()); pos += 2;
        buf[pos] = (pub_bytes.len() >> 8) as u8;
        buf[pos+1] = pub_bytes.len() as u8; pos += 2;
        buf[pos..pos+pub_bytes.len()].copy_from_slice(&pub_bytes);
        pos += pub_bytes.len();
    } else {
        sess.hybrid_active = false;
    }

    // Classical X25519 entry (always sent as fallback)
    buf[pos..pos+2].copy_from_slice(&X25519.to_be_bytes()); pos += 2;
    buf[pos] = 0; buf[pos+1] = 32; pos += 2; // key length
    buf[pos..pos+32].copy_from_slice(&sess.our_public);
    pos += 32;

    // Fill in key_share lengths
    let ks_inner_len = pos - ks_inner_len_pos - 2;
    buf[ks_inner_len_pos]   = (ks_inner_len >> 8) as u8;
    buf[ks_inner_len_pos+1] = ks_inner_len as u8;
    let ks_ext_len = pos - ks_len_pos - 2;
    buf[ks_len_pos]   = (ks_ext_len >> 8) as u8;
    buf[ks_len_pos+1] = ks_ext_len as u8;

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
pub fn process_server_hello(pcb_id: usize, data: &[u8]) -> Result<(), &'static str> {
    if data.len() < 5 { return Err("too short"); }

    let sess = session_mut(pcb_id);

    // Skip record header (5 bytes)
    let content = &data[5..];
    if content.is_empty() { return Err("empty"); }

    // Parse handshake message
    let msg_type = content[0];
    if msg_type == SERVER_HELLO {
        // ServerHello layout (after handshake header):
        // [0] msg_type (0x02)
        // [1..4] length (3 bytes)
        // [4..6] version (0x0303)
        // [6..38] server_random (32 bytes)
        // [38] session_id_length
        // [39..39+sid_len] session_id
        // then: cipher_suite (2), compression (1), extensions_length (2), extensions...

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
        // V8-ROOT-3 / V8-ARITH-B1: pos + ext_len wraps under overflow-checks
        // and panics the kernel. Use checked_add.
        let ext_end = match pos.checked_add(ext_len) {
            Some(e) => e.min(content.len()),
            None => return Err("SH: ext_len+pos overflow"),
        };

        // Parse extensions: must find key_share (51) AND supported_versions
        // (43) reporting TLS 1.3. NET2-003: without the supported_versions
        // check, an active MITM can strip the extension and downgrade us —
        // the negotiated TLS 1.3 keys still derive, so the GCM tag check
        // won't notice a protocol downgrade.
        let mut saw_tls13 = false;
        let mut saw_key_share = false;
        // V8-ROOT-3: all pos arithmetic uses checked_add to prevent
        // panic-DoS under overflow-checks=true on attacker-controlled lengths.
        while let Some(after_hdr) = pos.checked_add(4) {
            if after_hdr > ext_end || after_hdr > content.len() { break; }
            let ext_type = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
            let ext_data_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
            let data_end = match after_hdr.checked_add(ext_data_len) {
                Some(e) if e <= ext_end && e <= content.len() => e,
                _ => break,
            };
            pos = after_hdr;

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
                if ext_data_len >= 4 {
                    let group = ((content[pos] as u16) << 8) | content[pos + 1] as u16;
                    let key_len = ((content[pos + 2] as usize) << 8) | content[pos + 3] as usize;
                    let key_start = pos + 4;
                    let key_end = match key_start.checked_add(key_len) {
                        Some(e) if e <= content.len() => e,
                        _ => break,
                    };

                    if group == X25519 && key_len == 32 {
                        sess.peer_public.copy_from_slice(&content[key_start..key_end]);
                        saw_key_share = true;
                        tdbg("[tls] found X25519 key_share\n");
                    } else if group == X25519_MLKEM768
                        && key_len == crate::crypto::pq_hybrid::HYBRID_CT_LEN
                        && sess.hybrid_active
                    {
                        // Integration: server picked hybrid. Decapsulate
                        // using our stored decap material; put the hybrid
                        // shared secret into sess.shared_secret so the
                        // HKDF schedule downstream consumes it unchanged.
                        let ct_blob = &content[key_start..key_end];
                        let dk_bytes = &sess.hybrid_mlkem_dk[..sess.hybrid_mlkem_dk_len];
                        match crate::crypto::pq_hybrid::decapsulate_from_bytes(
                            &sess.hybrid_x25519_sk, dk_bytes, ct_blob)
                        {
                            Ok(ss) => {
                                // 64-byte hybrid SS: ml_kem_ss || x25519_ss
                                // per draft-ietf-tls-ecdhe-mlkem-04 §3.
                                // Both halves go into the (EC)DHE input.
                                sess.shared_secret.copy_from_slice(&ss);
                                sess.shared_secret_len = 64;
                                sess.hybrid_used = true;
                                saw_key_share = true;
                                uart::puts("[tls] server selected X25519+ML-KEM-768 hybrid — decap OK\n");
                            }
                            Err(e) => {
                                uart::puts("[tls] hybrid decapsulate failed: ");
                                uart::puts(e);
                                uart::puts(" — abort\n");
                                return Err("hybrid decap failed");
                            }
                        }
                    }
                }
            }

            pos = data_end;
        }

        if !saw_tls13 {
            uart::puts("[tls] ServerHello missing supported_versions=TLS1.3 — abort\n");
            return Err("TLS: ServerHello did not select TLS 1.3 (downgrade?)");
        }
        if !saw_key_share {
            return Err("TLS: ServerHello missing X25519 key_share");
        }

        // Classical X25519 path only runs when the server did NOT pick
        // the hybrid group. (When hybrid is used, shared_secret was
        // already populated by decapsulate_from_bytes above.)
        if !sess.hybrid_used {
            // ATTACK-CRYPTO-008: reject small-order / identity X25519 peer
            // public keys. RFC 7748 §6.1 lists the 12 known low-order inputs;
            // on Curve25519 they all force shared_secret = 0, which would
            // derive known session keys. An active MITM can inject one of
            // these in the server key_share to pwn the handshake.
            if is_low_order_x25519(&sess.peer_public) {
                uart::puts("[tls] rejected low-order X25519 peer public\n");
                return Err("X25519 peer public key has small order");
            }
            // AUDIT-CRYPTO-F10: route ECDH through x25519-dalek.
            let classical_ss = x25519_dalek::x25519(sess.our_private, sess.peer_public);
            if classical_ss.iter().all(|&b| b == 0) {
                uart::puts("[tls] shared_secret is all-zero — abort\n");
                return Err("X25519 shared secret is zero");
            }
            sess.shared_secret[..32].copy_from_slice(&classical_ss);
            sess.shared_secret_len = 32;
        }

        sess.state = TlsState::ServerHelloReceived;
        tdbg("[tls] ServerHello processed\n");
        Ok(())
    } else {
        Err("not ServerHello")
    }
}

/// Perform the full TLS 1.3 handshake over an established TCP connection
/// on a specific PCB. The HTTPS syscall uses this on a freshly-allocated
/// PCB; the legacy `handshake` calls this against `LEGACY_TLS_PCB`.
/// Sends ClientHello, receives ServerHello, derives keys, handles encrypted
/// handshake.
pub fn handshake_pcb(pcb_id: usize, hostname: &str) -> Result<(), &'static str> {
    let result = handshake_inner(pcb_id, hostname);
    if result.is_err() {
        // V8-ROOT-6: zero every derived secret on any handshake failure so
        // the static session-pool slot doesn't leak partial key material to
        // the next caller who reuses this PCB.
        session_mut(pcb_id).zeroize_secrets();
    }
    result
}

/// Legacy single-PCB handshake — thin wrapper over `handshake_pcb` against
/// `LEGACY_TLS_PCB`. Existing callers (fetch debug helpers, dns selftest)
/// keep working unchanged.
pub fn handshake(hostname: &str) -> Result<(), &'static str> {
    handshake_pcb(LEGACY_TLS_PCB, hostname)
}

fn handshake_inner(pcb_id: usize, hostname: &str) -> Result<(), &'static str> {
    let sess = session_mut(pcb_id);
    sess.leftover_len = 0; // Reset leftover buffer for new session

    // Step 1: Send ClientHello
    // Bumped from 512 → 4096 to accommodate the optional hybrid
    // key_share entry (X25519+ML-KEM-768 adds ~1220 B).
    let mut ch_buf = [0u8; 4096];
    let ch_len = build_client_hello(pcb_id, hostname, &mut ch_buf);
    crate::net::tcp::send_data_blocking_pcb(pcb_id, &ch_buf[..ch_len])
        .map_err(|_| "send ClientHello failed")?;
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
        match crate::net::tcp::recv_data_blocking_pcb(pcb_id, &mut chunk) {
            Ok(n) if n > 0 => {
                let copy = n.min(all_buf.len() - all_len);
                all_buf[all_len..all_len + copy].copy_from_slice(&chunk[..copy]);
                all_len += copy;
            }
            _ => break,
        }
    }
    if all_len < 10 {
        // Tiny replies are usually a TLS Alert. Decode the level/desc
        // for the operator so they don't have to dig the bytes out of
        // the log to figure out why the handshake bailed.
        if all_len >= 7 && all_buf[0] == 0x15 {
            crate::drivers::uart::puts("    [tls] alert during handshake: level=");
            crate::kernel::mm::print_num(all_buf[5] as usize);
            crate::drivers::uart::puts(" desc=");
            crate::kernel::mm::print_num(all_buf[6] as usize);
            crate::drivers::uart::puts("\n");
        }
        return Err("ServerHello too short");
    }

    // Parse first record (ServerHello)
    let sh_rec_len = ((all_buf[3] as usize) << 8) | all_buf[4] as usize;
    let sh_end = (5 + sh_rec_len).min(all_len);
    process_server_hello(pcb_id, &all_buf[..sh_end])?;

    // Add ServerHello handshake to transcript (skip 5-byte record header)
    transcript.update(&all_buf[5..sh_end]);

    // Remaining bytes after ServerHello contain more records
    let remaining_start = sh_end;

    // Check if we have the complete encrypted handshake record
    // If not, keep reading until we do
    let mut need_more = true;
    while need_more && all_len < all_buf.len() - 4096 {
        need_more = false;
        let mut scan = sh_end;
        // V8-ROOT-3: scan-walk uses checked arithmetic; all peer-controlled
        // record-length deltas could otherwise wrap usize and panic.
        loop {
            let after_hdr = match scan.checked_add(5) {
                Some(n) if n <= all_len => n,
                _ => break,
            };
            let _rt = all_buf[scan];
            let rl = ((all_buf[scan + 3] as usize) << 8) | all_buf[scan + 4] as usize;
            let next = match after_hdr.checked_add(rl) {
                Some(n) => n,
                None => { need_more = true; break; }
            };
            if next > all_len {
                // Record extends beyond buffer — need more data
                need_more = true;
                break;
            }
            scan = next;
        }
        if need_more {
            let mut chunk = [0u8; 4096];
            match crate::net::tcp::recv_data_blocking_pcb(pcb_id, &mut chunk) {
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
    let handshake_secret = crate::crypto::sha256::hkdf_extract(
        &derived_secret,
        &sess.shared_secret[..sess.shared_secret_len],
    );

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

    // V8-ROOT-3: encrypted-handshake outer loop uses checked arithmetic on
    // peer-controlled rec_len.
    loop {
        let after_hdr = match pos.checked_add(5) {
            Some(n) if n < all_len => n,
            _ => break,
        };
        let rec_type = all_buf[pos];
        let rec_len = ((all_buf[pos + 3] as usize) << 8) | all_buf[pos + 4] as usize;
        let rec_end = match after_hdr.checked_add(rec_len) {
            Some(n) => n.min(all_len),
            None => break,
        };
        let payload_len = rec_end.saturating_sub(after_hdr);

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
            // V8-ROOT-3 (regression fix): checked_add so a 2^64-record session
            // doesn't panic the kernel. RFC 8446 §5.5 mandates rekey or
            // session-close long before this — we abort here as a safety net.
            sess.server_seq = match sess.server_seq.checked_add(1) {
                Some(n) => n,
                None => return Err("TLS: server record-sequence exhausted, close session"),
            };

            // ROOT-4: authenticated decryption for handshake records too.
            // Same AAD format as recv_app_data (5-byte TLS record header).
            let mut k16 = [0u8; 16];
            k16.copy_from_slice(&sess.server_key[..16]);
            let hs_gcm = crate::crypto::gcm_verified::Aes128Gcm::new(&k16);
            let aad = [
                all_buf[pos], all_buf[pos + 1], all_buf[pos + 2],
                all_buf[pos + 3], all_buf[pos + 4],
            ];

            // 17408 = 16 KB max TLS record + 1 KB headroom (matches the
            // recv_app_data side). 4 KB was tight: amazon.com's cert
            // chain runs ~4.3 KB and overflowed.
            let mut decrypted = [0u8; 17408];
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
                    sess.shared_secret = [0u8; 64];
                    sess.shared_secret_len = 0;
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
                    // V8-ROOT-3: msg_len is a 24-bit peer-controlled field;
                    // hp + 4 + msg_len could wrap usize on hostile inputs.
                    let msg_end = match hp.checked_add(4).and_then(|h| h.checked_add(msg_len)) {
                        Some(n) => n,
                        None => break,
                    };
                    if msg_end > inner_len { break; }

                    if msg_type == 0x0b {
                        // NEW-CRYPTO-010 / NET2-001: Certificate message.
                        // Parse: 1-byte ctx length + ctx + 3-byte certs_len
                        // + entries. First entry = leaf cert (3-byte len +
                        // cert DER + 2-byte exts_len + exts).
                        // AUDIT-CRYPTO-F2: reject duplicate Certificate.
                        if sess.saw_cert {
                            return Err("TLS: duplicate Certificate");
                        }
                        let body_start = hp + 4;
                        let body_end = match body_start.checked_add(msg_len) {
                            Some(n) if n <= decrypted.len() => n,
                            _ => return Err("TLS: Certificate body OOB"),
                        };
                        let body = &decrypted[body_start..body_end];
                        if body.len() < 4 {
                            return Err("TLS: Certificate body too short");
                        }
                        let ctx_len = body[0] as usize;
                        // V8-ROOT-3: ctx_len + 4 could wrap; use checked_add.
                        let ctx_check = match ctx_len.checked_add(4) {
                            Some(n) => n,
                            None => return Err("TLS: bad ctx_len"),
                        };
                        if ctx_check > body.len() { return Err("TLS: bad ctx_len"); }
                        let after_ctx = 1 + ctx_len;
                        let certs_len = ((body[after_ctx] as usize) << 16)
                                      | ((body[after_ctx + 1] as usize) << 8)
                                      |  (body[after_ctx + 2] as usize);
                        // V8-ROOT-3: certs_len is 24-bit peer-controlled;
                        // after_ctx + 3 + certs_len could wrap.
                        let certs_end = match after_ctx
                            .checked_add(3)
                            .and_then(|x| x.checked_add(certs_len))
                        {
                            Some(n) => n,
                            None => return Err("TLS: bad certs_len"),
                        };
                        if certs_end > body.len() {
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
                            let cert_end = match p.checked_add(clen) {
                                Some(n) if n <= end => n,
                                _ => return Err("TLS: cert entry truncated"),
                            };
                            certs.push(&body[p..cert_end]);
                            p = cert_end;
                            // Skip 2-byte extensions length + extensions.
                            let ext_len_start = match p.checked_add(2) {
                                Some(n) if n <= end => n,
                                _ => break,
                            };
                            let ext_len = ((body[p] as usize) << 8) | body[p + 1] as usize;
                            p = match ext_len_start.checked_add(ext_len) {
                                Some(n) if n <= end => n,
                                _ => return Err("TLS: Certificate extensions length overflow"),
                            };
                        }
                        if certs.is_empty() {
                            return Err("TLS: no certs in Certificate message");
                        }

                        let host = &sess.expected_hostname[..sess.expected_hostname_len];
                        let leaf = certs[0];
                        let intermediates: Vec<&[u8]> = certs[1..].to_vec();
                        match crate::net::x509::verify_chain(leaf, &intermediates, host) {
                            crate::net::x509::VerifyOutcome::Ok { pubkey_der, pubkey_algorithm } => {
                                sess.peer_spki_len = pubkey_der.len().min(sess.peer_spki.len());
                                sess.peer_spki[..sess.peer_spki_len]
                                    .copy_from_slice(&pubkey_der[..sess.peer_spki_len]);
                                sess.peer_pubkey_alg = pubkey_algorithm as u8;
                                // AUDIT-CRYPTO-F1/F2: only mark saw_cert
                                // after full verify_chain success. A failed
                                // chain doesn't advance the state machine,
                                // so a follow-up CertificateVerify still
                                // fails the saw_cert gate.
                                sess.saw_cert = true;
                                uart::puts("[tls] cert chain ok (x509)\n");
                            }
                            crate::net::x509::VerifyOutcome::Err(e) => {
                                return Err(e.as_static_str());
                            }
                        }
                    }

                    if msg_type == 0x0f {
                        // CertificateVerify (RFC 8446 §4.4.3).
                        // Body layout: 2B SignatureScheme + 2B length + sig.
                        // AUDIT-CRYPTO-F1/F2: require prior Certificate and
                        // reject duplicate CV. Without saw_cert, an MITM
                        // could try to skip Certificate and forge a CV against
                        // an SPKI we never received.
                        if !sess.saw_cert {
                            return Err("TLS: CertificateVerify before Certificate");
                        }
                        if sess.saw_cv {
                            return Err("TLS: duplicate CertificateVerify");
                        }
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
                                Ok(()) => {
                                    // AUDIT-CRYPTO-F1/F2: only mark saw_cv
                                    // after the signature actually verifies.
                                    sess.saw_cv = true;
                                    uart::puts("[tls] CertificateVerify ok\n");
                                }
                                Err(_) => {
                                    uart::puts("[tls] CertificateVerify FAILED — aborting\n");
                                    return Err("TLS: CertificateVerify failed");
                                }
                            }
                        }
                    }

                    if msg_type == 0x14 {
                        // Finished — verify BEFORE hashing it.
                        // AUDIT-CRYPTO-F1/F2 (2026-05-15): require BOTH
                        // saw_cert and saw_cv before processing Finished.
                        // Without this, an MITM that negotiated X25519 with
                        // us can forge the Finished MAC (which is keyed off
                        // the X25519 shared secret) and skip presenting any
                        // valid Certificate / CertificateVerify. The
                        // signature in CertificateVerify is the ONLY step
                        // that ties the cert to the live ECDH; skipping it
                        // breaks server authentication entirely.
                        if !sess.saw_cert {
                            return Err("TLS: Finished before Certificate (auth bypass attempt)");
                        }
                        if !sess.saw_cv {
                            return Err("TLS: Finished before CertificateVerify (auth bypass attempt)");
                        }
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
    crate::net::tcp::send_data_blocking_pcb(pcb_id, &ccs).ok();

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
    crate::net::tcp::send_data_blocking_pcb(pcb_id, &fin_record[..5 + enc_len]).ok();

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

/// Encrypt and send application data as a TLS record on a specific PCB.
/// The HTTPS syscall path uses this directly with the cave's allocated
/// PCB; the legacy `send_app_data` is a thin wrapper.
pub fn send_app_data_pcb(pcb_id: usize, data: &[u8]) -> Result<(), &'static str> {
    let sess = session_mut(pcb_id);
    if sess.state != TlsState::Established { return Err("not established"); }

    // Build nonce: IV XOR sequence number
    let mut nonce = sess.client_iv;
    let seq_bytes = sess.client_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    // V8-ROOT-3 (regression fix): checked_add. See comment at server_seq
    // increment in handshake_inner.
    sess.client_seq = match sess.client_seq.checked_add(1) {
        Some(n) => n,
        None => return Err("TLS: client record-sequence exhausted, close session"),
    };

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
    crate::net::tcp::send_data_blocking_pcb(pcb_id, &record[..5 + enc_len])
}

/// Legacy `send_app_data` — thin wrapper over `send_app_data_pcb` for the
/// `LEGACY_TLS_PCB` slot. Existing single-session callers keep working.
pub fn send_app_data(data: &[u8]) -> Result<(), &'static str> {
    send_app_data_pcb(LEGACY_TLS_PCB, data)
}

/// Receive and decrypt application data from a TLS record.
// /
/// NET2-006 / NEW-CRYPTO-023 fix: the previous implementation called
/// `recv_app_data(buf)` recursively on ChangeCipherSpec records (and the
/// 16 KB `record` / `decrypted` stack frames compounded with every call).
/// A server that streamed CCS records could overflow the kernel stack.
/// We now loop up to 8 times for CCS skipping instead of recursing.
pub fn recv_app_data_pcb(pcb_id: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    let sess = session_mut(pcb_id);
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
        n = crate::net::tcp::recv_data_blocking_pcb(pcb_id, &mut record).map_err(|e| {
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
            match crate::net::tcp::recv_data_blocking_pcb(pcb_id, &mut record[n..n + space]) {
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
    // V8-ROOT-3 (regression fix): checked_add. See comment at handshake path.
    sess.server_seq = match sess.server_seq.checked_add(1) {
        Some(n) => n,
        None => return Err("TLS: server record-sequence exhausted, close session"),
    };

    // Decrypt with AUTHENTICATION (ROOT-4).
    //
    // TLS 1.3 AEAD additional_data is the 5-byte record header:
    // type(1) || legacy_version(2) || length(2) (RFC 8446 §5.2).
    // The wire format for the ciphertext payload is
    // [ciphertext || 16-byte tag]
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

/// Legacy `recv_app_data` — thin wrapper over `recv_app_data_pcb` for the
/// `LEGACY_TLS_PCB` slot.
pub fn recv_app_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    recv_app_data_pcb(LEGACY_TLS_PCB, buf)
}

/// V5-XLAYER-001 fix: reset every TLS_STATES entry (not just the legacy
/// slot 0) so a cave switch wipes session keys, SPKI, expected_hostname,
/// and cert-pinning state inherited from a prior tenant. Called from
/// cave::enter on every switch.
// /
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
            s.saw_cert = false;
            s.saw_cv = false;
            zeroize(&mut s.shared_secret);
            s.shared_secret_len = 0;
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

/// V8-ROOT-6: panic-handler-only secret wipe. Uses volatile writes so the
/// compiler cannot DCE, no locks (panic handler may be holding them).
/// Best-effort. Zeroes the derived secrets in every PCB session slot.
// /
/// # Safety
/// May only be called from the panic handler (via wipe::emergency_wipe).
pub unsafe fn panic_wipe() {
    let base = core::ptr::addr_of_mut!(TLS_STATES) as *mut TlsSession;
    for i in 0..TLS_MAX_PCBS {
        let s = unsafe { base.add(i) };
        // Write each secret byte volatile so the compiler preserves it.
        let pv = unsafe { core::ptr::addr_of_mut!((*s).our_private) } as *mut u8;
        let ss = unsafe { core::ptr::addr_of_mut!((*s).shared_secret) } as *mut u8;
        let ck = unsafe { core::ptr::addr_of_mut!((*s).client_key) } as *mut u8;
        let sk = unsafe { core::ptr::addr_of_mut!((*s).server_key) } as *mut u8;
        let ci = unsafe { core::ptr::addr_of_mut!((*s).client_iv) } as *mut u8;
        let si = unsafe { core::ptr::addr_of_mut!((*s).server_iv) } as *mut u8;
        for j in 0..32 {
            unsafe {
                core::ptr::write_volatile(pv.add(j), 0);
                core::ptr::write_volatile(ck.add(j), 0);
                core::ptr::write_volatile(sk.add(j), 0);
            }
        }
        // shared_secret grew to 64 bytes for the hybrid PQ path; wipe all 64.
        for j in 0..64 {
            unsafe { core::ptr::write_volatile(ss.add(j), 0); }
        }
        for j in 0..12 {
            unsafe {
                core::ptr::write_volatile(ci.add(j), 0);
                core::ptr::write_volatile(si.add(j), 0);
            }
        }
    }
}

/// Close a TLS session on a specific PCB. Wipes all secret-bearing fields
/// and marks the slot Closed. Does NOT close the underlying TCP PCB —
/// that's the caller's job (e.g. `tcp::close_pcb`).
pub fn close_pcb(pcb_id: usize) {
    use crate::security::zeroize::zeroize;
    let sess = session_mut(pcb_id);
    sess.state = TlsState::Closed;
    sess.client_seq = 0;
    sess.server_seq = 0;
    // ATTACK-CRYPTO-010: volatile-wipe ALL secret-bearing fields, not
    // just the three keys the old impl touched. Cold-boot / HVF
    // snapshot / DMA can recover anything we leave behind.
    zeroize(&mut sess.shared_secret);
    sess.shared_secret_len = 0;
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
    // V8-ROOT-6: peer_spki (server public key) and expected_hostname
    // aren't "secrets" in the key sense but they identify what we were
    // talking to. Zero them too so a later snapshot can't prove the
    // session's counterparty.
    zeroize(&mut sess.peer_spki);
    sess.peer_spki_len = 0;
    sess.peer_pubkey_alg = 0;
    zeroize(&mut sess.expected_hostname);
    sess.expected_hostname_len = 0;
    sess.finished_seen = false;
    sess.saw_cert = false;
    sess.saw_cv = false;
}

/// Legacy `close()` — thin wrapper over `close_pcb` for `LEGACY_TLS_PCB`.
pub fn close() {
    close_pcb(LEGACY_TLS_PCB);
}


// ─── X25519 Key Exchange (Curve25519) ───

/// Generate X25519 keypair via the SHA-chained DRBG.
// /
/// NEW-CRYPTO-005: the previous implementation read `cntpct_el0` directly,
/// giving a passive observer who could estimate boot time ~2^20 candidate
/// scalars. The DRBG in `crate::crypto::rng` chains SHA-256 over 8 spaced
/// timer reads plus prior state, so a single observed output cannot recover
/// the scalar even with boot-time knowledge.
/// AUDIT-CRYPTO-F10 (2026-05-15): X25519 keypair generation now
/// routes through the audited x25519-dalek crate instead of the
/// hand-rolled Montgomery ladder + 5×51-bit field arithmetic that
/// used to live below. The hand-rolled version was correct after
/// V8-ROOT-12 made field_reduce constant-time, but it was the only
/// production path with two divergent implementations (the PQ
/// hybrid in `crypto::pq_hybrid` also used dalek). Consolidating to
/// dalek removes ~250 LOC of unverified curve arithmetic and ends
/// the two-implementation maintenance burden.
fn generate_x25519_keypair(private: &mut [u8; 32], public: &mut [u8; 32]) {
    crate::crypto::rng::fill_bytes(private);
    // dalek's `x25519(sk, basepoint)` applies the RFC 7748 clamp
    // internally; we don't need to clamp `private` ourselves. But
    // since callers compare `private` bytes for storage, we apply
    // the clamp here too so the stored bytes match the effective
    // scalar used for ECDH.
    private[0] &= 248;
    private[31] &= 127;
    private[31] |= 64;
    *public = x25519_dalek::x25519(*private, x25519_dalek::X25519_BASEPOINT_BYTES);
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


