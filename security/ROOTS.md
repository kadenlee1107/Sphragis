# Sphragis — Root Cause Inventory (V8 synthesis)

Date: 2026-04-16
Source: 18 parallel V8 pentest agents, findings clustered by shared root.

## Meta-observation

Every prior round (V5→V6→V7) shipped point fixes that the next round broke or
reopened. The common cause is that each fix treated a **symptom** of a deeper
concurrency / state-management / information-exposure flaw, instead of the
flaw itself. This doc enumerates the ~12 real roots; each one, fixed
properly, retires a cluster of 5–20 prior findings.

Roots are ordered by **blast radius × ease of correct fix**.

---

## ROOT 1 — IRQ-unmasked multi-step state transitions  [CRIT, retires ~20 findings]

**Invariant violated**: "A multi-step kernel state transition must be atomic
w.r.t. preemption/IRQ, or another thread observes a mid-transition state."

V6 introduced deferred preemption (timer IRQ → `PREEMPT_REQUESTED` → syscall
entry `maybe_yield`). Every V5 multi-step sequence is now a preempt window.

**Affected sites** (from V8-IRQ-AUDIT, V8-TOCTOU, V8-XLAYER, V8-CHAINS, V8-KMEM):

- `cave::enter` (5 resets + activate) — chains-1, toctou-1, xlayer-D, irq-#1
- `cave::destroy` (fs_key zero + destroy_vfs order) — irq-#2, chains-1
- `execute_with_args` eret prologue (msr sp_el0 → eret window) — irq-#5
- `frame::alloc/free/alloc_kernel/free_contig` bitmap load/OR/store — kmem-1, toctou-1, irq-#7
- `threads::schedule` (two `with_table` flanks around cxt_switch) — irq-#10
- `threads::clone` slot publish — irq-#3
- `sealfs::init` nonce_prefix/counter publish — toctou-10, crypto-4
- `sys_pipe2`/`accept4`/`dup`/`dup3` charge→alloc→refund triples — irq-#4
- `tls::reset_all_sessions` loop — irq-#6
- `sockets::reset_for_cave_switch` / `tcp::reset_for_cave_switch` — irq-#8/9
- `exit_current` post-lock schedule+futex_wake — irq-#12
- `vfs::resolve_path_depth` symlink follow — irq-#11

**Correct fix**: add `src/kernel/sync/mod.rs` with `IrqGuard(u64)` RAII +
`critical_section! { ... }` macro. Apply to all 15 sites above.
Rule: any function touching two+ shared `static mut` must wrap in `IrqGuard`.

---

## ROOT 2 — Shared-global state not reset on cave switch  [CRIT, retires ~15]

**Invariant violated**: "Cave isolation implies every per-cave-observable
static is cleared before a new cave starts."

V6 added `reset_cave_statics` (covered syscall.rs + TLS_STATES). V6-XLAYER
added fd/sockets/tcp. Still missing:

- epoll INSTANCES (xlayer-A)
- futex TABLE (xlayer-A)
- async_fds eventfd/timerfd (xlayer-A)
- stdio_ring (xlayer-A)
- /sphragis/fb0 ChromiumFb pages (xlayer-B)
- VPN SEND/RECV_KEY+NONCE (xlayer-B)
- DNS EXPECTED_TXID/RESOLVED_IP (xlayer-B)
- PROC_FD_*, SOCK_DEST_IP/PORT/LOCAL_PORT, WORKER_BRK (static-audit-A)
- THREADS[128], CAVE_IPC[16] (static-audit-A)
- SealFS FILES/MERKLE_TREE/MASTER_KEY (global FS — per-cave namespace needed)

**Correct fix**: one `cave::reset_all_globals()` function in cave.rs fans
out to every subsystem's `reset_for_cave_switch()`. Every new `static mut`
MUST land a reset hook in this function. Make it a rule visible in
`kernel/sync/mod.rs`'s doc.

---

## ROOT 3 — Length/offset from external input used without bounds check  [CRIT, retires ~10]

**Invariant violated**: "Every arithmetic on attacker-controlled length
must use `checked_*` and validate against enclosing buffer."

Sites (V8-LENGTH-AUDIT, V8-PARSER, V8-ARITH):

- ELF `ph = phoff + i*phentsz` at 11 sites in loader.rs + elf.rs (the alt
  loader was never V5-hardened)
- ELF `dyn_offset` unchecked file offset (parser-1, kmem-3)
- `sys_execve path_ptr` ungated (parser-3, length-E) — only worker path was gated
- DNS name walk past data.len() on truncated packet
- TLS ext walk adds ext_len without re-check
- virtio RX trusts host `len` unconditionally
- `nanosleep tv_sec * freq` (arith-A1)
- `mmap len+4095 / 4096 * 4096` wrap (arith-A2)
- `writev total += iov_len as i64` sign confusion (arith-A5)
- `jpeg 1 << len` where len from input (arith-D1)

