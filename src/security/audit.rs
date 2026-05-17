// Sphragis — audit log (, Sprint 2.3).
//
// Append-only ring buffer for security-relevant events. Built so the
// renderer's hot paths (every fetch, every click, every script run)
// can call `record()` without touching disk. Operator dumps recent
// entries via the `audit` shell command, or flushes the whole buffer
// to BatFS as one encrypted blob with `audit-flush`.
//
// Format per entry:
// timestamp (u64 ticks from cntpct_el0)
// category (Category enum, 1 byte)
// message (up to MSG_LEN bytes of operator-readable detail)
//
// Sensitive content (form bodies, passphrases, key material) MUST NOT
// be passed in. The `record()` callers below redact body contents and
// pass only counts + URLs / DOM indices / box numbers. Treat the log
// as "what the user did," not "what the user said."

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::drivers::uart;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Category {
    Fetch       = 1,  // GET / POST against a URL
    Script      = 2,  // JS engine started/finished
    Click       = 3,  // user-initiated click (real or simulated)
    Nav         = 4,  // explicit URL navigation (e.g. <a href> followed)
    FormSubmit  = 5,  // form POST with N inputs
    Mode        = 6,  // tls-mode / js-mode flipped
    Auth        = 7,  // login / logout / failed attempt
    Boot        = 8,  // kernel boot, cave switch
    /// cave-table mutations
    /// (create / destroy / failed attempts). Distinct from `Boot`
    /// (one-shot per power-on) so the operator can grep-filter cave
    /// lifecycle events without drowning in boot noise.
    Cave        = 9,
    /// ai-agent activity: session start / tool call / session end.
    /// Per-session scope so a forensic reviewer can replay one
    /// conversation without untangling it from system noise.
    Ai          = 10,
    /// pipe lifecycle: create/close. Read/write are too high-frequency
    /// to log per-call — only the open and close ends are recorded so
    /// a reviewer can reconstruct which task owned which fd at which
    /// point without drowning the ring in byte-level traffic.
    Pipe        = 11,
    /// AF_UNIX socket lifecycle: bind/connect/accept/close. Same
    /// rate-limit philosophy as `Pipe` — byte-stream traffic is not
    /// logged, only the addressing events that a forensic reviewer
    /// needs to reconstruct who-talked-to-whom.
    Socket      = 12,
    /// POSIX shm lifecycle: create/open/close. Bulk reads/writes
    /// against the region's bytes are not logged for the same
    /// rate-limit reason.
    Shm         = 13,
    /// AUDIT-CAVE-M2 (2026-05-15): crypto subsystem events —
    /// primitive self-test result, key rotation, AEAD failure
    /// on persistent storage. Previously squashed into Boot or Mode.
    Crypto      = 14,
    /// AUDIT-CAVE-M2: network subsystem events — TLS handshake
    /// outcome, cert-pin mismatch, CRL/OCSP revocation hit,
    /// firewall decision. Previously logged ad-hoc as Fetch / Cave.
    Net         = 15,
    /// AUDIT-CAVE-M2: filesystem events — BatFS mount / wipe /
    /// integrity-verify result. Previously squashed into Cave.
    Fs          = 16,
    /// AUDIT-CAVE-M2: key-rotation events. Logged whenever a
    /// long-lived key is regenerated (master, session, etc.).
    KeyRotate   = 17,
    /// AUDIT-CAVE-M2: TPI-quorum operations — the operator-action
    /// approval channel for high-consequence ops like wipe.
    TpiOp       = 18,
    /// SP-AUD-003: NIAP GPOSPP FAU_GEN.1.1.b "authentication unlock
    /// events" — sessions opened, sessions closed, lockouts cleared.
    /// Distinct from `Auth` (which is the login-attempt event itself);
    /// `AuthSession` is the lifecycle of an authenticated session.
    AuthSession = 19,
    /// SP-AUD-003: NIAP GPOSPP FAU_GEN.1.1.b "privilege escalation"
    /// events — capability grant, role change, TPI-approved one-shot
    /// privileged operation. Distinct from `TpiOp` (which records the
    /// quorum-approval event) — `PrivEsc` records the *use* of the
    /// granted privilege.
    PrivEsc     = 20,
    /// SP-AUD-003: NIAP GPOSPP FAU_GEN.1.1.b "loadable software"
    /// events — kernel module load / unload, package install, package
    /// uninstall, signature-verify outcome on a loadable artifact.
    LoadableMod = 21,
    /// SP-AUD-003: NIAP GPOSPP FAU_GEN.1.1.b "trusted update" events —
    /// kernel image update applied, rollback, signature-verify outcome
    /// on a trusted-update artifact. Distinct from `LoadableMod`
    /// (per-module) — `UpdateApply` covers whole-system updates.
    UpdateApply = 22,
    /// SP-AUD-003: NIAP GPOSPP FAU_GEN.1.1.b "configurable file
    /// access" — file open/create/delete events where the operator has
    /// explicitly subscribed that path/inode to audit. Not every file
    /// access (too noisy); only subscribed paths. The default-deny
    /// state is "not audited"; operator opts-in via cave-policy.
    FileAccess  = 23,
    /// SP-C1.x: attestation surface events — quote produced, quote
    /// verified, attestation-key rotated, endorsement-chain failure.
    /// Audit-trail for the kernel-mediated attestation primitive
    /// (REQ-ATT-001) so an external verifier can cross-check
    /// quote-issuance frequency against the platform's claimed
    /// activity.
    Attest      = 24,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Fetch      => "fetch",
            Category::Script     => "script",
            Category::Click      => "click",
            Category::Nav        => "nav",
            Category::FormSubmit => "form",
            Category::Mode       => "mode",
            Category::Auth       => "auth",
            Category::Boot       => "boot",
            Category::Cave       => "cave",
            Category::Ai         => "ai",
            Category::Pipe       => "pipe",
            Category::Socket     => "sock",
            Category::Shm        => "shm",
            Category::Crypto    => "crypto",
            Category::Net       => "net",
            Category::Fs        => "fs",
            Category::KeyRotate => "keyrot",
            Category::TpiOp     => "tpi",
            Category::AuthSession => "session",
            Category::PrivEsc     => "privesc",
            Category::LoadableMod => "loadmod",
            Category::UpdateApply => "update",
            Category::FileAccess  => "filea",
            Category::Attest      => "attest",
        }
    }
}

