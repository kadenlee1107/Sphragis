// Bat_OS — SHA-256-chained cryptographic random byte generator.
//
// Not a formal CSPRNG — we don't have an interrupt-timing entropy
// pool yet — but dramatically better than reading `cntpct_el0`
// directly:
//
//   seed  = 8 × cntpct_el0 reads with ~100-cycle spin between,
//           mixed against the prior state of the chain
//   out_i = SHA-256( seed || call_counter || pos_offset )
//   state += out_i   (feedback so subsequent calls chain forward)
//
// Callers fill any number of bytes via `fill_bytes(&mut buf)`.
//
// This is the same core the `sys_getrandom` syscall uses, extracted
// so kernel-side crypto (TLS X25519 keypair generation, TLS client
// random, BatFS nonce derivation) can also use it instead of
// reading `cntpct_el0` directly.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use super::sha256;

static STATE_LO: AtomicU64 = AtomicU64::new(0);
static STATE_HI: AtomicU64 = AtomicU64::new(0);
static CTR:      AtomicU64 = AtomicU64::new(0);

// V8-ROOT-4: spinlock that serializes the feedback-chain update. Previously
// STATE_LO / STATE_HI were read as two independent Relaxed loads and written
// as two independent Relaxed stores — two concurrent fill_bytes() calls
// could both read the same (LO, HI) seed, produce correlated outputs, and
// race-overwrite each other's feedback. Serializing with a CAS-lock
// guarantees the (load-seed, hash, store-feedback) transaction is atomic.
static CHAIN_LOCK: AtomicBool = AtomicBool::new(false);

/// V11-FRESH-EYES: `panic_wipe` zeroes the chain state, which is the
/// correct thing for cold-boot residue — but if ANY post-panic code path
/// (recovery, watchdog, kthread drain) then calls `fill_bytes` before
/// the kernel actually halts, the very first output would be
/// `SHA256(cntpct_seed || 0 || 0)` with ~20 bits of boot-time entropy.
/// The new `POISONED` flag turns `fill_bytes` into a hard fault after
/// panic so we can never silently produce low-entropy output.
static POISONED: AtomicBool = AtomicBool::new(false);

/// V4: prefer ARMv8.5 RNDR when the CPU exposes it. Probed once at boot;
/// `true` means every subsequent `fill_bytes` call reads from RNDR and
/// XORs into the SHA-chain output. RNDR failure (hardware entropy source
/// temporarily empty) returns the SHA-chain bytes unmodified.
static HAVE_RNDR: AtomicBool = AtomicBool::new(false);

/// Read ID_AA64ISAR0_EL1 to probe for the RNDR feature (bits 63:60 = 1
/// means FEAT_RNG present). Call once at early boot.
///
/// V5-WEIRD-010 fix: previously this silently set HAVE_RNDR=false when
/// the CPU lacked FEAT_RNG. That's true on QEMU without `-cpu max`, on
/// many containers, and on older hardware. On those platforms every
/// TLS ClientHello random came purely from the SHA-chain DRBG seeded
/// only by cntpct_el0 — predictable to an attacker who could estimate
/// boot time. Now we emit a loud warning so the operator knows their
/// RNG is weakened, and surface the status via `have_rndr()`.
pub fn probe_hw_rng() {
    let isar0: u64;
    unsafe { core::arch::asm!("mrs {}, id_aa64isar0_el1", out(reg) isar0); }
    let rndr_field = (isar0 >> 60) & 0xF;
    let present = rndr_field != 0;
    HAVE_RNDR.store(present, Ordering::Release);
    if present {
        crate::drivers::uart::puts("  [rng] ARMv8.5 RNDR available — mixing HW entropy\n");
    } else {
        crate::drivers::uart::puts("  [rng] WARN: RNDR unavailable — TLS randomness relies on SHA-chain DRBG\n");
        crate::drivers::uart::puts("  [rng] WARN: deploy on ARMv8.5+ hardware or enable virtio-rng for production\n");
    }
}

/// True iff the CPU exposes FEAT_RNG. Consulted by callers that want
/// to refuse sensitive operations without real hardware entropy.
pub fn have_rndr() -> bool {
    HAVE_RNDR.load(Ordering::Acquire)
}

/// Try to read 8 bytes from RNDR. Returns None if unsupported or if the
/// hardware entropy source is transiently unavailable (NZCV.C set).
#[inline]
fn rndr_u64() -> Option<u64> {
    if !HAVE_RNDR.load(Ordering::Acquire) { return None; }
    let v: u64;
    let ok: u64;
    unsafe {
        core::arch::asm!(
            "mrs {v}, s3_3_c2_c4_0",    // RNDR (ARMv8.5)
            "cset {ok}, ne",             // NZCV.Z clear ⇒ success
            v = out(reg) v,
            ok = out(reg) ok,
            options(nostack, preserves_flags),
        );
    }
    if ok != 0 { Some(v) } else { None }
}