**Correct fix**: enable `overflow-checks = true` in Cargo.toml
`[profile.release]` + systematic `checked_mul`/`checked_add` with EINVAL
early-return. Backport loader.rs V5/V6 guards to elf.rs.

---

## ROOT 4 — Atomic ordering too weak / torn-state publication  [CRIT, retires ~8]

**Invariant violated**: "Atomic publishers must use Release / Acquire or
a single compound store; reader sees inconsistent field pairs otherwise."

- `ACTIVE_WIN_START/END` (V5 still has legacy pair alongside the V6
  packed-atomic fix — static-audit-B)
- `auth::ATTEMPT_COUNT` load+1+store non-atomic → **brute-force doubles
  under concurrent auth** (toctou-4)
- `crypto::rng::STATE_LO/STATE_HI` two Relaxed u64 stores → torn DRBG
  state → correlated keystreams (toctou-4, crypto-4)
- `sealfs::INITIALIZED` writer has SeqCst fence; reader is plain-bool
  (invariants-6, static-audit-B)
- `frame::BITMAP` load/OR/store no CAS (kmem-1, toctou-2)
- `ACTIVE_CAVE_ID` Relaxed both sides (invariants-1)

**Correct fix**: audit every atomic — if two fields must be seen together,
pack into one word or use a lock. Convert RMW patterns to CAS loops.
Never Relaxed for publish/consume of cross-thread-visible state.

---

## ROOT 5 — Secret deterministic from public kernel-image hash  [CRIT, retires 3]

**Invariant violated**: "Unlogged derived secrets are secret."

V6-WEIRD-002 replaced XOR-obfuscation with
`SHA256(kernel_text_hash || label)[0..8]` but:
- `print_kernel_hash` emits kernel_text_hash on UART every boot
- labels are public source constants
- charset + truncation leave ~40 bits
- derived strings are NEVER logged, so operator can't type them

Result: attacker computes both dev-fallback AND duress offline; operator
can't use either. (V8-WEIRD-ROOT-C, V8-CHAINS-ROOT-2, V7-SUPPLY-001,
V7-WEIRD-001)

**Correct fix**: do NOT derive from attacker-observable kernel state.
Require `SPHRAGIS_DEV_PASSPHRASE` / `SPHRAGIS_DURESS_CODE` env vars at build
via `build.rs` (currently empty) or refuse to build. Log neither over
UART. Document operator must record them at provisioning.

---

## ROOT 6 — Error/abort paths leak derived crypto material  [HIGH, retires ~8]

**Invariant violated**: "Any function that derives secrets must zero
them on every exit, including panic."

- `tls::handshake` has 15+ Err sites (tls.rs:673–837) that leave
  client_key/server_key/iv/shared_secret/our_private live in TLS_STATES
  (err-001)
- `process_server_hello` Err leaves peer_public + shared_secret (err-002)
- `recv_app_data` alert + GCM-fail only wipe 2 keys (err-003)
- `sealfs::read` leaves attacker-influenced ciphertext in caller buf on
  HMAC fail (err-004)
- `auth::authenticate` drops input_hash without volatile wipe (err-007)
- `panic_handler` does NOTHING — cold-boot capture gets all keys (err-008)

**Correct fix**: wrap `handshake` / `process_server_hello` in a
`on_error_zeroize!` scope that runs Drop-like wipe on any Err return.
Add `panic_handler` that zeros known secret statics before WFE.

---

## ROOT 7 — EL0-accessible high-resolution time source  [CRIT, retires ~6]

**Invariant violated**: "EL0 must not have access to a timer with
resolution sufficient for cache/branch timing attacks."

V5 set `CNTKCTL_EL1 = 0` (good). But:
- `sys_clock_gettime` returns ns regardless (side-5, weird-1)
- Back-to-back `getpid` RTT is itself a clock (side-chan-5)
- `cntpct_el0` indirectly readable via any syscall that uses it for
  seq ids, timeouts, etc.

Also: one `cntpct_el0` value simultaneously feeds deadman, RNG seed,
passphrase deadline, TCP state timeouts (weird-C) — hypervisor pause
disables four security features at once.

**Correct fix**: `sys_clock_gettime` quantizes to 1 µs and rate-limits per
cave. DRBG seeds independently from TRNG (RNDR mandatory, fall back to
virtio-rng, no cntpct-only path).

---

## ROOT 8 — Raw EL0 pointer derefs in modules that never got uaccess migration  [HIGH, retires ~6]

**Invariant violated**: "Every pointer from EL0 must pass
`uaccess::is_user_range` before deref."

Modules that PREDATE V4 uaccess and never migrated:
- `threads.rs` (NO uaccess import at all) — clone parent_tidptr (4-byte
  kernel write, ptr-001/002)
- `async_fds.rs` — timerfd_settime 16-byte write (ptr-003)
- `epoll.rs` — epoll_ctl 12-byte read+write (ptr-004)
- Legacy `sys_recvfrom` UDP sockaddr write path (ptr-005, pre-auth
  network-triggered)