pub const MSG_LEN: usize = 192;
pub const RING_CAP: usize = 1024;

#[derive(Clone, Copy)]
pub struct Entry {
    pub ts:   u64,
    pub cat:  u8,           // Category as raw u8 so we can const-init.
    pub mlen: u8,
    /// AUDIT-CAVE-M3 (2026-05-15): originating cave id. 0xFFFF =
    /// kernel context (boot, panic, scheduler). Populated by
    /// `record()` from `cave::get_active()`. Forensic reviewers
    /// can filter "everything done by Cave X" without doing
    /// substring search on the message text.
    pub cave_id: u16,
    pub msg:  [u8; MSG_LEN],
}

impl Entry {
    pub const fn empty() -> Self {
        Entry { ts: 0, cat: 0, mlen: 0, cave_id: 0xFFFF, msg: [0; MSG_LEN] }
    }
}

static mut RING: [Entry; RING_CAP] = [Entry::empty(); RING_CAP];
/// Monotonically-increasing event counter. `RING[head % RING_CAP]` is
/// the next slot to write. We never decrement so `count - RING_CAP`
/// gives the index of the oldest still-resident entry.
pub static HEAD: AtomicUsize = AtomicUsize::new(0);

/// Read-only view of the underlying ring storage. Used by the
/// audit_chain verifier to recompute hashes against the live data.
/// SAFETY: caller must not alias mutably with `record()`.
pub unsafe fn raw_ring() -> &'static [Entry; RING_CAP] {
    unsafe { &*core::ptr::addr_of!(RING) }
}

/// Test-only: flip one byte of an entry's `msg` field so the chain
/// verifier has something to detect. Used by `audit-chain-selftest`
/// to prove tamper detection actually fires. SAFETY: must not race
/// with a concurrent `record()` — the cooperative single-CPU model
/// makes this trivial for the selftest path.
#[allow(dead_code)]
pub unsafe fn tamper_test_flip_msg_byte(absolute_index: usize, msg_offset: usize) {
    let slot = absolute_index % RING_CAP;
    if msg_offset >= MSG_LEN { return; }
    unsafe {
        let p = core::ptr::addr_of_mut!(RING) as *mut Entry;
        let entry = &mut *p.add(slot);
        entry.msg[msg_offset] ^= 0xFF;
    }
}

