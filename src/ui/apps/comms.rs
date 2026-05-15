#![allow(dead_code)]
extern crate alloc;
// Sphragis — Comms Client (8th Desktop App)
//
// Real end-to-end encrypted peer-to-peer messaging over TCP. Wire
// protocol matches `scripts/comms_test_server.py`:
//
//   1. After TCP connect, both sides send a 128-byte handshake offer
//      (eph_pub || id_pub || ed25519_sig). Same shape as
//      `caves::ipc_session::build_offer`.
//   2. Both compute X25519(my_eph_sk, peer_eph_pub) and derive
//      directional keys via SHA-256:
//         c2s_key = SHA-256(b"SPHRAGIS-COMMS-c2s-v1" || shared
//                           || client_eph_pk || server_eph_pk)
//         s2c_key = SHA-256(b"SPHRAGIS-COMMS-s2c-v1" || shared ...)
//   3. Transport frames: len(4 BE) || nonce(12) || ct || tag(16).
//      ChaCha20-Poly1305. Separate counter per direction starting at 0.
//
// The caller must pin the expected server identity (Ed25519 pub key)
// at connect time — the shell command takes it as a hex argument and
// passes it here, so the offer-verify step rejects any MITM that
// can't sign as the real server.

use crate::ui::font;
use crate::ui::gpu;
use crate::crypto::{chacha20poly1305 as cp, sha256};
use ed25519_compact::{KeyPair, PublicKey, SecretKey, Seed, Signature};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};

// Connection state
#[derive(Clone, Copy, PartialEq)]
pub enum CommState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Read the current connection state. Used by the shell `comms`
/// command to check whether `send` should proceed and to print
/// `status` on demand.
pub fn state() -> CommState {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(STATE)) }
}

// Message log
const MAX_MESSAGES: usize = 32;
const MAX_MSG_LEN: usize = 80;

#[derive(Clone, Copy)]
struct ChatMsg {
    active: bool,
    outgoing: bool, // true = we sent it
    text: [u8; MAX_MSG_LEN],
    text_len: usize,
    timestamp: u64, // seconds since boot
}

impl ChatMsg {
    const fn empty() -> Self {
        ChatMsg { active: false, outgoing: false, text: [0; MAX_MSG_LEN], text_len: 0, timestamp: 0 }
    }
}

static mut STATE: CommState = CommState::Disconnected;
static mut MESSAGES: [ChatMsg; MAX_MESSAGES] = [ChatMsg::empty(); MAX_MESSAGES];
static mut MSG_COUNT: usize = 0;
// Per-direction transport keys, derived from the X25519 shared secret.
static mut C2S_KEY: [u8; 32] = [0; 32];
static mut S2C_KEY: [u8; 32] = [0; 32];
// Frame counters used to construct the 12-byte nonce. u64 big-endian
// padded with 4 zero bytes — overflow at 2^64 frames, never reached.
static mut SEND_CTR: u64 = 0;
static mut RECV_CTR: u64 = 0;
static mut PEER_IP: u32 = 0;
static mut PEER_PORT: u16 = 0;
// Pinned server identity. Required at connect time — without it we
// don't know what public key we're supposed to verify the offer
// against, and the protocol degrades to TOFU at best.
static mut PINNED_SERVER_ID: [u8; 32] = [0; 32];

const LABEL: &[u8] = b"SPHRAGIS-COMMS-v1";
const OFFER_LEN: usize = 32 + 32 + 64;
const KEY_DIR_C2S: &[u8] = b"SPHRAGIS-COMMS-c2s-v1";
const KEY_DIR_S2C: &[u8] = b"SPHRAGIS-COMMS-s2c-v1";

/// BatFS path for our persistent per-cave Ed25519 identity. 32-byte
/// raw seed. Persisting it across boots is what makes server-side
/// allowlists meaningful — without persistence, each session's
/// "identity" would be ephemeral and the server couldn't pin us.
const IDENTITY_PATH: &str = "comms_identity.key";

/// Lazy-loaded session identity. Generated + persisted to BatFS on
/// first call; reused for subsequent sessions in the same cave.
/// On cave switch the cached value is cleared via
/// `reset_for_cave_switch` so the new tenant doesn't inherit the
/// previous cave's identity.
static mut MY_IDENTITY_PK: [u8; 32] = [0; 32];
static mut MY_IDENTITY_SK: [u8; 64] = [0; 64];
static mut MY_IDENTITY_LOADED: bool = false;