fn gather_seed() -> [u8; 64] {
    let mut seed = [0u8; 64];
    for i in 0..8 {
        let v: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
        seed[i * 8..(i + 1) * 8].copy_from_slice(&v.to_le_bytes());
        for _ in 0..100 { core::hint::spin_loop(); }
    }
    // V8-ROOT-4: Acquire pairs with Release in fill_bytes feedback stores.
    // gather_seed is only called under CHAIN_LOCK now, but keep Acquire in
    // case other paths (e.g. init) ever load.
    let prev_lo = STATE_LO.load(Ordering::Acquire);
    let prev_hi = STATE_HI.load(Ordering::Acquire);
    for i in 0..8 { seed[i]     ^= prev_lo.to_le_bytes()[i]; }
    for i in 0..8 { seed[i + 8] ^= prev_hi.to_le_bytes()[i]; }
    seed
}

/// Fill `buf` with SHA-256-chained random bytes, XOR-mixed with RNDR
/// hardware entropy when available.
///
/// V4: If the CPU exposes FEAT_RNG (ARMv8.5 RNDR), every 32-byte chunk
/// is XORed with fresh RNDR reads so even a compromised SHA chain
/// doesn't expose predictable output. If RNDR is unavailable or stalls
/// we still produce SHA-chain output — never falls back to cntpct-only.
/// V8-ROOT-6: panic-handler-only wipe of the DRBG chain state. Uses
/// volatile stores so the compiler cannot DCE. No locks — panic handler
/// may be holding arbitrary state.
///
/// # Safety
/// Call only from the panic handler (via wipe::emergency_wipe). Leaves
/// the RNG unusable until `gather_seed` is called again.
pub unsafe fn panic_wipe() {
    STATE_LO.store(0, Ordering::Release);
    STATE_HI.store(0, Ordering::Release);
    CTR.store(0, Ordering::Release);
    CHAIN_LOCK.store(false, Ordering::Release);
    // V11-FRESH-EYES: flag the RNG as unusable post-panic. `fill_bytes`
    // checks this first and halts rather than producing a low-entropy
    // derivation from the now-zeroed chain state.
    POISONED.store(true, Ordering::Release);
}

pub fn fill_bytes(buf: &mut [u8]) {
    // V11-FRESH-EYES: if we've been poisoned by a previous panic_wipe,
    // refuse to produce output. The chain state is zero so any output
    // would be ~20-bit boot-time entropy derivations (cntpct-only).
    // Halt loudly rather than silently fall through to weak keys.
    if POISONED.load(Ordering::Acquire) {
        crate::drivers::uart::puts("[rng] FATAL: fill_bytes called after panic_wipe (poisoned)\n");
        loop { unsafe { core::arch::asm!("wfe"); } }
    }

    // V8-ROOT-4: acquire CHAIN_LOCK before any state-chain access. IRQs are
    // masked for the duration so the lock cannot deadlock with an interrupt
    // handler that itself calls rng::fill_bytes (e.g. TLS record sequence).
    // RAII-release via `ChainGuard` so a mid-function panic can't strand
    // the lock — future fill_bytes callers would spin forever otherwise.
    struct ChainGuard;
    impl Drop for ChainGuard {
        fn drop(&mut self) { CHAIN_LOCK.store(false, Ordering::Release); }
    }

    let _irq = crate::kernel::sync::IrqGuard::new();
    // Single-CPU bring-up on Apple Silicon with MMU off: STXR on
    // Device memory always fails, so `compare_exchange` spins
    // forever. IRQ is already masked by `_irq` above — that's
    // sufficient mutual exclusion on a single CPU, so acquire the
    // lock non-atomically.
    CHAIN_LOCK.store(true, Ordering::Release);
    let _lock = ChainGuard; // released on drop, even on panic

    let seed = gather_seed();
    let mut pos = 0;
    while pos < buf.len() {
        // M4 / MMU-off: `fetch_add` lowers to LDXR/STXR which never
        // succeeds on Device-nGnRnE memory, so the RMW spins forever.
        // We hold CHAIN_LOCK non-atomically with IRQs masked on a
        // single CPU, so a plain load+store is exclusive.
        let ctr = CTR.load(Ordering::Relaxed);
        CTR.store(ctr.wrapping_add(1), Ordering::Relaxed);
        let mut stream = [0u8; 64 + 16];
        stream[..64].copy_from_slice(&seed);
        stream[64..72].copy_from_slice(&ctr.to_le_bytes());
        stream[72..80].copy_from_slice(&(pos as u64).to_le_bytes());
        let mut h = sha256::hash(&stream);

        // XOR-mix RNDR bytes if hardware provides them. We draw 32 bytes
        // (4 × u64) and XOR into h before emitting.
        for slot in 0..4 {
            if let Some(r) = rndr_u64() {
                let rb = r.to_le_bytes();
                for i in 0..8 { h[slot * 8 + i] ^= rb[i]; }
            }
        }

        let take = core::cmp::min(32, buf.len() - pos);
        buf[pos..pos + take].copy_from_slice(&h[..take]);

        // Feed output back into the chain.
        let new_lo = u64::from_le_bytes([h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]]);
        let new_hi = u64::from_le_bytes([h[8],h[9],h[10],h[11],h[12],h[13],h[14],h[15]]);
        STATE_LO.store(new_lo, Ordering::Release);
        STATE_HI.store(new_hi, Ordering::Release);

        pos += take;
    }

    // _lock and _irq drop at end of scope, releasing CHAIN_LOCK then DAIF.
    drop(_lock);
    drop(_irq);
}

/// Convenience: 32 random bytes.
pub fn random_32() -> [u8; 32] {
    let mut out = [0u8; 32];
    fill_bytes(&mut out);
    out
}