/// Copy the most-recent `n` entries (capped at RING_CAP) into the
/// caller's buffer. Returns how many were actually written. The
/// AI agent's `query_audit_ring` tool dispatch calls this — it's
/// the one piece of audit-side state the model is allowed to read
/// at inference time.
///
/// SAFETY: single-writer assumption matches `record()`. The copy
/// is best-effort; a concurrent record() could overwrite an entry
/// we're in the middle of reading. The model gets a torn read at
/// worst; acceptable because the audit ring's job is to record
/// truths, not to be a synchronization primitive.
pub fn recent(buf: &mut [Entry]) -> usize {
    let head = HEAD.load(Ordering::Relaxed);
    if head == 0 || buf.is_empty() {
        return 0;
    }
    let resident = if head < RING_CAP { head } else { RING_CAP };
    let take = if buf.len() < resident { buf.len() } else { resident };
    let start = head - take;
    for i in 0..take {
        let slot = (start + i) % RING_CAP;
        // SAFETY: addr_of! avoids creating a &mut to the static.
        let entry = unsafe { (*core::ptr::addr_of!(RING))[slot] };
        buf[i] = entry;
    }
    take
}

/// Cave-scoped read (SP-ISO-009 / REQ-ISO-009 / REQ-AUD-006). Like
/// `recent` but filters to entries whose recorded `cave_id` matches
/// `cave_id_filter`. Use this when a non-privileged cave (one
/// without the `audit:read-all` capability) requests the audit ring
/// — it sees only entries from its own cave.
///
/// `cave_id_filter` semantics:
///   - 0xFFFF: kernel-context entries only
///   - 0..=0x7FFE: a specific cave's entries
///
/// Walks the same window as `recent` (last RING_CAP entries) but
/// only copies those whose cave_id matches. Returns the count of
/// MATCHING entries written into `buf`. Caller-side: a cave with
/// `audit:read-all` should use `recent` directly; one without
/// should call this with its own cave_id.
///
/// Privileged surface (callable by anything with mutable kernel
/// access) — Rust visibility doesn't enforce the capability check;
/// the caller's cave-policy gate is where that enforcement lives.
/// SP-ISO-009.1 (future) wires a cave-policy check into a
/// `recent_for_caller(buf)` wrapper that consults the active cave's
/// capability set.
pub fn recent_for_cave(cave_id_filter: u16, buf: &mut [Entry]) -> usize {
    let head = HEAD.load(Ordering::Relaxed);
    if head == 0 || buf.is_empty() {
        return 0;
    }
    let resident = if head < RING_CAP { head } else { RING_CAP };
    let start = head - resident;
    let mut written = 0usize;
    for i in 0..resident {
        if written >= buf.len() { break; }
        let slot = (start + i) % RING_CAP;
        let entry = unsafe { (*core::ptr::addr_of!(RING))[slot] };
        if entry.cave_id == cave_id_filter {
            buf[written] = entry;
            written += 1;
        }
    }
    written
}

/// the ring silently overwrites the oldest
/// entries when full. An adversary who suspects a forensic dump is
/// imminent can flood the log to evict their tracks. This counter
/// records how many entries have been EVICTED (not just rolled over)
/// so a post-incident reviewer sees `audit-flush` blob size + this
/// counter and can spot exfiltration. UART-warns the first time we
/// roll over so a live operator gets one chance to react.
static EVICTED: AtomicUsize = AtomicUsize::new(0);
static FIRST_OVERFLOW_WARNED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[inline]
fn now_ticks() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