/// Return our persistent identity, lazy-loading from BatFS (or
/// generating + persisting on first use). Returns the secret-key
/// blob (64 bytes per ed25519-compact layout) and the public key.
fn my_identity() -> Result<(SecretKey, [u8; 32]), &'static str> {
    unsafe {
        if core::ptr::read_volatile(core::ptr::addr_of!(MY_IDENTITY_LOADED)) {
            let sk_bytes = core::ptr::read_volatile(core::ptr::addr_of!(MY_IDENTITY_SK));
            let pk       = core::ptr::read_volatile(core::ptr::addr_of!(MY_IDENTITY_PK));
            let sk = SecretKey::from_slice(&sk_bytes)
                .map_err(|_| "cached identity sk is corrupt")?;
            return Ok((sk, pk));
        }

        // Try to load from BatFS first.
        // gap-audit 032: ns_* — each cave gets its own comms identity
        // (sys-wg's identity ≠ desktop's identity, even though the
        // file name is the same in the cave's view).
        let mut seed = [0u8; 32];
        let kp = match crate::fs::batfs::ns_read(IDENTITY_PATH, &mut seed) {
            Ok(32) => KeyPair::from_seed(Seed::new(seed)),
            _ => {
                // Generate + persist. Seed from RNDR (with fallback
                // inside rng::fill_bytes), then write the raw seed
                // to BatFS for future loads.
                crate::crypto::rng::fill_bytes(&mut seed);
                let kp = KeyPair::from_seed(Seed::new(seed));
                let _ = crate::fs::batfs::ns_create(IDENTITY_PATH, &seed);
                kp
            }
        };

        // Cache via raw pointer writes to avoid taking a &mut to the static.
        let sk_ptr = core::ptr::addr_of_mut!(MY_IDENTITY_SK) as *mut u8;
        for i in 0..64 {
            core::ptr::write_volatile(sk_ptr.add(i), kp.sk[i]);
        }
        let pk_ptr = core::ptr::addr_of_mut!(MY_IDENTITY_PK) as *mut u8;
        for i in 0..32 {
            core::ptr::write_volatile(pk_ptr.add(i), kp.pk[i]);
        }
        core::ptr::write_volatile(core::ptr::addr_of_mut!(MY_IDENTITY_LOADED), true);

        let pk_out = core::ptr::read_volatile(core::ptr::addr_of!(MY_IDENTITY_PK));
        Ok((kp.sk, pk_out))
    }
}

/// Hex-encode our identity pubkey for `comms my-id`. Returns false
/// if the identity can't be loaded.
pub fn my_id_hex(out: &mut [u8; 64]) -> bool {
    match my_identity() {
        Ok((_, pk)) => {
            let hex = b"0123456789abcdef";
            for i in 0..32 {
                out[i * 2]     = hex[(pk[i] >> 4) as usize];
                out[i * 2 + 1] = hex[(pk[i] & 0x0f) as usize];
            }
            true
        }
        Err(_) => false,
    }
}

// Compose buffer
static mut COMPOSE_BUF: [u8; MAX_MSG_LEN] = [0; MAX_MSG_LEN];
static mut COMPOSE_LEN: usize = 0;

/// Tracks whether a pin has been set this session. Connect refuses
/// to run when this is false — we never want to silently fall into
/// an unauthenticated session because the operator forgot to pin.
static mut PIN_SET: bool = false;

/// Store the expected server identity. Must be called before
/// `connect()` — without it the handshake can't verify who it's
/// talking to.
pub fn pin(server_id: &[u8; 32]) {
    unsafe {
        let dst = core::ptr::addr_of_mut!(PINNED_SERVER_ID) as *mut u8;
        for i in 0..32 {
            core::ptr::write_volatile(dst.add(i), server_id[i]);
        }
        PIN_SET = true;
    }
}

/// True if a server identity has been pinned in this session. Used
/// by the shell to decide between "connect" (needs pin) and "identify
/// then pin" (no pin yet).
pub fn pin_is_set() -> bool {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PIN_SET)) }
}

/// Hex-encode the pinned identity into a 64-byte ASCII buffer.
/// Returns None if no pin is set. Used by `comms pin show`.
pub fn pinned_hex(out: &mut [u8; 64]) -> Option<()> {
    if !pin_is_set() { return None; }
    let hex = b"0123456789abcdef";
    unsafe {
        let src = core::ptr::addr_of!(PINNED_SERVER_ID) as *const u8;
        for i in 0..32 {
            let b = core::ptr::read_volatile(src.add(i));
            out[i * 2]     = hex[(b >> 4) as usize];
            out[i * 2 + 1] = hex[(b & 0x0f) as usize];
        }
    }
    Some(())
}

