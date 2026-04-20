# Session Journal

**Format.** Newest entries at top. Each entry: one Claude session.
Header: `## YYYY-MM-DD HH:MM — Mac|Ubuntu — summary line`.

The LAST entry is what you (the Claude waking up next) need to read.
Earlier entries are context — skim if they seem relevant to the task.

Both Mac Claude and Ubuntu Claude append here. Commit + push at the
end of a session.

---

## 2026-04-19 17:35 — Ubuntu — ARGB2101010 color fix + remaining LL/SC sites

**Two follow-on fixes landed in one commit, and one observation about
iBoot-watchdog stability that matters for future sessions.**

### 1. `dcp::boot_splash` — ARGB2101010 color fix (VERIFIED on camera)

Symptom from the previous session: the splash rendered with a
bright-red wash instead of black. Root cause: color constants were
authored as ARGB8888 (`0xFF00_0000` = opaque black) but written
directly into the M4 framebuffer, which is 30-bpp ARGB2101010 per
`M4_GROUND_TRUTH.md §3.1b`. In that packing, `0xFF00_0000` decodes
as A=3, R≈max, G=0, B=0 — **red**.

Fix: a new `pub const fn argb8888_to_m4(argb8888: u32) -> u32` in
`src/drivers/apple/dcp.rs` re-encodes at const-eval time by scaling
each 8-bit channel into 10 bits (top-2-bit replication so saturated
values stay saturated). `boot_splash`'s constants now run through
it. `fill_screen(BG)` and the inner `crate::ui::font::draw_str`
calls see native ARGB2101010 values.

**Verified on camera** at 17:18: the splash renders as black
background with amber `BAT_OS` title, cool-blue subtitle, dim-gray
footer — exactly as intended. Frames `/tmp/frames/f_{010,030,058}.png`
from video `/tmp/batos_selftest.mp4` (gitignored).

### 2. Remaining LL/SC-on-Device-memory RMW sites (mechanical)

Applied the same rewrite pattern used for `heap` / `CHAIN_LOCK` /
`CTR.fetch_add`:

- `kernel::mm::frame::alloc_frame` — `compare_exchange_weak` loop →
  plain load + check + store (already holds `IrqGuard`, single-CPU).
- `kernel::mm::frame::alloc_kernel_frame` — `compare_exchange` → load
  + store.
- `kernel::mm::frame::alloc_contig` — the `fetch_or` (per-bit claim)
  and `fetch_and` (rollback) loops → load + store.