/// Record an event. Truncates `msg` to MSG_LEN bytes. Cheap — single
/// store of an Entry into the ring + an atomic increment. Safe to call
/// from any kernel context.
// /
/// non-printable bytes in `msg` are
/// rewritten as `?` before storage. Pre-fix, an attacker who could
/// influence a logged URL/cookie name with embedded `\r` or `\x1B`
/// (terminal escape) could overwrite earlier log lines visually
/// when the operator ran `audit` — log-tampering by carriage-return.
pub fn record(cat: Category, msg: &[u8]) {
    let h = HEAD.fetch_add(1, Ordering::Relaxed);
    // detect first wrap-around — that's
    // the moment we stop being able to tell the operator about the
    // earliest events. Single one-time UART line so a live tail
    // sees it.
    if h >= RING_CAP {
        EVICTED.fetch_add(1, Ordering::Relaxed);
        if !FIRST_OVERFLOW_WARNED.swap(true, Ordering::AcqRel) {
            uart::puts("[audit] WARNING: ring full, oldest entries now being overwritten — run audit-flush to persist\n");
        }
    }
    let slot = h % RING_CAP;
    let copy = msg.len().min(MSG_LEN);
    // AUDIT-CAVE-M3 (2026-05-15): snapshot the active cave id at
    // record time so each entry carries provenance independent of
    // the message text. 0xFFFF = kernel context (cave::get_active
    // returns usize::MAX for unset, which we map to that sentinel).
    let cid: u16 = {
        let a = crate::caves::cave::get_active();
        if a == usize::MAX { 0xFFFF } else { (a as u16) & 0x7FFF }
    };
    unsafe {
        let e = &mut RING[slot];
        e.ts = now_ticks();
        e.cat = cat as u8;
        e.mlen = copy as u8;
        e.cave_id = cid;
        for i in 0..copy {
            let b = msg[i];
            // Allow printable ASCII + space; everything else → `?`.
            // Includes the bullet 0xB7 from by accident
            // (>0x7E) — that's fine, audit log doesn't need bullets.
            e.msg[i] = if b >= 0x20 && b < 0x7F { b } else { b'?' };
        }
        if copy < MSG_LEN { e.msg[copy] = 0; }

        // Tamper-evident chain (§3.7 of the gap audit). Hash this
        // entry against the prior chain link, store in
        // `audit_chain::CHAIN[slot]`. A later `verify_chain` walks
        // the ring from start..head and recomputes — any silent
        // edit to an entry's canonical bytes turns into a hash
        // mismatch at that entry's index.
        //
        // `append_chain` takes `head` = the absolute index of THIS
        // entry (the OLD count, before fetch_add returned). So we
        // pass `h`, not `h + 1`. `prev_slot = (h - 1) % RING_CAP`
        // inside append_chain then points at the previous entry's
        // chain hash for h > 0 (and falls back to the all-zero
        // genesis for h == 0).
        //
        // Single-writer assumption is the same one `audit::record`
        // already makes: HEAD is a fetch_add atomic + slot writes
        // are not concurrent across CPUs in our cooperative single-
        // CPU model. The `unsafe` here piggybacks on that.
        crate::security::audit_chain::append_chain(slot, &RING[slot], h);
    }
}

/// Dump the most-recent `n` entries to the UART (operator-visible).
/// Used by the `audit` shell command.
pub fn dump_tail(n: usize) {
    let total = HEAD.load(Ordering::Relaxed);
    if total == 0 { uart::puts("  audit: log is empty\n"); return; }
    let want = n.min(total).min(RING_CAP);
    let start = total - want;
    uart::puts("  audit: showing last ");
    crate::kernel::mm::print_num(want);
    uart::puts(" of ");
    crate::kernel::mm::print_num(total);
    uart::puts(" entries\n");
    for i in 0..want {
        let idx = (start + i) % RING_CAP;
        let e = unsafe { &RING[idx] };
        let cat = match e.cat {
            1 => "fetch",
            2 => "script",
            3 => "click",
            4 => "nav",
            5 => "form",
            6 => "mode",
            7 => "auth",
            8 => "boot",
            9 => "cave",
            _ => "?",
        };
        uart::puts("  [");
        crate::kernel::mm::print_num((start + i) as usize);
        uart::puts("] ");
        uart::puts(cat);
        uart::puts(": ");
        let msg = unsafe { core::str::from_utf8_unchecked(&e.msg[..e.mlen as usize]) };
        uart::puts(msg);
        uart::puts("\n");
    }
}

/// Total events recorded since boot.
pub fn count() -> usize { HEAD.load(Ordering::Relaxed) }