/// Discovery — open a TCP session, exchange handshake offers, but
/// do NOT verify the server against any pin. Returns the server's
/// 32-byte identity pubkey. Caller is responsible for displaying it
/// to the operator and getting confirmation before pinning.
///
/// THIS IS NOT AUTHENTICATED. Anyone can sit between us and the
/// server and answer the offer with their own identity. The result
/// is only useful for the operator-in-the-loop TOFU step where the
/// user has out-of-band knowledge of what the server's pubkey
/// should look like.
pub fn identify(ip: u32, port: u16) -> Result<[u8; 32], &'static str> {
    crate::net::tcp::connect(ip, port)?;

    // Same offer as a real connect — server has no way to tell
    // discovery from connect, which is on purpose. We use the
    // persistent identity so the server's allowlist check sees the
    // same key during discovery and at real connect.
    let (id_sk, id_pk_bytes) = my_identity()?;

    let mut rng = crate::crypto::pq_hybrid::KernelRng;
    let eph_pk_bytes: [u8; 32] = {
        let eph_sk = EphemeralSecret::random_from_rng(&mut rng);
        // Scope-bound: discovery has no transport so the secret is
        // dropped here, not carried forward.
        X25519Public::from(&eph_sk).to_bytes()
    };

    let offer = build_offer(&id_sk, &id_pk_bytes, &eph_pk_bytes);
    crate::net::tcp::send_data(&offer)?;

    let mut srv_offer = [0u8; OFFER_LEN];
    recv_exact(&mut srv_offer)?;
    let mut srv_id = [0u8; 32];
    srv_id.copy_from_slice(&srv_offer[32..64]);

    // Close — discovery doesn't proceed to transport.
    crate::net::tcp::close();
    Ok(srv_id)
}

/// Run the handshake against the pinned server and bring the
/// session up. Caller must have called `pin()` first.
pub fn connect(ip: u32, port: u16) -> Result<(), &'static str> {
    if !pin_is_set() {
        return Err("no server identity pinned — run `comms identify` then `comms pin <hex>`");
    }
    let pinned: [u8; 32] = unsafe {
        let mut p = [0u8; 32];
        let src = core::ptr::addr_of!(PINNED_SERVER_ID) as *const u8;
        for i in 0..32 {
            p[i] = core::ptr::read_volatile(src.add(i));
        }
        p
    };

    // If we're already connected (or half-connected from a prior
    // attempt) the legacy TCP PCB still holds the previous session's
    // socket. tcp::connect would quietly reuse it and our fresh
    // handshake offer would land mid-AEAD-stream on the server,
    // which can't parse it -> server hangs up -> we recv-timeout.
    // Tear the previous session down first.
    if unsafe { STATE } != CommState::Disconnected {
        disconnect();
    } else {
        // Even Disconnected state may leak a stale PCB if the user
        // hit an Err mid-handshake on the previous try. Free-close
        // is idempotent on an unopened PCB.
        crate::net::tcp::close();
    }

    unsafe {
        STATE = CommState::Connecting;
        PEER_IP = ip;
        PEER_PORT = port;
        SEND_CTR = 0;
        RECV_CTR = 0;
    }

    if let Err(e) = crate::net::tcp::connect(ip, port) {
        unsafe { STATE = CommState::Error; }
        add_system_msg("TCP connect failed.");
        return Err(e);
    }

    // ── 1. Load our persistent identity + fresh ephemeral X25519 ──
    let (id_sk, id_pk_bytes) = match my_identity() {
        Ok(v) => v,
        Err(e) => {
            unsafe { STATE = CommState::Error; }
            crate::net::tcp::close();
            add_system_msg("Couldn't load comms identity.");
            return Err(e);
        }
    };

    let mut rng = crate::crypto::pq_hybrid::KernelRng;
    let eph_sk = EphemeralSecret::random_from_rng(&mut rng);
    let eph_pk_bytes: [u8; 32] = X25519Public::from(&eph_sk).to_bytes();

    // ── 2. Build + send our offer ─────────────────────────────────
    let offer = build_offer(&id_sk, &id_pk_bytes, &eph_pk_bytes);
    if let Err(e) = crate::net::tcp::send_data(&offer) {
        unsafe { STATE = CommState::Error; }
        crate::net::tcp::close();
        add_system_msg("Handshake send failed.");
        return Err(e);
    }

    // ── 3. Read + verify server's offer ───────────────────────────
    let mut srv_offer = [0u8; OFFER_LEN];
    if let Err(e) = recv_exact(&mut srv_offer) {
        unsafe { STATE = CommState::Error; }
        crate::net::tcp::close();
        add_system_msg("Handshake recv failed.");
        return Err(e);
    }
    let srv_eph_pk = match verify_offer(&srv_offer, &pinned) {
        Ok(eph) => eph,
        Err(e) => {
            unsafe { STATE = CommState::Error; }
            crate::net::tcp::close();
            add_system_msg("Server identity verify FAILED — possible MITM.");
            return Err(e);
        }
    };

    // ── 4. ECDH + key derivation ──────────────────────────────────
    let peer_eph = X25519Public::from(srv_eph_pk);
    let shared = eph_sk.diffie_hellman(&peer_eph);
    let (c2s, s2c) = derive_directional_keys(
        shared.as_bytes(),
        &eph_pk_bytes,
        &srv_eph_pk,
    );

    unsafe {
        C2S_KEY = c2s;
        S2C_KEY = s2c;
        STATE = CommState::Connected;
    }
    add_system_msg("Connected. ChaCha20-Poly1305 + Ed25519 pinned.");
    Ok(())
}