- `recvmsg` re-reads iovec post-gate (ptr-006, TOCTOU — sendmsg is
  correct)

**Correct fix**: grep every module for `*mut T` / `*const T` /
`from_raw_parts` / `ptr::read/write/copy_nonoverlapping` / `asm ldr/str
[user_ptr]`. Require every site to go through `uaccess::is_user_range`.
Add a `#[must_not_deref]` marker or CI grep rule.

---

## ROOT 9 — C/C++ FFI without isolation  [CRIT, growing, retires 3]

**Invariant violated**: "Rust's memory safety does not extend through
`extern "C"` calls into unsandboxed C."

- `ports/libc.c:830` — `vsprintf(buf, (size_t)-1, ...)` unbounded write
- `ports/blink_printf.c:77` — format-string injection
- `ports/blink_bridge.cpp:84` — raw FFI ptr → unbounded malloc
- `ports/blink_*.cpp`, `libblink_full.a`, `freetype-2.13.3/`,
  `libpng-1.6.43/`, `zlib-1.3.1/`, `skia/`, `chromium/` — all growing

**Correct fix**: no C/C++ in the kernel address space. Chromium must run
in a sandboxed cave (the existing Linux-compat cave model works) — never
link C code into the kernel ELF.

---

## ROOT 10 — Parse-and-trust without re-validation  [HIGH, retires ~6]

**Invariant violated**: "A type/length descriptor from input does NOT
validate the payload — re-check independently."

- TLS record walks advance by declared TLV without asserting consumed ==
  declared (parser-5)
- initrd `crc_valid`/`sig_valid` computed but UNGATED when INITRD_PUBKEY
  is all-zero (parser-6)
- x509 extension criticality not enforced (parser silently accepts
  unrecognized-critical)
- DNS trusts compression-pointer type byte without counter
- ELF trusts phentsz

**Correct fix**: every parser ends with
`if pos != declared_end { return Err(...) }`. Initrd refuses to boot if
`INITRD_PUBKEY == [0;32]`.

---

## ROOT 11 — AES with lookup-table S-box in SealFS  [CRIT, retires 2]

TLS migrated to RustCrypto `aes` (bitsliced/HW). SealFS still uses the
hand-rolled `Aes256` with LUT → cache-timing key recovery on every
file read (ct-audit P0-1).

**Correct fix**: SealFS → RustCrypto `aes::Aes256` + `gcm_verified`
(replace hand-rolled Aes256, migrate from CTR-with-plaintext-hash to
GCM or XTS).

---

## ROOT 12 — X25519 field_reduce has branchy canonicalize  [CRIT, retires 1]

(ct-audit P0-2) `src/net/tls.rs:1515-1563` branches on top-word of
Curve25519 field element — leaks ECDH shared secret bits per handshake.

**Correct fix**: replace with constant-time conditional subtract mask,
OR swap whole X25519 impl to a vetted crate (`x25519-dalek` no_std).

---

## Dependencies between roots

```
ROOT 1 (IRQ) ── fixes enable correctness of ──> ROOT 2 (state reset),
                                               ROOT 4 (atomic ordering),
                                               ROOT 6 (err cleanup atomicity)
ROOT 3 (length bounds) ── independent ──> (parsers, arithmetic)
ROOT 5 (duress derivation) ── independent ──> need build.rs + operator flow
ROOT 7 (EL0 time) ── weakens ──> many side-channel ROOT-6 attacks
ROOT 8 (uaccess) ── independent ──> direct kernel write primitives
ROOT 9 (C FFI) ── architectural ──> move to cave sandbox
ROOT 10 (parser trust) ── independent ──> multi-protocol
ROOT 11 (SealFS AES LUT) ── independent ──> crate swap
ROOT 12 (X25519 field_reduce) ── independent ──> small patch or crate swap
```

## Fix order

1. **ROOT 1** (IrqGuard/critical_section!) — unblocks ROOTs 2/4/6 verification
2. **ROOT 3** (overflow-checks=true + length gates)
3. **ROOT 8** (uaccess migration) — direct CRIT primitives
4. **ROOT 4** (atomic ordering) — now that critical_section! exists
5. **ROOT 2** (reset_all_globals() + per-cave FS namespace) — builds on ROOT 1
6. **ROOT 6** (err-zeroize wrappers) — uses critical_section!
7. **ROOT 11** (SealFS → Aes256 RustCrypto)
8. **ROOT 12** (X25519 field_reduce)
9. **ROOT 10** (parser trust)
10. **ROOT 7** (EL0 time quantization + TRNG mandatory)
11. **ROOT 5** (build.rs env-var pipeline)
12. **ROOT 9** (C/C++ sandbox) — architectural, longest

## Protocol for each fix

1. Write the fix at the root (not the symptom).
2. Apply at every site in the root's "affected" list.
3. Re-dispatch a focused audit of JUST that root.
4. If findings come back against the same root → fix is wrong, redesign.
5. If findings come back against other roots → queue them (chain propagation).
6. Only after all roots close: full V9 re-audit of all 18 categories.