/// how many entries have been overwritten
/// (i.e. lost forever) since boot. Surfaces in the `audit` shell
/// command so a forensic reviewer knows the log was potentially
/// tampered with by flooding.
pub fn evicted() -> usize { EVICTED.load(Ordering::Relaxed) }

/// Zero every audit ring entry + reset HEAD/EVICTED counters.
/// Designed to be a TPI-gated privileged op (`audit-wipe` shell
/// command). Sole exposure of mutating the ring outside of
/// `record()`. Doesn't touch `audit_chain::CHAIN` — the chain
/// table is reset implicitly because all entries it covers are
/// gone, so a fresh `verify_chain` walks zero entries.
///
/// SAFETY: must not race with a concurrent `record()` or
/// `audit_chain::append_chain` — cooperative single-CPU makes
/// this trivial; the privileged caller holds the shell input
/// path and no other writer exists.
pub unsafe fn wipe_ring() {
    unsafe {
        let ring_ptr = core::ptr::addr_of_mut!(RING);
        for i in 0..RING_CAP {
            (*ring_ptr)[i] = Entry::empty();
        }
    }
    HEAD.store(0, Ordering::Relaxed);
    EVICTED.store(0, Ordering::Relaxed);
    FIRST_OVERFLOW_WARNED.store(false, Ordering::Relaxed);
    // Also reset the chain table so verify_chain over the empty
    // ring doesn't find dangling hashes.
    crate::security::audit_chain::reset_for_test();
}

/// Serialize the whole resident ring (oldest-first) into `out` as
/// newline-delimited records. Returns the number of bytes written.
/// Used by `audit-flush` to push the buffer into BatFS as one file.
pub fn serialize(out: &mut [u8]) -> usize {
    let total = HEAD.load(Ordering::Relaxed);
    if total == 0 { return 0; }
    let resident = total.min(RING_CAP);
    let start = total - resident;
    let mut pos = 0usize;
    for i in 0..resident {
        let idx = (start + i) % RING_CAP;
        let e = unsafe { &RING[idx] };
        let cat = match e.cat {
            1 => "fetch", 2 => "script", 3 => "click",
            4 => "nav",   5 => "form",   6 => "mode",
            7 => "auth",  8 => "boot",   9 => "cave",
            _ => "?",
        };
        // ts cat msg\n — caller decodes ts.
        pos += write_u64(&mut out[pos..], e.ts);
        if pos < out.len() { out[pos] = b' '; pos += 1; }
        pos += copy_to(&mut out[pos..], cat.as_bytes());
        if pos < out.len() { out[pos] = b' '; pos += 1; }
        pos += copy_to(&mut out[pos..], &e.msg[..e.mlen as usize]);
        if pos < out.len() { out[pos] = b'\n'; pos += 1; }
        if pos >= out.len() { break; }
    }
    pos
}

/// flush the resident audit ring to BatFS as `/audit.log`.
// /
/// Used by `cave::seal` and `cave::destroy` to lock the trail in
/// place at security-sensitive transitions — without this, an
/// attacker who panics/reboots between the seal/destroy and the
/// operator's next `audit-flush` erases evidence. With this, every
/// seal and destroy has its event trail durably committed.
// /
/// Returns `Ok(bytes_written)` or `Err(reason)`. Callers ignore the
/// result — failure here is "the trail isn't durable for this
/// transition," not a reason to abort the lifecycle event itself.
// /
/// Static 256K buffer to avoid stack-staging. Single-CPU + IrqGuard-
/// scoped callers means no concurrent flush races.
pub fn flush_to_batfs() -> Result<usize, &'static str> {
    static mut FLUSH_BUF: [u8; 256 * 1024] = [0; 256 * 1024];
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(FLUSH_BUF);
        let n = serialize(buf);
        if n == 0 { return Ok(0); }
        // Idempotent overwrite: delete-then-create. BatFS::create
        // errors on duplicate name; we want every flush to replace
        // the prior log (rotation policy is a future STUMP).
        let _ = crate::fs::batfs::delete("audit.log");
        crate::fs::batfs::create("audit.log", &buf[..n])?;
        Ok(n)
    }
}