/// Send a message encrypted with c2s_key and counter nonce. Adds the
/// plaintext to the local timeline on success.
pub fn send_message(text: &[u8]) -> Result<(), &'static str> {
    if unsafe { STATE } != CommState::Connected { return Err("not connected"); }
    if text.len() > MAX_MSG_LEN { return Err("message too long"); }

    let nonce = unsafe { nonce_from_ctr(SEND_CTR) };
    let key  = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(C2S_KEY)) };

    let ct_tag = match cp::encrypt(&key, &nonce, &[], text) {
        Ok(v) => v,
        Err(_) => return Err("encrypt failed"),
    };

    // Frame: len(4 BE) || nonce(12) || ct||tag(N)
    let body_len = (cp::NONCE_LEN + ct_tag.len()) as u32;
    let mut frame = [0u8; 4 + cp::NONCE_LEN + MAX_MSG_LEN + cp::TAG_LEN];
    frame[0..4].copy_from_slice(&body_len.to_be_bytes());
    frame[4..4 + cp::NONCE_LEN].copy_from_slice(&nonce);
    frame[4 + cp::NONCE_LEN..4 + cp::NONCE_LEN + ct_tag.len()].copy_from_slice(&ct_tag);
    let total = 4 + cp::NONCE_LEN + ct_tag.len();
    if let Err(e) = crate::net::tcp::send_data(&frame[..total]) { return Err(e); }

    unsafe { SEND_CTR += 1; }
    add_msg(true, text);
    Ok(())
}

/// Receive one framed message: read 4-byte len, read body, verify
/// counter nonce, ChaCha20-Poly1305 decrypt with s2c_key, add to
/// timeline. Returns true if a message landed.
pub fn recv_message() -> bool {
    if unsafe { STATE } != CommState::Connected { return false; }

    let mut len_buf = [0u8; 4];
    if recv_exact(&mut len_buf).is_err() { return false; }
    let body_len = u32::from_be_bytes(len_buf) as usize;
    if body_len < cp::NONCE_LEN + cp::TAG_LEN
        || body_len > cp::NONCE_LEN + MAX_MSG_LEN + cp::TAG_LEN {
        add_system_msg("Recv: bad frame length.");
        return false;
    }

    let mut body = [0u8; 12 + MAX_MSG_LEN + 16];
    if recv_exact(&mut body[..body_len]).is_err() {
        add_system_msg("Recv: short body.");
        return false;
    }

    let nonce_bytes = &body[..cp::NONCE_LEN];
    let ct_tag = &body[cp::NONCE_LEN..body_len];

    let expected = unsafe { nonce_from_ctr(RECV_CTR) };
    if nonce_bytes != expected {
        add_system_msg("Recv: nonce/counter mismatch (replay or reorder).");
        return false;
    }

    let mut nonce_arr = [0u8; cp::NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let key = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(S2C_KEY)) };
    let pt = match cp::decrypt(&key, &nonce_arr, &[], ct_tag) {
        Ok(v) => v,
        Err(_) => {
            add_system_msg("Recv: AEAD tag verify FAILED (tampered).");
            return false;
        }
    };

    unsafe { RECV_CTR += 1; }
    add_msg(false, &pt);
    true
}