- `fs::batfs::next_nonce` — `NONCE_COUNTER.fetch_add` → load + store
  under a fresh `IrqGuard` (callers don't hold one).

These are the last atomic RMWs on any plausible Bat_OS boot path. A
future `batfs::create` / `frame::alloc_frame` call now won't hang.

### 3. Mac iBoot-watchdog degrades with repeated chainloads

**Unverified caveat on the LL/SC fixes.** After 5–6 chainload cycles
in this session the Mac entered a state where Bat_OS consistently
hard-resets within ~2 s of jumping to `_apple_start`. Camera frames
show the Apple-logo ROM splash across the full video; `ttyACM1/2`
(m1n1 USB CDC) vanishes immediately post-reload and the Mac loops
through ROM → iBoot → m1n1 without ever staying in Bat_OS long enough
to render the fixed splash again.

We confirmed this is **not** a regression from the frame/batfs/main
changes: reverting those and rechaining the known-good ARGB-only
binary still exhibited the 2-second reset. The Mac needs a cold power
cycle (hold power → Options → reboot-to-macOS, or disconnect+hold
power → back into m1n1) to reset the state before next verification.

The frame + batfs rewrites are committed on the strength of the
pattern (three prior applications verified: `heap`, `CHAIN_LOCK`,
`CTR.fetch_add`) and code review. Next session should cold-boot the
Mac and confirm the splash still renders, then exercise the
now-unlocked paths (`frame::alloc_frame`, `batfs::create/read`) via
a small self-test.

### Open follow-ups

- Verify LL/SC fixes on a freshly-booted Mac (camera capture of
  black splash with amber `BAT_OS`).
- Add the post-splash kernel self-test (scaffolding written and
  reverted this session — see `apple_kernel_self_test` from commit
  history if re-adding).
- `ui::desktop::run()` on M4 is a no-op: it drives virtio-gpu via
  `drivers::virtio::gpu::*` which isn't wired up on Apple Silicon,
  and uses `drivers::uart::getc` (PL011) instead of
  `drivers::apple::uart::getc`. Either add a platform dispatch in
  `wm` / `console` / `ui::desktop::run`, or write an
  Apple-native `desktop_apple::run` that targets `dcp::` + the
  dockchannel UART.
- Dockchannel-UART TX/RX already works from `drivers::apple::uart`
  at the MMIO level — but we have no USB CDC on the Mac post-m1n1,
  so Ubuntu can't read/write it until Bat_OS implements its own USB
  CDC class driver (non-trivial).

**Files touched:** `src/drivers/apple/dcp.rs`,
`src/fs/batfs.rs`, `src/kernel/mm/frame.rs`,
`docs/M4_GROUND_TRUTH.md`, `docs/SESSION_JOURNAL.md`.

---

## 2026-04-19 17:05 — Ubuntu — batfs::init returns (CTR.fetch_add LL/SC fix)

**Resolved the "batfs::init enters but never returns" hang.** The
failure was the third instance of the same M4 LL/SC-on-Device-memory
pattern we already fixed in `LockedHeap` and `CHAIN_LOCK`:

- `crypto::rng::fill_bytes` (called from `fs::batfs::init` to seed
  `BOOT_NONCE_PREFIX`) contains the loop
  ```rust
  while pos < buf.len() {
      let ctr = CTR.fetch_add(1, Ordering::Relaxed);
      ...
  }
  ```
  `AtomicU64::fetch_add` on `aarch64-unknown-none` (no `+lse`) lowers
  to an LDXR/STXR loop. With MMU off after m1n1 handoff, all memory
  is Device-nGnRnE and STXR silently fails forever — so the RMW never
  completes and `fill_bytes` wedges on its first iteration.

**Fix.** `fill_bytes` is already inside an `IrqGuard` holding
`CHAIN_LOCK` non-atomically. On a single-CPU bring-up with IRQs
masked, plain load-then-store is exclusive. Replaced:

```rust
let ctr = CTR.fetch_add(1, Ordering::Relaxed);
// ->
let ctr = CTR.load(Ordering::Relaxed);
CTR.store(ctr.wrapping_add(1), Ordering::Relaxed);
```

`STATE_LO.store(..)` / `STATE_HI.store(..)` further down in the same
loop already use `Ordering::Release` which lowers to STLR (not an
exclusive) and works fine on Device memory; they didn't need
changing.

**Verification.** Camera capture during chainload shows the M4
display rendering `dcp::boot_splash()` — the amber "BAT OS" banner
on its (unfortunately) bright-red `fill_screen(0xFF00_0000)`
background. `boot_splash()` is **downstream** of `batfs::init`:
```
batfs::init(...)  →  dcp::init_simple_fb()  →  dcp::boot_splash()
```
So seeing the splash means batfs::init returned and control advanced
past `dcp::init_simple_fb()` into the real splash renderer. First
time we've gotten past that wall. Video captured to
`/tmp/batos_run.mp4` (gitignored); sample frames in `/tmp/frames/`.

**What's still broken (queued for next session):**

- **ARGB2101010 color mismatch in `dcp::boot_splash`.** Constants are
  authored as ARGB8888 (e.g. `0xFF00_0000` = "opaque black"), but the
  M4 framebuffer is ARGB2101010 per `docs/M4_GROUND_TRUTH.md §3.1b`.
  In that encoding, `0xFF00_0000` decodes to A=3, R=0x3F0 (~max), G=0,
  B=0 — **bright red**, not black. The splash renders a red wash with
  an amber title. Functional but ugly. Fix: port all color literals
  in `src/drivers/apple/dcp.rs` (+ `ui::desktop` once we get there)
  to ARGB2101010.
- **No visible `ui::desktop::run()` output.** Video shows the splash
  persisting unchanged for 30+ seconds — so either `desktop::run()`
  hangs, or it renders using the same ARGB8888 constants and paints
  everything in shades of red/black that look like "nothing changed".
  Next bisection target after the color fix.
- **Any `AtomicX::fetch_*` path still live elsewhere hangs.**
  Remaining instances surveyed: `NONCE_COUNTER.fetch_add` in
  `batfs::next_nonce` (first `batfs::create()` hangs),
  `BITMAP[wi].fetch_and/fetch_or` in `kernel::mm::frame` (first
  `frame::free_frame` hangs), `BITMAP[wi].compare_exchange_weak` in
  `frame::alloc_frame` (first `frame::alloc_frame` hangs). None are
  on the current boot path; will need the same load+store rewrite
  when those paths are exercised.

**Files touched:** `src/crypto/rng.rs` (the 5-line fix).

**Next-Claude starting point:** fix the ARGB2101010 color constants
in `dcp::boot_splash` / `fill_screen` so the splash renders black
background + amber title as intended, then investigate why
`ui::desktop::run()` doesn't advance past the splash.

---

## 2026-04-19 11:01 — Ubuntu — Session end: live, animated boot screen

**Iterated past the static splash into a full animated boot screen.**
The Mac's internal display now shows, rendered entirely by our Rust
+ 8x16 font + direct-FB pipeline:

```
        ____________.   (ASCII bat silhouette, 4x scale, amber)
       /__.--.  .--.__\
          \/    \/

                  BAT_OS                    (8x scale, amber)

     Bare Metal // Apple Silicon (M4 / T8132)
              [booted via m1n1 chainload]

              Chip       : T8132 (Donan / H16G)
              Model      : Mac16,1
              CPU        : Apple M4  4P + 6E
              RAM        : 15759 MiB
              Revision   : 3
              ADT peripherals discovered: 0

  [ok] m1n1 handoff accepted  (boot_args rev 3)
  [ok] _apple_start  asm stages 1..5 complete
  [ok] bringup_vectors installed at VBAR_EL1/EL2
  [ok] boot_args::parse  OK  (devtree virt->phys)
  [ok] discover_from_adt  walker bounded, 9 paths
  [ok] kernel::process + scheduler + ipc  init
  [ok] kernel::arch::init_exceptions
  [ok] drivers::apple::aic::init
  [ok] splash rendered  —  awaiting  mm::init fix

                  uptime: 00:29              (live, updates)
                  tick: 4497                 (live, counts up)
```

**The uptime is actual wall-clock accurate** — read via
`CNTPCT_EL0` / `CNTFRQ_EL0` = 24 MHz Apple Silicon Generic Timer.
Verified by camera sync: 20 s of wall-clock between frames
matches 00:09 → 00:29 on-screen.

**12 commits this session, `a37af844` → `bab72f6a`.** The single
biggest root cause nailed was the BSS-zero bug in `boot.s` using
link-time symbols instead of PC-relative — once fixed everything
else fell into place fast.

**What still doesn't work (queued for next session):**

- `heap::init` on M4 hangs somewhere inside
  `linked_list_allocator::LockedHeap::lock()`. Theory: `spin::Mutex`
  uses LDXR/STXR which may require MMU-enabled Inner-Shareable
  memory attributes; with MMU off everything is Device-nGnRnE and
  exclusive monitors silently fail. Fix options: (a) bring up the
  MMU first with an identity map and proper attrs; (b) replace
  `LockedHeap` with a non-atomic bump allocator for early boot;
  (c) disable the mutex via `unsafe` + `&mut Heap`. Option (b) is
  the cleanest.
- `discover_from_adt` returns 0 for peripherals — all 9 paths under
  `/arm-io/...` fail to resolve on this run. `uart0`, `aic`, `disp0`
  etc. should exist on M4; the walker is bounded now so it doesn't
  hang, it just doesn't find them. Might be a sibling-enumeration
  bug surfaced by the bounded walker; needs inspection.
- Dockchannel UART driver still not written. `uart::puts` is a
  no-op; we have no out-of-band logging channel to Ubuntu.
- `dcp::init_simple_fb` + `boot_splash` never got to run via their
  real code paths — we inline-render instead.

**Files touched this full session:**
- `.cargo/config.toml`  (build-std + alloc)
- `.gitignore`  (exclude harness artifacts)
- `docs/M4_GROUND_TRUTH.md`  (ARGB2101010, MPIDR, devtree handoff)
- `docs/SESSION_JOURNAL.md`  (this file)
- `scripts/fix-udev.sh`  (NEW)
- `scripts/install-sudoers.sh`  (NEW)
- `src/arch/aarch64/apple/boot.s`  (BSS-zero PC-relative, stage paints)
- `src/drivers/apple/adt.rs`  (bounded `total_size`)
- `src/drivers/apple/boot_args.rs`  (devtree virt→phys, `top_of_kernel_data`)
- `src/drivers/apple/soc.rs`  (renamed M4 paths + positional stripes)
- `src/drivers/apple/uart.rs`  (`UART_READY` gate)
- `src/main.rs`  (bringup_vectors + full splash/log/uptime pipeline)
- `src/ui/font.rs`  (`draw_str_scaled` + `draw_char_scaled`)

**Next-Claude starting point:** fix heap (option (b) bump allocator
is fastest), then re-enable `bring_up_all` / `dcp::boot_splash` /
eventually `ui::desktop::run`. After that, port dockchannel UART
and we have true remote serial visibility.

---

## 2026-04-19 10:18 — Ubuntu — **BAT_OS SPLASH VISIBLE ON M4 DISPLAY** 🦇

**We reached the "see Bat_OS" milestone this session.** The Mac's
internal screen now shows:

- Solid black background (painted by our own Rust code)
- `"BAT_OS"` centered in amber
- `"Bare Metal // Apple Silicon (M4 / T8132)"` subtitle in cyan
- `"[booted via m1n1 chainload]"` footer in dim gray

Camera capture at
`captures/AI100.png` / `AI140.png` (not committed — gitignored) is
the evidence.

**Path we took after the BSS-zero breakthrough:**

1. `uart::init()` + `uart::puts()` / `putc()` now early-return if
   `UART_READY == false` — gates the S5L driver until we port
   dockchannel. Keeps the hundreds of `uart::puts(...)` call sites
   compiling unchanged.
2. Skipped `kernel::mm::init()` (heap not yet wired up for M4;
   faults on first static access inside it).
3. `process::init`, `scheduler::init`, `ipc::init`,
   `arch::init_exceptions`, `aic::init` all completed cleanly.
   Each got a distinct FB-color checkpoint (K2..K7). No faults.
4. Skipped `bring_up_all`, `read_passphrase_apple`,
   `derive_batfs_key`, `fs::batfs::init` — all need heap.
5. `dcp::init_simple_fb()` on its own is safe (no MMIO, just sets
   `INITIALIZED = true` after checking `soc::fb_*` are non-zero).
   But `boot_splash()` early-returns because `dcp::is_ready()`
   reads the same flag — either wasn't set, or paint helpers'
   checks kept bouncing. Side-stepped entirely.
6. Inlined a minimal splash directly into `kernel_main_apple`:
   `fb_mark` full-FB black, then `ui::font::draw_str` at three
   positions for title / subtitle / footer, using the known-good
   FB base `0x103e0050000` and stride `0x2f40 / 4`. Bypasses
   `dcp::*` entirely — just raw FB + font rasterizer.

**Current state of `src/main.rs::kernel_main_apple`:**

- Full prologue (asm stages, R1-R5, args parse, ADT walk, 7-stage
  kernel init markers K1-K7) reliably runs.
- At K8 it paints black + draws splash + halts at `wfe`.
- M4 display shows the splash until iBoot watchdog resets the Mac
  ~1-2 minutes later.

**What's missing before this is a "real" boot:**

- Proper heap for `mm::init` on M4 — linked_list_allocator needs a
  backing region we can dedicate to kernel heap. Probably just a
  reserved chunk after `__bss_end` in the linker script; but the
  key gotcha is making `mm::init` use the PC-relative resolved
  addresses, not link-time.
- Port the dockchannel UART driver to replace S5L. Only then can
  `uart::puts(...)` deliver text over USB-CDC back to Ubuntu.
- `boot_splash()` / `desktop::run()` full wire-up once heap works.

**But the headline:** Bat_OS owns the M4 screen, renders its own
text in our own 8x16 font, using exclusively code we wrote — no
macOS, no Asahi, no m1n1 splash. That's the first time this has
been demonstrated on an M4 in this new chainload-only bring-up
flow.

**Files touched this sub-session:**
- `src/main.rs`: big rewrite of kernel_main_apple tail — K1..K8
  stage markers, skip-mm/passphrase/batfs, inline splash render.
- `src/drivers/apple/uart.rs`: `UART_READY` gate on
  `init`/`putc`/`puts`.

---

## 2026-04-19 10:03 — Ubuntu — BREAKTHROUGH: BSS-zero bug fixed, R5 reproducible

**This is the biggest single commit of the M4 bring-up so far.** The
"intermittent static-write fault" we've been chasing for hours was a
single bug in `src/arch/aarch64/apple/boot.s`:

```asm
// OLD — broken under m1n1 chainload:
ldr  x1, =__bss_start       // loads link-time absolute (0x81xxxxxxx)
ldr  x2, =__bss_end         // loads link-time absolute (0x81xxxxxxx)
```

`ldr =label` emits the linker's absolute value through the literal
pool. Under chainload m1n1 relocates the binary to somewhere in
`0x1000xxxxxxx` — so the BSS-zero loop was writing zeros to
unmapped/arbitrary physical memory (at 0x81xxxxxxx) while our
**actual** BSS (containing every `AtomicU8`, `AtomicPtr`,
`AtomicUsize` in the kernel) remained whatever random bytes m1n1
had left there. The first Rust static write — `platform::set_platform`
doing `CURRENT_PLATFORM.store(1)` — hit that tainted memory and
faulted.

**Fix.** Rewrite boot.s BSS zero AND stack setup to use PC-relative
addressing:

```asm
adrp  x1, __bss_start
add   x1, x1, #:lo12:__bss_start
adrp  x2, __bss_end
add   x2, x2, #:lo12:__bss_end
```

`adrp` resolves relative to the **loaded** PC, so it produces the
actual-runtime BSS addresses. Same change applied to `__stack_start`.

**Result.** Bat_OS now reproducibly runs end-to-end through every
Rust checkpoint — `set_platform`, `boot_args::parse`, `stash`,
`args.video()`, `set_fb_info`, `set_mem_info`, `args.adt()`, the
full 9-entry `discover_from_adt` (with positional stripes), `R5
hot-pink` halt — with NO fault stripe and no Mac reset during the
observable window. The entire Rust kernel-setup prologue through
`discover_from_adt` is now reliable bring-up infrastructure.

**What this unblocks.** Everything downstream of `discover_from_adt`
is now testable one checkpoint at a time:
- `uart::init` (dockchannel driver)
- `kernel::mm::init`, `kernel::process::init`, etc.
- `kernel::arch::init_exceptions` (replaces our bringup_vectors
  with the real Rust-handler ones)
- `drivers::apple::aic::init`, `bring_up_all`, `dcp::init_simple_fb`
- The boot splash + desktop

Each of those will likely need its own M4-specific tuning but now
they run against a solid foundation instead of a tainted-BSS
foundation.

**Files touched:**
- `src/arch/aarch64/apple/boot.s`: PC-relative `adrp + :lo12:` for
  `__bss_start`, `__bss_end`, `__stack_start`.
- `src/main.rs`: reverted the `set_platform` bypass; R2 dark-orange
  checkpoint reinstated. VBAR install already using adrp.

---

## 2026-04-19 10:00 — Ubuntu — Positional stripes + adrp VBAR + static-write fault

**More infra landed, one new root cause localized (not yet fixed).**

**1. Positional-stripe discovery markers.** Added a `crate::fb_stripe(y,
h, pixel)` helper that paints a horizontal band rather than the full
framebuffer. `discover_from_adt` now uses it: path `idx` paints a
100-pixel stripe at Y = `idx * 100`, then attempts its lookup. Earlier
stripes aren't overwritten, so the final camera frame is a visual
"progress bar" of which paths we started. Unambiguous position-based
decoding, no reliance on camera hue fidelity.

**2. adrp-based VBAR install.** The previous `adr x0, bringup_vectors`
in `kernel_main_apple` could have been silently wrapping — `adr` is
only ±1 MiB and the vectors live in `.text.apple_boot` near the top
of the 15 MiB binary while the function sits deeper. Replaced with
`adrp + add :lo12:` which is ±4 GiB and unconditionally correct.

**3. `platform::set_platform` faults on M4 — static-write issue.**
Halting immediately after R1 orange paint = clean halt, no fault
stripe. Halting immediately after skipping `set_platform` and painting
R2 yellow-green = clean halt, no fault stripe. Running past R1 with
`set_platform` CALLED = fault stripe on top of whatever checkpoint
painted last.

`set_platform` is nothing but `CURRENT_PLATFORM.store(1, Relaxed)`
against a static `AtomicU8`. The fault fires on the `strb` that backs
it. Most likely cause: BSS zeroing in `boot.s` uses the link-script
symbols `__bss_start`/`__bss_end` which are LINK-TIME absolute
addresses (around `0x810???????`), but m1n1 relocates our kernel to
a physical address around `0x1000xxxxxxx`. So the BSS-zero loop is
writing zeros to unrelated phys memory while our real BSS
(containing `CURRENT_PLATFORM`) is at a different address. When Rust
later accesses `CURRENT_PLATFORM` through its PC-relative `adrp + add`,
it IS hitting the loaded-binary location correctly — so the store
itself should be to valid RAM. But something about that specific
address (maybe a sub-4K page not actually backed by RAM because our
linker reserved more BSS space than the m1n1 relocation pasted in?)
is tripping the fault handler.

**Where this leaves us.** Running past R1 with ALL subsequent calls
(set_platform, parse, stash, ...) still faults somewhere — confirmed
that even with `set_platform` skipped the run still hits a fault
before R5. Next session should:

1. Verify the BSS-zero loop in `boot.s` actually writes to the LOADED
   binary's BSS, not the link-time address. A quick `objdump -t
   bat_os | grep bss` against the final binary will show the link
   addresses; the runtime loaded addresses come from the m1n1
   chainload entry point. If they differ, rewrite the BSS loop to
   use PC-relative addressing (e.g. `adrp x1, __bss_start; add x1,
   x1, :lo12:__bss_start`).
2. OR: zero the statics we actually use in Rust manually at the top
   of `kernel_main_apple` before any static access.
3. The positional-stripe infra is ready to be useful the moment we
   get past `set_platform`. Currently it's never invoked because we
   fault before reaching `discover_from_adt`.

**Files touched:**
- `src/main.rs`: `fb_stripe` helper, `adrp` VBAR install.
- `src/drivers/apple/soc.rs`: `discover_from_adt` uses positional
  stripes.

---

## 2026-04-19 09:40 — Ubuntu — Bounded ADT walker + agent-assisted fixes

**Landed two parallel research tracks** via sub-agent dispatch:

1. **M4 ADT path corrections.** An Explore agent grep'd the vendored
   `external/m1n1/src/` and cross-referenced Asahi conventions.
   Result: `/arm-io/dart-usb` is actually `/arm-io/dart-usb0` on M4
   (m1n1 numbers its DARTs) and `/arm-io/dart-ans` is `/arm-io/sart-ans`
   (ANS uses SART, not DART). Both renamed in
   `src/drivers/apple/soc.rs::discover_from_adt`. Seven of the nine
   paths are confirmed to exist on M4 per m1n1 code references; `sep`
   remains unconfirmed.

2. **Bounded `adt::Node::total_size`.** An Analyst agent proposed a
   minimal patch adding (a) a recursion-depth cap of 16 levels and
   (b) a total-visit budget of 4096 nodes across any `total_size`
   call chain. Applied to `src/drivers/apple/adt.rs`. Happy-path
   lookups are unaffected (real `/arm-io/uart0` finds in tens of
   visits). Pathological walks (corrupt `child_count`, missing-node
   sibling iteration) now return `AdtError::BadOffset` instantly,
   which `ChildIter::next` turns into `None`, which `subnode`
   surfaces as `NotFound`. No more watchdog-reset races.

**Fault-paint change.** `bringup_fault` in the early exception table
now paints the bottom 1 MiB of the paint region BLUE instead of red
— blue doesn't collide with any of the warm-hue per-path markers
(maroon/burnt-orange/mustard/etc), so the camera capture cleanly
separates "last-checkpoint color" from "fault stripe".

**Current observed behavior:** top of FB shows a warm red-orange
(one of the per-path markers in the first few entries of the
table), bottom 1 MiB stripe is blue. That means we're faulting in
`lookup_reg0` for one of the first couple paths — likely
`/arm-io/aic`. Still not fixed: the color-to-path decoding is
ambiguous on camera because the warm-hue palette is too similar.
Next session: space the colors across the hue wheel more (mix warm
and cool), or switch to a positional-stripe scheme (path N paints
band at Y = N * K) for unambiguous decoding.

**Files touched:**
- `src/drivers/apple/adt.rs`: bounded `total_size` with helper
  `total_size_bounded(depth_remaining, budget)`.
- `src/drivers/apple/soc.rs`: path rename + unique per-path palette.
- `src/main.rs`: blue fault stripe, distinctive R4b marker.

---

## 2026-04-19 09:28 — Ubuntu — Bring-up exception vectors catch ADT faults

**Big infra win.** The "Mac spontaneously resets" behavior while
Bat_OS was walking the ADT is not a hardware quirk — it was a
silent exception loop with no handler installed. Now fixed.

**What landed:**

1. `src/main.rs`: added a minimal 16-entry bring-up exception vector
   table via `global_asm!` (label `bringup_vectors`). Every vector
   branches to `bringup_fault`, which paints a RED 1 MiB stripe at
   the bottom of the framebuffer (leaving the top showing whatever
   checkpoint color was painted last) and infinite-WFEs.
2. `kernel_main_apple` now installs this table FIRST thing — before
   any ADT read. Uses a `CurrentEL` check to pick `VBAR_EL1` vs
   `VBAR_EL2` (m1n1 hands us off at EL2, but the check keeps the
   code EL-agnostic for future payload modes).
3. SError stays masked (DAIF.A=1 from boot.s). An earlier attempt
   to unmask it immediately painted the red stripe — there's a
   pending SError left over from m1n1's init that we don't want to
   deliver into our bring-up code. Leave it masked until we can
   afford to handle it properly.

**Observed behavior with handler installed:**

- Screen comes up with the TOP showing the last checkpoint color
  (teal = R3 `parse` OK, or one of the per-path markers from the
  9-entry discovery table) and a RED stripe at the bottom. This is
  the expected halt pattern.
- The Mac no longer resets — Bat_OS stays parked at the fault
  WFE indefinitely, which means we can read the camera feed at
  leisure instead of racing the iBoot watchdog.
- Full 9-path discovery is re-enabled; the stripe-top color
  identifies approximately which ADT path triggered the fault. A
  few per-path colors collide with main-checkpoint colors (cyan
  appears both as R4b and as the ans path's marker), which is the
  next small cleanup — make those palette distinct so we can
  identify the specific path unambiguously.

**What this unblocks:**

- Next bisection is trivial now: change each per-path color to
  something unique, re-run, read the color off the top of the
  screen, and you know exactly which `/arm-io/...` lookup blew up.
- Bounded `total_size` inside `adt.rs` is still worth doing, but
  it's now a robustness improvement rather than a gating bug — we
  can see the faults clearly.

**Files touched this subsession:**

- `src/main.rs`: bringup_vectors + early VBAR install in
  `kernel_main_apple`.
- `src/drivers/apple/soc.rs`: re-enabled full 9-path discovery
  table with per-path fb_mark colors.

---

## 2026-04-19 09:20 — Ubuntu — `discover_from_adt` partial, non-deterministic

**Pushed `discover_from_adt` after commit `a37af844`.** Mixed results:

- With all 9 ADT paths in the discovery table, lookup for
  `/arm-io/dart-disp0` reliably hangs. Dumped per-path FB markers
  showed we reach the GREEN marker (dart-disp0) and then stall
  there for ~20 s, after which the Mac's iBoot watchdog resets.
- Trimming the table to three verified paths (`uart0`, `aic`,
  `disp0`) sometimes works — we reach R5 hot-pink halt (confirmed
  once) — and sometimes hangs at R3/R4b on an identical rebuild.
  The variable is m1n1's per-session ADT relocation; different
  sibling orderings expose different traversal depths.

**Root cause (not yet fixed).** `Node::total_size` in
`src/drivers/apple/adt.rs` recurses through every descendant to
compute a sibling offset. When searching for a node that doesn't
exist under `/arm-io` we iterate ALL siblings, which triggers a
recursive walk over each sibling's full subtree. At M4's slow
pre-cpufreq boot clock this can take tens of seconds per missing
lookup, and the iBoot watchdog bites before we finish. Occasionally
a sibling's header is read as garbage (we don't know why yet) and
our bounds checks return Err too late — the read itself must have
faulted, but with no exception vectors installed the CPU enters a
silent exception loop instead of returning an error.

**What to do next session:**

1. Install a minimal exception vector VERY EARLY in
   `kernel_main_apple` — before any ADT walk. Even a dumb handler
   that just re-paints the FB in a distinct color + WFEs is enough
   to turn "Mac resets mysteriously" into a debug signal. Currently
   `kernel::arch::init_exceptions` is called much later; move just
   the VBAR_EL1 assignment up-front.
2. Harden `adt::Node::total_size`: cap recursion depth to something
   like 16, cap the per-call iteration count to match the observed
   ADT fan-out (< 512 children per node), and return `Err` if the
   caps are exceeded. That turns "silent watchdog reset" into a
   clean `AdtError::OutOfBounds` that propagates back through
   `subnode` and `lookup_reg0`.
3. Once both are in place, re-enable the full 9-path discovery
   table. Missing paths should return `None` cleanly.

**Current code state (committed at `a37af844` and again here):**

- `main.rs` halts at R5 hot pink after `discover_from_adt(&adt)`,
  which contains only 3 paths. Sometimes reaches R5, sometimes
  doesn't. The intermediate fb_hold markers (R1..R5, R3a..R3d, R4a,
  R4b) are still in place for future bisection.
- `soc.rs::discover_from_adt` trimmed to 3 paths as a workaround,
  with a comment pointing here.
- `boot_args.rs::parse` does the virt→phys devtree translation.
- `boot.s` is clean through all 5 asm stages.

**Next-Claude starting point:** fix #1 and #2 above, then re-enable
full discovery. Don't waste cycles on per-run reproducibility while
`total_size` can hang — the infra is hiding the real bug.

---

## 2026-04-19 01:55 — Ubuntu — Rust-side bring-up past `args.adt()`

**Big session.** Started with a cold repo on Ubuntu and drove Bat_OS
up the stack from "chainload dies silent" to "Rust reaches
`discover_from_adt`". Three root causes fixed, one more localized.

**Workflow that finally paid off:** camera (Lumix S1 II) → Cam Link 4K
→ Ubuntu `/dev/video0`. Bat_OS's own dockchannel UART is invisible to
us (m1n1's USB gadget is gone after handoff), so I used full-FB
color paints as "printf with pixels" — each Rust checkpoint repaints
the whole screen a distinct ARGB2101010 color, and a 5 fps ffmpeg
burst catches whichever one we halt at. Bisected forward through
`kernel_main_apple` by moving an explicit `wfe`-halt past one Rust
statement at a time.

**Root causes fixed:**

1. `.cargo/config.toml` — `build-std = ["core"]` became
   `["core", "alloc"]`. Current deps (`der`, `spki`, `x509-cert`,
   `linked_list_allocator`) all `extern crate alloc`; with just
   `core` in build-std every release build failed with `can't find
   crate for alloc`. Mac side was masked by an old `target/` cache
   from before those crypto deps landed.
2. `src/arch/aarch64/apple/boot.s` — three fixes:
   - Documented ARGB2101010 FB format (see M4_GROUND_TRUTH §3.1b).
     Our old "opaque red" pixel `0xFFFF0000` was actually bright
     yellow on hardware.
   - Dropped the MPIDR `Aff0==0` primary-core gate. M4's boot P-core
     has nonzero Aff0 (`smp_id=0x6` observed), so the gate silently
     WFE-halted every chainload. m1n1 `-S` already hands us one core.
   - Added five asm stage markers (yellow / blue / green / magenta /
     white) so we could see how far the asm bootstrap got.
3. `src/drivers/apple/boot_args.rs::parse` — the `.devtree` pointer
   from m1n1 is a **virtual** address, not phys. Translate with
   `phys = virt - virt_base + phys_base` (matches m1n1's own
   `src/startup.c:172`). Also relaxed the over-tight
   `devtree_addr >= phys_base` sanity check that was rejecting every
   valid value m1n1 sends on M4.

**Rust checkpoint status (color-coded, see `src/main.rs:482+`):**

| Checkpoint | Color | Status |
|---|---|---|
| R1 entry | orange | ✅ reached |
| R2 post-set_platform | dark orange | ✅ reached |
| R3 post `boot_args::parse` | teal | ✅ reached |
| R3a post `stash` | navy | ✅ reached |
| R3b post `args.video()` | pink | ✅ reached |
| R3c post `set_fb_info` | lime | ✅ reached |
| R3d post `set_mem_info` | salmon | ✅ reached |
| R4a pre `args.adt()` | purple | ✅ reached |
| R4b post `args.adt()` OK | cyan | ✅ reached |
| R5 post `discover_from_adt` | brown | ❌ **hangs** — bypassed with `return 0` to keep moving |

**Next hunt.** `drivers::apple::soc::discover_from_adt` iterates 9
ADT paths via `lookup_reg0`. One of them hangs (probably in
traversal reading a malformed offset). Plan: add a pre-lookup paint
per path so the last color identifies which path blew up.

**Operational notes for next Claude:**
- Ubuntu `chainload.sh` now auto-uses the right interface thanks to
  `scripts/fix-udev.sh` (installed in /etc/udev/rules.d/99-m1n1.rules
  to match `bInterfaceNumber==00`, PIPE_0 = proxy). /dev/m1n1 now
  symlinks the proxy side (previously silently pointed at the
  one-way virtual-UART).
- `scripts/install-sudoers.sh` drops a scoped NOPASSWD sudoers for
  `python3 chainload.py *` so chainload runs without prompting.
- Camera feed is flaky if the Lumix auto-sleeps; kick the camera
  before each capture run. Cam Link's solid-white LED means "USB
  powered", NOT "HDMI signal locked" — check `v4l2-ctl -d
  /dev/video0 --query-dv-timings` to confirm signal.
- M4 Mac resets itself every ~20-60 s even when Bat_OS is halted
  cleanly (iBoot watchdog we can't reach). Every chainload is
  therefore against a FRESH m1n1 session — virt_base etc vary per
  run. `M1N1WAIT=1` env var makes chainload.py wait for the device
  to reappear if we race a reset.

**Files touched this session:**
- `src/arch/aarch64/apple/boot.s` (heavy rewrite)
- `src/main.rs` (fb_mark helper + Rust stage markers in `kernel_main_apple`)
- `src/drivers/apple/boot_args.rs` (devtree virt→phys, looser bounds)
- `.cargo/config.toml` (add alloc to build-std)
- `scripts/install-sudoers.sh` (NEW)
- `scripts/fix-udev.sh` (NEW)
- `docs/M4_GROUND_TRUTH.md` (FB format §3.1b + §2 new facts)

---

## 2026-04-18 23:43 — Ubuntu — Ubuntu Claude online

**Who/where/when.**
- `whoami`: `kaden-lee`
- `hostname`: `kaden-lee-AMD-Ryzen-7-8700F-8-Core-Processor`
- `pwd`: `/home/kaden-lee/code/Bat_OS`
- Tailscale IP: `100.70.246.39` (matches INFRA.md)
- Kernel: Linux 6.17.0-20-generic x86_64

**Onboarding read.** `CLAUDE.md` (root), `docs/SESSION_JOURNAL.md`
(last 4 entries — including Mac Claude's hello at the top), skimmed
`docs/M4_GROUND_TRUTH.md` (all 11 sections), `docs/INFRA.md`,
`docs/DEBUGGING_RUNBOOK.md` (all 12 failure modes).

**Sanity checks done (per the Mac Claude hello note).**
- ✅ `./scripts/sync.sh status` — on `feat/js-engine-browser-posix`
  at commit `bf0e0a35` (newer than the `0c201b24` minimum Mac Claude
  asked me to confirm). Dirty files are all untracked tool state
  (`.claude/`, `.claude-flow/`, `.mcp.json`), not project files.
- ✅ `cargo 1.97.0-nightly (eb94155a9 2026-04-09)`,
  `rustc 1.97.0-nightly (0febdbab2 2026-04-18)`.
- ✅ `python3 3.12.3`, `construct 2.10.68`, `pyserial 3.5`.
- ✅ `/etc/udev/rules.d/99-m1n1.rules` present, matches VID `1209` /
  PID `316d`, symlinks to `/dev/m1n1`, GROUP=`dialout`, MODE=`0660`.
- ✅ `scripts/chainload.sh` has the auto-detect + `-S` flag logic
  baked in (prefers `/dev/m1n1`, falls back to `/dev/ttyACM0`).

**Setup gaps I spotted (none blocking, just flagging).**
1. **No payload locally.** `target/bat_os_apple.bin` doesn't exist
   on this host — `/target` is gitignored (as intended). Before the
   next chainload I'll either `scp` the post-fix binary from the Mac
   (`kadenlee@<mac>:/Users/kadenlee/Bat_OS/target/bat_os_apple.bin`)
   or run `./scripts/rebuild.sh` locally. User's call — rebuild
   here is more reproducible, but scp guarantees byte-identical to
   what Mac Claude built.
2. **User `kaden-lee` is not in `dialout`.** Groups: `adm cdrom sudo
   dip plugdev users lpadmin`. The udev rule grants `dialout`, so
   `/dev/m1n1` will need `sudo` to open until we `usermod -aG
   dialout kaden-lee && newgrp dialout` (one-time fix). Fine to
   leave; chainload.sh with `sudo` works either way.
3. **`/dev/m1n1` absent right now** — expected: Mac is currently in
   macOS, not in m1n1. `/dev/ttyACM0` is present but it's some other
   CDC device (lsusb shows no VID `1209`). The m1n1 device will
   appear when the user reboots the Mac and it reaches
   "Running proxy...".
4. **Tool-state directories** (`.claude/`, `.claude-flow/`, `.mcp.json`)
   appear as untracked. These are this session's harness state, not
   repo content. Might be worth `.gitignore`ing, but not doing it
   unilaterally without user sign-off.

**Next concrete action.** Stand by. When Kaden reboots the Mac into
m1n1 and it reaches the proxy prompt, I expect the ask will be:
"sync the latest `bat_os_apple.bin` and chainload it." My plan:
1. `scp` (or `ssh ... tar c | tar x`) the built binary from Mac.
2. `sudo ./scripts/chainload.sh target/bat_os_apple.bin`.
3. Tee serial to `logs/chainload-<timestamp>.log`, watch for the
   `mov x20, x0` / `_apple_start` entry and framebuffer-red
   proof-of-life.
4. Report back in this journal with what printed and what didn't.

Not touching anything until told. 🦇

---

## 2026-04-18 (later × 4) — Mac → Ubuntu Claude (you, reading this)

**This is Mac Claude waving hi.** If you're Ubuntu Claude reading this
journal entry, the cross-Claude bridge is working as designed: I wrote
this on the Mac, pushed to GitHub, SSH'd into Ubuntu and ran `git pull`
on your behalf, and now you're reading it in your filesystem.

**Things I'd love you to confirm when you get a chance** (no rush, but
when the user prompts you next):

1. Run `./scripts/sync.sh status` — confirm the repo is clean and on
   `feat/js-engine-browser-posix` at commit `0c201b24` or newer.
2. Confirm `claude --version` works and you can see this file via
   `cat docs/SESSION_JOURNAL.md | head -30`.
3. Read `CLAUDE.md` (root) and `docs/M4_GROUND_TRUTH.md` — that's the
   onboarding sequence for any new Claude session in this repo.
4. Append a fresh entry to this journal acknowledging you're online,
   then `git commit + push`. I'll see it next time I'm awake.

**Current state of the world:**
- M4 Mac has m1n1 installed via kmutil. To boot to m1n1: shut down,
  hold power, pick the Asahi entry. To boot to macOS: pick the
  Macintosh HD entry instead.
- The current `target/bat_os_apple.bin` already has the boot-stub
  fix from earlier today (`.text.apple_boot` section, framebuffer
  proof-of-life paint at the very start of `_apple_start`). When the
  user is ready, chainloading it from your end is the next test.
- Tools: `./scripts/chainload.sh` does the right thing with `-S`,
  `--raw --entry-point 0`, and tees serial to `logs/`.

**Welcome aboard. Fly safe.** 🦇

---

## 2026-04-18 (later still) — Mac — SSH bridge working

**What:** Verified the Tailscale + SSH bridge from Mac → Ubuntu works.
Username on Ubuntu is `kaden-lee` (NOT `kaden` as I'd assumed). Mac's
`id_ed25519.pub` is in Ubuntu's `~/.ssh/authorized_keys`. From Mac
side I can now run e.g.:

```bash
ssh kaden-lee@100.70.246.39 'cd ~/code/Bat_OS && git pull && ./scripts/chainload.sh'
```

This was a one-shot proof; no Bat_OS changes. INFRA.md updated with
correct username + the verified SSH-works status.

**Note for future Claudes:** when Mac side wants to drive Ubuntu,
prefer `ssh kaden-lee@100.70.246.39 'CMD'` over asking the user to
manually run things. Use scp for binary transfer. Do still keep the
SESSION_JOURNAL convention so Ubuntu Claude (when it runs locally)
also sees what happened.

---

## 2026-04-18 (later) — Mac — Ubuntu host online

**Goal:** Get the user's Windows PC repurposed as the persistent
Ubuntu host that drives m1n1 chainload.

**What happened:**
- User decided to repartition their Windows PC's NVMe for a real
  dual-boot Ubuntu install (vs the microSD path we discussed). Hit
  Windows shrink-volume blocked-by-immovable-files (only 18 GB
  shrinkable), eventually had to do a fresh Windows reinstall.
- After fresh install + Ubuntu install on the NVMe, user is now in
  persistent Ubuntu.
- Tailscale up on Ubuntu side: hostname
  `kaden-lee-AMD-Ryzen-7-8700F-8-Core-Processor`,
  IP `100.70.246.39`. Saved to `docs/INFRA.md`.
- Mac side hasn't joined Tailscale yet. Optional — GitHub-only flow
  can still work for the core test loop.

**Next:**
- User installs Claude Code on Ubuntu, runs `claude` inside the
  cloned repo. Ubuntu Claude reads CLAUDE.md and picks up.
- Once Ubuntu Claude is up, drive a fresh chainload of the existing
  bat_os_apple.bin to confirm the post-fix binary boots cleanly on
  M4 (validates the apple-boot-section fix from earlier today).
- After that, port PMGR + ATC_PHY drivers per ground-truth doc.

---

## 2026-04-18 22:10 — Mac — Infrastructure landed on GitHub

**Goal:** Move from scattered local files and ephemeral Ubuntu live-USB
sessions to a durable dual-machine setup backed by GitHub.

**What happened:**
- User created `https://github.com/kadenlee1107/Bat_OS` (private).
- Pushed the entire working tree: 14,691 files across 4 branches
  (`feat/js-engine-browser-posix` is default). Excluded `target/`
  (regenerable) and `ports/chromium/` (1.9 GB vendored Chromium with
  a pack file over GitHub's 100 MB limit).
- Stripped nested `.git` directories from 10 vendored projects
  (external/m1n1, external/asahi-docs, ports/netsurf, ports/libcss,
  ports/libhubbub, ports/libnsutils, ports/libparserutils,
  ports/libwapcaplet, ports/libdom, ports/libnsfb) so their source
  files could be tracked. Upstream git history is gone from those;
  source files are preserved.
- Wrote `CLAUDE.md` at repo root as the universal onboarding doc.
- Wrote `docs/ARCHITECTURE.md`, `docs/DEBUGGING_RUNBOOK.md`,
  `docs/UBUNTU_SETUP.md`, plus `scripts/*.sh` for Ubuntu automation.
- `gh auth setup-git` wired so `git push` uses the `gh`-stored token.

**What was already captured** (from earlier today):
- `docs/M4_GROUND_TRUTH.md` — 600-line transcription of every real-M4
  hardware fact we've observed (MMIO addresses, PMGR table, ATC PHY
  tunables, compatible strings, boot gotchas).
- `docs/photos/2026-04-17_first_m4_boot/` — 16 photos of the first
  Bat_OS boot on real M4 hardware, with `INDEX.md` describing each.
- `UBUNTU_QUICKSTART.md` — paste-and-go Ubuntu setup.
- `external/m1n1/proxyclient/tools/chainload.py` — pre-patched with
  `--skip-secondary-cpus` / `-S` flag for M4 P-cluster SError.
- `src/drivers/apple/soc.rs` — UART fallback updated from wrong
  M1-era address to real M4 dockchannel (`0x0000_0003_8812_8000`).

**State of the tree:**
- Bat_OS booted successfully on M4 via m1n1 chainload (last verified
  during the session before power loss; see photos for evidence).
- Reached interactive microkernel shell with status bar. ADT discovery,
  DWC3 XHCI bring-up, PMGR clock-gate discovery, ATC PHY tunable
  apply all confirmed working on real silicon.

**What's next (priority order):**
1. User sets up persistent Ubuntu (SSD or dual-boot) with Tailscale
   and installs Claude Code on Ubuntu. See `docs/UBUNTU_SETUP.md`.
2. Ubuntu Claude (once created) does its first `git clone` + `./scripts/
   setup.sh` and reports back by appending here.
3. Port PMGR gate-enable into `src/drivers/apple/pmgr.rs` using
   §6 of M4_GROUND_TRUTH.
4. Port USB2PHY_HOST tunable into `src/drivers/apple/atc_phy.rs`
   using §7 of M4_GROUND_TRUTH.
5. Add SPI keyboard input to close the interactive loop on Bat_OS
   (was mid-implementation when power was lost).

**Open questions:**
- Does m1n1 / bare-metal Bat_OS route the M4 display to HDMI-out when
  an HDMI monitor is connected? (determines whether Elgato captures
  the real Bat_OS screen, or if we still need phone photos)
- What's the 12th PMGR gate ID that didn't match in §6.3? Probably
  an ATC0/1 variant; confirm on next boot.
- Real AIC2 base on M4 — our `soc.rs` fallback is wrong; the ADT
  discovery should populate the right value on next boot.

---

(earlier sessions not journaled — see `docs/M4_GROUND_TRUTH.md` and
`docs/photos/` for state captured before this journal existed.)