/// restore previously-persisted audit entries from a
/// `serialize`-format buffer (typically the contents of `/audit.log`
/// in BatFS, written by a prior boot's `audit-flush`).
// /
/// Re-populates the RING with the parsed entries so the operator's
/// `audit` command shows historical events. Each restored event has
/// its `ts` re-set from the serialized timestamp; the `cat` byte
/// matches by string name; the message bytes are copied verbatim
/// up to MSG_LEN.
// /
/// Returns the number of entries successfully restored.
// /
/// Format (per `serialize` above): `<ts> <cat> <msg>\n` lines. Lines
/// that fail to parse are skipped — we'd rather drop a corrupt entry
/// than panic during boot.
pub fn restore_from_persisted(buf: &[u8]) -> usize {
    let mut restored = 0usize;
    let mut start = 0usize;
    for i in 0..buf.len() {
        if buf[i] != b'\n' { continue; }
        let line = &buf[start..i];
        start = i + 1;
        if line.is_empty() { continue; }

        // Split: <ts> <cat> <msg>
        let sp1 = match line.iter().position(|&b| b == b' ') { Some(p) => p, None => continue };
        let rest = &line[sp1 + 1..];
        let sp2 = match rest.iter().position(|&b| b == b' ') { Some(p) => p, None => continue };
        let ts_bytes = &line[..sp1];
        let cat_bytes = &rest[..sp2];
        let msg_bytes = &rest[sp2 + 1..];

        // Parse ts as decimal u64.
        let mut ts: u64 = 0;
        for &b in ts_bytes {
            if !(b'0'..=b'9').contains(&b) { ts = 0; break; }
            ts = ts.wrapping_mul(10).wrapping_add((b - b'0') as u64);
        }

        // Map cat name back to enum byte. The serialize side uses
        // these short names; keep both sides in sync.
        let cat = match cat_bytes {
            b"fetch"   => Category::Fetch as u8,
            b"script"  => Category::Script as u8,
            b"click"   => Category::Click as u8,
            b"nav"     => Category::Nav as u8,
            b"form"    => Category::FormSubmit as u8,
            b"mode"    => Category::Mode as u8,
            b"auth"    => Category::Auth as u8,
            b"boot"    => Category::Boot as u8,
            b"cave"    => Category::Cave as u8,
            b"ai"      => Category::Ai as u8,
            // SP-AUD-003: NIAP FAU_GEN.1 categories
            b"session" => Category::AuthSession as u8,
            b"privesc" => Category::PrivEsc as u8,
            b"loadmod" => Category::LoadableMod as u8,
            b"update"  => Category::UpdateApply as u8,
            b"filea"   => Category::FileAccess as u8,
            b"attest"  => Category::Attest as u8,
            _ => continue,
        };

        // Find a slot. We want restored entries to APPEND to the
        // existing ring so live events recorded post-boot don't
        // collide. Take the next slot via fetch_add — same path
        // `record` uses.
        let h = HEAD.fetch_add(1, Ordering::Relaxed);
        if h >= RING_CAP {
            EVICTED.fetch_add(1, Ordering::Relaxed);
        }
        let slot = h % RING_CAP;
        let copy = msg_bytes.len().min(MSG_LEN);
        unsafe {
            let e = &mut RING[slot];
            e.ts = ts;
            e.cat = cat;
            e.mlen = copy as u8;
            for j in 0..copy {
                let b = msg_bytes[j];
                e.msg[j] = if b >= 0x20 && b < 0x7F { b } else { b'?' };
            }
            if copy < MSG_LEN { e.msg[copy] = 0; }
        }
        restored += 1;
    }
    restored
}

fn copy_to(out: &mut [u8], src: &[u8]) -> usize {
    let n = src.len().min(out.len());
    out[..n].copy_from_slice(&src[..n]);
    n
}

fn write_u64(out: &mut [u8], mut v: u64) -> usize {
    if v == 0 {
        if !out.is_empty() { out[0] = b'0'; return 1; }
        return 0;
    }
    let mut buf = [0u8; 24];
    let mut i = 0;
    while v > 0 && i < buf.len() { buf[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    let len = i.min(out.len());
    for j in 0..len { out[j] = buf[i - 1 - j]; }
    len
}