// ── handshake helpers ─────────────────────────────────────────────

/// Build a 128-byte offer signing (eph_pub || LABEL) with our
/// per-session Ed25519 identity.
fn build_offer(id_sk: &SecretKey, id_pk: &[u8; 32], eph_pk: &[u8; 32])
    -> [u8; OFFER_LEN]
{
    // 32 bytes eph_pk + LABEL ("SPHRAGIS-COMMS-v1" = 17 bytes).
    // Old buffer size 32 + 16 was stale and panicked at runtime when
    // LABEL grew past 16 chars.
    let mut msg = [0u8; 32 + 17];
    msg[..32].copy_from_slice(eph_pk);
    msg[32..32 + LABEL.len()].copy_from_slice(LABEL);
    let sig = id_sk.sign(&msg[..32 + LABEL.len()], None);

    let mut offer = [0u8; OFFER_LEN];
    offer[..32].copy_from_slice(eph_pk);
    offer[32..64].copy_from_slice(id_pk);
    offer[64..128].copy_from_slice(sig.as_slice());
    offer
}

/// Verify an incoming offer against the pinned identity. Returns the
/// peer's ephemeral X25519 public key on success.
fn verify_offer(offer: &[u8; OFFER_LEN], pinned_id: &[u8; 32])
    -> Result<[u8; 32], &'static str>
{
    let eph_bytes = &offer[..32];
    let id_bytes  = &offer[32..64];
    let sig_bytes = &offer[64..128];

    if id_bytes != &pinned_id[..] {
        return Err("server identity does not match pinned key");
    }

    let pk = PublicKey::from_slice(id_bytes).map_err(|_| "bad id pub")?;
    let sig = Signature::from_slice(sig_bytes).map_err(|_| "bad sig")?;
    let mut msg = [0u8; 32 + 17];
    msg[..32].copy_from_slice(eph_bytes);
    msg[32..32 + LABEL.len()].copy_from_slice(LABEL);
    pk.verify(&msg[..32 + LABEL.len()], &sig)
        .map_err(|_| "server sig verify failed")?;

    let mut out = [0u8; 32];
    out.copy_from_slice(eph_bytes);
    Ok(out)
}

/// Derive (c2s, s2c) directional keys. Mirrors the Python server's
/// derive_keys.
fn derive_directional_keys(shared: &[u8], client_eph_pk: &[u8; 32],
                            server_eph_pk: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // SHA-256(direction-label || shared || client_eph || server_eph).
    // KEY_DIR_C2S / KEY_DIR_S2C are both 21 bytes ("SPHRAGIS-COMMS-c2s-v1"
    // / "...-s2c-v1"); old 19-byte size was stale and panicked at runtime.
    let mut buf = [0u8; 21 + 32 + 32 + 32];

    buf[..KEY_DIR_C2S.len()].copy_from_slice(KEY_DIR_C2S);
    buf[KEY_DIR_C2S.len()..KEY_DIR_C2S.len() + 32].copy_from_slice(shared);
    buf[KEY_DIR_C2S.len() + 32..KEY_DIR_C2S.len() + 64].copy_from_slice(client_eph_pk);
    buf[KEY_DIR_C2S.len() + 64..KEY_DIR_C2S.len() + 96].copy_from_slice(server_eph_pk);
    let c2s = sha256::hash(&buf[..KEY_DIR_C2S.len() + 96]);

    buf[..KEY_DIR_S2C.len()].copy_from_slice(KEY_DIR_S2C);
    buf[KEY_DIR_S2C.len()..KEY_DIR_S2C.len() + 32].copy_from_slice(shared);
    buf[KEY_DIR_S2C.len() + 32..KEY_DIR_S2C.len() + 64].copy_from_slice(client_eph_pk);
    buf[KEY_DIR_S2C.len() + 64..KEY_DIR_S2C.len() + 96].copy_from_slice(server_eph_pk);
    let s2c = sha256::hash(&buf[..KEY_DIR_S2C.len() + 96]);

    (c2s, s2c)
}

/// 12-byte nonce: u64 counter big-endian + 4 zero bytes. Matches the
/// Python server's `make_nonce`.
unsafe fn nonce_from_ctr(ctr: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[..8].copy_from_slice(&ctr.to_be_bytes());
    n
}

/// Read exactly `buf.len()` bytes from the TCP connection. Loops over
/// recv_data because Sphragis's blocking recv returns whatever's
/// available, not a fixed length.
fn recv_exact(buf: &mut [u8]) -> Result<(), &'static str> {
    let mut off = 0;
    while off < buf.len() {
        let n = crate::net::tcp::recv_data(&mut buf[off..])?;
        if n == 0 {
            return Err("peer closed");
        }
        off += n;
    }
    Ok(())
}

fn add_msg(outgoing: bool, text: &[u8]) {
    unsafe {
        let idx = MSG_COUNT % MAX_MESSAGES;
        let ts: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) ts);
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);

        MESSAGES[idx] = ChatMsg {
            active: true,
            outgoing,
            text: {
                let mut t = [0u8; MAX_MSG_LEN];
                let len = text.len().min(MAX_MSG_LEN);
                t[..len].copy_from_slice(&text[..len]);
                t
            },
            text_len: text.len().min(MAX_MSG_LEN),
            timestamp: ts / freq,
        };
        MSG_COUNT += 1;
    }
}

fn add_system_msg(text: &str) {
    add_msg(false, text.as_bytes());
}

/// Disconnect from peer. The pinned identity is retained across
/// disconnects — a follow-up `connect` reuses the same pin without
/// the operator having to re-confirm.
pub fn disconnect() {
    crate::net::tcp::close();
    unsafe {
        STATE = CommState::Disconnected;
        C2S_KEY = [0; 32];
        S2C_KEY = [0; 32];
        SEND_CTR = 0;
        RECV_CTR = 0;
    }
    add_system_msg("Disconnected.");
}

/// V11-state-sweep: tear down the chat session on cave switch. Without
/// this, a new cave inherits the outgoing cave's AES session key, peer
/// tuple, AND the entire decrypted message history + compose buffer.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        STATE = CommState::Disconnected;
        C2S_KEY = [0; 32];
        S2C_KEY = [0; 32];
        SEND_CTR = 0;
        RECV_CTR = 0;
        PINNED_SERVER_ID = [0; 32];
        PIN_SET = false;
        // Wipe the cached identity — the new cave will lazy-load
        // its own from BatFS (or generate one) on first comms use.
        // Without this, the new cave would inherit the prior tenant's
        // identity, defeating cave isolation for comms.
        MY_IDENTITY_PK = [0; 32];
        MY_IDENTITY_SK = [0; 64];
        MY_IDENTITY_LOADED = false;
        PEER_IP = 0;
        PEER_PORT = 0;
        MSG_COUNT = 0;
        for m in (&mut *core::ptr::addr_of_mut!(MESSAGES)).iter_mut() {
            *m = ChatMsg::empty();
        }
        let cb = core::ptr::addr_of_mut!(COMPOSE_BUF) as *mut u8;
        for i in 0..MAX_MSG_LEN {
            core::ptr::write_volatile(cb.add(i), 0);
        }
        COMPOSE_LEN = 0;
    }
}

fn render(rect: crate::ui::wm::WindowRect) {
    use crate::ui::widgets::draw_strip;
    use crate::ui::palette as p;

    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, p::BG);

    let header_h: u32 = 32;
    let composer_h: u32 = 28;
    let body_y = rect.y + header_h;
    let composer_y = rect.y + rect.h - composer_h;
    let body_total_h = composer_y.saturating_sub(body_y);
    let shell_h = (body_total_h * 7 / 20).max(96);
    let shell_y = composer_y - shell_h - 1;
    let body_h = shell_y.saturating_sub(body_y);

    // ── HEADER STRIP ──────────────────────────────────────────────
    let _ = draw_strip(rect.x, rect.y, rect.w, header_h, false, true);
    let h_text_y = rect.y + (header_h - 16) / 2;
    font::draw_str(fb, sw, rect.x + 16, h_text_y, "COMMS", p::INK, p::BG);

    let st = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(STATE)) };
    let state_x = rect.x + 16 + 6 * 8;  // after "COMMS" + 8px gap
    let state_label = match st {
        CommState::Disconnected => alloc::string::String::from("DISCONNECTED"),
        CommState::Connecting   => format_state("CONNECTING"),
        CommState::Connected    => format_state("CONNECTED · peer"),
        CommState::Error        => alloc::string::String::from("ERROR"),
    };
    font::draw_str(fb, sw, state_x, h_text_y, state_label.as_str(), p::INK, p::BG);

    // Right side: cipher + key prefix as MID text (only when connected).
    if st == CommState::Connected {
        let cipher = "ChaCha20-Poly1305";
        let c2s = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(C2S_KEY)) };
        let hex = b"0123456789abcdef";
        let mut buf = [0u8; 48];
        let mut n = 0;
        for &b in cipher.as_bytes() { if n < buf.len() { buf[n] = b; n += 1; } }
        for &b in b" \xc2\xb7 K " { if n < buf.len() { buf[n] = b; n += 1; } }
        for i in 0..4 {
            if n < buf.len() { buf[n] = hex[(c2s[i] >> 4) as usize]; n += 1; }
            if n < buf.len() { buf[n] = hex[(c2s[i] & 0x0f) as usize]; n += 1; }
        }
        for &b in b"..." { if n < buf.len() { buf[n] = b; n += 1; } }
        let right = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
        let right_w = (n as u32) * 8;
        if rect.w > right_w + 16 {
            font::draw_str(fb, sw,
                rect.x + rect.w.saturating_sub(right_w + 16),
                h_text_y, right, p::MID, p::BG);
        }
    }

    // ── TIMELINE BODY ─────────────────────────────────────────────
    if st == CommState::Disconnected {
        draw_disconnected_empty(rect.x, body_y, rect.w, body_h, sw, fb);
    } else {
        draw_timeline(rect.x, body_y, rect.w, body_h, sw, fb);
    }

    // ── EMBEDDED SHELL STRIP ──────────────────────────────────────
    gpu::fill_rect(rect.x, shell_y, rect.w, 1, p::HAIRLINE);
    crate::ui::console::redraw_in_rect(crate::ui::wm::WindowRect {
        x: rect.x + 8, y: shell_y + 4,
        w: rect.w.saturating_sub(16), h: shell_h.saturating_sub(8),
    });

    // ── COMPOSER ──────────────────────────────────────────────────
    gpu::fill_rect(rect.x, composer_y, rect.w, composer_h, p::PANEL);
    gpu::fill_rect(rect.x, composer_y, rect.w, 1, p::HAIRLINE);
    let c_text_y = composer_y + (composer_h - 16) / 2;
    let prompt_color = if st == CommState::Disconnected { p::FAINT } else { p::INK };
    font::draw_str(fb, sw, rect.x + 16, c_text_y, ">", prompt_color, p::PANEL);
    let typed_x = rect.x + 16 + 2 * 8;

    let (compose_text, compose_len): (&str, usize) = unsafe {
        let len = core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN));
        let buf = &*core::ptr::addr_of!(COMPOSE_BUF);
        let bytes = &buf[..len];
        (core::str::from_utf8_unchecked(bytes), len)
    };
    if st == CommState::Disconnected {
        font::draw_str(fb, sw, typed_x, c_text_y,
            "(composer disabled - not connected)", p::FAINT, p::PANEL);
    } else {
        font::draw_str(fb, sw, typed_x, c_text_y, compose_text, p::INK, p::PANEL);
        // 1-cell INK block cursor at the next-char position.
        let cur_x = typed_x + (compose_len as u32) * 8;
        let cell_top = composer_y + (composer_h - 16) / 2;
        gpu::fill_rect(cur_x, cell_top, 8, 16, p::INK);
    }

    // Char counter on the right, always MID.
    let mut cbuf = [0u8; 16];
    let n = format_dec_local(compose_len, &mut cbuf);
    let n_str = unsafe { core::str::from_utf8_unchecked(&cbuf[..n]) };
    let suffix = " / 80";
    let total_w = (n as u32 + suffix.len() as u32) * 8;
    if rect.w > total_w + 16 {
        let cx = rect.x + rect.w - 16 - total_w;
        font::draw_str(fb, sw, cx, c_text_y, n_str, p::MID, p::PANEL);
        font::draw_str(fb, sw, cx + n as u32 * 8, c_text_y, suffix, p::FAINT, p::PANEL);
    }
}

/// Build `"<prefix> <ip>:<port>"` from PEER_IP / PEER_PORT statics.
fn format_state(prefix: &str) -> alloc::string::String {
    use alloc::format;
    let ip = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEER_IP)) };
    let port = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEER_PORT)) };
    if ip == 0 && port == 0 {
        alloc::string::String::from(prefix)
    } else {
        format!("{} {}.{}.{}.{}:{}",
            prefix,
            (ip >> 24) & 0xff,
            (ip >> 16) & 0xff,
            (ip >> 8) & 0xff,
            ip & 0xff,
            port,
        )
    }
}

fn draw_disconnected_empty(
    x: u32, y: u32, w: u32, h: u32,
    sw: u32, fb: *mut u32,
) {
    use crate::ui::palette as p;
    let text = "(no peer connected - use ";
    let cmd  = "comms connect <ip>:<port>";
    let tail = " in shell)";
    let total = (text.len() + cmd.len() + tail.len()) as u32 * 8;
    let cx = x + (w.saturating_sub(total)) / 2;
    let cy = y + h / 2 - 8;
    font::draw_str(fb, sw, cx, cy, text, p::MID, p::BG);
    font::draw_str(fb, sw, cx + text.len() as u32 * 8, cy, cmd, p::INK, p::BG);
    font::draw_str(fb, sw, cx + (text.len() + cmd.len()) as u32 * 8, cy, tail, p::MID, p::BG);
}

fn draw_timeline(
    x: u32, y: u32, _w: u32, h: u32,
    sw: u32, fb: *mut u32,
) {
    use crate::ui::palette as p;
    unsafe {
        let row_h: u32 = 18;
        let pad_l: u32 = 16;
        let max_rows = (h.saturating_sub(24) / row_h) as usize;
        let total = MSG_COUNT;
        let start = if total > max_rows { total - max_rows } else { 0 };
        let count = total - start;
        let baseline_y = y + h - 12 - (count as u32) * row_h;
        let mut row_y = baseline_y;
        for i in start..total {
            let idx = i % MAX_MESSAGES;
            let messages = &*core::ptr::addr_of!(MESSAGES);
            let msg = &messages[idx];
            if !msg.active { continue; }
            let mins = msg.timestamp / 60;
            let secs = msg.timestamp % 60;
            let mut ts_buf = [0u8; 7];
            ts_buf[0] = b'[';
            ts_buf[1] = b'0' + ((mins / 10) % 10) as u8;
            ts_buf[2] = b'0' + (mins % 10) as u8;
            ts_buf[3] = b':';
            ts_buf[4] = b'0' + ((secs / 10) % 10) as u8;
            ts_buf[5] = b'0' + (secs % 10) as u8;
            ts_buf[6] = b']';
            let ts_str = core::str::from_utf8_unchecked(&ts_buf);
            font::draw_str(fb, sw, x + pad_l, row_y, ts_str, p::MID, p::BG);
            let (arrow, sender) = if msg.outgoing {
                (">>", "you ")
            } else {
                ("<<", "peer")
            };
            font::draw_str(fb, sw, x + pad_l + 8 * 8, row_y, arrow, p::INK, p::BG);
            font::draw_str(fb, sw, x + pad_l + (8 + 4) * 8, row_y, sender, p::INK, p::BG);
            let text_x = x + pad_l + (8 + 4 + 7) * 8;
            let text = core::str::from_utf8_unchecked(&msg.text[..msg.text_len]);
            font::draw_str(fb, sw, text_x, row_y, text, p::INK, p::BG);
            row_y += row_h;
        }
    }
}

fn format_dec_local(mut n: usize, out: &mut [u8]) -> usize {
    if n == 0 { out[0] = b'0'; return 1; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 && i < tmp.len() { tmp[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    for j in 0..i { out[j] = tmp[i - 1 - j]; }
    i
}

/// Handle keyboard input for the compose bar.
pub fn handle_key(c: u8) -> crate::ui::apps_registry::AppEvent {
    use crate::ui::apps_registry::AppEvent;
    unsafe {
        match c {
            b'\r' | b'\n' => {
                if COMPOSE_LEN > 0 {
                    let compose_buf = &*core::ptr::addr_of!(COMPOSE_BUF);
                    let bytes = &compose_buf[..COMPOSE_LEN];
                    let _ = send_message(bytes);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), 0);
                }
                AppEvent::Repaint
            }
            0x08 | 0x7F => {
                if COMPOSE_LEN > 0 {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), COMPOSE_LEN - 1);
                }
                AppEvent::Repaint
            }
            0x20..=0x7E => {
                if COMPOSE_LEN < MAX_MSG_LEN - 1 {
                    let buf_ptr = core::ptr::addr_of_mut!(COMPOSE_BUF) as *mut u8;
                    core::ptr::write(buf_ptr.add(COMPOSE_LEN), c);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), COMPOSE_LEN + 1);
                }
                AppEvent::Repaint
            }
            _ => AppEvent::Unhandled,
        }
    }
}

pub fn handle_click(_mx: i32, _my: i32, _body: crate::ui::wm::WindowRect)
    -> crate::ui::apps_registry::AppEvent
{
    crate::ui::apps_registry::AppEvent::Consumed
}

pub fn paint(rect: crate::ui::wm::WindowRect) {
    render(rect);
}
